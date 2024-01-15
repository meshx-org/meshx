use hcl::Identifier;

use super::{ast, Const, ConstantValue, Element, LiteralConstant, Reference, Span};
use std::{cell::RefCell, str::FromStr};

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
            "f32" => Ok(PrimitiveSubtype::Float32),
            "f64" => Ok(PrimitiveSubtype::Float64),
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdentifierLayoutParameter {
    pub reference: Reference,

    as_type_ctor: RefCell<Option<TypeConstructor>>,
    as_constant: RefCell<Option<Const>>,
}

impl IdentifierLayoutParameter {
    // Disambiguates between type constructor and constant. Must call after
    // resolving the reference, but before calling AsTypeCtor or AsConstant.
    pub fn disambiguate(&self) {
        let resolved = self.reference.resolved();

        match resolved.unwrap().element {
            Element::Const(_) => {}
            // Element::BitsMember => {}
            // Element::EnumMember(_) => {
            //    self.as_constant =  IdentifierConstant (source_signature(), reference, span);
            //
            // }
            _ => {
                self.as_type_ctor.replace(Some(TypeConstructor {
                    layout: self.reference.clone(),
                    parameters: LayoutParameterList {
                        items: vec![],
                        span: None,
                    },
                    constraints: LayoutConstraints {},
                    r#type: None,
                }));
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeLayoutParameter {
    pub type_ctor: TypeConstructor,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiteralLayoutParameter {
    pub literal: LiteralConstant,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayoutParameter {
    Identifier(IdentifierLayoutParameter),
    Type(TypeLayoutParameter),
    Literal(LiteralLayoutParameter),
}

impl LayoutParameter {
    // A layout parameter is either a type constructor or a constant. One of these
    // two methods must return non-null, and the other one must return null.
    pub(crate) fn as_type_ctor(&self) -> Option<TypeConstructor> {
        None
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayoutParameterList {
    pub items: Vec<LayoutParameter>,

    /// The location of this parameter list in the text representation.
    pub span: Option<Span>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayoutConstraints;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeConstructor {
    // Set during construction.
    pub layout: Reference,
    pub parameters: LayoutParameterList,
    pub constraints: LayoutConstraints,

    // Set during compilation.
    pub(crate) r#type: Option<Type>,
    // TODO: resolved_params: Option<LayoutInvocation>,
}

impl TypeConstructor {
    pub(crate) fn new(layout: Reference, parameters: LayoutParameterList, constraints: LayoutConstraints) -> Self {
        Self {
            layout,
            parameters,
            constraints,
            r#type: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
