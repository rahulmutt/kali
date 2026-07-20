//! Scope-stack management for the ownership analyzer.

use kali_hir::{FunctionFlavor, HirNodeId, HirNodeKind};

use crate::{LayoutDescriptor, MirBindingKind, MirFunctionKind, OwnershipAnalyzer, ScopeState};

impl<'a> OwnershipAnalyzer<'a> {
    pub(crate) fn push_scope(
        &mut self,
        label: impl Into<String>,
        kind: MirFunctionKind,
        function_flavor: Option<FunctionFlavor>,
    ) {
        let label = label.into();
        // Record the function-nesting parent chain in the analysis's own label
        // key space, at the exact moment the scope stack knows the enclosing
        // function. Only real function scopes are recorded (the module root is
        // the reserved `""` plan key and never owns an env); the parent is the
        // enclosing function's label, or `None` when the enclosing scope is the
        // module root — matching `env_plan::scope_hops`'s root sentinel. This is
        // the single source of nesting for `derive_env_plans`: anonymous
        // functions carry their `__kali_fn_N` label like any other, and a
        // non-scope `Function` NODE (e.g. a class) never reaches here, so it
        // cannot inject a phantom hop.
        if kind != MirFunctionKind::Module {
            let parent = self.scope_stack.last().and_then(|scope| {
                (scope.kind != MirFunctionKind::Module).then(|| scope.label.clone())
            });
            self.parent_labels.insert(label.clone(), parent);
        }
        self.scope_stack
            .push(ScopeState::new(label, kind, function_flavor));
        self.arena_enter_function();
    }

    pub(crate) fn pop_scope_and_record(&mut self) {
        self.arena_finalize_current_function();
        if let Some(scope) = self.scope_stack.pop() {
            self.functions.push(scope.finalize());
        }
        self.arena_exit_function();
    }

    pub(crate) fn current_scope_label(&self) -> String {
        self.scope_stack
            .last()
            .map(|scope| scope.label.clone())
            .unwrap_or_else(|| "<module>".to_string())
    }

    pub(crate) fn current_scope_index(&self) -> usize {
        self.scope_stack.len().saturating_sub(1)
    }

    pub(crate) fn current_scope_mut(&mut self) -> Option<&mut ScopeState> {
        self.scope_stack.last_mut()
    }

    pub(crate) fn precollect_scope_bindings(&mut self, node_id: HirNodeId) {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::Program | HirNodeKind::Block => {
                for child in &node.children {
                    self.precollect_scope_bindings(*child);
                }
            }
            HirNodeKind::VarDecl => {
                for child in &node.children {
                    self.precollect_scope_bindings(*child);
                }
            }
            HirNodeKind::VarDeclarator => {
                if let Some(name) = node.text.as_ref() {
                    self.define_binding(
                        name.clone(),
                        MirBindingKind::Local,
                        LayoutDescriptor::TaggedVal,
                    );
                }
            }
            HirNodeKind::ImportDecl => {
                for child in &node.children {
                    self.collect_import_bindings(*child);
                }
            }
            HirNodeKind::FunctionDecl => {
                if let Some(name) = node.text.as_ref() {
                    self.define_binding(
                        name.clone(),
                        MirBindingKind::Function,
                        LayoutDescriptor::Closure {
                            captures: Vec::new(),
                        },
                    );
                }
            }
            HirNodeKind::FunctionExpr => {
                if let Some(name) = node.text.as_ref() {
                    self.define_binding(
                        name.clone(),
                        MirBindingKind::Function,
                        LayoutDescriptor::Closure {
                            captures: Vec::new(),
                        },
                    );
                }
            }
            HirNodeKind::ClassDecl => {
                if let Some(name) = node.text.as_ref() {
                    self.define_binding(
                        name.clone(),
                        MirBindingKind::Local,
                        LayoutDescriptor::TaggedVal,
                    );
                }
            }
            _ => {
                for child in &node.children {
                    self.precollect_scope_bindings(*child);
                }
            }
        }
    }

    pub(crate) fn define_binding(
        &mut self,
        name: String,
        kind: MirBindingKind,
        layout: LayoutDescriptor,
    ) {
        if let Some(scope) = self.current_scope_mut() {
            scope.define(name, kind, layout);
        }
    }

    pub(crate) fn collect_import_bindings(&mut self, node_id: HirNodeId) {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::Ident => {
                if let Some(name) = node.text.as_ref() {
                    self.define_binding(
                        name.clone(),
                        MirBindingKind::Import,
                        LayoutDescriptor::TaggedVal,
                    );
                }
            }
            HirNodeKind::ImportDecl => {
                for child in &node.children {
                    self.collect_import_bindings(*child);
                }
            }
            _ => {}
        }
    }
}
