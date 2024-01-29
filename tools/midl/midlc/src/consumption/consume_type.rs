use std::rc::Rc;

use crate::ast::Constant;
use crate::ast::Reference;
use crate::ast::Target;
use crate::compiler::ParsingContext;
use crate::consumption::consume_const::consume_constant;
use crate::consumption::consume_enum::consume_enum_layout;
use crate::consumption::consume_struct::consume_struct_layout;
use crate::consumption::helpers::consume_catch_all;

use super::ast;
use super::consume_compound_identifier;
use super::helpers::Pair;
use super::Rule;

fn consume_layout_parameters(
    pair: Pair<'_>,
    name_context: &Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> Vec<ast::LayoutParameter> {
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
                    type_ctor: consume_type_constructor(current.clone(), name_context, ctx),
                }));
            }
            _ => consume_catch_all(&current, "layout parameter"),
        }
    }

    params
}

enum Layout {
    Inline(ast::Declaration),
    Named,
}

pub(crate) fn consume_type_constructor(
    pair: Pair<'_>,
    name_context: &Rc<ast::NamingContext>,
    ctx: &mut ParsingContext<'_>,
) -> ast::TypeConstructor {
    debug_assert!(pair.as_rule() == Rule::type_constructor);

    let pair_span = pair.as_span();
    let mut layout: Option<Reference> = None;

    let mut params: Vec<ast::LayoutParameter> = vec![];
    let mut params_span = None;

    // TODO: params_signature
    // TODO: constraits_signature

    for current in pair.into_inner() {
        match current.as_rule() {
            Rule::compound_identifier => {
                let name = consume_compound_identifier(&current, ctx);
                layout = Some(ast::Reference::new_sourced(name));
            }
            Rule::inline_struct_layout => {
                let decl = consume_struct_layout(current, name_context.clone(), ctx).unwrap();
                ctx.library.declarations.borrow_mut().insert(decl.clone());
                layout = Some(ast::Reference::new_synthetic(Target::new(decl)));
            }
            Rule::inline_enum_layout => {
                let decl = consume_enum_layout(current, name_context.clone(), ctx).unwrap();
                ctx.library.declarations.borrow_mut().insert(decl.clone());
                layout = Some(ast::Reference::new_synthetic(Target::new(decl)));
            }
            Rule::inline_table_layout => {
                todo!()
            }
            Rule::inline_union_layout => {
                todo!()
            }
            Rule::layout_parameters => {
                params_span = Some(ast::Span::from_pest(pair_span, ctx.source_id));
                params.append(&mut consume_layout_parameters(current, name_context, ctx));
            }
            Rule::type_constraints => {
                todo!()
            }
            _ => consume_catch_all(&current, "type constructor"),
        }
    }

    ast::TypeConstructor {
        parameters: ast::LayoutParameterList {
            items: params,
            span: params_span,
        },
        constraints: ast::LayoutConstraints {},
        layout: layout.unwrap(),
        r#type: None,
    }
}
