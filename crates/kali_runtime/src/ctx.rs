//! Runtime context, builders, and accessors.

use crate::*;

/// Runtime context.
#[derive(Clone, Debug)]
pub struct RuntimeCtx {
    /// Sandbox policy.
    pub policy: Option<SandboxPolicy>,
    /// Host arguments exposed to the guest.
    pub args: Vec<String>,
    /// Environment view exposed to the guest.
    pub env: BTreeMap<String, String>,
    /// Current working directory used for relative host-path resolution.
    pub cwd: PathBuf,
    /// Host process identifier used for late process-control compatibility plumbing.
    pub process_id: u32,
    /// Selected API surface for the current execution context.
    pub api_surface: String,
    /// Requested runtime profiles for the current execution context.
    pub runtime_profiles: Vec<String>,
    /// Invocation-level thread budget override preserved for later threaded-profile enforcement.
    pub max_threads: Option<u64>,
    /// Invocation-level spawned-process budget preserved for subprocess resource enforcement.
    pub max_spawned_processes: Option<u64>,
}

impl Default for RuntimeCtx {
    fn default() -> Self {
        Self {
            policy: None,
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            process_id: std::process::id(),
            api_surface: "deno".to_string(),
            runtime_profiles: Vec::new(),
            max_threads: None,
            max_spawned_processes: None,
        }
    }
}

impl RuntimeCtx {
    pub fn new(policy: Option<SandboxPolicy>) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_host_context(policy, Vec::new(), capture_env(), cwd)
    }

    pub fn with_api_surface(policy: Option<SandboxPolicy>, api_surface: impl Into<String>) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_host_context_with_api_surface(
            policy,
            Vec::new(),
            capture_env(),
            cwd,
            api_surface,
        )
    }

    pub fn with_host_context(
        policy: Option<SandboxPolicy>,
        args: Vec<String>,
        env: BTreeMap<String, String>,
        cwd: PathBuf,
    ) -> Self {
        Self::with_host_context_with_api_surface(policy, args, env, cwd, "deno")
    }

    pub fn with_host_context_with_api_surface(
        policy: Option<SandboxPolicy>,
        args: Vec<String>,
        env: BTreeMap<String, String>,
        cwd: PathBuf,
        api_surface: impl Into<String>,
    ) -> Self {
        Self {
            policy,
            args,
            env,
            cwd,
            process_id: std::process::id(),
            api_surface: api_surface.into(),
            runtime_profiles: Vec::new(),
            max_threads: None,
            max_spawned_processes: None,
        }
    }

    /// Attach the requested runtime profiles to the current execution context.
    pub fn with_runtime_profiles(mut self, runtime_profiles: Vec<String>) -> Self {
        self.runtime_profiles = normalize_runtime_profiles(runtime_profiles);
        self
    }

    /// Attach an invocation-level thread budget override to the current execution context.
    pub fn with_max_threads(mut self, max_threads: Option<u64>) -> Self {
        self.max_threads = max_threads;
        self
    }

    /// Return the effective thread budget after combining policy and invocation overrides.
    ///
    /// This keeps the runtime-side budget resolution aligned with the sandbox policy helper so
    /// direct API callers can reason about the same canonical limit that the host state will use.
    pub fn effective_thread_budget(&self) -> Option<u64> {
        self.policy
            .as_ref()
            .map(|policy| policy.effective_thread_budget(self.max_threads))
            .unwrap_or(self.max_threads)
    }

    /// Return whether the captured environment contains a key.
    pub fn env_has(&self, key: &str) -> bool {
        self.env.contains_key(key)
    }

    /// Alias for the environment-presence helper.
    pub fn has(&self, key: &str) -> bool {
        self.env_has(key)
    }

    /// Return the deterministic environment snapshot captured for this execution context.
    pub fn env_snapshot(&self) -> BTreeMap<String, String> {
        self.env.clone()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn env_to_object(&self) -> BTreeMap<String, String> {
        self.env_snapshot()
    }

    /// Alias for the deterministic environment snapshot helper.
    pub fn snapshot(&self) -> BTreeMap<String, String> {
        self.env_snapshot()
    }

    /// Alias for the deterministic environment snapshot helper with an explicit object-value name.
    pub fn env_snapshot_object_value(&self) -> BTreeMap<String, String> {
        self.env_snapshot()
    }

    /// Return the deterministic environment snapshot as a JSON object value.
    pub fn env_snapshot_value(&self) -> serde_json::Value {
        env_snapshot_value(&self.env)
    }

    /// Alias for the deterministic JSON-ready environment snapshot helper.
    pub fn env_snapshot_json_value(&self) -> serde_json::Value {
        self.env_snapshot_value()
    }

    /// Alias for the deterministic environment snapshot helper with a generic value name.
    pub fn snapshot_value(&self) -> serde_json::Value {
        self.env_snapshot_value()
    }

    /// Alias for the JSON-ready environment snapshot helper with an explicit object-value name.
    pub fn snapshot_object_value(&self) -> serde_json::Value {
        self.env_snapshot_value()
    }

    /// Alias for the JSON-ready environment snapshot helper.
    pub fn snapshot_json_value(&self) -> serde_json::Value {
        self.env_snapshot_value()
    }

    /// Alias for the JSON-ready environment snapshot helper.
    pub fn env_to_json_value(&self) -> serde_json::Value {
        self.env_snapshot_value()
    }

    /// Return the canonical runtime-profile vector for the current execution context.
    ///
    /// This normalizes the public `runtime_profiles` field so direct API callers
    /// that mutate the field after construction still see the same deduplicated,
    /// trimmed, stable ordering that execution and store construction use.
    pub fn canonical_runtime_profiles(&self) -> Vec<String> {
        normalize_runtime_profiles(self.runtime_profiles.clone())
    }

    /// Attach an invocation-level spawned-process budget to the current execution context.
    pub fn with_max_spawned_processes(mut self, max_spawned_processes: Option<u64>) -> Self {
        self.max_spawned_processes = max_spawned_processes;
        self
    }

    /// Return the current high-level runtime host contract.
    pub fn host_contract(&self) -> RuntimeHostContract {
        if self.api_surface == "browser" {
            RuntimeHostContract::BrowserRequested
        } else {
            RuntimeHostContract::KaliHosted
        }
    }

    /// Return the canonical runtime backend for the current execution context.
    pub fn runtime_backend(&self) -> RuntimeBackend {
        if matches!(self.host_contract(), RuntimeHostContract::BrowserRequested)
            && self.browser_harness_command().is_some()
        {
            RuntimeBackend::BrowserHarness
        } else {
            RuntimeBackend::Wasmtime
        }
    }

    /// Return the host process identifier preserved in the execution context.
    pub fn process_id(&self) -> u32 {
        self.process_id
    }
}

pub(crate) fn capture_env() -> BTreeMap<String, String> {
    std::env::vars().collect()
}

pub(crate) fn env_snapshot_value(env: &BTreeMap<String, String>) -> serde_json::Value {
    serde_json::Value::Object(
        env.iter()
            .map(|(key, value)| (key.clone(), serde_json::Value::String(value.clone())))
            .collect(),
    )
}

#[cfg(test)]
#[path = "ctx_tests.rs"]
mod ctx_tests;
