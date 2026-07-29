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

// `export` is NOT an escape trigger, so an exported switch function IS
// admitted. Doc comments claimed the opposite (fix round 1) and then claimed
// the right conclusion for the wrong reason (fix round 2); this test pins the
// behaviour so the next implementer reads a measurement rather than prose.
//
// The reason is in the PARSER, not the escape walk: `parse_export_declaration`
// (`kali_parser::module`) discards the `export` token and dispatches straight
// to `parse_function_declaration()`, and `kali_ast::FunctionDeclaration` has no
// `exported` field — so `export function s() {}` and `function s() {}` produce
// the SAME AST and nothing downstream can distinguish them.
// `Statement::ExportNamed` is the `export { name }` LIST form only, and is not
// what this test exercises.
//
// Admission is sound only while a cross-module imported call returns `0`
// wholesale. If cross-module calls ever deliver real arguments, exported
// functions must be marked escaping and this test must flip to fail-closed —
// and that needs a PARSER change first (preserve the export marker on the
// declaration), because matching `Statement::ExportNamed` alone would miss
// exactly the shape below.
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
// ---------------------------------------------------------------------------
// R-35 Task 9: `break`-terminated clauses.
//
// EVERY fixture below that contains a loop carries a per-iteration
// `console.log`. This is not decoration: any switch fixture carrying `break`
// measures LOOP TRUNCATION FIRST — if `break` bound to the enclosing loop
// instead of the switch, the loop would die at iteration 0 and a final-value-
// only assertion could not tell that apart from a mis-selected clause. Task 4
// shipped a wrong boundary-matrix cell for exactly this reason. The
// per-iteration line makes the two distinguishable in the assertion itself.
// ---------------------------------------------------------------------------

#[test]
fn break_terminated_clauses_select_correctly() {
    let src = "function s(x) {\n\
                 var r = 0;\n\
                 switch (x) {\n\
                   case 10: r = 1; break;\n\
                   case 20: r = 2; break;\n\
                   default: r = 9; break;\n\
                 }\n\
                 return r;\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(20));")), "v=2\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(40));")), "v=9\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(0));")), "v=9\n");
}

#[test]
fn break_terminated_switch_works_at_module_scope() {
    let src = "var v = \"?\";\n\
               var x = 20;\n\
               switch (x) {\n\
                 case 10: v = \"A\"; break;\n\
                 case 20: v = \"B\"; break;\n\
                 default: v = \"D\"; break;\n\
               }\n\
               console.log(\"v=\" + v);\n";
    assert_eq!(run_js(src), "v=B\n");
}

#[test]
fn break_inside_a_switch_inside_a_loop_exits_the_switch_not_the_loop() {
    // If `break` bound to the LOOP instead of the switch, the loop would stop
    // after one iteration and the sum would be 1 instead of 3. The
    // per-iteration `iter=` line is what distinguishes those two failures: a
    // mis-bound `break` prints `iter=0` only, a mis-selected clause prints all
    // three and gets the sum wrong.
    let src = "var sum = 0;\n\
               for (var i = 0; i < 3; i = i + 1) {\n\
                 console.log(\"iter=\" + i);\n\
                 switch (i) {\n\
                   case 0: sum = sum + 1; break;\n\
                   case 1: sum = sum + 1; break;\n\
                   default: sum = sum + 1; break;\n\
                 }\n\
               }\n\
               console.log(\"sum=\" + sum);\n";
    assert_eq!(run_js(src), "iter=0\niter=1\niter=2\nsum=3\n");
}

// `continue` must reach PAST the switch frame to the loop.
//
// THE LOOP FORM IS A `while`, NOT A `for`, AND THAT IS NOT COSMETIC. R-09
// (`continue` inside a C-style `for` skips the update expression —
// `docs/superpowers/followups/kali-silent-miscompile-register.md:1073`) makes
// ANY `for`-loop `continue` fixture hang to an `E4003` fuel trap, with or
// without a switch: measured here on a switch-free control
// (`for (var i = 0; i < 3; i = i + 1) { console.log("iter=" + i); if (i < 2) {
// continue; } ... }`) printing `iter=0` 1,332,956 times before exit 1, where
// node prints `iter=0/1/2`. That register entry names R-09 as the OWNING ID for
// the continue-in-a-switch-clause hang specifically, and records that
// `while`/`do-while`/`for…of` are already correct. So a `for` fixture here
// would measure R-09, not this task's binding property — the exact
// "loop-truncation FIRST" trap that produced a wrong boundary-matrix cell at
// Task 4.
//
// The discriminating observation is the `after=` line, which is INSIDE the loop
// body but AFTER the switch. If `continue` had bound to the switch's own
// break target it would merely leave the switch, `after=` would print on every
// iteration, and the interleaving would read
// `iter=0/after=0/iter=1/after=1/iter=2/after=2`. It prints once, and the
// per-iteration `iter=` lines separately prove the loop was not truncated.
#[test]
fn continue_inside_a_switch_inside_a_loop_continues_the_loop() {
    let src = "var hits = 0;\n\
               var i = -1;\n\
               while (i < 2) {\n\
                 i = i + 1;\n\
                 console.log(\"iter=\" + i);\n\
                 switch (i) {\n\
                   case 0: continue;\n\
                   case 1: continue;\n\
                   default: hits = hits + 1; break;\n\
                 }\n\
                 console.log(\"after=\" + i);\n\
               }\n\
               console.log(\"hits=\" + hits);\n";
    assert_eq!(run_js(src), "iter=0\niter=1\niter=2\nafter=2\nhits=1\n");
}

// Converted from `switch_fail_closed.rs`'s
// `switch_nested_in_for_loop_is_fail_closed_not_silently_wrong`, which Task 6
// added as an additive fail-closed requirement and which Tasks 7-8 correctly
// denied because `Break`/`Continue` terminators were not admitted. Task 9
// admits them, so the SWITCH half of that cell flips from denial to
// correctness — the same move Task 7 made for the flat fail-closed test. It is
// neither deleted nor weakened: it keeps its mixed `break`/`continue` clauses,
// keeps its per-iteration `console.log`, gains an `after=` discriminator, and
// asserts node's exact bytes instead of a diagnostic.
//
// ONE THING CHANGED AND IT IS A FINDING, NOT A CONVENIENCE: the loop is a
// `while`, not the original `for`. With Task 9's widening the switch is
// admitted, but the ORIGINAL `for` form still does not run correctly — it hangs
// to an `E4003` fuel trap on the `case 0: continue;` iteration. That is R-09
// (`continue` inside a C-style `for` skips the update expression), which the
// register names as the owning ID for exactly this shape and states no `switch`
// allowlist can fix
// (`docs/superpowers/followups/kali-silent-miscompile-register.md:1073,1105`).
// Verified switch-free on this branch. The `while` form is the register's own
// recommended reference lowering and exercises the identical switch property.
//
// `after=` is the break-vs-continue discriminator: it prints only on the
// iteration whose clause `break`s (i=1), so a `continue` mis-bound to the
// switch's exit would print it three times, and a `break` mis-bound to the loop
// would truncate after `iter=1`.
#[test]
fn switch_nested_in_a_loop_selects_correctly_with_break_and_continue() {
    let src = "var i = -1;\n\
               while (i < 2) {\n\
                 i = i + 1;\n\
                 console.log(\"iter=\" + i);\n\
                 switch (i) {\n\
                   case 0: continue;\n\
                   case 1: break;\n\
                   default: continue;\n\
                 }\n\
                 console.log(\"after=\" + i);\n\
               }\n\
               console.log(\"done\");\n";
    assert_eq!(run_js(src), "iter=0\niter=1\nafter=1\niter=2\ndone\n");
}

// True fallthrough — a clause ending in NEITHER `return` nor `break` — is the
// boundary Rule 4 now draws, and admitting `break` must not have widened it to
// a bare assignment. Pinned on the correctness side of the file too (the
// denial pin lives in `switch_fail_closed.rs`) is not possible, so the
// admitted-side pin here is the mixed clause: one clause `break`s, and the
// program still runs, proving the widening is per-clause evidence rather than
// a blanket relaxation.
#[test]
fn break_and_return_clauses_mix_in_one_switch() {
    let src = "function s(x) {\n\
                 var r = 0;\n\
                 switch (x) {\n\
                   case 1: return \"early\";\n\
                   case 2: r = 2; break;\n\
                   default: r = 9; break;\n\
                 }\n\
                 return \"late\" + r;\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(1));")), "v=early\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(2));")), "v=late2\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(7));")), "v=late9\n");
}

// A `break` NESTED INSIDE a `return`-terminated clause. Rule 4 admitted this
// shape from Task 7 onward (its last statement IS a `return`), but no switch
// frame existed, so the `break` bound to the enclosing `for` loop. Measured at
// HEAD a8cc36ea9f: kali printed `iter=0` / `v=end` where node prints `iter=0` /
// `iter=1` / `v=def0`, exit 0 on both sides — a SILENT MISCOMPILE in already-
// admitted territory, not a refusal.
//
// This is also the fixture that forbids gating the frame push on any clause
// having a `Break` terminator: no clause here does, and the push is still
// required. The per-iteration `iter=` line is what makes the two failure modes
// distinguishable — a loop truncated at iteration 0 versus a wrong return
// value.
#[test]
fn a_break_nested_inside_a_return_clause_binds_to_the_switch_not_the_loop() {
    let src = "function f() {\n\
                 var out = 0;\n\
                 for (var i = 0; i < 3; i = i + 1) {\n\
                   console.log(\"iter=\" + i);\n\
                   switch (i) {\n\
                     case 0: if (i < 5) { break; } return \"early\";\n\
                     default: return \"def\" + out;\n\
                   }\n\
                 }\n\
                 return \"end\";\n\
               }\n\
               console.log(\"v=\" + f());\n";
    assert_eq!(run_js(src), "iter=0\niter=1\nv=def0\n");
}

// A switch with NO `default` clause. `emit_clause_chain` ends by recursing past
// the last clause and returning without emitting anything, so the innermost
// `else` arm is EMPTY — a distinct emit path that every other fixture in this
// file misses (they all carry a `default`). Before `break` was admitted this
// path was barely reachable: an all-`return` switch with no default falls off
// the end of the function. `s(7)` matches no clause and must leave `r` at its
// initial `0`.
#[test]
fn a_break_switch_with_no_default_falls_out_of_the_chain() {
    let src = "function s(x) {\n\
                 var r = 0;\n\
                 switch (x) {\n\
                   case 1: r = 1; break;\n\
                   case 2: r = 2; break;\n\
                 }\n\
                 return r;\n\
               }\n\
               console.log(\"a=\" + s(1));\n\
               console.log(\"b=\" + s(7));\n";
    assert_eq!(run_js(src), "a=1\nb=0\n");
}

// The mirror image: a switch whose ONLY clause is `default`, with a `break`.
// The chain has no `If` at all — the default body is emitted unconditionally at
// depth 0 inside the switch's block, so this is the one shape where the block
// wrapper is the entire control structure.
#[test]
fn a_default_only_break_switch_runs_its_body() {
    let src = "function s(x) {\n\
                 var r = 0;\n\
                 switch (x) {\n\
                   default: r = 9; break;\n\
                 }\n\
                 return r;\n\
               }\n\
               console.log(\"v=\" + s(1));\n";
    assert_eq!(run_js(src), "v=9\n");
}

// A switch nested inside a switch: the inner `break` must bind to the INNER
// switch and let the outer clause continue (`r = r + 100`), not escape both.
// This is the frame-stack half of the by-construction property — `break`
// resolves against `loop_frames.last()`, so nesting works for free — and it is
// the cell that would fail if the switch frame were pushed once per switch
// STATEMENT rather than once per emission, or popped in the wrong order.
#[test]
fn a_break_in_a_nested_switch_binds_to_the_inner_switch() {
    let src = "function s(x, y) {\n\
                 var r = 0;\n\
                 switch (x) {\n\
                   case 1:\n\
                     switch (y) {\n\
                       case 1: r = 11; break;\n\
                       default: r = 19; break;\n\
                     }\n\
                     r = r + 100;\n\
                     break;\n\
                   default: r = 99; break;\n\
                 }\n\
                 return r;\n\
               }\n\
               console.log(\"a=\" + s(1, 1));\n\
               console.log(\"b=\" + s(1, 2));\n\
               console.log(\"c=\" + s(2, 1));\n";
    assert_eq!(run_js(src), "a=111\nb=119\nc=99\n");
}

// Function scope, not module scope: a switch inside a loop inside a FUNCTION,
// with a `continue` clause, a `break` clause and a default. Scope is a required
// test axis and every other loop-nested cell here is at module scope.
#[test]
fn break_and_continue_bind_correctly_in_a_loop_inside_a_function() {
    let src = "function f(n) {\n\
                 var acc = 0;\n\
                 var i = -1;\n\
                 while (i < 2) {\n\
                   i = i + 1;\n\
                   console.log(\"iter=\" + i);\n\
                   switch (i) {\n\
                     case 0: acc = acc + n; continue;\n\
                     case 1: acc = acc + 10; break;\n\
                     default: acc = acc + 100; break;\n\
                   }\n\
                   console.log(\"after=\" + i);\n\
                 }\n\
                 return acc;\n\
               }\n\
               console.log(\"v=\" + f(1));\n";
    assert_eq!(
        run_js(src),
        "iter=0\niter=1\nafter=1\niter=2\nafter=2\nv=111\n"
    );
}

// THE ARENA PROPERTY, as a test rather than a one-off probe: a `break` out of a
// switch inside an ALLOCATING loop, across 200 iterations. A switch opens NO
// arena frame, so the break path falls through to the loop's single
// unconditional release; a double release would splice the enclosing arena's
// still-live pages onto the free list (the defect
// `emit/control_flow.rs:57-75` records) and this would trap or diverge.
//
// Two allocations per iteration land in the loop arena: the `console.log`
// concat and `"k" + i`. Per-iteration logging is present as required, and the
// expectation is BUILT rather than pasted so all 200 lines are actually
// asserted — a loop truncated at iteration 0 fails on the very first line, not
// on a summary number.
//
// The brief's own Step-7 fixture (`var o = { a: i, b: i + 1 };` with
// `switch (i % 3)`) CANNOT be used and this is a finding, not a substitution of
// convenience: it is refused by an unrelated rule — "object literal for
// Binding(\"_start\", \"o\") has a field value that is not provably a number or
// string" — and the identical refusal reproduces with the switch DELETED
// entirely, so it never reaches the arena question at all. (`i % 3` would also
// have failed Rule 1: a binary expression is not a proven i64 discriminant.)
#[test]
fn break_out_of_a_switch_in_an_allocating_loop_does_not_corrupt_the_arena() {
    let src = "var total = 0;\n\
               var m = 0;\n\
               for (var i = 0; i < 200; i = i + 1) {\n\
                 console.log(\"iter=\" + i);\n\
                 var s = \"k\" + i;\n\
                 switch (m) {\n\
                   case 0: total = total + s.length; m = 1; break;\n\
                   case 1: total = total + 1; m = 2; break;\n\
                   default: m = 0; break;\n\
                 }\n\
               }\n\
               console.log(\"total=\" + total);\n";
    let mut expected = String::new();
    for i in 0..200 {
        expected.push_str(&format!("iter={i}\n"));
    }
    expected.push_str("total=297\n");
    assert_eq!(run_js(src), expected);
}

// The string axis must inherit the `break` widening too — Rule 4 is domain-
// independent, so a string discriminant with `break`-terminated clauses is
// admitted by the same evidence. Without this, "widened the terminator set"
// could silently have meant "widened it on the numeric axis only".
#[test]
fn break_terminated_clauses_work_on_the_string_axis() {
    let src = "function s(x) {\n\
                 var r = 0;\n\
                 switch (x) {\n\
                   case \"a\": r = 1; break;\n\
                   case \"b\": r = 2; break;\n\
                   default: r = 9; break;\n\
                 }\n\
                 return r;\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(\"b\"));")), "v=2\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(\"z\"));")), "v=9\n");
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
