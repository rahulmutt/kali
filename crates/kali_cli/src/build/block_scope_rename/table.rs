//! The scope table built by pass A: every scope, in `enter` order, with the
//! bindings each one owns.

use std::collections::BTreeMap;

use super::walk::{BindKind, Hooks, ScopeKind};

pub(crate) type ScopeId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Binding {
    pub ordinal: u32,
    pub kind: BindKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Scope {
    pub parent: Option<ScopeId>,
    pub kind: ScopeKind,
    /// The nearest Module/Function scope (itself for those kinds).
    pub frame: ScopeId,
    /// Function nesting level of `frame`: module 0, a top-level function 1, …
    pub frame_level: u32,
    /// Block nesting depth inside `frame` (the frame scope itself is 0).
    pub depth: u32,
    pub bindings: BTreeMap<String, Binding>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ScopeTable {
    pub scopes: Vec<Scope>,
}

impl ScopeTable {
    /// The scope that binds `name` as seen from `from`, walking parents.
    pub(crate) fn resolve(&self, from: ScopeId, name: &str) -> Option<ScopeId> {
        let mut cursor = Some(from);
        while let Some(id) = cursor {
            if self.scopes[id].bindings.contains_key(name) {
                return Some(id);
            }
            cursor = self.scopes[id].parent;
        }
        None
    }

    /// Where a `bind(kind)` issued in `current` lands: `var` goes to the
    /// frame, everything else to `current`.
    pub(crate) fn target_scope(&self, current: ScopeId, kind: BindKind) -> ScopeId {
        if kind == BindKind::Var {
            self.scopes[current].frame
        } else {
            current
        }
    }
}

#[derive(Default)]
pub(crate) struct Collector {
    table: ScopeTable,
    stack: Vec<ScopeId>,
    next_ordinal: u32,
}

impl Collector {
    pub(crate) fn finish(self) -> ScopeTable {
        debug_assert!(self.stack.is_empty(), "unbalanced enter/exit");
        self.table
    }
}

impl Hooks for Collector {
    fn enter(&mut self, kind: ScopeKind, _label: Option<&str>) {
        let id = self.table.scopes.len();
        let parent = self.stack.last().copied();
        let (frame, frame_level, depth) = match (kind, parent) {
            (ScopeKind::Module, _) => (id, 0, 0),
            (ScopeKind::Function, Some(p)) => (id, self.table.scopes[p].frame_level + 1, 0),
            (ScopeKind::Function, None) => (id, 1, 0),
            (ScopeKind::Block, Some(p)) => {
                let ps = &self.table.scopes[p];
                (ps.frame, ps.frame_level, ps.depth + 1)
            }
            (ScopeKind::Block, None) => (id, 0, 0),
        };
        self.table.scopes.push(Scope {
            parent,
            kind,
            frame,
            frame_level,
            depth,
            bindings: BTreeMap::new(),
        });
        self.stack.push(id);
    }

    fn exit(&mut self) {
        self.stack.pop();
    }

    fn bind(&mut self, name: &mut String, kind: BindKind) {
        let current = *self.stack.last().expect("bind inside a scope");
        let target = self.table.target_scope(current, kind);
        let ordinal = self.next_ordinal;
        self.next_ordinal += 1;
        self.table.scopes[target]
            .bindings
            .entry(name.clone())
            .or_insert(Binding { ordinal, kind });
    }

    fn reference(&mut self, _name: &mut String) {}
}

#[cfg(test)]
#[path = "table_tests.rs"]
mod table_tests;
