use serde::{Deserialize, Serialize};

use super::namespace_condition::NamespaceCondition;

/// NamespaceStatus is information about the current status of a Namespace.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct NamespaceStatus {
    /// Represents the latest available observations of a namespace's current state.
    pub conditions: Option<Vec<NamespaceCondition>>,

    /// Phase is the current lifecycle phase of the namespace. More info: https://kubernetes.io/docs/tasks/administer-cluster/namespaces/
    pub phase: Option<String>,
}