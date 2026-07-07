//! Call-expression resolution.
use crate::*;

impl TypeContext {
    pub(crate) fn resolve_call_expression(&mut self, expr: &CallExpression) {
        if let Expression::SequenceExpression(sequence) = &expr.callee {
            if sequence.expressions.len() > 1 {
                for callee_expression in sequence
                    .expressions
                    .iter()
                    .take(sequence.expressions.len() - 1)
                {
                    self.resolve_expression(callee_expression);
                }
            }
        }

        if self.resolve_static_object_model_call(expr) {
            return;
        }

        if self.resolve_frozen_late_object_model_call(expr) {
            return;
        }

        if self.resolve_static_object_identity_call(expr) {
            return;
        }
        if self.resolve_number_identity_call(expr) {
            return;
        }
        if self.resolve_global_number_predicate_call(expr) {
            return;
        }
        if self.resolve_number_parse_int_call(expr) {
            return;
        }
        if self.resolve_number_parse_float_call(expr) {
            return;
        }
        if self.resolve_string_from_char_code_call(expr) {
            return;
        }
        if self.resolve_array_is_array_call(expr) {
            return;
        }

        self.resolve_expression(&expr.callee);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }
        // Spec 4a Task 5 fail-closed: a for-in-key VALUE passed as ANY call
        // argument (a general user/method call `id(c)`, or `console.log(c)`) is
        // a value escape — a non-materializable one (an aliased key, or a direct
        // key passed to a non-console call that is not a repr seed sink) would
        // leak the raw ordinal across the call boundary. A direct SEEDED key
        // (`console.log(c)`, repr `String`) is materialized and not rejected.
        // Call arguments are always value positions (never an index/truthiness/
        // alias-copy), so rejecting here never touches the ordinal-domain lanes.
        for arg in &expr.args {
            self.reject_nonmaterializable_forin_key_value(arg);
        }
        self.resolve_permission_query_call(expr);
        self.resolve_process_kill_call(expr);
        self.resolve_math_member_call(expr);
        self.resolve_array_callback_member_call(expr);
        self.resolve_array_search_member_call(expr);
        self.resolve_array_slice_member_call(expr);
        self.resolve_array_concat_member_call(expr);
        self.resolve_array_at_member_call(expr);
        self.resolve_array_join_member_call(expr);
        self.resolve_array_to_string_member_call(expr);
        self.resolve_array_fill_runtime_string(expr);
        self.resolve_string_search_member_call(expr);
        self.resolve_string_slice_member_call(expr);
        self.resolve_string_substring_member_call(expr);
        self.resolve_string_repeat_member_call(expr);
        self.resolve_string_concat_member_call(expr);
        self.resolve_string_pad_member_call(expr);
        self.resolve_string_at_member_call(expr);
        self.resolve_string_char_at_member_call(expr);
        self.resolve_string_char_code_at_member_call(expr);
        self.resolve_string_code_point_at_member_call(expr);
        self.resolve_string_trim_member_call(expr);
        self.resolve_string_case_member_call(expr);
        self.resolve_string_normalize_member_call(expr);
        self.resolve_string_replace_member_call(expr);
        self.resolve_string_split_member_call(expr);
        self.resolve_promise_member_call(expr);
    }

    pub(crate) fn call_member_access_name(expression: &Expression) -> Option<String> {
        match expression {
            Expression::MemberExpression(member) => Self::member_access_name(member),
            Expression::ParenthesizedExpression(expr) => {
                Self::call_member_access_name(&expr.expression)
            }
            Expression::TypeAssertion(expr) => Self::call_member_access_name(&expr.expression),
            Expression::SatisfiesExpression(expr) => {
                Self::call_member_access_name(&expr.expression)
            }
            Expression::ChainExpression(expr) => Self::call_member_access_name(&expr.expression),
            Expression::DecoratedExpression(expr) => {
                Self::call_member_access_name(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .and_then(Self::call_member_access_name),
            Expression::AwaitExpression(expr) => Self::call_member_access_name(&expr.argument),
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => Self::call_member_access_name(object),
            },
            _ => None,
        }
    }

    pub(crate) fn unwrap_static_callable_expression(expression: &Expression) -> &Expression {
        match expression {
            Expression::ParenthesizedExpression(expr) => {
                Self::unwrap_static_callable_expression(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                Self::unwrap_static_callable_expression(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                Self::unwrap_static_callable_expression(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                Self::unwrap_static_callable_expression(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                Self::unwrap_static_callable_expression(&expr.expression)
            }
            Expression::AwaitExpression(expr) => {
                Self::unwrap_static_callable_expression(&expr.argument)
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    Self::unwrap_static_callable_expression(object)
                }
            },
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .map(Self::unwrap_static_callable_expression)
                .unwrap_or(expression),
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .map(Self::unwrap_static_callable_expression)
                .unwrap_or(expression),
            _ => expression,
        }
    }

    pub(crate) fn resolve_static_callable_name(&self, expression: &Expression) -> Option<String> {
        let expression = Self::unwrap_static_callable_expression(expression);
        if let Expression::ConditionalExpression(expr) = expression {
            match self.resolve_static_object_identity_literal_value(&expr.test) {
                Some(StaticObjectIdentityValue::Boolean(true)) => {
                    return self.resolve_static_callable_name(&expr.consequent);
                }
                Some(StaticObjectIdentityValue::Boolean(false)) => {
                    return self.resolve_static_callable_name(&expr.alternate);
                }
                _ => {
                    let consequent = Self::unwrap_static_callable_expression(&expr.consequent);
                    let alternate = Self::unwrap_static_callable_expression(&expr.alternate);
                    let consequent_name = Self::call_member_access_name(consequent)
                        .or_else(|| self.resolve_static_reference_root(consequent));
                    let alternate_name = Self::call_member_access_name(alternate)
                        .or_else(|| self.resolve_static_reference_root(alternate));
                    if consequent_name.is_some() && consequent_name == alternate_name {
                        return consequent_name;
                    }
                }
            }
        }

        if let Expression::LogicalExpression(expr) = expression {
            let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
            let selected = match expr.operator {
                LogicalOperator::Coalesce => {
                    if left.is_nullish() {
                        self.resolve_static_callable_name(&expr.right)
                    } else {
                        self.resolve_static_callable_name(&expr.left)
                    }
                }
                LogicalOperator::And => match left.truthiness() {
                    Some(true) => self.resolve_static_callable_name(&expr.right),
                    Some(false) => self.resolve_static_callable_name(&expr.left),
                    None => {
                        let consequent = Self::unwrap_static_callable_expression(&expr.left);
                        let alternate = Self::unwrap_static_callable_expression(&expr.right);
                        let consequent_name = Self::call_member_access_name(consequent)
                            .or_else(|| self.resolve_static_reference_root(consequent));
                        let alternate_name = Self::call_member_access_name(alternate)
                            .or_else(|| self.resolve_static_reference_root(alternate));
                        if consequent_name.is_some() && consequent_name == alternate_name {
                            consequent_name
                        } else {
                            None
                        }
                    }
                },
                LogicalOperator::Or => match left.truthiness() {
                    Some(true) => self.resolve_static_callable_name(&expr.left),
                    Some(false) => self.resolve_static_callable_name(&expr.right),
                    None => {
                        let consequent = Self::unwrap_static_callable_expression(&expr.left);
                        let alternate = Self::unwrap_static_callable_expression(&expr.right);
                        let consequent_name = Self::call_member_access_name(consequent)
                            .or_else(|| self.resolve_static_reference_root(consequent));
                        let alternate_name = Self::call_member_access_name(alternate)
                            .or_else(|| self.resolve_static_reference_root(alternate));
                        if consequent_name.is_some() && consequent_name == alternate_name {
                            consequent_name
                        } else {
                            None
                        }
                    }
                },
            };
            if selected.is_some() {
                return selected;
            }
        }

        match expression {
            Expression::BinaryExpression(expr)
                if matches!(expr.operator.as_str(), "??" | "&&" | "||") =>
            {
                let left = self.resolve_static_object_identity_literal_value(&expr.left)?;
                let selected = match expr.operator.as_str() {
                    "??" => {
                        if left.is_nullish() {
                            self.resolve_static_callable_name(&expr.right)
                        } else {
                            self.resolve_static_callable_name(&expr.left)
                        }
                    }
                    "&&" => match left.truthiness() {
                        Some(true) => self.resolve_static_callable_name(&expr.right),
                        Some(false) => self.resolve_static_callable_name(&expr.left),
                        None => {
                            let consequent = Self::unwrap_static_callable_expression(&expr.left);
                            let alternate = Self::unwrap_static_callable_expression(&expr.right);
                            let consequent_name = Self::call_member_access_name(consequent)
                                .or_else(|| self.resolve_static_reference_root(consequent));
                            let alternate_name = Self::call_member_access_name(alternate)
                                .or_else(|| self.resolve_static_reference_root(alternate));
                            if consequent_name.is_some() && consequent_name == alternate_name {
                                consequent_name
                            } else {
                                None
                            }
                        }
                    },
                    "||" => match left.truthiness() {
                        Some(true) => self.resolve_static_callable_name(&expr.left),
                        Some(false) => self.resolve_static_callable_name(&expr.right),
                        None => {
                            let consequent = Self::unwrap_static_callable_expression(&expr.left);
                            let alternate = Self::unwrap_static_callable_expression(&expr.right);
                            let consequent_name = Self::call_member_access_name(consequent)
                                .or_else(|| self.resolve_static_reference_root(consequent));
                            let alternate_name = Self::call_member_access_name(alternate)
                                .or_else(|| self.resolve_static_reference_root(alternate));
                            if consequent_name.is_some() && consequent_name == alternate_name {
                                consequent_name
                            } else {
                                None
                            }
                        }
                    },
                    _ => unreachable!(),
                };
                if selected.is_some() {
                    return selected;
                }
            }
            _ => {}
        }

        Self::call_member_access_name(expression)
            .or_else(|| self.resolve_static_reference_root(expression))
    }

    pub(crate) fn contains_optional_chain(expression: &Expression) -> bool {
        match expression {
            Expression::OptionalChainExpression(_) => true,
            Expression::ParenthesizedExpression(expr) => {
                Self::contains_optional_chain(&expr.expression)
            }
            Expression::TypeAssertion(expr) => Self::contains_optional_chain(&expr.expression),
            Expression::SatisfiesExpression(expr) => {
                Self::contains_optional_chain(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                Self::contains_optional_chain(&expr.expression)
            }
            Expression::ChainExpression(expr) => Self::contains_optional_chain(&expr.expression),
            Expression::AwaitExpression(expr) => Self::contains_optional_chain(&expr.argument),
            Expression::SequenceExpression(expr) => {
                expr.expressions.iter().any(Self::contains_optional_chain)
            }
            Expression::CallExpression(call) => {
                Self::contains_optional_chain(&call.callee)
                    || call.args.iter().any(Self::contains_optional_chain)
            }
            Expression::MemberExpression(member) => Self::contains_optional_chain(&member.object),
            Expression::BinaryExpression(expr) => {
                Self::contains_optional_chain(&expr.left)
                    || Self::contains_optional_chain(&expr.right)
            }
            Expression::LogicalExpression(expr) => {
                Self::contains_optional_chain(&expr.left)
                    || Self::contains_optional_chain(&expr.right)
            }
            Expression::ConditionalExpression(expr) => {
                Self::contains_optional_chain(&expr.test)
                    || Self::contains_optional_chain(&expr.consequent)
                    || Self::contains_optional_chain(&expr.alternate)
            }
            _ => false,
        }
    }

    pub(crate) fn is_supported_static_callable_member_expression(
        &self,
        expr: &MemberExpression,
    ) -> bool {
        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());
        self.is_supported_static_callable_member_name(&dotted, &bracketed)
    }

    pub(crate) fn is_supported_static_callable_member_name(
        &self,
        dotted: &str,
        bracketed: &str,
    ) -> bool {
        matches!(
            dotted,
            "Object.is"
                | "globalThis.Object.is"
                | "Object.hasOwn"
                | "globalThis.Object.hasOwn"
                | "Object.prototype.hasOwnProperty.call"
                | "globalThis.Object.prototype.hasOwnProperty.call"
                | "Object.hasOwnProperty.call"
                | "globalThis[\"Object\"].prototype.hasOwnProperty[\"call\"]"
                | "globalThis.Object.hasOwnProperty.call"
                | "Number.isFinite"
                | "Number.isNaN"
                | "Number.isInteger"
                | "Number.isSafeInteger"
                | "Array.isArray"
                | "globalThis.Array.isArray"
                | "String.fromCharCode"
                | "globalThis.String.fromCharCode"
                | "String.fromCodePoint"
                | "globalThis.String.fromCodePoint"
                | "globalThis.Number.isFinite"
                | "globalThis.Number.isNaN"
                | "globalThis.Number.isInteger"
                | "globalThis.Number.isSafeInteger"
                | "Promise.all"
                | "Promise.allSettled"
                | "Promise.any"
                | "Promise.race"
                | "globalThis.Promise.all"
                | "globalThis.Promise.allSettled"
                | "globalThis.Promise.any"
                | "globalThis.Promise.race"
                | "Promise[\"all\"]"
                | "Promise[\"allSettled\"]"
                | "Promise[\"any\"]"
                | "Promise[\"race\"]"
                | "Promise['all']"
                | "Promise['allSettled']"
                | "Promise['any']"
                | "Promise['race']"
                | "globalThis.Promise[\"all\"]"
                | "globalThis.Promise[\"allSettled\"]"
                | "globalThis.Promise[\"any\"]"
                | "globalThis.Promise[\"race\"]"
                | "globalThis.Promise['all']"
                | "globalThis.Promise['allSettled']"
                | "globalThis.Promise['any']"
                | "globalThis.Promise['race']"
                | r#"globalThis["Promise"].all"#
                | r#"globalThis["Promise"].allSettled"#
                | r#"globalThis["Promise"].any"#
                | r#"globalThis["Promise"].race"#
                | r#"globalThis["Promise"]["all"]"#
                | r#"globalThis["Promise"]["allSettled"]"#
                | r#"globalThis["Promise"]["any"]"#
                | r#"globalThis["Promise"]["race"]"#
                | r#"globalThis["Promise"]['all']"#
                | r#"globalThis["Promise"]['allSettled']"#
                | r#"globalThis["Promise"]['any']"#
                | r#"globalThis["Promise"]['race']"#
                | r#"globalThis['Promise'].all"#
                | r#"globalThis['Promise'].allSettled"#
                | r#"globalThis['Promise'].any"#
                | r#"globalThis['Promise'].race"#
                | r#"globalThis['Promise']['all']"#
                | r#"globalThis['Promise']['allSettled']"#
                | r#"globalThis['Promise']['any']"#
                | r#"globalThis['Promise']['race']"#
                | r#"globalThis['Promise']["all"]"#
                | r#"globalThis['Promise']["allSettled"]"#
                | r#"globalThis['Promise']["any"]"#
                | r#"globalThis['Promise']["race"]"#
        ) || matches!(
            bracketed,
            r#"globalThis["Object"].is"#
                | r#"globalThis["Object"]["is"]"#
                | r#"Object["is"]"#
                | r#"globalThis.Object["is"]"#
                | r#"globalThis["Object"]["hasOwn"]"#
                | r#"globalThis.Object["hasOwn"]"#
                | r#"Object["hasOwn"]"#
                | r#"globalThis["Object"].hasOwn"#
                | r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#
                | r#"globalThis["Object"]["prototype"].hasOwnProperty["call"]"#
                | r#"globalThis["Object"].prototype["hasOwnProperty"]["call"]"#
                | r#"globalThis["Object"].prototype.hasOwnProperty.call"#
                | r#"globalThis.Object["prototype"]["hasOwnProperty"]["call"]"#
                | r#"globalThis.Object.prototype["hasOwnProperty"]["call"]"#
                | r#"globalThis.Object.prototype.hasOwnProperty.call"#
                | r#"globalThis.Object["hasOwnProperty"].call"#
                | r#"globalThis.Object["hasOwnProperty"]["call"]"#
                | r#"globalThis["Object"]["hasOwnProperty"].call"#
                | r#"globalThis["Object"]["hasOwnProperty"]["call"]"#
                | r#"globalThis["Object"].hasOwnProperty.call"#
                | r#"globalThis.Object.hasOwnProperty.call"#
                | r#"Object["hasOwnProperty"].call"#
                | r#"Object["hasOwnProperty"]["call"]"#
                | r#"Object.prototype.hasOwnProperty.call"#
                | r#"globalThis["Number"].isFinite"#
                | r#"globalThis["Number"].isNaN"#
                | r#"globalThis["Number"].isInteger"#
                | r#"globalThis["Number"].isSafeInteger"#
                | r#"globalThis.Number["isFinite"]"#
                | r#"globalThis.Number["isNaN"]"#
                | r#"globalThis.Number["isInteger"]"#
                | r#"globalThis.Number["isSafeInteger"]"#
                | r#"globalThis["Number"]["isFinite"]"#
                | r#"globalThis["Number"]["isNaN"]"#
                | r#"globalThis["Number"]["isInteger"]"#
                | r#"globalThis["Number"]["isSafeInteger"]"#
                | r#"globalThis["Array"].isArray"#
                | r#"globalThis['Array'].isArray"#
                | r#"globalThis.Array["isArray"]"#
                | r#"globalThis.Array['isArray']"#
                | r#"globalThis["Array"]["isArray"]"#
                | r#"globalThis['Array']['isArray']"#
                | r#"Array["isArray"]"#
                | r#"Array['isArray']"#
                | r#"globalThis["String"].fromCharCode"#
                | r#"globalThis['String'].fromCharCode"#
                | r#"globalThis.String["fromCharCode"]"#
                | r#"globalThis.String['fromCharCode']"#
                | r#"globalThis["String"]["fromCharCode"]"#
                | r#"globalThis['String']['fromCharCode']"#
                | r#"String["fromCharCode"]"#
                | r#"String['fromCharCode']"#
                | r#"globalThis["String"].fromCodePoint"#
                | r#"globalThis['String'].fromCodePoint"#
                | r#"globalThis.String["fromCodePoint"]"#
                | r#"globalThis.String['fromCodePoint']"#
                | r#"globalThis["String"]["fromCodePoint"]"#
                | r#"globalThis['String']['fromCodePoint']"#
                | r#"String["fromCodePoint"]"#
                | r#"String['fromCodePoint']"#
        )
    }
}

#[cfg(test)]
#[path = "call_tests.rs"]
mod call_tests;
