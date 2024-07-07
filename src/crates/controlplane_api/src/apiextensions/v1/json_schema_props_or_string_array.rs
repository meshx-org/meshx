use serde::{Deserialize, Serialize};

/// JSONSchemaPropsOrStringArray represents a JSONSchemaProps or a string array.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum JSONSchemaPropsOrStringArray {
    Schema(Box<crate::apiextensions::v1::JSONSchemaProps>),
    Strings(Vec<String>),
}
