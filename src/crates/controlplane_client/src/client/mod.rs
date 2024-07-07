mod body;
mod builder;
mod config_ext;
mod middleware;

use futures::{AsyncBufRead, StreamExt, TryStream, TryStreamExt};
use http::{self, Request, Response};
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use tower::{buffer::Buffer, util::BoxService, BoxError, Layer, Service, ServiceExt};
use tower_http::map_response_body::MapResponseBodyLayer;
use tokio_util::{
    codec::{FramedRead, LinesCodec, LinesCodecError},
    io::StreamReader,
};

use crate::config::Config;

pub use self::body::Body;
use super::error::Error;
use controlplane_core::{error::ErrorResponse, watch::WatchEvent};

/// Client for connecting with a Kubernetes cluster.
///
/// The easiest way to instantiate the client is either by
/// inferring the configuration from the environment using
/// [`Client::try_default`] or with an existing [`Config`]
/// using [`Client::try_from`].
#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
#[derive(Clone)]
pub struct Client {
    // - `Buffer` for cheap clone
    // - `BoxService` for dynamic response future type
    inner: Buffer<BoxService<Request<Body>, Response<Body>, BoxError>, Request<Body>>,
    default_ns: String,
}

/// Constructors and low-level api interfaces.
///
/// Most users only need [`Client::try_default`] or [`Client::new`] from this block.
///
/// The many various lower level interfaces here are for more advanced use-cases with specific requirements.
impl Client {
    /// Create a [`Client`] using a custom `Service` stack.
    ///
    /// [`ConfigExt`](crate::client::ConfigExt) provides extensions for
    /// building a custom stack.
    ///
    /// To create with the default stack with a [`Config`], use
    /// [`Client::try_from`].
    ///
    /// To create with the default stack with an inferred [`Config`], use
    /// [`Client::try_default`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # async fn doc() -> Result<(), Box<dyn std::error::Error>> {
    /// use kube::{client::ConfigExt, Client, Config};
    /// use tower::ServiceBuilder;
    /// use hyper_util::rt::TokioExecutor;
    ///
    /// let config = Config::infer().await?;
    /// let service = ServiceBuilder::new()
    ///     .layer(config.base_uri_layer())
    ///     .option_layer(config.auth_layer()?)
    ///     .service(hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build_http());
    /// let client = Client::new(service, config.default_namespace);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<S, B, T>(service: S, default_namespace: T) -> Self
    where
        S: Service<Request<Body>, Response = Response<B>> + Send + 'static,
        S::Future: Send + 'static,
        S::Error: Into<BoxError>,
        B: http_body::Body<Data = bytes::Bytes> + Send + 'static,
        B::Error: Into<BoxError>,
        T: Into<String>,
    {
        // Transform response body to `crate::client::Body` and use type erased error to avoid type parameters.
        let service = MapResponseBodyLayer::new(Body::wrap_body)
            .layer(service)
            .map_err(|e| e.into());

        Self {
            inner: Buffer::new(BoxService::new(service), 1024),
            default_ns: default_namespace.into(),
        }
    }

     /// Create and initialize a [`Client`] using the inferred configuration.
    ///
    /// Will use [`Config::infer`] which attempts to load the local kubeconfig first,
    /// and then if that fails, trying the in-cluster environment variables.
    ///
    /// Will fail if neither configuration could be loaded.
    ///
    /// ```rust
    /// # async fn doc() -> Result<(), Box<dyn std::error::Error>> {
    /// # use kube::Client;
    /// let client = Client::try_default().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// If you already have a [`Config`] then use [`Client::try_from`](Self::try_from)
    /// instead.
    pub async fn try_default() -> Result<Self, Error> {
        Self::try_from(Config::infer().await.map_err(Error::InferConfig)?)
    }

    /// Get the default namespace for the client
    ///
    /// The namespace is either configured on `context` in the kubeconfig,
    /// falls back to `default` when running locally,
    /// or uses the service account's namespace when deployed in-cluster.
    pub fn default_namespace(&self) -> &str {
        &self.default_ns
    }

    /// Perform a raw HTTP request against the API and return the raw response back.
    /// This method can be used to get raw access to the API which may be used to, for example,
    /// create a proxy server or application-level gateway between localhost and the API server.
    pub async fn send(&self, request: Request<Body>) -> Result<Response<Body>, Error> {
        let mut svc = self.inner.clone();
        let res = svc
            .ready()
            .await
            .map_err(Error::Service)?
            .call(request)
            .await
            .map_err(|err| {
                // Error decorating request
                err.downcast::<Error>()
                    .map(|e| *e)
                    // Error requesting
                    .or_else(|err| {
                        err.downcast::<hyper::Error>()
                            .map(|err| super::error::Error::HyperError(*err))
                    })
                    // Error from another middleware
                    .unwrap_or_else(super::error::Error::Service)
            })?;
        Ok(res)
    }

    /// Perform a raw HTTP request against the API and deserialize the response
    /// as JSON to some known type.
    pub async fn request<T>(&self, request: Request<Vec<u8>>) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let text = self.request_text(request).await?;

        serde_json::from_str(&text).map_err(|e| {
            tracing::warn!("{}, {:?}", text, e);
            Error::SerdeError(e)
        })
    }

    /// Perform a raw HTTP request against the API and get back the response
    /// as a string
    pub async fn request_text(&self, request: Request<Vec<u8>>) -> Result<String, Error> {
        let res = self.send(request.map(Body::from)).await?;
        let res = handle_api_errors(res).await?;
        let body_bytes = res.into_body().collect().await?.to_bytes();
        let text = String::from_utf8(body_bytes.to_vec()).map_err(Error::FromUtf8)?;
        Ok(text)
    }

    /// Perform a raw request and get back a stream of [`WatchEvent`] objects
    pub async fn request_events<T>(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<impl TryStream<Item = Result<WatchEvent<T>, Error>>, Error>
    where
        T: Clone + DeserializeOwned,
    {
        let res = self.send(request.map(Body::from)).await?;
        // trace!("Streaming from {} -> {}", res.url(), res.status().as_str());
        tracing::trace!("headers: {:?}", res.headers());

        let frames = FramedRead::new(
            StreamReader::new(res.into_body().into_data_stream().map_err(|e| {
                // Unexpected EOF from chunked decoder.
                // Tends to happen when watching for 300+s. This will be ignored.
                if e.to_string().contains("unexpected EOF during chunk") {
                    return std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e);
                }
                std::io::Error::other(e)
            })),
            LinesCodec::new(),
        );

        Ok(frames.filter_map(|res| async {
            match res {
                Ok(line) => match serde_json::from_str::<WatchEvent<T>>(&line) {
                    Ok(event) => Some(Ok(event)),
                    Err(e) => {
                        // Ignore EOF error that can happen for incomplete line from `decode_eof`.
                        if e.is_eof() {
                            return None;
                        }

                        // Got general error response
                        if let Ok(e_resp) = serde_json::from_str::<ErrorResponse>(&line) {
                            return Some(Err(Error::Api(e_resp)));
                        }
                        // Parsing error
                        Some(Err(Error::SerdeError(e)))
                    }
                },

                Err(LinesCodecError::Io(e)) => match e.kind() {
                    // Client timeout
                    std::io::ErrorKind::TimedOut => {
                        tracing::warn!("timeout in poll: {}", e); // our client timeout
                        None
                    }
                    // Unexpected EOF from chunked decoder.
                    // Tends to happen after 300+s of watching.
                    std::io::ErrorKind::UnexpectedEof => {
                        tracing::warn!("eof in poll: {}", e);
                        None
                    }
                    _ => Some(Err(Error::ReadEvents(e))),
                },

                // Reached the maximum line length without finding a newline.
                // This should never happen because we're using the default `usize::MAX`.
                Err(LinesCodecError::MaxLineLengthExceeded) => Some(Err(Error::LinesCodecMaxLineLengthExceeded)),
            }
        }))
    }
}

/// Low level discovery methods using `k8s_openapi` types.
///
/// Consider using the [`discovery`](crate::discovery) module for
/// easier-to-use variants of this functionality.
/// The following methods might be deprecated to avoid confusion between similarly named types within `discovery`.
impl Client {
    // Returns apiserver version.
    //pub async fn apiserver_version(&self) -> Result<k8s_openapi::apimachinery::pkg::version::Info> {
    //    self.request(
    //        Request::builder()
    //            .uri("/version")
    //            .body(vec![])
    //            .map_err(Error::HttpError)?,
    //    )
    //    .await
    //}
}

/// Kubernetes returned error handling
///
/// Either kube returned an explicit ApiError struct,
/// or it someohow returned something we couldn't parse as one.
///
/// In either case, present an ApiError upstream.
/// The latter is probably a bug if encountered.
async fn handle_api_errors(res: Response<Body>) -> Result<Response<Body>, Error> {
    let status = res.status();
    if status.is_client_error() || status.is_server_error() {
        // trace!("Status = {:?} for {}", status, res.url());
        let body_bytes = res.into_body().collect().await?.to_bytes();
        let text = String::from_utf8(body_bytes.to_vec()).map_err(Error::FromUtf8)?;
        // Print better debug when things do fail
        // trace!("Parsing error: {}", text);
        if let Ok(errdata) = serde_json::from_str::<ErrorResponse>(&text) {
            tracing::debug!("Unsuccessful: {errdata:?}");
            Err(Error::Api(errdata))
        } else {
            tracing::warn!("Unsuccessful data error parse: {}", text);
            let error_response = ErrorResponse {
                status: status.to_string(),
                code: status.as_u16(),
                message: format!("{text:?}"),
                reason: "Failed to parse error data".into(),
            };
            tracing::debug!("Unsuccessful: {error_response:?} (reconstruct)");
            Err(Error::Api(error_response))
        }
    } else {
        Ok(res)
    }
}

impl TryFrom<Config> for Client {
    type Error = Error;

    /// Builds a default [`Client`] from a [`Config`].
    ///
    /// See [`ClientBuilder`] or [`Client::new`] if more customization is required
    fn try_from(config: Config) -> Result<Self, Error> {
        Ok(builder::ClientBuilder::try_from(config)?.build())
    }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;

    use crate::client::api::Api;
    use crate::client::client::{Body, Client};

    use http::{Request, Response};
    use tower_test::mock;

    #[tokio::test]
    async fn test_default_ns() {
        let (mock_service, _) = mock::pair::<Request<Body>, Response<Body>>();
        let client = Client::new(mock_service, "test-namespace");
        assert_eq!(client.default_namespace(), "test-namespace");
    }

    /* #[tokio::test]
    async fn test_mock() {
        let (mock_service, handle) = mock::pair::<Request<Body>, Response<Body>>();
        let spawned = tokio::spawn(async move {
            // Receive a request for pod and respond with some data
            let mut handle = pin!(handle);
            let (request, send) = handle.next_request().await.expect("service not called");
            assert_eq!(request.method(), http::Method::GET);
            assert_eq!(request.uri().to_string(), "/api/v1/namespaces/default/events/test");

            let pod: Pod = serde_json::from_value(serde_json::json!({
                "apiVersion": "v1",
                "kind": "Pod",
                "metadata": {
                    "name": "test",
                    "annotations": { "kube-rs": "test" },
                },
                "spec": {
                    "containers": [{ "name": "test", "image": "test-image" }],
                }
            }))
            .unwrap();

            send.send_response(
                Response::builder()
                    .body(Body::from(serde_json::to_vec(&pod).unwrap()))
                    .unwrap(),
            );
        });

        let pods: Api<Pod> = Api::default_namespaced(Client::new(mock_service, "default"));
        let pod = pods.get("test").await.unwrap();
        assert_eq!(pod.metadata.annotations.unwrap().get("kube-rs").unwrap(), "test");
        spawned.await.unwrap();
    }*/
}
