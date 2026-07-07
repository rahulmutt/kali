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
    /// Names known to hold a *string* value (declared via a string/template
    /// literal or a `+` expression that is string-typed). Unlike `static_values`
    /// — which is polluted by number/boolean/null renderings and by numeric `+`
    /// results — this flag is a clean "this binding is a string at runtime"
    /// signal used to reject string-typed-variable `+` operands that codegen
    /// cannot lower.
    pub static_string_typed: IndexMap<String, bool>,
    pub static_numeric_values: IndexMap<String, String>,
    pub(crate) static_identity_values: IndexMap<String, StaticObjectIdentityValue>,
    pub static_arrays: IndexMap<String, bool>,
    /// Names bound to an array *literal* (`x = ["a", "b"]`) — tracked for
    /// EVERY declaration kind, including `var` (unlike `static_arrays`, which
    /// is non-`var` only). Codegen never linearizes a string/number literal
    /// array into a runtime linear-memory binding (only `new Array(n)`,
    /// `.fill`, array params, and array reassignment register in codegen's
    /// `array_bindings`), so the runtime `join` lane must REJECT a
    /// literal-array receiver (it would silently emit `0`). Fail-closed.
    pub array_literal_bindings: IndexMap<String, bool>,
    /// Names structurally proven to be a codegen RUNTIME linear-memory array
    /// binding — the types-side mirror of `kali_codegen`'s `array_bindings`
    /// set (emitter.rs). Populated at exactly the sites codegen registers:
    /// an array-typed PARAMETER (repr-table `is_array_binding`, at function
    /// entry), a `new Array(n)` / `Array(n)` / `.fill(...)` DECLARATOR init,
    /// and a `new Array(n)` / `Array(n)` / structural-identifier REASSIGNMENT
    /// target. Grow-only within a scope, mirroring codegen's insert-only
    /// `HashSet` (a reassignment never REMOVES a binding from codegen's set).
    ///
    /// The runtime string store / `.length` / `join` lanes trust this
    /// STRUCTURAL registry, not the repr-table proof alone: repr_infer
    /// over-proves `is_array_binding` for shapes codegen never registers (a
    /// call-result capture `const c = mk()`, a module-scope array read inside
    /// a nested function), so a gate keying on the repr proof would ACCEPT a
    /// receiver codegen falls through on and silently emits `0`. Requiring
    /// structural registration keeps types in lockstep with what codegen can
    /// lower. Fail-closed: a repr-proven-but-non-structural binding rejects.
    pub runtime_array_bindings: IndexMap<String, bool>,
    pub static_objects: IndexMap<String, bool>,
    pub static_reference_values: IndexMap<String, String>,
    pub static_object_keys: IndexMap<String, bool>,
    /// Bindings that hold a `for..in` key over a known-shape object: name ->
    /// the enumerated object's ShapeId. Grow-only, per the runtime_array_bindings
    /// convention. Seeded at the for..in left-hand var and at `last = c` aliases.
    pub for_in_key_bindings: IndexMap<String, kali_common::ShapeId>,
}

impl Scope {
    pub fn new(scope_type: ScopeType, parent: Option<NodeId>) -> Self {
        Self {
            scope_type,
            parent,
            bindings: IndexMap::new(),
            mutable_bindings: IndexMap::new(),
            static_values: IndexMap::new(),
            static_string_typed: IndexMap::new(),
            static_numeric_values: IndexMap::new(),
            static_identity_values: IndexMap::new(),
            static_arrays: IndexMap::new(),
            array_literal_bindings: IndexMap::new(),
            runtime_array_bindings: IndexMap::new(),
            static_objects: IndexMap::new(),
            static_reference_values: IndexMap::new(),
            static_object_keys: IndexMap::new(),
            for_in_key_bindings: IndexMap::new(),
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
        self.static_string_typed.shift_remove(name);
        self.static_numeric_values.shift_remove(name);
        self.static_identity_values.shift_remove(name);
        self.static_arrays.shift_remove(name);
        self.array_literal_bindings.shift_remove(name);
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

#[cfg(test)]
#[path = "scope_tests.rs"]
mod scope_tests;
