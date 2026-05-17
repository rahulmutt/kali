#![allow(
    clippy::cloned_ref_to_slice_refs,
    clippy::match_like_matches_macro,
    clippy::needless_borrows_for_generic_args,
    clippy::question_mark,
    clippy::too_many_arguments
)]

use clap::Parser;
use kali_capi::{
    arity_from_signature, generate_binding_package_manifest_with_provenance, generate_header,
    generate_metadata_with_provenance as generate_capi_metadata, parse_binding_package_manifest,
    parse_metadata, Export as CApiExport,
};
use kali_cli::{
    build, discover_source_files, discover_test_files, init, is_declaration_only_source_file,
    output::{
        self, validate_check_payload_value, validate_doctor_payload_value,
        validate_effects_payload_value, validate_fmt_payload_value, validate_init_payload_value,
        validate_install_payload_value, validate_lint_payload_value,
        validate_package_audit_payload_value, validate_package_effects_payload_value,
        validate_run_payload_value, validate_test_payload_value, CliOutputOptions,
    },
    Args, BundleFormat, Commands,
};
use kali_error::{
    _error_codes::{e5, e6},
    set_verbose_diagnostics, Diagnostic, DiagnosticContext, DiagnosticContextOrigin,
};
use kali_fmt::format_source;
use kali_lint::lint_with_options;
use kali_npm::{
    audit_registry_package, discover_project_root, ensure_project_ready, install_project,
    load_manifest, resolve_materialized_import, InstallOptions, ProjectManifest,
};
use kali_optimize::ProfileData;
use kali_runtime::{
    browser_harness_command_parts_checked, browser_runtime_request_context,
    browser_runtime_unavailable_diagnostic, normalize_runtime_profiles, BrowserRuntimeContract,
    RuntimeBackend, RuntimeCtx, RuntimeHostContract, BROWSER_HARNESS_COMMAND_ENV,
};
use kali_sandbox::{
    compare_effects_to_policy, effect_report_from_inference, infer_effects_from_roots,
    package_effects_report, EffectAnalysisContext, PackageCoordinate, SandboxPolicy,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
    env, fs,
    path::{Component as PathComponent, Path, PathBuf},
    process::Command as ProcessCommand,
    time::Instant,
};
use wasm_encoder::{Component, ComponentSectionId, CustomSection, RawSection, Section};
use wasmparser::{Parser as WasmParser, Payload};

fn main() {
    let args = Args::parse();
    let output = CliOutputOptions {
        format: args.output,
        pretty: args.pretty,
        verbose: args.verbose,
        quiet: args.quiet,
        color: args.color,
    };
    set_verbose_diagnostics(output.verbose);

    let pretty_allowed_without_json = command_allows_pretty_without_json(args.command.as_ref());
    if output.pretty && !output.is_json() && !pretty_allowed_without_json {
        let diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            "`--pretty` is only meaningful when JSON output is active",
        );
        eprintln!("{}", diagnostic);
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
            compat,
            wasm_threads,
            fix,
            files,
        } => {
            if let Err(exit_code) =
                check_command(files, sandbox, api, compat, wasm_threads, fix, &output)
            {
                std::process::exit(exit_code);
            }
        }
        Commands::Build {
            sandbox,
            api,
            compat,
            profile,
            validate_ir,
            wasm_threads,
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
                compat,
                profile,
                validate_ir,
                wasm_threads,
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
            compat,
            wasm_threads,
            max_specializations,
            max_spawned_processes,
            max_threads,
            file,
            guest_args,
        } => {
            if let Err(exit_code) = run_command(
                file,
                guest_args,
                api,
                compat,
                wasm_threads,
                max_specializations,
                max_spawned_processes,
                max_threads,
                sandbox,
                &output,
            ) {
                std::process::exit(exit_code);
            }
        }
        Commands::Test {
            sandbox,
            api,
            compat,
            wasm_threads,
            max_specializations,
            max_spawned_processes,
            max_threads,
            files,
            filter,
            coverage,
        } => {
            if let Err(exit_code) = test_command(
                files,
                api,
                compat,
                wasm_threads,
                max_specializations,
                max_spawned_processes,
                max_threads,
                filter,
                coverage,
                sandbox,
                &output,
            ) {
                std::process::exit(exit_code);
            }
        }
        Commands::Doctor => {
            if let Err(exit_code) = doctor_command(&output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Init { lib, api, sandbox } => {
            if let Err(exit_code) = reject_workflow_context_flags("init", api, sandbox, &output) {
                std::process::exit(exit_code);
            }
            match init::init_current_directory(lib) {
                Ok(summary) => {
                    if output.is_json() {
                        let payload = json!({
                            "root": summary.root,
                            "manifestPath": summary.manifest_path,
                            "sourcePath": summary.source_path,
                            "library": summary.library,
                        });
                        validate_init_payload_value(&payload)
                            .expect("constructed init payload must satisfy schema-v1 shape");
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
                        let (errors, warnings) =
                            single_diagnostic_to_values(diagnostic, None, None);
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
            }
        }
        Commands::Install {
            target,
            dev,
            api,
            sandbox,
            allow_scripts,
        } => {
            if let Err(exit_code) =
                install_command(target, dev, api, sandbox, allow_scripts, &output)
            {
                std::process::exit(exit_code);
            }
        }
        Commands::Fmt {
            check,
            api,
            sandbox,
            files,
        } => {
            if let Err(exit_code) = reject_workflow_context_flags("fmt", api, sandbox, &output) {
                std::process::exit(exit_code);
            }
            if let Err(exit_code) = fmt_command(files, check, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Lint {
            fix,
            api,
            sandbox,
            files,
        } => {
            if let Err(exit_code) = reject_workflow_context_flags("lint", api, sandbox, &output) {
                std::process::exit(exit_code);
            }
            if let Err(exit_code) = lint_command(files, fix, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Effects {
            api,
            compat,
            wasm_threads,
            sandbox,
            files,
        } => {
            if let Err(exit_code) =
                effects_command(api, files, compat, wasm_threads, sandbox, &output)
            {
                std::process::exit(exit_code);
            }
        }
        Commands::PackageEffects {
            api,
            compat,
            wasm_threads,
            sandbox,
            target,
        } => {
            if let Err(exit_code) =
                package_effects_command(target, api, compat, wasm_threads, sandbox, &output)
            {
                std::process::exit(exit_code);
            }
        }
        Commands::PackageAudit {
            api,
            compat,
            wasm_threads,
            sandbox,
            target,
            preview,
        } => {
            if let Err(exit_code) =
                package_audit_command(target, preview, api, compat, wasm_threads, sandbox, &output)
            {
                std::process::exit(exit_code);
            }
        }
    }
}

fn doctor_command(output: &CliOutputOptions) -> Result<(), i32> {
    let override_value = env::var(BROWSER_HARNESS_COMMAND_ENV).ok();
    let source = if override_value.is_some() {
        "env"
    } else {
        "auto"
    };
    let command_parts = match browser_harness_command_parts_checked(override_value.as_deref()) {
        Ok(parts) => parts,
        Err(message) => {
            let diagnostic = Diagnostic::error(e5::INVALID_CLI_USAGE as u32, message);
            return emit_diagnostics_and_exit("doctor", vec![diagnostic], 5, output, None, None);
        }
    };
    let executable = command_parts.first().cloned().unwrap_or_default();
    let args: Vec<String> = command_parts.iter().skip(1).cloned().collect();
    let executable_available = !executable.is_empty()
        && ProcessCommand::new(&executable)
            .arg("--version")
            .output()
            .is_ok();
    let browser_runtime_contract = BrowserRuntimeContract::descriptor();
    let payload = json!({
        "browserHarness": {
            "envVar": BROWSER_HARNESS_COMMAND_ENV,
            "source": source,
            "override": override_value.clone(),
            "command": command_parts.clone(),
            "executable": executable,
            "args": args,
            "executableAvailable": executable_available,
        },
        "browserRuntimeContract": {
            "hostLabel": browser_runtime_contract.host_label,
            "hostDescription": browser_runtime_contract.host_description,
            "hostDescriptionNote": browser_runtime_contract.host_description_note,
            "supportedCommands": browser_runtime_contract.supported_commands,
            "diagnosticHint": browser_runtime_contract.diagnostic_hint,
            "diagnosticNotes": BrowserRuntimeContract::diagnostic_notes(),
        }
    });
    validate_doctor_payload_value(&payload)
        .expect("constructed doctor payload must satisfy schema-v1 shape");

    if output.is_json() {
        print_envelope(
            "doctor",
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
        let harness = &payload["browserHarness"];
        println!("Browser harness:");
        println!("  env var: {}", BROWSER_HARNESS_COMMAND_ENV);
        println!("  source: {}", harness["source"].as_str().unwrap_or(source));
        if let Some(value) = override_value.as_deref() {
            println!("  override: {value}");
        }
        println!("  command: {}", command_parts.join(" "));
        println!("  executable available: {}", executable_available);
        println!("Browser runtime contract:");
        println!("  host label: {}", browser_runtime_contract.host_label);
        println!(
            "  host description: {}",
            browser_runtime_contract.host_description
        );
        println!(
            "  supported commands: {}",
            browser_runtime_contract.supported_commands.join(", ")
        );
        println!(
            "  diagnostic hint: {}",
            browser_runtime_contract.diagnostic_hint
        );
        for note in BrowserRuntimeContract::diagnostic_notes() {
            println!("  note: {note}");
        }
    }

    Ok(())
}

fn check_command(
    files: Vec<String>,
    sandbox: Option<PathBuf>,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    wasm_threads: bool,
    fix: bool,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let effective_api = match resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("check", diagnostics, 5, output, None, None)
        }
    };

    if fix {
        let diagnostic = Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "kali check --fix is unavailable in this phase; use kali lint --fix for autofix"
                .to_string(),
        );
        return emit_diagnostics_and_exit("check", vec![diagnostic], 1, output, None, None);
    }

    ensure_project_ready_or_exit(output)?;
    let effective_compat = match resolve_effective_compat_features(compat) {
        Ok(features) => features,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("check", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) =
        reject_unavailable_compat_features("check", &effective_compat, output, None, None)
    {
        return Err(exit_code);
    }
    let effective_runtime_profiles = match resolve_effective_runtime_profiles(wasm_threads) {
        Ok(profiles) => profiles,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("check", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) = reject_unavailable_runtime_profiles(
        "check",
        &effective_runtime_profiles,
        !matches!(effective_api, kali_cli::ApiSurface::Browser),
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let policy = load_policy_or_exit(sandbox, &effective_runtime_profiles, output)?;
    let compat_eval = effective_compat.iter().any(|feature| feature == "eval");

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

    let mut checked = 0usize;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut successful_files = Vec::new();

    for file in selected_files {
        checked += 1;
        match build::check_source_file(
            &file,
            effective_api,
            &effective_runtime_profiles,
            compat_eval,
            policy.is_some(),
        ) {
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
        validate_check_payload_value(&payload)
            .expect("constructed check payload must satisfy schema-v1 shape");
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
    ensure_project_ready_or_exit(output)?;
    let effective_compat = match resolve_effective_compat_features(compat) {
        Ok(features) => features,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) =
        reject_unavailable_compat_features("build", &effective_compat, output, None, None)
    {
        return Err(exit_code);
    }
    let compat_eval = effective_compat.iter().any(|feature| feature == "eval");
    let profile_data = match resolve_profile_data(profile) {
        Ok(profile_data) => profile_data,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None)
        }
    };

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

    let effective_runtime_profiles = match resolve_effective_runtime_profiles(wasm_threads) {
        Ok(profiles) => profiles,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) = reject_unavailable_runtime_profiles(
        "build",
        &effective_runtime_profiles,
        !matches!(effective_api, kali_cli::ApiSurface::Browser),
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let policy = load_policy_or_exit(sandbox, &effective_runtime_profiles, output)?;

    let Some(source) = single_or_error(files, "build", output)? else {
        return Err(1);
    };

    let source = source.to_string_lossy().to_string();
    let mode = build::build_mode_from_flags(fast, release, release_advanced);
    let max_specializations = match resolve_effective_max_specializations(max_specializations) {
        Ok(max_specializations) => max_specializations,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None)
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

fn config_diagnostic_context(config_path: &str) -> DiagnosticContext {
    DiagnosticContext::new(DiagnosticContextOrigin::Config).with_config_path(config_path)
}

fn config_diagnostic_context_with_value(
    config_path: &str,
    value: impl Into<String>,
) -> DiagnosticContext {
    config_diagnostic_context(config_path).with_effective_value(value)
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
        )
        .with_context(config_diagnostic_context("compilerOptions"))]);
    };

    let Some(value) = options.get("apiSurface") else {
        return Ok(None);
    };

    let Some(api_surface) = value.as_str() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions.apiSurface` must be a string",
        )
        .with_context(config_diagnostic_context("compilerOptions.apiSurface"))]);
    };

    match api_surface {
        "deno" => Ok(Some(kali_cli::ApiSurface::Deno)),
        "node" => Ok(Some(kali_cli::ApiSurface::Node)),
        "browser" => Ok(Some(kali_cli::ApiSurface::Browser)),
        _ => Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            format!("unsupported apiSurface '{}' in kali.json", api_surface),
        )
        .with_context(config_diagnostic_context_with_value(
            "compilerOptions.apiSurface",
            api_surface,
        ))]),
    }
}

fn resolve_effective_compat_features(
    explicit_compat: Vec<String>,
) -> Result<Vec<String>, Vec<Diagnostic>> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let Some(manifest) = load_manifest(&project_root).map_err(|diagnostic| vec![diagnostic])?
    else {
        return Ok(normalize_compat_features(explicit_compat));
    };

    let mut features = normalize_compat_features(explicit_compat);
    features.extend(manifest_compat_features(&manifest)?);
    Ok(normalize_compat_features(features))
}

fn resolve_effective_runtime_profiles(
    explicit_wasm_threads: bool,
) -> Result<Vec<String>, Vec<Diagnostic>> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let Some(manifest) = load_manifest(&project_root).map_err(|diagnostic| vec![diagnostic])?
    else {
        return Ok(normalize_runtime_profiles(if explicit_wasm_threads {
            vec!["wasm-threads".to_string()]
        } else {
            Vec::new()
        }));
    };

    let mut profiles = normalize_runtime_profiles(if explicit_wasm_threads {
        vec!["wasm-threads".to_string()]
    } else {
        Vec::new()
    });
    profiles.extend(manifest_runtime_profiles(&manifest)?);
    Ok(normalize_runtime_profiles(profiles))
}

fn resolve_effective_max_specializations(
    explicit_max_specializations: Option<usize>,
) -> Result<usize, Vec<Diagnostic>> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let Some(manifest) = load_manifest(&project_root).map_err(|diagnostic| vec![diagnostic])?
    else {
        return Ok(explicit_max_specializations.unwrap_or(16));
    };

    let manifest_max_specializations = manifest_max_specializations(&manifest)?;
    Ok(explicit_max_specializations
        .or(manifest_max_specializations)
        .unwrap_or(16))
}

fn resolve_profile_data(profile: Option<PathBuf>) -> Result<Option<ProfileData>, Vec<Diagnostic>> {
    match profile {
        Some(profile_path) => build::load_profile_data_file(profile_path).map(Some),
        None => Ok(None),
    }
}

fn manifest_compat_features(manifest: &ProjectManifest) -> Result<Vec<String>, Vec<Diagnostic>> {
    let Some(compat) = manifest.compat.as_ref() else {
        return Ok(Vec::new());
    };

    let Some(compat) = compat.as_object() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compat` must be a JSON object",
        )
        .with_context(config_diagnostic_context("compat"))]);
    };

    let Some(features) = compat.get("features") else {
        return Ok(Vec::new());
    };

    let Some(features) = features.as_array() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compat.features` must be an array of strings",
        )
        .with_context(config_diagnostic_context("compat.features"))]);
    };

    let mut normalized = BTreeSet::new();
    for feature in features {
        let Some(feature) = feature.as_str() else {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CONFIG as u32,
                "`compat.features` entries must be strings",
            )
            .with_context(config_diagnostic_context("compat.features"))]);
        };

        let feature = feature.trim();
        if feature.is_empty() {
            continue;
        }

        if !matches!(feature, "eval") {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CONFIG as u32,
                format!("unsupported compat.feature '{}' in kali.json", feature),
            )
            .with_context(config_diagnostic_context_with_value(
                "compat.features",
                feature,
            ))]);
        }

        if !normalized.insert(feature.to_string()) {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CONFIG as u32,
                format!("duplicate compat.feature '{}' in kali.json", feature),
            )
            .with_context(config_diagnostic_context_with_value(
                "compat.features",
                feature,
            ))]);
        }
    }

    Ok(normalized.into_iter().collect())
}

fn manifest_runtime_profiles(manifest: &ProjectManifest) -> Result<Vec<String>, Vec<Diagnostic>> {
    let Some(options) = manifest.compiler_options.as_ref() else {
        return Ok(Vec::new());
    };

    let Some(options) = options.as_object() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions` must be a JSON object",
        )
        .with_context(config_diagnostic_context("compilerOptions"))]);
    };

    let Some(profiles) = options.get("runtimeProfiles") else {
        return Ok(Vec::new());
    };

    let Some(profiles) = profiles.as_array() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions.runtimeProfiles` must be an array of strings",
        )
        .with_context(config_diagnostic_context(
            "compilerOptions.runtimeProfiles",
        ))]);
    };

    let mut raw_profiles = Vec::new();
    for profile in profiles {
        let Some(profile) = profile.as_str() else {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CONFIG as u32,
                "`compilerOptions.runtimeProfiles` entries must be strings",
            )
            .with_context(config_diagnostic_context(
                "compilerOptions.runtimeProfiles",
            ))]);
        };
        raw_profiles.push(profile.to_string());
    }

    build::validate_runtime_profiles(&raw_profiles, "kali.json").map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| {
                diagnostic
                    .with_context(config_diagnostic_context("compilerOptions.runtimeProfiles"))
            })
            .collect()
    })
}

fn manifest_max_specializations(
    manifest: &ProjectManifest,
) -> Result<Option<usize>, Vec<Diagnostic>> {
    let Some(options) = manifest.compiler_options.as_ref() else {
        return Ok(None);
    };

    let Some(options) = options.as_object() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions` must be a JSON object",
        )]);
    };

    let Some(value) = options.get("maxSpecializations") else {
        return Ok(None);
    };

    let Some(max_specializations) = value.as_u64() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions.maxSpecializations` must be a non-negative integer",
        )]);
    };

    let max_specializations = usize::try_from(max_specializations).map_err(|_| {
        vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions.maxSpecializations` is too large for this host",
        )]
    })?;

    Ok(Some(max_specializations))
}

fn normalize_compat_features(features: Vec<String>) -> Vec<String> {
    let mut normalized = BTreeSet::new();
    for feature in features {
        let feature = feature.trim();
        if !feature.is_empty() {
            normalized.insert(feature.to_string());
        }
    }
    normalized.into_iter().collect()
}

fn reject_unavailable_compat_features(
    command: &str,
    compat_features: &[String],
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_contents: Option<&str>,
) -> Result<(), i32> {
    let unavailable: Vec<String> = compat_features
        .iter()
        .filter(|feature| feature.as_str() != "eval")
        .cloned()
        .collect();

    if unavailable.is_empty() {
        return Ok(());
    }

    let diagnostic = Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "selected compatibility feature(s) {:?} are unavailable in this phase",
            unavailable
        ),
    )
    .note("canonical CLI flag: --compat")
    .note("canonical config path: compat.features");
    emit_diagnostics_and_exit(
        command,
        vec![diagnostic],
        5,
        output,
        source_path,
        source_contents,
    )
}

fn reject_unavailable_runtime_profiles(
    command: &str,
    runtime_profiles: &[String],
    allow_threaded_profile: bool,
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_contents: Option<&str>,
) -> Result<(), i32> {
    let unavailable: Vec<String> = runtime_profiles
        .iter()
        .filter(|profile| profile.as_str() == "wasm-threads" && !allow_threaded_profile)
        .cloned()
        .collect();

    if unavailable.is_empty() {
        return Ok(());
    }

    let diagnostic = Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "selected runtime profile(s) {:?} are unavailable in this phase",
            unavailable
        ),
    )
    .note("canonical CLI flag: --wasm-threads")
    .note("canonical config path: compilerOptions.runtimeProfiles");
    emit_diagnostics_and_exit(
        command,
        vec![diagnostic],
        5,
        output,
        source_path,
        source_contents,
    )
}

fn browser_runtime_harness_command_available() -> bool {
    std::env::var_os(BROWSER_HARNESS_COMMAND_ENV).is_some()
}

fn reject_unavailable_browser_runtime(
    command: &str,
    api_surface: kali_cli::ApiSurface,
    browser_runtime_available: bool,
    browser_context: Option<DiagnosticContext>,
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_contents: Option<&str>,
) -> Result<(), i32> {
    if !matches!(api_surface, kali_cli::ApiSurface::Browser) || browser_runtime_available {
        return Ok(());
    }

    let diagnostic = browser_runtime_unavailable_diagnostic(Some(command), browser_context);
    emit_diagnostics_and_exit(
        command,
        vec![diagnostic],
        1,
        output,
        source_path,
        source_contents,
    )
}

fn reject_unavailable_spawned_process_budget(
    command: &str,
    max_spawned_processes: Option<u64>,
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_contents: Option<&str>,
) -> Result<(), i32> {
    if max_spawned_processes.is_none_or(|count| count == 0) {
        return Ok(());
    }

    let mut diagnostic = Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        "selected resource budget(s) [\"resources.maxSpawnedProcesses\"] are unavailable in this phase",
    )
    .note("canonical CLI flag: --max-spawned-processes")
    .note("canonical config path: resources.maxSpawnedProcesses");

    if let Some(count) = max_spawned_processes {
        diagnostic = diagnostic.with_context(
            DiagnosticContext::new(DiagnosticContextOrigin::Cli)
                .with_flag("--max-spawned-processes")
                .with_requested_value(count.to_string())
                .with_effective_value(count.to_string()),
        );
    }

    emit_diagnostics_and_exit(
        command,
        vec![diagnostic],
        5,
        output,
        source_path,
        source_contents,
    )
}

fn reject_unavailable_zero_capable_budgets(
    command: &str,
    runtime_profiles: &[String],
    max_threads: Option<u64>,
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_contents: Option<&str>,
) -> Result<(), i32> {
    let mut unavailable = Vec::new();
    let has_threaded_profile = runtime_profiles
        .iter()
        .any(|profile| profile.as_str() == "wasm-threads");
    if max_threads.is_some_and(|count| count > 0) && !has_threaded_profile {
        unavailable.push("resources.maxThreads");
    }

    if unavailable.is_empty() {
        return Ok(());
    }

    let mut diagnostic = Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "selected resource budget(s) {:?} are unavailable in this phase",
            unavailable
        ),
    )
    .note("canonical CLI flag: --max-threads")
    .note("canonical config path: resources.maxThreads")
    .note("threaded runtime profile config path: compilerOptions.runtimeProfiles");

    if let Some(count) = max_threads {
        diagnostic = diagnostic.with_context(
            DiagnosticContext::new(DiagnosticContextOrigin::Cli)
                .with_flag("--max-threads")
                .with_requested_value(count.to_string())
                .with_effective_value(count.to_string()),
        );
    }
    emit_diagnostics_and_exit(
        command,
        vec![diagnostic],
        5,
        output,
        source_path,
        source_contents,
    )
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

fn run_command(
    file: String,
    guest_args: Vec<String>,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    wasm_threads: bool,
    max_specializations: Option<usize>,
    max_spawned_processes: Option<u64>,
    max_threads: Option<u64>,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let effective_api = match resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("run", diagnostics, 5, output, None, None)
        }
    };

    let browser_context = if matches!(effective_api, kali_cli::ApiSurface::Browser) {
        let origin = if api.is_some() {
            DiagnosticContextOrigin::Cli
        } else {
            DiagnosticContextOrigin::Config
        };
        let context = browser_runtime_request_context(origin);
        Some(if api.is_some() {
            context.with_flag("--api")
        } else {
            context.with_config_path("compilerOptions.apiSurface")
        })
    } else {
        None
    };

    // The opt-in browser harness can execute standalone browser-requested programs,
    // but it is not a Kali-hosted runtime sandbox. Keep `run --api browser --sandbox`
    // on the browser-runtime availability gate instead of silently dropping policy
    // enforcement when a harness override is present.
    let browser_runtime_available =
        browser_runtime_harness_command_available() && sandbox.is_none();
    if let Err(exit_code) = reject_unavailable_browser_runtime(
        "run",
        effective_api,
        browser_runtime_available,
        browser_context,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }

    ensure_project_ready_or_exit(output)?;
    let effective_compat = match resolve_effective_compat_features(compat) {
        Ok(features) => features,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("run", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) =
        reject_unavailable_compat_features("run", &effective_compat, output, None, None)
    {
        return Err(exit_code);
    }
    let effective_runtime_profiles = match resolve_effective_runtime_profiles(wasm_threads) {
        Ok(profiles) => profiles,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("run", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) = reject_unavailable_runtime_profiles(
        "run",
        &effective_runtime_profiles,
        true,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let policy = load_policy_or_exit(sandbox, &effective_runtime_profiles, output)?;
    if let Err(exit_code) =
        reject_unavailable_spawned_process_budget("run", max_spawned_processes, output, None, None)
    {
        return Err(exit_code);
    }
    if let Err(exit_code) = reject_unavailable_zero_capable_budgets(
        "run",
        &effective_runtime_profiles,
        max_threads,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let max_specializations = match resolve_effective_max_specializations(max_specializations) {
        Ok(max_specializations) => max_specializations,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("run", diagnostics, 5, output, None, None)
        }
    };
    let compat_eval = effective_compat.iter().any(|feature| feature == "eval");
    let source = PathBuf::from(file);

    if let Some(policy) = policy.as_ref() {
        if let Err(diagnostics) =
            validate_source_effects_against_policy(&source, policy, effective_api)
        {
            return emit_diagnostics_and_exit(
                "run",
                diagnostics,
                5,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
    }

    if let Err(diagnostic) = validate_runtime_entrypoint(&source, effective_api) {
        return emit_diagnostics_and_exit("run", vec![diagnostic], 5, output, None, None);
    }
    if let Err(diagnostics) =
        build::reject_async_and_generator_class_methods_in_runtime_entrypoint(&source)
    {
        return emit_diagnostics_and_exit(
            "run",
            diagnostics,
            1,
            output,
            Some(&source),
            fs::read_to_string(&source).ok().as_deref(),
        );
    }

    let wasm_bytes = match build::compile_source_file_with_specialization_cap_and_validation(
        &source,
        build::BuildMode::Fast,
        max_specializations,
        effective_api,
        &effective_runtime_profiles,
        compat_eval,
        false,
        false,
    ) {
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

    let runtime_args = if effective_api == kali_cli::ApiSurface::Node {
        let mut argv = vec!["node".to_string(), source.display().to_string()];
        let mut guest_args = guest_args;
        if guest_args.first().is_some_and(|arg| arg == "--") {
            guest_args.remove(0);
        }
        argv.extend(guest_args);
        argv
    } else {
        guest_args
    };
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        policy.clone(),
        runtime_args,
        env::vars().collect::<BTreeMap<_, _>>(),
        cwd,
        effective_api.to_string(),
    )
    .with_runtime_profiles(effective_runtime_profiles.clone())
    .with_max_threads(max_threads)
    .with_max_spawned_processes(max_spawned_processes);
    let start = Instant::now();
    match runtime.execute(&wasm_bytes) {
        Ok(outcome) => {
            if output.is_json() {
                let payload = json!({
                    "exitCode": outcome.exit_code,
                    "runtimeMs": start.elapsed().as_millis(),
                    "hostContract": runtime.host_contract().canonical_label(),
                    "runtimeBackend": runtime.runtime_backend().canonical_label(),
                    "threadTopology": outcome.thread_topology.thread_topology_snapshot_value(),
                });
                validate_run_payload_value(&payload)
                    .expect("constructed run payload must satisfy schema-v1 shape");
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
    compat: Vec<String>,
    wasm_threads: bool,
    max_specializations: Option<usize>,
    max_spawned_processes: Option<u64>,
    max_threads: Option<u64>,
    filter: Option<String>,
    coverage: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let effective_api = match resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("test", diagnostics, 5, output, None, None)
        }
    };

    let browser_context = if matches!(effective_api, kali_cli::ApiSurface::Browser) {
        let origin = if api.is_some() {
            DiagnosticContextOrigin::Cli
        } else {
            DiagnosticContextOrigin::Config
        };
        let context = browser_runtime_request_context(origin);
        Some(if api.is_some() {
            context.with_flag("--api")
        } else {
            context.with_config_path("compilerOptions.apiSurface")
        })
    } else {
        None
    };

    // The opt-in browser harness can execute standalone browser-requested tests,
    // but it is not a Kali-hosted runtime sandbox. Keep `test --api browser --sandbox`
    // on the browser-runtime availability gate instead of silently dropping policy
    // enforcement when a harness override is present.
    let browser_runtime_available =
        browser_runtime_harness_command_available() && sandbox.is_none();
    if let Err(exit_code) = reject_unavailable_browser_runtime(
        "test",
        effective_api,
        browser_runtime_available,
        browser_context,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }

    ensure_project_ready_or_exit(output)?;
    let effective_compat = match resolve_effective_compat_features(compat) {
        Ok(features) => features,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("test", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) =
        reject_unavailable_compat_features("test", &effective_compat, output, None, None)
    {
        return Err(exit_code);
    }
    let effective_runtime_profiles = match resolve_effective_runtime_profiles(wasm_threads) {
        Ok(profiles) => profiles,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("test", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) = reject_unavailable_runtime_profiles(
        "test",
        &effective_runtime_profiles,
        true,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let policy = load_policy_or_exit(sandbox, &effective_runtime_profiles, output)?;
    if let Err(exit_code) =
        reject_unavailable_spawned_process_budget("test", max_spawned_processes, output, None, None)
    {
        return Err(exit_code);
    }
    if let Err(exit_code) = reject_unavailable_zero_capable_budgets(
        "test",
        &effective_runtime_profiles,
        max_threads,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let max_specializations = match resolve_effective_max_specializations(max_specializations) {
        Ok(max_specializations) => max_specializations,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit("test", diagnostics, 5, output, None, None)
        }
    };
    let compat_eval = effective_compat.iter().any(|feature| feature == "eval");

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
        if let Err(diagnostic) = validate_runtime_entrypoint(&source, effective_api) {
            return emit_diagnostics_and_exit(
                "test",
                vec![diagnostic],
                5,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
        if let Err(diagnostics) =
            build::reject_async_and_generator_class_methods_in_runtime_entrypoint(&source)
        {
            return emit_diagnostics_and_exit(
                "test",
                diagnostics,
                1,
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
            let payload = if coverage {
                json!({
                    "total": 0,
                    "passed": 0,
                    "failed": 0,
                    "skipped": 0,
                    "runtimeMs": 0,
                    "coverage": {
                        "mode": "function",
                        "files": [],
                        "summary": {
                            "functionsTotal": 0,
                            "functionsCovered": 0,
                            "functionsMissed": 0,
                            "coveragePercent": 100.0,
                        },
                    },
                })
            } else {
                json!({ "total": 0, "passed": 0, "failed": 0, "skipped": 0, "runtimeMs": 0 })
            };
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

    if let Some(policy) = policy.as_ref() {
        let roots = filtered_files.iter().map(PathBuf::from).collect::<Vec<_>>();
        if let Err(diagnostics) =
            validate_source_effects_against_policy_for_roots(&roots, policy, effective_api)
        {
            return emit_diagnostics_and_exit("test", diagnostics, 5, output, None, None);
        }
    }

    let runtime = RuntimeCtx::with_api_surface(policy.clone(), effective_api.to_string())
        .with_runtime_profiles(effective_runtime_profiles.clone())
        .with_max_threads(max_threads)
        .with_max_spawned_processes(max_spawned_processes);
    let mut total = 0usize;
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut captured_stdout = String::new();
    let mut captured_stderr = String::new();
    let mut thread_topology = json!({
        "totalInstances": 0,
        "terminatedInstances": 0,
        "liveInstances": [],
    });
    let mut diagnostics = Vec::new();
    let mut coverage_reports = Vec::new();
    let project_root =
        discover_project_root(&env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
    let start = Instant::now();

    for file in filtered_files {
        let source = PathBuf::from(&file);
        let wasm_bytes = match build::compile_source_file_with_specialization_cap_and_validation(
            &source,
            build::BuildMode::Fast,
            max_specializations,
            effective_api,
            &effective_runtime_profiles,
            compat_eval,
            false,
            coverage,
        ) {
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

        let coverage_total = if coverage {
            coverage_function_count_from_wasm(&wasm_bytes).unwrap_or(0)
        } else {
            0
        };

        match runtime.execute_tests(&wasm_bytes) {
            Ok(outcome) => {
                total += outcome.tests_run;
                passed += outcome.tests_run.saturating_sub(outcome.tests_failed);
                failed += outcome.tests_failed;
                captured_stdout.push_str(&outcome.stdout);
                captured_stderr.push_str(&outcome.stderr);
                let outcome_thread_topology =
                    outcome.thread_topology.thread_topology_snapshot_value();
                output::merge_thread_topology_snapshot_values(
                    &mut thread_topology,
                    &outcome_thread_topology,
                );
                if coverage {
                    let covered = outcome.coverage_hits.len().min(coverage_total);
                    coverage_reports.push(json!({
                        "file": normalize_coverage_report_path(&file, &project_root),
                        "functionsTotal": coverage_total,
                        "functionsCovered": covered,
                        "functionsMissed": coverage_total.saturating_sub(covered),
                    }));
                }
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

    if coverage {
        sort_coverage_reports(&mut coverage_reports);
    }

    if output.is_json() {
        let payload = if coverage {
            let functions_total = coverage_reports
                .iter()
                .map(|report| report["functionsTotal"].as_u64().unwrap_or(0) as usize)
                .sum::<usize>();
            let functions_covered = coverage_reports
                .iter()
                .map(|report| report["functionsCovered"].as_u64().unwrap_or(0) as usize)
                .sum::<usize>();
            let functions_missed = functions_total.saturating_sub(functions_covered);
            json!({
                "total": total,
                "passed": passed,
                "failed": failed,
                "skipped": 0,
                "runtimeMs": start.elapsed().as_millis(),
                "hostContract": runtime.host_contract().canonical_label(),
                "runtimeBackend": runtime.runtime_backend().canonical_label(),
                "threadTopology": thread_topology,
                "coverage": {
                    "mode": "function",
                    "files": coverage_reports,
                    "summary": {
                        "functionsTotal": functions_total,
                        "functionsCovered": functions_covered,
                        "functionsMissed": functions_missed,
                        "coveragePercent": coverage_percent(functions_covered, functions_total),
                    },
                },
            })
        } else {
            json!({
                "total": total,
                "passed": passed,
                "failed": failed,
                "skipped": 0,
                "runtimeMs": start.elapsed().as_millis(),
                "hostContract": runtime.host_contract().canonical_label(),
                "runtimeBackend": runtime.runtime_backend().canonical_label(),
                "threadTopology": thread_topology,
            })
        };
        let success = diagnostics.is_empty();
        let (errors, warnings) = split_and_convert_diagnostics(&diagnostics, None, None);
        validate_test_payload_value(&payload)
            .expect("constructed test payload must satisfy schema-v1 shape");
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
            if coverage {
                let functions_total = coverage_reports
                    .iter()
                    .map(|report| report["functionsTotal"].as_u64().unwrap_or(0) as usize)
                    .sum::<usize>();
                let functions_covered = coverage_reports
                    .iter()
                    .map(|report| report["functionsCovered"].as_u64().unwrap_or(0) as usize)
                    .sum::<usize>();
                println!(
                    "ok {} (coverage: {}/{} functions, {:.1}%)",
                    passed,
                    functions_covered,
                    functions_total,
                    coverage_percent(functions_covered, functions_total)
                );
            } else {
                println!("ok {}", passed);
            }
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

fn coverage_function_count_from_wasm(bytes: &[u8]) -> Option<usize> {
    for payload in WasmParser::new(0).parse_all(bytes) {
        if let Ok(Payload::CustomSection(section)) = payload {
            if section.name() == "kali:coverage" && section.data().len() >= 4 {
                let mut raw = [0u8; 4];
                raw.copy_from_slice(&section.data()[..4]);
                return Some(u32::from_le_bytes(raw) as usize);
            }
        }
    }
    None
}

fn normalize_coverage_report_path(file: &str, project_root: &Path) -> String {
    let source = PathBuf::from(file);
    let candidate = if source.is_absolute() {
        source
    } else {
        env::current_dir()
            .ok()
            .map(|cwd| cwd.join(&source))
            .unwrap_or(source)
    };
    let canonical = fs::canonicalize(&candidate).unwrap_or(candidate);
    canonical
        .strip_prefix(project_root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| canonical.to_string_lossy().replace('\\', "/"))
}

fn sort_coverage_reports(reports: &mut [Value]) {
    reports.sort_by(|left, right| {
        let left_file = left.get("file").and_then(Value::as_str).unwrap_or("");
        let right_file = right.get("file").and_then(Value::as_str).unwrap_or("");
        left_file.cmp(right_file)
    });
}

fn coverage_percent(covered: usize, total: usize) -> f64 {
    if total == 0 {
        100.0
    } else {
        (covered as f64 / total as f64) * 100.0
    }
}

fn fmt_command(files: Vec<String>, check: bool, output: &CliOutputOptions) -> Result<(), i32> {
    ensure_project_ready_or_exit(output)?;
    let selected_files = selected_source_files(files, true);
    if selected_files.is_empty() {
        if output.is_json() {
            let payload = json!({"filesFormatted": 0, "filesChecked": 0});
            validate_fmt_payload_value(&payload)
                .expect("constructed fmt payload must satisfy schema-v1 shape");
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
        validate_fmt_payload_value(&payload)
            .expect("constructed fmt payload must satisfy schema-v1 shape");
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
            validate_lint_payload_value(&payload)
                .expect("constructed lint payload must satisfy schema-v1 shape");
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
        validate_lint_payload_value(&payload)
            .expect("constructed lint payload must satisfy schema-v1 shape");
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

fn effects_command(
    api: Option<kali_cli::ApiSurface>,
    files: Vec<String>,
    compat: Vec<String>,
    wasm_threads: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if sandbox.is_some() {
        let diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            "`effects` does not accept `--sandbox`; use `check` or `build --sandbox` for policy validation"
                .to_string(),
        );
        return emit_diagnostics_and_exit("effects", vec![diagnostic], 5, output, None, None);
    }

    let Some(source) = single_or_error(files, "effects", output)? else {
        return Err(1);
    };

    let effective_api = match resolve_effective_api_surface(api) {
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
    if let Err(diagnostic) = validate_runtime_entrypoint(&source, effective_api) {
        return emit_diagnostics_and_exit(
            "effects",
            vec![diagnostic],
            5,
            output,
            Some(&source),
            fs::read_to_string(&source).ok().as_deref(),
        );
    }
    let effective_compat = match resolve_effective_compat_features(compat) {
        Ok(features) => features,
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
    if let Err(exit_code) = reject_unavailable_compat_features(
        "effects",
        &effective_compat,
        output,
        Some(&source),
        fs::read_to_string(&source).ok().as_deref(),
    ) {
        return Err(exit_code);
    }
    let effective_runtime_profiles = match resolve_effective_runtime_profiles(wasm_threads) {
        Ok(profiles) => profiles,
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
    if let Err(exit_code) = reject_unavailable_runtime_profiles(
        "effects",
        &effective_runtime_profiles,
        !matches!(effective_api, kali_cli::ApiSurface::Browser),
        output,
        Some(&source),
        fs::read_to_string(&source).ok().as_deref(),
    ) {
        return Err(exit_code);
    }
    let context = analysis_context_for_api(
        effective_api,
        effective_runtime_profiles,
        effective_compat.clone(),
    );
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

fn reject_package_analysis_specific_flags(
    command: &str,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    wasm_threads: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if api.is_some() || !compat.is_empty() || wasm_threads || sandbox.is_some() {
        let diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            format!(
                "`{}` does not accept package-analysis-specific flags like `--api`, `--compat`, `--wasm-threads`, or `--sandbox`; use inherited project config instead",
                command
            ),
        );
        return emit_diagnostics_and_exit(command, vec![diagnostic], 5, output, None, None);
    }

    Ok(())
}

fn require_single_registry_package_target(
    command: &str,
    targets: Vec<String>,
    output: &CliOutputOptions,
) -> Result<String, i32> {
    let (message, exit_code) = match targets.as_slice() {
        [target] if target.trim().is_empty() => (
            format!("`{}` requires a non-empty package argument", command),
            5,
        ),
        [target] if target.trim() != target => (
            format!(
                "`{}` requires a package argument without leading or trailing whitespace",
                command
            ),
            5,
        ),
        [target] => return Ok(target.clone()),
        [] => (
            format!("`{}` requires exactly one package argument", command),
            5,
        ),
        _ => (
            format!("`{}` accepts exactly one package argument", command),
            5,
        ),
    };

    let diagnostic = Diagnostic::error(e5::INVALID_CLI_USAGE as u32, message);
    if output.is_json() {
        print_envelope(
            command,
            false,
            vec![output::diagnostic_to_json(&diagnostic, None, None, "error")],
            vec![],
            Value::Null,
            None,
            None,
            exit_code,
            output,
        );
    } else {
        eprintln!("{}", diagnostic);
    }
    Err(exit_code)
}

fn package_effects_command(
    target: Vec<String>,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    wasm_threads: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let target = require_single_registry_package_target("package-effects", target, output)?;
    reject_package_analysis_specific_flags(
        "package-effects",
        api,
        compat,
        wasm_threads,
        sandbox,
        output,
    )?;

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
    let entry_path = match resolve_materialized_import(&project_root, &parsed.install_name) {
        Some(path) => path,
        None => {
            let diagnostic = Diagnostic::error(
                e6::DEPENDENCY_STATE_MISSING as u32,
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
                e6::DEPENDENCY_STATE_MISSING as u32,
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
    let effective_runtime_profiles = match resolve_effective_runtime_profiles(false) {
        Ok(profiles) => profiles,
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
    // Registry-analysis inherits semantic analysis context, including the threaded profile.
    // The command still rejects explicit package-analysis flags, but inherited runtime profile
    // state is part of the stable report contract.
    if let Err(exit_code) = reject_unavailable_runtime_profiles(
        "package-effects",
        &effective_runtime_profiles,
        true,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let effective_compat = match resolve_effective_compat_features(Vec::new()) {
        Ok(features) => features,
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
    if let Err(exit_code) =
        reject_unavailable_compat_features("package-effects", &effective_compat, output, None, None)
    {
        return Err(exit_code);
    }
    let context =
        analysis_context_for_api(effective_api, effective_runtime_profiles, effective_compat);
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

fn sort_package_audit_findings(findings: &mut [Diagnostic]) {
    findings.sort_by(|left, right| {
        let left_rank = match left.severity {
            kali_error::Severity::Error => 0u8,
            kali_error::Severity::Warning => 1,
            kali_error::Severity::Info => 2,
        };
        let right_rank = match right.severity {
            kali_error::Severity::Error => 0u8,
            kali_error::Severity::Warning => 1,
            kali_error::Severity::Info => 2,
        };

        left_rank
            .cmp(&right_rank)
            .then_with(|| {
                left.code
                    .unwrap_or(u32::MAX)
                    .cmp(&right.code.unwrap_or(u32::MAX))
            })
            .then_with(|| left.message.cmp(&right.message))
            .then_with(|| left.notes.cmp(&right.notes))
            .then_with(|| left.suggestion.cmp(&right.suggestion))
            .then_with(|| diagnostic_span_sort_key(left).cmp(&diagnostic_span_sort_key(right)))
    });
}

fn diagnostic_span_sort_key(diagnostic: &Diagnostic) -> (u32, u32, u32, bool) {
    let Some(span) = diagnostic.span else {
        return (u32::MAX, u32::MAX, u32::MAX, true);
    };

    (span.file_id.as_u32(), span.start, span.end, false)
}

fn command_allows_pretty_without_json(command: Option<&Commands>) -> bool {
    matches!(
        command,
        Some(Commands::Effects { .. }) | Some(Commands::PackageEffects { .. })
    )
}

fn package_audit_preview_diagnostic() -> Diagnostic {
    Diagnostic::error(
        e5::INVALID_CLI_USAGE as u32,
        "legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape",
    )
    .with_context(
        DiagnosticContext::new(DiagnosticContextOrigin::Cli)
            .with_flag("--preview")
            .with_requested_value("true")
            .with_effective_value("true"),
    )
}

fn package_audit_command(
    target: Vec<String>,
    preview: bool,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    wasm_threads: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if preview {
        let diagnostic = package_audit_preview_diagnostic();
        return emit_diagnostics_and_exit("package-audit", vec![diagnostic], 5, output, None, None);
    }

    let target = require_single_registry_package_target("package-audit", target, output)?;

    reject_package_analysis_specific_flags(
        "package-audit",
        api,
        compat,
        wasm_threads,
        sandbox,
        output,
    )?;

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

    let audit = match audit_registry_package(&parsed.registry, &parsed.package_name) {
        Ok(audit) => audit,
        Err(diagnostic) => {
            return emit_diagnostics_and_exit(
                "package-audit",
                vec![diagnostic],
                1,
                output,
                None,
                None,
            );
        }
    };

    let kali_npm::RegistryPackageAudit {
        registry,
        name,
        version,
        mut findings,
    } = audit;
    sort_package_audit_findings(&mut findings);
    let has_errors = findings.iter().any(|diagnostic| diagnostic.is_error());
    let summary = if findings.is_empty() {
        format!(
            "Package audit completed for {} package '{}@{}'; no security findings were computed.",
            registry, name, version
        )
    } else {
        let error_count = findings
            .iter()
            .filter(|diagnostic| diagnostic.is_error())
            .count();
        let warning_count = findings.len() - error_count;
        format!(
            "Package audit completed for {} package '{}@{}'; {} error(s), {} warning(s).",
            registry, name, version, error_count, warning_count
        )
    };

    if output.is_json() {
        let (errors, warnings) = split_and_convert_diagnostics(&findings, None, None);
        print_envelope(
            "package-audit",
            errors.is_empty(),
            errors,
            warnings,
            Value::Null,
            Some(summary),
            None,
            if has_errors { 1 } else { 0 },
            output,
        );
    } else if !output.quiet {
        println!("{summary}");
        for diagnostic in &findings {
            eprintln!("{}", diagnostic);
        }
    }

    if has_errors {
        Err(1)
    } else {
        Ok(())
    }
}

fn reject_workflow_context_flags(
    command: &str,
    api: Option<kali_cli::ApiSurface>,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if api.is_some() || sandbox.is_some() {
        let diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            format!(
                "`{}` does not accept `--api` or `--sandbox` in early phases",
                command
            ),
        );
        return emit_diagnostics_and_exit(command, vec![diagnostic], 5, output, None, None);
    }

    Ok(())
}

fn reject_install_context_flags(
    api: Option<kali_cli::ApiSurface>,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if api.is_some() || sandbox.is_some() {
        let diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            "`install` does not accept `--api` or `--sandbox` in early phases; use the project manifest instead"
                .to_string(),
        );
        return emit_diagnostics_and_exit("install", vec![diagnostic], 5, output, None, None);
    }

    Ok(())
}

fn install_command(
    target: Option<String>,
    dev: bool,
    api: Option<kali_cli::ApiSurface>,
    sandbox: Option<PathBuf>,
    allow_scripts: bool,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    reject_install_context_flags(api, sandbox, output)?;
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
                validate_install_payload_value(&payload)
                    .expect("constructed install payload must satisfy schema-v1 shape");
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
    runtime_profiles: &[String],
    output: &CliOutputOptions,
) -> Result<Option<SandboxPolicy>, i32> {
    match sandbox {
        Some(path) => match kali_sandbox::SandboxPolicy::from_file_with_runtime_profiles(
            &path,
            runtime_profiles,
        ) {
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

fn validate_runtime_entrypoint(
    source: &PathBuf,
    api_surface: kali_cli::ApiSurface,
) -> Result<(), Diagnostic> {
    if is_declaration_only_source_file(source) {
        Err(Diagnostic::error(
            e5::INVALID_PRIMARY_INPUT_KIND as u32,
            format!(
                "declaration-only file '{}' cannot be used as a runtime entrypoint",
                source.display()
            ),
        )
        .with_suggestion("use `kali check` for declaration-only files"))
    } else if let Some(diagnostic) = validate_package_bin_runtime_entrypoint(source, api_surface) {
        Err(diagnostic)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct PackageBinEntrypoint {
    package_name: String,
    bin_name: String,
}

fn validate_package_bin_runtime_entrypoint(
    source: &Path,
    api_surface: kali_cli::ApiSurface,
) -> Option<Diagnostic> {
    if api_surface == kali_cli::ApiSurface::Node {
        return None;
    }

    let package_bin = detect_package_bin_entrypoint(source)?;
    let source_contents = fs::read_to_string(source).ok()?;
    let markers = collect_unsupported_node_bin_markers(&source_contents);
    if markers.is_empty() {
        return None;
    }

    Some(
        Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "npm package bin '{}' from package '{}' assumes Node.js CLI features ({}) that are unavailable on the '{}' API surface in this phase",
                package_bin.bin_name,
                package_bin.package_name,
                markers.join(", "),
                api_surface,
            ),
        )
        .note("package bins that depend on CommonJS `require()` or the Node `process` global require the later Node compatibility path")
        .with_suggestion("run the package through an imported library entrypoint, or use the documented later-phase Node compatibility target"),
    )
}

fn detect_package_bin_entrypoint(source: &Path) -> Option<PackageBinEntrypoint> {
    let package_root = package_root_for_node_modules_source(source)?;
    let package_json_path = package_root.join("package.json");
    let package_json_contents = fs::read_to_string(&package_json_path).ok()?;
    let package_json: serde_json::Value = serde_json::from_str(&package_json_contents).ok()?;
    let package_name = package_json
        .get("name")
        .and_then(|value| value.as_str())?
        .to_string();
    let bin = package_json.get("bin")?;
    let relative_path = source.strip_prefix(&package_root).ok()?.to_string_lossy();

    match bin {
        serde_json::Value::String(path) => {
            if relative_path == path.as_str() {
                Some(PackageBinEntrypoint {
                    package_name: package_name.clone(),
                    bin_name: package_name,
                })
            } else {
                None
            }
        }
        serde_json::Value::Object(entries) => entries.iter().find_map(|(name, value)| {
            let path = value.as_str()?;
            (relative_path == path).then(|| PackageBinEntrypoint {
                package_name: package_name.clone(),
                bin_name: name.clone(),
            })
        }),
        _ => None,
    }
}

fn package_root_for_node_modules_source(source: &Path) -> Option<PathBuf> {
    for ancestor in source.ancestors() {
        if !ancestor.join("package.json").exists() {
            continue;
        }

        let parent = ancestor.parent()?;
        if parent.file_name() == Some(std::ffi::OsStr::new("node_modules")) {
            return Some(ancestor.to_path_buf());
        }

        let grandparent = parent.parent()?;
        if parent
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with('@'))
            && grandparent.file_name() == Some(std::ffi::OsStr::new("node_modules"))
        {
            return Some(ancestor.to_path_buf());
        }
    }

    None
}

fn collect_unsupported_node_bin_markers(source: &str) -> Vec<&'static str> {
    let mut markers = Vec::new();
    if source.contains("require(") {
        markers.push("CommonJS require()")
    }
    if source_mentions_identifier(source, "process") {
        markers.push("Node process global")
    }
    markers
}

fn source_mentions_identifier(source: &str, ident: &str) -> bool {
    let bytes = source.as_bytes();
    let ident_bytes = ident.as_bytes();
    if ident_bytes.is_empty() || bytes.len() < ident_bytes.len() {
        return false;
    }

    let is_ident = |byte: u8| byte == b'_' || byte.is_ascii_alphanumeric();
    for start in 0..=bytes.len() - ident_bytes.len() {
        if &bytes[start..start + ident_bytes.len()] != ident_bytes {
            continue;
        }

        let before_ok = start == 0 || !is_ident(bytes[start - 1]);
        let after_index = start + ident_bytes.len();
        let after_ok = after_index == bytes.len() || !is_ident(bytes[after_index]);
        if before_ok && after_ok {
            return true;
        }
    }

    false
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

    if target.chars().any(|ch| ch.is_whitespace()) {
        return Err(Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            format!(
                "`kali {command}` accepts only registry package identifiers without whitespace, not '{}'",
                target
            ),
        ));
    }

    if let Some((scheme, _)) = target.split_once(':') {
        if scheme != "jsr" {
            return Err(Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                format!(
                    "`kali {command}` accepts only bare npm package names or `jsr:` identifiers, not '{}'",
                    target
                ),
            ));
        }
    }

    let (registry, package_name, install_name, report_label) =
        if let Some(spec) = target.strip_prefix("jsr:") {
            if spec.is_empty() {
                return Err(Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    format!("`kali {command}` requires a package name after `jsr:`"),
                ));
            }
            if spec.contains(':')
                || spec.starts_with("http://")
                || spec.starts_with("https://")
                || spec.starts_with("./")
                || spec.starts_with("../")
                || spec.starts_with('/')
            {
                return Err(Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    format!(
                        "`kali {command}` accepts only registry package identifiers, not '{}'",
                        target
                    ),
                ));
            }
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
            e6::DEPENDENCY_STATE_MISSING as u32,
            format!(
                "failed to read package metadata '{}': {}",
                path.display(),
                error
            ),
        )
    })?;
    let package_json: Value = serde_json::from_str(&raw).map_err(|error| {
        Diagnostic::error(
            e6::DEPENDENCY_STATE_MISSING as u32,
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
                e6::DEPENDENCY_STATE_MISSING as u32,
                format!("package metadata '{}' is missing a version", path.display()),
            )
        })
}

fn analysis_context_for_api(
    api: kali_cli::ApiSurface,
    runtime_profiles: Vec<String>,
    compat_features: Vec<String>,
) -> EffectAnalysisContext {
    let mut context = EffectAnalysisContext::new(api.to_string());
    context.runtime_profiles = runtime_profiles;
    context.compat_features = compat_features;
    context.normalized()
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
    let context = analysis_context_for_api(api, Vec::new(), Vec::new());
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
    let value = serde_json::to_value(payload).expect("serialize native json payload");
    match command {
        "effects" => validate_effects_payload_value(&value),
        "package-effects" => validate_package_effects_payload_value(&value),
        "package-audit" => validate_package_audit_payload_value(&value),
        _ => Ok(()),
    }
    .expect("constructed native json payload must satisfy schema-v1 shape");

    if output.is_json() {
        print_envelope(command, true, vec![], vec![], value, None, None, 0, output);
    } else if output.pretty {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).expect("serialize native json payload")
        );
    } else {
        println!(
            "{}",
            serde_json::to_string(&value).expect("serialize native json payload")
        );
    }
    Ok(())
}

fn diagnostics_exit_code(diagnostics: &[Diagnostic]) -> i32 {
    if diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.code,
            Some(code) if matches!(code, 5001 | 5506 | 5507 | 5508 | 5509 | 5510 | 5511)
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

#[cfg(test)]
mod tests {
    use super::{
        command_allows_pretty_without_json, emit_native_json_payload, manifest_compat_features,
        manifest_runtime_profiles, package_audit_command, package_audit_preview_diagnostic,
        sort_package_audit_findings, CliOutputOptions,
    };
    use kali_cli::{ColorChoice, OutputFormat};
    use kali_common::{FileId, Span};
    use kali_error::{_error_codes::e5, Diagnostic, DiagnosticContextOrigin};
    use kali_npm::ProjectManifest;
    use serde_json::json;

    fn diagnostic_with_span(file_id: u32, start: u32, end: u32) -> Diagnostic {
        Diagnostic::error(e5::INVALID_CLI_USAGE as u32, "shared finding").with_span(Span::new(
            FileId::new(file_id),
            start,
            end,
        ))
    }

    #[test]
    fn package_audit_findings_sort_by_span_as_final_tiebreaker() {
        let mut findings = vec![
            diagnostic_with_span(4, 20, 24),
            diagnostic_with_span(2, 10, 12),
            diagnostic_with_span(2, 8, 9),
            diagnostic_with_span(2, 10, 11),
        ];

        sort_package_audit_findings(&mut findings);

        let spans = findings
            .iter()
            .map(|diagnostic| diagnostic.span.expect("span"))
            .collect::<Vec<_>>();

        assert_eq!(
            spans,
            vec![
                Span::new(FileId::new(2), 8, 9),
                Span::new(FileId::new(2), 10, 11),
                Span::new(FileId::new(2), 10, 12),
                Span::new(FileId::new(4), 20, 24),
            ]
        );
    }

    #[test]
    fn diagnostics_exit_code_treats_feature_availability_as_usage_error() {
        let diagnostic = Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, "feature unavailable");

        assert_eq!(super::diagnostics_exit_code(&[diagnostic]), 5);
    }

    #[test]
    fn pretty_without_json_is_only_allowed_for_effects_and_package_effects() {
        let effects = super::Commands::Effects {
            api: None,
            compat: Vec::new(),
            wasm_threads: false,
            sandbox: None,
            files: Vec::new(),
        };
        let package_effects = super::Commands::PackageEffects {
            api: None,
            compat: Vec::new(),
            wasm_threads: false,
            sandbox: None,
            target: Vec::new(),
        };
        let package_audit = super::Commands::PackageAudit {
            api: None,
            compat: Vec::new(),
            wasm_threads: false,
            sandbox: None,
            target: Vec::new(),
            preview: false,
        };

        assert!(command_allows_pretty_without_json(Some(&effects)));
        assert!(command_allows_pretty_without_json(Some(&package_effects)));
        assert!(!command_allows_pretty_without_json(Some(&package_audit)));
        assert!(!command_allows_pretty_without_json(None));
    }

    #[test]
    fn package_audit_preview_rejects_before_target_validation() {
        let output = CliOutputOptions {
            format: OutputFormat::Text,
            pretty: false,
            verbose: false,
            quiet: false,
            color: ColorChoice::Auto,
        };

        let exit_code =
            package_audit_command(Vec::new(), true, None, Vec::new(), false, None, &output)
                .expect_err("preview should fail before target validation");

        assert_eq!(exit_code, 5);

        let diagnostic = package_audit_preview_diagnostic();
        let context = diagnostic.context.as_ref().expect("diagnostic context");
        assert_eq!(diagnostic.code, Some(e5::INVALID_CLI_USAGE as u32));
        assert_eq!(diagnostic.message, "legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape");
        assert_eq!(context.origin, DiagnosticContextOrigin::Cli);
        assert_eq!(context.flag.as_deref(), Some("--preview"));
        assert_eq!(context.requested_value.as_deref(), Some("true"));
        assert_eq!(context.effective_value.as_deref(), Some("true"));
    }

    #[test]
    fn native_json_payload_emission_validates_effects_payload_shape() {
        let output = CliOutputOptions {
            format: OutputFormat::Json,
            pretty: false,
            verbose: false,
            quiet: false,
            color: ColorChoice::Auto,
        };

        let result = std::panic::catch_unwind(|| {
            let _ = emit_native_json_payload("effects", &json!({"schemaVersion": 1}), &output);
        });

        assert!(
            result.is_err(),
            "invalid effects payload should panic before emission"
        );
    }

    #[test]
    fn manifest_compat_features_attach_config_context() {
        let manifest = ProjectManifest {
            compat: Some(json!({"features": ["eval", "future"]})),
            ..ProjectManifest::minimal()
        };

        let diagnostics = manifest_compat_features(&manifest)
            .expect_err("unsupported compat feature should fail manifest validation");
        let diagnostic = diagnostics.first().expect("diagnostic");
        let context = diagnostic.context.as_deref().expect("diagnostic context");

        assert_eq!(context.origin, DiagnosticContextOrigin::Config);
        assert_eq!(context.config_path.as_deref(), Some("compat.features"));
        assert_eq!(context.effective_value.as_deref(), Some("future"));
    }

    #[test]
    fn manifest_runtime_profiles_attach_config_context() {
        let manifest = ProjectManifest {
            compiler_options: Some(json!({"runtimeProfiles": ["future"]})),
            ..ProjectManifest::minimal()
        };

        let diagnostics = manifest_runtime_profiles(&manifest)
            .expect_err("unsupported runtime profile should fail manifest validation");
        let diagnostic = diagnostics.first().expect("diagnostic");
        let context = diagnostic.context.as_deref().expect("diagnostic context");

        assert_eq!(context.origin, DiagnosticContextOrigin::Config);
        assert_eq!(
            context.config_path.as_deref(),
            Some("compilerOptions.runtimeProfiles")
        );
    }
}
