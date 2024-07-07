use serde::{Deserialize, Serialize};

/// CustomResourceDefinitionVersion describes a version for CRD.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomResourceDefinitionVersion {
    // additionalPrinterColumns specifies additional columns returned in Table output. See https://kubernetes.io/docs/reference/using-api/api-concepts/#receiving-resources-as-tables for details. If no columns are specified, a single column displaying the age of the custom resource is used.
    // pub additional_printer_columns: Option<Vec<crate::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceColumnDefinition>>,

    /// deprecated indicates this version of the custom resource API is deprecated. When set to true, API requests to this version receive a warning header in the server response. Defaults to false.
    pub deprecated: Option<bool>,

    /// deprecationWarning overrides the default warning returned to API clients. May only be set when `deprecated` is true. The default warning indicates this version is deprecated and recommends use of the newest served version of equal or greater stability, if one exists.
    pub deprecation_warning: Option<String>,

    /// name is the version name, e.g. “v1”, “v2beta1”, etc. The custom resources are served under this version at `/apis/\<group\>/\<version\>/...` if `served` is true.
    pub name: String,

    // schema describes the schema used for validation, pruning, and defaulting of this version of the custom resource.
    pub schema: Option<crate::apiextensions::v1::CustomResourceValidation>,

    /// served is a flag enabling/disabling this version from being served via REST APIs
    pub served: bool,

    /// storage indicates this version should be used when persisting custom resources to storage. There must be exactly one version with storage=true.
    pub storage: bool,

    // subresources specify what subresources this version of the defined custom resource have.
    // pub subresources: Option<crate::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceSubresources>,
}