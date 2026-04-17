use clap::Parser;
use kali_capi::{
    arity_from_signature, generate_header, generate_metadata as generate_capi_metadata,
    Export as CApiExport,
};
use kali_cli::{
    build, discover_source_files, discover_test_files, init, is_declaration_only_source_file,
    load_sandbox_policy,
    output::{self, CliOutputOptions},
    Args, BundleFormat, Commands,
};
use kali_error::{_error_codes::e5, Diagnostic};
use kali_fmt::format_source;
use kali_lint::lint_with_options;
use kali_npm::{
    discover_project_root, ensure_project_ready, install_project, load_manifest, InstallOptions,
    ProjectManifest,
};
use kali_runtime::RuntimeCtx;
use kali_sandbox::{
    compare_effects_to_policy, effect_report_from_inference, infer_effects_from_roots,
    package_effects_report, EffectAnalysisContext, PackageCoordinate, SandboxPolicy,
};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use wasm_encoder::{Component, ComponentSectionId, CustomSection, RawSection, Section};

fn main() {
    let args = Args::parse();
    let output = CliOutputOptions {
        format: args.output,
        pretty: args.pretty,
        verbose: args.verbose,
        quiet: args.quiet,
        color: args.color,
    };

    let pretty_allowed_without_json = matches!(
        args.command,
        Some(Commands::Effects { .. }) | Some(Commands::PackageEffects { .. })
    );
    if output.pretty && !output.is_json() && !pretty_allowed_without_json {
        eprintln!("error[E5008]: `--pretty` is only meaningful when JSON output is active");
        std::process::exit(5);
    }

    if args.command.is_none() {
        println!("kali 0.1.0");
        return;
    }

    match args.command.unwrap() {
        Commands::Check {
            sandbox,
            api,
            files,
        } => {
            if let Err(exit_code) = check_command(files, sandbox, api, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Build {
            sandbox,
            api,
            files,
            fast,
            release,
            release_advanced,
            max_specializations,
            bundle,
            format,
            lib,
            capi,
            component,
            out_dir,
        } => {
            if let Err(exit_code) = build_command(
                files,
                sandbox,
                api,
                fast,
                release,
                release_advanced,
                max_specializations,
                bundle,
                format,
                lib,
                capi,
                component,
                out_dir,
                &output,
            ) {
                std::process::exit(exit_code);
            }
        }
        Commands::Run {
            sandbox,
            api,
            files,
        } => {
            if let Err(exit_code) = run_command(files, api, sandbox, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Test {
            sandbox,
            api,
            files,
            filter,
            coverage,
        } => {
            if let Err(exit_code) = test_command(files, api, filter, coverage, sandbox, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Init { lib } => match init::init_current_directory(lib) {
            Ok(summary) => {
                if output.is_json() {
                    let payload = json!({
                        "root": summary.root,
                        "manifestPath": summary.manifest_path,
                        "sourcePath": summary.source_path,
                        "library": summary.library,
                    });
                    print_envelope(
                        "init",
                        true,
                        vec![],
                        vec![],
                        payload,
                        None,
                        None,
                        0,
                        &output,
                    );
                } else if !output.quiet {
                    let template = if summary.library {
                        "library"
                    } else {
                        "application"
                    };
                    println!(
                        "Initialized {} scaffold at {}",
                        template,
                        summary.root.display()
                    );
                }
            }
            Err(diagnostic) => {
                let exit_code = diagnostics_exit_code(std::slice::from_ref(&diagnostic));
                if output.is_json() {
                    let (errors, warnings) = single_diagnostic_to_values(diagnostic, None, None);
                    print_envelope(
                        "init",
                        false,
                        errors,
                        warnings,
                        Value::Null,
                        None,
                        None,
                        exit_code,
                        &output,
                    );
                } else {
                    eprintln!("{}", diagnostic);
                }
                std::process::exit(exit_code);
            }
        },
        Commands::Install {
            target,
            dev,
            allow_scripts,
        } => {
            if let Err(exit_code) = install_command(target, dev, allow_scripts, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Fmt { check, files } => {
            if let Err(exit_code) = fmt_command(files, check, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Lint { fix, files } => {
            if let Err(exit_code) = lint_command(files, fix, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Effects { files } => {
            if let Err(exit_code) = effects_command(files, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::PackageEffects { target } => {
            if let Err(exit_code) = package_effects_command(target, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::PackageAudit { target, .. } => {
            if let Err(exit_code) = package_audit_command(target, &output) {
                std::process::exit(exit_code);
            }
        }
    }
}

fn check_command(
    files: Vec<String>,
    sandbox: Option<PathBuf>,
    api: Option<kali_cli::ApiSurface>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let effective_api = match resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("check", diagnostics, 5, output, None, None)
        }
    };

    let policy = load_policy_or_exit(sandbox, output)?;
    ensure_project_ready_or_exit(output)?;

    let selected_files = if files.is_empty() {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project_root = discover_project_root(&cwd).unwrap_or(cwd);
        discover_source_files(&project_root)
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>()
    } else {
        files
    };

    if let Some(policy) = policy.as_ref() {
        if let Err(diagnostics) = policy.validate() {
            return emit_diagnostics_and_exit("check", diagnostics, 5, output, None, None);
        }
    }

    let mut checked = 0usize;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut successful_files = Vec::new();

    for file in selected_files {
        checked += 1;
        match build::check_source_file(&file, effective_api) {
            Ok(()) => {
                successful_files.push(PathBuf::from(&file));
            }
            Err(diagnostics) => {
                let source = fs::read_to_string(&file).ok();
                let (file_errors, file_warnings) = split_and_convert_diagnostics(
                    &diagnostics,
                    Some(Path::new(&file)),
                    source.as_deref(),
                );
                errors.extend(file_errors);
                warnings.extend(file_warnings);
                if !output.is_json() {
                    for diagnostic in diagnostics {
                        eprintln!("{}", diagnostic);
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        if let Some(policy) = policy.as_ref() {
            if let Err(diagnostics) = validate_source_effects_against_policy_for_roots(
                &successful_files,
                policy,
                effective_api,
            ) {
                let (file_errors, file_warnings) =
                    split_and_convert_diagnostics(&diagnostics, None, None);
                errors.extend(file_errors);
                warnings.extend(file_warnings);
                if !output.is_json() {
                    for diagnostic in diagnostics {
                        eprintln!("{}", diagnostic);
                    }
                }
            }
        }
    }

    let success = errors.is_empty();
    if output.is_json() {
        let payload = json!({
            "filesChecked": checked,
            "errorCount": errors.len(),
            "warningCount": warnings.len(),
        });
        print_envelope(
            "check",
            success,
            errors,
            warnings,
            payload,
            None,
            None,
            if success { 0 } else { 1 },
            output,
        );
    } else if success && !output.quiet {
        println!("Checked {} file(s)", checked);
    }

    if success {
        Ok(())
    } else {
        Err(1)
    }
}

#[allow(clippy::too_many_arguments)]
fn build_command(
    files: Vec<String>,
    sandbox: Option<PathBuf>,
    api: Option<kali_cli::ApiSurface>,
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
    let policy = load_policy_or_exit(sandbox, output)?;
    ensure_project_ready_or_exit(output)?;

    if let Some(policy) = policy.as_ref() {
        if let Err(diagnostics) = policy.validate() {
            return emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None);
        }
    }

    let effective_api = match resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None)
        }
    };
    if format.is_some() && !bundle {
        let diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            "`--format` is only meaningful when `--bundle` is selected",
        );
        return emit_diagnostics_and_exit("build", vec![diagnostic], 5, output, None, None);
    }

    if bundle {
        if !matches!(effective_api, kali_cli::ApiSurface::Browser) {
            let diagnostic = Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                "`kali build --bundle` requires the effective browser API surface",
            );
            return emit_diagnostics_and_exit("build", vec![diagnostic], 5, output, None, None);
        }
    } else if matches!(effective_api, kali_cli::ApiSurface::Browser) {
        let diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            "`kali build` without `--bundle` is not valid for the browser API surface",
        );
        return emit_diagnostics_and_exit("build", vec![diagnostic], 5, output, None, None);
    }

    let Some(source) = single_or_error(files, "build", output)? else {
        return Err(1);
    };

    let source = source.to_string_lossy().to_string();
    let mode = build::build_mode_from_flags(fast, release, release_advanced);
    let max_specializations = max_specializations.unwrap_or(16);
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
        ),
        BuildArtifactSelection::Library => build_library_artifact(
            &source,
            mode,
            max_specializations,
            out_dir_path,
            policy.as_ref(),
            effective_api,
        ),
        BuildArtifactSelection::Capi => build_capi_artifact(
            &source,
            mode,
            max_specializations,
            out_dir_path,
            policy.as_ref(),
            effective_api,
        ),
        BuildArtifactSelection::Component => build_component_artifact(
            &source,
            mode,
            max_specializations,
            out_dir_path,
            policy.as_ref(),
            effective_api,
        ),
        BuildArtifactSelection::BrowserBundle => build_browser_bundle_artifact(
            &source,
            mode,
            max_specializations,
            out_dir_path,
            policy.as_ref(),
            effective_api,
            bundle_format,
        ),
    };

    match build_result {
        Ok(build_result) => {
            if output.is_json() {
                let payload = build_result.artifact_json();
                print_envelope(
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
        Err(diagnostics) => emit_diagnostics_and_exit(
            "build",
            diagnostics,
            1,
            output,
            Some(Path::new(&source)),
            fs::read_to_string(&source).ok().as_deref(),
        ),
    }
}

fn resolve_effective_api_surface(
    explicit_api: Option<kali_cli::ApiSurface>,
) -> Result<kali_cli::ApiSurface, Vec<Diagnostic>> {
    if let Some(api) = explicit_api {
        return Ok(api);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let Some(manifest) = load_manifest(&project_root).map_err(|diagnostic| vec![diagnostic])?
    else {
        return Ok(kali_cli::ApiSurface::Deno);
    };

    manifest_api_surface(&manifest).map(|surface| surface.unwrap_or(kali_cli::ApiSurface::Deno))
}

fn manifest_api_surface(
    manifest: &ProjectManifest,
) -> Result<Option<kali_cli::ApiSurface>, Vec<Diagnostic>> {
    let Some(options) = manifest.compiler_options.as_ref() else {
        return Ok(None);
    };

    let Some(options) = options.as_object() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions` must be a JSON object",
        )]);
    };

    let Some(value) = options.get("apiSurface") else {
        return Ok(None);
    };

    let Some(api_surface) = value.as_str() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions.apiSurface` must be a string",
        )]);
    };

    match api_surface {
        "deno" => Ok(Some(kali_cli::ApiSurface::Deno)),
        "node" => Ok(Some(kali_cli::ApiSurface::Node)),
        "browser" => Ok(Some(kali_cli::ApiSurface::Browser)),
        _ => Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            format!("unsupported apiSurface '{}' in kali.json", api_surface),
        )]),
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

impl BuildResult {
    fn artifact_json(&self) -> Value {
        match self {
            BuildResult::Executable {
                output_path,
                wasm_bytes,
                metadata,
            } => json!({
                "artifactKind": "executable",
                "outputPath": output_path,
                "sizeBytes": wasm_bytes.len(),
                "buildMode": metadata.build_mode.clone(),
                "sourceHash": metadata.source_hash.clone(),
            }),
            BuildResult::Library {
                output_path,
                wit_path,
                meta_path,
                wasm_bytes,
                metadata,
            } => json!({
                "artifactKind": "lib",
                "outputPath": output_path,
                "sizeBytes": wasm_bytes.len(),
                "buildMode": metadata.build_mode.clone(),
                "sourceHash": metadata.source_hash.clone(),
                "metadataPath": meta_path,
                "witPath": wit_path,
                "artifacts": [
                    { "kind": "wasm-module", "path": output_path },
                    { "kind": "wit", "path": wit_path },
                    { "kind": "meta-json", "path": meta_path },
                ],
                "exports": metadata.exports.clone().unwrap_or_default(),
            }),
            BuildResult::Capi {
                output_path,
                wit_path,
                header_path,
                meta_path,
                wasm_bytes,
                metadata,
            } => json!({
                "artifactKind": "capi",
                "outputPath": output_path,
                "sizeBytes": wasm_bytes.len(),
                "buildMode": metadata.build_mode.clone(),
                "sourceHash": metadata.source_hash.clone(),
                "metadataPath": meta_path,
                "witPath": wit_path,
                "headerPath": header_path,
                "artifacts": [
                    { "kind": "wasm-module", "path": output_path },
                    { "kind": "wit", "path": wit_path },
                    { "kind": "c-header", "path": header_path },
                    { "kind": "cabi-metadata", "path": meta_path },
                ],
                "exports": metadata.exports.clone().unwrap_or_default(),
            }),
            BuildResult::Component {
                output_path,
                wit_path,
                meta_path,
                wasm_bytes,
                metadata,
            } => json!({
                "artifactKind": "component",
                "outputPath": output_path,
                "sizeBytes": wasm_bytes.len(),
                "buildMode": metadata.build_mode.clone(),
                "sourceHash": metadata.source_hash.clone(),
                "metadataPath": meta_path,
                "witPath": wit_path,
                "artifacts": [
                    { "kind": "wasm-component", "path": output_path },
                    { "kind": "wit", "path": wit_path },
                    { "kind": "meta-json", "path": meta_path },
                ],
                "exports": metadata.exports.clone().unwrap_or_default(),
            }),
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
                json!({
                    "artifactKind": "bundle",
                    "outputPath": output_dir,
                    "sizeBytes": wasm_bytes.len(),
                    "buildMode": metadata.build_mode.clone(),
                    "sourceHash": metadata.source_hash.clone(),
                    "artifacts": artifacts,
                    "exports": metadata.exports.clone().unwrap_or_default(),
                    "bundleFormat": format.to_string(),
                })
            }
        }
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
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let mut wasm_bytes = build::compile_source_file_with_specialization_cap(
        &source,
        mode,
        max_specializations,
        api_surface,
    )?;
    let metadata = build::build_artifact_metadata(
        &source,
        "executable",
        mode,
        &api_surface.to_string(),
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
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let mut wasm_bytes = build::compile_source_file_with_specialization_cap(
        &source,
        mode,
        max_specializations,
        api_surface,
    )?;
    let exports = build::collect_library_exports(&source)?;
    let wit = build::library_wit_for(&source.display().to_string(), &exports);
    let metadata = build::build_artifact_metadata(
        &source,
        "lib",
        mode,
        &api_surface.to_string(),
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
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let mut wasm_bytes = build::compile_source_file_with_specialization_cap(
        &source,
        mode,
        max_specializations,
        api_surface,
    )?;
    let exports = build::collect_library_exports(&source)?;
    let wit = build::library_wit_for(&source.display().to_string(), &exports);
    let metadata = build::build_artifact_metadata(
        &source,
        "capi",
        mode,
        &api_surface.to_string(),
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
    );

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
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let wasm_bytes = build::compile_source_file_with_specialization_cap(
        &source,
        mode,
        max_specializations,
        api_surface,
    )?;
    let exports = build::collect_library_exports(&source)?;
    let wit = build::library_wit_for(&source.display().to_string(), &exports);
    let metadata = build::build_artifact_metadata(
        &source,
        "component",
        mode,
        &api_surface.to_string(),
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

    let (output_path, wit_path, meta_path) = build::component_output_paths_for(&source, out_dir);
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

    Ok(BuildResult::Component {
        output_path,
        wit_path,
        meta_path,
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
        format,
    )?;
    let extra_artifacts = collect_browser_bundle_chunk_artifacts(
        &source,
        mode,
        max_specializations,
        Some(bundle.output_dir.as_path()),
        policy,
        api_surface,
        format,
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
    format: BundleFormat,
) -> Result<BrowserBundleBuild, Vec<Diagnostic>> {
    let mut wasm_bytes = build::compile_source_file_with_specialization_cap(
        source,
        mode,
        max_specializations,
        api_surface,
    )?;
    let exports = build::collect_library_exports(source).unwrap_or_default();
    let metadata = build::build_artifact_metadata(
        source,
        "bundle",
        mode,
        &api_surface.to_string(),
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
        generate_browser_bundle_js(&wasm_path, &source_map_path, &exports, format),
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
    format: BundleFormat,
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
    for chunk_source in discover_literal_dynamic_import_targets(source, &source_contents)? {
        if !visited.insert(chunk_source.clone()) {
            continue;
        }
        let chunk_out_dir = build::bundle_chunk_output_dir_for(&chunk_source, out_dir);
        let chunk = write_browser_bundle_files(
            &chunk_source,
            mode,
            max_specializations,
            Some(&chunk_out_dir),
            policy,
            api_surface,
            format,
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
            &chunk_source,
            mode,
            max_specializations,
            out_dir,
            policy,
            api_surface,
            format,
            visited,
        )?;
        artifacts.extend(nested);
    }
    Ok(artifacts)
}

fn discover_literal_dynamic_import_targets(
    source: &Path,
    source_contents: &str,
) -> Result<Vec<PathBuf>, Vec<Diagnostic>> {
    let mut targets = Vec::new();
    let mut index = 0;
    while let Some(relative) = source_contents[index..].find("import(") {
        index += relative + "import(".len();
        let bytes = source_contents.as_bytes();
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        let quote = bytes[index] as char;
        if !matches!(quote, '"' | '\'' | '`') {
            continue;
        }
        index += 1;
        let start = index;
        let mut escaped = false;
        while index < bytes.len() {
            let ch = bytes[index] as char;
            index += 1;
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                let specifier = &source_contents[start..index - 1];
                while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                    index += 1;
                }
                if index < bytes.len() && bytes[index] as char == ')' {
                    if let Some(target) = resolve_dynamic_import_target(source, specifier) {
                        targets.push(target);
                    }
                }
                break;
            }
        }
    }
    Ok(targets)
}

fn resolve_dynamic_import_target(source: &Path, specifier: &str) -> Option<PathBuf> {
    let specifier = specifier.trim();
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return None;
    }
    let parent = source.parent()?;
    let candidate = parent.join(specifier);
    let try_paths = std::iter::once(candidate.clone()).chain([
        candidate.with_extension("ts"),
        candidate.with_extension("tsx"),
        candidate.with_extension("js"),
        candidate.with_extension("jsx"),
        candidate.with_extension("mts"),
        candidate.with_extension("mjs"),
        candidate.with_extension("cts"),
        candidate.with_extension("cjs"),
    ]);
    for path in try_paths {
        if let Ok(canonical) = fs::canonicalize(&path) {
            return Some(canonical);
        }
    }
    None
}

fn generate_browser_bundle_js(
    wasm_path: &Path,
    source_map_path: &Path,
    exports: &[build::LibraryExport],
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
    let mut content = match format {
        BundleFormat::Esm => r#"const wasmUrl = new URL("./__WASM_FILE__", import.meta.url);

const importObject = {
  "kali:rt": {
    test_register() {}
  }
};

async function instantiate() {
  if (typeof WebAssembly.instantiateStreaming === "function" && typeof fetch === "function") {
    try {
      const response = await fetch(wasmUrl);
      return await WebAssembly.instantiateStreaming(response, importObject);
    } catch (_) {
      // fall back to ArrayBuffer instantiation.
    }
  }
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  return await WebAssembly.instantiate(bytes, importObject);
}

const instancePromise = instantiate();

export async function load() {
  return await instancePromise;
}

"#
        .to_string(),
        BundleFormat::Cjs => r#"const { pathToFileURL } = require("url");
const wasmUrl = new URL("./__WASM_FILE__", pathToFileURL(__filename));

const importObject = {
  "kali:rt": {
    test_register() {}
  }
};

async function instantiate() {
  if (typeof WebAssembly.instantiateStreaming === "function" && typeof fetch === "function") {
    try {
      const response = await fetch(wasmUrl);
      return await WebAssembly.instantiateStreaming(response, importObject);
    } catch (_) {
      // fall back to ArrayBuffer instantiation.
    }
  }
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  return await WebAssembly.instantiate(bytes, importObject);
}

const instancePromise = instantiate();

async function load() {
  return await instancePromise;
}

const exported = { load };

"#
        .to_string(),
    }
    .replace("__WASM_FILE__", wasm_file);
    for export in exports {
        match format {
            BundleFormat::Esm => content.push_str(&format!(
                "export async function {}(...args) {{\n  const {{ instance }} = await instancePromise;\n  return instance.exports.{}(...args);\n}}\n\n",
                export.name, export.name
            )),
            BundleFormat::Cjs => content.push_str(&format!(
                "exported.{0} = async function {0}(...args) {{\n  const {{ instance }} = await instancePromise;\n  return instance.exports.{0}(...args);\n}};\n\n",
                export.name
            )),
        }
    }
    match format {
        BundleFormat::Esm => content.push_str(&format!("//# sourceMappingURL={}\n", map_file)),
        BundleFormat::Cjs => content.push_str(&format!(
            "module.exports = exported;\n//# sourceMappingURL={}\n",
            map_file
        )),
    }
    content
}

fn run_command(
    files: Vec<String>,
    api: Option<kali_cli::ApiSurface>,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let effective_api = match resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("run", diagnostics, 5, output, None, None)
        }
    };

    if matches!(effective_api, kali_cli::ApiSurface::Browser) {
        let diagnostic = Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "selected API surface is unavailable in this phase",
        );
        return emit_diagnostics_and_exit("run", vec![diagnostic], 1, output, None, None);
    }

    let policy = load_policy_or_exit(sandbox, output)?;
    ensure_project_ready_or_exit(output)?;
    let Some(source) = single_or_error(files, "run", output)? else {
        return Err(1);
    };

    if let Err(diagnostic) = validate_runtime_entrypoint(&source) {
        return emit_diagnostics_and_exit("run", vec![diagnostic], 5, output, None, None);
    }

    let wasm_bytes =
        match build::compile_source_file(&source, build::BuildMode::Fast, effective_api) {
            Ok(bytes) => bytes,
            Err(diagnostics) => {
                return emit_diagnostics_and_exit(
                    "run",
                    diagnostics,
                    1,
                    output,
                    Some(&source),
                    fs::read_to_string(&source).ok().as_deref(),
                )
            }
        };

    let runtime = RuntimeCtx::with_api_surface(policy.clone(), effective_api.to_string());
    let start = Instant::now();
    match runtime.execute(&wasm_bytes) {
        Ok(outcome) => {
            if output.is_json() {
                let payload = json!({
                    "exitCode": outcome.exit_code,
                    "runtimeMs": start.elapsed().as_millis(),
                });
                print_envelope(
                    "run",
                    outcome.exit_code == 0,
                    vec![],
                    vec![],
                    payload,
                    Some(outcome.stdout),
                    Some(outcome.stderr),
                    outcome.exit_code,
                    output,
                );
            } else {
                if !output.quiet {
                    if !outcome.stdout.is_empty() {
                        print!("{}", outcome.stdout);
                    }
                    if !outcome.stderr.is_empty() {
                        eprint!("{}", outcome.stderr);
                    }
                }
            }
            if outcome.exit_code == 0 {
                Ok(())
            } else {
                Err(outcome.exit_code)
            }
        }
        Err(diagnostics) => emit_diagnostics_and_exit(
            "run",
            diagnostics,
            1,
            output,
            Some(&source),
            fs::read_to_string(&source).ok().as_deref(),
        ),
    }
}

fn test_command(
    files: Vec<String>,
    api: Option<kali_cli::ApiSurface>,
    filter: Option<String>,
    coverage: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if matches!(api, Some(kali_cli::ApiSurface::Browser)) {
        let diagnostic = Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "selected API surface is unavailable in this phase",
        );
        return emit_diagnostics_and_exit("test", vec![diagnostic], 1, output, None, None);
    }

    let effective_api = match resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("test", diagnostics, 5, output, None, None)
        }
    };

    if coverage {
        let diagnostic = Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "test coverage reporting is unavailable in this phase",
        );
        if output.is_json() {
            let (errors, warnings) = single_diagnostic_to_values(diagnostic, None, None);
            print_envelope(
                "test",
                false,
                errors,
                warnings,
                Value::Null,
                None,
                None,
                1,
                output,
            );
        } else {
            eprintln!("{}", diagnostic);
        }
        return Err(1);
    }

    let policy = load_policy_or_exit(sandbox, output)?;
    ensure_project_ready_or_exit(output)?;

    let selected_files = if files.is_empty() {
        discover_test_files(".")
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>()
    } else {
        files
    };

    let mut valid_files = Vec::new();
    for file in selected_files {
        let source = PathBuf::from(&file);
        if let Err(diagnostic) = validate_runtime_entrypoint(&source) {
            return emit_diagnostics_and_exit(
                "test",
                vec![diagnostic],
                5,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
        valid_files.push(file);
    }

    let filtered_files = if let Some(pattern) = filter.as_deref() {
        valid_files
            .into_iter()
            .filter(|file| matches_test_filter(file, pattern))
            .collect::<Vec<_>>()
    } else {
        valid_files
    };

    if filtered_files.is_empty() {
        if output.is_json() {
            let payload = json!({ "total": 0, "passed": 0, "failed": 0, "skipped": 0 });
            print_envelope(
                "test",
                true,
                vec![],
                vec![],
                payload,
                Some(String::new()),
                Some(String::new()),
                0,
                output,
            );
        } else if !output.quiet {
            println!("ok 0");
        }
        return Ok(());
    }

    let runtime = RuntimeCtx::with_api_surface(policy.clone(), effective_api.to_string());
    let mut total = 0usize;
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut captured_stdout = String::new();
    let mut captured_stderr = String::new();
    let mut diagnostics = Vec::new();
    let start = Instant::now();

    for file in filtered_files {
        let source = PathBuf::from(&file);
        let wasm_bytes =
            match build::compile_source_file(&source, build::BuildMode::Fast, effective_api) {
                Ok(bytes) => bytes,
                Err(errs) => {
                    diagnostics.extend(errs.clone());
                    if !output.is_json() {
                        for diagnostic in errs {
                            eprintln!("{}", diagnostic);
                        }
                    }
                    failed += 1;
                    continue;
                }
            };

        match runtime.execute_tests(&wasm_bytes) {
            Ok(outcome) => {
                total += outcome.tests_run;
                passed += outcome.tests_run.saturating_sub(outcome.tests_failed);
                failed += outcome.tests_failed;
                captured_stdout.push_str(&outcome.stdout);
                captured_stderr.push_str(&outcome.stderr);
            }
            Err(errs) => {
                diagnostics.extend(errs.clone());
                if !output.is_json() {
                    for diagnostic in errs {
                        eprintln!("{}", diagnostic);
                    }
                }
                failed += 1;
            }
        }
    }

    if output.is_json() {
        let payload = json!({
            "total": total,
            "passed": passed,
            "failed": failed,
            "skipped": 0,
            "runtimeMs": start.elapsed().as_millis(),
        });
        let success = diagnostics.is_empty();
        let (errors, warnings) = split_and_convert_diagnostics(&diagnostics, None, None);
        print_envelope(
            "test",
            success,
            errors,
            warnings,
            payload,
            Some(captured_stdout),
            Some(captured_stderr),
            if success { 0 } else { 1 },
            output,
        );
    } else if !output.quiet {
        if !captured_stdout.is_empty() {
            print!("{}", captured_stdout);
        }
        if !captured_stderr.is_empty() {
            eprint!("{}", captured_stderr);
        }
        if diagnostics.is_empty() {
            println!("ok {}", passed);
        } else {
            println!("FAILED {}", failed);
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(1)
    }
}

fn fmt_command(files: Vec<String>, check: bool, output: &CliOutputOptions) -> Result<(), i32> {
    ensure_project_ready_or_exit(output)?;
    let selected_files = selected_source_files(files, true);
    if selected_files.is_empty() {
        if output.is_json() {
            let payload = json!({"filesFormatted": 0, "filesChecked": 0});
            print_envelope("fmt", true, vec![], vec![], payload, None, None, 0, output);
        } else if !output.quiet {
            println!("{} 0 file(s)", if check { "Checked" } else { "Formatted" });
        }
        return Ok(());
    }

    let mut changed = 0usize;
    let mut processed = 0usize;
    for file in selected_files {
        processed += 1;
        let path = PathBuf::from(&file);
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                let diagnostic = Diagnostic::error(
                    e5::OUTPUT_ERROR as u32,
                    format!("failed to read source file '{}': {}", path.display(), error),
                );
                return emit_diagnostics_and_exit(
                    "fmt",
                    vec![diagnostic],
                    1,
                    output,
                    Some(&path),
                    None,
                );
            }
        };
        let formatted = format_source(&source);
        if formatted != source {
            changed += 1;
            if !check {
                if let Err(error) = fs::write(&path, formatted) {
                    let diagnostic = Diagnostic::error(
                        e5::OUTPUT_ERROR as u32,
                        format!(
                            "failed to write formatted file '{}': {}",
                            path.display(),
                            error
                        ),
                    );
                    return emit_diagnostics_and_exit(
                        "fmt",
                        vec![diagnostic],
                        1,
                        output,
                        Some(&path),
                        Some(&source),
                    );
                }
            }
        }
    }

    if output.is_json() {
        let payload = json!({"filesFormatted": changed, "filesChecked": processed});
        let success = !check || changed == 0;
        print_envelope(
            "fmt",
            success,
            vec![],
            vec![],
            payload,
            None,
            None,
            if check && changed > 0 { 1 } else { 0 },
            output,
        );
    } else if !output.quiet {
        if check {
            if changed == 0 {
                println!("Checked {} file(s)", processed);
            } else {
                println!("Would format {} file(s)", changed);
            }
        } else {
            println!("Formatted {} file(s)", changed);
        }
    }

    if check && changed > 0 {
        Err(1)
    } else {
        Ok(())
    }
}

fn lint_command(files: Vec<String>, fix: bool, output: &CliOutputOptions) -> Result<(), i32> {
    ensure_project_ready_or_exit(output)?;
    let selected_files = selected_source_files(files, true);
    if selected_files.is_empty() {
        if output.is_json() {
            let payload =
                json!({"filesLinted": 0, "errorCount": 0, "warningCount": 0, "fixedCount": 0});
            print_envelope("lint", true, vec![], vec![], payload, None, None, 0, output);
        } else if !output.quiet {
            println!("Linted 0 file(s)");
        }
        return Ok(());
    }

    let mut processed = 0usize;
    let mut had_error = false;
    let mut fixed = 0usize;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for file in selected_files {
        processed += 1;
        let path = PathBuf::from(&file);
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                let diagnostic = Diagnostic::error(
                    e5::OUTPUT_ERROR as u32,
                    format!("failed to read source file '{}': {}", path.display(), error),
                );
                return emit_diagnostics_and_exit(
                    "lint",
                    vec![diagnostic],
                    1,
                    output,
                    Some(&path),
                    None,
                );
            }
        };

        let result = lint_with_options(&source, fix);
        let (file_errors, file_warnings) =
            split_and_convert_diagnostics(&result.diagnostics, Some(&path), Some(&source));
        had_error |= !file_errors.is_empty();
        errors.extend(file_errors);
        warnings.extend(file_warnings);

        if let Some(fixed_source) = result.fixed_source {
            if fix && fixed_source != source {
                if let Err(error) = fs::write(&path, fixed_source) {
                    let diagnostic = Diagnostic::error(
                        e5::OUTPUT_ERROR as u32,
                        format!("failed to write fixed file '{}': {}", path.display(), error),
                    );
                    return emit_diagnostics_and_exit(
                        "lint",
                        vec![diagnostic],
                        1,
                        output,
                        Some(&path),
                        Some(&source),
                    );
                }
                fixed += 1;
            }
        }
    }

    if output.is_json() {
        let payload = json!({
            "filesLinted": processed,
            "errorCount": errors.len(),
            "warningCount": warnings.len(),
            "fixedCount": fixed,
        });
        print_envelope(
            "lint",
            !had_error,
            errors,
            warnings,
            payload,
            None,
            None,
            if had_error { 1 } else { 0 },
            output,
        );
    } else if !output.quiet {
        if fix {
            println!("Fixed {} file(s)", fixed);
        }
        println!("Linted {} file(s)", processed);
    }

    if had_error {
        Err(1)
    } else {
        Ok(())
    }
}

fn effects_command(files: Vec<String>, output: &CliOutputOptions) -> Result<(), i32> {
    let Some(source) = single_or_error(files, "effects", output)? else {
        return Err(1);
    };

    if let Err(diagnostic) = validate_runtime_entrypoint(&source) {
        return emit_diagnostics_and_exit(
            "effects",
            vec![diagnostic],
            5,
            output,
            Some(&source),
            fs::read_to_string(&source).ok().as_deref(),
        );
    }

    let effective_api = match resolve_effective_api_surface(None) {
        Ok(api) => api,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit(
                "effects",
                diagnostics,
                5,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
    };
    let context = analysis_context_for_api(effective_api);
    let inference = match infer_effects_from_roots(&[source.clone()], context.clone()) {
        Ok(inference) => inference,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit(
                "effects",
                diagnostics,
                1,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
    };

    let report = effect_report_from_inference(
        vec![source.to_string_lossy().to_string()],
        context,
        inference,
    );
    emit_native_json_payload("effects", &report, output)
}

fn package_effects_command(target: String, output: &CliOutputOptions) -> Result<(), i32> {
    let parsed = match parse_registry_package_target("package-effects", &target) {
        Ok(parsed) => parsed,
        Err(diagnostic) => {
            return emit_diagnostics_and_exit(
                "package-effects",
                vec![diagnostic],
                5,
                output,
                None,
                None,
            );
        }
    };

    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            let diagnostic = Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                format!("failed to read current directory: {}", error),
            );
            return emit_diagnostics_and_exit(
                "package-effects",
                vec![diagnostic],
                1,
                output,
                None,
                None,
            );
        }
    };
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let entry_path = match resolve_installed_package_entry(&project_root, &parsed.install_name) {
        Some(path) => path,
        None => {
            let diagnostic = Diagnostic::error(
                e5::DEPENDENCY_STATE_MISSING as u32,
                format!(
                    "package '{}' is not materialized in the current project",
                    target
                ),
            );
            return emit_diagnostics_and_exit(
                "package-effects",
                vec![diagnostic],
                1,
                output,
                None,
                None,
            );
        }
    };

    let package_root = match find_package_root(&entry_path) {
        Some(root) => root,
        None => {
            let diagnostic = Diagnostic::error(
                e5::DEPENDENCY_STATE_MISSING as u32,
                format!("unable to locate package root for '{}'", target),
            );
            return emit_diagnostics_and_exit(
                "package-effects",
                vec![diagnostic],
                1,
                output,
                None,
                None,
            );
        }
    };

    let package_version = match read_package_version(&package_root) {
        Ok(version) => version,
        Err(diagnostic) => {
            return emit_diagnostics_and_exit(
                "package-effects",
                vec![diagnostic],
                1,
                output,
                None,
                None,
            );
        }
    };

    let effective_api = match resolve_effective_api_surface(None) {
        Ok(api) => api,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit(
                "package-effects",
                diagnostics,
                5,
                output,
                None,
                None,
            );
        }
    };
    let context = analysis_context_for_api(effective_api);
    let inference = match infer_effects_from_roots(&[entry_path.clone()], context.clone()) {
        Ok(inference) => inference,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit(
                "package-effects",
                diagnostics,
                1,
                output,
                None,
                None,
            );
        }
    };

    let report = effect_report_from_inference(vec![parsed.report_label], context, inference);
    let payload = package_effects_report(
        PackageCoordinate {
            name: parsed.package_name,
            version: package_version,
            registry: parsed.registry,
        },
        report,
    );
    emit_native_json_payload("package-effects", &payload, output)
}

fn package_audit_command(target: String, output: &CliOutputOptions) -> Result<(), i32> {
    let parsed = match parse_registry_package_target("package-audit", &target) {
        Ok(parsed) => parsed,
        Err(diagnostic) => {
            return emit_diagnostics_and_exit(
                "package-audit",
                vec![diagnostic],
                5,
                output,
                None,
                None,
            );
        }
    };

    let summary = format!(
        "Package audit scaffold for {} package '{}'; no security findings are computed yet.",
        parsed.registry, parsed.report_label
    );

    if output.is_json() {
        print_envelope(
            "package-audit",
            true,
            vec![],
            vec![],
            Value::Null,
            Some(summary),
            None,
            0,
            output,
        );
    } else if !output.quiet {
        println!("{summary}");
    }

    Ok(())
}


fn install_command(
    target: Option<String>,
    dev: bool,
    allow_scripts: bool,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            let diagnostic = Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                format!("failed to read current directory: {}", error),
            );
            return emit_diagnostics_and_exit("install", vec![diagnostic], 1, output, None, None);
        }
    };
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let result = install_project(
        project_root,
        InstallOptions {
            target,
            dev,
            allow_scripts,
            suppress_script_output: output.is_json() || output.quiet,
        },
    );
    match result {
        Ok(summary) => {
            if output.is_json() {
                let payload = json!({
                    "manifestPath": summary.manifest_path,
                    "lockPath": summary.lock_path,
                    "installed": summary.installed,
                    "updated": [],
                    "removed": [],
                });
                print_envelope(
                    "install",
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
                println!("Installed {} package(s)", summary.installed.len());
            }
            Ok(())
        }
        Err(diagnostics) => {
            let exit_code = diagnostics_exit_code(&diagnostics);
            emit_diagnostics_and_exit("install", diagnostics, exit_code, output, None, None)
        }
    }
}

fn load_policy_or_exit(
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<Option<SandboxPolicy>, i32> {
    match sandbox {
        Some(path) => match load_sandbox_policy(&path) {
            Ok(policy) => Ok(Some(policy)),
            Err(diagnostics) => {
                emit_diagnostics_and_exit("policy", diagnostics, 5, output, Some(&path), None)
                    .map(|_| None)
            }
        },
        None => Ok(None),
    }
}

fn ensure_project_ready_or_exit(output: &CliOutputOptions) -> Result<(), i32> {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            let diagnostic = Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                format!("failed to read current directory: {}", error),
            );
            return emit_diagnostics_and_exit("cli", vec![diagnostic], 1, output, None, None);
        }
    };
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    match ensure_project_ready(project_root) {
        Ok(()) => Ok(()),
        Err(diagnostic) => {
            emit_diagnostics_and_exit("cli", vec![diagnostic], 1, output, None, None)
        }
    }
}

fn selected_source_files(files: Vec<String>, discover: bool) -> Vec<String> {
    if files.is_empty() && discover {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project_root = discover_project_root(&cwd).unwrap_or(cwd);
        discover_source_files(&project_root)
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect()
    } else {
        files
    }
}

fn single_or_error(
    files: Vec<String>,
    command: &str,
    output: &CliOutputOptions,
) -> Result<Option<PathBuf>, i32> {
    match files.as_slice() {
        [] => {
            let diagnostic = Diagnostic::error(
                e5::MISSING_REQUIRED_ARGUMENT as u32,
                format!("{} requires at least one source file", command),
            );
            emit_diagnostics_and_exit(command, vec![diagnostic], 5, output, None, None)
                .map(|_| None)
        }
        [file] => Ok(Some(PathBuf::from(file))),
        _ => {
            let diagnostic = Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                format!(
                    "{} accepts only one primary source file in this stage",
                    command
                ),
            );
            emit_diagnostics_and_exit(command, vec![diagnostic], 5, output, None, None)
                .map(|_| None)
        }
    }
}

fn validate_runtime_entrypoint(source: &PathBuf) -> Result<(), Diagnostic> {
    if is_declaration_only_source_file(source) {
        Err(Diagnostic::error(
            e5::INVALID_PRIMARY_INPUT_KIND as u32,
            format!(
                "declaration-only file '{}' cannot be used as a runtime entrypoint",
                source.display()
            ),
        )
        .with_suggestion("use `kali check` for declaration-only files"))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ParsedRegistryPackageTarget {
    registry: String,
    package_name: String,
    install_name: String,
    report_label: String,
}

fn parse_registry_package_target(
    command: &str,
    target: &str,
) -> Result<ParsedRegistryPackageTarget, Diagnostic> {
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("./")
        || target.starts_with("../")
        || target.starts_with('/')
    {
        return Err(Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            format!(
                "`kali {command}` accepts only registry package identifiers, not '{}'",
                target
            ),
        ));
    }

    let (registry, package_name, install_name, report_label) =
        if let Some(spec) = target.strip_prefix("jsr:") {
            if is_version_suffixed_package_spec(spec) {
                return Err(Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    format!("`kali {command}` does not accept explicit package versions yet"),
                ));
            }
            (
                "jsr".to_string(),
                spec.to_string(),
                spec.to_string(),
                target.to_string(),
            )
        } else {
            if is_version_suffixed_package_spec(target) {
                return Err(Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    format!("`kali {command}` does not accept explicit package versions yet"),
                ));
            }
            (
                "npm".to_string(),
                target.to_string(),
                target.to_string(),
                target.to_string(),
            )
        };

    Ok(ParsedRegistryPackageTarget {
        registry,
        package_name,
        install_name,
        report_label,
    })
}

fn is_version_suffixed_package_spec(spec: &str) -> bool {
    match spec.rsplit_once('@') {
        Some((name, version)) if !name.is_empty() && !version.is_empty() => true,
        _ => false,
    }
}

fn resolve_installed_package_entry(project_root: &Path, package_name: &str) -> Option<PathBuf> {
    let package_dir = project_root.join("node_modules").join(package_name);
    if !package_dir.exists() {
        return None;
    }

    let package_json_path = package_dir.join("package.json");
    let package_json = fs::read_to_string(package_json_path)
        .ok()
        .and_then(|contents| serde_json::from_str::<Value>(&contents).ok())?;

    let candidate = package_json
        .get("main")
        .and_then(|value| value.as_str())
        .or_else(|| package_json.get("module").and_then(|value| value.as_str()))
        .or_else(|| package_json.get("types").and_then(|value| value.as_str()))
        .or_else(|| package_json.get("typings").and_then(|value| value.as_str()))
        .and_then(|path| resolve_package_candidate(&package_dir, path))
        .or_else(|| resolve_package_candidate(&package_dir, "index.js"))
        .or_else(|| resolve_package_candidate(&package_dir, "index.mjs"))
        .or_else(|| resolve_package_candidate(&package_dir, "index.ts"))
        .or_else(|| resolve_package_candidate(&package_dir, "index.d.ts"))
        .or_else(|| resolve_package_candidate(&package_dir, "index.d.mts"))
        .or_else(|| resolve_package_candidate(&package_dir, "index.d.cts"));

    candidate
}

fn resolve_package_candidate(package_dir: &Path, candidate: &str) -> Option<PathBuf> {
    let path = package_dir.join(candidate);
    if path.is_file() {
        return Some(path);
    }

    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        if matches!(
            ext,
            "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs"
        ) {
            return None;
        }
    }

    for extension in [
        "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "d.ts", "d.mts", "d.cts",
    ] {
        let with_ext = package_dir.join(format!("{}.{}", candidate, extension));
        if with_ext.is_file() {
            return Some(with_ext);
        }
    }

    None
}

fn find_package_root(entry_path: &Path) -> Option<PathBuf> {
    let mut current = entry_path.parent()?.to_path_buf();
    loop {
        if current.join("package.json").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn read_package_version(package_root: &Path) -> Result<String, Diagnostic> {
    let path = package_root.join("package.json");
    let raw = fs::read_to_string(&path).map_err(|error| {
        Diagnostic::error(
            e5::DEPENDENCY_STATE_MISSING as u32,
            format!(
                "failed to read package metadata '{}': {}",
                path.display(),
                error
            ),
        )
    })?;
    let package_json: Value = serde_json::from_str(&raw).map_err(|error| {
        Diagnostic::error(
            e5::DEPENDENCY_STATE_MISSING as u32,
            format!(
                "failed to parse package metadata '{}': {}",
                path.display(),
                error
            ),
        )
    })?;
    package_json
        .get("version")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| {
            Diagnostic::error(
                e5::DEPENDENCY_STATE_MISSING as u32,
                format!("package metadata '{}' is missing a version", path.display()),
            )
        })
}

fn analysis_context_for_api(api: kali_cli::ApiSurface) -> EffectAnalysisContext {
    EffectAnalysisContext::new(api.to_string())
}

fn validate_source_effects_against_policy(
    source: &Path,
    policy: &SandboxPolicy,
    api: kali_cli::ApiSurface,
) -> Result<(), Vec<Diagnostic>> {
    validate_source_effects_against_policy_for_roots(&[source.to_path_buf()], policy, api)
}

fn validate_source_effects_against_policy_for_roots(
    roots: &[PathBuf],
    policy: &SandboxPolicy,
    api: kali_cli::ApiSurface,
) -> Result<(), Vec<Diagnostic>> {
    let context = analysis_context_for_api(api);
    let inference = infer_effects_from_roots(roots, context)?;
    let diagnostics = compare_effects_to_policy(&inference.effects, policy);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn emit_native_json_payload<T: serde::Serialize>(
    command: &str,
    payload: &T,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if output.is_json() {
        let value = serde_json::to_value(payload).expect("serialize native json payload");
        print_envelope(command, true, vec![], vec![], value, None, None, 0, output);
    } else if output.pretty {
        println!(
            "{}",
            serde_json::to_string_pretty(payload).expect("serialize native json payload")
        );
    } else {
        println!(
            "{}",
            serde_json::to_string(payload).expect("serialize native json payload")
        );
    }
    Ok(())
}

fn diagnostics_exit_code(diagnostics: &[Diagnostic]) -> i32 {
    if diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.code,
            Some(code) if matches!(code, 5001 | 5007 | 5008 | 5009 | 5010)
        )
    }) {
        5
    } else {
        1
    }
}

fn matches_test_filter(file: &str, pattern: &str) -> bool {
    let path = PathBuf::from(file);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(file);
    file.contains(pattern) || name.contains(pattern)
}

#[allow(clippy::too_many_arguments)]
fn print_envelope(
    command: &str,
    success: bool,
    errors: Vec<Value>,
    warnings: Vec<Value>,
    payload: Value,
    stdout: Option<String>,
    stderr: Option<String>,
    exit_code: i32,
    output: &CliOutputOptions,
) {
    let value = output::emit_envelope_value(
        command,
        success,
        Value::Array(errors),
        Value::Array(warnings),
        payload,
        stdout,
        stderr,
        exit_code,
    );
    if output.pretty {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).expect("serialize json envelope")
        );
    } else {
        println!(
            "{}",
            serde_json::to_string(&value).expect("serialize json envelope")
        );
    }
}

fn emit_diagnostics_and_exit(
    command: &str,
    diagnostics: Vec<Diagnostic>,
    exit_code: i32,
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_text: Option<&str>,
) -> Result<(), i32> {
    if output.is_json() {
        let (errors, warnings) =
            split_and_convert_diagnostics(&diagnostics, source_path, source_text);
        print_envelope(
            command,
            errors.is_empty(),
            errors,
            warnings,
            Value::Null,
            None,
            None,
            exit_code,
            output,
        );
    } else {
        for diagnostic in diagnostics {
            eprintln!("{}", diagnostic);
        }
    }
    Err(exit_code)
}

fn split_and_convert_diagnostics(
    diagnostics: &[Diagnostic],
    source_path: Option<&Path>,
    source_text: Option<&str>,
) -> (Vec<Value>, Vec<Value>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for diagnostic in diagnostics {
        let value = output::diagnostic_to_json(diagnostic, source_path, source_text, "error");
        if diagnostic.is_error() {
            errors.push(value);
        } else {
            warnings.push(value);
        }
    }
    (errors, warnings)
}

fn single_diagnostic_to_values(
    diagnostic: Diagnostic,
    source_path: Option<&Path>,
    source_text: Option<&str>,
) -> (Vec<Value>, Vec<Value>) {
    let diagnostics = vec![diagnostic];
    split_and_convert_diagnostics(&diagnostics, source_path, source_text)
}
