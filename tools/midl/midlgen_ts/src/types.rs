use midlgen::ir;
use serde::Serialize;

#[derive(Serialize, Default, Debug)]
pub struct Import;

#[derive(Serialize, Debug)]
pub struct Const {
    pub ir: ir::Const,
    pub name: String,
    pub r#type: String,
    pub value: String,
}

#[derive(Serialize, Debug)]
pub struct Enum {
    pub ir: ir::Enum,
    pub name: String,
    pub underlying_type: String,
    pub members: Vec<EnumMember>,
    /// Member name with the minimum value, used as an arbitrary default value
    /// in Decodable::new_empty for strict enums.
    pub min_member: String,
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct EnumMember {
    pub ir: ir::EnumMember,
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Default, Debug)]
pub struct Bits;

// A request parameter
#[derive(Serialize, Debug)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
}

// A method request or response.
#[derive(Serialize, Debug, Default)]
pub struct Payload {
    /// The raw ir type of the parameter.
    // pub og_type: ir::Type,

    // The midl.encoding.MidlType type. It can be fidl::encoding::EmptyPayload,
    // a struct, table, or union, or (only for two-way responses) one of
    // midl.encoding::{Result,Flexible,FlexibleResult}Type.
    pub midl_type: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Serialize, Debug)]
pub struct MethodResponse {
    // WireParameters represent the parameters of the top-level response struct
    // that is sent on the wire
    pub wire_parameters: Vec<Parameter>,
    // MethodParameters represent the parameters that the user interacts with
    // when using generated methods. When has_error is false, this is the same as
    // wire_parameters. When has_error is true, method_parameters corresponds to the
    // fields of a successful response.
    pub method_parameters: Vec<Parameter>,
    pub has_error: bool,
    pub has_transport_error: bool,
    pub result_type_name: String,
    pub result_type_tag_name: String,
    pub value_type: Type,
    pub error_type: Type,
}

/// Method is a method defined in a protocol.
#[derive(Serialize, Debug)]
pub struct Method {
    /// Raw JSON IR data about this method. Embedded to provide access to fields
    /// common to all bindings.
    pub ir: ir::ProtocolMethod,
    pub ordinal: u64,
    pub ordinal_name: String,
    pub name: String,
    pub has_request: bool,
    pub has_response: bool,

    pub request: Payload,
    pub response: Payload,

    /// AsyncResponseClass is a named tuple that wraps the MethodParameters of
    /// a response, and is only generated when there is more than one parameter
    pub async_response_class: String,
    pub async_response_type: String,
    pub callback_type: String,
    pub type_symbol: String,
    pub type_expr: String,
    pub transitional: bool,
}

#[derive(Serialize, Debug)]
pub struct ResultOkEntry {
    pub og_type: ir::Type,
    pub r#type: String,
    pub has_handle_metadata: bool,
    pub handle_wrapper_name: String,
}

/// A Result is the result type used for a method that is flexible or uses error syntax.
#[derive(Serialize, Debug)]
pub struct CallResult {
    /// Compound identifier for the result type, used for lookups.
    pub eci: ir::EncodedCompoundIdentifier,
    /// Rust UpperCamelCase name for the result type used when generating or
    /// referencing it.
    pub name: String,
    pub ok: Vec<ResultOkEntry>,
    pub err_og_type: Option<ir::Type>,
    pub err_type: Option<String>,
    pub has_transport_error: bool,
}

/// Protocol is the definition of a protocol in the library being compiled.
#[derive(Serialize, Debug)]
pub struct Protocol {
    /// Raw JSON IR data about this protocol. Embedded to provide access to
    /// fields common to all bindings.
    pub ir: ir::Protocol,
    /// Compound identifier referring to this protocol.
    pub eci: ir::EncodedCompoundIdentifier,
    pub marker: String,
    pub proxy: String,
    pub proxy_interface: String,
    pub synchronous_proxy: String,
    pub request: String,
    pub request_stream: String,
    pub event: String,
    pub event_stream: String,
    pub control_handle: String,
    pub discoverable: bool,
    pub debug_name: String,
    pub methods: Vec<Method>,
    pub has_events: bool,
}

/// Overflowable stores information about a method's payloads, indicating whether
/// it is possible for either of them to overflow on either encode or decode.
#[derive(Serialize, Debug, Default)]
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

#[derive(Serialize, Default, Debug)]
pub struct Table;

#[derive(Debug, Serialize)]
pub struct StructMember {
    pub(crate) ir: ir::StructMember,
    pub(crate) r#type: Type,
    pub(crate) name: String,
    pub(crate) offset_v2: u32,
}

// Struct represents a struct declaration.
/*#[derive(Serialize, Debug)]
pub struct Struct {
    ir: ir::Struct,
    doc_comments: Vec<String>,
    name: String,
    members: Vec<StructMember>,
    paddings: Vec<StructPadding>,
    type_symbol: String,
    type_expr: String,
    has_nullable_field: bool,
    is_empty_struct: bool,
} */

#[derive(Debug, Serialize)]
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
}

#[derive(Serialize, Debug)]
pub struct UnionMember {
    pub ir: ir::UnionMember,
    pub name: String,
    pub r#type: Type,
    pub ordinal: u64,
}

#[derive(Serialize, Debug)]
pub struct Union {
    pub ir: ir::Union,
    pub name: String,
    pub members: Vec<UnionMember>,
}

#[derive(Serialize, Default, Debug)]
pub struct StructPadding;

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
pub struct Type {
    // TODO(https://fxbug.dev/7660): Remove Resourceness once stored on fidlgen.Type.
    pub resourceness: ir::Resourceness,

    // Information extracted from midlgen::ir::Type.
    pub kind: TypeKind,
    pub nullable: bool,
    pub primitive_subtype: Option<ir::PrimitiveSubtype>,
    pub element_type: Option<Box<Type>>,
    pub identifier: Option<ir::EncodedCompoundIdentifier>,
    pub decl_type: Option<ir::DeclType>,

    // The marker type that implements midl.MidlType.
    pub midl: String,
    pub ctor: String,

    pub value_type: String,

    // The type to use when this occurs as a method parameter.
    // TODO(https://fxbug.dev/122199): Once the transition to the new types if complete,
    // document this as being {Value,Resource}Type::Borrowed.
    pub param: String,
}

// Root holds all of the declarations for a MIDL library.
#[derive(Serialize, Default, Debug)]
pub struct Root {
    pub library_name: String,
    pub experiments: Vec<String>,
    pub imports: Vec<Import>,
    pub consts: Vec<Const>,
    pub enums: Vec<Enum>,
    pub bits: Vec<Bits>,
    pub protocols: Vec<Protocol>,
    pub structs: Vec<Struct>,
    pub tables: Vec<Table>,
    pub unions: Vec<Union>,
    pub external_structs: Vec<Struct>,
}
