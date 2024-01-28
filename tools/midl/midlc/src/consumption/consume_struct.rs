use std::cell::RefCell;
use std::rc::Rc;

use super::consume_identifier;
use super::consume_type::consume_type_constructor;
use super::helpers::consume_catch_all;
use super::{helpers::Pair, Rule};

use crate::ast;
use crate::compiler::ParsingContext;
use crate::consumption::consume_comments::{consume_comment_block, consume_trailing_comment};
use crate::diagnotics::DiagnosticsError;

fn consume_struct_member(
    pair: Pair<'_>,
    block_comment: Option<Pair<'_>>,
    name_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::StructMember, DiagnosticsError> {
    debug_assert!(pair.as_rule() == Rule::struct_layout_member);

    let pair_span = pair.as_span();
    let mut name = None;
    let attributes = Vec::new();
    let mut comment = block_comment.and_then(consume_comment_block);
    let mut type_ctor = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => name = Some(consume_identifier(&current, ctx)),
            Rule::type_constructor => type_ctor = Some(consume_type_constructor(current, &name_context, ctx)),
            Rule::inline_attribute_list => {}
            Rule::trailing_comment => {
                comment = match (comment, consume_trailing_comment(current)) {
                    (c, None) | (None, c) => c,
                    (Some(existing), Some(new)) => Some(ast::Comment {
                        text: [existing.text, new.text].join("\n"),
                    }),
                };
            }
            _ => consume_catch_all(&current, "struct member"),
        }
    }

    match (name, type_ctor) {
        (Some(name), Some(type_ctor)) => Ok(ast::StructMember {
            name,
            documentation: None,
            attributes: ast::AttributeList(attributes),
            type_ctor,
            span: ast::Span::from_pest(pair_span, ctx.source_id),
            maybe_default_value: None,
        }),
        _ => panic!("Encountered impossible struct member declaration during parsing"),
    }
}

pub(crate) fn consume_struct_layout(
    token: Pair<'_>,
    identifier: ast::Identifier,
    name: ast::Name,
    name_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Declaration, DiagnosticsError> {
    debug_assert!(token.as_rule() == Rule::inline_struct_layout);

    let token_span = token.as_span();

    let attributes = ast::AttributeList(vec![]);
    let mut members = Vec::new();
    let mut pending_field_comment = None;

    for current in token.into_inner() {
        match current.as_rule() {
            Rule::STRUCT_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics)) */ }
            Rule::struct_layout_member => {
                match consume_struct_member(current, pending_field_comment.take(), name_context.clone(), ctx) {
                    Ok(member) => {
                        members.push(Rc::new(RefCell::new(member)));
                    }
                    Err(err) => ctx.diagnostics.push_error(err),
                }
            }
            Rule::declaration_modifiers => {}
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => ctx.diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                ast::Span::from_pest(current.as_span(), ctx.source_id),
            )),
            _ => consume_catch_all(&current, "struct"),
        }
    }

    Ok(ast::Struct {
        identifier,
        name,
        members,
        attributes,
        documentation: None,
        span: ast::Span::from_pest(token_span, ctx.source_id),
        compiled: false,
        compiling: false,
        recursive: false,
    }
    .into())
}

/*pub(crate) fn parse_struct(token: Pair<'_>, doc_comment: Option<Pair<'_>>, diagnostics: &mut Diagnostics) -> Model {
    assert!(token.as_rule() == Rule::constant);

    let pair_span = pair.as_span();
    let mut name: Option<Identifier> = None;
    let mut attributes: Vec<Attribute> = Vec::new();
    let mut fields: Vec<Field> = Vec::new();
    let mut pending_field_comment: Option<Pair<'_>> = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::STRUCT_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => name = Some(current.into()),
            // Rule::block_attribute => attributes.push(parse_attribute(current, diagnostics)),
            Rule::field_declaration => match parse_field(
                &name.as_ref().unwrap().value,
                "model",
                current,
                pending_field_comment.take(),
                diagnostics,
            ) {
                Ok(field) => fields.push(field),
                Err(err) => diagnostics.push_error(err),
            },
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                current.as_span().into(),
            )),
            _ => parsing_catch_all(&current, "struct"),
        }
    }

    match name {
        Some(name) => Model {
            name,
            fields,
            attributes,
            documentation: doc_comment.and_then(parse_comment_block),
            is_view: false,
            span: Span::from(pair_span),
        },
        _ => panic!("Encountered impossible model declaration during parsing",),
    }
}*/
