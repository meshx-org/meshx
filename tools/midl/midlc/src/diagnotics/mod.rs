mod error;

pub use error::DiagnosticsError;

#[derive(Debug)]
pub(crate) struct DiagnosticsWarning;

/// Represents a list of validation or parser errors and warnings.
///
/// This is used to accumulate multiple errors and warnings during validation.
/// It is used to not error out early and instead show multiple errors at once.
#[derive(Debug, Default)]
pub(crate) struct Diagnostics {
    errors: Vec<DiagnosticsError>,
    warnings: Vec<DiagnosticsWarning>,
}

impl Diagnostics {
    pub fn new() -> Diagnostics {
        Diagnostics {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn warnings(&self) -> &[DiagnosticsWarning] {
        &self.warnings
    }

    pub fn errors(&self) -> &[DiagnosticsError] {
        &self.errors
    }

    pub fn push_error(&mut self, err: DiagnosticsError) {
        self.errors.push(err)
    }

    pub fn push_warning(&mut self, warning: DiagnosticsWarning) {
        self.warnings.push(warning)
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
