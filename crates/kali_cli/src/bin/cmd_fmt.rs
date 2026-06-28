//! fmt command handler.

use kali_cli::output::{validate_fmt_payload_value, CliOutputOptions};
use kali_error::{_error_codes::e5, Diagnostic};
use kali_fmt::format_source;
use serde_json::json;
use std::{fs, path::PathBuf};

use super::shared;

pub(crate) fn fmt_command(files: Vec<String>, check: bool, output: &CliOutputOptions) -> Result<(), i32> {
    shared::ensure_project_ready_or_exit(output)?;
    let selected_files = shared::selected_source_files(files, true);
    if selected_files.is_empty() {
        if output.is_json() {
            let payload = json!({"filesFormatted": 0, "filesChecked": 0});
            validate_fmt_payload_value(&payload)
                .expect("constructed fmt payload must satisfy schema-v1 shape");
            shared::print_envelope("fmt", true, vec![], vec![], payload, None, None, 0, output);
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
                return shared::emit_diagnostics_and_exit(
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
                    return shared::emit_diagnostics_and_exit(
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
        shared::print_envelope(
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
