use super::helpers::parsing_catch_all;

use super::{helpers::Pair, Rule};

use crate::ast::{CompoundIdentifier, Identifier, ProtocolMethod};
use crate::diagnotics::{Diagnostics, DiagnosticsError};
use crate::parse::parse_comments::{parse_comment_block, parse_trailing_comment};
use crate::{ast, error::ParserError};

fn parse_protocol_method(
    model_name: &str,
    container_type: &'static str,
    pair: Pair<'_>,
    block_comment: Option<Pair<'_>>,
    diagnostics: &mut Diagnostics,
) -> Result<ast::ProtocolMethod, DiagnosticsError> {
    let pair_span = pair.as_span();
    let mut name: Option<ast::Identifier> = None;
    let mut attributes: Vec<ast::Attribute> = Vec::new();
    let mut comment: Option<ast::Comment> = block_comment.and_then(parse_comment_block);

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => name = Some(current.into()),
            Rule::parameter_list => {}
            Rule::trailing_comment => {
                comment = match (comment, parse_trailing_comment(current)) {
                    (c, None) | (None, c) => c,
                    (Some(existing), Some(new)) => Some(ast::Comment {
                        text: [existing.text, new.text].join("\n"),
                    }),
                };
            }
            _ => parsing_catch_all(&current, "protocol method"),
        }
    }

    match name {
        Some(name) => Ok(ast::ProtocolMethod {
            name,
            documentation: None,
            request_payload: None,
            response_payload: None,
            span: ast::Span::from(pair_span),
        }),
        _ => Err(DiagnosticsError::new_protocol_validation_error(
            "This protocol method declaration is invalid. It is missing a name.",
            container_type,
            model_name,
            pair_span.into(),
        )),
    }
}

pub(crate) fn parse_protocol_declaration(
    pair: Pair<'_>,
    diagnostics: &mut Diagnostics,
) -> Result<(ast::Protocol, Vec<ast::Struct>), ParserError> {
    let pair_span = pair.as_span();

    let mut name: Option<ast::Identifier> = None;
    let mut pending_field_comment: Option<Pair<'_>> = None;
    let mut methods: Vec<ProtocolMethod> = Vec::new();
    let mut composes: Vec<CompoundIdentifier> = Vec::new();

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::PROTOCOL_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => name = Some(current.into()),
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics)) */ }
            Rule::protocol_method => match parse_protocol_method(
                &name.as_ref().unwrap().value,
                "model",
                current,
                pending_field_comment.take(),
                diagnostics,
            ) {
                Ok(method) => methods.push(method),
                Err(err) => diagnostics.push_error(err),
            },
            Rule::protocol_event => {}
            Rule::protocol_compose => match current.into_inner().next() {
                Some(id) => composes.push(id.into()),
                None => return Err(ParserError::UnexpectedToken),
            },
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                current.as_span().into(),
            )),
            _ => parsing_catch_all(&current, "protocol"),
        }
    }

    match name {
        Some(name) => Ok((
            ast::Protocol {
                name,
                methods,
                composes,
                attributes: vec![],
                documentation: None,
                span: ast::Span::from(pair_span),
            },
            Vec::new(),
        )),
        _ => Err(ParserError::UnexpectedToken),
    }
}
