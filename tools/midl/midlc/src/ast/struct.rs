use std::{cell::RefCell, rc::Rc};

use super::{
    traits::{Decl, TypeDecl},
    AttributeList, Comment, Constant, Declaration, Name, Span, TypeConstructor, WithAttributes,
    WithDocumentation, WithName, WithSpan,
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
    type Output = Rc<RefCell<StructMember>>;

    fn index(&self, index: StructMemberId) -> &Self::Output {
        &self.members[index.0 as usize]
    }
}

/// A struct member declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructMember {
    pub(crate) name: Span,

    pub(crate) type_ctor: TypeConstructor,

    /// The attributes of this struct member.
    ///     
    /// ```ignore
    /// struct Foo {
    ///  id i32 @id
    ///         ^^^
    /// }
    /// ```
    pub(crate) attributes: AttributeList,

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

    pub(crate) maybe_default_value: Option<Constant>,

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
    /// type Foo = struct { .. }
    ///      ^^^
    /// ```
    pub(crate) name: Name,

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
    pub(crate) members: Vec<Rc<RefCell<StructMember>>>,

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
    pub attributes: AttributeList,

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

    // Set during compilation
    pub(crate) compiled: bool,
    pub(crate) compiling: bool,
    pub(crate) recursive: bool,
}

impl Into<Declaration> for Struct {
    fn into(self) -> Declaration {
        Declaration::Struct {
            decl: Rc::new(RefCell::new(self)),
        }
    }
}

impl Struct {
    pub fn iter_members(&self) -> impl ExactSizeIterator<Item = (StructMemberId, &Rc<RefCell<StructMember>>)> + Clone {
        self.members
            .iter()
            .enumerate()
            .map(|(idx, field)| (StructMemberId(idx as u32), field))
    }
}

impl WithSpan for Struct {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl WithAttributes for Struct {
    fn attributes(&self) -> &AttributeList {
        &self.attributes
    }
}

impl WithDocumentation for Struct {
    fn documentation(&self) -> Option<&str> {
        self.documentation.as_ref().map(|doc| doc.text.as_str())
    }
}

impl WithName for Struct {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl Decl for Struct {
    fn compiling(&self) -> bool {
        self.compiling
    }

    fn compiled(&self) -> bool {
        self.compiled
    }

    fn set_compiling(&mut self, val: bool) {
        self.compiling = val;
    }

    fn set_compiled(&mut self, val: bool) {
        self.compiled = val;
    }
}

impl TypeDecl for Struct {
    fn set_recursive(&mut self, value: bool) {
        self.recursive = value;
    }
}
