use crate::ast::Name;
use crate::compiler::ParsingContext;
use crate::consumption::consume_compound_identifier;
use crate::consumption::helpers::consume_catch_all;
use crate::diagnotics::DiagnosticsError;

use crate::consumption::consume_value::consume_literal;

use super::ast;
use super::consume_identifier;
use super::consume_type_constructor;
use super::helpers::Pair;
use super::Rule;

pub(crate) fn consume_constant_value(pair: Pair<'_>, ctx: &mut ParsingContext<'_>) -> ast::ConstantValue {
    assert!(pair.as_rule() == Rule::constant);

    // let literal_token = pair.into_inner().next().unwrap();
    // let value = consume_literal(literal_token, ctx);
    let mut constant_value = None;

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::literal => {
                let value = consume_literal(current, ctx);
                let constant = ast::LiteralConstant { value };
                constant_value = Some(ast::ConstantValue::Literal(constant));
            }
            Rule::compound_identifier => {
                let name = consume_compound_identifier(&current, ctx);
                let constant = ast::IdentifierConstant {
                    reference: ast::Reference::new_sourced(name),
                };
                constant_value = Some(ast::ConstantValue::Identifier(constant));
            }
            _ => consume_catch_all(&current, "constant"),
        }
    }

    constant_value.unwrap()
}

pub(crate) fn consume_constant_declaration(
    pair: Pair<'_>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Const, DiagnosticsError> {
    let pair_span = pair.as_span();
    let mut parts = pair.into_inner();

    let _attribute_list = parts.next().unwrap().as_str();
    let identifier = parts.next().unwrap();
    let ty = parts.next().unwrap();
    let constant = parts.next().unwrap();
    let attributes: Vec<ast::Attribute> = Vec::new();

    Ok(ast::Const {
        name: {
            let name_span = identifier.as_span();
            let name_span = ast::Span::from_pest(name_span, ctx.source_id);
            Name::create_sourced(ctx.library.clone(), name_span)
        },
        identifier: consume_identifier(&identifier, ctx),
        type_ctor: consume_type_constructor(ty, ctx),
        value: consume_constant_value(constant, ctx),
        attributes,
        documentation: None,
        span: ast::Span::from_pest(pair_span, ctx.source_id),
    })
}
