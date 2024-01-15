use std::rc::Rc;

use super::helpers::consume_catch_all;

use super::consume_identifier;
use super::{helpers::Pair, Rule};

use crate::ast::{self, Name, Span};
use crate::ast::{CompoundIdentifier, Declaration};
use crate::compiler::ParsingContext;
use crate::consumption::consume_attribute_list;
use crate::consumption::consume_comments::{consume_comment_block, consume_trailing_comment};
use crate::consumption::consume_struct::consume_struct_declaration;
use crate::diagnotics::DiagnosticsError;

fn consume_parameter_list(
    pair: Pair<'_>,
    parameter_name: &str,
    declarations: &mut Vec<Declaration>,
    ctx: &mut ParsingContext<'_>,
) -> Result<(), DiagnosticsError> {
    assert!(pair.as_rule() == Rule::parameter_list);

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => {}
            Rule::struct_declaration => {
                let struct_declaration = consume_struct_declaration(
                    current,
                    Some(ast::Identifier {
                        value: String::from(parameter_name),
                        span: ast::Span::empty(),
                    }),
                    ctx,
                )
                .unwrap();
                declarations.push(struct_declaration.into());
            }
            _ => consume_catch_all(&current, "parameter list"),
        }
    }

    Ok(())
}

fn consume_protocol_method(
    protocol_name: &str,
    pair: Pair<'_>,
    block_comment: Option<Pair<'_>>,
    declarations: &mut Vec<Declaration>,
    ctx: &mut ParsingContext<'_>,
) -> Result<(ast::ProtocolMethod, Vec<ast::Declaration>), DiagnosticsError> {
    let pair_span = pair.as_span();
    let mut name: Option<ast::Identifier> = None;
    let attributes: Vec<ast::Attribute> = Vec::new();
    let mut documentation: Option<ast::Comment> = block_comment.and_then(consume_comment_block);

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => name = Some(consume_identifier(&current, ctx)),
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics))*/ }
            Rule::parameter_list => {
                consume_parameter_list(
                    current,
                    format!("{}{}", protocol_name, "Request").as_str(),
                    declarations,
                    ctx,
                )?;
            }
            Rule::trailing_comment => {
                documentation = match (documentation, consume_trailing_comment(current)) {
                    (c, None) | (None, c) => c,
                    (Some(existing), Some(new)) => Some(ast::Comment {
                        text: [existing.text, new.text].join("\n"),
                    }),
                };
            }
            _ => consume_catch_all(&current, "protocol method"),
        }
    }

    match name {
        Some(name) => Ok((
            ast::ProtocolMethod {
                name,
                documentation,
                attributes,
                request_payload: None,
                response_payload: None,
                maybe_request: None,
                maybe_response: None,
                has_error: false,
                has_request: false,
                has_response: false,
                span: ast::Span::from_pest(pair_span, ctx.source_id),
            },
            Vec::new(),
        )),
        _ => panic!("Encountered impossible protocol method declaration during parsing"),
    }
}

fn consume_compose(ctx: &mut ParsingContext<'_>) {}

pub(crate) fn consume_protocol_declaration(
    pair: Pair<'_>,
    ctx: &mut ParsingContext<'_>,
) -> Result<(ast::Protocol, Vec<ast::Declaration>), DiagnosticsError> {
    log::error!("consume_protocol_declaration");

    let pair_span = pair.as_span();

    let mut identifier: Option<ast::Identifier> = None;
    let mut name: Option<ast::Name> = None;
    let mut pending_field_comment: Option<Pair<'_>> = None;
    let mut methods: Vec<Rc<ast::ProtocolMethod>> = Vec::new();
    let mut attributes: Option<ast::AttributeList> = None;
    let mut composes: Vec<CompoundIdentifier> = Vec::new();
    let mut declarations: Vec<Declaration> = Vec::new();

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::PROTOCOL_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);

                identifier = Some(consume_identifier(&current, ctx));
                name = Some(Name::create_sourced(ctx.library.clone(), name_span));
            }
            Rule::block_attribute_list => {
                attributes = Some(consume_attribute_list(current, ctx));
            }
            Rule::protocol_method => match consume_protocol_method(
                &identifier.as_ref().unwrap().value,
                current,
                pending_field_comment.take(),
                &mut declarations,
                ctx,
            ) {
                Ok((method, mut decls)) => {
                    methods.push(Rc::from(method));
                    declarations.append(&mut decls);
                }
                Err(err) => ctx.diagnostics.push_error(err),
            },
            Rule::protocol_event => {}
            Rule::protocol_compose => {
                consume_compose(ctx)
            }
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => ctx.diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                Span::from_pest(current.as_span(), ctx.source_id),
            )),
            _ => consume_catch_all(&current, "protocol"),
        }
    }

    match name {
        Some(name) => Ok((
            ast::Protocol {
                identifier: identifier.unwrap(),
                name,
                methods,
                composes,
                attributes: attributes.unwrap(),
                documentation: None,
                span: ast::Span::from_pest(pair_span, ctx.source_id),
            },
            declarations,
        )),
        _ => panic!("Encountered impossible protocol declaration during parsing",),
    }
}
