//! Request builder type for arbitrary api types
use thiserror::Error;

use crate::params::{GetParams, ListParams, Patch, PatchParams, PostParams, WatchParams};

pub(crate) const JSON_MIME: &str = "application/json";

/// Extended Accept Header
///
/// Requests a meta.k8s.io/v1 PartialObjectMetadata resource (efficiently
/// retrieves object metadata)
///
/// API Servers running Kubernetes v1.14 and below will retrieve the object and then
/// convert the metadata.
pub(crate) const JSON_METADATA_MIME: &str = "application/json;as=PartialObjectMetadata;g=meta.k8s.io;v=v1";
pub(crate) const JSON_METADATA_LIST_MIME: &str = "application/json;as=PartialObjectMetadataList;g=meta.k8s.io;v=v1";

/// Possible errors when building a request.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to build a request.
    #[error("failed to build request: {0}")]
    BuildRequest(#[source] http::Error),

    /// Failed to serialize body.
    #[error("failed to serialize body: {0}")]
    SerializeBody(#[source] serde_json::Error),

    /// Failed to validate request.
    #[error("failed to validate request: {0}")]
    Validation(String),
}

/// A Kubernetes request builder
///
/// Takes a base_path and supplies constructors for common operations
/// The extra operations all return `http::Request` objects.
#[derive(Debug, Clone)]
pub struct Request {
    /// The path component of a url
    pub url_path: String,
}

impl Request {
    /// New request with a resource's url path
    pub fn new<S: Into<String>>(url_path: S) -> Self {
        Self {
            url_path: url_path.into(),
        }
    }
}

/// Convenience methods found from API conventions
impl Request {
    /// List a collection of a resource
    pub fn list(&self, params: &ListParams) -> Result<http::Request<Vec<u8>>, Error> {
        let target = format!("{}?", self.url_path);
        let mut qp = form_urlencoded::Serializer::new(target);
        params.validate()?;
        params.populate_qp(&mut qp);
        let urlstr = qp.finish();
        let req = http::Request::get(urlstr);
        req.body(vec![]).map_err(Error::BuildRequest)
    }

    /// Watch a resource at a given version
    pub fn watch(&self, wp: &WatchParams, ver: &str) -> Result<http::Request<Vec<u8>>, Error> {
        let target = format!("{}?", self.url_path);
        let mut qp = form_urlencoded::Serializer::new(target);
        wp.validate()?;
        wp.populate_qp(&mut qp);
        qp.append_pair("resourceVersion", ver);
        let urlstr = qp.finish();
        let req = http::Request::get(urlstr);
        req.body(vec![]).map_err(Error::BuildRequest)
    }

    /// Watch metadata of a resource at a given version
    pub fn watch_metadata(&self, wp: &WatchParams, ver: &str) -> Result<http::Request<Vec<u8>>, Error> {
        let target = format!("{}?", self.url_path);
        let mut qp = form_urlencoded::Serializer::new(target);
        wp.validate()?;
        wp.populate_qp(&mut qp);
        qp.append_pair("resourceVersion", ver);

        let urlstr = qp.finish();
        http::Request::get(urlstr)
            .header(http::header::ACCEPT, JSON_METADATA_MIME)
            .header(http::header::CONTENT_TYPE, JSON_MIME)
            .body(vec![])
            .map_err(Error::BuildRequest)
    }

    /// Create an instance of a resource
    pub fn create(&self, pp: &PostParams, data: Vec<u8>) -> Result<http::Request<Vec<u8>>, Error> {
        pp.validate()?;
        let target = format!("{}?", self.url_path);
        let mut qp = form_urlencoded::Serializer::new(target);
        pp.populate_qp(&mut qp);
        let urlstr = qp.finish();
        let req = http::Request::post(urlstr).header(http::header::CONTENT_TYPE, JSON_MIME);
        req.body(data).map_err(Error::BuildRequest)
    }

    /// Get a single instance
    pub fn get(&self, name: &str, gp: &GetParams) -> Result<http::Request<Vec<u8>>, Error> {
        let urlstr = if let Some(rv) = &gp.resource_version {
            let target = format!("{}/{}?", self.url_path, name);
            form_urlencoded::Serializer::new(target)
                .append_pair("resourceVersion", rv)
                .finish()
        } else {
            let target = format!("{}/{}", self.url_path, name);
            form_urlencoded::Serializer::new(target).finish()
        };
        let req = http::Request::get(urlstr);
        req.body(vec![]).map_err(Error::BuildRequest)
    }

    /// Replace an instance of a resource
    ///
    /// Requires `metadata.resourceVersion` set in data
    pub fn replace(&self, name: &str, pp: &PostParams, data: Vec<u8>) -> Result<http::Request<Vec<u8>>, Error> {
        let target = format!("{}/{}?", self.url_path, name);
        let mut qp = form_urlencoded::Serializer::new(target);
        pp.populate_qp(&mut qp);
        let urlstr = qp.finish();
        let req = http::Request::put(urlstr).header(http::header::CONTENT_TYPE, JSON_MIME);
        req.body(data).map_err(Error::BuildRequest)
    }

    /// Patch an instance of a resource
    ///
    /// Requires a serialized merge-patch+json at the moment.
    pub fn patch<P: serde::Serialize>(
        &self,
        name: &str,
        pp: &PatchParams,
        patch: &Patch<P>,
    ) -> Result<http::Request<Vec<u8>>, Error> {
        pp.validate(patch)?;
        let target = format!("{}/{}?", self.url_path, name);
        let mut qp = form_urlencoded::Serializer::new(target);
        pp.populate_qp(&mut qp);
        let urlstr = qp.finish();

        http::Request::patch(urlstr)
            .header(http::header::ACCEPT, JSON_MIME)
            .header(http::header::CONTENT_TYPE, patch.content_type())
            .body(patch.serialize().map_err(Error::SerializeBody)?)
            .map_err(Error::BuildRequest)
    }
}

/// Metadata-only request implementations
///
/// Requests set an extended Accept header compromised of JSON media type and
/// additional parameters that retrieve only necessary metadata from an object.
impl Request {
    /// Get a single metadata instance for a named resource
    pub fn get_metadata(&self, name: &str, gp: &GetParams) -> Result<http::Request<Vec<u8>>, Error> {
        let urlstr = if let Some(rv) = &gp.resource_version {
            let target = format!("{}/{}?", self.url_path, name);
            form_urlencoded::Serializer::new(target)
                .append_pair("resourceVersion", rv)
                .finish()
        } else {
            let target = format!("{}/{}", self.url_path, name);
            form_urlencoded::Serializer::new(target).finish()
        };
        let req = http::Request::get(urlstr)
            .header(http::header::ACCEPT, JSON_METADATA_MIME)
            .header(http::header::CONTENT_TYPE, JSON_MIME);
        req.body(vec![]).map_err(Error::BuildRequest)
    }
}

/// Subresources
impl Request {
    /// Get an instance of the subresource
    pub fn get_subresource(&self, subresource_name: &str, name: &str) -> Result<http::Request<Vec<u8>>, Error> {
        let target = format!("{}/{}/{}", self.url_path, name, subresource_name);
        let mut qp = form_urlencoded::Serializer::new(target);
        let urlstr = qp.finish();
        let req = http::Request::get(urlstr);
        req.body(vec![]).map_err(Error::BuildRequest)
    }

    /// Create an instance of the subresource
    pub fn create_subresource(
        &self,
        subresource_name: &str,
        name: &str,
        pp: &PostParams,
        data: Vec<u8>,
    ) -> Result<http::Request<Vec<u8>>, Error> {
        let target = format!("{}/{}/{}?", self.url_path, name, subresource_name);
        let mut qp = form_urlencoded::Serializer::new(target);
        pp.populate_qp(&mut qp);
        let urlstr = qp.finish();
        let req = http::Request::post(urlstr).header(http::header::CONTENT_TYPE, JSON_MIME);
        req.body(data).map_err(Error::BuildRequest)
    }

    /// Patch an instance of the subresource
    pub fn patch_subresource<P: serde::Serialize>(
        &self,
        subresource_name: &str,
        name: &str,
        pp: &PatchParams,
        patch: &Patch<P>,
    ) -> Result<http::Request<Vec<u8>>, Error> {
        pp.validate(patch)?;
        let target = format!("{}/{}/{}?", self.url_path, name, subresource_name);
        let mut qp = form_urlencoded::Serializer::new(target);
        pp.populate_qp(&mut qp);
        let urlstr = qp.finish();

        http::Request::patch(urlstr)
            .header(http::header::ACCEPT, JSON_MIME)
            .header(http::header::CONTENT_TYPE, patch.content_type())
            .body(patch.serialize().map_err(Error::SerializeBody)?)
            .map_err(Error::BuildRequest)
    }

     /// Replace an instance of the subresource
    pub fn replace_subresource(
        &self,
        subresource_name: &str,
        name: &str,
        pp: &PostParams,
        data: Vec<u8>,
    ) -> Result<http::Request<Vec<u8>>, Error> {
        let target = format!("{}/{}/{}?", self.url_path, name, subresource_name);
        let mut qp = form_urlencoded::Serializer::new(target);
        pp.populate_qp(&mut qp);
        let urlstr = qp.finish();
        let req = http::Request::put(urlstr).header(http::header::CONTENT_TYPE, JSON_MIME);
        req.body(data).map_err(Error::BuildRequest)
    }
}

/// Extensive tests for Request of k8s_openapi::Resource structs
///
/// Cheap sanity check to ensure type maps work as expected
#[cfg(test)]
mod test {
    use super::Request;
    use crate::{
        params::{ListParams, PostParams, VersionMatch, WatchParams},
        resource::Resource,
    };

    use controlplane_api::apiextensions::v1::crd::CustomResourceDefinition;
    use controlplane_api::core::event::Event;

    #[test]
    fn list_path() {
        let url = Event::url_path(&(), Some("ns"));
        let lp = ListParams::default();
        let req = Request::new(url).list(&lp).unwrap();
        assert_eq!(req.uri(), "/api/v1/namespaces/ns/events");
    }

    #[test]
    fn watch_path() {
        let url = Event::url_path(&(), Some("ns"));
        let wp = WatchParams::default();
        let req = Request::new(url).watch(&wp, "0").unwrap();
        assert_eq!(
            req.uri(),
            "/api/v1/namespaces/ns/events?&watch=true&timeoutSeconds=290&allowWatchBookmarks=true&resourceVersion=0"
        );
    }

    #[test]
    fn api_apiextsv1_crd() {
        let url = CustomResourceDefinition::url_path(&(), None);
        let req = Request::new(url).create(&PostParams::default(), vec![]).unwrap();
        assert_eq!(req.uri(), "/apis/apiextensions.k8s.io/v1/customresourcedefinitions?");
    }
}
