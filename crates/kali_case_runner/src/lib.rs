//! File-driven runner for `kali_cli`'s black-box CLI tests.
//!
//! One compiled target discovers `tests/cases/**/*.toml` at runtime, so adding
//! a test compiles nothing. See
//! `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`.

mod expand;
mod model;
pub use expand::{expand, Trial};
pub use model::{parse_case_file, Case, CaseFile, Exit, ExitStatusWord, Step, StepKind};
