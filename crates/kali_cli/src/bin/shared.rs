//! Shared CLI helpers: envelope printing, exit codes, preflight (crate-internal).

use kali_cli::{
    discover_source_files, is_declaration_only_source_file,
    output::{self, validate_effects_payload_value, validate_package_effects_payload_value,
    validate_package_audit_payload_value, CliOutputOptions},
    Commands,
};
use kali_error::{_error_codes::e5, Diagnostic};
use kali_npm::{discover_project_root, ensure_project_ready};
use kali_sandbox::SandboxPolicy;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[allow(clippy::too_many_arguments)]
pub(crate) fn print_envelope(
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


pub(crate) fn diagnostics_exit_code(diagnostics: &[Diagnostic]) -> i32 {
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


pub(crate) fn emit_native_json_payload<T: serde::Serialize>(
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


pub(crate) fn emit_diagnostics_and_exit(
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


pub(crate) fn split_and_convert_diagnostics(
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


pub(crate) fn single_diagnostic_to_values(
    diagnostic: Diagnostic,
    source_path: Option<&Path>,
    source_text: Option<&str>,
) -> (Vec<Value>, Vec<Value>) {
    let diagnostics = vec![diagnostic];
    split_and_convert_diagnostics(&diagnostics, source_path, source_text)
}


pub(crate) fn load_policy_or_exit(
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


pub(crate) fn ensure_project_ready_or_exit(output: &CliOutputOptions) -> Result<(), i32> {
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


pub(crate) fn selected_source_files(files: Vec<String>, discover: bool) -> Vec<String> {
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


pub(crate) fn single_or_error(
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


pub(crate) fn validate_runtime_entrypoint(
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
    } else if let Some(diagnostic) = super::cmd_package::validate_package_bin_runtime_entrypoint(source, api_surface) {
        Err(diagnostic)
    } else {
        Ok(())
    }
}


pub(crate) fn reject_workflow_context_flags(
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


pub(crate) fn reject_install_context_flags(
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


pub(crate) fn command_allows_pretty_without_json(command: Option<&Commands>) -> bool {
    matches!(
        command,
        Some(Commands::Effects { .. }) | Some(Commands::PackageEffects { .. })
    )
}


pub(crate) fn matches_test_filter(file: &str, pattern: &str) -> bool {
    let path = PathBuf::from(file);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(file);
    file.contains(pattern) || name.contains(pattern)
}


