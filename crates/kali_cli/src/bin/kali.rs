use clap::Parser;
use kali_cli::{
    build, discover_source_files, discover_test_files, init, is_declaration_only_source_file,
    load_sandbox_policy,
    output::{self, CliOutputOptions},
    Args, Commands,
};
use kali_error::{Diagnostic, _error_codes::e5};
use kali_fmt::format_source;
use kali_lint::lint_with_options;
use kali_npm::{
    discover_project_root, ensure_project_ready, install_project, load_manifest, InstallOptions,
    ProjectManifest,
};
use kali_runtime::RuntimeCtx;
use kali_sandbox::SandboxPolicy;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use wasm_encoder::{CustomSection, Section};

fn main() {
    let args = Args::parse();
    let output = CliOutputOptions {
        format: args.output,
        pretty: args.pretty,
        verbose: args.verbose,
        quiet: args.quiet,
        color: args.color,
    };

    if output.pretty && !output.is_json() {
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
            bundle,
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
                bundle,
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
            if let Err(exit_code) =
                unavailable_registry_analysis_command("package-effects", target, &output)
            {
                std::process::exit(exit_code);
            }
        }
        Commands::PackageAudit { target } => {
            if let Err(exit_code) =
                unavailable_registry_analysis_command("package-audit", target, &output)
            {
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

    if matches!(effective_api, kali_cli::ApiSurface::Node) {
        let diagnostic = Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "selected API surface is unavailable in this phase",
        );
        return emit_diagnostics_and_exit("check", vec![diagnostic], 5, output, None, None);
    }

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

    for file in selected_files {
        checked += 1;
        match build::check_source_file(&file) {
            Ok(()) => {}
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
    } else if success {
        if !output.quiet {
            println!("Checked {} file(s)", checked);
        }
    }

    if success {
        Ok(())
    } else {
        Err(1)
    }
}

fn build_command(
    files: Vec<String>,
    sandbox: Option<PathBuf>,
    api: Option<kali_cli::ApiSurface>,
    fast: bool,
    release: bool,
    release_advanced: bool,
    bundle: bool,
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

    if capi || component {
        let diagnostic = Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            if capi {
                "`kali build --capi` is unavailable in this phase"
            } else {
                "`kali build --component` is unavailable in this phase"
            },
        );
        return emit_diagnostics_and_exit("build", vec![diagnostic], 1, output, None, None);
    }

    let effective_api = match resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None)
        }
    };
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
    } else if matches!(effective_api, kali_cli::ApiSurface::Node) {
        let diagnostic = Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "selected API surface is unavailable in this phase",
        );
        return emit_diagnostics_and_exit("build", vec![diagnostic], 5, output, None, None);
    }

    let Some(source) = single_or_error(files, "build", output)? else {
        return Err(1);
    };

    let source = source.to_string_lossy().to_string();
    let mode = build::build_mode_from_flags(fast, release, release_advanced);
    let out_dir_path = out_dir.as_deref();
    let artifact_mode = if lib {
        BuildArtifactSelection::Library
    } else if bundle {
        BuildArtifactSelection::BrowserBundle
    } else {
        BuildArtifactSelection::Executable
    };

    let build_result = match artifact_mode {
        BuildArtifactSelection::Executable => {
            build_executable_artifact(&source, mode, out_dir_path, policy.as_ref(), effective_api)
        }
        BuildArtifactSelection::Library => {
            build_library_artifact(&source, mode, out_dir_path, policy.as_ref(), effective_api)
        }
        BuildArtifactSelection::BrowserBundle => build_browser_bundle_artifact(
            &source,
            mode,
            out_dir_path,
            policy.as_ref(),
            effective_api,
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
}

enum BuildResult {
    Executable {
        output_path: PathBuf,
        wasm_bytes: Vec<u8>,
        metadata: build::ArtifactMetadata,
    },
    Library {
        output_path: PathBuf,
        meta_path: PathBuf,
        wasm_bytes: Vec<u8>,
        metadata: build::ArtifactMetadata,
    },
    BrowserBundle {
        output_dir: PathBuf,
        wasm_path: PathBuf,
        js_path: PathBuf,
        meta_path: PathBuf,
        wasm_bytes: Vec<u8>,
        metadata: build::ArtifactMetadata,
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
                "exports": metadata.exports.clone().unwrap_or_default(),
            }),
            BuildResult::BrowserBundle {
                output_dir,
                wasm_path,
                js_path,
                meta_path,
                wasm_bytes,
                metadata,
            } => json!({
                "artifactKind": "bundle",
                "outputPath": output_dir,
                "sizeBytes": wasm_bytes.len(),
                "buildMode": metadata.build_mode.clone(),
                "sourceHash": metadata.source_hash.clone(),
                "artifacts": [
                    { "kind": "wasm-module", "path": wasm_path },
                    { "kind": "js-glue", "path": js_path },
                    { "kind": "meta-json", "path": meta_path },
                ],
                "exports": metadata.exports.clone().unwrap_or_default(),
            }),
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
            BuildResult::BrowserBundle { output_dir, .. } => {
                format!("Built browser bundle at {}", output_dir.display())
            }
        }
    }
}

fn build_executable_artifact(
    file: &str,
    mode: build::BuildMode,
    out_dir: Option<&Path>,
    policy: Option<&SandboxPolicy>,
    api_surface: kali_cli::ApiSurface,
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let mut wasm_bytes = build::compile_source_file(&source, mode)?;
    let metadata = build::build_artifact_metadata(
        &source,
        "executable",
        mode,
        &api_surface.to_string(),
        None,
    )?;
    build::append_metadata_section(&mut wasm_bytes, &metadata)?;

    if let Some(policy) = policy {
        let policy_bytes = policy
            .to_canonical_json_bytes()
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
    out_dir: Option<&Path>,
    policy: Option<&SandboxPolicy>,
    api_surface: kali_cli::ApiSurface,
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let mut wasm_bytes = build::compile_source_file(&source, mode)?;
    let exports = build::collect_library_exports(&source)?;
    let metadata = build::build_artifact_metadata(
        &source,
        "lib",
        mode,
        &api_surface.to_string(),
        Some(exports),
    )?;
    build::append_metadata_section(&mut wasm_bytes, &metadata)?;

    if let Some(policy) = policy {
        let policy_bytes = policy
            .to_canonical_json_bytes()
            .map_err(|diagnostic| vec![diagnostic])?;
        CustomSection {
            name: std::borrow::Cow::Borrowed("kali:policy"),
            data: std::borrow::Cow::Owned(policy_bytes),
        }
        .append_to(&mut wasm_bytes);
    }

    let (output_path, meta_path) = build::library_output_paths_for(&source, out_dir);
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
        meta_path,
        wasm_bytes,
        metadata,
    })
}

fn build_browser_bundle_artifact(
    file: &str,
    mode: build::BuildMode,
    out_dir: Option<&Path>,
    policy: Option<&SandboxPolicy>,
    api_surface: kali_cli::ApiSurface,
) -> Result<BuildResult, Vec<Diagnostic>> {
    let source = PathBuf::from(file);
    let mut wasm_bytes = build::compile_source_file(&source, mode)?;
    let exports = build::collect_library_exports(&source).unwrap_or_default();
    let metadata = build::build_artifact_metadata(
        &source,
        "bundle",
        mode,
        &api_surface.to_string(),
        Some(exports.clone()),
    )?;
    build::append_metadata_section(&mut wasm_bytes, &metadata)?;

    if let Some(policy) = policy {
        let policy_bytes = policy
            .to_canonical_json_bytes()
            .map_err(|diagnostic| vec![diagnostic])?;
        CustomSection {
            name: std::borrow::Cow::Borrowed("kali:policy"),
            data: std::borrow::Cow::Owned(policy_bytes),
        }
        .append_to(&mut wasm_bytes);
    }

    let (wasm_path, js_path, meta_path) = build::bundle_output_paths_for(&source, out_dir);
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
    fs::write(&js_path, generate_browser_bundle_js(&wasm_path, &exports)).map_err(|error| {
        vec![Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write browser bundle JS '{}': {}",
                js_path.display(),
                error
            ),
        )]
    })?;

    Ok(BuildResult::BrowserBundle {
        output_dir,
        wasm_path,
        js_path,
        meta_path,
        wasm_bytes,
        metadata,
    })
}

fn generate_browser_bundle_js(wasm_path: &Path, exports: &[build::LibraryExport]) -> String {
    let wasm_file = wasm_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("bundle.wasm");
    let mut content = r#"const wasmUrl = new URL("./__WASM_FILE__", import.meta.url);

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
    .replace("__WASM_FILE__", wasm_file);
    for export in exports {
        content.push_str(&format!(
            "export async function {}(...args) {{\n  const {{ instance }} = await instancePromise;\n  return instance.exports.{}(...args);\n}}\n\n",
            export.name, export.name
        ));
    }
    content
}

fn run_command(
    files: Vec<String>,
    api: Option<kali_cli::ApiSurface>,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if matches!(
        api,
        Some(kali_cli::ApiSurface::Node | kali_cli::ApiSurface::Browser)
    ) {
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

    let wasm_bytes = match build::compile_source_file(&source, build::BuildMode::Fast) {
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

    let runtime = RuntimeCtx::new(policy.clone());
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
    if matches!(
        api,
        Some(kali_cli::ApiSurface::Node | kali_cli::ApiSurface::Browser)
    ) {
        let diagnostic = Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "selected API surface is unavailable in this phase",
        );
        return emit_diagnostics_and_exit("test", vec![diagnostic], 1, output, None, None);
    }

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

    let runtime = RuntimeCtx::new(policy.clone());
    let mut total = 0usize;
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut captured_stdout = String::new();
    let mut captured_stderr = String::new();
    let mut diagnostics = Vec::new();
    let start = Instant::now();

    for file in filtered_files {
        let source = PathBuf::from(&file);
        let wasm_bytes = match build::compile_source_file(&source, build::BuildMode::Fast) {
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
        if had_error {
            println!("Linted {} file(s)", processed);
        } else {
            println!("Linted {} file(s)", processed);
        }
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

    let diagnostic = Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        "`kali effects` is unavailable in this phase",
    );
    emit_diagnostics_and_exit(
        "effects",
        vec![diagnostic],
        1,
        output,
        Some(&source),
        fs::read_to_string(&source).ok().as_deref(),
    )
}

fn unavailable_registry_analysis_command(
    command: &str,
    _target: String,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let diagnostic = Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!("`kali {}` is unavailable in this phase", command),
    );
    emit_diagnostics_and_exit(command, vec![diagnostic], 1, output, None, None)
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
