use super::{ConstantValue, Name, Span, WithName, WithSpan};

/// We parse `///` doc comments as nameless raw::Attribute with `provenance`
/// set to raw::Attribute::Provenance::kDocComment. When consuming into a
/// flat::Attribute, we set the name to kDocCommentName.
pub static DOC_COMMENT_NAME: &str = "doc";

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
    pub name: Name,

    /// The arguments of the attribute.
    ///
    /// ```ignore
    /// @discoverable(true)
    ///               ^^^^
    /// ```
    // pub arguments: ArgumentsList,

    /// The AST span of the node.
    pub span: Span,

    pub arguments: Vec<AttributeArg>,

    /// Set to true by Library::CompileAttribute.
    pub compiled: bool,
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

impl WithName for Attribute {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl WithSpan for Attribute {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttributeArg {
    /// Span of just the argument name, e.g. "bar". This is initially null for
    /// arguments like `@foo("abc")`, but will be set during compilation.
    name: Name,

    pub value: ConstantValue,

    /// Span of the entire argument, e.g. `bar="abc"`, or `"abc"` if unnamed.
    span: Span,
}

impl AttributeArg {
    pub(crate) fn new(name: Name, span: Span, value: ConstantValue) -> Self {
        Self {
            name,
            span,
            value,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttributeList(pub Vec<Attribute>);
