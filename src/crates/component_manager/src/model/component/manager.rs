/// A special instance identified with component manager, at the top of the tree.
#[derive(Debug)]
pub struct ComponentManagerInstance {

}

impl ComponentManagerInstance {
    pub fn new(
        //namespace_capabilities: NamespaceCapabilities,
        //builtin_capabilities: BuiltinCapabilities,
    ) -> Self {
        Self {
            //namespace_capabilities,
            //builtin_capabilities,
            //state: Mutex::new(ComponentManagerInstanceState::new()),
            //task_group: TaskGroup::new(),
        }
    }
}