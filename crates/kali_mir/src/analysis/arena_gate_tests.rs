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

// --- Review fixes: fail-open holes pinned ----------------------------------

#[test]
fn ineligible_on_ident_store_to_module_binding() {
    // The RHS is an identifier holding a fresh heap value (not a literal), so
    // the rhs_fresh check alone misses it; the LHS is a module binding, so the
    // value outlives any arena `build` would open. Must be Global (fail
    // closed) — otherwise `cache` dangles after a function-exit reset.
    let mir = analyze(
        "let cache;
         function build() {
           const node = { v: 1 };
           cache = node;
           return node.v;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("build"));
    assert!(!table.opens_arena("build"));
}

// NOTE (review item b): a destructuring/pattern-target variant is NOT
// expressible in today's surface — kali_ast has no ArrayPattern/ObjectPattern,
// and a source like `[a] = [{ v: 1 }];` is dropped at parse (verified: the
// function's arena facts show `allocates: false`, i.e. the statement never
// reaches the analyzer). The analyzer's unknown-LHS arm is nevertheless
// tightened to fail closed (global site + outflow on any heap-holding RHS).

#[test]
fn loop_veto_on_non_kali_write_stdout_bytes_receiver() {
    // A user method merely NAMED writeStdoutBytes may retain its argument; the
    // whitelist must require the `Kali` receiver, not just the method name.
    let mir = analyze(
        "function f(sink, n) {
           for (let i = 0; i < n; i = i + 1) { sink.writeStdoutBytes([i]); }
           return 0;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.loop_arena("f", 0));
}

#[test]
fn loop_whitelist_kali_write_stdout_bytes() {
    // The real host intrinsic (`Kali.writeStdoutBytes`) consumes its argument
    // and stays whitelisted.
    let mir = analyze(
        "function f(n) {
           for (let i = 0; i < n; i = i + 1) { Kali.writeStdoutBytes([i]); }
           return 0;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(table.loop_arena("f", 0));
}

// --- Review fixes round 2: conservative heap judgment ------------------------

#[test]
fn ineligible_on_ternary_store_with_scalar_alternate() {
    // NOTE: kali_parser has no ternary surface today, so this source does not
    // reach HirNodeKind::ConditionalExpr (the statement parses degenerately);
    // the test pins that the store is classified Global regardless of the
    // parse shape. The ConditionalExpr analyzer arm itself is pinned by the
    // hand-built-HIR test `ineligible_on_conditional_expr_store_hir_level`.
    let mir = analyze(
        "let cache;
         function f(c) {
           const node = { v: 1 };
           cache = c ? node : null;
           return 1;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

#[test]
fn ineligible_on_logical_or_store() {
    // `||` parses as BinaryExpr text "||"; infer_binary_layout maps it to
    // Scalar("bool") although `||` returns an operand. Heap if ANY operand is
    // heap.
    let mir = analyze(
        "let cache;
         function f(flag) {
           const node = { v: 1 };
           cache = flag || node;
           return 1;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

#[test]
fn ineligible_on_reassigned_scalar_binding_store() {
    // `x`'s layout is frozen Scalar("number") at the declarator; reassigning a
    // fresh literal into it must record x as fresh-heap so the later
    // module-binding store is classified Global.
    let mir = analyze(
        "let cache;
         function f() {
           let x = 0;
           x = { v: 1 };
           cache = x;
           return 1;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

#[test]
fn same_named_functions_fail_closed() {
    // Two function expressions sharing the name `h` collide in the name-keyed
    // facts (codegen is name-keyed too, so per-instance decisions are
    // impossible). The collision must poison the merged entry: OR-merging
    // could otherwise grant opens_arena("h") from the second body while the
    // first body's returned values would dangle after the function-exit reset.
    let mir = analyze(
        "const a = function h() { return { v: 1 }; };
         const b = function h() { const o = { v: 2 }; let s = o.v; return s; };",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("h"));
    assert!(!table.opens_arena("h"));
}

#[test]
fn ineligible_on_conditional_expr_store_hir_level() {
    // Hand-built HIR (the parser has no ternary surface): `let cache;
    // function f(c) { const node = { v: 1 }; cache = c ? node : null;
    // return 1; }`. infer_layout(ConditionalExpr) returns only the LAST
    // child's layout (alternate `null` => Scalar("unknown")); the gate's own
    // judgment must treat the ternary as heap because a branch is heap, and
    // classify the module store Global.
    use kali_hir::{HirNode, HirNodeId, HirNodeKind, LoweringResult as HirLoweringResult};

    let node = |kind: HirNodeKind, text: Option<&str>, children: Vec<u32>| HirNode {
        kind,
        span: None,
        text: text.map(str::to_string),
        children: children.into_iter().map(HirNodeId::new).collect(),
    };
    let hir = HirLoweringResult {
        root: HirNodeId::new(0),
        nodes: vec![
            node(HirNodeKind::Program, None, vec![1, 4]),
            node(HirNodeKind::VarDecl, Some("let"), vec![2]),
            node(HirNodeKind::VarDeclarator, Some("cache"), vec![3]),
            node(HirNodeKind::Ident, Some("cache"), vec![]),
            node(HirNodeKind::FunctionDecl, Some("f"), vec![5, 6]),
            node(HirNodeKind::Ident, Some("c"), vec![]),
            node(HirNodeKind::Block, None, vec![7, 14, 20]),
            node(HirNodeKind::VarDecl, Some("const"), vec![8]),
            node(HirNodeKind::VarDeclarator, Some("node"), vec![9, 10]),
            node(HirNodeKind::Ident, Some("node"), vec![]),
            node(HirNodeKind::ObjectExpr, None, vec![11]),
            node(HirNodeKind::ObjectProperty, Some("v"), vec![12, 13]),
            node(HirNodeKind::Ident, Some("v"), vec![]),
            node(HirNodeKind::Literal, Some("1"), vec![]),
            node(HirNodeKind::AssignmentExpr, Some("="), vec![15, 16]),
            node(HirNodeKind::Ident, Some("cache"), vec![]),
            node(HirNodeKind::ConditionalExpr, None, vec![17, 18, 19]),
            node(HirNodeKind::Ident, Some("c"), vec![]),
            node(HirNodeKind::Ident, Some("node"), vec![]),
            node(HirNodeKind::Literal, Some("null"), vec![]),
            node(HirNodeKind::ReturnStmt, None, vec![21]),
            node(HirNodeKind::Literal, Some("1"), vec![]),
        ],
        function_flavors: Vec::new(),
        diagnostics: vec![],
    };

    let mir = crate::MirLowerer::new().lower_hir_result(&hir);
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

// --- Review fixes round 3: laundering through scalar-initialized bindings ---

#[test]
fn loop_veto_on_heap_laundered_through_scalar_binding() {
    // `x` is frozen Scalar("number") at the declarator; `x = mk()` is heap
    // but NOT a fresh literal, so a fresh-set-only fix misses it and
    // `keep = x` sees the stale scalar layout — granting a loop arena while
    // `keep` retains an arena value past the per-iteration reset.
    let mir = analyze(
        "function mk() { return { v: 1 }; }
         function g(n) {
           let keep;
           for (let i = 0; i < n; i = i + 1) {
             let x = 0;
             x = mk();
             keep = x;
           }
           return keep;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.loop_arena("g", 0));
}

#[test]
fn ineligible_on_module_store_laundered_through_scalar_binding() {
    // Module-store form of the same launder. `local` gives f a real fresh
    // site so the eligibility assertion is not vacuous (a non-allocating f
    // is ineligible trivially).
    let mir = analyze(
        "let cache;
         function mk() { return { v: 1 }; }
         function f() {
           const local = { w: 2 };
           let x = 0;
           x = mk();
           cache = x;
           return local.w;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

// --- Review fixes round 4: closure-mediated launder --------------------------

#[test]
fn ineligible_on_closure_mediated_launder() {
    // `x = mk()` happens inside `inner`, so the maybe-heap fact must land on
    // x's OWNING function (f), and f's later `cache = x` must consult the
    // owner's sets — otherwise f keeps its stale Scalar view of x, the store
    // is invisible, and opens_arena("f") is granted while mk's tree (allocated
    // into f's arena via inner) escapes into `cache` past the exit reset.
    // `local` gives f a real ScopeLocal site so the assertions are not
    // vacuous.
    let mir = analyze(
        "let cache;
         function mk() { return { v: 1 }; }
         function f() {
           const local = { w: 2 };
           let x = 0;
           const inner = function() { x = mk(); };
           inner();
           cache = x;
           let s = local.w;
           return s;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.opens_arena("f"));
    assert!(!table.arena_eligible("f"));
}

// --- Interprocedural launder pins (round 5 xfails, closed by escape_flow) ----
// These two shapes corrupted arena_eligible itself under the old walk-order-
// sensitive judgment (task-4-report.md rounds 4-5b: plain-ident dataflow was
// unmodeled in both the gate and the engine; there was no axis containment,
// only pattern containment). The escape_flow fixpoint resolves heap-ness and
// param-escape summaries order-independently, so all assertions — loop_arena
// via the driver loop, opens_arena, arena_eligible — now hold.

#[test]
fn ineligible_on_hoisted_function_launder() {
    // `helper` is declared after `cache = x` but hoisted and called before
    // it: the walk classifies `cache = x` before helper's body records
    // f.maybe(x), so the store is invisible against the stale scalar layout.
    let mir = analyze(
        "let cache;
         function mk() { return { v: 1 }; }
         function f() {
           const local = { w: 2 };
           let x = 0;
           helper();
           cache = x;
           function helper() { x = mk(); }
           let s = local.w;
           return s;
         }
         function g(n) {
           for (let i = 0; i < n; i = i + 1) { f(); }
           return 0;
         }",
    );
    let table = compute_arena_table(&mir);
    // Loop axis first: the driver loop inherits the corrupted eligible("f").
    assert!(!table.loop_arena("g", 0));
    assert!(!table.opens_arena("f"));
    assert!(!table.arena_eligible("f"));
}

#[test]
fn ineligible_on_param_mediated_escape() {
    // `retain` stores its param into a module binding via a plain-ident LHS,
    // which neither the engine's escape flags (p.escapes == false) nor the
    // gate's per-function notes see from f's side; f keeps eligible+opens
    // while mk's tree (allocated in f's arena) is retained in `sink`.
    let mir = analyze(
        "let sink;
         function retain(p) { sink = p; }
         function mk() { return { v: 1 }; }
         function f() {
           const local = { w: 2 };
           const x = mk();
           retain(x);
           let s = local.w;
           return s;
         }
         function g(n) {
           for (let i = 0; i < n; i = i + 1) { f(); }
           return 0;
         }",
    );
    let table = compute_arena_table(&mir);
    // Loop axis first: the driver loop inherits the corrupted eligible("f").
    assert!(!table.loop_arena("g", 0));
    assert!(!table.opens_arena("f"));
    assert!(!table.arena_eligible("f"));
}

// --- Interprocedural round hardening pins ------------------------------------

#[test]
fn eligible_when_arg_passed_to_nonescaping_param() {
    // THE load-bearing precision grant: a may-heap call-result argument to a
    // callee whose param never escapes must NOT veto — this is the
    // `itemCheck(bottomUpTree(d))` shape at the heart of binary-trees.
    let mir = analyze(
        "function bottomUpTree(d) { return { left: null, right: null }; }
         function itemCheck(t) { if (t.left === null) { return 1; } return 2; }
         function f(n) {
           let sum = 0;
           for (let i = 0; i < n; i = i + 1) {
             sum = sum + itemCheck(bottomUpTree(3));
           }
           return sum;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(table.loop_arena("f", 0));
    assert!(table.arena_eligible("bottomUpTree"));
}

#[test]
fn ineligible_on_call_result_stored_into_member() {
    // A call result stored into a pre-existing object's field outlives the
    // frame. The old judgment only caught FRESH literals in member stores;
    // the class-based site catches laundered/call-result values too.
    let mir = analyze(
        "function mk() { return { v: 1 }; }
         function f(p) {
           const local = { w: 2 };
           const x = mk();
           p.left = x;
           let s = local.w;
           return s;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

#[test]
fn ineligible_on_transitive_chain_across_hoisted_helpers() {
    // The round-4 transitive shape (x -> keep -> cache across hoisted
    // helpers): every link is plain-ident dataflow, the store is above the
    // helper declarations, and only the fixpoint sees the whole chain.
    let mir = analyze(
        "let cache;
         function mk() { return { v: 1 }; }
         function f() {
           const local = { w: 2 };
           let x = 0;
           let keep = 0;
           step1();
           step2();
           cache = keep;
           function step1() { x = mk(); }
           function step2() { keep = x; }
           let s = local.w;
           return s;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

#[test]
fn ineligible_on_launder_through_returning_callee() {
    // Heap-ness survives a round trip through `id`: the returned value is
    // the module-stored value.
    let mir = analyze(
        "let cache;
         function id(p) { return p; }
         function mk() { return { v: 1 }; }
         function f() {
           const local = { w: 2 };
           const x = mk();
           const y = id(x);
           cache = y;
           let s = local.w;
           return s;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

#[test]
fn ineligible_on_param_embedded_in_literal_stored_outward() {
    // The callee wraps its param in a fresh literal and stores THAT outward:
    // the embeds set must carry p so the arg site still fires in the caller.
    let mir = analyze(
        "let cache;
         function stash(p) { cache = { v: p }; }
         function mk() { return { v: 1 }; }
         function f() {
           const local = { w: 2 };
           const x = mk();
           stash(x);
           let s = local.w;
           return s;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

// --- PROBE (finding 1, final whole-branch review) ---------------------------

#[test]
fn ineligible_on_for_of_loop_var_escape() {
    // A for-of loop-head binder (`x`) has no init child, so the only place a
    // binding gets a may-heap seed (VarDeclarator-with-init) never runs for
    // it; `precollect_scope_bindings` still defines the name, so an unseeded
    // binder was judged definitely-not-heap and `sink = x` failed to veto —
    // a fail-open regression versus the deleted `arena_is_heap_value`, which
    // fell back to the binding's layout (TaggedVal/unknown => heap) here.
    // `x` aliases an arena object from `items` and escapes to module `sink`,
    // so `f` must be vetoed.
    let mir = analyze(
        "let sink;
         function f() {
           const items = [{ a: 1 }, { a: 2 }];
           for (const x of items) { sink = x; }
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
}

// NOTE: for-in was investigated the same way (see the whole-branch review's
// finding 1) and does NOT reproduce: a for-in loop var is always a string
// property key, never an alias of the source object's values, so there is no
// shape where the missing-inflow gap lets a heap value escape undetected.
// `compute_arena_table` correctly keeps `f` eligible when the loop var is
// merely stored (verified: `arena_eligible("f") == true` for `for (const k in
// items) { sink = k; }` with `items` a fresh array of objects) — no fix
// needed for for-in.

#[test]
fn ineligible_on_fresh_literal_arg_to_escaping_callee() {
    // The Task-2->Task-3 window: a FRESH OBJECT LITERAL passed directly as a
    // call argument to a known callee whose param escapes (`cache = p`).
    // Existing pins only cover call-result (`stash(x)`) and reassigned-ident
    // (`x = mk()`) forms flowing into an escaping param; this pins the
    // literal-direct form so `push_arg_site` -> `into_facts` closes it too.
    let mir = analyze(
        "let cache;
         function keep(p) { cache = p; }
         function f() {
           keep({ v: 1 });
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
}

#[test]
fn ineligible_on_heap_ident_arg_to_unknown_callee() {
    // Old behavior only vetoed FRESH LITERAL args to unknown callees; a
    // may-heap IDENT handed to an unknown callee must veto too.
    let mir = analyze(
        "function mk() { return { v: 1 }; }
         function f(cb) {
           const local = { w: 2 };
           const x = mk();
           cb(x);
           let s = local.w;
           return s;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(!table.arena_eligible("f"));
    assert!(!table.opens_arena("f"));
}

// --- Final-review Critical: opens_arena excludes mixed Returned+ScopeLocal ---

#[test]
fn opens_arena_excludes_function_with_returned_and_scope_local_site() {
    // `scratch` is a fresh object whose fields are only read (dies inside
    // `make` ⇒ ScopeLocal), but `out` is a second fresh object that is
    // returned (⇒ Returned fate). Every fresh-heap site in an arena-eligible
    // function routes into the SAME current arena; if `make` opened its own
    // per-call function arena (as the old `has_scope_local_site`-only rule
    // would grant), that arena resets on every exit path and would splice
    // `out`'s backing page onto the free list right as it is handed back to
    // the caller — a use-after-reset. `make` must stay arena_eligible (its
    // sites may still target the CALLER's current arena) but must NOT open
    // its own function arena.
    let mir = analyze(
        "function make() {
           const scratch = { a: 1, b: 2, c: 3, d: 4, e: 5, f: 6 };
           const s = scratch.a + scratch.b + scratch.c + scratch.d + scratch.e + scratch.f;
           const out = { v: s };
           return out;
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(table.arena_eligible("make"));
    assert!(!table.opens_arena("make"));
}

#[test]
fn opens_arena_excludes_function_with_bare_literal_return_and_scope_local_site() {
    // Same hazard as `opens_arena_excludes_function_with_returned_and_scope_local_site`,
    // but the returned object is a BARE LITERAL in return position
    // (`return { v: s };`) rather than first bound to a name. This exercises
    // a genuinely different code path: a literal returned directly never
    // becomes a `fresh_heap_binding` (nothing ever calls
    // `arena_note_fresh_binding` for it, since that only fires on
    // `VarDeclarator` initializers), so the binding-based fate lattice in
    // `arena_finalize_current_function` can never see it — `arena_note_return`
    // must flag it directly at the raw site.
    let mir = analyze(
        "function make() {
           const scratch = { a: 1, b: 2, c: 3, d: 4, e: 5, f: 6 };
           const s = scratch.a + scratch.b + scratch.c + scratch.d + scratch.e + scratch.f;
           return { v: s };
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(table.arena_eligible("make"));
    assert!(!table.opens_arena("make"));
}

#[test]
fn opens_arena_excludes_function_with_returned_call_result_and_scope_local_site() {
    // Round-2 generalization: `make` returns the RESULT OF A CALL to
    // `factory`, not a literal at all — neither of round 1's two
    // shape-specific paths (bare-literal check in `arena_note_return`,
    // `binding.returned` in `arena_finalize_current_function`) can see this,
    // since there is no literal and no name-bound fresh-heap binding in
    // `make`'s own body. The deferred `push_returned_site` mechanism must
    // resolve `return factory(s)`'s class (`DependsOn(Return { factory })`)
    // against the escape-flow fixpoint: `factory` itself returns a fresh
    // object literal (definitely heap), so `factory`'s Return node is
    // may-heap, and `make`'s returned-site resolves true — vetoing `make`'s
    // own function arena even though `scratch` is a genuine ScopeLocal site.
    let mir = analyze(
        "function factory(s) { return { v: s }; }
         function make() {
           const scratch = { a: 1, b: 2, c: 3, d: 4, e: 5, f: 6 };
           const s = scratch.a + scratch.b + scratch.c + scratch.d + scratch.e + scratch.f;
           return factory(s);
         }",
    );
    let table = compute_arena_table(&mir);
    assert!(table.arena_eligible("make"));
    assert!(!table.opens_arena("make"));
}

#[test]
fn opens_arena_still_true_for_all_scope_local_function() {
    // Guard against over-tightening: a function whose only fresh-heap site is
    // purely ScopeLocal (no Returned site at all) must still open its own
    // function arena.
    let mir = analyze("function f() { const o = { v: 1 }; let s = o.v; return s; }");
    let table = compute_arena_table(&mir);
    assert!(table.arena_eligible("f"));
    assert!(table.opens_arena("f"));
}
