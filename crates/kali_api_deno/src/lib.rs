//! Deno API compatibility surface for Kali runtime.
//!
//! This crate provides the Deno-oriented host-support layer that sits on top of the shared Web
//! baseline. It keeps the Phase-1 standalone surface focused on deterministic file/env/permission
//! views without inventing a browser/runtime shim or a mutable process model.

pub use kali_api_web::{
    atob, btoa, fetch, fill_random_values, local_storage, navigator, parse_url, performance_now,
    resolve_url, session_storage, structured_clone, text_decode, text_encode, AbortController,
    AbortSignal, Base64Error, Blob, BroadcastChannel, CustomEvent, Event, EventTarget, File,
    FileReader, FileReaderState, FormData, FormDataEntry, FormDataValue, Headers, IndexedDb,
    Navigator, Request, Response, Storage, URLSearchParams, WebSocket, WebSocketReadyState, Worker,
    URL,
};

use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

/// Initialize the Deno API compatibility surface.
pub fn deno_api_init() {
    kali_api_web::web_api_init();
}

/// Read-only environment view for the Deno compatibility layer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DenoEnv {
    values: BTreeMap<String, String>,
}

impl DenoEnv {
    /// Create a read-only environment view from host-provided values.
    pub fn new(values: BTreeMap<String, String>) -> Self {
        Self { values }
    }

    /// Read an environment variable from the sandbox-filtered view.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    /// Return a deterministic snapshot of the visible environment.
    pub fn to_object(&self) -> BTreeMap<String, String> {
        self.values.clone()
    }

    /// Iterate over the captured key/value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }
}

/// Light-weight `Deno.args` projection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DenoArgs(Vec<String>);

impl DenoArgs {
    /// Create an argument view from a host-provided vector.
    pub fn new(values: Vec<String>) -> Self {
        Self(values)
    }

    /// Return the recorded arguments.
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    /// Return the recorded arguments as an owned vector.
    pub fn to_vec(&self) -> Vec<String> {
        self.0.clone()
    }
}

/// Canonical Deno permission descriptor subset used by the Phase-1 compatibility facade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DenoPermissionKind {
    Read,
    Write,
    Net,
    Env,
}

/// Permission query result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DenoPermissionStatus {
    Granted,
    Denied,
}

/// Errors produced by the compatibility permission facade.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoPermissionError {
    message: String,
}

impl DenoPermissionError {
    fn unavailable(member: &str) -> Self {
        Self {
            message: format!(
                "{} is not available in the Phase-1 Deno permission facade",
                member
            ),
        }
    }
}

impl std::fmt::Display for DenoPermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DenoPermissionError {}

/// Query-only permission view for the Deno compatibility layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoPermissions {
    read: bool,
    write: bool,
    net: bool,
    env: bool,
}

impl DenoPermissions {
    /// Create a permission view from explicit capability flags.
    pub fn new(read: bool, write: bool, net: bool, env: bool) -> Self {
        Self {
            read,
            write,
            net,
            env,
        }
    }

    /// Create the default open view used by standalone execution in Phase 1.
    pub fn open() -> Self {
        Self::new(true, true, true, true)
    }

    /// Query the current permission state for one descriptor kind.
    pub fn query(
        &self,
        kind: DenoPermissionKind,
    ) -> Result<DenoPermissionStatus, DenoPermissionError> {
        Ok(match kind {
            DenoPermissionKind::Read if self.read => DenoPermissionStatus::Granted,
            DenoPermissionKind::Write if self.write => DenoPermissionStatus::Granted,
            DenoPermissionKind::Net if self.net => DenoPermissionStatus::Granted,
            DenoPermissionKind::Env if self.env => DenoPermissionStatus::Granted,
            DenoPermissionKind::Read
            | DenoPermissionKind::Write
            | DenoPermissionKind::Net
            | DenoPermissionKind::Env => DenoPermissionStatus::Denied,
        })
    }

    /// Recognized-but-unavailable compatibility member.
    pub fn request(
        &self,
        _kind: DenoPermissionKind,
    ) -> Result<DenoPermissionStatus, DenoPermissionError> {
        Err(DenoPermissionError::unavailable(
            "Deno.permissions.request()",
        ))
    }

    /// Recognized-but-unavailable compatibility member.
    pub fn revoke(
        &self,
        _kind: DenoPermissionKind,
    ) -> Result<DenoPermissionStatus, DenoPermissionError> {
        Err(DenoPermissionError::unavailable(
            "Deno.permissions.revoke()",
        ))
    }
}

/// File metadata returned by the Deno compatibility filesystem view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoFileInfo {
    is_file: bool,
    is_dir: bool,
    len: u64,
    readonly: bool,
}

impl DenoFileInfo {
    fn from_metadata(metadata: &fs::Metadata) -> Self {
        Self {
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
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

    pub fn stat(&self, path: impl AsRef<Path>) -> Result<DenoFileInfo, std::io::Error> {
        Ok(DenoFileInfo::from_metadata(&fs::metadata(
            self.resolve(path),
        )?))
    }

    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        self.resolve(path).exists()
    }
}

/// Bundled Deno-oriented execution context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoRuntimeProjection {
    args: DenoArgs,
    env: DenoEnv,
    fs: DenoFs,
    permissions: DenoPermissions,
}

impl DenoRuntimeProjection {
    /// Create a projection using the default-open Phase-1 standalone view.
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self::from_host_context(Vec::new(), BTreeMap::new(), cwd, DenoPermissions::open())
    }

    /// Create a projection from host-supplied context and an explicit permission view.
    pub fn from_host_context(
        args: Vec<String>,
        env: BTreeMap<String, String>,
        cwd: impl Into<PathBuf>,
        permissions: DenoPermissions,
    ) -> Self {
        let cwd = cwd.into();
        Self {
            args: DenoArgs::new(args),
            env: DenoEnv::new(env),
            fs: DenoFs::new(cwd),
            permissions,
        }
    }

    pub fn args(&self) -> &DenoArgs {
        &self.args
    }

    pub fn env(&self) -> &DenoEnv {
        &self.env
    }

    pub fn fs(&self) -> &DenoFs {
        &self.fs
    }

    pub fn permissions(&self) -> &DenoPermissions {
        &self.permissions
    }
}

fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
            Component::Normal(part) => normalized.push(part),
        }
    }

    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}

fn resolve_path(base: impl AsRef<Path>, input: impl AsRef<Path>) -> PathBuf {
    let input = input.as_ref();
    if input.is_absolute() {
        normalize_path(input)
    } else {
        normalize_path(PathBuf::from(base.as_ref()).join(input))
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
