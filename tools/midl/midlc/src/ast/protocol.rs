use super::{Attribute, Comment, Identifier, Span};

#[derive(Debug)]
pub struct Protocol {
    /// The name of the protoocl.
    ///
    /// ```ignore
    /// protocol Foo { .. }
    ///          ^^^
    /// ```
    pub(crate) name: Identifier,

    /// The attributes of this protocol.
    ///
    /// ```ignore
    /// @dicoverable(true)
    /// ^^^^^^^^^^^^
    /// protocol Foo {
    ///   ...
    /// }
    /// ```
    pub attributes: Vec<Attribute>,

    /// The location of this protocol in the text representation.
    pub(crate) span: Span,
}