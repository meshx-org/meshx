// Generated from definition io.k8s.api.core.v1.Namespace

use serde::{Deserialize, Serialize};

use super::namespace_spec::NamespaceSpec;
use super::namespace_status::NamespaceStatus;

/// Namespace provides a scope for Names. Use of multiple namespaces is optional.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Namespace {
    /// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata
    pub metadata: crate::apimachinery::apis::meta::v1::ObjectMeta,

    /// Spec defines the behavior of the Namespace. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#spec-and-status
    pub spec: Option<NamespaceSpec>,

    // Status describes the current status of a Namespace. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#spec-and-status
    pub status: Option<NamespaceStatus>,
}

impl crate::Resource for Namespace {
    const API_VERSION: &'static str = "core.k8s.io/v1";
    const GROUP: &'static str = "core.k8s.io";
    const KIND: &'static str = "Namespace";
    const VERSION: &'static str = "v1";
    const URL_PATH_SEGMENT: &'static str = "namespaces";
    type Scope = crate::ClusterResourceScope;
}

impl crate::ListableResource for Namespace {
    const LIST_KIND: &'static str = "NamespaceList";
}

impl crate::Metadata for Namespace {
    type Ty = crate::apimachinery::apis::meta::v1::ObjectMeta;

    fn metadata(&self) -> &<Self as crate::Metadata>::Ty {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut<Self as crate::Metadata>::Ty {
        &mut self.metadata
    }
}