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
    io::{BufRead, BufReader, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
    thread::{self, JoinHandle},
};

use serde_json::Value;

mod args;
pub use args::*;

mod env;
pub use env::*;

mod fs;
pub use fs::*;

mod path;
use crate::path::{normalize_path, resolve_path};

mod permissions;
pub use permissions::*;

/// Initialize the Deno API compatibility surface.
pub fn deno_api_init() {
    kali_api_web::web_api_init();
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

    /// Alias for the environment presence check helper.
    pub fn has(&self, key: &str) -> bool {
        self.env_has(key)
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
    pub fn snapshot(&self) -> BTreeMap<String, String> {
        self.env_snapshot()
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


#[cfg(test)]
#[path = "tests.rs"]
mod tests;
