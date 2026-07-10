use sha2::{Digest, Sha256};
use std::{path::PathBuf, process::Command};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/benchmarks")
        .join(name)
}

// Tier 1 — small-N golden. The verbatim upstream fasta-node-1 source (with
// `+=`/`-=`/`i++`), read from the checked-in fixture, run under `--api node`
// with N=8, must match node v26.4.0 byte-for-byte (seed fixed at 42).
#[test]
fn fasta_small_n_matches_node_golden() {
    const GOLDEN: &str = ">ONE Homo sapiens alu\nGGCCGGGCGCGGTGGC\n>TWO IUB ambiguity codes\ncttBtatcatatgctaKggNcata\n>THREE Homo sapiens frequency\naatagctaaatcttgtgcttcgttagaagtctcgactacg\n";
    let source = fixture("fasta-benchmark-v1.ts");
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg(&source)
        .arg("--")
        .arg("8")
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), GOLDEN);
}

// Tier 2 — canonical large-N SHA-256. N=25,000,000 (254,166,745 bytes of
// output) is the fasta Spec 7 §4.1 canonical pin. It depends on fasta Spec
// 7's bounded-peak per-string-site arena reclamation (Tasks 4a-4g: the
// ArenaTable string-site channel, per-site iteration-locality analysis,
// `__join_arena`/`string_concat_arena` routing, string-site-triggered loop
// arenas, and module-constant for-in key tables) to keep peak allocator
// churn bounded well under the wasm32 4 GiB linear-memory ceiling — without
// it, fastaRepeat's `+`-concat and fastaRandom's `.join("")` loops leak
// their per-line temporaries and N=25M traps E4000 (see Task 4e's bounded-
// peak proof and Task 4f/4g's loop-arena-opening fix). The node v26.4.0
// reference hash below was independently re-derived twice (file-redirected,
// not piped, to avoid stdout-pipe truncation at this output volume).
#[test]
fn fasta_large_n_matches_node_sha256() {
    const N: &str = "25000000";
    const NODE_SHA256: &str = "6a26f1c843bebd234692ff1bd98ad517dd7df732fe93d2095845a2ddafc9ecee";
    let source = fixture("fasta-benchmark-v1.ts");
    let policy = fixture("fasta-benchmark-v1.policy.json");
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg("--sandbox")
        .arg(&policy)
        .arg(&source)
        .arg("--")
        .arg(N)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let digest = format!("{:x}", Sha256::digest(&output.stdout));
    assert_eq!(
        digest, NODE_SHA256,
        "fasta N={} output SHA-256 differs from the node v26.4.0 reference",
        N
    );
}
