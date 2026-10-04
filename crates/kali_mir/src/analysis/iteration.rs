//! Per-iteration env records (block-scoping spec §3.3, A-2): which loops own a
//! record of their own. Bookkeeping only; `env_plan` turns the result into
//! plans.
//!
//! Rule for "declared in the loop": a `VarDecl` with text `let` / `const`
//! walked while the loop frame is the innermost loop frame of the current
//! function. That includes declarations in plain blocks inside the loop head or
//! body, and excludes anything inside nested loops (they have their own frame)
//! and nested functions (they have their own scope).

use std::collections::{BTreeMap, BTreeSet};

use kali_hir::{HirNodeId, HirNodeKind};

use crate::{IterationScope, OwnershipAnalyzer};

const MODULE_SCOPE_LABEL: &str = "<module>";

/// The env-plan key of a scope label (`""` for the module root).
fn plan_key(scope_label: &str) -> String {
    if scope_label == MODULE_SCOPE_LABEL {
        String::new()
    } else {
        scope_label.to_string()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LoopFrame {
    label: String,
    /// Scope-stack depth of the function the loop is in.
    depth: usize,
    /// Scope label of that function (`"<module>"` at the root).
    scope_label: String,
    declared: BTreeSet<String>,
    closures: BTreeSet<String>,
    has_registration: bool,
}

#[derive(Debug, Default)]
pub(crate) struct IterationCollector {
    stack: Vec<LoopFrame>,
    counters: BTreeMap<String, usize>,
    /// Closure label -> the loop labels it was created in, outermost first.
    closure_loops: BTreeMap<String, Vec<String>>,
    /// Owner label -> (enclosing function's parent label, loop labels of the
    /// same function that enclosed it, outermost first).
    owners: BTreeMap<String, (Option<String>, Vec<String>)>,
    scopes: Vec<IterationScope>,
}

impl IterationCollector {
    fn open(&mut self, depth: usize, scope_label: &str) {
        let key = plan_key(scope_label);
        let n = self.counters.entry(key.clone()).or_insert(0);
        let label = crate::env_plan::iteration_label(&key, *n);
        *n += 1;
        self.stack.push(LoopFrame {
            label,
            depth,
            scope_label: scope_label.to_string(),
            declared: BTreeSet::new(),
            closures: BTreeSet::new(),
            has_registration: false,
        });
    }

    fn note_declared(&mut self, depth: usize, names: impl IntoIterator<Item = String>) {
        if let Some(frame) = self.stack.iter_mut().rev().find(|f| f.depth == depth) {
            frame.declared.extend(names);
        }
    }

    fn note_closure(&mut self, enclosing_depth: usize, closure_label: &str) {
        let mut loops = Vec::new();
        for frame in self.stack.iter_mut().filter(|f| f.depth == enclosing_depth) {
            frame.closures.insert(closure_label.to_string());
            loops.push(frame.label.clone());
        }
        if !loops.is_empty() {
            self.closure_loops.insert(closure_label.to_string(), loops);
        }
    }

    fn note_registration(&mut self, depth: usize) {
        for frame in self.stack.iter_mut().filter(|f| f.depth == depth) {
            frame.has_registration = true;
        }
    }

    /// Close the innermost frame. `captured_by` maps each declared name to the
    /// capturer labels of its binding in the loop's function scope; `parents`
    /// is the analyzer's `parent_labels`, used to tell whether a capturer is a
    /// closure of the frame or nested inside one.
    fn close(
        &mut self,
        captured_by: &BTreeMap<String, Vec<String>>,
        parents: &BTreeMap<String, Option<String>>,
    ) {
        let frame = self.stack.pop().expect("balanced loop frames");
        if !frame.has_registration {
            return;
        }
        let within = |capturer: &str, closure: &str| {
            let mut cursor = Some(capturer.to_string());
            while let Some(label) = cursor {
                if label == closure {
                    return true;
                }
                cursor = parents.get(&label).cloned().flatten();
            }
            false
        };
        let cells: Vec<String> = frame
            .declared
            .iter()
            .filter(|name| {
                captured_by.get(*name).is_some_and(|capturers| {
                    capturers
                        .iter()
                        .any(|c| frame.closures.iter().any(|closure| within(c, closure)))
                })
            })
            .cloned()
            .collect();
        if cells.is_empty() {
            return;
        }
        let enclosing = self
            .stack
            .iter()
            .filter(|f| f.depth == frame.depth)
            .map(|f| f.label.clone())
            .collect();
        let function_parent =
            (frame.scope_label != MODULE_SCOPE_LABEL).then(|| frame.scope_label.clone());
        self.owners
            .insert(frame.label.clone(), (function_parent, enclosing));
        self.scopes.push(IterationScope {
            label: frame.label,
            function: plan_key(&frame.scope_label),
            cells,
        });
    }

    /// Re-parent owners and the closures created inside them. Call after the
    /// whole walk: an enclosing loop only becomes an owner when it closes.
    pub(crate) fn relabel(&self, parent_labels: &mut BTreeMap<String, Option<String>>) {
        for (owner, (function_parent, enclosing)) in &self.owners {
            let parent = enclosing
                .iter()
                .rev()
                .find(|l| self.owners.contains_key(*l))
                .cloned()
                .or_else(|| function_parent.clone());
            parent_labels.insert(owner.clone(), parent);
        }
        for (closure, loops) in &self.closure_loops {
            if let Some(owner) = loops.iter().rev().find(|l| self.owners.contains_key(*l)) {
                parent_labels.insert(closure.clone(), Some(owner.clone()));
            }
        }
    }

    pub(crate) fn into_scopes(self) -> Vec<IterationScope> {
        self.scopes
    }
}

impl OwnershipAnalyzer<'_> {
    pub(crate) fn iteration_open_loop(&mut self) {
        let (depth, label) = (self.current_scope_index(), self.current_scope_label());
        self.iteration.open(depth, &label);
    }

    pub(crate) fn iteration_close_loop(&mut self) {
        let Some(frame) = self.iteration.stack.last() else {
            return;
        };
        let captured: BTreeMap<String, Vec<String>> = self
            .scope_stack
            .last()
            .map(|scope| {
                frame
                    .declared
                    .iter()
                    .filter_map(|name| {
                        let binding = scope.bindings.get(scope.get_binding_index(name)?)?;
                        Some((name.clone(), binding.captured_by.iter().cloned().collect()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.iteration.close(&captured, &self.parent_labels);
    }

    /// Record `let` / `const` declarator names of a `VarDecl` node.
    pub(crate) fn iteration_note_var_decl(
        &mut self,
        keyword: Option<&str>,
        declarators: &[HirNodeId],
    ) {
        if !matches!(keyword, Some("let" | "const")) {
            return;
        }
        let names: Vec<String> = declarators
            .iter()
            .map(|id| &self.nodes[id.0 as usize])
            .filter(|node| node.kind == HirNodeKind::VarDeclarator)
            .filter_map(|node| node.text.clone())
            .collect();
        let depth = self.current_scope_index();
        self.iteration.note_declared(depth, names);
    }

    /// Call before `push_scope` for a function: the current scope encloses it.
    pub(crate) fn iteration_note_closure(&mut self, closure_label: &str) {
        let depth = self.current_scope_index();
        self.iteration.note_closure(depth, closure_label);
    }

    pub(crate) fn iteration_note_call(&mut self, callee: HirNodeId) {
        let callee = &self.nodes[callee.0 as usize];
        let Some(name) = callee.text.as_deref() else {
            return;
        };
        if kali_common::is_deferred_registration_callee(name, !callee.children.is_empty()) {
            let depth = self.current_scope_index();
            self.iteration.note_registration(depth);
        }
    }
}

#[cfg(test)]
#[path = "iteration_tests.rs"]
mod iteration_tests;
