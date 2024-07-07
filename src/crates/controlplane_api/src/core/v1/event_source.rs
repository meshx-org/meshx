use serde::{Deserialize, Serialize};

/// EventSource contains information for an event.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EventSource {
    /// Component from which the event is generated.
    pub component: Option<String>,

    /// Node name on which the event is generated.
    pub host: Option<String>,
}