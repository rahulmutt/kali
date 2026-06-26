//! Deterministic filesystem view for the Deno compatibility layer.

use std::fs::{self, File as StdFile, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::path::{normalize_path, resolve_path};

/// File metadata returned by the Deno compatibility filesystem view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoFileInfo {
    is_file: bool,
    is_dir: bool,
    is_symlink: bool,
    len: u64,
    readonly: bool,
}

impl DenoFileInfo {
    fn from_metadata(metadata: &fs::Metadata) -> Self {
        let file_type = metadata.file_type();
        Self {
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            is_symlink: file_type.is_symlink(),
            len: metadata.len(),
            readonly: metadata.permissions().readonly(),
        }
    }

    pub fn is_file(&self) -> bool {
        self.is_file
    }

    pub fn is_dir(&self) -> bool {
        self.is_dir
    }

    pub fn is_symlink(&self) -> bool {
        self.is_symlink
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn readonly(&self) -> bool {
        self.readonly
    }
}

/// Deterministic file handle returned by the Deno compatibility filesystem view.
#[derive(Debug)]
pub struct DenoFile {
    file: StdFile,
}

impl DenoFile {
    fn new(file: StdFile) -> Self {
        Self { file }
    }

    pub fn read_to_string(&mut self) -> Result<String, std::io::Error> {
        let mut contents = String::new();
        self.file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    pub fn read_to_end(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let mut contents = Vec::new();
        self.file.read_to_end(&mut contents)?;
        Ok(contents)
    }

    pub fn write_all(&mut self, contents: impl AsRef<[u8]>) -> Result<(), std::io::Error> {
        self.file.write_all(contents.as_ref())
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.file.flush()
    }

    pub fn metadata(&self) -> Result<DenoFileInfo, std::io::Error> {
        Ok(DenoFileInfo::from_metadata(&self.file.metadata()?))
    }
}

/// Minimal filesystem view for Deno-compatible host helpers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoFs {
    cwd: PathBuf,
}

impl DenoFs {
    /// Create a filesystem view rooted at the supplied working directory.
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self {
            cwd: normalize_path(cwd.into()),
        }
    }

    /// Current working directory.
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Update the working-directory view used by relative path resolution.
    #[allow(dead_code)]
    pub(crate) fn chdir(&mut self, cwd: impl Into<PathBuf>) {
        self.cwd = normalize_path(cwd.into());
    }

    fn resolve(&self, path: impl AsRef<Path>) -> PathBuf {
        resolve_path(&self.cwd, path)
    }

    pub fn read_text_file(&self, path: impl AsRef<Path>) -> Result<String, std::io::Error> {
        fs::read_to_string(self.resolve(path))
    }

    pub fn read_file(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, std::io::Error> {
        fs::read(self.resolve(path))
    }

    pub fn write_text_file(
        &self,
        path: impl AsRef<Path>,
        contents: impl AsRef<str>,
    ) -> Result<(), std::io::Error> {
        fs::write(self.resolve(path), contents.as_ref())
    }

    pub fn write_file(
        &self,
        path: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> Result<(), std::io::Error> {
        fs::write(self.resolve(path), contents.as_ref())
    }

    pub fn open(&self, path: impl AsRef<Path>) -> Result<DenoFile, std::io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.resolve(path))?;
        Ok(DenoFile::new(file))
    }

    pub fn create(&self, path: impl AsRef<Path>) -> Result<DenoFile, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(self.resolve(path))?;
        Ok(DenoFile::new(file))
    }

    pub fn mkdir(&self, path: impl AsRef<Path>, recursive: bool) -> Result<(), std::io::Error> {
        let resolved = self.resolve(path);
        if recursive {
            fs::create_dir_all(resolved)
        } else {
            fs::create_dir(resolved)
        }
    }

    pub fn readdir(&self, path: impl AsRef<Path>) -> Result<Vec<String>, std::io::Error> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(self.resolve(path))? {
            let entry = entry?;
            entries.push(entry.file_name().to_string_lossy().into_owned());
        }
        entries.sort();
        Ok(entries)
    }

    pub fn rename(
        &self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> Result<(), std::io::Error> {
        fs::rename(self.resolve(from), self.resolve(to))
    }

    pub fn remove(&self, path: impl AsRef<Path>, recursive: bool) -> Result<(), std::io::Error> {
        let resolved = self.resolve(path);
        let metadata = fs::symlink_metadata(&resolved)?;
        if metadata.is_dir() {
            if recursive {
                fs::remove_dir_all(resolved)
            } else {
                fs::remove_dir(resolved)
            }
        } else {
            fs::remove_file(resolved)
        }
    }

    pub fn stat(&self, path: impl AsRef<Path>) -> Result<DenoFileInfo, std::io::Error> {
        Ok(DenoFileInfo::from_metadata(&fs::metadata(
            self.resolve(path),
        )?))
    }

    pub fn lstat(&self, path: impl AsRef<Path>) -> Result<DenoFileInfo, std::io::Error> {
        Ok(DenoFileInfo::from_metadata(&fs::symlink_metadata(
            self.resolve(path),
        )?))
    }

    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        self.resolve(path).exists()
    }
}

#[cfg(test)]
#[path = "fs_tests.rs"]
mod fs_tests;
