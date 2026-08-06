//! The only `kali_cli` integration test target that links wasmtime.
//!
//! Three suites cannot be black-box: two drive `kali_runtime::RuntimeCtx`
//! in-process to assert release-profile codegen, and one calls
//! `browser_runtime_execute_checked` from `kali_runtime::browser::execute`.
//! A fourth, `schema_validation`, doesn't need the runtime at all but calls
//! `kali_cli::{build, output}` schema-validator functions directly on
//! hand-constructed (frequently malformed) JSON values; there is no CLI
//! subcommand that validates arbitrary/malformed JSON, so those assertions
//! cannot be driven through the `kali` binary. All four are aggregated into
//! a single target so wasmtime is statically linked once (~450 MB) instead
//! of separately per binary.
//!
//! Add a module here only when a suite genuinely needs in-process runtime
//! or library access that the CLI binary cannot surface. Everything else
//! belongs in `tests/cases/` (see
//! `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`).

#[path = "inprocess/browser_harness_cdp_in_page_trap_propagates.rs"]
mod browser_harness_cdp_in_page_trap_propagates;

#[path = "inprocess/release_constant_condition_loop.rs"]
mod release_constant_condition_loop;

#[path = "inprocess/release_mutated_binding_specialization.rs"]
mod release_mutated_binding_specialization;

#[path = "inprocess/schema_validation.rs"]
mod schema_validation;
