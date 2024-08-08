//! Error handling and error types
use thiserror::Error;

pub use controlplane_core::error::ErrorResponse;

/// Possible errors from the [`Client`](crate::Client)
#[cfg_attr(docsrs, doc(cfg(any(feature = "config", feature = "client"))))]
#[derive(Error, Debug)]
pub enum Error {
    /// ApiError for when things fail
    ///
    /// This can be parsed into as an error handling fallback.
    /// It's also used in `WatchEvent` from watch calls.
    ///
    /// It's quite common to get a `410 Gone` when the `resourceVersion` is too old.
    #[error("ApiError: {0} ({0:?})")]
    Api(#[source] ErrorResponse),

    /// Hyper error
    #[error("HyperError: {0}")]
    HyperError(#[source] hyper::Error),

    /// Service error
    #[error("ServiceError: {0}")]
    Service(#[source] tower::BoxError),

    /// UTF-8 Error
    #[error("UTF-8 Error: {0}")]
    FromUtf8(#[source] std::string::FromUtf8Error),

    /// Failed to build request
    #[error("Failed to build request: {0}")]
    BuildRequest(#[source] controlplane_core::request::Error),

    /// Returned when failed to find a newline character within max length.
    /// Only returned by `Client::request_events` and this should never happen as
    /// the max is `usize::MAX`.
    #[error("Error finding newline character")]
    LinesCodecMaxLineLengthExceeded,

    /// Http based error
    #[error("HttpError: {0}")]
    HttpError(#[source] http::Error),
    
    /// Http based error
    #[error("LegacyHttpError: {0}")]
    LegacyHttpError(#[source] hyper_util::client::legacy::Error),

    /// Returned on `std::io::Error` when reading event stream.
    #[error("Error reading events stream: {0}")]
    ReadEvents(#[source] std::io::Error),

    /// Common error case when requesting parsing into own structs
    #[error("Error deserializing response: {0}")]
    SerdeError(#[source] serde_json::Error),

    /// Failed to infer config
    #[error("Failed to infer configuration: {0}")]
    InferConfig(#[source] crate::config::InferConfigError),

    /// Missing TLS stacks when TLS is required
    #[error("TLS required but no TLS stack selected")]
    TlsRequired,
   
    /// Missing TLS stacks when TLS is required
    #[error("Unimplemented error")]
    Unimplemented,
}
