use serde::{Deserialize, Serialize};

/// APIVersions lists the versions that are available, to allow clients to discover the API at /api, which is the root path of the legacy v1 API.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct APIVersions {
    pub kind: String,
    pub api_version: String,

    /// a map of client CIDR to server address that is serving this group. This is to help clients reach servers in the most network-efficient way possible. Clients can use the appropriate server address as per the CIDR that they match. In case of multiple matches, clients should use the longest matching CIDR. The server returns only those CIDRs that it thinks that the client can match. For example: the master will return an internal IP CIDR only, if the client reaches the server using an internal IP. Server looks at X-Forwarded-For header or X-Real-Ip header or request.RemoteAddr (in that order) to get the client IP.
    #[serde(rename(serialize = "serverAddressByClientCIDRs"))]
    pub server_address_by_client_cidrs: Vec<crate::apimachinery::apis::meta::v1::ServerAddressByClientCIDR>,

    /// versions are the api versions that are available.
    pub versions: Vec<String>,
}

impl crate::Resource for APIVersions {
    const API_VERSION: &'static str = "v1";
    const GROUP: &'static str = "";
    const KIND: &'static str = "APIVersions";
    const VERSION: &'static str = "v1";
    const URL_PATH_SEGMENT: &'static str = "";
    type Scope = crate::ClusterResourceScope;
}
