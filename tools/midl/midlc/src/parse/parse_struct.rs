use super::helpers::parsing_catch_all;
use super::parse_identifier;
use super::parse_type::parse_type_constructor;
use super::{helpers::Pair, Rule};

use crate::ast;
use crate::database::ParsingContext;
use crate::diagnotics::DiagnosticsError;
use crate::parse::parse_comments::{parse_comment_block, parse_trailing_comment};

fn parse_struct_member(
    pair: Pair<'_>,
    block_comment: Option<Pair<'_>>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::StructMember, DiagnosticsError> {
    debug_assert!(pair.as_rule() == Rule::struct_layout_member);

    let pair_span = pair.as_span();
    let mut name: Option<ast::Identifier> = None;
    let attributes: Vec<ast::Attribute> = Vec::new();
    let mut comment: Option<ast::Comment> = block_comment.and_then(parse_comment_block);
    let mut member_type_ctor: Option<ast::TypeConstructor> = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => name = Some(parse_identifier(&current, ctx)),
            Rule::type_constructor => member_type_ctor = Some(parse_type_constructor(current, ctx)),
            Rule::inline_attribute_list => {}
            Rule::trailing_comment => {
                comment = match (comment, parse_trailing_comment(current)) {
                    (c, None) | (None, c) => c,
                    (Some(existing), Some(new)) => Some(ast::Comment {
                        text: [existing.text, new.text].join("\n"),
                    }),
                };
            }
            _ => parsing_catch_all(&current, "struct member"),
        }
    }

    match (name, member_type_ctor) {
        (Some(name), Some(member_type_ctor)) => Ok(ast::StructMember {
            name,
            documentation: None,
            attributes,
            member_type_ctor,
            span: ast::Span::from_pest(pair_span, ctx.source_id),
        }),
        _ => panic!("Encountered impossible struct member declaration during parsing"),
    }
}

pub(crate) fn parse_struct_declaration(
    pair: Pair<'_>,
    initial_name: Option<ast::Identifier>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Struct, DiagnosticsError> {
    let pair_span = pair.as_span();

    let mut name: Option<ast::Identifier> = initial_name;
    let attributes: Vec<ast::Attribute> = Vec::new();
    let mut members: Vec<ast::StructMember> = Vec::new();
    let mut pending_field_comment: Option<Pair<'_>> = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::STRUCT_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => name = Some(parse_identifier(&current, ctx)),
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics)) */ }
            Rule::struct_layout_member => match parse_struct_member(current, pending_field_comment.take(), ctx) {
                Ok(member) => {
                    members.push(member);
                }
                Err(err) => ctx.diagnostics.push_error(err),
            },
            Rule::declaration_modifiers => {}
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => ctx.diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                ast::Span::from_pest(current.as_span(), ctx.source_id),
            )),
            _ => parsing_catch_all(&current, "struct"),
        }
    }

    match name {
        Some(name) => Ok(ast::Struct {
            name,
            members,
            attributes,
            documentation: None,
            span: ast::Span::from_pest(pair_span, ctx.source_id),
        }),
        _ => panic!("Encountered impossible struct declaration during parsing",),
    }
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
