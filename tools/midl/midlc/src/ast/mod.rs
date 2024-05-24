mod alias;
mod ast;
mod attributes;
mod bits;
mod comment;
mod r#const;
mod constraints;
mod r#enum;
mod identifier;
mod name;
mod properties;
mod protocol;
mod reference;
mod resource;
mod span;
mod r#struct;
mod table;
mod traits;
mod type_constructor;
mod union;
mod versioning_types;

use multimap::MultiMap;
use std::cell::{OnceCell, RefCell};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

pub use ast::*;

use crate::source_file::SourceId;
pub use alias::Alias;
pub use attributes::{Attribute, AttributeArg, AttributeList};
pub use bits::{Bits, BitsMember};
pub use comment::Comment;
pub use constraints::{
    ConstraintKind, HandleConstraints, MergeConstraints, NullabilityTrait, ProtocolTrait, ResolveAndMerge,
    TransportSideConstraints, VectorConstraints,
};
pub use identifier::{CompoundIdentifier, Identifier};
pub use name::{name_flat_name, Name, NameProvenance, NamingContext};
pub use properties::{Nullability, Resourceness, Strictness};
pub use protocol::{Protocol, ProtocolMethod};
pub use r#const::{
    BinaryOperatorConstant, Const, Constant, ConstantOp, ConstantTrait, ConstantValue, ConstantValueKind,
    IdentifierConstant, LiteralConstant,
};
pub use r#enum::{Enum, EnumMember};
pub use r#struct::{Struct, StructMember};
pub use reference::{Reference, ReferenceKey, ReferenceState, Target};
pub use resource::{Resource, ResourceProperty};
pub use span::Span;
pub use table::{Table, TableMember, TableMemberUsed};
pub use traits::{Decl, WithAttributes, WithDocumentation, WithIdentifier, WithName, WithSpan};
pub use type_constructor::{
    ArrayType, BoxType, HandleType, IdentifierType, InternalSubtype, InternalType, LayoutConstraints, LayoutParameter,
    LayoutParameterList, LiteralLayoutParameter, PrimitiveSubtype, PrimitiveType, StringType, TransportSide,
    TransportSideType, Type, TypeConstructor, TypeLayoutParameter, UntypedNumericType, VectorType,
};
pub use union::{Union, UnionMember, UnionMemberUsed};
pub use versioning_types::{Availability, AvailabilityInitArgs, Platform, Version, VersionRange, VersionSelection};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Element {
    // declarations
    Enum { inner: Rc<RefCell<Enum>> },
    Alias { inner: Rc<RefCell<Alias>> },
    Builtin { inner: Rc<RefCell<Builtin>> },
    Struct { inner: Rc<RefCell<Struct>> },
    Union { inner: Rc<RefCell<Union>> },
    Protocol { inner: Rc<RefCell<Protocol>> },
    Const { inner: Rc<RefCell<Const>> },
    Table { inner: Rc<RefCell<Table>> },
    Bits { inner: Rc<RefCell<Bits>> },
    Resource { inner: Rc<RefCell<Resource>> },
    NewType,
    Overlay,

    // Elements that are not declarations
    StructMember { inner: Rc<RefCell<StructMember>> },
    EnumMember { inner: Rc<RefCell<EnumMember>> },
    BitsMember { inner: Rc<RefCell<BitsMember>> },
    UnionMember { inner: Rc<RefCell<UnionMember>> },
    TableMember { inner: Rc<RefCell<TableMember>> },
    ProtocolMethod { inner: Rc<RefCell<ProtocolMethod>> },
    ResourceProperty { inner: Rc<RefCell<ResourceProperty>> },

    Library,
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
            Element::Enum { inner } => Some(Declaration::Enum { decl: inner.clone() }),
            Element::Union { inner } => Some(Declaration::Union { decl: inner.clone() }),
            Element::Table { inner } => Some(Declaration::Table { decl: inner.clone() }),
            Element::Alias { inner } => Some(Declaration::Alias { decl: inner.clone() }),
            Element::Bits { inner } => Some(Declaration::Bits { decl: inner.clone() }),
            Element::Resource { inner } => Some(Declaration::Resource { decl: inner.clone() }),
            Element::Protocol { inner } => Some(Declaration::Protocol { decl: inner.clone() }),
            _ => todo!("{:?}", self),
        }
    }

    pub fn get_name(&self) -> Option<String> {
        match self {
            Element::Const { inner } => Some(inner.borrow().name.clone().decl_name()),
            Element::Builtin { inner } => Some(inner.borrow().name.clone().decl_name()),
            Element::Struct { inner } => Some(inner.borrow().name.clone().decl_name()),
            Element::Enum { inner } => Some(inner.borrow().name.clone().decl_name()),
            Element::Bits { inner } => Some(inner.borrow().name.clone().decl_name()),
            Element::Union { inner } => Some(inner.borrow().name.clone().decl_name()),
            Element::Table { inner } => Some(inner.borrow().name.clone().decl_name()),
            Element::Protocol { inner } => Some(inner.borrow().name.clone().decl_name()),
            Element::Alias { inner } => Some(inner.borrow().name.clone().decl_name()),
            Element::Resource { inner } => Some(inner.borrow().name.clone().decl_name()),
            Element::BitsMember { inner } => {
                return Some(inner.borrow().name.data.clone());
            }
            Element::EnumMember { inner } => {
                return Some(inner.borrow().name.data.clone());
            }

            _ => todo!("{:?}", self),
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
    NewType,
    Overlay,
    Table { decl: Rc<RefCell<Table>> },
    Union { decl: Rc<RefCell<Union>> },
    Bits { decl: Rc<RefCell<Bits>> },
    Enum { decl: Rc<RefCell<Enum>> },
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
            Declaration::Union { ref decl } => Element::Union { inner: decl.clone() },
            Declaration::Enum { ref decl } => Element::Enum { inner: decl.clone() },
            Declaration::Table { ref decl } => Element::Table { inner: decl.clone() },
            Declaration::Alias { ref decl } => Element::Alias { inner: decl.clone() },
            Declaration::Bits { ref decl } => Element::Bits { inner: decl.clone() },
            Declaration::Resource { ref decl } => Element::Resource { inner: decl.clone() },

            Declaration::NewType => Element::NewType,
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
            Declaration::Enum { ref decl } => Element::Enum { inner: decl.clone() },
            Declaration::Union { ref decl } => Element::Union { inner: decl.clone() },
            Declaration::Table { ref decl } => Element::Table { inner: decl.clone() },
            Declaration::Bits { ref decl } => Element::Bits { inner: decl.clone() },
            Declaration::Resource { ref decl } => Element::Resource { inner: decl.clone() },

            Declaration::NewType => Element::NewType,
            Declaration::Overlay => Element::Overlay,
        }
    }
}

impl Declaration {
    pub fn name(&self) -> Name {
        match self {
            Declaration::Struct { decl } => decl.borrow().name().to_owned(),
            Declaration::Enum { decl } => decl.borrow().name().to_owned(),
            Declaration::Protocol { decl } => decl.borrow().name().to_owned(),
            Declaration::Const { decl } => decl.borrow().name().to_owned(),
            Declaration::Builtin { decl } => decl.borrow().name().to_owned(),
            Declaration::Alias { decl } => decl.borrow().name().to_owned(),
            Declaration::Resource { decl } => decl.borrow().name().to_owned(),
            Declaration::Union { decl } => decl.borrow().name().to_owned(),
            Declaration::Table { decl } => decl.borrow().name().to_owned(),
            Declaration::Bits { decl } => decl.borrow().name().to_owned(),
            Declaration::NewType => todo!(),
            Declaration::Overlay => todo!(),
        }
    }

    pub(crate) fn set_recursive(&self, val: bool) -> bool {
        match self {
            Declaration::Enum { decl } => {
                decl.borrow_mut().recursive = val;
                true
            }
            Declaration::Struct { decl } => {
                decl.borrow_mut().recursive = val;
                true
            }
            Declaration::Union { .. } => todo!(),
            Declaration::Table { .. } => todo!(),
            Declaration::Bits { .. } => todo!(),
            Declaration::Overlay => todo!(),
            Declaration::NewType => todo!(),
            _ => panic!("not type decl"),
        }
    }

    pub(crate) fn as_decl(&self) -> Rc<RefCell<dyn Decl>> {
        match self {
            Declaration::Enum { decl } => decl.clone() as Rc<RefCell<dyn Decl>>,
            Declaration::Struct { decl } => decl.clone() as Rc<RefCell<dyn Decl>>,
            Declaration::Union { decl } => decl.clone() as Rc<RefCell<dyn Decl>>,
            Declaration::Table { decl } => decl.clone() as Rc<RefCell<dyn Decl>>,
            Declaration::Const { decl } => decl.clone() as Rc<RefCell<dyn Decl>>,
            Declaration::Bits { decl } => decl.clone() as Rc<RefCell<dyn Decl>>,
            Declaration::Overlay => todo!(),
            Declaration::NewType => todo!(),
            Declaration::Resource { .. } => todo!(),
            Declaration::Alias { .. } => todo!(),
            Declaration::Protocol { .. } => todo!(),
            Declaration::Builtin { .. } => todo!(),
            // _ => panic!("not type decl"),
        }
    }

    pub(crate) fn compiling(&self) -> bool {
        match self {
            Declaration::Union { decl } => decl.borrow().compiling,
            Declaration::Enum { decl } => decl.borrow().compiling,
            Declaration::Const { decl } => decl.borrow().compiling,
            Declaration::Struct { decl } => decl.borrow().compiling,
            Declaration::Table { decl } => decl.borrow().compiling,
            Declaration::Alias { decl } => decl.borrow().compiling,
            Declaration::Protocol { decl } => decl.borrow().compiling,
            Declaration::Bits { decl } => decl.borrow().compiling,
            Declaration::Resource { .. } => todo!(),
            Declaration::Builtin { .. } => todo!(),
            Declaration::NewType => todo!(),
            Declaration::Overlay => todo!(),
        }
    }

    pub(crate) fn compiled(&self) -> bool {
        match self {
            Declaration::Union { decl } => decl.borrow().compiled,
            Declaration::Enum { decl } => decl.borrow().compiled,
            Declaration::Const { decl } => decl.borrow().compiled,
            Declaration::Struct { decl } => decl.borrow().compiled,
            Declaration::Table { decl } => decl.borrow().compiled,
            Declaration::Alias { decl } => decl.borrow().compiled,
            Declaration::Protocol { decl } => decl.borrow().compiled,
            Declaration::Bits { decl } => decl.borrow().compiled,
            Declaration::Resource { decl } => decl.borrow().compiled,
            Declaration::Builtin { .. } => todo!(),
            Declaration::NewType => todo!(),
            Declaration::Overlay => todo!(),
        }
    }

    pub(crate) fn set_compiling(&mut self, val: bool) {
        match self {
            Declaration::Union { decl } => decl.borrow_mut().compiling = val,
            Declaration::Enum { decl } => decl.borrow_mut().compiling = val,
            Declaration::Const { decl } => decl.borrow_mut().compiling = val,
            Declaration::Struct { decl } => decl.borrow_mut().compiling = val,
            Declaration::Resource { decl } => decl.borrow_mut().compiling = val,
            Declaration::Protocol { decl } => decl.borrow_mut().compiling = val,
            Declaration::Alias { decl } => decl.borrow_mut().compiling = val,
            Declaration::Table { decl } => decl.borrow_mut().compiling = val,
            Declaration::Bits { decl } => decl.borrow_mut().compiling = val,
            Declaration::Builtin { .. } => todo!(),
            Declaration::NewType => todo!(),
            Declaration::Overlay => todo!(),
        }
    }

    pub(crate) fn set_compiled(&mut self, val: bool) {
        match self {
            Declaration::Union { decl } => decl.borrow_mut().compiled = val,
            Declaration::Enum { decl } => decl.borrow_mut().compiled = val,
            Declaration::Const { decl } => decl.borrow_mut().compiled = val,
            Declaration::Struct { decl } => decl.borrow_mut().compiled = val,
            Declaration::Resource { decl } => decl.borrow_mut().compiled = val,
            Declaration::Protocol { decl } => decl.borrow_mut().compiled = val,
            Declaration::Alias { decl } => decl.borrow_mut().compiled = val,
            Declaration::Table { decl } => decl.borrow_mut().compiled = val,
            Declaration::Bits { decl } => decl.borrow_mut().compiled = val,
            Declaration::Builtin { .. } => todo!(),
            Declaration::NewType => todo!(),
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
                    visitor(Element::StructMember { inner: member.clone() });
                }
            }
            Declaration::Enum { decl } => {
                for (_, member) in decl.borrow().iter_members() {
                    visitor(Element::EnumMember { inner: member.clone() });
                }
            }
            Declaration::Union { decl } => {
                for (_, member) in decl.borrow().iter_members() {
                    visitor(Element::UnionMember { inner: member.clone() });
                }
            }
            Declaration::Protocol { decl } => {
                for (_, method) in decl.borrow().iter_methods() {
                    visitor(Element::ProtocolMethod { inner: method.clone() });
                }
            }
            Declaration::Table { decl } => {
                for (_, member) in decl.borrow().iter_members() {
                    visitor(Element::TableMember { inner: member.clone() });
                }
            }
            Declaration::Bits { decl } => {
                for (_, member) in decl.borrow().iter_members() {
                    visitor(Element::BitsMember { inner: member.clone() });
                }
            }
            Declaration::Const { .. } => {}
            Declaration::Builtin { .. } => {}
            Declaration::Alias { .. } => {}
            Declaration::Resource { decl } => {
                for (_, member) in decl.borrow().iter_properties() {
                    visitor(Element::ResourceProperty { inner: member.clone() });
                }
            }
            Declaration::NewType => todo!(),
            Declaration::Overlay => todo!(),
        };
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(usize)]
pub enum BuiltinIdentity {
    // Layouts (primitive)
    #[default]
    bool = 0,
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
    framework_err,
    // Constraints
    Optional,
    MAX,
    HEAD,
}

impl BuiltinIdentity {
    pub fn index(&self) -> usize {
        *self as usize
    }
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
            BuiltinIdentity::framework_err => true,
            _ => false,
        }
    }
}

impl WithName for Builtin {
    fn name(&self) -> &Name {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Declarations {
    pub structs: Vec<Declaration>,
    pub enums: Vec<Declaration>,
    pub bits: Vec<Declaration>,
    pub unions: Vec<Declaration>,
    pub tables: Vec<Declaration>,
    pub protocols: Vec<Declaration>,
    pub builtins: Vec<Declaration>,
    pub consts: Vec<Declaration>,
    pub resources: Vec<Declaration>,
    pub imports: Vec<Declaration>,

    pub all: MultiMap<String, Declaration>,
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
            Declaration::Enum { .. } => store_decl(decl, all_ref, &mut self.enums),
            Declaration::Protocol { .. } => store_decl(decl, all_ref, &mut self.protocols),
            Declaration::Const { .. } => store_decl(decl, all_ref, &mut self.consts),
            Declaration::Alias { .. } => store_decl(decl, all_ref, &mut self.builtins),
            Declaration::Builtin { .. } => store_decl(decl, all_ref, &mut self.builtins),
            Declaration::Resource { .. } => store_decl(decl, all_ref, &mut self.resources),
            Declaration::Union { .. } => store_decl(decl, all_ref, &mut self.unions),
            Declaration::Table { .. } => store_decl(decl, all_ref, &mut self.tables),
            Declaration::Bits { .. } => store_decl(decl, all_ref, &mut self.bits),
            Declaration::NewType => todo!(),
            Declaration::Overlay => todo!(),
        }
    }

    pub(crate) fn lookup_builtin(&self, id: BuiltinIdentity) -> Declaration {
        let index = id.index() - 1;
        assert!(index < self.builtins.len(), "builtin id out of range");
        let builtin = self.builtins[index].clone();
        // assert!(builtin.id == id, "builtin's id does not match index");
        return builtin;
    }
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
            let decl = Builtin::new(id, Name::create_intrinsic(Some(library.clone()), name));

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
        insert("framework_err", BuiltinIdentity::framework_err);
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
        let all = self.declarations.borrow().all.len();
        let mut s = 0;

        for (_, decl) in self.declarations.borrow().all.flat_iter() {
            s += 1;
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
pub struct EnumId(pub u32);

/// An opaque identifier for a generator block in a library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnionId(pub u32);

/// An opaque identifier for a generator block in a library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitsId(pub u32);

/// An opaque identifier for a generator block in a library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableId(pub u32);

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
    Import(ImportId),
    Const(ConstId),
    Protocol(ProtocolId),
    Struct(StructId),
    Enum(EnumId),
    Union(UnionId),
    Bits(BitsId),
    Table(TableId),
    Builtin(BuiltinId),
}

impl std::ops::Index<DeclarationId> for Declarations {
    type Output = Declaration;

    fn index(&self, index: DeclarationId) -> &Self::Output {
        match index {
            DeclarationId::Protocol(ProtocolId(idx)) => &self.protocols[idx as usize],
            DeclarationId::Struct(StructId(idx)) => &self.structs[idx as usize],
            DeclarationId::Enum(EnumId(idx)) => &self.enums[idx as usize],
            DeclarationId::Union(UnionId(idx)) => &self.unions[idx as usize],
            DeclarationId::Bits(BitsId(idx)) => &self.bits[idx as usize],
            DeclarationId::Table(TableId(idx)) => &self.tables[idx as usize],
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
        Declaration::Enum { .. } => DeclarationId::Enum(EnumId(top_idx as u32)),
        Declaration::Const { .. } => DeclarationId::Const(ConstId(top_idx as u32)),
        Declaration::Builtin { .. } => DeclarationId::Builtin(BuiltinId(top_idx as u32)),
        Declaration::Union { .. } => DeclarationId::Union(UnionId(top_idx as u32)),
        Declaration::Table { .. } => DeclarationId::Table(TableId(top_idx as u32)),
        Declaration::Bits { .. } => DeclarationId::Bits(BitsId(top_idx as u32)),
        Declaration::Alias { .. } => todo!(),
        Declaration::Resource { .. } => todo!(),
        Declaration::NewType => todo!(),
        Declaration::Overlay => todo!(),
    }
}
