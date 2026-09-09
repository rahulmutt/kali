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
fn nbody_runs_and_matches_canonical_output() {
    let source = fixture("nbody-benchmark-v1.ts");
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
        "-0.169075164\n-0.169087605\n"
    );
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
// FOLLOW-UP, WHICH TASK 8 FILES: "the optimizer inlines an allocating array
// initializer, destroying the array identity the computed-member gateway
// needs, so `--release`/`--release-advanced` refuse programs `--fast` compiles
// correctly." Closing that is what would make all three modes true again --
// and at that point these assertions go red and must be flipped deliberately.
#[test]
fn nbody_metadata_is_consistent() {
    let meta: Value = serde_json::from_str(
        &fs::read_to_string(fixture("nbody-benchmark-v1.json")).expect("read metadata"),
    )
    .expect("parse metadata");
    assert_eq!(meta["benchmark"], "nbody");
    assert_eq!(meta["version"], 1);
    assert_eq!(meta["sourceFile"], "nbody-benchmark-v1.ts");
    assert_eq!(
        meta["buildModes"],
        serde_json::json!(["--fast", "--release", "--release-advanced"])
    );
    let src = fs::read(fixture("nbody-benchmark-v1.ts")).expect("read source");
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
        .arg(fixture("nbody-benchmark-v1.ts"))
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
            .arg(fixture("nbody-benchmark-v1.ts"))
            .output()
            .expect("run kali build");
        let stderr = String::from_utf8_lossy(&built.stderr);
        assert!(
            !built.status.success(),
            "{mode}: expected the honest refusal, not a build. If this now \
             builds, do NOT just delete this -- the release tiers used to emit \
             zeros and invalid wasm here; verify the emitted module RUNS and \
             agrees with node, then update `buildModes` and its schema. \
             stderr: {stderr}"
        );
        assert!(stderr.contains("E5506"), "{mode}: stderr: {stderr}");
        assert!(
            stderr.contains("computed member access `o[k]` is unavailable in the current phase"),
            "{mode}: stderr: {stderr}"
        );
    }
}
