//! Node.js API compatibility surface for Kali runtime.
//!
//! This crate currently provides the first tranche of pure-Rust host-side helpers used by the
//! Phase-3 Node-compatibility work. The runtime still gates `--api node`, but the shared helper
//! layer is now concrete enough to be extended incrementally instead of remaining a stub.

use getrandom::fill as fill_random_bytes;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};
use url::Url;

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

    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        self.resolve(path).exists()
    }
}

/// Basic file metadata for Node-style `stat()` helpers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeFsMetadata {
    is_file: bool,
    is_dir: bool,
    len: u64,
    readonly: bool,
}

impl NodeFsMetadata {
    pub fn from_metadata(metadata: &fs::Metadata) -> Self {
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

    pub fn readonly(&self) -> bool {
        self.readonly
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
#[derive(Default, Debug, Clone, Copy)]
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

    pub fn fail(message: impl Into<String>) -> Result<(), String> {
        Err(message.into())
    }
}

/// Backwards-compatible assertion helper used by existing tests.
pub fn assert_true(condition: bool, message: impl Into<String>) -> Result<(), String> {
    NodeAssert::ok(condition, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
        assert_eq!(
            util_promisify(|callback| callback(Ok::<_, String>(42))),
            Ok(42)
        );
        assert_eq!(assert_true(true, "ok"), Ok(()));
        assert_eq!(assert_true(false, "fail"), Err("fail".to_string()));
    }

    #[test]
    fn assert_helpers_produce_clear_results() {
        assert_eq!(NodeAssert::ok(true, "ok"), Ok(()));
        assert_eq!(NodeAssert::ok(false, "bad"), Err("bad".to_string()));
        assert_eq!(NodeAssert::equal(&3, &3, "equal"), Ok(()));
        assert_eq!(
            NodeAssert::equal(&3, &4, "mismatch"),
            Err("mismatch: expected 4, got 3".to_string())
        );
        assert_eq!(NodeAssert::not_equal(&3, &4, "not equal"), Ok(()));
        assert_eq!(
            NodeAssert::not_equal(&3, &3, "same"),
            Err("same: value unexpectedly matched 3".to_string())
        );
        assert_eq!(
            NodeAssert::deep_equal(&vec![1, 2], &vec![1, 2], "deep"),
            Ok(())
        );
        assert_eq!(NodeAssert::fail("boom"), Err("boom".to_string()));
    }

    #[test]
    fn fs_helpers_round_trip_files_and_directories() {
        let dir = tempdir().expect("tempdir");
        let fs = NodeFs::new(dir.path());

        fs.mkdir("nested", false).expect("mkdir");
        fs.write_text_file("nested/alpha.txt", "alpha")
            .expect("write text");
        fs.write_file("nested/beta.bin", [0, 1, 2])
            .expect("write file");

        assert_eq!(
            fs.read_text_file("nested/alpha.txt").expect("read text"),
            "alpha"
        );
        assert_eq!(
            fs.read_file("nested/beta.bin").expect("read file"),
            vec![0, 1, 2]
        );
        assert_eq!(
            fs.readdir("nested").expect("readdir"),
            vec!["alpha.txt".to_string(), "beta.bin".to_string()]
        );

        let stat = fs.stat("nested/alpha.txt").expect("stat");
        assert!(stat.is_file());
        assert!(!stat.is_dir());
        assert_eq!(stat.len(), 5);

        fs.remove("nested/beta.bin", false).expect("remove file");
        fs.remove("nested", true).expect("remove dir");
        assert!(!fs.exists("nested"));
    }

    #[test]
    fn os_and_url_helpers_expose_expected_views() {
        let os = NodeOs;
        assert!(!os.platform().is_empty());
        assert!(!os.arch().is_empty());
        assert!(matches!(os.eol(), "\n" | "\r\n"));
        assert!(os.cpus() >= 1);
        assert_eq!(os.tmpdir(), std::env::temp_dir());

        let expected_home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from));
        assert_eq!(os.home_dir(), expected_home);

        let parsed = parse_url("https://example.com/path?query=1").expect("url");
        assert_eq!(parsed.as_str(), "https://example.com/path?query=1");

        let resolved = resolve_url("https://example.com/base/", "../child").expect("resolve");
        assert_eq!(resolved.as_str(), "https://example.com/child");
    }
}
