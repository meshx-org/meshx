use std::time::Duration;

use thiserror::Error;

/// Failed to infer config
#[derive(Error, Debug)]
#[error("failed to infer config: kubeconfig: ({kubeconfig})")]
pub struct InferConfigError {
    // We can only pick one source, but the kubeconfig failure is more likely to be a user error
    #[source]
    kubeconfig: KubeconfigError,
}

/// Possible errors when loading kubeconfig
#[derive(Error, Debug)]
pub enum KubeconfigError {
    /// Failed to determine current context
    #[error("failed to determine current context")]
    CurrentContextNotSet,
}

/// Configuration object detailing things like cluster URL, default namespace, root certificates, and timeouts.
///
/// # Usage
/// Construct a [`Config`] instance by using one of the many constructors.
///
/// Prefer [`Config::infer`] unless you have particular issues, and avoid manually managing
/// the data in this struct unless you have particular needs. It exists to be consumed by the [`Client`][crate::Client].
///
/// If you are looking to parse the kubeconfig found in a user's home directory see [`Kubeconfig`].
#[cfg_attr(docsrs, doc(cfg(feature = "config")))]
#[derive(Debug, Clone)]
pub struct Config {
    /// The configured cluster url
    pub cluster_url: http::Uri,
    /// The configured default namespace
    pub default_namespace: String,
    /// Set the timeout for connecting to the Kubernetes API.
    ///
    /// A value of `None` means no timeout
    pub connect_timeout: Option<std::time::Duration>,
    /// Set the timeout for the Kubernetes API response.
    ///
    /// A value of `None` means no timeout
    pub read_timeout: Option<std::time::Duration>,
    /// Set the timeout for the Kubernetes API request.
    ///
    /// A value of `None` means no timeout
    pub write_timeout: Option<std::time::Duration>,
}

impl Config {
    /// Construct a new config where only the `cluster_url` is set by the user.
    /// and everything else receives a default value.
    ///
    /// Most likely you want to use [`Config::infer`] to infer the config from
    /// the environment.
    pub fn new(cluster_url: http::Uri) -> Self {
        Self {
            cluster_url,
            default_namespace: String::from("default"),
            //root_cert: None,
            connect_timeout: Some(DEFAULT_CONNECT_TIMEOUT),
            read_timeout: Some(DEFAULT_READ_TIMEOUT),
            write_timeout: Some(DEFAULT_WRITE_TIMEOUT),
            //accept_invalid_certs: false,
            //auth_info: AuthInfo::default(),
            //proxy_url: None,
            //tls_server_name: None,
        }
    }

    /// Infer a Kubernetes client configuration.
    ///
    /// First, a user's kubeconfig is loaded from `KUBECONFIG` or
    /// `~/.kube/config`. If that fails, an in-cluster config is loaded via
    /// [`Config::incluster`]. If inference from both sources fails, then an
    /// error is returned.
    ///
    /// [`Config::apply_debug_overrides`] is used to augment the loaded
    /// configuration based on the environment.
    pub async fn infer() -> Result<Self, InferConfigError> {
        Ok(Self {
            cluster_url: http::Uri::builder()
                .scheme("http")
                .authority("localhost:3000")
                .path_and_query("/")
                .build()
                .unwrap(),
            default_namespace: String::from("default"),
            connect_timeout: Some(DEFAULT_CONNECT_TIMEOUT),
            read_timeout: Some(DEFAULT_READ_TIMEOUT),
            write_timeout: Some(DEFAULT_WRITE_TIMEOUT),
        })
    }
}

// https://github.com/kube-rs/kube/issues/146#issuecomment-590924397
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(295);
const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(295);
