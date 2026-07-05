//! Behavioral contract for the escape gate (the 13-test matrix).
//!
//! Each test lowers a small inline source, computes the `ArenaTable`, and
//! asserts the placement decision. Every ambiguity must fail closed.

use crate::compute_arena_table;
use crate::test_support::analyze;

// --- Per-function eligibility / fate lattice -------------------------------

#[test]
fn eligible_when_all_sites_only_returned() {
    // A factory whose only allocation is returned has `Returned` fate — no
    // global site — so its sites may target the current arena.
    let mir = analyze("function factory() { return { left: null, right: null }; }");
    let table = compute_arena_table(&mir);
    assert!(table.arena_eligible("factory"));
    // Purely-returned allocations do not die inside the function, so it does
    // not need a function-body arena.
    assert!(!table.opens_arena("factory"));
}

#[test]
fn ineligible_on_module_binding_store() {
    let mir = analyze("let g; function f() { g = { v: 1 }; }");
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
}

#[test]
fn ineligible_on_capture() {
    let mir = analyze(
        "function f() { const o = { v: 1 }; const inner = function() { return o; }; return inner; }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
}

#[test]
fn ineligible_on_store_into_preexisting() {
    let mir = analyze("function f(p) { p.left = { v: 1 }; }");
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
}

#[test]
fn eligible_child_joins_parent_fate() {
    // The inner literal is embedded as a field of the returned outer literal;
    // its fate joins the parent's (Returned), so it is NOT a global site.
    let mir = analyze("function f() { return { left: { v: 1 }, right: null }; }");
    let table = compute_arena_table(&mir);
    assert!(table.arena_eligible("f"));
}

#[test]
fn uniform_per_function() {
    // One global site (module store) plus one local site: v1 coarseness makes
    // the single global site poison the whole function.
    let mir =
        analyze("let g; function f() { g = { v: 1 }; const local = { w: 2 }; return local.w; }");
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
}

#[test]
fn opens_arena_only_with_local_sites() {
    // `o` is a fresh object that dies inside `f` (only a scalar field is read),
    // so `f` both is arena-eligible and should open a function-body arena.
    let mir = analyze("function f() { const o = { v: 1 }; let s = o.v; return s; }");
    let table = compute_arena_table(&mir);
    assert!(table.arena_eligible("f"));
    assert!(table.opens_arena("f"));
}

#[test]
fn no_arena_for_nonallocating_fn() {
    // itemCheck allocates nothing — no arena, no prologue.
    let mir = analyze(
        "function itemCheck(t) { if (t.left === null) { return t.v; } return itemCheck(t.left) + itemCheck(t.right); }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("itemCheck"));
    assert!(!table.opens_arena("itemCheck"));
}

// --- Per-loop gating -------------------------------------------------------

#[test]
fn loop_arena_when_no_outflow() {
    let mir = analyze(
        "function bottomUpTree(d) { return { left: null, right: null }; }
         function itemCheck(t) { return 1; }
         function f(n) {
           let sum = 0;
           for (let i = 0; i < n; i = i + 1) {
             const tree = bottomUpTree(3);
             sum = sum + itemCheck(tree);
           }
           return sum;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(table.loop_arena("f", 0));
}

#[test]
fn loop_veto_on_outer_binding_assignment() {
    let mir = analyze(
        "function bottomUpTree(d) { return { left: null, right: null }; }
         function f(n) {
           let keep;
           for (let i = 0; i < n; i = i + 1) { keep = bottomUpTree(3); }
           return keep;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.loop_arena("f", 0));
}

#[test]
fn loop_veto_on_unknown_call() {
    // `cb` is a parameter — an indirect/closure call that vetoes the arena.
    let mir = analyze(
        "function f(cb, n) {
           for (let i = 0; i < n; i = i + 1) { const t = { v: i }; cb(t); }
           return 0;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.loop_arena("f", 0));
}

#[test]
fn loop_whitelist_console_log() {
    // console.log is a non-retaining host call on the whitelist; the fresh
    // array it consumes gives the loop reachable allocation without outflow.
    let mir = analyze(
        "function f(n) {
           for (let i = 0; i < n; i = i + 1) { console.log([i]); }
           return 0;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(table.loop_arena("f", 0));
}

#[test]
fn loop_ordinals_are_preorder() {
    // Two sequential loops (0, 1) with a nested loop (2). Pre-order numbers the
    // nested loop AFTER its parent. Loop 0 and the nested loop 2 qualify; the
    // outer loop 1 leaks a tree into `keep` and is vetoed. The ordinal-specific
    // decisions pin that the nested loop is ordinal 2 (not 1).
    let mir = analyze(
        "function mk(d) { return { v: d }; }
         function f(n) {
           let keep;
           for (let i = 0; i < n; i = i + 1) { const a = mk(1); let s = a.v; }
           for (let j = 0; j < n; j = j + 1) {
             for (let k = 0; k < n; k = k + 1) { const b = mk(2); let t = b.v; }
             keep = mk(3);
           }
           return keep;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(table.loop_arena("f", 0));
    assert!(!table.loop_arena("f", 1));
    assert!(table.loop_arena("f", 2));
    assert!(!table.loop_arena("f", 3));
}
