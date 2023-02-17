use super::helpers::parsing_catch_all;

use super::{helpers::Pair, Rule};
use super::{parse_compound_identifier, parse_identifier};

use crate::ast::{self, Span};
use crate::ast::{CompoundIdentifier, Declaration, ProtocolMethod};
use crate::database::ParsingContext;
use crate::diagnotics::{Diagnostics, DiagnosticsError};
use crate::parse::parse_comments::{parse_comment_block, parse_trailing_comment};
use crate::parse::parse_struct::parse_struct_declaration;
use crate::source_file::SourceId;

fn parse_parameter_list(
    pair: Pair<'_>,
    parameter_name: &str,
    declarations: &mut Vec<Declaration>,
    ctx: &mut ParsingContext<'_, '_>,
) -> Result<(), DiagnosticsError> {
    assert!(pair.as_rule() == Rule::parameter_list);

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => {}
            Rule::struct_declaration => {
                let struct_declaration = parse_struct_declaration(
                    current,
                    Some(ast::Identifier {
                        value: String::from(parameter_name),
                        span: ast::Span::empty(),
                    }),
                    ctx,
                )
                .unwrap();
                declarations.push(ast::Declaration::Struct(struct_declaration));
            }
            _ => parsing_catch_all(&current, "parameter list"),
        }
    }

    Ok(())
}

fn parse_protocol_method(
    protocol_name: &str,
    container_type: &'static str,
    pair: Pair<'_>,
    block_comment: Option<Pair<'_>>,
    declarations: &mut Vec<Declaration>,
    ctx: &mut ParsingContext<'_, '_>,
) -> Result<(ast::ProtocolMethod, Vec<ast::Declaration>), DiagnosticsError> {
    let pair_span = pair.as_span();
    let mut name: Option<ast::Identifier> = None;
    let mut attributes: Vec<ast::Attribute> = Vec::new();
    let mut documentation: Option<ast::Comment> = block_comment.and_then(parse_comment_block);

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => name = Some(parse_identifier(&current, ctx)),
            Rule::block_attribute_list => {
                /*attributes.push(parse_attribute(current, diagnostics))*/
            }
            Rule::parameter_list => {
                parse_parameter_list(
                    current,
                    format!("{}{}", protocol_name, "Request").as_str(),
                    declarations,
                    ctx,
                )?;
            }
            Rule::trailing_comment => {
                documentation = match (documentation, parse_trailing_comment(current)) {
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
        Some(name) => Ok((
            ast::ProtocolMethod {
                name,
                documentation,
                attributes,
                request_payload: None,
                response_payload: None,
                span: ast::Span::from_pest(pair_span, ctx.source_id),
            },
            Vec::new(),
        )),
        _ => panic!("Encountered impossible protocol method declaration during parsing"),
    }
}

pub(crate) fn parse_protocol_declaration(
    pair: Pair<'_>,
    ctx: &mut ParsingContext<'_, '_>,
) -> Result<(ast::Protocol, Vec<ast::Declaration>), DiagnosticsError> {
    let pair_span = pair.as_span();

    let mut name: Option<ast::Identifier> = None;
    let mut pending_field_comment: Option<Pair<'_>> = None;
    let mut methods: Vec<ProtocolMethod> = Vec::new();
    let mut composes: Vec<CompoundIdentifier> = Vec::new();
    let mut attributes: Vec<ast::Attribute> = Vec::new();
    let mut declarations: Vec<Declaration> = Vec::new();

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::PROTOCOL_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => name = Some(parse_identifier(&current, ctx)),
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics)) */ }
            Rule::protocol_method => match parse_protocol_method(
                &name.as_ref().unwrap().value,
                "protocol",
                current,
                pending_field_comment.take(),
                &mut declarations,
                ctx,
            ) {
                Ok((method, mut decls)) => {
                    methods.push(method);
                    declarations.append(&mut decls);
                }
                Err(err) => ctx.diagnostics.push_error(err),
            },
            Rule::protocol_event => {}
            Rule::protocol_compose => match current.into_inner().next() {
                Some(id_pair) => composes.push(parse_compound_identifier(&id_pair, ctx)),
                None => panic!("Expected a compound identifier."),
            },
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => ctx.diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                Span::from_pest(current.as_span(), ctx.source_id),
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
                attributes,
                documentation: None,
                span: ast::Span::from_pest(pair_span, ctx.source_id),
            },
            declarations,
        )),
        _ => panic!("Encountered impossible protocol declaration during parsing",),
    }
}
