use serde::{Deserialize, Serialize};

/// APIResourceList is a list of APIResource, it is used to expose the name of the resources supported in a specific group and version, and if the resource is namespaced.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct APIResourceList {
    pub kind: String,
    pub api_version: String,

    /// groupVersion is the group and version this APIResourceList is for.
    pub group_version: String,

    /// resources contains the name of the resources and if they are namespaced.
    pub resources: Vec<crate::apimachinery::apis::meta::v1::APIResource>,
}

impl crate::Resource for APIResourceList {
    const API_VERSION: &'static str = "v1";
    const GROUP: &'static str = "";
    const KIND: &'static str = "APIResourceList";
    const VERSION: &'static str = "v1";
    const URL_PATH_SEGMENT: &'static str = "";
    type Scope = crate::ClusterResourceScope;
}
