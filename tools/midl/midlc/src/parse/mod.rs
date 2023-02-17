mod helpers;
mod parse_comments;
mod parse_const;
mod parse_import;
mod parse_library;
mod parse_protocol;
mod parse_struct;
mod parse_type;
mod parse_value;

use parse_const::parse_constant_declaration;
use parse_import::parse_import;
use parse_library::parse_library_declaration;
use parse_protocol::parse_protocol_declaration;
use parse_struct::parse_struct_declaration;
use parse_type::parse_type_constructor;

use crate::database::Context;

use super::ast;
use super::diagnotics::Diagnostics;

use pest::Parser;

use pest::iterators::Pair;

#[derive(Parser)]
#[grammar = "midl.pest"]
pub struct MIDLParser;

pub(crate) fn parse_identifier(pair: &Pair<'_, Rule>) -> ast::Identifier {
    debug_assert!(pair.as_rule() == Rule::identifier);

    ast::Identifier {
        value: pair.as_str().to_string(),
        span: ast::Span::from(pair.as_span()),
    }
}

pub(crate) fn parse_compound_identifier(pair: &Pair<'_, Rule>) -> ast::CompoundIdentifier {
    debug_assert!(pair.as_rule() == Rule::compound_identifier);

    let mut components = vec![];

    for identifier_pair in pair.clone().into_inner() {
        if let Rule::identifier = identifier_pair.as_rule() {
            components.push(parse_identifier(&identifier_pair))
        }
    }

    ast::CompoundIdentifier { components }
}

pub(crate) fn parse_source(midl_source: &str, ctx: &mut Context<'_, '_>) -> ast::Library {
    let pairs = MIDLParser::parse(Rule::library, &midl_source).unwrap();

    let mut name: Option<ast::CompoundIdentifier> = None;
    let mut declarations = ast::Declarations::default();
    let dependencies = ast::Dependencies::default();

    // initial parsing
    for pair in pairs {
        if let Rule::library = pair.as_rule() {
            for declaration_pair in pair.into_inner() {
                match declaration_pair.as_rule() {
                    Rule::struct_declaration => {
                        let struct_declaration = parse_struct_declaration(declaration_pair, None, ctx.diagnostics);
                        match struct_declaration {
                            Ok(decl) => declarations.insert(ast::Declaration::Struct(decl)),
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::const_declaration => {
                        let const_declaration = parse_constant_declaration(declaration_pair, ctx);
                        match const_declaration {
                            Ok(decl) => declarations.insert(ast::Declaration::Const(decl)),
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::protocol_declaration => {
                        let result = parse_protocol_declaration(declaration_pair, ctx.diagnostics);
                        match result {
                            Ok((protocol, mut decls)) => {
                                declarations.insert(ast::Declaration::Protocol(protocol));
                                decls.drain(..).for_each(|decl| declarations.insert(decl));
                            }
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::library_declaration => {
                        let library_name = parse_library_declaration(&declaration_pair);
                        name = Some(library_name)
                    }
                    Rule::import_declaration => {
                        parse_import(&declaration_pair, ctx.diagnostics);
                    }
                    Rule::layout_declaration => {}
                    _ => {}
                }
            }
        } else {
            panic!("Unexpected rule: {:?}", pair.as_rule())
        }
    }

    match name {
        Some(name) => ast::Library {
            name,
            declarations,
            dependencies,
            declaration_order: vec![],
        },
        None => panic!("Library name is not defined"),
    }
}
