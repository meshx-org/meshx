use serde::{Deserialize, Serialize};

/// Info contains versioning information. how we'll want to distribute that information.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Info {
    pub build_date: String,
    pub compiler: String,
    pub git_commit: String,
    pub git_tree_state: String,
    pub git_version: String,
    pub major: String,
    pub minor: String,
    pub platform: String,
}