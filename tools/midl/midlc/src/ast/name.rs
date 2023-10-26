use super::Span;
use crate::ast::Library;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SourcedNameContext {
    span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct IntrinsicNameContext {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NameContext {
    Sourced(SourcedNameContext),
    Intrinsic(IntrinsicNameContext),
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
        }
    }
}
