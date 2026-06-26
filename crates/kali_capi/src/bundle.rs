//! Binding-package "bundle" = manifest + metadata combined summary.
//! Composes the public surface of the `manifest` and `metadata` families.

use serde_json::Value;
use std::path::Path;

use crate::manifest::{
    binding_package_manifest_summary, discover_binding_package_manifest_path_with_name,
    load_binding_package_manifest,
};
use crate::metadata::{cabi_metadata_summary, load_metadata};

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
