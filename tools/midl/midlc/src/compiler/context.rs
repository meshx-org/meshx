use std::cell::RefCell;
use std::rc::Rc;

use crate::ast;
use crate::diagnotics::{Diagnostics, DiagnosticsError};
use crate::source_file::SourceId;

use super::libraries::Libraries;

/// Validation context. This is an implementation detail of ParserDatabase. It
/// contains the database itself, as well as context that is discarded after
/// validation.
///
/// ## Attribute Validation
///
/// The Context also acts as a state machine for attribute validation. The goal is to avoid manual
/// work validating things that are valid for every attribute set, and every argument set inside an
/// attribute: multiple unnamed arguments are not valid, attributes we do not use in parser-database
/// are not valid, multiple arguments with the same name are not valid, etc.
///
/// See `visit_attributes()`.
pub(crate) struct Context<'db> {
    pub(crate) library: Rc<ast::Library>,
    pub(crate) all_libraries: Rc<RefCell<Libraries>>,
    pub(crate) diagnostics: &'db mut Diagnostics,
}

impl<'db> Context<'db> {
    pub(super) fn new(
        library: Rc<ast::Library>,
        all_libraries: Rc<RefCell<Libraries>>,
        diagnostics: &'db mut Diagnostics,
    ) -> Self {
        Context {
            library,
            all_libraries,
            diagnostics,
        }
    }

    pub(super) fn push_error(&mut self, error: DiagnosticsError) {
        self.diagnostics.push_error(error)
    }
}

/// Parsing context. This is an implementation detail of ParserDatabase. It
/// contains the database itself, as well as source context that is discarded after
/// parsing is done.
pub(crate) struct ParsingContext<'db> {
    pub(crate) library: Rc<ast::Library>,
    pub(crate) all_libraries: Rc<RefCell<Libraries>>,
    pub(crate) diagnostics: &'db mut Diagnostics,
    pub(crate) source_id: SourceId,
}

impl<'db> ParsingContext<'db> {
    pub(super) fn new(
        library: Rc<ast::Library>,
        all_libraries: Rc<RefCell<Libraries>>,
        diagnostics: &'db mut Diagnostics,
        source_id: SourceId,
    ) -> Self {
        ParsingContext {
            library,
            source_id,
            diagnostics,
            all_libraries,
        }
    }

    pub(super) fn push_error(&mut self, error: DiagnosticsError) {
        self.diagnostics.push_error(error)
    }
}