use serde_json::json;

use crate::output::{emit_envelope_value, validate_envelope_value};

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
