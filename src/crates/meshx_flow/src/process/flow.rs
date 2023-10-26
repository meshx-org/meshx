use crate::errors::FlowError;
use std::collections::HashMap;

/// Holds all the possible callable flow outputs
pub struct FlowControls<'a> {
    // Contains a map of output port <-> node id conversion
    pub(crate) port_mapping: HashMap<String, String>,
    pub(crate) flow_stack: &'a mut Vec<String>,
}

impl FlowControls<'_> {
    pub(crate) fn flow(&mut self, port: String) -> Result<(), FlowError> {
        let nid = self.port_mapping.get(&port);
        let nid = nid.ok_or(FlowError::UnknownPort(port))?;

        // We are cloning instead of remoce because on port can be called
        // multiple times
        self.flow_stack.push(nid.clone());

        Ok(())
    }
}
