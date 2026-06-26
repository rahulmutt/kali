//! Node.js `process` compatibility surface.

use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{normalize_path, resolve_path};

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

#[cfg(test)]
#[path = "process_tests.rs"]
mod process_tests;
