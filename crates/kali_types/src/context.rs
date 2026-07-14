//! TypeContext struct definition, construction, configuration, and scope-management.

use super::*;

/// Result of name resolution over a source file/module.
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub diagnostics: Vec<Diagnostic>,
    pub scopes: IndexMap<NodeId, Scope>,
    pub global_scope: Scope,
    pub repr_table: kali_common::ReprTable,
}

/// Type / name-resolution context.
pub struct TypeContext {
    pub global_scope: Scope,
    pub scopes: IndexMap<NodeId, Scope>,
    pub scope_stack: Vec<NodeId>,
    pub type_env: IndexMap<NodeId, String>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) next_scope_id: u32,
    pub(crate) next_binding_id: u32,
    pub(crate) base_path: Option<PathBuf>,
    pub(crate) api_surface: String,
    pub(crate) runtime_profiles: Vec<String>,
    pub(crate) sandbox_policy_attached: bool,
    pub(crate) in_generator_function: bool,
    pub(crate) has_generator_function: bool,
    pub(crate) has_async_generator_function: bool,
    pub(crate) has_generator_yield_delegation: bool,
    /// When true, the string-typed-variable `+` rejection is suppressed. Set only
    /// while resolving positions whose `+` codegen folds to a static string (e.g.
    /// a for-of iterable or a dynamic-import specifier), which do not reach the
    /// buggy runtime `+` path and therefore compile correctly.
    pub(crate) suppress_string_addition_rejection: bool,
    /// Spec 4a Task 5 structural default-deny (allowlist): when `true`, a
    /// for-in-key VALUE identifier read is being resolved in a PROVEN-SAFE
    /// position (a computed index — handled by NOT resolving the index — an
    /// `if` truthiness test, or an alias-copy to another key binding), so the
    /// value-escape reject in `resolve_identifier` is suppressed. Every other
    /// value read of a non-materialized for-in-key value rejects by default.
    pub(crate) suppress_forin_key_value_reject: bool,
    pub(crate) repr_table: kali_common::ReprTable,
    /// Stack of enclosing function names; module scope is `_start`.
    pub(crate) current_function: Vec<String>,
    /// Stack of scope ids parallel to `current_function`: the `ScopeType::Function`
    /// scope pushed alongside each entry — named `FunctionDeclaration`s, function
    /// expressions, arrow functions, and class methods all push here (repr-tracked
    /// as of Task 3). Lets `current_function_scope` tell whether the scope chain
    /// at a given resolution point ever crosses the one remaining UNTRACKED
    /// function-shaped scope — anonymous `export default function() {}` (its body
    /// is not pushed, see `resolve_export_default`; deferred) — before reaching
    /// the scope that `current_function_name()` actually names.
    pub(crate) current_function_scopes: Vec<NodeId>,
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeContext {
    pub fn new() -> Self {
        let mut global_scope = Scope::new(ScopeType::Global, None);
        let mut next_binding_id = 0u32;
        for builtin in builtin_globals() {
            bind_builtin(&mut global_scope, &mut next_binding_id, builtin);
            if matches!(*builtin, "Set" | "Map") {
                global_scope
                    .static_reference_values
                    .insert((*builtin).to_string(), (*builtin).to_string());
            }
        }

        Self {
            global_scope,
            scopes: IndexMap::new(),
            scope_stack: Vec::new(),
            type_env: IndexMap::new(),
            diagnostics: Vec::new(),
            next_scope_id: 1,
            next_binding_id,
            base_path: None,
            api_surface: "deno".to_string(),
            runtime_profiles: Vec::new(),
            sandbox_policy_attached: false,
            in_generator_function: false,
            has_generator_function: false,
            has_async_generator_function: false,
            has_generator_yield_delegation: false,
            suppress_string_addition_rejection: false,
            suppress_forin_key_value_reject: false,
            repr_table: kali_common::ReprTable::default(),
            current_function: vec!["_start".to_string()],
            current_function_scopes: Vec::new(),
        }
    }

    /// Enclosing function name for the current resolution position (`_start`
    /// at module scope). Consumed by the `E3200` gate's `operand_repr_is_string`
    /// (`resolve/expression.rs`) — see `current_function_scope` for the
    /// companion scope-id check that guards against misattributing a name to
    /// this function when the scope chain actually crosses the one remaining
    /// untracked (anonymous `export default function`) boundary first.
    pub(crate) fn current_function_name(&self) -> &str {
        self.current_function
            .last()
            .map(String::as_str)
            .unwrap_or("_start")
    }

    /// The `ScopeType::Function` scope id that corresponds EXACTLY to
    /// `current_function_name()` — `None` at module scope (`_start`). Named
    /// `FunctionDeclaration`s, function expressions, arrow functions, and
    /// class methods all push here (mirroring `current_function`); only an
    /// anonymous `export default function` scope is never this value, so
    /// comparing a scope id encountered while walking the scope chain against
    /// this lets callers detect "we are inside a function
    /// `current_function_name()` does not actually name" and fail closed
    /// instead of guessing.
    pub(crate) fn current_function_scope(&self) -> Option<NodeId> {
        self.current_function_scopes.last().copied()
    }

    pub fn with_base_path(base_path: impl AsRef<Path>) -> Self {
        let mut ctx = Self::new();
        ctx.base_path = Some(base_path.as_ref().to_path_buf());
        ctx
    }

    pub fn with_base_path_and_api_surface(
        base_path: impl AsRef<Path>,
        api_surface: impl Into<String>,
    ) -> Self {
        let mut ctx = Self::with_base_path(base_path);
        ctx.set_api_surface(api_surface);
        ctx
    }

    pub fn with_base_path_and_api_surface_and_runtime_profiles(
        base_path: impl AsRef<Path>,
        api_surface: impl Into<String>,
        runtime_profiles: Vec<String>,
    ) -> Self {
        let mut ctx = Self::with_base_path_and_api_surface(base_path, api_surface);
        ctx.set_runtime_profiles(runtime_profiles);
        ctx
    }

    pub fn with_api_surface(api_surface: impl Into<String>) -> Self {
        let mut ctx = Self::new();
        ctx.set_api_surface(api_surface);
        ctx
    }

    pub fn with_api_surface_and_runtime_profiles(
        api_surface: impl Into<String>,
        runtime_profiles: Vec<String>,
    ) -> Self {
        let mut ctx = Self::with_api_surface(api_surface);
        ctx.set_runtime_profiles(runtime_profiles);
        ctx
    }

    pub fn api_surface(&self) -> &str {
        &self.api_surface
    }

    pub fn set_api_surface(&mut self, api_surface: impl Into<String>) {
        self.api_surface = api_surface.into();
        if self.api_surface == "node" {
            for builtin in node_builtin_globals() {
                bind_builtin(&mut self.global_scope, &mut self.next_binding_id, builtin);
            }
        }
    }

    pub fn set_runtime_profiles(&mut self, runtime_profiles: Vec<String>) {
        self.runtime_profiles = runtime_profiles;
    }

    pub fn set_sandbox_policy_attached(&mut self, sandbox_policy_attached: bool) {
        self.sandbox_policy_attached = sandbox_policy_attached;
    }

    pub(crate) fn has_threaded_runtime_profile(&self) -> bool {
        self.runtime_profiles
            .iter()
            .any(|profile| profile.trim() == "wasm-threads")
    }

    pub fn push_scope(&mut self, scope_type: ScopeType) -> NodeId {
        let parent = self.scope_stack.last().copied();
        let scope_id = NodeId::new(self.next_scope_id);
        self.next_scope_id = self
            .next_scope_id
            .checked_add(1)
            .expect("scope id overflow is unreachable in stage 1");
        self.scopes.insert(scope_id, Scope::new(scope_type, parent));
        self.scope_stack.push(scope_id);
        scope_id
    }

    pub fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    pub fn push_block_scope(&mut self) -> NodeId {
        self.push_scope(ScopeType::Block)
    }

    pub fn push_function_scope(&mut self) -> NodeId {
        self.push_scope(ScopeType::Function)
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.global_scope.contains(name)
    }

    pub fn define(&mut self, name: impl Into<String>) -> ScopeRef<'_> {
        let name = name.into();
        let binding_id = self.next_binding_id();
        self.global_scope.bind(&name, binding_id);
        ScopeRef {
            scope: &self.global_scope,
            name,
            binding_id,
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn drain_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
        self.has_generator_function = false;
        self.has_async_generator_function = false;
        self.has_generator_yield_delegation = false;
    }

    pub fn resolve_name(&self, name: &str) -> Option<NodeId> {
        let mut current = self.scope_stack.last().copied();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id)?;
            if let Some(binding) = scope.lookup(name) {
                return Some(*binding);
            }
            current = scope.parent;
        }

        self.global_scope.lookup(name).copied()
    }

    pub(crate) fn next_binding_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_binding_id);
        self.next_binding_id = self
            .next_binding_id
            .checked_add(1)
            .expect("binding id overflow is unreachable in stage 1");
        id
    }

    pub(crate) fn current_scope_id(&self) -> Option<NodeId> {
        self.scope_stack.last().copied()
    }

    pub(crate) fn scope_mut(&mut self, scope_id: NodeId) -> Option<&mut Scope> {
        self.scopes.get_mut(&scope_id)
    }

    /// Reject-don't-miscompile: a USER binding named `Array` (const/let/var,
    /// function/class name, parameter, catch/for-of variable, import, ...)
    /// shadows the built-in `Array`, but the parser's scope-blind
    /// `Array(a, b, …)` / `new Array(…)` array desugar (and codegen's bare-
    /// `Array(n)` allocation lane) assume the GLOBAL `Array`. A shadow would
    /// otherwise silently miscompile the user's `Array(...)` calls (node runs
    /// the user binding; this branch would produce an array literal /
    /// allocation). Every USER binding form funnels through `bind_current_scope`
    /// or `bind_in_scope`, so both call this and fail closed with a clean
    /// diagnostic. The built-in `Array` itself is registered via `bind_builtin`
    /// (NOT these paths), so this never fires on compiler setup.
    /// (Whole-branch review finding I1.) Returns `true` when it rejected.
    fn reject_builtin_array_shadow(&mut self, name: &str) -> bool {
        if name != "Array" {
            return false;
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "shadowing the built-in `Array` is unavailable in the current \
             phase: a user binding named `Array` would collide with the \
             scope-blind `Array(...)` / `new Array(...)` array desugar and \
             could miscompile; rename the binding"
                .to_string(),
        ));
        true
    }

    pub(crate) fn bind_current_scope(&mut self, name: impl Into<String>) {
        let name = name.into();
        if self.reject_builtin_array_shadow(&name) {
            return;
        }
        let binding_id = self.next_binding_id();
        match self.current_scope_id() {
            Some(scope_id) => {
                let scope = self.scope_mut(scope_id).expect("active scope exists");
                if scope.contains(&name) {
                    self.diagnostics.push(duplicate_binding(&name));
                    return;
                }
                scope.bind(name, binding_id);
            }
            None => {
                if self.global_scope.contains(&name) {
                    self.diagnostics.push(duplicate_binding(&name));
                    return;
                }
                self.global_scope.bind(name, binding_id);
            }
        }
    }

    pub(crate) fn bind_in_scope(&mut self, scope_id: NodeId, name: impl Into<String>) {
        let name = name.into();
        if self.reject_builtin_array_shadow(&name) {
            return;
        }
        let binding_id = self.next_binding_id();
        let scope = self.scope_mut(scope_id).expect("scope exists");
        if scope.contains(&name) {
            self.diagnostics.push(duplicate_binding(&name));
            return;
        }
        scope.bind(name, binding_id);
    }

    pub(crate) fn variable_binding_scope(&self, kind: &str) -> NodeId {
        if kind != "var" {
            return self.current_scope_id().unwrap_or_else(|| NodeId::new(0));
        }

        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            match scope.scope_type {
                ScopeType::Function | ScopeType::Module | ScopeType::Global => return scope_id,
                _ => current = scope.parent,
            }
        }

        self.current_scope_id().unwrap_or_else(|| NodeId::new(0))
    }

    pub(crate) fn bind_function_params(&mut self, params: &[FunctionParam]) {
        for param in params {
            self.bind_current_scope(param.name.clone());
        }
        // JS parameters are reassignable (mutable) — the same binding kind as a
        // `var`/`let` local. Mark them so `binding_is_mutable` reports true and
        // the fail-closed compound/update-assignment gate admits `n -= x` /
        // `n++` on a parameter, routing it through the same codegen local lane a
        // `var` local uses (fasta Spec 6 Task 1).
        if let Some(scope_id) = self.current_scope_id() {
            for param in params {
                self.mark_binding_mutable(scope_id, &param.name);
            }
        }
    }

    pub(crate) fn bind_name_list(&mut self, names: &[String]) {
        for name in names {
            self.bind_current_scope(name.clone());
        }
    }

    /// Mark an already-bound `name` as a mutable binding in `scope_id`. No-op
    /// if `name` is not bound in that scope. Used to flag function parameters
    /// mutable after they are bound (they are reassignable in JS).
    pub(crate) fn mark_binding_mutable(&mut self, scope_id: NodeId, name: &str) {
        if let Some(scope) = self.scope_mut(scope_id) {
            if scope.bindings.contains_key(name) {
                scope.mutable_bindings.insert(name.to_owned(), true);
            }
        }
    }

    pub(crate) fn bind_type_params(&mut self, type_params: &[String]) {
        self.bind_name_list(type_params)
    }
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod context_tests;
