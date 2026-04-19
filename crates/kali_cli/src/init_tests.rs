use super::*;
use tempfile::tempdir;

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
fn init_rejects_non_empty_directory() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("notes.txt"), "keep me").expect("write file");

    let error = init_project(dir.path(), false).expect_err("init should fail");
    assert_eq!(error.code, Some(e5::INVALID_CLI_USAGE as u32));
}
