use std::{cell::RefCell, rc::Rc};

use super::compile_step::CompileStep;
use crate::ast;

/// TypeResolver exposes resolve_* methods from CompileStep to Typespace and Type.

pub struct TypeResolver<'c, 'd> {
    compile_step: &'c CompileStep<'d, 'd>,
}

impl<'c, 'd> TypeResolver<'c, 'd> {
    pub fn new(compile_step: &'c CompileStep<'d, 'd>) -> Self {
        Self { compile_step }
    }

    pub fn compile_decl(&self, decl: &mut ast::Declaration) {
        self.compile_step.compile_decl(decl);
    }

    pub fn resolve_type(&self, type_ctor: &mut ast::TypeConstructor) -> bool {
        self.compile_step.compile_type_constructor(type_ctor);
        type_ctor.r#type.is_some()
    }

    pub fn get_decl_cycle(&self, decl: &mut ast::Declaration) -> Option<Vec<ast::Declaration>> {
        self.compile_step.get_decl_cycle(decl)
    }

    pub fn resolve_param_as_size(
        &self,
        layout: &ast::Reference,
        param: &ast::LayoutParameter,
    ) -> Result<ast::ConstantValue, ()> {
        Ok(ast::ConstantValue::Uint32(0))
    }

    pub fn resolve_as_protocol(&self, constant: &ast::Constant) -> Option<Rc<RefCell<ast::Protocol>>> {
        if let ast::Constant::Identifier(identifier) = constant {
            let target = identifier.reference.resolved().unwrap().element();
            if let ast::Element::Protocol { inner } = target {
                Some(inner)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn resolve_as_optional(&self, constant: &ast::Constant) -> bool {
        self.compile_step.resolve_as_optional(constant)
    }

    pub fn resolve_param_as_type(
        &self,
        layout: &ast::Reference,
        param: &ast::LayoutParameter,
    ) -> Result<ast::Type, ()> {
        let type_ctor = param.as_type_ctor();
        let check = self.compile_step.ctx.diagnostics.checkpoint();

        if type_ctor.is_none() {
            // if there were no errors reported but we couldn't resolve to a type, it must
            // mean that the parameter referred to a non-type, so report a new error here.
            if check.no_new_errors() {
                panic!("ErrExpectedType");
                // return reporter().Fail(ErrExpectedType, param.span);
            }

            // otherwise, there was an error during the type resolution process, so we
            // should just report that rather than add an extra error here
            return Err(());
        }

        let mut type_ctor = type_ctor.unwrap();

        if !self.resolve_type(&mut type_ctor) {
            // if there were no errors reported but we couldn't resolve to a type, it must
            // mean that the parameter referred to a non-type, so report a new error here.
            if check.no_new_errors() {
                panic!("ErrExpectedType");
                //return reporter().Fail(ErrExpectedType, param.span);
            }

            // otherwise, there was an error during the type resolution process, so we
            // should just report that rather than add an extra error here
            return Err(());
        }

        Ok(type_ctor.r#type.clone().unwrap())
    }
}
