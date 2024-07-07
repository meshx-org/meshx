pub use controller::{self, telemetry, State};
use controlplane_api::core::v1::namespace::Namespace;
use controlplane_client::{Api, Client};
use controlplane_core::params::PostParams;

use serde_json::json;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init().await;

    // Initiatilize Kubernetes controller state
    let state = State::default();
    controller::run(state.clone()).await;

    state.metrics();

    Ok(())
}
