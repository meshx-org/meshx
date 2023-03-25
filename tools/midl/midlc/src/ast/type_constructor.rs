use hcl::Identifier;

use super::{CompoundIdentifier, Reference};
use std::str::FromStr;

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
    type Err = &'static str;

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
            _ => Err("invalid primitive type"),
        }
    }
}

#[derive(Debug, Clone)]
enum TypeKind {
    Array,
    Box,
    Vector,
    String,
    Handle,
    TransportSide,
    Primitive,
    Internal,
    UntypedNumeric,
    Identifier,
}

#[derive(Debug, Clone)]
struct TypeInternal {
    name: Identifier,
    kind: TypeKind,
}

#[derive(Debug, Clone)]
struct LayoutParameterList {}

#[derive(Debug, Clone)]
struct LayoutConstraints {}

#[derive(Debug, Clone)]
struct TypeConstructor {
    // Set during construction.
    layout: Reference,
    parameters: Box<LayoutParameterList>,
    constraints: Box<LayoutConstraints>,

    // Set during compilation.
    r#type: Option<Type>,
    // TODO: resolved_params: Option<LayoutInvocation>,
}

#[derive(Debug, Clone)]
pub enum Type {
    ArrayType {
        element_type: Box<Type>,
    },
    VectorType {
        element_type: Box<Type>,
    },
    StringType {
        nullable: bool,
    },
    RequestType {
        nullable: bool,
        subtype: String, // "test.handles/DriverProtocol",
    },
    PrimitiveType {
        nullable: bool,
        subtype: PrimitiveSubtype,
    },
    IdentifierType {
        reference: Reference,
    },
    HandleType {
        nullable: bool,
        resource_identifier: String,
        subtype: String,
        rights: u32,
    },
}