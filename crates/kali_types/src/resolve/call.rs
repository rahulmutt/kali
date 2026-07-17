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
        self.reject_anonymous_function_argument(expr);
        // Spec 4a Task 5: a for-in-key VALUE passed as a call argument is a value
        // escape — rejected structurally by the default-deny in `resolve_identifier`
        // (call arguments are resolved as expressions, so a non-materialized key
        // arrives there and rejects; a materialized direct seeded key does not).
        self.resolve_permission_query_call(expr);
        self.resolve_process_kill_call(expr);
        self.resolve_performance_now_call(expr);
        self.resolve_crypto_call(expr);
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

    /// Fail-closed anonymous-function-argument gate.
    ///
    /// An anonymous function expression (`function () { … }`, i.e. `id == None`)
    /// or an arrow (`() => …`, `x => { … }`) passed as a call argument compiles
    /// to a real standalone wasm function — but the ONLY way an in-wasm call can
    /// reach a callback is monomorphized dispatch keyed on the callback's
    /// function NAME (a named function passed as a param works this way). An
    /// anonymous function has no name to key on, so invoking such a param
    /// (`cb(5)`) silently no-ops (verified). Reject those args so the limitation
    /// is a clean diagnostic instead of silent wrong behavior.
    ///
    /// SCOPE: reject only when the callee is a plain identifier that is NOT a
    /// builtin global — i.e. a user-defined function, the exact shape that goes
    /// through the name-keyed dispatch lane. Everything that legitimately
    /// consumes an anonymous callback is deliberately exempt:
    ///   * `Kali.test(name, cb)` — a MEMBER call; its callback is invoked BY THE
    ///     HOST via the `__kali_callback_<index>` export, never an in-wasm call.
    ///   * array callbacks `arr.map/filter/find/some/every/reduce/…` — MEMBER
    ///     calls, statically folded (the callback is never lowered as a real
    ///     function).
    ///   * promise `p.then/catch/finally` — MEMBER calls, runtime-driven.
    ///   * the builtin identifier scheduling consumers `queueMicrotask`,
    ///     `setTimeout`, `setInterval` — all three WIRED as of Stage D Tasks
    ///     4–5: codegen emits the `queue_microtask` / `set_timeout` /
    ///     `set_interval` registration and the runtime drains the microtask
    ///     FIFO / virtual-clock timer queue after `_start`, invoking each
    ///     callback's `__kali_callback_<index>` export
    ///     (`kali_runtime::host::enforce`). They take the generic builtin
    ///     exemption below like any other bound global — the codegen-side
    ///     provenance resolver (`scheduling_callback`) and `env_safety` own
    ///     the fail-closed decisions for an unresolvable or unsound callback,
    ///     so this gate does NOT need to force-reject them.
    ///
    /// Placed AFTER callee + argument resolution so a callee that independently
    /// rejects (e.g. an unsupported late-object-model global such as
    /// `FinalizationRegistry(() => {})`) surfaces its own diagnostic FIRST.
    pub(crate) fn reject_anonymous_function_argument(&mut self, expr: &CallExpression) {
        let Expression::Identifier(callee_name) = &expr.callee else {
            return;
        };
        // Only a callee BOUND to a user-defined function reaches the name-keyed
        // dispatch lane. Skip when the identifier is:
        //   * unbound — a typo, or a recognized-but-unsupported global such as
        //     `FinalizationRegistry` that rejects via its OWN late-object-model
        //     diagnostic; a second diagnostic here would double-count the error
        //     (and it is not a silent-no-op miscompile — the program already
        //     rejects), so leave its own error to stand.
        //   * a builtin global that legitimately invokes its callback — the
        //     three scheduling surfaces (`queueMicrotask`/`setTimeout`/
        //     `setInterval`) are bound but invoke the callback through the
        //     runtime, and the codegen provenance resolver + `env_safety` fail
        //     closed on any unsound callback (see the doc comment above).
        if self.resolve_name(callee_name).is_none() {
            return;
        }
        // I-2: the builtin exemption applies ONLY to an UNSHADOWED builtin. A
        // user binding of the same name (`let queueMicrotask = function(f){…}`)
        // shadows the builtin — the call takes the normal user-call lane, and
        // its anonymous-function argument must fall into the generic rejection
        // below, not the builtin exemption. Skipping the shadow check made a
        // shadowed scheduling-surface call a TOTAL silent no-op (no deny
        // anywhere; the shadow body never ran). A user shadow resolves to a
        // binding on the active scope chain (`resolves_to_user_binding`);
        // genuine builtins live only in `global_scope`.
        if Self::is_builtin_global_name(callee_name) && !self.resolves_to_user_binding(callee_name)
        {
            return;
        }
        for arg in &expr.args {
            let anonymous_fn = match arg {
                // Task 2's pre-pass (`name_anon_functions.rs`) runs BEFORE the
                // resolver and fills every anonymous `id: None` with a
                // synthetic `__kali_fn_{N}` name — so by the time this gate
                // runs, `func.id` is never actually `None` for the anonymous
                // case anymore. Detect "was anonymous in source" via the
                // synthetic-name marker instead (the collision guard in that
                // pass guarantees no SOURCE declaration is ever named
                // `__kali_fn_{N}`, so the check cannot mis-fire on a real
                // user-named function expression). `func.id.is_none()` is kept
                // as a defensive fallback in case this arg ever reaches here
                // before the pre-pass runs.
                Expression::FunctionExpression(func) => func
                    .id
                    .as_deref()
                    .is_none_or(Self::is_synthetic_function_name),
                // An arrow has no source-level named-function-expression
                // syntax at all — every arrow is anonymous in source, whether
                // its `id` is `None` (pre-pass didn't run) or a pre-pass-
                // assigned `__kali_fn_{N}` (the normal post-pre-pass case).
                Expression::ArrowFunctionExpression(_) => true,
                _ => false,
            };
            if anonymous_fn {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "an anonymous function as a call argument is unavailable in the current \
                     phase (nothing can invoke it); declare a named function and pass its name"
                        .to_string(),
                ));
            }
        }
    }

    /// Whether `name` is a builtin global (base surface or the node surface).
    /// Checked as a static superset — a name that is a builtin under EITHER
    /// surface is treated as builtin — so the anonymous-argument gate never
    /// mis-rejects a callback handed to a runtime consumer that only exists on
    /// one surface.
    fn is_builtin_global_name(name: &str) -> bool {
        builtin_globals().contains(&name) || node_builtin_globals().contains(&name)
    }

    /// Whether `name` is a Task 2 pre-pass synthetic name (`__kali_fn_{N}`,
    /// `name_anon_functions.rs`) — i.e. the node was anonymous in SOURCE. The
    /// pre-pass's collision guard (`collect_taken_names`) guarantees no
    /// source declaration is ever assigned this shape, so the check is exact
    /// in both directions: every synthetic name matches, and no real
    /// user-declared name ever does.
    fn is_synthetic_function_name(name: &str) -> bool {
        name.starts_with("__kali_fn_")
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

    /// Resolve a short-circuit (`??`/`&&`/`||`) or ternary (`?`) callable
    /// SELECTION to the identifier it statically selects. This rule is mirrored
    /// in `kali_optimize::helpers::Optimizer::callable_selection_member_access_name`
    /// — keep them synchronized.
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
