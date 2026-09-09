use kali_common::computed_member_access_unavailable_message;
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

// RE-PINNED 2026-09-09 at `6b59ddeef9`, by the computed-member-static-name
// project (docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md),
// on controller ruling R22 -- the same ruling and the same defect that re-pinned
// `spectral-norm` and `nbody`. This block mirrors theirs; read either of those
// files for the full mechanism.
//
// WHAT `buildModes` MEANS NOW. The metadata declares `["--fast", "--release",
// "--release-advanced"]`, and that was read as three modes that BUILD. TWO OF
// THE THREE NOW REFUSE. Measured at `6b59ddeef9` against node v26.8.1: `--fast`
// builds and runs correctly (`228` / `Pfannkuchen(7) = 16`), `--release` and
// `--release-advanced` both exit 1 with `error[E5506]: computed member access
// `o[k]` is unavailable ...` plus `warning[E3100]: undefined call target 'Array'
// reached codegen and was lowered through a zero placeholder compatibility
// fallback`. The optimizer inlines an ALLOCATING array initializer as if it were
// a pure constant, destroying the array identity the computed-member gateway
// needs. So the field is a DECLARATION OF THE MEASUREMENT MATRIX, not a claim
// that all three succeed, and the per-mode truth is asserted below so the record
// cannot drift from the measurement again.
//
// The JSON value itself is unchanged, for the reason spelled out at length in
// `clbg_spectral_norm_runtime.rs`: `buildModes` is schema-locked by
// `schemas/benchmark/v1.json` and asserted corpus-wide by
// `tests/schema_docs/misc.rs`, so shortening it is a controller call.
//
// WHY THIS FIXTURE WAS MISSED WHEN ITS TWO SIBLINGS WERE RE-PINNED. A stale
// pre-project wasm artifact in the gitignored on-disk incremental cache
// (`crates/kali_cli/tests/fixtures/.kali-cache/incremental/`) short-circuited
// `compile_source_file` before codegen, so fannkuch stayed green on the machine
// that swept the other two. See
// `docs/superpowers/followups/computed-member-static-name-discovered-defects.md`
// §6 for the cache-key defect that allows it.
//
// FOLLOW-UP:
// `docs/superpowers/followups/release-mode-optimizer-inlines-an-allocating-initializer.md`.
// Closing that is what would make all three modes true again -- and at that
// point these assertions go red and must be flipped deliberately.
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

    // The measured per-mode truth, asserted rather than described.
    let fast_dir = tempfile::tempdir().expect("tempdir");
    let fast = Command::new(kali_bin())
        .arg("build")
        .arg("--fast")
        .arg("--out-dir")
        .arg(fast_dir.path())
        .arg(fixture("fannkuch-redux-benchmark-v1.ts"))
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
            .arg(fixture("fannkuch-redux-benchmark-v1.ts"))
            .output()
            .expect("run kali build");
        let stderr = String::from_utf8_lossy(&built.stderr);
        assert!(
            !built.status.success(),
            "{mode}: expected the honest refusal, not a build. If this now \
             builds, do NOT just delete this -- the release tiers used to emit \
             zeros and invalid wasm for this defect; verify the emitted module \
             RUNS and agrees with node, then update `buildModes` and its schema. \
             stderr: {stderr}"
        );
        assert!(stderr.contains("E5506"), "{mode}: stderr: {stderr}");
        assert!(
            stderr.contains(computed_member_access_unavailable_message()),
            "{mode}: stderr: {stderr}"
        );
    }
}
