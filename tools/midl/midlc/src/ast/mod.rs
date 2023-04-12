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

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

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
pub use type_constructor::{
    LayoutConstraints, LayoutParameter, LayoutParameterList, PrimitiveSubtype, Type, TypeConstructor,
};

use crate::source_file::SourceId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Declaration {
    Import(Rc<RefCell<ImportDeclaration>>),
    Const(Rc<RefCell<Const>>),
    Struct(Rc<RefCell<Struct>>),
    Protocol(Rc<RefCell<Protocol>>),
    Builtin(Rc<RefCell<Builtin>>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Element {
    Builtin(Rc<RefCell<Builtin>>),
}

//#[derive(Debug)]
//pub enum Element {
//    Builtin(Builtin),
//    NewType(NewType)
//}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum BuiltinIdentity {
    #[default] bool,
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
    float32,
    float64,
    string,
    array,
    vector,
    r#box,
    client_end,
    server_end,
    optional,
    byte,
    transport_err,
    MAX,
    HEAD,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Name;

impl Name {
    fn new_intrinsic(library: &Library, name: &str) -> Self {
        Self {}
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Builtin {
    identity: BuiltinIdentity,
    name: Name,
}

impl Builtin {
    fn new(identity: BuiltinIdentity, name: Name) -> Self {
        Self { identity, name }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Declarations {
    pub structs: Vec<Declaration>,
    pub protocols: Vec<Declaration>,
    pub contants: Vec<Declaration>,
    pub builtins: Vec<Declaration>,
    pub imports: Vec<Declaration>,
}

impl Declarations {
    pub fn insert(&mut self, decl: Declaration) {
        match decl {
            Declaration::Struct(_) => self.structs.push(decl),
            Declaration::Protocol(_) => self.protocols.push(decl),
            Declaration::Const(_) => self.contants.push(decl),
            Declaration::Import(_) => self.imports.push(decl),
            Declaration::Builtin(_) => self.builtins.push(decl),
        }
    }
}

pub enum RegisterResult {
    Collision,
    Success,
    Duplicate,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LibraryRef {
    span: Span,
    dep_library: Rc<Library>,
}

// Per-file information about imports.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct PerFile {
    // References to dependencies, keyed by library name or by alias.
    refs: BTreeMap<CompoundIdentifier, Rc<LibraryRef>>,
    // Set containing ref->library for every ref in |refs|.
    libraries: BTreeSet<Rc<Library>>,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependencies {
    // All dependencies, in the order they were registered.
    refs: Vec<Rc<LibraryRef>>,

    // Maps a library's name to its LibraryRef.
    by_name: BTreeMap<CompoundIdentifier, Rc<LibraryRef>>,

    // Maps a library's filename to its PerFile.
    by_source_id: BTreeMap<SourceId, PerFile>,

    // All dependencies, including transitive dependencies.
    dependencies_aggregate: BTreeSet<Rc<Library>>,
}

impl Dependencies {
    // Registers a dependency to a library. The registration name is |maybe_alias|
    // if provided, otherwise the library's name. Afterwards, Dependencies::Lookup
    // will return |dep_library| given the registration name.
    pub fn register(
        &mut self,
        span: &Span,
        dep_library: Rc<Library>,
        alias: Option<CompoundIdentifier>,
    ) -> RegisterResult {
        self.refs.push(Rc::from(LibraryRef {
            span: span.clone(),
            dep_library: dep_library.clone(),
        }));

        let lib_ref = self.refs.last().unwrap();

        let name = if alias.is_some() {
            alias.unwrap()
        } else {
            dep_library.name.borrow().clone().unwrap()
        };

        log::debug!("register: {:?} as {}", dep_library.name.borrow(), name);

        let per_file = if self.by_source_id.contains_key(&span.source) {
            self.by_source_id.get_mut(&span.source).unwrap()
        } else {
            self.by_source_id.try_insert(span.source, PerFile::default()).unwrap()
        };

        log::debug!("per_file.refs: {:?}", per_file.refs.keys());

        if !per_file.libraries.insert(dep_library.clone()) {
            return RegisterResult::Duplicate;
        }

        if per_file.refs.insert(name, lib_ref.clone()).is_some() {
            return RegisterResult::Collision;
        }

        self.dependencies_aggregate.insert(dep_library);

        return RegisterResult::Success;
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
#[derive(Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Library {
    pub name: RefCell<Option<CompoundIdentifier>>,

    pub dependencies: RefCell<Dependencies>,

    /// All structs, enums, protocols, constants, bits and type aliase declarations.
    /// Populated by ParseStep, and then rewritten by ResolveStep.
    pub declarations: RefCell<Declarations>,

    pub elements: Vec<Element>,

    // Contains the same decls as `declarations`, but in a topologically sorted
    // order, i.e. later decls only depend on earlier ones. Populated by SortStep.
    pub declaration_order: Vec<Declaration>,

    pub arbitrary_name_span: RefCell<Option<Span>>,
}

impl Library {
    pub fn new_root() -> Self {
        let span = Span::empty();

        // TODO(fxbug.dev/67858): Because this library doesn't get compiled, we have
        // to simulate what AvailabilityStep would do (set the platform, inherit the
        // availabilities). Perhaps we could make the root library less special and
        // compile it as well. That would require addressing circularity issues.
        let mut library = Library::default();
        library.name = RefCell::new(Some(CompoundIdentifier {
            components: vec![Identifier {
                value: "".to_owned(),
                span,
            }],
            span,
        }));

        let mut insert = |name: &str, id: BuiltinIdentity| {
            let decl = Builtin::new(id, Name::new_intrinsic(&library, name));
            library
                .declarations
                .borrow_mut()
                .insert(Declaration::Builtin(Rc::new(RefCell::new(decl))));
        };

        // An assertion in Declarations::Insert ensures that these insertions
        // stays in sync with the order of Builtin::Identity.
        insert("bool", BuiltinIdentity::bool);
        insert("int8", BuiltinIdentity::i8);
        insert("int16", BuiltinIdentity::i16);
        insert("int32", BuiltinIdentity::i32);
        insert("int64", BuiltinIdentity::i64);
        insert("uint8", BuiltinIdentity::u8);
        insert("uint16", BuiltinIdentity::u16);
        insert("uint32", BuiltinIdentity::u32);
        insert("uint64", BuiltinIdentity::u64);
        insert("float32", BuiltinIdentity::float32);
        insert("float64", BuiltinIdentity::float64);
        insert("string", BuiltinIdentity::string);
        insert("box", BuiltinIdentity::r#box);
        insert("array", BuiltinIdentity::array);
        insert("vector", BuiltinIdentity::vector);
        insert("client_end", BuiltinIdentity::client_end);
        insert("server_end", BuiltinIdentity::server_end);
        insert("byte", BuiltinIdentity::byte);
        insert("TransportErr", BuiltinIdentity::transport_err);
        insert("optional", BuiltinIdentity::optional);
        insert("MAX", BuiltinIdentity::MAX);
        insert("HEAD", BuiltinIdentity::HEAD);

        // Simulate narrowing availabilities to maintain the invariant that they
        // always reach kNarrowed (except for the availability of `library`).
        // library->TraverseElements([](Element* element) {
        //     element->availability.Narrow(VersionRange(Version::Head(), Version::PosInf()));
        // });

        return library;
    }

    // want to traverse all elements which are simialr to declarations in this library
    pub fn traverse_elements(&self, visitor: &mut dyn FnMut(&Element)) {
        for element in &self.elements {
            visitor(element);
        }
    }

    // Iterate over all the top-level declarations.
}

/// An opaque identifier for a generator block in a library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstId(u32);

/// An opaque identifier for a builtin in a library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuiltinId(u32);

/// An opaque identifier for a generator block in a library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructId(pub u32);

/// An opaque identifier for a generator block in a library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolId(u32);

/// An opaque identifier for a datasource blèck in a library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImportId(u32);

/// An identifier for a top-level item in a library. Use the `schema[top_id]`
/// syntax to resolve the id to an `ast::Top`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationId {
    /// A composite type
    Import(ImportId),
    /// An enum declaration
    Const(ConstId),
    /// A generator block
    Protocol(ProtocolId),
    /// A datasource block
    Struct(StructId),
    /// A datasource block
    Builtin(BuiltinId),
}

impl std::ops::Index<DeclarationId> for Declarations {
    type Output = Declaration;

    fn index(&self, index: DeclarationId) -> &Self::Output {
        match index {
            DeclarationId::Protocol(ProtocolId(idx)) => &self.protocols[idx as usize],
            DeclarationId::Struct(StructId(idx)) => &self.structs[idx as usize],
            DeclarationId::Const(ConstId(idx)) => &self.contants[idx as usize],
            DeclarationId::Import(ImportId(idx)) => &self.imports[idx as usize],
            DeclarationId::Builtin(BuiltinId(idx)) => &self.builtins[idx as usize],
        }
    }
}

fn top_idx_to_top_id(top_idx: usize, decl: &Declaration) -> DeclarationId {
    match decl {
        Declaration::Protocol(_) => DeclarationId::Protocol(ProtocolId(top_idx as u32)),
        Declaration::Struct(_) => DeclarationId::Struct(StructId(top_idx as u32)),
        Declaration::Const(_) => DeclarationId::Const(ConstId(top_idx as u32)),
        Declaration::Import(_) => DeclarationId::Import(ImportId(top_idx as u32)),
        Declaration::Builtin(_) => DeclarationId::Builtin(BuiltinId(top_idx as u32)),
    }
}
