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

// Tier 2 — large-N SHA-256. N=2,000,000 (20 MB output, ~1.5s) sits with ~40%
// byte-headroom below the measured ~N>=4M allocation wall (E4000): the fasta
// output loops leak their per-line join/substring temporaries — there is NO
// per-line reclamation yet. Canonical N=25,000,000 (254 MB) awaits fasta
// Spec 7's arena reclamation; its node reference hash is
// 6a26f1c843bebd234692ff1bd98ad517dd7df732fe93d2095845a2ddafc9ecee. This
// interim tier proves the golden-free SHA validation harness against the
// N=2M node reference.
#[test]
fn fasta_large_n_matches_node_sha256() {
    const N: &str = "2000000";
    const NODE_SHA256: &str = "a6b7308b4f7ea37cbaef69bdb05448c8623549978dc24d30e4e197026c1e073a";
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
