use super::*;

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

#[test]
fn binding_package_manifest_helpers_load_discover_and_summarize_manifests() {
    let temp_root = std::env::temp_dir().join(format!(
        "kali_capi_binding_manifest_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("monotonic time")
            .as_nanos()
    ));
    fs::create_dir_all(&temp_root).expect("temp dir");

    let mut explicit_metadata = generate_metadata_with_provenance(
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
    explicit_metadata["profileDataHash"] = serde_json::json!("sha256:sample-profile");
    let explicit_metadata_path = temp_root.join("sample.cabi.json");
    fs::write(&explicit_metadata_path, explicit_metadata.to_string())
        .expect("write explicit metadata");

    let explicit_manifest = generate_binding_package_manifest(
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
        &["support.py".to_string(), "shim.py".to_string()],
    );
    let explicit_manifest_path = temp_root.join("binding-package.json");
    fs::write(&explicit_manifest_path, explicit_manifest.to_string())
        .expect("write explicit manifest");

    let discovered = discover_binding_package_manifest_path(&temp_root)
        .expect("discover explicit binding package manifest");
    assert_eq!(discovered, explicit_manifest_path);

    let loaded = load_binding_package_manifest(&discovered).expect("load explicit manifest");
    assert_eq!(loaded["kind"], "binding-package");
    assert_eq!(loaded["moduleName"], "sample");
    assert_eq!(loaded["maxSpecializations"], 8);
    assert_eq!(
        loaded["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(loaded["hostContract"], "kali-hosted");
    assert_eq!(loaded["runtimeBackend"], "wasmtime");
    assert_eq!(
        loaded["artifacts"]["glue"],
        serde_json::json!(["shim.py", "support.py"])
    );

    let loaded_summary = binding_package_manifest_summary(&loaded).expect("summarize manifest");
    assert_eq!(loaded_summary["moduleName"], "sample");
    assert_eq!(loaded_summary["hostAbiVersion"], HOST_ABI_VERSION);
    assert_eq!(loaded_summary["minHostAbiVersion"], HOST_ABI_VERSION);
    assert_eq!(
        loaded_summary["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(loaded_summary["hostContract"], "kali-hosted");
    assert_eq!(loaded_summary["runtimeBackend"], "wasmtime");
    assert_eq!(loaded_summary["maxSpecializations"], 8);
    assert_eq!(
        loaded_summary["artifacts"]["glue"],
        serde_json::json!(["shim.py", "support.py"])
    );

    let loaded_bundle_summary = load_binding_package_bundle_summary(&explicit_manifest_path)
        .expect("load and summarize explicit bundle");
    assert_eq!(loaded_bundle_summary["manifest"], loaded_summary);
    assert_eq!(
        loaded_bundle_summary["metadata"],
        cabi_metadata_summary(
            &load_metadata(&explicit_metadata_path).expect("load explicit metadata")
        )
        .expect("summarize explicit metadata")
    );
    assert_eq!(
        loaded_bundle_summary["metadata"]["profileDataHash"],
        "sha256:sample-profile"
    );

    let loaded_summary_from_path = load_binding_package_manifest_summary(&explicit_manifest_path)
        .expect("load and summarize explicit manifest");
    assert_eq!(loaded_summary_from_path, loaded_summary);

    let loaded_summary_from_root = load_binding_package_manifest_summary_from_root(&temp_root)
        .expect("discover, load, and summarize explicit manifest");
    assert_eq!(loaded_summary_from_root, loaded_summary);

    let loaded_bundle_summary_from_root = load_binding_package_bundle_summary_from_root(&temp_root)
        .expect("discover, load, and summarize explicit bundle");
    assert_eq!(loaded_bundle_summary_from_root, loaded_bundle_summary);

    let stem_metadata_path = temp_root.join("sample.cabi.json");
    fs::write(
        &stem_metadata_path,
        generate_metadata_with_provenance(
            "sample.capi.wasm",
            "sample.wit",
            "sample.h",
            &[],
            8,
            Some("kali-hosted"),
            Some("wasmtime"),
        )
        .to_string(),
    )
    .expect("write stem metadata");

    let stem_manifest_path = temp_root.join("sample.binding-package.json");
    fs::write(
        &stem_manifest_path,
        generate_binding_package_manifest(
            "sample",
            "sample.capi.wasm",
            "sample.cabi.json",
            "sample.h",
            &[],
            8,
            &["support.py".to_string(), "shim.py".to_string()],
        )
        .to_string(),
    )
    .expect("write stem manifest");

    let explicit_stem =
        discover_binding_package_manifest_path_with_name(&temp_root, "sample.binding-package.json")
            .expect("discover explicit stem-specific manifest");
    assert_eq!(explicit_stem, stem_manifest_path);

    let loaded_stem = load_binding_package_manifest_from_root_with_name(
        &temp_root,
        "sample.binding-package.json",
    )
    .expect("load explicit stem-specific manifest");
    assert_eq!(loaded_stem["kind"], "binding-package");
    assert_eq!(loaded_stem["moduleName"], "sample");
    assert_eq!(loaded_stem["maxSpecializations"], 8);
    assert_eq!(loaded_stem["runtimeProfiles"], serde_json::json!([]));
    assert_eq!(loaded_stem["hostContract"], "kali-hosted");
    assert_eq!(loaded_stem["runtimeBackend"], "wasmtime");

    let loaded_summary_from_stem = load_binding_package_manifest_summary_from_root_with_name(
        &temp_root,
        "sample.binding-package.json",
    )
    .expect("discover, load, and summarize explicit stem-specific manifest");
    assert_eq!(
        loaded_summary_from_stem,
        binding_package_manifest_summary(&loaded_stem).expect("summarize stem manifest")
    );

    let loaded_bundle_summary_from_stem = load_binding_package_bundle_summary_from_root_with_name(
        &temp_root,
        "sample.binding-package.json",
    )
    .expect("discover, load, and summarize explicit stem-specific bundle");
    assert_eq!(
        loaded_bundle_summary_from_stem["manifest"],
        loaded_summary_from_stem
    );
    assert_eq!(
        loaded_bundle_summary_from_stem["metadata"]["runtimeProfiles"],
        serde_json::json!([])
    );
}
