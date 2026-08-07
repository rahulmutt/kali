//! File-driven runner for `kali_cli`'s black-box CLI tests.
//!
//! One compiled target discovers `tests/cases/**/*.toml` at runtime, so adding
//! a test compiles nothing. See
//! `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`.

mod assertions;
mod discover;
mod expand;
mod jsonpath;
mod model;
mod steps;
pub use assertions::{check, check_json, Captured};
pub use discover::{discover, main_with};
pub use expand::{expand, Trial};
pub use jsonpath::{flatten_expected, lookup, values_equal};
pub use model::{parse_case_file, Case, CaseFile, Exit, ExitStatusWord, Step, StepKind};
pub use steps::{run_trial, RunnerConfig};
