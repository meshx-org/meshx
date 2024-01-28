use std::cell::RefCell;
use std::rc::Rc;

use super::consume_identifier;
use super::consume_type::consume_type_constructor;
use super::helpers::consume_catch_all;
use super::{helpers::Pair, Rule};

use crate::ast::{self, NamingContext, Strictness};
use crate::compiler::ParsingContext;
use crate::consumption::consume_comments::{consume_comment_block, consume_trailing_comment};
use crate::consumption::helpers::consume_ordinal64;
use crate::diagnotics::DiagnosticsError;

fn consume_table_member(
    pair: Pair<'_>,
    block_comment: Option<Pair<'_>>,
    name_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::UnionMember, DiagnosticsError> {
    debug_assert!(pair.as_rule() == Rule::ordinal_layout_member);

    let pair_span = pair.as_span();
    let mut name = None;
    let attributes = Vec::new();
    let mut comment = block_comment.and_then(consume_comment_block);
    let mut ordinal = None;
    let mut type_ctor = None;
    let mut reserved = false;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => name = Some(consume_identifier(&current, ctx)),
            Rule::ordinal => {
                ordinal = Some(consume_ordinal64(current, ctx)?);
            }
            Rule::type_constructor => type_ctor = Some(consume_type_constructor(current, &name_context, ctx)),
            Rule::inline_attribute_list => {}
            Rule::RESERVED_KEYWORD => {
                reserved = true;
            }
            Rule::trailing_comment => {
                comment = match (comment, consume_trailing_comment(current)) {
                    (c, None) | (None, c) => c,
                    (Some(existing), Some(new)) => Some(ast::Comment {
                        text: [existing.text, new.text].join("\n"),
                    }),
                };
            }
            _ => consume_catch_all(&current, "table member"),
        }
    }

    if !reserved {
        Ok(ast::UnionMember {
            documentation: None,
            attributes: ast::AttributeList(attributes),
            ordinal: ordinal.unwrap(),
            maybe_used: Some(ast::UnionMemberUsed {
                name: name.unwrap(),
                type_ctor: type_ctor.unwrap(),
            }),
            span: ast::Span::from_pest(pair_span, ctx.source_id),
        })
    } else {
        Ok(ast::UnionMember {
            documentation: None,
            ordinal: ordinal.unwrap(),
            attributes: ast::AttributeList(attributes),
            span: ast::Span::from_pest(pair_span, ctx.source_id),
            maybe_used: None,
        })
    }
}

pub(crate) fn consume_table_layout(
    token: Pair<'_>,
    identifier: ast::Identifier,
    name: ast::Name,
    name_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Declaration, DiagnosticsError> {
    debug_assert!(token.as_rule() == Rule::inline_table_layout);

    let token_span = token.as_span();

    let attributes = ast::AttributeList(vec![]);
    let mut members = Vec::new();
    let mut pending_field_comment = None;

    for current in token.into_inner() {
        match current.as_rule() {
            Rule::STRUCT_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::declaration_modifiers => todo!(),
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics)) */ }
            Rule::ordinal_layout_member => {
                match consume_table_member(current, pending_field_comment.take(), name_context.clone(), ctx) {
                    Ok(member) => {
                        members.push(Rc::new(RefCell::new(member)));
                    }
                    Err(err) => ctx.diagnostics.push_error(err),
                }
            }
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => ctx.diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                ast::Span::from_pest(current.as_span(), ctx.source_id),
            )),
            _ => consume_catch_all(&current, "table"),
        }
    }

    Ok(ast::Union {
        identifier,
        name,
        members,
        attributes,
        documentation: None,
        strictness: Strictness::Flexible,
        span: ast::Span::from_pest(token_span, ctx.source_id),
        compiled: false,
        compiling: false,
        recursive: false,
    }
    .into())
}
