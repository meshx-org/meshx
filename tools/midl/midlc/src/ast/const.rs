use super::{
    Attribute, Comment, Identifier, Literal, Span, Type, WithAttributes, WithDocumentation, WithIdentifier,
    WithSpan,
};

/// Represents a constant value.
#[derive(Debug)]
pub struct Constant(pub Literal);

#[derive(Debug)]
pub struct Const {
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

    /// The documentation for this constant.
    ///
    /// ```ignore
    /// /// Lorem ipsum
    ///     ^^^^^^^^^^^
    /// const FOO :u32 = 10
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The location of this constant in the text representation.
    pub(crate) span: Span,
}

impl WithIdentifier for Const {
    fn identifier(&self) -> &Identifier {
        &self.name
    }
}

impl WithSpan for Const {
    fn span(&self) -> Span {
        self.span
    }
}

impl WithAttributes for Const {
    fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}

impl WithDocumentation for Const {
    fn documentation(&self) -> Option<&str> {
        self.documentation.as_ref().map(|c| c.text.as_str())
    }
}
