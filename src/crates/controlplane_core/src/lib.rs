pub mod discovery;
pub mod dynamic;
pub mod error;
pub mod request;
pub mod watch;

#[cfg_attr(docsrs, doc(cfg(feature = "schema")))]
#[cfg(feature = "schema")]
pub mod schema;

pub mod stored_object;
pub use stored_object::StoredObject;

pub mod gvk;
pub use gvk::{GroupVersion, GroupVersionKind};

pub mod object;
pub use object::{Object, ObjectList};

pub mod params;
pub use params::{GetParams, ListParams, PostParams, WatchParams};

pub mod metadata;
pub use metadata::TypeMeta;

pub mod resource;
pub use resource::{
    api_version_from_group_version, ClusterResourceScope, /*DynamicResourceScope*/ NamespaceResourceScope,
    Resource, ResourceExt,
};

pub mod response;
pub use response::Status;

pub mod crd;
pub use crd::v1::CustomResourceExt;
