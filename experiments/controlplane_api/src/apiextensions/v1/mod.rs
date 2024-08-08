pub mod crd;
pub use self::crd::CustomResourceDefinition;

pub mod crd_spec;
pub use self::crd_spec::CustomResourceDefinitionSpec;

pub mod crd_version;
pub use self::crd_version::CustomResourceDefinitionVersion;

pub mod crd_names;
pub use self::crd_names::CustomResourceDefinitionNames;

pub mod crd_status;
pub use self::crd_status::CustomResourceDefinitionStatus;

pub mod custom_resource_column_definition;
pub use custom_resource_column_definition::CustomResourceColumnDefinition;

pub mod crd_validation;
pub use crd_validation::CustomResourceValidation;

pub mod json;
pub use json::JSON;

pub mod json_schema_props;
pub use json_schema_props::JSONSchemaProps;

pub mod json_schema_props_or_bool;
pub use json_schema_props_or_bool::JSONSchemaPropsOrBool;

pub mod json_schema_props_or_string_array;
pub use json_schema_props_or_string_array::JSONSchemaPropsOrStringArray;