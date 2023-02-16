mod ast;
mod attribute;
mod comment;
mod identifier;
mod span;
mod r#struct;
mod protocol;
mod traits;

pub use ast::*;

pub use attribute::Attribute;
pub use comment::Comment;
pub use identifier::{CompoundIdentifier, Identifier};
pub use protocol::{Protocol, ProtocolMethod};
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
