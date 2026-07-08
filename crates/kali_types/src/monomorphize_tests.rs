//! Unit tests for the object-shape monomorphization analysis (`MonoPlan`).
//!
//! These exercise the four acceptance cases from the Task 7a design doc
//! (§5 Task 1): the `dump(A);dump(B)` repro, the transitive `outer→inner`
//! chain (probe P4), the same-shape-twice identity case (probe P1), and the
//! fail-closed cases (ambiguous `cond ? A : B` merge and a non-converging
//! recursion) that MUST yield an empty plan so the existing E5506 conflict is
//! preserved downstream.

use super::monomorphize::{compute_mono_plan, MonoPlan, SpecKey};
use std::collections::BTreeSet;

fn plan(src: &str) -> MonoPlan {
    let parsed = crate::test_support::parse_statements(src);
    compute_mono_plan(&parsed)
}

/// Build a single-object-param `SpecKey` at param index `idx` with the given
/// ordered field names.
fn key1(idx: usize, fields: &[&str]) -> SpecKey {
    SpecKey {
        params: vec![(idx, fields.iter().map(|s| s.to_string()).collect())],
    }
}

/// The set of distinct specialization keys the plan assigns to `func`.
fn key_set(plan: &MonoPlan, func: &str) -> BTreeSet<SpecKey> {
    plan.specialization_keys(func)
        .map(|ks| ks.iter().cloned().collect())
        .unwrap_or_default()
}

/// The set of callee specialization keys targeted by calls to `callee`.
fn targeted_specs(plan: &MonoPlan, callee: &str) -> BTreeSet<SpecKey> {
    plan.call_bindings()
        .iter()
        .filter(|b| b.callee == callee)
        .map(|b| b.callee_spec.clone())
        .collect()
}

#[test]
fn dump_two_distinct_shapes_specializes_two_ways() {
    let p = plan(
        "function dump(t){var s=0;for(var k in t){s=s+1;}return s;} \
         var A={a:1.0,b:2.0,c:3.0}; var B={x:1.0,y:2.0}; \
         console.log(dump(A)); console.log(dump(B));",
    );
    let abc = key1(0, &["a", "b", "c"]);
    let xy = key1(0, &["x", "y"]);
    // dump is reached by two distinct object-param tuples => specialized twice.
    assert_eq!(
        key_set(&p, "dump"),
        BTreeSet::from([abc.clone(), xy.clone()]),
        "dump should specialize into {{a,b,c}} and {{x,y}}"
    );
    // The two call sites map to the two distinct keys.
    assert_eq!(
        targeted_specs(&p, "dump"),
        BTreeSet::from([abc, xy]),
        "the two dump(...) call sites map to the two distinct keys"
    );
    // Exactly two call bindings (one per site), both from module scope.
    let dump_bindings: Vec<_> = p
        .call_bindings()
        .iter()
        .filter(|b| b.callee == "dump")
        .collect();
    assert_eq!(dump_bindings.len(), 2);
    for b in dump_bindings {
        assert_eq!(b.caller, "_start");
        assert!(b.caller_spec.params.is_empty());
    }
    assert!(!p.is_empty());
}

#[test]
fn transitive_outer_inner_specializes_both_levels() {
    let p = plan(
        "function inner(t){var s=0;for(var k in t){s=s+1;}return s;} \
         function outer(t){return inner(t);} \
         var A={a:1.0,b:2.0,c:3.0}; var B={x:1.0,y:2.0}; \
         console.log(outer(A)); console.log(outer(B));",
    );
    let abc = key1(0, &["a", "b", "c"]);
    let xy = key1(0, &["x", "y"]);
    // Both levels specialize transitively.
    assert_eq!(
        key_set(&p, "outer"),
        BTreeSet::from([abc.clone(), xy.clone()]),
        "outer specializes on the two incoming shapes"
    );
    assert_eq!(
        key_set(&p, "inner"),
        BTreeSet::from([abc.clone(), xy.clone()]),
        "inner specializes transitively via outer's param"
    );
    // The nested inner(t) call inside outer maps per-context: outer$abc → inner$abc,
    // outer$xy → inner$xy.
    let nested: BTreeSet<(SpecKey, SpecKey)> = p
        .call_bindings()
        .iter()
        .filter(|b| b.caller == "outer" && b.callee == "inner")
        .map(|b| (b.caller_spec.clone(), b.callee_spec.clone()))
        .collect();
    assert_eq!(
        nested,
        BTreeSet::from([(abc.clone(), abc), (xy.clone(), xy)]),
        "inner(t) inside outer resolves per enclosing specialization"
    );
}

#[test]
fn same_shape_twice_is_not_specialized() {
    // Probe P1: two structurally-identical {a,b,c} objects reach dump.
    let p = plan(
        "function dump(t){var s=0;for(var k in t){s=s+1;}return s;} \
         var A={a:1.0,b:2.0,c:3.0}; var B={a:9.0,b:8.0,c:7.0}; \
         console.log(dump(A)); console.log(dump(B));",
    );
    assert!(
        p.specialization_keys("dump").is_none(),
        "one distinct tuple => no clone needed"
    );
    assert!(p.is_empty());
}

#[test]
fn single_shape_is_not_specialized() {
    // Probe P3: one object reaches dump.
    let p = plan(
        "function dump(t){var s=0;for(var k in t){s=s+1;}return s;} \
         var A={a:1.0,b:2.0,c:3.0}; console.log(dump(A));",
    );
    assert!(p.specialization_keys("dump").is_none());
    assert!(p.is_empty());
}

#[test]
fn ambiguous_conditional_merge_bails_to_empty_plan() {
    // `var o = cond ? A : B; dump(o)` — a single slot genuinely holds two
    // shapes at one use site; no per-call-site partition exists. Must NOT
    // specialize (E5506 preserved downstream).
    let p = plan(
        "function dump(t){var s=0;for(var k in t){s=s+1;}return s;} \
         var A={a:1.0,b:2.0,c:3.0}; var B={x:1.0,y:2.0}; var cond=1.0; \
         var o = cond ? A : B; console.log(dump(o));",
    );
    assert!(
        p.specialization_keys("dump").is_none(),
        "ambiguous merge must not specialize"
    );
    assert!(p.is_empty());
}

#[test]
fn non_converging_recursion_bails_to_empty_plan() {
    // A recursive call whose argument merges two shapes cannot be cleanly
    // partitioned into a finite specialization set — bail (empty plan),
    // preserving E5506. The external f(A) call is clean, but the recursive
    // f(o) passes an ambiguous shape.
    let p = plan(
        "function f(t){var s=0;for(var k in t){s=s+1;} var o = s>0.0 ? A : B; return f(o);} \
         var A={a:1.0,b:2.0,c:3.0}; var B={x:1.0,y:2.0}; console.log(f(A));",
    );
    assert!(
        p.specialization_keys("f").is_none(),
        "non-converging recursion must bail to an empty plan"
    );
    assert!(p.is_empty());
}

#[test]
fn bounded_self_recursion_same_shape_converges_and_specializes() {
    // Termination witness: genuine self-recursion that REUSES the same tuple
    // per external shape converges (f$abc recurses to f$abc, f$xy to f$xy) —
    // it specializes cleanly and terminates rather than bailing or hanging.
    let p = plan(
        "function f(t){var s=0;for(var k in t){s=s+1;} if(s>1.0){return f(t);} return s;} \
         var A={a:1.0,b:2.0,c:3.0}; var B={x:1.0,y:2.0}; \
         console.log(f(A)); console.log(f(B));",
    );
    let abc = key1(0, &["a", "b", "c"]);
    let xy = key1(0, &["x", "y"]);
    assert_eq!(key_set(&p, "f"), BTreeSet::from([abc.clone(), xy.clone()]));
    // The recursive call maps each context to itself.
    let rec: BTreeSet<(SpecKey, SpecKey)> = p
        .call_bindings()
        .iter()
        .filter(|b| b.caller == "f" && b.callee == "f")
        .map(|b| (b.caller_spec.clone(), b.callee_spec.clone()))
        .collect();
    assert_eq!(
        rec,
        BTreeSet::from([(abc.clone(), abc), (xy.clone(), xy)]),
        "self-recursion reuses the same tuple per specialization"
    );
}

#[test]
fn callee_through_unspecialized_multishape_caller_bails() {
    // Fail-closed regression (Task 7a-1 follow-up): `f` is ambiguous at the
    // `f(o)` site so `f` is (correctly) NOT specialized. But `f(A)`/`f(B)`
    // still seed `g` with two distinct shapes, so a naive fixpoint specializes
    // `g`. `f`'s single un-cloned body has ONE `g(t)` call site that would then
    // need two contradictory targets (g${a,b,c} vs g${x,y}) — a broken
    // call_site → tuple mapping. `g` must be bailed too (empty plan → E5506).
    let p = plan(
        "function g(t){var s=0;for(var k in t){s=s+1;}return s;} \
         function f(t){return g(t);} \
         var A={a:1.0,b:2.0,c:3.0}; var B={x:1.0,y:2.0}; var cond=1.0; \
         var o = cond ? A : B; \
         console.log(f(A)); console.log(f(B)); console.log(f(o));",
    );
    assert!(
        p.specialization_keys("f").is_none(),
        "f is ambiguous at f(o) and must not be specialized"
    );
    assert!(
        p.specialization_keys("g").is_none(),
        "g reached only through the un-specialized multi-shape caller f must \
         be bailed — its single g(t) call site cannot be cleanly routed"
    );
    // No call binding may target g (there is no clean site to rewrite).
    assert!(
        targeted_specs(&p, "g").is_empty(),
        "no contradictory call binding may target g"
    );
    // No two bindings may collide on (caller, caller_spec, ordinal) with
    // different callee_spec — the call_site → tuple functional contract.
    let mut seen: BTreeSet<(String, SpecKey, usize)> = BTreeSet::new();
    for b in p.call_bindings() {
        assert!(
            seen.insert((b.caller.clone(), b.caller_spec.clone(), b.ordinal)),
            "duplicate (caller, caller_spec, ordinal) key => contradictory \
             call_site → tuple mapping"
        );
    }
    assert!(p.is_empty());
}

#[test]
fn no_object_params_is_empty_plan() {
    let p = plan("function add(a,b){return a+b;} console.log(add(1.0,2.0));");
    assert!(p.is_empty());
    assert!(p.specialization_keys("add").is_none());
}
