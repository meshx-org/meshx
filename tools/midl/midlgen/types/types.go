package types



// Location gives the location of the MIDL declaration in its source `.midl` file.
type Location struct {
	Filename string `json:"filename"`
	Line     int    `json:"line"`
	Column   int    `json:"column"`
	Length   int    `json:"length"`
}

type PrimitiveSubtype string

const (
	Bool    PrimitiveSubtype = "bool"    // bool
	Int8    PrimitiveSubtype = "int8"    // byte
	Int16   PrimitiveSubtype = "int16"   // short
	Int32   PrimitiveSubtype = "int32"   // init
	Int64   PrimitiveSubtype = "int64"   // long
	Uint8   PrimitiveSubtype = "uint8"   // ubyte
	Uint16  PrimitiveSubtype = "uint16"  // ushort
	Uint32  PrimitiveSubtype = "uint32"  // uint
	Uint64  PrimitiveSubtype = "uint64"  // ulong
	Float32 PrimitiveSubtype = "float32" // float
	Float64 PrimitiveSubtype = "float64" // double
)

type HandleSubtype string

const (
	Handle   HandleSubtype = "handle"
	Channel  HandleSubtype = "channel"
	Job      HandleSubtype = "job"
	Process  HandleSubtype = "process"
	Resource HandleSubtype = "resource"
)

type ConstantKind string

type LiteralKind string

const (
	StringLiteral  LiteralKind = "string"
	NumericLiteral LiteralKind = "numeric"
	BoolLiteral    LiteralKind = "bool"
	DefaultLiteral LiteralKind = "default"
)

type Literal struct {
	Kind  LiteralKind `json:"kind"`
	Value string      `json:"value,omitempty"`
}

const (
	IdentifierConstant ConstantKind = "identifier"
	LiteralConstant    ConstantKind = "literal"
	BinaryOperator     ConstantKind = "binary_operator"
)

type Constant struct {
	Kind       ConstantKind              `json:"kind"`
	Identifier EncodedCompoundIdentifier `json:"identifier,omitempty"`
	Literal    Literal                   `json:"literal,omitempty"`
	Value      string                    `json:"value"`
}
