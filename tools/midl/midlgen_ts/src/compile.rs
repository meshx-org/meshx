use crate::types::Root;
use midlgen::ir;

struct Compiler {
    root: Root,
    decls: ir::DeclInfoMap,
    library: ir::LibraryIdentifier,
    // types_root: midlgen::Root,
    // paramable_types: HashMap<midlgen::EncodedCompoundIdentifier, Parameterizer>
}

fn format_library_name(library: ir::LibraryIdentifier) -> String {
    library.encode().0
}

// Compile the language independent type definition into the Dart-specific representation.
pub fn compile(ir: ir::Root) -> Root {
    let mut c = Compiler {
        decls: ir.decl_info(),
        root: Root::default(),
        // experiments: .experiments,
        // types_root: r,
        library: ir::LibraryIdentifier::from(ir::EncodedLibraryIdentifier("".to_owned())),
        // paramableTypes: map[fidlgen.EncodedCompoundIdentifier]Parameterizer{},
    };

    c.root.library_name = format!("fidl_{}", format_library_name(c.library));

    return c.root;
}
