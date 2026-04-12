//! Node.js API compatibility surface for Kali runtime.
//!
//! This crate currently provides the first tranche of pure-Rust host-side helpers used by the
//! Phase-3 Node-compatibility work. The runtime still gates `--api node`, but the shared helper
//! layer is now concrete enough to be extended incrementally instead of remaining a stub.

use getrandom::fill as fill_random_bytes;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Initialize the Node API compatibility surface.
pub fn node_api_init() {}

/// Process-like execution context for the Node API layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeProcess {
    argv: Vec<String>,
    env: BTreeMap<String, String>,
    cwd: PathBuf,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Default for NodeProcess {
    fn default() -> Self {
        Self {
            argv: Vec::new(),
            env: BTreeMap::new(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
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
        Self {
            argv,
            env,
            cwd: cwd.into(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    /// Command-line arguments.
    pub fn argv(&self) -> &[String] {
        &self.argv
    }

    /// Current working directory.
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Read a host environment variable from the process view.
    pub fn env_get(&self, key: &str) -> Option<&str> {
        self.env.get(key).map(String::as_str)
    }

    /// Set or replace an environment variable in the process view.
    pub fn env_set(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.env.insert(key.into(), value.into())
    }

    /// Return the captured environment view.
    pub fn env(&self) -> &BTreeMap<String, String> {
        &self.env
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

/// Compute a SHA-256 digest as a lowercase hex string.
pub fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    let digest = Sha256::digest(bytes.as_ref());
    format!("{:x}", digest)
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

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub fn to_utf8(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.0.clone())
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

/// Minimal assertion helper used by Node compatibility tests.
pub fn assert_true(condition: bool, message: impl Into<String>) -> Result<(), String> {
    condition.then_some(()).ok_or_else(|| message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_context_tracks_env_and_output() {
        let mut env = BTreeMap::new();
        env.insert("HOME".to_string(), "/tmp/home".to_string());
        let mut process = NodeProcess::with_host_context(
            vec!["node".into(), "script.js".into()],
            env,
            "/workspace/project",
        );

        assert_eq!(process.argv(), &["node", "script.js"]);
        assert_eq!(process.cwd(), Path::new("/workspace/project"));
        assert_eq!(process.env_get("HOME"), Some("/tmp/home"));
        assert_eq!(process.env_get("MISSING"), None);

        process.write_stdout("hello");
        process.write_stderr("oops");
        process.set_exit_code(7);

        assert_eq!(process.stdout(), "hello");
        assert_eq!(process.stderr(), "oops");
        assert_eq!(process.exit_code(), Some(7));
    }

    #[test]
    fn path_helpers_are_lexical_and_deterministic() {
        assert_eq!(
            normalize_path("./foo/../bar//baz"),
            PathBuf::from("bar/baz")
        );
        assert_eq!(
            join_path("/tmp", "project/src"),
            PathBuf::from("/tmp/project/src")
        );
        assert_eq!(
            resolve_path("/tmp/project", "../lib/index.js"),
            PathBuf::from("/tmp/lib/index.js")
        );
        assert_eq!(
            dirname("/tmp/project/src/main.ts"),
            PathBuf::from("/tmp/project/src")
        );
        assert_eq!(basename("/tmp/project/src/main.ts"), "main.ts");
        assert_eq!(extname("/tmp/project/src/main.ts"), ".ts");
    }

    #[test]
    fn crypto_helpers_produce_expected_formats() {
        assert_eq!(
            sha256_hex("hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        assert_eq!(random_bytes(16).expect("random bytes").len(), 16);

        let uuid = random_uuid_v4().expect("uuid");
        assert_eq!(uuid.len(), 36);
        assert_eq!(&uuid[14..15], "4");
        assert!(matches!(&uuid[19..20], "8" | "9" | "a" | "b"));
    }

    #[test]
    fn event_emitter_invokes_listeners_in_order() {
        use std::sync::{Arc, Mutex};

        let emitter = EventEmitter::new();
        let observed: Arc<Mutex<Vec<(String, i32)>>> = Arc::new(Mutex::new(Vec::new()));

        {
            let observed = Arc::clone(&observed);
            emitter.on("message", move |event| {
                observed
                    .lock()
                    .expect("observed mutex")
                    .push((event.event_type().to_string(), 1));
            });
        }
        {
            let observed = Arc::clone(&observed);
            emitter.on("message", move |event| {
                observed
                    .lock()
                    .expect("observed mutex")
                    .push((event.event_type().to_string(), 2));
            });
        }

        let event = NodeEvent::with_detail("message", "payload");
        assert_eq!(emitter.emit(&event), 2);
        assert_eq!(
            observed.lock().expect("observed mutex").clone(),
            vec![("message".to_string(), 1), ("message".to_string(), 2)]
        );
        assert_eq!(event.detail(), Some("payload"));
        assert_eq!(emitter.listener_count("message"), 2);
    }

    #[test]
    fn buffer_and_util_helpers_round_trip() {
        let buffer = NodeBuffer::from_utf8("hello");
        assert_eq!(buffer.as_slice(), b"hello");
        assert_eq!(buffer.to_utf8().expect("utf8"), "hello");

        let bytes = NodeBuffer::from_bytes(vec![1, 2, 3]).into_bytes();
        assert_eq!(bytes, vec![1, 2, 3]);

        let formatted = util_format(&["node", "compat", "layer"]);
        assert_eq!(formatted, "node compat layer");
        assert_eq!(util_inspect(&vec![1, 2, 3]), "[1, 2, 3]");
        assert_eq!(assert_true(true, "ok"), Ok(()));
        assert_eq!(assert_true(false, "fail"), Err("fail".to_string()));
    }
}
