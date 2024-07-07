use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("SerializationError: {0}")]
    SerializationError(#[source] serde_json::Error),

    #[error("Kube Error: {0}")]
    ClientError(#[source] controlplane_client::Error),

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

pub type Result<T, E = Error> = std::result::Result<T, E>;