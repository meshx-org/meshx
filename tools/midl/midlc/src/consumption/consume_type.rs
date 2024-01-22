use crate::ast::Constant;
use crate::compiler::ParsingContext;
use crate::consumption::consume_const::consume_constant;
use crate::consumption::helpers::consume_catch_all;

use super::ast;
use super::consume_compound_identifier;
use super::helpers::Pair;
use super::Rule;

fn consume_layout_parameters(pair: Pair<'_>, ctx: &mut ParsingContext<'_>) -> Vec<ast::LayoutParameter> {
    debug_assert!(pair.as_rule() == Rule::layout_parameters);

    let mut params: Vec<ast::LayoutParameter> = vec![];

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::constant => {
                let constant = consume_constant(current, ctx);

                if let Constant::Literal(literal) = constant {
                    let param = ast::LiteralLayoutParameter { literal };

                    params.push(ast::LayoutParameter::Literal(param))
                } else {
                    panic!("Not a literal")
                }
            }
            Rule::type_constructor => {
                params.push(ast::LayoutParameter::Type(ast::TypeLayoutParameter {
                    type_ctor: consume_type_constructor(current.clone(), ctx),
                }));
            }
            _ => consume_catch_all(&current, "layout parameter"),
        }
    }

    params
}

pub(crate) fn consume_type_constructor(pair: Pair<'_>, ctx: &mut ParsingContext<'_>) -> ast::TypeConstructor {
    debug_assert!(pair.as_rule() == Rule::type_constructor);

    let pair_span = pair.as_span();
    let mut name = None;

    let mut params: Vec<ast::LayoutParameter> = vec![];
    let mut params_span = None;
    // TODO: params_signature

    // TODO: constraits_signature

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::compound_identifier => {
                name = Some(consume_compound_identifier(&current, ctx));
            }
            Rule::inline_layout => {
                // TODO: inline layout parse
            }
            Rule::layout_parameters => {
                params_span = Some(ast::Span::from_pest(pair_span, ctx.source_id));
                params.append(&mut consume_layout_parameters(current, ctx));
            }
            Rule::type_constraints => {
                // TODO: constraints layout parse
            }
            _ => consume_catch_all(&current, "struct member"),
        }
    }

    ast::TypeConstructor {
        parameters: ast::LayoutParameterList {
            items: params,
            span: params_span,
        },
        constraints: ast::LayoutConstraints {},
        layout: ast::Reference::new_sourced(name.unwrap()),
        r#type: None,
    }
}
