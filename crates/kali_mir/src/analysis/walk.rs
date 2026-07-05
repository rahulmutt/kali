//! HIR scope-walking and use/binding resolution for the analyzer.

use kali_hir::{HirNodeId, HirNodeKind};

use crate::{
    parameter_escape_flags, LayoutDescriptor, MirBindingKind, MirFunctionKind, OwnershipAnalyzer,
    UseContext,
};

impl<'a> OwnershipAnalyzer<'a> {
    pub(crate) fn walk_scope_node(&mut self, node_id: HirNodeId, context: UseContext) {
        let node = &self.nodes[node_id.0 as usize];
        let kind = node.kind.clone();
        let text = node.text.clone();
        let children = node.children.clone();

        match kind {
            HirNodeKind::Program | HirNodeKind::Block => {
                for child in children {
                    self.walk_scope_node(child, context);
                }
            }
            HirNodeKind::VarDecl => {
                for child in children {
                    self.walk_scope_node(child, context);
                }
            }
            HirNodeKind::VarDeclarator => {
                if let Some(name) = text.as_ref() {
                    self.arena_note_declared_binding(name);
                    if let Some(init) = children.get(1).copied() {
                        let init_node = &self.nodes[init.0 as usize];
                        if matches!(
                            init_node.kind,
                            HirNodeKind::ObjectExpr | HirNodeKind::ArrayExpr
                        ) {
                            self.arena_note_fresh_binding(name);
                        }
                        let class = self.classify_value(init);
                        let owner = self.current_scope_label();
                        self.flow.note_value_into(
                            crate::analysis::escape_flow::FlowNode::Binding {
                                owner,
                                name: name.clone(),
                            },
                            &class,
                        );
                        let layout = self.infer_layout(init);
                        let functions_before = self.functions.len();
                        let direct_function_target = matches!(init_node.kind, HirNodeKind::Ident)
                            .then(|| self.function_target_from_node(init))
                            .flatten();
                        if let Some(scope) = self.current_scope_mut() {
                            if let Some(binding) = scope.get_binding_mut(name) {
                                binding.layout = layout;
                                if matches!(init_node.kind, HirNodeKind::FunctionExpr)
                                    || direct_function_target.is_some()
                                {
                                    binding.layout = LayoutDescriptor::Closure {
                                        captures: Vec::new(),
                                    };
                                }
                                if matches!(init_node.kind, HirNodeKind::FunctionExpr) {
                                    binding.kind = MirBindingKind::Function;
                                }
                            }
                        }
                        self.walk_scope_node(init, UseContext::Normal);
                        let function_name = if matches!(init_node.kind, HirNodeKind::FunctionExpr) {
                            self.function_name_from_recent_functions(functions_before)
                        } else {
                            direct_function_target
                        };
                        if let Some(function_name) = function_name {
                            if let Some(scope) = self.current_scope_mut() {
                                scope.alias_function(name.clone(), function_name);
                            }
                        }
                    }
                }
            }
            HirNodeKind::FunctionDecl => {
                let function_name = text.unwrap_or_else(|| self.next_function_name());
                let body = children.last().copied();
                let params_end = body.map_or(children.len(), |_| children.len().saturating_sub(1));
                self.push_scope(
                    function_name.clone(),
                    MirFunctionKind::Function,
                    self.function_flavor(node_id),
                );
                if let Some(scope) = self.current_scope_mut() {
                    scope.define(
                        function_name.clone(),
                        MirBindingKind::Function,
                        LayoutDescriptor::Closure {
                            captures: Vec::new(),
                        },
                    );
                }
                for (param_index, child) in children.iter().take(params_end).enumerate() {
                    if let Some(param_name) = self.nodes[child.0 as usize].text.as_ref() {
                        self.flow
                            .note_param(&function_name, param_index, param_name);
                        if let Some(scope) = self.current_scope_mut() {
                            scope.define(
                                param_name.clone(),
                                MirBindingKind::Parameter,
                                LayoutDescriptor::TaggedVal,
                            );
                        }
                    }
                }
                if let Some(body) = body {
                    self.precollect_scope_bindings(body);
                    self.walk_scope_node(body, UseContext::Normal);
                }
                self.pop_scope_and_record();
            }
            HirNodeKind::FunctionExpr => {
                let function_name = text.unwrap_or_else(|| self.next_function_name());
                let body = children.last().copied();
                let params_end = body.map_or(children.len(), |_| children.len().saturating_sub(1));
                self.push_scope(
                    function_name.clone(),
                    MirFunctionKind::Closure,
                    self.function_flavor(node_id),
                );
                if let Some(scope) = self.current_scope_mut() {
                    scope.define(
                        function_name.clone(),
                        MirBindingKind::Function,
                        LayoutDescriptor::Closure {
                            captures: Vec::new(),
                        },
                    );
                }
                for (param_index, child) in children.iter().take(params_end).enumerate() {
                    if let Some(param_name) = self.nodes[child.0 as usize].text.as_ref() {
                        self.flow
                            .note_param(&function_name, param_index, param_name);
                        if let Some(scope) = self.current_scope_mut() {
                            scope.define(
                                param_name.clone(),
                                MirBindingKind::Parameter,
                                LayoutDescriptor::TaggedVal,
                            );
                        }
                    }
                }
                if let Some(body) = body {
                    self.precollect_scope_bindings(body);
                    self.walk_scope_node(body, UseContext::Normal);
                }
                self.pop_scope_and_record();
            }
            HirNodeKind::ImportDecl => {
                for child in children {
                    self.walk_scope_node(child, context);
                }
            }
            HirNodeKind::ReturnStmt => {
                self.arena_note_return(&children);
                for child in children {
                    self.walk_scope_node(child, UseContext::Return);
                }
            }
            HirNodeKind::CallExpr => {
                self.arena_note_call_expr(&children);
                let mut direct_call_escape_flags = None;
                if let Some(callee) = children.first().copied() {
                    let callee_node = &self.nodes[callee.0 as usize];
                    match callee_node.kind {
                        HirNodeKind::FunctionExpr => {
                            let functions_before = self.functions.len();
                            self.walk_scope_node(callee, UseContext::Normal);
                            direct_call_escape_flags = self
                                .functions
                                .get(functions_before..)
                                .and_then(|functions| functions.last())
                                .map(parameter_escape_flags);
                        }
                        HirNodeKind::Ident => {
                            self.walk_scope_node(callee, UseContext::Normal);
                            if let Some(name) = callee_node.text.as_deref() {
                                if let Some(function_name) = self.resolve_function_target(name) {
                                    direct_call_escape_flags =
                                        self.function_parameter_escape_flags(&function_name);
                                }
                            }
                        }
                        _ => {
                            self.walk_scope_node(callee, UseContext::Normal);
                        }
                    }
                }

                for (index, child) in children.into_iter().enumerate().skip(1) {
                    let should_escape = direct_call_escape_flags
                        .as_ref()
                        .and_then(|flags| flags.get(index - 1).copied())
                        .unwrap_or(true);
                    self.walk_scope_node(
                        child,
                        if should_escape {
                            UseContext::Escape
                        } else {
                            UseContext::Normal
                        },
                    );
                }
            }
            HirNodeKind::NewExpr => {
                for (index, child) in children.into_iter().enumerate() {
                    let child_context = if index == 0 {
                        UseContext::Normal
                    } else {
                        UseContext::Escape
                    };
                    self.walk_scope_node(child, child_context);
                }
            }
            HirNodeKind::ArrayExpr => {
                self.arena_note_alloc();
                for child in children {
                    self.walk_scope_node(child, UseContext::Escape);
                }
            }
            HirNodeKind::ObjectExpr => {
                self.arena_note_alloc();
                for child in children {
                    self.walk_scope_node(child, UseContext::Escape);
                }
            }
            HirNodeKind::ObjectProperty => {
                if let Some(key) = children.first().copied() {
                    self.walk_scope_node(key, UseContext::Normal);
                }
                if let Some(value) = children.get(1).copied() {
                    self.walk_scope_node(value, UseContext::Escape);
                }
            }
            HirNodeKind::Unknown if text.as_deref() == Some("unknown") && !children.is_empty() => {
                for child in children {
                    self.walk_scope_node(child, UseContext::Escape);
                }
            }
            HirNodeKind::AssignmentExpr => {
                let left = children.first().copied();
                let right = children.get(1).copied();
                if let (Some(left), Some(right)) = (left, right) {
                    self.arena_note_assignment(left, right);
                }
                if let Some(left) = left {
                    self.walk_scope_node(left, UseContext::Normal);
                }
                if let Some(right) = right {
                    let rhs_context = left.is_some_and(|left| self.is_heap_store_target(left));
                    self.walk_scope_node(
                        right,
                        if rhs_context {
                            UseContext::Escape
                        } else {
                            context
                        },
                    );
                }
            }
            HirNodeKind::SequenceExpr => {
                for child in children.iter().take(children.len().saturating_sub(1)) {
                    self.walk_scope_node(*child, UseContext::Normal);
                }
                if let Some(last) = children.last().copied() {
                    self.walk_scope_node(last, context);
                }
            }
            HirNodeKind::Ident => {
                if let Some(name) = text.as_ref() {
                    self.resolve_use(name, context);
                }
            }
            HirNodeKind::ForStmt
            | HirNodeKind::ForInStmt
            | HirNodeKind::ForOfStmt
            | HirNodeKind::WhileStmt
            | HirNodeKind::DoWhileStmt => {
                // Pre-order loop ordinal (matches the order the LIR emitter
                // walks loops). Child walking is identical to the default arm,
                // so ownership verdicts are unchanged.
                self.arena_enter_loop();
                for child in children {
                    self.walk_scope_node(child, context);
                }
                self.arena_exit_loop();
            }
            _ => {
                for child in children {
                    self.walk_scope_node(child, context);
                }
            }
        }
    }

    pub(crate) fn resolve_use(&mut self, name: &str, context: UseContext) {
        let Some((scope_index, binding_index)) = self.resolve_binding(name) else {
            return;
        };

        let current_index = self.current_scope_index();
        let current_label = self.current_scope_label();
        if scope_index < current_index {
            let captured_by = current_label.clone();
            if let Some(binding) = self
                .scope_stack
                .get_mut(scope_index)
                .and_then(|scope| scope.bindings.get_mut(binding_index))
            {
                binding.captured_by.insert(captured_by);
            }
            if let Some(scope) = self.scope_stack.get_mut(current_index) {
                scope.capture_binding(name.to_string());
            }
        }

        if scope_index == current_index {
            if matches!(context, UseContext::Return) {
                if let Some(binding) = self
                    .scope_stack
                    .get_mut(scope_index)
                    .and_then(|scope| scope.bindings.get_mut(binding_index))
                {
                    binding.returned = true;
                }
            }

            if matches!(context, UseContext::Escape) {
                if let Some(binding) = self
                    .scope_stack
                    .get_mut(scope_index)
                    .and_then(|scope| scope.bindings.get_mut(binding_index))
                {
                    binding.escaped_via_flow = true;
                }
            }
        }
    }

    pub(crate) fn resolve_binding(&self, name: &str) -> Option<(usize, usize)> {
        for (scope_index, scope) in self.scope_stack.iter().enumerate().rev() {
            if let Some(binding_index) = scope.get_binding_index(name) {
                return Some((scope_index, binding_index));
            }
        }
        None
    }

    pub(crate) fn is_heap_store_target(&self, node_id: HirNodeId) -> bool {
        matches!(
            self.nodes[node_id.0 as usize].kind,
            HirNodeKind::MemberExpr | HirNodeKind::OptionalChain | HirNodeKind::ChainExpr
        )
    }
}
