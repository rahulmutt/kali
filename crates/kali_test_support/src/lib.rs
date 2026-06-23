//! Shared test helpers reused across kali crates' test suites.
//!
//! Keep only genuinely cross-crate helpers here (filesystem fixtures,
//! manifest/process setup). Crate-specific builders belong in each crate's
//! own `test_support` module.

pub mod fixtures {
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    /// Create a throwaway temp directory for fixture files.
    pub fn tempdir() -> TempDir {
        tempfile::tempdir().expect("create tempdir")
    }

    /// Write `contents` to `dir/rel`, creating parent directories, and
    /// return the absolute path written.
    pub fn write_file(dir: &Path, rel: &str, contents: &str) -> PathBuf {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create fixture parent dirs");
        }
        std::fs::write(&path, contents).expect("write fixture file");
        path
    }

    /// Write a `kali.json` manifest into `dir` and return its path.
    pub fn write_manifest(dir: &Path, json: &str) -> PathBuf {
        write_file(dir, "kali.json", json)
    }
}
