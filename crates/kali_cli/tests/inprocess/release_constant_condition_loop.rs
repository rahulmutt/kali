//! Regression: a loop whose condition is a bare constant-truthy literal
//! (`while (true)`, `while (1)`) that contains `break`/`continue` must lower
//! through the real loop path (establishing a loop frame + back-edge) under
//! every build mode.
//!
//! Previously the release-only constant-condition fold in `kali_optimize`
//! collapsed such a loop `Branch` node down to its body, discarding the loop
//! frame and leaving the inner `break`/`continue` stranded — codegen then
//! rejected it with `E5506`. `--fast` (which skips the fold) worked, but
//! `--release` / `--release-advanced` failed to build the fannkuch-redux
//! benchmark, whose two main loops use `while (true)`.
//!
//! THAT FOLD IS STILL FIXED and the four loop tests below still pass in all
//! three modes. The fannkuch fixture at the bottom of this file was refused at
//! both release tiers for an unrelated, wider defect (the release-mode
//! optimizer inlining an allocating array initializer as if it were a pure
//! constant); that defect is fixed as of 2026-09-10 by the
//! release-tier-allocation-identity project, and the fixture now builds AND
//! RUNS at all three modes, agreeing with node -- read the block above
//! `fannkuch_redux_runs_at_all_three_modes` for the restoration record.

use kali_cli::build::{compile_source_file, BuildMode};
use kali_cli::ApiSurface;
use kali_runtime::RuntimeCtx;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn compile(source: &str, mode: BuildMode) -> Vec<u8> {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.ts");
    fs::write(&path, source).expect("write source");
    compile_source_file(&path, mode, ApiSurface::Deno, &[], false, false)
        .unwrap_or_else(|diagnostics| panic!("compile failed under {mode:?}: {diagnostics:?}"))
}

fn compile_and_run(source: &str, mode: BuildMode) -> String {
    let wasm = compile(source, mode);
    let runtime = RuntimeCtx::new(None);
    let outcome = runtime
        .execute(&wasm)
        .unwrap_or_else(|diagnostics| panic!("execute failed under {mode:?}: {diagnostics:?}"));
    outcome.stdout
}

const WHILE_TRUE_BREAK: &str = "\
function main() {
  let i = 0;
  while (true) {
    if (i >= 3) { break; }
    i = i + 1;
  }
  console.log(i);
}
main();
";

const WHILE_ONE_CONTINUE: &str = "\
function main() {
  let i = 0;
  let sum = 0;
  while (1) {
    i = i + 1;
    if (i > 5) { break; }
    if (i === 3) { continue; }
    sum = sum + i;
  }
  console.log(sum);
}
main();
";

#[test]
fn while_true_break_runs_in_all_build_modes() {
    for mode in [
        BuildMode::Fast,
        BuildMode::Release,
        BuildMode::ReleaseAdvanced,
    ] {
        assert_eq!(
            compile_and_run(WHILE_TRUE_BREAK, mode),
            "3\n",
            "while (true) + break must run identically under {mode:?}"
        );
    }
}

#[test]
fn while_literal_one_continue_runs_in_all_build_modes() {
    // i runs 1..=5, skipping the contribution when i === 3: 1 + 2 + 4 + 5 = 12.
    for mode in [
        BuildMode::Fast,
        BuildMode::Release,
        BuildMode::ReleaseAdvanced,
    ] {
        assert_eq!(
            compile_and_run(WHILE_ONE_CONTINUE, mode),
            "12\n",
            "while (1) + continue must run identically under {mode:?}"
        );
    }
}

const DO_WHILE_TRUE_BREAK: &str = "\
function main() {
  let i = 0;
  do {
    i = i + 1;
    if (i >= 3) { break; }
  } while (true);
  console.log(i);
}
main();
";

const FOR_TRUE_BREAK: &str = "\
function main() {
  let n = 0;
  for (let i = 0; true; i = i + 1) {
    if (i >= 4) { break; }
    n = n + 1;
  }
  console.log(n);
}
main();
";

#[test]
fn do_while_true_break_runs_in_all_build_modes() {
    for mode in [
        BuildMode::Fast,
        BuildMode::Release,
        BuildMode::ReleaseAdvanced,
    ] {
        assert_eq!(
            compile_and_run(DO_WHILE_TRUE_BREAK, mode),
            "3\n",
            "do-while (true) + break must run identically under {mode:?}"
        );
    }
}

#[test]
fn for_true_break_runs_in_all_build_modes() {
    for mode in [
        BuildMode::Fast,
        BuildMode::Release,
        BuildMode::ReleaseAdvanced,
    ] {
        assert_eq!(
            compile_and_run(FOR_TRUE_BREAK, mode),
            "4\n",
            "for (; true; ) + break must run identically under {mode:?}"
        );
    }
}

fn fannkuch_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/benchmarks/fannkuch-redux-benchmark-v1.ts")
}

// RESTORED 2026-09-10 by the release-tier-allocation-identity project
// (docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md).
//
// This test used to be named
// `fannkuch_redux_builds_and_runs_at_fast_and_is_refused_at_both_release_tiers`
// and, per the 2026-09-09 re-pin below (controller ruling R22, at
// `6b59ddeef9`, by the computed-member-static-name project), asserted `Fast`
// builds and runs while `Release`/`ReleaseAdvanced` refuse with
// `error[E5506]: computed member access `o[k]` is unavailable ...`.
//
// The panic message this test used to carry at its `Release`/`ReleaseAdvanced`
// loop said exactly what would authorize reversing that: "verify the emitted
// module RUNS and agrees with node, then restore the three-mode build
// assertion deliberately." That verification is
// `crates/kali_cli/tests/inprocess/benchmark_execution.rs`, which compiles and
// executes all 68 benchmark fixtures (fannkuch included) at all three tiers
// against node v26.8.2 and pins fannkuch as `Expectation::Runs`. The root
// cause was `is_array_literal` in `kali_optimize` being negative space: a
// lowered `new Array(n).fill(v)` initializer qualified as a specializable
// binding, so every read of its name was overwritten with a clone of the
// initializer, destroying the array identity the computed-member gateway
// needs. That predicate is fixed, so the three-mode assertion is restored
// here too.
//
// `spectral-norm` and `nbody` were re-pinned under the same 2026-09-09 ruling
// at the same time as fannkuch; unlike fannkuch, closing this defect did NOT
// restore them (they fail differently and remain `Expectation::KnownBroken` in
// `benchmark_execution.rs`) -- see `clbg_spectral_norm_runtime.rs` and
// `clbg_nbody_runtime.rs`.
#[test]
fn fannkuch_redux_runs_at_all_three_modes() {
    let source = fannkuch_fixture();

    for mode in [
        BuildMode::Fast,
        BuildMode::Release,
        BuildMode::ReleaseAdvanced,
    ] {
        let wasm = compile_source_file(&source, mode, ApiSurface::Deno, &[], false, false)
            .unwrap_or_else(|diagnostics| {
                panic!("fannkuch build failed under {mode:?}: {diagnostics:?}")
            });
        let runtime = RuntimeCtx::new(None);
        let outcome = runtime.execute(&wasm).unwrap_or_else(|diagnostics| {
            panic!("fannkuch execute failed under {mode:?}: {diagnostics:?}")
        });
        assert_eq!(
            outcome.stdout, "228\nPfannkuchen(7) = 16\n",
            "{mode:?}: fannkuch must agree with node"
        );
    }
}
