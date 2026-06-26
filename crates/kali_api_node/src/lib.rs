//! Node.js API compatibility surface for Kali runtime.
//!
//! This crate currently provides the first tranche of pure-Rust host-side helpers used by the
//! Phase-3 Node-compatibility work. The runtime still gates `--api node`, but the shared helper
//! layer is now concrete enough to be extended incrementally instead of remaining a stub.

use serde_json::Value;
use std::{
    collections::BTreeMap,
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

mod url;
pub use url::*;

mod util;
pub use util::*;

/// Process-like execution context for the Node API layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeProcess {
    argv: Vec<String>,
    argv0: String,
    env: BTreeMap<String, String>,
    cwd: PathBuf,
    process_id: u32,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Default for NodeProcess {
    fn default() -> Self {
        Self {
            argv: Vec::new(),
            argv0: String::from("node"),
            env: BTreeMap::new(),
            cwd: normalize_path(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
            process_id: std::process::id(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

impl NodeProcess {
    /// Create a process context from host-provided data.
    pub fn with_host_context(
        argv: Vec<String>,
        env: BTreeMap<String, String>,
        cwd: impl Into<PathBuf>,
    ) -> Self {
        let argv0 = argv
            .first()
            .cloned()
            .unwrap_or_else(|| String::from("node"));
        Self {
            argv,
            argv0,
            env,
            cwd: normalize_path(cwd.into()),
            process_id: std::process::id(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    /// Command-line arguments.
    pub fn argv(&self) -> &[String] {
        &self.argv
    }

    /// Original `argv[0]` value associated with the process view.
    pub fn argv0(&self) -> &str {
        &self.argv0
    }

    /// Current working directory.
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Change the working directory tracked by the process view.
    pub fn chdir(&mut self, path: impl AsRef<Path>) {
        self.cwd = resolve_path(&self.cwd, path);
    }

    /// Host process identifier associated with the compatibility view.
    pub fn pid(&self) -> u32 {
        self.process_id
    }

    /// Read a host environment variable from the process view.
    pub fn env_get(&self, key: &str) -> Option<&str> {
        self.env.get(key).map(String::as_str)
    }

    /// Check whether a host environment variable is present in the process view.
    pub fn env_has(&self, key: &str) -> bool {
        self.env.contains_key(key)
    }

    /// Alias for the process-environment presence check helper.
    pub fn has(&self, key: &str) -> bool {
        self.env_has(key)
    }

    /// Set or replace an environment variable in the process view.
    pub fn env_set(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.env.insert(key.into(), value.into())
    }

    /// Remove an environment variable from the process view.
    pub fn env_remove(&mut self, key: &str) -> Option<String> {
        self.env.remove(key)
    }

    /// Alias for the environment removal helper.
    pub fn env_delete(&mut self, key: &str) -> Option<String> {
        self.env_remove(key)
    }

    /// Return the captured environment view.
    pub fn env(&self) -> &BTreeMap<String, String> {
        &self.env
    }

    /// Materialize the captured environment into an owned deterministic snapshot.
    pub fn env_snapshot(&self) -> BTreeMap<String, String> {
        self.env.clone()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn snapshot(&self) -> BTreeMap<String, String> {
        self.env_snapshot()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn env_to_object(&self) -> BTreeMap<String, String> {
        self.env_snapshot()
    }

    /// Alias for the deterministic environment snapshot helper with a generic object-value name.
    pub fn snapshot_object_value(&self) -> BTreeMap<String, String> {
        self.env_snapshot()
    }

    /// Alias for the deterministic environment snapshot helper with an explicit object-value name.
    pub fn env_snapshot_object_value(&self) -> BTreeMap<String, String> {
        self.env_snapshot()
    }

    /// Return the captured environment as a JSON object value.
    pub fn env_snapshot_value(&self) -> Value {
        Value::Object(
            self.env
                .iter()
                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                .collect(),
        )
    }

    /// Alias for the deterministic JSON-ready environment snapshot helper.
    pub fn env_snapshot_json_value(&self) -> Value {
        self.env_snapshot_value()
    }

    /// Alias for the deterministic JSON-ready environment snapshot helper with a generic value name.
    pub fn snapshot_json_value(&self) -> Value {
        self.env_snapshot_value()
    }

    /// Alias for the deterministic environment snapshot helper with a generic value name.
    pub fn snapshot_value(&self) -> Value {
        self.env_snapshot_value()
    }

    /// Alias for the JSON-ready environment snapshot helper.
    pub fn env_to_json_value(&self) -> Value {
        self.env_snapshot_value()
    }

    /// Return the number of captured argv entries.
    pub fn argv_len(&self) -> usize {
        self.argv.len()
    }

    /// Return a captured argv entry by index.
    pub fn argv_at(&self, index: usize) -> Option<&str> {
        self.argv.get(index).map(String::as_str)
    }

    /// Append text to stdout.
    pub fn write_stdout(&mut self, text: impl AsRef<str>) {
        self.stdout.push_str(text.as_ref());
    }

    /// Append text to stderr.
    pub fn write_stderr(&mut self, text: impl AsRef<str>) {
        self.stderr.push_str(text.as_ref());
    }

    /// Captured stdout.
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Captured stderr.
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    /// Record an exit code.
    pub fn set_exit_code(&mut self, exit_code: i32) {
        self.exit_code = Some(exit_code);
    }

    /// Mirror Node's `process.exit(code)`-style termination record in the
    /// compatibility helper surface.
    pub fn exit(&mut self, exit_code: i32) {
        self.set_exit_code(exit_code);
    }

    /// Return the recorded exit code, if any.
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }
}

/// Projection of the Node runtime host helpers used to service common builtins.
#[derive(Clone)]
pub struct NodeRuntimeProjection {
    process: NodeProcess,
    fs: NodeFs,
    fs_promises: NodeFsPromises,
    stream: NodeStream,
    http: NodeHttp,
    child_process: NodeChildProcess,
    os: NodeOs,
    url: NodeUrl,
    events: EventEmitter,
    util: NodeUtil,
    assert: NodeAssert,
}

impl NodeRuntimeProjection {
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        let cwd = cwd.into();
        Self {
            process: NodeProcess::with_host_context(Vec::new(), BTreeMap::new(), cwd.clone()),
            fs: NodeFs::new(cwd.clone()),
            fs_promises: NodeFsPromises::new(cwd),
            stream: NodeStream,
            http: NodeHttp,
            child_process: NodeChildProcess,
            os: NodeOs,
            url: NodeUrl,
            events: EventEmitter::new(),
            util: NodeUtil,
            assert: NodeAssert,
        }
    }

    pub fn from_host_context(
        argv: Vec<String>,
        env: BTreeMap<String, String>,
        cwd: impl Into<PathBuf>,
    ) -> Self {
        let cwd = cwd.into();
        Self {
            process: NodeProcess::with_host_context(argv, env, cwd.clone()),
            fs: NodeFs::new(cwd.clone()),
            fs_promises: NodeFsPromises::new(cwd),
            stream: NodeStream,
            http: NodeHttp,
            child_process: NodeChildProcess,
            os: NodeOs,
            url: NodeUrl,
            events: EventEmitter::new(),
            util: NodeUtil,
            assert: NodeAssert,
        }
    }

    pub fn process(&self) -> &NodeProcess {
        &self.process
    }

    pub fn process_mut(&mut self) -> &mut NodeProcess {
        &mut self.process
    }

    /// Check whether a captured process environment variable is present.
    pub fn env_has(&self, key: &str) -> bool {
        self.process.env_has(key)
    }

    /// Alias for the process-environment presence check helper.
    pub fn has(&self, key: &str) -> bool {
        self.env_has(key)
    }

    /// Return a deterministic snapshot of the captured process environment.
    pub fn env_snapshot(&self) -> BTreeMap<String, String> {
        self.process.env_snapshot()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn snapshot(&self) -> BTreeMap<String, String> {
        self.process.snapshot()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn env_to_object(&self) -> BTreeMap<String, String> {
        self.process.env_to_object()
    }

    /// Alias for the environment removal helper.
    pub fn env_delete(&mut self, key: &str) -> Option<String> {
        self.process.env_delete(key)
    }

    /// Alias for the deterministic environment snapshot helper with a generic object-value name.
    pub fn snapshot_object_value(&self) -> BTreeMap<String, String> {
        self.process.snapshot_object_value()
    }

    /// Alias for the deterministic environment snapshot helper with an explicit object-value name.
    pub fn env_snapshot_object_value(&self) -> BTreeMap<String, String> {
        self.process.env_snapshot_object_value()
    }

    /// Return the captured process environment as a JSON object value.
    pub fn env_snapshot_value(&self) -> Value {
        self.process.env_snapshot_value()
    }

    /// Alias for the deterministic JSON-ready environment snapshot helper.
    pub fn env_snapshot_json_value(&self) -> Value {
        self.process.env_snapshot_json_value()
    }

    /// Alias for the deterministic JSON-ready environment snapshot helper with a generic value name.
    pub fn snapshot_json_value(&self) -> Value {
        self.process.snapshot_json_value()
    }

    /// Alias for the deterministic environment snapshot helper with a generic value name.
    pub fn snapshot_value(&self) -> Value {
        self.process.snapshot_value()
    }

    /// Alias for the JSON-ready environment snapshot helper.
    pub fn env_to_json_value(&self) -> Value {
        self.process.env_to_json_value()
    }

    /// Change the working directory for the full Node compatibility projection.
    pub fn chdir(&mut self, path: impl AsRef<Path>) {
        self.process.chdir(path);
        let cwd = self.process.cwd().to_path_buf();
        self.fs = NodeFs::new(cwd.clone());
        self.fs_promises = NodeFsPromises::new(cwd);
    }

    pub fn fs(&self) -> &NodeFs {
        &self.fs
    }

    pub fn fs_promises(&self) -> &NodeFsPromises {
        &self.fs_promises
    }

    pub fn stream(&self) -> NodeStream {
        self.stream
    }

    pub fn http(&self) -> NodeHttp {
        self.http
    }

    pub fn child_process(&self) -> NodeChildProcess {
        self.child_process
    }

    pub fn os(&self) -> NodeOs {
        self.os
    }

    pub fn url(&self) -> NodeUrl {
        self.url
    }

    pub fn events(&self) -> &EventEmitter {
        &self.events
    }

    pub fn path(&self) -> NodePath {
        NodePath
    }

    pub fn crypto(&self) -> NodeCrypto {
        NodeCrypto
    }

    pub fn util(&self) -> NodeUtil {
        self.util
    }

    pub fn assert(&self) -> NodeAssert {
        self.assert
    }
}

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
