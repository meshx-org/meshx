use midlgen::Protocol;

use super::helpers::parsing_catch_all;
use super::{helpers::Pair, Rule};

use crate::diagnotics::{Diagnostics, DiagnosticsError};
use crate::{ast, error::ParserError};

pub(crate) fn parse_protocol_declaration(
    pair: Pair<'_>,
    diagnostics: &mut Diagnostics,
) -> Result<ast::Protocol, ParserError> {
    let pair_span = pair.as_span();

    let mut name: Option<ast::Identifier> = None;
    let mut pending_field_comment: Option<Pair<'_>> = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::PROTOCOL_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => name = Some(current.into()),
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics)) */ }
            Rule::protocol_member => {}
            Rule::comment_block => pending_field_comment = Some(current),
            Rule::BLOCK_LEVEL_CATCH_ALL => diagnostics.push_error(DiagnosticsError::new_validation_error(
                "This line is not a valid field or attribute definition.",
                current.as_span().into(),
            )),
            _ => parsing_catch_all(&current, "protocol"),
        }
    }

    match name {
        Some(name) => Ok(ast::Protocol {
            name,
            attributes: vec![],
            span: ast::Span::from(pair_span),
        }),
        _ => Err(ParserError::UnexpectedToken),
    }
}
