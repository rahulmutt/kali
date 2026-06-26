//! cabi-metadata sidecar: generation, parsing, summarizing, loading, discovery.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::validate::{
    normalize_string_list_value, reject_unexpected_keys, validate_host_abi_version_window,
    validate_non_empty_string_field, validate_non_negative_integer_field,
};
use crate::HOST_ABI_VERSION;

/// Generate the canonical C ABI metadata payload.
pub fn generate_metadata(
    wasm_module_path: impl AsRef<str>,
    wit_path: impl AsRef<str>,
    exports_header_path: impl AsRef<str>,
) -> Value {
    validate_generated_cabi_metadata(json!({
        "schemaVersion": 1,
        "kind": "cabi-metadata",
        "hostAbiVersion": HOST_ABI_VERSION,
        "minHostAbiVersion": HOST_ABI_VERSION,
        "artifacts": {
            "wasmModule": wasm_module_path.as_ref(),
            "wit": wit_path.as_ref(),
            "exportsHeader": exports_header_path.as_ref(),
        },
    }))
}

/// Generate the canonical C ABI metadata payload with build provenance.
pub fn generate_metadata_with_provenance(
    wasm_module_path: impl AsRef<str>,
    wit_path: impl AsRef<str>,
    exports_header_path: impl AsRef<str>,
    runtime_profiles: &[String],
    max_specializations: usize,
    host_contract: Option<&str>,
    runtime_backend: Option<&str>,
) -> Value {
    let mut runtime_profiles: Vec<_> = runtime_profiles.iter().map(String::as_str).collect();
    runtime_profiles.sort();
    runtime_profiles.dedup();

    let mut metadata = serde_json::Map::new();
    metadata.insert("schemaVersion".to_string(), Value::from(1));
    metadata.insert("kind".to_string(), Value::from("cabi-metadata"));
    metadata.insert("hostAbiVersion".to_string(), Value::from(HOST_ABI_VERSION));
    metadata.insert(
        "minHostAbiVersion".to_string(),
        Value::from(HOST_ABI_VERSION),
    );
    metadata.insert(
        "runtimeProfiles".to_string(),
        Value::Array(runtime_profiles.into_iter().map(Value::from).collect()),
    );
    metadata.insert(
        "maxSpecializations".to_string(),
        Value::from(max_specializations),
    );
    if let Some(host_contract) = host_contract {
        metadata.insert("hostContract".to_string(), Value::from(host_contract));
    }
    if let Some(runtime_backend) = runtime_backend {
        metadata.insert("runtimeBackend".to_string(), Value::from(runtime_backend));
    }

    metadata.insert(
        "artifacts".to_string(),
        json!({
            "wasmModule": wasm_module_path.as_ref(),
            "wit": wit_path.as_ref(),
            "exportsHeader": exports_header_path.as_ref(),
        }),
    );

    validate_generated_cabi_metadata(Value::Object(metadata))
}

/// Parse and validate generated C ABI metadata.
fn validate_generated_cabi_metadata(metadata: Value) -> Value {
    parse_metadata(&metadata.to_string())
        .expect("generated cabi metadata must satisfy the schema-v1 contract")
}

pub fn parse_metadata(metadata_text: &str) -> Result<Value, String> {
    let metadata: Value = serde_json::from_str(metadata_text)
        .map_err(|error| format!("cabi metadata is not valid JSON: {}", error))?;
    let metadata_object = metadata
        .as_object()
        .ok_or_else(|| "cabi metadata must be a JSON object".to_string())?;
    reject_unexpected_keys(
        metadata_object,
        &[
            "schemaVersion",
            "kind",
            "hostAbiVersion",
            "minHostAbiVersion",
            "runtimeProfiles",
            "maxSpecializations",
            "hostContract",
            "runtimeBackend",
            "profileDataHash",
            "artifacts",
        ],
        "cabi metadata",
    )?;

    let schema_version = metadata
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| "cabi metadata field 'schemaVersion' must be an integer".to_string())?;
    if schema_version != 1 {
        return Err(format!(
            "unsupported cabi metadata schemaVersion {}",
            schema_version
        ));
    }

    let kind = metadata
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| "cabi metadata field 'kind' must be a string".to_string())?;
    if kind != "cabi-metadata" {
        return Err(format!("unsupported cabi metadata kind '{}'", kind));
    }

    let host_abi_version = metadata
        .get("hostAbiVersion")
        .cloned()
        .ok_or_else(|| "cabi metadata field 'hostAbiVersion' must be an integer".to_string())?;

    let min_host_abi_version = validate_host_abi_version_window(
        &host_abi_version,
        metadata.get("minHostAbiVersion"),
        "cabi metadata",
    )?;

    let artifacts = metadata
        .get("artifacts")
        .and_then(Value::as_object)
        .ok_or_else(|| "cabi metadata field 'artifacts' must be a JSON object".to_string())?;
    reject_unexpected_keys(
        artifacts,
        &["wasmModule", "wit", "exportsHeader"],
        "cabi metadata field 'artifacts'",
    )?;
    let mut normalized_artifacts = serde_json::Map::new();
    for key in ["wasmModule", "wit", "exportsHeader"] {
        let value = artifacts
            .get(key)
            .cloned()
            .ok_or_else(|| format!("cabi metadata field 'artifacts.{}' is missing", key))?;
        if !value.is_string() {
            return Err(format!(
                "cabi metadata field 'artifacts.{}' must be a string",
                key
            ));
        }
        normalized_artifacts.insert(key.to_string(), value);
    }

    let runtime_profiles = metadata.get("runtimeProfiles").cloned();
    let max_specializations = metadata.get("maxSpecializations").cloned();
    let host_contract = metadata.get("hostContract").cloned();
    let runtime_backend = metadata.get("runtimeBackend").cloned();
    let profile_data_hash = metadata.get("profileDataHash").cloned();

    let mut normalized = serde_json::Map::new();
    normalized.insert("schemaVersion".to_string(), Value::from(1));
    normalized.insert("kind".to_string(), Value::from("cabi-metadata"));
    normalized.insert("hostAbiVersion".to_string(), host_abi_version);
    normalized.insert("minHostAbiVersion".to_string(), min_host_abi_version);
    if let Some(runtime_profiles) = runtime_profiles {
        normalized.insert(
            "runtimeProfiles".to_string(),
            normalize_string_list_value(&runtime_profiles, "cabi metadata", "runtimeProfiles")?,
        );
    }
    if let Some(max_specializations) = max_specializations {
        validate_non_negative_integer_field(
            &max_specializations,
            "cabi metadata",
            "maxSpecializations",
        )?;
        normalized.insert("maxSpecializations".to_string(), max_specializations);
    }
    if let Some(host_contract) = host_contract {
        validate_non_empty_string_field(&host_contract, "cabi metadata", "hostContract")?;
        normalized.insert("hostContract".to_string(), host_contract);
    }
    if let Some(runtime_backend) = runtime_backend {
        validate_non_empty_string_field(&runtime_backend, "cabi metadata", "runtimeBackend")?;
        normalized.insert("runtimeBackend".to_string(), runtime_backend);
    }
    if let Some(profile_data_hash) = profile_data_hash {
        validate_non_empty_string_field(&profile_data_hash, "cabi metadata", "profileDataHash")?;
        normalized.insert("profileDataHash".to_string(), profile_data_hash);
    }
    normalized.insert("artifacts".to_string(), Value::Object(normalized_artifacts));

    Ok(Value::Object(normalized))
}

/// Project generated C ABI metadata into a compact summary object.
pub fn cabi_metadata_summary(metadata: &Value) -> Result<Value, String> {
    let metadata = metadata
        .as_object()
        .ok_or_else(|| "cabi metadata summary must be built from a JSON object".to_string())?;

    let artifacts = metadata
        .get("artifacts")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            "cabi metadata summary field 'artifacts' must be a JSON object".to_string()
        })?;

    let exports_header = artifacts.get("exportsHeader").cloned().ok_or_else(|| {
        "cabi metadata summary field 'artifacts.exportsHeader' is missing".to_string()
    })?;
    let wasm_module = artifacts.get("wasmModule").cloned().ok_or_else(|| {
        "cabi metadata summary field 'artifacts.wasmModule' is missing".to_string()
    })?;
    let wit = artifacts
        .get("wit")
        .cloned()
        .ok_or_else(|| "cabi metadata summary field 'artifacts.wit' is missing".to_string())?;

    let schema_version = metadata
        .get("schemaVersion")
        .cloned()
        .ok_or_else(|| "cabi metadata summary field 'schemaVersion' is missing".to_string())?;
    let kind = metadata
        .get("kind")
        .cloned()
        .ok_or_else(|| "cabi metadata summary field 'kind' is missing".to_string())?;
    let host_abi_version = metadata
        .get("hostAbiVersion")
        .cloned()
        .ok_or_else(|| "cabi metadata summary field 'hostAbiVersion' is missing".to_string())?;
    let min_host_abi_version = validate_host_abi_version_window(
        &host_abi_version,
        metadata.get("minHostAbiVersion"),
        "cabi metadata summary",
    )?;

    let runtime_profiles = metadata
        .get("runtimeProfiles")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let host_contract = match metadata.get("hostContract") {
        Some(host_contract) => {
            validate_non_empty_string_field(
                host_contract,
                "cabi metadata summary",
                "hostContract",
            )?;
            host_contract.clone()
        }
        None => Value::String("kali-hosted".to_string()),
    };
    let runtime_backend = match metadata.get("runtimeBackend") {
        Some(runtime_backend) => {
            validate_non_empty_string_field(
                runtime_backend,
                "cabi metadata summary",
                "runtimeBackend",
            )?;
            runtime_backend.clone()
        }
        None => Value::String("wasmtime".to_string()),
    };

    let mut summary = serde_json::Map::new();
    summary.insert("schemaVersion".to_string(), schema_version);
    summary.insert("kind".to_string(), kind);
    summary.insert("hostAbiVersion".to_string(), host_abi_version);
    summary.insert("minHostAbiVersion".to_string(), min_host_abi_version);
    summary.insert("runtimeProfiles".to_string(), runtime_profiles);
    summary.insert("hostContract".to_string(), host_contract);
    summary.insert("runtimeBackend".to_string(), runtime_backend);

    if let Some(max_specializations) = metadata.get("maxSpecializations") {
        validate_non_negative_integer_field(
            max_specializations,
            "cabi metadata summary",
            "maxSpecializations",
        )?;
        summary.insert(
            "maxSpecializations".to_string(),
            max_specializations.clone(),
        );
    }
    if let Some(profile_data_hash) = metadata.get("profileDataHash") {
        validate_non_empty_string_field(
            profile_data_hash,
            "cabi metadata summary",
            "profileDataHash",
        )?;
        summary.insert("profileDataHash".to_string(), profile_data_hash.clone());
    }

    let mut summary_artifacts = serde_json::Map::new();
    summary_artifacts.insert("wasmModule".to_string(), wasm_module);
    summary_artifacts.insert("wit".to_string(), wit);
    summary_artifacts.insert("exportsHeader".to_string(), exports_header);
    summary.insert("artifacts".to_string(), Value::Object(summary_artifacts));

    Ok(Value::Object(summary))
}

/// Load generated C ABI metadata from disk.
pub fn load_metadata(path: impl AsRef<Path>) -> Result<Value, String> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read cabi metadata '{}': {}",
            path.display(),
            error
        )
    })?;
    parse_metadata(&raw)
}

/// Load and summarize generated C ABI metadata from disk.
pub fn load_metadata_summary(path: impl AsRef<Path>) -> Result<Value, String> {
    let metadata = load_metadata(path)?;
    cabi_metadata_summary(&metadata)
}

/// Discover the generated C ABI metadata sidecar inside a bundle root.
pub fn discover_metadata_path(bundle_root: impl AsRef<Path>) -> Result<PathBuf, String> {
    discover_metadata_path_with_name(bundle_root, "cabi-metadata.json")
}

/// Discover a specific generated C ABI metadata sidecar name inside a bundle root.
pub fn discover_metadata_path_with_name(
    bundle_root: impl AsRef<Path>,
    metadata_name: impl AsRef<str>,
) -> Result<PathBuf, String> {
    let bundle_root = bundle_root.as_ref();
    let metadata_name = metadata_name.as_ref();
    let explicit_path = bundle_root.join(metadata_name);
    if explicit_path.exists() {
        return Ok(explicit_path);
    }

    if metadata_name != "cabi-metadata.json" {
        return Err(format!(
            "cabi metadata '{}' was not found",
            explicit_path.display()
        ));
    }

    let mut discovered = Vec::new();
    let entries = fs::read_dir(bundle_root).map_err(|error| {
        format!(
            "failed to read cabi metadata directory '{}': {}",
            bundle_root.display(),
            error
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read cabi metadata directory '{}': {}",
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
        if name.ends_with(".capi.meta.json") {
            discovered.push(path);
        }
    }
    discovered.sort();

    match discovered.len() {
        0 => Err(format!(
            "cabi metadata '{}' was not found",
            explicit_path.display()
        )),
        1 => Ok(discovered.remove(0)),
        _ => Err(format!(
            "cabi metadata is ambiguous in '{}'; pass a metadata name explicitly",
            bundle_root.display()
        )),
    }
}

/// Load generated C ABI metadata from a discovered bundle root.
pub fn load_metadata_from_root(bundle_root: impl AsRef<Path>) -> Result<Value, String> {
    load_metadata_from_root_with_name(bundle_root, "cabi-metadata.json")
}

/// Load and summarize generated C ABI metadata from a discovered bundle root.
pub fn load_metadata_summary_from_root(bundle_root: impl AsRef<Path>) -> Result<Value, String> {
    load_metadata_summary_from_root_with_name(bundle_root, "cabi-metadata.json")
}

/// Discover and load a specific generated C ABI metadata sidecar name from a bundle root.
pub fn load_metadata_from_root_with_name(
    bundle_root: impl AsRef<Path>,
    metadata_name: impl AsRef<str>,
) -> Result<Value, String> {
    let path = discover_metadata_path_with_name(bundle_root, metadata_name)?;
    load_metadata(path)
}

/// Discover, load, and summarize a specific generated C ABI metadata sidecar name from a bundle root.
pub fn load_metadata_summary_from_root_with_name(
    bundle_root: impl AsRef<Path>,
    metadata_name: impl AsRef<str>,
) -> Result<Value, String> {
    let metadata = load_metadata_from_root_with_name(bundle_root, metadata_name)?;
    cabi_metadata_summary(&metadata)
}

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod metadata_tests;
