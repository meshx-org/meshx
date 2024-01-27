use iota::iota;
use midlgen::ir::{self, Identifier};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Derives(pub u16);

impl std::fmt::Debug for Derives {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Point")
    }
}

iota! {
    pub const DERIVES_DEBUG: u16 = 1 << iota;
        , DERIVES_COPY
        , DERIVES_CLONE
        , DERIVES_EQ
        , DERIVES_PARTIAL_EQ
        , DERIVES_ORD
        , DERIVES_PARTIAL_ORD
        , DERIVES_HASH
        , DERIVES_AS_BYTES
        , DERIVES_FROM_BYTES

    pub const DERIVES_ALL: u16 = (1 << iota) - 1;
}

pub const DERIVES_MINIMAL: u16 = DERIVES_DEBUG | DERIVES_PARTIAL_EQ;
pub const DERIVES_MINIMAL_NON_RESOURCE: u16 = DERIVES_MINIMAL | DERIVES_CLONE;
pub const DERIVES_ALL_BUT_ZEROCOPY: u16 = DERIVES_ALL & !DERIVES_AS_BYTES & !DERIVES_FROM_BYTES;

#[derive(Serialize, Deserialize, Debug)]
pub struct RustPaddingMarker {
    r#type: String,
    offset: i32,
    /// Mask is a string so it can be in hex.
    mask: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Const {
    pub ir: ir::Const,
    pub name: String,
    pub r#type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnionMember {
    pub ir: ir::UnionMember,
    pub name: String,
    pub r#type: Type,
    pub ordinal: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Union {
    pub name: String,
    pub members: Vec<UnionMember>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Enum {
    pub ir: ir::Enum,
    pub name: String,
    pub underlying_type: String,
    pub members: Vec<EnumMember>,
    /// Member name with the minimum value, used as an arbitrary default value
    /// in Decodable::new_empty for strict enums.
    pub min_member: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct EnumMember {
    pub ir: ir::EnumMember,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TypeKind {
    PrimitiveType,
    StringType,
    InternalType,
    HandleType,
    RequestType,
    ArrayType,
    VectorType,
    IdentifierType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Type {
    // TODO(https://fxbug.dev/7660): Remove Resourceness once stored on fidlgen.Type.
    pub resourceness: midlgen::ir::Resourceness,

    // Information extracted from fidlgen.Type.
    pub kind: TypeKind,
    pub nullable: bool,
    pub primitive_subtype: Option<midlgen::ir::PrimitiveSubtype>,
    pub element_type: Option<Box<Type>>,
    pub identifier: Option<ir::EncodedCompoundIdentifier>,
    pub decl_type: Option<midlgen::ir::DeclType>,

    // The marker type that implements midl::encoding::Type.
    pub midl: String,

    // The associated type midl::encoding::Type::Owned.
    pub owned: String,

    // The type to use when this occurs as a method parameter.
    // TODO(https://fxbug.dev/122199): Once the transition to the new types if complete,
    // document this as being {Value,Resource}Type::Borrowed.
    pub param: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructMember {
    pub(crate) ir: midlgen::ir::StructMember,
    pub(crate) r#type: Type,
    pub(crate) name: String,
    pub(crate) offset_v2: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    pub ir: midlgen::ir::Table,
    // pub derives: Derives,
    pub eci: ir::EncodedCompoundIdentifier,
    pub name: String,
    pub members: Vec<TableMember>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableMember {
    pub ir: midlgen::ir::TableMember,
    pub r#type: Type,
    pub name: String,
    pub ordinal: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Struct {
    pub(crate) ir: midlgen::ir::Struct,
    pub(crate) eci: ir::EncodedCompoundIdentifier,
    // pub(crate) derives: Derives,
    pub(crate) name: String,
    pub(crate) members: Vec<StructMember>,
    pub(crate) padding_markers_v2: Vec<midlgen::PaddingMarker>,
    pub(crate) flattened_padding_markers_v2: Vec<midlgen::PaddingMarker>,
    pub(crate) size_v2: u32,
    pub(crate) alignment_v2: u32,
    pub(crate) has_padding: bool,
    // True if the struct should be encoded and decoded by memcpy.
    pub(crate) use_midl_struct_copy: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResultOkEntry {
    pub og_type: ir::Type,
    pub r#type: String,
    pub has_handle_metadata: bool,
    pub handle_wrapper_name: String,
}

/// A Result is the result type used for a method that is flexible or uses error syntax.
#[derive(Serialize, Deserialize, Debug)]
pub struct CallResult {
    /// Compound identifier for the result type, used for lookups.
    pub eci: ir::EncodedCompoundIdentifier,
    pub derives: Derives,
    /// Rust UpperCamelCase name for the result type used when generating or
    /// referencing it.
    pub name: String,
    pub ok: Vec<ResultOkEntry>,
    pub err_og_type: Option<ir::Type>,
    pub err_type: Option<String>,
    pub has_transport_error: bool,
}

/// Protocol is the definition of a protocol in the library being compiled.
#[derive(Serialize, Deserialize, Debug)]
pub struct Protocol {
    /// Raw JSON IR data about this protocol. Embedded to provide access to
    /// fields common to all bindings.
    pub base: ir::Protocol,
    /// Compound identifier referring to this protocol.
    pub eci: ir::EncodedCompoundIdentifier,
    /// Name of the protocol as a Rust CamelCase identifier. Since only protocols
    /// from the same library are included, this will never be qualified, so it
    /// is just the CamelCase name of the protocol.
    pub name: String,
    /// List of methods that are part of this protocol. Processed from
    /// fidlgen.Protocol to add Rust-specific fields.
    pub methods: Vec<Method>,
    /// Name of this protocol for legacy (pre-RFC-0041) service discovery, if the
    /// protocol is marked as discoverable. This value does not include enclosing
    /// quote marks.
    pub protocol_name: String,
}

/// Overflowable stores information about a method's payloads, indicating whether
/// it is possible for either of them to overflow on either encode or decode.
#[derive(Serialize, Deserialize, Debug)]
pub struct Overflowable {
    /// OnRequestEncode indicates whether or not the parent method's request
    /// payload may be so large on encode as to require overflow handling.
    pub on_request_encode: bool,
    /// OnRequestDecode indicates whether or not the parent method's request
    /// payload may be so large on decode as to require overflow handling. This
    /// will always be true if OnRequestEncode is true, as the maximum size on
    /// decode is always larger than encode. This is because only the latter may
    /// include unknown, arbitrarily large data.
    pub on_request_decode: bool,
    /// OnResponseEncode indicates whether or not the parent method's response
    /// payload may be so large on encode as to require overflow handling.
    pub on_response_encode: bool,
    /// OnResponseDecode indicates whether or not the parent method's response
    /// payload may be so large on decode as to require overflow handling. This
    /// will always be true if OnResponseEncode is true, as the maximum size on
    /// decode is always larger than encode. This is because only the latter may
    /// include unknown, arbitrarily large data.
    pub on_response_decode: bool,
}

/// Method is a method defined in a protocol.
#[derive(Serialize, Deserialize, Debug)]
pub struct Method {
    /// Raw JSON IR data about this method. Embedded to provide access to fields
    /// common to all bindings.
    pub base: ir::Method,
    /// Name of the method converted to snake_case. Used when generating
    /// rust-methods associated with this method, such as proxy methods and
    /// encoder methods.
    pub name: String,
    /// Name of the method converted to CamelCase. Used when generating
    /// rust-types associated with this method, such as responders.
    pub camel_name: String,
    /// Parameters to this method extracted from the request type struct.
    pub request: Vec<Parameter>,
    /// Arguments used for method responses. If error syntax is used, this will
    /// contain a single element for the Result enum used in rust generated code.
    /// For methods which do not use error syntax, this will contain fields
    /// extracted from the response struct.
    ///
    /// Note that since methods being strict vs flexible is not exposed in the
    /// client API, this field does not reflect whether the method is strict or
    /// flexible. For flexible, the value is still either fields extracted from
    /// the response struct or the Rust Result enum, depending only on whether
    /// error syntax was used.  In the case of flexible methods without error
    /// syntax, the parameters are extracted from the success variant of the
    /// underlying result union.
    pub response: Vec<Parameter>,
    /// If error syntax was used, this will contain information about the result
    /// union.
    pub result: CallResult,
    /// Stores overflowing information for this method's payloads.
    pub overflowable: Overflowable,
}

/// A Parameter to either the requset or response of a method. Contains
/// information to assist in generating code using borrowed types and handle
/// wrappers.
#[derive(Serialize, Deserialize, Debug)]
pub struct Parameter {
    /// The raw fidlgen type of the parameter.
    pub og_type: ir::Type,
    /// String representing the type to use for this parameter when handling it
    /// by-value.
    pub r#type: String,
    /// String representing the type to use for this parameter when receiving it
    /// as a possibly-borrowed method argument.
    pub borrowed_type: String,
    /// Snake-case name to use for the parameter.
    pub name: String,
    /// Name of the wrapper type that should be used for handle validation, if
    /// HasHandleMetadata is true.
    pub handle_wrapper_name: String,
    /// True if the type of the parameter has handle metadata and so requires
    /// validation.
    pub has_handle_metadata: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    pub extern_crates: Vec<String>,
    // bits: Vec<Bits>,
    pub consts: Vec<Const>,
    pub enums: Vec<Enum>,
    pub unions: Vec<Union>,
    pub structs: Vec<Struct>,
    pub external_structs: Vec<Struct>,
    pub tables: Vec<Table>,
    /// Result types for methods with error syntax.
    // results: Vec<Result>,
    pub protocols: Vec<Protocol>,
    // services: Vec<Service>,
}
