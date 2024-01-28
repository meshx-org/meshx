use super::Span;
use crate::ast::Library;
use std::{cell::OnceCell, rc::Rc};

#[derive(PartialEq)]
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
/// library fuchsia.bluetooth.le;
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
pub struct NamingContext {
    name: Span,
    kind: NamingContextKind,
    parent: Option<Rc<NamingContext>>,
    flattened_name: String,
    flattened_name_override: Option<String>,
}

fn build_flattened_name(name: Span) -> String {
    String::new()
}

impl NamingContext {
    /// Usage should only be through shared pointers, so that shared_from_this is always
    /// valid. We use shared pointers to manage the lifetime of NamingContexts since the
    /// parent pointers need to always be valid. Managing ownership with unique_ptr is tricky
    /// (Push() would need to have access to a unique_ptr of this, and there would need to
    /// be a place to own all the root nodes, which are not owned by an anonymous name), and
    /// doing it manually is even worse.
    pub(crate) fn create(name: &Name) -> Rc<Self> {
        Rc::new(Self {
            name: name.span().unwrap(),
            parent: None,
            kind: NamingContextKind::Decl,
            flattened_name: build_flattened_name(name.span().unwrap()),
            flattened_name_override: None,
        })
    }

    fn new(name: Span, kind: NamingContextKind, parent: Rc<NamingContext>) -> Rc<Self> {
        Rc::new(NamingContext {
            name: name.clone(),
            flattened_name_override: None,
            flattened_name: build_flattened_name(name),
            kind,
            parent: Some(parent),
        })
    }

    pub(crate) fn to_name(&self, library: Rc<Library>, span: Span) -> Name {
        Name::create_sourced(library, span.clone())
    }

    fn push(self: Rc<Self>, name: Span, kind: NamingContextKind) -> Rc<Self> {
        NamingContext::new(name, kind, self.clone())
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
struct SourcedNameContext {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct IntrinsicNameContext {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AnonymousNameContext {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NameContext {
    Sourced(SourcedNameContext),
    Intrinsic(IntrinsicNameContext),
    Anonymous(AnonymousNameContext),
}

/// Name represents a named entry in a particular scope.
/// Names have different flavors based on their origins, which can be determined by the discriminant
/// `Name::Kind`. See the documentation for `Name::Kind` for details.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name {
    pub library: Rc<Library>,
    member_name: Option<String>,
    name_context: NameContext,
}

impl std::fmt::Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Name").field(&self.decl_name()).finish()
    }
}

impl Name {
    pub fn is_intrinsic(&self) -> bool {
        match self.name_context {
            NameContext::Intrinsic(_) => true,
            _ => false,
        }
    }

    pub fn create_sourced(library: Rc<Library>, span: Span) -> Self {
        Self {
            library,
            name_context: NameContext::Sourced(SourcedNameContext { span }),
            member_name: None,
        }
    }

    pub fn create_intrinsic(library: Rc<Library>, name: &str) -> Self {
        Self {
            library,
            name_context: NameContext::Intrinsic(IntrinsicNameContext { name: name.to_owned() }),
            member_name: None,
        }
    }

    pub fn decl_name(&self) -> String {
        match &self.name_context {
            NameContext::Sourced(ctx) => ctx.span.data.to_owned(),
            NameContext::Intrinsic(ctx) => ctx.name.clone(),
            NameContext::Anonymous(ctx) => "TODO".to_owned(),
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

pub fn name_flat_name(name: Name) -> String {
    let mut compiled_name = String::new();

    if !name.is_intrinsic() {
        compiled_name += library_name(name.library.name.clone(), ".").as_str();
        compiled_name += "/";
    }

    compiled_name += name.full_name().as_str();
    compiled_name
}
