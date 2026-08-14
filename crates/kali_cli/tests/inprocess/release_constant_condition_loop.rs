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

#[test]
fn fannkuch_redux_builds_in_all_release_modes() {
    let source = fannkuch_fixture();
    for mode in [
        BuildMode::Fast,
        BuildMode::Release,
        BuildMode::ReleaseAdvanced,
    ] {
        compile_source_file(&source, mode, ApiSurface::Deno, &[], false, false).unwrap_or_else(
            |diagnostics| panic!("fannkuch build failed under {mode:?}: {diagnostics:?}"),
        );
    }
}
