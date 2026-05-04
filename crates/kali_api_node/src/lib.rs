//! Node.js API compatibility surface for Kali runtime.
//!
//! This crate currently provides the first tranche of pure-Rust host-side helpers used by the
//! Phase-3 Node-compatibility work. The runtime still gates `--api node`, but the shared helper
//! layer is now concrete enough to be extended incrementally instead of remaining a stub.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use getrandom::fill as fill_random_bytes;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::{Digest, Sha256, Sha384, Sha512};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};
use url::Url;

/// Initialize the Node API compatibility surface.
pub fn node_api_init() {}

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

    /// Set or replace an environment variable in the process view.
    pub fn env_set(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.env.insert(key.into(), value.into())
    }

    /// Remove an environment variable from the process view.
    pub fn env_remove(&mut self, key: &str) -> Option<String> {
        self.env.remove(key)
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
    pub fn env_to_object(&self) -> BTreeMap<String, String> {
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

/// Canonicalize a path using lexical `.` / `..` resolution.
///
/// This intentionally stays filesystem-agnostic so the helper remains deterministic for tests
/// and build-time host analysis.
pub fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
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

/// Join a base path and segment.
pub fn join_path(base: impl AsRef<Path>, segment: impl AsRef<Path>) -> PathBuf {
    let mut joined = PathBuf::from(base.as_ref());
    joined.push(segment.as_ref());
    joined
}

/// Resolve a path against a base path.
pub fn resolve_path(base: impl AsRef<Path>, input: impl AsRef<Path>) -> PathBuf {
    let input = input.as_ref();
    if input.is_absolute() {
        normalize_path(input)
    } else {
        normalize_path(join_path(base, input))
    }
}

/// Compute a lexical relative path between two locations.
///
/// This keeps the helper deterministic while still mirroring the shape of
/// Node's `path.relative` API closely enough for the compatibility layer.
pub fn relative_path(from: impl AsRef<Path>, to: impl AsRef<Path>) -> PathBuf {
    let from = resolve_node_path(from);
    let to = resolve_node_path(to);

    if path_root_key(&from) != path_root_key(&to) {
        return to;
    }

    let from_components: Vec<_> = from.components().collect();
    let to_components: Vec<_> = to.components().collect();
    let mut shared_prefix = 0;
    while shared_prefix < from_components.len()
        && shared_prefix < to_components.len()
        && from_components[shared_prefix] == to_components[shared_prefix]
    {
        shared_prefix += 1;
    }

    let mut relative = PathBuf::new();
    for component in from_components.iter().skip(shared_prefix) {
        if matches!(
            component,
            Component::Normal(_) | Component::CurDir | Component::ParentDir
        ) {
            relative.push("..");
        }
    }

    for component in to_components.iter().skip(shared_prefix) {
        match component {
            Component::RootDir | Component::Prefix(_) => {}
            _ => relative.push(component.as_os_str()),
        }
    }

    if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    }
}

fn resolve_node_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        normalize_path(path)
    } else {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        normalize_path(join_path(cwd, path))
    }
}

fn path_root_key(path: impl AsRef<Path>) -> Option<String> {
    let mut components = path.as_ref().components();
    match components.next()? {
        Component::Prefix(prefix) => Some(prefix.as_os_str().to_string_lossy().into_owned()),
        Component::RootDir => Some(String::from("/")),
        _ => None,
    }
}

/// Return the parent directory of a path, or `.` if it has no parent.
pub fn dirname(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Return the final path component as a string.
pub fn basename(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

/// Return the final extension, including the leading `.` when present.
pub fn extname(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return String::new();
    };
    let Some(dot_index) = file_name.rfind('.') else {
        return String::new();
    };
    if dot_index == 0 {
        String::new()
    } else {
        file_name[dot_index..].to_string()
    }
}

/// A namespace-style projection of path helpers used by the Node compatibility layer.
#[derive(Clone, Copy, Debug, Default)]
pub struct NodePath;

impl NodePath {
    pub fn normalize(path: impl AsRef<Path>) -> PathBuf {
        normalize_path(path)
    }

    pub fn join(base: impl AsRef<Path>, segment: impl AsRef<Path>) -> PathBuf {
        join_path(base, segment)
    }

    pub fn resolve(base: impl AsRef<Path>, input: impl AsRef<Path>) -> PathBuf {
        resolve_path(base, input)
    }

    pub fn relative(from: impl AsRef<Path>, to: impl AsRef<Path>) -> PathBuf {
        relative_path(from, to)
    }

    pub fn dirname(path: impl AsRef<Path>) -> PathBuf {
        dirname(path)
    }

    pub fn basename(path: impl AsRef<Path>) -> String {
        basename(path)
    }

    pub fn extname(path: impl AsRef<Path>) -> String {
        extname(path)
    }
}

/// Compute a SHA-256 digest as a lowercase hex string.
pub fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    let digest = Sha256::digest(bytes.as_ref());
    format!("{:x}", digest)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeCryptoError {
    message: String,
}

impl NodeCryptoError {
    fn unsupported_algorithm(algorithm: &str) -> Self {
        Self {
            message: format!("unsupported Node crypto algorithm '{}'", algorithm),
        }
    }

    fn invalid_key_length(algorithm: &str, error: impl std::fmt::Display) -> Self {
        Self {
            message: format!("failed to initialize {} HMAC: {}", algorithm, error),
        }
    }
}

impl std::fmt::Display for NodeCryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for NodeCryptoError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeDigestAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl NodeDigestAlgorithm {
    fn parse(algorithm: impl AsRef<str>) -> Result<Self, NodeCryptoError> {
        match algorithm.as_ref().to_ascii_lowercase().as_str() {
            "sha256" => Ok(Self::Sha256),
            "sha384" => Ok(Self::Sha384),
            "sha512" => Ok(Self::Sha512),
            other => Err(NodeCryptoError::unsupported_algorithm(other)),
        }
    }

    fn digest_hex(self, bytes: impl AsRef<[u8]>) -> String {
        match self {
            Self::Sha256 => format!("{:x}", Sha256::digest(bytes.as_ref())),
            Self::Sha384 => format!("{:x}", Sha384::digest(bytes.as_ref())),
            Self::Sha512 => format!("{:x}", Sha512::digest(bytes.as_ref())),
        }
    }

    fn hmac_hex(
        self,
        key: impl AsRef<[u8]>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<String, NodeCryptoError> {
        match self {
            Self::Sha256 => {
                type HmacSha256 = Hmac<Sha256>;
                let mut mac = HmacSha256::new_from_slice(key.as_ref())
                    .map_err(|error| NodeCryptoError::invalid_key_length("sha256", error))?;
                mac.update(bytes.as_ref());
                Ok(format!("{:x}", mac.finalize().into_bytes()))
            }
            Self::Sha384 => {
                type HmacSha384 = Hmac<Sha384>;
                let mut mac = HmacSha384::new_from_slice(key.as_ref())
                    .map_err(|error| NodeCryptoError::invalid_key_length("sha384", error))?;
                mac.update(bytes.as_ref());
                Ok(format!("{:x}", mac.finalize().into_bytes()))
            }
            Self::Sha512 => {
                type HmacSha512 = Hmac<Sha512>;
                let mut mac = HmacSha512::new_from_slice(key.as_ref())
                    .map_err(|error| NodeCryptoError::invalid_key_length("sha512", error))?;
                mac.update(bytes.as_ref());
                Ok(format!("{:x}", mac.finalize().into_bytes()))
            }
        }
    }
}

/// Namespace-style projection of the common Node crypto helpers.
#[derive(Clone, Copy, Debug, Default)]
pub struct NodeCrypto;

impl NodeCrypto {
    pub fn create_hash(
        algorithm: impl AsRef<str>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<String, NodeCryptoError> {
        NodeDigestAlgorithm::parse(algorithm).map(|algo| algo.digest_hex(bytes))
    }

    pub fn create_hmac(
        algorithm: impl AsRef<str>,
        key: impl AsRef<[u8]>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<String, NodeCryptoError> {
        NodeDigestAlgorithm::parse(algorithm)?.hmac_hex(key, bytes)
    }

    pub fn random_bytes(length: usize) -> Result<Vec<u8>, getrandom::Error> {
        random_bytes(length)
    }

    pub fn random_uuid_v4() -> Result<String, getrandom::Error> {
        random_uuid_v4()
    }
}

/// Return cryptographically random bytes.
pub fn random_bytes(length: usize) -> Result<Vec<u8>, getrandom::Error> {
    let mut bytes = vec![0u8; length];
    fill_random_bytes(&mut bytes)?;
    Ok(bytes)
}

/// Return a random UUIDv4 string.
pub fn random_uuid_v4() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 16];
    fill_random_bytes(&mut bytes)?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    Ok(format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ))
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

    /// Return a deterministic snapshot of the captured process environment.
    pub fn env_snapshot(&self) -> BTreeMap<String, String> {
        self.process.env_snapshot()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn env_to_object(&self) -> BTreeMap<String, String> {
        self.process.env_to_object()
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

/// A minimal Node-style event object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeEvent {
    event_type: String,
    detail: Option<String>,
}

impl NodeEvent {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            detail: None,
        }
    }

    pub fn with_detail(event_type: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            detail: Some(detail.into()),
        }
    }

    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

type Listener = Box<dyn FnMut(&NodeEvent) + Send + 'static>;
type ListenerMap = BTreeMap<String, Vec<Listener>>;

/// Minimal Node-style `EventEmitter`.
#[derive(Clone, Default)]
pub struct EventEmitter {
    listeners: Arc<Mutex<ListenerMap>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on<F>(&self, event_type: impl Into<String>, listener: F)
    where
        F: FnMut(&NodeEvent) + Send + 'static,
    {
        let mut listeners = self
            .listeners
            .lock()
            .expect("event listener mutex poisoned");
        listeners
            .entry(event_type.into())
            .or_default()
            .push(Box::new(listener));
    }

    pub fn emit(&self, event: &NodeEvent) -> usize {
        let mut listeners = self
            .listeners
            .lock()
            .expect("event listener mutex poisoned");
        let Some(event_listeners) = listeners.get_mut(event.event_type()) else {
            return 0;
        };

        for listener in event_listeners.iter_mut() {
            listener(event);
        }

        event_listeners.len()
    }

    pub fn listener_count(&self, event_type: &str) -> usize {
        self.listeners
            .lock()
            .expect("event listener mutex poisoned")
            .get(event_type)
            .map(|listeners| listeners.len())
            .unwrap_or(0)
    }
}

/// Lightweight buffer wrapper for Node-style byte handling.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NodeBuffer(Vec<u8>);

impl NodeBuffer {
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    pub fn from_utf8(text: impl AsRef<str>) -> Self {
        Self(text.as_ref().as_bytes().to_vec())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub fn to_base64(&self) -> String {
        STANDARD.encode(&self.0)
    }

    pub fn from_base64(text: impl AsRef<str>) -> Result<Self, base64::DecodeError> {
        STANDARD.decode(text.as_ref()).map(Self)
    }

    pub fn to_hex(&self) -> String {
        use std::fmt::Write as _;

        let mut output = String::with_capacity(self.0.len() * 2);
        for byte in &self.0 {
            write!(&mut output, "{:02x}", byte).expect("hex formatting should be infallible");
        }
        output
    }

    pub fn from_hex(text: impl AsRef<str>) -> Result<Self, String> {
        let text = text.as_ref();
        if text.len() % 2 != 0 {
            return Err("hex input must contain an even number of digits".to_string());
        }

        let mut bytes = Vec::with_capacity(text.len() / 2);
        for chunk in text.as_bytes().chunks_exact(2) {
            let hi = hex_digit(chunk[0])
                .ok_or_else(|| format!("invalid hex digit '{}'", chunk[0] as char))?;
            let lo = hex_digit(chunk[1])
                .ok_or_else(|| format!("invalid hex digit '{}'", chunk[1] as char))?;
            bytes.push((hi << 4) | lo);
        }

        Ok(Self(bytes))
    }

    pub fn to_utf8(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.0.clone())
    }
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
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

/// Node-style HTTP error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeHttpError {
    message: String,
}

impl NodeHttpError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for NodeHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for NodeHttpError {}

/// Minimal Node-style HTTP client helpers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeHttp;

/// Minimal Node-style HTTP response wrapper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeHttpResponse {
    status: u16,
    body: Vec<u8>,
}

impl NodeHttpResponse {
    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }
}

impl NodeHttp {
    pub fn get(url: impl AsRef<str>) -> Result<NodeHttpResponse, NodeHttpError> {
        let response = reqwest::blocking::get(url.as_ref())
            .and_then(|resp| resp.error_for_status())
            .map_err(|error| {
                NodeHttpError::new(format!("failed to GET '{}': {}", url.as_ref(), error))
            })?;

        let status = response.status().as_u16();
        let body = response
            .bytes()
            .map_err(|error| {
                NodeHttpError::new(format!(
                    "failed to read '{}' response body: {}",
                    url.as_ref(),
                    error
                ))
            })?
            .to_vec();
        Ok(NodeHttpResponse { status, body })
    }

    pub fn request_get(&self, url: impl AsRef<str>) -> Result<NodeHttpResponse, NodeHttpError> {
        Self::get(url)
    }
}

/// Lightweight child-process helper used by the Node compatibility layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeChildProcess;

/// Result of a synchronous child-process run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeChildProcessOutput {
    status: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl NodeChildProcessOutput {
    pub fn status(&self) -> i32 {
        self.status
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeChildProcessError {
    message: String,
}

impl NodeChildProcessError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for NodeChildProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for NodeChildProcessError {}

impl NodeChildProcess {
    pub fn spawn_sync(
        command: impl AsRef<str>,
        args: &[impl AsRef<str>],
    ) -> Result<NodeChildProcessOutput, NodeChildProcessError> {
        let mut command = Command::new(command.as_ref());
        for arg in args {
            command.arg(arg.as_ref());
        }

        let program = command.get_program().to_string_lossy().into_owned();
        let output = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|error| {
                NodeChildProcessError::new(format!("failed to spawn '{}': {}", program, error))
            })?;

        Ok(NodeChildProcessOutput {
            status: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    pub fn spawn(
        &self,
        command: impl AsRef<str>,
        args: &[impl AsRef<str>],
    ) -> Result<NodeChildProcessOutput, NodeChildProcessError> {
        Self::spawn_sync(command, args)
    }
}

/// Lightweight OS view for Node-style environment helpers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeOs;

impl NodeOs {
    pub fn platform(&self) -> &'static str {
        env::consts::OS
    }

    pub fn arch(&self) -> &'static str {
        env::consts::ARCH
    }

    pub fn eol(&self) -> &'static str {
        if cfg!(windows) {
            "\r\n"
        } else {
            "\n"
        }
    }

    pub fn home_dir(&self) -> Option<PathBuf> {
        env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
    }

    pub fn tmpdir(&self) -> PathBuf {
        env::temp_dir()
    }

    pub fn cpus(&self) -> usize {
        std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
    }
}

/// Parse a URL string using the shared support library's URL parser.
pub fn parse_url(input: &str) -> Result<Url, url::ParseError> {
    Url::parse(input)
}

/// Resolve a URL against a base URL string.
pub fn resolve_url(base: &str, input: &str) -> Result<Url, url::ParseError> {
    Url::parse(base)?.join(input)
}

/// Namespace-style wrapper for URL helpers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeUrl;

impl NodeUrl {
    pub fn parse(input: impl AsRef<str>) -> Result<Url, url::ParseError> {
        parse_url(input.as_ref())
    }

    pub fn resolve(base: impl AsRef<str>, input: impl AsRef<str>) -> Result<Url, url::ParseError> {
        resolve_url(base.as_ref(), input.as_ref())
    }
}

/// A tiny `util.format`-style helper for deterministic test output.
pub fn util_format<T: AsRef<str>>(parts: &[T]) -> String {
    parts
        .iter()
        .map(|part| part.as_ref())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A deterministic `inspect` helper for debug-style summaries.
pub fn util_inspect<T: std::fmt::Debug>(value: &T) -> String {
    format!("{:?}", value)
}

/// Namespace-style wrapper for util helpers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeUtil;

impl NodeUtil {
    pub fn format<T: AsRef<str>>(parts: &[T]) -> String {
        util_format(parts)
    }

    pub fn inspect<T: std::fmt::Debug>(value: &T) -> String {
        util_inspect(value)
    }

    pub fn promisify<T: 'static, E: 'static, F>(operation: F) -> Result<T, E>
    where
        F: FnOnce(Box<dyn FnOnce(Result<T, E>)>),
    {
        util_promisify(operation)
    }
}

/// Minimal `util.promisify`-style helper for synchronous callback bridges.
///
/// The callback is invoked exactly once and its result is returned to the caller.
pub fn util_promisify<T: 'static, E: 'static, F>(operation: F) -> Result<T, E>
where
    F: FnOnce(Box<dyn FnOnce(Result<T, E>)>),
{
    let outcome = std::sync::Arc::new(std::sync::Mutex::new(None));
    let slot = std::sync::Arc::clone(&outcome);
    operation(Box::new(move |result| {
        *slot.lock().expect("promisify result mutex poisoned") = Some(result);
    }));

    let result = outcome
        .lock()
        .expect("promisify result mutex poisoned")
        .take()
        .expect("promisify callback was not invoked");
    result
}

/// Minimal assertion helpers used by Node compatibility tests.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeAssert;

impl NodeAssert {
    pub fn ok(condition: bool, message: impl Into<String>) -> Result<(), String> {
        condition.then_some(()).ok_or_else(|| message.into())
    }

    pub fn equal<T>(actual: &T, expected: &T, message: impl Into<String>) -> Result<(), String>
    where
        T: PartialEq + std::fmt::Debug,
    {
        (actual == expected).then_some(()).ok_or_else(|| {
            format!(
                "{}: expected {:?}, got {:?}",
                message.into(),
                expected,
                actual
            )
        })
    }

    pub fn not_equal<T>(actual: &T, expected: &T, message: impl Into<String>) -> Result<(), String>
    where
        T: PartialEq + std::fmt::Debug,
    {
        (actual != expected).then_some(()).ok_or_else(|| {
            format!(
                "{}: value unexpectedly matched {:?}",
                message.into(),
                expected
            )
        })
    }

    pub fn deep_equal<T>(actual: &T, expected: &T, message: impl Into<String>) -> Result<(), String>
    where
        T: PartialEq + std::fmt::Debug,
    {
        Self::equal(actual, expected, message)
    }

    pub fn strict_equal<T>(
        actual: &T,
        expected: &T,
        message: impl Into<String>,
    ) -> Result<(), String>
    where
        T: PartialEq + std::fmt::Debug,
    {
        Self::equal(actual, expected, message)
    }

    pub fn not_strict_equal<T>(
        actual: &T,
        expected: &T,
        message: impl Into<String>,
    ) -> Result<(), String>
    where
        T: PartialEq + std::fmt::Debug,
    {
        Self::not_equal(actual, expected, message)
    }

    pub fn fail(message: impl Into<String>) -> Result<(), String> {
        Err(message.into())
    }
}

/// Backwards-compatible assertion helper used by existing tests.
pub fn assert_true(condition: bool, message: impl Into<String>) -> Result<(), String> {
    NodeAssert::ok(condition, message)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
