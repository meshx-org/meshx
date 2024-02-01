use super::Span;
use crate::ast;
use convert_case::{Case, Casing};
use std::{
    borrow::BorrowMut,
    cell::{OnceCell, RefCell},
    rc::Rc,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NamingContextKind {
    Decl,
    LayoutMember,
    MethodRequest,
    MethodResponse,
}

/// A NamingContext is a list of names, from least specific to most specific, which
/// identifies the use of a layout. For example, for the FIDL:
///
/// ```rust
/// library meshx.bluetooth.le;
///
/// protocol Peripheral {
///     StartAdvertising(table { 1: data struct {}; });
/// };
/// ```
///
/// The context for the innermost empty struct can be built up by the calls:
///
///   auto ctx = NamingContext("Peripheral").FromRequest("StartAdvertising").EnterMember("data")
///
/// `ctx` will produce a `FlattenedName` of "data", and a `Context` of
/// ["Peripheral", "StartAdvertising", "data"].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NamingContext {
    pub(crate) name: ast::Span,
    pub(crate) parent: Option<Rc<NamingContext>>,
    kind: NamingContextKind,
    flattened_name: String,
    flattened_name_override: RefCell<Option<String>>,
}

fn to_upper_camel_case(str: String) -> String {
    str.to_case(Case::Pascal)
}

fn build_flattened_name(name: ast::Span, kind: NamingContextKind, parent: &Option<Rc<NamingContext>>) -> String {
    match kind {
        NamingContextKind::Decl => name.data,
        NamingContextKind::LayoutMember => to_upper_camel_case(name.data),
        NamingContextKind::MethodRequest => {
            let mut result = to_upper_camel_case(parent.as_ref().unwrap().name.data.clone());
            result.push_str(to_upper_camel_case(name.data).as_str());
            result.push_str("Request");

            result
        }
        NamingContextKind::MethodResponse => {
            let mut result = to_upper_camel_case(parent.as_ref().unwrap().name.data.clone());
            result.push_str(to_upper_camel_case(name.data).as_str());
            result.push_str("Response");
            result
        }
    }
}

impl NamingContext {
    /// Usage should only be through shared pointers, so that shared_from_this is always
    /// valid. We use shared pointers to manage the lifetime of NamingContexts since the
    /// parent pointers need to always be valid. Managing ownership with unique_ptr is tricky
    /// (Push() would need to have access to a unique_ptr of this, and there would need to
    /// be a place to own all the root nodes, which are not owned by an anonymous name), and
    /// doing it manually is even worse.
    pub(crate) fn create(name: &Name) -> Rc<Self> {
        NamingContext::new(name.span().unwrap(), NamingContextKind::Decl, None)
    }

    fn new(name: Span, kind: NamingContextKind, parent: Option<Rc<NamingContext>>) -> Rc<Self> {
        Rc::new(NamingContext {
            name: name.clone(),
            flattened_name_override: RefCell::new(None),
            flattened_name: build_flattened_name(name, kind, &parent),
            kind,
            parent,
        })
    }

    /// to_name() exists to handle the case where the caller does not necessarily know what
    /// kind of name (sourced or anonymous) this NamingContext corresponds to.
    /// For example, this happens for layouts where the consume_* functions all take a
    /// NamingContext and so the given layout may be at the top level of the library
    /// (with a user-specified name) or may be nested/anonymous.
    pub(crate) fn to_name(self: Rc<Self>, library: Rc<ast::Library>, declataion_span: Span) -> Name {
        if self.parent.is_none() {
            return Name::create_sourced(library, self.name.clone());
        }

        Name::create_anonymous(library, declataion_span, self, NameProvenance::AnonymousLayout)
    }

    pub(crate) fn set_name_override(&self, value: String) {
        *self.flattened_name_override.borrow_mut() = Some(value);
    }

    pub(crate) fn context(self: Rc<Self>) -> Vec<String> {
        let mut names = vec![];
        let mut current = Some(self);

        while let Some(local) = current {
            // Internally, we don't store a separate context item to represent whether a
            // layout is the request or response, since this bit of information is
            // embedded in the Kind. When collapsing the stack of contexts into a list
            // of strings, we need to flatten this case out to avoid losing this data.
            match local.kind {
                NamingContextKind::MethodRequest => {
                    names.push("Request".to_owned());
                }
                NamingContextKind::MethodResponse => {
                    names.push("Response".to_owned());
                }
                NamingContextKind::Decl | NamingContextKind::LayoutMember => {}
            }

            names.push(local.name.data.clone());
            current = local.parent.clone();
        }

        names.reverse();
        names
    }

    pub(crate) fn parent(&self) -> Rc<NamingContext> {
        assert!(self.parent.is_some(), "traversing above root");
        self.parent.as_ref().unwrap().clone()
    }

    pub(crate) fn name(&self) -> Span {
        self.name.clone()
    }

    fn push(self: Rc<Self>, name: Span, kind: NamingContextKind) -> Rc<Self> {
        NamingContext::new(name, kind, Some(self.clone()))
    }

    pub(crate) fn enter_member(self: Rc<Self>, member_name: Span) -> Rc<Self> {
        return self.push(member_name, NamingContextKind::LayoutMember);
    }

    pub(crate) fn enter_request(self: Rc<Self>, method_name: Span) -> Rc<Self> {
        assert!(self.kind == NamingContextKind::Decl, "request must follow protocol");
        return self.push(method_name, NamingContextKind::MethodRequest);
    }

    pub(crate) fn enter_event(self: Rc<Self>, method_name: Span) -> Rc<Self> {
        assert!(self.kind == NamingContextKind::Decl, "event must follow protocol");
        // an event is actually a request from the server's perspective, so we use request in the
        // naming context
        self.push(method_name, NamingContextKind::MethodRequest)
    }

    pub(crate) fn enter_response(self: Rc<Self>, method_name: Span) -> Rc<Self> {
        assert!(self.kind == NamingContextKind::Decl, "response must follow protocol");
        self.push(method_name, NamingContextKind::MethodResponse)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NameProvenance {
    /// The name refers to an anonymous layout, like `struct {}`.
    AnonymousLayout,
    /// The name refers to a result union generated by the compiler, e.g. the
    /// response of `strict Foo(A) -> (B) error uint32` or `flexible Foo(A) -> (B)`.
    GeneratedResultUnion,
    /// The name refers to an empty success struct generated by the compiler, e.g. the
    /// success variant for `strict Foo() -> () error uint32` or `flexible Foo() -> ()`.
    GeneratedEmptySuccessStruct,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SourcedNameContext {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct IntrinsicNameContext {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AnonymousNameContext {
    span: Span,
    provenance: NameProvenance,
    pub context: Rc<NamingContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NameContext {
    Sourced(SourcedNameContext),
    Intrinsic(IntrinsicNameContext),
    Anonymous(AnonymousNameContext),
}

/// Name represents a named entry in a particular scope.
/// Names have different flavors based on their origins, which can be determined by the discriminant
/// `Name::Kind`. See the documentation for `Name::Kind` for details.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name {
    pub library: Rc<ast::Library>,
    member_name: Option<String>,
    name_context: NameContext,
}

impl std::fmt::Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Name").field(&self.decl_name()).finish()
    }
}

impl Name {
    pub(crate) fn is_intrinsic(&self) -> bool {
        match self.name_context {
            NameContext::Intrinsic(_) => true,
            _ => false,
        }
    }

    pub(crate) fn create_sourced(library: Rc<ast::Library>, span: Span) -> Self {
        Self {
            library,
            name_context: NameContext::Sourced(SourcedNameContext { span }),
            member_name: None,
        }
    }

    pub(crate) fn create_intrinsic(library: Rc<ast::Library>, name: &str) -> Self {
        Self {
            library,
            name_context: NameContext::Intrinsic(IntrinsicNameContext { name: name.to_owned() }),
            member_name: None,
        }
    }

    pub(crate) fn create_anonymous(
        library: Rc<ast::Library>,
        span: Span,
        context: Rc<NamingContext>,
        provenance: NameProvenance,
    ) -> Self {
        Self {
            library,
            name_context: NameContext::Anonymous(AnonymousNameContext {
                context,
                span,
                provenance,
            }),
            member_name: None,
        }
    }

    pub(crate) fn as_anonymous(&self) -> Option<AnonymousNameContext> {
        match self.name_context {
            NameContext::Sourced(_) => None,
            NameContext::Intrinsic(_) => None,
            NameContext::Anonymous(ref ctx) => Some(ctx.clone()),
        }
    }

    pub fn decl_name(&self) -> String {
        match &self.name_context {
            NameContext::Sourced(ctx) => ctx.span.data.to_owned(),
            NameContext::Intrinsic(ctx) => ctx.name.clone(),
            NameContext::Anonymous(ctx) => ctx.context.flattened_name.clone(),
        }
    }

    pub fn with_member_name(&self, member_name: String) -> Name {
        assert!(!self.member_name.is_some(), "already has a member name");

        let mut new_name = self.clone();
        new_name.member_name = Some(member_name);
        new_name
    }

    pub fn full_name(&self) -> String {
        let mut name = self.decl_name();

        if self.member_name.is_some() {
            let member_name = self.member_name.clone().unwrap();
            let separator = ".";
            name.reserve(name.len() + separator.len() + member_name.len());

            name.push_str(separator);
            name.push_str(member_name.as_str());
        }

        name
    }

    pub fn span(&self) -> Option<Span> {
        match self.name_context {
            NameContext::Sourced(ref name_context) => return Some(name_context.span.clone()),
            NameContext::Intrinsic(_) => None,
            NameContext::Anonymous(ref name_context) => return Some(name_context.span.clone()),
        }
    }
}

fn library_name(name: OnceCell<Vec<String>>, sep: &str) -> String {
    name.get().unwrap().clone().join(sep)
}

pub fn name_flat_name(name: &Name) -> String {
    let mut compiled_name = String::new();

    if !name.is_intrinsic() {
        compiled_name += library_name(name.library.name.clone(), ".").as_str();
        compiled_name += "/";
    }

    compiled_name += name.full_name().as_str();
    compiled_name
}
