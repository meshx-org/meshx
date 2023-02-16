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

    pub fn new_validation_error(message: &str, span: Span) -> Self {
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
}
