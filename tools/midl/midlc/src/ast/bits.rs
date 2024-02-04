use std::{cell::RefCell, rc::Rc};

use super::{
    traits::{Decl, TypeDecl},
    Attribute, AttributeList, Comment, Constant, Declaration, Element, Identifier, Name, PrimitiveType, Span,
    TypeConstructor, WithAttributes, WithDocumentation, WithIdentifier, WithName, WithSpan,
};

/// An opaque identifier for a field in an AST model. Use the
/// `model[field_id]` syntax to resolve the id to an `ast::Field`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitsMemberId(pub(super) u32);

impl BitsMemberId {
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MIN: BitsMemberId = BitsMemberId(0);
    /// Used for range bounds when iterating over BTreeMaps.
    pub const MAX: BitsMemberId = BitsMemberId(u32::MAX);
}

impl std::ops::Index<BitsMemberId> for Bits {
    type Output = Rc<RefCell<BitsMember>>;

    fn index(&self, index: BitsMemberId) -> &Self::Output {
        &self.members[index.0 as usize]
    }
}

/// A enum member declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BitsMember {
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
    pub(crate) attributes: AttributeList,

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
pub struct Bits {
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
    pub(crate) members: Vec<Rc<RefCell<BitsMember>>>,

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

    // Set during compilation.
    pub(crate) mask: u64,

    // Set during compilation
    pub(crate) compiled: bool,
    pub(crate) compiling: bool,
    pub(crate) recursive: bool,
}

impl Into<Declaration> for Bits {
    fn into(self) -> Declaration {
        Declaration::Bits {
            decl: Rc::new(RefCell::new(self)),
        }
    }
}

impl Bits {
    pub fn iter_members(&self) -> impl ExactSizeIterator<Item = (BitsMemberId, &Rc<RefCell<BitsMember>>)> + Clone {
        self.members
            .iter()
            .enumerate()
            .map(|(idx, field)| (BitsMemberId(idx as u32), field))
    }
}

impl WithSpan for Bits {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl WithAttributes for Bits {
    fn attributes(&self) -> &AttributeList {
        &self.attributes
    }
}

impl WithDocumentation for Bits {
    fn documentation(&self) -> Option<&str> {
        self.documentation.as_ref().map(|doc| doc.text.as_str())
    }
}

impl WithName for Bits {
    fn name(&self) -> &Name {
        &self.name
    }
}

impl Decl for Bits {
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
