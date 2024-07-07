use serde::{Deserialize, Serialize};

/// GroupVersion contains the "group/version" and "version" string of a version. It is made a struct to keep extensibility.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct APIGroupVersion {
    /// groupVersion specifies the API group and version in the form "group/version"
    pub group_version: String,

    /// version specifies the version in the form of "version". This is to save the clients the trouble of splitting the GroupVersion.
    pub version: String,
}
