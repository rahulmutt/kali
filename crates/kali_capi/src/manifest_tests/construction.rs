use super::*;

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
