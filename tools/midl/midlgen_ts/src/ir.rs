use crate::types::Root;


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
        library: midlgen::LibraryIdentifier::from(midlgen::EncodedLibraryIdentifier("".to_owned())),
        // paramableTypes: map[fidlgen.EncodedCompoundIdentifier]Parameterizer{},
    };

    c.root.library_name = format!("fidl_{}", format_library_name(c.library));

    return c.root;
}
