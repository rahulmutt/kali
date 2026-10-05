//! Per-iteration env records (block-scoping spec §3.3, A-2): which loops own a
//! record of their own. Bookkeeping only; `env_plan` turns the result into
//! plans.
//!
//! Rule for "declared in the loop": a `VarDecl` with text `let` / `const`
//! walked while the loop frame is the innermost loop frame of the current
//! function. That includes declarations in plain blocks inside the loop head or
//! body, and excludes anything inside nested loops (they have their own frame)
//! and nested functions (they have their own scope).
//!
//! Rule for "registers a loop closure" (spec A-9, overriding A-2(b)): a
//! deferred-registration call textually inside the loop, including one in a
//! function nested in the loop (ruling R8), whose callback argument is a
//! closure created in the loop that captures, directly or through a nested
//! closure, a `let` / `const` declared directly in the loop, or an identifier
//! bound to such a closure. A callback the walk cannot resolve to a function
//! (a parameter, a member, a call result) is treated as such a closure: a
//! false owner only costs a refusal, a missed one a wrong value.

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
    /// Function labels registered as deferred callbacks inside the loop.
    registered: BTreeSet<String>,
    /// A deferred callback inside the loop did not resolve to a function.
    registers_unresolved: bool,
}

/// The callback argument of a deferred-registration call, as the walk sees it.
#[derive(Debug, Clone)]
pub(crate) enum RegisteredCallback {
    /// A function expression or a name that resolves to a function.
    Function(String),
    /// Anything else (a parameter, a member, a call result).
    Unresolved,
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
            registered: BTreeSet::new(),
            registers_unresolved: false,
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

    /// A registration call is noted on every open loop frame, including frames
    /// of enclosing functions: the call is textually inside each of those
    /// loops (ruling R8, spec A-9).
    fn note_registration(&mut self, callback: &RegisteredCallback) {
        for frame in &mut self.stack {
            match callback {
                RegisteredCallback::Function(label) => {
                    frame.registered.insert(label.clone());
                }
                RegisteredCallback::Unresolved => frame.registers_unresolved = true,
            }
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
        if frame.registered.is_empty() && !frame.registers_unresolved {
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
        // Spec A-9: a registered callback must be a closure of this loop (or
        // nested in one) that captures one of the loop's own bindings.
        let registers_loop_closure = frame.registers_unresolved
            || frame.registered.iter().any(|callback| {
                frame
                    .closures
                    .iter()
                    .any(|closure| within(callback, closure))
                    && cells.iter().any(|name| {
                        captured_by.get(name).is_some_and(|capturers| {
                            capturers.iter().any(|capturer| within(capturer, callback))
                        })
                    })
            });
        if !registers_loop_closure {
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

    /// The child index of call `children`'s callback argument when the call
    /// is a deferred registration (`children[0]` is the callee): the first
    /// argument of a bare scheduling callee, the second of `addEventListener`.
    pub(crate) fn iteration_registration_callback_index(
        &self,
        children: &[HirNodeId],
    ) -> Option<usize> {
        let callee = &self.nodes[children.first()?.0 as usize];
        let name = callee.text.as_deref()?;
        let is_member = !callee.children.is_empty();
        kali_common::is_deferred_registration_callee(name, is_member).then_some(if is_member {
            2
        } else {
            1
        })
    }

    /// Note a deferred registration of callback `argument`, already walked;
    /// `functions_before` is `self.functions.len()` before that walk.
    pub(crate) fn iteration_note_registration(
        &mut self,
        argument: HirNodeId,
        functions_before: usize,
    ) {
        let callback = match self.nodes[argument.0 as usize].kind {
            HirNodeKind::FunctionExpr => self.function_name_from_recent_functions(functions_before),
            HirNodeKind::Ident => self.function_target_from_node(argument),
            _ => None,
        }
        .map_or(RegisteredCallback::Unresolved, RegisteredCallback::Function);
        self.iteration.note_registration(&callback);
    }
}

#[cfg(test)]
#[path = "iteration_tests.rs"]
mod iteration_tests;
