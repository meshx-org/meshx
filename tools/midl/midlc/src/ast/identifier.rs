use std::ops::Deref;

use super::{Span, WithSpan};

#[derive(Debug, Clone, Eq)]
pub struct Identifier {
    pub value: String,
    pub span: Span,
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Deref for Identifier {
    type Target = Identifier;

    fn deref(&self) -> &Self::Target {
        &self
    }
}

impl WithSpan for Identifier {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundIdentifier {
    pub components: Vec<Identifier>,
    pub span: Span,
}
