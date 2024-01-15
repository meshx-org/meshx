use std::{cell::RefCell, rc::Rc};

use super::{
    AttributeList, Comment, Declaration, Name, Span, TypeConstructor, WithAttributes, WithDocumentation, WithName,
    WithSpan,
};

/// An alias declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Alias {
    /// The name of the type alias.
    ///
    /// ```ignore
    /// alias Foo = bool
    ///       ^^^
    /// ```
    pub(crate) name: Name,

    // The shape of this type constructor is more constrained than just being a
    // "partial" type constructor - it is either a normal type constructor
    // referring directly to a non-type-alias with all layout parameters fully
    // specified (e.g. alias foo = array<T, 3>), or it is a type constructor
    // referring to another alias that has no layout parameters (e.g. alias
    // bar = foo).
    // The constraints on the other hand are indeed "partial" - any alias
    // at any point in an "alias chain" can specify a constraint, but any
    // constraint can only specified once. This behavior will change in fxbug.dev/74193.
    pub(crate) partial_type_ctor: TypeConstructor,

    /// The attributes of this type alias.
    ///
    /// ```ignore
    /// @available(added=1)
    /// ^^^^^^^^^^^^^^^^^^^
    /// alias Foo = bool
    /// ```
    pub(crate) attributes: AttributeList,

    /// The documentation for this alias.
    ///
    /// ```ignore
    /// /// Lorem ipsum
    ///     ^^^^^^^^^^^
    /// alias Foo = bool
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The location of this alias in the text representation.
    pub(crate) span: Span,
}

impl Into<Declaration> for Alias {
    fn into(self) -> Declaration {
        Declaration::Alias(Rc::new(RefCell::new(self)))
    }
}

impl WithSpan for Alias {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl WithAttributes for Alias {
    fn attributes(&self) -> &AttributeList {
        &self.attributes
    }
}

impl WithDocumentation for Alias {
    fn documentation(&self) -> Option<&str> {
        self.documentation.as_ref().map(|doc| doc.text.as_str())
    }
}

impl WithName for Alias {
    fn name(&self) -> &Name {
        &self.name
    }
}
