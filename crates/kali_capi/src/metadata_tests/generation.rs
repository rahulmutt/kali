use super::*;

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
