use derivative::Derivative;

use crate::diagnotics::Diagnostics;

use super::{
    constraints::IdentifierConstraints, Alias, Const, Constant, ConstantValue, ConstraintKind, Declaration, Element,
    HandleConstraints, Identifier, LiteralConstant, MergeConstraints, Name, Nullability, NullabilityTrait, Reference,
    ResolveAndMerge, Resource, Span, TransportSideConstraints, VectorConstraints,
};
use std::{cell::RefCell, rc::Rc, str::FromStr};

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum TransportSide {
    Client,
    Server,
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
            Element::BitsMember { .. } => {}
            Element::EnumMember { .. } => {
                // self.as_constant = IdentifierConstant (source_signature(), reference, span);
            }
            _ => {
                self.as_type_ctor.replace(Some(TypeConstructor {
                    layout: self.reference.clone(),
                    parameters: LayoutParameterList {
                        items: vec![],
                        span: None,
                    },
                    constraints: LayoutConstraints {
                        items: vec![],
                        span: Some(Span::empty()),
                    },
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

/// This is a struct used to group together all data produced during compilation
/// that might be used by consumers that are downstream from type compilation
/// (e.g. typeshape code, declaration sorting, JSON generator), that can't be
/// obtained by looking at a type constructor's Type.
/// Unlike TypeConstructor::Type which will always refer to the fully resolved/
/// concrete (and eventually, canonicalized) type that the type constructor
/// resolves to, this struct stores data about the actual parameters on this
/// type constructor used to produce the type.
/// These fields should be set in the same place where the parameters actually get
/// resolved, i.e. create (for layout parameters) and apply_constraints (for type
/// constraints)
pub struct LayoutInvocation {
    /// Set if this type constructor refers to an alias
    from_alias: Option<Alias>,
    // Parameter data below: if a foo_resolved form is set, then its corresponding
    // foo_raw form must be defined as well (and vice versa).
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayoutParameterList {
    pub items: Vec<LayoutParameter>,

    /// The location of this parameter list in the text representation.
    pub span: Option<Span>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayoutConstraints {
    /// The location of this constraints list in the text representation.
    pub span: Option<Span>,

    pub items: Vec<Constant>,
}

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
        let size = if size.is_some() {
            size.unwrap().into()
        } else {
            std::u32::MAX
        };

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

#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct TransportSideType {
    #[derivative(PartialEq = "ignore", Ord = "ignore", PartialOrd = "ignore")]
    pub constraints: Box<TransportSideConstraints>,

    pub name: Name,
    pub end: TransportSide,
    pub protocol_transport: String,
}

impl TransportSideType {
    pub(crate) fn new(name: Name, end: TransportSide, transport: &str) -> Self {
        Self {
            name,
            end,
            protocol_transport: transport.to_owned(),
            constraints: Box::new(TransportSideConstraints::default()),
        }
    }

    pub fn apply_constraints(
        &mut self,
        resolver: &crate::compiler::TypeResolver<'_, '_>,
        diagnostics: Rc<Diagnostics>,
        constraints: &LayoutConstraints,
        layout: &Reference,
    ) -> Result<Type, bool> {
        let mut c = TransportSideConstraints::default();
        
        if !self.constraints.resolve_and_merge_constraints(
            resolver,
            diagnostics,
            constraints.span.clone(),
            &layout.resolved().unwrap().name(),
            None,
            &constraints.items,
            &mut c,
            /*out_params,*/
        ) {
            return Err(false);
        }

        if !c.has_constraint(ConstraintKind::Protocol) {
            panic!("ErrProtocolConstraintRequired {:?}", constraints.span.clone());
            //    return reporter.Fail(ErrProtocolConstraintRequired, layout.span(), layout.resolved().name());
        }

        // let transport_attribute = c.protocol_decl.attributes.Get("transport");
        // std::string_view transport("Channel");

        //if (transport_attribute) {
        //    let arg = if (transport_attribute.compiled) {
        //                 transport_attribute.GetArg(AttributeArg::kDefaultAnonymousName)} else {
        //                 transport_attribute.GetStandaloneAnonymousArg()};
        //    let quoted_transport = static_cast<const LiteralConstant*>(arg->value.get())->literal->span().data();
        //    // Remove quotes around the transport.
        //    transport = quoted_transport.substr(1, quoted_transport.size() - 2);
        //}

        Ok(Type::TransportSide(TransportSideType {
            constraints: Box::new(c),
            name: self.name.clone(),
            end: self.end,
            protocol_transport: self.protocol_transport.clone(),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InternalType {
    pub name: Name,
    pub subtype: InternalSubtype,
}

impl InternalType {
    fn new(name: Name, subtype: InternalSubtype) -> Self {
        Self { name, subtype }
    }

    pub fn apply_constraints(
        &self,
        resolver: &crate::compiler::TypeResolver<'_, '_>,
        diagnostics: Rc<Diagnostics>,
        constraints: &LayoutConstraints,
        layout: &Reference,
    ) -> Result<Type, bool> {
        //if (!ResolveAndMergeConstraints(resolver, reporter, constraints.span, layout.resolved().name(),
        //                          nullptr, constraints.items, nullptr)) {
        //    return false;
        //}

        Ok(Type::Internal(Rc::new(InternalType::new(
            self.name.clone(),
            self.subtype,
        ))))
    }
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

#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct BoxType {
    pub name: Name,
    pub boxed_type: Type,
}

impl BoxType {
    pub(crate) fn new(name: Name, boxed_type: Type) -> Self {
        Self { name, boxed_type }
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

        Ok(Type::Box(Rc::new(self.clone())))
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct HandleType {
    pub name: Name,
    pub resource: Rc<RefCell<Resource>>,
    pub constraints: HandleConstraints,
}

impl HandleType {
    pub(crate) fn new(name: Name, resource: Rc<RefCell<Resource>>) -> Self {
        Self {
            name,
            resource,
            constraints: HandleConstraints::default(),
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

        Ok(Type::Handle(Rc::new(self.clone())))
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct ArrayType {
    pub name: Name,
    pub element_type: Type,

    #[derivative(PartialEq = "ignore", Ord = "ignore", PartialOrd = "ignore")]
    pub size_value: ConstantValue,
}

impl ArrayType {
    pub(crate) fn new(name: Name, element_type: Type, size_value: ConstantValue) -> Self {
        Self {
            name,
            element_type,
            size_value,
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

        Ok(Type::Array(Rc::new(self.clone())))
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct UntypedNumericType {
    pub name: Name,
}

impl UntypedNumericType {
    pub(crate) fn new(name: Name) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Array(Rc<ArrayType>),
    Box(Rc<BoxType>),
    Vector(Rc<VectorType>),
    Primitive(Rc<PrimitiveType>),
    Internal(Rc<InternalType>),
    String(Rc<StringType>),
    Identifier(Rc<IdentifierType>),
    TransportSide(TransportSideType),
    Handle(Rc<HandleType>),
    UntypedNumeric(Rc<UntypedNumericType>),
    RequestType {
        nullable: bool,
        subtype: String, // "test.handles/DriverProtocol",
    },
}

impl Type {
    pub fn is_nullable(&self) -> bool {
        match self {
            Type::Array(_) => false,
            Type::Vector(_) => false,
            Type::Primitive(_) => false,
            // All boxes are implicitly nullable.
            Type::Box(_) => true,
            Type::Internal(_) => false,
            Type::String(string) => string.constraints.nullability() == Nullability::Nullable,
            Type::Identifier(_) => false,
            Type::RequestType { nullable, subtype } => *nullable,
            Type::TransportSide(transport) => transport.constraints.nullability() == Nullability::Nullable,
            Type::Handle(handle) => handle.constraints.nullability() == Nullability::Nullable,
            Type::UntypedNumeric(_) => false,
        }
    }

    pub fn name(&self) -> Name {
        match self {
            Type::Array(typ) => typ.name.clone(),
            Type::Vector(typ) => typ.name.clone(),
            Type::Primitive(typ) => typ.name.clone(),
            Type::Box(typ) => typ.name.clone(),
            Type::Internal(typ) => typ.name.clone(),
            Type::String(typ) => typ.name.clone(),
            Type::Identifier(typ) => typ.name.clone(),
            Type::RequestType { nullable, subtype } => todo!(),
            Type::TransportSide(typ) => typ.name.clone(),
            Type::UntypedNumeric(typ) => typ.name.clone(),
            Type::Handle(typ) => typ.name.clone(),
        }
    }

    pub(crate) fn apply_constraints(
        &self,
        resolver: &crate::compiler::TypeResolver<'_, '_>,
        diagnostics: Rc<Diagnostics>,
        constraints: &LayoutConstraints,
        layout: &Reference,
    ) -> Result<Type, bool> {
        match self {
            Type::Array(typ) => typ.apply_constraints(resolver, diagnostics, constraints, layout),
            Type::Vector(typ) => typ.apply_constraints(resolver, diagnostics, constraints, layout),
            Type::Primitive(typ) => typ.apply_constraints(resolver, diagnostics, constraints, layout),
            Type::Internal(_) => todo!(),
            Type::Box(typ) => typ.apply_constraints(resolver, diagnostics, constraints, layout),
            Type::String(typ) => typ.apply_constraints(resolver, diagnostics, constraints, layout),
            Type::Handle(typ) => typ.apply_constraints(resolver, diagnostics, constraints, layout),
            Type::Identifier(typ) => typ.apply_constraints(resolver, diagnostics, constraints, layout),
            Type::TransportSide(_) => todo!(),
            Type::UntypedNumeric(_) => todo!(),
            Type::RequestType { nullable, subtype } => todo!(),
        }
    }
}
