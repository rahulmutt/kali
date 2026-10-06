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
use wasm_encoder::{Function, Instruction};

/// An owner loop currently being emitted (spec §3.3).
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

/// `let` / `const` names declared in the subtree at `id` (a loop clause), not
/// in a nested loop or function: a `for` head's bindings when `id` is its init.
pub(crate) fn names_declared_in(nodes: &[LirNode], id: LirNodeId) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    collect_declared(nodes, id, &mut out);
    out
}

/// True when the subtree at `id` holds an unlabeled or labelled `continue` that
/// is not inside a nested loop or function, i.e. one that targets the loop
/// whose body `id` is (spec A-5). A `switch` is entered: its `continue`
/// reaches the enclosing loop.
pub(crate) fn contains_direct_continue(nodes: &[LirNode], id: LirNodeId) -> bool {
    let Some(node) = nodes.get(id.0 as usize) else {
        return false;
    };
    if is_loop_node(nodes, id) || crate::lower::is_function_like(nodes, id) {
        return false;
    }
    if node.kind == LirNodeKind::Branch
        && node
            .text
            .as_deref()
            .is_some_and(|text| text.starts_with("continue"))
    {
        return true;
    }
    node.children
        .iter()
        .any(|child| contains_direct_continue(nodes, *child))
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

/// E5506 per captured ref of depth 2 or more whose hop path crosses an
/// iteration record (`through_iteration`, spec A-4) or whose owner IS one
/// (ruling R13, spec A-9): neither walk is lowered, and the depth-2 fallback
/// would read a stale or zero value.
pub(crate) fn iteration_capture_diagnostics(plans: &BTreeMap<String, EnvPlan>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (capturer, plan) in plans {
        for reference in &plan.captured {
            let owned_by_iteration = plans
                .get(&reference.owner)
                .is_some_and(|owner| owner.iteration_of.is_some());
            if reference.depth >= 2 && (reference.through_iteration || owned_by_iteration) {
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

/// The owners of `capturer`'s LOWERED captures: depth-1 refs whose owner-keyed
/// cell is promotable, the predicate `resolve_capture_access` lowers on. Empty
/// for a function that is not a capturer.
pub(crate) fn lowered_capture_owners<'p>(
    plans: &'p BTreeMap<String, EnvPlan>,
    repr_table: &kali_common::ReprTable,
    capturer: &str,
) -> BTreeSet<&'p str> {
    let Some(plan) = plans.get(capturer) else {
        return BTreeSet::new();
    };
    plan.captured
        .iter()
        .filter(|reference| {
            // A capturer's ref: the widening its owner's promotion site used
            // (baseline for an iteration-plan owner, captured-bindings for a
            // function-plan owner), so this set is exactly what
            // `resolve_capture_access_inner` lowers.
            reference.depth == 1
                && crate::closure::cell_is_promotable(
                    repr_table,
                    owner_repr_namespace(plans, &reference.owner),
                    &reference.name,
                    reference.is_scalar,
                    crate::closure::Widening::for_captured_ref(plans, reference),
                )
        })
        .map(|reference| reference.owner.as_str())
        .collect()
}

/// Spec A-9 (ruling H5(b)): a direct call made inside an owner loop to
/// `callee` runs with `g8` switched to the env the outermost owner loop was
/// entered with, when `callee` is a capturer whose lowered captures are all
/// owned by functions (not by iteration records). `env_safety` admits such a
/// call site against the caller's body context because codegen performs the
/// switch; both decide it here.
pub(crate) fn call_leaves_iteration_record(
    plans: &BTreeMap<String, EnvPlan>,
    repr_table: &kali_common::ReprTable,
    callee: &str,
) -> bool {
    let owners = lowered_capture_owners(plans, repr_table, callee);
    !owners.is_empty()
        && owners.iter().all(|owner| {
            plans
                .get(*owner)
                .is_none_or(|plan| plan.iteration_of.is_none())
        })
}

/// Whether iteration plan `label` (of `plans`) has a promotable cell, so
/// codegen gives its loop per-iteration records (`lower.rs` reserves the save
/// local on the same predicate).
pub(crate) fn iteration_plan_has_records(
    plans: &BTreeMap<String, EnvPlan>,
    repr_table: &kali_common::ReprTable,
    label: &str,
) -> bool {
    plans.get(label).is_some_and(|plan| {
        let namespace = owner_repr_namespace(plans, label);
        // iteration cells are not widened (captured-bindings §1.1)
        plan.cells.iter().any(|cell| {
            crate::closure::cell_is_promotable(
                repr_table,
                namespace,
                &cell.name,
                cell.is_scalar,
                crate::closure::Widening::Baseline,
            )
        })
    })
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

/// The local that holds the current iteration record while a call switches
/// `g8` away from it (spec A-9).
pub(crate) fn iteration_call_save_local_name() -> String {
    "__iter_call_save#env".to_string()
}

/// The emission of per-iteration records (spec §3.3, A-5): `g8` holds the
/// current iteration's record for the whole of an owner loop's iteration, and
/// the record's parent is the `g8` the loop was entered with.
impl<'a> crate::FunctionEmitter<'a> {
    /// The iteration label of loop `id` (an MIR iteration owner), marking the
    /// plan placed for the A-6 backstop. `None` when no plan of this function
    /// matches the loop.
    pub(crate) fn iteration_label_of_loop(&mut self, id: LirNodeId) -> Option<String> {
        let plans: Vec<(&str, &EnvPlan)> = self
            .iteration_plans
            .iter()
            .map(|(label, plan)| (label.as_str(), plan))
            .collect();
        let label = iteration_label_for_loop(&self.program.nodes, id, &plans)?.to_string();
        self.emitted_iterations.insert(label.clone());
        Some(label)
    }

    /// Whether owner loop `label` carries per-iteration records: its plan has
    /// a promotable cell, so `lower.rs` reserved its save local. Otherwise its
    /// cells stay locals and nothing moves into a record.
    pub(crate) fn iteration_has_records(&self, label: &str) -> bool {
        self.locals.contains_key(&iteration_save_local_name(label))
    }

    /// The name of the first cell of iteration plan `label`, for diagnostics.
    pub(crate) fn iteration_first_cell(&self, label: &str) -> String {
        self.iteration_plans
            .iter()
            .find(|(candidate, _)| candidate == label)
            .and_then(|(_, plan)| plan.cells.first())
            .map(|cell| cell.name.clone())
            .unwrap_or_default()
    }

    /// Whether cell `name` of iteration plan `label` is promotable (it lives
    /// in the record, not a local): `lower.rs`'s predicate, in the namespace
    /// of the loop's function.
    fn iteration_cell_is_promotable(&self, label: &str, name: &str, is_scalar: bool) -> bool {
        // iteration cells are not widened (captured-bindings §1.1)
        crate::closure::cell_is_promotable(
            self.repr_table,
            owner_repr_namespace(self.env_plans, label),
            name,
            is_scalar,
            crate::closure::Widening::Baseline,
        )
    }

    /// The record offset of promotable cell `name` of iteration plan `label`
    /// (a loop binding the loop itself stores, such as a `for…in` key).
    pub(crate) fn iteration_record_cell_offset(&self, label: &str, name: &str) -> Option<u32> {
        let (_, plan) = self
            .iteration_plans
            .iter()
            .find(|(candidate, _)| candidate == label)?;
        let cell = plan.cell_for(name)?;
        self.iteration_cell_is_promotable(label, name, cell.is_scalar)
            .then_some(cell.offset)
    }

    /// Refuse an owner loop's records with E5506 `message` (spec A-5). The
    /// loop is then emitted without records; the program does not run.
    pub(crate) fn refuse_iteration_records(&mut self, message: String) {
        self.diagnostics
            .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
    }

    /// Enter owner loop `label`: save `g8` into the loop's save local and make
    /// the loop the innermost active iteration.
    pub(crate) fn enter_iteration(&mut self, function: &mut Function, label: &str) {
        let save_local = self.locals[&iteration_save_local_name(label)];
        let plan = self
            .iteration_plans
            .iter()
            .find(|(candidate, _)| candidate == label)
            .map(|(_, plan)| plan.clone())
            .unwrap_or_default();
        function.instruction(&Instruction::GlobalGet(self.current_env_global()));
        function.instruction(&Instruction::LocalSet(save_local));
        self.active_iterations.push(ActiveIteration {
            label: label.to_string(),
            save_local,
            plan,
        });
    }

    /// Allocate a record for the innermost active iteration, with parent = the
    /// `g8` that loop was entered with (its save local, never the previous
    /// record, so records never chain to each other), and set `g8` to it.
    pub(crate) fn alloc_iteration_record(&mut self, function: &mut Function) {
        let Some(active) = self.active_iterations.last() else {
            return;
        };
        let cell_count = active.plan.cells.len() as u32;
        let save_local = active.save_local;
        crate::closure::emit_env_alloc(
            function,
            self.alloc_global_fn_index(),
            cell_count,
            self.current_env_global(),
            save_local,
        );
    }

    /// `for (let …; test; update)`: allocate the next iteration's record and
    /// copy the current values of the head cells `head` into it, so `update`
    /// and the next iteration run on the copy and the closures of this
    /// iteration keep theirs.
    pub(crate) fn copy_head_cells_into_new_record(
        &mut self,
        function: &mut Function,
        head: &BTreeSet<String>,
    ) {
        let Some(active) = self.active_iterations.last() else {
            return;
        };
        let label = active.label.clone();
        let mut copied: Vec<u32> = active
            .plan
            .cells
            .iter()
            .filter(|cell| {
                head.contains(&cell.name)
                    && self.iteration_cell_is_promotable(&label, &cell.name, cell.is_scalar)
            })
            .map(|cell| cell.offset)
            .collect();
        copied.sort_unstable();
        let env_global = self.current_env_global();
        let prev = self.locals[&iteration_prev_local_name()];
        function.instruction(&Instruction::GlobalGet(env_global));
        function.instruction(&Instruction::LocalSet(prev));
        self.alloc_iteration_record(function);
        for offset in copied {
            crate::closure::emit_env_base_addr(function, env_global, 0);
            function.instruction(&Instruction::LocalGet(prev));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Load(crate::closure::env_memarg(
                8 + offset,
            )));
            function.instruction(&Instruction::I64Store(crate::closure::env_memarg(
                8 + offset,
            )));
        }
    }

    /// Emit the direct call `Call(index)` to `callee`, its arguments already
    /// on the stack. Inside an active owner loop, a callee whose captures are
    /// all owned by functions ([`call_leaves_iteration_record`]) runs with
    /// `g8` set to the outermost active loop's save local (the env that loop
    /// was entered with), and the iteration record is restored after the call
    /// (spec A-9). Every other call is emitted unchanged.
    pub(crate) fn emit_direct_call(&mut self, function: &mut Function, index: u32, callee: &str) {
        let outer = self
            .active_iterations
            .first()
            .map(|active| active.save_local);
        let switch =
            outer.filter(|_| call_leaves_iteration_record(self.env_plans, self.repr_table, callee));
        let Some(outer) = switch else {
            function.instruction(&Instruction::Call(index));
            return;
        };
        let env_global = self.current_env_global();
        let hold = self.locals[&iteration_call_save_local_name()];
        function.instruction(&Instruction::GlobalGet(env_global));
        function.instruction(&Instruction::LocalSet(hold));
        function.instruction(&Instruction::LocalGet(outer));
        function.instruction(&Instruction::GlobalSet(env_global));
        function.instruction(&Instruction::Call(index));
        function.instruction(&Instruction::LocalGet(hold));
        function.instruction(&Instruction::GlobalSet(env_global));
    }

    /// Leave the innermost owner loop: restore `g8` from its save local.
    pub(crate) fn exit_iteration(&mut self, function: &mut Function) {
        let Some(active) = self.active_iterations.pop() else {
            return;
        };
        function.instruction(&Instruction::LocalGet(active.save_local));
        function.instruction(&Instruction::GlobalSet(self.current_env_global()));
    }

    /// The A-6 backstop: one E5506 per iteration plan of this function whose
    /// loop was never placed. Called once the whole body has been emitted.
    pub(crate) fn push_unplaced_iteration_diagnostics(&mut self) {
        let unplaced: Vec<String> = self
            .iteration_plans
            .iter()
            .map(|(label, _)| label)
            .filter(|label| !self.emitted_iterations.contains(*label))
            .cloned()
            .collect();
        for label in unplaced {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::iteration_record_unplaced_message(&label),
            ));
        }
    }
}

#[cfg(test)]
#[path = "iteration_tests.rs"]
mod iteration_tests;
