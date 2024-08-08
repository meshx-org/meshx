// Generated from definition io.k8s.apimachinery.pkg.apis.meta.v1.APIGroup

use serde::{Deserialize, Serialize};

/// APIGroup contains the name, the supported versions, and the preferred version of a group.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct APIGroup {
    pub kind: String,
    pub api_version: String,

    /// name is the name of the group.
    pub name: String,

    /// preferredVersion is the version preferred by the API server, which probably is the storage version.
    pub preferred_version: Option<crate::apimachinery::apis::meta::v1::APIGroupVersion>,

    /// a map of client CIDR to server address that is serving this group. This is to help clients reach servers in the most network-efficient way possible. Clients can use the appropriate server address as per the CIDR that they match. In case of multiple matches, clients should use the longest matching CIDR. The server returns only those CIDRs that it thinks that the client can match. For example: the master will return an internal IP CIDR only, if the client reaches the server using an internal IP. Server looks at X-Forwarded-For header or X-Real-Ip header or request.RemoteAddr (in that order) to get the client IP.
    #[serde(rename(serialize = "serverAddressByClientCIDRs"))]
    pub server_address_by_client_cidrs: Option<Vec<crate::apimachinery::apis::meta::v1::ServerAddressByClientCIDR>>,

    /// versions are the versions supported in this group.
    pub versions: Vec<crate::apimachinery::apis::meta::v1::APIGroupVersion>,
}

impl crate::Resource for APIGroup {
    const API_VERSION: &'static str = "v1";
    const GROUP: &'static str = "";
    const KIND: &'static str = "APIGroup";
    const VERSION: &'static str = "v1";
    const URL_PATH_SEGMENT: &'static str = "";
    type Scope = crate::ClusterResourceScope;
}
