mod alias;
mod ast;
mod attributes;
mod comment;
mod r#const;
mod identifier;
mod name;
mod properties;
mod protocol;
mod reference;
mod resource;
mod span;
mod r#struct;
mod traits;
mod type_constructor;
mod versioning_types;
mod constraints;

use multimap::MultiMap;
use std::cell::{OnceCell, RefCell};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

pub use ast::*;

pub use alias::Alias;
pub use attributes::{Attribute, AttributeArg, AttributeList};
pub use comment::Comment;
pub use identifier::{CompoundIdentifier, Identifier};
pub use name::{name_flat_name, Name};
pub use properties::{Nullability, Resourceness, Strictness};
pub use protocol::{Protocol, ProtocolMethod};
pub use r#const::{Const, Constant, ConstantTrait, ConstantValue, IdentifierConstant, LiteralConstant};
pub use r#struct::{Struct, StructMember};
pub use reference::{Reference, ReferenceKey, ReferenceState, Target};
pub use resource::{Resource, ResourceProperty};
pub use span::Span;
pub use traits::{WithAttributes, WithDocumentation, WithIdentifier, WithName, WithSpan};
pub use constraints::VectorConstraints;
pub use type_constructor::{
    InternalSubtype, InternalType, LayoutConstraints, LayoutParameter, LayoutParameterList, LiteralLayoutParameter,
    PrimitiveSubtype, PrimitiveType, StringType, Type, TypeConstructor, TypeLayoutParameter,
};
pub use versioning_types::{Availability, AvailabilityInitArgs, Platform, Version, VersionRange, VersionSelection};

use crate::source_file::SourceId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Element {
    // declarations
    Bits,
    Enum,
    Resource,
    Alias { inner: Rc<RefCell<Alias>> },
    Builtin { inner: Rc<RefCell<Builtin>> },
    Struct { inner: Rc<RefCell<Struct>> },
    Protocol { inner: Rc<RefCell<Protocol>> },
    Const { inner: Rc<RefCell<Const>> },
    NewType,
    Table,
    Union,
    Overlay,

    // Elements that are not declarations
    StructMember { inner: Rc<StructMember> },
    ProtocolMethod { inner: Rc<ProtocolMethod> },
    TableMember,
    UnionMember,
    EnumMember,
}

impl Element {
    fn is_decl(&self) -> bool {
        match self {
            Element::Const { .. } | Element::Struct { .. } | Element::Protocol { .. } | Element::Builtin { .. } => true,
            _ => false,
        }
    }

    pub fn as_decl(&self) -> Option<Declaration> {
        match self {
            Element::Const { inner } => Some(Declaration::Const { decl: inner.clone() }),
            Element::Builtin { inner } => Some(Declaration::Builtin { decl: inner.clone() }),
            Element::Struct { inner } => Some(Declaration::Struct { decl: inner.clone() }),
            Element::Protocol { inner } => Some(Declaration::Protocol { decl: inner.clone() }),
            _ => None,
        }
    }

    pub fn availability(&self) -> Availability {
        let mut availability = Availability::unbounded();
        availability.narrow(VersionRange::new(Version::neg_inf(), Version::pos_inf()));

        availability
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Declaration {
    Bits,
    Enum,
    NewType,
    Table,
    Union,
    Overlay,
    Resource { decl: Rc<RefCell<Resource>> },
    Alias { decl: Rc<RefCell<Alias>> },
    Const { decl: Rc<RefCell<Const>> },
    Struct { decl: Rc<RefCell<Struct>> },
    Protocol { decl: Rc<RefCell<Protocol>> },
    Builtin { decl: Rc<RefCell<Builtin>> },
}

impl Into<Element> for Declaration {
    fn into(self) -> Element {
        match self {
            Declaration::Const { ref decl } => Element::Const { inner: decl.clone() },
            Declaration::Struct { ref decl } => Element::Struct { inner: decl.clone() },
            Declaration::Protocol { ref decl } => Element::Protocol { inner: decl.clone() },
            Declaration::Builtin { ref decl } => Element::Builtin { inner: decl.clone() },
            Declaration::Bits => Element::Bits,
            Declaration::Enum => Element::Enum,
            Declaration::Alias { ref decl } => Element::Alias { inner: decl.clone() },
            Declaration::Resource { .. } => Element::Resource,
            Declaration::NewType => Element::NewType,
            Declaration::Table => Element::Table,
            Declaration::Union => Element::Union,
            Declaration::Overlay => Element::Overlay,
        }
    }
}

impl Into<Element> for Rc<Declaration> {
    fn into(self) -> Element {
        match &*self {
            Declaration::Const { ref decl } => Element::Const { inner: decl.clone() },
            Declaration::Struct { ref decl } => Element::Struct { inner: decl.clone() },
            Declaration::Protocol { ref decl } => Element::Protocol { inner: decl.clone() },
            Declaration::Builtin { ref decl } => Element::Builtin { inner: decl.clone() },
            Declaration::Alias { ref decl } => Element::Alias { inner: decl.clone() },
            Declaration::Bits => Element::Bits,
            Declaration::Enum => Element::Enum,
            Declaration::Resource { .. } => Element::Resource,
            Declaration::NewType => Element::NewType,
            Declaration::Table => Element::Table,
            Declaration::Union => Element::Union,
            Declaration::Overlay => Element::Overlay,
        }
    }
}

impl Declaration {
    pub fn name(&self) -> Name {
        match self {
            Declaration::Struct { decl } => decl.borrow().name().to_owned(),
            Declaration::Protocol { decl } => decl.borrow().name().to_owned(),
            Declaration::Const { decl } => decl.borrow().name().to_owned(),
            Declaration::Builtin { decl } => decl.borrow().name().to_owned(),
            Declaration::Alias { decl } => decl.borrow().name().to_owned(),
            Declaration::Resource { decl } => decl.borrow().name().to_owned(),
            Declaration::Bits => todo!(),
            Declaration::Enum => todo!(),
            Declaration::NewType => todo!(),
            Declaration::Table => todo!(),
            Declaration::Union => todo!(),
            Declaration::Overlay => todo!(),
        }
    }

    pub fn availability(&self) -> Availability {
        let mut availability = Availability::unbounded();
        availability.narrow(VersionRange::new(Version::neg_inf(), Version::pos_inf()));
        availability
    }

    // Runs a function on every member of the decl, if it has any. Note that
    // unlike Library::traverse_elements, it does not call `visit(self)`.
    pub(crate) fn for_each_member(&self, visitor: &mut dyn FnMut(Element)) {
        match self {
            Declaration::Struct { decl } => {
                for (_, member) in decl.borrow().iter_members() {
                    visitor(member.clone());
                }
            }
            Declaration::Protocol { decl } => {
                for (_, method) in decl.borrow().iter_methods() {
                    visitor(Element::ProtocolMethod { inner: method.clone() });
                }
            }
            Declaration::Const { .. } => {}
            Declaration::Builtin { .. } => {}
            Declaration::Alias { .. } => {}
            Declaration::Resource { .. } => {}
            Declaration::Bits => todo!(),
            Declaration::Enum => todo!(),
            Declaration::NewType => todo!(),
            Declaration::Table => todo!(),
            Declaration::Union => todo!(),
            Declaration::Overlay => todo!(),
        };
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BuiltinIdentity {
    // Layouts (primitive)
    #[default]
    bool,
    int8,
    int16,
    int32,
    int64,
    uint8,
    uint16,
    uint32,
    uint64,
    float32,
    float64,
    // Layouts (other)
    String,
    // Layouts (templated)
    StringArray,
    Array,
    Vector,
    Box,
    ClientEnd,
    ServerEnd,
    // Layouts (aliases)
    Byte,
    // Layouts (internal)
    FrameworkErr,
    // Constraints
    Optional,
    MAX,
    HEAD,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Builtin {
    pub id: BuiltinIdentity,
    pub name: Name,
}

impl Builtin {
    fn new(id: BuiltinIdentity, name: Name) -> Self {
        Self { id, name }
    }

    pub fn is_internal(&self) -> bool {
        match self.id {
            BuiltinIdentity::FrameworkErr => true,
            _ => false,
        }
    }
}

impl WithName for Builtin {
    fn name(&self) -> &Name {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declarations {
    pub structs: Vec<Declaration>,
    pub protocols: Vec<Declaration>,
    pub builtins: Vec<Declaration>,
    pub consts: Vec<Declaration>,
    pub resources: Vec<Declaration>,
    pub imports: Vec<Declaration>,

    pub all: MultiMap<String, Declaration>,
}

impl Default for Declarations {
    fn default() -> Self {
        Declarations {
            consts: vec![],
            resources: vec![],
            structs: vec![],
            protocols: vec![],
            builtins: vec![],
            imports: vec![],
            all: MultiMap::new(),
        }
    }
}

impl PartialOrd for Declarations {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Declarations {
    fn cmp(&self, other: &Self) -> Ordering {
        self.structs.cmp(&other.structs)
    }
}

fn store_decl(decl: Declaration, all: &mut MultiMap<String, Declaration>, decls: &mut Vec<Declaration>) {
    all.insert(decl.name().decl_name(), decl.clone());
    decls.push(decl);
}

impl Declarations {
    pub fn insert(&mut self, decl: Declaration) {
        let all_ref = &mut self.all;

        match decl {
            Declaration::Struct { .. } => store_decl(decl, all_ref, &mut self.structs),
            Declaration::Protocol { .. } => store_decl(decl, all_ref, &mut self.protocols),
            Declaration::Const { .. } => store_decl(decl, all_ref, &mut self.consts),
            Declaration::Alias { .. } => store_decl(decl, all_ref, &mut self.builtins),
            Declaration::Builtin { .. } => store_decl(decl, all_ref, &mut self.builtins),
            Declaration::Resource { .. } => store_decl(decl, all_ref, &mut self.resources),
            Declaration::Bits => todo!(),
            Declaration::Enum => todo!(),
            Declaration::NewType => todo!(),
            Declaration::Table => todo!(),
            Declaration::Union => todo!(),
            Declaration::Overlay => todo!(),
        }
    }

    pub(crate) fn lookup_builtin(identity: BuiltinIdentity) {}
}

pub enum RegisterResult {
    Collision,
    Success,
    Duplicate,
}

/// A reference to a library, derived from a "using" statement.
#[derive(Debug, PartialEq, Eq)]
struct LibraryRef {
    span: Span,
    library: Rc<Library>,
    used: RefCell<bool>,
}

// Per-source information about imports.
#[derive(Debug, Default, PartialEq, Eq)]
struct PerSource {
    // References to dependencies, keyed by library name or by alias.
    refs: BTreeMap<Vec<String>, Rc<LibraryRef>>,
    // Set containing ref->library for every ref in |refs|.
    libraries: BTreeSet<Rc<Library>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Dependencies {
    // All dependencies, in the order they were registered.
    refs: Vec<Rc<LibraryRef>>,

    // Maps a library's name to its LibraryRef.
    by_name: BTreeMap<CompoundIdentifier, Rc<LibraryRef>>,

    // Maps a library's filename to its PerSource.
    by_source_id: BTreeMap<SourceId, PerSource>,

    // All dependencies, including transitive dependencies.
    dependencies_aggregate: BTreeSet<Rc<Library>>,
}

impl Dependencies {
    // Registers a dependency to a library. The registration name is |maybe_alias|
    // if provided, otherwise the library's name. Afterwards, Dependencies::Lookup
    // will return |dep_library| given the registration name.
    pub fn register(&mut self, span: &Span, dep_library: Rc<Library>, alias: Option<Vec<String>>) -> RegisterResult {
        self.refs.push(Rc::from(LibraryRef {
            span: span.clone(),
            library: dep_library.clone(),
            used: RefCell::from(false),
        }));

        let lib_ref = self.refs.last().unwrap();

        let name = if alias.is_some() {
            alias.unwrap()
        } else {
            dep_library.name.get().expect("expected initialized name").clone()
        };

        log::debug!("register: {:?} as {:?}", dep_library.name, name);

        let per_file = if self.by_source_id.contains_key(&span.source) {
            self.by_source_id.get_mut(&span.source).unwrap()
        } else {
            self.by_source_id.try_insert(span.source, PerSource::default()).unwrap()
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

    /// Looks up a dependency by filename (within the importing library, since
    /// "using" statements are file-scoped) and name (of the imported library).
    /// Also marks the library as used. Returns null if no library is found.
    pub fn lookup_and_mark_used(&self, source: SourceId, name: &Vec<String>) -> Option<Rc<Library>> {
        let iter1 = self.by_source_id.iter().find(|&(si, _)| *si == source);
        if iter1.is_none() {
            return None;
        }

        let (_, per_file) = iter1.unwrap();

        let iter2 = per_file.refs.iter().find(|&(n, _)| n == name);
        if iter2.is_none() {
            return None;
        }

        let (_, reference) = iter2.unwrap();

        reference.used.replace(true);
        Some(reference.library.clone())
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
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Library {
    pub name: OnceCell<Vec<String>>,

    pub dependencies: RefCell<Dependencies>,

    /// All structs, enums, protocols, constants, bits and type aliase declarations.
    /// Populated by ParseStep, and then rewritten by ResolveStep.
    pub declarations: RefCell<Declarations>,

    // Contains the same decls as `declarations`, but in a topologically sorted
    // order, i.e. later decls only depend on earlier ones. Populated by SortStep.
    pub declaration_order: Vec<Declaration>,

    pub arbitrary_name_span: RefCell<Option<Span>>,

    // Set during AvailabilityStep.
    pub platform: Option<Platform>,
}

impl PartialOrd for Library {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Library {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.get().cmp(&other.name.get())
    }
}

impl Library {
    pub fn new_root() -> Rc<Self> {
        // TODO(fxbug.dev/67858): Because this library doesn't get compiled, we have
        // to simulate what AvailabilityStep would do (set the platform, inherit the
        // availabilities). Perhaps we could make the root library less special and
        // compile it as well. That would require addressing circularity issues.
        let mut library = Library::default();
        library.name = OnceCell::from(vec!["midl".to_owned()]);

        let library = Rc::new(library);

        let insert = |name: &str, id: BuiltinIdentity| {
            let decl = Builtin::new(
                id,
                Name::create_intrinsic(library.clone(), name), /*Name::new_intrinsic(&library, name)*/
            );

            library.declarations.borrow_mut().insert(Declaration::Builtin {
                decl: Rc::new(RefCell::new(decl)),
            });
        };

        // An assertion in Declarations::Insert ensures that these insertions
        // stays in sync with the order of Builtin::Identity.
        insert("bool", BuiltinIdentity::bool);
        insert("int8", BuiltinIdentity::int8);
        insert("int16", BuiltinIdentity::int16);
        insert("int32", BuiltinIdentity::int32);
        insert("int64", BuiltinIdentity::int64);
        insert("uint8", BuiltinIdentity::uint8);
        insert("uint16", BuiltinIdentity::uint16);
        insert("uint32", BuiltinIdentity::uint32);
        insert("uint64", BuiltinIdentity::uint64);
        insert("float32", BuiltinIdentity::float32);
        insert("float64", BuiltinIdentity::float64);
        insert("string", BuiltinIdentity::String);
        insert("box", BuiltinIdentity::Box);
        insert("array", BuiltinIdentity::Array);
        insert("vector", BuiltinIdentity::Vector);
        insert("client_end", BuiltinIdentity::ClientEnd);
        insert("server_end", BuiltinIdentity::ServerEnd);
        insert("byte", BuiltinIdentity::Byte);
        insert("FrameworkErr", BuiltinIdentity::FrameworkErr);
        insert("optional", BuiltinIdentity::Optional);
        insert("MAX", BuiltinIdentity::MAX);
        insert("HEAD", BuiltinIdentity::HEAD);

        // Simulate narrowing availabilities to maintain the invariant that they
        // always reach kNarrowed (except for the availability of `library`).
        // library->TraverseElements([](Element* element) {
        //     element->availability.Narrow(VersionRange(Version::Head(), Version::PosInf()));
        // });

        return library;
    }

    // want to traverse all elements which are simialar to declarations in this library
    pub fn traverse_elements(&self, visitor: &mut dyn FnMut(Element)) {
        for (_, decl) in self.declarations.borrow().all.iter() {
            visitor(decl.clone().into());
            decl.for_each_member(visitor);
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
            DeclarationId::Const(ConstId(idx)) => &self.consts[idx as usize],
            DeclarationId::Import(ImportId(idx)) => &self.imports[idx as usize],
            DeclarationId::Builtin(BuiltinId(idx)) => &self.builtins[idx as usize],
        }
    }
}

fn top_idx_to_top_id(top_idx: usize, decl: &Declaration) -> DeclarationId {
    match decl {
        Declaration::Protocol { .. } => DeclarationId::Protocol(ProtocolId(top_idx as u32)),
        Declaration::Struct { .. } => DeclarationId::Struct(StructId(top_idx as u32)),
        Declaration::Const { .. } => DeclarationId::Const(ConstId(top_idx as u32)),
        Declaration::Builtin { .. } => DeclarationId::Builtin(BuiltinId(top_idx as u32)),
        Declaration::Bits => todo!(),
        Declaration::Enum => todo!(),
        Declaration::Alias { .. } => todo!(),
        Declaration::Resource { .. } => todo!(),
        Declaration::NewType => todo!(),
        Declaration::Table => todo!(),
        Declaration::Union => todo!(),
        Declaration::Overlay => todo!(),
    }
}
