//! Host state and scheduled timer types for the Kali runtime.

use crate::*;

/// Host-side state owned by the runtime.
#[derive(Clone, Debug)]
pub struct KaliHostState {
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
    /// Requested runtime profiles for the current execution context.
    pub runtime_profiles: Vec<String>,
    /// High-level host contract selected for the current execution context.
    pub host_contract: RuntimeHostContract,
    /// Canonical runtime backend selected for the current execution context.
    pub runtime_backend: RuntimeBackend,
    /// Thread budget derived from the active policy and any invocation override.
    pub max_threads: Option<u64>,
    /// Spawn budget derived from the active policy and any invocation override.
    pub max_spawned_processes: Option<u64>,
    /// Captured guest stdout.
    pub stdout: String,
    /// Captured guest stderr.
    pub stderr: String,
    /// Pending one-shot and repeating timers.
    pub pending_timers: BTreeMap<u32, ScheduledTimer>,
    /// Pending microtask callbacks.
    pub pending_microtasks: VecDeque<i32>,
    /// Timer ids that were cleared while a callback was firing.
    pub cancelled_timers: HashSet<u32>,
    /// Deterministic worker/thread topology used by the threaded runtime plumbing.
    pub thread_topology: ThreadRuntimeTopology,
    /// Monotonic timer id counter.
    pub next_timer_id: u32,
    /// Registered test callbacks collected from guest-side `Kali.test(...)` calls.
    pub registered_tests: Vec<i32>,
    /// Coverage hit ordinals recorded by instrumented guest modules.
    pub coverage_hits: BTreeSet<u32>,
    /// Registered Node-style event callbacks collected from guest-side `EventEmitter` calls.
    pub event_listeners: BTreeMap<String, Vec<i32>>,
    /// Memory/table limits for the current store.
    pub store_limits: wasmtime::StoreLimits,
    /// The most recent policy/resource diagnostic produced by a host operation.
    pub pending_diagnostic: Option<Diagnostic>,
    /// Active host file handles counted for policy enforcement.
    pub active_file_handles: usize,
    /// Active host network operations counted for policy enforcement.
    pub active_network_connections: usize,
    /// Active spawned processes counted for resource enforcement.
    pub active_spawned_processes: usize,
    /// Active worker/thread instances counted for later threaded-profile enforcement.
    pub active_threads: usize,
    /// Pending process exit code requested by the guest, if any.
    pub pending_exit_code: Option<i32>,
}

/// A scheduled timer callback.
#[derive(Clone, Debug)]
pub struct ScheduledTimer {
    /// Guest callback id.
    pub callback_id: i32,
    /// When the timer should fire.
    pub due_at: Instant,
    /// Repeat interval for setInterval-like timers.
    pub repeat_interval: Option<Duration>,
}

impl Default for KaliHostState {
    fn default() -> Self {
        Self {
            policy: None,
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            process_id: std::process::id(),
            runtime_profiles: Vec::new(),
            host_contract: RuntimeHostContract::KaliHosted,
            runtime_backend: RuntimeBackend::Wasmtime,
            max_threads: None,
            max_spawned_processes: None,
            stdout: String::new(),
            stderr: String::new(),
            pending_timers: BTreeMap::new(),
            pending_microtasks: VecDeque::new(),
            cancelled_timers: HashSet::new(),
            thread_topology: ThreadRuntimeTopology::default(),
            next_timer_id: 0,
            registered_tests: Vec::new(),
            coverage_hits: BTreeSet::new(),
            event_listeners: BTreeMap::new(),
            store_limits: wasmtime::StoreLimitsBuilder::new().build(),
            pending_diagnostic: None,
            active_file_handles: 0,
            active_network_connections: 0,
            active_spawned_processes: 0,
            active_threads: 0,
            pending_exit_code: None,
        }
    }
}

impl KaliHostState {
    /// Return the canonical runtime backend preserved in the runtime store state.
    pub fn runtime_backend(&self) -> RuntimeBackend {
        self.runtime_backend
    }

    /// Return the host process identifier preserved in the runtime store state.
    pub fn process_id(&self) -> u32 {
        self.process_id
    }

    /// Return whether the preserved environment contains a key.
    pub fn env_has(&self, key: &str) -> bool {
        self.env.contains_key(key)
    }

    /// Alias for the environment-presence helper.
    pub fn has(&self, key: &str) -> bool {
        self.env_has(key)
    }

    /// Return the deterministic environment snapshot preserved in the runtime store state.
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

    /// Return the deterministic environment snapshot as a JSON object value.
    pub fn env_snapshot_value(&self) -> serde_json::Value {
        env_snapshot_value(&self.env)
    }

    /// Alias for the deterministic JSON-ready environment snapshot helper.
    pub fn env_snapshot_json_value(&self) -> serde_json::Value {
        self.env_snapshot_value()
    }

    /// Alias for the deterministic threaded-topology snapshot helper with a generic value name.
    pub fn snapshot_value(&self) -> serde_json::Value {
        self.thread_topology_snapshot_value()
    }

    /// Alias for the JSON-ready environment snapshot helper.
    pub fn env_to_json_value(&self) -> serde_json::Value {
        self.env_snapshot_value()
    }

    /// Alias for the JSON-ready threaded-topology snapshot helper.
    pub fn thread_topology_snapshot_json_value(&self) -> serde_json::Value {
        self.thread_topology_snapshot_value()
    }

    /// Alias for the JSON-ready threaded-topology snapshot helper with an explicit object-value name.
    pub fn thread_topology_snapshot_object_value(&self) -> serde_json::Value {
        self.thread_topology_snapshot_value()
    }

    /// Alias for the JSON-ready threaded-topology snapshot helper with an explicit object-value name.
    pub fn snapshot_object_value(&self) -> serde_json::Value {
        self.thread_topology_snapshot_value()
    }

    /// Alias for the JSON-ready threaded-topology snapshot helper.
    pub fn snapshot_json_value(&self) -> serde_json::Value {
        self.thread_topology_snapshot_value()
    }

    /// Return a stable snapshot of the current threaded runtime topology.
    pub fn snapshot(&self) -> ThreadRuntimeShutdownReport {
        self.thread_topology_snapshot()
    }

    /// Return a stable snapshot of the current threaded runtime topology.
    pub fn thread_topology_snapshot(&self) -> ThreadRuntimeShutdownReport {
        self.thread_topology.snapshot()
    }

    /// Return a stable JSON-ready snapshot of the current threaded runtime topology.
    pub fn thread_topology_snapshot_value(&self) -> serde_json::Value {
        self.thread_topology.snapshot_value()
    }

    pub(crate) fn schedule_timer(
        &mut self,
        callback_id: i32,
        delay_ms: i32,
        repeat: bool,
    ) -> wasmtime::Result<i32> {
        if delay_ms < 0 {
            return Err(wasmtime::Error::msg("timer delay must be non-negative"));
        }

        let active_timers = self.pending_timers.len();
        if let Some(policy) = self.policy.as_ref() {
            policy
                .check_operation(HostOperation::TimerSchedule {
                    delay_ms: delay_ms as u64,
                    active_timers,
                })
                .map_err(|diagnostic| {
                    self.pending_diagnostic = Some(diagnostic.clone());
                    wasmtime::Error::msg(format!("KALI_E4003: {}", diagnostic.message))
                })?;
        }

        let timer_id = self.next_timer_id;
        self.next_timer_id = self
            .next_timer_id
            .checked_add(1)
            .ok_or_else(|| wasmtime::Error::msg("timer id overflow"))?;

        let delay = Duration::from_millis(delay_ms as u64);
        self.pending_timers.insert(
            timer_id,
            ScheduledTimer {
                callback_id,
                due_at: Instant::now() + delay,
                repeat_interval: repeat.then_some(delay),
            },
        );

        Ok(timer_id as i32)
    }

    pub(crate) fn cancel_timer(&mut self, timer_id: i32) -> wasmtime::Result<()> {
        let timer_id = u32::try_from(timer_id)
            .map_err(|_| wasmtime::Error::msg("timer id must be non-negative"))?;
        if self.pending_timers.remove(&timer_id).is_none() {
            self.cancelled_timers.insert(timer_id);
        }
        Ok(())
    }

    pub(crate) fn queue_microtask(&mut self, callback_id: i32) {
        self.pending_microtasks.push_back(callback_id);
    }

    pub(crate) fn register_event_listener(
        &mut self,
        event_type: impl Into<String>,
        callback_id: i32,
    ) {
        self.event_listeners
            .entry(event_type.into())
            .or_default()
            .push(callback_id);
    }

    pub(crate) fn event_listener_callbacks(&self, event_type: &str) -> Vec<i32> {
        self.event_listeners
            .get(event_type)
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn event_listener_count(&self, event_type: &str) -> usize {
        self.event_listeners
            .get(event_type)
            .map(|callbacks| callbacks.len())
            .unwrap_or(0)
    }

    pub(crate) fn begin_spawn(&mut self) -> wasmtime::Result<()> {
        if let Some(limit) = self.max_spawned_processes {
            if self.active_spawned_processes >= limit as usize {
                let diagnostic = Diagnostic::error(
                    e4::RESOURCE_LIMIT_EXCEEDED as u32,
                    format!(
                        "active child process count {} exceeds policy limit of {}",
                        self.active_spawned_processes.saturating_add(1),
                        limit
                    ),
                );
                self.pending_diagnostic = Some(diagnostic);
                return Err(wasmtime::Error::msg(format!(
                    "KALI_E4003: active child process count {} exceeds policy limit of {}",
                    self.active_spawned_processes.saturating_add(1),
                    limit
                )));
            }
        }
        self.active_spawned_processes = self.active_spawned_processes.saturating_add(1);
        Ok(())
    }

    pub(crate) fn finish_spawn(&mut self) {
        self.active_spawned_processes = self.active_spawned_processes.saturating_sub(1);
    }

    /// Register one deterministic guest-requested thread instance.
    pub fn spawn_thread_instance(
        &mut self,
        script_url: impl AsRef<str>,
    ) -> wasmtime::Result<usize> {
        let script_url = script_url.as_ref().trim();
        if script_url.is_empty() {
            return Err(wasmtime::Error::msg(
                "thread script URL must be a non-empty absolute URL",
            ));
        }

        let active_threads = self.active_threads;
        enforce_operation(self, HostOperation::ThreadSpawn { active_threads })?;
        self.begin_thread()?;
        match self.thread_topology.spawn_worker(script_url) {
            Ok(instance_id) => Ok(instance_id),
            Err(error) => {
                self.finish_thread();
                Err(wasmtime::Error::msg(error.to_string()))
            }
        }
    }

    /// Release one deterministic guest-requested thread instance.
    pub fn release_thread_instance(&mut self, instance_id: usize) -> bool {
        let was_live = self.thread_topology.is_live(instance_id);
        if was_live && self.thread_topology.terminate(instance_id) {
            self.finish_thread();
        }
        was_live
    }

    pub(crate) fn has_threaded_runtime_profile(&self) -> bool {
        self.runtime_profiles
            .iter()
            .any(|profile| profile.trim() == "wasm-threads")
    }

    #[allow(dead_code)]
    pub(crate) fn begin_thread(&mut self) -> wasmtime::Result<()> {
        if !self.has_threaded_runtime_profile() {
            let diagnostic = Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "threaded runtime profile is unavailable without an explicit `--wasm-threads` opt-in",
            );
            self.pending_diagnostic = Some(diagnostic);
            return Err(wasmtime::Error::msg(
                "KALI_E5506: threaded runtime profile is unavailable without an explicit `--wasm-threads` opt-in",
            ));
        }

        if let Some(limit) = self.max_threads {
            if self.active_threads >= limit as usize {
                let diagnostic = Diagnostic::error(
                    e4::RESOURCE_LIMIT_EXCEEDED as u32,
                    format!(
                        "active thread count {} exceeds policy limit of {}",
                        self.active_threads.saturating_add(1),
                        limit
                    ),
                );
                self.pending_diagnostic = Some(diagnostic);
                return Err(wasmtime::Error::msg(format!(
                    "KALI_E4003: active thread count {} exceeds policy limit of {}",
                    self.active_threads.saturating_add(1),
                    limit
                )));
            }
        } else {
            let diagnostic = Diagnostic::error(
                e4::RESOURCE_LIMIT_EXCEEDED as u32,
                "threaded runtime profile is unavailable without an explicit thread budget",
            );
            self.pending_diagnostic = Some(diagnostic);
            return Err(wasmtime::Error::msg(
                "KALI_E4003: threaded runtime profile is unavailable without an explicit thread budget",
            ));
        }

        self.active_threads = self.active_threads.saturating_add(1);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn finish_thread(&mut self) {
        self.active_threads = self.active_threads.saturating_sub(1);
    }

    pub(crate) fn take_pending_exit_code(&mut self) -> Option<i32> {
        self.pending_exit_code.take()
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
