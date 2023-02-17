use crate::ast::Library;
use crate::diagnotics::{Diagnostics, DiagnosticsError};
use crate::source_file::SourceId;

use super::libraries::Libraries;
use super::references::References;

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
pub(crate) struct Context<'lib, 'db> {
    pub(crate) ast: &'db Library,

    pub(crate) all_libraries: &'lib Libraries<'lib>,

    pub(crate) diagnostics: &'db mut Diagnostics,
    pub(crate) references: &'db mut References,
}

impl<'lib, 'db> Context<'lib, 'db> {
    pub(super) fn new(
        ast: &'db Library,
        all_libraries: &'lib Libraries<'lib>,
        references: &'db mut References,
        diagnostics: &'db mut Diagnostics,
    ) -> Self {
        Context {
            ast,
            diagnostics,
            all_libraries,
            references,
        }
    }

    pub(super) fn push_error(&mut self, error: DiagnosticsError) {
        self.diagnostics.push_error(error)
    }
}

/// Parsing context. This is an implementation detail of ParserDatabase. It
/// contains the database itself, as well as context that is discarded after
/// parsing is done.
pub(crate) struct ParsingContext<'lib, 'db> {
    pub(crate) all_libraries: &'lib Libraries<'lib>,
    pub(crate) diagnostics: &'db mut Diagnostics,
    pub(crate) source_id: SourceId,
}

impl<'lib, 'db> ParsingContext<'lib, 'db> {
    pub(super) fn new(
        all_libraries: &'lib Libraries<'lib>,
        diagnostics: &'db mut Diagnostics,
        source_id: SourceId,
    ) -> Self {
        ParsingContext {
            source_id,
            diagnostics,
            all_libraries,
        }
    }

    pub(super) fn push_error(&mut self, error: DiagnosticsError) {
        self.diagnostics.push_error(error)
    }
}
