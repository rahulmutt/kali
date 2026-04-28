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
        "timings": [{"label": "parse", "elapsedMs": 1}],
    });

    validate_envelope_value(&extended).expect("schema-permitted extension keys should validate");
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
