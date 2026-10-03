//! Unresolved-member-call spec §3.1-§3.2: whether a member call that reached
//! `emit_call`'s terminal fallback is on a value this program built (refuse)
//! or on a host value (keep the warn+0 escape hatch).

use std::collections::HashSet;

use crate::*;

impl<'a> FunctionEmitter<'a> {
    /// Walk a member chain (dot and computed, any depth, through transparent
    /// wrappers) down to the node it stops at: a root identifier, a literal,
    /// a call, or any other shape. Shared with the URL/USP root walk.
    pub(crate) fn receiver_chain_root(&self, receiver: LirNodeId) -> LirNodeId {
        let mut current = self.unwrap_transparent(receiver);
        loop {
            let node = self.node(current);
            if node.kind != LirNodeKind::Value {
                return current;
            }
            match node.children.len() {
                1 if node.text.as_deref().is_some_and(|text| !text.is_empty()) => {
                    current = self.unwrap_transparent(node.children[0]);
                }
                2 if !crate::lower::is_binary_operator_text(
                    node.text.as_deref().unwrap_or_default(),
                ) =>
                {
                    current = self.unwrap_transparent(node.children[0]);
                }
                _ => return current,
            }
        }
    }

    /// The §3.1 gate. `callee_node` is the call's callee (a member access);
    /// `callee_name` its method name.
    pub(crate) fn unresolved_member_call_refuses(
        &self,
        callee_node: &LirNode,
        callee_name: &str,
    ) -> bool {
        let Some(&receiver) = callee_node.children.first() else {
            return false;
        };
        if matches!(callee_name, "call" | "apply") && self.is_intrinsic_prototype_borrow(receiver) {
            return true;
        }
        let root = self.receiver_chain_root(receiver);
        let root_node = self.node(root);
        match root_node.kind {
            LirNodeKind::Literal => true,
            LirNodeKind::Value if root_node.children.is_empty() => {
                match root_node.text.as_deref() {
                    Some(name) if !name.is_empty() => {
                        !self.root_has_host_provenance(name, &mut HashSet::new())
                    }
                    // `{}` / `[]`: a text-less childless Value.
                    _ => true,
                }
            }
            // An array or object literal with two or more children.
            LirNodeKind::Value if root_node.text.is_none() && root_node.children.len() >= 2 => true,
            _ => false,
        }
    }

    /// `Array.prototype.m` / `Object.prototype.m` / `String.prototype.m` as
    /// the receiver of `.call` / `.apply`, rooted at the unshadowed global.
    fn is_intrinsic_prototype_borrow(&self, receiver: LirNodeId) -> bool {
        let method = self.node(self.unwrap_transparent(receiver));
        if method.children.len() != 1 {
            return false;
        }
        let prototype = self.node(self.unwrap_transparent(method.children[0]));
        if prototype.text.as_deref() != Some("prototype") || prototype.children.len() != 1 {
            return false;
        }
        let global = self.node(self.unwrap_transparent(prototype.children[0]));
        global.children.is_empty()
            && global.text.as_deref().is_some_and(|name| {
                matches!(name, "Array" | "Object" | "String") && self.is_free_global(name)
            })
    }

    fn is_free_global(&self, name: &str) -> bool {
        !self.name_is_program_bound(name) && !self.functions.contains_key(name)
    }

    fn program_reassigned_names(&self) -> &HashSet<String> {
        self.program_reassigned_names_cache
            .get_or_init(|| crate::lower::program_reassigned_names(&self.program.nodes))
    }

    /// §3.2. `seen` stops an alias cycle; a name seen twice is not proven.
    fn root_has_host_provenance(&self, name: &str, seen: &mut HashSet<String>) -> bool {
        if !seen.insert(name.to_string()) {
            return false;
        }
        if self.is_free_global(name) {
            return true;
        }
        if self.name_is_declared_parameter(name) {
            return false;
        }
        let Some((kind, Some(init))) = self.declarator_of(name) else {
            return false;
        };
        if kind != "const" && self.program_reassigned_names().contains(name) {
            return false;
        }
        self.init_has_host_provenance(init, seen)
    }

    /// An initializer has host provenance when its member/call/`new` chain
    /// reaches a host root, or calls a host-derived program class.
    fn init_has_host_provenance(&self, init: LirNodeId, seen: &mut HashSet<String>) -> bool {
        let mut current = self.unwrap_transparent(init);
        loop {
            let node = self.node(current);
            match node.kind {
                LirNodeKind::Call => match node.children.first() {
                    Some(&callee) => current = self.unwrap_transparent(callee),
                    None => return false,
                },
                LirNodeKind::Value if node.children.is_empty() => {
                    let Some(name) = node.text.as_deref().filter(|name| !name.is_empty()) else {
                        return false;
                    };
                    if self.repr_table.is_host_derived_class(name) {
                        return true;
                    }
                    return self.root_has_host_provenance(name, seen);
                }
                LirNodeKind::Value
                    if node.children.len() == 1
                        && node.text.as_deref().is_some_and(|t| !t.is_empty()) =>
                {
                    current = self.unwrap_transparent(node.children[0]);
                }
                LirNodeKind::Value
                    if node.children.len() == 2
                        && !crate::lower::is_binary_operator_text(
                            node.text.as_deref().unwrap_or_default(),
                        ) =>
                {
                    current = self.unwrap_transparent(node.children[0]);
                }
                _ => return false,
            }
        }
    }

    /// The nearest declarator of `name`: the current function body first,
    /// then the module body. Does not descend into nested function-like
    /// nodes (template: `binding_is_placeholder_construct`,
    /// `intrinsics/host.rs:1821`). Returns `(kind, init)`.
    fn declarator_of(&self, name: &str) -> Option<(&str, Option<LirNodeId>)> {
        self.declarator_in(self.body, name)
            .or_else(|| self.declarator_in(self.program.root, name))
    }

    fn declarator_in(&self, root: LirNodeId, name: &str) -> Option<(&str, Option<LirNodeId>)> {
        let nodes = &self.program.nodes;
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let node = self.node(id);
            if id != root && crate::lower::is_function_like(nodes, id) {
                continue;
            }
            if node.kind == LirNodeKind::Instruction {
                if let Some(kind @ ("const" | "let" | "var")) = node.text.as_deref() {
                    for &declarator_id in &node.children {
                        let declarator = self.node(declarator_id);
                        if declarator.text.as_deref() == Some(name) {
                            return Some((kind, declarator.children.get(1).copied()));
                        }
                    }
                }
            }
            stack.extend(node.children.iter().copied());
        }
        None
    }
}

#[cfg(test)]
#[path = "member_provenance_tests.rs"]
mod member_provenance_tests;
