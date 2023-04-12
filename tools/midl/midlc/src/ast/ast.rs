use super::{CompoundIdentifier, Span};

/// Represents arbitrary values.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal {
    /// Any numeric value e.g. floats or ints.
    NumericValue(String, Span),

    /// Any string value.
    StringValue(String, Span),
}

impl Literal {
    pub fn as_numeric_value(&self) -> Option<(&str, Span)> {
        match self {
            Literal::NumericValue(s, span) => Some((s, *span)),
            _ => None,
        }
    }

    pub fn as_string_value(&self) -> Option<(&str, Span)> {
        match self {
            Literal::StringValue(s, span) => Some((s, *span)),
            _ => None,
        }
    }

    pub fn span(&self) -> Span {
        match &self {
            Self::NumericValue(_, span) => *span,
            Self::StringValue(_, span) => *span,
        }
    }
}

#[derive(Debug)]
pub struct LibraryDeclaration {
    pub name: CompoundIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImportDeclaration {
    pub name: CompoundIdentifier,
}
