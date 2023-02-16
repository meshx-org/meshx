use super::{Attribute, Comment, Identifier, CompoundIdentifier, Span};

#[derive(Debug)]
struct RequestType {

}

#[derive(Debug)]
pub struct ProtocolMethod {
    /// The name of the protocol method.
    ///
    /// ```ignore
    /// protocol Foo { 
    ///     Bar()
    ///     ^^^
    /// }
    /// 
    /// ```
    pub(crate) name: Identifier,

    /// The documentation for this protocol.
    ///
    /// ```ignore
    /// protocol Foo {
    ///   /// Lorem ipsum
    ///       ^^^^^^^^^^^
    ///   Bar()
    /// }
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The identifier of the auto-generated method request type.
    ///
    /// ```ignore
    /// protocol Foo { 
    ///     Bar(struct { ... })
    ///         ^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) request_payload: Option<Identifier>,

    /// The identifier of the auto-generated method response type.
    ///
    /// ```ignore
    /// protocol Foo { 
    ///     Bar() -> (struct { ... })
    ///               ^^^^^^^^^^^^^^
    ///     // or
    ///     -> OnError(struct { status_code :u32 })
    ///                ^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) response_payload: Option<Identifier>,

    /// The location of this protocol member in the text representation.
    pub(crate) span: Span,
}

#[derive(Debug)]
pub struct Protocol {
    /// The names of the composed protocols.
    ///
    /// ```ignore
    /// protocol Foo {
    ///   compose Bar
    ///   ^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) composes: Vec<CompoundIdentifier>,

    /// The name of the protocol.
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
    ///   Bar()
    /// }
    /// ```
    pub attributes: Vec<Attribute>,

    /// The documentation for this protocol.
    ///
    /// ```ignore
    /// /// Lorem ipsum
    ///     ^^^^^^^^^^^
    /// protocol Foo {
    ///   Bar()
    /// }
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The methods of the protocol.
    ///
    /// ```ignore
    /// protocol Foo {
    ///   Bar()
    ///   ^^^^^
    /// }
    /// ```
    pub(crate) methods: Vec<ProtocolMethod>,

    /// The location of this protocol in the text representation.
    pub(crate) span: Span,
}
