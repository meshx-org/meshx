use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

type PrimitiveSubtype<'a> = &'a str;
static BOOL: PrimitiveSubtype = "bool";
static INT8: PrimitiveSubtype = "int8";
static INT16: PrimitiveSubtype = "int16";
static INT32: PrimitiveSubtype = "int32";
static INT64: PrimitiveSubtype = "int64";
static UINT8: PrimitiveSubtype = "uint8";
static UINT16: PrimitiveSubtype = "uint16";
static UINT32: PrimitiveSubtype = "uint32";
static UINT64: PrimitiveSubtype = "uint64";
static FLOAT32: PrimitiveSubtype = "float32";
static FLOAT64: PrimitiveSubtype = "float64";

type HandleSubtype = &'static str;
static HANDLE_SUBTYPE_NONE: HandleSubtype = "handle";
static HANDLE_SUBTYPE_CHANNEL: HandleSubtype = "channel";
static HANDLE_SUBTYPE_EVENT: HandleSubtype = "event";
static HANDLE_SUBTYPE_EVENTPAIR: HandleSubtype = "eventpair";
static HANDLE_SUBTYPE_EXCEPTION: HandleSubtype = "exception";
static HANDLE_SUBTYPE_JOB: HandleSubtype = "job";
static HANDLE_SUBTYPE_PROCESS: HandleSubtype = "process";
static HANDLE_SUBTYPE_STREAM: HandleSubtype = "stream";
static HANDLE_SUBTYPE_THREAD: HandleSubtype = "thread";
static HANDLE_SUBTYPE_TIME: HandleSubtype = "timer";
static HANDLE_SUBTYPE_VMAR: HandleSubtype = "vmar";
static HANDLE_SUBTYPE_VMO: HandleSubtype = "vmo";

pub type LiteralKind<'a> = &'a str;
pub static StringLiteral: LiteralKind = "string";
pub static NumericLiteral: LiteralKind = "numeric";
pub static BoolLiteral: LiteralKind = "bool";
pub static DefaultLiteral: LiteralKind = "default";

// An EncodedLibraryIdentifier is a LibraryIdentifier encoded as a string,
// suitable for use in map keys.
#[derive(Serialize, Deserialize, Debug)]
pub struct EncodedLibraryIdentifier(pub String);

// Encode formats a LibraryIdentifier as a EncodedLibraryIdentifier by joining the identifier
// components with ".", e.g.  "my.fidl.library".
impl From<LibraryIdentifier> for EncodedLibraryIdentifier {
    fn from(li: LibraryIdentifier) -> Self {
        EncodedLibraryIdentifier(li.into_iter().map(|id| id.value).collect::<Vec<String>>().join("."))
    }
}

// An EncodedCompoundIdentifier is a CompoundIdentifier encoded as a string,
// suitable for use in map keys.
#[derive(Serialize, Deserialize, Debug)]
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
            ci.name.value
        ))
    }
}

// A LibraryIdentifier identifies a FIDL library, from the library declaration
// at the start of a FIDL file.
pub type LibraryIdentifier = Vec<Identifier>;

impl From<EncodedLibraryIdentifier> for LibraryIdentifier {
    fn from(eci: EncodedLibraryIdentifier) -> Self {
        eci.0.split(".").map(|v| Identifier { value: v.to_owned() }).collect()
    }
}

#[derive(Debug)]
pub struct CompoundIdentifier {
    // Library the declaration is in.
    library: LibraryIdentifier,
    // Name of the declaration.
    name: Identifier,
    // Member of the declaration. If set to empty string, this
    // CompoundIdentifier refers to the declaration rather than a member.
    member: Option<Identifier>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Identifier {
    value: String,
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = &self.value;
        write!(f, "{}", v)
    }
}

pub trait Declaration {
    fn get_attributes();
    fn get_location() -> Location;
    fn get_name() -> EncodedCompoundIdentifier;
}

pub type DeclInfoMap = HashMap<String, ()>;

#[derive(Serialize, Deserialize, Debug)]
pub struct BaseDeclaration {
    pub name: String,
    pub location: Location,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub filename: String,
    pub line: i32,
    pub column: i32,
    pub length: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Type {
    // TODO
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    pub library: Library,

    pub r#const: Vec<Const>,
    pub r#enum: Vec<Enum>,
    pub r#struct: Vec<Struct>,
    pub r#protocol: Vec<Protocol>,
}

impl Root {
    pub fn decl_info(&self) -> DeclInfoMap {
        HashMap::new()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    pub name: String,
    pub doc: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfType {
    pub r#type: String,
    pub of: Option<Box<OfType>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnumMember {
    pub doc: Option<String>,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StructMember {
    pub doc: Option<String>,
    pub r#type: String,
    pub of: Option<OfType>,
    // TODO has_default: bool,
    // TODO default_value: String,
    // TODO has_handle_metadata: bool,
    // TODO handle_rights: String,
    // TODO handle_subtype: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Struct {
    pub base: BaseDeclaration,
    pub doc: Option<String>,
    pub member: HashMap<String, StructMember>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Const {
    pub base: BaseDeclaration,
    pub doc: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Enum {
    pub base: BaseDeclaration,
    pub doc: Option<String>,
    pub member: Option<HashMap<String, EnumMember>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Method {
    pub doc: Option<String>,
    pub ordinal: u64,
    pub has_response: bool,
    pub has_request: bool,
    pub has_error: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Protocol {
    pub doc: Option<String>,
    pub methods: Vec<Method>,
}
