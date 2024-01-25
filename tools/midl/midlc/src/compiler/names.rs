use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::WithIdentifier;
use crate::diagnotics::DiagnosticsError;

use super::ast;
use super::context::Context;

/// `verify_names()` is responsible for populating `ParserDatabase.names` and
/// validating that there are no name collisions in the following namespaces:
///
/// - Model, enum and type alias names
/// - Generators
/// - Datasources
/// - Model fields for each model
/// - Enum variants for each enum
pub(crate) fn verify_names<'ctx>(ctx: &'ctx mut Context<'_>) {
    let cloned_lib = { ctx.library.clone() };
    
    for decl in cloned_lib.declarations.borrow().structs.iter() {
        let test = decl.clone();
        match test {
            ast::Declaration::Struct{ decl } => {
                validate_identifier(decl.borrow().identifier(), "struct", ctx)
            }
            _ => unreachable!(),
        }
    }
}

fn validate_identifier<'db, 'ctx>(ident: &'db ast::Identifier, schema_item: &str, ctx: &'ctx mut Context<'_>) {
    if ident.value.is_empty() {
        ctx.push_error(DiagnosticsError::new_validation_error(
            &format!("The name of a {schema_item} must not be empty."),
            ident.span.clone(),
        ))
    } else if ident.value.chars().next().unwrap().is_numeric() {
        ctx.push_error(DiagnosticsError::new_validation_error(
            &format!("The name of a {schema_item} must not start with a number."),
            ident.span.clone(),
        ))
    } else if ident.value.contains('-') {
        ctx.push_error(DiagnosticsError::new_validation_error(
            &format!("The character `-` is not allowed in {schema_item} names."),
            ident.span.clone(),
        ))
    }
}
