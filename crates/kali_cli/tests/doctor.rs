use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_kali")))
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("parse json stdout")
}

#[test]
fn doctor_reports_env_selected_browser_harness_in_json() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("doctor")
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node --test")
        .output()
        .expect("run kali doctor");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "doctor");
    assert_eq!(json["success"], true);
    let harness = &json["payload"]["browserHarness"];
    assert_eq!(harness["envVar"], "KALI_BROWSER_BUNDLE_HARNESS_COMMAND");
    assert_eq!(harness["source"], "env");
    assert_eq!(harness["override"], "node --test");
    assert_eq!(harness["command"], serde_json::json!(["node", "--test"]));
    assert_eq!(harness["executable"], "node");
    assert_eq!(harness["args"], serde_json::json!(["--test"]));

    let contract = &json["payload"]["browserRuntimeContract"];
    assert_eq!(contract["hostLabel"], "browser-requested");
    assert_eq!(contract["hostDescription"], "real browser host");
    assert_eq!(
        contract["hostDescriptionNote"],
        "browser runtime host description: real browser host"
    );
    assert_eq!(
        contract["supportedCommands"],
        serde_json::json!(["run", "test"])
    );
    assert!(contract["diagnosticHint"]
        .as_str()
        .expect("diagnostic hint string")
        .contains("kali check --api browser"));
    assert_eq!(contract["diagnosticNotes"], serde_json::json!([
        "supported browser runtime commands: run, test",
        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
        "browser runtime host description: real browser host"
    ]));
}

#[test]
fn doctor_reports_env_selected_browser_harness_in_human_output() {
    let output = Command::new(kali_bin())
        .arg("doctor")
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node --test")
        .output()
        .expect("run kali doctor");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Browser harness:"), "stdout: {stdout}");
    assert!(stdout.contains("  env var: KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));
    assert!(stdout.contains("  source: env"));
    assert!(stdout.contains("  override: node --test"));
    assert!(stdout.contains("  command: node --test"));
    assert!(stdout.contains("Browser runtime contract:"));
    assert!(stdout.contains("  host label: browser-requested"));
    assert!(stdout.contains("  host description: real browser host"));
    assert!(stdout.contains("  supported commands: run, test"));
    assert!(stdout.contains("  diagnostic hint:"));
    assert!(stdout.contains("  note: supported browser runtime commands: run, test"));
    assert!(stdout.contains(
        "  note: browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work"
    ));
    assert!(stdout.contains(
        "  note: browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness"
    ));
    assert!(stdout.contains(
        "  note: browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid"
    ));
    assert!(stdout.contains("  note: browser runtime host description: real browser host"));
}

#[test]
fn doctor_reports_auto_selected_browser_harness_in_human_output() {
    let output = Command::new(kali_bin())
        .arg("doctor")
        .env_remove("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
        .output()
        .expect("run kali doctor");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Browser harness:"), "stdout: {stdout}");
    assert!(stdout.contains("  env var: KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));
    assert!(stdout.contains("  source: auto"));
    assert!(
        !stdout.contains("  override:"),
        "auto-selected harness should not print an override: {stdout}"
    );
    assert!(stdout.contains("  command:"));
    assert!(stdout.contains("  executable available:"));
    assert!(stdout.contains("Browser runtime contract:"));
    assert!(stdout.contains("  host label: browser-requested"));
    assert!(stdout.contains("  host description: real browser host"));
    assert!(stdout.contains("  supported commands: run, test"));
    assert!(stdout.contains("  diagnostic hint:"));
    assert!(stdout.contains("  note: supported browser runtime commands: run, test"));
    assert!(stdout.contains(
        "  note: browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work"
    ));
    assert!(stdout.contains(
        "  note: browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness"
    ));
    assert!(stdout.contains(
        "  note: browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid"
    ));
    assert!(stdout.contains("  note: browser runtime host description: real browser host"));
}

#[test]
fn doctor_reports_unavailable_browser_harness_executable() {
    let output = Command::new(kali_bin())
        .arg("doctor")
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "definitely-not-a-real-browser-harness --probe",
        )
        .output()
        .expect("run kali doctor");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Browser harness:"), "stdout: {stdout}");
    assert!(stdout.contains("  source: env"));
    assert!(stdout.contains("  override: definitely-not-a-real-browser-harness --probe"));
    assert!(stdout.contains("  command: definitely-not-a-real-browser-harness --probe"));
    assert!(stdout.contains("  executable available: false"));
    assert!(stdout.contains("Browser runtime contract:"));
    assert!(stdout.contains("  host label: browser-requested"));
    assert!(stdout.contains("  host description: real browser host"));
    assert!(stdout.contains("  supported commands: run, test"));
    assert!(stdout.contains("  diagnostic hint:"));
    assert!(stdout.contains("  note: supported browser runtime commands: run, test"));
    assert!(stdout.contains(
        "  note: browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work"
    ));
    assert!(stdout.contains(
        "  note: browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness"
    ));
    assert!(stdout.contains(
        "  note: browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid"
    ));
    assert!(stdout.contains("  note: browser runtime host description: real browser host"));
}

#[test]
fn doctor_reports_env_selected_browser_harness_in_pretty_json_under_quiet() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("--pretty")
        .arg("--quiet")
        .arg("doctor")
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node --test")
        .output()
        .expect("run kali doctor");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains('\n'),
        "pretty JSON should contain newlines: {stdout}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "doctor");
    assert_eq!(json["success"], true);
    let harness = &json["payload"]["browserHarness"];
    assert_eq!(harness["source"], "env");
    assert_eq!(harness["override"], "node --test");
    assert_eq!(harness["command"], serde_json::json!(["node", "--test"]));
    assert_eq!(harness["executable"], "node");
    assert_eq!(harness["args"], serde_json::json!(["--test"]));
}

#[test]
fn doctor_reports_auto_selected_browser_harness_in_pretty_json_under_quiet() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("--pretty")
        .arg("--quiet")
        .arg("doctor")
        .env_remove("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
        .output()
        .expect("run kali doctor");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains('\n'),
        "pretty JSON should contain newlines: {stdout}"
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "doctor");
    assert_eq!(json["success"], true);
    let harness = &json["payload"]["browserHarness"];
    assert_eq!(harness["source"], "auto");
    assert!(harness["override"].is_null());
    let command = harness["command"]
        .as_array()
        .expect("browser harness command array");
    assert!(
        !command.is_empty(),
        "browser harness command should not be empty"
    );
    assert_eq!(harness["executable"], command[0]);
    assert_eq!(harness["args"], json!(command[1..]));
    assert!(harness["executableAvailable"].is_boolean());

    let contract = &json["payload"]["browserRuntimeContract"];
    assert_eq!(contract["hostLabel"], "browser-requested");
    assert_eq!(contract["hostDescription"], "real browser host");
    assert_eq!(
        contract["hostDescriptionNote"],
        "browser runtime host description: real browser host"
    );
    assert_eq!(
        contract["supportedCommands"],
        serde_json::json!(["run", "test"])
    );
    assert!(contract["diagnosticHint"]
        .as_str()
        .expect("diagnostic hint string")
        .contains("kali check --api browser"));
}

#[test]
fn doctor_reports_auto_selected_browser_harness_in_json() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("doctor")
        .env_remove("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
        .output()
        .expect("run kali doctor");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "doctor");
    assert_eq!(json["success"], true);
    let harness = &json["payload"]["browserHarness"];
    assert_eq!(harness["envVar"], "KALI_BROWSER_BUNDLE_HARNESS_COMMAND");
    assert_eq!(harness["source"], "auto");
    assert!(harness["override"].is_null());
    let command = harness["command"]
        .as_array()
        .expect("browser harness command array");
    assert!(
        !command.is_empty(),
        "browser harness command should not be empty"
    );
    assert_eq!(harness["executable"], command[0]);
    assert_eq!(harness["args"], json!(command[1..]));
    assert!(harness["executableAvailable"].is_boolean());

    let contract = &json["payload"]["browserRuntimeContract"];
    assert_eq!(contract["hostLabel"], "browser-requested");
    assert_eq!(contract["hostDescription"], "real browser host");
    assert_eq!(
        contract["hostDescriptionNote"],
        "browser runtime host description: real browser host"
    );
    assert_eq!(
        contract["supportedCommands"],
        serde_json::json!(["run", "test"])
    );
    assert!(contract["diagnosticHint"]
        .as_str()
        .expect("diagnostic hint string")
        .contains("kali check --api browser"));
    assert_eq!(contract["diagnosticNotes"], serde_json::json!([
        "supported browser runtime commands: run, test",
        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
        "browser runtime host description: real browser host"
    ]));
}

#[test]
fn doctor_reports_unavailable_browser_harness_executable_in_json() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("doctor")
        .env(
            "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "definitely-not-a-real-browser-harness --probe",
        )
        .output()
        .expect("run kali doctor");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "doctor");
    assert_eq!(json["success"], true);
    let harness = &json["payload"]["browserHarness"];
    assert_eq!(harness["source"], "env");
    assert_eq!(
        harness["override"],
        "definitely-not-a-real-browser-harness --probe"
    );
    assert_eq!(
        harness["command"],
        serde_json::json!(["definitely-not-a-real-browser-harness", "--probe"])
    );
    assert_eq!(harness["executableAvailable"], false);

    let contract = &json["payload"]["browserRuntimeContract"];
    assert_eq!(contract["hostLabel"], "browser-requested");
    assert_eq!(contract["hostDescription"], "real browser host");
    assert_eq!(
        contract["hostDescriptionNote"],
        "browser runtime host description: real browser host"
    );
    assert_eq!(
        contract["supportedCommands"],
        serde_json::json!(["run", "test"])
    );
    assert!(contract["diagnosticHint"]
        .as_str()
        .expect("diagnostic hint string")
        .contains("kali check --api browser"));
    assert_eq!(contract["diagnosticNotes"], serde_json::json!([
        "supported browser runtime commands: run, test",
        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
        "browser runtime host description: real browser host"
    ]));
}

#[test]
fn doctor_reports_malformed_browser_harness_override_in_json() {
    for value in ["", "   "] {
        let output = Command::new(kali_bin())
            .arg("--output")
            .arg("json")
            .arg("doctor")
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", value)
            .output()
            .expect("run kali doctor");

        assert_eq!(output.status.code(), Some(5));
        let json = parse_json_stdout(&output);
        assert_eq!(json["command"], "doctor");
        assert_eq!(json["success"], false);
        assert_eq!(json["errors"][0]["code"], "E5508");
        assert!(json["errors"][0]["message"]
            .as_str()
            .expect("error message")
            .contains("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));
    }
}
