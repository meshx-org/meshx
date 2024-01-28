use crate::ast::Name;
use crate::compiler::ParsingContext;
use crate::diagnotics::DiagnosticsError;

use super::consume_attribute::consume_attribute_list;
use super::consume_type_constructor;
use crate::consumption::helpers::consume_catch_all;

use super::ast;
use super::helpers::Pair;
use super::Rule;

pub(crate) fn consume_alias_declaration(
    pair: Pair<'_>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Alias, DiagnosticsError> {
    let pair_span = pair.as_span();
    let parts = pair.into_inner();

    let mut alias_name = None;
    let mut type_ctor = None;
    let mut attributes = None;

    for current in parts {
        match current.as_rule() {
            Rule::ALIAS_KEYWORD => {}
            Rule::block_attribute_list => {
                attributes = Some(consume_attribute_list(current, ctx));
            }
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);

                alias_name = Some(Name::create_sourced(ctx.library.clone(), name_span));
            }
            Rule::type_constructor => {
                let mut name_context = ast::NamingContext::create(&alias_name.clone().unwrap());
                type_ctor = Some(consume_type_constructor(current, &name_context, ctx));
            }
            _ => consume_catch_all(&current, "const"),
        }
    }

    Ok(ast::Alias {
        name: alias_name.unwrap(),
        partial_type_ctor: type_ctor.unwrap(),
        attributes: attributes.unwrap(),
        documentation: None,
        span: ast::Span::from_pest(pair_span, ctx.source_id),
    })
}
