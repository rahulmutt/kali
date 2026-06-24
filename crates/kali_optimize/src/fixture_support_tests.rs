//! Smoke test that wires kali_test_support's filesystem fixtures so the
//! dev-dependency is genuinely exercised (kali_optimize has no other fs tests).
use kali_test_support::fixtures;

#[test]
fn kali_test_support_fixtures_round_trip_files() {
    // fixtures::tempdir() -> tempfile::TempDir; write_file(dir: &Path, rel, contents) -> PathBuf
    let dir = fixtures::tempdir();
    let path = fixtures::write_file(dir.path(), "profile.json", "{\"version\":1}");
    let contents = std::fs::read_to_string(&path).expect("written fixture file is readable");
    assert_eq!(contents, "{\"version\":1}");
}
