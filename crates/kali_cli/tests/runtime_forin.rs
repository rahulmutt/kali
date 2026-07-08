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
fn for_in_key_in_string_number_ternary_never_yields_garbage() {
    // Value-flow fail-open guard: a for-in key in a ternary arm whose OTHER arm
    // is a string literal (`r < 0 ? c : "?"`) is NOT a repr-lifted string in that
    // position (the ternary arm is not a seed sink), so codegen would emit the
    // raw ordinal. The types oracles now mirror codegen's solved-repr guard
    // (`identifier_repr_is_string`), so this must NOT compile to a bogus string
    // handle: either it matches node OR it fails closed — never garbage.
    let src = "function pick(t, r) {\n  for (var c in t) {\n    if (r < t[c]) return (r < 0.0 ? c : \"?\");\n  }\n  return \"?\";\n}\n\
const t = { a: 0.3, c: 0.6, g: 0.95 };\nconsole.log(pick(t, 0.1));\n";
    let out = run_source(src);
    // node: 0.1<0.3 -> (0.1<0 ? c : "?") -> "?". Accept a byte-identical match OR
    // a fail-closed rejection; reject a garbage-producing success.
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "?\n",
        "for-in key in a string/number ternary must match node or fail closed, not miscompile; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_stored_into_string_array_element_is_fail_closed() {
    // Value-flow fail-open guard: `strArr[i] = c` (element store, not a seed
    // sink) must NOT land in the Spec-3 string-element accept lane storing the
    // raw ordinal. The oracle↔codegen mirror keeps this fail-closed.
    let src = "function collect(t) {\n  let out = new Array(3);\n  out[0] = \"x\"; out[1] = \"y\"; out[2] = \"z\";\n  let i = 0;\n  for (var c in t) { out[i] = c; i = i + 1; }\n  return out.join(\",\");\n}\n\
const t = { a: 0.3, c: 0.6, g: 0.95 };\nconsole.log(collect(t));\n";
    let out = run_source(src);
    // node: "a,c,g". Accept a byte-identical match OR a fail-closed rejection;
    // reject a garbage-producing success (raw ordinals joined as a string).
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a,c,g\n",
        "for-in key stored into a string array must match node or fail closed, not miscompile; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_declarator_alias_returned_as_string_is_fail_closed() {
    // Dual-role × alias fail-open guard: a DECLARATOR-init alias `let d = c`
    // used as a string value (`return d`) is NOT materialized by codegen (only
    // a DIRECT seeded key is), so it must fail closed — never leak the raw
    // ordinal as a bogus string handle (the reviewer's exact probe: was `0`).
    let src = "const t = { a: 0.5, c: 0.5 };\n\
function f(tab) { for (var c in tab) { let d = c; return d; } return \"?\"; }\nconsole.log(f(t));\n";
    let out = run_source(src);
    // node: "a". Accept a byte-identical match OR fail-closed; reject garbage.
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "declarator-alias for-in key returned as a string must match node or fail closed, not leak the ordinal; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_assignment_alias_returned_as_string_is_fail_closed() {
    // Assignment-alias form `d = c; return d` — same fail-open class; must fail
    // closed, not leak the raw ordinal.
    let src = "const t = { a: 0.5, c: 0.5 };\n\
function f(tab) { var d; for (var c in tab) { d = c; return d; } return \"?\"; }\nconsole.log(f(t));\n";
    let out = run_source(src);
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "assignment-alias for-in key returned as a string must match node or fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_in_template_literal_is_fail_closed_or_matches_node() {
    // A DIRECT key inside a template interpolation `${c}` is a string escape
    // that is NOT a repr seed sink — must fail closed OR match node, never leak
    // the raw ordinal. (Real templates desugar to `+` chains, which DO
    // materialize the direct key; either outcome is acceptable, garbage is not.)
    let src = "const t = { a: 0.5, c: 0.5 };\n\
function f(tab) { for (var c in tab) { return `${c}`; } return \"?\"; }\nconsole.log(f(t));\n";
    let out = run_source(src);
    // node: "a".
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "for-in key in a template literal must match node or fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_as_general_call_argument_is_fail_closed() {
    // Value-escape class guard: a for-in key passed as a GENERAL (non-console)
    // function argument (`id(c)`) is not materialized across the call boundary
    // and would leak the raw ordinal — must fail closed.
    let src = "const t = { a: 0.5, c: 0.5 };\n\
function id(x) { return x; }\nfunction f(tab) { for (var c in tab) { return id(c); } return \"?\"; }\nconsole.log(f(t));\n";
    let out = run_source(src);
    // node: "a". Match node OR fail-closed; never the raw ordinal.
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "for-in key as a general call argument must match node or fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_as_array_literal_element_is_fail_closed() {
    // Value-escape class guard: an aliased for-in key as an ARRAY-LITERAL
    // element (`let d = c; let a = [d]; return a[0]`) would store the raw
    // ordinal — must fail closed.
    let src = "const t = { a: 0.5, c: 0.5 };\n\
function f(tab) { for (var c in tab) { let d = c; let a = [d]; return a[0]; } return \"?\"; }\nconsole.log(f(t));\n";
    let out = run_source(src);
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "for-in key as an array-literal element must match node or fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_as_object_property_value_is_fail_closed() {
    // Value-escape class guard: a for-in key as an OBJECT-LITERAL property value
    // (`let o = {k: c}; return o.k`) would store the raw ordinal — fail closed.
    let src = "const t = { a: 0.5, c: 0.5 };\n\
function f(tab) { for (var c in tab) { let o = { k: c }; return o.k; } return \"?\"; }\nconsole.log(f(t));\n";
    let out = run_source(src);
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "for-in key as an object-property value must match node or fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

// A for-in-key VALUE nested inside a value-forwarding expression (ternary arm,
// sequence, paren, cast) must be caught structurally at EVERY sink — the reject
// helper recurses through forwarding wrappers to the identifier leaves. Each of
// these leaks the raw ordinal (`0`) pre-fix; node prints `a`.
const TERN: &str = "1 < 2 ? c : d";

fn forin_ternary_case(body_return: &str) -> std::process::Output {
    // `d = c` is an assignment alias (in the ordinal domain); the ternary
    // `1<2?c:d` forwards a non-materializable key value → must fail closed.
    let src = format!(
        "const t = {{ a: 0.5, c: 0.5 }};\n\
function id(x) {{ return x; }}\n\
function f(tab) {{ var d; for (var c in tab) {{ d = c; {body_return} }} return \"?\"; }}\nconsole.log(f(t));\n"
    );
    run_source(&src)
}

#[test]
fn for_in_key_in_ternary_arm_at_return_is_fail_closed() {
    let out = forin_ternary_case(&format!("return ({TERN});"));
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "ternary-arm key at return must fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_in_ternary_arm_at_console_arg_is_fail_closed() {
    let out = forin_ternary_case(&format!("console.log({TERN}); return \"?\";"));
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n?\n",
        "ternary-arm key at console arg must fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_in_ternary_arm_at_general_call_arg_is_fail_closed() {
    let out = forin_ternary_case(&format!("return id({TERN});"));
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "ternary-arm key at general call arg must fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_in_ternary_arm_at_array_element_is_fail_closed() {
    let out = forin_ternary_case(&format!("let a = [{TERN}]; return a[0];"));
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "ternary-arm key at array element must fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_in_ternary_arm_at_object_property_is_fail_closed() {
    let out = forin_ternary_case(&format!("let o = {{ k: {TERN} }}; return o.k;"));
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "ternary-arm key at object property must fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_in_nested_ternary_is_fail_closed() {
    // `return (a ? (b ? c : d) : e)` — recursion must reach the doubly-nested leaf.
    let out = forin_ternary_case("return (1 < 2 ? (1 < 2 ? c : d) : d);");
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "nested-ternary key must fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

// Structural default-deny (allowlist): a non-materialized for-in-key value in
// ANY value position OTHER than the four safe ones (computed index, `if` test,
// alias-copy, materialized direct key) rejects E5506 by construction — no
// un-enumerated position can leak. These exercise positions the recursion
// denylist missed (compound-assign target, unary, optional chain, closure).
fn forin_leak_case(body: &str) -> std::process::Output {
    let src = format!(
        "const t = {{ a: 0.5, c: 0.5 }};\n\
function id(x) {{ return x; }}\nfunction f(tab) {{ {body} }}\nconsole.log(f(t));\n"
    );
    run_source(&src)
}

#[test]
fn for_in_key_in_compound_assign_target_value_is_fail_closed() {
    // `return (x += c)` — compound-assign numeric-target value escape.
    let out = forin_leak_case("var x = 0; for (var c in tab) { return (x += c); } return 0;");
    assert!(
        !out.status.success(),
        "x += c value escape must fail closed"
    );
}

#[test]
fn for_in_key_numeric_accumulate_is_fail_closed() {
    // `out += c` where out is numeric — key used as a number, not the ordinal.
    let out = forin_leak_case("var out = 0; for (var c in tab) { out += c; } return out;");
    assert!(!out.status.success(), "out += c must fail closed");
}

#[test]
fn for_in_key_unary_negate_is_fail_closed() {
    let out = forin_leak_case("for (var c in tab) { return -c; } return 0;");
    assert!(!out.status.success(), "-c must fail closed");
}

#[test]
fn for_in_key_optional_chain_is_fail_closed() {
    let out = forin_leak_case("for (var c in tab) { return c?.length; } return 0;");
    assert!(!out.status.success(), "c?.length must fail closed");
}

#[test]
fn for_in_key_closure_capture_is_fail_closed() {
    // `let g = () => c; return g()` — closure captures the key across a function
    // boundary; the ordinal would leak into the closure. Rejected via the
    // scope-chain (capture-aware) for-in-key detection.
    let out = forin_leak_case("for (var c in tab) { let g = () => c; return g(); } return 0;");
    assert!(
        !out.status.success(),
        "closure capture of a for-in key must fail closed"
    );
}

#[test]
fn for_in_key_in_sequence_expression_is_fail_closed() {
    // `return (0, c)` — the comma operator forwards its LAST element.
    let src = "const t = { a: 0.5, c: 0.5 };\n\
function f(tab) { for (var c in tab) { return (0, c); } return \"?\"; }\nconsole.log(f(t));\n";
    let out = run_source(src);
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "sequence-expression key must fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_in_assignment_expression_value_is_fail_closed() {
    // `return (x = c)` — a PLAIN assignment EXPRESSION evaluates to the assigned
    // value, forwarding `c` to the escape. Must fail closed (was `0`). The
    // ordinal-domain STATEMENT `x = c;` (not a sink) stays compiling.
    let src = "const t = { a: 0.5, c: 0.5 };\n\
function f(tab) { var x; for (var c in tab) { return (x = c); } return \"?\"; }\nconsole.log(f(t));\n";
    let out = run_source(src);
    // node: "a". Match node OR fail-closed; never the raw ordinal.
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\n",
        "assignment-expression value forwarding a key must fail closed; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_alias_copy_statement_still_compiles() {
    // Guardrail: the ordinal-domain alias-copy STATEMENT `x = c;` (not a value
    // escape) must NOT be rejected — proves the assignment-value recursion above
    // does not over-reject the statement form.
    let src = "const t = { a: 0.5, c: 0.5 };\n\
function f(tab) { var x; for (var c in tab) { x = c; } }\nf(t); console.log(\"ok\");\n";
    let out = run_source(src);
    assert!(
        out.status.success() && String::from_utf8_lossy(&out.stdout) == "ok\n",
        "alias-copy statement `x = c;` must compile (ordinal domain, not over-rejected); stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_declarator_alias_as_logical_operand_is_fail_closed() {
    // `&&`/`||`/`??`/`!` value-select or invert truthiness on the raw ordinal.
    // A DECLARATOR alias (`let d = c`) must reject there too — `reject_forin_key_
    // boolean_operand` keys on the full value-provenance predicate, not bare
    // `for_in_key_shape` (which misses declarator aliases). Pre-fix leaked `0`.
    let src = "const t = { a: 0.5, c: 0.5 };\n\
function f(tab) { for (var c in tab) { let d = c; return (d && 1); } return 99; }\nconsole.log(f(t));\n";
    let out = run_source(src);
    // node: 1 (d is truthy). Match node OR fail-closed; never the raw ordinal 0.
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "1\n",
        "declarator alias as a logical operand must fail closed, not leak the ordinal; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_index_through_ternary_is_not_over_rejected() {
    // POSITIVE guard: a for-in key in INDEX position inside a ternary arm
    // (`tab[c]`) is a FIELD value, NOT a key value — the reject must NOT recurse
    // into a MemberExpression. `return (cond ? tab[c] : tab.c)` must match node
    // (0.25), proving field values through a ternary stay accepted.
    let src = "const t = { a: 0.25, c: 0.75 };\n\
function f(tab) { for (var c in tab) { return (1 < 2 ? tab[c] : tab.c); } return tab.a; }\nconsole.log(f(t));\n";
    let out = run_source(src);
    assert!(
        out.status.success() && String::from_utf8_lossy(&out.stdout) == "0.25\n",
        "field value through a ternary must stay accepted and match node; got stdout={:?} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn for_in_key_alias_index_through_ternary_is_not_over_rejected_by_e5506() {
    // The reviewer's exact positive form `return (cond ? tab[c] : tab[d])` (both
    // arms field reads, `d` an assignment-alias index) must NOT be rejected by
    // the for-in-key-VALUE gate — both arms are index positions. It hits a
    // SEPARATE pre-existing codegen gap (alias dynamic-read in a ternary-return
    // → E4201), so assert it either matches node OR fails WITHOUT the value-gate
    // E5506 (never over-rejected, never a wrong-answer success).
    let src = "const t = { a: 0.25, c: 0.75 };\n\
function f(tab) { var d; for (var c in tab) { d = c; return (1 < 2 ? tab[c] : tab[d]); } return tab.a; }\nconsole.log(f(t));\n";
    let out = run_source(src);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("is only usable as a computed index"),
        "field values through a ternary must NOT be over-rejected by the for-in-key value gate; stderr={stderr}"
    );
    assert!(
        !out.status.success() || String::from_utf8_lossy(&out.stdout) == "0.25\n",
        "must match node or fail closed, never a wrong-answer success; got stdout={:?}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// ---------------------------------------------------------------------------
// Task 6: fail-closed matrix. Every out-of-scope for..in shape must FAIL CLOSED
// (non-zero exit / E5506), never miscompile.
// ---------------------------------------------------------------------------

#[test]
fn for_in_over_array_is_rejected() {
    let out =
        run_source("const a = new Array(2);\na[0]=1;\nfor (var c in a) { console.log(c); }\n");
    assert!(!out.status.success(), "for..in over an array must reject");
}

#[test]
fn computed_key_from_non_forin_string_is_rejected() {
    // A plain runtime string key not derived from for..in over `t` -> Spec 4b.
    let out = run_source(
        "function f(t, k) { return t[k]; }\nconst t = { a: 1.0, c: 2.0 };\nconsole.log(f(t, \"a\"));\n",
    );
    assert!(
        !out.status.success(),
        "general dynamic string key must reject"
    );
}

#[test]
fn string_value_into_object_field_is_rejected() {
    let out = run_source(
        "function f(table, s) { for (var c in table) { table[c] = s; } }\nconst t = { a: 1.0 };\nf(t, \"x\");\n",
    );
    assert!(
        !out.status.success(),
        "storing a string into a field must reject"
    );
}

#[test]
fn for_in_key_indexing_a_different_object_is_rejected() {
    let out = run_source(
        "function f(t, u) { for (var c in t) { console.log(u[c]); } }\nconst t = { a: 1.0 };\nconst u = { a: 9.0, b: 8.0 };\nf(t, u);\n",
    );
    assert!(
        !out.status.success(),
        "key used against a different object must reject"
    );
}

#[test]
fn for_in_over_mixed_repr_shape_is_rejected() {
    // Non-uniform field reprs: dynamic index can't pick a per-field type.
    let out = run_source(
        "function f(table) { for (var c in table) { console.log(table[c]); } }\nconst t = { a: 1, c: 2.5 };\nf(t);\n",
    );
    assert!(
        !out.status.success(),
        "mixed-repr shape dynamic access must reject"
    );
}

// ---------------------------------------------------------------------------
// Task 6 (controller handoff H2): a for-in-key/alias identifier used as a
// while / for / do-while condition or a ternary TEST lowers via default `!= 0`
// truthiness (`-1` null sentinel reads TRUTHY) with NO diagnostic — a
// fail-OPEN in the same class as the `!`/`&&`/`||` rejects but in loop/ternary
// test positions. fasta uses NONE of these, so reject fail-closed (E5506).
// `if (last)` (makeCumulative) must STILL compile (it is an `if`, lowered
// `>= 0`), and a normal while/for/ternary on a NON-for-in binding too.
// ---------------------------------------------------------------------------

#[test]
fn forin_key_alias_as_while_condition_is_rejected() {
    let out = run_source(
        "function f(table) {\n  var last = null;\n  for (var c in table) {\n    last = c;\n    while (last) { break; }\n  }\n}\nconst t = { a: 1, c: 2, g: 3 };\nf(t);\n",
    );
    assert!(
        !out.status.success(),
        "expected a fail-closed rejection of `while (last)` on a for-in-key alias"
    );
}

#[test]
fn forin_key_alias_as_for_condition_is_rejected() {
    let out = run_source(
        "function f(table) {\n  var last = null;\n  for (var c in table) {\n    last = c;\n    for (; last; ) { break; }\n  }\n}\nconst t = { a: 1, c: 2, g: 3 };\nf(t);\n",
    );
    assert!(
        !out.status.success(),
        "expected a fail-closed rejection of `for (; last;)` on a for-in-key alias"
    );
}

#[test]
fn forin_key_alias_as_do_while_condition_is_rejected() {
    let out = run_source(
        "function f(table) {\n  var last = null;\n  for (var c in table) {\n    last = c;\n    do { break; } while (last);\n  }\n}\nconst t = { a: 1, c: 2, g: 3 };\nf(t);\n",
    );
    assert!(
        !out.status.success(),
        "expected a fail-closed rejection of `do..while (last)` on a for-in-key alias"
    );
}

#[test]
fn forin_key_alias_as_ternary_test_is_rejected() {
    let out = run_source(
        "function f(table) {\n  var last = null;\n  for (var c in table) {\n    last = c;\n    let z = last ? 1 : 2;\n    console.log(z);\n  }\n}\nconst t = { a: 1, c: 2, g: 3 };\nf(t);\n",
    );
    assert!(
        !out.status.success(),
        "expected a fail-closed rejection of `last ? a : b` on a for-in-key alias"
    );
}

#[test]
fn normal_while_and_ternary_on_non_forin_binding_still_compile() {
    // Guard against over-rejection: a plain while-loop condition and a ternary
    // test on ordinary (non-for-in-key) bindings must STILL compile.
    let out = run_source(
        "let i = 0;\nwhile (i < 3) { i = i + 1; }\nlet flag = 1;\nlet z = flag ? 7 : 8;\nconsole.log(i);\nconsole.log(z);\n",
    );
    assert!(
        out.status.success(),
        "normal while/ternary on non-for-in bindings must still compile; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n7\n");
}

#[test]
fn fasta_make_cumulative_select_random_capstone_matches_node_byte_for_byte() {
    // Task 7 capstone: the exact fasta `makeCumulative` + `random` (LCG) +
    // `selectRandom` shell over the IUB table, run for 20 iterations. This is
    // the true end-to-end integration of Tasks 1-6 plus the two enabling
    // fixes: multi-declarator `var r = random(1.0), c;` (binds both r and c)
    // and a mutable module-scope scalar global `var rngLast = 42;` read+written
    // by `random()`. Exercises the declaration-form for..in in makeCumulative
    // (null-sentinel key alias, computed read/write both directions) plus the
    // BARE-form for..in in selectRandom (key used as a computed index AND
    // returned as a string in the same loop).
    // Golden captured from `node` v26.4.0 running these exact bytes.
    let src = r#"function makeCumulative(table) {
  var last = null;
  for (var c in table) {
    if (last) table[c] += table[last];
    last = c;
  }
}
var rngLast = 42;
function random(max) {
  rngLast = (rngLast * 3877 + 29573) % 139968;
  return (max * rngLast) / 139968;
}
function selectRandom(table) {
  var r = random(1.0), c;
  for (c in table) if (r < table[c]) return c;
  return c;
}
var iub = { a: 0.27, c: 0.12, g: 0.12, t: 0.27, B: 0.02, D: 0.02, H: 0.02 };
makeCumulative(iub);
var out = "";
for (var i = 0; i < 20; i = i + 1) out += selectRandom(iub);
console.log(out);
"#;
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let expected = "cttBtatcatatgctaHggH\n";
    assert_eq!(String::from_utf8_lossy(&out.stdout), expected);
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
