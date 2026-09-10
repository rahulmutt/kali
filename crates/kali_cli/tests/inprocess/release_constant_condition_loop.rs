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
//! three modes. The fannkuch fixture at the bottom of this file is now refused
//! at both release tiers again, for an unrelated and wider defect -- read the
//! block above `fannkuch_redux_builds_and_runs_at_fast_and_is_refused_at_both_release_tiers`
//! before reading that refusal as a regression of this one.

use kali_cli::build::{compile_source_file, BuildMode};
use kali_cli::ApiSurface;
use kali_common::computed_member_access_unavailable_message;
use kali_error::_error_codes::e5;
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

// RE-PINNED 2026-09-09 at `6b59ddeef9`, by the computed-member-static-name
// project (docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md),
// on the same controller ruling R22 that re-pinned `spectral-norm` and `nbody`
// (see `crates/kali_cli/tests/clbg_spectral_norm_runtime.rs` and
// `crates/kali_cli/tests/clbg_nbody_runtime.rs`, whose blocks this mirrors).
//
// WHAT THIS TEST USED TO CLAIM. It was named
// `fannkuch_redux_builds_in_all_release_modes` and asserted a successful build
// under `Fast`, `Release` and `ReleaseAdvanced`. TWO OF THE THREE NOW REFUSE.
// Measured at `6b59ddeef9`: `Fast` builds AND RUNS (`228` / `Pfannkuchen(7) =
// 16`, byte-identical to node); `Release` and `ReleaseAdvanced` both fail to
// build with `error[E5506]: computed member access `o[k]` is unavailable ...`
// plus a `warning[E3100]: undefined call target 'Array' reached codegen and was
// lowered through a zero placeholder compatibility fallback`.
//
// WHY, AND WHY IT IS NOT THIS FILE'S OWN REGRESSION. The constant-condition
// fold this file exists to pin is fine -- the four `while (true)` / `while (1)`
// / `do-while` / `for (;true;)` tests above still pass in all three modes.
// fannkuch is refused for a DIFFERENT, wider defect: the release-mode optimizer
// inlines an ALLOCATING array initializer (`new Array(n)`) as if it were a pure
// constant, destroying the array identity the computed-member gateway needs, so
// the release tiers refuse programs `--fast` compiles correctly and silently
// miscompile others. Filed in
// `docs/superpowers/followups/release-mode-optimizer-inlines-an-allocating-initializer.md`
// and named in
// `docs/superpowers/followups/computed-member-static-name-discovered-defects.md`.
//
// WHY FANNKUCH WAS MISSED WHEN SPECTRAL-NORM AND NBODY WERE SWEPT. A stale
// pre-project wasm artifact in the gitignored on-disk incremental cache
// (`crates/kali_cli/tests/fixtures/.kali-cache/incremental/`, live because
// `tests/fixtures/kali.json` makes that directory a project root) short-circuited
// `compile_source_file` before codegen, so this test read three cached 2026-07-16
// builds and passed in ~0.00s on the machine doing the sweep. CI runners are cold
// and compile for real, which is why PR #36 went red on both. The cache key
// USED TO carry no compiler-build identity at all -- it ended in a frozen
// `CARGO_PKG_VERSION` of `"0.1.0"` -- so an artifact survived arbitrary
// compiler-semantics changes. Fixed: the key now folds in a fingerprint of the
// running compiler build (`crates/kali_cli/src/build/fingerprint.rs`), and the
// fixtures decline the on-disk cache outright. Design:
// `docs/superpowers/specs/2026-09-09-incremental-cache-compiler-identity-design.md`.
//
// IF THE RELEASE TIERS BUILD THIS AGAIN, do NOT just delete this block: the
// release tiers used to emit zeros here and, for spectral-norm, invalid wasm.
// Verify the emitted module actually RUNS and agrees with node, then restore the
// three-mode build assertion deliberately.
#[test]
fn fannkuch_redux_builds_and_runs_at_fast_and_is_refused_at_both_release_tiers() {
    let source = fannkuch_fixture();

    // `Fast` is not merely quiet: it builds AND the module runs, byte-identical
    // to node. Keeping a positive claim here is what stops this test from
    // degrading into "everything refuses, therefore green".
    let wasm = compile_source_file(
        &source,
        BuildMode::Fast,
        ApiSurface::Deno,
        &[],
        false,
        false,
    )
    .unwrap_or_else(|diagnostics| panic!("fannkuch build failed under Fast: {diagnostics:?}"));
    let runtime = RuntimeCtx::new(None);
    let outcome = runtime.execute(&wasm).unwrap_or_else(|diagnostics| {
        panic!("fannkuch execute failed under Fast: {diagnostics:?}")
    });
    assert_eq!(
        outcome.stdout, "228\nPfannkuchen(7) = 16\n",
        "the fast lane must agree with node"
    );

    for mode in [BuildMode::Release, BuildMode::ReleaseAdvanced] {
        let diagnostics =
            match compile_source_file(&source, mode, ApiSurface::Deno, &[], false, false) {
                Ok(_) => panic!(
                    "{mode:?}: expected the honest refusal, not a build. If this now builds, \
                     do NOT just delete this -- the release tiers used to emit zeros and \
                     invalid wasm for this defect; verify the emitted module RUNS and agrees \
                     with node, then restore the three-mode build assertion deliberately."
                ),
                Err(diagnostics) => diagnostics,
            };
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.code == Some(u32::from(e5::FEATURE_UNAVAILABLE))
                    && diagnostic
                        .message
                        .contains(computed_member_access_unavailable_message())
            }),
            "{mode:?}: expected an E5506 carrying the canonical computed-member wording; \
             got {diagnostics:?}"
        );
    }
}
