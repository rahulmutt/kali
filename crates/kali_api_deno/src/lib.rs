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

mod command;
pub use command::*;

mod env;
pub use env::*;

mod fs;
pub use fs::*;

mod net;
pub use net::*;

mod path;
use crate::path::{normalize_path, resolve_path};

mod permissions;
pub use permissions::*;

/// Initialize the Deno API compatibility surface.
pub fn deno_api_init() {
    kali_api_web::web_api_init();
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
