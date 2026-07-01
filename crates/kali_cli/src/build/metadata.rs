//! Artifact metadata construction, build-result validation, metadata serialization.

use super::compile::{
    build_mode_name, profile_data_hash, source_hash_for_file, validate_runtime_profiles, BuildMode,
};
use super::wit::LibraryExport;

use std::borrow::Cow;
use std::path::Path;

use kali_error::{_error_codes::e8, Diagnostic};
use kali_optimize::ProfileData;
use kali_runtime::{RuntimeBackend, RuntimeHostContract};
use serde::Serialize;
use serde_json::Value;
use wasm_encoder::{CustomSection, Section};

use crate::output::validate_sorted_string_array_value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactMetadata {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "artifactKind")]
    pub artifact_kind: String,
    pub entrypoint: String,
    #[serde(rename = "buildMode")]
    pub build_mode: String,
    #[serde(rename = "apiSurface")]
    pub api_surface: String,
    #[serde(rename = "runtimeProfiles")]
    pub runtime_profiles: Vec<String>,
    #[serde(rename = "maxSpecializations")]
    pub max_specializations: usize,
    #[serde(rename = "hostContract", skip_serializing_if = "Option::is_none")]
    pub host_contract: Option<String>,
    #[serde(rename = "runtimeBackend", skip_serializing_if = "Option::is_none")]
    pub runtime_backend: Option<String>,
    #[serde(rename = "kaliVersion")]
    pub kali_version: String,
    #[serde(rename = "sourceHash")]
    pub source_hash: String,
    #[serde(rename = "profileDataHash", skip_serializing_if = "Option::is_none")]
    pub profile_data_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exports: Option<Vec<LibraryExport>>,
}

#[allow(clippy::too_many_arguments)]
pub fn build_artifact_metadata(
    source_path: &Path,
    artifact_kind: &str,
    mode: BuildMode,
    api_surface: &str,
    runtime_profiles: &[String],
    max_specializations: usize,
    profile_data: Option<&ProfileData>,
    exports: Option<Vec<LibraryExport>>,
) -> Result<ArtifactMetadata, Vec<Diagnostic>> {
    let source_hash = source_hash_for_file(source_path).map_err(|error| {
        vec![Diagnostic::error(
            e8::INTERNAL_ERROR as u32,
            format!(
                "failed to hash source file '{}': {}",
                source_path.display(),
                error
            ),
        )]
    })?;

    let runtime_profiles = validate_runtime_profiles(
        runtime_profiles,
        &format!("artifact metadata for '{}'", source_path.display()),
    )?;

    let metadata = ArtifactMetadata {
        schema_version: 1,
        artifact_kind: artifact_kind.to_string(),
        entrypoint: source_path.to_string_lossy().to_string(),
        build_mode: build_mode_name(mode).to_string(),
        api_surface: api_surface.to_string(),
        runtime_profiles,
        max_specializations,
        host_contract: Some(
            RuntimeHostContract::KaliHosted
                .canonical_label()
                .to_string(),
        ),
        runtime_backend: Some(RuntimeBackend::Wasmtime.canonical_label().to_string()),
        kali_version: env!("CARGO_PKG_VERSION").to_string(),
        source_hash,
        profile_data_hash: profile_data_hash(profile_data),
        exports,
    };
    validate_generated_artifact_metadata(&metadata).map_err(|error| {
        vec![Diagnostic::error(
            e8::INTERNAL_ERROR as u32,
            format!(
                "generated artifact metadata for '{}' failed validation: {}",
                source_path.display(),
                error
            ),
        )]
    })?;

    Ok(metadata)
}

pub(crate) fn validate_artifact_metadata_value(value: &Value) -> Result<(), String> {
    const REQUIRED_KEYS: [&str; 7] = [
        "schemaVersion",
        "artifactKind",
        "entrypoint",
        "buildMode",
        "apiSurface",
        "kaliVersion",
        "sourceHash",
    ];
    const VALID_ARTIFACT_KINDS: [&str; 5] = ["executable", "lib", "bundle", "capi", "component"];
    const VALID_BUILD_MODES: [&str; 3] = ["fast", "release", "release-advanced"];

    let Some(object) = value.as_object() else {
        return Err("artifact metadata must be a JSON object".to_string());
    };

    for key in REQUIRED_KEYS {
        if !object.contains_key(key) {
            return Err(format!("artifact metadata is missing required key `{key}`"));
        }
    }
    validate_no_unexpected_keys(
        object,
        "artifact metadata",
        &[
            "schemaVersion",
            "artifactKind",
            "entrypoint",
            "buildMode",
            "apiSurface",
            "runtimeProfiles",
            "maxSpecializations",
            "hostContract",
            "runtimeBackend",
            "kaliVersion",
            "sourceHash",
            "profileDataHash",
            "exports",
        ],
    )?;

    match object.get("schemaVersion") {
        Some(Value::Number(number)) if number.as_u64() == Some(1) => {}
        Some(other) => {
            return Err(format!(
                "artifact metadata schemaVersion must be the numeric value 1, got {other}"
            ));
        }
        None => unreachable!("validated above"),
    }

    validate_canonical_non_empty_string_field(
        object.get("artifactKind"),
        "artifact metadata artifactKind",
    )?;
    match object.get("artifactKind") {
        Some(Value::String(kind)) if VALID_ARTIFACT_KINDS.contains(&kind.as_str()) => {}
        Some(Value::String(kind)) => {
            return Err(format!("unsupported artifact metadata kind '{kind}'"));
        }
        Some(other) => {
            return Err(format!(
                "artifact metadata artifactKind must be a string, got {other}"
            ));
        }
        None => unreachable!("validated above"),
    }

    validate_canonical_non_empty_string_field(
        object.get("entrypoint"),
        "artifact metadata entrypoint",
    )?;

    validate_canonical_non_empty_string_field(
        object.get("buildMode"),
        "artifact metadata buildMode",
    )?;
    match object.get("buildMode") {
        Some(Value::String(mode)) if VALID_BUILD_MODES.contains(&mode.as_str()) => {}
        Some(Value::String(mode)) => {
            return Err(format!("unsupported artifact metadata buildMode '{mode}'"));
        }
        Some(other) => {
            return Err(format!(
                "artifact metadata buildMode must be a string, got {other}"
            ));
        }
        None => unreachable!("validated above"),
    }

    validate_canonical_non_empty_string_field(
        object.get("apiSurface"),
        "artifact metadata apiSurface",
    )?;

    validate_sorted_string_array_value(
        object.get("runtimeProfiles"),
        "artifact metadata runtimeProfiles",
        true,
    )?;

    match object.get("maxSpecializations") {
        Some(Value::Number(number)) if number.as_u64().is_some() => {}
        Some(other) => {
            return Err(format!(
                "artifact metadata maxSpecializations must be a non-negative integer, got {other}"
            ));
        }
        None => {}
    }

    for key in ["kaliVersion", "sourceHash"] {
        validate_canonical_non_empty_string_field(
            object.get(key),
            &format!("artifact metadata {key}"),
        )?;
    }

    for key in ["hostContract", "runtimeBackend", "profileDataHash"] {
        if object.get(key).is_some() {
            validate_canonical_non_empty_string_field(
                object.get(key),
                &format!("artifact metadata {key}"),
            )?;
        }
    }

    match object.get("exports") {
        Some(Value::Array(items)) => {
            let mut seen_names = std::collections::BTreeSet::new();
            for (index, item) in items.iter().enumerate() {
                let Some(export) = item.as_object() else {
                    return Err(format!(
                        "artifact metadata exports[{index}] must be an object, got {item}"
                    ));
                };
                let export_context = format!("artifact metadata exports[{index}]");
                validate_name_signature_object(export, &export_context)?;
                match export.get("name") {
                    Some(Value::String(name)) => {
                        if !seen_names.insert(name.clone()) {
                            return Err(format!(
                                "artifact metadata exports[{index}].name duplicates `{name}`"
                            ));
                        }
                    }
                    Some(other) => {
                        return Err(format!(
                            "artifact metadata exports[{index}].name must be a string, got {other}"
                        ));
                    }
                    None => unreachable!("validated above"),
                }
                match export.get("signature") {
                    Some(Value::String(_)) => {}
                    Some(other) => {
                        return Err(format!(
                            "artifact metadata exports[{index}].signature must be a string, got {other}"
                        ));
                    }
                    None => unreachable!("validated above"),
                }
            }
        }
        Some(other) => {
            return Err(format!(
                "artifact metadata exports must be an array, got {other}"
            ));
        }
        None => {}
    }

    Ok(())
}

fn validate_generated_artifact_metadata(metadata: &ArtifactMetadata) -> Result<(), String> {
    let value = serde_json::to_value(metadata)
        .map_err(|error| format!("artifact metadata could not be serialized: {error}"))?;
    validate_artifact_metadata_value(&value)
}

pub fn validate_build_result_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("build result must be a JSON object".to_string());
    };

    for key in [
        "artifactKind",
        "outputPath",
        "sizeBytes",
        "buildMode",
        "sourceHash",
    ] {
        if !object.contains_key(key) {
            return Err(format!("build result is missing required key `{key}`"));
        }
    }

    validate_canonical_non_empty_string_field(
        object.get("artifactKind"),
        "build result artifactKind",
    )?;
    let artifact_kind = object
        .get("artifactKind")
        .and_then(Value::as_str)
        .expect("validated above");

    validate_canonical_non_empty_string_field(object.get("outputPath"), "build result outputPath")?;

    match object.get("sizeBytes") {
        Some(Value::Number(number)) if number.as_u64().is_some() => {}
        Some(other) => {
            return Err(format!(
                "build result sizeBytes must be a non-negative integer, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    const VALID_BUILD_MODES: [&str; 3] = ["fast", "release", "release-advanced"];

    validate_canonical_non_empty_string_field(object.get("buildMode"), "build result buildMode")?;
    match object.get("buildMode") {
        Some(Value::String(mode)) if VALID_BUILD_MODES.contains(&mode.as_str()) => {}
        Some(Value::String(mode)) => {
            return Err(format!("unsupported build result buildMode '{mode}'"));
        }
        Some(other) => {
            return Err(format!(
                "build result buildMode must be a string, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    validate_canonical_non_empty_string_field(object.get("sourceHash"), "build result sourceHash")?;

    for key in ["hostContract", "runtimeBackend", "profileDataHash"] {
        if object.get(key).is_some() {
            validate_canonical_non_empty_string_field(
                object.get(key),
                &format!("build result {key}"),
            )?;
        }
    }

    let allowed_keys: &[&str] = match artifact_kind {
        "executable" => &[
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "hostContract",
            "runtimeBackend",
            "profileDataHash",
        ],
        "lib" => &[
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "hostContract",
            "runtimeBackend",
            "profileDataHash",
            "metadataPath",
            "witPath",
            "artifacts",
            "exports",
        ],
        "bundle" => &[
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "hostContract",
            "runtimeBackend",
            "profileDataHash",
            "artifacts",
            "exports",
            "bundleFormat",
        ],
        "capi" => &[
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "hostContract",
            "runtimeBackend",
            "profileDataHash",
            "metadataPath",
            "witPath",
            "headerPath",
            "artifacts",
            "exports",
        ],
        "component" => &[
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "hostContract",
            "runtimeBackend",
            "profileDataHash",
            "metadataPath",
            "witPath",
            "bindingPackagePath",
            "artifacts",
            "exports",
        ],
        other => return Err(format!("unsupported build result artifactKind '{other}'")),
    };
    validate_no_unexpected_keys(object, "build result", allowed_keys)?;

    match artifact_kind {
        "executable" => {}
        "lib" => {
            for key in ["metadataPath", "witPath", "artifacts", "exports"] {
                if !object.contains_key(key) {
                    return Err(format!("build result is missing required key `{key}`"));
                }
            }
            validate_canonical_non_empty_string_field(
                object.get("metadataPath"),
                "build result metadataPath",
            )?;
            validate_canonical_non_empty_string_field(
                object.get("witPath"),
                "build result witPath",
            )?;
            validate_build_result_artifacts_array(
                object.get("artifacts"),
                "build result artifacts",
            )?;
            validate_build_result_exports_array(object.get("exports"), "build result exports")?;
        }
        "bundle" => {
            for key in ["artifacts", "exports", "bundleFormat"] {
                if !object.contains_key(key) {
                    return Err(format!("build result is missing required key `{key}`"));
                }
            }
            validate_build_result_artifacts_array(
                object.get("artifacts"),
                "build result artifacts",
            )?;
            validate_build_result_exports_array(object.get("exports"), "build result exports")?;
            validate_canonical_non_empty_string_field(
                object.get("bundleFormat"),
                "build result bundleFormat",
            )?;
            match object.get("bundleFormat") {
                Some(Value::String(format)) if matches!(format.as_str(), "esm" | "cjs") => {}
                Some(Value::String(format)) => {
                    return Err(format!("unsupported build result bundleFormat '{format}'"));
                }
                Some(other) => {
                    return Err(format!(
                        "build result bundleFormat must be a string, got {other}"
                    ));
                }
                None => unreachable!("validated above"),
            }
        }
        "capi" => {
            for key in [
                "metadataPath",
                "witPath",
                "headerPath",
                "artifacts",
                "exports",
            ] {
                if !object.contains_key(key) {
                    return Err(format!("build result is missing required key `{key}`"));
                }
            }
            validate_canonical_non_empty_string_field(
                object.get("metadataPath"),
                "build result metadataPath",
            )?;
            validate_canonical_non_empty_string_field(
                object.get("headerPath"),
                "build result headerPath",
            )?;
            validate_canonical_non_empty_string_field(
                object.get("witPath"),
                "build result witPath",
            )?;
            validate_build_result_artifacts_array(
                object.get("artifacts"),
                "build result artifacts",
            )?;
            validate_build_result_exports_array(object.get("exports"), "build result exports")?;
        }
        "component" => {
            for key in [
                "metadataPath",
                "witPath",
                "bindingPackagePath",
                "artifacts",
                "exports",
            ] {
                if !object.contains_key(key) {
                    return Err(format!("build result is missing required key `{key}`"));
                }
            }
            validate_canonical_non_empty_string_field(
                object.get("metadataPath"),
                "build result metadataPath",
            )?;
            validate_canonical_non_empty_string_field(
                object.get("witPath"),
                "build result witPath",
            )?;
            validate_canonical_non_empty_string_field(
                object.get("bindingPackagePath"),
                "build result bindingPackagePath",
            )?;
            validate_build_result_artifacts_array(
                object.get("artifacts"),
                "build result artifacts",
            )?;
            validate_build_result_exports_array(object.get("exports"), "build result exports")?;
        }
        other => return Err(format!("unsupported build result artifactKind '{other}'")),
    }

    Ok(())
}

fn validate_build_result_artifacts_array(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("{context} must be an array"));
    };

    let mut seen_primary_executable = false;
    let mut seen_primary_library = false;
    let mut seen_primary_component = false;

    let mut seen_kind_path_pairs = std::collections::BTreeSet::new();
    let mut previous_sort_key: Option<(usize, String, String)> = None;

    for (index, item) in items.iter().enumerate() {
        let Some(object) = item.as_object() else {
            return Err(format!("{context}[{index}] must be an object, got {item}"));
        };

        validate_no_unexpected_keys(
            object,
            &format!("{context}[{index}]"),
            &["kind", "path", "role"],
        )?;

        match object.get("kind") {
            Some(Value::String(_)) => {
                validate_canonical_non_empty_string_field(
                    object.get("kind"),
                    &format!("{context}[{index}].kind"),
                )?;
            }
            Some(other) => {
                return Err(format!(
                    "{context}[{index}].kind must be a string, got {other}"
                ))
            }
            None => return Err(format!("{context}[{index}] is missing required key `kind`")),
        }
        validate_canonical_non_empty_string_field(
            object.get("path"),
            &format!("{context}[{index}].path"),
        )?;

        let kind = object
            .get("kind")
            .and_then(Value::as_str)
            .expect("validated above");
        let path = object
            .get("path")
            .and_then(Value::as_str)
            .expect("validated above");
        if !seen_kind_path_pairs.insert((kind.to_string(), path.to_string())) {
            return Err(format!(
                "{context}[{index}] duplicates artifact `{kind}` at `{path}`"
            ));
        }

        if let Some(role) = object.get("role") {
            match role {
                Value::String(role) => {
                    let role_value = Value::String(role.clone());
                    validate_canonical_non_empty_string_field(
                        Some(&role_value),
                        &format!("{context}[{index}].role"),
                    )?;
                    if !is_canonical_artifact_role(role) {
                        return Err(format!(
                            "{context}[{index}].role must be a canonical schema-v1 role, got `{role}`"
                        ));
                    }
                    match role.as_str() {
                        "primary-executable" => {
                            if seen_primary_executable {
                                return Err(format!(
                                    "{context}[{index}].role duplicates primary-executable"
                                ));
                            }
                            seen_primary_executable = true;
                        }
                        "primary-library" => {
                            if seen_primary_library {
                                return Err(format!(
                                    "{context}[{index}].role duplicates primary-library"
                                ));
                            }
                            seen_primary_library = true;
                        }
                        "primary-component" => {
                            if seen_primary_component {
                                return Err(format!(
                                    "{context}[{index}].role duplicates primary-component"
                                ));
                            }
                            seen_primary_component = true;
                        }
                        _ => {}
                    }
                }
                other => {
                    return Err(format!(
                        "{context}[{index}].role must be a string, got {other}"
                    ));
                }
            }
        }

        let sort_key = build_result_artifact_sort_key(object);
        if let Some(previous_sort_key) = &previous_sort_key {
            if previous_sort_key >= &sort_key {
                return Err(format!(
                    "{context}[{index}] must be sorted by role, kind, then path; got role rank {}, kind `{}`, path `{}` after role rank {}, kind `{}`, path `{}`",
                    sort_key.0,
                    sort_key.1,
                    sort_key.2,
                    previous_sort_key.0,
                    previous_sort_key.1,
                    previous_sort_key.2,
                ));
            }
        }
        previous_sort_key = Some(sort_key);
    }

    Ok(())
}

fn validate_name_signature_object(
    object: &serde_json::Map<String, Value>,
    context: &str,
) -> Result<(), String> {
    validate_no_unexpected_keys(object, context, &["name", "signature"])?;

    validate_canonical_non_empty_string_field(object.get("name"), &format!("{context}.name"))?;
    validate_canonical_non_empty_string_field(
        object.get("signature"),
        &format!("{context}.signature"),
    )?;

    Ok(())
}

fn validate_build_result_exports_array(value: Option<&Value>, context: &str) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("{context} must be an array"));
    };

    let mut seen_names = std::collections::BTreeSet::new();

    for (index, item) in items.iter().enumerate() {
        let Some(object) = item.as_object() else {
            return Err(format!("{context}[{index}] must be an object, got {item}"));
        };
        let export_context = format!("{context}[{index}]");
        validate_name_signature_object(object, &export_context)?;
        match object.get("name") {
            Some(Value::String(name)) => {
                if !seen_names.insert(name.clone()) {
                    return Err(format!("{context}[{index}].name duplicates `{name}`"));
                }
            }
            Some(other) => {
                return Err(format!(
                    "{context}[{index}].name must be a string, got {other}"
                ))
            }
            None => return Err(format!("{context}[{index}] is missing required key `name`")),
        }
        match object.get("signature") {
            Some(Value::String(_)) => {}
            Some(other) => {
                return Err(format!(
                    "{context}[{index}].signature must be a string, got {other}"
                ))
            }
            None => {
                return Err(format!(
                    "{context}[{index}] is missing required key `signature`"
                ))
            }
        }
    }

    Ok(())
}

fn build_result_artifact_sort_key(
    object: &serde_json::Map<String, Value>,
) -> (usize, String, String) {
    let role_rank = object
        .get("role")
        .and_then(Value::as_str)
        .map(build_result_artifact_role_rank)
        .unwrap_or(usize::MAX);
    let kind = object
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let path = object
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    (role_rank, kind, path)
}

fn build_result_artifact_role_rank(role: &str) -> usize {
    match role {
        "primary-executable" => 0,
        "primary-library" => 1,
        "primary-component" => 2,
        "browser-glue" => 3,
        "interface-wit" => 4,
        "embedding-header" => 5,
        "embedding-metadata" => 6,
        "binding-package-manifest" => 7,
        "debug-source-map" => 8,
        _ => usize::MAX,
    }
}

fn validate_no_unexpected_keys(
    object: &serde_json::Map<String, Value>,
    context: &str,
    allowed_keys: &[&str],
) -> Result<(), String> {
    for key in object.keys() {
        if !allowed_keys.contains(&key.as_str()) {
            return Err(format!("{context} has unexpected key `{key}`"));
        }
    }

    Ok(())
}

fn validate_canonical_non_empty_string_field(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    match value {
        Some(Value::String(value)) => {
            if value.trim().is_empty() {
                Err(format!(
                    "{context} must be a non-empty, non-whitespace string"
                ))
            } else if value.trim() != value {
                Err(format!(
                    "{context} must not have leading or trailing whitespace"
                ))
            } else {
                Ok(())
            }
        }
        Some(other) => Err(format!("{context} must be a string, got {other}")),
        None => Err(format!("{context} is missing required key")),
    }
}

fn is_canonical_artifact_role(role: &str) -> bool {
    matches!(
        role,
        "primary-executable"
            | "primary-library"
            | "primary-component"
            | "browser-glue"
            | "interface-wit"
            | "embedding-header"
            | "embedding-metadata"
            | "binding-package-manifest"
            | "debug-source-map"
    )
}

pub fn serialize_artifact_metadata(metadata: &ArtifactMetadata) -> Vec<u8> {
    validate_generated_artifact_metadata(metadata)
        .expect("serialized artifact metadata must satisfy schema-v1 shape");
    serde_json::to_vec(metadata).expect("serialize artifact metadata")
}

pub fn append_metadata_section(
    wasm_bytes: &mut Vec<u8>,
    metadata: &ArtifactMetadata,
) -> Result<(), Vec<Diagnostic>> {
    let metadata_bytes = serialize_artifact_metadata(metadata);
    CustomSection {
        name: Cow::Borrowed("kali:metadata"),
        data: Cow::Owned(metadata_bytes),
    }
    .append_to(wasm_bytes);
    Ok(())
}
