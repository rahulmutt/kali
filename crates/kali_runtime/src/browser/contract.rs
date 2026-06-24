//! Browser runtime contract, descriptor, and unavailability helpers.
use crate::*;

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
