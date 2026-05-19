use kali_common::{FileId, Span};
use kali_error::{_error_codes::e5, Diagnostic};
use kali_runtime::{browser_runtime_contract_value, BrowserRuntimeContract};
use serde_json::json;
use std::path::Path;

use crate::output::{
    diagnostic_to_json, emit_envelope_value, merge_thread_topology_snapshot_values,
    validate_check_payload_value, validate_doctor_payload_value, validate_effects_payload_value,
    validate_envelope_value, validate_fmt_payload_value, validate_init_payload_value,
    validate_install_payload_value, validate_lint_payload_value,
    validate_package_audit_payload_value, validate_package_effects_payload_value,
    validate_run_payload_value, validate_test_payload_value,
};

fn assert_payload_accepts_schema_permitted_extension_key(
    mut payload: serde_json::Value,
    validator: fn(&serde_json::Value) -> Result<(), String>,
) {
    payload
        .as_object_mut()
        .expect("payload object")
        .insert("extensionKey".to_string(), json!("allowed"));
    validator(&payload).expect("schema-permitted extension key should validate");
}

#[test]
fn emitted_cli_envelopes_satisfy_the_schema_v1_top_level_shape() {
    let value = emit_envelope_value(
        "doctor",
        true,
        json!([]),
        json!([]),
        json!({"answer": 42}),
        Some("stdout text".to_string()),
        None,
        0,
    );

    validate_envelope_value(&value).expect("constructed envelope should validate");

    let object = value.as_object().expect("envelope object");
    assert_eq!(object["schemaVersion"], json!(1));
    assert_eq!(object["command"], json!("doctor"));
    assert_eq!(object["success"], json!(true));
    assert_eq!(object["errors"], json!([]));
    assert_eq!(object["warnings"], json!([]));
    assert_eq!(object["payload"], json!({"answer": 42}));
    assert_eq!(object["stdout"], json!("stdout text"));
    assert_eq!(object["stderr"], serde_json::Value::Null);
    assert_eq!(object["exitCode"], json!(0));
}

#[test]
fn emitted_cli_envelopes_reject_empty_or_whitespace_command() {
    for command in ["", " \n\t "] {
        let mut value = emit_envelope_value(
            "doctor",
            true,
            json!([]),
            json!([]),
            json!({"answer": 42}),
            None,
            None,
            0,
        );
        value
            .as_object_mut()
            .expect("envelope object")
            .insert("command".to_string(), json!(command));

        let error = validate_envelope_value(&value)
            .expect_err("empty or whitespace command should fail validation");
        assert!(error.contains("command"), "unexpected error: {error}");
        assert!(
            error.contains("non-empty, non-whitespace string"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn emitted_cli_envelopes_preserve_empty_diagnostic_arrays_for_run_text_output() {
    let value = emit_envelope_value(
        "run",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        Some("stdout text".to_string()),
        Some("stderr text".to_string()),
        0,
    );

    validate_envelope_value(&value).expect("constructed envelope should validate");

    let object = value.as_object().expect("envelope object");
    assert_eq!(object["command"], json!("run"));
    assert_eq!(object["success"], json!(true));
    assert_eq!(object["errors"], json!([]));
    assert_eq!(object["warnings"], json!([]));
    assert_eq!(object["payload"], json!({"result": "ok"}));
    assert_eq!(object["stdout"], json!("stdout text"));
    assert_eq!(object["stderr"], json!("stderr text"));
    assert_eq!(object["exitCode"], json!(0));
}

#[test]
fn emitted_cli_envelopes_accept_artifacts_arrays() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 42},
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7}
        ]),
    );

    validate_envelope_value(&value).expect("artifacts array should validate");
}

#[test]
fn emitted_cli_envelopes_reject_out_of_order_artifacts() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7},
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 42}
        ]),
    );

    let error = validate_envelope_value(&value).expect_err("out-of-order artifacts should fail");
    assert!(
        error.contains("must be sorted by role, kind, then path"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_duplicate_primary_artifact_roles() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 42},
            {"path": "alt.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 11}
        ]),
    );

    let error = validate_envelope_value(&value).expect_err("duplicate primary role should fail");
    assert!(
        error.contains("duplicates primary-executable"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_unrecognized_artifact_roles() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "auxiliary", "bytes": 42},
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7}
        ]),
    );

    let error =
        validate_envelope_value(&value).expect_err("unrecognized artifact roles should fail");
    assert!(
        error.contains("canonical schema-v1 role") && error.contains("auxiliary"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_unexpected_artifact_keys() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 42, "extra": true},
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7}
        ]),
    );

    let error = validate_envelope_value(&value).expect_err("unexpected artifact keys should fail");
    assert!(
        error.contains("CLI envelope artifact") && error.contains("unexpected key `extra`"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_invalid_artifact_bytes() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": -1},
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7.5}
        ]),
    );

    let error = validate_envelope_value(&value).expect_err("invalid artifact bytes should fail");
    assert!(
        error.contains("bytes must be a non-negative integer"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_duplicate_artifact_kind_path_pairs() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 42},
            {"path": "main.wasm", "kind": "wasm-module", "role": "browser-glue", "bytes": 7}
        ]),
    );

    let error = validate_envelope_value(&value)
        .expect_err("duplicate artifact kind/path pairs should fail");
    assert!(
        error.contains("duplicates artifact `wasm-module` at `main.wasm`"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_unexpected_top_level_keys() {
    let mut value = emit_envelope_value(
        "doctor",
        true,
        json!([]),
        json!([]),
        json!({"answer": 42}),
        None,
        None,
        0,
    );
    value
        .as_object_mut()
        .expect("envelope object")
        .insert("extensionKey".to_string(), json!("not allowed"));

    let error = validate_envelope_value(&value).expect_err("unexpected keys should fail");
    assert!(
        error.contains("unexpected key `extensionKey`"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_preserve_empty_diagnostic_arrays_for_test_text_output() {
    let value = emit_envelope_value(
        "test",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        Some("stdout text".to_string()),
        Some("stderr text".to_string()),
        0,
    );

    validate_envelope_value(&value).expect("constructed envelope should validate");

    let object = value.as_object().expect("envelope object");
    assert_eq!(object["command"], json!("test"));
    assert_eq!(object["success"], json!(true));
    assert_eq!(object["errors"], json!([]));
    assert_eq!(object["warnings"], json!([]));
    assert_eq!(object["payload"], json!({"result": "ok"}));
    assert_eq!(object["stdout"], json!("stdout text"));
    assert_eq!(object["stderr"], json!("stderr text"));
    assert_eq!(object["exitCode"], json!(0));
}

#[test]
fn diagnostic_json_includes_the_top_level_file_mirror() {
    let diagnostic = Diagnostic::error(e5::INVALID_CLI_USAGE as u32, "message")
        .with_span(Span::new(FileId::new(1), 0, 4));
    let value = diagnostic_to_json(
        &diagnostic,
        Some(Path::new("src/main.ts")),
        Some("test"),
        "error",
    );

    assert_eq!(value["file"], json!("src/main.ts"));
    assert_eq!(value["span"]["file"], json!("src/main.ts"));
}

#[test]
fn diagnostic_json_rejects_a_top_level_file_mirror_mismatch() {
    let envelope = json!({
        "schemaVersion": 1,
        "command": "check",
        "success": false,
        "errors": [{
            "severity": "error",
            "code": "E5101",
            "message": "message",
            "file": "src/other.ts",
            "span": {
                "file": "src/main.ts",
                "line": 1,
                "column": 1,
                "endLine": 1,
                "endColumn": 1,
            },
            "labels": [],
            "related": [],
            "fix": null,
            "notes": [],
        }],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&envelope)
        .expect_err("mismatched file mirror should fail validation");
    assert!(
        err.contains("diagnostic file mirror must match span.file"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    validate_doctor_payload_value(&value).expect("doctor payload should validate");
}

#[test]
fn validate_doctor_payload_value_accepts_auto_browser_harness_override_null() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    validate_doctor_payload_value(&value).expect("auto browser harness payload should validate");
}

#[test]
fn validate_doctor_payload_value_rejects_browser_harness_source_override_mismatch() {
    for (source, override_value, expected_fragment) in [
        (
            "env",
            json!(null),
            "override must be a string when source is `env`",
        ),
        (
            "auto",
            json!("node --test"),
            "override must be null when source is `auto`",
        ),
    ] {
        let value = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": source,
                "override": override_value,
                "command": ["node", "--test"],
                "executable": "node",
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": "real browser host",
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&value)
            .expect_err("mismatched browserHarness source/override should fail");
        assert!(err.contains(expected_fragment), "unexpected error: {err}");
    }
}

#[test]
fn validate_doctor_payload_value_rejects_empty_browser_harness_override() {
    for value in ["", " \n\t "] {
        let payload = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": "env",
                "override": value,
                "command": ["node", "--test"],
                "executable": "node",
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": "real browser host",
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&payload)
            .expect_err("empty browser harness override should fail");
        assert!(
            err.contains("browserHarness override"),
            "unexpected error: {err}"
        );
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_accepts_whitespace_padded_browser_harness_override() {
    let payload = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": " node --test ",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    validate_doctor_payload_value(&payload)
        .expect("whitespace-padded browser harness override should validate");
}

#[test]
fn validate_doctor_payload_value_rejects_invalid_browser_harness_source() {
    for source in [json!("browser"), json!(42)] {
        let value = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": source,
                "override": "node --test",
                "command": ["node", "--test"],
                "executable": "node",
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": "real browser host",
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&value)
            .expect_err("invalid browserHarness source should fail");
        assert!(
            err.contains("source must be `env` or `auto`"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_rejects_whitespace_browser_harness_source() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": " \n\t ",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("whitespace browserHarness source should fail");
    assert!(
        err.contains("non-empty, non-whitespace string"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_empty_or_whitespace_browser_harness_env_var() {
    for env_var in ["", " \n\t "] {
        let value = json!({
            "browserHarness": {
                "envVar": env_var,
                "source": "auto",
                "override": null,
                "command": ["node", "--test"],
                "executable": "node",
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": "real browser host",
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&value)
            .expect_err("empty browserHarness envVar should fail");
        assert!(
            err.contains("browserHarness envVar"),
            "unexpected error: {err}"
        );
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_rejects_executable_command_mismatch() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "deno",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("mismatched browser harness executable should fail");
    assert!(
        err.contains("executable must match command[0]"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_empty_or_whitespace_browser_harness_executable() {
    for executable in ["", " \n\t "] {
        let value = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": "auto",
                "override": null,
                "command": ["node", "--test"],
                "executable": executable,
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": "real browser host",
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&value)
            .expect_err("empty browserHarness executable should fail");
        assert!(
            err.contains("browserHarness executable"),
            "unexpected error: {err}"
        );
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_rejects_args_command_mismatch() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--inspect"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("mismatched browser harness args should fail");
    assert!(
        err.contains("args must match command[1..]"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_non_boolean_executable_available() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": "yes",
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("non-boolean executableAvailable should fail");
    assert!(
        err.contains("executableAvailable must be a boolean"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_empty_browser_harness_strings() {
    for (field, value) in [
        ("envVar", ""),
        ("envVar", "   "),
        ("executable", ""),
        ("executable", " \t "),
    ] {
        let payload = json!({
            "browserHarness": {
                "envVar": if field == "envVar" { value } else { "KALI_BROWSER_BUNDLE_HARNESS_COMMAND" },
                "source": "env",
                "override": "node --test",
                "command": ["node", "--test"],
                "executable": if field == "executable" { value } else { "node" },
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": "real browser host",
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&payload)
            .expect_err("empty browser harness string should fail");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_rejects_empty_browser_runtime_host_description() {
    for value in ["", " \n\t "] {
        let payload = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": "env",
                "override": "node --test",
                "command": ["node", "--test"],
                "executable": "node",
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": value,
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&payload)
            .expect_err("empty browser runtime host description should fail");
        assert!(
            err.contains("browserRuntimeContract hostDescription"),
            "unexpected error: {err}"
        );
        assert!(
            err.contains(
                "doctor browserRuntimeContract hostDescription must be `real browser host`"
            ),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_accepts_padded_browser_runtime_host_label() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": " browser-requested ",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    validate_doctor_payload_value(&value)
        .expect("padded browser runtime host label should validate");
}

#[test]
fn validate_doctor_payload_value_accepts_padded_browser_runtime_host_description() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": " real browser host ",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    validate_doctor_payload_value(&value)
        .expect("padded browser runtime host description should validate");
}

#[test]
fn validate_doctor_payload_value_accepts_padded_browser_runtime_supported_commands() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": [" run ", " test "],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    validate_doctor_payload_value(&value)
        .expect("padded browser runtime supportedCommands should validate");
}

#[test]
fn validate_doctor_payload_value_accepts_padded_browser_runtime_diagnostic_notes() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                " supported browser runtime commands: run, test ",
                " browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work ",
                " browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness ",
                " browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid ",
                " browser runtime host description: real browser host "
            ]
        }
    });

    validate_doctor_payload_value(&value)
        .expect("padded browser runtime diagnosticNotes should validate");
}

#[test]
fn validate_doctor_payload_value_accepts_padded_browser_runtime_diagnostic_hint() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": " Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work. ",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    validate_doctor_payload_value(&value)
        .expect("padded browser runtime diagnostic hint should validate");
}

#[test]
fn validate_doctor_payload_value_rejects_empty_browser_harness_command_items() {
    for (field, payload, expected_fragment) in [
        (
            "browserHarness command",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", ""],
                    "executable": "node",
                    "args": ["--test"],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": "browser-requested",
                    "hostDescription": "real browser host",
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", "test"],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "command[1]",
        ),
        (
            "browserHarness args",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", "--test"],
                    "executable": "node",
                    "args": ["--test", "   "],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": "browser-requested",
                    "hostDescription": "real browser host",
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", "test"],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "args[1]",
        ),
    ] {
        let err = validate_doctor_payload_value(&payload)
            .expect_err("empty browser harness command item should fail");
        assert!(
            err.contains(expected_fragment),
            "{field} error mismatch: {err}"
        );
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_init_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "root": "/workspace/example",
        "manifestPath": "/workspace/example/kali.json",
        "sourcePath": "/workspace/example/src/main.ts",
        "library": false,
    });

    validate_init_payload_value(&value).expect("init payload should validate");
}

#[test]
fn validate_init_payload_value_rejects_blank_paths() {
    for (field, value) in [
        ("root", json!("")),
        ("manifestPath", json!("  \t ")),
        ("sourcePath", json!("\n")),
    ] {
        let payload = json!({
            "root": "/workspace/example",
            "manifestPath": "/workspace/example/kali.json",
            "sourcePath": "/workspace/example/src/main.ts",
            "library": false,
        });
        let mut payload = payload.as_object().expect("init payload object").clone();
        payload.insert(field.to_string(), value);

        let err = validate_init_payload_value(&serde_json::Value::Object(payload))
            .expect_err("blank init payload paths should fail");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_fmt_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "filesFormatted": 2,
        "filesChecked": 3,
    });

    validate_fmt_payload_value(&value).expect("fmt payload should validate");
}

#[test]
fn validate_fmt_payload_value_rejects_fractional_counts() {
    let value = json!({
        "filesFormatted": 2.5,
        "filesChecked": 3,
    });

    let err = validate_fmt_payload_value(&value)
        .expect_err("fractional fmt counts should fail validation");
    assert!(err.contains("filesFormatted"), "unexpected error: {err}");
}

#[test]
fn validate_lint_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "filesLinted": 4,
        "errorCount": 1,
        "warningCount": 2,
        "fixedCount": 3,
    });

    validate_lint_payload_value(&value).expect("lint payload should validate");
}

#[test]
fn validate_lint_payload_value_rejects_fractional_counts() {
    let value = json!({
        "filesLinted": 4.25,
        "errorCount": 1,
        "warningCount": 2,
        "fixedCount": 3,
    });

    let err = validate_lint_payload_value(&value)
        .expect_err("fractional lint counts should fail validation");
    assert!(err.contains("filesLinted"), "unexpected error: {err}");
}

#[test]
fn validate_install_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "manifestPath": "/workspace/example/kali.json",
        "lockPath": null,
        "installed": ["semver"],
        "updated": [],
        "removed": [],
    });

    validate_install_payload_value(&value).expect("install payload should validate");
}

#[test]
fn validate_install_payload_value_rejects_unexpected_top_level_keys() {
    let value = json!({
        "manifestPath": "/workspace/example/kali.json",
        "lockPath": null,
        "installed": ["semver"],
        "updated": [],
        "removed": [],
        "extensionKey": true,
    });

    let err = validate_install_payload_value(&value)
        .expect_err("unexpected install payload keys should fail validation");
    assert!(
        err.contains("unexpected key `extensionKey`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_check_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "filesChecked": 3,
        "errorCount": 1,
        "warningCount": 2,
    });

    validate_check_payload_value(&value).expect("check payload should validate");
}

#[test]
fn validate_check_payload_value_rejects_fractional_counts() {
    let value = json!({
        "filesChecked": 3.5,
        "errorCount": 1,
        "warningCount": 2,
    });

    let err = validate_check_payload_value(&value)
        .expect_err("fractional check counts should fail validation");
    assert!(err.contains("filesChecked"), "unexpected error: {err}");
}

#[test]
fn validate_run_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "exitCode": 0,
        "runtimeMs": 12,
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "threadTopology": {
            "totalInstances": 0,
            "terminatedInstances": 0,
            "liveInstances": [],
        },
    });

    validate_run_payload_value(&value).expect("run payload should validate");
}

#[test]
fn validate_run_payload_value_rejects_non_string_provenance_fields() {
    for (field, value) in [
        ("hostContract", json!(true)),
        ("runtimeBackend", json!(42)),
        ("hostContract", json!("")),
        ("runtimeBackend", json!("")),
        ("hostContract", json!("   ")),
        ("runtimeBackend", json!("   ")),
    ] {
        let payload = json!({
            "exitCode": 0,
            "runtimeMs": 12,
            field: value,
        });

        let err = validate_run_payload_value(&payload)
            .expect_err("invalid run payload provenance field should fail");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_run_payload_value_rejects_fractional_runtime_ms() {
    let value = json!({
        "exitCode": 0,
        "runtimeMs": 12.25,
    });

    let err = validate_run_payload_value(&value)
        .expect_err("fractional run runtimeMs should fail validation");
    assert!(err.contains("runtimeMs"), "unexpected error: {err}");
}

#[test]
fn validate_test_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "threadTopology": {
            "totalInstances": 0,
            "terminatedInstances": 0,
            "liveInstances": [],
        },
        "coverage": {
            "mode": "function",
            "files": [
                {
                    "file": "src/main.ts",
                    "functionsTotal": 4,
                    "functionsCovered": 3,
                    "functionsMissed": 1,
                }
            ],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    });

    validate_test_payload_value(&value).expect("test payload should validate");
}

#[test]
fn ordinary_cli_result_payloads_accept_schema_permitted_extension_keys() {
    assert_payload_accepts_schema_permitted_extension_key(
        json!({
            "filesChecked": 3,
            "errorCount": 1,
            "warningCount": 2,
        }),
        validate_check_payload_value,
    );
    assert_payload_accepts_schema_permitted_extension_key(
        json!({
            "exitCode": 0,
            "runtimeMs": 12,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "threadTopology": {
                "totalInstances": 0,
                "terminatedInstances": 0,
                "liveInstances": [],
            },
        }),
        validate_run_payload_value,
    );
    assert_payload_accepts_schema_permitted_extension_key(
        json!({
            "total": 4,
            "passed": 3,
            "failed": 1,
            "skipped": 0,
            "runtimeMs": 27,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "threadTopology": {
                "totalInstances": 0,
                "terminatedInstances": 0,
                "liveInstances": [],
            },
            "coverage": {
                "mode": "function",
                "files": [
                    {
                        "file": "src/main.ts",
                        "functionsTotal": 4,
                        "functionsCovered": 3,
                        "functionsMissed": 1,
                    }
                ],
                "summary": {
                    "functionsTotal": 4,
                    "functionsCovered": 3,
                    "functionsMissed": 1,
                    "coveragePercent": 75.0,
                },
            },
        }),
        validate_test_payload_value,
    );
    assert_payload_accepts_schema_permitted_extension_key(
        json!({
            "filesFormatted": 12,
            "filesChecked": 4,
            "durationMs": 8,
        }),
        validate_fmt_payload_value,
    );
    assert_payload_accepts_schema_permitted_extension_key(
        json!({
            "filesLinted": 4,
            "errorCount": 0,
            "warningCount": 1,
            "fixedCount": 2,
            "durationMs": 9,
        }),
        validate_lint_payload_value,
    );
}

#[test]
fn validate_test_payload_value_rejects_non_string_provenance_fields() {
    for (field, value) in [
        ("hostContract", json!(null)),
        ("runtimeBackend", json!(["wasmtime"])),
        ("hostContract", json!("")),
        ("runtimeBackend", json!("")),
        ("hostContract", json!("   ")),
        ("runtimeBackend", json!("   ")),
    ] {
        let payload = json!({
            "total": 4,
            "passed": 3,
            "failed": 1,
            "skipped": 0,
            "runtimeMs": 27,
            field: value,
            "coverage": {
                "mode": "function",
                "files": [],
                "summary": {
                    "functionsTotal": 4,
                    "functionsCovered": 3,
                    "functionsMissed": 1,
                    "coveragePercent": 75.0,
                },
            },
        });

        let err = validate_test_payload_value(&payload)
            .expect_err("invalid test payload provenance field should fail");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_test_payload_value_rejects_fractional_runtime_ms() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27.5,
        "coverage": {
            "mode": "function",
            "files": [],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("fractional test runtimeMs should fail validation");
    assert!(err.contains("runtimeMs"), "unexpected error: {err}");
}

#[test]
fn validate_run_and_test_payload_value_rejects_malformed_thread_topology() {
    let malformed_thread_topology = json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": "https://e.co/worker.js",
            "postedMessages": [],
            "postedSharedBuffers": [[[999]]],
            "wasTerminated": false,
        }],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": malformed_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": malformed_thread_topology,
                "coverage": {
                    "mode": "function",
                    "files": [],
                    "summary": {
                        "functionsTotal": 4,
                        "functionsCovered": 3,
                        "functionsMissed": 1,
                        "coveragePercent": 75.0,
                    },
                },
            }),
        ),
    ] {
        let err = validator(&payload).expect_err("malformed thread topology should fail");
        assert!(
            err.contains("threadTopology") || err.contains("postedSharedBuffers"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_duplicate_thread_topology_instance_ids() {
    let duplicated_thread_topology = json!({
        "totalInstances": 2,
        "terminatedInstances": 0,
        "liveInstances": [
            {
                "instanceId": 0,
                "scriptUrl": "https://e.co/worker-0.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
            {
                "instanceId": 0,
                "scriptUrl": "https://e.co/worker-1.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
        ],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": duplicated_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": duplicated_thread_topology,
                "coverage": {
                    "mode": "function",
                    "files": [],
                    "summary": {
                        "functionsTotal": 4,
                        "functionsCovered": 3,
                        "functionsMissed": 1,
                        "coveragePercent": 75.0,
                    },
                },
            }),
        ),
    ] {
        let err =
            validator(&payload).expect_err("duplicate thread topology instance ids should fail");
        assert!(
            err.contains("instanceId must be unique"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_unsorted_thread_topology_instance_ids() {
    let unsorted_thread_topology = json!({
        "totalInstances": 2,
        "terminatedInstances": 0,
        "liveInstances": [
            {
                "instanceId": 1,
                "scriptUrl": "https://e.co/worker-1.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
            {
                "instanceId": 0,
                "scriptUrl": "https://e.co/worker-0.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
        ],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": unsorted_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": unsorted_thread_topology,
                "coverage": {
                    "mode": "function",
                    "files": [],
                    "summary": {
                        "functionsTotal": 4,
                        "functionsCovered": 3,
                        "functionsMissed": 1,
                        "coveragePercent": 75.0,
                    },
                },
            }),
        ),
    ] {
        let err =
            validator(&payload).expect_err("unsorted thread topology instance ids should fail");
        assert!(
            err.contains("ordered by ascending instanceId"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_incoherent_thread_topology_counts() {
    let incoherent_thread_topology = json!({
        "totalInstances": 3,
        "terminatedInstances": 1,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": "https://e.co/worker-0.js",
            "postedMessages": [],
            "postedSharedBuffers": [],
            "wasTerminated": false,
        }],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": incoherent_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": incoherent_thread_topology,
                "coverage": {
                    "mode": "function",
                    "files": [],
                    "summary": {
                        "functionsTotal": 4,
                        "functionsCovered": 3,
                        "functionsMissed": 1,
                        "coveragePercent": 75.0,
                    },
                },
            }),
        ),
    ] {
        let err = validator(&payload).expect_err("incoherent thread topology counts should fail");
        assert!(
            err.contains("totalInstances must equal terminatedInstances + liveInstances.len()"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_whitespace_thread_topology_script_url() {
    let whitespace_thread_topology = json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": "   ",
            "postedMessages": [],
            "postedSharedBuffers": [],
            "wasTerminated": false,
        }],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": whitespace_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": whitespace_thread_topology,
                "coverage": {
                    "mode": "function",
                    "files": [],
                    "summary": {
                        "functionsTotal": 4,
                        "functionsCovered": 3,
                        "functionsMissed": 1,
                        "coveragePercent": 75.0,
                    },
                },
            }),
        ),
    ] {
        let err =
            validator(&payload).expect_err("whitespace thread topology scriptUrl should fail");
        assert_eq!(
            err,
            "threadTopology liveInstances[0] scriptUrl must be a non-empty, non-whitespace string",
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_whitespace_padded_thread_topology_script_url() {
    let padded_thread_topology = json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": " https://e.co/worker.js ",
            "postedMessages": [],
            "postedSharedBuffers": [],
            "wasTerminated": false,
        }],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": padded_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": padded_thread_topology,
                "coverage": {
                    "mode": "function",
                    "files": [],
                    "summary": {
                        "functionsTotal": 4,
                        "functionsCovered": 3,
                        "functionsMissed": 1,
                        "coveragePercent": 75.0,
                    },
                },
            }),
        ),
    ] {
        let err = validator(&payload)
            .expect_err("whitespace-padded thread topology scriptUrl should fail");
        assert!(err.contains("scriptUrl"), "{kind} error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_non_url_thread_topology_script_url() {
    let malformed_thread_topology = json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": "worker.js",
            "postedMessages": [],
            "postedSharedBuffers": [],
            "wasTerminated": false,
        }],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": malformed_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": malformed_thread_topology,
                "coverage": {
                    "mode": "function",
                    "files": [],
                    "summary": {
                        "functionsTotal": 4,
                        "functionsCovered": 3,
                        "functionsMissed": 1,
                        "coveragePercent": 75.0,
                    },
                },
            }),
        ),
    ] {
        let err = validator(&payload).expect_err("non-URL thread topology scriptUrl should fail");
        assert_eq!(
            err,
            "threadTopology liveInstances[0] scriptUrl must be a valid absolute URL, got worker.js",
            "{kind} error: {err}"
        );
    }
}

#[test]
fn merge_thread_topology_snapshot_values_renumbers_and_orders_live_instances() {
    let mut target = json!({
        "totalInstances": 2,
        "terminatedInstances": 1,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": "https://e.co/worker-0.js",
            "postedMessages": [],
            "postedSharedBuffers": [],
            "wasTerminated": false,
        }],
    });
    let source = json!({
        "totalInstances": 2,
        "terminatedInstances": 0,
        "liveInstances": [
            {
                "instanceId": 0,
                "scriptUrl": "https://e.co/worker-1.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
            {
                "instanceId": 1,
                "scriptUrl": "https://e.co/worker-2.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
        ],
    });

    merge_thread_topology_snapshot_values(&mut target, &source);

    assert_eq!(target["totalInstances"], json!(4));
    assert_eq!(target["terminatedInstances"], json!(1));
    assert_eq!(
        target["liveInstances"]
            .as_array()
            .expect("live instances")
            .len(),
        3
    );
    assert_eq!(target["liveInstances"][0]["instanceId"], json!(0));
    assert_eq!(target["liveInstances"][1]["instanceId"], json!(1));
    assert_eq!(target["liveInstances"][2]["instanceId"], json!(2));

    validate_test_payload_value(&json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "threadTopology": target,
        "coverage": {
            "mode": "function",
            "files": [],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    }))
    .expect("merged thread topology should validate");
}

#[test]
fn validate_test_payload_value_rejects_malformed_coverage() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "branch",
            "files": [],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("unsupported coverage mode should fail validation");
    assert!(err.contains("coverage mode"), "unexpected error: {err}");
}

#[test]
fn validate_test_payload_value_rejects_malformed_coverage_files() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [1],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("malformed coverage file entry should fail validation");
    assert!(
        err.contains("coverage files[0] must be an object"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_test_payload_value_rejects_duplicate_coverage_file_rows() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [
                {
                    "file": "src/main.ts",
                    "functionsTotal": 4,
                    "functionsCovered": 3,
                    "functionsMissed": 1,
                },
                {
                    "file": "src/main.ts",
                    "functionsTotal": 4,
                    "functionsCovered": 3,
                    "functionsMissed": 1,
                }
            ],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("duplicate coverage file rows should fail validation");
    assert!(
        err.contains("coverage files[1].file must be unique"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_test_payload_value_rejects_malformed_coverage_summary() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [],
            "summary": {
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("malformed coverage summary should fail validation");
    assert!(
        err.contains("coverage summary is missing required key `functionsTotal`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_test_payload_value_rejects_unexpected_coverage_root_keys() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
            "metadata": {"kind": "extra"},
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("unexpected coverage root keys should fail validation");
    assert!(
        err.contains("coverage contains unexpected key `metadata`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_test_payload_value_rejects_unexpected_coverage_row_keys() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [
                {
                    "file": "src/main.ts",
                    "functionsTotal": 4,
                    "functionsCovered": 3,
                    "functionsMissed": 1,
                    "metadata": true,
                }
            ],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("unexpected coverage row keys should fail validation");
    assert!(
        err.contains("coverage files[0] contains unexpected key `metadata`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_test_payload_value_rejects_unexpected_coverage_summary_keys() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
                "metadata": {"kind": "extra"},
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("unexpected coverage summary keys should fail validation");
    assert!(
        err.contains("coverage summary contains unexpected key `metadata`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_effects_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": ["wasm-threads"],
            "compatFeatures": [],
        },
        "entryPoints": ["src/main.ts"],
        "effects": [{
            "kind": "Network.Fetch",
            "locations": [{
                "file": "src/main.ts",
                "line": 12,
                "column": 3,
                "function": "main",
            }],
        }],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    validate_effects_payload_value(&value).expect("effects payload should validate");
}

#[test]
fn validate_effects_payload_value_rejects_whitespace_effect_kind() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": ["src/main.ts"],
        "effects": [{
            "kind": "   ",
            "locations": [{
                "file": "src/main.ts",
                "line": 12,
                "column": 3,
            }],
        }],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("whitespace effect kind should fail validation");
    assert!(err.contains("effects[0] kind"), "unexpected error: {err}");
    assert!(
        err.contains("non-empty, non-whitespace string"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_effects_payload_value_rejects_invalid_effect_locations() {
    for (field, location, expected_fragment) in [
        (
            "file",
            json!({"file": "   ", "line": 12, "column": 3, "function": "main"}),
            "non-empty, non-whitespace string",
        ),
        (
            "line",
            json!({"file": "src/main.ts", "line": 0, "column": 3, "function": "main"}),
            "line must be a positive integer",
        ),
        (
            "column",
            json!({"file": "src/main.ts", "line": 12, "column": 0, "function": "main"}),
            "column must be a positive integer",
        ),
        (
            "function",
            json!({"file": "src/main.ts", "line": 12, "column": 3, "function": "   "}),
            "non-empty, non-whitespace string",
        ),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["src/main.ts"],
            "effects": [{
                "kind": "Network.Fetch",
                "locations": [location],
            }],
            "dynamicEffects": false,
            "dynamicReasons": [],
        });

        let err = validate_effects_payload_value(&value)
            .expect_err("invalid effect location should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(err.contains(expected_fragment), "unexpected error: {err}");
    }
}

#[test]
fn validate_effects_payload_value_rejects_whitespace_analysis_context_api_surface() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "   ",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": [],
        "effects": [],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("whitespace analysisContext apiSurface should fail validation");
    assert!(
        err.contains("analysisContext apiSurface"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("non-empty, non-whitespace string"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_effects_payload_value_rejects_whitespace_analysis_context_sets() {
    for (field, analysis_context, expected_fragment) in [
        (
            "runtimeProfiles",
            json!({
                "apiSurface": "browser",
                "runtimeProfiles": ["   "],
                "compatFeatures": [],
            }),
            "non-empty, non-whitespace string",
        ),
        (
            "runtimeProfiles",
            json!({
                "apiSurface": "browser",
                "runtimeProfiles": [" wasm-threads "],
                "compatFeatures": [],
            }),
            "leading or trailing whitespace",
        ),
        (
            "compatFeatures",
            json!({
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": ["\n"],
            }),
            "non-empty, non-whitespace string",
        ),
        (
            "compatFeatures",
            json!({
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": [" eval "],
            }),
            "leading or trailing whitespace",
        ),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "analysisContext": analysis_context,
            "entryPoints": [],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        });

        let err = validate_effects_payload_value(&value)
            .expect_err("whitespace analysisContext set item should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(err.contains(expected_fragment), "unexpected error: {err}");
    }
}

#[test]
fn validate_effects_payload_value_rejects_whitespace_entry_points() {
    for (invalid_value, expected_fragment) in [
        ("   ", "non-empty, non-whitespace string"),
        (" src/main.ts ", "leading or trailing whitespace"),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": [invalid_value],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        });

        let err = validate_effects_payload_value(&value)
            .expect_err("whitespace entryPoints should fail validation");
        assert!(err.contains("entryPoints[0]"), "unexpected error: {err}");
        assert!(err.contains(expected_fragment), "unexpected error: {err}");
    }
}

#[test]
fn validate_effects_payload_value_rejects_whitespace_dynamic_reasons() {
    for (invalid_value, expected_fragment) in [
        ("   ", "non-empty, non-whitespace string"),
        (" eval ", "leading or trailing whitespace"),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": [],
            "effects": [],
            "dynamicEffects": true,
            "dynamicReasons": [invalid_value],
        });

        let err = validate_effects_payload_value(&value)
            .expect_err("whitespace dynamicReasons should fail validation");
        assert!(err.contains("dynamicReasons[0]"), "unexpected error: {err}");
        assert!(err.contains(expected_fragment), "unexpected error: {err}");
    }
}

#[test]
fn validate_effects_payload_value_rejects_duplicate_entry_points() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": ["src/main.ts", "src/main.ts"],
        "effects": [],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("duplicate entryPoints should fail validation");
    assert!(
        err.contains("entryPoints") && err.contains("duplicate item `src/main.ts`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_effects_payload_value_rejects_unexpected_keys() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": [],
        "effects": [],
        "dynamicEffects": false,
        "dynamicReasons": [],
        "unexpected": true,
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("unexpected effects keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_effects_payload_value_rejects_unexpected_nested_keys() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
            "unexpected": true,
        },
        "entryPoints": [],
        "effects": [{
            "kind": "Network.Fetch",
            "locations": [{
                "file": "src/main.ts",
                "line": 12,
                "column": 3,
                "function": "main",
                "unexpected": true,
            }],
        }],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("unexpected nested effects keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_effects_payload_value_rejects_dynamic_reasons_when_dynamic_effects_is_false() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": [],
        "effects": [],
        "dynamicEffects": false,
        "dynamicReasons": ["eval"],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("non-empty dynamicReasons should fail when dynamicEffects is false");
    assert!(
        err.contains("dynamicReasons") && err.contains("dynamicEffects is false"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_effects_payload_value_rejects_unsorted_dynamic_reasons() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": [],
        "effects": [],
        "dynamicEffects": true,
        "dynamicReasons": ["proxy-traps", "eval"],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("unsorted dynamicReasons should fail validation");
    assert!(
        err.contains("deduplicated and sorted in lexical order"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_effects_payload_value_rejects_duplicate_analysis_context_sets() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": ["wasm-threads", "wasm-threads"],
            "compatFeatures": ["eval", "eval"],
        },
        "entryPoints": [],
        "effects": [],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("duplicate analysisContext set items should fail validation");
    assert!(
        err.contains("runtimeProfiles") || err.contains("compatFeatures"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_package_effects_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    validate_package_effects_payload_value(&value)
        .expect("package-effects payload should validate");
}

#[test]
fn validate_package_effects_payload_value_rejects_non_single_root_reports() {
    for entry_points in [json!([]), json!(["semver", "semver-helpers"])] {
        let value = json!({
            "schemaVersion": 1,
            "package": {
                "name": "semver",
                "version": "7.6.3",
                "registry": "npm",
            },
            "report": {
                "schemaVersion": 1,
                "analysisContext": {
                    "apiSurface": "default",
                    "runtimeProfiles": [],
                    "compatFeatures": [],
                },
                "entryPoints": entry_points,
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });

        let err = validate_package_effects_payload_value(&value)
            .expect_err("non-single-root package-effects payloads should fail validation");
        assert!(
            err.contains("entryPoints must contain exactly one item"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_package_effects_payload_value_rejects_unexpected_keys() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
        "unexpected": true,
    });

    let err = validate_package_effects_payload_value(&value)
        .expect_err("unexpected package-effects keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_package_effects_payload_value_rejects_unexpected_nested_keys() {
    let invalid_package = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
            "unexpected": true,
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    let err = validate_package_effects_payload_value(&invalid_package)
        .expect_err("unexpected package keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");

    let invalid_report = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
            "unexpected": true,
        },
    });

    let err = validate_package_effects_payload_value(&invalid_report)
        .expect_err("unexpected report keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_package_effects_payload_value_rejects_unexpected_analysis_context_keys() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
                "unexpected": true,
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    let err = validate_package_effects_payload_value(&value)
        .expect_err("unexpected analysisContext keys should fail validation");
    assert!(err.contains("analysisContext"), "unexpected error: {err}");
    assert!(
        err.contains("unexpected key `unexpected`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_package_effects_payload_value_rejects_non_string_package_coordinate_fields() {
    for (field, value) in [
        ("name", json!(1)),
        ("version", json!(false)),
        ("registry", json!(["npm"])),
    ] {
        let payload = json!({
            "schemaVersion": 1,
            "package": {
                "name": "semver",
                "version": "7.6.3",
                "registry": "npm",
            },
            "report": {
                "schemaVersion": 1,
                "analysisContext": {
                    "apiSurface": "default",
                    "runtimeProfiles": [],
                    "compatFeatures": [],
                },
                "entryPoints": ["semver"],
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });
        let mut payload = payload
            .as_object()
            .expect("package-effects payload object")
            .clone();
        payload
            .get_mut("package")
            .expect("package coordinate")
            .as_object_mut()
            .expect("package coordinate object")
            .insert(field.to_string(), value);

        let err = validate_package_effects_payload_value(&serde_json::Value::Object(payload))
            .expect_err("invalid package coordinate field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_package_effects_payload_value_rejects_whitespace_package_coordinate_fields() {
    for (field, value) in [
        ("name", json!("   ")),
        ("version", json!("\n")),
        ("registry", json!("\t")),
    ] {
        let payload = json!({
            "schemaVersion": 1,
            "package": {
                "name": "semver",
                "version": "7.6.3",
                "registry": "npm",
            },
            "report": {
                "schemaVersion": 1,
                "analysisContext": {
                    "apiSurface": "default",
                    "runtimeProfiles": [],
                    "compatFeatures": [],
                },
                "entryPoints": ["semver"],
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });
        let mut payload = payload
            .as_object()
            .expect("package-effects payload object")
            .clone();
        payload
            .get_mut("package")
            .expect("package coordinate")
            .as_object_mut()
            .expect("package coordinate object")
            .insert(field.to_string(), value);

        let err = validate_package_effects_payload_value(&serde_json::Value::Object(payload))
            .expect_err("whitespace package coordinate field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_package_effects_payload_value_rejects_duplicate_analysis_context_sets() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": ["wasm-threads", "wasm-threads"],
                "compatFeatures": ["eval", "eval"],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    let err = validate_package_effects_payload_value(&value)
        .expect_err("duplicate analysisContext set items should fail validation");
    assert!(
        err.contains("runtimeProfiles") || err.contains("compatFeatures"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_package_effects_payload_value_rejects_whitespace_analysis_context_sets() {
    for (field, analysis_context) in [
        (
            "runtimeProfiles",
            json!({
                "apiSurface": "default",
                "runtimeProfiles": ["   "],
                "compatFeatures": [],
            }),
        ),
        (
            "compatFeatures",
            json!({
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": ["\n"],
            }),
        ),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "package": {
                "name": "semver",
                "version": "7.6.3",
                "registry": "npm",
            },
            "report": {
                "schemaVersion": 1,
                "analysisContext": analysis_context,
                "entryPoints": ["semver"],
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });

        let err = validate_package_effects_payload_value(&value)
            .expect_err("whitespace analysisContext set item should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_init_payload_value_rejects_unexpected_keys() {
    let value = json!({
        "root": "/workspace/example",
        "manifestPath": "/workspace/example/kali.json",
        "sourcePath": "/workspace/example/src/main.ts",
        "library": false,
        "extra": true,
    });

    let err = validate_init_payload_value(&value).expect_err("unexpected init keys should fail");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_init_payload_value_rejects_non_string_and_non_boolean_fields() {
    for (field, value) in [
        ("root", json!(1)),
        ("manifestPath", json!(false)),
        ("sourcePath", json!(["src/main.ts"])),
        ("library", json!("yes")),
    ] {
        let payload = json!({
            "root": "/workspace/example",
            "manifestPath": "/workspace/example/kali.json",
            "sourcePath": "/workspace/example/src/main.ts",
            "library": false,
        });
        let mut payload = payload.as_object().expect("init payload object").clone();
        payload.insert(field.to_string(), value);

        let err = validate_init_payload_value(&serde_json::Value::Object(payload))
            .expect_err("invalid init payload field should fail");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_install_payload_value_rejects_non_string_entries() {
    let value = json!({
        "manifestPath": "/workspace/example/kali.json",
        "lockPath": null,
        "installed": ["semver", 1],
        "updated": [],
        "removed": [],
    });

    let err =
        validate_install_payload_value(&value).expect_err("non-string install entries should fail");
    assert!(err.contains("installed[1]"), "unexpected error: {err}");
}

#[test]
fn validate_install_payload_value_rejects_non_string_manifest_and_lock_paths() {
    for (field, value) in [
        ("manifestPath", json!(1)),
        ("lockPath", json!(["lock.json"])),
    ] {
        let payload = json!({
            "manifestPath": "/workspace/example/kali.json",
            "lockPath": null,
            "installed": [],
            "updated": [],
            "removed": [],
        });
        let mut payload = payload.as_object().expect("install payload object").clone();
        payload.insert(field.to_string(), value);

        let err = validate_install_payload_value(&serde_json::Value::Object(payload))
            .expect_err("invalid install payload path should fail");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_package_audit_payload_value_accepts_null() {
    validate_package_audit_payload_value(&serde_json::Value::Null)
        .expect("package-audit payload should validate");
}

#[test]
fn validate_package_audit_payload_value_rejects_non_null_payloads() {
    let value = json!({"unexpected": true});

    let err = validate_package_audit_payload_value(&value)
        .expect_err("non-null package-audit payloads should fail");
    assert!(err.contains("must be null"), "unexpected error: {err}");
}

#[test]
fn validate_doctor_payload_value_rejects_empty_browser_harness_command() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": [],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("empty browser harness command should fail");
    assert!(
        err.contains("browserHarness command must contain at least one item"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_empty_or_whitespace_browser_harness_command_item() {
    for command_item in ["", "   "] {
        let value = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": "env",
                "override": "node --test",
                "command": ["node", command_item],
                "executable": "node",
                "args": [command_item],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": "real browser host",
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&value)
            .expect_err("empty browser harness command item should fail");
        assert!(
            err.contains("browserHarness command[1]"),
            "unexpected error: {err}"
        );
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_rejects_empty_or_whitespace_browser_harness_args_item() {
    for args_item in ["", "   "] {
        let value = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": "env",
                "override": "node --test",
                "command": ["node", "--test"],
                "executable": "node",
                "args": [args_item],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": "real browser host",
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&value)
            .expect_err("empty browser harness args item should fail");
        assert!(
            err.contains("browserHarness args[0]"),
            "unexpected error: {err}"
        );
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_rejects_unexpected_browser_harness_keys() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
            "unexpected": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("unexpected browserHarness keys should fail");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_doctor_payload_value_rejects_empty_supported_commands() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": [],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err =
        validate_doctor_payload_value(&value).expect_err("empty supported commands should fail");
    assert!(
        err.contains("supportedCommands must contain at least one item"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_unsupported_supported_commands_item() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "build"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("unsupported supportedCommands item should fail");
    assert!(
        err.contains("supportedCommands[1]"),
        "unexpected error: {err}"
    );
    assert!(err.contains("run` or `test"), "unexpected error: {err}");
}

#[test]
fn validate_doctor_payload_value_rejects_whitespace_supported_commands_item() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "   "],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("whitespace supportedCommands item should fail");
    assert!(
        err.contains("supportedCommands[1]"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("non-empty, non-whitespace string"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_duplicate_supported_commands() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": [" run ", "run"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("duplicate supportedCommands items should fail");
    assert!(
        err.contains("duplicate item `run`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_out_of_order_supported_commands() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["test", "run"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("out-of-order supportedCommands items should fail");
    assert!(
        err.contains("exactly [`run`, `test`]"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_duplicate_diagnostic_notes() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                " supported browser runtime commands: run, test ",
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err =
        validate_doctor_payload_value(&value).expect_err("duplicate diagnostic notes should fail");
    assert!(err.contains("duplicate item"), "unexpected error: {err}");
    assert!(err.contains("diagnosticNotes"), "unexpected error: {err}");
}

#[test]
fn validate_doctor_payload_value_rejects_out_of_order_diagnostic_notes() {
    let expected_notes = BrowserRuntimeContract::diagnostic_notes();
    let expected_notes_message = format!(
        "[{}]",
        expected_notes
            .iter()
            .map(|note| format!("`{note}`"))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "supported browser runtime commands: run, test",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("out-of-order diagnostic notes should fail");
    assert!(err.contains("diagnosticNotes"), "unexpected error: {err}");
    assert!(err.contains("exactly"), "unexpected error: {err}");
    assert!(
        err.contains(&expected_notes_message),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_empty_diagnostic_notes() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [],
        }
    });

    let err =
        validate_doctor_payload_value(&value).expect_err("empty diagnostic notes should fail");
    assert!(err.contains("diagnosticNotes"), "unexpected error: {err}");
}

#[test]
fn validate_doctor_payload_value_rejects_empty_or_whitespace_diagnostic_note_items() {
    for note in ["", " \n\t "] {
        let value = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": "auto",
                "override": null,
                "command": ["node", "--test"],
                "executable": "node",
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": "real browser host",
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    note,
                    "browser runtime host description: real browser host"
                ],
            }
        });

        let err = validate_doctor_payload_value(&value)
            .expect_err("empty or whitespace diagnostic note item should fail");
        assert!(
            err.contains("diagnosticNotes[1]"),
            "unexpected error: {err}"
        );
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_rejects_diagnostic_notes_drift() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "auto",
            "override": null,
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime host description: real browser host"
            ],
        }
    });

    let err = validate_doctor_payload_value(&value).expect_err("diagnosticNotes drift should fail");
    assert!(err.contains("diagnosticNotes"), "unexpected error: {err}");
    assert!(err.contains("exactly"), "unexpected error: {err}");
}

#[test]
fn validate_doctor_payload_value_rejects_unexpected_keys() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
            "unexpected": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ],
        }
    });

    let err =
        validate_doctor_payload_value(&value).expect_err("unexpected payload keys should fail");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_doctor_payload_value_rejects_unexpected_browser_runtime_contract_keys() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ],
            "unexpected": true,
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("unexpected browserRuntimeContract keys should fail");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_doctor_payload_value_rejects_non_string_browser_harness_and_runtime_contract_items() {
    for (field, payload, expected_fragment) in [
        (
            "browserHarness command",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", 42],
                    "executable": "node",
                    "args": ["--test"],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": "browser-requested",
                    "hostDescription": "real browser host",
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", "test"],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "command[1]",
        ),
        (
            "browserHarness args",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", "--test"],
                    "executable": "node",
                    "args": ["--test", false],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": "browser-requested",
                    "hostDescription": "real browser host",
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", "test"],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "args[1]",
        ),
        (
            "browserRuntimeContract supportedCommands",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", "--test"],
                    "executable": "node",
                    "args": ["--test"],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": "browser-requested",
                    "hostDescription": "real browser host",
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", null],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "supportedCommands[1]",
        ),
        (
            "browserRuntimeContract supportedCommands whitespace",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", "--test"],
                    "executable": "node",
                    "args": ["--test"],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": "browser-requested",
                    "hostDescription": "real browser host",
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", "   "],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "non-empty, non-whitespace string",
        ),
        (
            "browserRuntimeContract supportedCommands empty",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", "--test"],
                    "executable": "node",
                    "args": ["--test"],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": "browser-requested",
                    "hostDescription": "real browser host",
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", ""],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "non-empty, non-whitespace string",
        ),
        (
            "browserRuntimeContract diagnosticNotes whitespace",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", "--test"],
                    "executable": "node",
                    "args": ["--test"],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": "browser-requested",
                    "hostDescription": "real browser host",
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", "test"],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        "   ",
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "non-empty, non-whitespace string",
        ),
        (
            "browserRuntimeContract diagnosticNotes empty",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", "--test"],
                    "executable": "node",
                    "args": ["--test"],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": "browser-requested",
                    "hostDescription": "real browser host",
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", "test"],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        "",
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "non-empty, non-whitespace string",
        ),
        (
            "browserRuntimeContract diagnosticNotes",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", "--test"],
                    "executable": "node",
                    "args": ["--test"],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": "browser-requested",
                    "hostDescription": "real browser host",
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", "test"],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        17,
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "diagnosticNotes[1]",
        ),
        (
            "browserRuntimeContract hostLabel",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", "--test"],
                    "executable": "node",
                    "args": ["--test"],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": false,
                    "hostDescription": "real browser host",
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", "test"],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "hostLabel",
        ),
        (
            "browserRuntimeContract hostDescription",
            json!({
                "browserHarness": {
                    "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                    "source": "env",
                    "override": "node --test",
                    "command": ["node", "--test"],
                    "executable": "node",
                    "args": ["--test"],
                    "executableAvailable": true,
                },
                "browserRuntimeContract": {
                    "hostLabel": "browser-requested",
                    "hostDescription": null,
                    "hostDescriptionNote": "browser runtime host description: real browser host",
                    "supportedCommands": ["run", "test"],
                    "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                    "diagnosticNotes": [
                        "supported browser runtime commands: run, test",
                        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "hostDescription",
        ),
    ] {
        let err = validate_doctor_payload_value(&payload)
            .expect_err("non-string doctor array entries should fail");
        assert!(
            err.contains(expected_fragment),
            "{field} error mismatch: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_rejects_non_string_host_description_note() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": 42,
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("non-string hostDescriptionNote should fail");
    assert!(
        err.contains("hostDescriptionNote"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_empty_or_whitespace_host_description_note() {
    for value in ["", " \n\t "] {
        let payload = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": "env",
                "override": "node --test",
                "command": ["node", "--test"],
                "executable": "node",
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": "real browser host",
                "hostDescriptionNote": value,
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&payload)
            .expect_err("empty hostDescriptionNote should fail");
        assert!(
            err.contains("browserRuntimeContract hostDescriptionNote"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_rejects_wrong_host_label() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-runtime",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value).expect_err("wrong hostLabel should fail");
    assert!(
        err.contains("browserRuntimeContract hostLabel"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_accepts_trimmed_browser_runtime_contract_labels() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": " browser-requested ",
            "hostDescription": " real browser host ",
            "hostDescriptionNote": " browser runtime host description: real browser host ",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    validate_doctor_payload_value(&value)
        .expect("trimmed browser runtime contract labels should validate");
}

#[test]
fn validate_doctor_payload_value_accepts_trimmed_browser_runtime_contract_array_items() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": [" run ", " test "],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                " supported browser runtime commands: run, test ",
                " browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work ",
                " browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness ",
                " browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid ",
                " browser runtime host description: real browser host "
            ]
        }
    });

    validate_doctor_payload_value(&value)
        .expect("trimmed browser runtime contract array items should validate");
}

#[test]
fn validate_doctor_payload_value_accepts_trimmed_browser_runtime_contract_fields() {
    let mut browser_runtime_contract = browser_runtime_contract_value();
    let contract = browser_runtime_contract
        .as_object_mut()
        .expect("browser runtime contract object");
    contract.insert("hostLabel".to_string(), json!(" browser-requested "));
    contract.insert("hostDescription".to_string(), json!(" real browser host "));
    contract.insert(
        "hostDescriptionNote".to_string(),
        json!(" browser runtime host description: real browser host "),
    );
    contract.insert("supportedCommands".to_string(), json!([" run ", " test "]));
    contract.insert(
        "diagnosticHint".to_string(),
        json!(" Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work. "),
    );
    contract.insert(
        "diagnosticNotes".to_string(),
        json!([
            " supported browser runtime commands: run, test ",
            " browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work ",
            " browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness ",
            " browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid ",
            " browser runtime host description: real browser host ",
        ]),
    );

    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": browser_runtime_contract,
    });

    validate_doctor_payload_value(&value)
        .expect("trimmed browser runtime contract fields should validate");
}

#[test]
fn validate_doctor_payload_value_accepts_trimmed_browser_runtime_contract_note_fields() {
    for (field, padded_value) in [
        (
            "hostDescriptionNote",
            " browser runtime host description: real browser host ",
        ),
        (
            "diagnosticHint",
            " Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work. ",
        ),
    ] {
        let mut browser_runtime_contract = browser_runtime_contract_value();
        let contract = browser_runtime_contract
            .as_object_mut()
            .expect("browser runtime contract object");
        contract.insert(field.to_string(), json!(padded_value));

        let value = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": "env",
                "override": "node --test",
                "command": ["node", "--test"],
                "executable": "node",
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": browser_runtime_contract,
        });

        validate_doctor_payload_value(&value)
            .expect("trimmed browser runtime contract note fields should validate");
    }
}

#[test]
fn validate_doctor_payload_value_rejects_empty_or_whitespace_host_label() {
    for host_label in ["", " \n\t "] {
        let value = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": "env",
                "override": "node --test",
                "command": ["node", "--test"],
                "executable": "node",
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": host_label,
                "hostDescription": "real browser host",
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&value)
            .expect_err("empty or whitespace hostLabel should fail");
        assert!(err.contains("hostLabel"), "unexpected error: {err}");
    }
}

#[test]
fn validate_doctor_payload_value_rejects_wrong_host_description_note() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err =
        validate_doctor_payload_value(&value).expect_err("wrong hostDescriptionNote should fail");
    assert!(
        err.contains("browser runtime host description: real browser host"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_empty_or_whitespace_host_description() {
    for host_description in ["", " \n\t "] {
        let value = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": "env",
                "override": "node --test",
                "command": ["node", "--test"],
                "executable": "node",
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": host_description,
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&value)
            .expect_err("empty or whitespace hostDescription should fail");
        assert!(
            err.contains("browserRuntimeContract hostDescription"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_accepts_trimmed_diagnostic_hint() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": " Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work. ",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    validate_doctor_payload_value(&value).expect("trimmed diagnosticHint should validate");
}

#[test]
fn validate_doctor_payload_value_rejects_empty_or_whitespace_diagnostic_hint() {
    for diagnostic_hint in ["", "   "] {
        let value = json!({
            "browserHarness": {
                "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
                "source": "env",
                "override": "node --test",
                "command": ["node", "--test"],
                "executable": "node",
                "args": ["--test"],
                "executableAvailable": true,
            },
            "browserRuntimeContract": {
                "hostLabel": "browser-requested",
                "hostDescription": "real browser host",
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": diagnostic_hint,
                "diagnosticNotes": [
                    "supported browser runtime commands: run, test",
                    "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                    "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                    "browser runtime host description: real browser host"
                ]
            }
        });

        let err = validate_doctor_payload_value(&value)
            .expect_err("empty or whitespace diagnosticHint should fail");
        assert!(err.contains("diagnosticHint"), "unexpected error: {err}");
    }
}

#[test]
fn validate_doctor_payload_value_rejects_diagnostic_hint_drift() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value).expect_err("diagnosticHint drift should fail");
    assert!(err.contains("diagnosticHint"), "unexpected error: {err}");
}

#[test]
fn validate_doctor_payload_value_rejects_non_string_diagnostic_hint() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_BUNDLE_HARNESS_COMMAND",
            "source": "env",
            "override": "node --test",
            "command": ["node", "--test"],
            "executable": "node",
            "args": ["--test"],
            "executableAvailable": true,
        },
        "browserRuntimeContract": {
            "hostLabel": "browser-requested",
            "hostDescription": "real browser host",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": false,
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err =
        validate_doctor_payload_value(&value).expect_err("non-string diagnosticHint should fail");
    assert!(err.contains("diagnosticHint"), "unexpected error: {err}");
}

#[test]
fn emitted_cli_envelopes_preserve_stdout_and_stderr_strings() {
    let value = emit_envelope_value(
        "doctor",
        false,
        json!([]),
        json!([]),
        serde_json::Value::Null,
        Some("stdout text".to_string()),
        Some("stderr text".to_string()),
        1,
    );

    validate_envelope_value(&value).expect("constructed envelope should validate");

    let object = value.as_object().expect("envelope object");
    assert_eq!(object["errors"], json!([]));
    assert_eq!(object["warnings"], json!([]));
    assert_eq!(object["stdout"], json!("stdout text"));
    assert_eq!(object["stderr"], json!("stderr text"));
    assert_eq!(object["exitCode"], json!(1));
}

#[test]
fn validate_envelope_value_rejects_wrong_top_level_shapes() {
    let wrong_schema_version = json!({
        "schemaVersion": 2,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
    });
    let err = validate_envelope_value(&wrong_schema_version)
        .expect_err("schema version drift should be rejected");
    assert!(err.contains("schemaVersion"), "unexpected error: {err}");

    let missing_key = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
    });
    let err = validate_envelope_value(&missing_key).expect_err("missing exitCode should fail");
    assert!(err.contains("exitCode"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_empty_or_whitespace_command() {
    for command in ["", "   "] {
        let value = json!({
            "schemaVersion": 1,
            "command": command,
            "success": true,
            "errors": [],
            "warnings": [],
            "payload": null,
            "stdout": null,
            "stderr": null,
            "exitCode": 0,
        });

        let err =
            validate_envelope_value(&value).expect_err("empty command should fail validation");
        assert!(err.contains("command"), "unexpected error: {err}");
        assert!(err.contains("non-empty"), "unexpected error: {err}");
    }
}

#[test]
fn validate_envelope_value_allows_schema_permitted_extension_keys() {
    let extended = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [{"phase": "parse", "milliseconds": 1}],
    });

    validate_envelope_value(&extended).expect("schema-permitted extension keys should validate");
}

#[test]
fn validate_envelope_value_rejects_unexpected_timing_keys() {
    let extended_timings = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [{
            "phase": "parse",
            "milliseconds": 1,
            "label": "warmup",
            "metadata": {"kind": "synthetic"},
        }],
    });

    let err = validate_envelope_value(&extended_timings)
        .expect_err("unexpected timing keys should fail validation");
    assert!(
        err.contains("timing") && err.contains("unexpected key `label`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_allows_schema_permitted_diagnostic_context_extensions() {
    let extended_context = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic context",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": [],
                "context": {
                    "origin": "config",
                    "configPath": "compilerOptions.apiSurface",
                    "requestedValue": {"apiSurface": "browser"},
                    "effectiveValue": ["browser"]
                }
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    validate_envelope_value(&extended_context)
        .expect("schema-permitted diagnostic context extensions should validate");
}

#[test]
fn validate_envelope_value_allows_arbitrary_diagnostic_context_value_shapes() {
    let arbitrary_context_shapes = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "diagnostic context values can be any JSON shape",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": [],
                "context": {
                    "origin": "source",
                    "requestedValue": ["browser", "deno"],
                    "effectiveValue": false
                }
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    validate_envelope_value(&arbitrary_context_shapes)
        .expect("arbitrary diagnostic context shapes should validate");
}

#[test]
fn validate_envelope_value_rejects_unexpected_diagnostic_context_extension_keys() {
    let unexpected_context_key = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "diagnostic context has an extra key",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": [],
                "context": {
                    "origin": "config",
                    "configPath": "compilerOptions.apiSurface",
                    "extensionKey": true
                }
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&unexpected_context_key)
        .expect_err("unexpected diagnostic context keys should fail validation");
    assert!(
        err.contains("diagnostic context") && err.contains("unexpected key `extensionKey`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_accepts_all_canonical_diagnostic_context_origins() {
    for (origin, context) in [
        (
            "cli",
            json!({
                "origin": "cli",
                "flag": "--api",
                "requestedValue": "browser",
                "effectiveValue": "browser",
            }),
        ),
        (
            "config",
            json!({
                "origin": "config",
                "configPath": "compilerOptions.apiSurface",
                "requestedValue": {"apiSurface": "browser"},
                "effectiveValue": "browser",
            }),
        ),
        (
            "default",
            json!({
                "origin": "default",
                "requestedValue": null,
                "effectiveValue": "deno",
            }),
        ),
        (
            "source",
            json!({
                "origin": "source",
                "requestedValue": ["browser", "deno"],
                "effectiveValue": {"apiSurface": "browser"},
            }),
        ),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "command": "doctor",
            "success": false,
            "errors": [
                {
                    "severity": "error",
                    "code": "E5508",
                    "message": format!("diagnostic context origin {origin} should validate"),
                    "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                    "labels": [],
                    "related": [],
                    "fix": null,
                    "notes": [],
                    "context": context,
                }
            ],
            "warnings": [],
            "payload": null,
            "stdout": null,
            "stderr": null,
            "exitCode": 1,
        });

        validate_envelope_value(&value)
            .expect("canonical diagnostic context origin should validate");
    }
}

#[test]
fn validate_envelope_value_rejects_unexpected_suggested_fix_keys() {
    let extended_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix extension",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 1},
                            "end": {"file": "src/main.ts", "line": 1, "column": 2},
                            "newText": "console.log(1);"
                        }
                    ],
                    "metadata": {"origin": "autofix"}
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&extended_fix)
        .expect_err("unexpected suggested fix keys should fail validation");
    assert!(
        err.contains("suggested fix") && err.contains("unexpected key `metadata`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_non_object_fix() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad fix shape",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": [],
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix)
        .expect_err("non-object suggested fix should fail validation");
    assert!(err.contains("suggested fix"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_malformed_timings() {
    let invalid_timings = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": ["parse"],
    });
    let err = validate_envelope_value(&invalid_timings)
        .expect_err("non-object timings items should fail validation");
    assert!(err.contains("timings[0]"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_malformed_timing_objects() {
    let missing_phase = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [{"milliseconds": 1}],
    });
    let err = validate_envelope_value(&missing_phase)
        .expect_err("timings missing phase should fail validation");
    assert!(err.contains("phase"), "unexpected error: {err}");

    let invalid_phase = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [{"phase": 1, "milliseconds": 1}],
    });
    let err = validate_envelope_value(&invalid_phase)
        .expect_err("timings with numeric phase should fail validation");
    assert!(err.contains("phase"), "unexpected error: {err}");

    let invalid_milliseconds = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [{"phase": "parse", "milliseconds": "fast"}],
    });
    let err = validate_envelope_value(&invalid_milliseconds)
        .expect_err("timings with string milliseconds should fail validation");
    assert!(err.contains("milliseconds"), "unexpected error: {err}");

    let unexpected_key = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [{"phase": "parse", "milliseconds": 1, "metadata": "extra"}],
    });
    let err = validate_envelope_value(&unexpected_key)
        .expect_err("timings with unexpected keys should fail validation");
    assert!(
        err.contains("timing") && err.contains("unexpected key `metadata`"),
        "unexpected error: {err}"
    );

    let negative_milliseconds = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [{"phase": "parse", "milliseconds": -1}],
    });
    let err = validate_envelope_value(&negative_milliseconds)
        .expect_err("timings with negative milliseconds should fail validation");
    assert!(
        err.contains("finite non-negative number"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_duplicate_timing_phases() {
    let duplicate_timings = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [
            {"phase": "parse", "milliseconds": 1},
            {"phase": "parse", "milliseconds": 2},
        ],
    });

    let err = validate_envelope_value(&duplicate_timings)
        .expect_err("duplicate timing phases should fail validation");
    assert!(
        err.contains("duplicates phase `parse`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_out_of_order_timing_phases() {
    let out_of_order_timings = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [
            {"phase": "typecheck", "milliseconds": 1},
            {"phase": "parse", "milliseconds": 2},
        ],
    });

    let err = validate_envelope_value(&out_of_order_timings)
        .expect_err("out-of-order timing phases should fail validation");
    assert!(
        err.contains("canonical phase order") && err.contains("typecheck") && err.contains("parse"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_empty_timing_phases() {
    let empty_phase = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [{"phase": "", "milliseconds": 1}],
    });

    let err = validate_envelope_value(&empty_phase)
        .expect_err("empty timing phases should fail validation");
    assert!(
        err.contains("timing phase must be a non-empty string"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_whitespace_only_timing_phases() {
    let whitespace_phase = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [{"phase": "   ", "milliseconds": 1}],
    });

    let err = validate_envelope_value(&whitespace_phase)
        .expect_err("whitespace-only timing phases should fail validation");
    assert!(
        err.contains("timing phase must be a non-empty string"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_fractional_exit_code() {
    let invalid_exit_code = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1.5,
    });
    let err = validate_envelope_value(&invalid_exit_code)
        .expect_err("fractional exitCode should fail validation");
    assert!(err.contains("exitCode"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_negative_exit_code() {
    let invalid_exit_code = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": -1,
    });
    let err = validate_envelope_value(&invalid_exit_code)
        .expect_err("negative exitCode should fail validation");
    assert!(err.contains("exitCode"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_malformed_diagnostics() {
    let invalid_diagnostic = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [
                    {
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                        "message": "label",
                        "style": "primary"
                    }
                ],
                "related": [],
                "fix": null,
                "notes": [],
                "context": {"origin": "cli", "flag": "--api"}
            },
            {
                "severity": "error",
                "code": "E5508",
                "message": "missing span",
                "labels": [],
                "related": [],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_diagnostic)
        .expect_err("malformed diagnostic should fail validation");
    assert!(err.contains("errors[1]"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_malformed_warning_diagnostics() {
    let invalid_warning = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [
            {
                "severity": "warning",
                "code": "W5501",
                "message": 42,
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": []
            }
        ],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
    });

    let err = validate_envelope_value(&invalid_warning)
        .expect_err("malformed warning should fail validation");
    assert!(err.contains("warnings[0]"), "unexpected error: {err}");
    assert!(err.contains("message"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_allows_null_diagnostic_help() {
    let valid_help = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "optional diagnostic help",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "help": null,
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    validate_envelope_value(&valid_help).expect("null diagnostic help should validate");
}

#[test]
fn validate_envelope_value_rejects_diagnostic_with_non_string_help() {
    let invalid_help = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic help",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "help": 42,
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_help)
        .expect_err("non-string diagnostic help should fail validation");
    assert!(err.contains("help"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_diagnostic_notes_that_are_not_string_arrays() {
    let invalid_notes_item = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic notes",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "help": null,
                "fix": null,
                "notes": ["ok", 42]
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });
    let err = validate_envelope_value(&invalid_notes_item)
        .expect_err("non-string diagnostic note should fail validation");
    assert!(err.contains("notes"), "unexpected error: {err}");

    let invalid_notes_shape = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic notes shape",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "help": null,
                "fix": null,
                "notes": null
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });
    let err = validate_envelope_value(&invalid_notes_shape)
        .expect_err("non-array diagnostic notes should fail validation");
    assert!(err.contains("notes"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_malformed_diagnostic_labels() {
    let invalid_label = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic label",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [
                    {
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                        "message": "label",
                        "style": "tertiary"
                    }
                ],
                "related": [],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_label)
        .expect_err("malformed diagnostic label should fail validation");
    assert!(err.contains("labels[0]"), "unexpected error: {err}");
    assert!(err.contains("style"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_malformed_related_items() {
    let invalid_related = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad related item",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [
                    {
                        "message": "follow-up note"
                    }
                ],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_related)
        .expect_err("malformed related item should fail validation");
    assert!(err.contains("related[0]"), "unexpected error: {err}");
    assert!(err.contains("span"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_related_item_with_non_string_message() {
    let invalid_related_message = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad related item message",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [
                    {
                        "message": 42,
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2}
                    }
                ],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_related_message)
        .expect_err("non-string related item message should fail validation");
    assert!(err.contains("related[0]"), "unexpected error: {err}");
    assert!(err.contains("message"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_unexpected_label_extensions() {
    let invalid_label_extension = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic label extension",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [
                    {
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                        "message": "label",
                        "style": "primary",
                        "metadata": {"kind": "extra"}
                    }
                ],
                "related": [],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_label_extension)
        .expect_err("unexpected label extensions should fail validation");
    assert!(err.contains("labels[0]"), "unexpected error: {err}");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_unexpected_related_item_extensions() {
    let invalid_related_extension = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad related item extension",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [
                    {
                        "message": "follow-up note",
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                        "metadata": {"kind": "extra"}
                    }
                ],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_related_extension)
        .expect_err("unexpected related item extensions should fail validation");
    assert!(err.contains("related[0]"), "unexpected error: {err}");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_unexpected_label_source_span_keys() {
    let invalid_label_span = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic label span",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [
                    {
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2, "metadata": true},
                        "message": "label",
                        "style": "primary"
                    }
                ],
                "related": [],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_label_span)
        .expect_err("unexpected label source-span keys should fail validation");
    assert!(err.contains("labels[0]"), "unexpected error: {err}");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_unexpected_related_item_source_span_keys() {
    let invalid_related_span = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad related item span",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [
                    {
                        "message": "follow-up note",
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2, "metadata": true}
                    }
                ],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_related_span)
        .expect_err("unexpected related item source-span keys should fail validation");
    assert!(err.contains("related[0]"), "unexpected error: {err}");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_empty_nested_source_span_files() {
    for (context, payload) in [
        (
            "labels[0]",
            json!({
                "schemaVersion": 1,
                "command": "doctor",
                "success": false,
                "errors": [
                    {
                        "severity": "error",
                        "code": "E5508",
                        "message": "bad label span file",
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                        "labels": [
                            {
                                "span": {"file": "", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                                "message": "label",
                                "style": "primary"
                            }
                        ],
                        "related": [],
                        "fix": null,
                        "notes": []
                    }
                ],
                "warnings": [],
                "payload": null,
                "stdout": null,
                "stderr": null,
                "exitCode": 1,
            }),
        ),
        (
            "labels[0]",
            json!({
                "schemaVersion": 1,
                "command": "doctor",
                "success": false,
                "errors": [
                    {
                        "severity": "error",
                        "code": "E5508",
                        "message": "bad label span file",
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                        "labels": [
                            {
                                "span": {"file": "   ", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                                "message": "label",
                                "style": "primary"
                            }
                        ],
                        "related": [],
                        "fix": null,
                        "notes": []
                    }
                ],
                "warnings": [],
                "payload": null,
                "stdout": null,
                "stderr": null,
                "exitCode": 1,
            }),
        ),
        (
            "related[0]",
            json!({
                "schemaVersion": 1,
                "command": "doctor",
                "success": false,
                "errors": [
                    {
                        "severity": "error",
                        "code": "E5508",
                        "message": "bad related span file",
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                        "labels": [],
                        "related": [
                            {
                                "message": "follow-up note",
                                "span": {"file": "", "line": 1, "column": 1, "endLine": 1, "endColumn": 2}
                            }
                        ],
                        "fix": null,
                        "notes": []
                    }
                ],
                "warnings": [],
                "payload": null,
                "stdout": null,
                "stderr": null,
                "exitCode": 1,
            }),
        ),
        (
            "related[0]",
            json!({
                "schemaVersion": 1,
                "command": "doctor",
                "success": false,
                "errors": [
                    {
                        "severity": "error",
                        "code": "E5508",
                        "message": "bad related span file",
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                        "labels": [],
                        "related": [
                            {
                                "message": "follow-up note",
                                "span": {"file": "   ", "line": 1, "column": 1, "endLine": 1, "endColumn": 2}
                            }
                        ],
                        "fix": null,
                        "notes": []
                    }
                ],
                "warnings": [],
                "payload": null,
                "stdout": null,
                "stderr": null,
                "exitCode": 1,
            }),
        ),
    ] {
        let err = validate_envelope_value(&payload)
            .expect_err("empty nested source-span files should fail validation");
        assert!(err.contains(context), "unexpected error: {err}");
        assert!(err.contains("span file"), "unexpected error: {err}");
        assert!(err.contains("non-empty"), "unexpected error: {err}");
    }
}

#[test]
fn validate_envelope_value_rejects_malformed_suggested_fix_edits() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix edits",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 1},
                            "end": {"file": "src/main.ts", "line": 1, "column": 2}
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix)
        .expect_err("missing suggested-fix text edit newText should fail validation");
    assert!(
        err.contains("suggested fix edits[0]"),
        "unexpected error: {err}"
    );
    assert!(err.contains("newText"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_unexpected_text_edit_keys() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix text edit",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 1},
                            "end": {"file": "src/main.ts", "line": 1, "column": 2},
                            "newText": "console.log(1);",
                            "metadata": true
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix)
        .expect_err("unexpected suggested-fix text edit keys should fail validation");
    assert!(
        err.contains("suggested fix edits[0]"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("unexpected key `metadata`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_unexpected_source_location_keys_in_suggested_fix_edits() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix source location",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 1, "metadata": true},
                            "end": {"file": "src/main.ts", "line": 1, "column": 2},
                            "newText": "console.log(1);"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix).expect_err(
        "unexpected source-location keys in suggested-fix edits should fail validation",
    );
    assert!(
        err.contains("suggested fix edits[0]"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("text edit start") && err.contains("unexpected key `metadata`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_unexpected_source_location_keys_in_suggested_fix_end_edits() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix source location",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 1},
                            "end": {"file": "src/main.ts", "line": 1, "column": 2, "metadata": true},
                            "newText": "console.log(1);"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix).expect_err(
        "unexpected source-location keys in suggested-fix edits should fail validation",
    );
    assert!(
        err.contains("suggested fix edits[0]"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("text edit end") && err.contains("unexpected key `metadata`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_missing_text_edit_source_location_fields() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix source location",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "column": 1},
                            "end": {"file": "src/main.ts", "line": 1, "column": 2},
                            "newText": "console.log(1);"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix)
        .expect_err("missing text-edit source-location fields should fail validation");
    assert!(err.contains("text edit start"), "unexpected error: {err}");
    assert!(err.contains("line"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_mismatched_suggested_fix_edit_file_mirrors() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix file mirror",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/other.ts", "line": 1, "column": 1},
                            "end": {"file": "src/other.ts", "line": 1, "column": 2},
                            "newText": "console.log(1);"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix)
        .expect_err("mismatched suggested-fix file mirrors should fail validation");
    assert!(
        err.contains("suggested fix edits[0]"),
        "unexpected error: {err}"
    );
    assert!(err.contains("start.file"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_mismatched_suggested_fix_edit_end_file_mirror() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix file mirror",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 1},
                            "end": {"file": "src/other.ts", "line": 1, "column": 2},
                            "newText": "console.log(1);"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix)
        .expect_err("mismatched suggested-fix file mirrors should fail validation");
    assert!(
        err.contains("suggested fix edits[0]"),
        "unexpected error: {err}"
    );
    assert!(err.contains("end.file"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_whitespace_suggested_fix_edit_end_file() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix end file",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 1},
                            "end": {"file": " ", "line": 1, "column": 2},
                            "newText": "console.log(1);"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix)
        .expect_err("whitespace suggested-fix end file should fail validation");
    assert!(err.contains("text edit end"), "unexpected error: {err}");
    assert!(err.contains("non-empty"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_unexpected_diagnostic_keys() {
    let invalid_diagnostic = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic shape",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": [],
                "extensionKey": true
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_diagnostic)
        .expect_err("unexpected diagnostic keys should fail validation");
    assert!(
        err.contains("diagnostic") && err.contains("unexpected key `extensionKey`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_accepts_non_overlapping_suggested_fix_edits() {
    let value = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "good suggested fix range",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 1},
                            "end": {"file": "src/main.ts", "line": 1, "column": 2},
                            "newText": "console.log(1);"
                        },
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 2},
                            "end": {"file": "src/main.ts", "line": 1, "column": 3},
                            "newText": "console.log(2);"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    validate_envelope_value(&value).expect("non-overlapping suggested-fix edits should validate");
}

#[test]
fn validate_envelope_value_rejects_overlapping_suggested_fix_edits() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix overlap",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 1},
                            "end": {"file": "src/main.ts", "line": 1, "column": 3},
                            "newText": "console.log(1);"
                        },
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 2},
                            "end": {"file": "src/main.ts", "line": 1, "column": 4},
                            "newText": "console.log(2);"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix)
        .expect_err("overlapping suggested-fix edits should fail validation");
    assert!(
        err.contains("suggested fix edits[1] overlaps with suggested fix edits[0]"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_duplicate_zero_length_suggested_fix_edits() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix duplicate insertion",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 2},
                            "end": {"file": "src/main.ts", "line": 1, "column": 2},
                            "newText": "console.log(1);"
                        },
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 1, "column": 2},
                            "end": {"file": "src/main.ts", "line": 1, "column": 2},
                            "newText": "console.log(2);"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix)
        .expect_err("duplicate zero-length suggested-fix edits should fail validation");
    assert!(
        err.contains("suggested fix edits[1] overlaps with suggested fix edits[0]"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_reversed_suggested_fix_edit_ranges() {
    let invalid_fix = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad suggested fix range",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 2, "column": 1},
                            "end": {"file": "src/main.ts", "line": 1, "column": 1},
                            "newText": "console.log(1);"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_fix)
        .expect_err("reversed suggested-fix range should fail validation");
    assert!(
        err.contains("suggested fix edits[0]"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("must not precede its start position"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_related_item_with_non_positive_span() {
    let invalid_related_span = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad related item span",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [
                    {
                        "message": "follow-up note",
                        "span": {"file": "src/main.ts", "line": 0, "column": 1, "endLine": 1, "endColumn": 2}
                    }
                ],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_related_span)
        .expect_err("non-positive related item span should fail validation");
    assert!(err.contains("related[0]"), "unexpected error: {err}");
    assert!(err.contains("line"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_empty_source_location_files() {
    let invalid_span_file = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad span file",
                "span": {"file": "", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });
    let err = validate_envelope_value(&invalid_span_file)
        .expect_err("empty span file should fail validation");
    assert!(err.contains("span file"), "unexpected error: {err}");
    assert!(err.contains("non-empty"), "unexpected error: {err}");

    let invalid_text_edit_file = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad text edit file",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": " ",
                            "start": {"file": " ", "line": 1, "column": 1},
                            "end": {"file": " ", "line": 1, "column": 2},
                            "newText": "console.log(1);"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });
    let err = validate_envelope_value(&invalid_text_edit_file)
        .expect_err("whitespace text edit source location file should fail validation");
    assert!(err.contains("text edit start"), "unexpected error: {err}");
    assert!(err.contains("non-empty"), "unexpected error: {err}");
}

#[test]
#[should_panic(expected = "CLI envelope errors must be an array")]
fn emit_envelope_value_rejects_non_array_errors() {
    let _ = emit_envelope_value(
        "doctor",
        true,
        json!({"severity": "error"}),
        json!([]),
        json!({"answer": 42}),
        Some("stdout text".to_string()),
        None,
        0,
    );
}

#[test]
#[should_panic(expected = "CLI envelope warnings must be an array")]
fn emit_envelope_value_rejects_non_array_warnings() {
    let _ = emit_envelope_value(
        "doctor",
        true,
        json!([]),
        json!({"severity": "warning"}),
        json!({"answer": 42}),
        Some("stdout text".to_string()),
        None,
        0,
    );
}

#[test]
fn validate_envelope_value_rejects_non_string_context_fields() {
    for (field, context, expected_fragment) in [
        (
            "configPath",
            json!({"origin": "config", "configPath": 42}),
            "configPath",
        ),
        (
            "configPath",
            json!({"origin": "config", "configPath": null}),
            "configPath",
        ),
        ("flag", json!({"origin": "cli", "flag": true}), "flag"),
        ("flag", json!({"origin": "cli", "flag": null}), "flag"),
    ] {
        let envelope = json!({
            "schemaVersion": 1,
            "command": "doctor",
            "success": false,
            "errors": [
                {
                    "severity": "error",
                    "code": "E5508",
                    "message": "bad diagnostic context",
                    "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                    "labels": [],
                    "related": [],
                    "fix": null,
                    "notes": [],
                    "context": context
                }
            ],
            "warnings": [],
            "payload": null,
            "stdout": null,
            "stderr": null,
            "exitCode": 1,
        });
        let err = validate_envelope_value(&envelope)
            .expect_err("non-string diagnostic context field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(err.contains(expected_fragment), "unexpected error: {err}");
    }
}

#[test]
fn validate_envelope_value_rejects_non_object_diagnostic_context() {
    let invalid_context = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic context",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": [],
                "context": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_context)
        .expect_err("array diagnostic context should fail validation");
    assert!(
        err.contains("diagnostic context"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_diagnostic_context_missing_origin() {
    let missing_origin = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic context",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": [],
                "context": {"configPath": "compilerOptions.apiSurface"}
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&missing_origin)
        .expect_err("missing diagnostic context origin should fail validation");
    assert!(err.contains("origin"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_unexpected_diagnostic_context_keys() {
    let unexpected_key = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic context",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": [],
                "context": {"origin": "cli", "flag": "--api", "extra": true}
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&unexpected_key)
        .expect_err("unexpected diagnostic context keys should fail validation");
    assert!(
        err.contains("diagnostic context"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("unexpected key `extra`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_allows_canonical_diagnostic_context_origins() {
    for origin in ["cli", "config", "default", "source"] {
        let value = json!({
            "schemaVersion": 1,
            "command": "doctor",
            "success": false,
            "errors": [
                {
                    "severity": "error",
                    "code": "E5508",
                    "message": "bad diagnostic context origin",
                    "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                    "labels": [],
                    "related": [],
                    "fix": null,
                    "notes": [],
                    "context": {"origin": origin}
                }
            ],
            "warnings": [],
            "payload": null,
            "stdout": null,
            "stderr": null,
            "exitCode": 1,
        });

        validate_envelope_value(&value)
            .unwrap_or_else(|err| panic!("canonical origin `{origin}` should validate: {err}"));
    }
}

#[test]
fn validate_envelope_value_rejects_invalid_diagnostic_context_origin() {
    let invalid_origin = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad diagnostic context origin",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": [],
                "context": {"origin": "browser"}
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&invalid_origin)
        .expect_err("unexpected diagnostic context origin should fail validation");
    assert!(err.contains("origin"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_empty_or_whitespace_diagnostic_context_origin() {
    for origin in ["", "   "] {
        let value = json!({
            "schemaVersion": 1,
            "command": "doctor",
            "success": false,
            "errors": [
                {
                    "severity": "error",
                    "code": "E5508",
                    "message": "bad diagnostic context origin",
                    "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                    "labels": [],
                    "related": [],
                    "fix": null,
                    "notes": [],
                    "context": {"origin": origin}
                }
            ],
            "warnings": [],
            "payload": null,
            "stdout": null,
            "stderr": null,
            "exitCode": 1,
        });

        let err = validate_envelope_value(&value)
            .expect_err("empty or whitespace diagnostic context origin should fail validation");
        assert!(
            err.contains("canonical origin string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_envelope_value_rejects_empty_diagnostic_context_paths() {
    for (field, context) in [
        ("configPath", json!({"origin": "config", "configPath": ""})),
        ("flag", json!({"origin": "cli", "flag": "   "})),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "command": "doctor",
            "success": false,
            "errors": [
                {
                    "severity": "error",
                    "code": "E5508",
                    "message": "bad diagnostic context path",
                    "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                    "labels": [],
                    "related": [],
                    "fix": null,
                    "notes": [],
                    "context": context
                }
            ],
            "warnings": [],
            "payload": null,
            "stdout": null,
            "stderr": null,
            "exitCode": 1,
        });

        let err = validate_envelope_value(&value).expect_err(&format!(
            "empty diagnostic context {field} should fail validation"
        ));
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_envelope_value_rejects_non_string_transport_fields() {
    let invalid_stdout = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": 42,
        "stderr": null,
        "exitCode": 0,
    });
    let err = validate_envelope_value(&invalid_stdout)
        .expect_err("numeric stdout should fail validation");
    assert!(err.contains("stdout"), "unexpected error: {err}");

    let invalid_stderr = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": ["bad"],
        "exitCode": 0,
    });
    let err =
        validate_envelope_value(&invalid_stderr).expect_err("array stderr should fail validation");
    assert!(err.contains("stderr"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_non_positive_span_and_location_fields() {
    let invalid_span = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad span",
                "span": {"file": "src/main.ts", "line": 0, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });
    let err =
        validate_envelope_value(&invalid_span).expect_err("span line zero should fail validation");
    assert!(err.contains("line"), "unexpected error: {err}");

    let invalid_column = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad span column",
                "span": {"file": "src/main.ts", "line": 1, "column": 0, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });
    let err = validate_envelope_value(&invalid_column)
        .expect_err("span column zero should fail validation");
    assert!(err.contains("column"), "unexpected error: {err}");

    let invalid_end_column = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad span end column",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 0},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });
    let err = validate_envelope_value(&invalid_end_column)
        .expect_err("span end column zero should fail validation");
    assert!(err.contains("endColumn"), "unexpected error: {err}");

    let invalid_location = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "bad fix location",
                "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": {
                    "message": "adjust location",
                    "edits": [
                        {
                            "file": "src/main.ts",
                            "start": {"file": "src/main.ts", "line": 0, "column": 1},
                            "end": {"file": "src/main.ts", "line": 1, "column": 2},
                            "newText": "let answer = 42;"
                        }
                    ]
                },
                "notes": []
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });
    let err = validate_envelope_value(&invalid_location)
        .expect_err("source location line zero should fail validation");
    assert!(err.contains("source location"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_rejects_inconsistent_success_and_exit_code() {
    let success_with_nonzero_exit_code = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });
    let err = validate_envelope_value(&success_with_nonzero_exit_code)
        .expect_err("success with nonzero exitCode should fail validation");
    assert!(err.contains("success=true"), "unexpected error: {err}");

    let failure_with_zero_exit_code = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
    });
    let err = validate_envelope_value(&failure_with_zero_exit_code)
        .expect_err("failure with zero exitCode should fail validation");
    assert!(err.contains("success=false"), "unexpected error: {err}");
}

#[test]
#[should_panic(expected = "CLI envelope success=true requires exitCode 0")]
fn emit_envelope_value_rejects_success_with_nonzero_exit_code() {
    let _ = emit_envelope_value(
        "doctor",
        true,
        json!([]),
        json!([]),
        json!({"answer": 42}),
        Some("stdout text".to_string()),
        None,
        1,
    );
}

#[test]
#[should_panic(expected = "CLI envelope success=false requires a non-zero exitCode")]
fn emit_envelope_value_rejects_failure_with_zero_exit_code() {
    let _ = emit_envelope_value(
        "doctor",
        false,
        json!([]),
        json!([]),
        serde_json::Value::Null,
        None,
        Some("stderr text".to_string()),
        0,
    );
}
