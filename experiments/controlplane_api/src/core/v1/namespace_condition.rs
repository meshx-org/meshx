use serde::{Deserialize, Serialize};

/// NamespaceCondition contains details about state of namespace.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct NamespaceCondition {
    // pub last_transition_time: Option<crate::apimachinery::pkg::apis::meta::v1::Time>,

    pub message: Option<String>,

    pub reason: Option<String>,

    /// Status of the condition, one of True, False, Unknown.
    pub status: String,

    /// Type of namespace controller condition.
    pub type_: String,
}
