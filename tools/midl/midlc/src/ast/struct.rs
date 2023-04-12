use std::{cell::RefCell, rc::Rc};

use super::{
    Attribute, Comment, Declaration, Identifier, Span, TypeConstructor, WithAttributes, WithDocumentation,
    WithIdentifier, WithSpan,
};

/// An opaque identifier for a field in an AST model. Use the
/// `model[field_id]` syntax to resolve the id to an `ast::Field`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructMemberId(pub(super) u32);

impl StructMemberId {
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MIN: StructMemberId = StructMemberId(0);
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MAX: StructMemberId = StructMemberId(u32::MAX);
}

impl std::ops::Index<StructMemberId> for Struct {
    type Output = StructMember;

    fn index(&self, index: StructMemberId) -> &Self::Output {
        &self.members[index.0 as usize]
    }
}

/// A struct member declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructMember {
    pub(crate) name: Identifier,

    pub(crate) member_type_ctor: TypeConstructor,

    /// The attributes of this struct member.
    ///     
    /// ```ignore
    /// struct Foo {
    ///  id i32 @id
    ///         ^^^
    /// }
    /// ```
    pub(crate) attributes: Vec<Attribute>,

    /// The documentation for this struct member.
    /// 
    /// ```ignore
    /// struct Foo {
    ///  /// Lorem ipsum
    ///  ^^^^^^^^^^^^^^^
    ///  id i32
    /// }
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The location of this struct in the text representation.
    pub(crate) span: Span,
}

/// A struct declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Struct {
    /// The name of the struct.
    /// NOTE: inline structs get their name automatically from the complier
    ///
    /// ```ignore
    /// struct Foo { .. }
    ///        ^^^
    /// ```
    pub(crate) name: Identifier,

    /// The members of the struct.
    ///
    /// ```ignore
    /// struct Foo {
    ///   id    :int    @id
    ///   ^^^^^^^^^^^^^^^^
    ///   member :string
    ///   ^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) members: Vec<StructMember>,

    /// The attributes of this struct.
    ///
    /// ```ignore
    /// @db.map("Bar")
    /// ^^^^^^^^^^^^
    /// struct Foo {
    ///   id    :u32    @id
    ///   member :string
    /// }
    /// ```
    pub attributes: Vec<Attribute>,

    /// The documentation for this struct.
    ///
    /// ```ignore
    /// /// Lorem ipsum
    ///     ^^^^^^^^^^^
    /// struct Foo {
    ///   id    :u32   @id
    ///   field :string
    /// }
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The location of this struct in the text representation.
    pub(crate) span: Span,
}

impl Into<Declaration> for Struct {
    fn into(self) -> Declaration {
        Declaration::Struct(Rc::new(RefCell::new(self)))
    }
}

impl Struct {
    pub fn iter_members(&self) -> impl ExactSizeIterator<Item = (StructMemberId, &StructMember)> + Clone {
        self.members
            .iter()
            .enumerate()
            .map(|(idx, field)| (StructMemberId(idx as u32), field))
    }
}

impl WithIdentifier for Struct {
    fn identifier(&self) -> &Identifier {
        &self.name
    }
}

impl WithSpan for Struct {
    fn span(&self) -> Span {
        self.span
    }
}

impl WithAttributes for Struct {
    fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}

impl WithDocumentation for Struct {
    fn documentation(&self) -> Option<&str> {
        self.documentation.as_ref().map(|doc| doc.text.as_str())
    }
}
