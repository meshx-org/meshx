use std::borrow::Cow;

use controlplane_api::apimachinery::apis::meta::v1::ObjectMeta;
use serde::{Deserialize, Serialize};

use crate::{
    discovery::ApiResource,
    metadata::TypeMeta,
    object::{HasSpec, HasStatus},
    resource::Resource,
};

/// Indicates that a [`Resource`] is of an indeterminate dynamic scope.
pub struct DynamicResourceScope {}
// impl ResourceScope for DynamicResourceScope {}

/// A standard Kubernetes object with `.spec` and `.status`.
///
/// This is a convenience struct provided for serialization/deserialization.
/// It is slightly stricter than ['DynamicObject`] in that it enforces the spec/status convention,
/// and as such will not in general work with all api-discovered resources.
///
/// This can be used to tie existing resources to smaller, local struct variants to optimize for memory use.
/// E.g. if you are only interested in a few fields, but you store tons of them in memory with reflectors.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StoredObject<P, U>
where
    P: Clone,
    U: Clone,
{
    /// The type fields, not always present
    #[serde(flatten, default)]
    pub types: Option<TypeMeta>,

    /// Resource metadata
    ///
    /// Contains information common to most resources about the Resource,
    /// including the object name, annotations, labels and more.
    pub metadata: ObjectMeta,

    /// The Spec struct of a resource. I.e. `PodSpec`, `DeploymentSpec`, etc.
    ///
    /// This defines the desired state of the Resource as specified by the user.
    pub spec: P,

    /// The Status of a resource. I.e. `PodStatus`, `DeploymentStatus`, etc.
    ///
    /// This publishes the state of the Resource as observed by the controller.
    /// Use `U = NotUsed` when a status does not exist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<U>,
}

impl<P, U> StoredObject<P, U>
where
    P: Clone,
    U: Clone,
{
    // /// A constructor that takes Resource values from an `ApiResource`
    pub fn new(name: &str, ar: &ApiResource, spec: P) -> Self {
        Self {
            types: Some(TypeMeta {
                api_version: ar.api_version.clone(),
                kind: ar.kind.clone(),
            }),
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                ..Default::default()
            },
            spec,
            status: None,
        }
    }

    /// Attach a namespace to an Object
    #[must_use]
    pub fn within(mut self, ns: &str) -> Self {
        self.metadata.namespace = Some(ns.into());
        self
    }
}

impl<P, U> Resource for StoredObject<P, U>
where
    P: Clone,
    U: Clone,
{
    type DynamicType = ApiResource;
    type Scope = DynamicResourceScope;

    fn group(dt: &ApiResource) -> Cow<'_, str> {
        dt.group.as_str().into()
    }

    fn version(dt: &ApiResource) -> Cow<'_, str> {
        dt.version.as_str().into()
    }

    fn kind(dt: &ApiResource) -> Cow<'_, str> {
        dt.kind.as_str().into()
    }

    fn plural(dt: &ApiResource) -> Cow<'_, str> {
        dt.plural.as_str().into()
    }

    fn api_version(dt: &ApiResource) -> Cow<'_, str> {
        dt.api_version.as_str().into()
    }

    fn meta(&self) -> &ObjectMeta {
        &self.metadata
    }

    fn meta_mut(&mut self) -> &mut ObjectMeta {
        &mut self.metadata
    }
}

impl<P, U> HasSpec for StoredObject<P, U>
where
    P: Clone,
    U: Clone,
{
    type Spec = P;

    fn spec(&self) -> &Self::Spec {
        &self.spec
    }

    fn spec_mut(&mut self) -> &mut Self::Spec {
        &mut self.spec
    }
}

impl<P, U> HasStatus for StoredObject<P, U>
where
    P: Clone,
    U: Clone,
{
    type Status = U;

    fn status(&self) -> Option<&Self::Status> {
        self.status.as_ref()
    }

    fn status_mut(&mut self) -> &mut Option<Self::Status> {
        &mut self.status
    }
}
