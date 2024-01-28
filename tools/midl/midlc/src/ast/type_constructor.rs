use derivative::Derivative;

use crate::diagnotics::Diagnostics;

use super::{
    constraints::IdentifierConstraints, Const, Constant, Declaration, Element, Identifier, LiteralConstant, Name,
    Nullability, Reference, Span, VectorConstraints,
};
use std::{borrow::Borrow, cell::RefCell, rc::Rc, str::FromStr};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum InternalSubtype {
    FrameworkErr,
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

        match resolved.unwrap().element() {
            Element::Const { .. } => {}
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
        match self {
            LayoutParameter::Identifier(id) => id.as_type_ctor.borrow().clone(),
            LayoutParameter::Type(typ) => Some(typ.type_ctor.clone()),
            LayoutParameter::Literal(_) => None,
        }
    }

    pub(crate) fn as_constant(&self) -> Option<Constant> {
        match self {
            LayoutParameter::Identifier(_) => todo!(),
            LayoutParameter::Type(_) => todo!(),
            LayoutParameter::Literal(_) => todo!(),
        }
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

    /// Set during compilation.
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

#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct VectorType {
    pub name: Name,
    pub element_type: Type,

    #[derivative(PartialEq = "ignore", Ord = "ignore", PartialOrd = "ignore")]
    pub constraints: VectorConstraints,
}

impl VectorType {
    pub fn new(name: Name, element_type: Type) -> Self {
        Self {
            name,
            element_type,
            constraints: VectorConstraints::default(),
        }
    }

    pub fn element_size(&self) -> u32 {
        let size = self.constraints.size().clone();
        let size = if size.is_some() { size.unwrap().into() } else { std::u32::MAX };

        size
    }

    pub fn apply_constraints(
        &self,
        resolver: &crate::compiler::TypeResolver<'_, '_>,
        diagnostics: Rc<Diagnostics>,
        constraints: &LayoutConstraints,
        layout: &Reference,
    ) -> Result<Type, bool> {
        Ok(Type::Vector(Rc::from(self.clone())))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrimitiveType {
    pub name: Name,
    pub subtype: PrimitiveSubtype,
}

impl PrimitiveType {
    pub fn apply_constraints(
        &self,
        resolver: &crate::compiler::TypeResolver<'_, '_>,
        diagnostics: Rc<Diagnostics>,
        constraints: &LayoutConstraints,
        layout: &Reference,
    ) -> Result<Type, bool> {
        /*if !resolve_and_merge_constraints(
            resolver,
            reporter,
            constraints.span,
            layout.resolved().name(),
            nullptr,
            constraints.items,
            nullptr,
        ) {
            return Err(false);
        }*/

        Ok(Type::Primitive(Rc::from(PrimitiveType {
            name: self.name.clone(),
            subtype: self.subtype.clone(),
        })))
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct StringType {
    #[derivative(PartialEq = "ignore", Ord = "ignore", PartialOrd = "ignore")]
    pub constraints: VectorConstraints,
    pub name: Name,
}

impl StringType {
    pub fn new(name: Name) -> Self {
        Self {
            constraints: VectorConstraints::default(),
            name,
        }
    }

    pub fn new_with_constraints(name: Name, constraints: VectorConstraints) -> Self {
        Self { constraints, name }
    }

    pub fn max_size(&self) -> u32 {
        0
    }

    pub fn apply_constraints(
        &self,
        resolver: &crate::compiler::TypeResolver<'_, '_>,
        diagnostics: Rc<Diagnostics>,
        constraints: &LayoutConstraints,
        layout: &Reference,
    ) -> Result<Type, bool> {
        let c = VectorConstraints::default();
        //if (!self.resolve_and_merge_constraints(resolver, reporter, constraints.span, layout.resolved().name(), nullptr, constraints.items, &c, out_params)) {
        //  return false;
        //}

        Ok(Type::String(Rc::new(StringType::new_with_constraints(
            self.name.clone(),
            c,
        ))))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InternalType {
    pub name: Name,
    pub subtype: InternalSubtype,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct IdentifierType {
    pub name: Name,
    pub decl: Declaration,
    #[derivative(PartialEq = "ignore", Ord = "ignore", PartialOrd = "ignore")]
    pub constraints: IdentifierConstraints,
}

impl IdentifierType {
    pub fn new(decl: Declaration) -> Self {
        Self {
            decl: decl.clone(),
            name: decl.name(),
            constraints: IdentifierConstraints::default(),
        }
    }
    pub fn apply_constraints(
        &self,
        resolver: &crate::compiler::TypeResolver<'_, '_>,
        diagnostics: Rc<Diagnostics>,
        constraints: &LayoutConstraints,
        layout: &Reference,
    ) -> Result<Type, bool> {
        //if (!self.resolve_and_merge_constraints(resolver, reporter, constraints.span, layout.resolved().name(), nullptr, constraints.items, &c, out_params)) {
        //  return false;
        //}

        Ok(Type::Identifier(Rc::new(self.clone())))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    ArrayType {
        element_type: Box<Type>,
    },
    Vector(Rc<VectorType>),
    Primitive(Rc<PrimitiveType>),
    Internal(Rc<InternalType>),
    String(Rc<StringType>),
    Identifier(Rc<IdentifierType>),
    RequestType {
        nullable: bool,
        subtype: String, // "test.handles/DriverProtocol",
    },
    HandleType {
        nullable: bool,
        resource_identifier: String,
        subtype: String,
        rights: u32,
    },
}

impl Type {
    pub fn is_nullable(&self) -> bool {
        match self {
            Type::ArrayType { element_type } => false,
            Type::Vector(_) => false,
            Type::Primitive(_) => false,
            Type::Internal(_) => false,
            Type::String(string) => string.constraints.nullabilty() == Nullability::Nullable,
            Type::Identifier(_) => false,
            Type::RequestType { nullable, subtype } => *nullable,
            Type::HandleType {
                nullable,
                resource_identifier,
                subtype,
                rights,
            } => *nullable,
        }
    }
}
