package types

// TypeShape represents the shape of the type on the wire.
// See JSON IR schema, e.g. fidlc --json-schema
type TypeShape struct {
	InlineSize          int  `json:"inline_size"`
	Alignment           int  `json:"alignment"`
	Depth               int  `json:"depth"`
	MaxHandles          int  `json:"max_handles"`
	MaxOutOfLine        int  `json:"max_out_of_line"`
	HasPadding          bool `json:"has_padding"`
	HasFlexibleEnvelope bool `json:"has_flexible_envelope"`
}

// FieldShape represents the shape of the field on the wire.
// See JSON IR schema, e.g. fidlc --json-schema
type FieldShape struct {
	Offset  int `json:"offset"`
	Padding int `json:"padding"`
}

type TypeKind string

const (
	ArrayType      TypeKind = "array"
	VectorType     TypeKind = "vector"
	StringType     TypeKind = "string"
	HandleType     TypeKind = "handle"
	// RequestType    TypeKind = "request"
	PrimitiveType  TypeKind = "primitive"
	IdentifierType TypeKind = "identifier"
)

type Type struct {
	Kind          TypeKind
	ElementType   *Type
	ElementCount  *int
	HandleSubtype HandleSubtype
	// HandleRights      HandleRights
	RequestSubtype    EncodedCompoundIdentifier
	PrimitiveSubtype  PrimitiveSubtype
	Identifier        EncodedCompoundIdentifier
	Nullable          bool
	ProtocolTransport string
	ObjType           uint32
	TypeShapeV1       TypeShape
	TypeShapeV2       TypeShape
}
