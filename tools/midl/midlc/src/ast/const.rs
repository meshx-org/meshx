use super::{Literal, Identifier, Type, Attribute, WithIdentifier, WithSpan, Span, WithAttributes};

/// Represents a constant value.
#[derive(Debug)]
pub struct Constant(pub Literal);

#[derive(Debug)]
pub struct ConstDeclaration {
    pub name: Identifier,
    pub ty: Type,
    pub value: Constant,

    /// The attributes of this constant.
    ///
    /// ```ignore
    /// @example("Bar")
    /// ^^^^^^^^^^^^
    /// const FOO :u32 = 10
    /// ```
    pub attributes: Vec<Attribute>,
    
    /// The location of this constant in the text representation.
    pub(crate) span: Span,
}

impl WithIdentifier for ConstDeclaration {
    fn identifier(&self) -> &Identifier {
        &self.name
    }
}

impl WithSpan for ConstDeclaration {
    fn span(&self) -> Span {
        self.span
    }
}

impl WithAttributes for ConstDeclaration {
    fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}
