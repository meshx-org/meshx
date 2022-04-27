package types

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
)

type Declaration interface {
	GetName() EncodedCompoundIdentifier
}

type Decl struct {
	Attributes
	Name     EncodedCompoundIdentifier `json:"name"`
	Location `json:"location"`
}

func (d *Decl) GetName() EncodedCompoundIdentifier {
	return d.Name
}

// Layout represents data specific to bits/enums/structs/tables/unions. All
// layouts are decls, but not all decls are layouts (e.g. protocols).
type Layout struct {
	Decl
	//NamingContext NamingContext `json:"naming_context"`
}

// Assert that declarations conform to the Declaration interface
var _ = []Declaration{
	// NOTE: (*TypeAlias)(nil),
	(*Union)(nil),
	(*Table)(nil),
	(*Struct)(nil),
	(*Protocol)(nil),
	// NOTE: (*Service)(nil),
	(*Enum)(nil),
	// NOTE: (*Bits)(nil),
	// NOTE: (*Const)(nil),
}

// Union represents the declaration of a FIDL union.
type Union struct {
	Layout
	Members []UnionMember `json:"members"`
	// Strictness   `json:"strict"`
	// Resourceness `json:"resource"`
	TypeShapeV1 TypeShape `json:"type_shape_v1"`
	TypeShapeV2 TypeShape `json:"type_shape_v2"`
}

// UnionMember represents the declaration of a field in a FIDL extensible
// union.
type UnionMember struct {
	Attributes
	Reserved     bool       `json:"reserved"`
	Ordinal      int        `json:"ordinal"`
	Type         Type       `json:"type"`
	Name         Identifier `json:"name"`
	Offset       int        `json:"offset"`
	MaxOutOfLine int        `json:"max_out_of_line"`
}

// Struct represents a declaration of a MIDL struct.
type Struct struct {
	Layout
	Members []StructMember `json:"members"`
	// Resourceness `json:"resource"`
	TypeShapeV1 TypeShape `json:"type_shape_v1"`
	TypeShapeV2 TypeShape `json:"type_shape_v2"`
}

// StructMember represents the declaration of a field in a MIDL struct.
type StructMember struct {
	Attributes
	Type              Type       `json:"type"`
	Name              Identifier `json:"name"`
	MaybeDefaultValue *Constant  `json:"maybe_default_value,omitempty"`
	MaxHandles        int        `json:"max_handles"`
	FieldShapeV1      FieldShape `json:"field_shape_v1"`
	FieldShapeV2      FieldShape `json:"field_shape_v2"`
}

// Table represents a declaration of a MIDL table.
type Table struct {
	Layout
	Members []TableMember `json:"members"`
	// Resourceness `json:"resource"`
	TypeShapeV1 TypeShape `json:"type_shape_v1"`
	TypeShapeV2 TypeShape `json:"type_shape_v2"`
}

// TableMember represents the declaration of a field in a MIDL table.
type TableMember struct {
	Attributes
	Reserved          bool       `json:"reserved"`
	Type              Type       `json:"type"`
	Name              Identifier `json:"name"`
	Ordinal           int        `json:"ordinal"`
	MaybeDefaultValue *Constant  `json:"maybe_default_value,omitempty"`
	MaxOutOfLine      int        `json:"max_out_of_line"`
}

// Enum represents a FIDL declaration of an enum.
type Enum struct {
	Layout
	Type    PrimitiveSubtype `json:"type"`
	Members []EnumMember     `json:"members"`
	// Strictness      `json:"strict"`
	RawUnknownValue int64OrUint64 `json:"maybe_unknown_value"`
}

// EnumMember represents a single variant in a FIDL enum.
type EnumMember struct {
	Attributes
	Name  Identifier `json:"name"`
	Value Constant   `json:"value"`
}

// Protocol represents the declaration of a FIDL protocol.
type Protocol struct {
	Decl
	Methods []Method `json:"methods"`
}

// Method represents the declaration of a FIDL method.
type Method struct {
	Attributes
	Ordinal         uint64     `json:"ordinal"`
	Name            Identifier `json:"name"`
	IsComposed      bool       `json:"is_composed"`
	HasRequest      bool       `json:"has_request"`
	RequestPayload  *Type      `json:"maybe_request_payload,omitempty"`
	RequestPadding  bool       `json:"maybe_request_has_padding,omitempty"`
	RequestFlexible bool       `json:"experimental_maybe_request_has_flexible_envelope,omitempty"`
	HasResponse     bool       `json:"has_response"`
	ResponsePayload *Type      `json:"maybe_response_payload,omitempty"`
	HasError        bool       `json:"has_error"`
	ResultType      *Type      `json:"maybe_response_result_type,omitempty"`
	ValueType       *Type      `json:"maybe_response_success_type,omitempty"`
	ErrorType       *Type      `json:"maybe_response_err_type,omitempty"`
}

// GetRequestPayloadIdentifier retrieves the identifier that points to the
// declaration of the request payload.
func (m *Method) GetRequestPayloadIdentifier() (EncodedCompoundIdentifier, bool) {
	if m.RequestPayload == nil {
		return "", false
	}
	return m.RequestPayload.Identifier, true
}

// GetResponsePayloadIdentifier retrieves the identifier that points to the
// declaration of the response payload.
func (m *Method) GetResponsePayloadIdentifier() (EncodedCompoundIdentifier, bool) {
	if m.ResponsePayload == nil {
		return "", false
	}
	return m.ResponsePayload.Identifier, true
}

func (m *Method) HasRequestPayload() bool {
	return m.RequestPayload != nil
}

func (m *Method) HasResponsePayload() bool {
	return m.ResponsePayload != nil
}

type DeclType string

const (
	LibraryDeclType DeclType = "library"

	ConstDeclType     DeclType = "const"
	BitsDeclType      DeclType = "bits"
	EnumDeclType      DeclType = "enum"
	ProtocolDeclType  DeclType = "interface"
	ServiceDeclType   DeclType = "service"
	StructDeclType    DeclType = "struct"
	TableDeclType     DeclType = "table"
	UnionDeclType     DeclType = "union"
	TypeAliasDelcType DeclType = "type_alias"
)

type DeclInfo struct {
	Type DeclType `json:"kind"`
}

type DeclMap map[EncodedCompoundIdentifier]DeclType
type DeclInfoMap map[EncodedCompoundIdentifier]DeclInfo

// Library represents a FIDL dependency on a separate library.
type Library struct {
	Name  EncodedLibraryIdentifier `json:"name,omitempty"`
	Decls DeclInfoMap              `json:"declarations,omitempty"`
}

// Root is the top-level object for a FIDL library.
// It contains lists of all declarations and dependencies within the library.
type Root struct {
	Name EncodedLibraryIdentifier `json:"name,omitempty"`
	//Consts          []Const                     `json:"const_declarations,omitempty"`
	//Bits            []Bits                      `json:"bits_declarations,omitempty"`
	Enums           []Enum     `json:"enum_declarations,omitempty"`
	Protocols       []Protocol `json:"interface_declarations,omitempty"`
	Structs         []Struct   `json:"struct_declarations,omitempty"`
	ExternalStructs []Struct   `json:"external_struct_declarations,omitempty"`
	Tables          []Table    `json:"table_declarations,omitempty"`
	Unions          []Union    `json:"union_declarations,omitempty"`
	//TypeAliases     []TypeAlias                 `json:"type_alias_declarations,omitempty"`
	DeclOrder    []EncodedCompoundIdentifier `json:"declaration_order,omitempty"`
	Decls        DeclMap                     `json:"declarations,omitempty"`
	Libraries    []Library                   `json:"library_dependencies,omitempty"`
	declarations map[EncodedCompoundIdentifier]Declaration
}

func (r *Root) initializeDeclarationsMap() {
	r.declarations = make(map[EncodedCompoundIdentifier]Declaration)
	//for i, d := range r.Consts {
	//	r.declarations[d.Name] = &r.Consts[i]
	//}
	//for i, d := range r.Bits {
	//	r.declarations[d.Name] = &r.Bits[i]
	//}
	for i, d := range r.Enums {
		r.declarations[d.Name] = &r.Enums[i]
	}
	//for i, d := range r.Protocols {
	//	r.declarations[d.Name] = &r.Protocols[i]
	//}
	//for i, d := range r.Services {
	//	r.declarations[d.Name] = &r.Services[i]
	//}
	for i, d := range r.Structs {
		r.declarations[d.Name] = &r.Structs[i]
	}
	for i, d := range r.Tables {
		r.declarations[d.Name] = &r.Tables[i]
	}
	for i, d := range r.Unions {
		r.declarations[d.Name] = &r.Unions[i]
	}
	for i, d := range r.ExternalStructs {
		r.declarations[d.Name] = &r.ExternalStructs[i]
	}
}

type int64OrUint64 struct {
	i int64
	u uint64
}

func (n *int64OrUint64) readInt64() int64 {
	if n.i != 0 {
		return n.i
	}
	return int64(n.u)
}

func (n *int64OrUint64) readUint64() uint64 {
	if n.i != 0 {
		return uint64(n.i)
	}
	return n.u
}

/*
This file contains types which describe FIDL protocols.

These types are intended to be directly deserialized from the FIDL protocol
JSON representation. The types are then passed directly to language-specific
generators which produce source code.

Note that these are different from a naive AST-based representation of
FIDL text. Before being transformed into JSON, FIDL sources are preprocessed
to generate metadata required by all of the backends, such as the size of
types. Importantly, this removes the need for language-specific backends to
implement field, name, or type resolution and analysis.
*/

// ReadJSONIr reads a JSON IR file.
func ReadJSONIr(filename string) (Root, error) {
	f, err := os.Open(filename)
	if err != nil {
		return Root{}, fmt.Errorf("Error reading from %s: %w", filename, err)
	}
	return DecodeJSONIr(f)
}

// DecodeJSONIr reads the JSON content from a reader.
func DecodeJSONIr(r io.Reader) (Root, error) {
	d := json.NewDecoder(r)
	var root Root
	if err := d.Decode(&root); err != nil {
		return Root{}, fmt.Errorf("Error parsing JSON IR: %w", err)
	}

	root.initializeDeclarationsMap()

	return root, nil
}
