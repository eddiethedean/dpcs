//! Network registry client and reference server (ROADMAP 0.10).

#[cfg(feature = "registry-client")]
pub mod cache;
#[cfg(feature = "registry-client")]
pub mod client;
#[cfg(feature = "registry-server")]
pub mod server;

#[cfg(feature = "registry-client")]
pub use cache::RegistryCache;
#[cfg(feature = "registry-client")]
pub use client::{PublishRequest, RegistryClient, RegistryClientError};
#[cfg(feature = "registry-server")]
pub use server::{serve, serve_listener, ServeOptions};
