use super::data::OutputPorts;

/// A temporary memory for flow processing. It needs to be serializable
#[derive(Debug)]
pub(crate) struct NodeState {
    pub processed: bool,
    pub outputs: OutputPorts,
}
