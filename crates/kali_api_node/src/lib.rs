//! Node.js API compatibility surface for Kali runtime.
//!
//! This crate currently provides the first tranche of pure-Rust host-side helpers used by the
//! Phase-3 Node-compatibility work. The runtime still gates `--api node`, but the shared helper
//! layer is now concrete enough to be extended incrementally instead of remaining a stub.

use std::{
    fs,
    path::{Path, PathBuf},
};
mod assert;
pub use assert::*;

mod buffer;
pub use buffer::*;

mod child_process;
pub use child_process::*;

mod crypto;
pub use crypto::*;

mod events;
pub use events::*;

mod http;
pub use http::*;

mod os;
pub use os::*;

mod path;
pub use path::*;

mod process;
pub use process::*;

mod runtime;
pub use runtime::*;

mod url;
pub use url::*;

mod util;
pub use util::*;

/// Lightweight filesystem view for Node-style file operations.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NodeFs {
    cwd: PathBuf,
}

impl NodeFs {
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self {
            cwd: normalize_path(cwd.into()),
        }
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
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
        let metadata = fs::metadata(&resolved)?;
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

    pub fn stat(&self, path: impl AsRef<Path>) -> Result<NodeFsMetadata, std::io::Error> {
        Ok(NodeFsMetadata::from_metadata(&fs::metadata(
            self.resolve(path),
        )?))
    }

    pub fn lstat(&self, path: impl AsRef<Path>) -> Result<NodeFsMetadata, std::io::Error> {
        Ok(NodeFsMetadata::from_metadata(&fs::symlink_metadata(
            self.resolve(path),
        )?))
    }

    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        self.resolve(path).exists()
    }
}

/// Promise-style filesystem helpers for Node compatibility.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NodeFsPromises {
    fs: NodeFs,
}

impl NodeFsPromises {
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self {
            fs: NodeFs::new(cwd),
        }
    }

    pub fn cwd(&self) -> &Path {
        self.fs.cwd()
    }

    pub fn read_text_file(&self, path: impl AsRef<Path>) -> Result<String, std::io::Error> {
        self.fs.read_text_file(path)
    }

    pub fn read_file(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, std::io::Error> {
        self.fs.read_file(path)
    }

    pub fn write_text_file(
        &self,
        path: impl AsRef<Path>,
        contents: impl AsRef<str>,
    ) -> Result<(), std::io::Error> {
        self.fs.write_text_file(path, contents)
    }

    pub fn write_file(
        &self,
        path: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> Result<(), std::io::Error> {
        self.fs.write_file(path, contents)
    }

    pub fn mkdir(&self, path: impl AsRef<Path>, recursive: bool) -> Result<(), std::io::Error> {
        self.fs.mkdir(path, recursive)
    }

    pub fn readdir(&self, path: impl AsRef<Path>) -> Result<Vec<String>, std::io::Error> {
        self.fs.readdir(path)
    }

    pub fn rename(
        &self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> Result<(), std::io::Error> {
        self.fs.rename(from, to)
    }

    pub fn remove(&self, path: impl AsRef<Path>, recursive: bool) -> Result<(), std::io::Error> {
        self.fs.remove(path, recursive)
    }

    pub fn stat(&self, path: impl AsRef<Path>) -> Result<NodeFsMetadata, std::io::Error> {
        self.fs.stat(path)
    }

    pub fn lstat(&self, path: impl AsRef<Path>) -> Result<NodeFsMetadata, std::io::Error> {
        self.fs.lstat(path)
    }

    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        self.fs.exists(path)
    }
}

/// Basic file metadata for Node-style `stat()` helpers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeFsMetadata {
    is_file: bool,
    is_dir: bool,
    is_symlink: bool,
    len: u64,
    readonly: bool,
}

impl NodeFsMetadata {
    pub fn from_metadata(metadata: &fs::Metadata) -> Self {
        Self {
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            is_symlink: metadata.file_type().is_symlink(),
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

/// A minimal namespace of stream-style byte helpers for Node compatibility.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeStream;

impl NodeStream {
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Vec<u8> {
        bytes.into()
    }

    pub fn from_utf8(text: impl AsRef<str>) -> Vec<u8> {
        text.as_ref().as_bytes().to_vec()
    }

    pub fn concat(left: impl AsRef<[u8]>, right: impl AsRef<[u8]>) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(left.as_ref().len() + right.as_ref().len());
        bytes.extend_from_slice(left.as_ref());
        bytes.extend_from_slice(right.as_ref());
        bytes
    }

    pub fn concat_bytes(&self, left: impl AsRef<[u8]>, right: impl AsRef<[u8]>) -> Vec<u8> {
        Self::concat(left, right)
    }

    pub fn to_utf8(bytes: impl AsRef<[u8]>) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(bytes.as_ref().to_vec())
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
