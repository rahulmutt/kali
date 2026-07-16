//! Execution methods for `RuntimeCtx` and browser runtime dispatch.

use crate::*;

impl RuntimeCtx {
    pub(crate) fn browser_harness_command(&self) -> Option<&str> {
        self.env
            .get(BROWSER_HARNESS_COMMAND_ENV)
            .map(String::as_str)
    }

    pub(crate) fn reject_unavailable_threaded_requests(&self) -> Option<Diagnostic> {
        let has_threaded_profile = self
            .canonical_runtime_profiles()
            .iter()
            .any(|profile| profile == "wasm-threads");

        if self.max_threads.is_some_and(|count| count > 0) && !has_threaded_profile {
            return Some(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "selected resource budget `resources.maxThreads` is unavailable without the `--wasm-threads` runtime profile",
            ));
        }

        None
    }

    /// Execute a WASM module.
    pub fn execute(&self, wasm_bytes: &[u8]) -> Result<RuntimeOutcome, Vec<Diagnostic>> {
        self.execute_inner(wasm_bytes, false)
    }

    /// Execute a WASM module as a test suite, running guest-registered test callbacks.
    pub fn execute_tests(&self, wasm_bytes: &[u8]) -> Result<RuntimeOutcome, Vec<Diagnostic>> {
        self.execute_inner(wasm_bytes, true)
    }

    pub(crate) fn execute_inner(
        &self,
        wasm_bytes: &[u8],
        run_registered_tests: bool,
    ) -> Result<RuntimeOutcome, Vec<Diagnostic>> {
        let normalized_runtime_profiles = self.canonical_runtime_profiles();
        if let Some(diagnostic) = self.reject_unavailable_threaded_requests() {
            return Err(vec![diagnostic]);
        }

        if matches!(self.host_contract(), RuntimeHostContract::BrowserRequested) {
            if let Some(browser_harness_command) = self.browser_harness_command() {
                return execute_browser_runtime(
                    self,
                    wasm_bytes,
                    run_registered_tests,
                    normalized_runtime_profiles,
                    browser_harness_command,
                );
            }

            return Err(vec![browser_runtime_unavailable_diagnostic(
                None,
                Some(browser_runtime_request_context(
                    DiagnosticContextOrigin::Default,
                )),
            )]);
        }

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
                process_id: self.process_id,
                runtime_profiles: normalized_runtime_profiles.clone(),
                host_contract: self.host_contract(),
                runtime_backend: self.runtime_backend(),
                max_threads: self.effective_thread_budget(),
                max_spawned_processes: self
                    .policy
                    .as_ref()
                    .map(|policy| policy.effective_spawn_budget(self.max_spawned_processes))
                    .unwrap_or(self.max_spawned_processes),
                stdout: String::new(),
                stdout_bytes: Vec::new(),
                stderr: String::new(),
                pending_timers: BTreeMap::new(),
                pending_microtasks: VecDeque::new(),
                cancelled_timers: HashSet::new(),
                thread_topology: ThreadRuntimeTopology::default(),
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
                pending_exit_code: None,
            },
        );
        store.limiter(|state| &mut state.store_limits);
        // Default CPU-fuel budget when no sandbox policy sets `maxCpuTimeMs`:
        // ~60s-equivalent (60_000 * 1_000 fuel). Benchmark-scale programs such
        // as spectral-norm(100) need ~12-15M fuel and would trap under the old
        // 10s/10M default. An explicit policy `resources.maxCpuTimeMs` still
        // overrides this fallback.
        let default_fuel = self
            .policy
            .as_ref()
            .and_then(|policy| policy.resources.max_cpu_time_ms)
            .unwrap_or(60_000);
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
            if let Some(exit_code) = store.data_mut().take_pending_exit_code() {
                let state = store.data();
                return Ok(RuntimeOutcome {
                    exit_code,
                    tests_run: 0,
                    tests_failed: 0,
                    stdout: state.stdout.clone(),
                    stdout_bytes: state.stdout_bytes.clone(),
                    stderr: state.stderr.clone(),
                    coverage_hits: state.coverage_hits.iter().copied().collect(),
                    runtime_profiles: normalized_runtime_profiles.clone(),
                    host_contract: self.host_contract(),
                    runtime_backend: self.runtime_backend(),
                    thread_topology: state.thread_topology_snapshot(),
                    trap: None,
                });
            }
            if let Some(diagnostic) = store.data_mut().pending_diagnostic.take() {
                return Err(vec![diagnostic]);
            }
            let diagnostic = match error.downcast_ref::<wasmtime::Trap>() {
                Some(wasmtime::Trap::OutOfFuel) => Diagnostic::error(
                    e4::RESOURCE_LIMIT_EXCEEDED as u32,
                    "CPU fuel budget exhausted: the program ran past the runaway guard \
                     (default ~60s-equivalent when no sandbox policy is set); grant more \
                     compute by raising `resources.maxCpuTimeMs` in a --sandbox policy"
                        .to_string(),
                ),
                Some(wasmtime::Trap::MemoryOutOfBounds) => runtime_error_diagnostic(format!(
                    "runtime trap (out-of-bounds memory access): {}",
                    error
                )),
                Some(wasmtime::Trap::UnreachableCodeReached) => runtime_error_diagnostic(format!(
                    "runtime trap (unreachable — allocation failure or an unsupported-path guard): {}",
                    error
                )),
                _ => runtime_error_diagnostic(format!("runtime trap: {}", error)),
            };
            let state = store.data();
            return Ok(RuntimeOutcome {
                exit_code: 1,
                tests_run: 0,
                tests_failed: 0,
                stdout: state.stdout.clone(),
                stdout_bytes: state.stdout_bytes.clone(),
                stderr: state.stderr.clone(),
                coverage_hits: state.coverage_hits.iter().copied().collect(),
                runtime_profiles: normalized_runtime_profiles.clone(),
                host_contract: self.host_contract(),
                runtime_backend: self.runtime_backend(),
                thread_topology: state.thread_topology_snapshot(),
                trap: Some(diagnostic),
            });
        }

        if let Err(diagnostic) = drain_event_loop(&instance, &mut store) {
            if let Some(exit_code) = store.data_mut().take_pending_exit_code() {
                let state = store.data();
                return Ok(RuntimeOutcome {
                    exit_code,
                    tests_run: 0,
                    tests_failed: 0,
                    stdout: state.stdout.clone(),
                    stdout_bytes: state.stdout_bytes.clone(),
                    stderr: state.stderr.clone(),
                    coverage_hits: state.coverage_hits.iter().copied().collect(),
                    runtime_profiles: normalized_runtime_profiles.clone(),
                    host_contract: self.host_contract(),
                    runtime_backend: self.runtime_backend(),
                    thread_topology: state.thread_topology_snapshot(),
                    trap: None,
                });
            }
            return Err(vec![diagnostic]);
        }

        if !run_registered_tests {
            let state = store.data();
            return Ok(RuntimeOutcome {
                exit_code: 0,
                tests_run: 0,
                tests_failed: 0,
                stdout: state.stdout.clone(),
                stdout_bytes: state.stdout_bytes.clone(),
                stderr: state.stderr.clone(),
                coverage_hits: state.coverage_hits.iter().copied().collect(),
                runtime_profiles: normalized_runtime_profiles.clone(),
                host_contract: self.host_contract(),
                runtime_backend: self.runtime_backend(),
                thread_topology: state.thread_topology_snapshot(),
                trap: None,
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
                stdout_bytes: state.stdout_bytes.clone(),
                stderr: state.stderr.clone(),
                coverage_hits: state.coverage_hits.iter().copied().collect(),
                runtime_profiles: normalized_runtime_profiles.clone(),
                host_contract: self.host_contract(),
                runtime_backend: self.runtime_backend(),
                thread_topology: state.thread_topology_snapshot(),
                trap: None,
            });
        }

        let mut tests_run = 0usize;
        let mut tests_failed = 0usize;
        for (callback_id, env_ptr) in registered_tests {
            tests_run += 1;
            match invoke_callback(&instance, &mut store, callback_id, env_ptr) {
                Ok(()) => {}
                Err(diagnostic) => {
                    if let Some(exit_code) = store.data_mut().take_pending_exit_code() {
                        let state = store.data();
                        return Ok(RuntimeOutcome {
                            exit_code,
                            tests_run,
                            tests_failed,
                            stdout: state.stdout.clone(),
                            stdout_bytes: state.stdout_bytes.clone(),
                            stderr: state.stderr.clone(),
                            coverage_hits: state.coverage_hits.iter().copied().collect(),
                            runtime_profiles: normalized_runtime_profiles.clone(),
                            host_contract: self.host_contract(),
                            runtime_backend: self.runtime_backend(),
                            thread_topology: state.thread_topology_snapshot(),
                            trap: None,
                        });
                    }
                    let rendered = diagnostic.to_string();
                    store.data_mut().stderr.push_str(&rendered);
                    store.data_mut().stderr.push('\n');
                    tests_failed += 1;
                }
            }

            if let Err(diagnostic) = drain_event_loop(&instance, &mut store) {
                if let Some(exit_code) = store.data_mut().take_pending_exit_code() {
                    let state = store.data();
                    return Ok(RuntimeOutcome {
                        exit_code,
                        tests_run,
                        tests_failed,
                        stdout: state.stdout.clone(),
                        stdout_bytes: state.stdout_bytes.clone(),
                        stderr: state.stderr.clone(),
                        coverage_hits: state.coverage_hits.iter().copied().collect(),
                        runtime_profiles: normalized_runtime_profiles.clone(),
                        host_contract: self.host_contract(),
                        runtime_backend: self.runtime_backend(),
                        thread_topology: state.thread_topology_snapshot(),
                        trap: None,
                    });
                }
                return Err(vec![diagnostic]);
            }
        }

        let state = store.data();
        Ok(RuntimeOutcome {
            exit_code: if tests_failed == 0 { 0 } else { 1 },
            tests_run,
            tests_failed,
            stdout: state.stdout.clone(),
            stdout_bytes: state.stdout_bytes.clone(),
            stderr: state.stderr.clone(),
            coverage_hits: state.coverage_hits.iter().copied().collect(),
            runtime_profiles: normalized_runtime_profiles,
            host_contract: self.host_contract(),
            runtime_backend: self.runtime_backend(),
            thread_topology: state.thread_topology_snapshot(),
            trap: None,
        })
    }
}

/// Failed-test accounting for a browser-harness `kali test` run. The JS
/// harness's per-callback try/catch feeds `testsFailed` through the summary
/// file, but a test body that executes inline during `_start` (the flattened
/// callback lane) traps OUTSIDE that try/catch and kills the harness process
/// before the summary is emitted, reporting zero failures. A non-zero harness
/// exit with no failure otherwise accounted must count as (at least) one
/// failed test — otherwise `kali test` reports a crashed run as passing.
/// (throw-fallout Stage 0: harness trap-swallow, crash lane.)
fn browser_tests_failed(
    reported_tests_failed: usize,
    run_registered_tests: bool,
    harness_status_success: bool,
) -> usize {
    if run_registered_tests && !harness_status_success && reported_tests_failed == 0 {
        1
    } else {
        reported_tests_failed
    }
}

pub(crate) fn execute_browser_runtime(
    runtime: &RuntimeCtx,
    wasm_bytes: &[u8],
    run_registered_tests: bool,
    normalized_runtime_profiles: Vec<String>,
    browser_harness_command: &str,
) -> Result<RuntimeOutcome, Vec<Diagnostic>> {
    let outcome = browser_runtime_execute_checked(
        Some(browser_harness_command),
        wasm_bytes,
        &runtime.args,
        &runtime.cwd,
        run_registered_tests,
    )
    .map_err(|error| {
        vec![runtime_error_diagnostic(format!(
            "failed to execute browser runtime harness: {}",
            error
        ))]
    })?;

    let tests_run = if run_registered_tests {
        outcome.tests_run().max(1)
    } else {
        0
    };

    Ok(RuntimeOutcome {
        exit_code: if outcome.status.success() {
            0
        } else {
            outcome.status.code().unwrap_or(1)
        },
        tests_run,
        tests_failed: browser_tests_failed(
            outcome.tests_failed,
            run_registered_tests,
            outcome.status.success(),
        ),
        stdout: outcome.stdout,
        // The browser harness does not (yet) plumb a raw binary-stdout sink;
        // `Kali.writeStdoutBytes` is host-only for this task (see spec follow-up).
        stdout_bytes: Vec::new(),
        stderr: outcome.stderr,
        coverage_hits: outcome.coverage_hits,
        runtime_profiles: normalized_runtime_profiles,
        host_contract: outcome.host_contract,
        runtime_backend: outcome.runtime_backend,
        thread_topology: outcome.thread_topology,
        trap: None,
    })
}

#[cfg(test)]
#[path = "execute_tests.rs"]
mod execute_tests;
