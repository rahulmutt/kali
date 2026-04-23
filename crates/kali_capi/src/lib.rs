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
    json!({
        "schemaVersion": 1,
        "kind": "cabi-metadata",
        "hostAbiVersion": HOST_ABI_VERSION,
        "minHostAbiVersion": HOST_ABI_VERSION,
        "artifacts": {
            "wasmModule": wasm_module_path.as_ref(),
            "wit": wit_path.as_ref(),
            "exportsHeader": exports_header_path.as_ref(),
        },
    })
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
    let mut runtime_profiles: Vec<_> = runtime_profiles.iter().map(String::as_str).collect();
    runtime_profiles.sort();
    runtime_profiles.dedup();

    let mut glue_paths: Vec<_> = glue_paths.iter().map(String::as_str).collect();
    glue_paths.sort();
    glue_paths.dedup();

    json!({
        "schemaVersion": 1,
        "kind": "binding-package",
        "moduleName": module_name.as_ref(),
        "hostAbiVersion": HOST_ABI_VERSION,
        "minHostAbiVersion": HOST_ABI_VERSION,
        "runtimeProfiles": runtime_profiles,
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "maxSpecializations": max_specializations,
        "artifacts": {
            "library": library_path.as_ref(),
            "metadata": metadata_path.as_ref(),
            "exportsHeader": exports_header_path.as_ref(),
            "glue": glue_paths,
        },
    })
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

    if let Some(runtime_profiles) = manifest.get("runtimeProfiles") {
        let normalized_runtime_profiles =
            normalize_string_list_value(runtime_profiles, "binding package", "runtimeProfiles")?;
        if let Some(manifest_object) = manifest.as_object_mut() {
            manifest_object.insert("runtimeProfiles".to_string(), normalized_runtime_profiles);
        }
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

/// Discover and load the generated binding package manifest from a bundle root.
pub fn load_binding_package_manifest_from_root(
    bundle_root: impl AsRef<Path>,
) -> Result<Value, String> {
    load_binding_package_manifest_from_root_with_name(bundle_root, "binding-package.json")
}

/// Discover and load a specific generated binding package manifest name from a bundle root.
pub fn load_binding_package_manifest_from_root_with_name(
    bundle_root: impl AsRef<Path>,
    manifest_name: impl AsRef<str>,
) -> Result<Value, String> {
    let path = discover_binding_package_manifest_path_with_name(bundle_root, manifest_name)?;
    load_binding_package_manifest(path)
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
