use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-fasta-output-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

const FASTA_RANDOM_SHELL: &str = "\
var last = 42;
function rand(max) { last = (last * 3877 + 29573) % 139968; return max * last / 139968; }
function makeCumulative(table) {
  var prev = null;
  for (var c in table) {
    if (prev) table[c] = table[c] + table[prev];
    prev = c;
  }
}
function fastaRandom(n, table) {
  var line = new Array(60);
  makeCumulative(table);
  while (n > 0) {
    if (n < line.length) line = new Array(n);
    for (var i = 0; i < line.length; i = i + 1) {
      var r = rand(1);
      for (var c in table) { if (r < table[c]) break; }
      line[i] = c;
    }
    console.log(line.join(\"\"));
    n = n - line.length;
  }
}
var IUB = { a: 0.27, c: 0.12, g: 0.12, t: 0.27, B: 0.02, D: 0.02, H: 0.02, K: 0.02, M: 0.02, N: 0.02, R: 0.02, S: 0.02, V: 0.02, W: 0.02, Y: 0.02 };
fastaRandom(70, IUB);
";

#[test]
fn fasta_random_shell_matches_node() {
    // GOLDEN: derived by running FASTA_RANDOM_SHELL verbatim (with the
    // `table[c] = table[c] + table[prev]` form, which is numerically
    // identical to node's `+=`) under node v26.4.0 via a temp .mjs file and
    // capturing its exact stdout. fastaRandom(70, IUB) with `line = new
    // Array(60)` produces one 60-char line (n=70 >= 60) followed by one
    // 10-char line (remaining n=10 < 60) -- NOT three lines, since
    // 60 + 10 == 70 exhausts n after two iterations of the while loop.
    // Independently re-derived twice; both runs produced byte-identical
    // output (seed is fixed at 42, so the shell is fully deterministic).
    const GOLDEN: &str =
        "cttBtatcatatgctaKggNcataaaSatgtaaaDcDRtBggDtctttataattcBgtcg\ntactDtDagc\n";
    let out = run_source(FASTA_RANDOM_SHELL);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), GOLDEN);
}

const FASTA_REPEAT_SHELL: &str = "\
function fastaRepeat(n, seq) {
  var seqi = 0;
  var lenOut = 60;
  while (n > 0) {
    if (n < lenOut) lenOut = n;
    if (seqi + lenOut < seq.length) {
      console.log(seq.substring(seqi, seqi + lenOut));
      seqi = seqi + lenOut;
    } else {
      console.log(seq.substring(seqi) + seq.substring(0, lenOut - (seq.length - seqi)));
      seqi = lenOut - (seq.length - seqi);
    }
    n = n - lenOut;
  }
}
var ALU = \"GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG\" + \"GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA\";
fastaRepeat(120, ALU);
";

#[test]
fn fasta_repeat_shell_matches_node() {
    // ALU here is 84 chars (two 42-char segments). fastaRepeat(120) emits:
    //   line 1: chars [0,60)               (mid-string branch)
    //   line 2: chars [60,84)+[0,36)       (wrap-boundary else branch)
    // GOLDEN: derived by running FASTA_REPEAT_SHELL verbatim under node
    // v26.4.0 via a temp .mjs file and capturing its exact stdout.
    // Independently re-derived twice; both runs produced byte-identical
    // output (no randomness involved -- fully deterministic).
    const GOLDEN: &str = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGGGAGGCCGAGGCGGGCGGA\nTCACCTGAGGTCAGGAGTTCGAGAGGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCA\n";
    let out = run_source(FASTA_REPEAT_SHELL);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), GOLDEN);
}
