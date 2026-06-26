use crate::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn metadata_generation_includes_expected_artifacts() {
    let metadata = generate_metadata("lib.capi.wasm", "lib.wit", "lib.h");
    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["kind"], "cabi-metadata");
    assert_eq!(metadata["hostAbiVersion"], HOST_ABI_VERSION);
    assert_eq!(metadata["artifacts"]["wasmModule"], "lib.capi.wasm");
    assert_eq!(metadata["artifacts"]["wit"], "lib.wit");
    assert_eq!(metadata["artifacts"]["exportsHeader"], "lib.h");
}

#[test]
fn metadata_generation_with_provenance_keeps_optional_fields_deterministic() {
    let metadata = generate_metadata_with_provenance(
        "lib.capi.wasm",
        "lib.wit",
        "lib.h",
        &[
            "wasm-threads".to_string(),
            "fiber-threads".to_string(),
            "wasm-threads".to_string(),
        ],
        8,
        Some("kali-hosted"),
        Some("wasmtime"),
    );

    assert_eq!(metadata["schemaVersion"], 1);
    assert_eq!(metadata["kind"], "cabi-metadata");
    assert_eq!(metadata["hostAbiVersion"], HOST_ABI_VERSION);
    assert_eq!(metadata["minHostAbiVersion"], HOST_ABI_VERSION);
    assert_eq!(
        metadata["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(metadata["maxSpecializations"], 8);
    assert_eq!(metadata["hostContract"], "kali-hosted");
    assert_eq!(metadata["runtimeBackend"], "wasmtime");
    assert_eq!(metadata["artifacts"]["wasmModule"], "lib.capi.wasm");
    assert_eq!(metadata["artifacts"]["wit"], "lib.wit");
    assert_eq!(metadata["artifacts"]["exportsHeader"], "lib.h");
}

#[test]
fn cabi_metadata_helpers_load_and_summarize_generated_payloads() {
    let temp_root = std::env::temp_dir().join(format!(
        "kali_capi_metadata_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("monotonic time")
            .as_nanos()
    ));
    fs::create_dir_all(&temp_root).expect("temp dir");

    let metadata_path = temp_root.join("sample.cabi.json");
    let mut metadata = generate_metadata_with_provenance(
        "sample.capi.wasm",
        "sample.wit",
        "sample.h",
        &[
            "wasm-threads".to_string(),
            "fiber-threads".to_string(),
            "wasm-threads".to_string(),
        ],
        8,
        Some("kali-hosted"),
        Some("wasmtime"),
    );
    metadata["profileDataHash"] = serde_json::json!("sha256:sample-profile");
    fs::write(&metadata_path, metadata.to_string()).expect("write cabi metadata");

    let loaded = load_metadata(&metadata_path).expect("load cabi metadata");
    assert_eq!(loaded["schemaVersion"], 1);
    assert_eq!(loaded["kind"], "cabi-metadata");
    assert_eq!(loaded["hostAbiVersion"], HOST_ABI_VERSION);
    assert_eq!(loaded["minHostAbiVersion"], HOST_ABI_VERSION);
    assert_eq!(
        loaded["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(loaded["maxSpecializations"], 8);
    assert_eq!(loaded["hostContract"], "kali-hosted");
    assert_eq!(loaded["runtimeBackend"], "wasmtime");
    assert_eq!(loaded["profileDataHash"], "sha256:sample-profile");
    assert_eq!(loaded["artifacts"]["wasmModule"], "sample.capi.wasm");
    assert_eq!(loaded["artifacts"]["wit"], "sample.wit");
    assert_eq!(loaded["artifacts"]["exportsHeader"], "sample.h");

    let summary = cabi_metadata_summary(&loaded).expect("summarize cabi metadata");
    assert_eq!(summary["schemaVersion"], 1);
    assert_eq!(summary["kind"], "cabi-metadata");
    assert_eq!(summary["hostAbiVersion"], HOST_ABI_VERSION);
    assert_eq!(summary["minHostAbiVersion"], HOST_ABI_VERSION);
    assert_eq!(summary["maxSpecializations"], 8);
    assert_eq!(
        summary["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(summary["hostContract"], "kali-hosted");
    assert_eq!(summary["runtimeBackend"], "wasmtime");
    assert_eq!(summary["profileDataHash"], "sha256:sample-profile");
    assert_eq!(summary["artifacts"]["wasmModule"], "sample.capi.wasm");
    assert_eq!(summary["artifacts"]["wit"], "sample.wit");
    assert_eq!(summary["artifacts"]["exportsHeader"], "sample.h");

    let loaded_summary =
        load_metadata_summary(&metadata_path).expect("load and summarize cabi metadata");
    assert_eq!(loaded_summary, summary);
}

#[test]
fn cabi_metadata_helpers_discover_load_and_summarize_root_sidecars() {
    let temp_root = std::env::temp_dir().join(format!(
        "kali_capi_metadata_root_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("monotonic time")
            .as_nanos()
    ));
    fs::create_dir_all(&temp_root).expect("temp dir");

    let metadata_path = temp_root.join("sample.capi.meta.json");
    let mut metadata = generate_metadata_with_provenance(
        "sample.capi.wasm",
        "sample.wit",
        "sample.h",
        &[
            "wasm-threads".to_string(),
            "fiber-threads".to_string(),
            "wasm-threads".to_string(),
        ],
        8,
        Some("kali-hosted"),
        Some("wasmtime"),
    );
    metadata["profileDataHash"] = serde_json::json!("sha256:sample-profile");
    fs::write(&metadata_path, metadata.to_string()).expect("write cabi metadata sidecar");
    fs::write(temp_root.join("noise.txt"), "ignore me").expect("write noise file");

    let discovered = discover_metadata_path(&temp_root).expect("discover cabi metadata sidecar");
    assert_eq!(discovered, metadata_path);

    let explicit = discover_metadata_path_with_name(&temp_root, "sample.capi.meta.json")
        .expect("discover explicit cabi metadata sidecar");
    assert_eq!(explicit, metadata_path);

    let loaded = load_metadata_from_root(&temp_root).expect("load cabi metadata from root");
    assert_eq!(loaded["kind"], "cabi-metadata");
    assert_eq!(
        loaded["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(loaded["hostContract"], "kali-hosted");
    assert_eq!(loaded["runtimeBackend"], "wasmtime");
    assert_eq!(loaded["profileDataHash"], "sha256:sample-profile");

    let summary = load_metadata_summary_from_root(&temp_root)
        .expect("load and summarize cabi metadata from root");
    assert_eq!(summary["kind"], "cabi-metadata");
    assert_eq!(
        summary["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(summary["hostContract"], "kali-hosted");
    assert_eq!(summary["runtimeBackend"], "wasmtime");
    assert_eq!(summary["profileDataHash"], "sha256:sample-profile");

    let explicit_summary =
        load_metadata_summary_from_root_with_name(&temp_root, "sample.capi.meta.json")
            .expect("load and summarize explicit cabi metadata sidecar");
    assert_eq!(explicit_summary, summary);
}

#[test]
fn cabi_metadata_helpers_reject_incompatible_host_abi_version_windows() {
    let metadata = serde_json::json!({
        "schemaVersion": 1,
        "kind": "cabi-metadata",
        "hostAbiVersion": 2,
        "minHostAbiVersion": 3,
        "artifacts": {
            "wasmModule": "sample.capi.wasm",
            "wit": "sample.wit",
            "exportsHeader": "sample.h"
        }
    });

    let error =
        parse_metadata(&metadata.to_string()).expect_err("invalid host ABI window should fail");
    assert!(
        error.contains("minHostAbiVersion"),
        "unexpected error: {error}"
    );
    assert!(
        error.contains("hostAbiVersion"),
        "unexpected error: {error}"
    );

    let error = cabi_metadata_summary(&metadata).expect_err("invalid host ABI window should fail");
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
fn cabi_metadata_helpers_reject_ambiguous_auto_discovery() {
    let temp_root = std::env::temp_dir().join(format!(
        "kali_capi_metadata_root_{}_ambiguous_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("monotonic time")
            .as_nanos()
    ));
    fs::create_dir_all(&temp_root).expect("temp dir");

    for stem in ["first", "second"] {
        let metadata_path = temp_root.join(format!("{}.capi.meta.json", stem));
        fs::write(
            &metadata_path,
            generate_metadata_with_provenance(
                format!("{}.capi.wasm", stem),
                format!("{}.wit", stem),
                format!("{}.h", stem),
                &[],
                8,
                Some("kali-hosted"),
                Some("wasmtime"),
            )
            .to_string(),
        )
        .expect("write ambiguous metadata sidecar");
    }

    let error = discover_metadata_path(&temp_root).expect_err("ambiguous discovery should fail");
    assert!(error.contains("ambiguous"), "unexpected error: {error}");
}

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

#[test]
fn cabi_metadata_helpers_reject_empty_provenance_fields() {
    for (field, value) in [
        ("hostContract", serde_json::json!("")),
        ("runtimeBackend", serde_json::json!("   ")),
        ("profileDataHash", serde_json::json!("\t")),
    ] {
        let mut metadata = generate_metadata_with_provenance(
            "sample.capi.wasm",
            "sample.wit",
            "sample.h",
            &[],
            8,
            Some("kali-hosted"),
            Some("wasmtime"),
        );
        metadata[field] = value;

        let error =
            parse_metadata(&metadata.to_string()).expect_err("empty provenance field should fail");
        assert!(error.contains(field), "unexpected error: {error}");

        let error =
            cabi_metadata_summary(&metadata).expect_err("empty provenance field should fail");
        assert!(error.contains(field), "unexpected error: {error}");
    }
}
