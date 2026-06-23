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

    pub(crate) fn resolve_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Identifier(name) => self.resolve_identifier(name),
            Expression::Literal(_) => {}
            Expression::BinaryExpression(expr) => {
                self.resolve_expression(&expr.left);
                self.resolve_expression(&expr.right);
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
                        self.invalidate_static_binding(&name);
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
        self.resolve_expression(&expr.source);

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
