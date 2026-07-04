//! C ABI bindings for Kali.
//!
//! This crate owns the deterministic C-header and metadata helpers used by the
//! public embedding projection.

mod validate;

mod header;
pub use header::*;

mod metadata;
pub use metadata::*;

mod manifest;
pub use manifest::*;

mod bundle;
pub use bundle::*;

/// Current host ABI version expected by generated embedding metadata.
pub const HOST_ABI_VERSION: u32 = 2;

#[cfg(test)]
mod test_support;

#[cfg(test)]
#[path = "binding_tests.rs"]
mod binding_tests;
