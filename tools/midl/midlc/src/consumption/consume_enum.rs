use std::cell::RefCell;
use std::rc::Rc;

use super::consume_identifier;
use super::consume_type::consume_type_constructor;
use super::helpers::consume_catch_all;
use super::{helpers::Pair, Rule};

use crate::ast;
use crate::compiler::ParsingContext;
use crate::consumption::consume_comments::{consume_comment_block, consume_trailing_comment};
use crate::consumption::consume_const::consume_constant;
use crate::consumption::identifier_type_for_decl;
use crate::diagnotics::DiagnosticsError;

fn consume_enum_member(
    pair: Pair<'_>,
    block_comment: Option<Pair<'_>>,
    name_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::EnumMember, DiagnosticsError> {
    debug_assert!(pair.as_rule() == Rule::value_layout_member);

    let pair_span = pair.as_span();
    let mut name = None;
    let attributes = Vec::new();
    let mut comment = block_comment.and_then(consume_comment_block);
    let mut member_value = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => name = Some(consume_identifier(&current, ctx)),
            Rule::constant => {
                member_value = Some(consume_constant(current, ctx));
            }
            Rule::block_attribute_list => {},
            Rule::inline_attribute_list => {}
            Rule::trailing_comment => {
                comment = match (comment, consume_trailing_comment(current)) {
                    (c, None) | (None, c) => c,
                    (Some(existing), Some(new)) => Some(ast::Comment {
                        text: [existing.text, new.text].join("\n"),
                    }),
                };
            }
            _ => consume_catch_all(&current, "enum member"),
        }
    }

    Ok(ast::EnumMember {
        name: name.unwrap(),
        documentation: None,
        attributes: ast::AttributeList(attributes),
        value: member_value.unwrap(),
        span: ast::Span::from_pest(pair_span, ctx.source_id),
    })
}

pub(crate) fn consume_enum_layout(
    token: Pair<'_>,
    name_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Declaration, DiagnosticsError> {
    debug_assert!(token.as_rule() == Rule::inline_enum_layout);

    let span = token.as_span();
    let enum_span = ast::Span::from_pest(span, ctx.source_id);

    let attributes = ast::AttributeList(vec![]);
    let mut members = Vec::new();
    let mut pending_field_comment = None;
    let mut subtype_ctor = None;

    for current in token.into_inner() {
        match current.as_rule() {
            Rule::ENUM_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::type_constructor => {
                subtype_ctor = Some(consume_type_constructor(current, &name_context, ctx));
            }
            Rule::inline_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics)) */ }
            Rule::value_layout_member => {
                let name_context = name_context.clone();

                match consume_enum_member(current, pending_field_comment.take(), name_context, ctx) {
                    Ok(member) => {
                        members.push(Rc::new(RefCell::new(member)));
                    }
                    Err(err) => ctx.diagnostics.push_error(err),
                }
            }
            Rule::layout_subtype => {
                // TODO: todo!() 
            }
            Rule::declaration_modifiers => {}
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => ctx.diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                ast::Span::from_pest(current.as_span(), ctx.source_id),
            )),
            _ => consume_catch_all(&current, "enum"),
        }
    }

    Ok(ast::Enum {
        name: name_context.to_name(ctx.library.clone(), enum_span.clone()),
        span: enum_span,
        members,
        attributes,
        documentation: None,
        unknown_value_signed: 0,
        unknown_value_unsigned: 0,
        subtype_ctor: subtype_ctor.unwrap_or(identifier_type_for_decl(ctx.default_underlying_type.clone())),
        r#type: None,
        compiled: false,
        compiling: false,
        recursive: false,
    }
    .into())
}