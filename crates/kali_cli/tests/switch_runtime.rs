use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_js(source: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

const S: &str = "function s(x) {\n\
                   switch (x) {\n\
                     case 10: return \"A\";\n\
                     case 20: return \"B\";\n\
                     default: return \"D\";\n\
                   }\n\
                 }\n";

// Anti-spot-check: s(10) is EXCLUDED on purpose. The pre-fix lowering returned
// "A" for every truthy discriminant, so s(10) agreed with node by coincidence
// and proves nothing. Every assertion below uses a discriminant the broken
// lowering got wrong, and the answers must vary with the input.
#[test]
fn numeric_switch_selects_the_matching_clause() {
    assert_eq!(run_js(&format!("{S}console.log(\"v=\" + s(20));")), "v=B\n");
}
#[test]
fn numeric_switch_falls_to_default_on_no_match() {
    assert_eq!(run_js(&format!("{S}console.log(\"v=\" + s(40));")), "v=D\n");
}
#[test]
fn numeric_switch_handles_a_zero_discriminant() {
    // The pre-fix lowering truthiness-tested the discriminant, so 0 took the
    // else branch and returned clause 1's "B". This is the cell that proves
    // the discriminant is compared, not tested for truth.
    assert_eq!(run_js(&format!("{S}console.log(\"v=\" + s(0));")), "v=D\n");
}
#[test]
fn numeric_switch_reaches_the_third_clause() {
    // Clauses beyond the second were never emitted at all (the Tier-1 half).
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case 1: return \"one\";\n\
                   case 2: return \"two\";\n\
                   case 3: return \"three\";\n\
                   case 4: return \"four\";\n\
                   default: return \"other\";\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(3));")), "v=three\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(4));")), "v=four\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(9));")), "v=other\n");
}
#[test]
fn numeric_switch_evaluates_the_discriminant_exactly_once() {
    // If the chain re-emitted the discriminant per clause test, `hits` would
    // count once per comparison instead of once per call.
    //
    // Fix round 1, two changes from the original fixture:
    //
    // 1. The discriminant is `d(2)`, a call with a LITERAL argument -- not
    //    `d(x)`, a passthrough of `s`'s own parameter. A bare-identifier call
    //    argument is never scalar EVIDENCE (see
    //    `crates/kali_types/src/repr_infer.rs:4099-4107`), so `d(x)`'s param
    //    would be vetoed under the stricter `return_is_proven_numeric` proof
    //    `is_provable_i64_scalar`'s call arm now requires.
    //
    // 2. `d` returns `x + 0`, not a bare `x`. This is NOT cosmetic: ANY
    //    function whose return is a BARE identifier is unconditionally
    //    excluded from ever earning `return_is_proven_numeric`, regardless of
    //    how provably-scalar that identifier is -- `repr_infer.rs`'s
    //    `visit_statement` (`ReturnStatement` arm, ~line 2291-2300)
    //    unconditionally records ANY `return <identifier>;` into
    //    `array_binding_returns` (the "might dynamically be an array" guard),
    //    and the `numeric_return_candidates` finalize step (~line 5320-5327)
    //    skips a function with ANY such entry outright, before it ever
    //    reaches the params-proven check. `return x + 0;` is arithmetic, not
    //    a bare identifier, so it never enters that registry and the
    //    positive proof can actually fire. Measured empirically (not
    //    assumed): `return x;` here made `d` fail `is_provable_i64_scalar`
    //    even with a literal argument at every call site; `return x + 0;`
    //    passes, and both kali and node agree byte-for-byte
    //    (`/workspace/.cache/scratch/eval_once2.js`, deleted after
    //    verification: `v=two` / `hits=1` on both engines).
    let src = "var hits = 0;\n\
               function d(x) { hits = hits + 1; return x + 0; }\n\
               function s() {\n\
                 switch (d(2)) {\n\
                   case 1: return \"one\";\n\
                   case 2: return \"two\";\n\
                   default: return \"other\";\n\
                 }\n\
               }\n\
               s();\n\
               console.log(\"hits=\" + hits);\n";
    assert_eq!(run_js(src), "hits=1\n");
}
// Moved from `switch_fail_closed.rs`'s `switch_is_fail_closed_not_silently_wrong`:
// this exact shape (numeric discriminant, `case 10`/`case 20`/`default`, all
// `return`) is now ADMITTED and correct, so it belongs here among the
// correctness tests rather than in a file named "fail_closed".
#[test]
fn numeric_switch_is_correct_for_the_shape_that_was_previously_denied() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case 10: return \"A\";\n\
                   case 20: return \"B\";\n\
                   default: return \"D\";\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(20));")), "v=B\n");
}
#[test]
fn numeric_switch_selects_correctly_with_a_negative_case_test() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case -1: return \"neg\";\n\
                   case 1: return \"pos\";\n\
                   default: return \"other\";\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(-1));")), "v=neg\n");
}

// ---------------------------------------------------------------------------
// R-35 Task 8: string discriminants.
// ---------------------------------------------------------------------------

const SS: &str = "function s(x) {\n\
                    switch (x) {\n\
                      case \"a\": return 1;\n\
                      case \"b\": return 2;\n\
                      default: return 3;\n\
                    }\n\
                  }\n";

#[test]
fn string_switch_selects_the_matching_clause() {
    assert_eq!(run_js(&format!("{SS}console.log(\"v=\" + s(\"b\"));")), "v=2\n");
}
#[test]
fn string_switch_falls_to_default() {
    assert_eq!(run_js(&format!("{SS}console.log(\"v=\" + s(\"z\"));")), "v=3\n");
}
#[test]
fn string_switch_compares_by_content_not_handle() {
    // Two equal strings built differently must select the same clause. If the
    // comparison were handle identity rather than __streq content equality,
    // the runtime-built string would miss every case and fall to default.
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case \"ab\": return 1;\n\
                   default: return 3;\n\
                 }\n\
               }\n\
               var built = \"a\" + \"b\";\n\
               console.log(\"v=\" + s(built));\n";
    assert_eq!(run_js(src), "v=1\n");
}

// Anti-spot-check on the string axis: the answer must VARY with the
// discriminant, and each of these three is a different clause of the same
// program (first case, second case, default).
#[test]
fn string_switch_selects_the_first_clause() {
    assert_eq!(run_js(&format!("{SS}console.log(\"v=\" + s(\"a\"));")), "v=1\n");
}

// The empty string is the string-axis analogue of the `0` discriminant cell: a
// truthiness-testing lowering would take the else branch for it and return the
// wrong clause. `""` matching `case "":` proves the discriminant is COMPARED.
#[test]
fn string_switch_handles_an_empty_string_discriminant() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case \"\": return 1;\n\
                   case \"b\": return 2;\n\
                   default: return 3;\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(\"\"));")), "v=1\n");
}

// A non-ASCII discriminant: `__streq` compares BYTES, so a multi-byte literal
// must still select its own clause and not collide with a same-length-in-chars
// neighbour.
#[test]
fn string_switch_handles_a_non_ascii_discriminant() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case \"\u{e9}\": return 1;\n\
                   case \"b\": return 2;\n\
                   default: return 3;\n\
                 }\n\
               }\n";
    assert_eq!(
        run_js(&format!("{src}console.log(\"v=\" + s(\"\u{e9}\"));")),
        "v=1\n"
    );
}

// Same-length, same-prefix cases: a length-only or prefix-only comparison
// would pick the wrong clause. `"alphb"` must not select `case "alpha"`.
#[test]
fn string_switch_distinguishes_same_length_cases() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case \"alpha\": return 1;\n\
                   case \"alphb\": return 2;\n\
                   default: return 3;\n\
                 }\n\
               }\n";
    assert_eq!(
        run_js(&format!("{src}console.log(\"v=\" + s(\"alphb\"));")),
        "v=2\n"
    );
}

// A module-scope `const` read from inside the function, resolved through the
// fold-alias before the repr lookup. Scope is a required test axis: this is
// the module-scope binding half, the tests above are the parameter half.
#[test]
fn string_switch_on_a_module_const_binding() {
    let src = "const d = \"b\";\n\
               function s() {\n\
                 switch (d) {\n\
                   case \"a\": return 1;\n\
                   case \"b\": return 2;\n\
                   default: return 3;\n\
                 }\n\
               }\n\
               console.log(\"v=\" + s());\n";
    assert_eq!(run_js(src), "v=2\n");
}

// A function-local `var` binding -- the function-scope half of the same axis.
#[test]
fn string_switch_on_a_function_local_binding() {
    let src = "function s() {\n\
                 var d = \"b\";\n\
                 switch (d) {\n\
                   case \"a\": return 1;\n\
                   case \"b\": return 2;\n\
                   default: return 3;\n\
                 }\n\
               }\n\
               console.log(\"v=\" + s());\n";
    assert_eq!(run_js(src), "v=2\n");
}

// A RUNTIME-produced discriminant that is not an interned literal at all:
// `t.substring(1, 2)` allocates a fresh handle. Handle identity would miss
// every clause; `__streq` content equality selects `case "b"`. This is the
// same guarantee as `string_switch_compares_by_content_not_handle` but through
// a different string producer, so the two cannot both pass by coincidence.
#[test]
fn string_switch_matches_a_runtime_substring_discriminant() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case \"b\": return 1;\n\
                   default: return 3;\n\
                 }\n\
               }\n\
               var t = \"abc\";\n\
               console.log(\"v=\" + s(t.substring(1, 2)));\n";
    assert_eq!(run_js(src), "v=1\n");
}

// The string-axis twin of `numeric_switch_evaluates_the_discriminant_exactly
// once`, in BOTH directions: a matching discriminant and a default-taking one.
// A chain that re-emitted the discriminant per clause test would call `d` once
// per clause, and the default-taking variant is the cell that exposes it
// (every clause test runs before the default is reached).
#[test]
fn string_switch_evaluates_the_discriminant_exactly_once() {
    let base = "var hits = 0;\n\
                function d(x) { hits = hits + 1; return \"k\" + x; }\n";
    let matching = format!(
        "{base}function s() {{\n\
           switch (d(2)) {{\n\
             case \"k1\": return 1;\n\
             case \"k2\": return 2;\n\
             default: return 3;\n\
           }}\n\
         }}\n\
         s();\n\
         console.log(\"hits=\" + hits);\n"
    );
    assert_eq!(run_js(&matching), "hits=1\n");
    let defaulting = format!(
        "{base}function s() {{\n\
           switch (d(9)) {{\n\
             case \"k1\": return 1;\n\
             case \"k2\": return 2;\n\
             default: return 3;\n\
           }}\n\
         }}\n\
         s();\n\
         console.log(\"hits=\" + hits);\n"
    );
    assert_eq!(run_js(&defaulting), "hits=1\n");
}

// A call-expression discriminant whose callee's return repr is proven String
// (the `is_string_valued` call arm), selecting a non-first clause.
#[test]
fn string_switch_on_a_string_returning_call_discriminant() {
    let src = "function g(n) { return \"k\" + n; }\n\
               function s() {\n\
                 switch (g(2)) {\n\
                   case \"k1\": return 1;\n\
                   case \"k2\": return 2;\n\
                   default: return 3;\n\
                 }\n\
               }\n\
               console.log(\"v=\" + s());\n";
    assert_eq!(run_js(src), "v=2\n");
}

// Fix round 1: `export` is NOT an escape trigger. `Statement::ExportNamed` is
// an explicit no-op in the walk that builds `escaping_function_names`, so an
// exported switch function IS admitted — three doc comments claimed otherwise
// and were corrected. This pins the ACTUAL behaviour so the next implementer
// reads a test rather than prose. It is safe only while a cross-module
// imported call returns `0` wholesale; if that ever changes, `export` must
// become an escape trigger and this test must flip to fail-closed.
#[test]
fn an_exported_function_is_admitted_because_export_is_not_an_escape() {
    let src = "export function s(x) {\n\
                 switch (x) {\n\
                   case \"a\": return 1;\n\
                   case \"b\": return 2;\n\
                   default: return 3;\n\
                 }\n\
               }\n\
               console.log(\"v=\" + s(\"b\"));\n";
    assert_eq!(run_js(src), "v=2\n");
}

// The numeric axis must be UNCHANGED by the string widening: the same function
// answers correctly for two different string call sites in one program, and
// duplicate cases stay first-match-wins.
#[test]
fn string_switch_is_correct_across_two_call_sites() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case \"a\": return 1;\n\
                   case \"b\": return 2;\n\
                   default: return 3;\n\
                 }\n\
               }\n\
               console.log(\"v=\" + s(\"b\"));\n\
               console.log(\"w=\" + s(\"q\"));\n";
    assert_eq!(run_js(src), "v=2\nw=3\n");
}
#[test]
fn a_duplicate_string_case_is_first_match_wins() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case \"a\": return 1;\n\
                   case \"a\": return 2;\n\
                   default: return 3;\n\
                 }\n\
               }\n\
               console.log(\"v=\" + s(\"a\"));\n";
    assert_eq!(run_js(src), "v=1\n");
}
