//! build command: artifact + browser-bundle generation.

use kali_capi::{
    arity_from_signature, generate_binding_package_manifest_with_provenance, generate_header,
    generate_metadata_with_provenance as generate_capi_metadata, parse_binding_package_manifest,
    parse_metadata, Export as CApiExport,
};
use kali_cli::{build, output::CliOutputOptions, BundleFormat};
use kali_error::{_error_codes::e5, Diagnostic};
use kali_optimize::ProfileData;
use kali_runtime::{RuntimeBackend, RuntimeHostContract};
use kali_sandbox::SandboxPolicy;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component as PathComponent, Path, PathBuf},
};
use wasm_encoder::{Component, ComponentSectionId, CustomSection, RawSection, Section};

use super::shared;
use super::config;
use super::cmd_package::validate_source_effects_against_policy;

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_command(
    files: Vec<String>,
    sandbox: Option<PathBuf>,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    profile: Option<PathBuf>,
    validate_ir: bool,
    wasm_threads: bool,
    fast: bool,
    release: bool,
    release_advanced: bool,
    max_specializations: Option<usize>,
    bundle: bool,
    format: Option<BundleFormat>,
    lib: bool,
    capi: bool,
    component: bool,
    out_dir: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    shared::ensure_project_ready_or_exit(output)?;
    let effective_compat = match config::resolve_effective_compat_features(compat) {
        Ok(features) => features,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) =
        config::reject_unavailable_compat_features("build", &effective_compat, output, None, None)
    {
        return Err(exit_code);
    }
    let compat_eval = effective_compat.iter().any(|feature| feature == "eval");
    let profile_data = match config::resolve_profile_data(profile) {
        Ok(profile_data) => profile_data,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None)
        }
    };

    let effective_api = match config::resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None)
        }
    };
    if format.is_some() && !bundle {
        let diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            "`--format` is only meaningful when `--bundle` is selected",
        );
        return shared::emit_diagnostics_and_exit("build", vec![diagnostic], 5, output, None, None);
    }

    if bundle {
        if !matches!(effective_api, kali_cli::ApiSurface::Browser) {
            let diagnostic = Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                "`kali build --bundle` requires the effective browser API surface",
            );
            return shared::emit_diagnostics_and_exit("build", vec![diagnostic], 5, output, None, None);
        }
    } else if matches!(effective_api, kali_cli::ApiSurface::Browser) {
        let diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            "`kali build` without `--bundle` is not valid for the browser API surface",
        );
        return shared::emit_diagnostics_and_exit("build", vec![diagnostic], 5, output, None, None);
    }

    let effective_runtime_profiles = match config::resolve_effective_runtime_profiles(wasm_threads) {
        Ok(profiles) => profiles,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) = config::reject_unavailable_runtime_profiles(
        "build",
        &effective_runtime_profiles,
        !matches!(effective_api, kali_cli::ApiSurface::Browser),
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let policy = shared::load_policy_or_exit(sandbox, &effective_runtime_profiles, output)?;

    let Some(source) = shared::single_or_error(files, "build", output)? else {
        return Err(1);
    };

    let source = source.to_string_lossy().to_string();
    let mode = build::build_mode_from_flags(fast, release, release_advanced);
    let max_specializations = match config::resolve_effective_max_specializations(max_specializations) {
        Ok(max_specializations) => max_specializations,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None)
        }
    };
    let out_dir_path = out_dir.as_deref();
    let bundle_format = format.unwrap_or(BundleFormat::Esm);
    let artifact_mode = if lib {
        BuildArtifactSelection::Library
    } else if capi {
        BuildArtifactSelection::Capi
    } else if component {
        BuildArtifactSelection::Component
    } else if bundle {
        BuildArtifactSelection::BrowserBundle
    } else {
        BuildArtifactSelection::Executable
    };

    let build_result = match artifact_mode {
        BuildArtifactSelection::Executable => build_executable_artifact(
            &source,
            mode,
            max_specializations,
            out_dir_path,
            policy.as_ref(),
            effective_api,
            compat_eval,
            validate_ir,
            profile_data.as_ref(),
            &effective_runtime_profiles,
        ),
        BuildArtifactSelection::Library => build_library_artifact(
            &source,
            mode,
            max_specializations,
            out_dir_path,
            policy.as_ref(),
            effective_api,
            compat_eval,
            validate_ir,
            profile_data.as_ref(),
            &effective_runtime_profiles,
        ),
        BuildArtifactSelection::Capi => build_capi_artifact(
            &source,
            mode,
            max_specializations,
            out_dir_path,
            policy.as_ref(),
            effective_api,
            compat_eval,
            validate_ir,
            profile_data.as_ref(),
            &effective_runtime_profiles,
        ),
        BuildArtifactSelection::Component => build_component_artifact(
            &source,
            mode,
            max_specializations,
            out_dir_path,
            policy.as_ref(),
            effective_api,
            compat_eval,
            validate_ir,
            profile_data.as_ref(),
            &effective_runtime_profiles,
        ),
        BuildArtifactSelection::BrowserBundle => build_browser_bundle_artifact(
            &source,
            mode,
            max_specializations,
            out_dir_path,
            policy.as_ref(),
            effective_api,
            compat_eval,
            validate_ir,
            profile_data.as_ref(),
            &effective_runtime_profiles,
            bundle_format,
        ),
    };

    match build_result {
        Ok(build_result) => {
            if output.is_json() {
                let payload = build_result.artifact_json();
                shared::print_envelope(
                    "build",
                    true,
                    vec![],
                    vec![],
                    payload,
                    None,
                    None,
                    0,
                    output,
                );
            } else if !output.quiet {
                println!("{}", build_result.human_message());
            }
            Ok(())
        }
        Err(diagnostics) => shared::emit_diagnostics_and_exit(
            "build",
            diagnostics,
            1,
            output,
            Some(Path::new(&source)),
            fs::read_to_string(&source).ok().as_deref(),
        ),
    }
}

enum BuildArtifactSelection {
    Executable,
    BrowserBundle,
    Library,
    Capi,
    Component,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BundleArtifact {
    kind: String,
    path: PathBuf,
}

struct BrowserBundleBuild {
    output_dir: PathBuf,
    wasm_path: PathBuf,
    js_path: PathBuf,
    source_map_path: PathBuf,
    meta_path: PathBuf,
    wasm_bytes: Vec<u8>,
    metadata: build::ArtifactMetadata,
    format: BundleFormat,
    extra_artifacts: Vec<BundleArtifact>,
}

enum BuildResult {
    Executable {
        output_path: PathBuf,
        wasm_bytes: Vec<u8>,
        metadata: build::ArtifactMetadata,
    },
    Library {
        output_path: PathBuf,
        wit_path: PathBuf,
        meta_path: PathBuf,
        wasm_bytes: Vec<u8>,
        metadata: build::ArtifactMetadata,
    },
    Capi {
        output_path: PathBuf,
        wit_path: PathBuf,
        header_path: PathBuf,
        meta_path: PathBuf,
        wasm_bytes: Vec<u8>,
        metadata: build::ArtifactMetadata,
    },
    Component {
        output_path: PathBuf,
        wit_path: PathBuf,
        meta_path: PathBuf,
        binding_package_path: PathBuf,
        wasm_bytes: Vec<u8>,
        metadata: build::ArtifactMetadata,
    },
    BrowserBundle {
        output_dir: PathBuf,
        wasm_path: PathBuf,
        js_path: PathBuf,
        source_map_path: PathBuf,
        meta_path: PathBuf,
        wasm_bytes: Vec<u8>,
        metadata: build::ArtifactMetadata,
        format: BundleFormat,
        extra_artifacts: Vec<BundleArtifact>,
    },
}

fn build_result_artifact_sort_key(value: &Value) -> (usize, String, String) {
    let object = value
        .as_object()
        .expect("build result artifact entries must be JSON objects");
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

impl BuildResult {
    fn inject_metadata_fields(mut value: Value, metadata: &build::ArtifactMetadata) -> Value {
        if let Some(object) = value.as_object_mut() {
            if let Some(host_contract) = &metadata.host_contract {
                object.insert("hostContract".to_string(), json!(host_contract));
            }
            if let Some(runtime_backend) = &metadata.runtime_backend {
                object.insert("runtimeBackend".to_string(), json!(runtime_backend));
            }
            if let Some(profile_data_hash) = &metadata.profile_data_hash {
                object.insert("profileDataHash".to_string(), json!(profile_data_hash));
            }
        }

        value
    }

    fn sort_build_result_artifacts(artifacts: &mut [Value]) {
        artifacts.sort_by(|left, right| {
            build_result_artifact_sort_key(left).cmp(&build_result_artifact_sort_key(right))
        });
    }

    fn artifact_json(&self) -> Value {
        let value = match self {
            BuildResult::Executable {
                output_path,
                wasm_bytes,
                metadata,
            } => Self::inject_metadata_fields(
                json!({
                    "artifactKind": "executable",
                    "outputPath": output_path,
                    "sizeBytes": wasm_bytes.len(),
                    "buildMode": metadata.build_mode.clone(),
                    "sourceHash": metadata.source_hash.clone(),
                }),
                metadata,
            ),
            BuildResult::Library {
                output_path,
                wit_path,
                meta_path,
                wasm_bytes,
                metadata,
            } => {
                let mut artifacts = vec![
                    json!({ "kind": "wasm-module", "path": output_path }),
                    json!({ "kind": "wit", "path": wit_path }),
                    json!({ "kind": "meta-json", "path": meta_path }),
                ];
                Self::sort_build_result_artifacts(&mut artifacts);
                Self::inject_metadata_fields(
                    json!({
                        "artifactKind": "lib",
                        "outputPath": output_path,
                        "sizeBytes": wasm_bytes.len(),
                        "buildMode": metadata.build_mode.clone(),
                        "sourceHash": metadata.source_hash.clone(),
                        "metadataPath": meta_path,
                        "witPath": wit_path,
                        "artifacts": artifacts,
                        "exports": metadata.exports.clone().unwrap_or_default(),
                    }),
                    metadata,
                )
            }
            BuildResult::Capi {
                output_path,
                wit_path,
                header_path,
                meta_path,
                wasm_bytes,
                metadata,
            } => {
                let mut artifacts = vec![
                    json!({ "kind": "wasm-module", "path": output_path }),
                    json!({ "kind": "wit", "path": wit_path }),
                    json!({ "kind": "c-header", "path": header_path }),
                    json!({ "kind": "cabi-metadata", "path": meta_path }),
                ];
                Self::sort_build_result_artifacts(&mut artifacts);
                Self::inject_metadata_fields(
                    json!({
                        "artifactKind": "capi",
                        "outputPath": output_path,
                        "sizeBytes": wasm_bytes.len(),
                        "buildMode": metadata.build_mode.clone(),
                        "sourceHash": metadata.source_hash.clone(),
                        "metadataPath": meta_path,
                        "witPath": wit_path,
                        "headerPath": header_path,
                        "artifacts": artifacts,
                        "exports": metadata.exports.clone().unwrap_or_default(),
                    }),
                    metadata,
                )
            }
            BuildResult::Component {
                output_path,
                wit_path,
                meta_path,
                binding_package_path,
                wasm_bytes,
                metadata,
            } => {
                let mut artifacts = vec![
                    json!({ "kind": "wasm-component", "path": output_path, "role": "primary-component" }),
                    json!({ "kind": "wit", "path": wit_path, "role": "interface-wit" }),
                    json!({ "kind": "meta-json", "path": meta_path }),
                    json!({ "kind": "binding-package", "path": binding_package_path, "role": "binding-package-manifest" }),
                ];
                Self::sort_build_result_artifacts(&mut artifacts);
                Self::inject_metadata_fields(
                    json!({
                        "artifactKind": "component",
                        "outputPath": output_path,
                        "sizeBytes": wasm_bytes.len(),
                        "buildMode": metadata.build_mode.clone(),
                        "sourceHash": metadata.source_hash.clone(),
                        "metadataPath": meta_path,
                        "witPath": wit_path,
                        "bindingPackagePath": binding_package_path,
                        "artifacts": artifacts,
                        "exports": metadata.exports.clone().unwrap_or_default(),
                    }),
                    metadata,
                )
            }
            BuildResult::BrowserBundle {
                output_dir,
                wasm_path,
                js_path,
                source_map_path,
                meta_path,
                wasm_bytes,
                metadata,
                format,
                extra_artifacts,
            } => {
                let mut artifacts = vec![
                    json!({ "kind": "wasm-module", "path": wasm_path }),
                    json!({ "kind": "js-glue", "path": js_path }),
                    json!({ "kind": "source-map", "path": source_map_path }),
                    json!({ "kind": "meta-json", "path": meta_path }),
                ];
                artifacts.extend(extra_artifacts.iter().map(
                    |artifact| json!({ "kind": artifact.kind.clone(), "path": artifact.path }),
                ));
                Self::sort_build_result_artifacts(&mut artifacts);
                Self::inject_metadata_fields(
                    json!({
                        "artifactKind": "bundle",
                        "outputPath": output_dir,
                        "sizeBytes": wasm_bytes.len(),
                        "buildMode": metadata.build_mode.clone(),
                        "sourceHash": metadata.source_hash.clone(),
                        "artifacts": artifacts,
                        "exports": metadata.exports.clone().unwrap_or_default(),
                        "bundleFormat": format.to_string(),
                    }),
                    metadata,
                )
            }
        };

        build::validate_build_result_value(&value)
            .expect("constructed build result must satisfy schema-v1 shape");
        value
    }

    fn human_message(&self) -> String {
        match self {
            BuildResult::Executable { output_path, .. } => {
                format!("Built executable artifact at {}", output_path.display())
            }
            BuildResult::Library { output_path, .. } => {
                format!("Built library artifact at {}", output_path.display())
            }
            BuildResult::Capi { output_path, .. } => {
                format!("Built C ABI artifact at {}", output_path.display())
            }
            BuildResult::Component { output_path, .. } => {
                format!("Built component artifact at {}", output_path.display())
            }
            BuildResult::BrowserBundle {
                output_dir, format, ..
            } => {
                format!(
                    "Built browser bundle ({}) at {}",
                    format,
                    output_dir.display()
                )
            }
        }
    }
}

fn build_executable_artifact(
    file: &str,
    mode: build::BuildMode,
    max_specializations: usize,
    out_dir: Option<&Path>,
    policy: Option<&SandboxPolicy>,
    api_surface: kali_cli::ApiSurface,
    compat_eval: bool,
    validate_ir: bool,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let mut wasm_bytes =
        build::compile_source_file_with_specialization_cap_and_profile_data_and_validation(
            &source,
            mode,
            max_specializations,
            api_surface,
            profile_data,
            runtime_profiles,
            compat_eval,
            policy.is_some(),
            validate_ir,
            false,
        )?;
    let metadata = build::build_artifact_metadata(
        &source,
        "executable",
        mode,
        &api_surface.to_string(),
        runtime_profiles,
        max_specializations,
        profile_data,
        None,
    )?;
    build::append_metadata_section(&mut wasm_bytes, &metadata)?;

    if let Some(policy) = policy {
        validate_source_effects_against_policy(&source, policy, api_surface)?;
        let policy_bytes = policy
            .to_embedded_json_bytes()
            .map_err(|diagnostic| vec![diagnostic])?;
        CustomSection {
            name: std::borrow::Cow::Borrowed("kali:policy"),
            data: std::borrow::Cow::Owned(policy_bytes),
        }
        .append_to(&mut wasm_bytes);
    }

    let output_path = build::executable_output_path_for(&source, out_dir);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![Diagnostic::error(
                e5::OUTPUT_ERROR as u32,
                format!(
                    "failed to create output directory '{}': {}",
                    parent.display(),
                    error
                ),
            )]
        })?;
    }

    fs::write(&output_path, &wasm_bytes).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write WASM artifact '{}': {}",
                output_path.display(),
                error
            ),
        )]
    })?;

    Ok(BuildResult::Executable {
        output_path,
        wasm_bytes,
        metadata,
    })
}

fn build_library_artifact(
    file: &str,
    mode: build::BuildMode,
    max_specializations: usize,
    out_dir: Option<&Path>,
    policy: Option<&SandboxPolicy>,
    api_surface: kali_cli::ApiSurface,
    compat_eval: bool,
    validate_ir: bool,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let mut wasm_bytes =
        build::compile_source_file_with_specialization_cap_and_profile_data_and_validation(
            &source,
            mode,
            max_specializations,
            api_surface,
            profile_data,
            runtime_profiles,
            compat_eval,
            policy.is_some(),
            validate_ir,
            false,
        )?;
    let exports = build::collect_library_exports(&source, api_surface, runtime_profiles)?;
    let wit = build::library_wit_for(&source.display().to_string(), &exports);
    let metadata = build::build_artifact_metadata(
        &source,
        "lib",
        mode,
        &api_surface.to_string(),
        runtime_profiles,
        max_specializations,
        profile_data,
        Some(exports.clone()),
    )?;
    build::append_metadata_section(&mut wasm_bytes, &metadata)?;

    if let Some(policy) = policy {
        validate_source_effects_against_policy(&source, policy, api_surface)?;
        let policy_bytes = policy
            .to_embedded_json_bytes()
            .map_err(|diagnostic| vec![diagnostic])?;
        CustomSection {
            name: std::borrow::Cow::Borrowed("kali:policy"),
            data: std::borrow::Cow::Owned(policy_bytes),
        }
        .append_to(&mut wasm_bytes);
    }

    let (output_path, wit_path, meta_path) = build::library_output_paths_for(&source, out_dir);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![Diagnostic::error(
                e5::OUTPUT_ERROR as u32,
                format!(
                    "failed to create output directory '{}': {}",
                    parent.display(),
                    error
                ),
            )]
        })?;
    }
    fs::write(&output_path, &wasm_bytes).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write library artifact '{}': {}",
                output_path.display(),
                error
            ),
        )]
    })?;
    fs::write(&wit_path, wit).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write library WIT sidecar '{}': {}",
                wit_path.display(),
                error
            ),
        )]
    })?;
    fs::write(
        &meta_path,
        serde_json::to_string_pretty(&metadata).expect("serialize library metadata"),
    )
    .map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write library metadata '{}': {}",
                meta_path.display(),
                error
            ),
        )]
    })?;

    Ok(BuildResult::Library {
        output_path,
        wit_path,
        meta_path,
        wasm_bytes,
        metadata,
    })
}

fn build_capi_artifact(
    file: &str,
    mode: build::BuildMode,
    max_specializations: usize,
    out_dir: Option<&Path>,
    policy: Option<&SandboxPolicy>,
    api_surface: kali_cli::ApiSurface,
    compat_eval: bool,
    validate_ir: bool,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let mut wasm_bytes =
        build::compile_source_file_with_specialization_cap_and_profile_data_and_validation(
            &source,
            mode,
            max_specializations,
            api_surface,
            profile_data,
            runtime_profiles,
            compat_eval,
            policy.is_some(),
            validate_ir,
            false,
        )?;
    let exports = build::collect_library_exports(&source, api_surface, runtime_profiles)?;
    let wit = build::library_wit_for(&source.display().to_string(), &exports);
    let metadata = build::build_artifact_metadata(
        &source,
        "capi",
        mode,
        &api_surface.to_string(),
        runtime_profiles,
        max_specializations,
        profile_data,
        Some(exports.clone()),
    )?;
    build::append_metadata_section(&mut wasm_bytes, &metadata)?;

    if let Some(policy) = policy {
        validate_source_effects_against_policy(&source, policy, api_surface)?;
        let policy_bytes = policy
            .to_embedded_json_bytes()
            .map_err(|diagnostic| vec![diagnostic])?;
        CustomSection {
            name: std::borrow::Cow::Borrowed("kali:policy"),
            data: std::borrow::Cow::Owned(policy_bytes),
        }
        .append_to(&mut wasm_bytes);
    }

    let (output_path, wit_path, header_path, meta_path) =
        build::capi_output_paths_for(&source, out_dir);
    let binding_package_path = build::binding_package_manifest_output_path_for(&source, out_dir);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![Diagnostic::error(
                e5::OUTPUT_ERROR as u32,
                format!(
                    "failed to create output directory '{}': {}",
                    parent.display(),
                    error
                ),
            )]
        })?;
    }

    fs::write(&output_path, &wasm_bytes).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write C ABI WASM artifact '{}': {}",
                output_path.display(),
                error
            ),
        )]
    })?;

    fs::write(&wit_path, wit).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write C ABI WIT sidecar '{}': {}",
                wit_path.display(),
                error
            ),
        )]
    })?;

    let header_exports = exports
        .iter()
        .map(|export| CApiExport::new(export.name.clone(), arity_from_signature(&export.signature)))
        .collect::<Vec<_>>();
    let header = generate_header(&source.display().to_string(), &header_exports);
    fs::write(&header_path, header).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write C header '{}': {}",
                header_path.display(),
                error
            ),
        )]
    })?;

    let metadata_json = generate_capi_metadata(
        output_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("lib.capi.wasm"),
        wit_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("lib.wit"),
        header_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("lib.h"),
        runtime_profiles,
        max_specializations,
        Some(RuntimeHostContract::KaliHosted.canonical_label()),
        Some(RuntimeBackend::Wasmtime.canonical_label()),
    );
    parse_metadata(&metadata_json.to_string())
        .expect("generated C ABI metadata must satisfy schema-v1 shape");

    fs::write(
        &meta_path,
        serde_json::to_string_pretty(&metadata_json).expect("serialize capi metadata"),
    )
    .map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write C ABI metadata '{}': {}",
                meta_path.display(),
                error
            ),
        )]
    })?;

    let binding_package_json = generate_binding_package_manifest_with_provenance(
        &source.display().to_string(),
        output_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("lib.capi.wasm"),
        meta_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("lib.capi.meta.json"),
        header_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("lib.h"),
        runtime_profiles,
        max_specializations,
        Some(RuntimeHostContract::KaliHosted.canonical_label()),
        Some(RuntimeBackend::Wasmtime.canonical_label()),
        &[
            "bindings/python/README.md".to_string(),
            "bindings/python/kali_capi/__init__.py".to_string(),
            "bindings/python/pyproject.toml".to_string(),
        ],
    );
    parse_binding_package_manifest(&binding_package_json.to_string())
        .expect("generated binding package manifest must satisfy schema-v1 shape");
    fs::write(
        &binding_package_path,
        serde_json::to_string_pretty(&binding_package_json)
            .expect("serialize binding package manifest"),
    )
    .map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write binding package manifest '{}': {}",
                binding_package_path.display(),
                error
            ),
        )]
    })?;

    Ok(BuildResult::Capi {
        output_path,
        wit_path,
        header_path,
        meta_path,
        wasm_bytes,
        metadata,
    })
}

fn build_component_artifact(
    file: &str,
    mode: build::BuildMode,
    max_specializations: usize,
    out_dir: Option<&Path>,
    policy: Option<&SandboxPolicy>,
    api_surface: kali_cli::ApiSurface,
    compat_eval: bool,
    validate_ir: bool,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let wasm_bytes =
        build::compile_source_file_with_specialization_cap_and_profile_data_and_validation(
            &source,
            mode,
            max_specializations,
            api_surface,
            profile_data,
            runtime_profiles,
            compat_eval,
            policy.is_some(),
            validate_ir,
            false,
        )?;
    let exports = build::collect_library_exports(&source, api_surface, runtime_profiles)?;
    let wit = build::library_wit_for(&source.display().to_string(), &exports);
    let metadata = build::build_artifact_metadata(
        &source,
        "component",
        mode,
        &api_surface.to_string(),
        runtime_profiles,
        max_specializations,
        profile_data,
        Some(exports),
    )?;

    let mut component = Component::new();
    component.section(&RawSection {
        id: ComponentSectionId::CoreModule.into(),
        data: &wasm_bytes,
    });
    let mut component_bytes = component.finish();
    build::append_metadata_section(&mut component_bytes, &metadata)?;

    if let Some(policy) = policy {
        validate_source_effects_against_policy(&source, policy, api_surface)?;
        let policy_bytes = policy
            .to_embedded_json_bytes()
            .map_err(|diagnostic| vec![diagnostic])?;
        CustomSection {
            name: std::borrow::Cow::Borrowed("kali:policy"),
            data: std::borrow::Cow::Owned(policy_bytes),
        }
        .append_to(&mut component_bytes);
    }

    let (output_path, wit_path, meta_path, binding_package_path) =
        build::component_output_paths_for(&source, out_dir);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![Diagnostic::error(
                e5::OUTPUT_ERROR as u32,
                format!(
                    "failed to create output directory '{}': {}",
                    parent.display(),
                    error
                ),
            )]
        })?;
    }

    fs::write(&output_path, &component_bytes).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write component artifact '{}': {}",
                output_path.display(),
                error
            ),
        )]
    })?;

    fs::write(&wit_path, wit).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write component WIT sidecar '{}': {}",
                wit_path.display(),
                error
            ),
        )]
    })?;

    fs::write(
        &meta_path,
        serde_json::to_string_pretty(&metadata).expect("serialize component metadata"),
    )
    .map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write component metadata '{}': {}",
                meta_path.display(),
                error
            ),
        )]
    })?;

    let binding_package_json = generate_binding_package_manifest_with_provenance(
        &source.display().to_string(),
        output_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("lib.component.wasm"),
        meta_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("lib.component.meta.json"),
        wit_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("lib.wit"),
        runtime_profiles,
        max_specializations,
        Some(RuntimeHostContract::KaliHosted.canonical_label()),
        Some(RuntimeBackend::Wasmtime.canonical_label()),
        &[
            "bindings/python/README.md".to_string(),
            "bindings/python/kali_capi/__init__.py".to_string(),
            "bindings/python/pyproject.toml".to_string(),
        ],
    );
    fs::write(
        &binding_package_path,
        serde_json::to_string_pretty(&binding_package_json)
            .expect("serialize component binding package manifest"),
    )
    .map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write component binding package manifest '{}': {}",
                binding_package_path.display(),
                error
            ),
        )]
    })?;

    Ok(BuildResult::Component {
        output_path,
        wit_path,
        meta_path,
        binding_package_path,
        wasm_bytes: component_bytes,
        metadata,
    })
}

fn build_browser_bundle_artifact(
    file: &str,
    mode: build::BuildMode,
    max_specializations: usize,
    out_dir: Option<&Path>,
    policy: Option<&SandboxPolicy>,
    api_surface: kali_cli::ApiSurface,
    compat_eval: bool,
    validate_ir: bool,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
    format: BundleFormat,
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let canonical_source = fs::canonicalize(&source).unwrap_or_else(|_| source.clone());
    let mut visited = std::collections::BTreeSet::new();
    visited.insert(canonical_source);
    let bundle = write_browser_bundle_files(
        &source,
        mode,
        max_specializations,
        out_dir,
        policy,
        api_surface,
        compat_eval,
        validate_ir,
        profile_data,
        runtime_profiles,
        format,
        true,
    )?;
    let extra_artifacts = collect_browser_bundle_chunk_artifacts(
        &source,
        mode,
        max_specializations,
        Some(bundle.output_dir.as_path()),
        policy,
        api_surface,
        compat_eval,
        validate_ir,
        profile_data,
        runtime_profiles,
        format,
        true,
        &mut visited,
    )?;

    Ok(BuildResult::BrowserBundle {
        output_dir: bundle.output_dir,
        wasm_path: bundle.wasm_path,
        js_path: bundle.js_path,
        source_map_path: bundle.source_map_path,
        meta_path: bundle.meta_path,
        wasm_bytes: bundle.wasm_bytes,
        metadata: bundle.metadata,
        format: bundle.format,
        extra_artifacts,
    })
}

fn write_browser_bundle_files(
    source: &Path,
    mode: build::BuildMode,
    max_specializations: usize,
    out_dir: Option<&Path>,
    policy: Option<&SandboxPolicy>,
    api_surface: kali_cli::ApiSurface,
    compat_eval: bool,
    validate_ir: bool,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
    format: BundleFormat,
    tree_shake_exports: bool,
) -> Result<BrowserBundleBuild, Vec<Diagnostic>> {
    let mut wasm_bytes =
        build::compile_source_file_with_specialization_cap_and_profile_data_and_validation(
            source,
            mode,
            max_specializations,
            api_surface,
            profile_data,
            runtime_profiles,
            compat_eval,
            policy.is_some(),
            validate_ir,
            false,
        )?;
    let exports =
        build::collect_browser_bundle_exports(source, tree_shake_exports).unwrap_or_default();
    let metadata = build::build_artifact_metadata(
        source,
        "bundle",
        mode,
        &api_surface.to_string(),
        runtime_profiles,
        max_specializations,
        profile_data,
        Some(exports.clone()),
    )?;
    build::append_metadata_section(&mut wasm_bytes, &metadata)?;

    if let Some(policy) = policy {
        validate_source_effects_against_policy(source, policy, api_surface)?;
        let policy_bytes = policy
            .to_embedded_json_bytes()
            .map_err(|diagnostic| vec![diagnostic])?;
        CustomSection {
            name: std::borrow::Cow::Borrowed("kali:policy"),
            data: std::borrow::Cow::Owned(policy_bytes),
        }
        .append_to(&mut wasm_bytes);
    }

    let (wasm_path, js_path, source_map_path, meta_path) =
        build::bundle_output_paths_for(source, out_dir, format);
    let output_dir = js_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    if let Some(parent) = js_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![Diagnostic::error(
                e5::OUTPUT_ERROR as u32,
                format!(
                    "failed to create output directory '{}': {}",
                    parent.display(),
                    error
                ),
            )]
        })?;
    }

    let source_contents = fs::read_to_string(source).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to read browser bundle source '{}': {}",
                source.display(),
                error
            ),
        )]
    })?;
    let dynamic_import_targets = build::discover_dynamic_import_targets(source, &source_contents)?;
    let dynamic_import_map =
        browser_bundle_dynamic_import_map(&output_dir, format, &dynamic_import_targets)?;

    fs::write(&wasm_path, &wasm_bytes).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write browser bundle wasm '{}': {}",
                wasm_path.display(),
                error
            ),
        )]
    })?;
    fs::write(
        &meta_path,
        serde_json::to_string_pretty(&metadata).expect("serialize bundle metadata"),
    )
    .map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write browser bundle metadata '{}': {}",
                meta_path.display(),
                error
            ),
        )]
    })?;
    fs::write(
        &source_map_path,
        build::browser_bundle_source_map(source, &js_path, &source_contents, &exports),
    )
    .map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write browser bundle source map '{}': {}",
                source_map_path.display(),
                error
            ),
        )]
    })?;
    fs::write(
        &js_path,
        generate_browser_bundle_js(
            &wasm_path,
            &source_map_path,
            &exports,
            &dynamic_import_map,
            format,
        ),
    )
    .map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write browser bundle JS '{}': {}",
                js_path.display(),
                error
            ),
        )]
    })?;

    Ok(BrowserBundleBuild {
        output_dir,
        wasm_path,
        js_path,
        source_map_path,
        meta_path,
        wasm_bytes,
        metadata,
        format,
        extra_artifacts: Vec::new(),
    })
}

fn collect_browser_bundle_chunk_artifacts(
    source: &Path,
    mode: build::BuildMode,
    max_specializations: usize,
    out_dir: Option<&Path>,
    policy: Option<&SandboxPolicy>,
    api_surface: kali_cli::ApiSurface,
    compat_eval: bool,
    validate_ir: bool,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
    format: BundleFormat,
    _tree_shake_exports: bool,
    visited: &mut std::collections::BTreeSet<PathBuf>,
) -> Result<Vec<BundleArtifact>, Vec<Diagnostic>> {
    let source_contents = fs::read_to_string(source).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to read browser bundle source '{}': {}",
                source.display(),
                error
            ),
        )]
    })?;
    let mut artifacts = Vec::new();
    for chunk_target in build::discover_dynamic_import_targets(source, &source_contents)? {
        if !visited.insert(chunk_target.target.clone()) {
            continue;
        }
        let chunk_out_dir = build::bundle_chunk_output_dir_for(&chunk_target.target, out_dir);
        let chunk = write_browser_bundle_files(
            &chunk_target.target,
            mode,
            max_specializations,
            Some(&chunk_out_dir),
            policy,
            api_surface,
            compat_eval,
            validate_ir,
            profile_data,
            runtime_profiles,
            format,
            false,
        )?;
        artifacts.push(BundleArtifact {
            kind: "chunk-wasm".to_string(),
            path: chunk.wasm_path.clone(),
        });
        artifacts.push(BundleArtifact {
            kind: "chunk-js".to_string(),
            path: chunk.js_path.clone(),
        });
        artifacts.push(BundleArtifact {
            kind: "chunk-source-map".to_string(),
            path: chunk.source_map_path.clone(),
        });
        artifacts.push(BundleArtifact {
            kind: "chunk-meta-json".to_string(),
            path: chunk.meta_path.clone(),
        });
        artifacts.extend(chunk.extra_artifacts);
        let nested = collect_browser_bundle_chunk_artifacts(
            &chunk_target.target,
            mode,
            max_specializations,
            out_dir,
            policy,
            api_surface,
            compat_eval,
            validate_ir,
            profile_data,
            runtime_profiles,
            format,
            false,
            visited,
        )?;
        artifacts.extend(nested);
    }
    Ok(artifacts)
}

fn browser_bundle_dynamic_import_map(
    bundle_root: &Path,
    format: BundleFormat,
    targets: &[build::DynamicImportTarget],
) -> Result<BTreeMap<String, String>, Vec<Diagnostic>> {
    let mut map = BTreeMap::new();
    for target in targets {
        let chunk_out_dir = build::bundle_chunk_output_dir_for(&target.target, Some(bundle_root));
        let (_, chunk_js_path, _, _) =
            build::bundle_output_paths_for(&target.target, Some(&chunk_out_dir), format);
        let relative = relative_path(bundle_root, &chunk_js_path)
            .to_string_lossy()
            .replace('\\', "/");
        let relative = if relative.starts_with('.') {
            relative
        } else {
            format!("./{}", relative)
        };
        map.insert(
            normalize_dynamic_import_specifier(&target.specifier),
            relative,
        );
    }

    Ok(map)
}

fn normalize_dynamic_import_specifier(specifier: &str) -> String {
    let specifier = specifier.trim().replace('\\', "/");
    if specifier.is_empty() {
        return specifier;
    }

    let is_absolute = specifier.starts_with('/');
    let mut segments = Vec::new();

    for segment in specifier.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            if matches!(segments.last(), Some(last) if last != "..") {
                segments.pop();
            } else if !is_absolute {
                segments.push("..".to_string());
            }
            continue;
        }
        segments.push(segment.to_string());
    }

    if segments.is_empty() {
        return if is_absolute {
            "/".to_string()
        } else {
            ".".to_string()
        };
    }

    let mut normalized = String::new();
    if is_absolute {
        normalized.push('/');
    } else if !matches!(segments.first().map(String::as_str), Some("..")) {
        normalized.push_str("./");
    }
    normalized.push_str(&segments.join("/"));
    normalized
}

fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let from_components: Vec<PathComponent<'_>> = from.components().collect();
    let to_components: Vec<PathComponent<'_>> = to.components().collect();

    let mut common_prefix = 0usize;
    while common_prefix < from_components.len()
        && common_prefix < to_components.len()
        && from_components[common_prefix] == to_components[common_prefix]
    {
        common_prefix += 1;
    }

    let mut path = PathBuf::new();
    for component in &from_components[common_prefix..] {
        if !matches!(component, PathComponent::CurDir) {
            path.push("..");
        }
    }
    for component in &to_components[common_prefix..] {
        path.push(component.as_os_str());
    }

    if path.as_os_str().is_empty() {
        path.push(".");
    }

    path
}

fn generate_browser_bundle_js(
    wasm_path: &Path,
    source_map_path: &Path,
    exports: &[build::LibraryExport],
    dynamic_import_targets: &BTreeMap<String, String>,
    format: BundleFormat,
) -> String {
    let wasm_file = wasm_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("bundle.wasm");
    let map_file = source_map_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(match format {
            BundleFormat::Esm => "bundle.js.map",
            BundleFormat::Cjs => "bundle.cjs.map",
        });
    let dynamic_import_entries = dynamic_import_targets
        .iter()
        .map(|(specifier, target)| {
            format!(
                "  [{}, {}],\n",
                serde_json::to_string(specifier).expect("serialize import specifier"),
                serde_json::to_string(target).expect("serialize import target")
            )
        })
        .collect::<String>();
    let mut content = match format {
        BundleFormat::Esm => format!(
            r#"const wasmUrl = new URL("./{wasm_file}", import.meta.url);
const bundleBaseUrl = import.meta.url;
const dynamicImportTargets = new Map([
{dynamic_import_entries}]);

const defaultImportObject = {{
  "kali:rt": {{
    test_register() {{}},
    args_len() {{ return 0; }},
    process_pid() {{
      return 0;
    }},
    cwd(_pathPtr, _pathLen, _outPtr, _outCap) {{
      return 0;
    }},
    math_max(left, right) {{
      return left > right ? left : right;
    }},
    math_min(left, right) {{
      return left < right ? left : right;
    }},
    math_abs(value) {{
      return value < 0n ? -value : value;
    }},
    math_sign(value) {{
      if (value === 0n) {{
        return 0n;
      }}
      return value < 0n ? -1n : 1n;
    }},
    math_round(value) {{
      return value;
    }},
    math_imul(left, right) {{
      return BigInt.asIntN(32, left * right);
    }},
    math_clz32(value) {{
      return BigInt(Math.clz32(Number(BigInt.asUintN(32, value))));
    }},
    math_pow(left, right) {{
      if (right < 0n) {{
        throw new Error('Math.pow negative exponents are unavailable in the current phase; use a non-negative exponent or the later compatibility path');
      }}
      return BigInt.asIntN(64, left ** right);
    }},
    console_log(val) {{
      if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
    console_error(val) {{
      if (typeof console !== 'undefined' && typeof console.error === 'function') {{
        console.error(formatConsoleValue(val));
      }}
    }},
    console_warn(val) {{
      if (typeof console !== 'undefined' && typeof console.warn === 'function') {{
        console.warn(formatConsoleValue(val));
      }}
    }},
    console_info(val) {{
      if (typeof console !== 'undefined' && typeof console.info === 'function') {{
        console.info(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
    console_debug(val) {{
      if (typeof console !== 'undefined' && typeof console.debug === 'function') {{
        console.debug(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }}
  }}
}};

function mergeImportObject(overrides = {{}}) {{
  const mergedRt = {{
    ...defaultImportObject["kali:rt"],
    ...((overrides["kali:rt"] ?? {{}})),
  }};
  return {{
    ...defaultImportObject,
    ...overrides,
    "kali:rt": mergedRt,
  }};
}}

async function instantiate(importObject) {{
  if (typeof WebAssembly.instantiateStreaming === "function" && typeof fetch === "function") {{
    try {{
      const response = await fetch(wasmUrl);
      return await WebAssembly.instantiateStreaming(response, importObject);
    }} catch (_) {{
      // fall back to ArrayBuffer instantiation.
    }}
  }}
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  return await WebAssembly.instantiate(bytes, importObject);
}}

let wasmMemory = null;
const instancePromise = instantiate(defaultImportObject).then((instance) => {{
  wasmMemory = instance.instance.exports.memory ?? null;
  return instance.instance;
}});

const NULL_VALUE_TAG = -9223372036854775808n;
const UNDEFINED_VALUE_TAG = -9223372036854775807n;
function formatConsoleValue(val) {{
  if (typeof val === 'bigint') {{
    if (val === NULL_VALUE_TAG) {{
      return 'null';
    }}
    if (val === UNDEFINED_VALUE_TAG) {{
      return 'undefined';
    }}
    if ((val & 0x8000000000000000n) !== 0n && wasmMemory !== null) {{
      const offset = Number((val >> 32n) & 0x7fffffffn);
      const length = Number(val & 0xffffffffn);
      if (offset >= 0 && length >= 0 && offset + length <= wasmMemory.buffer.byteLength) {{
        const bytes = new Uint8Array(wasmMemory.buffer, offset, length);
        return new TextDecoder().decode(bytes);
      }}
    }}
    return val.toString();
  }}
  return String(val);
}}

function normalizeDynamicImportSpecifier(specifier) {{
  const normalized = String(specifier).trim().replace(/\\/g, '/');
  if (normalized.length === 0) {{
    return normalized;
  }}

  const absolute = normalized.startsWith('/');
  const segments = [];
  for (const segment of normalized.split('/')) {{
    if (!segment || segment === '.') {{
      continue;
    }}
    if (segment === '..') {{
      if (segments.length && segments[segments.length - 1] !== '..') {{
        segments.pop();
      }} else if (!absolute) {{
        segments.push('..');
      }}
      continue;
    }}
    segments.push(segment);
  }}

  if (segments.length === 0) {{
    return absolute ? '/' : '.';
  }}

  const prefix = absolute ? '/' : segments[0] === '..' ? '' : './';
  return prefix + segments.join('/');
}}

function resolveDynamicImportTarget(specifier) {{
  const target = dynamicImportTargets.get(normalizeDynamicImportSpecifier(specifier));
  if (!target) {{
    throw new Error(`unknown dynamic import target: ${{specifier}}`);
  }}
  return new URL(target, bundleBaseUrl);
}}

export async function load() {{
  return await instancePromise;
}}

export async function loadWithImports(overrides = {{}}) {{
  const instance = await instantiate(mergeImportObject(overrides));
  wasmMemory = instance.instance.exports.memory ?? null;
  return instance.instance;
}}

export async function loadDynamicImport(specifier) {{
  return await import(resolveDynamicImportTarget(specifier).href);
}}

"#
        ),
        BundleFormat::Cjs => format!(
            r#"const {{ pathToFileURL }} = require("url");
const wasmUrl = new URL("./{wasm_file}", pathToFileURL(__filename));
const bundleBaseUrl = pathToFileURL(__filename);
const dynamicImportTargets = new Map([
{dynamic_import_entries}]);

const defaultImportObject = {{
  "kali:rt": {{
    test_register() {{}},
    args_len() {{ return 0; }},
    process_pid() {{
      return 0;
    }},
    cwd(_pathPtr, _pathLen, _outPtr, _outCap) {{
      return 0;
    }},
    math_max(left, right) {{
      return left > right ? left : right;
    }},
    math_min(left, right) {{
      return left < right ? left : right;
    }},
    math_abs(value) {{
      return value < 0n ? -value : value;
    }},
    math_sign(value) {{
      if (value === 0n) {{
        return 0n;
      }}
      return value < 0n ? -1n : 1n;
    }},
    math_round(value) {{
      return value;
    }},
    math_imul(left, right) {{
      return BigInt.asIntN(32, left * right);
    }},
    math_clz32(value) {{
      return BigInt(Math.clz32(Number(BigInt.asUintN(32, value))));
    }},
    math_pow(left, right) {{
      if (right < 0n) {{
        throw new Error('Math.pow negative exponents are unavailable in the current phase; use a non-negative exponent or the later compatibility path');
      }}
      return BigInt.asIntN(64, left ** right);
    }},
    console_log(val) {{
      if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
    console_error(val) {{
      if (typeof console !== 'undefined' && typeof console.error === 'function') {{
        console.error(formatConsoleValue(val));
      }}
    }},
    console_warn(val) {{
      if (typeof console !== 'undefined' && typeof console.warn === 'function') {{
        console.warn(formatConsoleValue(val));
      }}
    }},
    console_info(val) {{
      if (typeof console !== 'undefined' && typeof console.info === 'function') {{
        console.info(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
    console_debug(val) {{
      if (typeof console !== 'undefined' && typeof console.debug === 'function') {{
        console.debug(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }}
  }}
}};

function mergeImportObject(overrides = {{}}) {{
  const mergedRt = {{
    ...defaultImportObject["kali:rt"],
    ...((overrides["kali:rt"] ?? {{}})),
  }};
  return {{
    ...defaultImportObject,
    ...overrides,
    "kali:rt": mergedRt,
  }};
}}

async function instantiate(importObject) {{
  if (typeof WebAssembly.instantiateStreaming === "function" && typeof fetch === "function") {{
    try {{
      const response = await fetch(wasmUrl);
      return await WebAssembly.instantiateStreaming(response, importObject);
    }} catch (_) {{
      // fall back to ArrayBuffer instantiation.
    }}
  }}
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  return await WebAssembly.instantiate(bytes, importObject);
}}

let wasmMemory = null;
const instancePromise = instantiate(defaultImportObject).then((instance) => {{
  wasmMemory = instance.instance.exports.memory ?? null;
  return instance.instance;
}});

const NULL_VALUE_TAG = -9223372036854775808n;
const UNDEFINED_VALUE_TAG = -9223372036854775807n;
function formatConsoleValue(val) {{
  if (typeof val === 'bigint') {{
    if (val === NULL_VALUE_TAG) {{
      return 'null';
    }}
    if (val === UNDEFINED_VALUE_TAG) {{
      return 'undefined';
    }}
    if ((val & 0x8000000000000000n) !== 0n && wasmMemory !== null) {{
      const offset = Number((val >> 32n) & 0x7fffffffn);
      const length = Number(val & 0xffffffffn);
      if (offset >= 0 && length >= 0 && offset + length <= wasmMemory.buffer.byteLength) {{
        const bytes = new Uint8Array(wasmMemory.buffer, offset, length);
        return new TextDecoder().decode(bytes);
      }}
    }}
    return val.toString();
  }}
  return String(val);
}}

function normalizeDynamicImportSpecifier(specifier) {{
  const normalized = String(specifier).trim().replace(/\\/g, '/');
  if (normalized.length === 0) {{
    return normalized;
  }}

  const absolute = normalized.startsWith('/');
  const segments = [];
  for (const segment of normalized.split('/')) {{
    if (!segment || segment === '.') {{
      continue;
    }}
    if (segment === '..') {{
      if (segments.length && segments[segments.length - 1] !== '..') {{
        segments.pop();
      }} else if (!absolute) {{
        segments.push('..');
      }}
      continue;
    }}
    segments.push(segment);
  }}

  if (segments.length === 0) {{
    return absolute ? '/' : '.';
  }}

  const prefix = absolute ? '/' : segments[0] === '..' ? '' : './';
  return prefix + segments.join('/');
}}

function resolveDynamicImportTarget(specifier) {{
  const target = dynamicImportTargets.get(normalizeDynamicImportSpecifier(specifier));
  if (!target) {{
    throw new Error(`unknown dynamic import target: ${{specifier}}`);
  }}
  return new URL(target, bundleBaseUrl);
}}

async function load() {{
  return await instancePromise;
}}

async function loadWithImports(overrides = {{}}) {{
  const instance = await instantiate(mergeImportObject(overrides));
  wasmMemory = instance.instance.exports.memory ?? null;
  return instance.instance;
}}

async function loadDynamicImport(specifier) {{
  return await import(resolveDynamicImportTarget(specifier).href);
}}

const exported = {{ load, loadWithImports, loadDynamicImport }};

"#
        ),
    };
    for export in exports {
        match format {
            BundleFormat::Esm => content.push_str(&format!(
                "export async function {}(...args) {{\n  const instance = await instancePromise;\n  return instance.exports.{}(...args);\n}}\n\n",
                export.name, export.name
            )),
            BundleFormat::Cjs => content.push_str(&format!(
                "exported.{0} = async function {0}(...args) {{\n  const instance = await instancePromise;\n  return instance.exports.{0}(...args);\n}};\n\n",
                export.name
            )),
        }
    }
    match format {
        BundleFormat::Esm => content.push_str(&format!("//# sourceMappingURL={}\n", map_file)),
        BundleFormat::Cjs => {
            content.push_str("module.exports = exported;\n");
            for export in exports {
                content.push_str(&format!(
                    "module.exports.{0} = exported.{0};\n",
                    export.name
                ));
            }
            content.push_str(&format!("//# sourceMappingURL={}\n", map_file));
        }
    }
    content
}
