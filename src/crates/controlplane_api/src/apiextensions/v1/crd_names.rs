use serde::{Deserialize, Serialize};

/// CustomResourceDefinitionNames indicates the names to serve this CustomResourceDefinition
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomResourceDefinitionNames {
    /// categories is a list of grouped resources this custom resource belongs to (e.g. 'all'). This is published in API discovery documents, and used by clients to support invocations like `kubectl get all`.
    pub categories: Option<Vec<String>>,

    /// kind is the serialized kind of the resource. It is normally CamelCase and singular. Custom resource instances will use this value as the `kind` attribute in API calls.
    pub kind: String,

    /// listKind is the serialized kind of the list for this resource. Defaults to "`kind`List".
    pub list_kind: Option<String>,

    /// plural is the plural name of the resource to serve. The custom resources are served under `/apis/\<group\>/\<version\>/.../\<plural\>`. Must match the name of the CustomResourceDefinition (in the form `\<names.plural\>.\<group\>`). Must be all lowercase.
    pub plural: String,

    /// shortNames are short names for the resource, exposed in API discovery documents, and used by clients to support invocations like `kubectl get \<shortname\>`. It must be all lowercase.
    pub short_names: Option<Vec<String>>,

    /// singular is the singular name of the resource. It must be all lowercase. Defaults to lowercased `kind`.
    pub singular: Option<String>,
}