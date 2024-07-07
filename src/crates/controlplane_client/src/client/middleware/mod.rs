//! Middleware types returned from `ConfigExt` methods.

mod base_uri;
mod extra_headers;

pub use base_uri::{BaseUri, BaseUriLayer};
pub use extra_headers::{ExtraHeaders, ExtraHeadersLayer};
