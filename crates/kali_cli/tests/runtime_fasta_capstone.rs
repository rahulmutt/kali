use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_node_source_with_args(src: &str, args: &[&str]) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-fasta-capstone-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    let mut cmd = Command::new(kali_bin());
    cmd.arg("run").arg("--api").arg("node").arg(&path).arg("--");
    for a in args {
        cmd.arg(a);
    }
    cmd.output().expect("run kali")
}

// The full fasta-node-1 shell (Ian Osgood's CLBG fasta, output layer +
// process.argv/N — fasta Spec 5, Task 7 capstone). All three sections
// (`fastaRepeat` over `ALU`, `fastaRandom` over `IUB`, `fastaRandom` over
// `HomoSap`), the three section headers, and `n = +process.argv[2]`.
//
// Uses ONLY the supported statement forms established in Tasks 1-6:
// `x = x + y` / `x = x - y` instead of `+=`/`-=`, and explicit `i = i + 1`
// instead of `i++`. This is numerically identical to the upstream program's
// compound-assignment forms (see GOLDEN derivation below).
const FASTA_CAPSTONE_SHELL: &str = "\
var last = 42;
function rand(max) { last = (last * 3877 + 29573) % 139968; return max * last / 139968; }
function makeCumulative(table) {
  var prev = null;
  for (var c in table) {
    if (prev) table[c] = table[c] + table[prev];
    prev = c;
  }
}
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
var ALU = \"GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG\" +
\"GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA\" +
\"CCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAAT\" +
\"ACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCA\" +
\"GCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGG\" +
\"AGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCC\" +
\"AGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAAA\";
var IUB = { a: 0.27, c: 0.12, g: 0.12, t: 0.27, B: 0.02, D: 0.02, H: 0.02, K: 0.02, M: 0.02, N: 0.02, R: 0.02, S: 0.02, V: 0.02, W: 0.02, Y: 0.02 };
var HomoSap = { a: 0.3029549426680, c: 0.1979883004921, g: 0.1975473066391, t: 0.3015094502008 };
var n = +process.argv[2];
console.log(\">ONE Homo sapiens alu\");
fastaRepeat(2 * n, ALU);
console.log(\">TWO IUB ambiguity codes\");
fastaRandom(3 * n, IUB);
console.log(\">THREE Homo sapiens frequency\");
fastaRandom(5 * n, HomoSap);
";

#[test]
fn full_fasta_shell_matches_node_at_small_n() {
    const N: &str = "8";
    // GOLDEN: derived by running FASTA_CAPSTONE_SHELL (with `+=`/`-=`/`i++`
    // restored for readability -- numerically identical to the `= a + b` /
    // `= a - b` / `i = i + 1` forms used above) under node v26.4.0 via a
    // temp .mjs file as `node <file> 8`, capturing its exact stdout.
    // Independently re-derived twice; both runs produced byte-identical
    // output (seed fixed at 42, so the shell is fully deterministic).
    //
    // At N=8: fastaRepeat(16, ALU) emits one 16-char line (16 < ALU.length,
    // mid-string branch, no wrap). fastaRandom(24, IUB) and
    // fastaRandom(40, HomoSap) each emit one line (24 < 60 and 40 < 60, so
    // `line = new Array(n)` fires once and the while loop exits after a
    // single iteration -- no multi-line wraps at this small N).
    const GOLDEN: &str = ">ONE Homo sapiens alu\nGGCCGGGCGCGGTGGC\n>TWO IUB ambiguity codes\ncttBtatcatatgctaKggNcata\n>THREE Homo sapiens frequency\naatagctaaatcttgtgcttcgttagaagtctcgactacg\n";
    let out = run_node_source_with_args(FASTA_CAPSTONE_SHELL, &[N]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), GOLDEN);
}
