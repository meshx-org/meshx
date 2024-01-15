use std::{cell::RefCell, rc::Rc};

use super::{
    AttributeList, Comment, Declaration, Identifier, Literal, Name, Reference, Span, TypeConstructor, WithAttributes,
    WithDocumentation, WithIdentifier, WithName, WithSpan,
};

/// Represents an identifier constant
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdentifierConstant {
    /// The referenced identifier of the contant.
    ///
    /// ```ignore
    /// const FOO u32 = foo.BAR
    ///                 ^^^^^^^
    /// ```
    pub(crate) reference: Reference,
}

/// Represents a literal constant value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiteralConstant {
    /// The literal value of the constant.
    ///
    /// ```ignore
    /// const FOO u32 = 10
    ///                 ^^
    /// ```
    pub(crate) value: Literal,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConstantValue {
    Identifier(IdentifierConstant),
    Literal(LiteralConstant),
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
    pub identifier: Identifier,

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
    pub(crate) value: ConstantValue,

    /// The location of this constant in the text representation.
    pub(crate) span: Span,
}

impl Into<Declaration> for Const {
    fn into(self) -> Declaration {
        Declaration::Const(Rc::new(RefCell::new(self)))
    }
}

impl WithIdentifier for Const {
    fn identifier(&self) -> &Identifier {
        &self.identifier
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
