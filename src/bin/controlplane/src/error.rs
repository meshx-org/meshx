//! Error handling and error types
use axum::{response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ServiceNotFound")]
    ServiceNotFound,

    #[error("ResourceNotFound: {0}")]
    ResourceNotFound(String),

    #[error("Error talking to etcd: {0}")]
    EtcdError(#[from] etcd_client::Error),

    /// Common error case when requesting parsing into own structs
    #[error("Error deserializing response: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Namespace is missing")]
    NamespaceMissing,

    #[error("Unknown error happened")]
    Unknown,
}

// We implement `IntoResponse` so ApiError can be used as a response
impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Error::EtcdError(ref error) => (http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Error::SerdeError(ref error) => (http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            Error::ServiceNotFound => (http::StatusCode::NOT_FOUND, self.to_string()),
            Error::ResourceNotFound(ref error) => (http::StatusCode::NOT_FOUND, self.to_string()),
            Error::SerdeError(ref error) => (http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            _ => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let payload = json!({
            "message": message,
        });

        (status, Json(payload)).into_response()
    }
}
