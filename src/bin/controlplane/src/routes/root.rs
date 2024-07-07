use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct ServerRoot {
    paths: Vec<String>,
}

async fn handler() -> Result<Json<ServerRoot>, ()> {
    Ok(Json(ServerRoot {
        paths: vec![
            String::from("/api"),
            String::from("/api/v1"),
            String::from("/apis"),
            String::from("/apis/"),
            String::from("/metrics"),
            String::from("/version"),
            String::from("/healthz"),
        ],
    }))
}

pub fn root_route() -> Router<AppState> {
    Router::new().route("/", get(handler))
}
