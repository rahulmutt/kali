//! Lexical scope model and resolver scope handle.
use super::*;
use indexmap::IndexMap;
use kali_ast::NodeId;

/// Scope types recognized by the stage-1 resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
    Global,
    Module,
    Block,
    Function,
    Class,
    TypeAlias,
    Interface,
    Catch,
}

/// A lexical scope.
#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub scope_type: ScopeType,
    pub parent: Option<NodeId>,
    pub bindings: IndexMap<String, NodeId>,
    pub mutable_bindings: IndexMap<String, bool>,
    pub static_values: IndexMap<String, String>,
    pub static_numeric_values: IndexMap<String, String>,
    pub(crate) static_identity_values: IndexMap<String, StaticObjectIdentityValue>,
    pub static_arrays: IndexMap<String, bool>,
    pub static_objects: IndexMap<String, bool>,
    pub static_reference_values: IndexMap<String, String>,
    pub static_object_keys: IndexMap<String, bool>,
}

impl Scope {
    pub fn new(scope_type: ScopeType, parent: Option<NodeId>) -> Self {
        Self {
            scope_type,
            parent,
            bindings: IndexMap::new(),
            mutable_bindings: IndexMap::new(),
            static_values: IndexMap::new(),
            static_numeric_values: IndexMap::new(),
            static_identity_values: IndexMap::new(),
            static_arrays: IndexMap::new(),
            static_objects: IndexMap::new(),
            static_reference_values: IndexMap::new(),
            static_object_keys: IndexMap::new(),
        }
    }

    pub fn bind(&mut self, name: impl Into<String>, node_id: NodeId) {
        let name = name.into();
        self.bindings.insert(name.clone(), node_id);
        self.mutable_bindings.insert(name, false);
    }

    pub fn lookup(&self, name: &str) -> Option<&NodeId> {
        self.bindings.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    pub(crate) fn invalidate_static_binding(&mut self, name: &str) {
        self.static_values.shift_remove(name);
        self.static_numeric_values.shift_remove(name);
        self.static_identity_values.shift_remove(name);
        self.static_arrays.shift_remove(name);
        self.static_objects.shift_remove(name);
        self.static_reference_values.shift_remove(name);
        self.static_object_keys.shift_remove(name);
    }
}

/// Reference to a scope binding.
pub struct ScopeRef<'a> {
    pub(crate) scope: &'a Scope,
    pub(crate) name: String,
    pub(crate) binding_id: NodeId,
}

impl<'a> ScopeRef<'a> {
    pub fn binding_id(&self) -> NodeId {
        self.binding_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn scope(&self) -> &Scope {
        self.scope
    }
}
