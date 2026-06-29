use super::*;

#[test]
fn cabi_metadata_parsing_rejects_unexpected_keys() {
    let error = parse_metadata(
        r#"{
            "schemaVersion": 1,
            "kind": "cabi-metadata",
            "hostAbiVersion": 2,
            "artifacts": {
                "wasmModule": "sample.wasm",
                "wit": "sample.wit",
                "exportsHeader": "sample.h",
                "extra": true
            },
            "unexpected": true
        }"#,
    )
    .expect_err("unexpected cabi metadata keys should fail");

    assert!(
        error.contains("unexpected key"),
        "unexpected error: {error}"
    );
}

#[test]
fn cabi_metadata_parsing_rejects_negative_max_specializations() {
    let metadata = serde_json::json!({
        "schemaVersion": 1,
        "kind": "cabi-metadata",
        "hostAbiVersion": 2,
        "maxSpecializations": -1,
        "artifacts": {
            "wasmModule": "sample.wasm",
            "wit": "sample.wit",
            "exportsHeader": "sample.h"
        }
    });

    let error =
        parse_metadata(&metadata.to_string()).expect_err("negative maxSpecializations should fail");
    assert!(
        error.contains("must be a non-negative integer"),
        "unexpected error: {error}"
    );
    let error = cabi_metadata_summary(&metadata)
        .expect_err("negative maxSpecializations should fail in summary mode");
    assert!(
        error.contains("must be a non-negative integer"),
        "unexpected error: {error}"
    );
}
