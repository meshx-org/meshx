mod helpers;
mod parse_comments;
mod parse_const;
mod parse_library;
mod parse_import;
mod parse_protocol;
mod parse_struct;
mod parse_type;
mod parse_value;

use parse_const::parse_constant_declaration;
use parse_library::parse_library_declaration;
use parse_import::parse_import_declaration;
use parse_protocol::parse_protocol_declaration;
use parse_struct::parse_struct_declaration;
use parse_type::parse_type_constructor;

use super::ast;
use super::diagnotics::Diagnostics;
use super::error::ParserError;

use pest::iterators::{Pair, Pairs};

#[derive(Parser)]
#[grammar = "midl.pest"]
pub struct MIDLParser;

#[derive(Debug)]
struct FQN {
    library: ast::CompoundIdentifier,
    declation_name: ast::Identifier,
}

enum Nullability {
    Nullable,
    Nonnullable,
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum HandleSubtype {
    // special case to indicate subtype is not specified.
    Handle = 0,
}

pub(crate) fn parse_identifier(identifier_pair: &Pair<'_, Rule>) -> Result<ast::Identifier, ParserError> {
    Ok(ast::Identifier {
        value: identifier_pair.as_str().to_string(),
        span: ast::Span::from(identifier_pair.as_span()),
    })
}

pub(crate) fn parse_compound_identifier(
    compound_pair: &Pair<'_, Rule>,
) -> Result<ast::CompoundIdentifier, ParserError> {
    let mut identifiers = vec![];

    for identifier_pair in compound_pair.clone().into_inner() {
        if let Rule::identifier = identifier_pair.as_rule() {
            identifiers.push(parse_identifier(&identifier_pair)?)
        }
    }

    Ok(ast::CompoundIdentifier(identifiers))
}

pub(crate) fn parse(pairs: Pairs<'_, Rule>, diagnostics: &mut Diagnostics) -> Result<ast::SchamaAST, ParserError> {
    let mut declarations = vec![];

    // initial parsing
    for pair in pairs {
        if let Rule::library = pair.as_rule() {
            for declaration_pair in pair.into_inner() {
                match declaration_pair.as_rule() {
                    Rule::struct_declaration => {
                        let struct_declaration = parse_struct_declaration(declaration_pair, diagnostics)?;
                        declarations.push(ast::Declaration::Struct(struct_declaration));
                    }
                    Rule::const_declaration => {
                        let const_declaration = parse_constant_declaration(declaration_pair, diagnostics)?;
                        declarations.push(ast::Declaration::Const(const_declaration));
                    }
                    Rule::library_declaration => {
                        let library_declaration = parse_library_declaration(&declaration_pair)?;
                        declarations.push(ast::Declaration::Library(library_declaration));
                    }
                    Rule::protocol_declaration => {
                        let (protocol_declaration, _) = parse_protocol_declaration(declaration_pair, diagnostics)?;
                        declarations.push(ast::Declaration::Protocol(protocol_declaration));
                    }
                    Rule::import_declaration => {
                        let declaration = parse_import_declaration(&declaration_pair)?;
                        declarations.push(ast::Declaration::Import(declaration));
                    }
                    Rule::layout_declaration => {}
                    _ => {}
                }
            }
        } else {
            return Err(ParserError::UnexpectedToken);
        }
    }

    // validation

    Ok(ast::SchamaAST { declarations })
}
