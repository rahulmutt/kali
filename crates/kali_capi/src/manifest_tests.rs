use crate::*;
use std::fs;

fn valid_binding_package_manifest() -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": 1,
        "kind": "binding-package",
        "moduleName": "sample",
        "hostAbiVersion": HOST_ABI_VERSION,
        "minHostAbiVersion": HOST_ABI_VERSION,
        "maxSpecializations": 8,
        "runtimeProfiles": ["wasm-threads", "fiber-threads", "wasm-threads"],
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "artifacts": {
            "library": "sample.capi.wasm",
            "metadata": "sample.cabi.json",
            "exportsHeader": "sample.h",
            "glue": ["z.py", "a.py", "z.py"]
        }
    })
}

#[test]
fn binding_package_manifest_orders_and_deduplicates_glue_deterministically() {
    let manifest = generate_binding_package_manifest(
        "sample",
        "sample.capi.wasm",
        "sample.cabi.json",
        "sample.h",
        &[
            "wasm-threads".to_string(),
            "fiber-threads".to_string(),
            "wasm-threads".to_string(),
        ],
        8,
        &[
            "support.py".to_string(),
            "shim.py".to_string(),
            "support.py".to_string(),
        ],
    );

    assert_eq!(manifest["schemaVersion"], 1);
    assert_eq!(manifest["kind"], "binding-package");
    assert_eq!(manifest["moduleName"], "sample");
    assert_eq!(manifest["hostAbiVersion"], HOST_ABI_VERSION);
    assert_eq!(manifest["minHostAbiVersion"], HOST_ABI_VERSION);
    assert_eq!(manifest["maxSpecializations"], 8);
    assert_eq!(
        manifest["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(manifest["hostContract"], "kali-hosted");
    assert_eq!(manifest["runtimeBackend"], "wasmtime");
    assert_eq!(manifest["artifacts"]["library"], "sample.capi.wasm");
    assert_eq!(manifest["artifacts"]["metadata"], "sample.cabi.json");
    assert_eq!(manifest["artifacts"]["exportsHeader"], "sample.h");
    assert_eq!(
        manifest["artifacts"]["glue"],
        serde_json::json!(["shim.py", "support.py"])
    );
}

#[test]
fn binding_package_manifest_with_provenance_uses_explicit_contract_labels() {
    let manifest = generate_binding_package_manifest_with_provenance(
        "sample",
        "sample.capi.wasm",
        "sample.cabi.json",
        "sample.h",
        &[
            "wasm-threads".to_string(),
            "fiber-threads".to_string(),
            "wasm-threads".to_string(),
        ],
        8,
        Some("browser-requested"),
        Some("browser-harness"),
        &[
            "support.py".to_string(),
            "shim.py".to_string(),
            "support.py".to_string(),
        ],
    );

    assert_eq!(manifest["schemaVersion"], 1);
    assert_eq!(manifest["kind"], "binding-package");
    assert_eq!(manifest["hostContract"], "browser-requested");
    assert_eq!(manifest["runtimeBackend"], "browser-harness");
    assert_eq!(
        manifest["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(
        manifest["artifacts"]["glue"],
        serde_json::json!(["shim.py", "support.py"])
    );
}

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
fn binding_package_manifest_summary_normalizes_string_lists() {
    let manifest = valid_binding_package_manifest();

    let summary = binding_package_manifest_summary(&manifest).expect("summarize manifest");

    assert_eq!(
        summary["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(
        summary["artifacts"]["glue"],
        serde_json::json!(["a.py", "z.py"])
    );
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

#[test]
fn binding_package_manifest_helpers_reject_whitespace_padded_module_name() {
    let mut manifest = valid_binding_package_manifest();
    manifest["moduleName"] = serde_json::json!(" sample ");

    let error = parse_binding_package_manifest(&manifest.to_string())
        .expect_err("padded moduleName should fail");
    assert!(error.contains("moduleName"), "unexpected error: {error}");

    let error =
        binding_package_manifest_summary(&manifest).expect_err("padded moduleName should fail");
    assert!(error.contains("moduleName"), "unexpected error: {error}");
}

#[test]
fn binding_package_manifest_helpers_reject_empty_provenance_fields() {
    for (field, value) in [
        ("hostContract", serde_json::json!("")),
        ("runtimeBackend", serde_json::json!("   ")),
    ] {
        let mut manifest = valid_binding_package_manifest();
        manifest[field] = value;

        let error = parse_binding_package_manifest(&manifest.to_string())
            .expect_err("empty provenance field should fail");
        assert!(error.contains(field), "unexpected error: {error}");

        let error = binding_package_manifest_summary(&manifest)
            .expect_err("empty provenance field should fail");
        assert!(error.contains(field), "unexpected error: {error}");
    }
}

#[test]
fn binding_package_manifest_helpers_reject_empty_or_whitespace_artifact_paths() {
    for (field, value) in [
        ("artifacts.library", serde_json::json!("")),
        ("artifacts.metadata", serde_json::json!("   ")),
        ("artifacts.exportsHeader", serde_json::json!("")),
    ] {
        let mut manifest = valid_binding_package_manifest();
        match field {
            "artifacts.library" => manifest["artifacts"]["library"] = value,
            "artifacts.metadata" => manifest["artifacts"]["metadata"] = value,
            "artifacts.exportsHeader" => manifest["artifacts"]["exportsHeader"] = value,
            other => panic!("unexpected field {other}"),
        }

        let error = parse_binding_package_manifest(&manifest.to_string())
            .expect_err("empty artifact path should fail");
        assert!(error.contains(field), "unexpected error: {error}");
        assert!(
            error.contains("non-empty, non-whitespace string"),
            "unexpected error: {error}"
        );

        let error = binding_package_manifest_summary(&manifest)
            .expect_err("empty artifact path should fail");
        assert!(error.contains(field), "unexpected error: {error}");
        assert!(
            error.contains("non-empty, non-whitespace string"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn binding_package_manifest_rejects_incompatible_host_abi_version_window() {
    let manifest = serde_json::json!({
        "schemaVersion": 1,
        "kind": "binding-package",
        "moduleName": "sample",
        "hostAbiVersion": 2,
        "minHostAbiVersion": 3,
        "artifacts": {
            "library": "sample.capi.wasm",
            "metadata": "sample.cabi.json",
            "exportsHeader": "sample.h",
            "glue": []
        }
    });

    let error = parse_binding_package_manifest(&manifest.to_string())
        .expect_err("invalid host ABI window should fail");
    assert!(
        error.contains("minHostAbiVersion"),
        "unexpected error: {error}"
    );
    assert!(
        error.contains("hostAbiVersion"),
        "unexpected error: {error}"
    );

    let error = binding_package_manifest_summary(&manifest)
        .expect_err("invalid host ABI window should fail");
    assert!(
        error.contains("minHostAbiVersion"),
        "unexpected error: {error}"
    );
    assert!(
        error.contains("hostAbiVersion"),
        "unexpected error: {error}"
    );
}

#[test]
fn binding_package_manifest_summary_rejects_invalid_required_field_types() {
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

        let error = binding_package_manifest_summary(&manifest)
            .expect_err("invalid required field type should fail");

        assert!(error.contains(field), "unexpected error: {error}");
    }
}

#[test]
fn binding_package_manifest_summary_rejects_non_string_provenance_fields() {
    for (field, value) in [
        ("hostContract", serde_json::json!(1)),
        ("runtimeBackend", serde_json::json!(false)),
    ] {
        let mut manifest = valid_binding_package_manifest();
        manifest[field] = value;

        let error = binding_package_manifest_summary(&manifest)
            .expect_err("invalid provenance field should fail");

        assert!(error.contains(field), "unexpected error: {error}");
    }
}

#[test]
fn binding_package_manifest_helpers_reject_ambiguous_auto_discovery() {
    let temp_root = std::env::temp_dir().join(format!(
        "kali_capi_binding_manifest_{}_ambiguous_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("monotonic time")
            .as_nanos()
    ));
    fs::create_dir_all(&temp_root).expect("temp dir");

    for stem in ["first", "second"] {
        let manifest_path = temp_root.join(format!("{}.binding-package.json", stem));
        fs::write(
            &manifest_path,
            generate_binding_package_manifest(
                stem,
                format!("{}.capi.wasm", stem),
                format!("{}.cabi.json", stem),
                format!("{}.h", stem),
                &[],
                8,
                &[],
            )
            .to_string(),
        )
        .expect("write ambiguous manifest");
    }

    let error = discover_binding_package_manifest_path(&temp_root)
        .expect_err("ambiguous discovery should fail");
    assert!(error.contains("ambiguous"), "unexpected error: {error}");
}
