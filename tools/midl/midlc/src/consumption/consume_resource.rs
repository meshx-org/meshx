use std::rc::Rc;

use super::consume_type::consume_type_constructor;
use super::helpers::consume_catch_all;
use super::{helpers::Pair, Rule};

use crate::ast;
use crate::compiler::ParsingContext;
use crate::consumption::consume_attribute::consume_attribute_list;
use crate::diagnotics::DiagnosticsError;

pub(super) fn consume_resource_properties(
    token: Pair<'_>,
    name_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> ast::ResourceProperty {
    debug_assert!(token.as_rule() == Rule::resource_properties);

    let mut name: Option<ast::Name> = None;
    let mut type_ctor: Option<ast::TypeConstructor> = None;

    for current in token.into_inner() {
        match current.as_rule() {
            Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);

                name = Some(ast::Name::create_sourced(ctx.library.clone(), name_span));
            }
            Rule::type_constructor => {
                let member_name = name.clone().unwrap();
                let name_context = name_context.clone().enter_member(member_name.span().unwrap());

                type_ctor = Some(consume_type_constructor(current, &name_context, ctx));
            }
            _ => consume_catch_all(&current, "resource"),
        }
    }

    ast::ResourceProperty {
        attributes: ast::AttributeList(vec![]),
        type_ctor: type_ctor.unwrap(),
    }
}

pub(super) fn consume_resource_declaration(
    token: Pair<'_>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Resource, DiagnosticsError> {
    debug_assert!(token.as_rule() == Rule::resource_declaration);

    let token_span = token.as_span();
    let parts = token.into_inner();

    let mut name = None;
    let mut name_context = None;

    let mut maybe_type_ctor: Option<ast::TypeConstructor> = None;
    let mut attributes: Option<ast::AttributeList> = None;
    let mut properties: Vec<ast::ResourceProperty> = vec![];

    for current in parts {
        match current.as_rule() {
            Rule::RESOURCE_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::block_attribute_list => {
                attributes = Some(consume_attribute_list(current, ctx));
            }
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);
                let sourced = ast::Name::create_sourced(ctx.library.clone(), name_span);

                name_context = Some(ast::NamingContext::create(&sourced));
                name = Some(sourced);
            }
            Rule::type_constructor => {
                let name_context = name_context.as_ref().unwrap();
                maybe_type_ctor = Some(consume_type_constructor(current, &name_context, ctx));
            }
            Rule::resource_properties => {
                let name_context = name_context.as_ref().unwrap().clone();
                consume_resource_properties(current, name_context, ctx);
            }
            _ => consume_catch_all(&current, "resource"),
        }
    }

    /*let default_type = self.ctx
                        .all_libraries
                        .borrow()
                        .root_library()
                        .declarations
                        .borrow()
                        .lookup_builtin(ast::BuiltinIdentity::uint32);
    */
    //identifier_type_for_decl(  );

    Ok(ast::Resource {
        name: name.unwrap(),
        attributes: attributes.unwrap_or_default(),
        documentation: None,
        properties,
        span: ast::Span::from_pest(token_span, ctx.source_id),
        compiled: false,
        compiling: false,
        recursive: false, // subtype_ctor: maybe_type_ctor.unwrap_or(TypeConstructor {}),
    })
}
