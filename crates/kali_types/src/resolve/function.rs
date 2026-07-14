//! Function, arrow, and class resolution.
use crate::resolve::block_contains_yield_delegation;
use crate::*;

impl TypeContext {
    pub(crate) fn resolve_function_expression(&mut self, expr: &FunctionExpression) {
        let function_scope_id = self.push_scope(ScopeType::Function);
        // Repr-tracking (Stage 6): a function-expression body is a REAL function
        // scope, and codegen already compiles it as a named function (Task 2's
        // AST-assigned `id`). Push it onto current_function so
        // binding_repr_function_key resolves inside the body instead of
        // returning None — which is what made compound/typeof/new fail closed
        // in here. `expr.id` is `None` only when this AST did not go through
        // the CLI's `name_anonymous_functions` pre-pass (e.g. a kali_types
        // unit test resolving a hand-built AST directly — see
        // `resolve/call_tests.rs`); fail SAFE with a deterministic synthetic
        // name derived from the scope id rather than panicking (mirrors
        // `kali_hir::next_synthetic_function_name`'s fallback intent, though
        // this name never has to agree with HIR's counter: HIR is never
        // invoked on an AST the pre-pass skipped).
        let name = expr
            .id
            .clone()
            .unwrap_or_else(|| format!("__kali_untracked_fn_{}", function_scope_id.as_u32()));
        self.current_function.push(name.clone());
        self.current_function_scopes.push(function_scope_id);
        let previous_generator = self.in_generator_function;
        self.in_generator_function = expr.generator;
        if expr.generator {
            self.record_generator_function_lowering(expr.is_async);
        }
        if let Some(id_name) = &expr.id {
            self.bind_current_scope(id_name.clone());
        }
        self.bind_function_params(&expr.params);
        // Structural runtime-array registry (C1): an array-typed PARAMETER is
        // registered by codegen's emitter (emitter.rs:
        // `repr_table.is_array_binding(function_name, name)`). Mirror that
        // here so a `join`/store/`.length` on an array param stays in
        // lockstep with what codegen lowers (mirrors the FunctionDeclaration
        // arm, `resolve/mod.rs`).
        for param in &expr.params {
            if self.repr_table.is_array_binding(&name, &param.name) {
                if let Some(scope) = self.scopes.get_mut(&function_scope_id) {
                    scope
                        .runtime_array_bindings
                        .insert(param.name.clone(), true);
                }
            }
        }
        if let Some(body) = &expr.body {
            self.resolve_block_body(body);
        }
        self.in_generator_function = previous_generator;
        self.current_function_scopes.pop();
        self.current_function.pop();
        self.pop_scope();
    }

    pub(crate) fn resolve_arrow_function(&mut self, expr: &ArrowFunctionExpression) {
        let function_scope_id = self.push_scope(ScopeType::Function);
        // Repr-tracking (Stage 6): see `resolve_function_expression` above —
        // same reasoning applies to arrow-function bodies. Task 2's AST pass
        // always assigns `expr.id`; the fallback only covers hand-built ASTs
        // in kali_types unit tests that bypass the pre-pass.
        let name = expr
            .id
            .clone()
            .unwrap_or_else(|| format!("__kali_untracked_fn_{}", function_scope_id.as_u32()));
        self.current_function.push(name.clone());
        self.current_function_scopes.push(function_scope_id);
        self.bind_function_params(&expr.params);
        // Structural runtime-array registry (C1) — mirrors
        // `resolve_function_expression` above.
        for param in &expr.params {
            if self.repr_table.is_array_binding(&name, &param.name) {
                if let Some(scope) = self.scopes.get_mut(&function_scope_id) {
                    scope
                        .runtime_array_bindings
                        .insert(param.name.clone(), true);
                }
            }
        }
        if let Some(return_type) = &expr.returnType {
            self.resolve_type_annotation_text(return_type);
        }
        self.resolve_expression(&expr.body);
        self.current_function_scopes.pop();
        self.current_function.pop();
        self.pop_scope();
    }

    pub(crate) fn resolve_class_expression(&mut self, expr: &ClassExpression) {
        self.push_scope(ScopeType::Class);
        if let Some(name) = &expr.id {
            self.bind_current_scope(name.clone());
        }
        self.resolve_class_body(&expr.body);
        self.pop_scope();
    }

    pub(crate) fn resolve_class_body(&mut self, body: &ClassBody) {
        self.push_scope(ScopeType::Class);
        let mut has_generator = false;
        let mut has_async_generator = false;
        let mut has_yield_delegation = false;
        for method in &body.methods {
            if method.generator {
                has_generator = true;
                if method.is_async {
                    has_async_generator = true;
                }
                if method
                    .body
                    .as_deref()
                    .is_some_and(block_contains_yield_delegation)
                {
                    has_yield_delegation = true;
                }
                continue;
            }
            self.bind_current_scope(method.name.clone());
            let function_scope_id = self.push_scope(ScopeType::Function);
            // Repr-tracking (Stage 6): a class-method body is a REAL function
            // scope compiled under its own name (mirrors the
            // FunctionDeclaration arm, `resolve/mod.rs`). Unlike a function
            // expression / arrow, `method.name` is a plain `String` (never
            // anonymous — a class method always has a name), so there is no
            // fallback to reason about here.
            self.current_function.push(method.name.clone());
            self.current_function_scopes.push(function_scope_id);
            self.bind_name_list(&method.params);
            if let Some(scope_id) = self.current_scope_id() {
                for param in &method.params {
                    self.mark_binding_mutable(scope_id, param);
                }
            }
            // Structural runtime-array registry (C1) — mirrors the
            // FunctionDeclaration arm and `resolve_function_expression`.
            for param in &method.params {
                if self.repr_table.is_array_binding(&method.name, param) {
                    if let Some(scope) = self.scopes.get_mut(&function_scope_id) {
                        scope.runtime_array_bindings.insert(param.clone(), true);
                    }
                }
            }
            if let Some(body) = &method.body {
                self.resolve_block_body(body);
            }
            self.current_function_scopes.pop();
            self.current_function.pop();
            self.pop_scope();
        }
        if has_generator || has_async_generator {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                generator_class_method_yield_lowering_unavailable_message_for_flavors(
                    has_generator,
                    has_async_generator,
                    has_yield_delegation,
                ),
            ));
        }
        self.pop_scope();
    }
}

#[cfg(test)]
#[path = "function_tests.rs"]
mod function_tests;
