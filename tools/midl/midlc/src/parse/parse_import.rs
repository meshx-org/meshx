use super::helpers::Pair;
use super::parse_compound_identifier;
use super::ast;
use super::ParserError;

pub(crate) fn parse_import_declaration(pair: &Pair<'_>) -> Result<ast::ImportDeclaration, ParserError> {
    let name = pair.clone().into_inner().next().unwrap();

    Ok(ast::ImportDeclaration {
        name: parse_compound_identifier(&name)?,
    })
}