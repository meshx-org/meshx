use crate::ast::Constant;
use crate::ast::LayoutParameterList;
use crate::database::ParsingContext;
use crate::parse::helpers::parsing_catch_all;
use crate::parse::parse_const::parse_constant;

use super::ast;
use super::helpers::Pair;
use super::parse_compound_identifier;
use super::Rule;

fn parse_layout_parameters(pair: Pair<'_>, ctx: &mut ParsingContext<'_>) -> Vec<ast::LayoutParameter> {
    debug_assert!(pair.as_rule() == Rule::layout_parameters);

    let mut params: Vec<ast::LayoutParameter> = vec![];

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::constant => params.push(ast::LayoutParameter::Constant(parse_constant(current, ctx))),
            Rule::type_constructor => {
                params.push(ast::LayoutParameter::TypeConstructor(parse_type_constructor(
                    current.clone(),
                    ctx,
                )));
            }
            _ => parsing_catch_all(&current, "layout parameter"),
        }
    }

    params
}

pub(crate) fn parse_type_constructor(pair: Pair<'_>, ctx: &mut ParsingContext<'_>) -> ast::TypeConstructor {
    debug_assert!(pair.as_rule() == Rule::type_constructor);

    let pair_span = pair.as_span();
    let mut name = None;

    let mut params: Vec<ast::LayoutParameter> = vec![];
    let mut params_span = None;
    // TODO: params_signature

    let mut constraits: Vec<ast::Constant> = vec![];
    // TODO: constraits_signature

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::compound_identifier => {
                name = Some(parse_compound_identifier(&current, ctx));
            }
            Rule::inline_layout => {
                // TODO: inline layout parse
            }
            Rule::layout_parameters => {
                params_span = Some(ast::Span::from_pest(pair_span, ctx.source_id));
                params.append(&mut parse_layout_parameters(current, ctx));
            }
            Rule::type_constraints => {
                // TODO: constraints layout parse
            }
            _ => parsing_catch_all(&current, "struct member"),
        }
    }

    ast::TypeConstructor {
        parameters: ast::LayoutParameterList {
            parameters: params,
            span: params_span,
        },
        constraints: ast::LayoutConstraints {},
        layout: ast::Reference::new_sourced(name.unwrap()),
        r#type: None,
    }
}
