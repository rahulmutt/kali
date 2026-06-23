//! Function, arrow, and class resolution.
use crate::resolve::block_contains_yield_delegation;
use crate::*;

impl TypeContext {
    pub(crate) fn resolve_function_expression(&mut self, expr: &FunctionExpression) {
        self.push_scope(ScopeType::Function);
        let previous_generator = self.in_generator_function;
        self.in_generator_function = expr.generator;
        if expr.generator {
            self.record_generator_function_lowering(expr.is_async);
        }
        if let Some(name) = &expr.id {
            self.bind_current_scope(name.clone());
        }
        self.bind_function_params(&expr.params);
        if let Some(body) = &expr.body {
            self.resolve_block_body(body);
        }
        self.in_generator_function = previous_generator;
        self.pop_scope();
    }

    pub(crate) fn resolve_arrow_function(&mut self, expr: &ArrowFunctionExpression) {
        self.push_scope(ScopeType::Function);
        self.bind_function_params(&expr.params);
        if let Some(return_type) = &expr.returnType {
            self.resolve_type_annotation_text(return_type);
        }
        self.resolve_expression(&expr.body);
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
            self.push_scope(ScopeType::Function);
            self.bind_name_list(&method.params);
            if let Some(body) = &method.body {
                self.resolve_block_body(body);
            }
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
