use std::{cell::RefCell, rc::Rc};

use super::{
    Attribute, Comment, CompoundIdentifier, Declaration, Identifier, Span, TypeConstructor, WithAttributes,
    WithDocumentation, WithIdentifier, WithSpan, WithName, Name, AttributeList,
};

#[derive(Debug)]
struct RequestType {}

/// An opaque identifier for a field in an AST model. Use the
/// `model[field_id]` syntax to resolve the id to an `ast::Field`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolMethodId(pub(super) u32);

impl ProtocolMethodId {
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MIN: ProtocolMethodId = ProtocolMethodId(0);
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MAX: ProtocolMethodId = ProtocolMethodId(u32::MAX);
}

impl std::ops::Index<ProtocolMethodId> for Protocol {
    type Output = ProtocolMethod;

    fn index(&self, index: ProtocolMethodId) -> &Self::Output {
        &self.methods[index.0 as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    pub(crate) has_request: bool,
    pub(crate) has_response: bool,
    pub(crate) has_error: bool,

    pub(crate) maybe_request: Option<TypeConstructor>,
    pub(crate) maybe_response: Option<TypeConstructor>,

    // Set during compilation
    // TODO: generated_ordinal64: raw::Ordinal64,
    /// The attributes of this protocol method.
    ///
    /// ```ignore
    /// protocol Foo {
    ///   Bar() @attr(true)
    ///         ^^^^^^^^^^^
    /// }
    /// ```
    pub attributes: Vec<Attribute>,

    /// The location of this protocol member in the text representation.
    pub(crate) span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Protocol {
    pub name: Name,
    
    /// The name of the protocol.
    ///
    /// ```ignore
    /// protocol Foo { .. }
    ///          ^^^
    /// ```
    pub(crate) identifier: Identifier,

    /// The names of the composed protocols.
    ///
    /// ```ignore
    /// protocol Foo {
    ///   compose Bar
    ///   ^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) composes: Vec<CompoundIdentifier>,

    /// The attributes of this protocol.
    ///
    /// ```ignore
    /// @dicoverable(true)
    /// ^^^^^^^^^^^^
    /// protocol Foo {
    ///   Bar()
    /// }
    /// ```
    pub attributes: AttributeList,

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
    pub(crate) methods: Vec<Rc<ProtocolMethod>>,

    /// The location of this protocol in the text representation.
    pub(crate) span: Span,
}

impl Protocol {
    pub fn iter_methods(&self) -> impl ExactSizeIterator<Item = (ProtocolMethodId, &Rc<ProtocolMethod>)> + Clone {
        self.methods
            .iter()
            .enumerate()
            .map(|(idx, method)| (ProtocolMethodId(idx as u32), method))
    }
}

impl Into<Declaration> for Protocol {
    fn into(self) -> Declaration {
        Declaration::Protocol(Rc::new(RefCell::new(self)))
    }
}

impl WithIdentifier for Protocol {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl WithSpan for Protocol {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl WithAttributes for Protocol {
    fn attributes(&self) -> &AttributeList {
        &self.attributes
    }
}

impl WithDocumentation for Protocol {
    fn documentation(&self) -> Option<&str> {
        self.documentation.as_ref().map(|doc| doc.text.as_str())
    }
}

impl WithName for Protocol {
    fn name(&self) -> &Name {
        &self.name
    }
}
