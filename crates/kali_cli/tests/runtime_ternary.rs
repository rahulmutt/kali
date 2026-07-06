use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    // A per-process AtomicU64 counter makes the slug unique even when two
    // sources share a length (sharing a length previously collided the dir and
    // caused macOS CI temp-slug flakes — repo convention is a counter).
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-ternary-{}-{}-{}",
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
fn int_ternary_selects_branch() {
    let out =
        run_source("let a = 1;\nconsole.log(a > 0 ? 10 : 20);\nconsole.log(a < 0 ? 10 : 20);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "10\n20\n");
}

#[test]
fn float_ternary_selects_and_prints_float() {
    let out = run_source("let a = 1;\nlet x = a > 0 ? 1.5 : 2.5;\nconsole.log(x);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1.5\n");
}

#[test]
fn mixed_int_float_arms_promote_to_float() {
    let out = run_source("let a = 0;\nlet x = a > 0 ? 1.5 : 2;\nconsole.log(x);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}

#[test]
fn string_arms_ternary_prints() {
    let out =
        run_source("let a = 1;\nlet s = \"x\";\nconsole.log(a > 0 ? s + \"1\" : s + \"2\");\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "x1\n");
}

#[test]
fn string_ternary_as_concat_operand_prints() {
    // Handle-leak pin: a string-armed ternary used as a `+` operand must be
    // concatenated as a STRING. Without a ternary arm in `is_string_valued`,
    // `emit_as_string` misclassifies the ternary as numeric and routes its
    // tagged handle through `int_to_string` — printing the raw handle bits
    // (e.g. "-9223354375949254654!") with exit 0 and no diagnostic.
    let out = run_source(
        "let c = 1;\nlet x = \"z\";\nconsole.log((c > 0 ? \"a\" + x : \"b\" + x) + \"!\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "az!\n");
}

#[test]
fn string_ternary_equality_is_rejected() {
    // Fail-closed pin: `==` on a string-armed ternary must reject, not
    // handle-compare. Pre-fix this silently printed the WRONG branch ("2":
    // two equal-valued concat results have different handles) with exit 0;
    // the `is_string_valued` ternary arm makes `is_runtime_concat_string`'s
    // fallback classify the ternary as a runtime string, tripping the
    // equality gate in `emit_binary`.
    let out = run_source(
        "let c = 1;\nlet x = \"z\";\nif ((c > 0 ? \"a\" + x : \"b\" + x) == \"az\") { console.log(1); } else { console.log(2); }\n",
    );
    assert!(
        !out.status.success(),
        "string-armed ternary == must be rejected, not compared by handle"
    );
}

#[test]
fn only_taken_arm_evaluates() {
    // Laziness pin: the untaken arm's side effect must not run.
    //
    // ADAPTED from the brief's module-`let` mutation form (`let n = 0; function
    // inc() { n = n + 1; ... }`). That form is rejected by a PRE-EXISTING E5506
    // gate on module-binding read/write from a function — it fails to compile
    // even when `inc` is called directly, so it is unrelated to the ternary and
    // was never a valid RED/GREEN target. The laziness property the brief means
    // to pin is preserved here with a compilable observable side effect: the
    // untaken arm calls `boom()`, whose `console.log(999)` must NOT appear. If
    // both arms evaluated, stdout would be "999\n5\n"; only the taken arm gives
    // "5\n".
    let out = run_source(
        "function boom() { console.log(999);\nreturn 1; }\nlet a = 1;\nlet x = a > 0 ? 5 : boom();\nconsole.log(x);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5\n");
}

#[test]
fn nested_ternary_selects() {
    let out = run_source("let a = 2;\nconsole.log(a == 1 ? 10 : a == 2 ? 20 : 30);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "20\n");
}

#[test]
fn string_and_number_arms_are_rejected() {
    // Repr conflict (merge_nodes) or codegen guard — either way: no compile.
    let out = run_source("let a = 1;\nlet s = \"x\";\nlet v = a > 0 ? s : 5;\nconsole.log(v);\n");
    assert!(
        !out.status.success(),
        "string/number arm mix must be rejected"
    );
}

#[test]
fn string_and_float_arms_are_rejected() {
    // A float-typed result block would promote a handle to f64 — reject.
    let out = run_source("let a = 1;\nlet s = \"x\";\nconsole.log(a > 0 ? s + \"!\" : 1.5);\n");
    assert!(
        !out.status.success(),
        "string/float arm mix must be rejected"
    );
}

#[test]
fn length_of_non_ascii_literal_armed_ternary_is_rejected() {
    // Final-review CRITICAL 1: codegen's `is_string_valued` ternary arm outran
    // the types-side string predicates (all ternary-blind), so the `.length`
    // gate never classified a ternary receiver as a string and codegen emitted
    // `handle & 0xFFFF_FFFF` (a BYTE count: HEAD printed 6, node 5). The types
    // predicates now carry a ConditionalExpression arm; a non-ASCII-armed
    // ternary `.length` fails closed. `expression_repr_is_ascii_string` stays
    // ternary-blind, so EVERY string-armed ternary `.length` rejects.
    let out = run_source("let c = 1;\nconsole.log((c > 0 ? \"héllo\" : \"x\").length);\n");
    assert!(
        !out.status.success(),
        "non-ASCII-armed ternary .length must be rejected, not byte-counted"
    );
}

#[test]
fn length_of_ascii_armed_ternary_is_rejected_fail_closed() {
    // Decision pin: `expression_repr_is_ascii_string` stays ternary-blind, so
    // even an ALL-ASCII-armed ternary `.length` rejects as unprovable-ASCII
    // (fail-closed, never fail-open) rather than becoming a supported shape.
    // If a precise `ascii(cons) && ascii(alt)` arm is added later, this pin
    // flips to a green per-arm-length expectation.
    let out = run_source("let c = 1;\nconsole.log((c > 0 ? \"abcd\" : \"wx\").length);\n");
    assert!(
        !out.status.success(),
        "ASCII-armed ternary .length is pinned as a REJECT while the ASCII predicate is ternary-blind"
    );
}

#[test]
fn length_of_runtime_string_var_armed_ternary_is_rejected() {
    // CRITICAL 1 through runtime string variables: both arms are non-ASCII
    // runtime string bindings. Same fail-closed rejection.
    let out = run_source(
        "let c = 1;\nlet a = \"héllo\";\nlet b = \"x\";\nconsole.log((c > 0 ? a : b).length);\n",
    );
    assert!(
        !out.status.success(),
        "runtime-string-var-armed ternary .length must be rejected"
    );
}

#[test]
fn storing_tainted_concat_armed_ternary_into_element_is_rejected() {
    // Final-review CRITICAL 2: the F1 store gate was bypassed by a ternary
    // wrapping a runtime string. `t` is a tainted concat; the ternary must be
    // classified as a runtime string value so the element store fails closed
    // (HEAD compiled silently and printed 0; node prints the string).
    let out = run_source(
        "let c = 1;\nlet x = \"x\";\nlet t = x + \"y\";\nlet arr = [0];\narr[0] = c > 0 ? t : t;\nconsole.log(arr[0]);\n",
    );
    assert!(
        !out.status.success(),
        "ternary-wrapped runtime-string element store must be rejected"
    );
}

#[test]
fn filling_with_tainted_concat_armed_ternary_is_rejected() {
    // CRITICAL 2 via Array.prototype.fill.
    let out = run_source(
        "let c = 1;\nlet x = \"x\";\nlet t = x + \"y\";\nlet a = [0, 0];\na.fill(c > 0 ? t : t);\nconsole.log(a[0]);\n",
    );
    assert!(
        !out.status.success(),
        "fill with a ternary-wrapped runtime string must be rejected"
    );
}

#[test]
fn storing_substring_armed_ternary_into_element_is_rejected() {
    // CRITICAL 2: one arm is a runtime substring slice; the store must reject.
    let out = run_source(
        "let a2 = \"GGCC\";\nlet i = 1;\nlet arr2 = [0];\narr2[0] = i > 0 ? a2.substring(0, i) : a2;\nconsole.log(arr2[0]);\n",
    );
    assert!(
        !out.status.success(),
        "ternary-selected substring element store must be rejected"
    );
}

#[test]
fn storing_all_substring_armed_ternary_into_element_is_rejected() {
    // Re-review residual Critical: when EVERY arm is a substring member-call,
    // neither `expression_is_string_typed` nor `operand_repr_is_string`
    // recognizes the arms (no member-call arm in either), so only
    // `expression_is_runtime_string_value`'s own ternary recursion reaches its
    // substring fallthrough. Pre-fix this compiled silently and printed 0.
    let out = run_source(
        "let a = \"GGCC\";\nlet i = 1;\nlet arr = [0];\narr[0] = i > 0 ? a.substring(0, i) : a.substring(i);\nconsole.log(arr[0]);\n",
    );
    assert!(
        !out.status.success(),
        "all-substring-armed ternary element store must be rejected"
    );
}

#[test]
fn filling_with_all_substring_armed_ternary_is_rejected() {
    // Same residual Critical via Array.prototype.fill (pre-fix: printed 0).
    let out = run_source(
        "let a = \"GGCC\";\nlet i = 1;\nlet arr = [0];\narr.fill(i > 0 ? a.substring(0, i) : a.substring(i));\nconsole.log(arr[0]);\n",
    );
    assert!(
        !out.status.success(),
        "fill with an all-substring-armed ternary must be rejected"
    );
}

#[test]
fn storing_all_substring_armed_ternary_into_field_is_rejected() {
    // Same residual Critical via an object-field store. Pre-fix the field lane
    // happened to print the right value, but the design (§7 row 5) mandates a
    // compile error — the gate must fire, not rely on luck.
    let out = run_source(
        "let a = \"GGCC\";\nlet i = 1;\nlet o = { v: 1 };\no.v = i > 0 ? a.substring(0, i) : a.substring(i);\nconsole.log(o.v);\n",
    );
    assert!(
        !out.status.success(),
        "all-substring-armed ternary field store must be rejected"
    );
}

#[test]
fn ternary_of_substrings_in_read_position_prints() {
    // MUST-STAY-GREEN companion (reviewer's p20): a ternary of substring calls
    // in READ position (console.log) is supported — the store/fill gates must
    // not leak into read positions.
    let out = run_source(
        "let a = \"abcd\";\nlet c = 1;\nlet i = 1;\nconsole.log(c > 0 ? a.substring(i) : a.substring(0, i));\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "bcd\n");
}

#[test]
fn ternary_in_never_called_function_still_compiles() {
    let out = run_source("function unused(a) { return a > 0 ? 1 : 2; }\nconsole.log(7);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
}
