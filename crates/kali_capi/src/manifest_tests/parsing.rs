use super::*;

#[test]
fn binding_package_manifest_parsing_normalizes_string_lists() {
    let manifest = parse_binding_package_manifest(
        r#"{
            "schemaVersion": 1,
            "kind": "binding-package",
            "moduleName": "sample",
            "hostAbiVersion": 2,
            "runtimeProfiles": ["wasm-threads", "fiber-threads", "wasm-threads"],
            "artifacts": {
                "library": "sample.capi.wasm",
                "metadata": "sample.cabi.json",
                "exportsHeader": "sample.h",
                "glue": ["z.py", "a.py", "z.py"]
            }
        }"#,
    )
    .expect("parse normalized manifest");

    assert_eq!(
        manifest["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(
        manifest["artifacts"]["glue"],
        serde_json::json!(["a.py", "z.py"])
    );
}

#[test]
fn binding_package_manifest_parsing_rejects_whitespace_padded_string_lists() {
    for (field, manifest) in [
        (
            "runtimeProfiles",
            serde_json::json!({
                "schemaVersion": 1,
                "kind": "binding-package",
                "moduleName": "sample",
                "hostAbiVersion": HOST_ABI_VERSION,
                "runtimeProfiles": [" wasm-threads "],
                "artifacts": {
                    "library": "sample.capi.wasm",
                    "metadata": "sample.cabi.json",
                    "exportsHeader": "sample.h",
                    "glue": ["z.py"]
                }
            }),
        ),
        (
            "artifacts.glue",
            serde_json::json!({
                "schemaVersion": 1,
                "kind": "binding-package",
                "moduleName": "sample",
                "hostAbiVersion": HOST_ABI_VERSION,
                "runtimeProfiles": ["wasm-threads"],
                "artifacts": {
                    "library": "sample.capi.wasm",
                    "metadata": "sample.cabi.json",
                    "exportsHeader": "sample.h",
                    "glue": [" shim.py "]
                }
            }),
        ),
    ] {
        let error = parse_binding_package_manifest(&manifest.to_string())
            .expect_err("padded string list entries should fail");
        assert!(error.contains(field), "unexpected error: {error}");
        assert!(
            error.contains("leading or trailing whitespace"),
            "unexpected error: {error}"
        );

        let error = binding_package_manifest_summary(&manifest)
            .expect_err("padded string list entries should fail");
        assert!(error.contains(field), "unexpected error: {error}");
        assert!(
            error.contains("leading or trailing whitespace"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn binding_package_manifest_parsing_rejects_whitespace_padded_artifact_paths() {
    for (field, manifest) in [
        (
            "artifacts.library",
            serde_json::json!({
                "schemaVersion": 1,
                "kind": "binding-package",
                "moduleName": "sample",
                "hostAbiVersion": HOST_ABI_VERSION,
                "runtimeProfiles": ["wasm-threads"],
                "artifacts": {
                    "library": " sample.capi.wasm ",
                    "metadata": "sample.cabi.json",
                    "exportsHeader": "sample.h",
                    "glue": ["shim.py"]
                }
            }),
        ),
        (
            "artifacts.metadata",
            serde_json::json!({
                "schemaVersion": 1,
                "kind": "binding-package",
                "moduleName": "sample",
                "hostAbiVersion": HOST_ABI_VERSION,
                "runtimeProfiles": ["wasm-threads"],
                "artifacts": {
                    "library": "sample.capi.wasm",
                    "metadata": " sample.cabi.json ",
                    "exportsHeader": "sample.h",
                    "glue": ["shim.py"]
                }
            }),
        ),
        (
            "artifacts.exportsHeader",
            serde_json::json!({
                "schemaVersion": 1,
                "kind": "binding-package",
                "moduleName": "sample",
                "hostAbiVersion": HOST_ABI_VERSION,
                "runtimeProfiles": ["wasm-threads"],
                "artifacts": {
                    "library": "sample.capi.wasm",
                    "metadata": "sample.cabi.json",
                    "exportsHeader": " sample.h ",
                    "glue": ["shim.py"]
                }
            }),
        ),
    ] {
        let error = parse_binding_package_manifest(&manifest.to_string())
            .expect_err("padded artifact path entries should fail");
        assert!(error.contains(field), "unexpected error: {error}");
        assert!(
            error.contains("non-empty, non-whitespace string"),
            "unexpected error: {error}"
        );

        let error = binding_package_manifest_summary(&manifest)
            .expect_err("padded artifact path entries should fail");
        assert!(error.contains(field), "unexpected error: {error}");
        assert!(
            error.contains("non-empty, non-whitespace string"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn binding_package_manifest_parsing_rejects_non_integer_max_specializations() {
    let error = parse_binding_package_manifest(
        r#"{
            "schemaVersion": 1,
            "kind": "binding-package",
            "moduleName": "sample",
            "hostAbiVersion": 2,
            "maxSpecializations": "eight",
            "artifacts": {
                "library": "sample.capi.wasm",
                "metadata": "sample.cabi.json",
                "exportsHeader": "sample.h",
                "glue": []
            }
        }"#,
    )
    .expect_err("invalid maxSpecializations should fail");

    assert!(
        error.contains("maxSpecializations"),
        "unexpected error: {error}"
    );
}

#[test]
fn binding_package_manifest_parsing_rejects_negative_max_specializations() {
    let manifest = serde_json::json!({
        "schemaVersion": 1,
        "kind": "binding-package",
        "moduleName": "sample",
        "hostAbiVersion": 2,
        "maxSpecializations": -1,
        "artifacts": {
            "library": "sample.capi.wasm",
            "metadata": "sample.cabi.json",
            "exportsHeader": "sample.h",
            "glue": []
        }
    });

    let error = parse_binding_package_manifest(&manifest.to_string())
        .expect_err("negative maxSpecializations should fail");
    assert!(
        error.contains("must be a non-negative integer"),
        "unexpected error: {error}"
    );
    let error = binding_package_manifest_summary(&manifest)
        .expect_err("negative maxSpecializations should fail in summary mode");
    assert!(
        error.contains("must be a non-negative integer"),
        "unexpected error: {error}"
    );
}

#[test]
fn binding_package_manifest_parsing_rejects_unexpected_keys() {
    let error = parse_binding_package_manifest(
        r#"{
            "schemaVersion": 1,
            "kind": "binding-package",
            "moduleName": "sample",
            "hostAbiVersion": 2,
            "artifacts": {
                "library": "sample.capi.wasm",
                "metadata": "sample.cabi.json",
                "exportsHeader": "sample.h",
                "glue": [],
                "extra": true
            },
            "unexpected": true
        }"#,
    )
    .expect_err("unexpected binding package manifest keys should fail");

    assert!(
        error.contains("unexpected key"),
        "unexpected error: {error}"
    );
}

#[test]
fn binding_package_manifest_parsing_rejects_invalid_required_field_types() {
    let cases = [
        ("moduleName", serde_json::json!(1)),
        ("hostAbiVersion", serde_json::json!("two")),
        ("minHostAbiVersion", serde_json::json!(false)),
        ("artifacts.library", serde_json::json!(1)),
        ("artifacts.metadata", serde_json::json!(null)),
        ("artifacts.exportsHeader", serde_json::json!(["sample.h"])),
        ("artifacts.glue", serde_json::json!("sample.py")),
        ("artifacts.glue", serde_json::json!(["sample.py", 1])),
    ];

    for (field, value) in cases {
        let mut manifest = valid_binding_package_manifest();
        match field {
            "moduleName" => manifest["moduleName"] = value,
            "hostAbiVersion" => manifest["hostAbiVersion"] = value,
            "minHostAbiVersion" => manifest["minHostAbiVersion"] = value,
            "artifacts.library" => manifest["artifacts"]["library"] = value,
            "artifacts.metadata" => manifest["artifacts"]["metadata"] = value,
            "artifacts.exportsHeader" => manifest["artifacts"]["exportsHeader"] = value,
            "artifacts.glue" => manifest["artifacts"]["glue"] = value,
            _ => unreachable!("unexpected field: {field}"),
        }

        let error = parse_binding_package_manifest(&manifest.to_string())
            .expect_err("invalid required field type should fail");

        assert!(error.contains(field), "unexpected error: {error}");
    }
}

#[test]
fn binding_package_manifest_parsing_rejects_non_string_provenance_fields() {
    for (field, value) in [
        ("hostContract", serde_json::json!(1)),
        ("runtimeBackend", serde_json::json!(false)),
    ] {
        let mut manifest = valid_binding_package_manifest();
        manifest[field] = value;

        let error = parse_binding_package_manifest(&manifest.to_string())
            .expect_err("invalid provenance field should fail");

        assert!(error.contains(field), "unexpected error: {error}");
    }
}
