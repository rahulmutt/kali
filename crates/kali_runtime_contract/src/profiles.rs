//! Runtime host contract, backend, and profile normalization helpers.

use serde_json::Value;
use std::collections::BTreeSet;

/// The current high-level runtime host contract selected for execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostContract {
    /// The current standalone Kali-hosted runtime contract backed by wasmtime.
    KaliHosted,
    /// A browser API-surface request that remains gated until the standalone browser runtime exists.
    BrowserRequested,
}

impl RuntimeHostContract {
    /// Return a stable, human-readable label for the selected host contract.
    pub const fn canonical_label(self) -> &'static str {
        match self {
            Self::KaliHosted => "kali-hosted",
            Self::BrowserRequested => "browser-requested",
        }
    }
}

/// Canonical runtime backend label for the current execution host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeBackend {
    /// The current canonical pure-Rust Wasmtime backend.
    Wasmtime,
    /// A browser-harnessed backend selected when the later browser runtime path executes.
    BrowserHarness,
}

impl RuntimeBackend {
    /// Return a stable, human-readable label for the selected runtime backend.
    pub const fn canonical_label(self) -> &'static str {
        match self {
            Self::Wasmtime => "wasmtime",
            Self::BrowserHarness => "browser-harness",
        }
    }
}

/// Normalize runtime-profile inputs into the canonical deterministic order used
/// across CLI, runtime, and artifact metadata paths.
pub fn normalize_runtime_profiles(runtime_profiles: Vec<String>) -> Vec<String> {
    let mut normalized = BTreeSet::new();
    for profile in runtime_profiles {
        let profile = profile.trim();
        if !profile.is_empty() {
            normalized.insert(profile.to_string());
        }
    }
    normalized.into_iter().collect()
}

pub(crate) fn parse_runtime_host_contract_label(label: &str) -> Option<RuntimeHostContract> {
    match label {
        "kali-hosted" => Some(RuntimeHostContract::KaliHosted),
        "browser-requested" => Some(RuntimeHostContract::BrowserRequested),
        _ => None,
    }
}

pub(crate) fn parse_runtime_backend_label(label: &str) -> Option<RuntimeBackend> {
    match label {
        "wasmtime" => Some(RuntimeBackend::Wasmtime),
        "browser-harness" => Some(RuntimeBackend::BrowserHarness),
        _ => None,
    }
}

pub fn parse_optional_runtime_host_contract_label(
    value: Option<&Value>,
) -> Option<RuntimeHostContract> {
    let label = value?.as_str()?.trim();
    if label.is_empty() {
        return None;
    }

    parse_runtime_host_contract_label(label)
}

pub fn parse_optional_runtime_backend_label(value: Option<&Value>) -> Option<RuntimeBackend> {
    let label = value?.as_str()?.trim();
    if label.is_empty() {
        return None;
    }

    parse_runtime_backend_label(label)
}

#[cfg(test)]
#[path = "profiles_tests.rs"]
mod profiles_tests;
