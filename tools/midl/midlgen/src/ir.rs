use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::PaddingMarker;

pub type LiteralKind<'a> = &'a str;
pub static StringLiteral: LiteralKind = "string";
pub static NumericLiteral: LiteralKind = "numeric";
pub static BoolLiteral: LiteralKind = "bool";
pub static DefaultLiteral: LiteralKind = "default";

// An EncodedLibraryIdentifier is a LibraryIdentifier encoded as a string,
// suitable for use in map keys.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct EncodedLibraryIdentifier(pub String);

// Encode formats a LibraryIdentifier as a EncodedLibraryIdentifier by joining the identifier
// components with ".", e.g.  "my.fidl.library".
impl From<LibraryIdentifier> for EncodedLibraryIdentifier {
    fn from(li: LibraryIdentifier) -> Self {
        EncodedLibraryIdentifier(li.0.into_iter().map(|id| id.0).collect::<Vec<String>>().join("."))
    }
}

impl EncodedLibraryIdentifier {
    // Parse decodes an EncodedLibraryIdentifier back into a LibraryIdentifier.
    pub fn parse(&self) -> LibraryIdentifier {
        let parts = self.parts();
        let mut idents = Vec::new();

        for part in parts.iter() {
            idents.push(Identifier(part.to_string()));
        }

        LibraryIdentifier(idents)
    }

    fn parts(&self) -> Vec<&str> {
        self.0.split(".").collect()
    }
}

// An EncodedCompoundIdentifier is a CompoundIdentifier encoded as a string,
// suitable for use in map keys.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct EncodedCompoundIdentifier(pub String);

// Encodes the fully-qualified declaration portion of the CompoundIdentifier.
//
// Encoded form consists of the encoded library identifier, then the declaration
// name. If a member is specified, it will come after the declaration name,
// separated by a dot. Example:
// - With no Member: "my.midl.library/MyProtocol"
// - With Member: "my.midl.library/MyProtocol.SomeMethod"
impl From<CompoundIdentifier> for EncodedCompoundIdentifier {
    fn from(ci: CompoundIdentifier) -> Self {
        EncodedCompoundIdentifier(format!(
            "{:?}/{}",
            EncodedLibraryIdentifier::from(ci.library),
            ci.name.0
        ))
    }
}

impl EncodedCompoundIdentifier {
    /// Parse converts an EncodedCompoundIdentifier back into a CompoundIdentifier.
    pub fn parse(&self) -> CompoundIdentifier {
        let parts = self.parts();

        let mut raw_library = "";
        let mut raw_name = parts[0];

        if parts.len() == 2 {
            raw_library = parts[0];
            raw_name = parts[1];
        };

        let library = EncodedLibraryIdentifier(raw_library.to_string()).parse();
        let name_parts = raw_name.split(".").collect::<Vec<&str>>();
        let name = Identifier(name_parts[0].to_string());
        let mut member = None;
        if name_parts.len() == 2 {
            member = Some(Identifier(name_parts[1].to_string()))
        }

        return CompoundIdentifier { library, name, member };
    }

    /// library_name() retrieves the library name from an EncodedCompoundIdentifier.
    pub fn library_name(&self) -> EncodedLibraryIdentifier {
        let mut raw_library = String::from("");
        let parts = self.parts();

        if parts.len() == 2 {
            raw_library = parts[0].to_string();
        }

        return EncodedLibraryIdentifier(raw_library);
    }

    /// DeclName retrieves the fully-qualified declaration name from an
    /// EncodedCompoundIdentifier. This operation is idempotent.
    pub fn decl_name(&self) -> EncodedCompoundIdentifier {
        let ci = self.parse();
        let mut parts = vec![];

        for l in ci.library.0 {
            parts.push(l.0)
        }

        EncodedCompoundIdentifier(format!("{}/{}", parts.join("."), ci.name))
    }

    // Parts splits an EncodedCompoundIdentifier into an optional library name and
    // declaration or member id.
    //
    // This splits off the library name, but does not check whether the referenced
    // member is a delaration or member of a declaration.
    fn parts(&self) -> Vec<&str> {
        self.0.splitn(2, "/").collect()
    }
}

// A LibraryIdentifier identifies a FIDL library, from the library declaration
// at the start of a FIDL file.
#[derive(Debug)]
pub struct LibraryIdentifier(Vec<Identifier>);

impl From<EncodedLibraryIdentifier> for LibraryIdentifier {
    fn from(eci: EncodedLibraryIdentifier) -> Self {
        Self(eci.0.split(".").map(|v| Identifier(v.to_owned())).collect())
    }
}

impl LibraryIdentifier {
    /// Encode formats a LibraryIdentifier as a string by joining the identifier
    /// components with ".", e.g.  "my.midl.library".
    pub fn encode(&self) -> EncodedLibraryIdentifier {
        let mut ss = Vec::new();

        for s in self.0.iter() {
            ss.push(s.0.clone());
        }

        EncodedLibraryIdentifier(ss.join("."))
    }
}

#[derive(Debug)]
pub struct CompoundIdentifier {
    // Library the declaration is in.
    library: LibraryIdentifier,
    // Name of the declaration.
    pub name: Identifier,
    // Member of the declaration. If set to empty string, this
    // CompoundIdentifier refers to the declaration rather than a member.
    pub member: Option<Identifier>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Identifier(pub String);

impl PartialEq<String> for Identifier {
    fn eq(&self, other: &String) -> bool {
        self.0.eq(other)
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = &self.0;
        write!(f, "{}", v)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DeclType {
    #[serde(rename = "const")]
    ConstDecl,
    #[serde(rename = "alias")]
    AliasDecl,
    #[serde(rename = "enum")]
    EnumDecl,
    #[serde(rename = "bits")]
    BitsDecl,
    #[serde(rename = "protocol")]
    ProtocolDecl,
    #[serde(rename = "struct")]
    StructDecl,
    #[serde(rename = "table")]
    TableDecl,
    #[serde(rename = "union")]
    UnionDecl,
    #[serde(rename = "overlay")]
    OverlayDecl,
    #[serde(rename = "experimental_resource")]
    ExperimentalResourceDecl,
}

pub type DeclMap = HashMap<EncodedCompoundIdentifier, DeclType>;
pub type DeclInfoMap = HashMap<EncodedCompoundIdentifier, DeclInfo>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeclInfo {
    #[serde(rename = "kind")]
    pub r#type: DeclType,
    // Present for structs, tables, and unions.
    pub resource: Option<Resourceness>,
}

pub trait Decl {
    fn get_type(&self) -> DeclType;
    fn get_name(&self) -> EncodedCompoundIdentifier;
    fn get_resourceness(&self) -> Option<Resourceness>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    pub filename: String,
    pub line: i32,
    pub column: i32,
    pub length: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TypeShapeV2 {
    pub inline_size: u32,
    pub alignment: u32,
    pub depth: u32,
    pub max_handles: u32,
    pub max_out_of_line: u32,
    pub has_padding: bool,
    pub has_flexible_envelope: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind")]
pub enum Type {
    #[serde(rename = "vector")]
    VectorType {
        element_type: Box<Type>,
        element_count: Option<u32>,
        nullable: bool,
    },
    #[serde(rename = "array")]
    ArrayType {
        element_type: Box<Type>,
        element_count: u32,
        type_shape_v2: TypeShape,
    },
    #[serde(rename = "string")]
    StringType {
        element_count: Option<u32>,
        #[serde(default)]
        nullable: bool,
        type_shape_v2: TypeShape,
    },
    #[serde(rename = "string_array")]
    StringArray {
        #[serde(default = "default_string_array")]
        element_type: Box<Type>,
        element_count: u32,
        type_shape_v2: TypeShape,
    },
    #[serde(rename = "primitive")]
    PrimitiveType {
        #[serde(rename = "subtype")]
        primitive_subtype: PrimitiveSubtype,
    },
    #[serde(rename = "handle")]
    HandleType {
        #[serde(rename = "subtype")]
        handle_subtype: HandleSubtype,
        #[serde(rename = "rights")]
        handle_rights: HandleRights,
        nullable: bool,
        type_shape_v2: TypeShape,
    },
    #[serde(rename = "request")]
    RequestType {
        #[serde(rename = "subtype")]
        request_subtype: EncodedCompoundIdentifier,
        nullable: bool,
        type_shape_v2: TypeShape,
    },
    #[serde(rename = "identifier")]
    IdentifierType {
        identifier: EncodedCompoundIdentifier,
        nullable: bool,
        type_shape_v2: TypeShape,
    },
    #[serde(rename = "internal")]
    InternalType {
        #[serde(rename = "subtype")]
        internal_subtype: InternalSubtype,
        type_shape_v2: TypeShape,
    },
}

fn default_string_array() -> Box<Type> {
    Box::from(Type::PrimitiveType {
        primitive_subtype: PrimitiveSubtype::Uint8,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HandleRights(u32);

bitflags::bitflags! {
   impl HandleRights: u32 {
        const READ       = 0b00000001;
        const WRITE      = 0b00000010;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveSubtype {
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "float32")]
    Float32,
    #[serde(rename = "float64")]
    Float64,
    #[serde(rename = "uint64")]
    Uint64,
    #[serde(rename = "uint32")]
    Uint32,
    #[serde(rename = "uint16")]
    Uint16,
    #[serde(rename = "uint8")]
    Uint8,
    #[serde(rename = "int8")]
    Int8,
    #[serde(rename = "int64")]
    Int64,
    #[serde(rename = "int32")]
    Int32,
    #[serde(rename = "int16")]
    Int16,
}

impl PrimitiveSubtype {
    pub fn is_signed(&self) -> bool {
        match self {
            PrimitiveSubtype::Int64 => true,
            PrimitiveSubtype::Int32 => true,
            PrimitiveSubtype::Int16 => true,
            PrimitiveSubtype::Int8 => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InternalSubtype {
    #[serde(rename = "framework_error")]
    FrameworkErr,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum HandleSubtype {
    #[serde(rename = "channel")]
    Channel,
    #[serde(rename = "event")]
    Event,
    #[serde(rename = "eventpair")]
    EventPair,
    #[serde(rename = "exception")]
    Exception,
    #[serde(rename = "guest")]
    Guest,
    #[serde(rename = "fifo")]
    Fifo,
    #[serde(rename = "bti")]
    Bti,
    #[serde(rename = "clock")]
    Clock,
    #[serde(rename = "debuglog")]
    Debuglog,
    #[serde(rename = "handle")]
    None,

    #[serde(rename = "interrupt")]
    Interrupt,
    #[serde(rename = "iommu")]
    Iommu,
    #[serde(rename = "job")]
    Job,
    #[serde(rename = "msi")]
    Msi,
    #[serde(rename = "pager")]
    Pager,
    #[serde(rename = "pcidevice")]
    PciDevice,
    #[serde(rename = "pmt")]
    Pmt,
    #[serde(rename = "port")]
    Port,
    #[serde(rename = "process")]
    Process,
    #[serde(rename = "profile")]
    Profile,
    #[serde(rename = "resource")]
    Resource,
    #[serde(rename = "socket")]
    Socket,
    #[serde(rename = "stream")]
    Stream,
    #[serde(rename = "suspendtoken")]
    SuspendToken,
    #[serde(rename = "thread")]
    Thread,
    #[serde(rename = "timer")]
    Timer,
    #[serde(rename = "vcpu")]
    Vcpu,
    #[serde(rename = "vmar")]
    Vmar,
    #[serde(rename = "vmo")]
    Vmo,
}

/// Library represents a MIDL dependency on a separate library.
#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    name: EncodedLibraryIdentifier,
    declarations: DeclInfoMap,
}

/// Root is the top-level object for a MIDL library.
/// It contains lists of all declarations and dependencies within the library.
#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    pub name: EncodedLibraryIdentifier,
    pub experiments: Vec<String>,

    pub const_declarations: Vec<Const>,
    pub enum_declarations: Vec<Enum>,
    pub struct_declarations: Vec<Struct>,
    pub protocol_declarations: Vec<Protocol>,
    pub table_declarations: Vec<Table>,
    pub union_declarations: Vec<Union>,
    pub bits_declarations: Vec<Bits>,

    pub library_dependencies: Vec<Library>,
}

impl Root {
    /// DeclInfo returns information on the MIDL library's local and imported
    /// declarations.
    pub fn decl_info(&self) -> DeclInfoMap {
        let mut info_map = DeclInfoMap::new();

        self.for_each_decl(&mut |decl| {
            let info = DeclInfo {
                r#type: decl.get_type(),
                resource: decl.get_resourceness(),
            };

            //if resDecl, ok := decl.(ResourceableLayoutDecl); ok {
            //    info.Resourceness = new(Resourceness)
            //    *info.Resourceness = resDecl.GetResourceness()
            //};

            info_map.insert(decl.get_name(), info);
        });

        for l in self.library_dependencies.iter() {
            for (k, v) in l.declarations.iter() {
                info_map.insert(k.clone(), v.clone());
            }
        }

        info_map
    }

    /// ForEachDecl calls a provided callback on each associated declaration. Logic
    /// that needs to iterate over all declarations should rely on this method as
    /// opposed to hardcoding the known (at the time) set of declaration types.
    fn for_each_decl(&self, cb: &mut dyn FnMut(&dyn Decl)) {
        for enum_decl in self.enum_declarations.iter() {
            cb(enum_decl)
        }

        for const_decl in self.const_declarations.iter() {
            cb(const_decl)
        }

        for bits_decl in self.bits_declarations.iter() {
            cb(bits_decl)
        }

        for protocol_decl in self.protocol_declarations.iter() {
            cb(protocol_decl)
        }

        for union_decl in self.union_declarations.iter() {
            cb(union_decl)
        }

        for struct_decl in self.struct_declarations.iter() {
            cb(struct_decl)
        }

        for table_decl in self.table_declarations.iter() {
            cb(table_decl)
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct EnumMember {
    pub name: Identifier,
    pub value: Constant,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructMember {
    pub location: Location,
    pub name: Identifier,
    pub r#type: Type,
    pub field_shape_v2: FieldShape,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct Resourceness(pub bool);

pub const RESOURCE_TYPE: Resourceness = Resourceness(true);
pub const VALUE_TYPE: Resourceness = Resourceness(true);

impl Resourceness {
    pub fn is_resource_type(&self) -> bool {
        self.0
    }

    pub fn is_value_type(&self) -> bool {
        !self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub location: Location,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Struct {
    #[serde(rename = "resource")]
    pub resourceness: Resourceness,

    pub maybe_attributes: Option<Vec<Attribute>>,

    pub name: EncodedCompoundIdentifier,
    pub naming_context: NamingContext,

    pub location: Location,

    // pub naming_context: NamingContext,
    pub is_empty_success_struct: bool,
    pub members: Vec<StructMember>,
    pub max_handles: Option<u32>,
    pub type_shape_v2: TypeShape,
}

impl Decl for Struct {
    fn get_type(&self) -> DeclType {
        DeclType::StructDecl
    }

    fn get_name(&self) -> EncodedCompoundIdentifier {
        self.name.clone()
    }

    fn get_resourceness(&self) -> Option<Resourceness> {
        Some(self.resourceness.clone())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bits {
    pub name: EncodedCompoundIdentifier,
    pub location: Location,
}

impl Decl for Bits {
    fn get_type(&self) -> DeclType {
        DeclType::BitsDecl
    }

    fn get_name(&self) -> EncodedCompoundIdentifier {
        self.name.clone()
    }

    fn get_resourceness(&self) -> Option<Resourceness> {
        None
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct UnionMember {
    pub name: Option<Identifier>,
    pub reserved: bool,
    pub ordinal: u64,
    #[serde(default)]
    pub max_out_of_line: i64,
    #[serde(rename = "type")]
    pub r#type: Option<Type>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Union {
    pub name: EncodedCompoundIdentifier,
    pub location: Location,

    pub members: Vec<UnionMember>,

    #[serde(rename = "resource")]
    pub resourceness: Resourceness,
}

impl Decl for Union {
    fn get_type(&self) -> DeclType {
        DeclType::UnionDecl
    }

    fn get_name(&self) -> EncodedCompoundIdentifier {
        self.name.clone()
    }

    fn get_resourceness(&self) -> Option<Resourceness> {
        Some(self.resourceness)
    }
}

/// TypeShape represents the shape of the type on the wire.
/// See JSON IR schema, e.g. midlc --json-schema
#[derive(Serialize, Deserialize, Debug, Copy, Default, Clone)]
pub struct TypeShape {
    pub inline_size: u32,
    pub alignment: u32,
    pub depth: u32,
    pub max_handles: u32,
    pub max_out_of_line: u32,
    pub has_padding: bool,
    pub has_flexible_envelope: bool,
}

/// FieldShape represents the shape of the field on the wire.
/// See JSON IR schema, e.g. midlc --json-schema
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FieldShape {
    pub offset: u32,
    pub padding: u32,
}

#[derive(Default)]
pub struct PaddingConfig {
    pub flatten_structs: bool,
    pub flatten_arrays: bool,
}

/// NamingContext represents the content of the `naming_context` JSON IR field,
/// which enumerates inr order the names of the parent declarations of some
/// declaration. Top-level declarations have a list of size 1, with their own
/// name as the only member. Nested (ie, anonymous) declarations are lists of a
/// size greater than 1, starting with the outer most ancestor declaration.
///
/// While the `name` and the last string in a `naming_context` are usually
/// identical, the `name` can be arbitrarily changed using the
/// `@generated_name()` MIDL annotation, so this is not guaranteed to be the
/// case.
pub type NamingContext = Vec<String>;

impl Struct {
    pub fn build_padding_markers(&self, conf: PaddingConfig) -> Vec<PaddingMarker> {
        vec![]
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(tag = "kind")]
pub enum Literal {
    #[serde(rename = "string")]
    StringLiteral { value: String },
    #[serde(rename = "numeric")]
    NumericLiteral { value: String },
    #[serde(rename = "bool")]
    BoolLiteral { value: bool },
    #[default]
    #[serde(rename = "default")]
    DefaultLiteral,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum Constant {
    #[serde(rename = "identifier")]
    Identifier {
        identifier: EncodedCompoundIdentifier,
        value: String,
        expression: String,
    },
    #[serde(rename = "literal")]
    LiteralConstant {
        value: String,
        literal: Literal,
        expression: String,
    },
    #[serde(rename = "binary_operator")]
    BinaryOperator { value: String, expression: String },
}

impl Default for Constant {
    fn default() -> Self {
        Constant::LiteralConstant {
            value: Default::default(),
            literal: Default::default(),
            expression: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Const {
    pub name: EncodedCompoundIdentifier,
    pub location: Location,
    pub r#type: Type,

    pub value: Constant,
}

impl Decl for Const {
    fn get_type(&self) -> DeclType {
        DeclType::ConstDecl
    }

    fn get_name(&self) -> EncodedCompoundIdentifier {
        self.name.clone()
    }

    fn get_resourceness(&self) -> Option<Resourceness> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Enum {
    #[serde(default)]
    pub maybe_attributes: Vec<Attribute>,
    pub name: EncodedCompoundIdentifier,
    pub location: Location,

    pub r#type: PrimitiveSubtype,
    pub members: Vec<EnumMember>,

    #[serde(rename = "strict")]
    pub is_strict: bool,

    #[serde(rename = "maybe_unknown_value")]
    pub raw_unknown_value: Option<u64>,
}

impl Decl for Enum {
    fn get_type(&self) -> DeclType {
        DeclType::EnumDecl
    }

    fn get_name(&self) -> EncodedCompoundIdentifier {
        self.name.clone()
    }

    fn get_resourceness(&self) -> Option<Resourceness> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtocolMethod {
    pub name: Identifier,
    pub ordinal: u64,
    pub has_response: bool,
    pub has_request: bool,
    pub has_error: bool,

    pub maybe_request_payload: Option<Type>,
    pub maybe_response_payload: Option<Type>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Protocol {
    pub location: Location,
    pub name: EncodedCompoundIdentifier,
    pub methods: Vec<ProtocolMethod>,
}

impl Decl for Protocol {
    fn get_type(&self) -> DeclType {
        DeclType::ProtocolDecl
    }

    fn get_name(&self) -> EncodedCompoundIdentifier {
        self.name.clone()
    }

    fn get_resourceness(&self) -> Option<Resourceness> {
        None
    }
}

/// Table represents a declaration of a MIDL table.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Table {
    pub name: EncodedCompoundIdentifier,
    pub location: Location,
    pub resource: Resourceness,
    // resourceableLayoutDecl
    pub members: Vec<TableMember>,
    pub type_shape_v2: TypeShape, // Strictness: `json:"strict"`
}

// TableMember represents the declaration of a field in a FIDL table.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TableMember {
    // member
    pub name: Option<Identifier>,
    pub reserved: bool,
    pub r#type: Option<Type>,
    pub ordinal: i64,
    pub maybe_default_value: Option<Constant>,
    #[serde(default)]
    pub max_out_of_line: i64,
}

impl Decl for Table {
    fn get_type(&self) -> DeclType {
        DeclType::TableDecl
    }

    fn get_name(&self) -> EncodedCompoundIdentifier {
        self.name.clone()
    }

    fn get_resourceness(&self) -> Option<Resourceness> {
        Some(self.resource)
    }
}
