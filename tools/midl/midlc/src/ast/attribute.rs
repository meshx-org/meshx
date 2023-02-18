use super::{Identifier, Span, WithIdentifier, WithSpan};

/// An attribute (following `@`) on a struct, struct member, enum, enum value or constant
/// field.
#[derive(Debug, Clone)]
pub struct Attribute {
    /// The name of the attribute:
    ///
    /// ```ignore
    /// @discoverable(true)
    ///  ^^^^^^^^^^^^
    /// ```
    pub name: Identifier,

    /// The arguments of the attribute.
    ///
    /// ```ignore
    /// @discoverable(true)
    ///               ^^^^
    /// ```
    // pub arguments: ArgumentsList,

    /// The AST span of the node.
    pub span: Span,
}

/*impl Attribute {
    /// Try to find the argument and return its span.
    pub fn span_for_argument(&self, argument: &str) -> Option<Span> {
        self.arguments
            .iter()
            .find(|a| a.name.as_ref().map(|n| n.name.as_str()) == Some(argument))
            .map(|a| a.span)
    }
}*/

impl WithIdentifier for Attribute {
    fn identifier(&self) -> &Identifier {
        &self.name
    }
}

impl WithSpan for Attribute {
    fn span(&self) -> Span {
        self.span
    }
}
