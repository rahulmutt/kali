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
fn spectral_norm_runs_and_matches_canonical_output() {
    let source = fixture("spectral-norm-benchmark-v1.ts");
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
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1.274219991\n");
}

// RE-PINNED 2026-09-09 at `dbaf05767b` (controller ruling R22), by the
// computed-member-static-name project
// (docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md).
//
// WHAT `buildModes` MEANS NOW, AND WHY THIS TEST NO LONGER LEAVES IT ALONE.
// This fixture's metadata declares `["--fast", "--release",
// "--release-advanced"]`, and until this project that was read as three modes
// that BUILD -- the 2026-07-0X design's acceptance criterion says in as many
// words that "the existing `assert_optimization_benchmark_fixture`
// compile-in-three-modes path passes for it". TWO OF THE THREE NOW REFUSE.
// Measured at `dbaf05767b` against node v26.8.1: `--fast` builds and runs
// correctly, `--release` and `--release-advanced` both exit 1 with
// `error[E5506]: computed member access `o[k]` is unavailable ...`. The
// optimizer inlines an ALLOCATING array initializer as if it were a pure
// constant, destroying the array identity the computed-member gateway needs;
// the controller established by LIR dump and by reinstating the pre-project
// name fabrication behind a temporary gate that the release tiers had been
// silently emitting ZEROS and, for spectral-norm, INVALID WASM. Ruling R22:
// the refusal stands, nothing is restored, and the fixtures are not rewritten
// (they are the upstream Benchmarks Game programs).
//
// SO THE FIELD IS NOW A DECLARATION OF THE MEASUREMENT MATRIX, NOT A CLAIM
// THAT ALL THREE SUCCEED, and this test says so instead of asserting the old
// reading as "consistent" and stopping. The per-mode truth is asserted below,
// in this same test, so the record cannot drift from the measurement again
// without something going red.
//
// WHY THE JSON VALUE ITSELF IS UNCHANGED. `buildModes` is schema-locked, not
// free text: `schemas/benchmark/v1.json` pins `prefixItems` to those three
// consts with `items: false`, `minItems: 3`, `maxItems: 3` and
// `additionalProperties: false` over exactly five keys -- so the file can
// carry neither a shorter list nor an explanatory key -- and
// `tests/schema_docs/misc.rs` asserts BOTH that schema and, corpus-wide, that
// EVERY benchmark fixture carries all three. Shortening these two would mean
// changing a published schema and weakening a ~60-fixture invariant, which is
// the controller's call and not a fix-round edit. Escalated in the Task 7
// report.
//
// UPDATED 2026-09-10 at Task 8 of the release-tier-allocation-identity
// project (docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md,
// docs/superpowers/sdd/2026-09-10-release-tier-allocation-identity/).
//
// STAGE 1 CLOSED THE E5506 REFUSAL THIS TEST USED TO ASSERT, SO THIS
// ASSERTION WENT STALE -- FIXED HERE, NOT LEFT RED FOR A LATER READER TO
// TRIP OVER. `is_array_literal` (crates/kali_optimize/src/layout.rs) gained
// a positive element check that stops a `new Array(n).fill(v)` declarator
// initializer from entering the layout-binding specialization environment,
// which is what was destroying this fixture's array identity. Measured at
// this branch's tip (`d67d820f98`): `--release` and `--release-advanced`
// now BUILD CLEAN, no E5506, exactly like `--fast`. **This is NOT full
// restoration** -- the emitted module fails to LOAD at both release tiers
// (`error[E4201]`, `wasm[0]::function[46]`/`[41]`), a different, still-open
// mechanism (task-5-report.md, "Ruling 2 verdict: different mechanism").
// `--fast` still matches node byte-for-byte (`1.274219991`).
//
// This file has no in-process wasmtime access (inprocess.rs is the ONLY
// kali_cli integration test target that statically links wasmtime, by
// design -- a second linking target costs another ~450MB and this pod's
// disk has been exhausted by extra target directories before), so it
// cannot itself assert the E4201 load failure. The standing gate for the
// full build+load+execute+compare truth, at every tier, is
// `crates/kali_cli/tests/inprocess/benchmark_execution.rs`'s
// `Expectation::KnownBroken` entry for this stem -- see it for the
// authoritative, currently-true account. This test asserts only what it
// can verify without wasmtime: that the release tiers now BUILD, which is
// the half of the old assertion that Stage 1 actually changed.
//
// `buildModes` stays `["--fast", "--release", "--release-advanced"]`
// unchanged, for the schema-locked reason the paragraph above already
// gives -- this project's own fix did not move that field, and closing the
// remaining E4201 gap is a different project's job.
#[test]
fn spectral_norm_metadata_is_consistent() {
    let meta: Value = serde_json::from_str(
        &fs::read_to_string(fixture("spectral-norm-benchmark-v1.json")).expect("read metadata"),
    )
    .expect("parse metadata");
    assert_eq!(meta["benchmark"], "spectral-norm");
    assert_eq!(meta["version"], 1);
    assert_eq!(meta["sourceFile"], "spectral-norm-benchmark-v1.ts");
    assert_eq!(
        meta["buildModes"],
        serde_json::json!(["--fast", "--release", "--release-advanced"])
    );
    let src = fs::read(fixture("spectral-norm-benchmark-v1.ts")).expect("read source");
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

    // The measured per-mode truth, asserted rather than described.
    let fast_dir = tempfile::tempdir().expect("tempdir");
    let fast = Command::new(kali_bin())
        .arg("build")
        .arg("--fast")
        .arg("--out-dir")
        .arg(fast_dir.path())
        .arg(fixture("spectral-norm-benchmark-v1.ts"))
        .output()
        .expect("run kali build --fast");
    assert!(
        fast.status.success(),
        "--fast must still build; stderr: {}",
        String::from_utf8_lossy(&fast.stderr)
    );
    for mode in ["--release", "--release-advanced"] {
        let out_dir = tempfile::tempdir().expect("tempdir");
        let built = Command::new(kali_bin())
            .arg("build")
            .arg(mode)
            .arg("--out-dir")
            .arg(out_dir.path())
            .arg(fixture("spectral-norm-benchmark-v1.ts"))
            .output()
            .expect("run kali build");
        let stderr = String::from_utf8_lossy(&built.stderr);
        assert!(
            built.status.success(),
            "{mode}: Stage 1 (release-tier-allocation-identity, Task 2) closed the \
             E5506 refusal this used to assert -- the build should succeed now, matching \
             --fast. If this refuses again, Stage 1's fix regressed; do not just flip this \
             assertion back without re-reading task-5-report.md's Ruling 2 verdict first. \
             stderr: {stderr}"
        );
        assert!(
            !stderr.contains("E5506"),
            "{mode}: no E5506 expected post-Stage-1; stderr: {stderr}"
        );
    }
    // The E4201 load-time failure this fixture still has at both release
    // tiers is NOT asserted here -- this test binary has no wasmtime access
    // (see the comment above). `inprocess/benchmark_execution.rs`'s
    // `Expectation::KnownBroken` entry for this stem does have wasmtime
    // access and executes the module: it pins `--fast`'s exact byte-for-byte
    // stdout (`"1.274219991\n"`) via `fast:`, and its backstop requires that
    // at least one tier actually fail to build, execute, or agree with node
    // -- Release/ReleaseAdvanced's execute-time E4201 failure is what
    // satisfies that requirement. That entry does NOT check for the E4201
    // code specifically (it carries no `release_refusal_code`: E4201 is a
    // load failure, not a diagnosed build refusal), only that the tier
    // fails.
}
