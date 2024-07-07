use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use controlplane_core::{params::PostParams, StoredObject};
use serde::Serialize;
use serde_json::Value;

use crate::{error::Error, state::AppState, PathParams};

#[derive(Serialize)]
struct ServerRoot {
    paths: Vec<String>,
}

async fn create_namespaced_resource(
    State(app): State<AppState>,
    Path(path): Path<PathParams>,
    Query(params): Query<PostParams>,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse, Error> {
    let service = app
        .services
        .get(&format!("{}/{}", path.api_group, path.version))
        .ok_or(Error::ServiceNotFound)?;

    let res = service.find_resource(path.resource.clone(), true, String::from("create"))?;
    let created = res.create(payload, params).await?;

    Ok(Json(created))
}

async fn create_resource(
    State(app): State<AppState>,
    Path(path): Path<PathParams>,
    Query(params): Query<PostParams>,
    Json(payload): Json<Value>,
) -> Result<impl IntoResponse, Error> {
    let service = app
        .services
        .get(&format!("{}/{}", path.api_group, path.version))
        .ok_or(Error::ServiceNotFound)?;

    let res = service.find_resource(path.resource.clone(), false, String::from("create"))?;
    let created = res.create(payload, params).await?;

    Ok(Json(created))
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/apis/:api_group/:version/:resource", post(create_resource))
        .route(
            "/apis/:api_group/:version/namespaces/:namespace/:resource",
            post(create_namespaced_resource),
        )
}
