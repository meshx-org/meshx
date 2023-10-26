mod consume_comments;
mod consume_const;
mod consume_import;
mod consume_library;
mod consume_protocol;
mod consume_struct;
mod consume_type;
mod consume_value;
mod helpers;
mod parser;

use std::{cell::RefCell, rc::Rc};

use consume_const::consume_constant_declaration;
use consume_import::consume_import;
use consume_library::consume_library_declaration;
use consume_protocol::consume_protocol_declaration;
use consume_struct::consume_struct_declaration;
use consume_type::consume_type_constructor;
pub use parser::{MIDLParser, Rule};

use super::ast;
use crate::{compiler::ParsingContext};

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

pub(crate) fn parse_source(pairs: Pairs<'_, Rule>, ctx: &mut ParsingContext<'_>) {
   

    // initial parsing
    for pair in pairs {
        if let Rule::library = pair.as_rule() {
            for declaration_pair in pair.into_inner() {
                match declaration_pair.as_rule() {
                    Rule::struct_declaration => {
                        let struct_declaration = consume_struct_declaration(declaration_pair, None, ctx);
                        match struct_declaration {
                            Ok(decl) => ctx.library.declarations.borrow_mut().insert(decl.into()),
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::const_declaration => {
                        let const_declaration = consume_constant_declaration(declaration_pair, ctx);
                        match const_declaration {
                            Ok(decl) => ctx
                                .library
                                .declarations
                                .borrow_mut()
                                .insert(ast::Declaration::Const(Rc::new(RefCell::new(decl)))),
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
                    Rule::layout_declaration => {}
                    _ => {}
                }
            }
        } else {
            panic!("Unexpected rule: {:?}", pair.as_rule())
        }
    }
}
