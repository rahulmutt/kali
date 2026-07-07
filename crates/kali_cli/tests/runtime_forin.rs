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
fn transitive_forin_key_alias_two_levels_matches_node() {
    // Two-level alias chain `c -> last -> y`: `y` must be recognized as a
    // for-in-key alias transitively, so `if (y)` lowers to `y >= 0` (NOT `!= 0`,
    // which would treat the first-field ordinal 0 as falsy and skip it) and
    // `table[y]` routes through the dynamic slot lane. Doubles every field.
    let src = "function doubleEach(table) {\n  var last = null;\n  var y = null;\n  for (var c in table) {\n    last = c;\n    y = last;\n    if (y) table[c] += table[y];\n  }\n}\n\
function dump(table) {\n  for (var c in table) { console.log(table[c]); }\n}\n\
const t = { a: 0.2, c: 0.3, g: 0.5 };\ndoubleEach(t);\ndump(t);\n";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // node v26.4.0: each field doubled -> "0.4\n0.6\n1\n" (console.log(1.0) is `1`).
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0.4\n0.6\n1\n");
}

#[test]
fn forin_key_alias_under_not_is_rejected() {
    // `!last` on a for-in-key alias would invert the null-sentinel: reject
    // fail-closed rather than miscompile (valid JS in node, out of scope here).
    let out = run_source(
        "function f(table) {\n  var last = null;\n  for (var c in table) {\n    last = c;\n    if (!last) { console.log(1); }\n  }\n}\nconst t = { a: 1, c: 2, g: 3 };\nf(t);\n",
    );
    assert!(
        !out.status.success(),
        "expected a fail-closed rejection of `!last` on a for-in-key alias"
    );
}

#[test]
fn forin_key_alias_under_logical_and_is_rejected() {
    let out = run_source(
        "function f(table) {\n  var last = null;\n  for (var c in table) {\n    last = c;\n    if (last && true) { console.log(1); }\n  }\n}\nconst t = { a: 1, c: 2, g: 3 };\nf(t);\n",
    );
    assert!(
        !out.status.success(),
        "expected a fail-closed rejection of `last && x` on a for-in-key alias"
    );
}

#[test]
fn forin_key_alias_under_logical_or_is_rejected() {
    let out = run_source(
        "function f(table) {\n  var last = null;\n  for (var c in table) {\n    last = c;\n    if (last || false) { console.log(1); }\n  }\n}\nconst t = { a: 1, c: 2, g: 3 };\nf(t);\n",
    );
    assert!(
        !out.status.success(),
        "expected a fail-closed rejection of `last || x` on a for-in-key alias"
    );
}

#[test]
fn for_in_key_returned_as_string_matches_node() {
    // selectRandom shape: return the key whose cumulative field first exceeds r.
    // The key `c` is used BOTH as a computed index (`table[c]`, a raw ordinal)
    // AND as a returned string value (`return c`, an interned field-name handle)
    // in the SAME loop — the dual-role crux of Task 5.
    let src = "function selectRandom(table, r) {\n  for (var c in table) {\n    if (r < table[c]) return c;\n  }\n  return \"?\";\n}\n\
const t = { a: 0.3, c: 0.6, g: 0.95 };\n\
console.log(selectRandom(t, 0.1));\nconsole.log(selectRandom(t, 0.5));\nconsole.log(selectRandom(t, 0.9));\n";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // node v26.4.0: 0.1<0.3 -> "a"; 0.5<0.6 -> "c"; 0.9<1.0 -> "g"
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nc\ng\n");
}

#[test]
fn for_in_bare_key_returned_as_string_matches_node() {
    // R1: the BARE `for (c in table)` form (key pre-declared `var c;`, no
    // `var`/`let`/`const` in the head) with the key returned as a string —
    // the exact shape the Task 7 capstone's `selectRandom` uses. Proves the
    // bare-form for-in-key provenance (types + codegen) supports key-as-string.
    let src = "function selectRandom(table, r) {\n  var c;\n  for (c in table) {\n    if (r < table[c]) return c;\n  }\n  return \"?\";\n}\n\
const t = { a: 0.3, c: 0.6, g: 0.95 };\n\
console.log(selectRandom(t, 0.1));\nconsole.log(selectRandom(t, 0.5));\nconsole.log(selectRandom(t, 0.9));\n";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // node v26.4.0: same as the declaration form -> "a\nc\ng\n"
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nc\ng\n");
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
