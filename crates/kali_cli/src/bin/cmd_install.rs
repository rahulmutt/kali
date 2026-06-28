//! install command handler.

use kali_cli::output::{validate_install_payload_value, CliOutputOptions};
use kali_error::{_error_codes::e5, Diagnostic};
use kali_npm::{discover_project_root, install_project, InstallOptions};
use serde_json::json;
use std::path::PathBuf;

use super::shared;

pub(crate) fn install_command(
    target: Option<String>,
    dev: bool,
    api: Option<kali_cli::ApiSurface>,
    sandbox: Option<PathBuf>,
    allow_scripts: bool,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    super::shared::reject_install_context_flags(api, sandbox, output)?;
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            let diagnostic = Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                format!("failed to read current directory: {}", error),
            );
            return shared::emit_diagnostics_and_exit("install", vec![diagnostic], 1, output, None, None);
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
                    "removed": summary.removed,
                });
                validate_install_payload_value(&payload)
                    .expect("constructed install payload must satisfy schema-v1 shape");
                shared::print_envelope(
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
            let exit_code = shared::diagnostics_exit_code(&diagnostics);
            shared::emit_diagnostics_and_exit("install", diagnostics, exit_code, output, None, None)
        }
    }
}
