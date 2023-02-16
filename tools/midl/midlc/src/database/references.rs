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

use crate::diagnotics::DiagnosticsError;

use super::context::Context;
use crate::ast;
use petgraph::Graph;

// A key identifies a family of elements with a particular name. Unlike
// RawSourced, the roles of each component have been decided, and the library
// has been resolved. Unlike RawSynthetic, the key is stable across
// decomposition, i.e. we can choose it before decomposing the AST, and then
// use it for lookups after decomposing.
struct Key {}

pub(crate) struct Target;

pub(crate) struct Reference {
    state: ReferenceState,
}

pub(crate) enum ReferenceState {
    RawSourced,
    RawSynthetic { target: Target },
    Key,
    Contextual,
    Target,
    Failed,
}

impl Reference {
    fn new_sourced() -> Self {
        Reference {
            state: ReferenceState::RawSourced,
        }
    }

    fn new_synthetic(target: Target) -> Self {
        Reference {
            state: ReferenceState::RawSynthetic { target },
        }
    }

    fn set_key(key: Key) {}
    fn mark_contextual() {}
    fn resolve_to(target: Target) {}
    fn mark_failed() {}
}

#[derive(Default, Debug)]
pub(crate) struct References {
    // The version graph for this library: directed, possibly cyclic, possibly
    // disconnected. Contains only elements from the current library's platform:
    // all the current library's elements, plus elements from external libraries
    // it references. The latter have in-degree zero, i.e. they only appear as map
    // keys and never in the sets of outgoing neighbors.
    graph: Graph<u32, u32>,
}

pub(super) fn run_resolve_step(ctx: &mut Context<'_>) {
    let mut references = References::default();

    //for rf in ctx.types.relation_fields.iter() {
    //    let evidence = relation_evidence(rf, ctx);
    //    ingest_relation(evidence, &mut relations, ctx);
    //}

    for (decl_id, decl) in ctx.ast.iter_decls() {
        match (decl_id, decl) {
            (ast::TopId::Struct(struct_id), ast::Declaration::Struct(ast_struct)) => {
                visit_struct(struct_id, ast_struct, ctx)
            }
            (ast::TopId::Protocol(_), ast::Declaration::Protocol(proto)) => visit_protocol(proto, ctx),
            (ast::TopId::Const(_), ast::Declaration::Const(constant)) => visit_const(constant, ctx),
            (_, ast::Declaration::Import(_)) | (_, ast::Declaration::Library(_)) => (),
            _ => unreachable!(),
        }
    }

    let _ = std::mem::replace(ctx.references, references);
}

fn visit_struct<'db>(struct_id: ast::StructId, ast_struct: &'db ast::Struct, ctx: &mut Context<'db>) {
    for (member_id, ast_member) in ast_struct.iter_members() {}
}

fn visit_protocol<'db>(ast_protocol: &'db ast::Protocol, ctx: &mut Context<'db>) {
    for (ast_method) in ast_protocol.iter_methods() {}
}

fn visit_const<'db>(ast_const: &'db ast::ConstDeclaration, ctx: &mut Context<'db>) {}

/// Calls ref.SetKey, ref.MarkContextual, or ref.MarkFailed.
fn parse_reference(r#ref: &Reference, ctx: &mut Context<'_>) {}

/// Helpers for ParseReference.
fn parse_synthetic_reference(r#ref: &Reference, ctx: &mut Context<'_>) {}
fn parse_sourced_reference(r#ref: &Reference, ctx: &mut Context<'_>) {}

/// Inserts edges into graph_ for a parsed reference.
fn insert_reference_edges(r#ref: &Reference, ctx: &mut Context<'_>) {}

/// Calls ref.ResolveTo or ref.MarkFailed.
fn resolve_reference(r#ref: &Reference, ctx: &mut Context<'_>) {}

/// Helpers for ResolveReference.
fn resolve_contextual_reference(r#ref: &Reference, ctx: &mut Context<'_>) {}
fn resolve_key_reference(r#ref: &Reference, ctx: &mut Context<'_>) {}

// Validates a resolved reference (e.g. checks deprecation rules).
fn validate_reference(r#ref: &Reference, ctx: &mut Context<'_>) {}
