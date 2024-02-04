use derivative::Derivative;
use std::{cell::RefCell, rc::Rc};

use super::{
    AttributeList, Comment, Decl, Declaration, Literal, Name, Reference, Span, Type, TypeConstructor, WithAttributes,
    WithDocumentation, WithName, WithSpan,
};

struct Numeric<T>(T);

#[derive(Debug, Copy, Clone)]
pub enum ConstantValueKind {
    Float64,
    Float32,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Bool,
    String,
}

#[derive(Debug, Clone)]
pub enum ConstantValue {
    Float64(f64),
    Float32(f32),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Bool(bool),
    String(String),
}

impl ConstantValue {
    pub fn convert(&self, kind: ConstantValueKind, val: &mut ConstantValue) -> bool {
        false
    }
}

impl From<ConstantValue> for u8 {
    fn from(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Uint8(v) => v,
            _ => panic!("cannot covert constant {} into u8 value", value),
        }
    }
}

impl From<ConstantValue> for u16 {
    fn from(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Uint16(v) => v,
            _ => panic!("cannot covert constant {} into u16 value", value),
        }
    }
}

impl From<ConstantValue> for u32 {
    fn from(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Uint32(v) => v,
            _ => panic!("cannot covert constant {} into u32 value", value),
        }
    }
}

impl From<ConstantValue> for u64 {
    fn from(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Uint64(v) => v,
            _ => panic!("cannot covert constant {} into u64 value", value),
        }
    }
}

impl From<ConstantValue> for i8 {
    fn from(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Int8(v) => v,
            _ => panic!("cannot covert constant {} into i8 value", value),
        }
    }
}

impl From<ConstantValue> for i16 {
    fn from(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Int16(v) => v,
            _ => panic!("cannot covert constant {} into i16 value", value),
        }
    }
}

impl From<ConstantValue> for i32 {
    fn from(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Int32(v) => v,
            _ => panic!("cannot covert constant {} into i32 value", value),
        }
    }
}

impl From<ConstantValue> for i64 {
    fn from(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Int64(v) => v,
            _ => panic!("cannot covert constant {} into i64 value", value),
        }
    }
}

impl std::fmt::Display for ConstantValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstantValue::Bool(val) => write!(f, "{}", if *val { "true" } else { "false" }),
            ConstantValue::Float64(val) => write!(f, "{}", val),
            ConstantValue::Float32(val) => write!(f, "{}", val),
            ConstantValue::Int8(val) => write!(f, "{}", val),
            ConstantValue::Int16(val) => write!(f, "{}", val),
            ConstantValue::Int32(val) => write!(f, "{}", val),
            ConstantValue::Int64(val) => write!(f, "{}", val),
            ConstantValue::String(val) => write!(f, "{}", val),
            ConstantValue::Uint8(val) => write!(f, "{}", val),
            ConstantValue::Uint16(val) => write!(f, "{}", val),
            ConstantValue::Uint32(val) => write!(f, "{}", val),
            ConstantValue::Uint64(val) => write!(f, "{}", val),
        }
    }
}

pub trait ConstantTrait {
    fn value(&self) -> ConstantValue;
    fn span(&self) -> &Span;
    fn is_resolved(&self) -> bool;
    fn resolve_to(&mut self, value: ConstantValue, r#type: Type);
}

/// Represents an identifier constant
#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct IdentifierConstant {
    /// The referenced identifier of the contant.
    ///
    /// ```ignore
    /// const FOO u32 = foo.BAR
    ///                 ^^^^^^^
    /// ```
    pub(crate) reference: Reference,

    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Ord = "ignore")]
    pub(crate) constant_value: Option<ConstantValue>,
    pub(crate) span: Span,

    /// compiled tracks whether we attempted to resolve this constant, to avoid
    /// resolving twice a constant which cannot be resolved.
    pub compiled: bool,
}

impl ConstantTrait for IdentifierConstant {
    fn value(&self) -> ConstantValue {
        assert!(self.is_resolved(), "accessing the value of an unresolved Constant: %s",);
        self.constant_value.as_ref().expect("assert made").clone()
    }

    fn is_resolved(&self) -> bool {
        self.constant_value.is_some()
    }

    fn span(&self) -> &Span {
        &self.span
    }

    fn resolve_to(&mut self, value: ConstantValue, r#type: Type) {
        assert!(!self.is_resolved(), "constants should only be resolved once");
        self.constant_value = Some(value);
        // self.r#type = r#type;
    }
}

/// Represents a literal constant value.
#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct LiteralConstant {
    /// The literal value of the constant.
    ///
    /// ```ignore
    /// const FOO uint32 = 10
    ///                    ^^
    /// ```
    pub(crate) literal: Literal,

    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Ord = "ignore")]
    pub(crate) constant_value: Option<ConstantValue>,

    pub(crate) span: Span,

    /// compiled tracks whether we attempted to resolve this constant, to avoid
    /// resolving twice a constant which cannot be resolved.
    pub compiled: bool,
}

impl ConstantTrait for LiteralConstant {
    fn value(&self) -> ConstantValue {
        assert!(
            self.is_resolved(),
            "accessing the value of an unresolved Constant: {:?}",
            self.constant_value
        );
        self.constant_value.as_ref().expect("assert made").clone()
    }

    fn is_resolved(&self) -> bool {
        self.constant_value.is_some()
    }

    fn span(&self) -> &Span {
        &self.span
    }

    fn resolve_to(&mut self, value: ConstantValue, r#type: Type) {
        assert!(!self.is_resolved(), "constants should only be resolved once");
        self.constant_value = Some(value);
    }
}

// Constant represents the _use_ of a constant. (For the _declaration_, see
// Const. For the _value_, see ConstantValue.) A Constant can either be a
// reference to another constant (IdentifierConstant), a literal value
// (LiteralConstant). Every Constant resolves to a concrete ConstantValue.
#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryOperatorConstant {
    pub lhs: Box<Constant>,
    pub op: ConstantOp,
    pub rhs: Box<Constant>,

    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Ord = "ignore")]
    pub(crate) constant_value: Option<ConstantValue>,

    /// compiled tracks whether we attempted to resolve this constant, to avoid
    /// resolving twice a constant which cannot be resolved.
    pub compiled: bool,
}

impl ConstantTrait for BinaryOperatorConstant {
    fn value(&self) -> ConstantValue {
        todo!()
    }

    fn span(&self) -> &Span {
        todo!()
    }

    fn is_resolved(&self) -> bool {
        todo!()
    }

    fn resolve_to(&mut self, value: ConstantValue, r#type: Type) {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum ConstantOp {
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Constant {
    Identifier(IdentifierConstant),
    Literal(LiteralConstant),
    BinaryOperator(BinaryOperatorConstant),
}

impl Constant {
    pub fn is_resolved(&self) -> bool {
        match self {
            Constant::Identifier(c) => c.constant_value.is_some(),
            Constant::Literal(c) => c.constant_value.is_some(),
            Constant::BinaryOperator(c) => c.constant_value.is_some(),
        }
    }

    pub fn value(&self) -> ConstantValue {
        assert!(
            self.is_resolved(),
            "accessing the value of an unresolved Constant: %s",
            // span.data().c_str()
        );

        match self {
            Constant::Identifier(c) => c.constant_value.clone().unwrap(),
            Constant::Literal(c) => c.constant_value.clone().unwrap(),
            Constant::BinaryOperator(c) => c.constant_value.clone().unwrap(),
        }
    }

    pub fn set_compiled(&mut self, value: bool) {
        match self {
            Constant::Identifier(c) => c.compiled = value,
            Constant::Literal(c) => c.compiled = value,
            Constant::BinaryOperator(c) => c.compiled = value,
        }
    }

    pub fn compiled(&self) -> bool {
        match self {
            Constant::Identifier(c) => c.compiled,
            Constant::Literal(c) => c.compiled,
            Constant::BinaryOperator(c) => c.compiled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Const {
    pub name: Name,

    /// The identifier of the constant.
    ///     
    /// ```ignore
    /// const FOO u32 = 10
    ///       ^^^
    /// ```
    // identifier: Identifier,

    /// The type of the constant.
    ///
    /// ```ignore
    /// const FOO u32 = 10
    ///           ^^^
    /// ```
    pub type_ctor: TypeConstructor,

    /// The attributes of the constant.
    ///
    /// ```ignore
    /// @example("Bar")
    /// ^^^^^^^^^^^^
    /// const FOO u32 = 10
    /// ```
    pub attributes: AttributeList,

    /// The documentation for the constant.
    ///
    /// ```ignore
    /// /// Lorem ipsum
    ///     ^^^^^^^^^^^
    /// const FOO u32 = 10
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The constant value
    pub(crate) value: Constant,

    /// The location of this constant in the text representation.
    pub(crate) span: Span,

    // Set during compilation
    pub(crate) compiled: bool,
    pub(crate) compiling: bool,
    pub(crate) recursive: bool,
}

impl Into<Declaration> for Const {
    fn into(self) -> Declaration {
        Declaration::Const {
            decl: Rc::new(RefCell::new(self)),
        }
    }
}

impl WithSpan for Const {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl WithAttributes for Const {
    fn attributes(&self) -> &AttributeList {
        &self.attributes
    }
}

impl WithDocumentation for Const {
    fn documentation(&self) -> Option<&str> {
        self.documentation.as_ref().map(|c| c.text.as_str())
    }
}

impl WithName for Const {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl Decl for Const {
    fn compiling(&self) -> bool {
        self.compiling
    }

    fn compiled(&self) -> bool {
        self.compiled
    }

    fn set_compiling(&mut self, val: bool) {
        self.compiling = val;
    }

    fn set_compiled(&mut self, val: bool) {
        self.compiled = val;
    }
}
