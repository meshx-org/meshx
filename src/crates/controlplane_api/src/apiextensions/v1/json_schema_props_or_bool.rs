use serde::{Deserialize, Serialize};

/// JSONSchemaPropsOrBool represents JSONSchemaProps or a boolean value. Defaults to true for the boolean property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum JSONSchemaPropsOrBool {
    Schema(Box<crate::apiextensions::v1::JSONSchemaProps>),
    Bool(bool),
}