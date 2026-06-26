//! C ABI bindings for Kali.
//!
//! This crate owns the deterministic C-header and metadata helpers used by the
//! public embedding projection.

mod validate;
use crate::validate::*;

mod header;
pub use header::*;

mod metadata;
pub use metadata::*;

mod manifest;
pub use manifest::*;

mod bundle;
pub use bundle::*;

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Current host ABI version expected by generated embedding metadata.
pub const HOST_ABI_VERSION: u32 = 2;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
