use std::rc::Rc;

use super::consume_identifier;
use super::consume_type::consume_type_constructor;
use super::helpers::consume_catch_all;
use super::{helpers::Pair, Rule};

use crate::ast::{self, Name};
use crate::compiler::ParsingContext;
use crate::consumption::consume_comments::{consume_comment_block, consume_trailing_comment};
use crate::diagnotics::DiagnosticsError;

fn consume_struct_member(
    pair: Pair<'_>,
    block_comment: Option<Pair<'_>>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::StructMember, DiagnosticsError> {
    debug_assert!(pair.as_rule() == Rule::struct_layout_member);

    let pair_span = pair.as_span();
    let mut name: Option<ast::Identifier> = None;
    let attributes: Vec<ast::Attribute> = Vec::new();
    let mut comment: Option<ast::Comment> = block_comment.and_then(consume_comment_block);
    let mut member_type_ctor: Option<ast::TypeConstructor> = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::identifier => name = Some(consume_identifier(&current, ctx)),
            Rule::type_constructor => member_type_ctor = Some(consume_type_constructor(current, ctx)),
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

pub(crate) fn consume_struct_declaration(
    pair: Pair<'_>,
    initial_name: Option<ast::Identifier>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Struct, DiagnosticsError> {
    let pair_span = pair.as_span();

    let mut identifier: Option<ast::Identifier> = initial_name;
    let mut name: Option<ast::Name> = None;
    let attributes: Vec<ast::Attribute> = Vec::new();
    let mut members: Vec<Rc<ast::StructMember>> = Vec::new();
    let mut pending_field_comment: Option<Pair<'_>> = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::STRUCT_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);

                identifier = Some(consume_identifier(&current, ctx));
                name = Some(Name::create_sourced(ctx.library.clone(), name_span));
            }
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics)) */ }
            Rule::struct_layout_member => match consume_struct_member(current, pending_field_comment.take(), ctx) {
                Ok(member) => {
                    members.push(Rc::from(member));
                }
                Err(err) => ctx.diagnostics.push_error(err),
            },
            Rule::declaration_modifiers => {}
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => ctx.diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                ast::Span::from_pest(current.as_span(), ctx.source_id),
            )),
            _ => consume_catch_all(&current, "struct"),
        }
    }

    match identifier {
        Some(identifier) => Ok(ast::Struct {
            identifier,
            name: name.unwrap(),
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
