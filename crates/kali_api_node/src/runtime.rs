//! Node.js runtime projection bundling all host-side API surfaces.

use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::{
    EventEmitter, NodeAssert, NodeChildProcess, NodeCrypto, NodeFs, NodeFsPromises, NodeHttp,
    NodeOs, NodePath, NodeProcess, NodeStream, NodeUrl, NodeUtil,
};

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

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;
