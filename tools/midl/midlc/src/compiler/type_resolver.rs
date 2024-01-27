use crate::ast;

use super::compile_step::CompileStep;

/// TypeResolver exposes resolve_* methods from CompileStep to Typespace and Type.

pub struct TypeResolver;

impl TypeResolver {
    pub fn new<'ctx, 'd>(step: &CompileStep<'ctx, 'd>) -> Self {
        Self
    }


    pub fn compile_decl(&self, decl: ast::Declaration) {

    }
}