use pest::pratt_parser::PrattParser;

use crate::ast::Name;
use crate::compiler::ParsingContext;
use crate::diagnotics::DiagnosticsError;

use super::consume_attribute::consume_attribute_list;
use super::consume_type_constructor;
use crate::consumption::consume_compound_identifier;
use crate::consumption::consume_value::consume_literal;
use crate::consumption::helpers::consume_catch_all;

use super::ast;
use super::helpers::Pair;
use super::Rule;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use super::Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(or_op, Left))
    };
}

pub(crate) fn consume_constant(pair: Pair<'_>, ctx: &mut ParsingContext<'_>) -> ast::Constant {
    assert!(pair.as_rule() == Rule::constant);

    let constant = PRATT_PARSER
        .map_primary(|primary| {
            // let literal_token = pair.into_inner().next().unwrap();
            // let value = consume_literal(literal_token, ctx);
            let mut constant = None;

            let span = primary.as_span();
            let span = ast::Span::from_pest(span, ctx.source_id);

            match primary.as_rule() {
                Rule::literal => {
                    let literal = consume_literal(primary, ctx);
                    let concrete_constant = ast::LiteralConstant {
                        literal,
                        constant_value: None,
                        span,
                        compiled: false,
                    };

                    constant = Some(ast::Constant::Literal(concrete_constant));
                }
                Rule::compound_identifier => {
                    let name = consume_compound_identifier(&primary, ctx);
                    let concrete_constant = ast::IdentifierConstant {
                        reference: ast::Reference::new_sourced(name),
                        constant_value: None,
                        span,
                        compiled: false,
                    };

                    constant = Some(ast::Constant::Identifier(concrete_constant));
                }
                rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
            }

            constant.unwrap()
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::or_op => ast::ConstantOp::Or,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };

            ast::Constant::BinaryOperator(ast::BinaryOperatorConstant {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
                span: ast::Span::empty(),
                constant_value: None,
                compiled: false,
            })
        })
        .parse(pair.into_inner());

    constant
}

pub(crate) fn consume_constant_declaration(
    pair: Pair<'_>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Const, DiagnosticsError> {
    let pair_span = pair.as_span();
    let parts = pair.into_inner();

    let mut name: Option<ast::Name> = None;
    let mut value: Option<ast::Constant> = None;
    let mut type_ctor: Option<ast::TypeConstructor> = None;
    let mut attributes: Option<ast::AttributeList> = None;

    for current in parts {
        match current.as_rule() {
            Rule::STRUCT_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);
                name = Some(Name::create_sourced(ctx.library.clone(), name_span));
            }
            Rule::block_attribute_list => {
                attributes = Some(consume_attribute_list(current, ctx));
            }
            Rule::constant => {
                value = Some(consume_constant(current, ctx));
            }
            Rule::type_constructor => {
                let naming_context = ast::NamingContext::create(&name.clone().unwrap());
                type_ctor = Some(consume_type_constructor(current, &naming_context, ctx));
            }
            _ => consume_catch_all(&current, "const"),
        }
    }

    Ok(ast::Const {
        name: name.unwrap(),
        // identifier: identifier.unwrap(),
        type_ctor: type_ctor.unwrap(),
        value: value.unwrap(),
        attributes: attributes.unwrap(),
        documentation: None,
        span: ast::Span::from_pest(pair_span, ctx.source_id),
        compiled: false,
        compiling: false,
        recursive: false,
    })
}
