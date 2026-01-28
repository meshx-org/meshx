#![doc = include_str!("../../../README.md")]

/// The current version of the meshx package, set at build time
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Command line interface implementations for meshx
pub mod cli;
/// Configuration management for meshx
pub mod config;
/// Create new meshx projects
pub mod new;
/// Types for meshx
pub(crate) mod types;
/// Manage WebAssembly Interface Types (WIT) for meshx components
pub(crate) mod wit;
/// Development utilities for meshx
pub(crate) mod dev_utils;