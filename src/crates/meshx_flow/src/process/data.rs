use crate::errors::FlowError;
use std::collections::HashMap;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum RuntimeType {
    Null,
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<RuntimeType>),
    Map(HashMap<String, RuntimeType>),
}

/// Holds the node output data
#[derive(Debug)]
pub struct OutputPorts {
    pub(crate) ports: HashMap<String, Arc<RuntimeType>>,
}

impl OutputPorts {
    pub(crate) fn new() -> Self {
        OutputPorts { ports: HashMap::new() }
    }

    pub(crate) fn write(&mut self, port: String, data: RuntimeType) -> Result<(), FlowError> {
        self.ports.insert(port, Arc::from(data));
        Ok(())
    }
}

/// Holds the node input data
#[derive(Debug)]
pub struct InputPorts {
    pub(crate) ports: HashMap<String, Arc<RuntimeType>>,
}

impl InputPorts {
    pub(crate) fn new() -> Self {
        InputPorts { ports: HashMap::new() }
    }

    pub(crate) fn read(&self, port: String) -> Result<Arc<RuntimeType>, FlowError> {
        let data = self.ports.get(&port).cloned();
        data.ok_or(FlowError::UnknownPort(port))
    }
}
