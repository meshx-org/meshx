use std::fmt;

#[allow(dead_code)]
pub enum FlowError {
    AlreadyEvaulated,
    MissingNode,
    MissingNodeState,
    MissingWorker(String),
    UnknownPort(String),
    FailedDispatch,
}

// Implement std::fmt::Debug for FlowError
impl fmt::Debug for FlowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FlowError::AlreadyEvaulated => write!(f, "the node are already evaulated"),
            FlowError::MissingNodeState => write!(f, "missing node state",),
            FlowError::MissingNode => write!(f, "missing node"),
            FlowError::FailedDispatch => write!(f, "dispatch failed"),
            FlowError::MissingWorker(cid) => write!(f, "worker not registered: {}", cid),
            FlowError::UnknownPort(port) => write!(f, "unknow port: {}", port),
        }
    }
}
