use std::{collections::HashMap, rc::Rc};

use crate::ast::{self, Library};

#[derive(Debug)]
pub(crate) struct Typespace;

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
        Self {
            typespace: Rc::new(Typespace),
            libraries: vec![],
            libraries_by_name: HashMap::new(),
            root_library: Library::new_root(),
        }
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
}
