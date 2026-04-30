use kali_cli::{build, output};
use serde_json::json;

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
