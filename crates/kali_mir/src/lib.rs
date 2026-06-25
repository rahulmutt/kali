//! Mid-level IR (MIR) for the Kali compiler.
//!
//! MIR is a conservative structural lowering of HIR that preserves the source
//! shape while providing a stable bridge for later memory/ownership analysis.

mod binding;
mod function;
mod layout;
mod lower;
mod node;
mod ownership;
mod program;

pub use binding::{BorrowedLifetime, MirBinding, MirBindingKind};
pub use function::{MirFunction, MirFunctionKind};
pub use layout::LayoutDescriptor;
pub use lower::MirLowerer;
pub use node::{MirBuilder, MirNode, MirNodeId, MirNodeKind, PlaceRef, PlaceValue};
pub use ownership::{OwnershipClass, ThreadBoundaryBinding, ThreadBoundaryDisposition, ThreadBoundaryProfile};
pub use program::MirProgram;

use std::collections::{BTreeMap, BTreeSet};

use kali_hir::{FunctionFlavor, HirNode, HirNodeId, HirNodeKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UseContext {
    Normal,
    Return,
    Escape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BindingState {
    pub(crate) name: String,
    pub(crate) kind: MirBindingKind,
    pub(crate) ownership: OwnershipClass,
    pub(crate) layout: LayoutDescriptor,
    pub(crate) returned: bool,
    pub(crate) escaped_via_flow: bool,
    pub(crate) captured_by: BTreeSet<String>,
}

impl BindingState {
    pub(crate) fn new(name: impl Into<String>, kind: MirBindingKind, layout: LayoutDescriptor) -> Self {
        Self {
            name: name.into(),
            kind,
            ownership: default_ownership(kind),
            layout,
            returned: false,
            escaped_via_flow: false,
            captured_by: BTreeSet::new(),
        }
    }
}

pub(crate) fn default_ownership(kind: MirBindingKind) -> OwnershipClass {
    match kind {
        MirBindingKind::Parameter => OwnershipClass::Borrowed,
        MirBindingKind::Function => OwnershipClass::Stack,
        MirBindingKind::Import => OwnershipClass::Borrowed,
        MirBindingKind::Local => OwnershipClass::Stack,
    }
}

pub(crate) fn parameter_escape_flags(function: &MirFunction) -> Vec<bool> {
    function
        .bindings
        .iter()
        .filter(|binding| binding.kind == MirBindingKind::Parameter)
        .map(|binding| binding.escapes)
        .collect()
}

pub(crate) fn function_binding_escapes(
    bindings: &[BindingState],
    name: &str,
    cache: &mut BTreeMap<String, bool>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    if let Some(&cached) = cache.get(name) {
        return cached;
    }

    if !visiting.insert(name.to_string()) {
        return true;
    }

    let escapes = bindings
        .iter()
        .find(|binding| binding.kind == MirBindingKind::Function && binding.name == name)
        .map(|binding| {
            binding.returned
                || binding.escaped_via_flow
                || binding
                    .captured_by
                    .iter()
                    .any(|capturer| function_binding_escapes(bindings, capturer, cache, visiting))
        })
        .unwrap_or(false);

    visiting.remove(name);
    cache.insert(name.to_string(), escapes);
    escapes
}

pub(crate) fn finalise_binding(
    binding: BindingState,
    scope_bindings: &[BindingState],
    capture_escape_cache: &mut BTreeMap<String, bool>,
    visiting: &mut BTreeSet<String>,
) -> MirBinding {
    let capture_escapes = binding.captured_by.iter().any(|capturing| {
        function_binding_escapes(scope_bindings, capturing, capture_escape_cache, visiting)
    });

    let escapes = binding.returned || binding.escaped_via_flow || capture_escapes;
    let ownership = if capture_escapes {
        match binding.kind {
            MirBindingKind::Import => binding.ownership,
            MirBindingKind::Parameter | MirBindingKind::Local | MirBindingKind::Function => {
                OwnershipClass::SharedHeap
            }
        }
    } else if binding.returned || binding.escaped_via_flow {
        match binding.kind {
            MirBindingKind::Local | MirBindingKind::Function => OwnershipClass::OwnedHeap,
            MirBindingKind::Parameter | MirBindingKind::Import => binding.ownership,
        }
    } else if !binding.captured_by.is_empty() {
        match binding.kind {
            MirBindingKind::Import | MirBindingKind::Parameter => binding.ownership,
            MirBindingKind::Local | MirBindingKind::Function => OwnershipClass::Borrowed,
        }
    } else {
        binding.ownership
    };

    MirBinding {
        name: binding.name,
        kind: binding.kind,
        ownership,
        layout: binding.layout,
        escapes,
        captured_by: binding.captured_by.into_iter().collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScopeState {
    pub(crate) label: String,
    pub(crate) kind: MirFunctionKind,
    pub(crate) function_flavor: Option<FunctionFlavor>,
    pub(crate) bindings: Vec<BindingState>,
    pub(crate) binding_index: BTreeMap<String, usize>,
    pub(crate) function_aliases: BTreeMap<String, String>,
    pub(crate) captured_bindings: BTreeSet<String>,
}

impl ScopeState {
    pub(crate) fn new(
        label: impl Into<String>,
        kind: MirFunctionKind,
        function_flavor: Option<FunctionFlavor>,
    ) -> Self {
        Self {
            label: label.into(),
            kind,
            function_flavor,
            bindings: Vec::new(),
            binding_index: BTreeMap::new(),
            function_aliases: BTreeMap::new(),
            captured_bindings: BTreeSet::new(),
        }
    }

    pub(crate) fn define(&mut self, name: impl Into<String>, kind: MirBindingKind, layout: LayoutDescriptor) {
        let name = name.into();
        let binding = BindingState::new(name.clone(), kind, layout);
        if let Some(index) = self.binding_index.get(&name).copied() {
            self.bindings[index] = binding;
        } else {
            let index = self.bindings.len();
            self.bindings.push(binding);
            self.binding_index.insert(name, index);
        }
    }

    pub(crate) fn get_binding_index(&self, name: &str) -> Option<usize> {
        self.binding_index.get(name).copied()
    }

    pub(crate) fn get_binding_mut(&mut self, name: &str) -> Option<&mut BindingState> {
        let index = self.get_binding_index(name)?;
        self.bindings.get_mut(index)
    }

    pub(crate) fn alias_function(
        &mut self,
        binding_name: impl Into<String>,
        function_name: impl Into<String>,
    ) {
        self.function_aliases
            .insert(binding_name.into(), function_name.into());
    }

    pub(crate) fn capture_binding(&mut self, name: impl Into<String>) {
        self.captured_bindings.insert(name.into());
    }

    pub(crate) fn finalize(self) -> MirFunction {
        let ScopeState {
            label,
            kind,
            function_flavor,
            mut bindings,
            binding_index: _,
            function_aliases: _,
            captured_bindings,
        } = self;
        let captured_bindings = captured_bindings.into_iter().collect::<Vec<_>>();
        if !captured_bindings.is_empty() {
            if let Some(binding) = bindings
                .iter_mut()
                .find(|binding| binding.kind == MirBindingKind::Function && binding.name == label)
            {
                binding.layout = LayoutDescriptor::Closure {
                    captures: captured_bindings,
                };
            }
        }
        let scope_bindings = bindings.clone();
        let mut capture_escape_cache = BTreeMap::new();
        let mut visiting = BTreeSet::new();
        MirFunction {
            name: if kind == MirFunctionKind::Module {
                None
            } else {
                Some(label)
            },
            kind,
            function_flavor,
            bindings: bindings
                .into_iter()
                .map(|binding| {
                    finalise_binding(
                        binding,
                        &scope_bindings,
                        &mut capture_escape_cache,
                        &mut visiting,
                    )
                })
                .collect(),
        }
    }
}

pub(crate) struct OwnershipAnalyzer<'a> {
    pub(crate) nodes: &'a [HirNode],
    pub(crate) function_flavors: &'a [(HirNodeId, FunctionFlavor)],
    pub(crate) functions: Vec<MirFunction>,
    pub(crate) scope_stack: Vec<ScopeState>,
    pub(crate) synthetic_function_counter: usize,
}

impl<'a> OwnershipAnalyzer<'a> {
    pub(crate) fn new(nodes: &'a [HirNode], function_flavors: &'a [(HirNodeId, FunctionFlavor)]) -> Self {
        Self {
            nodes,
            function_flavors,
            functions: Vec::new(),
            scope_stack: Vec::new(),
            synthetic_function_counter: 0,
        }
    }

    pub(crate) fn analyze_program(mut self, root: HirNodeId) -> Vec<MirFunction> {
        self.push_scope("<module>", MirFunctionKind::Module, None);
        self.precollect_scope_bindings(root);
        self.walk_scope_node(root, UseContext::Normal);
        self.pop_scope_and_record();
        self.functions
    }

    pub(crate) fn push_scope(
        &mut self,
        label: impl Into<String>,
        kind: MirFunctionKind,
        function_flavor: Option<FunctionFlavor>,
    ) {
        self.scope_stack
            .push(ScopeState::new(label, kind, function_flavor));
    }

    pub(crate) fn pop_scope_and_record(&mut self) {
        if let Some(scope) = self.scope_stack.pop() {
            self.functions.push(scope.finalize());
        }
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

    pub(crate) fn function_flavor(&self, node_id: HirNodeId) -> Option<FunctionFlavor> {
        self.function_flavors
            .iter()
            .find(|(id, _)| *id == node_id)
            .map(|(_, flavor)| *flavor)
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

    pub(crate) fn define_binding(&mut self, name: String, kind: MirBindingKind, layout: LayoutDescriptor) {
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
                    if let Some(init) = children.get(1).copied() {
                        let init_node = &self.nodes[init.0 as usize];
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
                for child in children.iter().take(params_end) {
                    if let Some(param_name) = self.nodes[child.0 as usize].text.as_ref() {
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
                for child in children.iter().take(params_end) {
                    if let Some(param_name) = self.nodes[child.0 as usize].text.as_ref() {
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
                for child in children {
                    self.walk_scope_node(child, UseContext::Return);
                }
            }
            HirNodeKind::CallExpr => {
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
                for child in children {
                    self.walk_scope_node(child, UseContext::Escape);
                }
            }
            HirNodeKind::ObjectExpr => {
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

    pub(crate) fn infer_layout(&self, node_id: HirNodeId) -> LayoutDescriptor {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::Literal => match node.text.as_deref() {
                Some("true") | Some("false") => LayoutDescriptor::scalar("bool"),
                Some("null") | Some("undefined") => LayoutDescriptor::scalar("unknown"),
                Some(text) if text.parse::<i64>().is_ok() || text.parse::<f64>().is_ok() => {
                    LayoutDescriptor::scalar("number")
                }
                Some(_) => LayoutDescriptor::scalar("string"),
                None => LayoutDescriptor::TaggedVal,
            },
            HirNodeKind::ArrayExpr => {
                let element = node
                    .children
                    .first()
                    .copied()
                    .map(|child| Box::new(self.infer_layout(child)))
                    .unwrap_or_else(|| Box::new(LayoutDescriptor::TaggedVal));
                LayoutDescriptor::Array {
                    element,
                    length: Some(node.children.len()),
                }
            }
            HirNodeKind::ObjectExpr => {
                let mut fields = Vec::new();
                for (source_index, child) in node.children.iter().copied().enumerate() {
                    let property = &self.nodes[child.0 as usize];
                    if matches!(property.kind, HirNodeKind::ObjectProperty)
                        && property.children.len() >= 2
                    {
                        let key = self.layout_field_name(property);
                        let value = property.children[1];
                        fields.push((key, source_index, Box::new(self.infer_layout(value))));
                    }
                }
                if fields.is_empty() {
                    LayoutDescriptor::TaggedVal
                } else {
                    fields.sort_by(
                        |(left_key, left_index, _), (right_key, right_index, _)| match (
                            Self::object_property_order_key(left_key),
                            Self::object_property_order_key(right_key),
                        ) {
                            (Some(left_order), Some(right_order)) => left_order
                                .cmp(&right_order)
                                .then_with(|| left_index.cmp(right_index)),
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            (None, None) => left_index.cmp(right_index),
                        },
                    );
                    LayoutDescriptor::Struct {
                        fields: fields
                            .into_iter()
                            .map(|(key, _, layout)| (key, layout))
                            .collect(),
                    }
                }
            }
            HirNodeKind::FunctionExpr | HirNodeKind::FunctionDecl => LayoutDescriptor::Closure {
                captures: Vec::new(),
            },
            HirNodeKind::Ident => self
                .resolve_binding_layout(node.text.as_deref().unwrap_or_default())
                .unwrap_or(LayoutDescriptor::TaggedVal),
            HirNodeKind::CallExpr | HirNodeKind::NewExpr | HirNodeKind::ImportExpr => {
                LayoutDescriptor::TaggedVal
            }
            HirNodeKind::BinaryExpr => self.infer_binary_layout(node),
            HirNodeKind::UnaryExpr => self.infer_unary_layout(node),
            HirNodeKind::ConditionalExpr | HirNodeKind::SequenceExpr => node
                .children
                .last()
                .copied()
                .map(|child| self.infer_layout(child))
                .unwrap_or(LayoutDescriptor::TaggedVal),
            _ => LayoutDescriptor::TaggedVal,
        }
    }

    pub(crate) fn infer_binary_layout(&self, node: &HirNode) -> LayoutDescriptor {
        let op = node.text.as_deref().unwrap_or_default();
        match op {
            "+" | "-" | "*" | "/" | "%" | "**" => LayoutDescriptor::scalar("number"),
            "==" | "!=" | "===" | "!==" | "<" | "<=" | ">" | ">=" | "&&" | "||" => {
                LayoutDescriptor::scalar("bool")
            }
            _ => LayoutDescriptor::TaggedVal,
        }
    }

    pub(crate) fn infer_unary_layout(&self, node: &HirNode) -> LayoutDescriptor {
        match node.text.as_deref().unwrap_or_default() {
            "!" => LayoutDescriptor::scalar("bool"),
            "-" | "+" | "~" => LayoutDescriptor::scalar("number"),
            _ => LayoutDescriptor::TaggedVal,
        }
    }

    pub(crate) fn resolve_binding_layout(&self, name: &str) -> Option<LayoutDescriptor> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(index) = scope.get_binding_index(name) {
                return scope
                    .bindings
                    .get(index)
                    .map(|binding| binding.layout.clone());
            }
        }
        None
    }

    pub(crate) fn function_parameter_escape_flags(&self, name: &str) -> Option<Vec<bool>> {
        self.functions
            .iter()
            .rev()
            .find(|function| function.name.as_deref() == Some(name))
            .map(parameter_escape_flags)
    }

    pub(crate) fn resolve_function_target(&self, name: &str) -> Option<String> {
        let mut current = name.to_string();
        let mut seen = BTreeSet::new();

        loop {
            if !seen.insert(current.clone()) {
                return None;
            }

            if self
                .functions
                .iter()
                .any(|function| function.name.as_deref() == Some(current.as_str()))
            {
                return Some(current);
            }

            let (scope_index, binding_index) = self.resolve_binding(&current)?;
            let scope = self.scope_stack.get(scope_index)?;
            let binding = scope.bindings.get(binding_index)?;
            if let Some(next) = scope.function_aliases.get(&current) {
                current = next.clone();
                continue;
            }

            if binding.kind == MirBindingKind::Function {
                return Some(current);
            }

            return None;
        }
    }

    pub(crate) fn function_target_from_node(&self, node_id: HirNodeId) -> Option<String> {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::Ident => node
                .text
                .as_deref()
                .and_then(|name| self.resolve_function_target(name)),
            _ => None,
        }
    }

    pub(crate) fn function_name_from_recent_functions(&self, functions_before: usize) -> Option<String> {
        self.functions
            .get(functions_before..)
            .and_then(|functions| functions.last())
            .and_then(|function| function.name.clone())
    }

    pub(crate) fn layout_field_name(&self, node: &HirNode) -> String {
        if let Some(key) = node.children.first() {
            let key_node = &self.nodes[key.0 as usize];
            if let Some(text) = key_node.text.as_ref() {
                return text.clone();
            }
        }

        node.text
            .clone()
            .unwrap_or_else(|| format!("field_{}", node.children.len()))
    }

    pub(crate) fn object_property_order_key(key: &str) -> Option<u64> {
        let normalized = key.trim_matches('"');
        if normalized.is_empty() || (normalized.len() > 1 && normalized.starts_with('0')) {
            return None;
        }

        let value = normalized.parse::<u64>().ok()?;
        (value < u32::MAX as u64).then_some(value)
    }

    pub(crate) fn next_function_name(&mut self) -> String {
        let name = format!("__kali_fn_{}", self.synthetic_function_counter);
        self.synthetic_function_counter += 1;
        name
    }

    pub(crate) fn is_heap_store_target(&self, node_id: HirNodeId) -> bool {
        matches!(
            self.nodes[node_id.0 as usize].kind,
            HirNodeKind::MemberExpr | HirNodeKind::OptionalChain | HirNodeKind::ChainExpr
        )
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
