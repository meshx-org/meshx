mod consume_alias;
mod consume_attribute;
mod consume_comments;
mod consume_const;
mod consume_enum;
mod consume_import;
mod consume_library;
mod consume_protocol;
mod consume_struct;
mod consume_table;
mod consume_type;
mod consume_union;
mod consume_value;
mod helpers;
mod parser;

use consume_alias::consume_alias_declaration;
use consume_attribute::consume_attribute_list;
use consume_const::consume_constant_declaration;
use consume_enum::consume_enum_layout;
use consume_import::consume_import;
use consume_library::consume_library_declaration;
use consume_protocol::consume_protocol_declaration;
use consume_struct::consume_struct_layout;
use consume_table::consume_table_layout;
use consume_type::consume_type_constructor;
use consume_union::consume_union_layout;

use self::helpers::consume_catch_all;
use super::ast;
use crate::{ast::Name, compiler::ParsingContext, diagnotics::DiagnosticsError};
pub use parser::{MIDLParser, Rule};
use pest::iterators::{Pair, Pairs};

pub(crate) fn consume_identifier(pair: &Pair<'_, Rule>, ctx: &mut ParsingContext<'_>) -> ast::Identifier {
    debug_assert!(pair.as_rule() == Rule::identifier);

    let pair_span = pair.as_span();

    ast::Identifier {
        value: pair.as_str().to_string(),
        span: ast::Span::from_pest(pair_span, ctx.source_id),
    }
}

pub(crate) fn consume_compound_identifier(
    pair: &Pair<'_, Rule>,
    ctx: &mut ParsingContext<'_>,
) -> ast::CompoundIdentifier {
    debug_assert!(pair.as_rule() == Rule::compound_identifier);

    let pair_span = pair.as_span();

    let mut components = vec![];

    for identifier_pair in pair.clone().into_inner() {
        if let Rule::identifier = identifier_pair.as_rule() {
            components.push(consume_identifier(&identifier_pair, ctx))
        }
    }

    ast::CompoundIdentifier {
        components,
        span: ast::Span::from_pest(pair_span, ctx.source_id),
    }
}

// Create a type constructor pointing to an anonymous layout.
fn identifier_type_for_decl(decl: ast::Declaration) -> ast::TypeConstructor {
    ast::TypeConstructor::new(
        ast::Reference::new_synthetic(ast::Target::new(decl)),
        ast::LayoutParameterList::default(),
        ast::LayoutConstraints::default(),
    )
}

fn consume_resource_properties(token: Pair<'_, Rule>, ctx: &mut ParsingContext<'_>) -> ast::ResourceProperty {
    debug_assert!(token.as_rule() == Rule::resource_properties);

    let mut name: Option<ast::Name> = None;
    let mut type_ctor: Option<ast::TypeConstructor> = None;

    for current in token.into_inner() {
        match current.as_rule() {
            Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);

                name = Some(Name::create_sourced(ctx.library.clone(), name_span));
            }
            Rule::type_constructor => {
                type_ctor = Some(consume_type_constructor(current, ctx));
            }
            _ => consume_catch_all(&current, "resource"),
        }
    }

    ast::ResourceProperty {
        attributes: ast::AttributeList(vec![]),
        type_ctor: type_ctor.unwrap(),
    }
}

fn consume_resource_declaration(
    token: Pair<'_, Rule>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Resource, DiagnosticsError> {
    debug_assert!(token.as_rule() == Rule::resource_declaration);

    let token_span = token.as_span();
    let parts = token.into_inner();

    let mut name: Option<ast::Name> = None;
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

                name = Some(Name::create_sourced(ctx.library.clone(), name_span));
            }
            Rule::type_constructor => {
                maybe_type_ctor = Some(consume_type_constructor(current, ctx));
            }
            Rule::resource_properties => {
                consume_resource_properties(current, ctx);
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

pub(crate) fn consume_layout_declaration(
    token: Pair<'_, Rule>,
    ctx: &mut ParsingContext<'_>,
) -> Result<ast::Declaration, DiagnosticsError> {
    debug_assert!(token.as_rule() == Rule::layout_declaration);

    let mut identifier = None;
    let mut name = None;
    let mut declaration: ast::Declaration;
    let span = token.as_span();

    for current in token.into_inner() {
        match current.as_rule() {
            Rule::TYPE_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
            Rule::identifier => {
                let name_span = current.as_span();
                let name_span = ast::Span::from_pest(name_span, ctx.source_id);

                identifier = Some(consume_identifier(&current, ctx));
                name = Some(Name::create_sourced(ctx.library.clone(), name_span));
            }
            Rule::block_attribute_list => { /*attributes.push(parse_attribute(current, diagnostics)) */ }
            Rule::inline_struct_layout => {
                return consume_struct_layout(current, identifier.unwrap().clone(), name.unwrap(), ctx);
            }
            Rule::inline_enum_layout => {
                return consume_enum_layout(current, identifier.unwrap().clone(), name.unwrap(), ctx);
            }
            Rule::inline_union_layout => {
                return consume_union_layout(current, identifier.unwrap().clone(), name.unwrap(), ctx);
            }
            Rule::inline_table_layout => {
                return consume_table_layout(current, identifier.unwrap().clone(), name.unwrap(), ctx);
            }
            Rule::CATCH_ALL => consume_catch_all(&current, "layout_declaration"),
            _ => todo!(),
        }
    }

    Err(DiagnosticsError::new("", ast::Span::from_pest(span, ctx.source_id)))
}

pub(crate) fn consume_source(pairs: Pairs<'_, Rule>, ctx: &mut ParsingContext<'_>) {
    // initial parsing
    for pair in pairs {
        if let Rule::library = pair.as_rule() {
            for declaration_pair in pair.into_inner() {
                match declaration_pair.as_rule() {
                    //Rule::struct_declaration => {
                    //    let struct_declaration = consume_struct_declaration(declaration_pair, None, ctx);
                    //    match struct_declaration {
                    //        Ok(decl) => ctx.library.declarations.borrow_mut().insert(decl.into()),
                    //        Err(err) => ctx.diagnostics.push_error(err),
                    //    }
                    //}
                    Rule::layout_declaration => {
                        let any_declaration = consume_layout_declaration(declaration_pair, ctx);

                        match any_declaration {
                            Ok(decl) => ctx.library.declarations.borrow_mut().insert(decl.into()),
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::const_declaration => {
                        let const_declaration = consume_constant_declaration(declaration_pair, ctx);
                        match const_declaration {
                            Ok(decl) => ctx.library.declarations.borrow_mut().insert(decl.into()),
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::alias_declaration => {
                        let alias_declaration = consume_alias_declaration(declaration_pair, ctx);
                        match alias_declaration {
                            Ok(decl) => ctx.library.declarations.borrow_mut().insert(decl.into()),
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::resource_declaration => {
                        let alias_declaration = consume_resource_declaration(declaration_pair, ctx);
                        match alias_declaration {
                            Ok(decl) => ctx.library.declarations.borrow_mut().insert(decl.into()),
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::protocol_declaration => {
                        let result = consume_protocol_declaration(declaration_pair, ctx);
                        match result {
                            Ok((protocol, mut decls)) => {
                                let mut declarations = ctx.library.declarations.borrow_mut();
                                declarations.insert(protocol.into());
                                decls.drain(..).for_each(|decl| declarations.insert(decl));
                            }
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::library_declaration => {
                        // All midl files in a library should agree on the library name.
                        consume_library_declaration(&declaration_pair, ctx);

                        //ctx.library
                        //    .library_name_declarations
                        //    .emplace_back(file_.library_decl.path.span());
                    }
                    Rule::import_declaration => {
                        consume_import(&declaration_pair, ctx);
                    }
                    Rule::EOI => {}
                    _ => consume_catch_all(&declaration_pair, "declaration"),
                }
            }
        } else {
            panic!("Unexpected rule: {:?}", pair.as_rule())
        }
    }
}
