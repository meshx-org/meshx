use thiserror::Error;

/// Expose all controller components used by main
pub mod controller;
pub use crate::controller::*;

/// Log and trace integrations
pub mod telemetry;

/// Metrics
mod metrics;
pub use metrics::Metrics;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client Error: {0}")]
    ClientError(#[source] controlplane_client::Error),

    #[error("SerializationError: {0}")]
    SerializationError(#[source] serde_json::Error),

    #[error("Finalizer Error: {0}")]
    // NB: awkward type because finalizer::Error embeds the reconciler error (which is this)
    // so boxing this error to break cycles
    FinalizerError(#[source] Box<controlplane_runtime::finalizer::Error<Error>>),

    #[error("IllegalDocument")]
    IllegalDocument,
}

impl Error {
    pub fn metric_label(&self) -> String {
        format!("{self:?}").to_lowercase()
    }
}