use std::collections::HashMap;
use async_trait::async_trait;

use crate::routing::resolving::ComponentAddress;
use crate::routing::resolving::ResolvedComponent;
use crate::routing::resolving::ResolverError;

/// Resolves a component URL to its content.
#[async_trait]
pub trait Resolver: std::fmt::Debug {
    /// Resolves a component URL to its content. This function takes in the
    /// `component_address` (from an absolute or relative URL), and the `target`
    /// component that is trying to be resolved.
    async fn resolve(&self, component_address: &ComponentAddress) -> Result<ResolvedComponent, ResolverError>;
}

/// Resolves a component URL using a resolver selected based on the URL's scheme.
#[derive(Debug, Default)]
pub struct ResolverRegistry {
    resolvers: HashMap<String, Box<dyn Resolver + Send + Sync + 'static>>,
}
