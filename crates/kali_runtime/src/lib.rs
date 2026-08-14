//! Runtime execution for Kali-generated WASM modules.

mod ctx;
pub use ctx::RuntimeCtx;
pub(crate) use ctx::*;
mod outcome;
pub use outcome::RuntimeOutcome;
mod state;
// Explicit, like every other re-export in this crate. A glob here hid which
// of `kali_runtime_contract`'s names this crate actually depends on, and
// silently absorbed anything the contract crate later added -- including a
// name that would shadow a local one.
pub(crate) use kali_runtime_contract::{
    browser_harness_uses_html_entrypoint, parse_optional_runtime_backend_label,
    parse_optional_runtime_host_contract_label, BROWSER_HARNESS_SUMMARY_FILE_ENV,
};
pub use kali_runtime_contract::{normalize_runtime_profiles, RuntimeBackend, RuntimeHostContract};
pub use state::{KaliHostState, ScheduledTimer};
mod host;
pub(crate) use host::{diagnostics::*, enforce::*, io::*, memory::*};
pub(crate) use host::{imports_default::*, imports_node::*};
mod browser;
pub(crate) use browser::summary::*;
mod execute;
pub use browser::execute::{
    browser_bundle_runtime_execute_checked, browser_harness_invocation_checked,
    browser_harness_run_checked, browser_harness_run_checked_with_env,
    browser_runtime_execute_checked, BrowserHarnessError, BrowserHarnessInvocation,
    BrowserHarnessOutcome, BrowserRuntimeExecutionOutcome,
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
#[cfg(test)]
use kali_error::DiagnosticContext;
use kali_error::{
    _error_codes::{e4, e5},
    Diagnostic, DiagnosticContextOrigin,
};
pub use kali_runtime_contract::{
    browser_bundle_harness_page, browser_bundle_harness_prelude, browser_bundle_harness_script,
    browser_bundle_runtime_harness_module_script, browser_bundle_runtime_harness_page,
    browser_bundle_runtime_harness_script, browser_runtime_harness_page,
    browser_runtime_harness_script, BROWSER_HARNESS_DONE_BINDING,
};
pub use kali_runtime_contract::{
    browser_harness_command_parts, browser_harness_command_parts_checked,
    browser_harness_command_parts_for, split_command_spec,
};
pub use kali_runtime_contract::{
    browser_runtime_contract_value, browser_runtime_request_context,
    browser_runtime_unavailable_diagnostic, BrowserRuntimeContract,
    BrowserRuntimeContractDescriptor, BROWSER_HARNESS_COMMAND_ENV,
};
use kali_sandbox::{HostOperation, SandboxPolicy};
use reqwest::blocking;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    process::Command,
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
