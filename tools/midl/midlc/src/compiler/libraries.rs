use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use crate::{
    ast::{self, Declaration, Library},
    compiler::Dependency,
    diagnotics::Diagnostics,
};

use super::{typespace::Typespace, Compilation};

/// Helper struct to calculate Compilation::direct_and_composed_dependencies.
#[derive(PartialEq, Eq, PartialOrd)]
struct LibraryCompare(Rc<ast::Library>);

impl Ord for LibraryCompare {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

struct CalcDependencies;

impl CalcDependencies {
    fn from(roots: &Vec<ast::Declaration>) -> BTreeSet<LibraryCompare> {
        BTreeSet::new()
    }
}

// Libraries manages a set of compiled libraries along with resources common to
// all of them (e.g. the shared typespace). The libraries must be inserted in
// order: first the dependencies, with each one only depending on those that
// came before it, and lastly the target library.
#[derive(Debug)]
pub(crate) struct Libraries {
    typespace: Rc<Typespace>,
    libraries: Vec<Rc<ast::Library>>,
    libraries_by_name: HashMap<Vec<String>, Rc<ast::Library>>,
    root_library: Rc<ast::Library>,
}

impl Libraries {
    pub(crate) fn new() -> Self {
        let root_library = Library::new_root();

        Self {
            typespace: Rc::new(Typespace::new(root_library.clone(), Rc::from(Diagnostics::new()))),
            libraries: Vec::new(),
            libraries_by_name: HashMap::new(),
            root_library,
        }
    }

    pub fn typespace(&self) -> Rc<Typespace> {
        self.typespace.clone()
    }

    /// Insert |library|. It must only depend on already-inserted libraries.
    pub(crate) fn insert(&mut self, library: Rc<ast::Library>) -> bool {
        if library.name.get().is_none() {
            return false;
        }

        let library_name = library.name.get().expect("initialized library name").clone();

        let multiple_entry = self.libraries_by_name.contains_key(&library_name);
        if multiple_entry {
            return false; // Fail(ErrMultipleLibrariesWithSameName, library.arbitrary_name_span, library.name);
        }

        self.libraries.push(library.clone());
        let library = self.libraries.last().unwrap();
        self.libraries_by_name.insert(library_name, library.clone());

        return true;
    }

    /// Lookup a library by its |library_name|, or returns null if none is found.
    pub(crate) fn lookup(&self, library_name: Vec<String>) -> Option<Rc<ast::Library>> {
        let iter = self.libraries_by_name.iter().find(|(id, _)| **id == library_name);
        iter.map(|(_, lib)| lib.clone())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.libraries.is_empty()
    }

    // Returns the root library, which defines builtin types.
    pub fn root_library(&self) -> Rc<Library> {
        self.root_library.clone()
    }

    pub fn filter(&self, version_selection: u32) -> Compilation {
        assert!(!self.libraries.is_empty());

        /*fn keep(decl: Declaration) -> bool{
            // Copies decl pointers for which keep() returns true from src to dst.
            return decl.availability.range().Contains(
            version_selection.lookup(decl.name.library().platform.value()))
        } */
        fn keep(decl: &Declaration) -> bool {
            true
        }

        fn filter_internal(dst: &mut Vec<Declaration>, src: Vec<Declaration>) {
            for decl in src.iter() {
                if keep(decl) {
                    dst.push(decl.clone());
                }
            }
        }

        /// Filters a ast::Declarations into a compiler::Declarations.
        fn filter_declarations(dst: &mut super::Declarations, src: &RefCell<ast::Declarations>) {
            let src = src.borrow().clone();

            //filter_internal(&dst.bits, src.bits);
            filter_internal(&mut dst.builtins, src.builtins);
            filter_internal(&mut dst.consts, src.consts);
            filter_internal(&mut dst.enums, src.enums);
            //filter_internal(&dst.new_types, src.new_types);
            filter_internal(&mut dst.protocols, src.protocols);
            filter_internal(&mut dst.resources, src.resources);
            //filter_internal(&dst.services, src.services);
            filter_internal(&mut dst.structs, src.structs);
            //filter_internal(&dst.tables, src.tables);
            //filter_internal(&dst.aliases, src.aliases);
            filter_internal(&mut dst.unions, src.unions);
            //filter_internal(&dst.overlays, src.overlays);
        }

        let mut declarations = super::Declarations::default();
        let mut declaration_order = vec![];
        let mut direct_and_composed_dependencies = vec![];

        let library = self.libraries.last().unwrap();
        let library_name = library.name.get().unwrap().clone();
        //let library_declarations = library.library_name_declarations;
        //let library_attributes = library.attributes.get();

        filter_declarations(&mut declarations, &library.declarations);

        // let external_structs = ExternalStructs(library, declarations.protocols);
        // TODO let using_references = library.dependencies.library_references();

        filter_internal(&mut declaration_order, library.declaration_order.clone());

        let mut dependencies = CalcDependencies::from(&declaration_order);
        dependencies.remove(&LibraryCompare(library.clone()));
        dependencies.remove(&LibraryCompare(self.root_library()));

        for dep_library in dependencies {
            let mut declarations = super::Declarations::default();
            filter_declarations(&mut declarations, &dep_library.0.declarations);

            direct_and_composed_dependencies.push(Dependency {
                library: dep_library.0,
                declarations,
            });
        }

        Compilation {
            library_name,
            declaration_order,
            direct_and_composed_dependencies,
            version_selection,
            declarations,
            external_structs: vec![],
        }
    }
}
