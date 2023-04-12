use std::cell::RefCell;
use std::{collections::HashMap, rc::Rc};

use crate::ast;

#[derive(Debug)]
pub(crate) struct Typespace;

// Libraries manages a set of compiled libraries along with resources common to
// all of them (e.g. the shared typespace). The libraries must be inserted in
// order: first the dependencies, with each one only depending on those that
// came before it, and lastly the target library.
#[derive(Debug)]
pub(crate) struct Libraries {
    // typespace: Rc<Typespace>,
    libraries: Vec<Rc<ast::Library>>,

    // root_library: &'lib ast::Library,
    libraries_by_name: HashMap<ast::CompoundIdentifier, Rc<ast::Library>>,
}

impl Libraries {
    pub(crate) fn new() -> Self {
        Self {
            // typespace: Rc::new(Typespace),
            libraries: vec![],
            libraries_by_name: HashMap::new(),
        }
    }

    /// Insert |library|. It must only depend on already-inserted libraries.
    pub(crate) fn insert(&mut self, library: Rc<ast::Library>) -> bool {
        let library_name = library.name.borrow();

        if library_name.is_none() {
            return false;
        }

        let library_name = library_name.clone().unwrap();

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
    pub(crate) fn lookup(&self, library_name: ast::CompoundIdentifier) -> Option<Rc<ast::Library>> {
        let iter = self.libraries_by_name.iter().find(|(id, _)| **id == library_name);
        iter.map(|(_, lib)| lib.clone())
    }
}
