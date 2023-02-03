use crate::ast::Span;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct DiagnosticsError {
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
}
