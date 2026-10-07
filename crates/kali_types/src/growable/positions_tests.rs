//! Growable-runtime-arrays spec §3.3, §3.5-§3.6, A-3, A-6, A-8.

use std::collections::BTreeMap;

use super::growable_refusals;
use crate::growable::flow::solve;

fn params(src: &str) -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::new();
    for stmt in crate::test_support::parse_statements(src) {
        if let kali_ast::Statement::FunctionDeclaration(decl) = stmt {
            map.insert(decl.name.clone(), decl.params.clone());
        }
    }
    map
}

fn refusals(src: &str) -> Vec<String> {
    let facts = crate::repr_infer::growable_facts_for(&crate::test_support::parse_statements(src));
    let solution = solve(&facts);
    growable_refusals(&facts, &solution, &params(src))
}

fn assert_one(src: &str, needle: &str) {
    let messages = refusals(src);
    assert!(
        messages.iter().any(|m| m.contains(needle)),
        "{src}\nexpected a refusal containing {needle:?}, got {messages:?}"
    );
}

#[test]
fn a_program_that_uses_growable_arrays_only_in_supported_positions_is_quiet() {
    let src = "function build(n) { const out = []; for (let i = 0; i < n; i++) out.push(i); return out; }\n\
               function total(a) { let s = 0; for (const x of a) s += x; return s; }\n\
               function main() { const xs = build(3); xs[0] = 5; const ys = xs.slice(1); console.log(xs.length, xs[1], xs.join(\",\"), ys.indexOf(2), total(xs), xs.pop()); }\n\
               main();";
    assert_eq!(refusals(src), Vec::<String>::new());
}

#[test]
fn a_plain_use_is_refused_naming_the_binding() {
    assert_one(
        "function main() { const a = []; a.push(1); console.log(typeof a); }",
        "the growable array `a` in `main` is used as a plain value here",
    );
}

#[test]
fn printing_a_whole_growable_array_is_refused() {
    assert_one(
        "function main() { const a = []; a.push(1); console.log(\"n\", a); }",
        "printing a whole runtime array is unavailable",
    );
}

#[test]
fn a_function_using_a_module_level_growable_array_is_refused() {
    assert_one(
        "const out = []; out.push(1); function size() { return out.length; } console.log(size());",
        "function `size` uses the module-level growable array `out`",
    );
}

#[test]
fn a_captured_growable_array_is_refused() {
    assert_one(
        "function main() { const o = []; o.push(1); const f = () => o.length; console.log(f()); }",
        "the growable array `o` in `main` is captured",
    );
}

#[test]
fn a_class_body_beside_a_growable_array_is_refused_as_a_capture() {
    assert_one(
        "function main() { const o = []; o.push(1); class C {} console.log(o.length); }",
        "the growable array `o` in `main` is captured",
    );
}

#[test]
fn from_index_length_writes_and_unsupported_methods_are_refused() {
    assert_one(
        "const a = []; a.push(1); a.indexOf(1, 1);",
        "`indexOf` with a `fromIndex` argument",
    );
    assert_one(
        "const a = []; a.push(1); a.includes(1, 1);",
        "`includes` with a `fromIndex` argument",
    );
    assert_one(
        "const a = []; a.push(1); a.length = 0;",
        "assigning to `.length` of the growable array `a` at module scope",
    );
    assert_one(
        "const a = []; a.push(1); a.reverse();",
        "`.reverse()` on the growable array `a` at module scope",
    );
    assert_one(
        "const a = []; a.push(1); a[\"push\"](2);",
        "`[\"push\"]()` on the growable array `a`",
    );
}

#[test]
fn indexing_or_mutating_a_call_result_directly_is_refused() {
    let src = "function make() { const o = []; o.push(1); return o; } console.log(make()[0]); make().push(2);";
    assert_one(
        src,
        "indexing, `push` or `pop` directly on the array `make(…)` returns",
    );
}

#[test]
fn a_mixed_layout_a_literal_expression_and_a_non_array_write_are_rendered() {
    assert_one(
        "function total(a) { return a.length; } const xs = []; xs.push(1); const p = new Array(2); total(xs); total(p);",
        "`a` in `total` would hold both a growable array",
    );
    assert_one(
        "function f(c) { if (c) return []; const o = []; o.push(1); return o; } const r = f(false); console.log(r.length);",
        "an array literal written directly as a call argument or `return` value in `f`",
    );
    assert_one(
        "function f(a) { a.push(1); return a.length; } const xs = [0]; f(xs); f(5);",
        "`a` in `f` holds a growable array and is also given a value that is not an array",
    );
}

#[test]
fn a_push_on_the_iterated_array_inside_its_for_of_is_refused() {
    assert_one(
        "const a = []; a.push(1); for (const x of a) { if (x < 3) a.push(x + 1); }",
        "`push` or `pop` on the growable array `a` at module scope",
    );
}

#[test]
fn a_push_through_an_alias_inside_a_for_of_over_the_original_is_refused() {
    // Review Focus 2.
    assert_one(
        "function main() { const a = []; a.push(1); const b = a; for (const x of a) { if (x < 3) b.push(x + 1); } }",
        "`push` or `pop` on the growable array `a` in `main`",
    );
}

#[test]
fn a_call_that_pushes_the_iterated_array_through_a_parameter_is_refused() {
    // Review Focus 4: node grows the iteration; kali's snapshot would not.
    let src = "function add(arr, v) { arr.push(v); }\n\
               function relay(arr, v) { add(arr, v); }\n\
               function main() { const a = []; a.push(1); for (const x of a) { if (x < 3) relay(a, x + 1); } }";
    assert_one(src, "`push` or `pop` on the growable array `a` in `main`");
}

#[test]
fn a_loop_that_pushes_another_array_or_only_reads_its_own_is_admitted() {
    let src = "function total(a) { let s = 0; for (const x of a) s += x; return s; }\n\
               function main() { const a = []; a.push(1); const b = []; b.push(0); for (const x of a) { b.push(x); console.log(total(a)); } }";
    assert_eq!(refusals(src), Vec::<String>::new());
}

#[test]
fn a_program_without_growable_arrays_is_quiet() {
    assert_eq!(
        refusals("function f() { const a = [1, 2]; return a.length; } const p = new Array(3); p[0] = 1; console.log(f(), typeof p);"),
        Vec::<String>::new()
    );
}

#[test]
fn pop_with_an_argument_is_refused() {
    // Task 10 review I1: the argument was dropped unevaluated (its side
    // effects lost) in codegen; inference refuses so `check` agrees.
    assert_one(
        "let c = 0; function bump() { c++; return 0; } function main() { const a = []; a.push(1); a.push(2); a.pop(bump()); console.log(c, a.length); } main();",
        "`.pop()` with an argument on the growable array `a` in `main` is unavailable",
    );
    assert_one(
        "const a = []; a.push(1); a.pop(1);",
        "`.pop()` with an argument on the growable array `a` at module scope is unavailable",
    );
}

#[test]
fn slice_with_more_than_two_arguments_is_refused() {
    assert_one(
        "function main() { const a = []; a.push(1); console.log(a.slice(0, 1, 2).length); } main();",
        "`.slice()` with more than two arguments on the growable array `a` in `main` is unavailable",
    );
    assert_one(
        "const a = []; a.push(1); const t = a.slice(0, 1, 2); console.log(t.length);",
        "`.slice()` with more than two arguments on the growable array `a` at module scope is unavailable",
    );
}

#[test]
fn slice_with_two_arguments_and_pop_without_one_stay_quiet() {
    assert_eq!(
        refusals("function main() { const a = []; a.push(1); a.push(2); const t = a.slice(0, 1); console.log(t.length, a.pop()); } main();"),
        Vec::<String>::new()
    );
}
