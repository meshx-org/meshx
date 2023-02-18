use crate::database::ParsingContext;

use super::ast;
use super::helpers::Pair;
use super::parse_compound_identifier;

pub(crate) fn parse_library_declaration(pair: &Pair<'_>, ctx: &mut ParsingContext<'_>) -> ast::CompoundIdentifier {
    let name = pair.clone().into_inner().next().unwrap();

    parse_compound_identifier(&name, ctx)
}
