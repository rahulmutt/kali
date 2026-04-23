use super::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
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
    let metadata = generate_metadata_with_provenance(
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
    let metadata = generate_metadata_with_provenance(
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

    let summary = load_metadata_summary_from_root(&temp_root)
        .expect("load and summarize cabi metadata from root");
    assert_eq!(summary["kind"], "cabi-metadata");
    assert_eq!(
        summary["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(summary["hostContract"], "kali-hosted");
    assert_eq!(summary["runtimeBackend"], "wasmtime");

    let explicit_summary =
        load_metadata_summary_from_root_with_name(&temp_root, "sample.capi.meta.json")
            .expect("load and summarize explicit cabi metadata sidecar");
    assert_eq!(explicit_summary, summary);
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
fn python_binding_package_metadata_is_present() {
    let cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = cargo_manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root");
    let pyproject_path = repo_root.join("bindings/python/pyproject.toml");
    let readme_path = repo_root.join("bindings/python/README.md");

    let pyproject = fs::read_to_string(&pyproject_path).expect("read python pyproject");
    assert!(pyproject.contains("name = \"kali-capi\""));
    assert!(pyproject.contains("version = \"0.1.0\""));
    assert!(pyproject.contains("build-backend = \"setuptools.build_meta\""));
    assert!(pyproject.contains("include = [\"kali_capi*\"]"));

    let readme = fs::read_to_string(&readme_path).expect("read python binding readme");
    assert!(readme.contains("kali_capi"));
    assert!(readme.contains("deterministic Python ctypes bindings for Kali's stable C ABI"));
}

#[test]
fn header_generation_produces_c_compatible_prototypes() {
    let header = generate_header("lib", &[Export::new("add", 2), Export::new("1bad-name", 0)]);

    assert!(header.contains("#include <stdint.h>"));
    assert!(header.contains("extern int32_t add(int32_t arg0, int32_t arg1);"));
    assert!(header.contains("extern int32_t _1bad_name(void);"));
}

#[test]
fn identifier_sanitization_is_deterministic() {
    assert_eq!(sanitize_identifier("foo-bar"), "foo_bar");
    assert_eq!(sanitize_identifier("1foo"), "_1foo");
    assert_eq!(sanitize_identifier(""), "_");
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

    let loaded_summary_from_path = load_binding_package_manifest_summary(&explicit_manifest_path)
        .expect("load and summarize explicit manifest");
    assert_eq!(loaded_summary_from_path, loaded_summary);

    let loaded_summary_from_root = load_binding_package_manifest_summary_from_root(&temp_root)
        .expect("discover, load, and summarize explicit manifest");
    assert_eq!(loaded_summary_from_root, loaded_summary);

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
fn binding_package_manifest_summary_normalizes_string_lists() {
    let manifest = serde_json::json!({
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
            "exportsHeader": "sample.h",
            "glue": ["z.py", "a.py", "z.py"],
            "library": "sample.capi.wasm",
            "metadata": "sample.cabi.json"
        }
    });

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
fn binding_package_manifest_parsing_rejects_non_string_provenance_fields() {
    for (field, value) in [("hostContract", "1"), ("runtimeBackend", "true")] {
        let error = parse_binding_package_manifest(&format!(
            r#"{{
                "schemaVersion": 1,
                "kind": "binding-package",
                "moduleName": "sample",
                "hostAbiVersion": 2,
                "{}": {},
                "artifacts": {{
                    "library": "sample.capi.wasm",
                    "metadata": "sample.cabi.json",
                    "exportsHeader": "sample.h",
                    "glue": []
                }}
            }}"#,
            field, value
        ))
        .expect_err("invalid provenance field should fail");

        assert!(error.contains(field), "unexpected error: {error}");
    }
}

#[test]
fn binding_package_manifest_summary_rejects_non_string_provenance_fields() {
    for (field, value) in [
        ("hostContract", serde_json::json!(1)),
        ("runtimeBackend", serde_json::json!(false)),
    ] {
        let manifest = serde_json::json!({
            "schemaVersion": 1,
            "kind": "binding-package",
            "moduleName": "sample",
            "hostAbiVersion": HOST_ABI_VERSION,
            "minHostAbiVersion": HOST_ABI_VERSION,
            (field): value,
            "artifacts": {
                "exportsHeader": "sample.h",
                "glue": [],
                "library": "sample.capi.wasm",
                "metadata": "sample.cabi.json"
            }
        });

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
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
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
fn python_binding_wraps_generated_header_exports() {
    if Command::new("python3").arg("--version").output().is_err() {
        return;
    }

    let cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = cargo_manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root");
    let binding_root = repo_root.join("bindings/python");
    let temp_root = std::env::temp_dir().join(format!(
        "kali_capi_python_binding_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("monotonic time")
            .as_nanos()
    ));
    fs::create_dir_all(&temp_root).expect("temp dir");

    let header = generate_header("sample", &[Export::new("add", 2), Export::new("zero", 0)]);
    let metadata = generate_metadata("sample.capi.wasm", "sample.wit", "sample.exports.h");
    let header_path = temp_root.join("sample.h");
    let metadata_path = temp_root.join("sample.cabi.json");
    let script_path = temp_root.join("exercise_binding.py");
    fs::write(&header_path, header).expect("write header fixture");
    fs::write(&metadata_path, metadata.to_string()).expect("write metadata fixture");

    let script = format!(
        r#"from pathlib import Path
import sys

sys.path.insert(0, r"{}")
from kali_capi import (
    HOST_ABI_VERSION,
    BindingPackageManifest,
    Export,
    KaliCAPI,
    ensure_compatible_binding_package_manifest,
    ensure_compatible_metadata,
    load_binding_package_manifest,
    load_metadata,
    parse_exports,
)

header = Path(r"{}").read_text()
metadata_path = Path(r"{}")
metadata = load_metadata(metadata_path)
exports = parse_exports(header)
assert exports == [Export("add", 2), Export("zero", 0)]
assert metadata.host_abi_version == HOST_ABI_VERSION
assert metadata.min_host_abi_version == HOST_ABI_VERSION
assert metadata.artifacts == {{
    "exportsHeader": "sample.exports.h",
    "wit": "sample.wit",
    "wasmModule": "sample.capi.wasm",
}}
assert ensure_compatible_metadata(metadata) == metadata

manifest = load_binding_package_manifest(Path(r"{}"))
assert isinstance(manifest, BindingPackageManifest)
assert manifest.host_abi_version == HOST_ABI_VERSION
assert manifest.min_host_abi_version == HOST_ABI_VERSION
assert manifest.module_name == "sample"
assert manifest.runtime_profiles == ()
assert manifest.host_contract == "kali-hosted"
assert manifest.runtime_backend == "wasmtime"
assert manifest.artifacts["glue"] == ("shim.py", "support.py")
assert manifest.artifacts["library"] == "sample.capi.wasm"
assert ensure_compatible_binding_package_manifest(manifest) == manifest

class DummyLibrary:
    def __init__(self):
        self.calls = []

    def add(self, left, right):
        self.calls.append(("add", left, right))
        return left + right

    def zero(self):
        self.calls.append(("zero",))
        return 7

binding = KaliCAPI.from_binding_package(DummyLibrary(), Path(r"{}"))
assert binding.exports == tuple(exports)
assert binding.runtime_profiles == ()
assert binding.host_contract == "kali-hosted"
assert binding.runtime_backend == "wasmtime"
assert binding.add(2, 3) == 5
assert binding.zero() == 7
assert binding._library.calls == [("add", 2, 3), ("zero",)]
"#,
        binding_root.display(),
        header_path.display(),
        metadata_path.display(),
        temp_root.join("binding-package.json").display(),
        temp_root.display(),
    );
    fs::write(&script_path, script).expect("write python exercise script");

    let binding_manifest = generate_binding_package_manifest(
        "sample",
        "sample.capi.wasm",
        "sample.cabi.json",
        "sample.h",
        &[],
        8,
        &["support.py".to_string(), "shim.py".to_string()],
    );
    fs::write(
        temp_root.join("binding-package.json"),
        binding_manifest.to_string(),
    )
    .expect("write binding manifest fixture");
    fs::write(temp_root.join("sample.capi.wasm"), b"").expect("write library placeholder");

    let status = Command::new("python3")
        .arg(&script_path)
        .current_dir(repo_root)
        .status()
        .expect("run python binding test");
    assert!(
        status.success(),
        "python binding helper exited with {status}"
    );
}

#[test]
fn python_binding_auto_discovers_stem_specific_binding_package_manifest() {
    if Command::new("python3").arg("--version").output().is_err() {
        return;
    }

    let cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = cargo_manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root");
    let binding_root = repo_root.join("bindings/python");
    let temp_root = std::env::temp_dir().join(format!(
        "kali_capi_python_binding_{}_discover_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("monotonic time")
            .as_nanos()
    ));
    fs::create_dir_all(&temp_root).expect("temp dir");

    let header = generate_header("sample", &[Export::new("add", 2), Export::new("zero", 0)]);
    let metadata = generate_metadata("sample.capi.wasm", "sample.wit", "sample.exports.h");
    let header_path = temp_root.join("sample.h");
    let metadata_path = temp_root.join("sample.cabi.json");
    let script_path = temp_root.join("exercise_discovered_binding.py");
    fs::write(&header_path, header).expect("write header fixture");
    fs::write(&metadata_path, metadata.to_string()).expect("write metadata fixture");

    let script = format!(
        r#"from pathlib import Path
import sys

sys.path.insert(0, r"{}")
from kali_capi import KaliCAPI

class DummyLibrary:
    def __init__(self):
        self.calls = []

    def add(self, left, right):
        self.calls.append(("add", left, right))
        return left + right

    def zero(self):
        self.calls.append(("zero",))
        return 7

binding = KaliCAPI.from_binding_package(DummyLibrary(), Path(r"{}"))
assert binding.runtime_profiles == ()
assert binding.host_contract == "kali-hosted"
assert binding.runtime_backend == "wasmtime"
assert binding.add(2, 3) == 5
assert binding.zero() == 7
assert binding._library.calls == [("add", 2, 3), ("zero",)]
"#,
        binding_root.display(),
        temp_root.display(),
    );
    fs::write(&script_path, script).expect("write python exercise script");

    let binding_manifest = generate_binding_package_manifest(
        "sample",
        "sample.capi.wasm",
        "sample.cabi.json",
        "sample.h",
        &[],
        8,
        &["support.py".to_string(), "shim.py".to_string()],
    );
    fs::write(
        temp_root.join("sample.binding-package.json"),
        binding_manifest.to_string(),
    )
    .expect("write binding manifest fixture");
    fs::write(temp_root.join("sample.capi.wasm"), b"").expect("write library placeholder");

    let status = Command::new("python3")
        .arg(&script_path)
        .current_dir(repo_root)
        .status()
        .expect("run python binding test");
    assert!(
        status.success(),
        "python binding helper auto-discovery exited with {status}"
    );
}

#[test]
fn python_binding_rejects_incompatible_host_abi_metadata() {
    if Command::new("python3").arg("--version").output().is_err() {
        return;
    }

    let cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = cargo_manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root");
    let binding_root = repo_root.join("bindings/python");
    let temp_root = std::env::temp_dir().join(format!(
        "kali_capi_python_binding_{}_reject_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("monotonic time")
            .as_nanos()
    ));
    fs::create_dir_all(&temp_root).expect("temp dir");

    let header = generate_header("sample", &[Export::new("add", 2)]);
    let metadata = generate_metadata("sample.capi.wasm", "sample.wit", "sample.exports.h");
    let header_path = temp_root.join("sample.h");
    let metadata_path = temp_root.join("sample.cabi.json");
    let script_path = temp_root.join("exercise_incompatible_binding.py");
    fs::write(&header_path, header).expect("write header fixture");
    fs::write(&metadata_path, metadata.to_string()).expect("write metadata fixture");

    let script = format!(
        r#"from pathlib import Path
import sys

sys.path.insert(0, r"{}")
from kali_capi import KaliCAPI

class DummyLibrary:
    def add(self, left, right):
        return left + right

header = Path(r"{}").read_text()
metadata = Path(r"{}").read_text()
try:
    KaliCAPI.from_header_and_metadata(
        DummyLibrary(),
        header,
        metadata,
        available_host_abi_version=3,
    )
except ValueError as error:
    assert "incompatible" in str(error)
else:
    raise AssertionError("expected incompatible metadata to be rejected")
"#,
        binding_root.display(),
        header_path.display(),
        metadata_path.display(),
    );
    fs::write(&script_path, script).expect("write python exercise script");

    let status = Command::new("python3")
        .arg(&script_path)
        .current_dir(repo_root)
        .status()
        .expect("run python binding test");
    assert!(
        status.success(),
        "python binding helper exited with {status}"
    );
}

#[test]
fn python_unittest_smoke_covers_the_binding_helper_package() {
    if Command::new("python3").arg("--version").output().is_err() {
        return;
    }

    let cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = cargo_manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root");
    let binding_root = repo_root.join("bindings/python");

    let status = Command::new("python3")
        .arg("-m")
        .arg("unittest")
        .arg("discover")
        .arg("-s")
        .arg("tests")
        .arg("-t")
        .arg(".")
        .current_dir(&binding_root)
        .status()
        .expect("run python unittest smoke");
    assert!(
        status.success(),
        "python unittest smoke exited with {status}"
    );
}

#[test]
fn javascript_binding_package_metadata_is_present() {
    let cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = cargo_manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root");
    let package_json_path = repo_root.join("bindings/node/package.json");
    let readme_path = repo_root.join("bindings/node/README.md");

    let package_json = fs::read_to_string(&package_json_path).expect("read node package json");
    assert!(package_json.contains("\"name\": \"kali-capi-node\""));
    assert!(package_json.contains("\"type\": \"module\""));
    assert!(package_json.contains("\"import\": \"./kali_capi.mjs\""));
    assert!(package_json.contains("\"require\": \"./kali_capi.cjs\""));

    let readme = fs::read_to_string(&readme_path).expect("read node binding readme");
    assert!(readme.contains("kali_capi Node binding helper"));
    assert!(readme.contains("deterministic helpers for generated C headers"));
    assert!(readme.contains("ESM `import` and CommonJS `require` entrypoints"));
}

#[test]
fn javascript_node_test_smoke_covers_the_binding_helper_package() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = cargo_manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root");
    let binding_root = repo_root.join("bindings/node");

    let status = Command::new("node")
        .arg("--test")
        .arg("tests/test_kali_capi.mjs")
        .current_dir(&binding_root)
        .status()
        .expect("run node unittest smoke");
    assert!(status.success(), "node unittest smoke exited with {status}");
}
