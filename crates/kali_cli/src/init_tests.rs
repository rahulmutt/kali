use super::*;
use std::path::PathBuf;
use tempfile::tempdir;

struct CurrentDirGuard(PathBuf);

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

#[test]
fn init_scaffolds_app_project() {
    let dir = tempdir().expect("tempdir");
    let summary = init_project(dir.path(), false).expect("init");

    assert_eq!(summary.manifest_path, dir.path().join("kali.json"));
    assert_eq!(summary.source_path, dir.path().join("main.ts"));
    assert!(!summary.library);

    let manifest = fs::read_to_string(dir.path().join("kali.json")).expect("manifest");
    assert!(manifest.contains("\"schemaVersion\": 1"));
    let source = fs::read_to_string(dir.path().join("main.ts")).expect("source");
    assert!(source.contains("Hello, world!"));
}

#[test]
fn init_scaffolds_library_project() {
    let dir = tempdir().expect("tempdir");
    let summary = init_project(dir.path(), true).expect("init");

    assert_eq!(summary.manifest_path, dir.path().join("kali.json"));
    assert_eq!(summary.source_path, dir.path().join("lib.ts"));
    assert!(summary.library);

    let manifest = fs::read_to_string(dir.path().join("kali.json")).expect("manifest");
    assert!(manifest.contains("\"schemaVersion\": 1"));
    let source = fs::read_to_string(dir.path().join("lib.ts")).expect("source");
    assert!(source.contains("export function add"));
}

#[test]
fn init_rejects_existing_manifest() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), "{}").expect("manifest");

    let error = init_project(dir.path(), false).expect_err("init should fail");
    assert_eq!(error.code, Some(e5::INVALID_CLI_USAGE as u32));
}

#[test]
fn init_scaffolds_a_nested_child_project_from_the_current_directory() {
    let ancestor = tempdir().expect("tempdir");
    fs::write(ancestor.path().join("kali.json"), r#"{"schemaVersion":1}"#)
        .expect("ancestor manifest");
    let child = ancestor.path().join("child");
    fs::create_dir(&child).expect("child directory");

    let previous = std::env::current_dir().expect("current dir");
    let _guard = CurrentDirGuard(previous);
    std::env::set_current_dir(&child).expect("enter child directory");

    let summary = init_current_directory(false).expect("init current directory");

    assert_eq!(summary.root, child);
    assert_eq!(summary.manifest_path, child.join("kali.json"));
    assert_eq!(summary.source_path, child.join("main.ts"));
    assert!(!summary.library);
    assert!(summary.manifest_path.exists());
    assert!(summary.source_path.exists());
}

#[test]
fn init_scaffolds_a_missing_target_directory() {
    let dir = tempdir().expect("tempdir");
    let target = dir.path().join("child");

    let summary = init_project(&target, false).expect("init");

    assert_eq!(summary.root, target);
    assert_eq!(summary.manifest_path, target.join("kali.json"));
    assert_eq!(summary.source_path, target.join("main.ts"));
    assert!(summary.manifest_path.exists());
    assert!(summary.source_path.exists());
}

#[test]
fn init_rejects_non_empty_directory() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("notes.txt"), "keep me").expect("write file");

    let error = init_project(dir.path(), false).expect_err("init should fail");
    assert_eq!(error.code, Some(e5::INVALID_CLI_USAGE as u32));
}
