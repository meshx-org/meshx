use super::{ConstantValue, Identifier, Span, WithIdentifier, WithSpan};

/// An attribute (following `@`) on a struct, struct member, enum, enum value or constant
/// field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
        self.span.clone()
    }
}

pub struct AttributeArg {
    /// Span of just the argument name, e.g. "bar". This is initially null for
    /// arguments like `@foo("abc")`, but will be set during compilation.
    name: Option<Span>,

    pub value: Box<ConstantValue>,

    /// Span of the entire argument, e.g. `bar="abc"`, or `"abc"` if unnamed.
    span: Span,
}

impl AttributeArg {
    pub(crate) fn new(span: Span, value: Box<ConstantValue>) -> Self {
        Self { name: None, span, value }
    }
}
