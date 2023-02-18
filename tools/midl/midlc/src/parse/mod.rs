mod helpers;
mod parse_comments;
mod parse_const;
mod parse_import;
mod parse_library;
mod parse_protocol;
mod parse_struct;
mod parse_type;
mod parse_value;
mod parser;

use parse_const::parse_constant_declaration;
use parse_import::parse_import;
use parse_library::parse_library_declaration;
use parse_protocol::parse_protocol_declaration;
use parse_struct::parse_struct_declaration;
use parse_type::parse_type_constructor;
pub use parser::{MIDLParser, Rule};

use super::ast;
use crate::{database::ParsingContext, diagnotics::DiagnosticsError};

use pest::iterators::{Pair, Pairs};

pub(crate) fn parse_identifier(pair: &Pair<'_, Rule>, ctx: &mut ParsingContext<'_>) -> ast::Identifier {
    debug_assert!(pair.as_rule() == Rule::identifier);
    let pair_span = pair.as_span();

    ast::Identifier {
        value: pair.as_str().to_string(),
        span: ast::Span::from_pest(pair_span, ctx.source_id),
    }
}

pub(crate) fn parse_compound_identifier(
    pair: &Pair<'_, Rule>,
    ctx: &mut ParsingContext<'_>,
) -> ast::CompoundIdentifier {
    debug_assert!(pair.as_rule() == Rule::compound_identifier);
    let pair_span = pair.as_span();

    let mut components = vec![];

    for identifier_pair in pair.clone().into_inner() {
        if let Rule::identifier = identifier_pair.as_rule() {
            components.push(parse_identifier(&identifier_pair, ctx))
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
                        let struct_declaration = parse_struct_declaration(declaration_pair, None, ctx);
                        match struct_declaration {
                            Ok(decl) => {
                                let mut lib_lock = ctx.library.lock().unwrap();
                                lib_lock.declarations.insert(ast::Declaration::Struct(decl))
                            }
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::const_declaration => {
                        let const_declaration = parse_constant_declaration(declaration_pair, ctx);
                        match const_declaration {
                            Ok(decl) => {
                                let mut lib_lock = ctx.library.lock().unwrap();
                                lib_lock.declarations.insert(ast::Declaration::Const(decl))
                            }
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::protocol_declaration => {
                        let result = parse_protocol_declaration(declaration_pair, ctx);
                        match result {
                            Ok((protocol, mut decls)) => {
                                let mut lib_lock = ctx.library.lock().unwrap();
                                lib_lock.declarations.insert(ast::Declaration::Protocol(protocol));
                                decls.drain(..).for_each(|decl| lib_lock.declarations.insert(decl));
                            }
                            Err(err) => ctx.diagnostics.push_error(err),
                        }
                    }
                    Rule::library_declaration => {
                        let span = ast::Span::from_pest(declaration_pair.as_span(), ctx.source_id);
                        // All midl files in a library should agree on the library name.
                        let new_name = parse_library_declaration(&declaration_pair, ctx);
                        let mut lib_lock = ctx.library.lock().unwrap();

                        if lib_lock.name.is_none() {
                            lib_lock.name = Some(new_name);
                            lib_lock.arbitrary_name_span = Some(span);
                        } else {
                            if !lib_lock.name.contains(&new_name) {
                                ctx.diagnostics
                                    .push_error(DiagnosticsError::new("ErrFilesDisagreeOnLibraryName", span));
                                continue;
                            }
                            // Prefer setting arbitrary_name_span to a file which has attributes on the
                            // library declaration, if any do, since it's conventional to put all
                            // library attributes and the doc comment in a single file (overview.fidl).
                            // if self.library.attributes.Empty() && source.library_decl.attributes {
                            //    self.library.arbitrary_name_span = source.library_decl.span();
                            // }
                        }

                        //ctx.library
                        //    .library_name_declarations
                        //    .emplace_back(file_.library_decl.path.span());
                    }
                    Rule::import_declaration => {
                        parse_import(&declaration_pair, ctx);
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
