use super::{Attribute, Identifier, Name, Span};

/// An AST node with a span.
pub trait WithSpan {
    /// The span of the node.
    fn span(&self) -> Span;
}

/// An AST node with a name (from the identifier).
pub trait WithName {
    /// The name of the item.
    fn name(&self) -> &Name;
}

/// An AST node with an identifier.
pub trait WithIdentifier {
    /// The identifier.
    fn identifier(&self) -> &Identifier;
}

/// An AST node with attributes.
pub trait WithAttributes {
    /// The attributes.
    fn attributes(&self) -> &[Attribute];
}

/// An AST node with documentation.
pub trait WithDocumentation {
    /// The documentation string, if defined.
    fn documentation(&self) -> Option<&str>;
}
