use super::*;

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
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
    });
    let err = validate_envelope_value(&missing_key).expect_err("missing payload should fail");
    assert!(err.contains("payload"), "unexpected error: {err}");
}

#[test]
fn validate_envelope_value_allows_missing_optional_stream_and_exit_code_fields() {
    let value = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
    });

    validate_envelope_value(&value)
        .expect("envelopes may omit optional stdout, stderr, and exitCode fields");
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
fn validate_envelope_value_rejects_non_string_command() {
    let value = json!({
        "schemaVersion": 1,
        "command": 1,
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
    });

    let err =
        validate_envelope_value(&value).expect_err("non-string command should fail validation");
    assert!(err.contains("command"), "unexpected error: {err}");
    assert!(
        err.contains("non-empty, non-whitespace string"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_non_string_stdout_and_stderr() {
    for (field, value) in [("stdout", json!(1)), ("stderr", json!({"bad": true}))] {
        let mut envelope = json!({
            "schemaVersion": 1,
            "command": "doctor",
            "success": true,
            "errors": [],
            "warnings": [],
            "payload": null,
            "stdout": null,
            "stderr": null,
            "exitCode": 0,
        });
        envelope[field] = value;

        let err = validate_envelope_value(&envelope)
            .expect_err("non-string stdout/stderr should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(err.contains("string or null"), "unexpected error: {err}");
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
        err.contains("timing phase must be a non-empty, non-whitespace string"),
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
        err.contains("timing phase must be a non-empty, non-whitespace string"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_envelope_value_rejects_whitespace_padded_timing_phases() {
    let padded_phase = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 0,
        "timings": [{"phase": " parse ", "milliseconds": 1}],
    });

    let err = validate_envelope_value(&padded_phase)
        .expect_err("whitespace-padded timing phases should fail validation");
    assert!(
        err.contains("timing phase must be a non-empty, non-whitespace string"),
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
fn validate_envelope_value_rejects_out_of_order_diagnostics() {
    let invalid_diagnostics = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": false,
        "errors": [
            {
                "severity": "error",
                "code": "E5508",
                "message": "later diagnostic",
                "span": {"file": "src/z.ts", "line": 2, "column": 1, "endLine": 2, "endColumn": 2},
                "labels": [],
                "related": [],
                "fix": null,
                "notes": []
            },
            {
                "severity": "error",
                "code": "E5501",
                "message": "earlier diagnostic",
                "span": {"file": "src/a.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
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

    let err = validate_envelope_value(&invalid_diagnostics)
        .expect_err("out-of-order diagnostics should fail validation");
    assert!(err.contains("errors[1]"), "unexpected error: {err}");
    assert!(
        err.contains("sorted by file, line, column, then code"),
        "unexpected error: {err}"
    );
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
fn validate_envelope_value_rejects_reversed_nested_source_spans() {
    for (context, payload) in [
        (
            "errors[0]",
            json!({
                "schemaVersion": 1,
                "command": "doctor",
                "success": false,
                "errors": [
                    {
                        "severity": "error",
                        "code": "E5508",
                        "message": "bad diagnostic span order",
                        "span": {"file": "src/main.ts", "line": 2, "column": 1, "endLine": 1, "endColumn": 1},
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
                        "message": "bad label span order",
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                        "labels": [
                            {
                                "span": {"file": "src/main.ts", "line": 2, "column": 1, "endLine": 1, "endColumn": 1},
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
                        "message": "bad related span order",
                        "span": {"file": "src/main.ts", "line": 1, "column": 1, "endLine": 1, "endColumn": 2},
                        "labels": [],
                        "related": [
                            {
                                "message": "follow-up note",
                                "span": {"file": "src/main.ts", "line": 2, "column": 1, "endLine": 1, "endColumn": 1}
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
            .expect_err("reversed nested source spans should fail validation");
        assert!(err.contains(context), "unexpected error: {err}");
        assert!(
            err.contains("must not precede its start position"),
            "unexpected error: {err}"
        );
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
