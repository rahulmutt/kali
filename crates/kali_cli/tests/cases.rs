//! File-driven CLI test target.
//!
//! Every case lives in `tests/cases/**/*.toml`; adding one compiles nothing.
//! Filter with the path: `cargo test -p kali_cli --test cases -- switch/`.
//!
//! Do not add Rust test logic here. Cases that the format cannot express stay
//! as their own hand-written target -- see
//! `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md` 5.11.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    kali_case_runner::main_with(kali_case_runner::RunnerConfig {
        kali_bin: PathBuf::from(env!("CARGO_BIN_EXE_kali")),
        cases_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases"),
    })
}
