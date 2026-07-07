use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-forin-{}-{}-{}",
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

#[test]
fn for_in_over_fixed_shape_object_iterates_once_per_field() {
    // The body runs once per own field of the statically-shaped object.
    // Key not used yet; only the iteration count is observable.
    let out = run_source(
        "const table = { a: 1, c: 2, g: 3 };\nlet count = 0;\nfor (var c in table) {\n  count = count + 1;\n}\nconsole.log(count);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}

#[test]
fn for_in_computed_key_read_write_doubles_each_field() {
    // makeCumulative-shaped index use without the `last`/null pattern:
    // read obj[c], write obj[c]. Sum after doubling proves both directions.
    let src = "const t = { a: 0.25, c: 0.25, g: 0.5 };\n\
function dbl(table) {\n  for (var c in table) {\n    table[c] = table[c] * 2;\n  }\n}\n\
function sum(table) {\n  let s = 0.0;\n  for (var c in table) {\n    s = s + table[c];\n  }\n  return s;\n}\n\
dbl(t);\nconsole.log(sum(t));\n";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // node: (0.25+0.25+0.5)*2 = 2 -> "2\n"
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}

#[test]
fn make_cumulative_matches_node_byte_for_byte() {
    // The full fasta `makeCumulative` pattern: a null-sentinel key alias
    // (`var last = null`), truthiness-guarded computed read of the prior key
    // (`if (last) table[c] += table[last]`), and the key alias (`last = c`).
    let src = "function makeCumulative(table) {\n  var last = null;\n  for (var c in table) {\n    if (last) table[c] += table[last];\n    last = c;\n  }\n}\n\
function dump(table) {\n  for (var c in table) { console.log(table[c]); }\n}\n\
const t = { a: 0.2, c: 0.3, g: 0.5 };\nmakeCumulative(t);\ndump(t);\n";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // node v26.4.0 cumulative: a=0.2, c=0.5, g=1 -> "0.2\n0.5\n1\n"
    // (console.log(1.0) prints `1`).
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0.2\n0.5\n1\n");
}

#[test]
fn for_in_over_fixed_shape_object_with_bare_key_iterates_once_per_field() {
    // The bare-identifier `for (c in obj)` form (key pre-declared, no
    // `var`/`let`/`const` in the head) — the exact shape fasta's `selectRandom`
    // uses. Proves `ForInLefthand::Expression` lowers end-to-end, not just the
    // declaration form. Key not used yet; only the iteration count is observable.
    let out = run_source(
        "const table = { a: 1, c: 2, g: 3 };\nlet count = 0;\nlet c;\nfor (c in table) {\n  count = count + 1;\n}\nconsole.log(count);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}
