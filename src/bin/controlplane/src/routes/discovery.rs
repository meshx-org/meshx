use axum::{routing::get, Json, Router};
use controlplane_api::Resource;
use controlplane_api::apimachinery::apis::meta::v1::{APIGroup, APIGroupVersion, APIVersions, APIResource, APIResourceList};

use crate::state::AppState;

async fn api_version() -> Json<APIVersions> {
    Json(APIVersions {
        kind: APIVersions::KIND.to_string(),
        api_version: APIVersions::API_VERSION.to_string(),
        versions: vec![String::from("v1")],
        server_address_by_client_cidrs: vec![],
    })
}

async fn list_api_groups() -> Result<Json<APIGroup>, ()> {
    Ok(Json(APIGroup {
        kind: APIGroup::KIND.to_string(),
        api_version: APIGroup::API_VERSION.to_string(),
        name: "core.k8s.io".to_owned(),
        preferred_version: None,
        server_address_by_client_cidrs: None,
        versions: vec![APIGroupVersion {
            group_version: "core.k8s.io/v1".to_owned(),
            version: "v1".to_owned(),
        }],
    }))
}

async fn list_resource_kinds() -> Result<Json<APIResourceList>, ()> {
    let apiservice = APIResource {
        categories: None,
        group: None,
        version: None,
        storage_version_hash: None,
        kind: String::from("APIResource"),
        name: String::from("apiservices"),
        singular_name: String::from("apiservice"),
        namespaced: false,
        verbs: vec![String::from("get"), String::from("list")],
        short_names: None,
    };

    let pod = APIResource {
        categories: None,
        group: None,
        version: None,
        storage_version_hash: None,
        kind: String::from("APIResource"),
        name: String::from("components"),
        singular_name: String::from("component"),
        namespaced: true,
        verbs: vec![String::from("get"), String::from("list")],
        short_names: Some(vec![String::from("po")]),
    };

    Ok(Json(APIResourceList {
        kind: APIResourceList::KIND.to_owned(),
        api_version:APIResourceList::API_VERSION.to_owned(),
        group_version: "core.k8s.io/v1".to_owned(),
        resources: vec![apiservice, pod],
    }))
}

pub fn discovery_routes() -> Router<AppState> {
    Router::new()
        .route("/api", get(api_version))
        .route("/api/:version", get(list_resource_kinds))
        .route("/apis", get(list_api_groups))
        // by group
        .route("/apis/:api_group", get(list_api_groups))
        .route("/apis/:api_group/:version", get(list_resource_kinds))
}
