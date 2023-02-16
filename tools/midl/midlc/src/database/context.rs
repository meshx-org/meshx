use crate::ast::SchamaAST;
use crate::diagnotics::{Diagnostics, DiagnosticsError};

use super::references::{self, References};

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
    pub(crate) ast: &'db SchamaAST,
    pub(crate) diagnostics: &'db mut Diagnostics,

    pub(crate) references: &'db mut References,
}

impl<'db> Context<'db> {
    pub(super) fn new(ast: &'db SchamaAST, references: &'db mut References, diagnostics: &'db mut Diagnostics) -> Self {
        Context { ast, diagnostics, references }
    }

    pub(super) fn push_error(&mut self, error: DiagnosticsError) {
        self.diagnostics.push_error(error)
    }
}
