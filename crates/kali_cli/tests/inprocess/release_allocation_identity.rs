//! The release tiers substitute a const binding's initializer for its name
//! (`kali_optimize/src/specialize.rs:120-127`), so an array produced by an
//! allocating initializer loses the identity `--fast` gives it.
//!
//! Spec: `docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md`
//!
//! These tests must EXECUTE the module, not merely build it: the defect's
//! dangerous half is a literal-index read that builds clean and returns the
//! wrong value. `kali run` takes a source file and has no tier flag, so this
//! cannot be observed through the CLI — hence an in-process test here.

use kali_cli::build::{compile_source_file, BuildMode};
use kali_cli::ApiSurface;
use kali_error::Diagnostic;
use kali_runtime::RuntimeCtx;
use std::fs;
use tempfile::tempdir;

/// Compile `source` at `mode`, returning the wasm bytes or the diagnostics.
pub(crate) fn compile_at(source: &str, mode: BuildMode) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.ts");
    fs::write(&path, source).expect("write source");
    compile_source_file(&path, mode, ApiSurface::Deno, &[], false, false)
}

/// Compile and execute `source` at `mode`, returning captured guest stdout.
pub(crate) fn run_at(source: &str, mode: BuildMode) -> String {
    let wasm =
        compile_at(source, mode).unwrap_or_else(|d| panic!("compile failed under {mode:?}: {d:?}"));
    RuntimeCtx::new(None)
        .execute(&wasm)
        .unwrap_or_else(|d| panic!("execute failed under {mode:?}: {d:?}"))
        .stdout
}

/// The SILENT half. A literal index has a static name, so it takes the
/// ordinary member lane and never reaches the computed-member gateway — it
/// builds clean at every tier. node prints `7`.
const LITERAL_INDEX: &str = "\
const u = new Array(4).fill(7);
console.log(u[0]);
";

#[test]
fn literal_index_read_off_an_allocation_agrees_with_node_at_every_tier() {
    for mode in [
        BuildMode::Fast,
        BuildMode::Release,
        BuildMode::ReleaseAdvanced,
    ] {
        assert_eq!(
            run_at(LITERAL_INDEX, mode),
            "7\n",
            "{mode:?}: a literal-index read off `new Array(4).fill(7)` must be 7, \
             as it is at --fast and under node"
        );
    }
}
