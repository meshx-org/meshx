use std::str::FromStr;

use super::{CompoundIdentifier, Identifier, Span};
use crate::error::ParserError;

/// Represents arbitrary values.
#[derive(Debug, Clone)]
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

/// Represents a constant value.
#[derive(Debug)]
pub struct Constant(pub Literal);

#[derive(Debug)]
pub struct ConstDeclaration {
    pub name: Identifier,
    pub ty: Type,
    pub value: Constant,
}

#[derive(Debug)]
pub struct LibraryDeclaration {
    pub name: CompoundIdentifier,
}

#[derive(Debug)]
pub struct ImportDeclaration {
    pub name: CompoundIdentifier,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum PrimitiveSubtype {
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
}

impl FromStr for PrimitiveSubtype {
    type Err = ParserError;

    fn from_str(input: &str) -> Result<PrimitiveSubtype, Self::Err> {
        match input {
            "bool" => Ok(PrimitiveSubtype::Bool),
            "u8" => Ok(PrimitiveSubtype::Uint8),
            "u16" => Ok(PrimitiveSubtype::Uint16),
            "u32" => Ok(PrimitiveSubtype::Uint32),
            "u64" => Ok(PrimitiveSubtype::Uint64),
            "i8" => Ok(PrimitiveSubtype::Int8),
            "i16" => Ok(PrimitiveSubtype::Int16),
            "i32" => Ok(PrimitiveSubtype::Int32),
            "i64" => Ok(PrimitiveSubtype::Int64),
            _ => Err(ParserError::UnknownPrimitiveType),
        }
    }
}

#[derive(Debug)]
pub enum Type {
    Array {
        element_type: Box<Type>,
    },
    Vector {
        element_type: Box<Type>,
    },
    String {
        nullable: bool,
    },
    Request {
        nullable: bool,
        subtype: String, // "test.handles/DriverProtocol",
    },
    Primitive {
        nullable: bool,
        subtype: PrimitiveSubtype,
    },
    Identifier {
        identifier: String, // "test.handlesintypes/obj_type"
        nullable: bool,
    },
    Handle {
        nullable: bool,
        resource_identifier: String,
        subtype: String,
        rights: u32,
    },
}


