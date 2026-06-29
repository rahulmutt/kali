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
fn published_cli_envelope_schema_matches_fixed_shape_validator_posture() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../schemas/envelope/v1.json"))
            .expect("envelope schema should be valid JSON");
    let object = schema.as_object().expect("envelope schema object");

    assert_eq!(object.get("additionalProperties"), Some(&json!(false)));

    let properties = object
        .get("properties")
        .and_then(serde_json::Value::as_object)
        .expect("envelope schema properties");
    for key in [
        "schemaVersion",
        "command",
        "success",
        "errors",
        "warnings",
        "payload",
        "stdout",
        "stderr",
        "exitCode",
        "artifacts",
        "timings",
    ] {
        assert!(
            properties.contains_key(key),
            "missing envelope property {key}"
        );
    }
    assert_eq!(properties["exitCode"].get("minimum"), Some(&json!(0)));
    assert_eq!(properties["command"].get("minLength"), Some(&json!(1)));

    let defs = object
        .get("$defs")
        .and_then(serde_json::Value::as_object)
        .expect("envelope schema defs");
    assert_eq!(
        defs["artifact"].get("additionalProperties"),
        Some(&json!(false))
    );
    assert_eq!(
        defs["timing"].get("additionalProperties"),
        Some(&json!(false))
    );
}

#[test]
fn published_diagnostic_schema_matches_fixed_shape_validator_posture() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../schemas/diagnostic/v1.json"))
            .expect("diagnostic schema should be valid JSON");
    let object = schema.as_object().expect("diagnostic schema object");

    assert_eq!(object.get("additionalProperties"), Some(&json!(false)));
    assert!(object
        .get("properties")
        .and_then(|properties| properties.get("severity"))
        .and_then(|severity| severity.get("enum"))
        .and_then(serde_json::Value::as_array)
        .expect("severity enum")
        .iter()
        .any(|value| value == "hint"));

    let defs = object
        .get("$defs")
        .and_then(serde_json::Value::as_object)
        .expect("diagnostic schema defs");
    for key in [
        "sourceLocation",
        "sourceSpan",
        "label",
        "relatedInfo",
        "textEdit",
        "suggestedFix",
        "diagnosticContext",
    ] {
        assert_eq!(
            defs[key].get("additionalProperties"),
            Some(&json!(false)),
            "{key} should be fixed-shape"
        );
    }
}

#[path = "output_tests/envelope.rs"]
mod envelope;

#[path = "output_tests/doctor.rs"]
mod doctor;

#[path = "output_tests/package.rs"]
mod package;

#[path = "output_tests/run.rs"]
mod run;

#[path = "output_tests/effects.rs"]
mod effects;

#[path = "output_tests/test.rs"]
mod test;

#[path = "output_tests/payloads_misc.rs"]
mod payloads_misc;

#[path = "output_tests/emit.rs"]
mod emit;
