use std::{cell::RefCell, rc::Rc};

use super::{
    Attribute, AttributeList, Comment, Constant, Declaration, Element, Identifier, Name, RawOrdinal64, Span, Strictness, TypeConstructor, WithAttributes, WithDocumentation, WithIdentifier, WithName, WithSpan
};

/// An opaque identifier for a field in an AST model. Use the
/// `model[field_id]` syntax to resolve the id to an `ast::Field`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnionMemberId(pub(super) u32);

impl UnionMemberId {
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MIN: UnionMemberId = UnionMemberId(0);
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MAX: UnionMemberId = UnionMemberId(u32::MAX);
}

impl std::ops::Index<UnionMemberId> for Union {
    type Output = Rc<RefCell<UnionMember>>;

    fn index(&self, index: UnionMemberId) -> &Self::Output {
        &self.members[index.0 as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnionMemberUsed {
    pub(crate) name: Identifier,
    pub(crate) type_ctor: TypeConstructor,
}

/// A union member declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnionMember {
    /// The attributes of this union member.
    ///     
    /// ```ignore
    /// union Foo {
    ///  id i32 @id
    ///         ^^^
    /// }
    /// ```
    pub(crate) attributes: AttributeList,

    /// The documentation for this union member.
    ///
    /// ```ignore
    /// type Foo = union {
    ///  /// Lorem ipsum
    ///  ^^^^^^^^^^^^^^^
    ///  id i32
    /// }
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The ordinal for this union member.
    ///
    /// ```ignore
    /// type Foo = union {
    ///  1: id i32
    ///  ^
    /// }
    /// ```
    pub(crate) ordinal: RawOrdinal64,

    pub(crate) maybe_used: Option<UnionMemberUsed>,

    /// The location of this union in the text representation.
    pub(crate) span: Span,
}

/// A union declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Union {
    /// The name of the union.
    pub(crate) name: Name,

    /// The identifier of the union.
    /// NOTE: inline unions get their name automatically from the complier
    ///
    /// ```ignore
    /// union Foo { .. }
    ///        ^^^
    /// ```
    pub(crate) identifier: Identifier,

    /// The members of the union.
    ///
    /// ```ignore
    /// union Foo {
    ///   id    :int    @id
    ///   ^^^^^^^^^^^^^^^^
    ///   member :string
    ///   ^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) members: Vec<Rc<RefCell<UnionMember>>>,

    /// The attributes of this union.
    ///
    /// ```ignore
    /// @db.map("Bar")
    /// ^^^^^^^^^^^^
    /// union Foo {
    ///   id    :u32    @id
    ///   member :string
    /// }
    /// ```
    pub attributes: AttributeList,

    /// The documentation for this union.
    ///
    /// ```ignore
    /// /// Lorem ipsum
    ///     ^^^^^^^^^^^
    /// union Foo {
    ///   id    :u32   @id
    ///   field :string
    /// }
    /// ```
    pub(crate) documentation: Option<Comment>,

    pub(crate) strictness: Strictness,

    /// The location of this union in the text representation.
    pub(crate) span: Span,
}

impl Into<Declaration> for Union {
    fn into(self) -> Declaration {
        Declaration::Union {
            decl: Rc::new(RefCell::new(self)),
        }
    }
}

impl Union {
    pub fn iter_members(&self) -> impl ExactSizeIterator<Item = (UnionMemberId, &Rc<RefCell<UnionMember>>)> + Clone {
        self.members
            .iter()
            .enumerate()
            .map(|(idx, field)| (UnionMemberId(idx as u32), field))
    }
}

impl WithIdentifier for Union {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl WithSpan for Union {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl WithAttributes for Union {
    fn attributes(&self) -> &AttributeList {
        &self.attributes
    }
}

impl WithDocumentation for Union {
    fn documentation(&self) -> Option<&str> {
        self.documentation.as_ref().map(|doc| doc.text.as_str())
    }
}

impl WithName for Union {
    fn name(&self) -> &Name {
        &self.name
    }
}
