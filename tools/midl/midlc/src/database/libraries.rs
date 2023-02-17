use std::{collections::HashMap, rc::Rc};

use crate::ast;

#[derive(Debug)]
pub(crate) struct Typespace;

// Libraries manages a set of compiled libraries along with resources common to
// all of them (e.g. the shared typespace). The libraries must be inserted in
// order: first the dependencies, with each one only depending on those that
// came before it, and lastly the target library.
#[derive(Debug)]
pub(crate) struct Libraries<'lib> {
    // typespace: Rc<Typespace>,
    libraries: Vec<ast::Library>,

    // root_library: &'lib ast::Library,
    libraries_by_name: HashMap<ast::CompoundIdentifier, &'lib ast::Library>,
}

impl<'lib> Libraries<'lib> {
    pub(crate) fn new() -> Self {
        Self {
            // typespace: Rc::new(Typespace),
            libraries: vec![],
            libraries_by_name: HashMap::new(),
        }
    }

    /// Insert |library|. It must only depend on already-inserted libraries.
    pub(crate) fn insert(library: Box<ast::Library>) -> bool {
        false
    }

    /// Lookup a library by its |library_name|, or returns null if none is found.
    pub(crate) fn lookup(&self, library_name: ast::CompoundIdentifier) -> Option<&'lib ast::Library> {
        let iter = self.libraries_by_name.iter().find(|(id, _)| **id == library_name);
        iter.map(|(_, lib)| *lib)
    }
}
