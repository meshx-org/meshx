struct StructMember;
struct StructPadding;

// Struct represents a struct declaration.
struct Struct {
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

struct Import;
struct Const;
struct Enum;
struct Bits;
struct Protocol;
struct Table;
struct Union;

// Root holds all of the declarations for a MIDL library.
#[derive(Default)]
pub struct Root {
    library_name: String,
    imports: Vec<Import>,
    consts: Vec<Const>,
    enums: Vec<Enum>,
    bits: Vec<Bits>,
    protocols: Vec<Protocol>,
    structs: Vec<Struct>,
    tables: Vec<Table>,
    unions: Vec<Union>,
    external_structs: Vec<Struct>,
}

struct Compiler {
    root: Root,
    decls: midlgen::DeclInfoMap,
    library: midlgen::LibraryIdentifier,
    // types_root: midlgen::Root,
    // paramable_types: HashMap<midlgen::EncodedCompoundIdentifier, Parameterizer>
}

fn format_library_name(library: midlgen::LibraryIdentifier) -> String {
    library
        .into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join("_")
}

// Compile the language independent type definition into the Dart-specific representation.
pub fn compile(r: midlgen::Root) -> Root {
    let mut c = Compiler {
        decls: r.decl_info(),
        root: Root::default(),
        // experiments: .experiments,
        // types_root: r,
        library: midlgen::LibraryIdentifier::from(r.name),
        // paramableTypes: map[fidlgen.EncodedCompoundIdentifier]Parameterizer{},
    };

    c.root.library_name = format!("fidl_{}", format_library_name(c.library));

    return c.root;
}
