//! doctor command handler.

use kali_cli::output::{validate_doctor_payload_value, CliOutputOptions};
use kali_error::{_error_codes::e5, Diagnostic};
use kali_runtime::{
    browser_harness_command_parts_checked, browser_runtime_contract_value,
    BrowserRuntimeContract, BROWSER_HARNESS_COMMAND_ENV,
};
use serde_json::json;
use std::{env, process::Command as ProcessCommand};

use super::shared;

pub(crate) fn doctor_command(output: &CliOutputOptions) -> Result<(), i32> {
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
            return shared::emit_diagnostics_and_exit("doctor", vec![diagnostic], 5, output, None, None);
        }
    };
    let executable = command_parts.first().cloned().unwrap_or_default();
    let args: Vec<String> = command_parts.iter().skip(1).cloned().collect();
    let executable_available = !executable.is_empty()
        && ProcessCommand::new(&executable)
            .arg("--version")
            .output()
            .is_ok();
    let browser_runtime_contract_json = browser_runtime_contract_value();
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
        "browserRuntimeContract": browser_runtime_contract_json,
    });
    validate_doctor_payload_value(&payload)
        .expect("constructed doctor payload must satisfy schema-v1 shape");

    if output.is_json() {
        shared::print_envelope(
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
            "  host description note: {}",
            browser_runtime_contract.host_description_note
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
