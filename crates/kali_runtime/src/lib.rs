//! Runtime execution for Kali-generated WASM modules.

use kali_api_node::{
    NodeAssert, NodeBuffer, NodeChildProcess, NodeCrypto, NodePath, NodeRuntimeProjection, NodeUrl,
    NodeUtil,
};
use kali_api_web::{fill_random_values, performance_now};
use kali_error::{_error_codes::e4, Diagnostic};
use kali_sandbox::{HostOperation, SandboxPolicy};
use reqwest::blocking;
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
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
}

impl Default for RuntimeCtx {
    fn default() -> Self {
        Self {
            policy: None,
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            api_surface: "deno".to_string(),
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
        }
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
                stdout: String::new(),
                stderr: String::new(),
                pending_timers: BTreeMap::new(),
                pending_microtasks: VecDeque::new(),
                cancelled_timers: HashSet::new(),
                next_timer_id: 0,
                registered_tests: Vec::new(),
                event_listeners: BTreeMap::new(),
                store_limits,
                pending_diagnostic: None,
                active_file_handles: 0,
                active_network_connections: 0,
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
                append_stdout(caller.data_mut(), format_tagged_val(val));
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
                append_stderr(caller.data_mut(), format_tagged_val(val));
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
                append_stderr(
                    caller.data_mut(),
                    format!("[warn] {}", format_tagged_val(val)),
                );
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
                let output = child_process
                    .spawn(&command, &args)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                let stdout = output.stdout();
                write_guest_bytes(&mut caller, out_ptr, out_cap, stdout)?;
                Ok(output.status())
            },
        )
        .map_err(|error| host_import_error("process_spawn", error))?;

    Ok(())
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

fn format_tagged_val(value: i64) -> String {
    value.to_string()
}

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
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    fn compile_wat(wat: &str) -> Vec<u8> {
        wat::parse_str(wat).unwrap_or_else(|error| panic!("valid wat error: {error}\n{wat}"))
    }

    fn wat_assert_buffer_eq(start: i32, expected: &str) -> String {
        let mut checks = String::new();
        for (index, byte) in expected.as_bytes().iter().enumerate() {
            checks.push_str(&format!(
                "                i32.const {}\n                i32.load8_u\n                i32.const {}\n                i32.ne\n                if\n                    unreachable\n                end\n",
                start + index as i32,
                byte
            ));
        }
        checks
    }

    #[test]
    fn runtime_executes_modules_with_console_host_imports() {
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (import "kali:rt" "console_error" (func $console_error (param i64)))
                (import "kali:rt" "console_warn" (func $console_warn (param i64)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i64.const 1
                    call $console_log
                    i64.const 2
                    call $console_error
                    i64.const 3
                    call $console_warn))
            "#,
        );

        let runtime = RuntimeCtx::default();
        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_exposes_arguments() {
        let runtime = RuntimeCtx::with_host_context(
            None,
            vec!["alpha".to_string(), "beta".to_string()],
            capture_env(),
            PathBuf::from("."),
        );

        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "args_len" (func $args_len (result i32)))
                (func (export "_start")
                    call $args_len
                    i32.const 2
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
        );

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_exposes_environment_variables() {
        let mut env = BTreeMap::new();
        env.insert("KALI_RUNTIME_TEST_ENV".to_string(), "hello".to_string());
        let runtime = RuntimeCtx::with_host_context(
            None,
            vec!["alpha".to_string(), "beta".to_string()],
            env,
            PathBuf::from("."),
        );

        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "env_get" (func $env_get (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "KALI_RUNTIME_TEST_ENV")
                (func (export "_start")
                    i32.const 0
                    i32.const 21
                    i32.const 128
                    i32.const 64
                    call $env_get
                    i32.const 5
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
        );

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_writes_text_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let runtime = RuntimeCtx::with_host_context(
            None,
            Vec::new(),
            capture_env(),
            dir.path().to_path_buf(),
        );
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "fs_write_text_file" (func $write (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "./written.txt")
                (data (i32.const 64) "hello runtime")
                (func (export "_start")
                    i32.const 0
                    i32.const 13
                    i32.const 64
                    i32.const 13
                    call $write
                    i32.const 0
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
        );

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);

        let written = fs::read_to_string(dir.path().join("written.txt")).expect("written file");
        assert_eq!(written, "hello runtime");
    }

    #[test]
    fn runtime_fetches_http_responses() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        let body = "hello fetch";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = [0u8; 1024];
            let _ = stream.read(&mut buffer);
            let _ = stream.write_all(response.as_bytes());
        });

        let runtime =
            RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
        let url = format!("http://127.0.0.1:{}/", addr.port());
        let wat = format!(
            r#"
            (module
                (import "kali:rt" "fetch" (func $fetch (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{}")
                (func (export "_start")
                    i32.const 0
                    i32.const {}
                    i32.const 128
                    i32.const 64
                    call $fetch
                    i32.const {}
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
            url,
            url.len(),
            body.len()
        );

        let wasm = compile_wat(&wat);
        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
        server.join().expect("server thread");
    }

    #[test]
    fn runtime_reports_mocked_fetch_failures() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = [0u8; 1024];
            let _ = stream.read(&mut buffer);
            let _ = stream.write_all(response.as_bytes());
        });

        let runtime =
            RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
        let url = format!("http://127.0.0.1:{}/missing", addr.port());
        let wat = format!(
            r#"
            (module
                (import "kali:rt" "fetch" (func $fetch (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{}")
                (func (export "_start")
                    i32.const 0
                    i32.const {}
                    i32.const 128
                    i32.const 64
                    call $fetch
                    drop))
            "#,
            url,
            url.len()
        );

        let wasm = compile_wat(&wat);
        let diagnostics = runtime.execute(&wasm).expect_err("fetch should fail");
        assert_eq!(diagnostics[0].code, Some(e4::UNCAUGHT_ERROR as u32));
        assert!(
            diagnostics[0].message.contains("runtime trap"),
            "diagnostic: {:?}",
            diagnostics[0]
        );
        server.join().expect("server thread");
    }

    #[test]
    fn runtime_executes_node_fs_promises_host_imports() {
        let dir = tempfile::tempdir().expect("tempdir");
        let runtime = RuntimeCtx::with_host_context_with_api_surface(
            None,
            Vec::new(),
            capture_env(),
            dir.path().to_path_buf(),
            "node",
        );
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:node" "fs_promises_write_text_file" (func $write (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "fs_promises_read_text_file" (func $read (param i32 i32 i32 i32) (result i32)))
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (memory (export "memory") 1)
                (data (i32.const 0) "./node-promises.txt")
                (data (i32.const 64) "hello node fs")
                (func (export "_start")
                    i32.const 0
                    i32.const 19
                    i32.const 64
                    i32.const 13
                    call $write
                    drop
                    i32.const 0
                    i32.const 19
                    i32.const 128
                    i32.const 64
                    call $read
                    i32.const 13
                    i32.eq
                    if
                        i64.const 13
                        call $console_log
                    else
                        unreachable
                    end))
            "#,
        );

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
        let written =
            fs::read_to_string(dir.path().join("node-promises.txt")).expect("written file");
        assert_eq!(written, "hello node fs");
    }

    #[test]
    fn runtime_executes_node_stream_host_imports() {
        let runtime = RuntimeCtx::with_host_context_with_api_surface(
            None,
            Vec::new(),
            capture_env(),
            PathBuf::from("."),
            "node",
        );
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:node" "stream_concat" (func $concat (param i32 i32 i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "hello ")
                (data (i32.const 32) "node")
                (func (export "_start")
                    i32.const 0
                    i32.const 6
                    i32.const 32
                    i32.const 4
                    i32.const 64
                    i32.const 32
                    call $concat
                    i32.const 10
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
        );

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_executes_node_http_host_imports() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        let body = "hello node http";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = [0u8; 1024];
            let _ = stream.read(&mut buffer);
            let _ = stream.write_all(response.as_bytes());
        });

        let runtime = RuntimeCtx::with_host_context_with_api_surface(
            None,
            Vec::new(),
            capture_env(),
            PathBuf::from("."),
            "node",
        );
        let url = format!("http://127.0.0.1:{}/", addr.port());
        let wat = format!(
            r#"
            (module
                (import "kali:node" "http_get" (func $http_get (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{}")
                (func (export "_start")
                    i32.const 0
                    i32.const {}
                    i32.const 128
                    i32.const 64
                    call $http_get
                    i32.const {}
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
            url,
            url.len(),
            body.len()
        );

        let wasm = compile_wat(&wat);
        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
        server.join().expect("server thread");
    }

    #[test]
    fn runtime_executes_node_process_host_imports() {
        let runtime = RuntimeCtx::with_host_context_with_api_surface(
            None,
            vec!["node".into(), "script.ts".into()],
            BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))]),
            PathBuf::from("."),
            "node",
        );
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:node" "process_args_len" (func $args_len (result i32)))
                (import "kali:node" "process_args_get" (func $args_get (param i32 i32 i32) (result i32)))
                (import "kali:node" "process_env_get" (func $env_get (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "process_stdout_write" (func $stdout_write (param i32 i32) (result i32)))
                (import "kali:node" "process_stderr_write" (func $stderr_write (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "HOME")
                (data (i32.const 32) "script.ts")
                (data (i32.const 64) "node stdout")
                (data (i32.const 96) "node stderr")
                (func (export "_start")
                    call $args_len
                    i32.const 2
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 1
                    i32.const 160
                    i32.const 16
                    call $args_get
                    i32.const 9
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 160
                    i32.load8_u
                    i32.const 115
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 0
                    i32.const 4
                    i32.const 192
                    i32.const 16
                    call $env_get
                    i32.const 9
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 192
                    i32.load8_u
                    i32.const 47
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 64
                    i32.const 11
                    call $stdout_write
                    drop
                    i32.const 96
                    i32.const 11
                    call $stderr_write
                    drop))
            "#,
        );

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(outcome.stdout, "node stdout");
        assert_eq!(outcome.stderr, "node stderr");
    }

    #[cfg(not(windows))]
    #[test]
    fn runtime_executes_node_child_process_host_imports() {
        let runtime = RuntimeCtx::with_host_context_with_api_surface(
            None,
            Vec::new(),
            capture_env(),
            PathBuf::from("."),
            "node",
        );
        let command = "sh";
        let args = "-lc|printf child-process";
        let expected_stdout = "child-process";
        let wat = format!(
            r#"
            (module
                (import "kali:node" "process_spawn" (func $spawn (param i32 i32 i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{command}")
                (data (i32.const 32) "{args}")
                (func (export "_start")
                    i32.const 0
                    i32.const {command_len}
                    i32.const 32
                    i32.const {args_len}
                    i32.const 96
                    i32.const 32
                    call $spawn
                    i32.const 0
                    i32.ne
                    if
                        unreachable
                    end
{stdout_checks}
                )
            )
            "#,
            command = command,
            command_len = command.len(),
            args = args,
            args_len = args.len(),
            stdout_checks = wat_assert_buffer_eq(96, expected_stdout),
        );

        let wasm = compile_wat(&wat);
        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_executes_node_util_buffer_and_assert_host_imports() {
        let runtime = RuntimeCtx::with_host_context_with_api_surface(
            None,
            Vec::new(),
            capture_env(),
            PathBuf::from("."),
            "node",
        );
        let format_left = "node";
        let format_right = "compat";
        let buffer_input = "hello node";
        let buffer_hex = "68656c6c6f206e6f6465";
        let wat = format!(
            r#"
            (module
                (import "kali:node" "util_format" (func $format (param i32 i32 i32 i32 i32 i32) (result i32)))
                (import "kali:node" "buffer_to_hex" (func $buffer_to_hex (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "buffer_from_hex" (func $buffer_from_hex (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "assert_equal" (func $assert_equal (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{format_left}")
                (data (i32.const 32) "{format_right}")
                (data (i32.const 64) "{buffer_input}")
                (data (i32.const 128) "{buffer_hex}")
                (data (i32.const 192) "kali")
                (data (i32.const 224) "kali")
                (func (export "_start")
                    i32.const 0
                    i32.const {format_left_len}
                    i32.const 32
                    i32.const {format_right_len}
                    i32.const 256
                    i32.const 64
                    call $format
                    i32.const {format_output_len}
                    i32.ne
                    if
                        unreachable
                    end
{format_checks}
                    i32.const 64
                    i32.const {buffer_input_len}
                    i32.const 320
                    i32.const 32
                    call $buffer_to_hex
                    i32.const {buffer_hex_len}
                    i32.ne
                    if
                        unreachable
                    end
{buffer_hex_checks}
                    i32.const 128
                    i32.const {buffer_hex_len}
                    i32.const 384
                    i32.const {buffer_input_len}
                    call $buffer_from_hex
                    i32.const {buffer_input_len}
                    i32.ne
                    if
                        unreachable
                    end
{buffer_round_trip_checks}
                    i32.const 192
                    i32.const 4
                    i32.const 224
                    i32.const 4
                    call $assert_equal
                    i32.const 0
                    i32.ne
                    if
                        unreachable
                    end)
            )
            "#,
            format_left = format_left,
            format_left_len = format_left.len(),
            format_right = format_right,
            format_right_len = format_right.len(),
            format_output_len = format!("{} {}", format_left, format_right).len(),
            format_checks = wat_assert_buffer_eq(256, &format!("{} {}", format_left, format_right)),
            buffer_input = buffer_input,
            buffer_input_len = buffer_input.len(),
            buffer_hex_len = buffer_hex.len(),
            buffer_hex_checks = wat_assert_buffer_eq(320, buffer_hex),
            buffer_round_trip_checks = wat_assert_buffer_eq(384, buffer_input),
        );

        let wasm = compile_wat(&wat);
        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_executes_node_event_emitter_host_imports() {
        let runtime = RuntimeCtx::with_host_context_with_api_surface(
            None,
            Vec::new(),
            capture_env(),
            PathBuf::from("."),
            "node",
        );
        let wat = r#"
            (module
                (import "kali:node" "event_on" (func $event_on (param i32 i32 i32) (result i32)))
                (import "kali:node" "event_listener_count" (func $listener_count (param i32 i32) (result i32)))
                (import "kali:node" "event_emit" (func $event_emit (param i32 i32) (result i32)))
                (import "kali:node" "process_stdout_write" (func $stdout_write (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "message")
                (data (i32.const 32) "event fired")
                (func (export "__kali_callback_1")
                    i32.const 32
                    i32.const 11
                    call $stdout_write
                    drop)
                (func (export "_start")
                    i32.const 0
                    i32.const 7
                    i32.const 1
                    call $event_on
                    drop
                    i32.const 0
                    i32.const 7
                    call $listener_count
                    i32.const 1
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 0
                    i32.const 7
                    call $event_emit
                    i32.const 1
                    i32.ne
                    if
                        unreachable
                    end)
            )
        "#;

        let wasm = compile_wat(wat);
        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(outcome.stdout, "event fired");
    }

    #[test]
    fn runtime_executes_node_path_host_imports() {
        let runtime = RuntimeCtx::with_host_context_with_api_surface(
            None,
            Vec::new(),
            capture_env(),
            PathBuf::from("."),
            "node",
        );
        let normalize_input = "./foo/../bar//baz";
        let join_base = "/tmp";
        let join_segment = "project/src";
        let resolve_base = "/tmp/project";
        let resolve_input = "../lib/index.js";
        let dirname_input = "/tmp/project/src/main.ts";
        let basename_input = "/tmp/project/src/main.ts";
        let extname_input = "/tmp/project/src/main.ts";
        let relative_from = "/tmp/project/src";
        let relative_to = "/tmp/project/lib/index.js";
        let normalized_output = "bar/baz";
        let joined_output = "/tmp/project/src";
        let resolved_output = "/tmp/lib/index.js";
        let dirname_output = "/tmp/project/src";
        let basename_output = "main.ts";
        let extname_output = ".ts";
        let relative_output = "../lib/index.js";
        let wat = format!(
            r#"
            (module
                (import "kali:node" "path_normalize" (func $normalize (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_join" (func $join (param i32 i32 i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_resolve" (func $resolve (param i32 i32 i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_dirname" (func $dirname (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_basename" (func $basename (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_extname" (func $extname (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "path_relative" (func $relative (param i32 i32 i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{normalize_input}")
                (data (i32.const 64) "{join_base}")
                (data (i32.const 80) "{join_segment}")
                (data (i32.const 112) "{resolve_base}")
                (data (i32.const 144) "{resolve_input}")
                (data (i32.const 192) "{dirname_input}")
                (data (i32.const 224) "{basename_input}")
                (data (i32.const 256) "{extname_input}")
                (data (i32.const 768) "{relative_from}")
                (data (i32.const 832) "{relative_to}")
                (func (export "_start")
                    ;; normalize
                    i32.const 0
                    i32.const {normalize_len}
                    i32.const 320
                    i32.const 32
                    call $normalize
                    i32.const {normalized_len}
                    i32.ne
                    if
                        unreachable
                    end
{normalize_checks}
                    ;; join
                    i32.const 64
                    i32.const {join_base_len}
                    i32.const 80
                    i32.const {join_segment_len}
                    i32.const 384
                    i32.const 32
                    call $join
                    i32.const {joined_len}
                    i32.ne
                    if
                        unreachable
                    end
{join_checks}
                    ;; resolve
                    i32.const 112
                    i32.const {resolve_base_len}
                    i32.const 144
                    i32.const {resolve_input_len}
                    i32.const 448
                    i32.const 32
                    call $resolve
                    i32.const {resolved_len}
                    i32.ne
                    if
                        unreachable
                    end
{resolve_checks}
                    ;; dirname
                    i32.const 192
                    i32.const {dirname_input_len}
                    i32.const 512
                    i32.const 32
                    call $dirname
                    i32.const {dirname_len}
                    i32.ne
                    if
                        unreachable
                    end
{dirname_checks}
                    ;; basename
                    i32.const 224
                    i32.const {basename_input_len}
                    i32.const 576
                    i32.const 32
                    call $basename
                    i32.const {basename_len}
                    i32.ne
                    if
                        unreachable
                    end
{basename_checks}
                    ;; extname
                    i32.const 256
                    i32.const {extname_input_len}
                    i32.const 640
                    i32.const 32
                    call $extname
                    i32.const {extname_len}
                    i32.ne
                    if
                        unreachable
                    end
{extname_checks}
                    ;; relative
                    i32.const 768
                    i32.const {relative_from_len}
                    i32.const 832
                    i32.const {relative_to_len}
                    i32.const 704
                    i32.const 32
                    call $relative
                    i32.const {relative_len}
                    i32.ne
                    if
                        unreachable
                    end
{relative_checks}
                )
            )
            "#,
            normalize_input = normalize_input,
            normalize_len = normalize_input.len(),
            normalized_len = normalized_output.len(),
            normalize_checks = wat_assert_buffer_eq(320, normalized_output),
            join_base = join_base,
            join_base_len = join_base.len(),
            join_segment = join_segment,
            join_segment_len = join_segment.len(),
            joined_len = joined_output.len(),
            join_checks = wat_assert_buffer_eq(384, joined_output),
            resolve_base = resolve_base,
            resolve_base_len = resolve_base.len(),
            resolve_input = resolve_input,
            resolve_input_len = resolve_input.len(),
            resolved_len = resolved_output.len(),
            resolve_checks = wat_assert_buffer_eq(448, resolved_output),
            dirname_input = dirname_input,
            dirname_input_len = dirname_input.len(),
            dirname_len = dirname_output.len(),
            dirname_checks = wat_assert_buffer_eq(512, dirname_output),
            basename_input = basename_input,
            basename_input_len = basename_input.len(),
            basename_len = basename_output.len(),
            basename_checks = wat_assert_buffer_eq(576, basename_output),
            extname_input = extname_input,
            extname_input_len = extname_input.len(),
            extname_len = extname_output.len(),
            extname_checks = wat_assert_buffer_eq(640, extname_output),
            relative_from = relative_from,
            relative_from_len = relative_from.len(),
            relative_to = relative_to,
            relative_to_len = relative_to.len(),
            relative_len = relative_output.len(),
            relative_checks = wat_assert_buffer_eq(704, relative_output),
        );

        let wasm = compile_wat(&wat);
        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_executes_node_url_host_imports() {
        let runtime = RuntimeCtx::with_host_context_with_api_surface(
            None,
            Vec::new(),
            capture_env(),
            PathBuf::from("."),
            "node",
        );
        let parse_input = "https://example.com/path?query=1";
        let resolve_base = "https://example.com/base/";
        let resolve_input = "../child";
        let parse_output = "https://example.com/path?query=1";
        let resolve_output = "https://example.com/child";
        let wat = format!(
            r#"
            (module
                (import "kali:node" "url_parse" (func $parse (param i32 i32 i32 i32) (result i32)))
                (import "kali:node" "url_resolve" (func $resolve (param i32 i32 i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{parse_input}")
                (data (i32.const 64) "{resolve_base}")
                (data (i32.const 96) "{resolve_input}")
                (func (export "_start")
                    i32.const 0
                    i32.const {parse_len}
                    i32.const 256
                    i32.const 64
                    call $parse
                    i32.const {parse_output_len}
                    i32.ne
                    if
                        unreachable
                    end
{parse_checks}
                    i32.const 64
                    i32.const {resolve_base_len}
                    i32.const 96
                    i32.const {resolve_input_len}
                    i32.const 320
                    i32.const 64
                    call $resolve
                    i32.const {resolve_output_len}
                    i32.ne
                    if
                        unreachable
                    end
{resolve_checks}
                )
            )
            "#,
            parse_input = parse_input,
            parse_len = parse_input.len(),
            parse_output_len = parse_output.len(),
            parse_checks = wat_assert_buffer_eq(256, parse_output),
            resolve_base = resolve_base,
            resolve_base_len = resolve_base.len(),
            resolve_input = resolve_input,
            resolve_input_len = resolve_input.len(),
            resolve_output_len = resolve_output.len(),
            resolve_checks = wat_assert_buffer_eq(320, resolve_output),
        );

        let wasm = compile_wat(&wat);
        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_executes_node_crypto_and_os_host_imports() {
        let runtime = RuntimeCtx::with_host_context_with_api_surface(
            None,
            Vec::new(),
            capture_env(),
            PathBuf::from("."),
            "node",
        );
        let hash_input = "hello";
        let hmac_key = "key";
        let hmac_input = "The quick brown fox jumps over the lazy dog";
        let expected_hash = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        let expected_hmac = "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8";
        let expected_platform = std::env::consts::OS;
        let expected_arch = std::env::consts::ARCH;
        let expected_eol = if cfg!(windows) { "\r\n" } else { "\n" };
        let eol_checks = if expected_eol.len() == 1 {
            format!(
                "                    i32.const 288\n                    i32.load8_u\n                    i32.const {}\n                    i32.ne\n                    if\n                        unreachable\n                    end\n",
                expected_eol.as_bytes()[0]
            )
        } else {
            format!(
                "                    i32.const 288\n                    i32.load8_u\n                    i32.const {}\n                    i32.ne\n                    if\n                        unreachable\n                    end\n                    i32.const 289\n                    i32.load8_u\n                    i32.const {}\n                    i32.ne\n                    if\n                        unreachable\n                    end\n",
                expected_eol.as_bytes()[0],
                expected_eol.as_bytes()[1]
            )
        };
        let wat = format!(
            r#"
            (module
                (import "kali:node" "crypto_create_hash" (func $hash (param i32 i32 i32 i32 i32 i32) (result i32)))
                (import "kali:node" "crypto_create_hmac" (func $hmac (param i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
                (import "kali:node" "crypto_random_uuid" (func $uuid (param i32 i32) (result i32)))
                (import "kali:node" "os_platform" (func $platform (param i32 i32) (result i32)))
                (import "kali:node" "os_arch" (func $arch (param i32 i32) (result i32)))
                (import "kali:node" "os_eol" (func $eol (param i32 i32) (result i32)))
                (import "kali:node" "os_cpus" (func $cpus (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{hash_input}")
                (data (i32.const 32) "sha256")
                (data (i32.const 64) "{hmac_key}")
                (data (i32.const 96) "{hmac_input}")
                (data (i32.const 160) "{expected_platform}")
                (data (i32.const 224) "{expected_arch}")
                (func (export "_start")
                    i32.const 32
                    i32.const 6
                    i32.const 0
                    i32.const {hash_input_len}
                    i32.const 320
                    i32.const 80
                    call $hash
                    i32.const {expected_hash_len}
                    i32.ne
                    if
                        unreachable
                    end
{hash_checks}
                    i32.const 32
                    i32.const 6
                    i32.const 64
                    i32.const {hmac_key_len}
                    i32.const 96
                    i32.const {hmac_input_len}
                    i32.const 416
                    i32.const 80
                    call $hmac
                    i32.const {expected_hmac_len}
                    i32.ne
                    if
                        unreachable
                    end
{hmac_checks}
                    i32.const 480
                    i32.const 36
                    call $uuid
                    i32.const 36
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 488
                    i32.load8_u
                    i32.const 45
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 493
                    i32.load8_u
                    i32.const 45
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 494
                    i32.load8_u
                    i32.const 52
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 498
                    i32.load8_u
                    i32.const 45
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 503
                    i32.load8_u
                    i32.const 45
                    i32.ne
                    if
                        unreachable
                    end
                    i32.const 160
                    i32.const {expected_platform_len}
                    call $platform
                    i32.const {expected_platform_len}
                    i32.ne
                    if
                        unreachable
                    end
{platform_checks}
                    i32.const 224
                    i32.const {expected_arch_len}
                    call $arch
                    i32.const {expected_arch_len}
                    i32.ne
                    if
                        unreachable
                    end
{arch_checks}
                    i32.const 288
                    i32.const {expected_eol_len}
                    call $eol
                    i32.const {expected_eol_len}
                    i32.ne
                    if
                        unreachable
                    end
{eol_checks}
                    call $cpus
                    i32.const 1
                    i32.lt_s
                    if
                        unreachable
                    end)
            )
            "#,
            hash_input = hash_input,
            hash_input_len = hash_input.len(),
            expected_hash_len = expected_hash.len(),
            hash_checks = wat_assert_buffer_eq(320, expected_hash),
            hmac_key = hmac_key,
            hmac_key_len = hmac_key.len(),
            hmac_input = hmac_input,
            hmac_input_len = hmac_input.len(),
            expected_hmac_len = expected_hmac.len(),
            hmac_checks = wat_assert_buffer_eq(416, expected_hmac),
            expected_platform = expected_platform,
            expected_platform_len = expected_platform.len(),
            platform_checks = wat_assert_buffer_eq(160, expected_platform),
            expected_arch = expected_arch,
            expected_arch_len = expected_arch.len(),
            arch_checks = wat_assert_buffer_eq(224, expected_arch),
            expected_eol_len = expected_eol.len(),
            eol_checks = eol_checks,
        );

        let wasm = compile_wat(&wat);
        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_exposes_performance_now() {
        let runtime =
            RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "performance_now" (func $now (result f64)))
                (func (export "_start")
                    call $now
                    f64.const 0.0
                    f64.ge
                    if
                    else
                        unreachable
                    end))
            "#,
        );

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_fills_random_values() {
        let runtime =
            RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "crypto_get_random_values" (func $random (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i32.const 0
                    i32.const 16
                    call $random
                    i32.const 16
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
        );

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_rejects_console_calls_when_policy_denies_them() {
        let policy = SandboxPolicy {
            schema_version: 1,
            schema_uri: None,
            effects: kali_sandbox::EffectsPolicy {
                file_system: kali_sandbox::FileSystemPolicy {
                    read: kali_sandbox::AccessRule::Deny(false),
                    write: kali_sandbox::AccessRule::Deny(false),
                },
                network: kali_sandbox::NetworkPolicy {
                    fetch: kali_sandbox::AccessRule::Deny(false),
                    connect: kali_sandbox::AccessRule::Deny(false),
                    listen: kali_sandbox::AccessRule::Deny(false),
                    max_connections: Some(1),
                },
                process: kali_sandbox::ProcessPolicy {
                    spawn: kali_sandbox::AccessRule::Deny(false),
                    env_read: kali_sandbox::AccessRule::Deny(false),
                    env_write: kali_sandbox::AccessRule::Deny(false),
                },
                timer: kali_sandbox::TimerPolicy {
                    schedule: true,
                    max_timeout_ms: Some(1000),
                    max_active_timers: Some(1),
                },
                eval: false,
                random: true,
                console: false,
            },
            resources: kali_sandbox::ResourceLimits {
                max_memory_mb: Some(256),
                max_cpu_time_ms: Some(1000),
                max_open_files: Some(8),
                max_spawned_processes: Some(0),
                max_threads: Some(0),
            },
            base_dir: PathBuf::from("."),
            serialized_source: None,
        };
        let runtime = RuntimeCtx::with_host_context(
            Some(policy),
            Vec::new(),
            capture_env(),
            PathBuf::from("."),
        );
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i64.const 1
                    call $console_log))
            "#,
        );

        let diagnostics = runtime
            .execute(&wasm)
            .expect_err("console should be denied");
        assert_eq!(diagnostics[0].code, Some(e4::EFFECT_NOT_PERMITTED as u32));
    }

    #[test]
    fn runtime_drains_microtasks_before_timers() {
        let runtime =
            RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "queueMicrotask" (func $queue_microtask (param i32)))
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (func (export "__kali_callback_1")
                    i32.const 1
                    global.set $state)
                (func (export "__kali_callback_2")
                    global.get $state
                    i32.const 1
                    i32.eq
                    if
                        i32.const 2
                        global.set $state
                    else
                        unreachable
                    end)
                (func (export "_start")
                    i32.const 1
                    call $queue_microtask
                    i32.const 2
                    i32.const 0
                    call $set_timeout
                    drop)
            )
            "#,
        );

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_repeating_intervals_can_be_cleared_from_callbacks() {
        let runtime =
            RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "setInterval" (func $set_interval (param i32 i32) (result i32)))
                (import "kali:rt" "clearInterval" (func $clear_interval (param i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (global $timer_id (mut i32) (i32.const -1))
                (func (export "__kali_callback_3")
                    global.get $state
                    i32.const 1
                    i32.add
                    global.set $state
                    global.get $state
                    i32.const 2
                    i32.eq
                    if
                        global.get $timer_id
                        call $clear_interval
                    else
                        global.get $state
                        i32.const 2
                        i32.gt_s
                        if
                            unreachable
                        end
                    end)
                (func (export "_start")
                    i32.const 3
                    i32.const 0
                    call $set_interval
                    global.set $timer_id)
            )
            "#,
        );

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_reports_traps_from_the_entrypoint() {
        let runtime =
            RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
        let wasm = compile_wat(
            r#"
            (module
                (memory (export "memory") 1)
                (func (export "_start")
                    i64.const 1
                    i64.const 0
                    i64.div_s
                    drop)
            )
            "#,
        );

        let diagnostics = runtime
            .execute(&wasm)
            .expect_err("division by zero should trap");
        assert_eq!(diagnostics[0].code, Some(e4::UNCAUGHT_ERROR as u32));
        assert!(diagnostics[0].message.contains("runtime trap"));
    }

    #[test]
    fn runtime_can_clear_scheduled_timers() {
        let runtime =
            RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32) (result i32)))
                (import "kali:rt" "clearTimeout" (func $clear_timeout (param i32)))
                (func (export "__kali_callback_7")
                    unreachable)
                (func (export "_start")
                    i32.const 7
                    i32.const 0
                    call $set_timeout
                    call $clear_timeout)
            )
            "#,
        );

        let outcome = runtime.execute(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
    }

    #[test]
    fn runtime_collects_and_runs_registered_tests() {
        let runtime =
            RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i32)))
                (func (export "__kali_callback_1")
                    i32.const 1
                    i32.const 1
                    i32.add
                    drop)
                (func (export "_start")
                    i32.const 1
                    call $test_register)
            )
            "#,
        );

        let outcome = runtime.execute_tests(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(outcome.tests_run, 1);
        assert_eq!(outcome.tests_failed, 0);
    }

    #[test]
    fn runtime_reports_failed_registered_tests() {
        let runtime =
            RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
        let wasm = compile_wat(
            r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i32)))
                (func (export "__kali_callback_2")
                    unreachable)
                (func (export "_start")
                    i32.const 2
                    call $test_register)
            )
            "#,
        );

        let outcome = runtime.execute_tests(&wasm).expect("runtime outcome");
        assert_eq!(outcome.exit_code, 1);
        assert_eq!(outcome.tests_run, 1);
        assert_eq!(outcome.tests_failed, 1);
    }
}
