use serde_json::json;

use crate::output::{
    emit_envelope_value, validate_check_payload_value, validate_doctor_payload_value,
    validate_effects_payload_value, validate_envelope_value, validate_fmt_payload_value,
    validate_init_payload_value, validate_install_payload_value, validate_lint_payload_value,
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                    "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
    });

    validate_run_payload_value(&value).expect("run payload should validate");
}

#[test]
fn validate_run_payload_value_rejects_non_string_provenance_fields() {
    for (field, value) in [("hostContract", json!(true)), ("runtimeBackend", json!(42))] {
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
    assert_payload_accepts_schema_permitted_extension_key(
        json!({
            "manifestPath": "/workspace/example/kali.json",
            "lockPath": null,
            "installed": ["semver"],
            "updated": [],
            "removed": [],
        }),
        validate_install_payload_value,
    );
}

#[test]
fn validate_test_payload_value_rejects_non_string_provenance_fields() {
    for (field, value) in [
        ("hostContract", json!(null)),
        ("runtimeBackend", json!(["wasmtime"])),
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
            "entryPoints": [],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    validate_package_effects_payload_value(&value)
        .expect("package-effects payload should validate");
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
            "entryPoints": [],
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
            "entryPoints": [],
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
            "entryPoints": [],
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
                "entryPoints": [],
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
            "entryPoints": [],
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
            "supportedCommands": ["run", "run"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
                        "browser runtime host description: real browser host"
                    ]
                }
            }),
            "supportedCommands[1]",
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
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
fn validate_doctor_payload_value_rejects_empty_host_description_note() {
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
            "hostDescriptionNote": "",
            "supportedCommands": ["run", "test"],
            "diagnosticHint": "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work.",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err =
        validate_doctor_payload_value(&value).expect_err("empty hostDescriptionNote should fail");
    assert!(
        err.contains("browserRuntimeContract hostDescriptionNote"),
        "unexpected error: {err}"
    );
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
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
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
fn validate_doctor_payload_value_rejects_empty_diagnostic_hint() {
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
            "diagnosticHint": "",
            "diagnosticNotes": [
                "supported browser runtime commands: run, test",
                "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work",
                "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness",
                "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, or shape-invalid",
                "browser runtime host description: real browser host"
            ]
        }
    });

    let err = validate_doctor_payload_value(&value).expect_err("empty diagnosticHint should fail");
    assert!(err.contains("non-empty string"), "unexpected error: {err}");
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
fn validate_envelope_value_allows_schema_permitted_timing_extensions() {
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

    validate_envelope_value(&extended_timings)
        .expect("schema-permitted timing extensions should validate");
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
fn validate_envelope_value_allows_schema_permitted_suggested_fix_extensions() {
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
                            "newText": "console.log(1);",
                            "metadata": {"kind": "replacement"}
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

    validate_envelope_value(&extended_fix)
        .expect("schema-permitted suggested fix extensions should validate");
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
    let invalid_config_path = json!({
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
                "context": {"origin": "config", "configPath": 42}
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });
    let err = validate_envelope_value(&invalid_config_path)
        .expect_err("numeric configPath should fail validation");
    assert!(err.contains("configPath"), "unexpected error: {err}");

    let invalid_flag = json!({
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
                "context": {"origin": "cli", "flag": true}
            }
        ],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });
    let err =
        validate_envelope_value(&invalid_flag).expect_err("boolean flag should fail validation");
    assert!(err.contains("flag"), "unexpected error: {err}");
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
