use serde::{Deserialize, Serialize};

/// ServerAddressByClientCIDR helps the client to determine the server address that they should use, depending on the clientCIDR that they match.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerAddressByClientCIDR {
    /// The CIDR with which clients can match their IP to figure out the server address that they should use.
    #[serde(rename(serialize = "clientCIDR"))]
    pub client_cidr: String,

    /// Address of this server, suitable for a client that matches the above CIDR. This can be a hostname, hostname:port, IP or IP:port.
    pub server_address: String,
}