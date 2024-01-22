use crate::diagnotics::DiagnosticColorer;
use colored::{ColoredString, Colorize};

pub struct ErrorColorer {}

impl DiagnosticColorer for ErrorColorer {
    fn title(&self) -> &'static str {
        "error:"
    }

    fn primary_color(&self, token: &'_ str) -> ColoredString {
        token.red()
    }
}
