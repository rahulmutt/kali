use super::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
    discover_binding_package_manifest_path_with_name,
    ensure_compatible_binding_package_manifest,
    ensure_compatible_metadata,
    load_binding_package_manifest,
    load_binding_package_manifest_from_root_with_name,
    load_binding_package_manifest_summary_from_root_with_name,
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

manifest_root = Path(r"{}")
manifest = load_binding_package_manifest(manifest_root)
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
assert discover_binding_package_manifest_path_with_name(Path(r"{}"), "binding-package.json") == manifest_root
assert load_binding_package_manifest_from_root_with_name(Path(r"{}"), "binding-package.json") == manifest
assert load_binding_package_manifest_summary_from_root_with_name(Path(r"{}"), "binding-package.json") == {{
    "moduleName": "sample",
    "hostAbiVersion": HOST_ABI_VERSION,
    "minHostAbiVersion": HOST_ABI_VERSION,
    "runtimeProfiles": [],
    "hostContract": "kali-hosted",
    "runtimeBackend": "wasmtime",
    "maxSpecializations": 8,
    "artifacts": {{
        "exportsHeader": "sample.h",
        "glue": ["shim.py", "support.py"],
        "library": "sample.capi.wasm",
        "metadata": "sample.cabi.json",
    }},
}}

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
        temp_root.display(),
        temp_root.display(),
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
