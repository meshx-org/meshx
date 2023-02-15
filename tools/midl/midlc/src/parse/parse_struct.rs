use super::helpers::parsing_catch_all;
use super::{helpers::Pair, Rule};

use crate::diagnotics::{Diagnostics, DiagnosticsError};
use crate::{ast, error::ParserError};

pub(crate) fn parse_struct_declaration(
    pair: Pair<'_>,
    diagnostics: &mut Diagnostics,
) -> Result<ast::Struct, ParserError> {
    let pair_span = pair.as_span();

    let mut name: Option<ast::Identifier> = None;
    // let mut attributes: Vec<Attribute> = Vec::new();
    let mut members: Vec<ast::StructMember> = Vec::new();
    let mut pending_field_comment: Option<Pair<'_>> = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::STRUCT_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => name = Some(current.into()),
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics)) */},
            Rule::struct_layout_member => {},
            /*Rule::member_field => match parse_field(
                &name.as_ref().unwrap().value,
                "model",
                current,
                pending_field_comment.take(),
                diagnostics,
            ) {
                Ok(field) => members.push(field),
                Err(err) => diagnostics.push_error(err),
            },*/
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                current.as_span().into(),
            )),
            _ => parsing_catch_all(&current, "struct"),
        }
    }

    match name {
        Some(name) => Ok(ast::Struct {
            name,
            members,
            attributes: vec![],
            documentation: None,
            span: ast::Span::from(pair_span),
        }),
        _ => Err(ParserError::UnexpectedToken),
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
