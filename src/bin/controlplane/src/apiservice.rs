use controlplane_api::{apimachinery::apis::meta::v1::APIResource, apimachinery::apis::meta::v1::ObjectMeta};

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct APIServiceCondition {
    status: String,
    r#type: String,
    message: Option<String>,
    reason: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct APIServiceStatus {
    conditions: Vec<APIServiceCondition>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct APIServiceSpec {
    pub group: String,
    pub version: String,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct APIService {
    pub kind: String,
    pub api_version: String,
    pub metadata: ObjectMeta,
    pub spec: APIServiceSpec,
    pub status: APIServiceStatus,
}

impl APIService {
    pub fn new(group: String, version: String) -> APIService {
        APIService {
            kind: String::from("APIService"),
            api_version: String::from("v1"),
            metadata: ObjectMeta {
                name: Some(group.clone()),
                namespace: None,
                labels: None,
                annotations: None,
                resource_version: None,
                finalizers: None,
                generate_name: None,
                generation: None,
                uid: None,
                creation_timestamp: None,
                deletion_grace_period_seconds: None,
                deletion_timestamp: None,
            },
            spec: APIServiceSpec { group, version },
            status: APIServiceStatus {
                conditions: vec![APIServiceCondition {
                    r#type: String::from("Available"),
                    status: String::from("True"),
                    reason: Some(String::from("Local")),
                    message: Some(String::from("Local APIServices are always available")),
                }],
            },
        }
    }
}
