//! C ABI bindings for Kali.
//!
//! This crate owns the deterministic C-header and metadata helpers used by the
//! public embedding projection.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Current host ABI version expected by generated embedding metadata.
pub const HOST_ABI_VERSION: u32 = 2;

/// Description of an exported entrypoint in the generated C header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    /// Exported symbol name.
    pub name: String,
    /// Number of C arguments emitted for the prototype.
    pub arity: usize,
}

impl Export {
    /// Create a new export description.
    pub fn new(name: impl Into<String>, arity: usize) -> Self {
        Self {
            name: name.into(),
            arity,
        }
    }
}

/// Infer the prototype arity from a Kali export signature string.
pub fn arity_from_signature(signature: &str) -> usize {
    let Some((params, _)) = signature.split_once(") =>") else {
        return 0;
    };
    let params = params.trim_start_matches('(').trim_end();
    if params.is_empty() {
        0
    } else {
        params.split(',').count()
    }
}

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

/// Parse and validate generated C ABI metadata.
fn validate_generated_cabi_metadata(metadata: Value) -> Value {
    parse_metadata(&metadata.to_string())
        .expect("generated cabi metadata must satisfy the schema-v1 contract")
}

fn validate_generated_binding_package_manifest(manifest: Value) -> Value {
    parse_binding_package_manifest(&manifest.to_string())
        .expect("generated binding package manifest must satisfy the schema-v1 contract")
}

pub fn parse_metadata(metadata_text: &str) -> Result<Value, String> {
    let metadata: Value = serde_json::from_str(metadata_text)
        .map_err(|error| format!("cabi metadata is not valid JSON: {}", error))?;

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
        .and_then(Value::as_u64)
        .ok_or_else(|| "cabi metadata field 'hostAbiVersion' must be an integer".to_string())?;

    let min_host_abi_version = metadata
        .get("minHostAbiVersion")
        .cloned()
        .unwrap_or_else(|| Value::from(host_abi_version));
    if min_host_abi_version.as_u64().is_none() {
        return Err("cabi metadata field 'minHostAbiVersion' must be an integer".to_string());
    }

    let artifacts = metadata
        .get("artifacts")
        .and_then(Value::as_object)
        .ok_or_else(|| "cabi metadata field 'artifacts' must be a JSON object".to_string())?;
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
    normalized.insert("hostAbiVersion".to_string(), Value::from(host_abi_version));
    normalized.insert("minHostAbiVersion".to_string(), min_host_abi_version);
    if let Some(runtime_profiles) = runtime_profiles {
        normalized.insert(
            "runtimeProfiles".to_string(),
            normalize_string_list_value(&runtime_profiles, "cabi metadata", "runtimeProfiles")?,
        );
    }
    if let Some(max_specializations) = max_specializations {
        if max_specializations.as_u64().is_none() {
            return Err("cabi metadata field 'maxSpecializations' must be an integer".to_string());
        }
        normalized.insert("maxSpecializations".to_string(), max_specializations);
    }
    if let Some(host_contract) = host_contract {
        if !host_contract.is_string() {
            return Err("cabi metadata field 'hostContract' must be a string".to_string());
        }
        normalized.insert("hostContract".to_string(), host_contract);
    }
    if let Some(runtime_backend) = runtime_backend {
        if !runtime_backend.is_string() {
            return Err("cabi metadata field 'runtimeBackend' must be a string".to_string());
        }
        normalized.insert("runtimeBackend".to_string(), runtime_backend);
    }
    if let Some(profile_data_hash) = profile_data_hash {
        if !profile_data_hash.is_string() {
            return Err("cabi metadata field 'profileDataHash' must be a string".to_string());
        }
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
    let min_host_abi_version = metadata
        .get("minHostAbiVersion")
        .cloned()
        .unwrap_or_else(|| host_abi_version.clone());

    let runtime_profiles = metadata
        .get("runtimeProfiles")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let host_contract = metadata
        .get("hostContract")
        .cloned()
        .unwrap_or_else(|| Value::String("kali-hosted".to_string()));
    let runtime_backend = metadata
        .get("runtimeBackend")
        .cloned()
        .unwrap_or_else(|| Value::String("wasmtime".to_string()));

    let mut summary = serde_json::Map::new();
    summary.insert("schemaVersion".to_string(), schema_version);
    summary.insert("kind".to_string(), kind);
    summary.insert("hostAbiVersion".to_string(), host_abi_version);
    summary.insert("minHostAbiVersion".to_string(), min_host_abi_version);
    summary.insert("runtimeProfiles".to_string(), runtime_profiles);
    summary.insert("hostContract".to_string(), host_contract);
    summary.insert("runtimeBackend".to_string(), runtime_backend);

    if let Some(max_specializations) = metadata.get("maxSpecializations") {
        summary.insert(
            "maxSpecializations".to_string(),
            max_specializations.clone(),
        );
    }
    if let Some(profile_data_hash) = metadata.get("profileDataHash") {
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

    validate_string_field(
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
    for key in ["library", "metadata", "exportsHeader", "glue"] {
        if !artifacts.contains_key(key) {
            return Err(format!(
                "binding package manifest field 'artifacts.{}' is missing",
                key
            ));
        }
    }
    for key in ["library", "metadata", "exportsHeader"] {
        validate_string_field(
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
    }

    if let Some(max_specializations) = manifest.get("maxSpecializations") {
        validate_integer_field(
            max_specializations,
            "binding package manifest",
            "maxSpecializations",
        )?;
    }

    if let Some(host_contract) = manifest.get("hostContract") {
        validate_string_field(host_contract, "binding package manifest", "hostContract")?;
    }

    if let Some(runtime_backend) = manifest.get("runtimeBackend") {
        validate_string_field(
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

fn validate_string_field(value: &Value, context: &str, field_name: &str) -> Result<(), String> {
    if value.is_string() {
        Ok(())
    } else {
        Err(format!(
            "{} field '{}' must be a string",
            context, field_name
        ))
    }
}

fn validate_integer_field(value: &Value, context: &str, field_name: &str) -> Result<(), String> {
    if value.as_i64().is_some() || value.as_u64().is_some() {
        Ok(())
    } else {
        Err(format!(
            "{} field '{}' must be an integer",
            context, field_name
        ))
    }
}

fn normalize_string_list_value(
    value: &Value,
    context: &str,
    field_name: &str,
) -> Result<Value, String> {
    let items = value.as_array().ok_or_else(|| {
        format!(
            "{} field '{}' must be an array of strings",
            context, field_name
        )
    })?;

    let mut normalized = Vec::with_capacity(items.len());
    for item in items {
        let string = item
            .as_str()
            .ok_or_else(|| format!("{} field '{}' entries must be strings", context, field_name))?;
        normalized.push(string.to_string());
    }

    normalized.sort();
    normalized.dedup();

    Ok(Value::Array(
        normalized.into_iter().map(Value::String).collect(),
    ))
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
    validate_string_field(
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
    validate_string_field(
        library,
        "binding package manifest summary",
        "artifacts.library",
    )?;
    let metadata = artifacts.get("metadata").ok_or_else(|| {
        "binding package manifest summary field 'artifacts.metadata' is missing".to_string()
    })?;
    validate_string_field(
        metadata,
        "binding package manifest summary",
        "artifacts.metadata",
    )?;

    let module_name = manifest.get("moduleName").ok_or_else(|| {
        "binding package manifest summary field 'moduleName' is missing".to_string()
    })?;
    validate_string_field(
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
    let min_host_abi_version = manifest
        .get("minHostAbiVersion")
        .map(|value| {
            validate_integer_field(
                value,
                "binding package manifest summary",
                "minHostAbiVersion",
            )
            .map(|_| value.clone())
        })
        .transpose()?
        .unwrap_or_else(|| host_abi_version.clone());
    let runtime_profiles = match manifest.get("runtimeProfiles") {
        Some(runtime_profiles) => {
            normalize_string_list_value(runtime_profiles, "binding package", "runtimeProfiles")?
        }
        None => Value::Array(Vec::new()),
    };
    let host_contract = match manifest.get("hostContract") {
        Some(host_contract) => {
            validate_string_field(
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
            validate_string_field(
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

/// Generate a deterministic C header for the provided export surface.
pub fn generate_header(module_name: &str, exports: &[Export]) -> String {
    let mut header = String::new();
    header.push_str("#ifndef KALI_CAPI_GENERATED_H\n");
    header.push_str("#define KALI_CAPI_GENERATED_H\n\n");
    header.push_str("#include <stdint.h>\n\n");
    header.push_str("#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n");
    header.push_str(&format!("/* Generated by Kali for {}. */\n", module_name));

    for export in exports {
        let name = sanitize_identifier(&export.name);
        header.push_str("extern int32_t ");
        header.push_str(&name);
        header.push('(');
        if export.arity == 0 {
            header.push_str("void");
        } else {
            for index in 0..export.arity {
                if index > 0 {
                    header.push_str(", ");
                }
                header.push_str(&format!("int32_t arg{index}"));
            }
        }
        header.push_str(");\n");
    }

    if exports.is_empty() {
        header.push_str("/* No exported symbols were discovered. */\n");
    }

    header.push_str("\n#ifdef __cplusplus\n}\n#endif\n\n");
    header.push_str("#endif\n");
    header
}

/// Convert a symbol into a stable C identifier.
pub fn sanitize_identifier(name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in name.chars().enumerate() {
        let keep = ch.is_ascii_alphanumeric() || ch == '_';
        if index == 0 && ch.is_ascii_digit() {
            out.push('_');
            out.push(ch);
        } else if keep {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    if out.is_empty() {
        "_".to_string()
    } else {
        out
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
