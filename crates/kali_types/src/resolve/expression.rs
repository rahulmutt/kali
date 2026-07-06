//! Expression resolution.
use crate::*;

impl TypeContext {
    pub(crate) fn is_simple_for_of_binding_expression(&self, expression: &Expression) -> bool {
        matches!(
            self.unwrap_for_of_wrapper_expression(expression),
            Expression::Identifier(_)
        )
    }

    pub(crate) fn is_simple_update_target_expression(&self, expression: &Expression) -> bool {
        matches!(
            expression,
            Expression::Identifier(_)
                | Expression::ParenthesizedExpression(_)
                | Expression::TypeAssertion(_)
                | Expression::SatisfiesExpression(_)
                | Expression::DecoratedExpression(_)
        )
    }

    pub(crate) fn resolve_update_binding_name(&self, expression: &Expression) -> Option<String> {
        match expression {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_update_binding_name(&expr.expression)
            }
            Expression::TypeAssertion(expr) => self.resolve_update_binding_name(&expr.expression),
            Expression::SatisfiesExpression(expr) => {
                self.resolve_update_binding_name(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_update_binding_name(&expr.expression)
            }
            _ => None,
        }
    }

    /// True when a binding named `name` is known to hold a *string* value
    /// (recorded by `resolve_variable_declaration` when its initializer is
    /// string-typed). Walks the scope chain, then the global scope. Reassignment
    /// clears the flag via `invalidate_static_binding`, so a name that was a string
    /// but has since been reassigned (e.g. to a number) is not reported as a
    /// string here — keeping the check flow-aware and sound.
    pub(crate) fn binding_is_string_typed(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let Some(scope) = self.scopes.get(&scope_id) else {
                return false;
            };
            if let Some(&value) = scope.static_string_typed.get(name) {
                return value;
            }
            current = scope.parent;
        }
        self.global_scope
            .static_string_typed
            .get(name)
            .copied()
            .unwrap_or(false)
    }

    /// Semantic string-typedness of an expression: does it evaluate to a string at
    /// runtime? Covers string/template literals, `+` expressions with a string
    /// operand (JS `string + any` is a string), and *identifiers bound to a string
    /// value* (transparent wrappers unwrapped). This intentionally recognizes
    /// string-typed variables, which codegen's structural check does not.
    pub(crate) fn expression_is_string_typed(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Literal(LiteralValue::String(_)) => true,
            Expression::TemplateLiteral(_) => true,
            Expression::Identifier(name) => self.binding_is_string_typed(name),
            Expression::BinaryExpression(expr) if expr.operator == "+" => {
                self.expression_is_string_typed(&expr.left)
                    || self.expression_is_string_typed(&expr.right)
            }
            Expression::ParenthesizedExpression(expr) => {
                self.expression_is_string_typed(&expr.expression)
            }
            Expression::TypeAssertion(expr) => self.expression_is_string_typed(&expr.expression),
            Expression::SatisfiesExpression(expr) => {
                self.expression_is_string_typed(&expr.expression)
            }
            _ => false,
        }
    }

    /// Mirror of codegen's structural `is_string_valued`
    /// (`kali_codegen/src/emit/operators.rs`): recognizes only string/template
    /// literals and `+` chains rooted in one — NOT a variable that holds a string.
    /// Operands for which this is true are lowered to string concatenation
    /// correctly and must not be rejected.
    fn expression_is_codegen_string_valued(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Literal(LiteralValue::String(_)) => true,
            Expression::TemplateLiteral(_) => true,
            Expression::BinaryExpression(expr) if expr.operator == "+" => {
                self.expression_is_codegen_string_valued(&expr.left)
                    || self.expression_is_codegen_string_valued(&expr.right)
            }
            Expression::ParenthesizedExpression(expr) => {
                self.expression_is_codegen_string_valued(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.expression_is_codegen_string_valued(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.expression_is_codegen_string_valued(&expr.expression)
            }
            _ => false,
        }
    }

    /// True when `name`'s runtime representation at the CURRENT resolution
    /// point is proven `Repr::String` by the repr inference — the SAME signal
    /// codegen's `is_string_valued` identifier arm uses
    /// (`kali_codegen/src/emit/operators.rs`), so the gate and codegen never
    /// disagree.
    ///
    /// Mirrors codegen's local/module-const dichotomy (`self.locals.contains_key
    /// (name) ... else self.repr_table.scalar("_start", name)`), which is a
    /// FLAT per-function-body test — codegen has no lexical/block scoping
    /// inside one wasm function, so any binding declared anywhere in the
    /// current function counts as "local" to it. Concretely: walk the
    /// resolver's scope chain from the current position outward.
    ///
    /// - If `name` is found in a scope at or before the tracked function's own
    ///   `ScopeType::Function` scope (`current_function_scope()`), it is local
    ///   to the SAME function codegen is about to emit: consult
    ///   `scalar(current_function_name(), name)`.
    /// - If we reach module/global scope without finding it (and without
    ///   crossing an untracked boundary), it is a free reference to a
    ///   module-level binding: consult `scalar("_start", name)`, mirroring
    ///   codegen's fallback.
    /// - If, before either of the above, we reach a `ScopeType::Function`
    ///   scope that is NOT `current_function_scope()` — an arrow function,
    ///   function expression, class method, or `export default function`,
    ///   none of which push onto `current_function` (see
    ///   `TypeContext::current_function_scope`'s doc) — `current_function_name()`
    ///   does not actually name the function whose body we are in, so neither
    ///   table lookup above is safe: a same-named module binding or a
    ///   same-named binding in a DIFFERENT enclosing function could wrongly
    ///   suppress the gate. FAIL CLOSED (return `false`) instead of guessing.
    fn identifier_repr_is_string(&self, name: &str) -> bool {
        use kali_common::Repr;
        let tracked_scope = self.current_function_scope();
        let mut current = self.current_scope_id();
        loop {
            let Some(scope_id) = current else {
                // Reached module/global scope: free top-level reference.
                return self.repr_table.scalar("_start", name) == Repr::String;
            };
            let Some(scope) = self.scopes.get(&scope_id) else {
                return false;
            };
            if scope.scope_type == ScopeType::Function && Some(scope_id) != tracked_scope {
                return false;
            }
            if scope.contains(name) {
                return self.repr_table.scalar(self.current_function_name(), name) == Repr::String;
            }
            if scope.scope_type == ScopeType::Function {
                // Reached the tracked function's own top scope without finding
                // `name` there: mirror codegen's `self.locals`-miss fallback,
                // which unconditionally consults the module `_start` table
                // regardless of any further-enclosing scope (codegen does not
                // model closures over an outer function's locals).
                return self.repr_table.scalar("_start", name) == Repr::String;
            }
            current = scope.parent;
        }
    }

    /// True when `operand`'s runtime representation is proven `Repr::String` by
    /// the repr inference — the SAME signal codegen's `is_string_valued` uses,
    /// so the gate and codegen never disagree. Covers a string-typed identifier
    /// (variable/param, via `identifier_repr_is_string`) and a call to a
    /// string-returning function.
    fn operand_repr_is_string(&self, operand: &Expression) -> bool {
        use kali_common::Repr;
        match operand {
            Expression::Identifier(name) => self.identifier_repr_is_string(name),
            Expression::CallExpression(call) => match &call.callee {
                Expression::Identifier(callee) => {
                    self.repr_table.return_repr(callee) == Repr::String
                }
                _ => false,
            },
            Expression::ParenthesizedExpression(inner) => {
                self.operand_repr_is_string(&inner.expression)
            }
            _ => false,
        }
    }

    /// True when `name`'s string value may contain non-ASCII text. Checks BOTH
    /// the current-function and module scopes (over-approximate: either scope
    /// non-ASCII rejects — fail-closed against the scope-resolution ambiguity
    /// `identifier_repr_is_string` handles precisely for the String bit).
    fn identifier_string_may_be_non_ascii(&self, name: &str) -> bool {
        let func = self.current_function_name();
        self.repr_table.is_string_non_ascii(func, name)
            || self.repr_table.is_string_non_ascii("_start", name)
    }

    /// True when `expr` is proven an ASCII-only runtime string: `Repr::String`
    /// via the inference AND never reached by a non-ASCII seed. The receivers
    /// the substring/.length lanes accept. Fail-closed: unknown shapes are false.
    pub(crate) fn expression_repr_is_ascii_string(&self, expr: &Expression) -> bool {
        use kali_common::Repr;
        match expr {
            Expression::Identifier(name) => {
                self.identifier_repr_is_string(name)
                    && !self.identifier_string_may_be_non_ascii(name)
            }
            Expression::CallExpression(call) => match &call.callee {
                Expression::Identifier(callee) => {
                    self.repr_table.return_repr(callee) == Repr::String
                        && !self.repr_table.is_string_non_ascii_return(callee)
                }
                // A chained substring: ASCII iff ITS receiver is.
                Expression::MemberExpression(member)
                    if member.computed_index.is_none()
                        && member.property.as_str() == "substring" =>
                {
                    self.expression_repr_is_ascii_string(&member.object)
                }
                _ => false,
            },
            Expression::ParenthesizedExpression(inner) => {
                self.expression_repr_is_ascii_string(&inner.expression)
            }
            _ => false,
        }
    }

    /// True when `arg` is safe as a runtime substring bound: provably integer-
    /// repr at runtime. Float/string/unknown shapes reject (JS ToInteger on
    /// NaN/fractions is unimplemented). Fail-closed.
    pub(crate) fn expression_is_int_repr_bound(&self, arg: &Expression) -> bool {
        use kali_common::Repr;
        match arg {
            Expression::Literal(LiteralValue::Number(n)) => n.is_finite() && n.fract() == 0.0,
            Expression::Identifier(name) => {
                let func = self.current_function_name();
                self.repr_table.scalar(func, name) == Repr::I64
                    && self.repr_table.scalar("_start", name) == Repr::I64
            }
            Expression::BinaryExpression(binary)
                if matches!(binary.operator.as_str(), "+" | "-" | "*" | "%") =>
            {
                self.expression_is_int_repr_bound(&binary.left)
                    && self.expression_is_int_repr_bound(&binary.right)
            }
            Expression::UnaryExpression(unary) if unary.operator == "-" => {
                self.expression_is_int_repr_bound(&unary.argument)
            }
            Expression::ParenthesizedExpression(inner) => {
                self.expression_is_int_repr_bound(&inner.expression)
            }
            Expression::CallExpression(call) => match &call.callee {
                Expression::Identifier(callee) => self.repr_table.return_repr(callee) == Repr::I64,
                _ => false,
            },
            _ => false,
        }
    }

    /// Rejects a `+` whose lowering codegen cannot perform correctly: any operand
    /// that is *string-typed* but is not a codegen-recognized structural string
    /// expression (i.e. a string-typed variable / dynamic value that codegen's
    /// `is_string_valued` will not see) AND not proven `Repr::String` by the
    /// repr inference (`operand_repr_is_string` — the same signal codegen's
    /// runtime identifier/call arms now consult, so an operand this predicate
    /// lets through is one codegen lowers correctly). For any other unsupported
    /// operand codegen either integer-adds two string handles or coerces a
    /// string handle through `int_to_string`, both of which silently produce
    /// garbage. Rejecting with a clear `E3200` diagnostic makes the outcome
    /// sound (a compile error instead of a wrong result) while leaving every
    /// literal-rooted concatenation (e.g. `"x" + 3`, `"P(" + n + ")"`) and every
    /// `Repr::String`-backed variable/param/return compiling and correct.
    fn reject_unsupported_string_variable_addition(&mut self, expr: &BinaryExpression) {
        if expr.operator != "+" || self.suppress_string_addition_rejection {
            return;
        }
        let operand_is_unsupported_string = |operand: &Expression| {
            self.expression_is_string_typed(operand)
                && !self.expression_is_codegen_string_valued(operand)
                && !self.operand_repr_is_string(operand)
        };
        if operand_is_unsupported_string(&expr.left) || operand_is_unsupported_string(&expr.right) {
            self.diagnostics.push(
                Diagnostic::error(
                    e3::TYPE_MISMATCH as u32,
                    "'+' with a string-typed variable operand is unavailable in the current direct-runtime path: only string concatenation rooted in a string or template literal (for example \"x\" + 3) is lowered to runtime concatenation; a variable that holds a string is not recognized and would be miscompiled".to_string(),
                )
                .with_suggestion(
                    "root the concatenation in a string literal (\"\" + value), build the string with literal-rooted concatenation, or use the later compatibility path",
                ),
            );
        }
    }

    /// Reject a string-typed expression used as a ternary condition (fail-closed).
    /// Uses the same string-typedness signal as the `+` gate
    /// (`expression_is_string_typed`), covering string literals/templates, `+`
    /// chains rooted in one, and string-typed variables.
    fn reject_string_condition_expression(&mut self, test: &Expression) {
        if self.expression_is_string_typed(test) {
            self.diagnostics.push(Diagnostic::error(
                e3::TYPE_MISMATCH as u32,
                "a string value is unavailable as a ternary condition in the current direct-runtime path; its truthiness is not evaluated".to_string(),
            ));
        }
    }

    /// Resolves `expression` in a position where codegen folds a string-typed `+`
    /// to a static string (a for-of iterable, a dynamic-import specifier). Such a
    /// `+` never reaches the buggy runtime `+` path, so the string-typed-variable
    /// rejection is suppressed for its duration.
    pub(crate) fn resolve_static_string_fold_position(&mut self, expression: &Expression) {
        let previous = self.suppress_string_addition_rejection;
        self.suppress_string_addition_rejection = true;
        self.resolve_expression(expression);
        self.suppress_string_addition_rejection = previous;
    }

    pub(crate) fn resolve_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Identifier(name) => self.resolve_identifier(name),
            Expression::Literal(_) => {}
            Expression::BinaryExpression(expr) => {
                self.resolve_expression(&expr.left);
                self.resolve_expression(&expr.right);
                self.reject_unsupported_string_variable_addition(expr);
            }
            Expression::UnaryExpression(expr) => {
                if expr.operator == "delete" {
                    if let Expression::MemberExpression(member) = &expr.argument {
                        if self.resolve_late_process_env_mutation_member(member) {
                            return;
                        }
                    }
                }
                self.resolve_expression(&expr.argument)
            }
            Expression::CallExpression(expr) => self.resolve_call_expression(expr),
            Expression::MemberExpression(expr) => self.resolve_member_expression(expr),
            Expression::ArrayExpression(ArrayExpression { elements }) => {
                for element in elements.iter().flatten() {
                    match element {
                        ExpressionOrSpread::Expression(expr) => self.resolve_expression(expr),
                        ExpressionOrSpread::Spread(spread) => {
                            self.resolve_expression(&spread.argument)
                        }
                        ExpressionOrSpread::Empty => {}
                    }
                }
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                for property in properties {
                    self.resolve_object_property(property);
                }
            }
            Expression::FunctionExpression(expr) => self.resolve_function_expression(expr),
            Expression::ArrowFunctionExpression(expr) => self.resolve_arrow_function(expr),
            Expression::ClassExpression(expr) => self.resolve_class_expression(expr),
            Expression::NewExpression(expr) => {
                self.resolve_expression(&expr.callee);
                for arg in &expr.args {
                    self.resolve_expression(arg);
                }
            }
            Expression::MetaProperty(_) => {}
            Expression::TemplateLiteral(template) => self.resolve_template_literal(template),
            Expression::TaggedTemplateExpression(expr) => {
                self.resolve_expression(&expr.tag);
                self.resolve_template_literal(&expr.template);
            }
            Expression::UpdateExpression(expr) => self.resolve_update_expression(expr),
            Expression::AssignmentExpression(expr) => {
                self.resolve_expression(&expr.left);
                self.resolve_expression(&expr.right);

                if self.resolve_late_env_assignment_mutation(expr) {
                    return;
                }

                if matches!(expr.operator, AssignmentOperator::Assign) {
                    if let Expression::MemberExpression(member) = &expr.left {
                        let dotted = Self::member_access_name(member)
                            .unwrap_or_else(|| member.property.clone());
                        if self.api_surface == "node"
                            && Self::is_process_env_mutation_path(&dotted)
                            && !Self::is_process_env_root_path(&dotted)
                        {
                            return;
                        }
                    }
                }

                if matches!(expr.operator, AssignmentOperator::Assign) {
                    if let Some(name) = self.resolve_update_binding_name(&expr.left) {
                        // Reassignment clears the previous static tracking, then
                        // re-establishes string-typedness from the new value so a
                        // later `+` on this binding is still recognized. When the
                        // right-hand side is provably non-string the flag stays
                        // cleared, keeping the check flow-aware (e.g.
                        // `let s = "x"; s = 5; s + 1` stays a valid numeric `6`).
                        let right_is_string = self.expression_is_string_typed(&expr.right);
                        self.invalidate_static_binding(&name);
                        if right_is_string {
                            self.mark_binding_string_typed(&name);
                        }
                    }
                    return;
                }

                let Some(name) = self.resolve_update_binding_name(&expr.left) else {
                    let message = if matches!(expr.operator, AssignmentOperator::NullishAssign) {
                        "nullish assignment lowering is unavailable unless the target is a mutable local binding; use a mutable variable or the later compatibility path".to_string()
                    } else {
                        "compound assignment lowering is unavailable unless the target is a mutable local binding; use a mutable variable or the later compatibility path".to_string()
                    };
                    self.diagnostics
                        .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
                    return;
                };

                self.invalidate_static_binding(&name);

                if !self.binding_is_mutable(&name) {
                    let message = if matches!(expr.operator, AssignmentOperator::NullishAssign) {
                        format!(
                            "nullish assignment lowering is unavailable for binding '{}' unless it was declared with a mutable binding kind; use a mutable variable or the later compatibility path",
                            name
                        )
                    } else {
                        format!(
                            "compound assignment lowering is unavailable for binding '{}' unless it was declared with a mutable binding kind; use a mutable variable or the later compatibility path",
                            name
                        )
                    };
                    self.diagnostics
                        .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
                }
            }
            Expression::LogicalExpression(expr) => {
                self.resolve_expression(&expr.left);
                self.resolve_expression(&expr.right);
            }
            Expression::ConditionalExpression(expr) => {
                self.resolve_expression(&expr.test);
                self.resolve_expression(&expr.consequent);
                self.resolve_expression(&expr.alternate);
                // A string-typed ternary TEST cannot be truthiness-tested here:
                // the conditional lowering is degenerate (it yields the test
                // value itself, ignoring the branches), so a string test would
                // print/return the raw string instead of selecting a branch.
                // Reject fail-closed. No base-correct string ternary exists (the
                // degenerate lowering was always wrong for a string test).
                self.reject_string_condition_expression(&expr.test);
            }
            Expression::SequenceExpression(expr) => {
                for subexpr in &expr.expressions {
                    self.resolve_expression(subexpr);
                }
            }
            Expression::ParenthesizedExpression(expr) => self.resolve_expression(&expr.expression),
            Expression::YieldExpression(expr) => {
                if self.in_generator_function && expr.delegate {
                    self.has_generator_yield_delegation = true;
                }
                if !self.in_generator_function {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        generator_function_yield_lowering_unavailable_message(false, expr.delegate),
                    ));
                }
                if let Some(argument) = &expr.argument {
                    self.resolve_expression(argument);
                }
            }
            Expression::AwaitExpression(expr) => self.resolve_expression(&expr.argument),
            Expression::OptionalChainExpression(expr) => self.resolve_optional_chain(expr),
            Expression::ChainExpression(expr) => self.resolve_expression(&expr.expression),
            Expression::SpreadElement(expr) => self.resolve_expression(&expr.argument),
            Expression::RestElement(expr) => self.resolve_expression(&expr.argument),
            Expression::ImportExpression(expr) => self.resolve_import_expression(expr),
            Expression::DecoratedExpression(DecoratedExpression { expression }) => {
                self.resolve_expression(expression)
            }
            Expression::JsxElement(expr) => self.resolve_jsx_element(expr),
            Expression::JsxFragment(expr) => self.resolve_jsx_fragment(expr),
            Expression::JsxEmptyExpression => {}
            Expression::TypeAssertion(expr) => self.resolve_type_assertion(expr),
            Expression::SatisfiesExpression(expr) => self.resolve_satisfies_expression(expr),
            Expression::ThisExpression | Expression::SuperExpression => {}
            Expression::PrivateIdentifier(_) | Expression::BigIntLiteral(_) => {}
        }
    }

    pub(crate) fn resolve_update_expression(&mut self, expr: &UpdateExpression) {
        self.resolve_expression(&expr.argument);

        if !self.is_simple_update_target_expression(&expr.argument) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "update expression lowering is unavailable unless the target is a mutable local binding; use a local binding or the later compatibility path",
            ));
            return;
        }

        let Some(name) = self.resolve_update_binding_name(&expr.argument) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "update expression lowering is unavailable unless the target is a mutable local binding; use a local binding or the later compatibility path",
            ));
            return;
        };

        self.invalidate_static_binding(&name);

        if !self.binding_is_mutable(&name) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "update expression lowering is unavailable for binding '{}' unless it was declared with a mutable binding kind; use a mutable local binding or the later compatibility path",
                    name
                ),
            ));
        }
    }

    pub(crate) fn invalidate_static_binding(&mut self, name: &str) {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            if let Some(scope) = self.scopes.get_mut(&scope_id) {
                if scope.bindings.contains_key(name) {
                    scope.invalidate_static_binding(name);
                    return;
                }
                current = scope.parent;
            } else {
                return;
            }
        }

        if self.global_scope.bindings.contains_key(name) {
            self.global_scope.invalidate_static_binding(name);
        }
    }

    /// Records that the binding `name` currently holds a string value, in the
    /// scope where `name` is declared. Used after an assignment whose right-hand
    /// side is string-typed so that a later `+` on `name` is recognized as an
    /// unsupported string-typed-variable operand (see
    /// `reject_unsupported_string_variable_addition`).
    pub(crate) fn mark_binding_string_typed(&mut self, name: &str) {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            if let Some(scope) = self.scopes.get_mut(&scope_id) {
                if scope.bindings.contains_key(name) {
                    scope.static_string_typed.insert(name.to_string(), true);
                    return;
                }
                current = scope.parent;
            } else {
                return;
            }
        }

        if self.global_scope.bindings.contains_key(name) {
            self.global_scope
                .static_string_typed
                .insert(name.to_string(), true);
        }
    }

    pub(crate) fn binding_is_mutable(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if scope.bindings.contains_key(name) {
                return scope.mutable_bindings.get(name).copied().unwrap_or(false);
            }
            current = scope.parent;
        }

        self.global_scope.bindings.contains_key(name)
            && self
                .global_scope
                .mutable_bindings
                .get(name)
                .copied()
                .unwrap_or(false)
    }

    pub(crate) fn resolve_import_expression(&mut self, expr: &ImportExpression) {
        self.resolve_static_string_fold_position(&expr.source);

        if let Some(source) = self.resolve_static_import_source(&expr.source) {
            match self.resolve_import_source(&source) {
                Ok(true) => {}
                Ok(false) => {
                    self.diagnostics.push(
                        Diagnostic::error(
                            e4::DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH as u32,
                            format!(
                                "dynamic import target '{}' could not be resolved in the linked graph",
                                source
                            ),
                        )
                        .with_suggestion(
                            "use a statically known import specifier or link the module in the build graph",
                        ),
                    );
                }
                Err(diagnostic) => self.diagnostics.push(diagnostic),
            }
        } else {
            self.diagnostics.push(
                Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "non-literal dynamic import() is unavailable in the current phase; use a statically known import specifier that can be resolved in the linked graph".to_string(),
                )
                .with_suggestion(
                    "rewrite the import() target so the compiler can determine a linked-graph module at compile time",
                ),
            );
        }
    }

    pub(crate) fn resolve_static_import_source(&self, expression: &Expression) -> Option<String> {
        self.resolve_static_string_expression(expression)
    }

    pub(crate) fn normalize_import_segment(value: &str) -> String {
        let trimmed = value.trim();
        if trimmed.len() >= 2 {
            let mut chars = trimmed.chars();
            let first = chars.next().unwrap();
            let last = chars.next_back().unwrap();
            if matches!((first, last), ('"', '"') | ('\'', '\'') | ('`', '`')) {
                return trimmed[1..trimmed.len() - 1].to_string();
            }
        }
        trimmed.to_string()
    }

    pub(crate) fn resolve_identifier(&mut self, name: &str) {
        if matches!(name, "unknown" | "undefined") {
            return;
        }

        if matches!(name, "SharedArrayBuffer" | "Atomics") {
            if self.has_threaded_runtime_profile() {
                return;
            }
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "threaded runtime global '{}' is unavailable until the WASM-threaded profile is enabled",
                    name
                ),
            ));
            return;
        }

        if name == "Intl" {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "broader Intl support is unavailable until the later web/Intl compatibility path is enabled".to_string(),
            ));
            return;
        }

        if matches!(
            name,
            "Proxy" | "WeakMap" | "WeakSet" | "WeakRef" | "FinalizationRegistry"
        ) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "late object-model API '{}' is unavailable until the later object-model compatibility path is enabled",
                    name
                ),
            ));
            return;
        }

        if self.resolve_name(name).is_none() {
            self.diagnostics.push(
                Diagnostic::error(
                    e3::UNDEFINED_IDENTIFIER as u32,
                    format!("undefined identifier '{}'", name),
                )
                .with_suggestion("declare the name in the current module or import it"),
            );
        }
    }

    pub(crate) fn resolve_optional_chain(&mut self, expr: &OptionalChainExpression) {
        match expr.inner.as_ref() {
            OptionalChainInner::NonNull { object, .. } => self.resolve_expression(object),
        }
    }

    pub(crate) fn resolve_template_literal(&mut self, template: &TemplateLiteral) {
        for expr in &template.expressions {
            self.resolve_expression(expr);
        }
    }

    pub(crate) fn resolve_object_property(&mut self, property: &ObjectProperty) {
        self.resolve_property_name(&property.key);
        self.resolve_expression(&property.value);
    }

    pub(crate) fn resolve_property_name(&mut self, name: &PropertyName) {
        match name {
            PropertyName::Identifier(_) | PropertyName::Number(_) | PropertyName::String(_) => {}
        }
    }

    pub(crate) fn resolve_type_assertion(&mut self, expr: &TypeAssertion) {
        self.resolve_type_annotation_text(&expr.type_name);
        self.resolve_expression(&expr.expression);
    }

    pub(crate) fn resolve_satisfies_expression(&mut self, expr: &kali_ast::SatisfiesExpression) {
        self.resolve_type_annotation_text(&expr.type_name);
        self.resolve_expression(&expr.expression);
    }

    pub(crate) fn resolve_relative_import_source(&self, base_dir: &Path, source: &str) -> bool {
        let candidate = base_dir.join(source);
        if candidate.is_file() {
            return true;
        }

        if candidate.is_dir() && self.resolve_directory_index_candidate(&candidate) {
            return true;
        }

        let extensions = [
            "ts", "tsx", "js", "jsx", "mts", "cts", "d.ts", "d.mts", "d.cts",
        ];
        extensions.iter().any(|extension| {
            let candidate = if source.ends_with(extension) {
                base_dir.join(source)
            } else {
                base_dir.join(format!("{}.{}", source, extension))
            };
            candidate.is_file()
                || (candidate.is_dir() && self.resolve_directory_index_candidate(&candidate))
        })
    }

    pub(crate) fn resolve_directory_index_candidate(&self, directory: &Path) -> bool {
        for index_name in [
            "index.ts",
            "index.tsx",
            "index.js",
            "index.jsx",
            "index.mts",
            "index.mjs",
            "index.cts",
            "index.cjs",
            "index.d.ts",
            "index.d.mts",
            "index.d.cts",
        ] {
            if directory.join(index_name).is_file() {
                return true;
            }
        }

        false
    }

    pub(crate) fn resolve_import_source(&self, source: &str) -> Result<bool, Diagnostic> {
        if self.api_surface == "node" && is_node_builtin_specifier(source) {
            return Ok(true);
        }

        if self.api_surface == "node" && source.starts_with("node:") {
            return Err(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "node builtin '{}' is not available on the explicit Node API surface",
                    source
                ),
            ));
        }

        let base_dir = self
            .base_path
            .as_ref()
            .and_then(|path| path.parent())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let project_root =
            kali_npm::discover_project_root(&base_dir).unwrap_or_else(|| base_dir.clone());

        if self.resolve_relative_import_source(&base_dir, source) {
            return Ok(true);
        }

        let Some(resolved) = kali_npm::resolve_materialized_import_with_browser_context(
            project_root,
            source,
            self.api_surface == "browser",
        ) else {
            return Ok(false);
        };

        if let Some(diagnostic) = reject_native_addon_package_source(&resolved) {
            return Err(diagnostic);
        }

        if self.api_surface != "node" {
            if let Ok(contents) = fs::read_to_string(&resolved) {
                if let Some(builtin) = kali_npm::source_mentions_node_only_host_api(&contents) {
                    return Err(Diagnostic::error(
                        e6::NODE_ONLY_HOST_APIS as u32,
                        format!(
                            "package uses Node-only host API '{}' in '{}' and falls outside the default standalone context; use the Phase-3 Node compatibility target",
                            builtin,
                            resolved.display()
                        ),
                    ));
                }
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
#[path = "expression_tests.rs"]
mod expression_tests;
