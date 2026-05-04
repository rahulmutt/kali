//! Deno API compatibility surface for Kali runtime.
//!
//! This crate provides the Deno-oriented host-support layer that sits on top of the shared Web
//! baseline. It keeps the Phase-1 standalone surface focused on deterministic file/env/permission
//! views without inventing a browser/runtime shim or a mutable process model.

pub use kali_api_web::{
    atob, btoa, crypto, fetch, fill_random_values, local_storage, navigator, parse_url,
    performance_now, random_uuid, resolve_url, session_storage, structured_clone, text_decode,
    text_encode, AbortController, AbortSignal, Base64Error, Blob, BroadcastChannel, Crypto,
    CustomEvent, Event, EventTarget, File, FileReader, FileReaderState, FormData, FormDataEntry,
    FormDataValue, Headers, IndexedDB, IndexedDb, Navigator, ReadableStream, Request, Response,
    Storage, TransformStream, URLSearchParams, WebSocket, WebSocketReadyState, Worker,
    WritableStream, URL,
};

use std::{
    collections::BTreeMap,
    fs::{self, File as StdFile, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    thread::{self, JoinHandle},
};

use serde_json::Value;

/// Initialize the Deno API compatibility surface.
pub fn deno_api_init() {
    kali_api_web::web_api_init();
}

/// Deterministic environment view for the Deno compatibility layer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DenoEnv {
    values: BTreeMap<String, String>,
}

impl DenoEnv {
    /// Create an environment view from host-provided values.
    pub fn new(values: BTreeMap<String, String>) -> Self {
        Self { values }
    }

    /// Read an environment variable from the sandbox-filtered view.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    /// Check whether an environment variable is present in the captured view.
    pub fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Set or replace an environment variable in the captured view.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.values.insert(key.into(), value.into())
    }

    /// Remove an environment variable from the captured view.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.values.remove(key)
    }

    /// Alias for the mutable environment removal helper.
    pub fn delete(&mut self, key: &str) -> Option<String> {
        self.remove(key)
    }

    /// Return a deterministic snapshot of the visible environment.
    pub fn to_object(&self) -> BTreeMap<String, String> {
        self.values.clone()
    }

    /// Alias for the deterministic environment snapshot helper with a generic object-value name.
    pub fn snapshot_object_value(&self) -> BTreeMap<String, String> {
        self.to_object()
    }

    /// Return the visible environment as a JSON object value.
    pub fn to_json_value(&self) -> Value {
        Value::Object(
            self.values
                .iter()
                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                .collect(),
        )
    }

    /// Alias for the deterministic JSON-ready environment snapshot helper with a generic value name.
    pub fn snapshot_json_value(&self) -> Value {
        self.to_json_value()
    }

    /// Alias for the deterministic environment snapshot helper with a generic value name.
    pub fn snapshot_value(&self) -> Value {
        self.to_json_value()
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

/// Result of a Deno-style command invocation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoCommandOutput {
    status: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl DenoCommandOutput {
    pub fn status(&self) -> i32 {
        self.status
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    pub fn text_stdout(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.stdout.clone())
    }

    pub fn text_stderr(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.stderr.clone())
    }
}

/// Error produced by the Deno command helper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoCommandError {
    message: String,
}

impl DenoCommandError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for DenoCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DenoCommandError {}

/// Minimal Deno-style process command helper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoCommand {
    command: String,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    cwd: Option<PathBuf>,
}

impl DenoCommand {
    /// Create a command builder for one executable.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: None,
        }
    }

    /// Append one argument.
    pub fn arg(&mut self, arg: impl Into<String>) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    /// Append multiple arguments.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Set or replace one environment variable for the child process.
    pub fn env(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set the child process working directory.
    pub fn current_dir(&mut self, cwd: impl Into<PathBuf>) -> &mut Self {
        self.cwd = Some(normalize_path(cwd.into()));
        self
    }

    /// Run the command to completion, capturing stdout/stderr.
    pub fn output(&self) -> Result<DenoCommandOutput, DenoCommandError> {
        let mut command = Command::new(&self.command);
        command.args(&self.args);
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        for (key, value) in &self.env {
            command.env(key, value);
        }
        let output = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|error| {
                DenoCommandError::new(format!("failed to run '{}': {}", self.command, error))
            })?;
        Ok(DenoCommandOutput {
            status: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }

    /// Synonym for [`output`](Self::output) to match the builder-style API.
    pub fn spawn(&self) -> Result<DenoCommandOutput, DenoCommandError> {
        self.output()
    }
}

/// Deterministic TCP connection wrapper for the Deno compatibility surface.
#[derive(Debug)]
pub struct DenoTcpConnection {
    stream: TcpStream,
}

impl DenoTcpConnection {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    /// Read the remaining bytes from the connection.
    pub fn read_to_end(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let mut contents = Vec::new();
        self.stream.read_to_end(&mut contents)?;
        Ok(contents)
    }

    /// Write bytes to the connection.
    pub fn write_all(&mut self, contents: impl AsRef<[u8]>) -> Result<(), std::io::Error> {
        self.stream.write_all(contents.as_ref())
    }

    /// Flush buffered writes.
    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.stream.flush()
    }

    /// Return the local socket address.
    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.stream.local_addr()
    }

    /// Return the peer socket address.
    pub fn peer_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.stream.peer_addr()
    }

    /// Close the write half of the connection.
    pub fn shutdown_write(&self) -> Result<(), std::io::Error> {
        self.stream.shutdown(Shutdown::Write)
    }

    /// Close the connection in both directions.
    pub fn shutdown(&self) -> Result<(), std::io::Error> {
        self.stream.shutdown(Shutdown::Both)
    }
}

/// Deterministic TCP listener wrapper for the Deno compatibility surface.
#[derive(Debug)]
pub struct DenoTcpListener {
    listener: TcpListener,
}

impl DenoTcpListener {
    fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    /// Accept a single incoming connection.
    pub fn accept(&self) -> Result<(DenoTcpConnection, SocketAddr), std::io::Error> {
        let (stream, addr) = self.listener.accept()?;
        Ok((DenoTcpConnection::new(stream), addr))
    }

    /// Return the local socket address.
    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.listener.local_addr()
    }
}

/// Result of a Deno-style HTTP server helper.
#[derive(Debug)]
pub struct DenoHttpServer {
    local_addr: SocketAddr,
    join_handle: Option<JoinHandle<std::io::Result<()>>>,
}

impl DenoHttpServer {
    /// Return the bound address.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Wait for the one-shot server worker to finish.
    pub fn join(mut self) -> Result<(), std::io::Error> {
        let handle = self
            .join_handle
            .take()
            .expect("DenoHttpServer join handle already consumed");
        match handle.join() {
            Ok(result) => result,
            Err(_) => Err(std::io::Error::other("Deno.serve worker panicked")),
        }
    }
}

/// Connect to one TCP peer using the deterministic Deno compatibility surface.
pub fn connect(hostname: impl AsRef<str>, port: u16) -> Result<DenoTcpConnection, std::io::Error> {
    let stream = TcpStream::connect((hostname.as_ref(), port))?;
    let _ = stream.set_nodelay(true);
    Ok(DenoTcpConnection::new(stream))
}

/// Bind one TCP listener using the deterministic Deno compatibility surface.
pub fn listen(hostname: impl AsRef<str>, port: u16) -> Result<DenoTcpListener, std::io::Error> {
    let listener = TcpListener::bind((hostname.as_ref(), port))?;
    Ok(DenoTcpListener::new(listener))
}

fn read_http_request(
    stream: &TcpStream,
    local_addr: SocketAddr,
) -> Result<Request, std::io::Error> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let request_line = request_line.trim_end_matches(['\r', '\n']);
    if request_line.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "missing HTTP request line",
        ));
    }

    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing HTTP method")
    })?;
    let path = request_parts.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing HTTP path")
    })?;

    let headers = Headers::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim();
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.parse::<usize>().unwrap_or(0);
            }
            headers.append(name.trim(), value);
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let url = if path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else {
        format!("http://{}{}", local_addr, path)
    };

    Request::with_parts(url, method, headers, body)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error.to_string()))
}

fn write_http_response(stream: &mut TcpStream, response: Response) -> Result<(), std::io::Error> {
    let headers = response.headers();
    let mut entries = headers.entries();
    if !entries
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("content-length"))
    {
        entries.push((
            String::from("content-length"),
            response.body().len().to_string(),
        ));
    }

    write!(
        stream,
        "HTTP/1.1 {} {}\r\n",
        response.status(),
        response.status_text()
    )?;
    for (name, value) in entries {
        write!(stream, "{}: {}\r\n", name, value)?;
    }
    write!(stream, "\r\n")?;
    stream.write_all(response.body())?;
    stream.flush()?;
    Ok(())
}

/// Serve a single HTTP request on a deterministic Deno compatibility socket.
pub fn serve<F>(
    handler: F,
    hostname: impl AsRef<str>,
    port: u16,
) -> Result<DenoHttpServer, std::io::Error>
where
    F: Fn(Request) -> Response + Send + Sync + 'static,
{
    let listener = TcpListener::bind((hostname.as_ref(), port))?;
    let local_addr = listener.local_addr()?;
    let handler = Arc::new(handler);
    let join_handle = thread::spawn(move || -> Result<(), std::io::Error> {
        let (mut stream, _) = listener.accept()?;
        let request = read_http_request(&stream, local_addr)?;
        let response = handler(request);
        write_http_response(&mut stream, response)
    });

    Ok(DenoHttpServer {
        local_addr,
        join_handle: Some(join_handle),
    })
}

/// Bundled Deno-oriented execution context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenoRuntimeProjection {
    args: DenoArgs,
    env: DenoEnv,
    fs: DenoFs,
    permissions: DenoPermissions,
    process_id: u32,
    exit_code: Option<i32>,
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
            process_id: std::process::id(),
            exit_code: None,
        }
    }

    pub fn args(&self) -> &DenoArgs {
        &self.args
    }

    pub fn env(&self) -> &DenoEnv {
        &self.env
    }

    /// Check whether a captured environment variable is present.
    pub fn env_has(&self, key: &str) -> bool {
        self.env.has(key)
    }

    /// Mutable access to the captured environment view.
    pub fn env_mut(&mut self) -> &mut DenoEnv {
        &mut self.env
    }

    /// Return a deterministic snapshot of the captured environment view.
    pub fn env_snapshot(&self) -> BTreeMap<String, String> {
        self.env.to_object()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn env_to_object(&self) -> BTreeMap<String, String> {
        self.env_snapshot()
    }

    /// Alias for the deterministic environment snapshot helper with an explicit object-value name.
    pub fn env_snapshot_object_value(&self) -> BTreeMap<String, String> {
        self.env_snapshot()
    }

    /// Alias for the deterministic environment snapshot helper with a generic object-value name.
    pub fn snapshot_object_value(&self) -> BTreeMap<String, String> {
        self.env_snapshot()
    }

    /// Return the captured environment as a JSON object value.
    pub fn env_snapshot_value(&self) -> Value {
        self.env.to_json_value()
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

    pub fn fs(&self) -> &DenoFs {
        &self.fs
    }

    /// Host process identifier captured for the compatibility view.
    pub fn pid(&self) -> u32 {
        self.process_id
    }

    /// Record a termination code for the compatibility view.
    pub fn exit(&mut self, exit_code: i32) {
        self.exit_code = Some(exit_code);
    }

    /// Return the recorded termination code, if any.
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Update the working-directory view used by relative path resolution.
    ///
    /// This stays a Rust-side compatibility helper; the language-visible
    /// `Deno.chdir` member remains phase-gated by the type checker.
    pub fn chdir(&mut self, cwd: impl Into<PathBuf>) {
        self.fs.chdir(cwd);
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
