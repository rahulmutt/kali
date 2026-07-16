//! Per-function closure environment plans derived from the MIR capture set.
//!
//! Stage C gives kali environment-pointer closures. This module is the pure
//! analysis bridge: from the capture edges MIR already records
//! (`MirBinding::captured_by`, populated in `analysis/walk.rs::resolve_use`)
//! plus the function nesting the analysis records in its own scope-label key
//! space (`MirProgram::parent_labels`, populated in
//! `analysis/scope.rs::push_scope`), it derives a per-function [`EnvPlan`] — the
//! set of bindings a function must promote into an env record, and the
//! outer-env references it reads through the parent chain.
//!
//! Both inputs are keyed on the SAME labels (`__kali_fn_N` / function names), so
//! anonymous functions are first-class and the node tree is never consulted for
//! nesting — a non-scope `Function` node (e.g. a class) cannot inject a phantom
//! hop into a capture depth.
//!
//! No codegen decisions live here; later Stage C tasks consume these plans.

use std::collections::BTreeMap;

use crate::{LayoutDescriptor, MirFunctionKind, MirProgram};

/// One promoted binding: it lives in an env cell because a nested function
/// captures it. `offset` is its byte offset within the owning env record,
/// AFTER the 8-byte parent_env_ptr header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvCell {
    pub name: String,
    pub offset: u32,
    pub is_scalar: bool,
}

/// A reference, from inside function F, to a binding owned by an ancestor
/// env `depth` links up the parent chain (0 = F's own env).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedRef {
    pub name: String,
    pub depth: u32,
    pub offset: u32,
    pub is_scalar: bool,
}

/// The closure plan for a single function, keyed by its `__kali_fn_N` name
/// (module root uses the reserved key "" — it never owns an env; its captured
/// scalars are module globals, handled elsewhere).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvPlan {
    /// This function has >=1 promoted cell.
    pub owns_env: bool,
    /// Its own promoted bindings, in fixed order.
    pub cells: Vec<EnvCell>,
    /// Outer bindings it reads/writes.
    pub captured: Vec<CapturedRef>,
}

impl EnvPlan {
    /// The own promoted cell for `name` (a binding this function OWNS in its own
    /// env record), if any. Depth-0 access.
    pub fn cell_for(&self, name: &str) -> Option<&EnvCell> {
        self.cells.iter().find(|cell| cell.name == name)
    }

    /// The outer-scope capture reference for `name` (a binding owned by an
    /// ancestor env this function reads/writes through the parent chain), if
    /// any.
    pub fn captured_for(&self, name: &str) -> Option<&CapturedRef> {
        self.captured.iter().find(|reference| reference.name == name)
    }
}

/// Classify an env cell's storage from its MIR layout.
///
/// Exhaustive over [`LayoutDescriptor`] (`crates/kali_mir/src/layout.rs:5`) —
/// no `_ =>` arm, so a future layout variant is a compile error here rather
/// than a silent fail-open:
/// - `Scalar` → scalar cell (an i64/f64 stored inline).
/// - `Struct | Array | Closure | TaggedVal` → heap cell (an i64 heap pointer).
fn is_scalar_cell(layout: &LayoutDescriptor) -> bool {
    match layout {
        LayoutDescriptor::Scalar(_) => true,
        LayoutDescriptor::Struct { .. }
        | LayoutDescriptor::Array { .. }
        | LayoutDescriptor::Closure { .. }
        | LayoutDescriptor::TaggedVal => false,
    }
}

/// The reserved plan key for the module root (see [`EnvPlan`]).
fn function_key(name: &Option<String>) -> String {
    name.clone().unwrap_or_default()
}

/// Number of function-scope hops from `from` up to `to` (0 = same function).
///
/// Walks the parent chain the analysis recorded in `MirProgram::parent_labels`
/// (see [`derive_env_plans`]). Returns `None` if `to` is not an ancestor of
/// `from` (defensive: capture edges always point from a descendant to an
/// ancestor, so this should not happen for a well-formed capture set).
fn scope_hops(from: &str, to: &str, parents: &BTreeMap<String, Option<String>>) -> Option<u32> {
    let mut current = from.to_string();
    let mut depth = 0u32;
    loop {
        if current == to {
            return Some(depth);
        }
        match parents.get(&current) {
            Some(Some(parent)) => {
                current = parent.clone();
                depth += 1;
            }
            // Reached the module root (Some(None)) or an unknown scope without
            // matching `to`: `to` is not an ancestor.
            Some(None) | None => return None,
        }
    }
}

/// Derive an [`EnvPlan`] per function name from a completed MIR analysis.
///
/// The `MirProgram` is the crate's finalized analysis handle (produced by
/// `MirLowerer::lower_hir_result`); its public `functions`/`bindings` tables
/// carry the capture set, and `parent_labels` carries the scope nesting in the
/// same label key space (so anonymous functions are first-class and the node
/// tree is never consulted for nesting).
pub fn derive_env_plans(program: &MirProgram) -> BTreeMap<String, EnvPlan> {
    let parents = &program.parent_labels;

    let mut plans: BTreeMap<String, EnvPlan> = BTreeMap::new();
    // fn key -> (binding name -> (offset, is_scalar)) for its promoted cells.
    let mut cell_offsets: BTreeMap<String, BTreeMap<String, (u32, bool)>> = BTreeMap::new();

    // Pass 1: each function's own promoted cells.
    for function in &program.functions {
        let key = function_key(&function.name);
        let is_module = function.kind == MirFunctionKind::Module;

        // Cells: bindings owned by this function that some nested function
        // captures (non-empty `captured_by`), sorted by name for determinism,
        // packed at 8 bytes each. The module root never owns an env — its
        // captured scalars are module globals, handled elsewhere.
        let mut owned: Vec<_> = function
            .bindings
            .iter()
            .filter(|binding| !binding.captured_by.is_empty())
            .collect();
        owned.sort_by(|a, b| a.name.cmp(&b.name));

        let mut cells = Vec::new();
        let mut offsets = BTreeMap::new();
        if !is_module {
            for (index, binding) in owned.iter().enumerate() {
                let offset = index as u32 * 8;
                let is_scalar = is_scalar_cell(&binding.layout);
                cells.push(EnvCell {
                    name: binding.name.clone(),
                    offset,
                    is_scalar,
                });
                offsets.insert(binding.name.clone(), (offset, is_scalar));
            }
        }

        let plan = plans.entry(key.clone()).or_default();
        plan.owns_env = !cells.is_empty();
        plan.cells = cells;
        cell_offsets.insert(key, offsets);
    }

    // Pass 2: invert the capture edges into per-capturer refs. A binding owned
    // by ancestor A with `captured_by` containing F becomes a `CapturedRef` in
    // F, at A's cell offset, `depth` scope-hops up. Module-owned captures are
    // module globals (A has no cells) and are excluded here.
    for function in &program.functions {
        if function.kind == MirFunctionKind::Module {
            continue;
        }
        let owner_key = function_key(&function.name);
        let owner_offsets = &cell_offsets[&owner_key];
        for binding in &function.bindings {
            let Some(&(offset, is_scalar)) = owner_offsets.get(&binding.name) else {
                continue;
            };
            for capturer in &binding.captured_by {
                if let Some(depth) = scope_hops(capturer, &owner_key, parents) {
                    plans.entry(capturer.clone()).or_default().captured.push(
                        CapturedRef {
                            name: binding.name.clone(),
                            depth,
                            offset,
                            is_scalar,
                        },
                    );
                }
            }
        }
    }

    // Deterministic capture order.
    for plan in plans.values_mut() {
        plan.captured.sort_by(|a, b| {
            (a.name.as_str(), a.depth, a.offset).cmp(&(b.name.as_str(), b.depth, b.offset))
        });
        plan.captured.dedup();
    }

    plans
}

#[cfg(test)]
mod tests {
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
                is_scalar: true
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
                is_scalar: true
            }]
        );
    }

    /// a() owns `g`; c() (nested a > b > c) reads it two function-scope hops up.
    /// Defines the `depth` contract: hops count function scopes, not env owners.
    #[test]
    fn grandparent_capture_is_depth_two() {
        let analysis = crate::test_support::analyze(
            "function a(){ let g = 5; function b(){ function c(){ return g; } return c(); } return b(); }",
        );
        let plans = derive_env_plans(&analysis);
        let c = plans.get("c").expect("c plan");
        assert_eq!(
            c.captured,
            vec![CapturedRef {
                name: "g".into(),
                depth: 2,
                offset: 0,
                is_scalar: true
            }]
        );
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
                is_scalar: true
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
    /// anonymous hop must be counted: depth = 2, not 1.
    ///
    /// NB: like the sibling test, HIR names the anonymous intermediate
    /// `__kali_fn_N` in the node text, so this already reports depth 2 on HEAD.
    /// Kept as a by-construction regression pin under the label-keyed map.
    #[test]
    fn anonymous_intermediate_hop_is_counted_depth_two() {
        let analysis = crate::test_support::analyze(
            "function outer(){ let v = 7; let mid = function(){ function inner(){ return v; } return inner(); }; return mid(); }",
        );
        let plans = derive_env_plans(&analysis);
        let inner = plans.get("inner").expect("inner plan");
        assert_eq!(
            inner.captured,
            vec![CapturedRef {
                name: "v".into(),
                depth: 2,
                offset: 0,
                is_scalar: true
            }],
            "the anonymous intermediate scope must count as a hop (depth 2)"
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
}
