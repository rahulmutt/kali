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

/// The LOUD declarator half. A non-literal index reaches the computed-member
/// gateway, which cannot resolve a base with no name -- `E5506` at the release
/// tiers before 2026-09-10.
const DYNAMIC_INDEX: &str = "\
const u = new Array(4).fill(7);
let i = 0;
console.log(u[i]);
";

/// The LOUD parameter half. Ruling R22 recorded this as a SECOND class,
/// "deliberately NOT admitted". It is the same substitution arriving through an
/// inlined argument: measured `fast=0 / release>=1`, and correct at the default
/// tier. Spec §2.4 overturns R22 on that evidence.
const PARAMETER_READ: &str = "\
function f(a) { let i = 0; return a[i]; }
const u = new Array(2).fill(8);
console.log(f(u));
";

/// The parameter half with a STORE through the alias, which is the shape
/// spectral-norm's `Au(u, v)` actually has.
const PARAMETER_STORE: &str = "\
function g(a, b) { for (let i = 0; i < a.length; i = i + 1) { b[i] = a[i]; } }
const u = new Array(3).fill(5);
const v = new Array(3);
g(u, v);
console.log(v[0]);
";

/// The spectral-norm inner-loop shape at n=3, reduced.
const SPECTRAL_SHAPE: &str = "\
function Au(u, v) {
  for (let i = 0; i < u.length; i = i + 1) {
    let t = 0;
    for (let j = 0; j < u.length; j = j + 1) { t = t + u[j]; }
    v[i] = t;
  }
}
const u = new Array(3).fill(1);
const v = new Array(3);
Au(u, v);
console.log(v[0]);
";

#[test]
fn allocation_backed_reads_agree_with_node_at_every_tier() {
    for (label, source, expected) in [
        ("dynamic index off a declarator", DYNAMIC_INDEX, "7\n"),
        ("read through a parameter", PARAMETER_READ, "8\n"),
        ("store through a parameter", PARAMETER_STORE, "5\n"),
        ("the spectral-norm shape", SPECTRAL_SHAPE, "3\n"),
    ] {
        for mode in [
            BuildMode::Fast,
            BuildMode::Release,
            BuildMode::ReleaseAdvanced,
        ] {
            assert_eq!(
                run_at(source, mode),
                expected,
                "{label} under {mode:?}: must agree with node, as --fast already does"
            );
        }
    }
}
