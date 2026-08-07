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

// `trim_end_matches(".toml")` would strip every trailing `.toml`, collapsing
// `pad.toml` and `pad.toml.toml` to the same stem `pad` -- a silent trial-id
// collision. `file_stem` only ever strips the last extension, so the two
// stay distinct.
#[test]
fn a_repeated_toml_suffix_does_not_collapse_into_a_duplicate_stem() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/pad.toml", MINIMAL);
    write(root.path(), "string/pad.toml.toml", MINIMAL);
    let found = discover(root.path()).expect("discover");
    let mut stems: Vec<&str> = found.iter().map(|(stem, _)| stem.as_str()).collect();
    stems.sort();
    assert_eq!(stems, vec!["string/pad", "string/pad.toml"]);
}

// A miscased extension must not vanish silently -- that is the same failure
// class the empty-tree guard exists to prevent (a case file that never
// runs, unreported), just at per-file scale.
#[test]
fn a_miscased_toml_extension_is_still_discovered() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/UP.TOML", MINIMAL);
    let found = discover(root.path()).expect("discover");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, "string/UP");
}

// Distinct from "does not exist": the path is present, just not a
// directory. Reporting "does not exist" for an existing file would send a
// case author looking for a typo that isn't there.
#[test]
fn a_case_directory_that_is_actually_a_file_is_a_hard_error_naming_it() {
    let root = tempfile::tempdir().expect("tempdir");
    let file_path = root.path().join("not_a_dir");
    std::fs::write(&file_path, "oops").expect("write");
    let err = discover(&file_path).expect_err("must reject a file where a directory is expected");
    assert!(!err.contains("does not exist"), "{err}");
    assert!(err.contains("not a directory"), "{err}");
}
