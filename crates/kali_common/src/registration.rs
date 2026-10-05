//! The callees whose call registers its callback argument for a later
//! host-driven invocation, with the env active at the call. One list, read by
//! MIR (per-iteration env records, block-scoping spec A-2) and by codegen's
//! dynamic-env safety gate (`kali_codegen/src/env_safety.rs`).

/// True when a call to `name` registers a callback for later: a bare
/// `queueMicrotask` / `setTimeout` / `setInterval`, or a member
/// `addEventListener`.
pub fn is_deferred_registration_callee(name: &str, is_member: bool) -> bool {
    if is_member {
        name == "addEventListener"
    } else {
        matches!(name, "queueMicrotask" | "setTimeout" | "setInterval")
    }
}

#[cfg(test)]
#[path = "registration_tests.rs"]
mod registration_tests;
