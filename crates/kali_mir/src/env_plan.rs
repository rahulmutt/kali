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

use std::collections::{BTreeMap, BTreeSet};

use crate::{LayoutDescriptor, MirBindingKind, MirFunctionKind, MirProgram};

/// One promoted binding: it lives in an env cell because a nested function
/// captures it. `offset` is its byte offset within the owning env record,
/// AFTER the 8-byte parent_env_ptr header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvCell {
    pub name: String,
    pub offset: u32,
    pub is_scalar: bool,
    /// The owner's MIR binding is a parameter (captured-bindings A-1).
    pub is_parameter: bool,
    /// The binding's layout is `TaggedVal` (captured-bindings A-1).
    pub is_tagged: bool,
}

/// A reference, from inside function F, to a binding owned by an ancestor
/// env `depth` env-record hops up the parent chain.
///
/// `depth` counts only ancestors that OWN an env record (have >=1 promoted
/// cell), NOT lexical function-scope hops: a function that owns no cell
/// allocates no record and is transparent to the env chain (spec §3.4), so it
/// contributes no hop. Depth 1 = the nearest env-OWNING ancestor. See
/// [`env_owning_hops`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedRef {
    pub name: String,
    pub depth: u32,
    pub offset: u32,
    pub is_scalar: bool,
    /// The owner's MIR binding is a parameter (captured-bindings A-1).
    pub is_parameter: bool,
    /// The binding's layout is `TaggedVal` (captured-bindings A-1).
    pub is_tagged: bool,
    /// Plan key of the function that OWNS this binding (the ancestor whose env
    /// record holds the cell). Codegen must consult the OWNER's repr namespace
    /// (not the capturer's) when deciding whether the cell was promoted — the
    /// owner's promotion verdict is what actually allocated (or did not
    /// allocate) the cell. See the C1 review Finding 1.
    pub owner: String,
    /// The hop path from the capturer to `owner` passes through a per-iteration
    /// record that is not `owner` (spec A-4).
    pub through_iteration: bool,
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
    /// `Some(function plan key)` for a per-iteration plan.
    pub iteration_of: Option<String>,
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
        self.captured
            .iter()
            .find(|reference| reference.name == name)
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

/// What a reference to an env cell needs to know about it.
#[derive(Debug, Clone, Copy)]
struct CellFacts {
    offset: u32,
    is_scalar: bool,
    is_parameter: bool,
    is_tagged: bool,
}

/// The reserved plan key for the module root (see [`EnvPlan`]).
fn function_key(name: &Option<String>) -> String {
    name.clone().unwrap_or_default()
}

/// The label of the `n`th iteration candidate (loop, in pre-order) of the
/// function with plan key `function_key` (`""` for the module root).
pub fn iteration_label(function_key: &str, n: usize) -> String {
    format!("{function_key}{{iter{n}}}")
}

/// The repr namespace of an env owner: an iteration plan's function, else the owner.
pub fn repr_owner<'a>(plans: &'a BTreeMap<String, EnvPlan>, owner: &'a str) -> &'a str {
    plans
        .get(owner)
        .and_then(|plan| plan.iteration_of.as_deref())
        .unwrap_or(owner)
}

/// Whether the parent walk from `from` to `to` steps through an iteration
/// label other than `to` (spec A-4).
fn path_crosses_iteration(
    from: &str,
    to: &str,
    parents: &BTreeMap<String, Option<String>>,
    iteration_labels: &BTreeSet<String>,
) -> bool {
    let mut cursor = parents.get(from).cloned().flatten();
    while let Some(label) = cursor {
        if label == to {
            return false;
        }
        if iteration_labels.contains(&label) {
            return true;
        }
        cursor = parents.get(&label).cloned().flatten();
    }
    false
}

fn cell_facts(cell: &EnvCell) -> CellFacts {
    CellFacts {
        offset: cell.offset,
        is_scalar: cell.is_scalar,
        is_parameter: cell.is_parameter,
        is_tagged: cell.is_tagged,
    }
}

/// Sort cells by name and pack them at 8 bytes each.
fn renumber(mut cells: Vec<EnvCell>) -> Vec<EnvCell> {
    cells.sort_by(|a, b| a.name.cmp(&b.name));
    for (index, cell) in cells.iter_mut().enumerate() {
        cell.offset = index as u32 * 8;
    }
    cells
}

/// Number of ENV-OWNING ancestor hops from `from` up to `to` (`to` inclusive,
/// `from` exclusive).
///
/// Walks the parent chain the analysis recorded in `MirProgram::parent_labels`
/// (see [`derive_env_plans`]), counting a hop ONLY when the ancestor stepped
/// into owns an env record (`env_owners` membership). A function that owns no
/// cell allocates no record and is transparent to the env chain (spec §3.4), so
/// it is skipped — the depth counts env-record hops, not lexical function-scope
/// hops. `to` is the capture's owner, which always owns an env (it holds the
/// promoted cell), so a well-formed capture yields `depth >= 1`.
///
/// Returns `None` if `to` is not an ancestor of `from` (defensive: capture
/// edges always point from a descendant to an ancestor, so this should not
/// happen for a well-formed capture set).
fn env_owning_hops(
    from: &str,
    to: &str,
    parents: &BTreeMap<String, Option<String>>,
    env_owners: &BTreeSet<String>,
) -> Option<u32> {
    let mut current = from.to_string();
    let mut depth = 0u32;
    loop {
        if current == to {
            return Some(depth);
        }
        match parents.get(&current) {
            Some(Some(parent)) => {
                // §3.4: count this step only if the ancestor being stepped INTO
                // owns an env record; a no-cell ancestor is transparent.
                if env_owners.contains(parent) {
                    depth += 1;
                }
                current = parent.clone();
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
    // fn key -> (binding name -> facts) for its promoted cells.
    let mut cell_offsets: BTreeMap<String, BTreeMap<String, CellFacts>> = BTreeMap::new();

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
                let cell = EnvCell {
                    name: binding.name.clone(),
                    offset,
                    is_scalar: is_scalar_cell(&binding.layout),
                    is_parameter: binding.kind == MirBindingKind::Parameter,
                    is_tagged: matches!(binding.layout, LayoutDescriptor::TaggedVal),
                };
                offsets.insert(binding.name.clone(), cell_facts(&cell));
                cells.push(cell);
            }
        }

        let plan = plans.entry(key.clone()).or_default();
        plan.owns_env = !cells.is_empty();
        plan.cells = cells;
        cell_offsets.insert(key, offsets);
    }

    // Per-iteration plans: an owner loop's cells move out of its function's plan.
    let iteration_labels: BTreeSet<String> = program
        .iteration_scopes
        .iter()
        .map(|scope| scope.label.clone())
        .collect();
    // (function key, binding name) -> owning iteration label.
    let mut owner_of: BTreeMap<(String, String), String> = BTreeMap::new();
    for scope in &program.iteration_scopes {
        let moved: Vec<EnvCell> = if scope.function.is_empty() {
            // The module plan never has cells; build them from its bindings.
            program
                .functions
                .iter()
                .filter(|f| f.kind == MirFunctionKind::Module)
                .flat_map(|f| f.bindings.iter())
                .filter(|b| !b.captured_by.is_empty() && scope.cells.contains(&b.name))
                .map(|b| EnvCell {
                    name: b.name.clone(),
                    offset: 0,
                    is_scalar: is_scalar_cell(&b.layout),
                    is_parameter: b.kind == MirBindingKind::Parameter,
                    is_tagged: matches!(b.layout, LayoutDescriptor::TaggedVal),
                })
                .collect()
        } else {
            plans
                .get_mut(&scope.function)
                .map(|f| {
                    let (moved, kept): (Vec<_>, Vec<_>) = f
                        .cells
                        .drain(..)
                        .partition(|c| scope.cells.contains(&c.name));
                    f.cells = renumber(kept);
                    f.owns_env = !f.cells.is_empty();
                    moved
                })
                .unwrap_or_default()
        };
        let cells = renumber(moved);
        for cell in &cells {
            owner_of.insert(
                (scope.function.clone(), cell.name.clone()),
                scope.label.clone(),
            );
        }
        plans.insert(
            scope.label.clone(),
            EnvPlan {
                owns_env: !cells.is_empty(),
                cells,
                captured: Vec::new(),
                iteration_of: Some(scope.function.clone()),
            },
        );
    }
    // Rebuild the offset tables from the final plans.
    cell_offsets = plans
        .iter()
        .map(|(key, plan)| {
            let offsets = plan
                .cells
                .iter()
                .map(|c| (c.name.clone(), cell_facts(c)))
                .collect();
            (key.clone(), offsets)
        })
        .collect();

    // The set of functions that own an env record (>=1 promoted cell). Only
    // these contribute a hop to a capture depth (spec §3.4). Built after Pass 1
    // set every plan's `owns_env`. The module root ("") never owns an env.
    let env_owners: BTreeSet<String> = plans
        .iter()
        .filter(|(_, plan)| plan.owns_env)
        .map(|(key, _)| key.clone())
        .collect();

    // Pass 2: invert the capture edges into per-capturer refs. A binding owned
    // by ancestor A with `captured_by` containing F becomes a `CapturedRef` in
    // F, at A's cell offset, `depth` ENV-OWNING hops up. Module-owned captures
    // are module globals (A has no cells) and are excluded here.
    for function in &program.functions {
        let function_name = function_key(&function.name);
        for binding in &function.bindings {
            let owner_key = owner_of
                .get(&(function_name.clone(), binding.name.clone()))
                .cloned()
                .unwrap_or_else(|| function_name.clone());
            let Some(&facts) = cell_offsets
                .get(&owner_key)
                .and_then(|offsets| offsets.get(&binding.name))
            else {
                continue;
            };
            for capturer in &binding.captured_by {
                if let Some(depth) = env_owning_hops(capturer, &owner_key, parents, &env_owners) {
                    let through_iteration =
                        path_crosses_iteration(capturer, &owner_key, parents, &iteration_labels);
                    plans
                        .entry(capturer.clone())
                        .or_default()
                        .captured
                        .push(CapturedRef {
                            name: binding.name.clone(),
                            depth,
                            offset: facts.offset,
                            is_scalar: facts.is_scalar,
                            is_parameter: facts.is_parameter,
                            is_tagged: facts.is_tagged,
                            owner: owner_key.clone(),
                            through_iteration,
                        });
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
#[path = "env_plan_tests.rs"]
mod env_plan_tests;
