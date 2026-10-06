use super::*;

/// outer() owns `c` (captured by inc); inc() captures `c` at depth 1.
/// `c` is scalar → is_scalar true; offset 0 (first cell after header).
#[test]
fn scalar_capture_one_level_produces_owner_cell_and_ref() {
    let analysis = crate::test_support::analyze(
        "function outer(){ let c = 0; function inc(){ c += 1; } inc(); return c; }",
    );
    let plans = derive_env_plans(&analysis);

    let outer = plans.get("outer").expect("outer plan");
    assert!(outer.owns_env);
    assert_eq!(
        outer.cells,
        vec![EnvCell {
            name: "c".into(),
            offset: 0,
            is_scalar: true,
            is_parameter: false,
            is_tagged: false,
        }]
    );

    let inc = plans.get("inc").expect("inc plan");
    assert!(!inc.owns_env);
    assert_eq!(
        inc.captured,
        vec![CapturedRef {
            name: "c".into(),
            depth: 1,
            offset: 0,
            is_scalar: true,
            is_parameter: false,
            is_tagged: false,
            owner: "outer".into(),
            through_iteration: false,
        }]
    );
}

/// a() owns `g`; c() (nested a > b > c) reads it through the intermediate
/// `b`, which owns NO cell. Per spec §3.4 a no-cell function allocates no env
/// record and is transparent to the env chain, so it contributes no hop:
/// `depth` counts env-OWNING ancestors (only `a`), NOT lexical function
/// scopes. Updated in Task 5 (env chains) from the original depth-2
/// lexical-hop assumption — the depth-2 count would over-walk the runtime
/// parent chain (which links only env-owning records) and address `a`'s
/// parent instead of `a`.
#[test]
fn grandparent_capture_skips_no_cell_intermediate_depth_one() {
    let analysis = crate::test_support::analyze(
        "function a(){ let g = 5; function b(){ function c(){ return g; } return c(); } return b(); }",
    );
    let plans = derive_env_plans(&analysis);
    let c = plans.get("c").expect("c plan");
    assert_eq!(
        c.captured,
        vec![CapturedRef {
            name: "g".into(),
            depth: 1,
            offset: 0,
            is_scalar: true,
            is_parameter: false,
            is_tagged: false,
            owner: "a".into(),
            through_iteration: false,
        }]
    );
}

/// §3.4 counting pin with an env-OWNING intermediate: `a` owns `g`, `b` owns
/// `h` (both captured by `c`), `c` owns nothing. `c` capturing `h` is one
/// env hop (`b`); `c` capturing `g` is TWO env hops (`b` then `a`) — because
/// here `b` DOES own a record, unlike the transparent-intermediate case
/// above. This is the shape whose runtime parent chain genuinely links two
/// records.
#[test]
fn two_env_owning_ancestors_is_depth_two() {
    let analysis = crate::test_support::analyze(
        "function a(){ let g = 5; function b(){ let h = 6; function c(){ return g + h; } return c(); } return b(); }",
    );
    let plans = derive_env_plans(&analysis);
    let c = plans.get("c").expect("c plan");
    let g = c
        .captured
        .iter()
        .find(|r| r.name == "g")
        .expect("captures g");
    let h = c
        .captured
        .iter()
        .find(|r| r.name == "h")
        .expect("captures h");
    assert_eq!(
        (g.depth, g.owner.as_str()),
        (2, "a"),
        "g: two env hops via b then a"
    );
    assert_eq!((h.depth, h.owner.as_str()), (1, "b"), "h: one env hop (b)");
}

/// An ANONYMOUS function expression is a capture OWNER: `g` captures `inner`,
/// which is owned by the anonymous `function(){...}` assigned to `f`. The
/// anonymous owner must be first-class in the nesting map (keyed by its
/// `__kali_fn_N` analysis label), so `g`'s CapturedRef for `inner` is NOT
/// silently dropped and the owner's plan is discoverable.
///
/// NB: on this branch HIR already assigns each anonymous function a
/// `__kali_fn_N` name into the node `text`, and the analysis reuses that
/// text as its scope label, so the finding's stated `text = None`
/// transparency does not trigger here — this passes on HEAD too. It is kept
/// as a by-construction regression pin: the label-keyed map must keep
/// anonymous owners first-class even if the two naming channels ever diverge.
#[test]
fn anonymous_owner_is_first_class_capture_ref_not_dropped() {
    let analysis = crate::test_support::analyze(
        "function outer(){ let c = 0; let f = function(){ let inner = 1; function g(){ return inner + c; } return g(); }; return f(); }",
    );
    let plans = derive_env_plans(&analysis);

    // The anonymous fn-expr is labeled __kali_fn_0 (first synthetic name);
    // it owns `inner` (captured by g), so it owns an env.
    let anon = plans.get("__kali_fn_0").expect("anonymous owner plan");
    assert!(anon.owns_env, "anonymous fn-expr owns env for `inner`");
    assert_eq!(
        anon.cells,
        vec![EnvCell {
            name: "inner".into(),
            offset: 0,
            is_scalar: true,
            is_parameter: false,
            is_tagged: false,
        }]
    );

    // g captures `inner` (owned by the anonymous fn, depth 1) — this ref was
    // silently dropped when the anonymous owner was transparent in the map.
    let g = plans.get("g").expect("g plan");
    assert!(
        g.captured
            .iter()
            .any(|c| c.name == "inner" && c.depth == 1 && c.is_scalar),
        "g must capture `inner` at depth 1 (owned by the anonymous fn), got {:?}",
        g.captured
    );
}

/// An ANONYMOUS function expression is an INTERMEDIATE: `inner` (named) is
/// nested inside an anonymous `function(){...}` (assigned to `mid`) which is
/// nested inside named `outer`. `inner` captures `v` from `outer`. The
/// anonymous intermediate owns NO cell, so per spec §3.4 it is transparent to
/// the env chain and contributes no hop: depth = 1 (only `outer` owns a
/// record). Updated in Task 5 from the original depth-2 lexical-hop
/// assumption — env depth counts env-OWNING ancestors, and an ownership-less
/// intermediate (anonymous or not) does not add one. The sibling
/// `anonymous_owner_is_first_class_capture_ref_not_dropped` still counts an
/// anonymous fn that DOES own a cell, so anonymous first-class-ness is
/// unaffected — only ownership decides a hop.
#[test]
fn anonymous_no_cell_intermediate_is_transparent_depth_one() {
    let analysis = crate::test_support::analyze(
        "function outer(){ let v = 7; let mid = function(){ function inner(){ return v; } return inner(); }; return mid(); }",
    );
    let plans = derive_env_plans(&analysis);
    let inner = plans.get("inner").expect("inner plan");
    assert_eq!(
        inner.captured,
        vec![CapturedRef {
            name: "v".into(),
            depth: 1,
            offset: 0,
            is_scalar: true,
            is_parameter: false,
            is_tagged: false,
            owner: "outer".into(),
            through_iteration: false,
        }],
        "a no-cell anonymous intermediate is transparent to the env chain (§3.4)"
    );
}

/// A class is lowered to a `MirNodeKind::Function` node (`lower.rs`), but the
/// analysis walk creates NO scope for it — the class body's method nests
/// directly under the enclosing function. Keying the nesting map on the
/// node tree therefore injects a PHANTOM hop for the class, OVERCOUNTING
/// capture depth.
///
/// Here `h` (nested in method `m`, itself in `outer`) captures `z` from
/// `outer` at the true function-scope depth 2 (`outer` > `m` > `h`). The
/// node-tree map counted class `K` as a third hop (depth 3) — a real
/// miscompile-class defect. This is the concrete, RED-on-HEAD reproduction
/// of the finding's "node-tree nesting diverges from the analysis labels"
/// class (reviewer Minor note 1). The label-keyed map never sees `K`, so the
/// depth is 2.
#[test]
fn class_node_does_not_inject_phantom_capture_hop() {
    let analysis = crate::test_support::analyze(
        "function outer(){ let z = 0; class K { m(){ let q = 1; function h(){ return q + z; } return h(); } } return new K().m(); }",
    );
    let plans = derive_env_plans(&analysis);
    let h = plans.get("h").expect("h plan");

    let z = h
        .captured
        .iter()
        .find(|c| c.name == "z")
        .expect("h captures z");
    assert_eq!(
        z.depth, 2,
        "class K must not add a phantom hop: outer>m>h = depth 2, got {}",
        z.depth
    );

    // `q` (owned by method `m`, one function-scope hop up) stays depth 1.
    let q = h
        .captured
        .iter()
        .find(|c| c.name == "q")
        .expect("h captures q");
    assert_eq!(q.depth, 1);
}

#[test]
fn an_iteration_owner_holds_the_loop_cells_and_the_closure_reads_them_at_depth_one() {
    let p = crate::test_support::analyze(
        "function m(){ for(let i=0;i<3;i++){ setTimeout(()=>console.log(i), 0); } } m();",
    );
    let plans = derive_env_plans(&p);
    let iter = plans.get("m{iter0}").expect("iteration plan");
    assert_eq!(iter.iteration_of.as_deref(), Some("m"));
    assert_eq!(
        iter.cells,
        vec![EnvCell {
            name: "i".into(),
            offset: 0,
            is_scalar: true,
            is_parameter: false,
            is_tagged: false,
        }]
    );
    assert!(!plans.get("m").map(|f| f.owns_env).unwrap_or(false));
    let capture = plans
        .values()
        .flat_map(|plan| plan.captured.iter())
        .find(|r| r.name == "i")
        .expect("closure captures i");
    assert_eq!(
        (
            capture.depth,
            capture.owner.as_str(),
            capture.through_iteration
        ),
        (1, "m{iter0}", false)
    );
    assert_eq!(repr_owner(&plans, "m{iter0}"), "m");
}

#[test]
fn a_function_binding_captured_from_inside_an_owner_is_through_iteration_at_depth_two() {
    let p = crate::test_support::analyze(
        "function m(){ let a=10; for(let i=0;i<2;i++){ setTimeout(()=>console.log(a+i),0); } } m();",
    );
    let plans = derive_env_plans(&p);
    let a = plans
        .values()
        .flat_map(|plan| plan.captured.iter())
        .find(|r| r.name == "a")
        .expect("closure captures a");
    assert_eq!(
        (a.depth, a.owner.as_str(), a.through_iteration),
        (2, "m", true)
    );
}

#[test]
fn nested_owners_put_the_outer_cell_at_depth_two_without_crossing_a_foreign_record() {
    let p = crate::test_support::analyze(
        "for(let i=0;i<2;i++){ for(let j=0;j<2;j++){ setTimeout(()=>console.log(i,j),0); } }",
    );
    let plans = derive_env_plans(&p);
    let refs: Vec<_> = plans
        .values()
        .flat_map(|plan| plan.captured.iter())
        .collect();
    let j = refs.iter().find(|r| r.name == "j").unwrap();
    let i = refs.iter().find(|r| r.name == "i").unwrap();
    assert_eq!(
        (j.depth, j.owner.as_str(), j.through_iteration),
        (1, "{iter1}", false)
    );
    assert_eq!(
        (i.depth, i.owner.as_str(), i.through_iteration),
        (2, "{iter0}", true)
    );
}

#[test]
fn a_program_without_owners_gets_the_same_plans_as_before() {
    let p = crate::test_support::analyze(
        "function outer(){ let c = 0; function inc(){ c += 1; } inc(); return c; }",
    );
    let plans = derive_env_plans(&p);
    assert!(plans.values().all(|plan| plan.iteration_of.is_none()));
    assert!(plans
        .values()
        .flat_map(|plan| plan.captured.iter())
        .all(|r| !r.through_iteration));
}

/// Captured-bindings A-1: a parameter's layout is TaggedVal, so its cell is
/// a heap cell; the plan says so, and that it is a parameter.
#[test]
fn a_captured_parameter_cell_is_tagged_and_a_parameter() {
    let analysis = crate::test_support::analyze("function f(k){ const g=()=>k; return g(); }");
    let plans = derive_env_plans(&analysis);
    assert_eq!(
        plans["f"].cells,
        vec![EnvCell {
            name: "k".into(),
            offset: 0,
            is_scalar: false,
            is_parameter: true,
            is_tagged: true,
        }]
    );
    let reference = plans["__kali_fn_0"]
        .captured_for("k")
        .expect("g captures k");
    assert_eq!(reference.depth, 1);
    assert!(reference.is_parameter);
    assert!(reference.is_tagged);
}

#[test]
fn a_local_copied_from_a_parameter_is_tagged_but_not_a_parameter() {
    let analysis =
        crate::test_support::analyze("function f(k){ let n=k; const g=()=>n; return g(); }");
    let cell = derive_env_plans(&analysis)["f"]
        .cell_for("n")
        .cloned()
        .expect("n cell");
    assert!(cell.is_tagged);
    assert!(!cell.is_parameter);
    assert!(!cell.is_scalar);
}

#[test]
fn a_local_from_arithmetic_is_scalar_and_not_tagged() {
    let analysis =
        crate::test_support::analyze("function f(k){ let n=k+0; const g=()=>n; return g(); }");
    let cell = derive_env_plans(&analysis)["f"]
        .cell_for("n")
        .cloned()
        .expect("n cell");
    assert!(cell.is_scalar);
    assert!(!cell.is_tagged);
    assert!(!cell.is_parameter);
}
