use std::{cell::RefCell, rc::Rc};

use super::{
    Attribute, AttributeList, Comment, Constant, Decl, Declaration, Element, Identifier, Name, RawOrdinal64, Span, Strictness, TypeConstructor, WithAttributes, WithDocumentation, WithIdentifier, WithName, WithSpan
};

/// An opaque identifier for a field in an AST model. Use the
/// `model[field_id]` syntax to resolve the id to an `ast::Field`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableMemberId(pub(super) u32);

impl TableMemberId {
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MIN: TableMemberId = TableMemberId(0);
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MAX: TableMemberId = TableMemberId(u32::MAX);
}

impl std::ops::Index<TableMemberId> for Table {
    type Output = Rc<RefCell<TableMember>>;

    fn index(&self, index: TableMemberId) -> &Self::Output {
        &self.members[index.0 as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableMemberUsed {
    pub(crate) name: Span,
    pub(crate) type_ctor: TypeConstructor,
}

/// A table member declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableMember {
    /// The attributes of this table member.
    ///     
    /// ```ignore
    /// table Foo {
    ///  id i32 @id
    ///         ^^^
    /// }
    /// ```
    pub(crate) attributes: AttributeList,

    /// The documentation for this table member.
    ///
    /// ```ignore
    /// type Foo = table {
    ///  /// Lorem ipsum
    ///  ^^^^^^^^^^^^^^^
    ///  id i32
    /// }
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The ordinal for this table member.
    ///
    /// ```ignore
    /// type Foo = table {
    ///  1: id i32
    ///  ^
    /// }
    /// ```
    pub(crate) ordinal: RawOrdinal64,

    pub(crate) maybe_used: Option<TableMemberUsed>,

    /// The location of this table in the text representation.
    pub(crate) span: Span,
}

/// A table declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Table {
    /// The name of the table.
    /// NOTE: inline tables get their name automatically from the complier
    ///
    /// ```ignore
    /// type Foo = table { .. }
    ///      ^^^
    /// ```
    pub(crate) name: Name,

    /// The members of the table.
    ///
    /// ```ignore
    /// type Foo = table {
    ///   1: id int @id
    ///   ^^^^^^^^^^^^^
    ///   2: member string
    ///   ^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) members: Vec<Rc<RefCell<TableMember>>>,

    /// The attributes of this table.
    ///
    /// ```ignore
    /// @db.map("Bar")
    /// ^^^^^^^^^^^^
    /// table Foo {
    ///   id    :u32    @id
    ///   member :string
    /// }
    /// ```
    pub attributes: AttributeList,

    /// The documentation for this table.
    ///
    /// ```ignore
    /// /// Lorem ipsum
    ///     ^^^^^^^^^^^
    /// table Foo {
    ///   id    :u32   @id
    ///   field :string
    /// }
    /// ```
    pub(crate) documentation: Option<Comment>,

    pub(crate) strictness: Strictness,

    /// The location of this table in the text representation.
    pub(crate) span: Span,

    // Set during compilation
    pub(crate) compiled: bool,
    pub(crate) compiling: bool,
    pub(crate) recursive: bool,
}

impl Into<Declaration> for Table {
    fn into(self) -> Declaration {
        Declaration::Table {
            decl: Rc::new(RefCell::new(self)),
        }
    }
}

impl Table {
    pub fn iter_members(&self) -> impl ExactSizeIterator<Item = (TableMemberId, &Rc<RefCell<TableMember>>)> + Clone {
        self.members
            .iter()
            .enumerate()
            .map(|(idx, field)| (TableMemberId(idx as u32), field))
    }
}

impl WithSpan for Table {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl WithAttributes for Table {
    fn attributes(&self) -> &AttributeList {
        &self.attributes
    }
}

impl WithDocumentation for Table {
    fn documentation(&self) -> Option<&str> {
        self.documentation.as_ref().map(|doc| doc.text.as_str())
    }
}

impl WithName for Table {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl Decl for Table {
    fn compiling(&self) -> bool {
        todo!()
    }

    fn compiled(&self) -> bool {
        todo!()
    }

    fn set_compiling(&mut self, val: bool) {
        todo!()
    }

    fn set_compiled(&mut self, val: bool) {
        todo!()
    }
}