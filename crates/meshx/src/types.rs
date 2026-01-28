use serde::Deserialize;
use serde::de::Deserializer;
use std::collections::{HashMap, HashSet};
use wash_runtime::wit::WitInterface;

pub(crate) type WorkloadKey = (String, String); // (namespace, name)

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub metadata: Option<Metadata>,
    pub workloads: Vec<Workload>,
    pub resources: Option<HashMap<String, Resource>>,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct Resource {
    #[serde(rename = "type")]
    pub resource_type: String,
}

/// The type of volume - either host path or empty directory.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum VolumeType {
    HostPath(HostPathVolume),
    EmptyDir(EmptyDirVolume),
}

/// An ephemeral empty directory volume that exists for the lifetime of the workload.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct EmptyDirVolume {}

/// A volume that mounts a directory from the host filesystem.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct HostPathVolume {
    pub local_path: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Volume {
    pub name: String,
    pub volume_type: VolumeType,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Workload {
    pub name: String,
    pub namespace: String,
    pub annotations: Option<HashMap<String, String>>,
    pub service: Option<Service>,
    pub components: Option<Vec<Component>>,
    #[serde(rename = "hostInterfaces")]
    pub host_interfaces: Option<Vec<HostInterface>>,
    pub volumes: Option<Vec<Volume>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageRef {
    Oci(String),
    Blob(String),
    File(String),
}

impl<'de> Deserialize<'de> for ImageRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if let Some(rest) = s.strip_prefix("oci://") {
            Ok(ImageRef::Oci(rest.to_string()))
        } else if let Some(rest) = s.strip_prefix("blob://") {
            Ok(ImageRef::Blob(rest.to_string()))
        } else if let Some(rest) = s.strip_prefix("file://") {
            Ok(ImageRef::File(rest.to_string()))
        } else {
            // default to OCI if no prefix is given
            Ok(ImageRef::Oci(s))
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Service {
    pub name: String,
    pub image: ImageRef,
    #[serde(rename = "maxRestarts")]
    pub max_restarts: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Component {
    pub name: String,
    pub image: ImageRef, // e.g. oci://ghcr.io/...
    #[serde(rename = "poolSize")]
    pub pool_size: i32,
    #[serde(rename = "maxInvocations")]
    pub max_invocations: i32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct HostInterface {
    pub namespace: String,
    pub package: String,
    pub interfaces: HashSet<String>,
    pub config: HashMap<String, String>,
}

impl From<&HostInterface> for WitInterface {
    fn from(hi: &HostInterface) -> Self {
        WitInterface {
            namespace: hi.namespace.clone(),
            package: hi.package.clone(),
            interfaces: hi.interfaces.iter().cloned().collect(),
            config: hi.config.clone(),
            version: None,
        }
    }
}

impl From<HostInterface> for WitInterface {
    fn from(hi: HostInterface) -> Self {
        WitInterface {
            namespace: hi.namespace,
            package: hi.package,
            interfaces: hi.interfaces.into_iter().collect(),
            config: hi.config.clone(),
            version: None,
        }
    }
}
