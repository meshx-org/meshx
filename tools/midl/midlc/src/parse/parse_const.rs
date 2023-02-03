use crate::diagnotics::Diagnostics;
use crate::error::ParserError;
use crate::parse::parse_value::parse_literal;

use super::ast;
use super::helpers::Pair;
use super::parse_identifier;
use super::parse_type_constructor;
use super::Rule;

pub(crate) fn parse_constant(token: Pair<'_>, diagnostics: &mut Diagnostics) -> Result<ast::Constant, ParserError> {
    assert!(token.as_rule() == Rule::constant);

    let literal_token = token.into_inner().next().unwrap();
    let value = parse_literal(literal_token, diagnostics);

    Ok(ast::Constant(value))
}

pub(crate) fn parse_constant_declaration(
    pair: &Pair<'_>,
    diagnostics: &mut Diagnostics,
) -> Result<ast::ConstDeclaration, ParserError> {
    let mut parts = pair.clone().into_inner();

    let attribute_list = parts.next().unwrap().as_str();
    let identifier = parts.next().unwrap();
    let ty = parts.next().unwrap().into_inner().next().unwrap();
    let constant = parts.next().unwrap();

    Ok(ast::ConstDeclaration {
        name: parse_identifier(&identifier)?,
        ty: parse_type_constructor(&ty)?,
        value: parse_constant(constant.clone(), diagnostics)?,
    })
}
