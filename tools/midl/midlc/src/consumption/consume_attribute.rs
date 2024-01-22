use super::{helpers::Pair, Rule};

use crate::{
    ast::{self, *},
    compiler::ParsingContext,
    consumption::{consume_const::consume_constant, helpers::consume_catch_all},
};

pub(crate) fn consume_attribute_arg(token: Pair<'_>, ctx: &mut ParsingContext<'_>) -> ast::AttributeArg {
    debug_assert!(token.as_rule() == Rule::attribute_arg);

    let token_span = token.as_span();
    let mut name = None;
    let mut value = None;

    for current in token.into_inner() {
        match current.as_rule() {
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);

                name = Some(Name::create_sourced(ctx.library.clone(), name_span));
            }
            Rule::constant => {
                value = Some(consume_constant(current, ctx));
            }
            _ => consume_catch_all(&current, "attribute arg"),
        }
    }

    AttributeArg::new(
        name.unwrap(),
        ast::Span::from_pest(token_span, ctx.source_id),
        value.unwrap(),
    )
}

pub(crate) fn consume_attribute(token: Pair<'_>, ctx: &mut ParsingContext<'_>) -> ast::Attribute {
    debug_assert!(token.as_rule() == Rule::attribute);

    let token_span = token.as_span();
    let mut name = None;
    let mut arguments = vec![];

    for current in token.into_inner() {
        match current.as_rule() {
            Rule::compound_identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);

                name = Some(Name::create_sourced(ctx.library.clone(), name_span));
            }
            Rule::attribute_args => {
                for arg in current.into_inner() {
                    match arg.as_rule() {
                        Rule::attribute_arg => {
                            let arg = consume_attribute_arg(arg, ctx);
                            arguments.push(arg)
                        }
                        _ => consume_catch_all(&arg, "attribute args"),
                    }
                }
            }
            _ => consume_catch_all(&current, "attribute"),
        }
    }

    Attribute {
        name: name.unwrap(),
        arguments,
        span: ast::Span::from_pest(token_span, ctx.source_id),
        compiled: false,
    }
}

pub(crate) fn consume_attribute_list(token: Pair<'_>, ctx: &mut ParsingContext<'_>) -> ast::AttributeList {
    debug_assert!(token.as_rule() == Rule::inline_attribute_list || token.as_rule() == Rule::block_attribute_list);

    let mut attributes = vec![];

    for current in token.into_inner() {
        match current.as_rule() {
            Rule::attribute => {
                let attribute = consume_attribute(current, ctx);
                attributes.push(attribute)
            }
            _ => consume_catch_all(&current, "attribute list"),
        }
    }

    ast::AttributeList(attributes)
}
