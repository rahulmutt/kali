use kali_cli::build::{compile_source_file, BuildMode};
use kali_cli::ApiSurface;
use kali_runtime::RuntimeCtx;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, process::Command};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/benchmarks")
        .join(name)
}

#[test]
fn fannkuch_redux_runs_and_matches_canonical_output() {
    let source = fixture("fannkuch-redux-benchmark-v1.ts");
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "228\nPfannkuchen(7) = 16\n"
    );
}

// RESTORED 2026-09-10 by the release-tier-allocation-identity project
// (docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md),
// discharging the 2026-09-09 re-pin below.
//
// WHAT WAS WRONG. `is_array_literal` in `kali_optimize` was negative space: a
// lowered `new Array(n).fill(v)` initializer qualified as a specializable
// binding, so every read of its name was overwritten with a clone of the
// initializer -- destroying the array identity the computed-member gateway
// needs and producing the `--release`/`--release-advanced` refusals described
// below. That predicate is fixed.
//
// WHAT IS MEASURED NOW, against node v26.8.2: all three modes -- `--fast`,
// `--release`, `--release-advanced` -- build AND RUN, agreeing with node
// byte-for-byte (`228` / `Pfannkuchen(7) = 16`). Asserted below per-mode so the
// record cannot silently drift from the measurement again. The standing gate
// for this (all 68 benchmark fixtures, all three tiers, executed and compared
// to node) is `crates/kali_cli/tests/inprocess/benchmark_execution.rs`, which
// pins fannkuch as `Expectation::Runs`.
//
// The original 2026-09-09 re-pin (by the computed-member-static-name project,
// controller ruling R22, at `6b59ddeef9`) recorded `--release` and
// `--release-advanced` exiting 1 with `error[E5506]: computed member access
// `o[k]` is unavailable ...`. `spectral-norm` and `nbody` were re-pinned under
// the same ruling at the same time; unlike fannkuch, closing this defect did
// NOT restore them -- see `clbg_spectral_norm_runtime.rs` and
// `clbg_nbody_runtime.rs` (still `KnownBroken` in `benchmark_execution.rs` for
// a different, still-open mechanism).
//
// The JSON value itself is unchanged (it always declared all three modes);
// `buildModes` is schema-locked by `schemas/benchmark/v1.json` and asserted
// corpus-wide by `tests/schema_docs/misc.rs`.
#[test]
fn fannkuch_redux_metadata_is_consistent() {
    let meta: Value = serde_json::from_str(
        &fs::read_to_string(fixture("fannkuch-redux-benchmark-v1.json")).expect("read metadata"),
    )
    .expect("parse metadata");
    assert_eq!(meta["benchmark"], "fannkuch-redux");
    assert_eq!(meta["version"], 1);
    assert_eq!(meta["sourceFile"], "fannkuch-redux-benchmark-v1.ts");
    assert_eq!(
        meta["buildModes"],
        serde_json::json!(["--fast", "--release", "--release-advanced"])
    );
    let src = fs::read(fixture("fannkuch-redux-benchmark-v1.ts")).expect("read source");
    let digest_bytes = Sha256::digest(&src);
    let digest = format!(
        "sha256-{}",
        digest_bytes
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    assert_eq!(
        meta["sourceSha256"], digest,
        "metadata sha256 must match the source file"
    );

    // The measured per-mode truth, asserted rather than described: all three
    // modes build AND the emitted module runs, agreeing with node.
    for mode in [
        BuildMode::Fast,
        BuildMode::Release,
        BuildMode::ReleaseAdvanced,
    ] {
        let wasm = compile_source_file(
            fixture("fannkuch-redux-benchmark-v1.ts"),
            mode,
            ApiSurface::Deno,
            &[],
            false,
            false,
        )
        .unwrap_or_else(|diagnostics| {
            panic!("fannkuch build failed under {mode:?}: {diagnostics:?}")
        });
        let outcome = RuntimeCtx::new(None)
            .execute(&wasm)
            .unwrap_or_else(|diagnostics| {
                panic!("fannkuch execute failed under {mode:?}: {diagnostics:?}")
            });
        assert_eq!(
            outcome.stdout, "228\nPfannkuchen(7) = 16\n",
            "{mode:?}: fannkuch must agree with node"
        );
    }
}
