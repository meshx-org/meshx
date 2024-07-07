use axum::{routing::get, Json, Router};
use controlplane_api::apimachinery::version::info::Info;

use crate::state::AppState;

#[axum::debug_handler]
async fn version() -> Json<Info> {
    Json(Info {
        major: String::from(""),
        minor: String::from(""),
        git_version: String::from(""),
        git_commit: String::from(""),
        git_tree_state: String::from(""),
        build_date: String::from(""),
        compiler: String::from(""),
        platform: String::from(""),
    })
}

pub fn version_route() -> Router<AppState> {
    Router::new().route("/version", get(version))
}