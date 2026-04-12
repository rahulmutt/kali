//! Runtime execution for Kali-generated WASM modules.

use kali_api_web::{fill_random_values, performance_now};
use kali_error::{Diagnostic, _error_codes::e4};
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

fn capture_env() -> BTreeMap<String, String> {
    std::env::vars().collect()
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

fn append_stderr(state: &mut KaliHostState, text: String) {
    state.stderr.push_str(&text);
    state.stderr.push('\n');
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
        wat::parse_str(wat).expect("valid wat")
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
