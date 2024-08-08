use serde::{Deserialize, Serialize};

use super::crd_spec::CustomResourceDefinitionSpec;
use super::crd_status::CustomResourceDefinitionStatus;

/// CustomResourceDefinition represents a resource that should be exposed on the API server.  Its name MUST be in the format \<.spec.name\>.\<.spec.group\>.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct CustomResourceDefinition {
    /// Standard object's metadata More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata
    pub metadata: crate::apimachinery::apis::meta::v1::ObjectMeta,

    /// spec describes how the user wants the resources to appear
    pub spec: CustomResourceDefinitionSpec,

    /// status indicates the actual state of the CustomResourceDefinition
    pub status: Option<CustomResourceDefinitionStatus>,
}

impl serde::Serialize for CustomResourceDefinition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let mut state = serializer.serialize_struct(
            <Self as crate::Resource>::KIND,
            4 +
            self.status.as_ref().map_or(0, |_| 1),
        )?;
        serde::ser::SerializeStruct::serialize_field(&mut state, "apiVersion", <Self as crate::Resource>::API_VERSION)?;
        serde::ser::SerializeStruct::serialize_field(&mut state, "kind", <Self as crate::Resource>::KIND)?;
        serde::ser::SerializeStruct::serialize_field(&mut state, "metadata", &self.metadata)?;
        serde::ser::SerializeStruct::serialize_field(&mut state, "spec", &self.spec)?;
        if let Some(value) = &self.status {
            serde::ser::SerializeStruct::serialize_field(&mut state, "status", value)?;
        }
        serde::ser::SerializeStruct::end(state)
    }
}

impl crate::Resource for CustomResourceDefinition {
    const API_VERSION: &'static str = "apiextensions.k8s.io/v1";
    const GROUP: &'static str = "apiextensions.k8s.io";
    const KIND: &'static str = "CustomResourceDefinition";
    const VERSION: &'static str = "v1";
    const URL_PATH_SEGMENT: &'static str = "customresourcedefinitions";
    type Scope = crate::ClusterResourceScope;
}

impl crate::ListableResource for CustomResourceDefinition {
    const LIST_KIND: &'static str = "CustomResourceDefinitionList";
}

impl crate::Metadata for CustomResourceDefinition {
    type Ty = crate::apimachinery::apis::meta::v1::ObjectMeta;

    fn metadata(&self) -> &<Self as crate::Metadata>::Ty {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut <Self as crate::Metadata>::Ty {
        &mut self.metadata
    }
}
