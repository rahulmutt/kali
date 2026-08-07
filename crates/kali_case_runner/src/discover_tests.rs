use super::*;

fn write(dir: &std::path::Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, text).expect("write");
}

const MINIMAL: &str = r#"
[[case]]
name = "c"
args = ["run", "main.js"]
"#;

#[test]
fn discovery_returns_family_relative_stems_sorted() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/pad.toml", MINIMAL);
    write(root.path(), "array/at.toml", MINIMAL);
    write(root.path(), "string/repeat.toml", MINIMAL);
    let found = discover(root.path()).expect("discover");
    let stems: Vec<&str> = found.iter().map(|(stem, _)| stem.as_str()).collect();
    assert_eq!(stems, vec!["array/at", "string/pad", "string/repeat"]);
}

#[test]
fn non_toml_files_are_ignored() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/pad.toml", MINIMAL);
    write(root.path(), "string/README.md", "notes");
    let found = discover(root.path()).expect("discover");
    assert_eq!(found.len(), 1);
}

// A wrong discovery path must not report "0 tests, ok" and turn CI green.
#[test]
fn an_empty_case_tree_is_a_hard_error() {
    let root = tempfile::tempdir().expect("tempdir");
    let err = discover(root.path()).expect_err("must reject empty tree");
    assert!(err.contains("no case files"), "{err}");
}

#[test]
fn a_missing_case_directory_is_a_hard_error() {
    let root = tempfile::tempdir().expect("tempdir");
    let err = discover(&root.path().join("absent")).expect_err("must reject missing dir");
    assert!(err.contains("absent"), "{err}");
}

#[test]
fn a_malformed_case_file_errors_with_its_path() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/broken.toml", "[[case]]\nname = ");
    let err = discover(root.path()).expect_err("must reject malformed toml");
    assert!(err.contains("string/broken.toml"), "{err}");
}
