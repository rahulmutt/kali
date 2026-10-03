//! Unresolved-member-call spec §3.1-§3.2: whether a member call that reached
//! `emit_call`'s terminal fallback is on a value this program built (refuse)
//! or on a host value (keep the warn+0 escape hatch).

use std::collections::HashSet;

use crate::*;

/// One lexical scope of the host-provenance lookup (ruling R7).
struct LexicalScope {
    /// The body walked for declarators (nested function-like nodes skipped).
    body: LirNodeId,
    params: ScopeParams,
}

enum ScopeParams {
    /// The function being emitted: `name_is_declared_parameter`.
    Current,
    /// An enclosing function's parameters (empty for the module body).
    Names(Vec<String>),
}

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
                        // No lexical scope chain (the current body is not
                        // reachable from the module root): not proven, refuse.
                        let Some(scopes) = self.lexical_scopes() else {
                            return true;
                        };
                        !self.root_has_host_provenance(name, &scopes, 0, &mut HashSet::new())
                    }
                    // `this`, `{}` or `[]`: a text-less childless Value.
                    // `this` in a method or constructor of a host-derived
                    // class is that class's instance (ruling R8). LIR spells
                    // `this` like `{}` / `[]`, so an empty literal start in
                    // such a method keeps warn+0 too.
                    _ => !self.emitting_method_of_host_derived_class(),
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
    /// The lookup starts at `scopes[from]` and moves outward (ruling R7): the
    /// nearest scope that binds `name` decides. A parameter binding is not
    /// host; a declarator is followed through its initializer, resolved from
    /// the scope that declared it.
    fn root_has_host_provenance(
        &self,
        name: &str,
        scopes: &[LexicalScope],
        from: usize,
        seen: &mut HashSet<String>,
    ) -> bool {
        if !seen.insert(name.to_string()) {
            return false;
        }
        if self.is_free_global(name) {
            return true;
        }
        for (index, scope) in scopes.iter().enumerate().skip(from) {
            let is_param = match &scope.params {
                ScopeParams::Current => self.name_is_declared_parameter(name),
                ScopeParams::Names(params) => params.iter().any(|param| param == name),
            };
            if is_param {
                return false;
            }
            let Some((kind, init)) = self.declarator_in(scope.body, name) else {
                continue;
            };
            let Some(init) = init else {
                return false;
            };
            if kind != "const" && self.program_reassigned_names().contains(name) {
                return false;
            }
            return self.init_has_host_provenance(init, scopes, index, seen);
        }
        false
    }

    /// An initializer has host provenance when its member/call/`new` chain
    /// reaches a host root, or calls a host-derived program class. Names in it
    /// are resolved from `scopes[from]`, the scope that declared the binding.
    fn init_has_host_provenance(
        &self,
        init: LirNodeId,
        scopes: &[LexicalScope],
        from: usize,
        seen: &mut HashSet<String>,
    ) -> bool {
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
                    // A host-derived class only when `name` resolves to a
                    // class: no binding in scope (the class declaration), or a
                    // declarator bound to a class expression. `const C = mk`
                    // shadows a host-derived `class C`.
                    if self.repr_table.is_host_derived_class(name)
                        && self.name_resolves_to_a_class(name, scopes, from)
                    {
                        return true;
                    }
                    return self.root_has_host_provenance(name, scopes, from, seen);
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

    /// The ancestors of `self.body`, nearest first, ending at the module
    /// root (empty when `self.body` is the root). `None` when `self.body` is
    /// not reachable from the root.
    fn body_ancestors(&self) -> Option<Vec<LirNodeId>> {
        let root = self.program.root;
        if self.body == root {
            return Some(Vec::new());
        }
        let mut parent: HashMap<LirNodeId, LirNodeId> = HashMap::new();
        let mut stack = vec![root];
        let mut found = false;
        while let Some(id) = stack.pop() {
            if id == self.body {
                found = true;
                break;
            }
            for &child in &self.node(id).children {
                if child != root && !parent.contains_key(&child) {
                    parent.insert(child, id);
                    stack.push(child);
                }
            }
        }
        if !found {
            return None;
        }
        let mut ancestors = Vec::new();
        let mut current = self.body;
        while let Some(&up) = parent.get(&current) {
            ancestors.push(up);
            current = up;
        }
        Some(ancestors)
    }

    /// Ruling R8: the function being emitted is a method (or the
    /// constructor) of a class in `ReprTable::host_derived_classes`. A class
    /// lowers to a function-like node with no `function_flavor` whose body
    /// block holds its methods, each a function-like node with a flavor. A
    /// nameless `const K = class …` is named by its declarator, as
    /// `program_classes` records it.
    fn emitting_method_of_host_derived_class(&self) -> bool {
        let nodes = &self.program.nodes;
        let Some(ancestors) = self.body_ancestors() else {
            return false;
        };
        let [method, class_body, class, ..] = ancestors[..] else {
            return false;
        };
        let declarator_name = ancestors.get(3).and_then(|&up| {
            let declarator = self.node(up);
            (declarator.children.get(1) == Some(&class))
                .then_some(declarator.text.as_deref())
                .flatten()
        });
        let is_method = self.node(method).function_flavor.is_some()
            && crate::lower::function_body_and_params(nodes, method)
                .is_some_and(|(body, _)| body == self.body);
        let class_node = self.node(class);
        let is_class = class_node.function_flavor.is_none()
            && self.node(class_body).kind == LirNodeKind::Block
            && crate::lower::function_body_and_params(nodes, class)
                .is_some_and(|(body, _)| body == class_body);
        let class_name = class_node
            .text
            .as_deref()
            .filter(|name| !name.is_empty())
            .or(declarator_name);
        is_method
            && is_class
            && class_name.is_some_and(|name| self.repr_table.is_host_derived_class(name))
    }

    /// The lexical scopes visible from the function being emitted, nearest
    /// first: its own body, each enclosing function's body (outward), then the
    /// module body (ruling R7). Enclosing functions are the function-like LIR
    /// ancestors of `self.body`, found by a walk from the module root; their
    /// bodies and parameters come from `lower::function_body_and_params`, the
    /// shape `collect_functions` compiles them by. `None` when `self.body` is
    /// not reachable from the root, so the caller answers "not proven".
    fn lexical_scopes(&self) -> Option<Vec<LexicalScope>> {
        let nodes = &self.program.nodes;
        let mut scopes = vec![LexicalScope {
            body: self.body,
            params: ScopeParams::Current,
        }];
        for up in self.body_ancestors()? {
            if let Some((body, params)) = crate::lower::function_body_and_params(nodes, up) {
                // The current function's own node is scope 0 already.
                if body != self.body {
                    scopes.push(LexicalScope {
                        body,
                        params: ScopeParams::Names(params),
                    });
                }
            }
        }
        scopes.push(LexicalScope {
            body: self.program.root,
            params: ScopeParams::Names(Vec::new()),
        });
        Some(scopes)
    }

    /// Whether the nearest binding of `name` from `scopes[from]` outward is a
    /// class: no parameter or `const` / `let` / `var` declarator binds it (so
    /// it is the class declaration), or the declarator's initializer is a
    /// class expression (a function-like node with no `function_flavor`).
    fn name_resolves_to_a_class(&self, name: &str, scopes: &[LexicalScope], from: usize) -> bool {
        for scope in scopes.iter().skip(from) {
            let is_param = match &scope.params {
                ScopeParams::Current => self.name_is_declared_parameter(name),
                ScopeParams::Names(params) => params.iter().any(|param| param == name),
            };
            if is_param {
                return false;
            }
            if let Some((_, init)) = self.declarator_in(scope.body, name) {
                return init.is_some_and(|init| {
                    let init = self.unwrap_transparent(init);
                    self.node(init).function_flavor.is_none()
                        && crate::lower::is_function_like(&self.program.nodes, init)
                });
            }
        }
        true
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
