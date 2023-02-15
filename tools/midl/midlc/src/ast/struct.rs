use super::{Attribute, Comment, Identifier, Span};

/// A struct member declaration.
#[derive(Debug, Clone)]
pub struct StructMember;

/// A struct declaration.
#[derive(Debug, Clone)]
pub struct Struct {
    /// The name of the struct.
    ///
    /// ```ignore
    /// type Foo = struct { .. }
    ///      ^^^
    /// ```
    pub(crate) name: Identifier,

    /// The members of the struct.
    ///
    /// ```ignore
    /// type Foo = struct {
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
    /// type Foo = struct {
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
    /// type Foo = struct {
    ///   id    :u32   @id
    ///   field :string
    /// }
    /// ```
    pub(crate) documentation: Option<Comment>,

    /// The location of this struct in the text representation.
    pub(crate) span: Span,
}
