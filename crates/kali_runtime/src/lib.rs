//! Runtime execution for Kali-generated WASM modules.

mod ctx;
pub(crate) use ctx::*;
pub use ctx::RuntimeCtx;
mod outcome;
pub(crate) use outcome::*;
pub use outcome::RuntimeOutcome;
mod state;
pub(crate) use state::*;
pub use state::{KaliHostState, ScheduledTimer};
mod profiles;
pub(crate) use profiles::*;
pub use profiles::{normalize_runtime_profiles, RuntimeBackend, RuntimeHostContract};
mod host;
pub(crate) use host::{diagnostics::*, enforce::*, io::*, memory::*};
pub(crate) use host::{imports_default::*, imports_node::*};

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use kali_api_node::{
    NodeAssert, NodeBuffer, NodeChildProcess, NodeCrypto, NodePath, NodeRuntimeProjection, NodeUrl,
    NodeUtil,
};
use kali_api_web::{
    fill_random_values, performance_now, random_uuid, ThreadRuntimeInstanceSnapshot,
    ThreadRuntimeShutdownReport, ThreadRuntimeTopology,
};
use kali_error::{
    _error_codes::{e4, e5},
    Diagnostic, DiagnosticContext, DiagnosticContextOrigin,
};
use kali_sandbox::{HostOperation, SandboxPolicy};
use reqwest::blocking;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};
use tempfile::tempdir;
use url::Url;
use wasmtime::{
    Caller, Config, Engine, Extern, Instance, Linker, Memory, Module, Store, StoreLimitsBuilder,
};


/// Canonical metadata for the later standalone browser runtime contract.
///
/// The contract is intentionally declarative for now: it documents the intended
/// execution surface without claiming the runtime itself is available yet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrowserRuntimeContract;

/// Structured descriptor for the later standalone browser runtime contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrowserRuntimeContractDescriptor {
    /// Canonical host-contract label used in diagnostics.
    pub host_label: &'static str,
    /// High-level description of the intended browser host.
    pub host_description: &'static str,
    /// Stable note that names the intended browser host.
    pub host_description_note: &'static str,
    /// Future browser runtime command names.
    pub supported_commands: &'static [&'static str],
    /// Stable note that names the browser runtime command family.
    pub supported_commands_note: &'static str,
    /// Diagnostic hint that points users back to the browser-targeted analysis/build lane.
    pub diagnostic_hint: &'static str,
    /// Stable note that summarizes the later browser runtime contract.
    pub summary_note: &'static str,
    /// Stable note that summarizes the future browser runtime contract scope.
    pub contract_scope_note: &'static str,
}

pub(crate) fn browser_runtime_contract_descriptor_is_canonical(
    descriptor: &BrowserRuntimeContractDescriptor,
) -> bool {
    let trimmed = |value: &str| !value.trim().is_empty() && value.trim() == value;
    let unique = |values: &[&str]| {
        let mut seen = BTreeSet::new();
        !values.is_empty()
            && values
                .iter()
                .copied()
                .all(|value| trimmed(value) && seen.insert(value))
    };

    [
        descriptor.host_label,
        descriptor.host_description,
        descriptor.host_description_note,
        descriptor.supported_commands_note,
        descriptor.diagnostic_hint,
        descriptor.summary_note,
        descriptor.contract_scope_note,
    ]
    .into_iter()
    .all(trimmed)
        && unique(descriptor.supported_commands)
        && unique(BrowserRuntimeContract::diagnostic_notes())
}

/// Canonical JSON fixture for the later standalone browser runtime contract.
pub fn browser_runtime_contract_value() -> serde_json::Value {
    let descriptor = BrowserRuntimeContract::descriptor();
    assert!(
        browser_runtime_contract_descriptor_is_canonical(&descriptor),
        "browser runtime contract descriptor must stay canonical"
    );

    serde_json::json!({
        "hostLabel": descriptor.host_label,
        "hostDescription": descriptor.host_description,
        "hostDescriptionNote": descriptor.host_description_note,
        "supportedCommands": descriptor.supported_commands,
        "diagnosticHint": descriptor.diagnostic_hint,
        "summaryNote": descriptor.summary_note,
        "contractScopeNote": descriptor.contract_scope_note,
        "diagnosticNotes": BrowserRuntimeContract::diagnostic_notes(),
    })
}

/// Environment variable used to override the browser harness command.
pub const BROWSER_HARNESS_COMMAND_ENV: &str = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND";

/// Environment variable used to request deterministic browser-harness summary capture.
pub(crate) const BROWSER_HARNESS_SUMMARY_FILE_ENV: &str = "KALI_BROWSER_HARNESS_SUMMARY_FILE";

impl BrowserRuntimeContract {
    /// The command family the future browser runtime contract will own.
    pub const SUPPORTED_COMMANDS: [&'static str; 2] = ["run", "test"];

    /// Canonical diagnostic notes for the browser runtime contract.
    pub const DIAGNOSTIC_NOTES: [&'static str; 5] = [
        Self::supported_commands_note(),
        Self::summary_note(),
        Self::contract_scope_note(),
        Self::summary_file_fallback_note(),
        Self::host_description_note(),
    ];

    /// Return the canonical host-contract label used in diagnostics.
    pub const fn host_label() -> &'static str {
        RuntimeHostContract::BrowserRequested.canonical_label()
    }

    /// Return the high-level host description for the future browser runtime.
    pub const fn host_description() -> &'static str {
        "real browser host"
    }

    /// Return the future browser runtime contract's supported command names.
    pub const fn supported_commands() -> &'static [&'static str] {
        &Self::SUPPORTED_COMMANDS
    }

    /// Return a canonical ordered list of the browser runtime contract notes.
    pub const fn diagnostic_notes() -> &'static [&'static str] {
        &Self::DIAGNOSTIC_NOTES
    }

    /// Return a structured descriptor for the browser runtime contract.
    pub const fn descriptor() -> BrowserRuntimeContractDescriptor {
        BrowserRuntimeContractDescriptor {
            host_label: Self::host_label(),
            host_description: Self::host_description(),
            host_description_note: Self::host_description_note(),
            supported_commands: Self::supported_commands(),
            supported_commands_note: Self::supported_commands_note(),
            diagnostic_hint: Self::diagnostic_hint(),
            summary_note: Self::summary_note(),
            contract_scope_note: Self::contract_scope_note(),
        }
    }

    /// Return a stable note that names the browser runtime command family.
    pub const fn supported_commands_note() -> &'static str {
        "supported browser runtime commands: run, test"
    }

    /// Return the browser-runtime request diagnostic hint.
    pub const fn diagnostic_hint() -> &'static str {
        "Use the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) for browser-targeted analysis/build work."
    }

    /// Return a stable note that names the intended browser runtime host.
    pub const fn host_description_note() -> &'static str {
        "browser runtime host description: real browser host"
    }

    /// Return a stable note that summarizes the later browser runtime contract.
    pub const fn summary_note() -> &'static str {
        "browser runtime contract summary: run and test remain later-compatibility commands; use the Phase-1 browser-targeted check/build lane for browser-facing analysis/build work"
    }

    /// Return a stable note that summarizes the future browser runtime contract scope.
    pub const fn contract_scope_note() -> &'static str {
        "browser runtime contract scope: run and test only; entrypoints, stdout/stderr capture, and exit status are mapped by the future browser harness"
    }

    /// Return a stable note that describes browser-harness summary fallback behavior.
    pub const fn summary_file_fallback_note() -> &'static str {
        "browser runtime summary fallback: stdout wins when the configured browser harness summary file is missing, unparseable, unreadable, whitespace-only, or shape-invalid"
    }
}

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
            if let Some(exit_code) = store.data_mut().take_pending_exit_code() {
                let state = store.data();
                return Ok(RuntimeOutcome {
                    exit_code,
                    tests_run: 0,
                    tests_failed: 0,
                    stdout: state.stdout.clone(),
                    stderr: state.stderr.clone(),
                    coverage_hits: state.coverage_hits.iter().copied().collect(),
                    runtime_profiles: normalized_runtime_profiles.clone(),
                    host_contract: self.host_contract(),
                    runtime_backend: self.runtime_backend(),
                    thread_topology: state.thread_topology_snapshot(),
                });
            }
            if let Some(diagnostic) = store.data_mut().pending_diagnostic.take() {
                return Err(vec![diagnostic]);
            }
            return Err(vec![runtime_error_diagnostic(format!(
                "runtime trap: {}",
                error
            ))]);
        }

        if let Err(diagnostic) = drain_event_loop(&instance, &mut store) {
            if let Some(exit_code) = store.data_mut().take_pending_exit_code() {
                let state = store.data();
                return Ok(RuntimeOutcome {
                    exit_code,
                    tests_run: 0,
                    tests_failed: 0,
                    stdout: state.stdout.clone(),
                    stderr: state.stderr.clone(),
                    coverage_hits: state.coverage_hits.iter().copied().collect(),
                    runtime_profiles: normalized_runtime_profiles.clone(),
                    host_contract: self.host_contract(),
                    runtime_backend: self.runtime_backend(),
                    thread_topology: state.thread_topology_snapshot(),
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
                stderr: state.stderr.clone(),
                coverage_hits: state.coverage_hits.iter().copied().collect(),
                runtime_profiles: normalized_runtime_profiles.clone(),
                host_contract: self.host_contract(),
                runtime_backend: self.runtime_backend(),
                thread_topology: state.thread_topology_snapshot(),
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
                runtime_profiles: normalized_runtime_profiles.clone(),
                host_contract: self.host_contract(),
                runtime_backend: self.runtime_backend(),
                thread_topology: state.thread_topology_snapshot(),
            });
        }

        let mut tests_run = 0usize;
        let mut tests_failed = 0usize;
        for callback_id in registered_tests {
            tests_run += 1;
            match invoke_callback(&instance, &mut store, callback_id) {
                Ok(()) => {}
                Err(diagnostic) => {
                    if let Some(exit_code) = store.data_mut().take_pending_exit_code() {
                        let state = store.data();
                        return Ok(RuntimeOutcome {
                            exit_code,
                            tests_run,
                            tests_failed,
                            stdout: state.stdout.clone(),
                            stderr: state.stderr.clone(),
                            coverage_hits: state.coverage_hits.iter().copied().collect(),
                            runtime_profiles: normalized_runtime_profiles.clone(),
                            host_contract: self.host_contract(),
                            runtime_backend: self.runtime_backend(),
                            thread_topology: state.thread_topology_snapshot(),
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
                        stderr: state.stderr.clone(),
                        coverage_hits: state.coverage_hits.iter().copied().collect(),
                        runtime_profiles: normalized_runtime_profiles.clone(),
                        host_contract: self.host_contract(),
                        runtime_backend: self.runtime_backend(),
                        thread_topology: state.thread_topology_snapshot(),
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
            stderr: state.stderr.clone(),
            coverage_hits: state.coverage_hits.iter().copied().collect(),
            runtime_profiles: normalized_runtime_profiles,
            host_contract: self.host_contract(),
            runtime_backend: self.runtime_backend(),
            thread_topology: state.thread_topology_snapshot(),
        })
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
        tests_failed: outcome.tests_failed,
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        coverage_hits: Vec::new(),
        runtime_profiles: normalized_runtime_profiles,
        host_contract: outcome.host_contract,
        runtime_backend: outcome.runtime_backend,
        thread_topology: outcome.thread_topology,
    })
}

pub fn browser_runtime_unavailable_diagnostic(
    command: Option<&str>,
    context: Option<DiagnosticContext>,
) -> Diagnostic {
    let browser_contract = BrowserRuntimeContract::descriptor();
    let hint = browser_contract.diagnostic_hint;
    let contract = browser_contract.host_label;
    let message = match command {
        Some(command) => format!(
            "{command} does not support the browser API surface in this phase; Kali does not yet define a standalone browser runtime contract (selected host contract: {contract}). {hint}"
        ),
        None => format!(
            "browser API surface is not available in the current runtime contract (selected host contract: {contract}); Kali does not yet define a standalone browser runtime contract. {hint}"
        ),
    };
    let mut diagnostic = Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message)
        .note(format!("selected host contract: {contract}"))
        .note(format!(
            "current runtime backend: {}",
            RuntimeBackend::Wasmtime.canonical_label()
        ))
        .note(format!(
            "browser harness opt-in env var: {}",
            BROWSER_HARNESS_COMMAND_ENV
        ));
    for note in BrowserRuntimeContract::diagnostic_notes() {
        diagnostic = diagnostic.note(*note);
    }
    if let Some(context) = context {
        diagnostic = diagnostic.with_context(context);
    }
    diagnostic
}

pub fn browser_runtime_request_context(origin: DiagnosticContextOrigin) -> DiagnosticContext {
    DiagnosticContext::new(origin)
        .with_requested_value("browser")
        .with_effective_value("browser")
}

/// Split an argv-style command specification into deterministic tokens.
///
/// The parser accepts the small shell-like subset used by browser harness
/// overrides: whitespace separates tokens, single and double quotes group
/// whitespace, and backslashes escape the next character outside single quotes.
/// The function returns `None` for malformed input such as unterminated quotes,
/// a dangling escape, or an empty or whitespace-only command string.
pub fn split_command_spec(command: &str) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut token_open = false;
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut escaped = false;

    for ch in command.chars() {
        if escaped {
            current.push(ch);
            token_open = true;
            escaped = false;
            continue;
        }

        match ch {
            '\\' if !in_single_quotes => {
                escaped = true;
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                token_open = true;
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                token_open = true;
            }
            ch if ch.is_whitespace() && !in_single_quotes && !in_double_quotes => {
                if token_open {
                    parts.push(std::mem::take(&mut current));
                    token_open = false;
                }
            }
            ch => {
                current.push(ch);
                token_open = true;
            }
        }
    }

    if escaped || in_single_quotes || in_double_quotes {
        return None;
    }

    if token_open {
        parts.push(current);
    }

    if parts.is_empty() || parts.first().is_some_and(|part| part.is_empty()) {
        return None;
    }

    Some(parts)
}

pub(crate) fn browser_harness_normalized_executable_name(executable: &str) -> String {
    let executable = Path::new(executable)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(executable)
        .to_ascii_lowercase();

    let mut normalized = executable;
    loop {
        let next = normalized
            .strip_suffix(".desktop")
            .or_else(|| normalized.strip_suffix(".app"))
            .or_else(|| normalized.strip_suffix(".command"))
            .or_else(|| normalized.strip_suffix(".lnk"))
            .or_else(|| normalized.strip_suffix(".exe"))
            .or_else(|| normalized.strip_suffix(".cmd"))
            .or_else(|| normalized.strip_suffix(".bat"))
            .or_else(|| normalized.strip_suffix(".com"))
            .or_else(|| normalized.strip_suffix(".ps1"))
            .or_else(|| normalized.strip_suffix(".url"));
        match next {
            Some(next) => normalized = next.to_string(),
            None => return normalized,
        }
    }
}

pub(crate) const BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES: &[&str] = &[
    "chrome",
    "chrome-beta",
    "chrome-canary",
    "chrome-headless-shell",
    "chrome-unstable",
    "chrome-dev",
    "chrome-for-testing",
    "chrome for testing",
    "chromium",
    "chromium-browser",
    "chromium-headless-shell",
    "chromium-dev",
    "chromium-for-testing",
    "chromium for testing",
    "google-chrome",
    "google-chrome-headless-shell",
    "google-chrome-stable",
    "google-chrome-beta",
    "google-chrome-canary",
    "google-chrome-unstable",
    "google-chrome-dev",
    "google-chrome-for-testing",
    "google chrome",
    "google chrome stable",
    "google chrome beta",
    "google chrome canary",
    "google chrome unstable",
    "google chrome dev",
    "google chrome for testing",
    "brave",
    "brave-browser",
    "brave-browser-stable",
    "brave browser stable",
    "brave-browser-beta",
    "brave-browser-dev",
    "brave-browser-nightly",
    "brave browser",
    "brave browser beta",
    "brave browser dev",
    "brave browser nightly",
    "vivaldi",
    "vivaldi-stable",
    "vivaldi-snapshot",
    "vivaldi snapshot",
    "opera",
    "opera-stable",
    "opera-beta",
    "opera-developer",
    "opera-unstable",
    "opera beta",
    "opera developer",
    "opera unstable",
    "msedge",
    "msedge-stable",
    "msedge-beta",
    "msedge-canary",
    "msedge-dev",
    "msedge-insider",
    "edge",
    "edge-stable",
    "edge-beta",
    "edge-canary",
    "edge-dev",
    "edge-insider",
    "microsoft-edge",
    "microsoft-edge-stable",
    "microsoft-edge-beta",
    "microsoft-edge-canary",
    "microsoft-edge-dev",
    "microsoft-edge-insider",
    "microsoft edge",
    "microsoft edge stable",
    "microsoft edge beta",
    "microsoft edge canary",
    "microsoft edge dev",
    "microsoft edge insider",
    "firefox",
    "firefox-esr",
    "firefox-beta",
    "firefox-nightly",
    "firefox-developer-edition",
    "firefox developer edition",
    "firefox beta",
    "librewolf",
    "waterfox",
    "mullvad-browser",
    "mullvad browser",
    "privacy-browser",
    "privacy browser",
    "zen-browser",
    "zen browser",
    "thorium-browser",
    "thorium browser",
];

pub(crate) fn browser_harness_is_browser_executable_name(executable: &str) -> bool {
    BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES.contains(&executable)
}

pub(crate) fn browser_harness_command_parts_for_browser_executable(executable: &str) -> Option<Vec<String>> {
    let executable = browser_harness_normalized_executable_name(executable);

    if browser_harness_is_browser_executable_name(&executable) {
        Some(vec![executable, "--headless".to_string()])
    } else {
        None
    }
}

pub(crate) fn browser_harness_default_browser_command_parts() -> Option<Vec<String>> {
    for candidate in BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES {
        if Command::new(candidate).arg("--version").output().is_ok() {
            if let Some(parts) = browser_harness_command_parts_for_browser_executable(candidate) {
                return Some(parts);
            }
        }
    }

    None
}

pub(crate) fn browser_harness_default_command_parts() -> Vec<String> {
    static BROWSER_HARNESS_COMMAND: OnceLock<Vec<String>> = OnceLock::new();
    BROWSER_HARNESS_COMMAND
        .get_or_init(|| {
            if let Some(parts) = browser_harness_default_browser_command_parts() {
                return parts;
            }
            if Command::new("bun").arg("--version").output().is_ok() {
                vec!["bun".to_string()]
            } else {
                vec!["node".to_string()]
            }
        })
        .clone()
}

/// Return the command used by browser smoke or future browser-runtime harnesses.
///
/// The helper accepts the same argv-style shell subset as [`split_command_spec`]
/// and falls back to the deterministic default host command when no override is
/// supplied.
pub fn browser_harness_command_parts_checked(command: Option<&str>) -> Result<Vec<String>, String> {
    if let Some(command) = command {
        let raw_command = command;
        let command = command.trim();
        if command.is_empty() {
            return Err(format!(
                "malformed {BROWSER_HARNESS_COMMAND_ENV} override: {raw_command:?}"
            ));
        }
        match split_command_spec(command) {
            Some(parts) if !parts.is_empty() => {
                if parts.first().is_some_and(|part| part.starts_with('-')) {
                    return Err(format!(
                        "malformed {BROWSER_HARNESS_COMMAND_ENV} override: {raw_command:?}"
                    ));
                }
                return Ok(parts);
            }
            _ => {
                return Err(format!(
                    "malformed {BROWSER_HARNESS_COMMAND_ENV} override: {raw_command:?}"
                ));
            }
        }
    }

    Ok(browser_harness_default_command_parts())
}

/// Return the command used by browser smoke or future browser-runtime harnesses.
///
/// This convenience wrapper preserves the historical infallible shape for tests
/// and other call sites that expect a guaranteed command vector.
pub fn browser_harness_command_parts_for(command: Option<&str>) -> Vec<String> {
    browser_harness_command_parts_checked(command).unwrap_or_else(|error| panic!("{error}"))
}

/// Build the shared browser-bundle smoke harness prelude.
///
/// The generated snippet installs a deterministic `fetch` shim that can resolve the
/// emitted `.wasm` file alongside the bundle glue, so higher-level browser-harness
/// callers only need to append the command-specific body that exercises the exports.
pub fn browser_bundle_harness_prelude(bundle_dir: &str, allow_subpaths: bool) -> String {
    if allow_subpaths {
        format!(
            r#"import fs from 'node:fs/promises';
import {{ fileURLToPath }} from 'node:url';

const bundleJs = new URL('./{bundle_dir}/{bundle_dir}.js', import.meta.url);
const bundleRoot = new URL('./{bundle_dir}/', import.meta.url);

globalThis.fetch = async (input) => {{
  const url = input instanceof URL ? input : new URL(String(input));
  if (url.href.startsWith(bundleRoot.href) && url.pathname.endsWith('.wasm')) {{
    const bytes = await fs.readFile(fileURLToPath(url));
    return new Response(bytes, {{ headers: {{ 'content-type': 'application/wasm' }} }});
  }}
  throw new Error(`unexpected fetch ${{String(input)}}`);
}};

"#,
            bundle_dir = bundle_dir,
        )
    } else {
        format!(
            r#"import fs from 'node:fs/promises';
import {{ fileURLToPath }} from 'node:url';

const bundleJs = new URL('./{bundle_dir}/{bundle_dir}.js', import.meta.url);
const wasmUrl = new URL('./{bundle_dir}/{bundle_dir}.wasm', import.meta.url);

globalThis.fetch = async (input) => {{
  const url = input instanceof URL ? input : new URL(String(input));
  if (url.href === wasmUrl.href) {{
    const bytes = await fs.readFile(fileURLToPath(url));
    return new Response(bytes, {{ headers: {{ 'content-type': 'application/wasm' }} }});
  }}
  throw new Error(`unexpected fetch ${{String(input)}}`);
}};

"#,
            bundle_dir = bundle_dir,
        )
    }
}

/// Build a complete browser-bundle harness script from the shared prelude and a body snippet.
pub fn browser_bundle_harness_script(bundle_dir: &str, allow_subpaths: bool, body: &str) -> String {
    format!(
        "{}{}",
        browser_bundle_harness_prelude(bundle_dir, allow_subpaths),
        body
    )
}

/// Build a browser-bundle runtime harness module that loads the emitted bundle glue.
///
/// The generated module reuses the shared browser-bundle fetch shim, imports the emitted bundle,
/// and re-instantiates it with the canonical Kali runtime imports so future browser runtime
/// flows can observe console output and registered tests from the browser-targeted artifact set.
pub fn browser_bundle_runtime_harness_module_script(
    bundle_dir: &str,
    allow_subpaths: bool,
    args: &[String],
    run_registered_tests: bool,
) -> String {
    let args_json = serde_json::to_string(args).expect("serialize browser bundle runtime args");
    format!(
        r#"{}const runtimeArgs = {args_json};
const runRegisteredTests = {run_registered_tests};
let wasmMemory = null;
const collectedTests = [];
let registeredTestFailures = 0;

const NULL_VALUE_TAG = -9223372036854775808n;
const UNDEFINED_VALUE_TAG = -9223372036854775807n;
function formatConsoleValue(val) {{
  if (typeof val === 'bigint') {{
    if (val === NULL_VALUE_TAG) {{
      return 'null';
    }}
    if (val === UNDEFINED_VALUE_TAG) {{
      return 'undefined';
    }}
    if ((val & 0x8000000000000000n) !== 0n && wasmMemory !== null) {{
      const offset = Number((val >> 32n) & 0x7fffffffn);
      const length = Number(val & 0xffffffffn);
      if (offset >= 0 && length >= 0 && offset + length <= wasmMemory.buffer.byteLength) {{
        const bytes = new Uint8Array(wasmMemory.buffer, offset, length);
        return new TextDecoder().decode(bytes);
      }}
    }}
    return val.toString();
  }}
  return String(val);
}}

const summaryFile = globalThis.process?.env?.["KALI_BROWSER_HARNESS_SUMMARY_FILE"]
  ?? globalThis.Deno?.env?.get?.("KALI_BROWSER_HARNESS_SUMMARY_FILE")
  ?? null;

async function emitBrowserRuntimeSummary(summary) {{
  const serialized = JSON.stringify(summary);
  if (summaryFile !== null) {{
    if (globalThis.Deno?.writeTextFile) {{
      await globalThis.Deno.writeTextFile(summaryFile, serialized);
      return;
    }}
    if (globalThis.process?.versions?.node !== undefined) {{
      const fs = await import('node:fs/promises');
      await fs.writeFile(summaryFile, serialized);
      return;
    }}
  }}
  console.log(serialized);
}}

const importObject = {{
  "kali:rt": {{
    test_register(val) {{
      collectedTests.push(formatConsoleValue(val));
    }},
    args_len() {{
      return runtimeArgs.length;
    }},
    process_pid() {{
      return Number(globalThis.process?.pid ?? 0);
    }},
    cwd(_pathPtr, _pathLen, _outPtr, _outCap) {{
      return 0;
    }},
    math_max(left, right) {{
      return left > right ? left : right;
    }},
    math_min(left, right) {{
      return left < right ? left : right;
    }},
    math_abs(value) {{
      return value < 0n ? -value : value;
    }},
    math_sign(value) {{
      if (value === 0n) {{
        return 0n;
      }}
      return value < 0n ? -1n : 1n;
    }},
    math_round(value) {{
      return value;
    }},
    math_imul(left, right) {{
      return BigInt.asIntN(32, left * right);
    }},
    math_clz32(value) {{
      return BigInt(Math.clz32(Number(BigInt.asUintN(32, value))));
    }},
    math_pow(left, right) {{
      if (right < 0n) {{
        if (left === 1n) {{
          return 1n;
        }}
        if (left === -1n) {{
          return right % 2n === 0n ? 1n : -1n;
        }}
        throw new Error('Math.pow negative exponents are unavailable unless the base is a statically-known ±1 in the current phase; use a non-negative exponent or the later compatibility path');
      }}
      return BigInt.asIntN(64, left ** right);
    }},
    console_log(val) {{
      console.log(formatConsoleValue(val));
    }},
    console_error(val) {{
      console.error(formatConsoleValue(val));
    }},
    console_warn(val) {{
      console.warn(formatConsoleValue(val));
    }},
    console_info(val) {{
      if (typeof console !== 'undefined' && typeof console.info === 'function') {{
        console.info(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
    console_debug(val) {{
      if (typeof console !== 'undefined' && typeof console.debug === 'function') {{
        console.debug(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
  }},
}};

const bundle = await import(bundleJs.href);
if (typeof bundle.loadWithImports !== 'function') {{
  throw new Error('missing loadWithImports helper');
}}
const instance = await bundle.loadWithImports(importObject);
wasmMemory = instance.exports.memory ?? null;
if (typeof instance.exports._start === 'function') {{
  await instance.exports._start();
}}
if (runRegisteredTests) {{
  for (const callbackId of collectedTests) {{
    const callbackName = `__kali_callback_${{callbackId}}`;
    const callback = instance.exports[callbackName];
    if (typeof callback !== 'function') {{
      throw new Error(`missing browser runtime test callback: ${{callbackName}}`);
    }}
    try {{
      await callback();
    }} catch (error) {{
      registeredTestFailures += 1;
      console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    }}
  }}
}}
let summaryEmissionError = null;
try {{
  await emitBrowserRuntimeSummary({{ args: runtimeArgs, hostContract: "browser-requested", runtimeBackend: "browser-harness", tests: collectedTests, testsFailed: registeredTestFailures }});
}} catch (error) {{
  summaryEmissionError = error;
}}
if (registeredTestFailures > 0) {{
  throw new Error(`browser runtime test failures: ${{registeredTestFailures}}`);
}}
if (summaryEmissionError !== null) {{
  throw summaryEmissionError;
}}
"#,
        browser_bundle_harness_prelude(bundle_dir, allow_subpaths),
    )
}

/// Build a browser-host HTML wrapper for the browser-bundle runtime harness.
pub fn browser_bundle_runtime_harness_page(
    bundle_dir: &str,
    allow_subpaths: bool,
    args: &[String],
    run_registered_tests: bool,
) -> String {
    let module_script = browser_bundle_runtime_harness_module_script(
        bundle_dir,
        allow_subpaths,
        args,
        run_registered_tests,
    );
    format!(
        r#"<!doctype html>
<meta charset="utf-8">
<title>Kali browser bundle runtime harness</title>
<script type="module">
{module_script}
</script>
"#,
        module_script = module_script,
    )
}

/// Build a browser-bundle runtime harness script that loads the emitted bundle glue.
///
/// The generated module reuses the shared browser-bundle fetch shim, imports the emitted bundle,
/// and re-instantiates it with the canonical Kali runtime imports so future browser runtime
/// flows can observe console output and registered tests from the browser-targeted artifact set.
pub fn browser_bundle_runtime_harness_script(
    bundle_dir: &str,
    allow_subpaths: bool,
    args: &[String],
    run_registered_tests: bool,
) -> String {
    browser_bundle_runtime_harness_module_script(
        bundle_dir,
        allow_subpaths,
        args,
        run_registered_tests,
    )
}

/// Execute an emitted browser-targeted bundle through the browser harness.
///
/// The bundle harness is written next to the emitted bundle directory so the shared prelude can
/// resolve the bundle glue with the expected relative layout.
pub fn browser_bundle_runtime_execute_checked(
    command: Option<&str>,
    bundle_root: impl AsRef<Path>,
    args: &[String],
    allow_subpaths: bool,
    run_registered_tests: bool,
) -> Result<BrowserRuntimeExecutionOutcome, BrowserHarnessError> {
    let bundle_root = bundle_root.as_ref();
    let bundle_dir = bundle_root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| BrowserHarnessError::PreparationFailed {
            message: format!(
                "bundle root {:?} does not have a valid directory name",
                bundle_root
            ),
        })?;
    let current_dir =
        bundle_root
            .parent()
            .ok_or_else(|| BrowserHarnessError::PreparationFailed {
                message: format!(
                    "bundle root {:?} does not have a parent directory",
                    bundle_root
                ),
            })?;
    let browser_command = browser_harness_command_parts_checked(command)
        .map_err(|message| BrowserHarnessError::PreparationFailed { message })?;
    let use_html_entrypoint = browser_command
        .first()
        .is_some_and(|executable| browser_harness_uses_html_entrypoint(executable));
    let script_name = if use_html_entrypoint {
        "browser-bundle-runtime.html"
    } else {
        "browser-bundle-runtime.mjs"
    };
    let script_path = current_dir.join(script_name);
    let summary_path = current_dir.join("browser-bundle-runtime-summary.json");
    let script_contents = if use_html_entrypoint {
        browser_bundle_runtime_harness_page(bundle_dir, allow_subpaths, args, run_registered_tests)
    } else {
        browser_bundle_runtime_harness_script(
            bundle_dir,
            allow_subpaths,
            args,
            run_registered_tests,
        )
    };
    fs::write(&script_path, script_contents).map_err(|error| {
        BrowserHarnessError::PreparationFailed {
            message: error.to_string(),
        }
    })?;

    let outcome = browser_harness_run_checked_with_env(
        command,
        &script_path,
        &[],
        current_dir,
        &[(BROWSER_HARNESS_SUMMARY_FILE_ENV, summary_path.as_os_str())],
    )?;
    let summary = browser_runtime_summary_for_outcome(&summary_path, &outcome);

    Ok(BrowserRuntimeExecutionOutcome {
        command: outcome.command,
        status: outcome.status,
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        host_contract: summary
            .host_contract
            .unwrap_or(RuntimeHostContract::BrowserRequested),
        runtime_backend: summary
            .runtime_backend
            .unwrap_or(RuntimeBackend::BrowserHarness),
        reported_args: summary.args,
        registered_tests: summary.tests,
        tests_failed: summary.tests_failed.unwrap_or(0),
        thread_topology: summary.thread_topology.unwrap_or_default(),
    })
}

pub(crate) fn browser_harness_uses_html_entrypoint(executable: &str) -> bool {
    browser_harness_command_parts_for_browser_executable(executable).is_some()
}

pub(crate) fn browser_runtime_harness_module_script(
    wasm_bytes: &[u8],
    args: &[String],
    run_registered_tests: bool,
) -> String {
    let wasm_base64 = BASE64_STANDARD.encode(wasm_bytes);
    let args_json = serde_json::to_string(args).expect("serialize browser runtime args");
    format!(
        r#"const runtimeArgs = {args_json};
const runRegisteredTests = {run_registered_tests};
const runtimeWasm = decodeBase64("{wasm_base64}");
let wasmMemory = null;
const collectedTests = [];
let registeredTestFailures = 0;
let threadTopology = {{
  totalInstances: 0,
  terminatedInstances: 0,
  liveInstances: [],
}};
let nextThreadInstanceId = 0;

function readGuestString(ptr, len) {{
  if (wasmMemory === null) {{
    throw new Error('guest memory is unavailable before thread spawn handling');
  }}
  const bytes = new Uint8Array(wasmMemory.buffer, ptr, len);
  return new TextDecoder().decode(bytes);
}}

function recordThreadInstance(scriptUrlValue) {{
  const trimmedScriptUrl = scriptUrlValue.trim();
  if (trimmedScriptUrl.length === 0 || trimmedScriptUrl !== scriptUrlValue) {{
    throw new Error('browser runtime thread_spawn scriptUrl must be a canonical absolute URL');
  }}
  let parsedScriptUrl;
  try {{
    parsedScriptUrl = new URL(trimmedScriptUrl);
  }} catch {{
    throw new Error('browser runtime thread_spawn scriptUrl must be a canonical absolute URL');
  }}
  if (parsedScriptUrl.href !== trimmedScriptUrl) {{
    throw new Error('browser runtime thread_spawn scriptUrl must be a canonical absolute URL');
  }}
  const instanceId = nextThreadInstanceId++;
  threadTopology.liveInstances.push({{
    instanceId,
    scriptUrl: parsedScriptUrl.href,
    postedMessages: [],
    postedSharedBuffers: [],
    wasTerminated: false,
  }});
  threadTopology.totalInstances =
    threadTopology.terminatedInstances + threadTopology.liveInstances.length;
  return instanceId;
}}

function decodeBase64(base64) {{
  const binary = typeof atob === 'function'
    ? atob(base64)
    : (typeof Buffer !== 'undefined'
        ? Buffer.from(base64, 'base64').toString('binary')
        : (() => {{ throw new Error('base64 decoding is unavailable in this host'); }})());
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {{
    bytes[index] = binary.charCodeAt(index);
  }}
  return bytes;
}}

const NULL_VALUE_TAG = -9223372036854775808n;
const UNDEFINED_VALUE_TAG = -9223372036854775807n;
function formatConsoleValue(val) {{
  if (typeof val === 'bigint') {{
    if (val === NULL_VALUE_TAG) {{
      return 'null';
    }}
    if (val === UNDEFINED_VALUE_TAG) {{
      return 'undefined';
    }}
    if ((val & 0x8000000000000000n) !== 0n && wasmMemory !== null) {{
      const offset = Number((val >> 32n) & 0x7fffffffn);
      const length = Number(val & 0xffffffffn);
      if (offset >= 0 && length >= 0 && offset + length <= wasmMemory.buffer.byteLength) {{
        const bytes = new Uint8Array(wasmMemory.buffer, offset, length);
        return new TextDecoder().decode(bytes);
      }}
    }}
    return val.toString();
  }}
  return String(val);
}}

const summaryFile = globalThis.process?.env?.["KALI_BROWSER_HARNESS_SUMMARY_FILE"]
  ?? globalThis.Deno?.env?.get?.("KALI_BROWSER_HARNESS_SUMMARY_FILE")
  ?? null;

async function emitBrowserRuntimeSummary(summary) {{
  const serialized = JSON.stringify(summary);
  if (summaryFile !== null) {{
    if (globalThis.Deno?.writeTextFile) {{
      await globalThis.Deno.writeTextFile(summaryFile, serialized);
      return;
    }}
    if (globalThis.process?.versions?.node !== undefined) {{
      const fs = await import('node:fs/promises');
      await fs.writeFile(summaryFile, serialized);
      return;
    }}
  }}
  console.log(serialized);
}}

const importObject = {{
  "kali:rt": {{
    test_register(val) {{
      collectedTests.push(formatConsoleValue(val));
    }},
    thread_spawn(scriptUrlPtr, scriptUrlLen) {{
      const scriptUrl = readGuestString(scriptUrlPtr, scriptUrlLen);
      return recordThreadInstance(scriptUrl);
    }},
    args_len() {{
      return runtimeArgs.length;
    }},
    process_pid() {{
      return Number(globalThis.process?.pid ?? 0);
    }},
    cwd(_pathPtr, _pathLen, _outPtr, _outCap) {{
      return 0;
    }},
    math_max(left, right) {{
      return left > right ? left : right;
    }},
    math_min(left, right) {{
      return left < right ? left : right;
    }},
    math_abs(value) {{
      return value < 0n ? -value : value;
    }},
    math_sign(value) {{
      if (value === 0n) {{
        return 0n;
      }}
      return value < 0n ? -1n : 1n;
    }},
    math_round(value) {{
      return value;
    }},
    math_imul(left, right) {{
      return BigInt.asIntN(32, left * right);
    }},
    math_clz32(value) {{
      return BigInt(Math.clz32(Number(BigInt.asUintN(32, value))));
    }},
    math_pow(left, right) {{
      if (right < 0n) {{
        if (left === 1n) {{
          return 1n;
        }}
        if (left === -1n) {{
          return right % 2n === 0n ? 1n : -1n;
        }}
        throw new Error('Math.pow negative exponents are unavailable unless the base is a statically-known ±1 in the current phase; use a non-negative exponent or the later compatibility path');
      }}
      return BigInt.asIntN(64, left ** right);
    }},
    console_log(val) {{
      console.log(formatConsoleValue(val));
    }},
    console_error(val) {{
      console.error(formatConsoleValue(val));
    }},
    console_warn(val) {{
      console.warn(formatConsoleValue(val));
    }},
    console_info(val) {{
      if (typeof console !== 'undefined' && typeof console.info === 'function') {{
        console.info(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
    console_debug(val) {{
      if (typeof console !== 'undefined' && typeof console.debug === 'function') {{
        console.debug(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
  }},
}};

const {{ instance }} = await WebAssembly.instantiate(runtimeWasm, importObject);
wasmMemory = instance.exports.memory ?? null;
if (typeof instance.exports._start === 'function') {{
  await instance.exports._start();
}}
if (runRegisteredTests) {{
  for (const callbackId of collectedTests) {{
    const callbackName = `__kali_callback_${{callbackId}}`;
    const callback = instance.exports[callbackName];
    if (typeof callback !== 'function') {{
      throw new Error(`missing browser runtime test callback: ${{callbackName}}`);
    }}
    try {{
      await callback();
    }} catch (error) {{
      registeredTestFailures += 1;
      console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    }}
  }}
}}
let summaryEmissionError = null;
try {{
  await emitBrowserRuntimeSummary({{ args: runtimeArgs, hostContract: "browser-requested", runtimeBackend: "browser-harness", tests: collectedTests, testsFailed: registeredTestFailures, threadTopology }});
}} catch (error) {{
  summaryEmissionError = error;
}}
if (registeredTestFailures > 0) {{
  throw new Error(`browser runtime test failures: ${{registeredTestFailures}}`);
}}
if (summaryEmissionError !== null) {{
  throw summaryEmissionError;
}}
"#,
        args_json = args_json,
        run_registered_tests = run_registered_tests,
        wasm_base64 = wasm_base64,
    )
}

/// Build a self-contained browser-runtime harness script from embedded WASM bytes.
///
/// The generated module is intentionally generic: it instantiates the supplied WASM bytes, wires
/// the canonical Kali runtime imports for console/argument handling, and optionally emits a simple
/// test summary payload for future browser-runtime test plumbing.
pub fn browser_runtime_harness_script(
    wasm_bytes: &[u8],
    args: &[String],
    run_registered_tests: bool,
) -> String {
    browser_runtime_harness_module_script(wasm_bytes, args, run_registered_tests)
}

/// Build a browser-host HTML wrapper for the self-contained browser-runtime harness.
///
/// This wrapper is intended for real browser hosts that can open an HTML entrypoint while still
/// executing the same browser-friendly module body used by the in-process harness.
pub fn browser_runtime_harness_page(
    wasm_bytes: &[u8],
    args: &[String],
    run_registered_tests: bool,
) -> String {
    let module_script =
        browser_runtime_harness_module_script(wasm_bytes, args, run_registered_tests);
    format!(
        r#"<!doctype html>
<meta charset="utf-8">
<title>Kali browser runtime harness</title>
<script type="module">
{module_script}
</script>
"#,
        module_script = module_script,
    )
}

/// Result of executing a browser-harnessed WASM module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserRuntimeExecutionOutcome {
    /// The fully resolved command line used to launch the harness, including the script path and
    /// any trailing entrypoint arguments.
    pub command: Vec<String>,
    /// The harness process exit status.
    pub status: std::process::ExitStatus,
    /// Captured harness stdout.
    pub stdout: String,
    /// Captured harness stderr.
    pub stderr: String,
    /// The high-level host contract selected for the browser harness request.
    pub host_contract: RuntimeHostContract,
    /// The browser backend reported by the harness summary.
    pub runtime_backend: RuntimeBackend,
    /// Runtime arguments reported by the harness summary.
    pub reported_args: Vec<String>,
    /// Test callbacks registered by the guest and reported by the browser harness summary.
    pub registered_tests: Vec<String>,
    /// Test callbacks that failed inside the browser harness summary.
    pub tests_failed: usize,
    /// Deterministic worker/thread shutdown snapshot reported by the harness summary.
    pub thread_topology: ThreadRuntimeShutdownReport,
}

impl BrowserRuntimeExecutionOutcome {
    /// Return the number of registered guest tests reported by the harness summary.
    pub fn tests_run(&self) -> usize {
        self.registered_tests.len()
    }
}

#[derive(Default)]
pub(crate) struct BrowserRuntimeSummary {
    pub(crate) args: Vec<String>,
    pub(crate) tests: Vec<String>,
    pub(crate) tests_failed: Option<usize>,
    pub(crate) host_contract: Option<RuntimeHostContract>,
    pub(crate) runtime_backend: Option<RuntimeBackend>,
    pub(crate) thread_topology: Option<ThreadRuntimeShutdownReport>,
}


pub(crate) fn parse_non_blank_string_array_field(value: Option<&serde_json::Value>) -> Option<Vec<String>> {
    let items = value?.as_array()?;
    let mut strings = Vec::with_capacity(items.len());
    for item in items {
        let item = item.as_str()?;
        if item.trim().is_empty() || item.trim() != item {
            return None;
        }
        strings.push(item.to_owned());
    }
    Some(strings)
}

pub(crate) fn parse_browser_runtime_summary(stdout: &str) -> BrowserRuntimeSummary {
    parse_browser_runtime_summary_opt(stdout).unwrap_or_default()
}

pub(crate) fn parse_thread_runtime_instance_snapshot_value(
    value: &serde_json::Value,
) -> Option<ThreadRuntimeInstanceSnapshot> {
    let object = value.as_object()?;
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "instanceId" | "scriptUrl" | "postedMessages" | "postedSharedBuffers" | "wasTerminated"
        )
    }) {
        return None;
    }

    let instance_id = object.get("instanceId")?.as_u64()? as usize;
    let script_url = object.get("scriptUrl")?.as_str()?;
    let trimmed_script_url = script_url.trim();
    if trimmed_script_url.is_empty() || trimmed_script_url != script_url {
        return None;
    }
    let parsed_script_url = url::Url::parse(trimmed_script_url).ok()?;
    if parsed_script_url.as_str() != script_url {
        return None;
    }

    let posted_messages = object.get("postedMessages")?.as_array()?.clone();
    let posted_shared_buffers = object
        .get("postedSharedBuffers")?
        .as_array()?
        .iter()
        .map(|buffer| {
            let bytes = buffer.as_array()?;
            let mut output = Vec::with_capacity(bytes.len());
            for byte in bytes {
                let byte = byte.as_u64()?;
                if byte > u8::MAX as u64 {
                    return None;
                }
                output.push(byte as u8);
            }
            Some(output)
        })
        .collect::<Option<Vec<Vec<u8>>>>()?;
    let was_terminated = object.get("wasTerminated")?.as_bool()?;

    Some(ThreadRuntimeInstanceSnapshot {
        instance_id,
        script_url: script_url.to_owned(),
        posted_messages,
        posted_shared_buffers,
        was_terminated,
    })
}

pub(crate) fn parse_thread_runtime_shutdown_report_value(
    value: Option<&serde_json::Value>,
) -> Option<ThreadRuntimeShutdownReport> {
    let value = value?;
    let object = value.as_object()?;
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "totalInstances" | "terminatedInstances" | "liveInstances"
        )
    }) {
        return None;
    }

    let total_instances = object.get("totalInstances")?.as_u64()? as usize;
    let terminated_instances = object.get("terminatedInstances")?.as_u64()? as usize;
    let live_instances = object
        .get("liveInstances")?
        .as_array()?
        .iter()
        .map(parse_thread_runtime_instance_snapshot_value)
        .collect::<Option<Vec<_>>>()?;

    let mut previous_instance_id = None;
    let mut seen_instance_ids = BTreeSet::new();
    for instance in &live_instances {
        if !seen_instance_ids.insert(instance.instance_id) {
            return None;
        }
        if previous_instance_id.is_some_and(|previous| instance.instance_id < previous) {
            return None;
        }
        previous_instance_id = Some(instance.instance_id);
    }

    if total_instances != terminated_instances + live_instances.len() {
        return None;
    }

    Some(ThreadRuntimeShutdownReport {
        total_instances,
        terminated_instances,
        live_instances,
    })
}

pub(crate) fn parse_browser_runtime_summary_value(value: &serde_json::Value) -> Option<BrowserRuntimeSummary> {
    let object = value.as_object()?;
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "args" | "tests" | "testsFailed" | "hostContract" | "runtimeBackend" | "threadTopology"
        )
    }) {
        return None;
    }

    let args = parse_non_blank_string_array_field(object.get("args"))?;
    let tests = parse_non_blank_string_array_field(object.get("tests"))?;
    let tests_failed = match object.get("testsFailed") {
        Some(value) => Some(value.as_u64()? as usize),
        None => None,
    };

    Some(BrowserRuntimeSummary {
        args,
        tests,
        tests_failed,
        host_contract: parse_optional_runtime_host_contract_label(object.get("hostContract")),
        runtime_backend: parse_optional_runtime_backend_label(object.get("runtimeBackend")),
        thread_topology: parse_thread_runtime_shutdown_report_value(object.get("threadTopology")),
    })
}



pub(crate) fn parse_browser_runtime_summary_opt(stdout: &str) -> Option<BrowserRuntimeSummary> {
    stdout.lines().rev().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let value = serde_json::from_str::<serde_json::Value>(trimmed).ok()?;
        parse_browser_runtime_summary_value(&value)
    })
}

pub(crate) fn browser_runtime_summary_for_outcome(
    summary_path: &Path,
    outcome: &BrowserHarnessOutcome,
) -> BrowserRuntimeSummary {
    let stdout_summary = parse_browser_runtime_summary(&outcome.stdout);
    match fs::read_to_string(summary_path) {
        Ok(text) => {
            if text.trim().is_empty() {
                return stdout_summary;
            }

            match parse_browser_runtime_summary_opt(&text) {
                Some(mut summary) => {
                    if summary.tests_failed.is_none() {
                        summary.tests_failed = stdout_summary.tests_failed;
                    }
                    if summary.host_contract.is_none() {
                        summary.host_contract = stdout_summary.host_contract;
                    }
                    if summary.runtime_backend.is_none() {
                        summary.runtime_backend = stdout_summary.runtime_backend;
                    }
                    if summary.thread_topology.is_none() {
                        summary.thread_topology = stdout_summary.thread_topology;
                    }
                    summary
                }
                None => stdout_summary,
            }
        }
        Err(_) => stdout_summary,
    }
}

/// Execute a WASM module through the browser harness and capture the resulting summary.
pub fn browser_runtime_execute_checked(
    command: Option<&str>,
    wasm_bytes: &[u8],
    args: &[String],
    current_dir: impl AsRef<Path>,
    run_registered_tests: bool,
) -> Result<BrowserRuntimeExecutionOutcome, BrowserHarnessError> {
    let tempdir = tempdir().map_err(|error| BrowserHarnessError::PreparationFailed {
        message: error.to_string(),
    })?;
    let browser_command = browser_harness_command_parts_checked(command)
        .map_err(|message| BrowserHarnessError::PreparationFailed { message })?;
    let use_html_entrypoint = browser_command
        .first()
        .is_some_and(|executable| browser_harness_uses_html_entrypoint(executable));
    let script_name = if use_html_entrypoint {
        "browser-runtime.html"
    } else {
        "browser-runtime.mjs"
    };
    let script_path = tempdir.path().join(script_name);
    let summary_path = tempdir.path().join("browser-runtime-summary.json");
    let script_contents = if use_html_entrypoint {
        browser_runtime_harness_page(wasm_bytes, args, run_registered_tests)
    } else {
        browser_runtime_harness_script(wasm_bytes, args, run_registered_tests)
    };
    fs::write(&script_path, script_contents).map_err(|error| {
        BrowserHarnessError::PreparationFailed {
            message: error.to_string(),
        }
    })?;

    let outcome = browser_harness_run_checked_with_env(
        command,
        &script_path,
        &[],
        current_dir,
        &[(BROWSER_HARNESS_SUMMARY_FILE_ENV, summary_path.as_os_str())],
    )?;
    let summary = browser_runtime_summary_for_outcome(&summary_path, &outcome);

    Ok(BrowserRuntimeExecutionOutcome {
        command: outcome.command,
        status: outcome.status,
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        host_contract: summary
            .host_contract
            .unwrap_or(RuntimeHostContract::BrowserRequested),
        runtime_backend: summary
            .runtime_backend
            .unwrap_or(RuntimeBackend::BrowserHarness),
        reported_args: summary.args,
        registered_tests: summary.tests,
        tests_failed: summary.tests_failed.unwrap_or(0),
        thread_topology: summary.thread_topology.unwrap_or_default(),
    })
}

/// A deterministic browser-harness launch plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserHarnessInvocation {
    /// The executable used to launch the harness.
    pub executable: String,
    /// Arguments passed to the harness before the browser script path.
    pub harness_args: Vec<String>,
    /// The script or entrypoint that will be executed by the harness.
    pub script: PathBuf,
    /// Trailing arguments forwarded to the browser script.
    pub args: Vec<String>,
    /// Current working directory for the harness process.
    pub current_dir: PathBuf,
    /// The fully resolved command line used to launch the harness, including the script path and
    /// any trailing entrypoint arguments.
    pub command: Vec<String>,
}

impl BrowserHarnessInvocation {
    /// Launch the browser harness and capture stdout/stderr and exit status.
    pub fn launch(self) -> Result<BrowserHarnessOutcome, BrowserHarnessError> {
        self.launch_with_env(&[])
    }

    /// Launch the browser harness with additional environment variables.
    pub fn launch_with_env(
        self,
        extra_env: &[(&str, &std::ffi::OsStr)],
    ) -> Result<BrowserHarnessOutcome, BrowserHarnessError> {
        let BrowserHarnessInvocation {
            executable,
            harness_args,
            script,
            args,
            current_dir,
            command,
        } = self;

        let mut harness = Command::new(&executable);
        harness.args(&harness_args);
        let script_arg = if browser_harness_uses_html_entrypoint(&executable) {
            Url::from_file_path(&script)
                .map_err(|_| BrowserHarnessError::PreparationFailed {
                    message: format!(
                        "failed to convert browser harness script path {:?} into a file URL",
                        script
                    ),
                })?
                .to_string()
        } else {
            script.to_string_lossy().into_owned()
        };
        harness.arg(&script_arg);
        harness.args(&args);
        harness.current_dir(current_dir);
        for &(key, value) in extra_env {
            harness.env(key, value);
        }

        let output = harness
            .output()
            .map_err(|error| BrowserHarnessError::LaunchFailed {
                executable,
                script: script.clone(),
                command: command.clone(),
                message: error.to_string(),
            })?;

        Ok(BrowserHarnessOutcome {
            command,
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// A deterministic browser-harness execution result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserHarnessOutcome {
    /// The fully resolved command line used to launch the harness, including the script path and
    /// any trailing entrypoint arguments.
    pub command: Vec<String>,
    /// The harness process exit status.
    pub status: std::process::ExitStatus,
    /// Captured harness stdout.
    pub stdout: String,
    /// Captured harness stderr.
    pub stderr: String,
}

/// Error returned when launching a browser harness command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrowserHarnessError {
    /// The configured command override was malformed.
    MalformedOverride {
        /// The environment variable that carried the malformed override.
        env_var: &'static str,
        /// The malformed override value.
        value: String,
    },
    /// Browser-runtime harness preparation failed before launch.
    PreparationFailed {
        /// The preparation error message.
        message: String,
    },
    /// The harness command could not be launched.
    LaunchFailed {
        /// The executable that failed to launch.
        executable: String,
        /// The script or entrypoint that was being executed.
        script: PathBuf,
        /// The fully resolved command line that was being launched.
        command: Vec<String>,
        /// The launch error message.
        message: String,
    },
}

impl std::fmt::Display for BrowserHarnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedOverride { env_var, value } => {
                write!(f, "malformed {env_var} override: {value:?}")
            }
            Self::PreparationFailed { message } => {
                write!(f, "failed to prepare browser harness execution: {message}")
            }
            Self::LaunchFailed {
                executable,
                script,
                command,
                message,
            } => write!(
                f,
                "failed to launch browser harness command {executable:?} for {script:?} with resolved command {command:?}: {message}"
            ),
        }
    }
}

impl std::error::Error for BrowserHarnessError {}

/// Build a browser harness launch plan from the configured environment override.
pub fn browser_harness_invocation_checked(
    command: Option<&str>,
    script: impl AsRef<Path>,
    args: &[String],
    current_dir: impl AsRef<Path>,
) -> Result<BrowserHarnessInvocation, BrowserHarnessError> {
    let mut parts = browser_harness_command_parts_checked(command).map_err(|value| {
        BrowserHarnessError::MalformedOverride {
            env_var: BROWSER_HARNESS_COMMAND_ENV,
            value,
        }
    })?;

    let executable = parts.remove(0);
    let script = script.as_ref().to_path_buf();
    let current_dir = current_dir.as_ref().to_path_buf();
    let mut command = Vec::with_capacity(2 + parts.len() + args.len());
    command.push(executable.clone());
    command.extend(parts.iter().cloned());
    let script_arg = if browser_harness_uses_html_entrypoint(&executable) {
        Url::from_file_path(&script)
            .map_err(|_| BrowserHarnessError::PreparationFailed {
                message: format!(
                    "failed to convert browser harness script path {:?} into a file URL",
                    script
                ),
            })?
            .to_string()
    } else {
        script.to_string_lossy().into_owned()
    };
    command.push(script_arg);
    command.extend(args.iter().cloned());

    Ok(BrowserHarnessInvocation {
        executable,
        harness_args: parts,
        script,
        args: args.to_vec(),
        current_dir,
        command,
    })
}

/// Launch the browser harness command, capturing stdout/stderr and exit status.
pub fn browser_harness_run_checked(
    command: Option<&str>,
    script: impl AsRef<Path>,
    args: &[String],
    current_dir: impl AsRef<Path>,
) -> Result<BrowserHarnessOutcome, BrowserHarnessError> {
    browser_harness_invocation_checked(command, script, args, current_dir)?.launch()
}

/// Launch the browser harness with additional environment variables.
pub fn browser_harness_run_checked_with_env(
    command: Option<&str>,
    script: impl AsRef<Path>,
    args: &[String],
    current_dir: impl AsRef<Path>,
    extra_env: &[(&str, &std::ffi::OsStr)],
) -> Result<BrowserHarnessOutcome, BrowserHarnessError> {
    browser_harness_invocation_checked(command, script, args, current_dir)?
        .launch_with_env(extra_env)
}

/// Return the effective browser harness command using the configured environment override.
pub fn browser_harness_command_parts() -> Vec<String> {
    browser_harness_command_parts_for(std::env::var(BROWSER_HARNESS_COMMAND_ENV).ok().as_deref())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
