use std::{fs, path::Path, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn write_browser_bundle_cjs_source(path: &Path) {
    fs::write(
        path,
        "// kali-tree-shake: greet\nfunction greet(name) { return name; }\n",
    )
    .expect("write source");
}

fn assert_browser_bundle_cjs_source_class(filename: &str, inherited: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    write_browser_bundle_cjs_source(&source_path);

    if inherited {
        fs::write(
            dir.path().join("kali.json"),
            r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
        )
        .expect("write manifest");
    }

    let mut command = Command::new(kali_bin());
    command
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--format")
        .arg("cjs");
    if !inherited {
        command.arg("--api").arg("browser");
    }
    let output = command.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let js_path = bundle_dir.join("app.cjs");
    let source_map_path = bundle_dir.join("app.cjs.map");
    let meta_path = bundle_dir.join("app.meta.json");
    assert!(js_path.exists(), "missing {}", js_path.display());
    assert!(
        source_map_path.exists(),
        "missing {}",
        source_map_path.display()
    );
    assert!(meta_path.exists(), "missing {}", meta_path.display());

    let js = fs::read_to_string(&js_path).expect("read bundle js");
    assert!(js.contains("module.exports = exported"), "bundle js: {js}");
    assert!(
        js.contains("sourceMappingURL=app.cjs.map"),
        "bundle js: {js}"
    );

    let source_map: Value =
        serde_json::from_str(&fs::read_to_string(&source_map_path).expect("read source map"))
            .expect("parse source map json");
    assert_eq!(source_map["version"], 3);
    assert_eq!(source_map["file"], "app.cjs");
    assert_eq!(source_map["sources"][0], filename);

    let metadata: Value = serde_json::from_str(&fs::read_to_string(&meta_path).expect("read meta"))
        .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");
    assert_eq!(metadata["artifactKind"], "bundle");

    let envelope: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["command"], "build");
    assert_eq!(envelope["success"], true);
    assert_eq!(envelope["exitCode"], 0);
    assert_eq!(envelope["payload"]["artifactKind"], "bundle");
    assert_eq!(envelope["payload"]["bundleFormat"], "cjs");
    assert!(envelope["errors"]
        .as_array()
        .expect("errors array")
        .is_empty());
}

#[test]
fn build_emits_browser_bundle_cjs_artifacts_in_js_input() {
    assert_browser_bundle_cjs_source_class("app.js", false);
}

#[test]
fn build_emits_browser_bundle_cjs_artifacts_in_ts_input() {
    assert_browser_bundle_cjs_source_class("app.ts", false);
}

#[test]
fn build_emits_browser_bundle_cjs_artifacts_in_jsx_input() {
    assert_browser_bundle_cjs_source_class("app.jsx", false);
}

#[test]
fn build_emits_browser_bundle_cjs_artifacts_in_tsx_input() {
    assert_browser_bundle_cjs_source_class("app.tsx", false);
}

#[test]
fn json_build_emits_inherited_browser_bundle_cjs_artifacts_in_js_input() {
    assert_browser_bundle_cjs_source_class("app.js", true);
}

#[test]
fn json_build_emits_inherited_browser_bundle_cjs_artifacts_in_ts_input() {
    assert_browser_bundle_cjs_source_class("app.ts", true);
}

#[test]
fn json_build_emits_inherited_browser_bundle_cjs_artifacts_in_jsx_input() {
    assert_browser_bundle_cjs_source_class("app.jsx", true);
}

#[test]
fn json_build_emits_inherited_browser_bundle_cjs_artifacts_in_tsx_input() {
    assert_browser_bundle_cjs_source_class("app.tsx", true);
}
