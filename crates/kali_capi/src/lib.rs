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

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Current host ABI version expected by generated embedding metadata.
pub const HOST_ABI_VERSION: u32 = 2;

/// Generate a deterministic packaging manifest for higher-level language bindings.
#[allow(clippy::too_many_arguments)]
pub fn generate_binding_package_manifest_with_provenance(
    module_name: impl AsRef<str>,
    library_path: impl AsRef<str>,
    metadata_path: impl AsRef<str>,
    exports_header_path: impl AsRef<str>,
    runtime_profiles: &[String],
    max_specializations: usize,
    host_contract: Option<&str>,
    runtime_backend: Option<&str>,
    glue_paths: &[String],
) -> Value {
    let mut runtime_profiles: Vec<_> = runtime_profiles.iter().map(String::as_str).collect();
    runtime_profiles.sort();
    runtime_profiles.dedup();

    let mut glue_paths: Vec<_> = glue_paths.iter().map(String::as_str).collect();
    glue_paths.sort();
    glue_paths.dedup();

    let mut manifest = serde_json::Map::new();
    manifest.insert("schemaVersion".to_string(), Value::from(1));
    manifest.insert("kind".to_string(), Value::from("binding-package"));
    manifest.insert("moduleName".to_string(), Value::from(module_name.as_ref()));
    manifest.insert("hostAbiVersion".to_string(), Value::from(HOST_ABI_VERSION));
    manifest.insert(
        "minHostAbiVersion".to_string(),
        Value::from(HOST_ABI_VERSION),
    );
    manifest.insert(
        "runtimeProfiles".to_string(),
        Value::Array(runtime_profiles.into_iter().map(Value::from).collect()),
    );
    if let Some(host_contract) = host_contract {
        manifest.insert("hostContract".to_string(), Value::from(host_contract));
    }
    if let Some(runtime_backend) = runtime_backend {
        manifest.insert("runtimeBackend".to_string(), Value::from(runtime_backend));
    }
    manifest.insert(
        "maxSpecializations".to_string(),
        Value::from(max_specializations),
    );
    manifest.insert(
        "artifacts".to_string(),
        json!({
            "library": library_path.as_ref(),
            "metadata": metadata_path.as_ref(),
            "exportsHeader": exports_header_path.as_ref(),
            "glue": glue_paths,
        }),
    );

    validate_generated_binding_package_manifest(Value::Object(manifest))
}

/// Generate a deterministic packaging manifest for higher-level language bindings.
pub fn generate_binding_package_manifest(
    module_name: impl AsRef<str>,
    library_path: impl AsRef<str>,
    metadata_path: impl AsRef<str>,
    exports_header_path: impl AsRef<str>,
    runtime_profiles: &[String],
    max_specializations: usize,
    glue_paths: &[String],
) -> Value {
    generate_binding_package_manifest_with_provenance(
        module_name,
        library_path,
        metadata_path,
        exports_header_path,
        runtime_profiles,
        max_specializations,
        Some("kali-hosted"),
        Some("wasmtime"),
        glue_paths,
    )
}

fn validate_generated_binding_package_manifest(manifest: Value) -> Value {
    parse_binding_package_manifest(&manifest.to_string())
        .expect("generated binding package manifest must satisfy the schema-v1 contract")
}

/// Discover the generated binding package manifest inside a bundle root.
pub fn discover_binding_package_manifest_path(
    bundle_root: impl AsRef<Path>,
) -> Result<PathBuf, String> {
    discover_binding_package_manifest_path_with_name(bundle_root, "binding-package.json")
}

/// Discover a specific generated binding package manifest name inside a bundle root.
pub fn discover_binding_package_manifest_path_with_name(
    bundle_root: impl AsRef<Path>,
    manifest_name: impl AsRef<str>,
) -> Result<PathBuf, String> {
    let bundle_root = bundle_root.as_ref();
    let manifest_name = manifest_name.as_ref();
    let explicit_path = bundle_root.join(manifest_name);
    if explicit_path.exists() {
        return Ok(explicit_path);
    }

    if manifest_name != "binding-package.json" {
        return Err(format!(
            "binding package manifest '{}' was not found",
            explicit_path.display()
        ));
    }

    let mut discovered = Vec::new();
    let entries = fs::read_dir(bundle_root).map_err(|error| {
        format!(
            "failed to read binding package manifest directory '{}': {}",
            bundle_root.display(),
            error
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read binding package manifest directory '{}': {}",
                bundle_root.display(),
                error
            )
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.ends_with(".binding-package.json") {
            discovered.push(path);
        }
    }
    discovered.sort();

    match discovered.len() {
        0 => Err(format!(
            "binding package manifest '{}' was not found",
            explicit_path.display()
        )),
        1 => Ok(discovered.remove(0)),
        _ => Err(format!(
            "binding package manifest is ambiguous in '{}'; pass a manifest name explicitly",
            bundle_root.display()
        )),
    }
}

/// Parse and validate a generated binding package manifest.
pub fn parse_binding_package_manifest(manifest_text: &str) -> Result<Value, String> {
    let mut manifest: Value = serde_json::from_str(manifest_text)
        .map_err(|error| format!("binding package manifest is not valid JSON: {}", error))?;
    let manifest_object = manifest
        .as_object()
        .ok_or_else(|| "binding package manifest must be a JSON object".to_string())?;
    reject_unexpected_keys(
        manifest_object,
        &[
            "schemaVersion",
            "kind",
            "moduleName",
            "hostAbiVersion",
            "minHostAbiVersion",
            "maxSpecializations",
            "runtimeProfiles",
            "hostContract",
            "runtimeBackend",
            "artifacts",
        ],
        "binding package manifest",
    )?;

    validate_integer_field(
        manifest.get("schemaVersion").ok_or_else(|| {
            "binding package manifest field 'schemaVersion' must be an integer".to_string()
        })?,
        "binding package manifest",
        "schemaVersion",
    )?;
    let schema_version = manifest
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            "binding package manifest field 'schemaVersion' must be an integer".to_string()
        })?;
    if schema_version != 1 {
        return Err(format!(
            "unsupported binding package manifest schemaVersion {}",
            schema_version
        ));
    }

    validate_string_field(
        manifest
            .get("kind")
            .ok_or_else(|| "binding package manifest field 'kind' must be a string".to_string())?,
        "binding package manifest",
        "kind",
    )?;
    let kind = manifest
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| "binding package manifest field 'kind' must be a string".to_string())?;
    if kind != "binding-package" {
        return Err(format!(
            "unsupported binding package manifest kind '{}'",
            kind
        ));
    }

    validate_non_empty_string_field(
        manifest.get("moduleName").ok_or_else(|| {
            "binding package manifest field 'moduleName' must be a string".to_string()
        })?,
        "binding package manifest",
        "moduleName",
    )?;
    validate_integer_field(
        manifest.get("hostAbiVersion").ok_or_else(|| {
            "binding package manifest field 'hostAbiVersion' must be an integer".to_string()
        })?,
        "binding package manifest",
        "hostAbiVersion",
    )?;

    let artifacts = manifest
        .get("artifacts")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            "binding package manifest field 'artifacts' must be a JSON object".to_string()
        })?;
    reject_unexpected_keys(
        artifacts,
        &["library", "metadata", "exportsHeader", "glue"],
        "binding package manifest field 'artifacts'",
    )?;
    for key in ["library", "metadata", "exportsHeader", "glue"] {
        if !artifacts.contains_key(key) {
            return Err(format!(
                "binding package manifest field 'artifacts.{}' is missing",
                key
            ));
        }
    }
    for key in ["library", "metadata", "exportsHeader"] {
        validate_non_empty_string_field(
            artifacts.get(key).ok_or_else(|| {
                format!(
                    "binding package manifest field 'artifacts.{}' is missing",
                    key
                )
            })?,
            "binding package manifest",
            &format!("artifacts.{key}"),
        )?;
    }
    if let Some(runtime_profiles) = manifest.get("runtimeProfiles") {
        let normalized_runtime_profiles =
            normalize_string_list_value(runtime_profiles, "binding package", "runtimeProfiles")?;
        if let Some(manifest_object) = manifest.as_object_mut() {
            manifest_object.insert("runtimeProfiles".to_string(), normalized_runtime_profiles);
        }
    }

    if let Some(min_host_abi_version) = manifest.get("minHostAbiVersion") {
        validate_integer_field(
            min_host_abi_version,
            "binding package manifest",
            "minHostAbiVersion",
        )?;
        validate_host_abi_version_window(
            manifest.get("hostAbiVersion").expect("validated above"),
            Some(min_host_abi_version),
            "binding package manifest",
        )?;
    }

    if let Some(max_specializations) = manifest.get("maxSpecializations") {
        validate_non_negative_integer_field(
            max_specializations,
            "binding package manifest",
            "maxSpecializations",
        )?;
    }

    if let Some(host_contract) = manifest.get("hostContract") {
        validate_non_empty_string_field(host_contract, "binding package manifest", "hostContract")?;
    }

    if let Some(runtime_backend) = manifest.get("runtimeBackend") {
        validate_non_empty_string_field(
            runtime_backend,
            "binding package manifest",
            "runtimeBackend",
        )?;
    }

    if let Some(artifacts) = manifest.get_mut("artifacts").and_then(Value::as_object_mut) {
        let glue = artifacts
            .get("glue")
            .ok_or_else(|| "binding package field 'artifacts.glue' is missing".to_string())?;
        let normalized_glue =
            normalize_string_list_value(glue, "binding package", "artifacts.glue")?;
        artifacts.insert("glue".to_string(), normalized_glue);
    }

    Ok(manifest)
}

/// Load the generated binding package manifest from disk.
pub fn load_binding_package_manifest(path: impl AsRef<Path>) -> Result<Value, String> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read binding package manifest '{}': {}",
            path.display(),
            error
        )
    })?;
    parse_binding_package_manifest(&raw)
}

/// Load and summarize the generated binding package manifest from disk.
pub fn load_binding_package_manifest_summary(path: impl AsRef<Path>) -> Result<Value, String> {
    let manifest = load_binding_package_manifest(path)?;
    binding_package_manifest_summary(&manifest)
}

/// Discover and load the generated binding package manifest from a bundle root.
pub fn load_binding_package_manifest_from_root(
    bundle_root: impl AsRef<Path>,
) -> Result<Value, String> {
    load_binding_package_manifest_from_root_with_name(bundle_root, "binding-package.json")
}

/// Discover, load, and summarize the generated binding package manifest from a bundle root.
pub fn load_binding_package_manifest_summary_from_root(
    bundle_root: impl AsRef<Path>,
) -> Result<Value, String> {
    load_binding_package_manifest_summary_from_root_with_name(bundle_root, "binding-package.json")
}

/// Discover and load a specific generated binding package manifest name from a bundle root.
pub fn load_binding_package_manifest_from_root_with_name(
    bundle_root: impl AsRef<Path>,
    manifest_name: impl AsRef<str>,
) -> Result<Value, String> {
    let path = discover_binding_package_manifest_path_with_name(bundle_root, manifest_name)?;
    load_binding_package_manifest(path)
}

/// Discover, load, and summarize a specific generated binding package manifest name from a bundle root.
pub fn load_binding_package_manifest_summary_from_root_with_name(
    bundle_root: impl AsRef<Path>,
    manifest_name: impl AsRef<str>,
) -> Result<Value, String> {
    let manifest = load_binding_package_manifest_from_root_with_name(bundle_root, manifest_name)?;
    binding_package_manifest_summary(&manifest)
}

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

/// Project a normalized binding package manifest into a compact summary object.
pub fn binding_package_manifest_summary(manifest: &Value) -> Result<Value, String> {
    let manifest = manifest.as_object().ok_or_else(|| {
        "binding package manifest summary must be built from a JSON object".to_string()
    })?;

    let artifacts = manifest
        .get("artifacts")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            "binding package manifest summary field 'artifacts' must be a JSON object".to_string()
        })?;

    let exports_header = artifacts.get("exportsHeader").ok_or_else(|| {
        "binding package manifest summary field 'artifacts.exportsHeader' is missing".to_string()
    })?;
    validate_non_empty_string_field(
        exports_header,
        "binding package manifest summary",
        "artifacts.exportsHeader",
    )?;
    let glue = artifacts.get("glue").ok_or_else(|| {
        "binding package manifest summary field 'artifacts.glue' is missing".to_string()
    })?;
    let glue = normalize_string_list_value(glue, "binding package", "artifacts.glue")?;
    let library = artifacts.get("library").ok_or_else(|| {
        "binding package manifest summary field 'artifacts.library' is missing".to_string()
    })?;
    validate_non_empty_string_field(
        library,
        "binding package manifest summary",
        "artifacts.library",
    )?;
    let metadata = artifacts.get("metadata").ok_or_else(|| {
        "binding package manifest summary field 'artifacts.metadata' is missing".to_string()
    })?;
    validate_non_empty_string_field(
        metadata,
        "binding package manifest summary",
        "artifacts.metadata",
    )?;

    let module_name = manifest.get("moduleName").ok_or_else(|| {
        "binding package manifest summary field 'moduleName' is missing".to_string()
    })?;
    validate_non_empty_string_field(
        module_name,
        "binding package manifest summary",
        "moduleName",
    )?;
    let host_abi_version = manifest.get("hostAbiVersion").ok_or_else(|| {
        "binding package manifest summary field 'hostAbiVersion' is missing".to_string()
    })?;
    validate_integer_field(
        host_abi_version,
        "binding package manifest summary",
        "hostAbiVersion",
    )?;
    let min_host_abi_version = validate_host_abi_version_window(
        host_abi_version,
        manifest.get("minHostAbiVersion"),
        "binding package manifest summary",
    )?;
    let runtime_profiles = match manifest.get("runtimeProfiles") {
        Some(runtime_profiles) => {
            normalize_string_list_value(runtime_profiles, "binding package", "runtimeProfiles")?
        }
        None => Value::Array(Vec::new()),
    };
    let host_contract = match manifest.get("hostContract") {
        Some(host_contract) => {
            validate_non_empty_string_field(
                host_contract,
                "binding package manifest summary",
                "hostContract",
            )?;
            host_contract.clone()
        }
        None => Value::String("kali-hosted".to_string()),
    };
    let runtime_backend = match manifest.get("runtimeBackend") {
        Some(runtime_backend) => {
            validate_non_empty_string_field(
                runtime_backend,
                "binding package manifest summary",
                "runtimeBackend",
            )?;
            runtime_backend.clone()
        }
        None => Value::String("wasmtime".to_string()),
    };

    let mut summary = serde_json::Map::new();
    summary.insert("moduleName".to_string(), module_name.clone());
    summary.insert("hostAbiVersion".to_string(), host_abi_version.clone());
    summary.insert("minHostAbiVersion".to_string(), min_host_abi_version);
    summary.insert("runtimeProfiles".to_string(), runtime_profiles);
    summary.insert("hostContract".to_string(), host_contract);
    summary.insert("runtimeBackend".to_string(), runtime_backend);

    if let Some(max_specializations) = manifest.get("maxSpecializations") {
        validate_non_negative_integer_field(
            max_specializations,
            "binding package manifest summary",
            "maxSpecializations",
        )?;
        summary.insert(
            "maxSpecializations".to_string(),
            max_specializations.clone(),
        );
    }

    let mut summary_artifacts = serde_json::Map::new();
    summary_artifacts.insert("exportsHeader".to_string(), exports_header.clone());
    summary_artifacts.insert("glue".to_string(), glue);
    summary_artifacts.insert("library".to_string(), library.clone());
    summary_artifacts.insert("metadata".to_string(), metadata.clone());
    summary.insert("artifacts".to_string(), Value::Object(summary_artifacts));

    Ok(Value::Object(summary))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
