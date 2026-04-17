//! Project scaffolding for `kali init`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use kali_error::{_error_codes::e5, Diagnostic};
use kali_npm::{save_manifest, ProjectManifest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitSummary {
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub source_path: PathBuf,
    pub library: bool,
}

pub fn init_current_directory(lib: bool) -> Result<InitSummary, Diagnostic> {
    let cwd = std::env::current_dir().map_err(|error| {
        Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            format!("failed to read current directory: {}", error),
        )
    })?;

    init_project(cwd, lib)
}

pub fn init_project(root: impl AsRef<Path>, lib: bool) -> Result<InitSummary, Diagnostic> {
    let root = root.as_ref();
    let manifest_path = root.join("kali.json");
    if manifest_path.exists() {
        return Err(Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            format!(
                "project scaffold already exists at '{}'; remove kali.json or choose a new directory",
                root.display()
            ),
        ));
    }

    let source_name = if lib { "lib.ts" } else { "main.ts" };
    let source_path = root.join(source_name);
    if source_path.exists() {
        return Err(Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            format!(
                "project scaffold source '{}' already exists; remove it or choose a new directory",
                source_path.display()
            ),
        ));
    }

    if fs::read_dir(root)
        .map_err(|error| {
            Diagnostic::error(
                e5::OUTPUT_ERROR as u32,
                format!(
                    "failed to inspect scaffold directory '{}': {}",
                    root.display(),
                    error
                ),
            )
        })?
        .next()
        .is_some()
    {
        return Err(Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            format!("init target directory '{}' is not empty", root.display()),
        ));
    }

    fs::create_dir_all(root).map_err(|error| {
        Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to prepare scaffold directory '{}': {}",
                root.display(),
                error
            ),
        )
    })?;

    save_manifest(root, &ProjectManifest::minimal()).map_err(|error| {
        Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write project manifest '{}': {}",
                manifest_path.display(),
                error
            ),
        )
    })?;

    let source_contents = if lib {
        "export function add(a: number, b: number): number {\n  return a + b;\n}\n"
    } else {
        "console.log(\"Hello, world!\");\n"
    };
    fs::write(&source_path, source_contents).map_err(|error| {
        Diagnostic::error(
            e5::OUTPUT_ERROR as u32,
            format!(
                "failed to write starter source '{}': {}",
                source_path.display(),
                error
            ),
        )
    })?;

    Ok(InitSummary {
        root: root.to_path_buf(),
        manifest_path,
        source_path,
        library: lib,
    })
}

#[cfg(test)]
mod tests {
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
}
