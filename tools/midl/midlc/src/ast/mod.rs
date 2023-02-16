mod ast;
mod attribute;
mod comment;
mod r#const;
mod identifier;
mod protocol;
mod span;
mod r#struct;
mod traits;

pub use ast::*;

pub use attribute::Attribute;
pub use comment::Comment;
pub use identifier::{CompoundIdentifier, Identifier};
pub use protocol::{Protocol, ProtocolMethod};
pub use r#const::{ConstDeclaration, Constant};
pub use r#struct::{Struct, StructMember};
pub use span::Span;
pub use traits::{WithAttributes, WithDocumentation, WithIdentifier, WithName, WithSpan};

#[derive(Debug)]
pub enum Declaration {
    Library(LibraryDeclaration),
    Import(ImportDeclaration),
    Const(ConstDeclaration),
    Struct(Struct),
    Protocol(Protocol),
}

/// AST representation of a MIDL schema.
///
/// This module is used internally to represent an AST. The AST's nodes can be used
/// during validation of a schema, especially when implementing custom attributes.
///
/// The AST is not validated, also fields and attributes are not resolved. Every node is
/// annotated with its location in the text representation.
/// Basically, the AST is an object oriented representation of the midl's text.
#[derive(Debug)]
pub struct SchamaAST {
    /// All structs, enums, protocols, constants, bits and type aliase declarations.
    pub declarations: Vec<Declaration>,
}

impl SchamaAST {
    /// Iterate over all the top-level items in the schema.
    pub fn iter_decls(&self) -> impl Iterator<Item = (TopId, &Declaration)> {
        self.declarations
            .iter()
            .enumerate()
            .map(|(top_idx, top)| (top_idx_to_top_id(top_idx, top), top))
    }
}

/// An opaque identifier for a generator block in a schema AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstId(u32);

/// An opaque identifier for a generator block in a schema AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructId(u32);

/// An opaque identifier for a generator block in a schema AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LibraryId(u32);

/// An opaque identifier for a generator block in a schema AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolId(u32);

/// An opaque identifier for a datasource blèck in a schema AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImportId(u32);

/// An identifier for a top-level item in a schema AST. Use the `schema[top_id]`
/// syntax to resolve the id to an `ast::Top`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TopId {
    /// A composite type
    Import(ImportId),
    /// A model declaration
    Library(LibraryId),
    /// An enum declaration
    Const(ConstId),
    /// A generator block
    Protocol(ProtocolId),
    /// A datasource block
    Struct(StructId),
}

impl std::ops::Index<TopId> for SchamaAST {
    type Output = Declaration;

    fn index(&self, index: TopId) -> &Self::Output {
        let idx = match index {
            TopId::Protocol(ProtocolId(idx)) => idx,
            TopId::Struct(StructId(idx)) => idx,
            TopId::Const(ConstId(idx)) => idx,
            TopId::Library(LibraryId(idx)) => idx,
            TopId::Import(ImportId(idx)) => idx,
        };

        &self.declarations[idx as usize]
    }
}

fn top_idx_to_top_id(top_idx: usize, decl: &Declaration) -> TopId {
    match decl {
        Declaration::Protocol(_) => TopId::Protocol(ProtocolId(top_idx as u32)),
        Declaration::Struct(_) => TopId::Struct(StructId(top_idx as u32)),
        Declaration::Const(_) => TopId::Const(ConstId(top_idx as u32)),
        Declaration::Import(_) => TopId::Import(ImportId(top_idx as u32)),
        Declaration::Library(_) => TopId::Library(LibraryId(top_idx as u32)),
    }
}
