use std::rc::Rc;

use super::helpers::consume_catch_all;

use super::consume_identifier;
use super::{helpers::Pair, Rule};

use crate::ast::{self, Name, Span, TypeConstructor};
use crate::ast::{CompoundIdentifier, Declaration};
use crate::compiler::ParsingContext;
use crate::consumption::consume_attribute_list;
use crate::consumption::consume_comments::{consume_comment_block, consume_trailing_comment};
use crate::consumption::consume_type::consume_type_constructor;
use crate::diagnotics::DiagnosticsError;

fn consume_parameter_list(
    pair: Pair<'_>,
    name_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> Option<TypeConstructor> {
    assert!(pair.as_rule() == Rule::parameter_list);

    let mut maybe_type_ctor = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::PARENT_OPEN | Rule::PARENT_CLOSE => {}
            Rule::type_constructor => {
                maybe_type_ctor = Some(consume_type_constructor(current, &name_context, ctx));
                // declarations.push(struct_layout.into());
            }
            _ => consume_catch_all(&current, "parameter list"),
        }
    }

    maybe_type_ctor
}

fn consume_protocol_request() -> Option<TypeConstructor> {
    None
}

fn consume_protocol_response() -> Option<TypeConstructor> {
    None
}

fn consume_protocol_method(
    pair: Pair<'_>,
    block_comment: Option<Pair<'_>>,
    protocol_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::ProtocolMethod, DiagnosticsError> {
    let pair_span = pair.as_span();
    let mut method_name: Option<ast::Identifier> = None;

    let mut attributes: Vec<ast::Attribute> = Vec::new();
    let mut documentation: Option<ast::Comment> = block_comment.and_then(consume_comment_block);

    let mut maybe_request = None;
    let mut maybe_response = None;
    let mut maybe_error: Option<TypeConstructor> = None;

    let mut has_request = false;
    let mut has_response = false;
    let mut has_error = false;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => method_name = Some(consume_identifier(&current, ctx)),
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics))*/ }
            Rule::protocol_request => {
                let protocol_context = protocol_context.clone();

                has_request = true;
                maybe_request = consume_parameter_list(
                    current.into_inner().next().unwrap(),
                    protocol_context.enter_request(method_name.clone().unwrap().span),
                    ctx,
                );
            }
            Rule::protocol_response => {
                todo!()
                /*

                let response_context = if has_request {
                    protocol_context.enter_response(method_name)
                } else {
                    protocol_context.enter_event(method_name)
                };

                for current in current.into_inner() {
                    match current.as_rule() {
                        Rule::parameter_list => {
                            has_response = true;
                            maybe_request = consume_parameter_list(current, name_context, ctx);
                        }
                        Rule::type_constructor => {
                            has_error = true;
                            maybe_error = Some(consume_type_constructor(current, name_context, ctx));
                        }
                        _ => panic!(""),
                    }
                }
                */
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

    Ok(ast::ProtocolMethod {
        name: method_name.unwrap(),
        documentation,
        attributes,
        request_payload: None,
        response_payload: None,
        has_request,
        has_response,
        has_error,
        maybe_request,
        maybe_response,
        span: ast::Span::from_pest(pair_span, ctx.source_id),
    })
}

fn consume_compose(ctx: &mut ParsingContext<'_>) {}

pub(crate) fn consume_protocol_declaration(
    pair: Pair<'_>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Protocol, DiagnosticsError> {
    let pair_span = pair.as_span();

    let mut identifier: Option<ast::Identifier> = None;
    let mut name: Option<ast::Name> = None;
    let mut name_context = None;

    let mut pending_field_comment: Option<Pair<'_>> = None;
    let mut methods: Vec<Rc<ast::ProtocolMethod>> = Vec::new();
    let mut attributes: Option<ast::AttributeList> = None;
    let mut composes: Vec<CompoundIdentifier> = Vec::new();

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::PROTOCOL_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);
                let sourced = Name::create_sourced(ctx.library.clone(), name_span);

                identifier = Some(consume_identifier(&current, ctx));

                name_context = Some(ast::NamingContext::create(&sourced));
                name = Some(sourced);
            }
            Rule::block_attribute_list => {
                attributes = Some(consume_attribute_list(current, ctx));
            }
            Rule::protocol_method => {
                let name_context = name_context.as_ref().unwrap().clone();

                match consume_protocol_method(current, pending_field_comment.take(), name_context, ctx) {
                    Ok(method) => {
                        methods.push(Rc::from(method));
                    }
                    Err(err) => ctx.diagnostics.push_error(err),
                }
            }
            Rule::protocol_event => {}
            Rule::protocol_compose => consume_compose(ctx),
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => ctx.diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                Span::from_pest(current.as_span(), ctx.source_id),
            )),
            _ => consume_catch_all(&current, "protocol"),
        }
    }

    Ok(ast::Protocol {
        identifier: identifier.unwrap(),
        name: name.unwrap(),
        methods,
        composes,
        attributes: attributes.unwrap(),
        documentation: None,
        span: ast::Span::from_pest(pair_span, ctx.source_id),
        compiled: false,
        compiling: false,
        recursive: false,
    })
}
