//! check command handler.

use kali_cli::{
    build, discover_source_files,
    output::{validate_check_payload_value, CliOutputOptions},
};
use kali_error::{_error_codes::e5, Diagnostic};
use kali_npm::discover_project_root;
use serde_json::json;
use std::{fs, path::{Path, PathBuf}};

use super::shared;
use super::cmd_package::validate_source_effects_against_policy_for_roots;
use super::config;

pub(crate) fn check_command(
    files: Vec<String>,
    sandbox: Option<PathBuf>,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    wasm_threads: bool,
    fix: bool,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let effective_api = match config::resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("check", diagnostics, 5, output, None, None)
        }
    };

    if fix {
        let diagnostic = Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "kali check --fix is unavailable in this phase; use kali lint --fix for autofix"
                .to_string(),
        );
        return shared::emit_diagnostics_and_exit("check", vec![diagnostic], 1, output, None, None);
    }

    shared::ensure_project_ready_or_exit(output)?;
    let effective_compat = match config::resolve_effective_compat_features(compat) {
        Ok(features) => features,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("check", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) =
        config::reject_unavailable_compat_features("check", &effective_compat, output, None, None)
    {
        return Err(exit_code);
    }
    let effective_runtime_profiles = match config::resolve_effective_runtime_profiles(wasm_threads) {
        Ok(profiles) => profiles,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("check", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) = config::reject_unavailable_runtime_profiles(
        "check",
        &effective_runtime_profiles,
        !matches!(effective_api, kali_cli::ApiSurface::Browser),
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let policy = shared::load_policy_or_exit(sandbox, &effective_runtime_profiles, output)?;
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
                let (file_errors, file_warnings) = shared::split_and_convert_diagnostics(
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
                    shared::split_and_convert_diagnostics(&diagnostics, None, None);
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
        shared::print_envelope(
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
