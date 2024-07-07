use serde::{Deserialize, Serialize};

/// CustomResourceValidation is a list of validation methods for CustomResources.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CustomResourceValidation {
    /// openAPIV3Schema is the OpenAPI v3 schema to use for validation and pruning.
    #[serde(rename = "openAPIV3Schema")]
    pub open_api_v3_schema: Option<crate::apiextensions::v1::JSONSchemaProps>,
}
