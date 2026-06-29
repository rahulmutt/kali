use super::*;

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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
fn validate_doctor_payload_value_rejects_noncanonical_browser_harness_env_var() {
    let value = json!({
        "browserHarness": {
            "envVar": "KALI_BROWSER_HARNESS_COMMAND",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
        .expect_err("noncanonical browserHarness envVar should fail");
    assert_eq!(
        err,
        "doctor browserHarness envVar must be `KALI_BROWSER_BUNDLE_HARNESS_COMMAND`"
    );
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
fn validate_doctor_payload_value_accepts_trimmed_browser_runtime_host_description() {
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
            "hostDescription": " real browser host ",
            "hostDescriptionNote": "browser runtime host description: real browser host",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
        .expect("trimmed browser runtime host description should validate");
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
fn validate_doctor_payload_value_rejects_trimmed_duplicate_supported_commands() {
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
            "supportedCommands": [" run ", "  run  "],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
        .expect_err("trimmed duplicate supportedCommands items should fail");
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
fn validate_doctor_payload_value_rejects_trimmed_duplicate_diagnostic_notes() {
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
            "diagnosticNotes": [
                " supported browser runtime commands: run, test ",
                "  supported browser runtime commands: run, test  "
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("trimmed duplicate diagnostic notes should fail");
    assert!(
        err.contains("duplicate item `supported browser runtime commands: run, test`"),
        "unexpected error: {err}"
    );
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
fn validate_doctor_payload_value_rejects_empty_or_whitespace_summary_note() {
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
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "summaryNote": value,
                "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            .expect_err("empty or whitespace summaryNote should fail");
        assert!(
            err.contains("browserRuntimeContract summaryNote"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_doctor_payload_value_rejects_empty_or_whitespace_contract_scope_note() {
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
                "hostDescriptionNote": "browser runtime host description: real browser host",
                "supportedCommands": ["run", "test"],
                "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
                "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "contractScopeNote": value,
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
            .expect_err("empty or whitespace contractScopeNote should fail");
        assert!(
            err.contains("browserRuntimeContract contractScopeNote"),
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
fn validate_doctor_payload_value_rejects_duplicate_browser_runtime_supported_commands() {
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
            "supportedCommands": ["run", " run "],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
        .expect_err("duplicate browser runtime supported commands should fail");
    assert!(
        err.contains("supportedCommands must not contain duplicate item `run`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_doctor_payload_value_rejects_duplicate_browser_runtime_diagnostic_notes() {
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                " browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work ",
                " browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work ",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value)
        .expect_err("duplicate browser runtime diagnostic notes should fail");
    assert!(
        err.contains("diagnosticNotes must not contain duplicate item `browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work`"),
        "unexpected error: {err}"
    );
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
                "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
            "summaryNote": "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
            "contractScopeNote": "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
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
