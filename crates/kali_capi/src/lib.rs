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

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Current host ABI version expected by generated embedding metadata.
pub const HOST_ABI_VERSION: u32 = 2;

/// Project a normalized binding package manifest and its generated C ABI metadata into a compact bundle summary.
pub fn binding_package_bundle_summary(manifest: &Value, metadata: &Value) -> Result<Value, String> {
    let manifest = binding_package_manifest_summary(manifest)?;
    let metadata = cabi_metadata_summary(metadata)?;

    let mut bundle_summary = serde_json::Map::new();
    bundle_summary.insert("manifest".to_string(), manifest);
    bundle_summary.insert("metadata".to_string(), metadata);

    Ok(Value::Object(bundle_summary))
}

/// Load and summarize the generated binding package bundle from disk.
pub fn load_binding_package_bundle_summary(path: impl AsRef<Path>) -> Result<Value, String> {
    let path = path.as_ref();
    let manifest = load_binding_package_manifest(path)?;
    let manifest_summary = binding_package_manifest_summary(&manifest)?;
    let metadata_path = manifest_summary
        .get("artifacts")
        .and_then(Value::as_object)
        .and_then(|artifacts| artifacts.get("metadata"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            "binding package bundle summary field 'artifacts.metadata' is missing".to_string()
        })?;
    let metadata_path = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(metadata_path);
    let metadata = load_metadata(&metadata_path)?;
    binding_package_bundle_summary(&manifest, &metadata)
}

/// Discover, load, and summarize the generated binding package bundle from a bundle root.
pub fn load_binding_package_bundle_summary_from_root(
    bundle_root: impl AsRef<Path>,
) -> Result<Value, String> {
    load_binding_package_bundle_summary_from_root_with_name(bundle_root, "binding-package.json")
}

/// Discover, load, and summarize a specific generated binding package bundle name from a bundle root.
pub fn load_binding_package_bundle_summary_from_root_with_name(
    bundle_root: impl AsRef<Path>,
    manifest_name: impl AsRef<str>,
) -> Result<Value, String> {
    let manifest_path =
        discover_binding_package_manifest_path_with_name(bundle_root, manifest_name)?;
    load_binding_package_bundle_summary(manifest_path)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
