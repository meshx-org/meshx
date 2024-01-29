use std::{cell::RefCell, rc::Rc};

use super::{
    traits::{Decl, TypeDecl},
    Attribute, AttributeList, Comment, Constant, Declaration, Element, Identifier, Name, PrimitiveType, Span,
    TypeConstructor, WithAttributes, WithDocumentation, WithIdentifier, WithName, WithSpan,
};

/// An opaque identifier for a field in an AST model. Use the
/// `model[field_id]` syntax to resolve the id to an `ast::Field`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnumMemberId(pub(super) u32);

impl EnumMemberId {
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MIN: EnumMemberId = EnumMemberId(0);
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MAX: EnumMemberId = EnumMemberId(u32::MAX);
}

impl std::ops::Index<EnumMemberId> for Enum {
    type Output = Rc<RefCell<EnumMember>>;

    fn index(&self, index: EnumMemberId) -> &Self::Output {
        &self.members[index.0 as usize]
    }
}

/// A enum member declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnumMember {
    pub(crate) name: Span,

    /// The value of this enum member.
    ///     
    /// ```ignore
    /// type Foo = enum {
    ///  FOO = 1
    ///        ^
    /// }
    /// ```
    pub(crate) value: Constant,

    /// The attributes of this enum member.
    ///     
    /// ```ignore
    /// type Foo = enum {
    ///  FOO = 1 @id
    ///          ^^^
    /// }
    /// ```
    pub(crate) attributes: Vec<Attribute>,

    /// The documentation for this struct member.
    ///
    /// ```ignore
    /// type Foo = enum {
    ///  /// Lorem ipsum
    ///  ^^^^^^^^^^^^^^^
    ///  id i32
    /// }
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The location of this enum in the text representation.
    pub(crate) span: Span,
}

/// A enum declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Enum {
    /// The name of the enum.
    /// NOTE: inline enums get their name automatically from the complier
    ///
    /// ```ignore
    /// type Foo = flexible enum { .. }
    ///      ^^^
    /// ```
    pub(crate) name: Name,

    /// Subtype of the enum.
    /// NOTE: when it is missing in the definition it is defaulted to uint32
    ///
    /// ```ignore
    /// type Foo = flexible enum : uint8 { .. }
    ///                            ^^^^^
    /// ```
    pub(crate) subtype_ctor: TypeConstructor,

    /// Legacy subtype
    pub(crate) r#type: Option<Rc<PrimitiveType>>,

    /// The members of the enum.
    ///
    /// ```ignore
    /// type Foo = flexible enum {
    ///   FOO = 1
    ///   ^^^^^^^
    ///   BAR = 2
    ///   ^^^^^^^
    /// }
    /// ```
    pub(crate) members: Vec<Rc<RefCell<EnumMember>>>,

    /// The attributes of this struct.
    ///
    /// ```ignore
    /// @doc("Bar")
    /// ^^^^^^^^^^^
    /// type Foo = enum {
    ///   FOO = 1
    ///   BAR = 2
    /// }
    /// ```
    pub attributes: AttributeList,

    /// The documentation for this struct.
    ///
    /// ```ignore
    /// /// Lorem ipsum
    ///     ^^^^^^^^^^^
    /// type Foo = enum {
    ///   FOO = 1
    ///   fBAR = 2
    /// }
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The location of this enum in the text representation.
    pub(crate) span: Span,

    pub(crate) unknown_value_signed: i64,
    pub(crate) unknown_value_unsigned: u64,

    // Set during compilation
    pub(crate) compiled: bool,
    pub(crate) compiling: bool,
    pub(crate) recursive: bool,
}

impl Into<Declaration> for Enum {
    fn into(self) -> Declaration {
        Declaration::Enum {
            decl: Rc::new(RefCell::new(self)),
        }
    }
}

impl Enum {
    pub fn iter_members(&self) -> impl ExactSizeIterator<Item = (EnumMemberId, &Rc<RefCell<EnumMember>>)> + Clone {
        self.members
            .iter()
            .enumerate()
            .map(|(idx, field)| (EnumMemberId(idx as u32), field))
    }
}

impl WithSpan for Enum {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl WithAttributes for Enum {
    fn attributes(&self) -> &AttributeList {
        &self.attributes
    }
}

impl WithDocumentation for Enum {
    fn documentation(&self) -> Option<&str> {
        self.documentation.as_ref().map(|doc| doc.text.as_str())
    }
}

impl WithName for Enum {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl Decl for Enum {
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

impl TypeDecl for Enum {
    fn set_recursive(&mut self, value: bool) {
        self.recursive = value;
    }
}
