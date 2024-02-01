use std::{array::TryFromSliceError, fmt};

use serde::{Deserialize, Serialize};

pub fn version_history() -> Result<Vec<Version>, ()> {
    Ok(vec![Version {
        api_level: 1,
        abi_revision: AbiRevision::new(1),
        status: Status::Supported,
    }])
}

pub fn latest_sdk_version() -> Version {
    let versions = version_history().expect("version-history.json to be parsed");

    let latest_version = versions
        .last()
        .expect("version-history.json did not contain any versions");

    latest_version.clone()
}

/// VERSION_HISTORY is an array of all the known SDK versions.  It is guaranteed
/// (at compile-time) by the proc_macro to be non-empty.
///pub const VERSION_HISTORY: &[Version] = &[];

/// SUPPORTED_API_LEVELS are the supported API levels.
pub const SUPPORTED_API_LEVELS: &[Version] = &[];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct ApiLevel {
    pub abi_revision: AbiRevision,
    pub status: String,
}

/// An `AbiRevision` represents the ABI revision of a Fuchsia Package.
/// https://fuchsia.dev/fuchsia-src/contribute/governance/rfcs/0135_package_abi_revision?#design
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct AbiRevision(pub u64);

impl AbiRevision {
    pub const PATH: &'static str = "meta/fuchsia.abi/abi-revision";

    pub fn new(u: u64) -> AbiRevision {
        AbiRevision(u)
    }

    /// Parse the ABI revision from little-endian bytes.
    pub fn from_bytes(b: [u8; 8]) -> Self {
        AbiRevision(u64::from_le_bytes(b))
    }

    /// Encode the ABI revision into little-endian bytes.
    pub fn as_bytes(&self) -> [u8; 8] {
        self.0.to_le_bytes()
    }
}

impl fmt::Display for AbiRevision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl From<u64> for AbiRevision {
    fn from(abi_revision: u64) -> AbiRevision {
        AbiRevision(abi_revision)
    }
}

impl From<&u64> for AbiRevision {
    fn from(abi_revision: &u64) -> AbiRevision {
        AbiRevision(*abi_revision)
    }
}

impl From<AbiRevision> for u64 {
    fn from(abi_revision: AbiRevision) -> u64 {
        abi_revision.0
    }
}

impl TryFrom<&[u8]> for AbiRevision {
    type Error = TryFromSliceError;

    fn try_from(abi_revision: &[u8]) -> Result<AbiRevision, Self::Error> {
        let abi_revision: [u8; 8] = abi_revision.try_into()?;
        Ok(AbiRevision::from_bytes(abi_revision))
    }
}

impl std::ops::Deref for AbiRevision {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Hash)]
pub enum Status {
    #[serde(rename = "in-development")]
    InDevelopment,
    #[serde(rename = "supported")]
    Supported,
    #[serde(rename = "unsupported")]
    Unsupported,
}

/// Version is a mapping between the supported API level and the ABI revisions.
///
/// See https://fuchsia.dev/fuchsia-src/contribute/governance/rfcs/0002_platform_versioning for more
/// details.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Version {
    /// The API level denotes a set of APIs available when building an application for a given
    /// release of the FUCHSIA IDK.
    pub api_level: u64,

    /// The ABI revision denotes the semantics of the Fuchsia System Interface that an application
    /// expects the platform to provide.
    pub abi_revision: AbiRevision,

    /// The Status denotes the current status of the API level, either unsupported, supported, in-development.
    pub status: Status,
}

impl Version {
    /// Returns true if this version is supported - that is, whether components
    /// targeting this version will be able to run on this device.
    pub fn is_supported(&self) -> bool {
        match self.status {
            Status::InDevelopment | Status::Supported => true,
            Status::Unsupported => false,
        }
    }
}

pub fn version_from_abi_revision(abi_revision: AbiRevision) -> Option<Version> {
    // TODO(https://fxbug.dev/42068452): Store APIs and ABIs in a map instead of a list.
    version_history().unwrap().iter().find(|v| v.abi_revision == abi_revision).cloned()
}

/// Returns true if the given abi_revision is listed in the VERSION_HISTORY of
/// known SDK versions.
pub fn is_valid_abi_revision(abi_revision: AbiRevision) -> bool {
    version_from_abi_revision(abi_revision).is_some()
}

/// Returns true if the given abi_revision is listed in SUPPORTED_API_LEVELS.
pub fn is_supported_abi_revision(abi_revision: AbiRevision) -> bool {
    if let Some(version) = version_from_abi_revision(abi_revision) {
        version.is_supported()
    } else {
        false
    }
}

/// Returns a vector of the API levels in SUPPORTED_API_LEVELS.
pub fn get_supported_api_levels() -> Vec<u64> {
    let mut api_levels: Vec<u64> = Vec::new();
    for version in SUPPORTED_API_LEVELS {
        api_levels.push(version.api_level);
    }

    return api_levels;
}

/// Returns a vector of the ABI revisions in SUPPORTED_API_LEVELS.
pub fn get_supported_abi_revisions() -> Vec<u64> {
    let mut abi_revisions: Vec<u64> = Vec::new();
    for version in SUPPORTED_API_LEVELS {
        abi_revisions.push(version.abi_revision.0);
    }

    return abi_revisions;
}

pub fn get_latest_abi_revision() -> u64 {
    return latest_sdk_version().abi_revision.0;
}
