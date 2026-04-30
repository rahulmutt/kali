use kali_cli::{build, output};
use serde_json::json;

fn diagnostic_envelope_with_span(end_line: u64, end_column: u64) -> serde_json::Value {
    json!({
        "schemaVersion": 1,
        "command": "check",
        "success": false,
        "errors": [{
            "severity": "error",
            "code": "E5101",
            "message": "span check",
            "span": {
                "file": "src/main.ts",
                "line": 1,
                "column": 2,
                "endLine": end_line,
                "endColumn": end_column,
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
    })
}

#[test]
fn package_audit_payload_is_null_only() {
    assert!(output::validate_package_audit_payload_value(&serde_json::Value::Null).is_ok());

    let error = output::validate_package_audit_payload_value(&json!({"package": "lodash"}))
        .expect_err("non-null package-audit payload should be rejected");
    assert!(
        error.contains("package-audit payload must be null"),
        "unexpected error: {error}"
    );
}

#[test]
fn diagnostic_spans_allow_zero_length_ranges() {
    let envelope = diagnostic_envelope_with_span(1, 2);
    output::validate_envelope_value(&envelope).expect("zero-length span should be accepted");
}

#[test]
fn diagnostic_spans_reject_backwards_ranges() {
    let envelope = diagnostic_envelope_with_span(1, 1);
    let error =
        output::validate_envelope_value(&envelope).expect_err("backwards span should be rejected");
    assert!(
        error.contains("must not precede"),
        "unexpected error: {error}"
    );
}

#[test]
fn diagnostic_spans_reject_unexpected_keys() {
    let mut envelope = diagnostic_envelope_with_span(1, 2);
    envelope["errors"][0]["span"]["unexpected"] = json!(true);
    let error = output::validate_envelope_value(&envelope)
        .expect_err("unexpected span keys should be rejected");
    assert!(
        error.contains("unexpected key"),
        "unexpected error: {error}"
    );
}

#[test]
fn build_result_artifact_roles_reject_duplicate_primary_roles() {
    let build_result = json!({
        "artifactKind": "lib",
        "outputPath": "out/lib.wasm",
        "sizeBytes": 42,
        "buildMode": "release",
        "sourceHash": "sha256-test",
        "metadataPath": "out/lib.json",
        "witPath": "out/lib.wit",
        "artifacts": [
            {"kind": "first", "path": "out/a.wasm", "role": "primary-library"},
            {"kind": "second", "path": "out/b.wasm", "role": "primary-library"}
        ],
        "exports": [
            {"name": "foo", "signature": "func()"}
        ]
    });

    let error = build::validate_build_result_value(&build_result)
        .expect_err("duplicate primary-library role should be rejected");
    assert!(
        error.contains("duplicates primary-library"),
        "unexpected error: {error}"
    );
}

#[test]
fn build_result_artifacts_reject_duplicate_kind_path_pairs() {
    let build_result = json!({
        "artifactKind": "lib",
        "outputPath": "out/lib.wasm",
        "sizeBytes": 42,
        "buildMode": "release",
        "sourceHash": "sha256-test",
        "metadataPath": "out/lib.json",
        "witPath": "out/lib.wit",
        "artifacts": [
            {"kind": "first", "path": "out/a.wasm"},
            {"kind": "first", "path": "out/a.wasm", "role": "auxiliary"}
        ],
        "exports": [
            {"name": "foo", "signature": "func()"}
        ]
    });

    let error = build::validate_build_result_value(&build_result)
        .expect_err("duplicate artifact kind/path pairs should be rejected");
    assert!(
        error.contains("duplicates artifact `first` at `out/a.wasm`"),
        "unexpected error: {error}"
    );
}
