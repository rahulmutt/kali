use super::*;

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
