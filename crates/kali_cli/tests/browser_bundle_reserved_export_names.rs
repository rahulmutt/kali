//! The browser bundle glue reserves `load`, `loadWithImports`,
//! `loadDynamicImport`, and `start` — a user export with one of those names
//! previously built green but emitted an unloadable module (duplicate
//! declaration at import time). It must be a build-time error.
use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn build_bundle_with_export(name: &str) -> (bool, String) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(
        &source_path,
        format!(
            "// kali-tree-shake: {name}\nexport async function {name}(left, right) {{\n  return left - left + right - right;\n}}\n"
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn build_rejects_exports_that_collide_with_reserved_glue_names() {
    for name in ["load", "loadWithImports", "loadDynamicImport", "start"] {
        let (ok, stderr) = build_bundle_with_export(name);
        assert!(!ok, "[{name}] build unexpectedly succeeded");
        assert!(stderr.contains("E5511"), "[{name}] stderr: {stderr}");
        assert!(stderr.contains(name), "[{name}] stderr: {stderr}");
    }
}

#[test]
fn build_accepts_non_reserved_export_names() {
    let (ok, stderr) = build_bundle_with_export("startup");
    assert!(ok, "stderr: {stderr}");
}
