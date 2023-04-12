use colored::{ColoredString, Colorize};

use crate::ast::Span;
use std::borrow::Cow;

use super::pretty_print::{pretty_print, DiagnosticColorer};

#[derive(Debug, Clone)]
pub(crate) struct DiagnosticsError {
    span: Span,
    message: Cow<'static, str>,
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

    pub fn span(&self) -> Span {
        self.span
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
