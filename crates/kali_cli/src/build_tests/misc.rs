use super::*;

#[test]
fn build_source_file_writes_valid_wasm_artifact() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "function add(a, b) { return a + b; } add(1, 2);",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_writes_valid_wasm_artifact_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "function add(a, b) { return a + b; } add(1, 2);",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_and_check_source_file_accept_supported_array_callback_slices_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const values = [1, 2, 3];
console.log("map:" + values.map((value) => value).join(','));
console.log("filter:" + [1, 2].filter((value) => value).join(','));
console.log("some:" + [0, 1].some((value) => value));
console.log("every:" + [1, 0].every((value) => value));
console.log("flatMap:" + [1, 2].flatMap((value) => [value]).join(','));
"#,
    )
    .expect("write source");

    check_source_file(&source_path, ApiSurface::Deno, &[], false, false)
        .expect("check should succeed");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_and_check_source_file_accepts_static_string_split_without_separator_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const whole = 'abc'.split();
console.log(whole.length);
console.log(whole[0]);
"#,
    )
    .expect("write source");

    check_source_file(&source_path, ApiSurface::Deno, &[], false, false)
        .expect("check should succeed");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn compile_source_file_uses_incremental_cache_on_repeat_builds() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let first = compile_source_file_with_cache_state(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        false,
        false,
    )
    .expect("first compile");
    assert!(!first.cache_hit);
    let first_cache_path = first
        .cache_path
        .as_ref()
        .expect("cache path should be recorded for project-root builds");
    assert!(
        first_cache_path.exists(),
        "cache path should be written on first build"
    );

    let second = compile_source_file_with_cache_state(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        false,
        false,
    )
    .expect("second compile");
    assert!(second.cache_hit);
    assert_eq!(first.wasm_bytes, second.wasm_bytes);
    assert_eq!(first.cache_path, second.cache_path);
}

#[test]
fn compile_source_file_invalidates_incremental_cache_when_source_changes() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write initial source");

    let first = compile_source_file_with_cache_state(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        false,
        false,
    )
    .expect("first compile");
    let first_cache_path = first
        .cache_path
        .clone()
        .expect("cache path should be recorded for project-root builds");

    fs::write(&source_path, "console.log(2);").expect("rewrite source");

    let second = compile_source_file_with_cache_state(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        false,
        false,
    )
    .expect("second compile after source change");

    assert!(
        !second.cache_hit,
        "source edits must invalidate the incremental cache"
    );
    assert_ne!(
        first_cache_path.as_path(),
        second
            .cache_path
            .as_ref()
            .expect("cache path should still be recorded after source changes")
            .as_path(),
        "source hash should be part of the cache key"
    );
    assert_ne!(
        first.wasm_bytes, second.wasm_bytes,
        "changing the source should produce a distinct artifact"
    );
}

#[test]
fn compile_source_file_with_cache_state_rejects_invalid_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let error = compile_source_file_with_cache_state(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &["wasm-threads".to_string(), "wasm-threads".to_string()],
        false,
        false,
    )
    .expect_err("invalid runtime profiles should fail");

    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)));
}

#[test]
fn incremental_cache_path_includes_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let base = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("base cache path")
    .expect("base cache path should exist");
    let normalized = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[" wasm-threads ".to_string(), "wasm-threads".to_string()],
        None,
        false,
        false,
    )
    .expect("normalized cache path")
    .expect("normalized cache path should exist");
    let canonical = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &["wasm-threads".to_string()],
        None,
        false,
        false,
    )
    .expect("canonical cache path")
    .expect("canonical cache path should exist");

    assert_ne!(base, normalized);
    assert_eq!(normalized, canonical);
}

#[test]
fn incremental_cache_path_separates_build_modes() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let fast = incremental_cache_path(
        &source_path,
        BuildMode::Fast,
        16,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("fast cache path")
    .expect("fast cache path should exist");
    let release = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("release cache path")
    .expect("release cache path should exist");
    let advanced = incremental_cache_path(
        &source_path,
        BuildMode::ReleaseAdvanced,
        16,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("release-advanced cache path")
    .expect("release-advanced cache path should exist");

    assert_ne!(fast, release);
    assert_ne!(fast, advanced);
    assert_ne!(release, advanced);
}

#[test]
fn incremental_cache_path_separates_specialization_budgets() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let narrow = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        8,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("narrow cache path")
    .expect("narrow cache path should exist");
    let wide = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        32,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("wide cache path")
    .expect("wide cache path should exist");

    assert_ne!(narrow, wide);
}

#[test]
fn load_profile_data_file_validates_version_and_normalizes_samples() {
    let dir = tempdir().expect("tempdir");
    let profile_path = dir.path().join("profile.json");
    fs::write(
        &profile_path,
        r#"{"version":1,"samples":[{"kind":"function","key":" hot-path ","weight":2},{"kind":"function","key":"hot-path","weight":3}]}"#,
    )
    .expect("write profile");

    let profile = load_profile_data_file(&profile_path).expect("profile data");
    assert!(profile.is_current_version());
    assert_eq!(
        profile.samples,
        vec![ProfileSample::new(
            ProfileSampleKind::Function,
            "hot-path",
            5
        )]
    );

    fs::write(&profile_path, r#"{"version":2,"samples":[]}"#).expect("rewrite profile");
    let error = load_profile_data_file(&profile_path).expect_err("version mismatch should fail");
    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)));

    fs::write(
        &profile_path,
        r#"{"version":1,"samples":[],"unexpected":true}"#,
    )
    .expect("rewrite profile with unknown field");
    let error = load_profile_data_file(&profile_path).expect_err("unknown fields should fail");
    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)));

    fs::write(&profile_path, " \n\t").expect("rewrite whitespace-only profile");
    let error =
        load_profile_data_file(&profile_path).expect_err("whitespace-only profile should fail");
    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)));

    fs::write(
        &profile_path,
        r#"{"version":1,"samples":[{"kind":"function","key":" ","weight":1}]}"#,
    )
    .expect("rewrite profile with blank key");
    let error = load_profile_data_file(&profile_path).expect_err("blank keys should fail");
    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.message.contains("profile sample[0].key")));

    fs::write(
        &profile_path,
        r#"{"version":1,"samples":[{"kind":"function","key":"hot-path","weight":0}]}"#,
    )
    .expect("rewrite profile with zero weight");
    let error = load_profile_data_file(&profile_path).expect_err("zero weights should fail");
    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.message.contains("profile sample[0].weight")));
}

#[test]
fn compile_source_file_with_profile_data_uses_profile_specific_cache_key() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "function hot_add(a, b) { return a + b; } hot_add(1, 2);",
    )
    .expect("write source");

    let hot_profile = ProfileData::new(vec![ProfileSample::new(
        ProfileSampleKind::Function,
        "hot_add",
        8,
    )]);

    let cold = compile_source_file_with_cache_state_and_profile_data(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        None,
        &[],
        false,
        false,
    )
    .expect("cold compile");
    let hot = compile_source_file_with_cache_state_and_profile_data(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        Some(&hot_profile),
        &[],
        false,
        false,
    )
    .expect("hot compile");

    assert_ne!(cold.cache_path, hot.cache_path);
    assert_eq!(cold.wasm_bytes, hot.wasm_bytes);
}

#[test]
fn build_artifact_metadata_preserves_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let runtime_profiles = vec!["wasm-threads".to_string()];
    let metadata = build_artifact_metadata(
        &source_path,
        "executable",
        BuildMode::Fast,
        "deno",
        &runtime_profiles,
        16,
        None,
        None,
    )
    .expect("build metadata");

    assert_eq!(metadata.runtime_profiles, runtime_profiles);
    assert_eq!(metadata.max_specializations, 16);
}

#[test]
fn build_artifact_metadata_serializes_runtime_provenance_fields() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let metadata = build_artifact_metadata(
        &source_path,
        "component",
        BuildMode::ReleaseAdvanced,
        "browser",
        &["wasm-threads".to_string()],
        24,
        None,
        None,
    )
    .expect("build metadata");

    let json: serde_json::Value = serde_json::from_slice(&serialize_artifact_metadata(&metadata))
        .expect("serialize metadata");

    assert_eq!(json["runtimeProfiles"], serde_json::json!(["wasm-threads"]));
    assert_eq!(json["maxSpecializations"], 24);
    assert_eq!(json["hostContract"], "kali-hosted");
    assert_eq!(json["runtimeBackend"], "wasmtime");
    assert!(json.get("profileDataHash").is_none());
}

#[test]
fn build_artifact_metadata_round_trips_through_schema_validation() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let metadata = build_artifact_metadata(
        &source_path,
        "component",
        BuildMode::ReleaseAdvanced,
        "browser",
        &["wasm-threads".to_string()],
        24,
        None,
        Some(vec![LibraryExport {
            name: "main".to_string(),
            signature: "(input) => number".to_string(),
        }]),
    )
    .expect("build metadata");

    let value = serde_json::to_value(&metadata).expect("serialize metadata");
    validate_artifact_metadata_value(&value).expect("metadata should satisfy schema validation");
}

#[test]
fn build_artifact_metadata_accepts_zero_max_specializations() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let metadata = build_artifact_metadata(
        &source_path,
        "component",
        BuildMode::ReleaseAdvanced,
        "browser",
        &[],
        0,
        None,
        None,
    )
    .expect("build metadata with zero maxSpecializations");

    let value = serde_json::to_value(&metadata).expect("serialize metadata");
    assert_eq!(value["maxSpecializations"], serde_json::json!(0));
    validate_artifact_metadata_value(&value).expect("metadata should satisfy schema validation");
}

#[test]
fn build_artifact_metadata_rejects_negative_max_specializations() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let metadata = build_artifact_metadata(
        &source_path,
        "component",
        BuildMode::ReleaseAdvanced,
        "browser",
        &[],
        24,
        None,
        None,
    )
    .expect("build metadata");

    let mut value = serde_json::to_value(&metadata).expect("serialize metadata");
    value["maxSpecializations"] = serde_json::json!(-1);

    let err = validate_artifact_metadata_value(&value)
        .expect_err("negative maxSpecializations should fail validation");
    assert!(
        err.contains("maxSpecializations"),
        "unexpected error: {err}"
    );
}

#[test]
fn build_browser_bundle_result_round_trips_through_schema_validation() {
    let value = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "source-map", "path": "browser.js.map" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    validate_build_result_value(&value).expect("browser bundle result should validate");
}

#[test]
fn build_browser_bundle_result_accepts_cjs_format_through_schema_validation() {
    let value = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "js-glue", "path": "browser.cjs" },
            { "kind": "source-map", "path": "browser.cjs.map" },
            { "kind": "wasm-module", "path": "browser.wasm" }
        ],
        "exports": [],
        "bundleFormat": "cjs"
    });

    validate_build_result_value(&value).expect("browser bundle cjs result should validate");
}

#[test]
fn build_library_result_round_trips_through_schema_validation() {
    let value = serde_json::json!({
        "artifactKind": "lib",
        "outputPath": "/workspace/dist/lib",
        "sizeBytes": 42,
        "buildMode": "release",
        "sourceHash": "sha256-deadbeef",
        "profileDataHash": "sha256-feedface",
        "metadataPath": "/workspace/dist/lib/lib.meta.json",
        "witPath": "/workspace/dist/lib/lib.wit",
        "artifacts": [
            { "kind": "meta-json", "path": "lib.meta.json" },
            { "kind": "wasm-module", "path": "lib.wasm" }
        ],
        "exports": [
            { "name": "main", "signature": "(input) => number" }
        ]
    });

    validate_build_result_value(&value).expect("library result should validate");
}

#[test]
fn build_capi_result_round_trips_through_schema_validation() {
    let value = serde_json::json!({
        "artifactKind": "capi",
        "outputPath": "/workspace/dist/capi",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "profileDataHash": "sha256-feedface",
        "metadataPath": "/workspace/dist/capi/capi.meta.json",
        "witPath": "/workspace/dist/capi/capi.wit",
        "headerPath": "/workspace/dist/capi/capi.h",
        "artifacts": [
            { "kind": "header", "path": "capi.h" },
            { "kind": "meta-json", "path": "capi.meta.json" },
            { "kind": "wasm-module", "path": "capi.wasm" }
        ],
        "exports": []
    });

    validate_build_result_value(&value).expect("capi result should validate");
}

#[test]
fn build_component_result_accepts_artifact_roles_through_schema_validation() {
    let value = serde_json::json!({
        "artifactKind": "component",
        "outputPath": "/workspace/dist/component",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "metadataPath": "/workspace/dist/component/component.meta.json",
        "witPath": "/workspace/dist/component/component.wit",
        "bindingPackagePath": "/workspace/dist/component/component.binding-package.json",
        "artifacts": [
            { "kind": "wasm-component", "path": "component.wasm", "role": "primary-component" },
            { "kind": "wit", "path": "component.wit", "role": "interface-wit" },
            { "kind": "binding-package", "path": "component.binding-package.json", "role": "binding-package-manifest" },
            { "kind": "meta-json", "path": "component.meta.json" }
        ],
        "exports": []
    });

    validate_build_result_value(&value)
        .expect("component result with artifact roles should validate");
}

#[test]
fn build_result_variants_accept_artifact_roles_through_schema_validation() {
    let values = [
        serde_json::json!({
            "artifactKind": "lib",
            "outputPath": "/workspace/dist/lib",
            "sizeBytes": 42,
            "buildMode": "release",
            "sourceHash": "sha256-deadbeef",
            "profileDataHash": "sha256-feedface",
            "metadataPath": "/workspace/dist/lib/lib.meta.json",
            "witPath": "/workspace/dist/lib/lib.wit",
            "artifacts": [
                { "kind": "wasm-module", "path": "lib.wasm", "role": "primary-library" },
                { "kind": "source-map", "path": "lib.map", "role": "debug-source-map" },
                { "kind": "meta-json", "path": "lib.meta.json" }
            ],
            "exports": [
                { "name": "main", "signature": "(input) => number" }
            ]
        }),
        serde_json::json!({
            "artifactKind": "bundle",
            "outputPath": "/workspace/dist/browser",
            "sizeBytes": 42,
            "buildMode": "release-advanced",
            "sourceHash": "sha256-deadbeef",
            "artifacts": [
                { "kind": "wasm-module", "path": "browser.wasm", "role": "primary-executable" },
                { "kind": "js-glue", "path": "browser.js", "role": "browser-glue" },
                { "kind": "source-map", "path": "browser.map", "role": "debug-source-map" }
            ],
            "exports": [],
            "bundleFormat": "esm"
        }),
        serde_json::json!({
            "artifactKind": "capi",
            "outputPath": "/workspace/dist/capi",
            "sizeBytes": 42,
            "buildMode": "release-advanced",
            "sourceHash": "sha256-deadbeef",
            "profileDataHash": "sha256-feedface",
            "metadataPath": "/workspace/dist/capi/capi.meta.json",
            "witPath": "/workspace/dist/capi/capi.wit",
            "headerPath": "/workspace/dist/capi/capi.h",
            "artifacts": [
                { "kind": "wasm-module", "path": "capi.wasm", "role": "primary-library" },
                { "kind": "source-map", "path": "capi.map", "role": "debug-source-map" },
                { "kind": "header", "path": "capi.h" },
                { "kind": "meta-json", "path": "capi.meta.json" }
            ],
            "exports": []
        }),
    ];

    for value in values {
        validate_build_result_value(&value)
            .expect("build result variant with artifact roles should validate");
    }
}

#[test]
fn build_artifact_metadata_records_profile_data_hash() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let profile_data = ProfileData::new(vec![
        ProfileSample::new(ProfileSampleKind::Function, "hot", 4),
        ProfileSample::new(ProfileSampleKind::Branch, "branch:hot", 3),
    ]);
    let expected_hash = {
        let normalized = profile_data.clone().normalized();
        let profile_json = serde_json::to_vec(&normalized).expect("serialize profile data");
        format!("sha256-{}", hex_encode(Sha256::digest(profile_json)))
    };

    let metadata = build_artifact_metadata(
        &source_path,
        "component",
        BuildMode::Release,
        "deno",
        &[],
        16,
        Some(&profile_data),
        None,
    )
    .expect("build metadata");

    assert_eq!(
        metadata.profile_data_hash.as_deref(),
        Some(expected_hash.as_str())
    );

    let json: serde_json::Value = serde_json::from_slice(&serialize_artifact_metadata(&metadata))
        .expect("serialize metadata");
    assert_eq!(json["profileDataHash"], expected_hash);
}

#[test]
fn build_artifact_metadata_normalizes_equivalent_profile_data_hashes() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let equivalent_profiles = [
        ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Function, " hot-path ", 2),
            ProfileSample::new(ProfileSampleKind::Branch, "branch:hot", 3),
            ProfileSample::new(ProfileSampleKind::Function, "hot-path", 4),
        ]),
        ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Branch, "branch:hot", 3),
            ProfileSample::new(ProfileSampleKind::Function, "hot-path", 6),
        ]),
    ];

    let expected_hash = {
        let normalized = equivalent_profiles[0].clone().normalized();
        let profile_json = serde_json::to_vec(&normalized).expect("serialize profile data");
        format!("sha256-{}", hex_encode(Sha256::digest(profile_json)))
    };

    let hashes: Vec<_> = equivalent_profiles
        .iter()
        .map(|profile_data| {
            let metadata = build_artifact_metadata(
                &source_path,
                "component",
                BuildMode::Release,
                "deno",
                &[],
                16,
                Some(profile_data),
                None,
            )
            .expect("build metadata");

            metadata.profile_data_hash.expect("profile data hash")
        })
        .collect();

    assert_eq!(hashes, vec![expected_hash.clone(), expected_hash.clone()]);
}

#[test]
fn build_artifact_metadata_rejects_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let runtime_profiles = vec!["wasm-threads".to_string(), "wasm-threads".to_string()];
    let error = build_artifact_metadata(
        &source_path,
        "executable",
        BuildMode::Fast,
        "deno",
        &runtime_profiles,
        16,
        None,
        None,
    )
    .expect_err("duplicate runtime profiles should fail");

    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)));
}

#[test]
fn build_artifact_metadata_rejects_unknown_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let runtime_profiles = vec!["fiber-threads".to_string()];
    let error = build_artifact_metadata(
        &source_path,
        "executable",
        BuildMode::Fast,
        "deno",
        &runtime_profiles,
        16,
        None,
        None,
    )
    .expect_err("unknown runtime profiles should fail");

    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)));
}

#[test]
fn output_path_uses_source_stem() {
    let source = PathBuf::from("/tmp/demo/main.ts");
    let output = executable_output_path_for(&source, Some(Path::new("dist")));
    assert_eq!(output, PathBuf::from("dist/main.wasm"));
}

#[test]
fn capi_binding_package_manifest_path_uses_source_stem() {
    let source = PathBuf::from("/tmp/demo/main.ts");
    let output = binding_package_manifest_output_path_for(&source, Some(Path::new("dist")));
    assert_eq!(output, PathBuf::from("dist/main.binding-package.json"));
}

#[test]
fn component_output_paths_use_source_stem_and_binding_manifest() {
    let source = PathBuf::from("/tmp/demo/main.ts");
    let (wasm, wit, meta, binding_package) =
        component_output_paths_for(&source, Some(Path::new("dist")));
    assert_eq!(wasm, PathBuf::from("dist/main.component.wasm"));
    assert_eq!(wit, PathBuf::from("dist/main.wit"));
    assert_eq!(meta, PathBuf::from("dist/main.component.meta.json"));
    assert_eq!(
        binding_package,
        PathBuf::from("dist/main.binding-package.json")
    );
}
