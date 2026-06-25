//! Ownership/escape analysis engine (split by concern).

use std::collections::{BTreeMap, BTreeSet};

use kali_hir::{FunctionFlavor, HirNode, HirNodeId};

use crate::{LayoutDescriptor, MirBinding, MirBindingKind, MirFunction, MirFunctionKind, OwnershipClass};

mod infer;
mod resolve;
mod scope;
mod walk;

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

}
