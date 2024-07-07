pub mod api;
pub mod client;
pub mod config;
pub mod error;

pub use api::Api;
pub use client::Client;
pub use error::Error;

/// Re-exports from kube_core
pub use controlplane_core as core;
pub use crate::core::{CustomResourceExt, Resource, ResourceExt};
