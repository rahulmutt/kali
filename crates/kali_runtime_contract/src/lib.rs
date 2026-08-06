//! Runtime contract surface without the runtime.
//!
//! Holds the declarative half of `kali_runtime`: host-contract and backend
//! labels, profile normalization, the browser runtime contract, browser harness
//! command resolution, and browser harness script generation. None of it links
//! wasmtime, which is the entire reason this crate exists — 154 `kali_cli`
//! integration test binaries import these items and would otherwise each carry
//! a ~400 MB statically linked wasmtime.
//!
//! See `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`.

mod profiles;
pub use profiles::{
    normalize_runtime_profiles, parse_optional_runtime_backend_label,
    parse_optional_runtime_host_contract_label, RuntimeBackend, RuntimeHostContract,
};

mod browser;
pub use browser::command::{
    browser_harness_command_parts, browser_harness_command_parts_checked,
    browser_harness_command_parts_for, browser_harness_uses_html_entrypoint, split_command_spec,
};
#[cfg(test)]
pub(crate) use browser::command::{
    browser_harness_command_parts_for_browser_executable,
    browser_harness_default_command_parts_from, BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES,
};
#[cfg(test)]
pub(crate) use browser::contract::browser_runtime_contract_descriptor_is_canonical;
pub use browser::contract::{
    browser_runtime_contract_value, browser_runtime_request_context,
    browser_runtime_unavailable_diagnostic, BrowserRuntimeContract,
    BrowserRuntimeContractDescriptor, BROWSER_HARNESS_COMMAND_ENV,
    BROWSER_HARNESS_SUMMARY_FILE_ENV,
};

pub use browser::harness::{
    browser_bundle_harness_page, browser_bundle_harness_prelude, browser_bundle_harness_script,
    browser_bundle_runtime_harness_module_script, browser_bundle_runtime_harness_page,
    browser_bundle_runtime_harness_script, browser_runtime_harness_page,
    browser_runtime_harness_script, BROWSER_HARNESS_DONE_BINDING,
};
