//! Type information structs for API discovery
use crate::{gvk::GroupVersionKind, resource::Resource};
use serde::{Deserialize, Serialize};

/// Information about a Kubernetes API resource
///
/// Enough information to use it like a `Resource` by passing it to the dynamic `Api`
/// constructors like `Api::all_with` and `Api::namespaced_with`.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApiResource {
    /// Resource group, empty for core group.
    pub group: String,
    /// group version
    pub version: String,
    /// apiVersion of the resource (v1 for core group,
    /// groupName/groupVersions for other).
    pub api_version: String,
    /// Singular PascalCase name of the resource
    pub kind: String,
    /// Plural name of the resource
    pub plural: String,
}

impl ApiResource {
    /// Creates an ApiResource by type-erasing a Resource
    pub fn erase<K: Resource>(dt: &K::DynamicType) -> Self {
        ApiResource {
            group: K::group(dt).to_string(),
            version: K::version(dt).to_string(),
            api_version: K::api_version(dt).to_string(),
            kind: K::kind(dt).to_string(),
            plural: K::plural(dt).to_string(),
        }
    }

    /// Creates an ApiResource from group, version, kind and plural name.
    pub fn from_gvk_with_plural(gvk: &GroupVersionKind, plural: &str) -> Self {
        ApiResource {
            api_version: gvk.api_version(),
            group: gvk.group.clone(),
            version: gvk.version.clone(),
            kind: gvk.kind.clone(),
            plural: plural.to_string(),
        }
    }
}
