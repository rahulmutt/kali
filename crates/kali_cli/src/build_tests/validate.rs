use super::*;

#[test]
fn validate_build_result_value_rejects_duplicate_primary_artifact_roles() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm", "role": "primary-executable" },
            { "kind": "wasm-module", "path": "browser-shadow.wasm", "role": "primary-executable" },
            { "kind": "js-glue", "path": "browser.js", "role": "browser-glue" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("duplicate primary artifact roles should fail validation");
    assert!(
        err.contains("primary-executable"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_build_result_value_rejects_duplicate_artifact_kind_path_pairs() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm" },
            { "kind": "wasm-module", "path": "browser.wasm" },
            { "kind": "js-glue", "path": "browser.js" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("duplicate artifact kind/path pairs should fail validation");
    assert!(
        err.contains("duplicates artifact `wasm-module` at `browser.wasm`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_build_result_value_rejects_out_of_order_artifacts() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "js-glue", "path": "browser-z.js", "role": "browser-glue" },
            { "kind": "js-glue", "path": "browser-a.js", "role": "browser-glue" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("out-of-order build result artifacts should fail validation");
    assert!(
        err.contains("must be sorted by role, kind, then path"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_build_result_value_rejects_non_string_artifact_roles() {
    let invalid_component = serde_json::json!({
        "artifactKind": "component",
        "outputPath": "/workspace/dist/component",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "metadataPath": "/workspace/dist/component/component.meta.json",
        "witPath": "/workspace/dist/component/component.wit",
        "bindingPackagePath": "/workspace/dist/component/component.binding-package.json",
        "artifacts": [
            { "kind": "wasm-component", "path": "component.wasm", "role": 1 },
            { "kind": "wit", "path": "component.wit", "role": "interface-wit" }
        ],
        "exports": []
    });

    let err = validate_build_result_value(&invalid_component)
        .expect_err("non-string artifact roles should fail validation");
    assert!(err.contains("role"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_noncanonical_artifact_roles() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm", "role": "bundle-module" },
            { "kind": "js-glue", "path": "browser.js", "role": "browser-glue" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("noncanonical artifact roles should fail validation");
    assert!(
        err.contains("canonical schema-v1 role"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_build_result_value_rejects_whitespace_padded_artifact_roles() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm", "role": " browser-glue " },
            { "kind": "js-glue", "path": "browser.js", "role": "browser-glue" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("whitespace padded artifact roles should fail validation");
    assert!(err.contains("role"), "unexpected error: {err}");
    assert!(
        err.contains("leading or trailing whitespace"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_build_result_value_rejects_fractional_size_bytes() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42.5,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("fractional build result sizeBytes should fail validation");
    assert!(err.contains("sizeBytes"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_negative_size_bytes() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": -1,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("negative build result sizeBytes should fail validation");
    assert!(err.contains("sizeBytes"), "unexpected error: {err}");
}

#[test]
fn validate_artifact_metadata_value_rejects_unexpected_top_level_keys() {
    let invalid_metadata = serde_json::json!({
        "schemaVersion": 1,
        "artifactKind": "component",
        "entrypoint": "src/main.ts",
        "buildMode": "release",
        "apiSurface": "browser",
        "runtimeProfiles": ["wasm-threads"],
        "maxSpecializations": 24,
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "kaliVersion": "1.2.3",
        "sourceHash": "sha256-deadbeef",
        "exports": [],
        "unexpected": true
    });

    let err = validate_artifact_metadata_value(&invalid_metadata)
        .expect_err("unexpected artifact metadata keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_artifact_metadata_value_rejects_invalid_export_shape() {
    let invalid_metadata = serde_json::json!({
        "schemaVersion": 1,
        "artifactKind": "component",
        "entrypoint": "src/main.ts",
        "buildMode": "release",
        "apiSurface": "browser",
        "runtimeProfiles": ["wasm-threads"],
        "maxSpecializations": 24,
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "kaliVersion": "1.2.3",
        "sourceHash": "sha256-deadbeef",
        "exports": [
            {"name": "main", "signature": "(input) => number", "extra": true}
        ]
    });

    let err = validate_artifact_metadata_value(&invalid_metadata)
        .expect_err("extra export keys should fail validation");
    assert!(err.contains("exports[0]"), "unexpected error: {err}");
}

#[test]
fn validate_artifact_metadata_value_rejects_unexpected_export_keys() {
    let invalid_metadata = serde_json::json!({
        "schemaVersion": 1,
        "artifactKind": "component",
        "entrypoint": "src/main.ts",
        "buildMode": "release",
        "apiSurface": "browser",
        "runtimeProfiles": ["wasm-threads"],
        "maxSpecializations": 24,
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "kaliVersion": "1.2.3",
        "sourceHash": "sha256-deadbeef",
        "exports": [
            {"name": "main", "signature": "(input) => number", "extra": true}
        ]
    });

    let err = validate_artifact_metadata_value(&invalid_metadata)
        .expect_err("unexpected artifact metadata export keys should fail validation");
    assert!(
        err.contains("artifact metadata exports[0]"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("unexpected key `extra`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_artifact_metadata_value_rejects_duplicate_export_names() {
    let invalid_metadata = serde_json::json!({
        "schemaVersion": 1,
        "artifactKind": "component",
        "entrypoint": "src/main.ts",
        "buildMode": "release",
        "apiSurface": "browser",
        "runtimeProfiles": ["wasm-threads"],
        "maxSpecializations": 24,
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "kaliVersion": "1.2.3",
        "sourceHash": "sha256-deadbeef",
        "exports": [
            {"name": "main", "signature": "(input) => number"},
            {"name": "main", "signature": "(input) => number"}
        ]
    });

    let err = validate_artifact_metadata_value(&invalid_metadata)
        .expect_err("duplicate export names should fail validation");
    assert!(err.contains("duplicates `main`"), "unexpected error: {err}");
}

#[test]
fn validate_artifact_metadata_value_rejects_empty_or_whitespace_artifact_kind() {
    for artifact_kind in ["", "   "] {
        let invalid_metadata = serde_json::json!({
            "schemaVersion": 1,
            "artifactKind": artifact_kind,
            "entrypoint": "src/main.ts",
            "buildMode": "release",
            "apiSurface": "browser",
            "runtimeProfiles": ["wasm-threads"],
            "maxSpecializations": 24,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "kaliVersion": "1.2.3",
            "sourceHash": "sha256-deadbeef",
            "exports": []
        });

        let err = validate_artifact_metadata_value(&invalid_metadata)
            .expect_err("empty artifact kind should fail validation");
        assert!(err.contains("artifactKind"), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_artifact_metadata_value_rejects_empty_or_whitespace_build_mode() {
    for (field, invalid_value) in [("buildMode", ""), ("buildMode", "   ")] {
        let invalid_metadata = serde_json::json!({
            "schemaVersion": 1,
            "artifactKind": "component",
            "entrypoint": "src/main.ts",
            "buildMode": invalid_value,
            "apiSurface": "browser",
            "runtimeProfiles": ["wasm-threads"],
            "maxSpecializations": 24,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "kaliVersion": "1.2.3",
            "sourceHash": "sha256-deadbeef",
            "exports": []
        });

        let err = validate_artifact_metadata_value(&invalid_metadata)
            .expect_err("empty or whitespace artifact metadata buildMode should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_artifact_metadata_value_rejects_padded_canonical_labels() {
    for (field, invalid_value) in [("artifactKind", " component "), ("buildMode", " release ")] {
        let invalid_metadata = serde_json::json!({
            "schemaVersion": 1,
            "artifactKind": if field == "artifactKind" { invalid_value } else { "component" },
            "entrypoint": "src/main.ts",
            "buildMode": if field == "buildMode" { invalid_value } else { "release" },
            "apiSurface": "browser",
            "runtimeProfiles": ["wasm-threads"],
            "maxSpecializations": 24,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "kaliVersion": "1.2.3",
            "sourceHash": "sha256-deadbeef",
            "exports": []
        });

        let err = validate_artifact_metadata_value(&invalid_metadata)
            .expect_err("padded canonical labels should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_artifact_metadata_value_rejects_padded_entrypoint_and_api_surface() {
    for (field, invalid_value) in [("entrypoint", " src/main.ts "), ("apiSurface", " browser ")] {
        let invalid_metadata = serde_json::json!({
            "schemaVersion": 1,
            "artifactKind": "component",
            "entrypoint": if field == "entrypoint" { invalid_value } else { "src/main.ts" },
            "buildMode": "release",
            "apiSurface": if field == "apiSurface" { invalid_value } else { "browser" },
            "runtimeProfiles": ["wasm-threads"],
            "maxSpecializations": 24,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "kaliVersion": "1.2.3",
            "sourceHash": "sha256-deadbeef",
            "exports": []
        });

        let err = validate_artifact_metadata_value(&invalid_metadata)
            .expect_err("padded artifact metadata fields should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_artifact_metadata_value_rejects_padded_export_names_and_signatures() {
    for (field, invalid_value) in [("name", " main "), ("signature", " (input) => number ")] {
        let invalid_metadata = serde_json::json!({
            "schemaVersion": 1,
            "artifactKind": "component",
            "entrypoint": "src/main.ts",
            "buildMode": "release",
            "apiSurface": "browser",
            "runtimeProfiles": ["wasm-threads"],
            "maxSpecializations": 24,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "kaliVersion": "1.2.3",
            "sourceHash": "sha256-deadbeef",
            "exports": [
                {
                    "name": if field == "name" { invalid_value } else { "main" },
                    "signature": if field == "signature" { invalid_value } else { "(input) => number" }
                }
            ]
        });

        let err = validate_artifact_metadata_value(&invalid_metadata)
            .expect_err("padded export metadata should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_build_result_value_rejects_padded_export_names_and_signatures() {
    for (field, invalid_value) in [("name", " main "), ("signature", " (input) => number ")] {
        let invalid_result = serde_json::json!({
            "artifactKind": "component",
            "outputPath": "/workspace/dist/component",
            "sizeBytes": 42,
            "buildMode": "release",
            "sourceHash": "sha256-deadbeef",
            "metadataPath": "/workspace/dist/component.cabi.json",
            "witPath": "/workspace/dist/component.wit",
            "bindingPackagePath": "/workspace/dist/component.binding.json",
            "artifacts": [],
            "exports": [
                {
                    "name": if field == "name" { invalid_value } else { "main" },
                    "signature": if field == "signature" { invalid_value } else { "(input) => number" }
                }
            ]
        });

        let err = validate_build_result_value(&invalid_result)
            .expect_err("padded build-result export metadata should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_artifact_metadata_value_rejects_empty_or_whitespace_entrypoint_and_api_surface() {
    for (field, invalid_value) in [
        ("entrypoint", ""),
        ("entrypoint", "   "),
        ("apiSurface", ""),
        ("apiSurface", " \n\t "),
    ] {
        let invalid_metadata = serde_json::json!({
            "schemaVersion": 1,
            "artifactKind": "component",
            "entrypoint": if field == "entrypoint" { invalid_value } else { "src/main.ts" },
            "buildMode": "release",
            "apiSurface": if field == "apiSurface" { invalid_value } else { "browser" },
            "runtimeProfiles": ["wasm-threads"],
            "maxSpecializations": 24,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "kaliVersion": "1.2.3",
            "sourceHash": "sha256-deadbeef",
            "exports": []
        });

        let err = validate_artifact_metadata_value(&invalid_metadata)
            .expect_err("empty or whitespace artifact metadata field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_artifact_metadata_value_rejects_empty_or_whitespace_export_names_and_signatures() {
    for (field, invalid_value) in [
        ("name", ""),
        ("name", "   "),
        ("signature", ""),
        ("signature", " \n\t "),
    ] {
        let mut export = serde_json::json!({
            "name": "main",
            "signature": "(input) => number",
        });
        export
            .as_object_mut()
            .expect("export object")
            .insert(field.to_string(), serde_json::json!(invalid_value));

        let invalid_metadata = serde_json::json!({
            "schemaVersion": 1,
            "artifactKind": "component",
            "entrypoint": "src/main.ts",
            "buildMode": "release",
            "apiSurface": "browser",
            "runtimeProfiles": ["wasm-threads"],
            "maxSpecializations": 24,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "kaliVersion": "1.2.3",
            "sourceHash": "sha256-deadbeef",
            "exports": [export]
        });

        let err = validate_artifact_metadata_value(&invalid_metadata)
            .expect_err("empty or whitespace export field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_artifact_metadata_value_rejects_invalid_optional_provenance_fields() {
    for (field, invalid_metadata) in [
        (
            "profileDataHash",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef",
                "profileDataHash": 1
            }),
        ),
        (
            "profileDataHash",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef",
                "profileDataHash": ""
            }),
        ),
        (
            "profileDataHash",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef",
                "profileDataHash": "   "
            }),
        ),
        (
            "hostContract",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "hostContract",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "   ",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "runtimeBackend",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "runtimeBackend",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "   ",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "runtimeProfiles[1]",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads", 1],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "kaliVersion",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "kaliVersion",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "   ",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "sourceHash",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": ""
            }),
        ),
        (
            "sourceHash",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "   "
            }),
        ),
        (
            "maxSpecializations",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 1.5,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "non-negative integer",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": -1,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
    ] {
        let err = validate_artifact_metadata_value(&invalid_metadata)
            .expect_err("invalid artifact metadata field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_artifact_metadata_value_rejects_padded_provenance_fields() {
    for (field, invalid_metadata) in [
        (
            "profileDataHash",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef",
                "profileDataHash": " sha256-feedface "
            }),
        ),
        (
            "hostContract",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": " kali-hosted ",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "runtimeBackend",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": " wasmtime ",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "kaliVersion",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": " 1.2.3 ",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "sourceHash",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": " sha256-deadbeef "
            }),
        ),
    ] {
        let err = validate_artifact_metadata_value(&invalid_metadata)
            .expect_err("padded provenance field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_artifact_metadata_value_rejects_duplicate_runtime_profiles() {
    let invalid_metadata = serde_json::json!({
        "schemaVersion": 1,
        "artifactKind": "component",
        "entrypoint": "src/main.ts",
        "buildMode": "release",
        "apiSurface": "browser",
        "runtimeProfiles": ["wasm-threads", "wasm-threads"],
        "maxSpecializations": 24,
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "kaliVersion": "1.2.3",
        "sourceHash": "sha256-deadbeef"
    });

    let err = validate_artifact_metadata_value(&invalid_metadata)
        .expect_err("duplicate runtime profiles should fail validation");
    assert!(err.contains("runtimeProfiles"), "unexpected error: {err}");
}

#[test]
fn validate_artifact_metadata_value_rejects_empty_or_whitespace_runtime_profiles() {
    for (index, (runtime_profiles, expected_fragment)) in [
        (vec!["".to_string()], "non-empty, non-whitespace"),
        (vec!["   ".to_string()], "non-empty, non-whitespace"),
        (
            vec!["wasm-threads".to_string(), "\t".to_string()],
            "non-empty, non-whitespace",
        ),
        (
            vec![" wasm-threads ".to_string()],
            "leading or trailing whitespace",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let invalid_metadata = serde_json::json!({
            "schemaVersion": 1,
            "artifactKind": "component",
            "entrypoint": "src/main.ts",
            "buildMode": "release",
            "apiSurface": "browser",
            "runtimeProfiles": runtime_profiles,
            "maxSpecializations": 24,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "kaliVersion": "1.2.3",
            "sourceHash": "sha256-deadbeef"
        });

        let err = validate_artifact_metadata_value(&invalid_metadata)
            .expect_err("blank runtime profiles should fail validation");
        assert!(
            err.contains("runtimeProfile"),
            "unexpected error {index}: {err}"
        );
        assert!(
            err.contains(expected_fragment),
            "unexpected error {index}: {err}"
        );
    }
}

#[test]
fn validate_build_result_value_rejects_empty_or_whitespace_path_and_kind_fields() {
    for (field, invalid_result) in [
        (
            "outputPath",
            serde_json::json!({
                "artifactKind": "executable",
                "outputPath": " ",
                "sizeBytes": 42,
                "buildMode": "release",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "metadataPath",
            serde_json::json!({
                "artifactKind": "lib",
                "outputPath": "/workspace/dist/lib",
                "sizeBytes": 42,
                "buildMode": "release",
                "sourceHash": "sha256-deadbeef",
                "metadataPath": "",
                "witPath": "lib.wit",
                "artifacts": [],
                "exports": []
            }),
        ),
        (
            "headerPath",
            serde_json::json!({
                "artifactKind": "capi",
                "outputPath": "/workspace/dist/capi",
                "sizeBytes": 42,
                "buildMode": "release",
                "sourceHash": "sha256-deadbeef",
                "metadataPath": "/workspace/dist/capi.cabi.json",
                "witPath": "lib.wit",
                "headerPath": "   ",
                "artifacts": [],
                "exports": []
            }),
        ),
        (
            "bindingPackagePath",
            serde_json::json!({
                "artifactKind": "component",
                "outputPath": "/workspace/dist/component",
                "sizeBytes": 42,
                "buildMode": "release",
                "sourceHash": "sha256-deadbeef",
                "metadataPath": "/workspace/dist/component.cabi.json",
                "witPath": "lib.wit",
                "bindingPackagePath": "",
                "artifacts": [],
                "exports": []
            }),
        ),
        (
            "artifacts[0].path",
            serde_json::json!({
                "artifactKind": "bundle",
                "outputPath": "/workspace/dist/browser",
                "sizeBytes": 42,
                "buildMode": "release-advanced",
                "sourceHash": "sha256-deadbeef",
                "artifacts": [
                    { "kind": "js-glue", "path": "" },
                    { "kind": "wasm-module", "path": "browser.wasm" }
                ],
                "exports": [],
                "bundleFormat": "esm"
            }),
        ),
        (
            "artifacts[0].kind",
            serde_json::json!({
                "artifactKind": "bundle",
                "outputPath": "/workspace/dist/browser",
                "sizeBytes": 42,
                "buildMode": "release-advanced",
                "sourceHash": "sha256-deadbeef",
                "artifacts": [
                    { "kind": "", "path": "browser.js" },
                    { "kind": "wasm-module", "path": "browser.wasm" }
                ],
                "exports": [],
                "bundleFormat": "esm"
            }),
        ),
    ] {
        let err = validate_build_result_value(&invalid_result).expect_err(
            "empty or whitespace build result path and kind fields should fail validation",
        );
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(err.contains("non-empty"), "unexpected error: {err}");
    }
}

#[test]
fn validate_build_result_value_rejects_padded_output_and_sidecar_paths() {
    for (field, invalid_result) in [
        (
            "outputPath",
            serde_json::json!({
                "artifactKind": "executable",
                "outputPath": " /workspace/dist/executable ",
                "sizeBytes": 42,
                "buildMode": "release",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "metadataPath",
            serde_json::json!({
                "artifactKind": "lib",
                "outputPath": "/workspace/dist/lib",
                "sizeBytes": 42,
                "buildMode": "release",
                "sourceHash": "sha256-deadbeef",
                "metadataPath": " /workspace/dist/lib.cabi.json ",
                "witPath": "lib.wit",
                "artifacts": [],
                "exports": []
            }),
        ),
        (
            "witPath",
            serde_json::json!({
                "artifactKind": "capi",
                "outputPath": "/workspace/dist/capi",
                "sizeBytes": 42,
                "buildMode": "release",
                "sourceHash": "sha256-deadbeef",
                "metadataPath": "/workspace/dist/capi.cabi.json",
                "witPath": " lib.wit ",
                "headerPath": "/workspace/dist/capi.h",
                "artifacts": [],
                "exports": []
            }),
        ),
        (
            "headerPath",
            serde_json::json!({
                "artifactKind": "capi",
                "outputPath": "/workspace/dist/capi",
                "sizeBytes": 42,
                "buildMode": "release",
                "sourceHash": "sha256-deadbeef",
                "metadataPath": "/workspace/dist/capi.cabi.json",
                "witPath": "lib.wit",
                "headerPath": " /workspace/dist/capi.h ",
                "artifacts": [],
                "exports": []
            }),
        ),
        (
            "bindingPackagePath",
            serde_json::json!({
                "artifactKind": "component",
                "outputPath": "/workspace/dist/component",
                "sizeBytes": 42,
                "buildMode": "release",
                "sourceHash": "sha256-deadbeef",
                "metadataPath": "/workspace/dist/component.cabi.json",
                "witPath": "lib.wit",
                "bindingPackagePath": " /workspace/dist/component.binding.json ",
                "artifacts": [],
                "exports": []
            }),
        ),
    ] {
        let err = validate_build_result_value(&invalid_result)
            .expect_err("padded build result path fields should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_build_result_value_rejects_unexpected_top_level_keys() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "esm",
        "unexpected": true
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("unexpected build result keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_unexpected_artifact_keys() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm", "extra": true },
            { "kind": "js-glue", "path": "browser.js" }
        ],
        "exports": [],
        "bundleFormat": "umd"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("unexpected artifact keys should fail validation");
    assert!(err.contains("artifacts[0]"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_duplicate_export_names() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [
            { "name": "main", "signature": "(input) => number" },
            { "name": "main", "signature": "(input) => number" }
        ],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("duplicate export names should fail validation");
    assert!(err.contains("duplicates `main`"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_empty_or_whitespace_export_names_and_signatures() {
    for (field, invalid_value) in [
        ("name", ""),
        ("name", "   "),
        ("signature", ""),
        ("signature", " \n\t "),
    ] {
        let mut export = serde_json::json!({
            "name": "main",
            "signature": "(input) => number",
        });
        export
            .as_object_mut()
            .expect("export object")
            .insert(field.to_string(), serde_json::json!(invalid_value));

        let invalid_bundle = serde_json::json!({
            "artifactKind": "bundle",
            "outputPath": "/workspace/dist/browser",
            "sizeBytes": 42,
            "buildMode": "release-advanced",
            "sourceHash": "sha256-deadbeef",
            "artifacts": [
                { "kind": "js-glue", "path": "browser.js" },
                { "kind": "wasm-module", "path": "browser.wasm" }
            ],
            "exports": [export],
            "bundleFormat": "esm"
        });

        let err = validate_build_result_value(&invalid_bundle)
            .expect_err("empty or whitespace export field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_build_result_value_rejects_unexpected_export_keys() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [
            { "name": "main", "signature": "(input) => number", "extra": true }
        ],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("unexpected export keys should fail validation");
    assert!(
        err.contains("build result exports[0]"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("unexpected key `extra`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_build_result_value_rejects_invalid_bundle_format() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "umd"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("unsupported bundleFormat should fail validation");
    assert!(err.contains("bundleFormat"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_non_string_bundle_format() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": 1
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("non-string bundleFormat should fail validation");
    assert!(err.contains("bundleFormat"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_empty_or_whitespace_bundle_format() {
    for invalid_value in ["", "   "] {
        let invalid_bundle = serde_json::json!({
            "artifactKind": "bundle",
            "outputPath": "/workspace/dist/browser",
            "sizeBytes": 42,
            "buildMode": "release-advanced",
            "sourceHash": "sha256-deadbeef",
            "artifacts": [
                { "kind": "js-glue", "path": "browser.js" },
                { "kind": "wasm-module", "path": "browser.wasm" }
            ],
            "exports": [],
            "bundleFormat": invalid_value
        });

        let err = validate_build_result_value(&invalid_bundle)
            .expect_err("empty or whitespace bundleFormat should fail validation");
        assert!(err.contains("bundleFormat"), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_build_result_value_rejects_empty_or_whitespace_build_mode() {
    for invalid_value in ["", "   "] {
        let invalid_bundle = serde_json::json!({
            "artifactKind": "bundle",
            "outputPath": "/workspace/dist/browser",
            "sizeBytes": 42,
            "buildMode": invalid_value,
            "sourceHash": "sha256-deadbeef",
            "artifacts": [
                { "kind": "js-glue", "path": "browser.js" },
                { "kind": "wasm-module", "path": "browser.wasm" }
            ],
            "exports": [],
            "bundleFormat": "esm"
        });

        let err = validate_build_result_value(&invalid_bundle)
            .expect_err("empty or whitespace buildMode should fail validation");
        assert!(err.contains("buildMode"), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_build_result_value_rejects_unsupported_build_mode() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "debug",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("unsupported buildMode should fail validation");
    assert!(err.contains("buildMode"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_empty_profile_data_hash() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "profileDataHash": "",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("empty profileDataHash should fail validation");
    assert!(err.contains("profileDataHash"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_whitespace_profile_data_hash() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "profileDataHash": "   ",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("whitespace profileDataHash should fail validation");
    assert!(err.contains("profileDataHash"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_accepts_optional_provenance_fields() {
    let value = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "profileDataHash": "sha256-feedface",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    validate_build_result_value(&value).expect("build result provenance fields should validate");
}

#[test]
fn validate_build_result_value_rejects_whitespace_optional_provenance_fields() {
    for key in ["hostContract", "runtimeBackend"] {
        let invalid_bundle = serde_json::json!({
            "artifactKind": "bundle",
            "outputPath": "/workspace/dist/browser",
            "sizeBytes": 42,
            "buildMode": "release-advanced",
            "sourceHash": "sha256-deadbeef",
            (key): "   ",
            "artifacts": [
                { "kind": "js-glue", "path": "browser.js" },
                { "kind": "wasm-module", "path": "browser.wasm" }
            ],
            "exports": [],
            "bundleFormat": "esm"
        });

        let err = validate_build_result_value(&invalid_bundle)
            .expect_err("whitespace provenance field should fail validation");
        assert!(err.contains(key), "unexpected error: {err}");
    }
}

#[test]
fn validate_build_result_value_rejects_empty_source_hash() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("empty sourceHash should fail validation");
    assert!(err.contains("sourceHash"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_whitespace_source_hash() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "   ",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("whitespace sourceHash should fail validation");
    assert!(err.contains("sourceHash"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_padded_provenance_fields() {
    for (field, invalid_bundle) in [
        (
            "hostContract",
            serde_json::json!({
                "artifactKind": "bundle",
                "outputPath": "/workspace/dist/browser",
                "sizeBytes": 42,
                "buildMode": "release-advanced",
                "sourceHash": "sha256-deadbeef",
                "hostContract": " kali-hosted ",
                "runtimeBackend": "wasmtime",
                "profileDataHash": "sha256-feedface",
                "artifacts": [
                    { "kind": "js-glue", "path": "browser.js" },
                    { "kind": "wasm-module", "path": "browser.wasm" }
                ],
                "exports": [],
                "bundleFormat": "esm"
            }),
        ),
        (
            "runtimeBackend",
            serde_json::json!({
                "artifactKind": "bundle",
                "outputPath": "/workspace/dist/browser",
                "sizeBytes": 42,
                "buildMode": "release-advanced",
                "sourceHash": "sha256-deadbeef",
                "hostContract": "kali-hosted",
                "runtimeBackend": " wasmtime ",
                "profileDataHash": "sha256-feedface",
                "artifacts": [
                    { "kind": "js-glue", "path": "browser.js" },
                    { "kind": "wasm-module", "path": "browser.wasm" }
                ],
                "exports": [],
                "bundleFormat": "esm"
            }),
        ),
        (
            "profileDataHash",
            serde_json::json!({
                "artifactKind": "bundle",
                "outputPath": "/workspace/dist/browser",
                "sizeBytes": 42,
                "buildMode": "release-advanced",
                "sourceHash": "sha256-deadbeef",
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "profileDataHash": " sha256-feedface ",
                "artifacts": [
                    { "kind": "js-glue", "path": "browser.js" },
                    { "kind": "wasm-module", "path": "browser.wasm" }
                ],
                "exports": [],
                "bundleFormat": "esm"
            }),
        ),
    ] {
        let err = validate_build_result_value(&invalid_bundle)
            .expect_err("padded provenance field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_build_result_value_rejects_empty_artifact_kind() {
    for artifact_kind in ["", "   "] {
        let invalid_result = serde_json::json!({
            "artifactKind": artifact_kind,
            "outputPath": "/workspace/dist/browser",
            "sizeBytes": 42,
            "buildMode": "release-advanced",
            "sourceHash": "sha256-deadbeef",
        });

        let err = validate_build_result_value(&invalid_result)
            .expect_err("empty artifact kind should fail validation");
        assert!(err.contains("artifactKind"), "unexpected error: {err}");
    }
}

#[test]
fn validate_build_result_value_rejects_padded_canonical_labels() {
    for (field, invalid_result) in [
        (
            "artifactKind",
            serde_json::json!({
                "artifactKind": " bundle ",
                "outputPath": "/workspace/dist/browser",
                "sizeBytes": 42,
                "buildMode": "release-advanced",
                "sourceHash": "sha256-deadbeef",
            }),
        ),
        (
            "buildMode",
            serde_json::json!({
                "artifactKind": "bundle",
                "outputPath": "/workspace/dist/browser",
                "sizeBytes": 42,
                "buildMode": " release-advanced ",
                "sourceHash": "sha256-deadbeef",
            }),
        ),
        (
            "bundleFormat",
            serde_json::json!({
                "artifactKind": "bundle",
                "outputPath": "/workspace/dist/browser",
                "sizeBytes": 42,
                "buildMode": "release-advanced",
                "sourceHash": "sha256-deadbeef",
                "artifacts": [
                    { "kind": "js-glue", "path": "browser.js" },
                    { "kind": "wasm-module", "path": "browser.wasm" }
                ],
                "exports": [],
                "bundleFormat": " esm "
            }),
        ),
    ] {
        let err = validate_build_result_value(&invalid_result)
            .expect_err("padded canonical labels should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_build_result_value_rejects_unsupported_artifact_kind() {
    let invalid_result = serde_json::json!({
        "artifactKind": "meta-json",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
    });

    let err = validate_build_result_value(&invalid_result)
        .expect_err("unsupported build result artifactKind should fail validation");
    assert!(err.contains("artifactKind"), "unexpected error: {err}");
}
