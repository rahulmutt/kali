use super::*;

/// A per-process, per-call-unique scratch directory. The brief's tests used
/// fixed names under `std::env::temp_dir()`; two concurrent runs of this test
/// binary (two shells, or a CI job sharing a machine) would then race on the
/// same directory and flake. Uniqueness must not rest on the wall clock alone
/// -- a coarse `SystemTime` can hand two threads the same nanosecond -- so a
/// process-wide counter carries it, mirroring the precedent in
/// `crates/kali_cli/tests/clbg_binary_trees_runtime.rs`.
fn scratch_dir(label: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir =
        std::env::temp_dir().join(format!("blast-radius-{label}-{}-{seq}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn the_corpus_hash_is_order_independent_and_content_sensitive() {
    let a = ManifestFile {
        path: "b.js".into(),
        stratum: "anchor".into(),
        sha256: "22".into(),
    };
    let b = ManifestFile {
        path: "a.js".into(),
        stratum: "anchor".into(),
        sha256: "11".into(),
    };
    let forward = corpus_hash(&[a.clone(), b.clone()]);
    let reversed = corpus_hash(&[b.clone(), a.clone()]);
    assert_eq!(forward, reversed, "hash must not depend on listing order");

    let changed = ManifestFile {
        sha256: "33".into(),
        ..a
    };
    assert_ne!(
        forward,
        corpus_hash(&[changed, b]),
        "a changed file must change the corpus hash"
    );
}

#[test]
fn verify_rejects_a_file_whose_content_changed() {
    let dir = scratch_dir("manifest-test");
    std::fs::create_dir_all(dir.join("anchor")).expect("mkdir");
    std::fs::write(dir.join("anchor/x.js"), "console.log(1);\n").expect("write");

    let good = ManifestFile {
        path: "anchor/x.js".into(),
        stratum: "anchor".into(),
        sha256: sha256_of("console.log(1);\n".as_bytes()),
    };
    let manifest = Manifest {
        corpus_hash: corpus_hash(std::slice::from_ref(&good)),
        files: vec![good],
    };
    verify_manifest(&dir, &manifest).expect("an unmodified corpus verifies");

    std::fs::write(dir.join("anchor/x.js"), "console.log(2);\n").expect("write");
    let error = verify_manifest(&dir, &manifest).expect_err("a modified corpus must not verify");
    assert!(
        error.contains("anchor/x.js"),
        "error names the file: {error}"
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn verify_rejects_an_untracked_file_in_the_corpus() {
    // A file present on disk but absent from the manifest would be counted by
    // the counter while the frozen hash still looked unchanged -- the exact
    // post-hoc corpus edit the freeze rule exists to prevent.
    let dir = scratch_dir("untracked-test");
    std::fs::create_dir_all(dir.join("anchor")).expect("mkdir");
    std::fs::write(dir.join("anchor/x.js"), "console.log(1);\n").expect("write");
    std::fs::write(dir.join("anchor/sneaky.js"), "console.log(2);\n").expect("write");

    let tracked = ManifestFile {
        path: "anchor/x.js".into(),
        stratum: "anchor".into(),
        sha256: sha256_of("console.log(1);\n".as_bytes()),
    };
    let manifest = Manifest {
        corpus_hash: corpus_hash(std::slice::from_ref(&tracked)),
        files: vec![tracked],
    };
    let error = verify_manifest(&dir, &manifest).expect_err("an untracked file must not verify");
    assert!(error.contains("sneaky.js"), "error names the file: {error}");

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn the_shipped_corpus_matches_its_manifest() {
    let root = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/blast-radius/corpus"
    ));
    let text = std::fs::read_to_string(root.join("manifest.json")).expect("manifest is readable");
    let manifest = parse_manifest(&text).expect("manifest parses");
    assert!(
        !manifest.files.is_empty(),
        "an empty manifest must not verify as frozen"
    );
    assert_eq!(
        manifest.corpus_hash,
        corpus_hash(&manifest.files),
        "the recorded corpus hash does not match its own file list"
    );
    verify_manifest(root, &manifest).expect("the shipped corpus matches its manifest");
}
