//! Runtime execution for Kali-generated WASM modules.

mod ctx;
pub use ctx::RuntimeCtx;
pub(crate) use ctx::*;
mod outcome;
pub use outcome::RuntimeOutcome;
mod state;
pub use state::{KaliHostState, ScheduledTimer};
mod profiles;
pub(crate) use profiles::*;
pub use profiles::{normalize_runtime_profiles, RuntimeBackend, RuntimeHostContract};
mod host;
pub(crate) use host::{diagnostics::*, enforce::*, io::*, memory::*};
pub(crate) use host::{imports_default::*, imports_node::*};
mod browser;
pub(crate) use browser::{command::*, contract::*, summary::*};
mod execute;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
pub use browser::command::{
    browser_harness_command_parts, browser_harness_command_parts_checked,
    browser_harness_command_parts_for, split_command_spec,
};
pub use browser::contract::{
    browser_runtime_contract_value, browser_runtime_request_context,
    browser_runtime_unavailable_diagnostic, BrowserRuntimeContract,
    BrowserRuntimeContractDescriptor, BROWSER_HARNESS_COMMAND_ENV,
};
pub use browser::execute::{
    browser_bundle_runtime_execute_checked, browser_harness_invocation_checked,
    browser_harness_run_checked, browser_harness_run_checked_with_env,
    browser_runtime_execute_checked, BrowserHarnessError, BrowserHarnessInvocation,
    BrowserHarnessOutcome, BrowserRuntimeExecutionOutcome,
};
pub use browser::harness::{
    browser_bundle_harness_page, browser_bundle_harness_prelude, browser_bundle_harness_script,
    browser_bundle_runtime_harness_module_script, browser_bundle_runtime_harness_page,
    browser_bundle_runtime_harness_script, browser_runtime_harness_page,
    browser_runtime_harness_script, BROWSER_HARNESS_DONE_BINDING,
};
#[cfg(test)]
pub(crate) use execute::execute_browser_runtime;
use kali_api_node::{
    NodeAssert, NodeBuffer, NodeChildProcess, NodeCrypto, NodePath, NodeRuntimeProjection, NodeUrl,
    NodeUtil,
};
use kali_api_web::{
    fill_random_values, performance_now, random_uuid, SubtleCrypto, ThreadRuntimeInstanceSnapshot,
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
    Val,
};

#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;
