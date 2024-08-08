// Generated from definition io.k8s.api.core.v1.NamespaceSpec

use serde::{Deserialize, Serialize};

/// NamespaceSpec describes the attributes on a Namespace.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct NamespaceSpec {
    /// Finalizers is an opaque list of values that must be empty to permanently remove object from storage. More info: https://kubernetes.io/docs/tasks/administer-cluster/namespaces/
    pub finalizers: Option<Vec<String>>,
}
