use colored::{ColoredString, Colorize};

use crate::ast::Span;
use std::borrow::Cow;

use super::pretty_print::{pretty_print, DiagnosticColorer};

#[derive(Debug, Clone)]
pub(crate) struct DiagnosticsError {
    span: Span,
    message: Cow<'static, str>,
}

pub(crate) enum Error {
    StrictUnionMustHaveNonReservedMember { span: Span },
    DuplicateUnionMemberOrdinal { span: Span, prev: Span },
    NonDenseOrdinal { span: Span, ordinal: u64 },
    OrdinalOutOfBound { span: Span },
    OrdinalsMustStartAtOne { span: Span }
}

impl From<Error> for DiagnosticsError {
    fn from(item: Error) -> Self {
        match item {
            Error::StrictUnionMustHaveNonReservedMember { span } => DiagnosticsError {
                message: "strict unions must have at least one non-reserved member".into(),
                span,
            },
            Error::NonDenseOrdinal { span, ordinal } => DiagnosticsError {
                message: format!("missing ordinal {ordinal} (ordinals must be dense); consider marking it reserved")
                    .into(),
                span,
            },
            Error::DuplicateUnionMemberOrdinal { span, prev } => DiagnosticsError {
                message: format!(
                    "multiple union fields with the same ordinal; previous was at {}",
                    prev.data
                )
                .into(),
                span,
            },
            Error::OrdinalOutOfBound { span } => DiagnosticsError {
                message: format!("ordinal out-of-bound").into(),
                span,
            },
            Error::OrdinalsMustStartAtOne { span } => DiagnosticsError {
                message: format!("ordinals must start at 1").into(),
                span,
            },
        }
    }
}

impl DiagnosticsError {
    pub fn new(message: impl Into<Cow<'static, str>>, span: Span) -> Self {
        let message = message.into();
        DiagnosticsError { message, span }
    }

    pub fn new_static(message: &'static str, span: Span) -> Self {
        Self::new(message, span)
    }

    pub fn new_unknown_library(message: &str, span: Span) -> Self {
        Self::new(format!("UnknownLibrary: {message}"), span)
    }

    pub fn new_name_not_found(span: Span, name: &String, library_name: &Vec<String>) -> Self {
        Self::new(
            format!("Cannot find '{}' in library '{}'", name, library_name.join(".")),
            span,
        )
    }

    pub fn new_unknown_dependent_library(
        span: Span,
        long_library_name: &Vec<String>,
        short_library_name: &Vec<String>,
    ) -> Self {
        Self::new(
            format!(
                "Unknown dependent library {} or reference to member of library {}. Did you imported it with `import`?",
                long_library_name.join("."),
                short_library_name.join(".")
            ),
            span,
        )
    }

    pub fn new_duplicate_import(message: &str, span: Span) -> Self {
        Self::new(format!("Error importing: {message}"), span)
    }

    pub fn new_conflicting_aliased_import(message: &str, span: Span) -> Self {
        Self::new(format!("Error importing: {message}"), span)
    }

    pub fn new_conflicting_import(message: &str, span: Span) -> Self {
        Self::new(format!("Error importing: {message}"), span)
    }

    pub fn new_validation_error(message: &str, span: Span) -> Self {
        Self::new(format!("Error validating: {message}"), span)
    }

    pub fn new_import_error(message: &str, span: Span) -> Self {
        Self::new(format!("Error validating: {message}"), span)
    }

    pub fn new_protocol_validation_error(
        message: &str,
        block_type: &'static str,
        protocol_name: &str,
        span: Span,
    ) -> DiagnosticsError {
        Self::new(
            format!("Error validating {block_type} \"{protocol_name}\": {message}"),
            span,
        )
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn pretty_print(&self, f: &mut dyn std::io::Write, file_name: &str, text: &str) -> std::io::Result<()> {
        pretty_print(
            f,
            file_name,
            text,
            self.span(),
            self.message.as_ref(),
            &DiagnosticsErrorColorer {},
        )
    }
}

struct DiagnosticsErrorColorer {}

impl DiagnosticColorer for DiagnosticsErrorColorer {
    fn title(&self) -> &'static str {
        "error"
    }

    fn primary_color(&self, token: &'_ str) -> ColoredString {
        token.red()
    }
}
