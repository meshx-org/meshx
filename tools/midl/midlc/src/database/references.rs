// This step resolves all references in the library. It does so in three steps:
//
// 1. Parse the structure of each reference. For example, given `foo.bar`, this
//    means choosing between "library foo, decl bar" and "decl foo, member bar".
//    This step does not consult availabilities nor the version selection.
// 2. Perform temporal decomposition, splitting declarations into finer-grained
//    pieces such that for each one, nothing changes over its availability.
// 3. Resolve all references in the decomposed AST, linking each one to the
//    specific Element* it refers to.
//
// Note that ResolveStep does not resolve constant values (i.e. calling
// Constant::ResolveTo). That happens in the CompileStep.

use super::context::Context;
use crate::ast;
use petgraph::Graph;

#[derive(Default, Debug)]
pub(crate) struct References {
    // The version graph for this library: directed, possibly cyclic, possibly
    // disconnected. Contains only elements from the current library's platform:
    // all the current library's elements, plus elements from external libraries
    // it references. The latter have in-degree zero, i.e. they only appear as map
    // keys and never in the sets of outgoing neighbors.
    graph: Graph<u32, u32>,
}

pub(super) fn run_resolve_step<'db>(ctx: &'db mut Context<'db, '_>) {
    let mut references = References::default();

    //for rf in ctx.types.relation_fields.iter() {
    //    let evidence = relation_evidence(rf, ctx);
    //    ingest_relation(evidence, &mut relations, ctx);
    //}

    for (decl_id, decl) in ctx.ast.iter_decls() {
        match (decl_id, decl) {
            (ast::DeclarationId::Struct(struct_id), ast::Declaration::Struct(ast_struct)) => {
                visit_struct(struct_id, ast_struct, ctx)
            }
            (ast::DeclarationId::Protocol(_), ast::Declaration::Protocol(proto)) => visit_protocol(proto, ctx),
            (ast::DeclarationId::Const(_), ast::Declaration::Const(constant)) => visit_const(constant, ctx),
            (_, ast::Declaration::Import(_)) => (),
            _ => unreachable!(),
        }
    }

    let _ = std::mem::replace(ctx.references, references);
}

fn visit_struct<'db>(struct_id: ast::StructId, ast_struct: &'db ast::Struct, ctx: &mut Context<'db, '_>) {
    for (member_id, ast_member) in ast_struct.iter_members() {}
}

fn visit_protocol<'db>(ast_protocol: &'db ast::Protocol, ctx: &mut Context<'db, '_>) {
    for (ast_method) in ast_protocol.iter_methods() {}
}

fn visit_const<'db>(ast_const: &'db ast::Const, ctx: &mut Context<'db, '_>) {}

/// Calls ref.SetKey, ref.MarkContextual, or ref.MarkFailed.
fn parse_reference(r#ref: &ast::Reference, ctx: &mut Context<'_, '_>) {}

/// Helpers for ParseReference.
fn parse_synthetic_reference(r#ref: &ast::Reference, ctx: &mut Context<'_, '_>) {}
fn parse_sourced_reference(r#ref: &ast::Reference, ctx: &mut Context<'_, '_>) {}

/// Inserts edges into graph_ for a parsed reference.
fn insert_reference_edges(r#ref: &ast::Reference, ctx: &mut Context<'_, '_>) {}

/// Calls ref.ResolveTo or ref.MarkFailed.
fn resolve_reference(r#ref: &ast::Reference, ctx: &mut Context<'_, '_>) {}

/// Helpers for ResolveReference.
fn resolve_contextual_reference(r#ref: &ast::Reference, ctx: &mut Context<'_, '_>) {}
fn resolve_key_reference(r#ref: &ast::Reference, ctx: &mut Context<'_, '_>) {}

// Validates a resolved reference (e.g. checks deprecation rules).
fn validate_reference(r#ref: &ast::Reference, ctx: &mut Context<'_, '_>) {}
