use std::{borrow::Cow, marker::PhantomData};

use crate::{dynamic::DynamicObject, resource::Resource};
pub use controlplane_api::apimachinery::apis::meta::v1::{ListMeta, ObjectMeta};
use serde::{Deserialize, Serialize};

/// Type information that is flattened into every kubernetes object
#[derive(Deserialize, Serialize, Clone, Default, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TypeMeta {
    /// The version of the API
    pub api_version: String,

    /// The name of the API
    pub kind: String,
}

impl TypeMeta {
    /// Construct a new `TypeMeta` for the object list from the given resource.
    ///
    /// ```no_run
    /// # use k8s_openapi::api::core::v1::Pod;
    /// # use kube_core::TypeMeta;
    ///
    /// let type_meta = TypeMeta::list::<Pod>();
    /// assert_eq!(type_meta.kind, "PodList");
    /// assert_eq!(type_meta.api_version, "v1");
    /// ```
    pub fn list<K: Resource<DynamicType = ()>>() -> Self {
        TypeMeta {
            api_version: K::api_version(&()).into(),
            kind: K::kind(&()).to_string() + "List",
        }
    }

    /// Construct a new `TypeMeta` for the object from the given resource.
    ///
    /// ```no_run
    /// # use k8s_openapi::api::core::v1::Pod;
    /// # use kube_core::TypeMeta;
    ///
    /// let type_meta = TypeMeta::resource::<Pod>();
    /// assert_eq!(type_meta.kind, "Pod");
    /// assert_eq!(type_meta.api_version, "v1");
    /// ```
    pub fn resource<K: Resource<DynamicType = ()>>() -> Self {
        TypeMeta {
            api_version: K::api_version(&()).into(),
            kind: K::kind(&()).into(),
        }
    }
}

/// A generic representation of any object with `ObjectMeta`.
///
/// It allows clients to get access to a particular `ObjectMeta`
/// schema without knowing the details of the version.
///
/// See the [`PartialObjectMetaExt`] trait for how to construct one safely.
#[derive(Deserialize, Serialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PartialObjectMeta<K = DynamicObject> {
    /// The type fields, not always present
    #[serde(flatten, default)]
    pub types: Option<TypeMeta>,

    /// Standard object's metadata
    #[serde(default)]
    pub metadata: ObjectMeta,

    /// Type information for static dispatch
    #[serde(skip, default)]
    pub _phantom: PhantomData<K>,
}

impl<K: Resource> Resource for PartialObjectMeta<K> {
    type DynamicType = K::DynamicType;
    type Scope = K::Scope;

    fn kind(dt: &Self::DynamicType) -> Cow<'_, str> {
        K::kind(dt)
    }

    fn group(dt: &Self::DynamicType) -> Cow<'_, str> {
        K::group(dt)
    }

    fn version(dt: &Self::DynamicType) -> Cow<'_, str> {
        K::version(dt)
    }

    fn plural(dt: &Self::DynamicType) -> Cow<'_, str> {
        K::plural(dt)
    }

    fn meta(&self) -> &ObjectMeta {
        &self.metadata
    }

    fn meta_mut(&mut self) -> &mut ObjectMeta {
        &mut self.metadata
    }
}
