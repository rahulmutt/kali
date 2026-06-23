//! Object static-analysis helpers.
use crate::*;

impl TypeContext {
    pub(crate) fn resolve_static_object_identity_binding(
        &self,
        name: &str,
    ) -> Option<StaticObjectIdentityValue> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id)?;
            if let Some(value) = scope.static_identity_values.get(name) {
                return Some(value.clone());
            }
            current = scope.parent;
        }

        self.global_scope.static_identity_values.get(name).cloned()
    }

    pub(crate) fn resolve_static_object_identity_reference_name(
        &self,
        expression: &Expression,
    ) -> Option<String> {
        self.resolve_static_reference_root(expression)
    }

    pub(crate) fn resolve_static_object_identity_literal_value(
        &self,
        expression: &Expression,
    ) -> Option<StaticObjectIdentityValue> {
        match expression {
            Expression::Literal(LiteralValue::Boolean(value)) => {
                Some(StaticObjectIdentityValue::Boolean(*value))
            }
            Expression::Literal(LiteralValue::Number(value)) => {
                Some(StaticObjectIdentityValue::Number(*value))
            }
            Expression::Literal(LiteralValue::String(value)) => {
                Some(StaticObjectIdentityValue::String(value.clone()))
            }
            Expression::TemplateLiteral(template) => self
                .resolve_static_string_expression(&Expression::TemplateLiteral(template.clone()))
                .map(StaticObjectIdentityValue::String),
            Expression::BigIntLiteral(value) => value
                .strip_suffix('n')
                .and_then(|value| value.parse::<i64>().ok())
                .map(StaticObjectIdentityValue::BigInt),
            Expression::Literal(LiteralValue::Null) => Some(StaticObjectIdentityValue::Null),
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.expression)
            }
            Expression::AwaitExpression(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.argument)
            }
            Expression::UnaryExpression(expr) if expr.operator == "+" => {
                match self.resolve_static_object_identity_literal_value(&expr.argument) {
                    Some(StaticObjectIdentityValue::BigInt(_)) => None,
                    other => other,
                }
            }
            Expression::UnaryExpression(expr) if expr.operator == "-" => self
                .resolve_static_object_identity_literal_value(&expr.argument)
                .and_then(|value| match value {
                    StaticObjectIdentityValue::Number(number) => {
                        Some(StaticObjectIdentityValue::Number(if number == 0.0 {
                            -0.0
                        } else {
                            -number
                        }))
                    }
                    StaticObjectIdentityValue::BigInt(value) => {
                        Some(StaticObjectIdentityValue::BigInt(-value))
                    }
                    _ => None,
                }),
            Expression::UnaryExpression(expr) if expr.operator == "void" => {
                Some(StaticObjectIdentityValue::Undefined)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_object_identity_literal_value(&expr.expression)
            }
            Expression::SequenceExpression(expr) => {
                expr.expressions.last().and_then(|expression| {
                    self.resolve_static_object_identity_literal_value(expression)
                })
            }
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_object_identity_literal_value(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_object_identity_literal_value(&expr.alternate)
                    }
                    _ => {
                        let consequent =
                            self.resolve_static_object_identity_literal_value(&expr.consequent);
                        let alternate =
                            self.resolve_static_object_identity_literal_value(&expr.alternate);
                        match (consequent, alternate) {
                            (Some(consequent), Some(alternate))
                                if consequent.same_value(&alternate) =>
                            {
                                Some(consequent)
                            }
                            _ => None,
                        }
                    }
                }
            }
            Expression::LogicalExpression(expr) => {
                let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
                match expr.operator {
                    LogicalOperator::Coalesce => {
                        if left.is_nullish() {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        } else {
                            Some(left)
                        }
                    }
                    LogicalOperator::And => match left.truthiness() {
                        Some(true) => {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        }
                        Some(false) => Some(left),
                        None => {
                            let right =
                                self.resolve_static_object_identity_literal_value(&expr.right)?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    },
                    LogicalOperator::Or => match left.truthiness() {
                        Some(true) => Some(left),
                        Some(false) => {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        }
                        None => {
                            let right =
                                self.resolve_static_object_identity_literal_value(&expr.right)?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    },
                }
            }
            Expression::BinaryExpression(expr)
                if matches!(expr.operator.as_str(), "??" | "&&" | "||") =>
            {
                let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
                let selected = match expr.operator.as_str() {
                    "??" => {
                        if left.is_nullish() {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        } else {
                            Some(left)
                        }
                    }
                    "&&" => match left.truthiness() {
                        Some(true) => {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        }
                        Some(false) => Some(left),
                        None => {
                            let right =
                                self.resolve_static_object_identity_literal_value(&expr.right)?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    },
                    "||" => match left.truthiness() {
                        Some(true) => Some(left),
                        Some(false) => {
                            self.resolve_static_object_identity_literal_value(&expr.right)
                        }
                        None => {
                            let right =
                                self.resolve_static_object_identity_literal_value(&expr.right)?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    },
                    _ => unreachable!(),
                };
                selected
            }
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => {
                call.args.first().and_then(|expression| {
                    self.resolve_static_object_identity_literal_value(expression)
                })
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    self.resolve_static_object_identity_literal_value(object)
                }
            },
            Expression::Identifier(name) => match name.as_str() {
                "Infinity" => Some(StaticObjectIdentityValue::Number(f64::INFINITY)),
                "NaN" => Some(StaticObjectIdentityValue::Number(f64::NAN)),
                "undefined" => Some(StaticObjectIdentityValue::Undefined),
                _ => self
                    .resolve_static_object_identity_binding(name)
                    .or_else(|| {
                        self.resolve_static_reference_binding_name(name)
                            .map(StaticObjectIdentityValue::Reference)
                    }),
            },
            _ => None,
        }
    }

    pub(crate) fn resolve_static_object_binding_name(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if scope.static_objects.contains_key(name) {
                return true;
            }
            current = scope.parent;
        }

        self.global_scope.static_objects.contains_key(name)
    }

    pub(crate) fn resolve_static_reference_binding_name(&self, name: &str) -> Option<String> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if let Some(root) = scope.static_reference_values.get(name) {
                return Some(root.clone());
            }
            current = scope.parent;
        }

        self.global_scope.static_reference_values.get(name).cloned()
    }

    pub(crate) fn resolve_static_reference_root(&self, expression: &Expression) -> Option<String> {
        match expression {
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_reference_root(&expr.expression)
            }
            Expression::AwaitExpression(expr) => self.resolve_static_reference_root(&expr.argument),
            Expression::TypeAssertion(expr) => self.resolve_static_reference_root(&expr.expression),
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_reference_root(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_reference_root(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_reference_root(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .and_then(|expression| self.resolve_static_reference_root(expression)),
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_reference_root(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_reference_root(&expr.alternate)
                    }
                    _ => {
                        let consequent = self.resolve_static_reference_root(&expr.consequent);
                        let alternate = self.resolve_static_reference_root(&expr.alternate);
                        match (consequent, alternate) {
                            (Some(consequent), Some(alternate)) if consequent == alternate => {
                                Some(consequent)
                            }
                            _ => None,
                        }
                    }
                }
            }
            Expression::LogicalExpression(expr) => {
                let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
                match expr.operator {
                    LogicalOperator::Coalesce => {
                        if left.is_nullish() {
                            self.resolve_static_reference_root(&expr.right)
                        } else {
                            self.resolve_static_reference_root(&expr.left)
                        }
                    }
                    LogicalOperator::And => match left.truthiness() {
                        Some(true) => self.resolve_static_reference_root(&expr.right),
                        Some(false) => self.resolve_static_reference_root(&expr.left),
                        None => {
                            let consequent = self.resolve_static_reference_root(&expr.left);
                            let alternate = self.resolve_static_reference_root(&expr.right);
                            match (consequent, alternate) {
                                (Some(consequent), Some(alternate)) if consequent == alternate => {
                                    Some(consequent)
                                }
                                _ => None,
                            }
                        }
                    },
                    LogicalOperator::Or => match left.truthiness() {
                        Some(true) => self.resolve_static_reference_root(&expr.left),
                        Some(false) => self.resolve_static_reference_root(&expr.right),
                        None => {
                            let consequent = self.resolve_static_reference_root(&expr.left);
                            let alternate = self.resolve_static_reference_root(&expr.right);
                            match (consequent, alternate) {
                                (Some(consequent), Some(alternate)) if consequent == alternate => {
                                    Some(consequent)
                                }
                                _ => None,
                            }
                        }
                    },
                }
            }
            Expression::BinaryExpression(expr)
                if matches!(expr.operator.as_str(), "??" | "&&" | "||") =>
            {
                let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
                let selected = match expr.operator.as_str() {
                    "??" => {
                        if left.is_nullish() {
                            self.resolve_static_reference_root(&expr.right)
                        } else {
                            self.resolve_static_reference_root(&expr.left)
                        }
                    }
                    "&&" => match left.truthiness() {
                        Some(true) => self.resolve_static_reference_root(&expr.right),
                        Some(false) => self.resolve_static_reference_root(&expr.left),
                        None => {
                            let consequent = self.resolve_static_reference_root(&expr.left);
                            let alternate = self.resolve_static_reference_root(&expr.right);
                            match (consequent, alternate) {
                                (Some(consequent), Some(alternate)) if consequent == alternate => {
                                    Some(consequent)
                                }
                                _ => None,
                            }
                        }
                    },
                    "||" => match left.truthiness() {
                        Some(true) => self.resolve_static_reference_root(&expr.left),
                        Some(false) => self.resolve_static_reference_root(&expr.right),
                        None => {
                            let consequent = self.resolve_static_reference_root(&expr.left);
                            let alternate = self.resolve_static_reference_root(&expr.right);
                            match (consequent, alternate) {
                                (Some(consequent), Some(alternate)) if consequent == alternate => {
                                    Some(consequent)
                                }
                                _ => None,
                            }
                        }
                    },
                    _ => unreachable!(),
                };
                selected
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    self.resolve_static_reference_root(object)
                }
            },
            Expression::MemberExpression(member) => Self::member_access_name(member),
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .and_then(|expression| self.resolve_static_reference_root(expression)),
            Expression::Identifier(name) => self
                .resolve_static_reference_binding_name(name)
                .or_else(|| {
                    self.resolve_static_object_binding_name(name)
                        .then(|| name.clone())
                })
                .or_else(|| {
                    self.resolve_static_array_binding_name(name)
                        .then(|| name.clone())
                })
                .or_else(|| matches!(name.as_str(), "Set" | "Map").then(|| name.clone())),
            _ => None,
        }
    }

    pub(crate) fn resolve_static_object_keys_binding_name(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if scope.static_object_keys.contains_key(name) {
                return true;
            }
            current = scope.parent;
        }

        self.global_scope.static_object_keys.contains_key(name)
    }

    pub(crate) fn resolve_static_object_model_target(&self, expression: &Expression) -> bool {
        match expression {
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .is_some_and(|expression| self.resolve_static_object_model_target(expression)),
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_object_model_target(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_object_model_target(&expr.alternate)
                    }
                    _ => {
                        self.resolve_static_object_model_target(&expr.consequent)
                            && self.resolve_static_object_model_target(&expr.alternate)
                    }
                }
            }
            Expression::AwaitExpression(expr) => {
                self.resolve_static_object_model_target(&expr.argument)
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                properties.iter().all(|property| {
                    matches!(property.kind, ObjectPropertyKind::Init)
                        && matches!(
                            property.key,
                            PropertyName::Identifier(_)
                                | PropertyName::Number(_)
                                | PropertyName::String(_)
                        )
                })
            }
            Expression::ArrayExpression(_) => true,
            Expression::CallExpression(call) => {
                self.resolve_static_object_from_entries_call(call)
                    || Self::is_object_freeze_call(call)
                        && call.args.first().is_some_and(|argument| {
                            self.resolve_static_object_model_target(argument)
                        })
            }
            Expression::Identifier(name) => {
                name != "globalThis" && self.resolve_static_object_binding_name(name)
            }
            _ => false,
        }
    }

    pub(crate) fn resolve_static_object_keys_target(&self, expression: &Expression) -> bool {
        match expression {
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_object_keys_target(&expr.expression)
            }
            Expression::AwaitExpression(expr) => {
                self.resolve_static_object_keys_target(&expr.argument)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_object_keys_target(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_object_keys_target(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_object_keys_target(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_object_keys_target(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .is_some_and(|expression| self.resolve_static_object_keys_target(expression)),
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_object_keys_target(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_object_keys_target(&expr.alternate)
                    }
                    _ => {
                        self.resolve_static_object_keys_target(&expr.consequent)
                            && self.resolve_static_object_keys_target(&expr.alternate)
                    }
                }
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                properties.iter().all(|property| {
                    matches!(
                        property.kind,
                        ObjectPropertyKind::Init
                            | ObjectPropertyKind::Get
                            | ObjectPropertyKind::Set
                    ) && matches!(
                        property.key,
                        PropertyName::Identifier(_)
                            | PropertyName::Number(_)
                            | PropertyName::String(_)
                    )
                })
            }
            Expression::CallExpression(call) => {
                self.resolve_static_object_from_entries_call(call)
                    || Self::is_object_freeze_call(call)
                        && call.args.first().is_some_and(|argument| {
                            self.resolve_static_object_keys_target(argument)
                        })
                    || Self::is_reflect_own_keys_call(call)
                        && call.args.first().is_some_and(|argument| {
                            self.resolve_static_object_keys_target(argument)
                        })
            }
            Expression::Identifier(name) => self.resolve_static_object_keys_binding_name(name),
            other => self
                .resolve_static_string_iterable_expression(other)
                .is_some(),
        }
    }

    pub(crate) fn is_object_freeze_call(call: &CallExpression) -> bool {
        matches!(
            Self::call_member_access_name(&call.callee).as_deref(),
            Some("Object.freeze")
                | Some("globalThis.Object.freeze")
                | Some(r#"globalThis["Object"].freeze"#)
                | Some(r#"globalThis["Object"]["freeze"]"#)
                | Some(r#"globalThis['Object'].freeze"#)
                | Some(r#"globalThis['Object']['freeze']"#)
                | Some(r#"Object["freeze"]"#)
                | Some(r#"Object['freeze']"#)
                | Some(r#"globalThis.Object["freeze"]"#)
                | Some(r#"globalThis.Object['freeze']"#)
        ) && call.args.len() == 1
    }

    pub(crate) fn is_reflect_own_keys_call(call: &CallExpression) -> bool {
        matches!(
            Self::call_member_access_name(&call.callee).as_deref(),
            Some("Reflect.ownKeys")
                | Some("Reflect[\"ownKeys\"]")
                | Some("Reflect['ownKeys']")
                | Some("globalThis.Reflect.ownKeys")
                | Some("globalThis.Reflect[\"ownKeys\"]")
                | Some("globalThis.Reflect['ownKeys']")
                | Some(r#"globalThis["Reflect"].ownKeys"#)
                | Some(r#"globalThis["Reflect"]["ownKeys"]"#)
                | Some(r#"globalThis["Reflect"]['ownKeys']"#)
                | Some(r#"globalThis['Reflect'].ownKeys"#)
                | Some(r#"globalThis['Reflect']['ownKeys']"#)
                | Some(r#"globalThis['Reflect']["ownKeys"]"#)
        ) && call.args.len() == 1
    }

    pub(crate) fn resolve_static_object_from_entries_call(&self, call: &CallExpression) -> bool {
        let Some(callee_name) = self.resolve_static_callable_name(&call.callee) else {
            return false;
        };

        if !matches!(
            callee_name.as_str(),
            "Object.fromEntries"
                | "globalThis.Object.fromEntries"
                | r#"globalThis["Object"].fromEntries"#
                | r#"globalThis["Object"]["fromEntries"]"#
                | r#"globalThis['Object'].fromEntries"#
                | r#"globalThis['Object']['fromEntries']"#
                | r#"Object["fromEntries"]"#
                | r#"Object['fromEntries']"#
                | r#"globalThis.Object["fromEntries"]"#
                | r#"globalThis.Object['fromEntries']"#
        ) {
            return false;
        }

        let Some(entries) = call.args.first() else {
            return false;
        };
        if call.args.len() != 1 {
            return false;
        }

        self.resolve_static_from_entries_entries(entries)
    }

    pub(crate) fn resolve_static_from_entries_entries(&self, expression: &Expression) -> bool {
        match expression {
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_from_entries_entries(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_from_entries_entries(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_from_entries_entries(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_from_entries_entries(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_from_entries_entries(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .is_some_and(|expression| self.resolve_static_from_entries_entries(expression)),
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_from_entries_entries(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_from_entries_entries(&expr.alternate)
                    }
                    _ => {
                        self.resolve_static_from_entries_entries(&expr.consequent)
                            && self.resolve_static_from_entries_entries(&expr.alternate)
                    }
                }
            }
            Expression::ArrayExpression(entries) => entries.elements.iter().all(|entry| {
                let Some(ExpressionOrSpread::Expression(Expression::ArrayExpression(pair))) = entry
                else {
                    return false;
                };

                if pair.elements.len() != 2 {
                    return false;
                }

                let Some(ExpressionOrSpread::Expression(key)) =
                    pair.elements.first().and_then(|element| element.as_ref())
                else {
                    return false;
                };
                let Some(ExpressionOrSpread::Expression(value)) =
                    pair.elements.get(1).and_then(|element| element.as_ref())
                else {
                    return false;
                };

                self.resolve_static_object_identity_literal_value(key)
                    .is_some()
                    && self
                        .resolve_static_object_identity_literal_value(value)
                        .is_some()
            }),
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .is_some_and(|expression| self.resolve_static_from_entries_entries(expression)),
            _ => false,
        }
    }

    pub(crate) fn resolve_frozen_late_object_model_call(&mut self, expr: &CallExpression) -> bool {
        if !Self::is_object_freeze_call(expr) {
            return false;
        }

        let Some(argument) = expr.args.first() else {
            return false;
        };
        let Some((dotted, bracketed)) = self.resolve_frozen_late_object_model_name(argument) else {
            return false;
        };

        if !matches!(
            dotted.as_str(),
            "Proxy.revocable" | "globalThis.Proxy.revocable"
        ) {
            return false;
        }

        let single_quoted = bracketed.replace("[\"", "['").replace("\"]", "']");
        let single_quoted_root_dotted = Self::member_access_single_quoted_root_name(argument)
            .map(|root| {
                format!(
                    "{}.{}",
                    root,
                    dotted
                        .rsplit_once('.')
                        .map(|(_, leaf)| leaf)
                        .unwrap_or(dotted.as_str())
                )
            })
            .unwrap_or_else(|| single_quoted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "late object-model API '{}' (aka {}, {}, {}) is unavailable until the later object-model compatibility path is enabled",
                dotted, bracketed, single_quoted, single_quoted_root_dotted
            ),
        ));
        true
    }

    pub(crate) fn resolve_frozen_late_object_model_name(
        &self,
        expression: &Expression,
    ) -> Option<(String, String)> {
        let mut current = expression;
        loop {
            match current {
                Expression::ParenthesizedExpression(expr) => current = &expr.expression,
                Expression::TypeAssertion(expr) => current = &expr.expression,
                Expression::SatisfiesExpression(expr) => current = &expr.expression,
                Expression::ChainExpression(expr) => current = &expr.expression,
                Expression::DecoratedExpression(expr) => current = &expr.expression,
                Expression::SequenceExpression(expr) => {
                    let last = expr.expressions.last()?;
                    current = last;
                }
                Expression::AwaitExpression(expr) => current = &expr.argument,
                Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                    OptionalChainInner::NonNull { object, .. } => current = object,
                },
                Expression::MemberExpression(member) => {
                    let dotted = Self::member_access_name(member)?;
                    let bracketed = Self::member_access_name_bracketed(member)?;
                    return Some((dotted, bracketed));
                }
                _ => return None,
            }
        }
    }

    pub(crate) fn resolve_static_object_model_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return false;
        };

        let is_object_has_own = matches!(
            callee_name.as_str(),
            "Object.hasOwn" | "globalThis.Object.hasOwn"
        );
        let is_has_own_property_call = matches!(
            callee_name.as_str(),
            "Object.prototype.hasOwnProperty.call"
                | "globalThis.Object.prototype.hasOwnProperty.call"
                | "Object.hasOwnProperty.call"
                | "globalThis[\"Object\"].prototype.hasOwnProperty[\"call\"]"
                | "globalThis.Object.hasOwnProperty.call"
        );
        if !is_object_has_own && !is_has_own_property_call {
            return false;
        }

        let Some(object_arg) = expr.args.first() else {
            return false;
        };
        let Some(key_arg) = expr.args.get(1) else {
            return false;
        };

        if !self.resolve_static_object_model_target(object_arg) {
            return false;
        }
        if matches!(object_arg, Expression::Identifier(name) if name == "globalThis") {
            return false;
        }
        if self.resolve_static_string_expression(key_arg).is_none() {
            return false;
        }

        self.resolve_expression(object_arg);
        self.resolve_expression(key_arg);
        for arg in expr.args.iter().skip(2) {
            self.resolve_expression(arg);
        }
        true
    }

    pub(crate) fn resolve_static_object_identity_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return false;
        };

        if !matches!(
            callee_name.as_str(),
            "Object.is"
                | "globalThis.Object.is"
                | r#"globalThis["Object"].is"#
                | r#"globalThis["Object"]["is"]"#
                | r#"globalThis['Object'].is"#
                | r#"globalThis['Object']['is']"#
                | r#"Object["is"]"#
                | r#"Object['is']"#
                | r#"globalThis.Object["is"]"#
                | r#"globalThis.Object['is']"#
        ) {
            return false;
        }

        let Some(left) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Object.is requires at least two statically-known primitive literal arguments or the same statically-known reference in the current phase; use explicit constants or the later compatibility path",
            ));
            return true;
        };
        let Some(right) = expr.args.get(1) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Object.is requires at least two statically-known primitive literal arguments in the current phase; use explicit constants or the later compatibility path",
            ));
            return true;
        };

        let left_reference = self.resolve_static_object_identity_reference_name(left);
        let right_reference = self.resolve_static_object_identity_reference_name(right);
        if let (Some(left_reference), Some(right_reference)) = (left_reference, right_reference) {
            if left_reference == right_reference {
                self.resolve_expression(left);
                self.resolve_expression(right);
                for arg in expr.args.iter().skip(2) {
                    self.resolve_expression(arg);
                }
                return true;
            }
        }

        let left_value = self.resolve_static_object_identity_literal_value(left);
        let right_value = self.resolve_static_object_identity_literal_value(right);
        if let (Some(left_value), Some(right_value)) = (left_value, right_value) {
            let _ = left_value.same_value(&right_value);
            self.resolve_expression(left);
            self.resolve_expression(right);
            for arg in expr.args.iter().skip(2) {
                self.resolve_expression(arg);
            }
            return true;
        }

        if self.resolve_static_object_model_target(left)
            || self.resolve_static_object_model_target(right)
        {
            self.resolve_expression(left);
            self.resolve_expression(right);
            for arg in expr.args.iter().skip(2) {
                self.resolve_expression(arg);
            }
            return true;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "Object.is is unavailable unless both arguments are statically-known primitive literals or the same statically-known reference in the current phase; use explicit constants or the later compatibility path",
        ));
        true
    }

    pub(crate) fn resolve_static_object_model_call_target(&self, call: &CallExpression) -> bool {
        self.resolve_static_object_from_entries_call(call)
    }
}

#[cfg(test)]
#[path = "object_tests.rs"]
mod object_tests;
