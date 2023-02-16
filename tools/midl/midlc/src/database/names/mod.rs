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
pub(super) fn verify_names(ctx: &mut Context<'_>) {
    for (decl_id, decl) in ctx.ast.iter_decls() {
        // assert_is_not_a_reserved_scalar_type(decl.identifier(), ctx);

        let namespace = match (decl_id, decl) {
            (_, ast::Declaration::Struct(ast_struct)) => {
                validate_identifier(ast_struct.identifier(), "Struct", ctx);
            }
            (_, ast::Declaration::Protocol(ast_protocol)) => {
                validate_identifier(ast_protocol.identifier(), "Protocol", ctx);
            }
            (_, ast::Declaration::Const(ast_const)) => {
                validate_identifier(ast_const.identifier(), "Const", ctx);
            }
            (_, ast::Declaration::Import(_)) | (_, ast::Declaration::Library(_)) => (),
            _ => unreachable!(),
        };

        // insert_name(top_id, decl, namespace, ctx)
    }
}

fn validate_identifier(ident: &ast::Identifier, schema_item: &str, ctx: &mut Context<'_>) {
    if ident.value.is_empty() {
        ctx.push_error(DiagnosticsError::new_validation_error(
            &format!("The name of a {schema_item} must not be empty."),
            ident.span,
        ))
    } else if ident.value.chars().next().unwrap().is_numeric() {
        ctx.push_error(DiagnosticsError::new_validation_error(
            &format!("The name of a {schema_item} must not start with a number."),
            ident.span,
        ))
    } else if ident.value.contains('-') {
        ctx.push_error(DiagnosticsError::new_validation_error(
            &format!("The character `-` is not allowed in {schema_item} names."),
            ident.span,
        ))
    }
}
