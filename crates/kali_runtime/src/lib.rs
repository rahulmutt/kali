//! Runtime execution for Kali-generated WASM modules.

use kali_api_node::{
    NodeAssert, NodeBuffer, NodeChildProcess, NodeCrypto, NodePath, NodeRuntimeProjection, NodeUrl,
    NodeUtil,
};
use kali_api_web::{fill_random_values, performance_now, random_uuid};
use kali_error::{_error_codes::e4, Diagnostic};
use kali_sandbox::{HostOperation, SandboxPolicy};
use reqwest::blocking;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
use wasmtime::{
    Caller, Config, Engine, Extern, Instance, Linker, Memory, Module, Store, StoreLimitsBuilder,
};

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
    /// Selected API surface for the current execution context.
    pub api_surface: String,
    /// Requested runtime profiles for the current execution context.
    pub runtime_profiles: Vec<String>,
    /// Invocation-level thread budget override preserved for later threaded-profile enforcement.
    pub max_threads: Option<u64>,
    /// Invocation-level spawned-process budget preserved for subprocess resource enforcement.
    pub max_spawned_processes: Option<u64>,
}

/// Host-side state owned by the runtime.
#[derive(Clone, Debug, Default)]
pub struct KaliHostState {
    /// Sandbox policy.
    pub policy: Option<SandboxPolicy>,
    /// Host arguments exposed to the guest.
    pub args: Vec<String>,
    /// Environment view exposed to the guest.
    pub env: BTreeMap<String, String>,
    /// Current working directory used for relative host-path resolution.
    pub cwd: PathBuf,
    /// Requested runtime profiles for the current execution context.
    pub runtime_profiles: Vec<String>,
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

/// Result of executing a WASM module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOutcome {
    /// Process exit code.
    pub exit_code: i32,
    /// Number of tests executed during `kali test`.
    pub tests_run: usize,
    /// Number of failing tests during `kali test`.
    pub tests_failed: usize,
    /// Captured guest stdout.
    pub stdout: String,
    /// Captured guest stderr.
    pub stderr: String,
    /// Coverage hit ordinals recorded during the execution.
    pub coverage_hits: Vec<u32>,
}

impl Default for RuntimeCtx {
    fn default() -> Self {
        Self {
            policy: None,
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
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

    /// Attach an invocation-level spawned-process budget to the current execution context.
    pub fn with_max_spawned_processes(mut self, max_spawned_processes: Option<u64>) -> Self {
        self.max_spawned_processes = max_spawned_processes;
        self
    }

    /// Execute a WASM module.
    pub fn execute(&self, wasm_bytes: &[u8]) -> Result<RuntimeOutcome, Vec<Diagnostic>> {
        self.execute_inner(wasm_bytes, false)
    }

    /// Execute a WASM module as a test suite, running guest-registered test callbacks.
    pub fn execute_tests(&self, wasm_bytes: &[u8]) -> Result<RuntimeOutcome, Vec<Diagnostic>> {
        self.execute_inner(wasm_bytes, true)
    }

    fn execute_inner(
        &self,
        wasm_bytes: &[u8],
        run_registered_tests: bool,
    ) -> Result<RuntimeOutcome, Vec<Diagnostic>> {
        let mut config = Config::new();
        config.consume_fuel(true);
        let engine = Engine::new(&config).map_err(|error| {
            vec![Diagnostic::error(
                e4::IO_ERROR as u32,
                format!("failed to initialize WASM engine: {}", error),
            )]
        })?;
        let module = Module::from_binary(&engine, wasm_bytes).map_err(|error| {
            vec![Diagnostic::error(
                e4::IO_ERROR as u32,
                format!("failed to load WASM module: {}", error),
            )]
        })?;

        let store_limits = self
            .policy
            .as_ref()
            .and_then(|policy| policy.resources.max_memory_mb)
            .map(|max_memory_mb| {
                StoreLimitsBuilder::new()
                    .memory_size((max_memory_mb as usize) * 1024 * 1024)
                    .build()
            })
            .unwrap_or_else(|| StoreLimitsBuilder::new().build());

        let mut store = Store::new(
            &engine,
            KaliHostState {
                policy: self.policy.clone(),
                args: self.args.clone(),
                env: self.env.clone(),
                cwd: self.cwd.clone(),
                runtime_profiles: self.runtime_profiles.clone(),
                max_threads: self
                    .policy
                    .as_ref()
                    .map(|policy| policy.effective_thread_budget(self.max_threads))
                    .unwrap_or(self.max_threads),
                max_spawned_processes: self
                    .policy
                    .as_ref()
                    .map(|policy| policy.effective_spawn_budget(self.max_spawned_processes))
                    .unwrap_or(self.max_spawned_processes),
                stdout: String::new(),
                stderr: String::new(),
                pending_timers: BTreeMap::new(),
                pending_microtasks: VecDeque::new(),
                cancelled_timers: HashSet::new(),
                next_timer_id: 0,
                registered_tests: Vec::new(),
                coverage_hits: BTreeSet::new(),
                event_listeners: BTreeMap::new(),
                store_limits,
                pending_diagnostic: None,
                active_file_handles: 0,
                active_network_connections: 0,
                active_spawned_processes: 0,
                active_threads: 0,
            },
        );
        store.limiter(|state| &mut state.store_limits);
        let default_fuel = self
            .policy
            .as_ref()
            .and_then(|policy| policy.resources.max_cpu_time_ms)
            .unwrap_or(10_000);
        store
            .set_fuel(default_fuel.saturating_mul(1_000))
            .map_err(|error| {
                vec![Diagnostic::error(
                    e4::RESOURCE_LIMIT_EXCEEDED as u32,
                    format!("failed to configure CPU fuel budget: {}", error),
                )]
            })?;
        let mut linker = Linker::new(&engine);
        register_default_host_imports(&mut linker).map_err(|diagnostic| vec![diagnostic])?;
        if self.api_surface == "node" {
            let node_projection = NodeRuntimeProjection::from_host_context(
                self.args.clone(),
                self.env.clone(),
                self.cwd.clone(),
            );
            register_node_host_imports(&mut linker, node_projection)
                .map_err(|diagnostic| vec![diagnostic])?;
        }

        let instance = linker.instantiate(&mut store, &module).map_err(|error| {
            vec![runtime_error_diagnostic(format!(
                "failed to instantiate WASM module: {}",
                error
            ))]
        })?;

        let start = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .map_err(|error| {
                vec![Diagnostic::error(
                    e4::UNCAUGHT_ERROR as u32,
                    format!("missing _start export: {}", error),
                )]
            })?;

        if let Err(error) = start.call(&mut store, ()) {
            if let Some(diagnostic) = store.data_mut().pending_diagnostic.take() {
                return Err(vec![diagnostic]);
            }
            return Err(vec![runtime_error_diagnostic(format!(
                "runtime trap: {}",
                error
            ))]);
        }

        drain_event_loop(&instance, &mut store).map_err(|diagnostic| vec![diagnostic])?;

        if !run_registered_tests {
            let state = store.data();
            return Ok(RuntimeOutcome {
                exit_code: 0,
                tests_run: 0,
                tests_failed: 0,
                stdout: state.stdout.clone(),
                stderr: state.stderr.clone(),
                coverage_hits: state.coverage_hits.iter().copied().collect(),
            });
        }

        let registered_tests = {
            let state = store.data_mut();
            std::mem::take(&mut state.registered_tests)
        };

        if registered_tests.is_empty() {
            let state = store.data();
            return Ok(RuntimeOutcome {
                exit_code: 0,
                tests_run: 1,
                tests_failed: 0,
                stdout: state.stdout.clone(),
                stderr: state.stderr.clone(),
                coverage_hits: state.coverage_hits.iter().copied().collect(),
            });
        }

        let mut tests_run = 0usize;
        let mut tests_failed = 0usize;
        for callback_id in registered_tests {
            tests_run += 1;
            match invoke_callback(&instance, &mut store, callback_id) {
                Ok(()) => {}
                Err(diagnostic) => {
                    let rendered = diagnostic.to_string();
                    store.data_mut().stderr.push_str(&rendered);
                    store.data_mut().stderr.push('\n');
                    tests_failed += 1;
                }
            }

            drain_event_loop(&instance, &mut store).map_err(|diagnostic| vec![diagnostic])?;
        }

        let state = store.data();
        Ok(RuntimeOutcome {
            exit_code: if tests_failed == 0 { 0 } else { 1 },
            tests_run,
            tests_failed,
            stdout: state.stdout.clone(),
            stderr: state.stderr.clone(),
            coverage_hits: state.coverage_hits.iter().copied().collect(),
        })
    }
}

fn register_default_host_imports(linker: &mut Linker<KaliHostState>) -> Result<(), Diagnostic> {
    linker
        .func_wrap(
            "kali:rt",
            "console_log",
            |mut caller: Caller<'_, KaliHostState>, val: i64| -> wasmtime::Result<()> {
                enforce_operation(caller.data_mut(), HostOperation::Console)?;
                let rendered = format_console_value(&mut caller, val);
                append_stdout(caller.data_mut(), rendered);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("console_log", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "console_error",
            |mut caller: Caller<'_, KaliHostState>, val: i64| -> wasmtime::Result<()> {
                enforce_operation(caller.data_mut(), HostOperation::Console)?;
                let rendered = format_console_value(&mut caller, val);
                append_stderr(caller.data_mut(), rendered);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("console_error", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "console_warn",
            |mut caller: Caller<'_, KaliHostState>, val: i64| -> wasmtime::Result<()> {
                enforce_operation(caller.data_mut(), HostOperation::Console)?;
                let rendered = format_console_value(&mut caller, val);
                append_stderr(caller.data_mut(), format!("[warn] {}", rendered));
                Ok(())
            },
        )
        .map_err(|error| host_import_error("console_warn", error))?;

    linker
        .func_wrap("kali:rt", "performance_now", || -> f64 {
            performance_now()
        })
        .map_err(|error| host_import_error("performance_now", error))?;

    linker
        .func_wrap("kali:rt", "performanceNow", || -> f64 { performance_now() })
        .map_err(|error| host_import_error("performanceNow", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "crypto_get_random_values",
            |mut caller: Caller<'_, KaliHostState>,
             out_ptr: i32,
             out_len: i32|
             -> wasmtime::Result<i32> {
                enforce_operation(caller.data_mut(), HostOperation::Random)?;
                let memory = guest_memory(&mut caller)?;
                let start = checked_offset(out_ptr)?;
                let len = checked_offset(out_len)?;
                start
                    .checked_add(len)
                    .ok_or_else(|| wasmtime::Error::msg("guest memory access overflow"))?;
                let mut bytes = vec![0u8; len];
                fill_random_values(&mut bytes).map_err(|error| {
                    wasmtime::Error::msg(format!("failed to generate random bytes: {}", error))
                })?;
                memory.write(&mut caller, start, &bytes).map_err(|error| {
                    wasmtime::Error::msg(format!("failed to write guest memory: {}", error))
                })?;
                Ok(out_len)
            },
        )
        .map_err(|error| host_import_error("crypto_get_random_values", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "cryptoGetRandomValues",
            |mut caller: Caller<'_, KaliHostState>,
             out_ptr: i32,
             out_len: i32|
             -> wasmtime::Result<i32> {
                let memory = guest_memory(&mut caller)?;
                let start = checked_offset(out_ptr)?;
                let len = checked_offset(out_len)?;
                start
                    .checked_add(len)
                    .ok_or_else(|| wasmtime::Error::msg("guest memory access overflow"))?;
                let mut bytes = vec![0u8; len];
                fill_random_values(&mut bytes).map_err(|error| {
                    wasmtime::Error::msg(format!("failed to generate random bytes: {}", error))
                })?;
                memory.write(&mut caller, start, &bytes).map_err(|error| {
                    wasmtime::Error::msg(format!("failed to write guest memory: {}", error))
                })?;
                Ok(out_len)
            },
        )
        .map_err(|error| host_import_error("cryptoGetRandomValues", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "crypto_random_uuid",
            |mut caller: Caller<'_, KaliHostState>,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                enforce_operation(caller.data_mut(), HostOperation::Random)?;
                let uuid = random_uuid().map_err(|error| {
                    wasmtime::Error::msg(format!("failed to generate random UUID: {}", error))
                })?;
                write_guest_string(&mut caller, out_ptr, out_cap, uuid)
            },
        )
        .map_err(|error| host_import_error("crypto_random_uuid", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "cryptoRandomUUID",
            |mut caller: Caller<'_, KaliHostState>,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                enforce_operation(caller.data_mut(), HostOperation::Random)?;
                let uuid = random_uuid().map_err(|error| {
                    wasmtime::Error::msg(format!("failed to generate random UUID: {}", error))
                })?;
                write_guest_string(&mut caller, out_ptr, out_cap, uuid)
            },
        )
        .map_err(|error| host_import_error("cryptoRandomUUID", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "test_register",
            |mut caller: Caller<'_, KaliHostState>, callback_id: i32| -> wasmtime::Result<()> {
                caller.data_mut().registered_tests.push(callback_id);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("test_register", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "coverage_hit",
            |mut caller: Caller<'_, KaliHostState>, coverage_id: i32| -> wasmtime::Result<()> {
                if coverage_id >= 0 {
                    caller.data_mut().coverage_hits.insert(coverage_id as u32);
                }
                Ok(())
            },
        )
        .map_err(|error| host_import_error("coverage_hit", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fs_read_text_file",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileRead {
                        path: host_path.clone(),
                    },
                )?;
                let bytes = fs::read(&host_path).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to read '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, &bytes)
            },
        )
        .map_err(|error| host_import_error("fs_read_text_file", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fs_read_file",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileRead {
                        path: host_path.clone(),
                    },
                )?;
                let bytes = fs::read(&host_path).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to read '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, &bytes)
            },
        )
        .map_err(|error| host_import_error("fs_read_file", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fs_write_text_file",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             data_ptr: i32,
             data_len: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let data = read_guest_bytes(&mut caller, data_ptr, data_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileWrite {
                        path: host_path.clone(),
                    },
                )?;
                if let Some(parent) = host_path.parent() {
                    fs::create_dir_all(parent).map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to create '{}': {}",
                            parent.display(),
                            error
                        ))
                    })?;
                }
                fs::write(&host_path, data).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to write '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("fs_write_text_file", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fs_mkdir",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileWrite {
                        path: host_path.clone(),
                    },
                )?;
                fs::create_dir_all(&host_path).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to create '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("fs_mkdir", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fs_remove",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             recursive: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileWrite {
                        path: host_path.clone(),
                    },
                )?;
                let metadata = fs::metadata(&host_path).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to inspect '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                if metadata.is_dir() {
                    if recursive != 0 {
                        fs::remove_dir_all(&host_path).map_err(|error| {
                            wasmtime::Error::msg(format!(
                                "failed to remove '{}': {}",
                                host_path.display(),
                                error
                            ))
                        })?;
                    } else {
                        fs::remove_dir(&host_path).map_err(|error| {
                            wasmtime::Error::msg(format!(
                                "failed to remove '{}': {}",
                                host_path.display(),
                                error
                            ))
                        })?;
                    }
                } else {
                    fs::remove_file(&host_path).map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to remove '{}': {}",
                            host_path.display(),
                            error
                        ))
                    })?;
                }
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("fs_remove", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "env_get",
            |mut caller: Caller<'_, KaliHostState>,
             key_ptr: i32,
             key_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let key = read_guest_string(&mut caller, key_ptr, key_len)?;
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::EnvironmentRead { key: key.clone() },
                )?;
                let Some(value) = caller.data().env.get(&key).cloned() else {
                    return Ok(-1);
                };
                write_guest_bytes(&mut caller, out_ptr, out_cap, value.as_bytes())
            },
        )
        .map_err(|error| host_import_error("env_get", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "args_len",
            |caller: Caller<'_, KaliHostState>| -> i32 { caller.data().args.len() as i32 },
        )
        .map_err(|error| host_import_error("args_len", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "args_get",
            |mut caller: Caller<'_, KaliHostState>,
             index: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let Some(value) = caller.data().args.get(index as usize).cloned() else {
                    return Ok(-1);
                };
                write_guest_bytes(&mut caller, out_ptr, out_cap, value.as_bytes())
            },
        )
        .map_err(|error| host_import_error("args_get", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fetch",
            |mut caller: Caller<'_, KaliHostState>,
             url_ptr: i32,
             url_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let url = read_guest_string(&mut caller, url_ptr, url_len)?;
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::NetworkFetch { url: url.clone() },
                )?;
                let response = blocking::get(&url)
                    .and_then(|resp| resp.error_for_status())
                    .map_err(|error| {
                        wasmtime::Error::msg(format!("failed to fetch '{}': {}", url, error))
                    })?;
                let bytes = response.bytes().map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to read response body from '{}': {}",
                        url, error
                    ))
                })?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, bytes.as_ref())
            },
        )
        .map_err(|error| host_import_error("fetch", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "timer_set",
            |mut caller: Caller<'_, KaliHostState>,
             callback_id: i32,
             delay_ms: i32,
             repeat: i32|
             -> wasmtime::Result<i32> {
                caller
                    .data_mut()
                    .schedule_timer(callback_id, delay_ms, repeat != 0)
            },
        )
        .map_err(|error| host_import_error("timer_set", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "setTimeout",
            |mut caller: Caller<'_, KaliHostState>,
             callback_id: i32,
             delay_ms: i32|
             -> wasmtime::Result<i32> {
                caller
                    .data_mut()
                    .schedule_timer(callback_id, delay_ms, false)
            },
        )
        .map_err(|error| host_import_error("setTimeout", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "setInterval",
            |mut caller: Caller<'_, KaliHostState>,
             callback_id: i32,
             delay_ms: i32|
             -> wasmtime::Result<i32> {
                caller
                    .data_mut()
                    .schedule_timer(callback_id, delay_ms, true)
            },
        )
        .map_err(|error| host_import_error("setInterval", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "timer_clear",
            |mut caller: Caller<'_, KaliHostState>, timer_id: i32| -> wasmtime::Result<()> {
                caller.data_mut().cancel_timer(timer_id)?;
                Ok(())
            },
        )
        .map_err(|error| host_import_error("timer_clear", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "clearTimeout",
            |mut caller: Caller<'_, KaliHostState>, timer_id: i32| -> wasmtime::Result<()> {
                caller.data_mut().cancel_timer(timer_id)?;
                Ok(())
            },
        )
        .map_err(|error| host_import_error("clearTimeout", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "clearInterval",
            |mut caller: Caller<'_, KaliHostState>, timer_id: i32| -> wasmtime::Result<()> {
                caller.data_mut().cancel_timer(timer_id)?;
                Ok(())
            },
        )
        .map_err(|error| host_import_error("clearInterval", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "queue_microtask",
            |mut caller: Caller<'_, KaliHostState>, callback_id: i32| -> wasmtime::Result<()> {
                caller.data_mut().queue_microtask(callback_id);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("queue_microtask", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "queueMicrotask",
            |mut caller: Caller<'_, KaliHostState>, callback_id: i32| -> wasmtime::Result<()> {
                caller.data_mut().queue_microtask(callback_id);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("queueMicrotask", error))?;

    Ok(())
}

fn register_node_host_imports(
    linker: &mut Linker<KaliHostState>,
    node_projection: NodeRuntimeProjection,
) -> Result<(), Diagnostic> {
    let fs_promises = node_projection.fs_promises().clone();
    let fs_promises_for_read_file = fs_promises.clone();
    let fs_promises_for_write_text = fs_promises.clone();
    let fs_promises_for_write_file = fs_promises.clone();
    let process = std::sync::Arc::new(std::sync::Mutex::new(node_projection.process().clone()));
    let process_for_argv_get = std::sync::Arc::clone(&process);
    let process_for_env_get = std::sync::Arc::clone(&process);
    let stream = node_projection.stream();
    let http = node_projection.http();
    let child_process: NodeChildProcess = node_projection.child_process();
    let os = node_projection.os();

    linker
        .func_wrap(
            "kali:node",
            "path_normalize",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let normalized = NodePath::normalize(Path::new(&path));
                write_guest_string(&mut caller, out_ptr, out_cap, normalized.to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("path_normalize", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_join",
            |mut caller: Caller<'_, KaliHostState>,
             base_ptr: i32,
             base_len: i32,
             segment_ptr: i32,
             segment_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let base = read_guest_string(&mut caller, base_ptr, base_len)?;
                let segment = read_guest_string(&mut caller, segment_ptr, segment_len)?;
                let joined = NodePath::join(Path::new(&base), Path::new(&segment));
                write_guest_string(&mut caller, out_ptr, out_cap, joined.to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("path_join", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_resolve",
            |mut caller: Caller<'_, KaliHostState>,
             base_ptr: i32,
             base_len: i32,
             input_ptr: i32,
             input_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let base = read_guest_string(&mut caller, base_ptr, base_len)?;
                let input = read_guest_string(&mut caller, input_ptr, input_len)?;
                let resolved = NodePath::resolve(Path::new(&base), Path::new(&input));
                write_guest_string(&mut caller, out_ptr, out_cap, resolved.to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("path_resolve", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_dirname",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let dirname = NodePath::dirname(Path::new(&path));
                write_guest_string(&mut caller, out_ptr, out_cap, dirname.to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("path_dirname", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_basename",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let basename = NodePath::basename(Path::new(&path));
                write_guest_string(&mut caller, out_ptr, out_cap, basename)
            },
        )
        .map_err(|error| host_import_error("path_basename", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_extname",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let extname = NodePath::extname(Path::new(&path));
                write_guest_string(&mut caller, out_ptr, out_cap, extname)
            },
        )
        .map_err(|error| host_import_error("path_extname", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_relative",
            |mut caller: Caller<'_, KaliHostState>,
             from_ptr: i32,
             from_len: i32,
             to_ptr: i32,
             to_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let from = read_guest_string(&mut caller, from_ptr, from_len)?;
                let to = read_guest_string(&mut caller, to_ptr, to_len)?;
                let relative = NodePath::relative(Path::new(&from), Path::new(&to));
                write_guest_string(&mut caller, out_ptr, out_cap, relative.to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("path_relative", error))?;

    linker
        .func_wrap(
            "kali:node",
            "url_parse",
            |mut caller: Caller<'_, KaliHostState>,
             input_ptr: i32,
             input_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let input = read_guest_string(&mut caller, input_ptr, input_len)?;
                let parsed = NodeUrl::parse(&input)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_string(&mut caller, out_ptr, out_cap, parsed.as_str())
            },
        )
        .map_err(|error| host_import_error("url_parse", error))?;

    linker
        .func_wrap(
            "kali:node",
            "url_resolve",
            |mut caller: Caller<'_, KaliHostState>,
             base_ptr: i32,
             base_len: i32,
             input_ptr: i32,
             input_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let base = read_guest_string(&mut caller, base_ptr, base_len)?;
                let input = read_guest_string(&mut caller, input_ptr, input_len)?;
                let resolved = NodeUrl::resolve(&base, &input)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_string(&mut caller, out_ptr, out_cap, resolved.as_str())
            },
        )
        .map_err(|error| host_import_error("url_resolve", error))?;

    linker
        .func_wrap(
            "kali:node",
            "crypto_create_hash",
            |mut caller: Caller<'_, KaliHostState>,
             algorithm_ptr: i32,
             algorithm_len: i32,
             data_ptr: i32,
             data_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let algorithm = read_guest_string(&mut caller, algorithm_ptr, algorithm_len)?;
                let data = read_guest_bytes(&mut caller, data_ptr, data_len)?;
                let digest = NodeCrypto::create_hash(&algorithm, &data)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_string(&mut caller, out_ptr, out_cap, digest)
            },
        )
        .map_err(|error| host_import_error("crypto_create_hash", error))?;

    linker
        .func_wrap(
            "kali:node",
            "crypto_create_hmac",
            |mut caller: Caller<'_, KaliHostState>,
             algorithm_ptr: i32,
             algorithm_len: i32,
             key_ptr: i32,
             key_len: i32,
             data_ptr: i32,
             data_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let algorithm = read_guest_string(&mut caller, algorithm_ptr, algorithm_len)?;
                let key = read_guest_bytes(&mut caller, key_ptr, key_len)?;
                let data = read_guest_bytes(&mut caller, data_ptr, data_len)?;
                let digest = NodeCrypto::create_hmac(&algorithm, &key, &data)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_string(&mut caller, out_ptr, out_cap, digest)
            },
        )
        .map_err(|error| host_import_error("crypto_create_hmac", error))?;

    linker
        .func_wrap(
            "kali:node",
            "crypto_random_uuid",
            |mut caller: Caller<'_, KaliHostState>,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let uuid = NodeCrypto::random_uuid_v4()
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_string(&mut caller, out_ptr, out_cap, uuid)
            },
        )
        .map_err(|error| host_import_error("crypto_random_uuid", error))?;

    linker
        .func_wrap(
            "kali:node",
            "crypto_random_bytes",
            |mut caller: Caller<'_, KaliHostState>,
             length: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let length = checked_offset(length)?;
                let bytes = NodeCrypto::random_bytes(length)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, &bytes)
            },
        )
        .map_err(|error| host_import_error("crypto_random_bytes", error))?;

    linker
        .func_wrap(
            "kali:node",
            "os_platform",
            move |mut caller: Caller<'_, KaliHostState>,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                write_guest_string(&mut caller, out_ptr, out_cap, os.platform())
            },
        )
        .map_err(|error| host_import_error("os_platform", error))?;

    linker
        .func_wrap(
            "kali:node",
            "os_arch",
            move |mut caller: Caller<'_, KaliHostState>,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                write_guest_string(&mut caller, out_ptr, out_cap, os.arch())
            },
        )
        .map_err(|error| host_import_error("os_arch", error))?;

    linker
        .func_wrap(
            "kali:node",
            "os_eol",
            move |mut caller: Caller<'_, KaliHostState>,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                write_guest_string(&mut caller, out_ptr, out_cap, os.eol())
            },
        )
        .map_err(|error| host_import_error("os_eol", error))?;

    linker
        .func_wrap(
            "kali:node",
            "os_tmpdir",
            move |mut caller: Caller<'_, KaliHostState>,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                write_guest_string(&mut caller, out_ptr, out_cap, os.tmpdir().to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("os_tmpdir", error))?;

    linker
        .func_wrap("kali:node", "os_cpus", move || -> i32 { os.cpus() as i32 })
        .map_err(|error| host_import_error("os_cpus", error))?;

    linker
        .func_wrap(
            "kali:node",
            "fs_promises_read_text_file",
            move |mut caller: Caller<'_, KaliHostState>,
                  path_ptr: i32,
                  path_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileRead {
                        path: host_path.clone(),
                    },
                )?;
                let text = fs_promises.read_text_file(&host_path).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to read '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, text.as_bytes())
            },
        )
        .map_err(|error| host_import_error("fs_promises_read_text_file", error))?;

    linker
        .func_wrap(
            "kali:node",
            "fs_promises_read_file",
            move |mut caller: Caller<'_, KaliHostState>,
                  path_ptr: i32,
                  path_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileRead {
                        path: host_path.clone(),
                    },
                )?;
                let bytes = fs_promises_for_read_file
                    .read_file(&host_path)
                    .map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to read '{}': {}",
                            host_path.display(),
                            error
                        ))
                    })?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, &bytes)
            },
        )
        .map_err(|error| host_import_error("fs_promises_read_file", error))?;

    linker
        .func_wrap(
            "kali:node",
            "fs_promises_write_text_file",
            move |mut caller: Caller<'_, KaliHostState>,
                  path_ptr: i32,
                  path_len: i32,
                  data_ptr: i32,
                  data_len: i32|
                  -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let data = read_guest_bytes(&mut caller, data_ptr, data_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileWrite {
                        path: host_path.clone(),
                    },
                )?;
                if let Some(parent) = host_path.parent() {
                    fs::create_dir_all(parent).map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to create '{}': {}",
                            parent.display(),
                            error
                        ))
                    })?;
                }
                let text = String::from_utf8(data).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "node fs/promises write_text_file expects UTF-8: {}",
                        error
                    ))
                })?;
                fs_promises_for_write_text
                    .write_text_file(&host_path, text)
                    .map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to write '{}': {}",
                            host_path.display(),
                            error
                        ))
                    })?;
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("fs_promises_write_text_file", error))?;

    linker
        .func_wrap(
            "kali:node",
            "fs_promises_write_file",
            move |mut caller: Caller<'_, KaliHostState>,
                  path_ptr: i32,
                  path_len: i32,
                  data_ptr: i32,
                  data_len: i32|
                  -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let data = read_guest_bytes(&mut caller, data_ptr, data_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileWrite {
                        path: host_path.clone(),
                    },
                )?;
                if let Some(parent) = host_path.parent() {
                    fs::create_dir_all(parent).map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to create '{}': {}",
                            parent.display(),
                            error
                        ))
                    })?;
                }
                fs_promises_for_write_file
                    .write_file(&host_path, &data)
                    .map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to write '{}': {}",
                            host_path.display(),
                            error
                        ))
                    })?;
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("fs_promises_write_file", error))?;

    linker
        .func_wrap(
            "kali:node",
            "stream_concat",
            move |mut caller: Caller<'_, KaliHostState>,
                  left_ptr: i32,
                  left_len: i32,
                  right_ptr: i32,
                  right_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let left = read_guest_bytes(&mut caller, left_ptr, left_len)?;
                let right = read_guest_bytes(&mut caller, right_ptr, right_len)?;
                let concatenated = stream.concat_bytes(&left, &right);
                write_guest_bytes(&mut caller, out_ptr, out_cap, &concatenated)
            },
        )
        .map_err(|error| host_import_error("stream_concat", error))?;

    linker
        .func_wrap(
            "kali:node",
            "http_get",
            move |mut caller: Caller<'_, KaliHostState>,
                  url_ptr: i32,
                  url_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let url = read_guest_string(&mut caller, url_ptr, url_len)?;
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::NetworkFetch { url: url.clone() },
                )?;
                let response = http
                    .request_get(&url)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, response.body())
            },
        )
        .map_err(|error| host_import_error("http_get", error))?;

    linker
        .func_wrap(
            "kali:node",
            "buffer_to_hex",
            move |mut caller: Caller<'_, KaliHostState>,
                  data_ptr: i32,
                  data_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let data = read_guest_bytes(&mut caller, data_ptr, data_len)?;
                let hex = NodeBuffer::from_bytes(data).to_hex();
                write_guest_string(&mut caller, out_ptr, out_cap, hex)
            },
        )
        .map_err(|error| host_import_error("buffer_to_hex", error))?;

    linker
        .func_wrap(
            "kali:node",
            "buffer_from_hex",
            move |mut caller: Caller<'_, KaliHostState>,
                  input_ptr: i32,
                  input_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let input = read_guest_string(&mut caller, input_ptr, input_len)?;
                let buffer = NodeBuffer::from_hex(&input)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, buffer.as_slice())
            },
        )
        .map_err(|error| host_import_error("buffer_from_hex", error))?;

    linker
        .func_wrap(
            "kali:node",
            "event_on",
            move |mut caller: Caller<'_, KaliHostState>,
                  event_ptr: i32,
                  event_len: i32,
                  callback_id: i32|
                  -> wasmtime::Result<i32> {
                let event_type = read_guest_string(&mut caller, event_ptr, event_len)?;
                caller
                    .data_mut()
                    .register_event_listener(event_type, callback_id);
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("event_on", error))?;

    linker
        .func_wrap(
            "kali:node",
            "event_listener_count",
            move |mut caller: Caller<'_, KaliHostState>,
                  event_ptr: i32,
                  event_len: i32|
                  -> wasmtime::Result<i32> {
                let event_type = read_guest_string(&mut caller, event_ptr, event_len)?;
                Ok(caller.data().event_listener_count(&event_type) as i32)
            },
        )
        .map_err(|error| host_import_error("event_listener_count", error))?;

    linker
        .func_wrap(
            "kali:node",
            "event_emit",
            move |mut caller: Caller<'_, KaliHostState>,
                  event_ptr: i32,
                  event_len: i32|
                  -> wasmtime::Result<i32> {
                let event_type = read_guest_string(&mut caller, event_ptr, event_len)?;
                let callback_ids = caller.data().event_listener_callbacks(&event_type);
                for callback_id in &callback_ids {
                    caller.data_mut().queue_microtask(*callback_id);
                }
                Ok(callback_ids.len() as i32)
            },
        )
        .map_err(|error| host_import_error("event_emit", error))?;

    linker
        .func_wrap(
            "kali:node",
            "util_format",
            move |mut caller: Caller<'_, KaliHostState>,
                  left_ptr: i32,
                  left_len: i32,
                  right_ptr: i32,
                  right_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let left = read_guest_string(&mut caller, left_ptr, left_len)?;
                let right = read_guest_string(&mut caller, right_ptr, right_len)?;
                let formatted = NodeUtil::format(&[left.as_str(), right.as_str()]);
                write_guest_string(&mut caller, out_ptr, out_cap, formatted)
            },
        )
        .map_err(|error| host_import_error("util_format", error))?;

    linker
        .func_wrap(
            "kali:node",
            "assert_equal",
            move |mut caller: Caller<'_, KaliHostState>,
                  actual_ptr: i32,
                  actual_len: i32,
                  expected_ptr: i32,
                  expected_len: i32|
                  -> wasmtime::Result<i32> {
                let actual = read_guest_string(&mut caller, actual_ptr, actual_len)?;
                let expected = read_guest_string(&mut caller, expected_ptr, expected_len)?;
                NodeAssert::equal(&actual, &expected, "assert_equal")
                    .map_err(wasmtime::Error::msg)?;
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("assert_equal", error))?;

    linker
        .func_wrap("kali:node", "process_args_len", move || -> i32 {
            process
                .lock()
                .expect("node process mutex poisoned")
                .argv_len() as i32
        })
        .map_err(|error| host_import_error("process_args_len", error))?;

    linker
        .func_wrap(
            "kali:node",
            "process_args_get",
            move |mut caller: Caller<'_, KaliHostState>,
                  index: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let Some(value) = process_for_argv_get
                    .lock()
                    .expect("node process mutex poisoned")
                    .argv_at(index as usize)
                    .map(str::to_owned)
                else {
                    return Ok(-1);
                };
                write_guest_bytes(&mut caller, out_ptr, out_cap, value.as_bytes())
            },
        )
        .map_err(|error| host_import_error("process_args_get", error))?;

    linker
        .func_wrap(
            "kali:node",
            "process_env_get",
            move |mut caller: Caller<'_, KaliHostState>,
                  key_ptr: i32,
                  key_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let key = read_guest_string(&mut caller, key_ptr, key_len)?;
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::EnvironmentRead { key: key.clone() },
                )?;
                let Some(value) = process_for_env_get
                    .lock()
                    .expect("node process mutex poisoned")
                    .env_get(&key)
                    .map(str::to_owned)
                else {
                    return Ok(-1);
                };
                write_guest_bytes(&mut caller, out_ptr, out_cap, value.as_bytes())
            },
        )
        .map_err(|error| host_import_error("process_env_get", error))?;

    linker
        .func_wrap(
            "kali:node",
            "process_stdout_write",
            move |mut caller: Caller<'_, KaliHostState>,
                  text_ptr: i32,
                  text_len: i32|
                  -> wasmtime::Result<i32> {
                let text = read_guest_string(&mut caller, text_ptr, text_len)?;
                append_stdout_raw(caller.data_mut(), text);
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("process_stdout_write", error))?;

    linker
        .func_wrap(
            "kali:node",
            "process_stderr_write",
            move |mut caller: Caller<'_, KaliHostState>,
                  text_ptr: i32,
                  text_len: i32|
                  -> wasmtime::Result<i32> {
                let text = read_guest_string(&mut caller, text_ptr, text_len)?;
                append_stderr_raw(caller.data_mut(), text);
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("process_stderr_write", error))?;

    linker
        .func_wrap(
            "kali:node",
            "process_spawn",
            move |mut caller: Caller<'_, KaliHostState>,
                  command_ptr: i32,
                  command_len: i32,
                  args_ptr: i32,
                  args_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let command = read_guest_string(&mut caller, command_ptr, command_len)?;
                let encoded_args = read_guest_string(&mut caller, args_ptr, args_len)?;
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::ProcessSpawn {
                        executable: command.clone(),
                    },
                )?;
                let args = decode_spawn_args(&encoded_args);
                {
                    let state = caller.data_mut();
                    state.begin_spawn()?;
                }
                let output = match child_process.spawn(&command, &args) {
                    Ok(output) => output,
                    Err(error) => {
                        caller.data_mut().finish_spawn();
                        return Err(wasmtime::Error::msg(error.to_string()));
                    }
                };
                {
                    let state = caller.data_mut();
                    state.finish_spawn();
                }
                let stdout = output.stdout();
                write_guest_bytes(&mut caller, out_ptr, out_cap, stdout)?;
                Ok(output.status())
            },
        )
        .map_err(|error| host_import_error("process_spawn", error))?;

    Ok(())
}

fn normalize_runtime_profiles(runtime_profiles: Vec<String>) -> Vec<String> {
    let mut normalized = BTreeSet::new();
    for profile in runtime_profiles {
        let profile = profile.trim();
        if !profile.is_empty() {
            normalized.insert(profile.to_string());
        }
    }
    normalized.into_iter().collect()
}

fn capture_env() -> BTreeMap<String, String> {
    std::env::vars().collect()
}

fn decode_spawn_args(encoded: &str) -> Vec<String> {
    if encoded.is_empty() {
        return Vec::new();
    }

    let mut args: Vec<String> = encoded.split('|').map(str::to_owned).collect();
    if args.last().is_some_and(|arg| arg.is_empty()) {
        args.pop();
    }
    args
}

fn read_guest_string(
    caller: &mut Caller<'_, KaliHostState>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<String> {
    let bytes = read_guest_bytes(caller, ptr, len)?;
    String::from_utf8(bytes).map_err(|error| {
        wasmtime::Error::msg(format!("guest string is not valid UTF-8: {}", error))
    })
}

fn read_guest_bytes(
    caller: &mut Caller<'_, KaliHostState>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<Vec<u8>> {
    let memory = guest_memory(caller)?;
    let start = checked_offset(ptr)?;
    let length = checked_offset(len)?;
    let end = start
        .checked_add(length)
        .ok_or_else(|| wasmtime::Error::msg("guest memory access overflow"))?;
    let data = memory.data(caller);
    let slice = data
        .get(start..end)
        .ok_or_else(|| wasmtime::Error::msg("guest memory access out of bounds"))?;
    Ok(slice.to_vec())
}

fn write_guest_bytes(
    caller: &mut Caller<'_, KaliHostState>,
    ptr: i32,
    cap: i32,
    bytes: &[u8],
) -> wasmtime::Result<i32> {
    let memory = guest_memory(caller)?;
    let start = checked_offset(ptr)?;
    let capacity = checked_offset(cap)?;
    if bytes.len() > capacity {
        return Err(wasmtime::Error::msg(format!(
            "guest output buffer too small: need {}, have {}",
            bytes.len(),
            capacity
        )));
    }
    memory.write(caller, start, bytes).map_err(|error| {
        wasmtime::Error::msg(format!("failed to write guest memory: {}", error))
    })?;
    Ok(bytes.len() as i32)
}

fn write_guest_string(
    caller: &mut Caller<'_, KaliHostState>,
    ptr: i32,
    cap: i32,
    value: impl AsRef<str>,
) -> wasmtime::Result<i32> {
    write_guest_bytes(caller, ptr, cap, value.as_ref().as_bytes())
}

fn guest_memory(caller: &mut Caller<'_, KaliHostState>) -> wasmtime::Result<Memory> {
    match caller.get_export("memory") {
        Some(Extern::Memory(memory)) => Ok(memory),
        _ => Err(wasmtime::Error::msg("guest module does not export memory")),
    }
}

fn checked_offset(value: i32) -> wasmtime::Result<usize> {
    usize::try_from(value).map_err(|_| wasmtime::Error::msg("negative guest memory offset"))
}

fn resolve_host_path(state: &KaliHostState, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        state.cwd.join(path)
    }
}

fn append_stdout(state: &mut KaliHostState, text: String) {
    state.stdout.push_str(&text);
    state.stdout.push('\n');
}

fn append_stdout_raw(state: &mut KaliHostState, text: String) {
    state.stdout.push_str(&text);
}

fn append_stderr(state: &mut KaliHostState, text: String) {
    state.stderr.push_str(&text);
    state.stderr.push('\n');
}

fn append_stderr_raw(state: &mut KaliHostState, text: String) {
    state.stderr.push_str(&text);
}

fn format_console_value(caller: &mut Caller<'_, KaliHostState>, value: i64) -> String {
    let raw = value as u64;
    if raw & STRING_HANDLE_TAG != 0 {
        let offset = ((raw >> 32) & 0x7fff_ffff) as i32;
        let len = (raw & 0xffff_ffff) as i32;
        if let Ok(bytes) = read_guest_bytes(caller, offset, len) {
            if let Ok(text) = String::from_utf8(bytes) {
                return text;
            }
        }
    }

    value.to_string()
}

const STRING_HANDLE_TAG: u64 = 0x8000_0000_0000_0000;

fn host_import_error(name: &str, error: impl std::fmt::Display) -> Diagnostic {
    Diagnostic::error(
        e4::UNCAUGHT_ERROR as u32,
        format!("failed to register host import '{}': {}", name, error),
    )
}

fn runtime_error_diagnostic(error: impl std::fmt::Display) -> Diagnostic {
    let message = error.to_string();
    if message.contains("KALI_E4001") || message.contains("E4001") {
        Diagnostic::error(e4::EFFECT_NOT_PERMITTED as u32, message)
    } else if message.contains("KALI_E4003")
        || message.contains("E4003")
        || message.contains("fuel")
        || message.contains("memory limit")
        || message.contains("resource limit")
    {
        Diagnostic::error(e4::RESOURCE_LIMIT_EXCEEDED as u32, message)
    } else {
        Diagnostic::error(e4::UNCAUGHT_ERROR as u32, message)
    }
}

fn enforce_operation(state: &mut KaliHostState, op: HostOperation) -> wasmtime::Result<()> {
    if let Some(policy) = state.policy.as_ref() {
        policy.check_operation(op).map_err(|diagnostic| {
            state.pending_diagnostic = Some(diagnostic.clone());
            let marker = match diagnostic.code {
                Some(code) if code == e4::EFFECT_NOT_PERMITTED as u32 => "KALI_E4001",
                Some(code) if code == e4::RESOURCE_LIMIT_EXCEEDED as u32 => "KALI_E4003",
                _ => "KALI_E4000",
            };
            wasmtime::Error::msg(format!("{}: {}", marker, diagnostic.message))
        })
    } else {
        Ok(())
    }
}

impl KaliHostState {
    fn schedule_timer(
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

    fn cancel_timer(&mut self, timer_id: i32) -> wasmtime::Result<()> {
        let timer_id = u32::try_from(timer_id)
            .map_err(|_| wasmtime::Error::msg("timer id must be non-negative"))?;
        if self.pending_timers.remove(&timer_id).is_none() {
            self.cancelled_timers.insert(timer_id);
        }
        Ok(())
    }

    fn queue_microtask(&mut self, callback_id: i32) {
        self.pending_microtasks.push_back(callback_id);
    }

    fn register_event_listener(&mut self, event_type: impl Into<String>, callback_id: i32) {
        self.event_listeners
            .entry(event_type.into())
            .or_default()
            .push(callback_id);
    }

    fn event_listener_callbacks(&self, event_type: &str) -> Vec<i32> {
        self.event_listeners
            .get(event_type)
            .cloned()
            .unwrap_or_default()
    }

    fn event_listener_count(&self, event_type: &str) -> usize {
        self.event_listeners
            .get(event_type)
            .map(|callbacks| callbacks.len())
            .unwrap_or(0)
    }

    fn begin_spawn(&mut self) -> wasmtime::Result<()> {
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

    fn finish_spawn(&mut self) {
        self.active_spawned_processes = self.active_spawned_processes.saturating_sub(1);
    }

    #[allow(dead_code)]
    fn begin_thread(&mut self) -> wasmtime::Result<()> {
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
    fn finish_thread(&mut self) {
        self.active_threads = self.active_threads.saturating_sub(1);
    }
}

fn drain_event_loop(
    instance: &Instance,
    store: &mut Store<KaliHostState>,
) -> Result<(), Diagnostic> {
    loop {
        let microtask_id = {
            let state = store.data_mut();
            state.pending_microtasks.pop_front()
        };

        if let Some(callback_id) = microtask_id {
            invoke_callback(instance, store, callback_id)?;
            continue;
        }

        let next_timer = {
            let state = store.data();
            state
                .pending_timers
                .iter()
                .min_by_key(|(_, timer)| timer.due_at)
                .map(|(timer_id, timer)| (*timer_id, timer.clone()))
        };

        let Some((timer_id, timer)) = next_timer else {
            break;
        };

        let now = Instant::now();
        if timer.due_at > now {
            thread::sleep(timer.due_at - now);
            continue;
        }

        {
            let state = store.data_mut();
            state.pending_timers.remove(&timer_id);
        }

        invoke_callback(instance, store, timer.callback_id)?;

        if let Some(interval) = timer.repeat_interval {
            let cancelled = {
                let state = store.data_mut();
                state.cancelled_timers.remove(&timer_id)
            };

            if !cancelled {
                let state = store.data_mut();
                state.pending_timers.insert(
                    timer_id,
                    ScheduledTimer {
                        callback_id: timer.callback_id,
                        due_at: Instant::now() + interval,
                        repeat_interval: Some(interval),
                    },
                );
            }
        }
    }

    Ok(())
}

fn invoke_callback(
    instance: &Instance,
    store: &mut Store<KaliHostState>,
    callback_id: i32,
) -> Result<(), Diagnostic> {
    // The current guest ABI uses exported callback stubs named
    // `__kali_callback_<id>` for timer and microtask scheduling.
    let export_name = format!("__kali_callback_{}", callback_id);
    let callback = instance
        .get_typed_func::<(), ()>(&mut *store, &export_name)
        .map_err(|error| {
            Diagnostic::error(
                e4::UNCAUGHT_ERROR as u32,
                format!("missing timer callback '{}': {}", export_name, error),
            )
        })?;

    if let Err(error) = callback.call(&mut *store, ()) {
        if let Some(diagnostic) = store.data_mut().pending_diagnostic.take() {
            return Err(diagnostic);
        }
        return Err(runtime_error_diagnostic(format!(
            "runtime trap in callback '{}': {}",
            export_name, error
        )));
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
