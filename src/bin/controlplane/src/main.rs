#![allow(dead_code)]

mod apiservice;
mod error;
mod routes;
mod services;
mod state;

use apiservice::APIService;
use axum_streams::*;
use futures::StreamExt;
use http::header;
use serde::Deserialize;
use services::{Service, StoredResource};
use state::AppState;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use controlplane_api::{
    apimachinery::apis::meta::v1::{APIResource, ListMeta},
    core::v1::namespace::Namespace,
};
use controlplane_core::{
    dynamic::DynamicObject,
    metadata::TypeMeta,
    params::{ListParams, PostParams, WatchParams},
    Object, ObjectList, StoredObject,
};

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::{MatchedPath, Path, Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use etcd_client::EventType;
use etcd_client::{Client, ConnectOptions, Error, GetOptions, KvClient, WatchOptions};
use routes::{create_routes, discovery_routes, metrics_route, root_route, version_route};
use serde_json::{json, Value};

async fn bootstrap_apiservices(
    mut kv_client: KvClient,
    services: HashMap<String, Service>,
) -> Result<(), error::Error> {
    for (_, service) in services {
        let service = service.api_service;
        let key = format!(
            "/registry/apiregistration.k8s.io/apiservices/{}.{}",
            service.spec.version, service.spec.group
        );

        let object: Object<Value, Value> = Object {
            types: Some(TypeMeta {
                kind: service.kind,
                api_version: service.api_version,
            }),
            metadata: service.metadata,
            spec: serde_json::to_value(service.spec)?,
            status: None,
        };

        let value = serde_json::to_vec(&object)?;
        kv_client.put(key, value, None).await?;
    }

    Ok(())
}

async fn track_metrics(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [("method", method.to_string()), ("path", path), ("status", status)];

    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_requests_duration_seconds", &labels).record(latency);

    response
}

#[derive(Deserialize)]
struct PathParams {
    api_group: String,
    version: String,
    namespace: Option<String>,
    resource: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "controlplane=trace,tower_http=trace,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let client = Client::connect(
        ["localhost:2379"],
        Some(ConnectOptions::new().with_timeout(Duration::from_secs(1))),
    )
    .await?;

    let n_client = Arc::from(RwLock::new(client.clone()));

    let apiservice = APIResource {
        categories: None,
        group: None,
        version: None,
        storage_version_hash: None,
        kind: String::from("APIResource"),
        name: String::from("apiservices"),
        singular_name: String::from("apiservice"),
        namespaced: false,
        verbs: vec![String::from("list")],
        short_names: None,
    };

    let component = APIResource {
        categories: None,
        storage_version_hash: None,
        namespaced: false,
        group: Some(String::from("core.k8s.io")),
        version: Some(String::from("v1")),
        name: String::from("components"),
        kind: String::from("Component"),
        singular_name: String::from("component"),
        verbs: vec![String::from("create"), String::from("get"), String::from("list")],
        short_names: Some(vec![String::from("po")]),
    };

    let namespace = APIResource {
        categories: None,
        storage_version_hash: None,
        namespaced: false,
        kind: String::from("Namespace"),
        group: Some(String::from("core.k8s.io")),
        version: Some(String::from("v1")),
        name: String::from("namespaces"),
        singular_name: String::from("namespace"),
        verbs: vec![String::from("create"), String::from("get"), String::from("list")],
        short_names: Some(vec![String::from("ns")]),
    };

    let core_service = APIService::new(String::from("core.k8s.io"), String::from("v1"));
    let mut core_service = Service::new(client.clone(), core_service);

    core_service.add_resource(StoredResource::new(n_client.clone(), component));
    core_service.add_resource(StoredResource::new(n_client.clone(), namespace));

    let apiregistration_service = APIService::new(String::from("apiregistration.k8s.io"), String::from("v1"));
    let apiregistration_service = Service::new(client.clone(), apiregistration_service);

    let mut services = HashMap::new();
    services.insert(String::from("core.k8s.io/v1"), core_service);
    services.insert(String::from("apiregistration.k8s.io/v1"), apiregistration_service);

    let _ = bootstrap_apiservices(client.kv_client(), services.clone()).await;

    let state = AppState { client, services };

    let app = Router::new()
        .merge(routes::metrics_route())
        .merge(routes::version_route())
        .merge(routes::root_route())
        .merge(routes::discovery_routes())
        .merge(routes::create_routes())
        .route("/apis/:api_group/:version/:resource", get(list_resources))
        .route(
            "/apis/:api_group/:version/namespaces/:namespace/:resource",
            get(list_or_watch_namespaced_resources),
        )
        .route_layer(middleware::from_fn(track_metrics))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

#[axum::debug_handler]
async fn list_resources(
    State(app): State<AppState>,
    Path(path): Path<PathParams>,
    Query(params): Query<ListParams>,
) -> Result<Response, error::Error> {
    let service = app
        .services
        .get(&format!("{}/{}", path.api_group, path.version))
        .ok_or(error::Error::ServiceNotFound)?;

    let res = service.find_resource(path.resource.clone(), false, "list".to_string())?;
    let objects = res.list::<StoredObject<Value, Value>>(None, params).await?;
    Ok(Json(objects).into_response())
}

async fn list_namespaced_resources(
    app: AppState,
    api_group: String,
    _version: String,
    namespace: String,
    resource: String,
) -> Result<Response, error::Error> {
    let key = format!("/registry/{}/{}/{}/", api_group, resource, namespace);

    let mut resp = app
        .client
        .kv_client()
        .get(key, Some(GetOptions::new().with_prefix()))
        .await?;

    let mut items = vec![];
    for k in resp.take_kvs() {
        k.mod_revision();

        let object = serde_json::from_slice::<Object<Value, Value>>(k.value())?;
        items.push(object);
    }

    return Ok(Json(ObjectList {
        types: TypeMeta {
            kind: "TestList".to_owned(),
            api_version: "v1".to_owned(),
        },
        metadata: ListMeta::default(),
        items,
    })
    .into_response());
}

async fn list_or_watch_namespaced_resources(
    State(app): State<AppState>,
    Path(path): Path<PathParams>,
    Query(params): Query<Value>,
) -> Result<Response, error::Error> {
    let service = app
        .services
        .get(&format!("{}/{}", path.api_group, path.version))
        .ok_or(error::Error::ServiceNotFound)?;

    let res = service.find_resource(path.resource.clone(), true, String::from("list"))?;

    // If ?watch=1 is present we handle that case separately
    if let Some(param) = params.get("watch") {
        if let Some(watch) = param.as_str() {
            if let "1" = watch {
                let watch_params: WatchParams = serde_json::from_value(params)?;
                let stream = res.watch(path.namespace, watch_params).await?;

                let header = [
                    (header::CONTENT_TYPE, "application/stream+json"),
                    (header::CONTENT_ENCODING, "chunked"),
                ];

                return Ok((header, StreamBodyAs::json_nl(stream)).into_response());
            }
        }
    }

    let list_params = serde_json::from_value::<ListParams>(params)?;
    return list_namespaced_resources(
        app,
        path.api_group,
        path.version,
        path.namespace.unwrap(),
        path.resource,
    )
    .await;
}
