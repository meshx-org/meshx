pub mod metrics;
pub use metrics::metrics_route;

pub mod version;
pub use version::version_route;

pub mod root;
pub use root::root_route;

pub mod discovery;
pub use discovery::discovery_routes;

pub mod create;
pub use create::create_routes;
