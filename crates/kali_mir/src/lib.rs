//! Mid-level IR (MIR) for the Kali compiler.
//!
//! MIR is a conservative structural lowering of HIR that preserves the source
//! shape while providing a stable bridge for later memory/ownership analysis.

mod analysis;
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

    pub(crate) fn function_flavor(&self, node_id: HirNodeId) -> Option<FunctionFlavor> {
        self.function_flavors
            .iter()
            .find(|(id, _)| *id == node_id)
            .map(|(_, flavor)| *flavor)
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

}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
