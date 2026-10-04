//! Per-iteration env records (block-scoping spec §3.3, A-4, A-6): codegen's
//! side of the iteration plans `kali_mir::derive_env_plans` produces.
//!
//! MIR and LIR share no loop id, so an iteration plan is matched to its LIR
//! loop by cell name (A-6): after the AST rename pass, the `let` / `const`
//! names declared directly in a loop are unique within their function, and the
//! plan's loop is the one that directly declares all of the plan's cells.
//!
//! "Declared directly in a loop" must agree exactly with MIR's rule
//! (`kali_mir/src/analysis/iteration.rs`): a `let` / `const` declaration
//! reached from the loop node without passing through a nested loop or a
//! nested function. That covers the loop head and plain blocks (and `if` /
//! `switch` arms) in the body.

use std::collections::{BTreeMap, BTreeSet};

use kali_error::{_error_codes::e5, Diagnostic};
use kali_lir::{LirNode, LirNodeId, LirNodeKind};
use kali_mir::EnvPlan;

/// An owner loop currently being emitted (spec §3.3).
// Read by the per-iteration record emission (block-scoping Task 8).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct ActiveIteration {
    pub(crate) label: String,
    pub(crate) save_local: u32,
    pub(crate) plan: EnvPlan,
}

/// Iteration plans whose `iteration_of` is `function_key` (`""` for `_start`).
pub(crate) fn iteration_plans_of<'p>(
    plans: &'p BTreeMap<String, EnvPlan>,
    function_key: &str,
) -> Vec<(&'p str, &'p EnvPlan)> {
    plans
        .iter()
        .filter(|(_, plan)| plan.iteration_of.as_deref() == Some(function_key))
        .map(|(label, plan)| (label.as_str(), plan))
        .collect()
}

/// `"_start"` → `""`, else the name.
pub(crate) fn plan_key(function_name: &str) -> &str {
    if function_name == "_start" {
        ""
    } else {
        function_name
    }
}

/// The `ReprTable` namespace of env owner `owner`: an iteration label resolves
/// to its function (`kali_mir::repr_owner`), and the module root's plan key
/// `""` to `"_start"`, the namespace module bindings are recorded under
/// (`kali_types` `binding_repr_function_key`). Identity for every function
/// owner, so programs without iteration plans are unaffected.
pub(crate) fn owner_repr_namespace<'a>(
    plans: &'a BTreeMap<String, EnvPlan>,
    owner: &'a str,
) -> &'a str {
    match kali_mir::repr_owner(plans, owner) {
        "" => "_start",
        key => key,
    }
}

/// Loop kinds, by their LIR `Branch` text.
pub(crate) fn is_loop_text(text: Option<&str>) -> bool {
    matches!(
        text,
        Some("for" | "while" | "do-while" | "for-of" | "for-await-of" | "for-in")
    )
}

/// True when `id` is a LIR loop node.
pub(crate) fn is_loop_node(nodes: &[LirNode], id: LirNodeId) -> bool {
    nodes
        .get(id.0 as usize)
        .is_some_and(|node| node.kind == LirNodeKind::Branch && is_loop_text(node.text.as_deref()))
}

/// `let` / `const` names declared directly in loop `loop_id` (not in a nested
/// loop or function): its head and body, matching MIR's rule.
pub(crate) fn names_declared_in_loop(nodes: &[LirNode], loop_id: LirNodeId) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some(node) = nodes.get(loop_id.0 as usize) {
        for child in &node.children {
            collect_declared(nodes, *child, &mut out);
        }
    }
    out
}

fn collect_declared(nodes: &[LirNode], id: LirNodeId, out: &mut BTreeSet<String>) {
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    if is_loop_node(nodes, id) || crate::lower::is_function_like(nodes, id) {
        // A nested loop's declarations are its own; a nested function has its
        // own scope.
        return;
    }
    if node.kind == LirNodeKind::Instruction
        && matches!(node.text.as_deref(), Some("let" | "const"))
    {
        for declarator in &node.children {
            if let Some(name) = nodes
                .get(declarator.0 as usize)
                .and_then(|d| d.text.clone())
            {
                out.insert(name);
            }
        }
    }
    for child in &node.children {
        collect_declared(nodes, *child, out);
    }
}

/// The iteration plan whose cells are all declared directly in `loop_id`.
pub(crate) fn iteration_label_for_loop<'p>(
    nodes: &[LirNode],
    loop_id: LirNodeId,
    plans: &[(&'p str, &'p EnvPlan)],
) -> Option<&'p str> {
    if plans.is_empty() || !is_loop_node(nodes, loop_id) {
        return None;
    }
    let declared = names_declared_in_loop(nodes, loop_id);
    plans
        .iter()
        .find(|(_, plan)| {
            !plan.cells.is_empty() && plan.cells.iter().all(|cell| declared.contains(&cell.name))
        })
        .map(|(label, _)| *label)
}

/// E5506 per captured ref with `through_iteration && depth >= 2` (spec A-4).
pub(crate) fn iteration_capture_diagnostics(plans: &BTreeMap<String, EnvPlan>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (capturer, plan) in plans {
        for reference in &plan.captured {
            if reference.through_iteration && reference.depth >= 2 {
                out.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    kali_common::iteration_capture_through_record_message(
                        &reference.name,
                        capturer,
                    ),
                ));
            }
        }
    }
    out
}

/// The local that saves `g8` across iteration `label`'s record (spec §3.3).
/// The `#env` suffix is unrepresentable as a source identifier, following
/// [`crate::closure::env_save_local_name`].
pub(crate) fn iteration_save_local_name(label: &str) -> String {
    format!("__iter_save{label}#env")
}

/// The scratch local the next-iteration copy reads the previous record through.
pub(crate) fn iteration_prev_local_name() -> String {
    "__iter_prev#env".to_string()
}

#[cfg(test)]
#[path = "iteration_tests.rs"]
mod iteration_tests;
