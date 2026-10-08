//! Growable-runtime-arrays residual fixes R1 (spec A-40) and R2 (spec A-39).

use kali_common::ReprTable;

fn table(src: &str) -> ReprTable {
    crate::repr_infer::infer_reprs(&crate::test_support::parse_statements(src))
}

fn conflicts(src: &str) -> Vec<String> {
    table(src).shape_conflicts().to_vec()
}

fn refused(src: &str, needle: &str) -> bool {
    conflicts(src).iter().any(|m| m.contains(needle))
}

const SEARCH: &str = "result of a growable array";
const FRACTIONAL: &str = "not proven to be a whole number";

// ---- R1 (A-40) ----------------------------------------------------------

#[test]
fn a_stored_includes_result_is_a_boolean_binding() {
    let src = "function main() { const xs = []; xs.push(3); const r = xs.includes(3); \
               let found = false; found = xs.includes(4); console.log(r, found, typeof r); } main();";
    let t = table(src);
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
    assert!(t.binding_is_search_boolean("main", "r"));
    assert!(t.binding_is_search_boolean("main", "found"));
}

#[test]
fn a_module_scope_includes_result_is_a_boolean_binding() {
    let t = table("const xs = []; xs.push(3); const r = xs.includes(3); console.log(r);");
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
    assert!(t.binding_is_search_boolean("_start", "r"));
}

#[test]
fn a_function_returning_an_includes_result_is_boolean() {
    let src = "function has(a, v) { return a.includes(v); } \
               function main() { const xs = []; xs.push(3); const b = has(xs, 3); console.log(b, has(xs, 4)); } main();";
    let t = table(src);
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
    assert!(t.return_is_search_boolean("has"));
    assert!(t.binding_is_search_boolean("main", "b"));
}

#[test]
fn an_includes_result_in_an_unkept_position_is_refused() {
    for body in [
        "function show(v) { console.log(v); } show(xs.includes(3));",
        "console.log(xs.includes(4) || 0);",
        "console.log(xs.includes(4) ? xs.includes(3) : 1);",
        "let r = 0; r = xs.includes(3); console.log(r);",
        "const o = { k: xs.includes(3) }; console.log(o.k);",
        "const r = xs.includes(3); const f = () => r; console.log(f());",
        "function g() { if (xs.length > 5) return xs.includes(3); } console.log(g());",
        "let r = false; console.log(r = xs.includes(3));",
    ] {
        let module = format!("const xs = []; xs.push(3); {body}");
        assert!(
            refused(&module, SEARCH),
            "{module}\n{:?}",
            conflicts(&module)
        );
    }
}

#[test]
fn a_kept_includes_result_and_a_plain_includes_are_quiet() {
    let src = "function main() { const xs = []; xs.push(3); \
               if (xs.includes(3) && xs.length > 0) console.log(1); \
               console.log(!xs.includes(3), xs.includes(3) ? 1 : 2, \"x\" + xs.includes(3), xs.includes(3) === true); \
               xs.includes(4); \
               const lit = [1, 2].indexOf(2); console.log(lit, xs.indexOf(3)); } main();";
    assert!(!refused(src, SEARCH), "{:?}", conflicts(src));
}

// ---- R2 (A-39) ----------------------------------------------------------

#[test]
fn a_rounded_float_index_is_admitted() {
    let src = "function main() { const xs = []; for (let i = 0; i < 7; i++) xs.push(i); const x = 9; \
               const m = Math.floor(xs.length / 2); \
               console.log(xs[Math.floor(x / 2)], xs[m], xs[m + 1], xs[Math.min(Math.ceil(x / 4), 6)]); \
               let lo = 0; let hi = 6; while (lo < hi) { const mid = Math.floor((lo + hi) / 2); \
               if (xs[mid] < 3) lo = mid + 1; else hi = mid; } console.log(xs[lo]); } main();";
    assert!(!refused(src, FRACTIONAL), "{:?}", conflicts(src));
    assert_eq!(table(src).scalar("main", "m"), kali_common::Repr::F64);
}

#[test]
fn a_fractional_float_index_is_refused() {
    for index in [
        "x / 2",
        "Math.max(x / 2, 1)",
        "w",
        "Math.floor(x / 2) + 0.5",
    ] {
        let src = format!(
            "function main() {{ const xs = []; xs.push(1); const x = 3; let w = Math.floor(x / 2); w = x / 4; console.log(xs[{index}]); }} main();"
        );
        assert!(refused(&src, FRACTIONAL), "{src}\n{:?}", conflicts(&src));
    }
}

#[test]
fn an_integer_index_is_never_checked() {
    let src = "function f(i) { return i * 2; } function main() { const xs = []; xs.push(1); xs.push(2); \
               const s = \"ab\"; console.log(xs[f(0)], xs[s.charCodeAt(0) - 97], xs[xs.length - 1]); } main();";
    assert!(!refused(src, FRACTIONAL), "{:?}", conflicts(src));
}

#[test]
fn a_math_rounding_of_a_float_is_f64_and_of_a_literal_is_not() {
    let t = table(
        "function main() { const x = 9 / 2; const a = Math.floor(x); const b = Math.floor(4.5); \
         const c = Math.max(2, x); const d = Math.max(2, 3); console.log(a, b, c, d); } main();",
    );
    assert_eq!(t.scalar("main", "a"), kali_common::Repr::F64);
    assert_eq!(t.scalar("main", "b"), kali_common::Repr::I64);
    assert_eq!(t.scalar("main", "c"), kali_common::Repr::F64);
    assert_eq!(t.scalar("main", "d"), kali_common::Repr::I64);
}

// ---- residual round 1 ---------------------------------------------------

#[test]
fn a_rounding_of_a_const_alias_of_a_literal_is_not_f64() {
    // N1: `const t = 7.9; Math.floor(t)` folds to an i64 in codegen, so
    // inference gives it no float edge (it ran on `main`; round 0 made it
    // f64 and `% 3` failed to load).
    for src in [
        "function main() { const t = 7.9; const k = Math.floor(t); console.log(k % 3); } main();",
        "function main() { const v = 2.6; const t = v; const k = Math.round(-t); console.log(k); } main();",
        "const t = 3.2; function main() { const k = Math.ceil(t); console.log(k % 2); } main();",
    ] {
        assert_eq!(table(src).scalar("main", "k"), kali_common::Repr::I64, "{src}");
    }
    let runtime = "function main() { let t = 7; t = t / 2; const k = Math.floor(t); console.log(k); } main();";
    assert_eq!(table(runtime).scalar("main", "k"), kali_common::Repr::F64);
}

#[test]
fn a_negated_or_compared_includes_result_is_a_boolean_binding() {
    // N2: `!xs.includes(v)` and a comparison over an `includes` result.
    let src = "function absent(a, v) { return !a.includes(v); } \
               function main() { const xs = []; xs.push(2); const r = !xs.includes(2); \
               let f = !xs.includes(2); f = !f; const e = xs.includes(2) === false; \
               const s = xs.includes(2) == xs.includes(3); const c = 1 < 2; \
               console.log(r, f, e, s, c, absent(xs, 1)); } main();";
    let t = table(src);
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
    for name in ["r", "f", "e", "s"] {
        assert!(t.binding_is_search_boolean("main", name), "{name}");
    }
    assert!(!t.binding_is_search_boolean("main", "c"));
    assert!(t.return_is_search_boolean("absent"));
}

#[test]
fn a_negated_includes_result_in_an_unkept_position_is_refused() {
    for body in [
        "function show(v) { console.log(v); } show(!xs.includes(3));",
        "let r = 0; r = !xs.includes(3); console.log(r);",
        "const o = { k: xs.includes(3) === true }; console.log(o.k);",
    ] {
        let src = format!("function main() {{ const xs = []; xs.push(3); {body} }} main();");
        assert!(refused(&src, SEARCH), "{src}\n{:?}", conflicts(&src));
    }
}

#[test]
fn a_remainder_index_over_a_float_is_refused() {
    // Minor: a float `%` does not lower, so it is not proven whole.
    let src = "function main() { const xs = []; xs.push(1); xs.push(2); const q = 3; \
               const i = Math.floor(q / 2); console.log(xs[i % 2]); } main();";
    assert!(refused(src, FRACTIONAL), "{:?}", conflicts(src));
    let int = "function main() { const xs = []; xs.push(1); for (let i = 0; i < 3; i++) console.log(xs[i % 1]); } main();";
    assert!(!refused(int, FRACTIONAL), "{:?}", conflicts(int));
}

// ---- residual round 2 ---------------------------------------------------

#[test]
fn a_rounding_of_a_module_const_chain_read_from_a_function_is_not_f64() {
    // N1b: inference and codegen follow the same chains.
    for src in [
        "const t = 7.9; const u = t; function f() { const k = Math.floor(u); return k % 3; } console.log(f());",
        "const a = 7.9; const b = -a; function f() { const k = Math.floor(b); return k % 3; } console.log(f());",
        "const t = 7.9; function f() { const t2 = t; const k = Math.floor(t2); return k | 1; } console.log(f());",
    ] {
        assert_eq!(table(src).scalar("f", "k"), kali_common::Repr::I64, "{src}");
    }
}

#[test]
fn a_negated_logical_over_includes_results_is_a_boolean_binding() {
    // N2b: `!` over `||`, `&&`, `??` and `?:` holding search values (a `,`
    // is refused since round 3: `a_comma_holding_an_includes_result_is_refused`).
    let src = "function none(ys, v) { return !(ys.includes(v) || ys.includes(v + 1)); } \
               function main() { const xs = []; xs.push(2); const a = xs.includes(2); \
               const r1 = !(xs.includes(2) || xs.includes(4)); const r2 = !(xs.includes(2) && xs.includes(4)); \
               const r3 = !(xs.includes(2) ? xs.includes(3) : false); const r4 = !(xs.includes(2) ?? false); \
               const r6 = !(a || xs.includes(4)); \
               let r7 = !(xs.includes(2) || 0); r7 = !r7; \
               console.log(r1, r2, r3, r4, r6, r7, none(xs, 7)); } main();";
    let t = table(src);
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
    for name in ["r1", "r2", "r3", "r4", "r6", "r7"] {
        assert!(t.binding_is_search_boolean("main", name), "{name}");
    }
    assert!(t.return_is_search_boolean("none"));
}

#[test]
fn a_comparison_over_a_logical_of_includes_results_is_refused() {
    let src = "function main() { const xs = []; xs.push(3); \
               const r = (xs.includes(5) || xs.includes(3)) === true; console.log(r); } main();";
    assert!(refused(src, SEARCH), "{:?}", conflicts(src));
}

// ---- residual round 3 ---------------------------------------------------

#[test]
fn a_const_loop_variable_over_numeric_literals_is_a_compile_time_number() {
    // N1c: codegen unrolls the loop and folds `Math.floor(x)`; inference
    // must not give it a float edge, and publishes the binding.
    for (src, func) in [
        ("for (const x of [1.5, 2.5]) { const k = Math.floor(x); console.log(k % 2); }", "_start"),
        ("function f() { for (const x of [1.5, 2.5]) { const k = Math.floor(x); console.log(k % 2); } } f();", "f"),
        ("const xs = [1.5, 2.5]; for (const x of xs) { const k = Math.floor(x); console.log(k / 2); }", "_start"),
    ] {
        let t = table(src);
        assert_eq!(t.scalar(func, "k"), kali_common::Repr::I64, "{src}");
        assert!(t.binding_is_static_numeric(func, "x"), "{src}");
    }
    // A growable iterable is a runtime loop: its variable is not static.
    let t = table("const xs = [1.5]; xs.push(2.5); for (const x of xs) { const k = Math.floor(x); console.log(k); }");
    assert!(!t.binding_is_static_numeric("_start", "x"));
    assert_eq!(t.scalar("_start", "k"), kali_common::Repr::F64);
}

#[test]
fn a_comma_holding_an_includes_result_is_refused() {
    // N2c: kali's comma value is not JS's (`(1, 5)` prints `2`).
    for e in [
        "!(xs.includes(1), 5)",
        "!(xs.includes(1), xs.includes(5))",
        "!(1, xs.includes(2))",
    ] {
        let src = format!("function main() {{ const xs = []; xs.push(5); const r = {e}; console.log(r); }} main();");
        assert!(refused(&src, SEARCH), "{src}\n{:?}", conflicts(&src));
    }
}

#[test]
fn a_unary_sign_of_an_includes_result_is_refused() {
    // Round 3: `+r` printed `false` (node `0`).
    for e in ["+r", "-xs.includes(5)", "+!xs.includes(5)"] {
        let src = format!("function main() {{ const xs = []; xs.push(5); const r = !xs.includes(5); console.log({e}); }} main();");
        assert!(refused(&src, SEARCH), "{src}\n{:?}", conflicts(&src));
    }
}

// ---- residual round 4 ---------------------------------------------------

#[test]
fn a_ts_wrapped_const_is_a_compile_time_number() {
    // HIR drops `as`/`satisfies`, so codegen folds these as literals;
    // inference must publish them too (u02, u07, u10).
    for (src, func) in [
        ("const t = 1.5 as number; const s = (2.5 satisfies number); const u = t as number; const k = Math.floor(u); console.log(k % 2, Math.floor(s) % 2);", "_start"),
        ("function f() { const t = 1.5 as number; const s = (2.5 satisfies number); const u = t as number; const k = Math.floor(u); console.log(k % 2, Math.floor(s) % 2); } f();", "f"),
    ] {
        let t = table(src);
        assert!(t.shape_conflicts().is_empty(), "{src}: {:?}", t.shape_conflicts());
        for name in ["t", "s", "u"] {
            assert!(t.binding_is_static_numeric(func, name), "{src}: {name}");
        }
        assert_eq!(t.scalar(func, "k"), kali_common::Repr::I64, "{src}");
    }
}

#[test]
fn a_plain_for_of_without_a_growable_array_carries_no_item_repr() {
    // Item 4 (t01, s07, r23): Task 7's item flow feeds only a growable
    // element, so without a growable array the loop variable stays as on
    // `main` (codegen unrolls it): no both-axes refusal, no float item.
    for src in [
        "for (const x of [3, \"a\"]) { console.log(x); }",
        "for (const x of [1.5, 2.5]) { console.log(x); } for (const x of [3, \"a\"]) { console.log(x); }",
        "function main() { for (const x of [3, \"a\"]) { console.log(x); } } main();",
    ] {
        let t = table(src);
        assert!(t.shape_conflicts().is_empty(), "{src}: {:?}", t.shape_conflicts());
    }
    let t = table("for (let x of [1.5, 2.5]) { const k = Math.floor(x); console.log(k % 2); }");
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
    assert_ne!(t.scalar("_start", "x"), kali_common::Repr::F64);
    assert_eq!(t.scalar("_start", "k"), kali_common::Repr::I64);
}

#[test]
fn a_plain_for_of_in_a_program_with_a_growable_array_keeps_its_item_repr() {
    // The flow still runs where it matters: a mixed literal item pushed onto
    // a growable array is the both-axes refusal, and a float `let` item
    // floats the rounding call (the runtime f64 lane).
    let t =
        table("const xs = []; for (const x of [3, \"a\"]) { xs.push(x); } console.log(xs.length);");
    assert!(!t.shape_conflicts().is_empty());
    let t = table("const ys = []; ys.push(1); for (let x of [1.5, 2.5]) { const k = Math.floor(x); console.log(k); }");
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
    assert_eq!(t.scalar("_start", "x"), kali_common::Repr::F64);
    assert!(!t.binding_is_static_numeric("_start", "x"));
    assert_eq!(t.scalar("_start", "k"), kali_common::Repr::F64);
}

#[test]
fn a_const_chain_of_any_length_is_a_compile_time_number() {
    // Round 3 cut chains at 1024 hops; `main` folds any length, so a longer
    // chain reached the runtime f64 lane and `% 2` failed at load. The bound
    // is now the number of bindings (a cycle is the only way past it).
    let mut src = String::from("const c0 = 1.5;\n");
    for i in 1..1500 {
        src.push_str(&format!("const c{i} = c{};\n", i - 1));
    }
    src.push_str("const k = Math.floor(c1499); console.log(k % 2);\n");
    let t = table(&src);
    assert!(t.binding_is_static_numeric("_start", "c1499"));
    assert_eq!(t.scalar("_start", "k"), kali_common::Repr::I64);
}

#[test]
fn a_compile_time_loop_variable_still_floats_a_local_it_is_copied_into() {
    // c21, r29: `const y = x` needs an f64 slot; the rounding call over `y`
    // stays on the fold because `y` is a compile-time number too.
    let t = table("for (const x of [1.5, 2.5]) { const y = x; const z = y; const k = Math.floor(z); console.log(k % 2); }");
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
    assert_eq!(t.scalar("_start", "y"), kali_common::Repr::F64);
    assert!(t.binding_is_static_numeric("_start", "z"));
    assert_eq!(t.scalar("_start", "k"), kali_common::Repr::I64);
}
