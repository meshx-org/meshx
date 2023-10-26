use crate::flow::FlowModule;
use crate::process::Process;
use crate::workers::Workers;

use std::rc::Rc;

pub struct Module(pub FlowModule);

pub struct Modules {
    modules: Vec<Rc<Module>>,
}

pub struct FlowVM<'h> {
    workers: Rc<Workers>,
    modules: Modules,
    import_hook: &'h dyn Fn(String) -> Module,
}

impl<'h> FlowVM<'h> {
    pub fn new(workers: Workers, import_hook: &'h dyn Fn(String) -> Module) -> Self {
        FlowVM {
            workers: Rc::new(workers),
            modules: Modules { modules: vec![] },
            import_hook,
        }
    }

    /// Import a module into the vm and return
    pub(crate) fn import(&mut self, identifier: &str) {
        let module = (*self.import_hook)(identifier.to_owned());
        self.modules.modules.push(Rc::new(module));
    }

    pub(crate) fn invoke(&self, event_name: &str) {
        let main_module = self
            .modules
            .modules
            .last()
            .expect("should imported at least one module");

        // Run a single flow in an process (blocking)
        let mut process = Process::new(Rc::clone(&self.workers), main_module.0.graph.clone());

        process.dispatch("1".to_string()).unwrap();
    }
}
