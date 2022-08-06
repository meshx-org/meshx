use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PackageMetadata {
    api_version: String,
    keywords: Option<Vec<String>>,

    pub name: String,
    version: String,
    description: Option<String>,

    pub manifests: Vec<String>,
    pub files: Vec<String>,
}