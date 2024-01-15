use crate::ast::Name;
use crate::compiler::ParsingContext;
use crate::diagnotics::DiagnosticsError;

use super::consume_attribute::consume_attribute_list;
use super::consume_identifier;
use super::consume_type_constructor;
use crate::consumption::consume_compound_identifier;
use crate::consumption::consume_value::consume_literal;
use crate::consumption::helpers::consume_catch_all;

use super::ast;
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
    let parts = pair.into_inner();

    let mut identifier: Option<ast::Identifier> = None;
    let mut name: Option<ast::Name> = None;
    let mut value: Option<ast::ConstantValue> = None;
    let mut type_ctor: Option<ast::TypeConstructor> = None;
    let mut attributes: Option<ast::AttributeList> = None;

    for current in parts {
        match current.as_rule() {
            Rule::STRUCT_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);

                identifier = Some(consume_identifier(&current, ctx));
                name = Some(Name::create_sourced(ctx.library.clone(), name_span));
            }
            Rule::block_attribute_list => {
                attributes = Some(consume_attribute_list(current, ctx));
            }
            Rule::constant => {
                value = Some(consume_constant_value(current, ctx));
            }
            Rule::type_constructor => {
                type_ctor = Some(consume_type_constructor(current, ctx));
            }
            _ => consume_catch_all(&current, "const"),
        }
    }

    Ok(ast::Const {
        name: name.unwrap(),
        identifier: identifier.unwrap(),
        type_ctor: type_ctor.unwrap(),
        value: value.unwrap(),
        attributes: attributes.unwrap(),
        documentation: None,
        span: ast::Span::from_pest(pair_span, ctx.source_id),
    })
}
