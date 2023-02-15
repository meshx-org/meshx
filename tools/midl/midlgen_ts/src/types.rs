use midlgen::{CompoundIdentifier, Identifier};

#[derive(Default, Debug)]
pub struct Import;
#[derive(Default, Debug)]
pub struct Const;
#[derive(Default, Debug)]
pub struct Enum;
#[derive(Default, Debug)]
pub struct Bits;
#[derive(Default, Debug)]
pub struct Protocol;
#[derive(Default, Debug)]
pub struct Table;
#[derive(Default, Debug)]
pub struct Union;

#[derive(Default, Debug)]
pub struct StructMember;
#[derive(Default, Debug)]
pub struct StructPadding;

// Struct represents a struct declaration.
#[derive(Debug)]
pub struct Struct {
    ir: midlgen::Struct,
    doc_comments: Vec<String>,
    name: String,
    members: Vec<StructMember>,
    paddings: Vec<StructPadding>,
    type_symbol: String,
    type_expr: String,
    has_nullable_field: bool,
    is_empty_struct: bool,
}

// Root holds all of the declarations for a MIDL library.
#[derive(Default, Debug)]
pub struct Root {
    pub library_name: String,
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
