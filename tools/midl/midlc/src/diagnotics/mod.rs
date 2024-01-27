mod error;
mod pretty_print;
mod warning;

use std::cell::{Ref, RefCell};

pub(crate) use error::{DiagnosticsError, Error};
pub(crate) use pretty_print::{pretty_print_error_text, DiagnosticColorer};
pub(crate) use warning::DiagnosticsWarning;

pub(crate) struct Counts<'d> {
    diagnostics: &'d Diagnostics,
    num_errors: usize,
}

impl<'d> Counts<'d> {
    fn new(diagnostics: &'d Diagnostics) -> Self {
        Self {
            diagnostics,
            num_errors: diagnostics.errors().len(),
        }
    }

    pub(crate) fn num_new_errors(&self) -> usize {
        self.diagnostics.errors().len() - self.num_errors
    }

    pub(crate) fn no_new_errors(&self) -> bool {
        self.num_new_errors() == 0
    }
}

/// Represents a list of validation or parser errors and warnings.
///
/// This is used to accumulate multiple errors and warnings during validation.
/// It is used to not error out early and instead show multiple errors at once.
#[derive(Debug)]
pub(crate) struct Diagnostics {
    errors: RefCell<Vec<DiagnosticsError>>,
    warnings: RefCell<Vec<DiagnosticsWarning>>,
}

impl Diagnostics {
    pub(crate) fn new() -> Diagnostics {
        Diagnostics {
            errors: RefCell::new(Vec::new()),
            warnings: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn warnings(&self) -> Ref<'_, Vec<DiagnosticsWarning>> {
        self.warnings.borrow()
    }

    pub(crate) fn errors(&self) -> Ref<'_, Vec<DiagnosticsError>> {
        self.errors.borrow()
    }

    pub(crate) fn push_error(&self, err: DiagnosticsError) {
        self.errors.borrow_mut().push(err)
    }

    pub(crate) fn push_warning(&self, warning: DiagnosticsWarning) {
        self.warnings.borrow_mut().push(warning)
    }

    /// Returns true, if there is at least one error in this collection.
    pub(crate) fn has_errors(&self) -> bool {
        !self.errors.borrow().is_empty()
    }

    pub(crate) fn checkpoint(&self) -> Counts<'_> {
        Counts::new(self)
    }
}

impl From<DiagnosticsError> for Diagnostics {
    fn from(error: DiagnosticsError) -> Self {
        let mut col = Diagnostics::new();
        col.push_error(error);
        col
    }
}

impl From<DiagnosticsWarning> for Diagnostics {
    fn from(warning: DiagnosticsWarning) -> Self {
        let mut col = Diagnostics::new();
        col.push_warning(warning);
        col
    }
}
