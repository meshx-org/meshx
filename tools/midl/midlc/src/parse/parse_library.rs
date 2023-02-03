use super::helpers::Pair;
use super::parse_compound_identifier;
use super::ast;
use super::ParserError;

// pub(crate) fn parse_library_declaration(pair: Pair<'_>, diagnostics: &mut Diagnostics) -> Top {}

pub(crate) fn parse_library_declaration(pair: &Pair<'_>) -> Result<ast::LibraryDeclaration, ParserError> {
    let mut name = pair.clone().into_inner().next().unwrap();

    Ok(ast::LibraryDeclaration {
        name: parse_compound_identifier(&name)?,
    })
}