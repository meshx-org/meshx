use super::{CompoundIdentifier, Span, WithSpan};

/// Represents arbitrary values.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal {
    /// Any numeric value e.g. floats or ints.
    NumericValue(String, Span),

    /// Any string value.
    StringValue(String, Span),
}

impl Literal {
    pub fn as_numeric_value(&self) -> Option<(f64, Span)> {
        match self {
            Literal::NumericValue(val, span) => Some((val.parse().unwrap(), span.clone())),
            _ => None,
        }
    }

    pub fn as_string_value(&self) -> Option<(&str, Span)> {
        match self {
            Literal::StringValue(val, span) => Some((val, span.clone())),
            _ => None,
        }
    }
}

impl WithSpan for Literal {
    fn span(&self) -> Span {
        match &self {
            Self::NumericValue(_, span) => span.clone(),
            Self::StringValue(_, span) => span.clone(),
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
