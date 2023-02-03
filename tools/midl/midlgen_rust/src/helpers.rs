use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, JsonValue, Output, RenderError};
use handlebars::{RenderContext, Renderable};

#[derive(Clone, Copy)]
pub struct IfCondHelper {}

impl HelperDef for IfCondHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param_a = h
            .param(0)
            .ok_or_else(|| RenderError::new("Param 'a' not found for helper \"if_cond\""))?;
        let param_op = h
            .param(1)
            .ok_or_else(|| RenderError::new("Param 'op' not found for helper \"if_cond\""))?;
        let param_b = h
            .param(2)
            .ok_or_else(|| RenderError::new("Param 'b' not found for helper \"if_cond\""))?;

        let param_a = param_a
            .value()
            .as_bool()
            .ok_or_else(|| RenderError::new("Param 'a' is not a boolean"))?;

        let param_op = param_op
            .value()
            .as_str()
            .ok_or_else(|| RenderError::new("Param 'op' is not a string"))?;

        let param_b = param_b
            .value()
            .as_bool()
            .ok_or_else(|| RenderError::new("Param 'b' is not a boolean"))?;

        let tmpl = match param_op {
            "&&" => {
                if param_a && param_b {
                    h.template()
                } else {
                    h.inverse()
                }
            }
            _ => unimplemented!(),
        };

        match tmpl {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PrintfHelper {}

impl HelperDef for PrintfHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars<'reg>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let tmpl = h.template();

        match tmpl {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }
}
