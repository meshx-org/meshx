use crate::database::Context;
use crate::diagnotics::Diagnostics;
use crate::diagnotics::DiagnosticsError;
 
use crate::parse::parse_value::parse_literal;

use super::ast;
use super::helpers::Pair;
use super::parse_identifier;
use super::parse_type_constructor;
use super::Rule;

pub(crate) fn parse_constant(token: Pair<'_>, ctx: &mut Context<'_, '_>) -> ast::Constant {
    assert!(token.as_rule() == Rule::constant);

    let literal_token = token.into_inner().next().unwrap();
    let value = parse_literal(literal_token, ctx);

    ast::Constant(value)
}

pub(crate) fn parse_constant_declaration(
    pair: Pair<'_>,
    ctx: &mut Context<'_, '_>,
) -> Result<ast::Const, DiagnosticsError> {
    let pair_span = pair.as_span();
    let mut parts = pair.into_inner();

    let _attribute_list = parts.next().unwrap().as_str();
    let identifier = parts.next().unwrap();
    let ty = parts.next().unwrap().into_inner().next().unwrap();
    let constant = parts.next().unwrap();
    let mut attributes: Vec<ast::Attribute> = Vec::new();

    Ok(ast::Const {
        name: parse_identifier(&identifier),
        ty: parse_type_constructor(ty, ctx.diagnostics)?,
        value: parse_constant(constant.clone(), ctx),
        attributes,
        documentation: None,
        span: ast::Span::from(pair_span),
    })
}
