use std::cell::RefCell;
use std::rc::Rc;

use super::consume_type::consume_type_constructor;
use super::helpers::consume_catch_all;
use super::{helpers::Pair, Rule};

use crate::ast;
use crate::compiler::ParsingContext;
use crate::consumption::consume_attribute::consume_attribute_list;
use crate::consumption::{consume_identifier, identifier_type_for_decl};
use crate::diagnotics::DiagnosticsError;

pub(super) fn consume_resource_property(
    token: Pair<'_>,
    name_context: Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> ast::ResourceProperty {
    debug_assert!(token.as_rule() == Rule::resource_property);

    let mut name = None;
    let mut type_ctor: Option<ast::TypeConstructor> = None;

    for current in token.into_inner() {
        match current.as_rule() {
            Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::resource_property => {
                // todo!()
            }
            Rule::identifier => {
                name = Some(consume_identifier(&current, ctx));
            }
            Rule::type_constructor => {
                let member_name = name.clone().unwrap();
                let name_context = name_context.clone().enter_member(member_name);

                type_ctor = Some(consume_type_constructor(current, &name_context, ctx));
            }
            _ => consume_catch_all(&current, "resource property"),
        }
    }

    ast::ResourceProperty {
        name: name.unwrap(),
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

    let mut subtype_ctor: Option<ast::TypeConstructor> = None;
    let mut attributes: Option<ast::AttributeList> = None;
    let mut properties = vec![];

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
            Rule::layout_subtype => {
                let name_context = name_context.as_ref().unwrap();
                subtype_ctor = Some(consume_type_constructor(
                    current.into_inner().next().unwrap(),
                    name_context,
                    ctx,
                ));
            }
            Rule::resource_property => {
                let name_context = name_context.as_ref().unwrap().clone();
                let prop = consume_resource_property(current, name_context, ctx);

                properties.push(Rc::new(RefCell::new(prop)))
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
        subtype_ctor: subtype_ctor.unwrap_or(identifier_type_for_decl(ctx.default_underlying_type.clone())),
        span: ast::Span::from_pest(token_span, ctx.source_id),
        compiled: false,
        compiling: false,
        recursive: false, // subtype_ctor: maybe_type_ctor.unwrap_or(TypeConstructor {}),
    })
}
