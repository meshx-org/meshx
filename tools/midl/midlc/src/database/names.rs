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
pub(crate) fn verify_names(ctx: Rc<RefCell<Context<'_>>>) {
    let cloned_lib = { ctx.borrow().library.clone() };
    
    for decl in cloned_lib.declarations.borrow().structs.iter() {
        let test = decl.clone();
        match test {
            ast::Declaration::Struct(r#struct) => {
                validate_identifier(r#struct.borrow().identifier(), "struct", ctx.clone())
            }
            _ => unreachable!(),
        }
    }
}

fn validate_identifier<'db>(ident: &'db ast::Identifier, schema_item: &str, ctx: Rc<RefCell<Context<'_>>>) {
    if ident.value.is_empty() {
        ctx.borrow_mut().push_error(DiagnosticsError::new_validation_error(
            &format!("The name of a {schema_item} must not be empty."),
            ident.span,
        ))
    } else if ident.value.chars().next().unwrap().is_numeric() {
        ctx.borrow_mut().push_error(DiagnosticsError::new_validation_error(
            &format!("The name of a {schema_item} must not start with a number."),
            ident.span,
        ))
    } else if ident.value.contains('-') {
        ctx.borrow_mut().push_error(DiagnosticsError::new_validation_error(
            &format!("The character `-` is not allowed in {schema_item} names."),
            ident.span,
        ))
    }
}
