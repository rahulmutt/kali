//! The only `kali_cli` integration test target that links wasmtime.
//!
//! These three suites cannot be black-box: two drive `kali_runtime::RuntimeCtx`
//! in-process to assert release-profile codegen, and one calls
//! `browser_runtime_execute_checked` from `kali_runtime::browser::execute`.
//! They are aggregated into a single target so wasmtime is statically linked
//! once (~450 MB) instead of three times (~1.2 GB).
//!
//! Add a module here only when a suite genuinely needs in-process runtime
//! access. Everything else belongs in `tests/cases/` (see
//! `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`).

#[path = "inprocess/browser_harness_cdp_in_page_trap_propagates.rs"]
mod browser_harness_cdp_in_page_trap_propagates;

#[path = "inprocess/release_constant_condition_loop.rs"]
mod release_constant_condition_loop;

#[path = "inprocess/release_mutated_binding_specialization.rs"]
mod release_mutated_binding_specialization;
