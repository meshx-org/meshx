mod ast;
mod attribute;
mod comment;
mod r#const;
mod identifier;
mod protocol;
mod reference;
mod span;
mod r#struct;
mod traits;
mod type_constructor;

pub use ast::*;

pub use attribute::Attribute;
pub use comment::Comment;
pub use identifier::{CompoundIdentifier, Identifier};
pub use protocol::{Protocol, ProtocolMethod};
pub use r#const::{Const, Constant};
pub use r#struct::{Struct, StructMember};
pub use reference::Reference;
pub use span::Span;
pub use traits::{WithAttributes, WithDocumentation, WithIdentifier, WithName, WithSpan};
pub use type_constructor::{PrimitiveSubtype, Type};

#[derive(Debug)]
pub enum Declaration {
    Import(ImportDeclaration),
    Const(Const),
    Struct(Struct),
    Protocol(Protocol),
}

//#[derive(Debug)]
//pub enum Element {
//    Builtin(Builtin),
//    NewType(NewType)
//}

#[derive(Debug, Default)]
pub struct Declarations {
    pub structs: Vec<Declaration>,
    pub protocols: Vec<Declaration>,
    pub contants: Vec<Declaration>,

    pub imports: Vec<Declaration>,
}

impl Declarations {
    fn iter(&self) -> impl Iterator<Item = &Declaration> {
        self.structs
            .iter()
            .chain(self.protocols.iter())
            .chain(self.contants.iter())
            .chain(self.imports.iter())
    }

    pub fn insert(&mut self, decl: Declaration) {
        match decl {
            Declaration::Struct(_) => self.structs.push(decl),
            Declaration::Protocol(_) => self.protocols.push(decl),
            Declaration::Const(_) => self.contants.push(decl),
            Declaration::Import(_) => self.imports.push(decl),
        }
    }
}

pub enum RegisterResult {
    Collision,
    Success,
    Duplicate,
}

#[derive(Debug, Default)]
pub struct Dependencies;

impl Dependencies {
    // Registers a dependency to a library. The registration name is |maybe_alias|
    // if provided, otherwise the library's name. Afterwards, Dependencies::Lookup
    // will return |dep_library| given the registration name.
    fn register(span: &Span, dep_library: &Library, maybe_alias: Identifier) -> RegisterResult {
        // let filename = span.source_file().filename();
        RegisterResult::Success
    }
}

/// AST representation of a MIDL library.
///
/// This module is used internally to represent an AST. The AST's nodes can be used
/// during validation of a library, especially when implementing custom attributes.
///
/// The AST is not validated, also fields and attributes are not resolved. Every node is
/// annotated with its location in the text representation.
/// Basically, the AST is an object oriented representation of the midl's text.
#[derive(Debug)]
pub struct Library {
    pub name: CompoundIdentifier,

    pub dependencies: Dependencies,

    /// All structs, enums, protocols, constants, bits and type aliase declarations.
    /// Populated by ParseStep, and then rewritten by ResolveStep.
    pub declarations: Declarations,

    // Contains the same decls as `declarations`, but in a topologically sorted
    // order, i.e. later decls only depend on earlier ones. Populated by SortStep.
    pub declaration_order: Vec<Declaration>,
}

impl Library {
    pub fn new_root() -> Self {
        Library {
            name: CompoundIdentifier { components: vec![] },
            dependencies: Dependencies::default(),
            declarations: Declarations::default(),
            declaration_order: Vec::new(),
        }
    }

    /// Iterate over all the top-level items in the library.
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
    /// An enum declaration
    Const(ConstId),
    /// A generator block
    Protocol(ProtocolId),
    /// A datasource block
    Struct(StructId),
}

impl std::ops::Index<TopId> for Declarations {
    type Output = Declaration;

    fn index(&self, index: TopId) -> &Self::Output {
        match index {
            TopId::Protocol(ProtocolId(idx)) => &self.protocols[idx as usize],
            TopId::Struct(StructId(idx)) => &self.structs[idx as usize],
            TopId::Const(ConstId(idx)) => &self.contants[idx as usize],
            TopId::Import(ImportId(idx)) => &self.imports[idx as usize],
        }
    }
}

fn top_idx_to_top_id(top_idx: usize, decl: &Declaration) -> TopId {
    match decl {
        Declaration::Protocol(_) => TopId::Protocol(ProtocolId(top_idx as u32)),
        Declaration::Struct(_) => TopId::Struct(StructId(top_idx as u32)),
        Declaration::Const(_) => TopId::Const(ConstId(top_idx as u32)),
        Declaration::Import(_) => TopId::Import(ImportId(top_idx as u32)),
    }
}
