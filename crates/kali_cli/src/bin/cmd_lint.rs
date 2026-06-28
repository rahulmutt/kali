//! lint command handler.

use kali_cli::output::{validate_lint_payload_value, CliOutputOptions};
use kali_error::{_error_codes::e5, Diagnostic};
use kali_lint::lint_with_options;
use serde_json::json;
use std::{fs, path::PathBuf};

use super::shared;

pub(crate) fn lint_command(files: Vec<String>, fix: bool, output: &CliOutputOptions) -> Result<(), i32> {
    shared::ensure_project_ready_or_exit(output)?;
    let selected_files = shared::selected_source_files(files, true);
    if selected_files.is_empty() {
        if output.is_json() {
            let payload =
                json!({"filesLinted": 0, "errorCount": 0, "warningCount": 0, "fixedCount": 0});
            validate_lint_payload_value(&payload)
                .expect("constructed lint payload must satisfy schema-v1 shape");
            shared::print_envelope("lint", true, vec![], vec![], payload, None, None, 0, output);
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
                return shared::emit_diagnostics_and_exit(
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
            shared::split_and_convert_diagnostics(&result.diagnostics, Some(&path), Some(&source));
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
                    return shared::emit_diagnostics_and_exit(
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
        shared::print_envelope(
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
