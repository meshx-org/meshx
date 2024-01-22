use super::Context;

pub(crate) struct CompileStep {}

impl CompileStep {
    pub fn new<'ctx>(ctx: &'ctx Context<'_>) -> Self {
        Self {}
    }

    pub(crate) fn run(&self) -> bool {
        log::debug!("running compile step");

        true
    }
}
