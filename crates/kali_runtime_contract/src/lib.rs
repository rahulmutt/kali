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
#[cfg(test)]
pub(crate) use browser::contract::*;
pub use browser::contract::{
    browser_runtime_contract_value, browser_runtime_request_context,
    browser_runtime_unavailable_diagnostic, BrowserRuntimeContract,
    BrowserRuntimeContractDescriptor, BROWSER_HARNESS_COMMAND_ENV,
    BROWSER_HARNESS_SUMMARY_FILE_ENV,
};
