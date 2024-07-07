//! Contains types for using resource kinds not known at compile-time.
//!
//! For concrete usage see [examples prefixed with dynamic_](https://github.com/kube-rs/kube/tree/main/examples).
use controlplane_api::apimachinery::apis::meta::v1::ObjectMeta;
use std::borrow::Cow;
use thiserror::Error;

use super::{metadata::TypeMeta, object::DynamicResourceScope, resource::Resource};
pub use crate::discovery::ApiResource;

#[derive(Debug, Error)]
#[error("failed to parse this DynamicObject into a Resource: {source}")]
/// Failed to parse `DynamicObject` into `Resource`
pub struct ParseDynamicObjectError {
    #[from]
    source: serde_json::Error,
}

/// A dynamic representation of a kubernetes object
///
/// This will work with any non-list type object.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct DynamicObject {
    /// The type fields, not always present
    #[serde(flatten, default)]
    pub types: Option<TypeMeta>,

    /// Object metadata
    #[serde(default)]
    pub metadata: ObjectMeta,

    /// All other keys
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl DynamicObject {
    /// Create a DynamicObject with minimal values set from ApiResource.
    #[must_use]
    pub fn new(name: &str, resource: &ApiResource) -> Self {
        Self {
            types: Some(TypeMeta {
                api_version: resource.api_version.to_string(),
                kind: resource.kind.to_string(),
            }),
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                ..Default::default()
            },
            data: Default::default(),
        }
    }

    /// Attach dynamic data to a DynamicObject
    #[must_use]
    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Attach a namespace to a DynamicObject
    #[must_use]
    pub fn within(mut self, ns: &str) -> Self {
        self.metadata.namespace = Some(ns.into());
        self
    }

    /// Attempt to convert this `DynamicObject` to a `Resource`
    pub fn try_parse<K: Resource + for<'a> serde::Deserialize<'a>>(self) -> Result<K, ParseDynamicObjectError> {
        Ok(serde_json::from_value(serde_json::to_value(self)?)?)
    }
}

impl Resource for DynamicObject {
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

    fn api_version(dt: &ApiResource) -> Cow<'_, str> {
        dt.api_version.as_str().into()
    }

    fn plural(dt: &ApiResource) -> Cow<'_, str> {
        dt.plural.as_str().into()
    }

    fn meta(&self) -> &ObjectMeta {
        &self.metadata
    }

    fn meta_mut(&mut self) -> &mut ObjectMeta {
        &mut self.metadata
    }
}
