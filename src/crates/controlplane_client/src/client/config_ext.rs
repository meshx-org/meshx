use std::sync::Arc;

use crate::{config::Config, Error};

use super::middleware::{BaseUriLayer, ExtraHeadersLayer};

/// Extensions to [`Config`](crate::Config) for custom [`Client`](crate::Client).
///
/// See [`Client::new`](crate::Client::new) for an example.
///
/// This trait is sealed and cannot be implemented.
pub trait ConfigExt: private::Sealed {
    /// Layer to set the base URI of requests to the configured server.
    fn base_uri_layer(&self) -> BaseUriLayer;
    /// Layer to add non-authn HTTP headers depending on the config.
    fn extra_headers_layer(&self) -> Result<ExtraHeadersLayer, Error>;
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::Config {}
}

impl ConfigExt for Config {
    fn base_uri_layer(&self) -> BaseUriLayer {
        BaseUriLayer::new(self.cluster_url.clone())
    }

    fn extra_headers_layer(&self) -> Result<ExtraHeadersLayer, Error> {
        let mut headers = Vec::new();
        /*if let Some(impersonate_user) = &self.auth_info.impersonate {
            headers.push((
                HeaderName::from_static("impersonate-user"),
                HeaderValue::from_str(impersonate_user)
                    .map_err(http::Error::from)
                    .map_err(Error::HttpError)?,
            ));
        }*/
        /*if let Some(impersonate_groups) = &self.auth_info.impersonate_groups {
            for group in impersonate_groups {
                headers.push((
                    HeaderName::from_static("impersonate-group"),
                    HeaderValue::from_str(group)
                        .map_err(http::Error::from)
                        .map_err(Error::HttpError)?,
                ));
            }
        }*/
        Ok(ExtraHeadersLayer {
            headers: Arc::new(headers),
        })
    }
}
