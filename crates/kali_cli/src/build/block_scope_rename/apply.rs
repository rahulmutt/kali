//! Pass B, step 2: rewrite binding sites and references per the plan.

use super::plan::RenamePlan;
use super::table::{ScopeId, ScopeTable};
use super::walk::{BindKind, Hooks, ScopeKind};

/// Replays the walk with the same scope numbering as the `Collector`.
pub(crate) struct Renamer<'a> {
    table: &'a ScopeTable,
    plan: &'a RenamePlan,
    stack: Vec<ScopeId>,
    next_scope: ScopeId,
}

impl<'a> Renamer<'a> {
    pub(crate) fn new(table: &'a ScopeTable, plan: &'a RenamePlan) -> Self {
        Self {
            table,
            plan,
            stack: Vec::new(),
            next_scope: 0,
        }
    }

    fn rewrite(&self, scope: ScopeId, name: &mut String) {
        if let Some(new) = self.plan.get(&(scope, name.clone())) {
            *name = new.clone();
        }
    }
}

impl Hooks for Renamer<'_> {
    fn enter(&mut self, _kind: ScopeKind, _label: Option<&str>) {
        self.stack.push(self.next_scope);
        self.next_scope += 1;
    }

    fn exit(&mut self) {
        self.stack.pop();
    }

    fn bind(&mut self, name: &mut String, kind: BindKind) {
        if name.is_empty() {
            return;
        }
        let current = *self.stack.last().expect("bind inside a scope");
        let target = self.table.target_scope(current, kind);
        self.rewrite(target, name);
    }

    fn reference(&mut self, name: &mut String) {
        if name.is_empty() {
            return;
        }
        let current = *self.stack.last().expect("reference inside a scope");
        if let Some(scope) = self.table.resolve(current, name) {
            self.rewrite(scope, name);
        }
    }
}

#[cfg(test)]
#[path = "apply_tests.rs"]
mod apply_tests;
