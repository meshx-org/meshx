#![doc = include_str!("../../../README.md")]

/// The current version of the meshx package, set at build time
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Command line interface implementations for meshx
pub mod cli;
/// Configuration management for meshx
pub mod config;
pub mod types;
