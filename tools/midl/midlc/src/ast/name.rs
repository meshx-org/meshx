use super::Span;
use crate::ast::Library;
use std::{cell::OnceCell, rc::Rc};

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
