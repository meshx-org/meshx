use colored::{ColoredString, Colorize};
use crate::diagnotics::{pretty_print_error_text, DiagnosticColorer};

pub struct ErrorColorer {}

impl DiagnosticColorer for ErrorColorer {
    fn title(&self) -> &'static str {
        "error:"
    }

    fn primary_color(&self, token: &'_ str) -> ColoredString {
        token.red()
    }
}