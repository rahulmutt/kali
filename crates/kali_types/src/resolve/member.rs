//! Member-expression resolution.
use crate::*;
use kali_common::js_number::format_js_number;

impl TypeContext {
    pub(crate) fn resolve_member_expression(&mut self, expr: &MemberExpression) {
        self.reject_unprovable_string_length(expr);
        self.reject_nonuniform_forin_key_object_access(expr);

        if !self.gate_static_string_receiver_index(expr) {
            self.gate_nameless_computed_member(expr);
        }

        if self.resolve_late_intl_member(expr) {
            return;
        }

        if self.resolve_late_object_model_member(expr) {
            return;
        }

        if self.resolve_deno_args_member(expr) {
            return;
        }

        if self.resolve_late_env_object_member(expr) {
            return;
        }

        if self.resolve_late_env_mutation_member(expr) {
            return;
        }

        if self.is_supported_static_callable_member_expression(expr) {
            return;
        }

        self.resolve_expression(&expr.object);
        self.resolve_threaded_runtime_member(expr);
        self.resolve_late_host_control_member(expr);
        if self.resolve_late_subprocess_member(expr) {
            return;
        }
        if self.resolve_late_network_member(expr) {
            return;
        }
        self.resolve_late_permission_escalation_member(expr);
    }

    pub(crate) fn member_access_name(expr: &MemberExpression) -> Option<String> {
        let object_name = Self::member_access_root_name(&expr.object)?;

        Some(format!("{}.{}", object_name, expr.static_name()?))
    }

    pub(crate) fn is_runtime_args_slice_member(expr: &MemberExpression) -> bool {
        if expr.static_name() != Some("slice") {
            return false;
        }

        matches!(
            &expr.object,
            Expression::MemberExpression(object)
                if matches!(Self::member_access_name(object).as_deref(), Some("process.argv" | "Deno.args"))
        )
    }

    pub(crate) fn member_access_name_bracketed(expr: &MemberExpression) -> Option<String> {
        let object_name = Self::member_access_bracketed_root_name(&expr.object)?;

        Some(format!("{}[\"{}\"]", object_name, expr.static_name()?))
    }

    pub(crate) fn member_access_name_single_quoted(expr: &MemberExpression) -> Option<String> {
        let object_name = Self::member_access_single_quoted_root_name(&expr.object)?;

        Some(format!("{}['{}']", object_name, expr.static_name()?))
    }

    pub(crate) fn member_access_single_quoted_root_name(object: &Expression) -> Option<String> {
        match object {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::MemberExpression(member) => Self::member_access_name_single_quoted(member),
            Expression::ParenthesizedExpression(expr) => {
                Self::member_access_single_quoted_root_name(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                Self::member_access_single_quoted_root_name(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                Self::member_access_single_quoted_root_name(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                Self::member_access_single_quoted_root_name(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .and_then(Self::member_access_single_quoted_root_name),
            Expression::AwaitExpression(expr) => {
                Self::member_access_single_quoted_root_name(&expr.argument)
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    Self::member_access_single_quoted_root_name(object)
                }
            },
            Expression::ChainExpression(expr) => {
                Self::member_access_single_quoted_root_name(&expr.expression)
            }
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .and_then(Self::member_access_single_quoted_root_name),
            _ => None,
        }
    }

    pub(crate) fn member_access_bracketed_root_name(object: &Expression) -> Option<String> {
        match object {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::MemberExpression(member) => Self::member_access_name_bracketed(member),
            Expression::ParenthesizedExpression(expr) => {
                Self::member_access_bracketed_root_name(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                Self::member_access_bracketed_root_name(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                Self::member_access_bracketed_root_name(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                Self::member_access_bracketed_root_name(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .and_then(Self::member_access_bracketed_root_name),
            Expression::AwaitExpression(expr) => {
                Self::member_access_bracketed_root_name(&expr.argument)
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    Self::member_access_bracketed_root_name(object)
                }
            },
            Expression::ChainExpression(expr) => {
                Self::member_access_bracketed_root_name(&expr.expression)
            }
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .and_then(Self::member_access_bracketed_root_name),
            _ => None,
        }
    }

    pub(crate) fn member_access_root_name(object: &Expression) -> Option<String> {
        match object {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::MemberExpression(member) => Self::member_access_name(member),
            Expression::ParenthesizedExpression(expr) => {
                Self::member_access_root_name(&expr.expression)
            }
            Expression::TypeAssertion(expr) => Self::member_access_root_name(&expr.expression),
            Expression::SatisfiesExpression(expr) => {
                Self::member_access_root_name(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                Self::member_access_root_name(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .and_then(Self::member_access_root_name),
            Expression::AwaitExpression(expr) => Self::member_access_root_name(&expr.argument),
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => Self::member_access_root_name(object),
            },
            Expression::ChainExpression(expr) => Self::member_access_root_name(&expr.expression),
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => {
                call.args.first().and_then(Self::member_access_root_name)
            }
            _ => None,
        }
    }

    pub(crate) fn member_object_name(object: &Expression) -> Option<String> {
        match object {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::MemberExpression(member) if matches!(&member.object, Expression::Identifier(name) if name == "globalThis") => {
                member.property.clone()
            }
            Expression::ParenthesizedExpression(expr) => Self::member_object_name(&expr.expression),
            Expression::TypeAssertion(expr) => Self::member_object_name(&expr.expression),
            Expression::SatisfiesExpression(expr) => Self::member_object_name(&expr.expression),
            Expression::DecoratedExpression(expr) => Self::member_object_name(&expr.expression),
            Expression::SequenceExpression(expr) => {
                expr.expressions.last().and_then(Self::member_object_name)
            }
            Expression::AwaitExpression(expr) => Self::member_object_name(&expr.argument),
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => Self::member_object_name(object),
            },
            Expression::ChainExpression(expr) => Self::member_object_name(&expr.expression),
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => {
                call.args.first().and_then(Self::member_object_name)
            }
            _ => None,
        }
    }

    /// The recorded literal name of a `const` binding, walking the scope
    /// chain like `resolve_static_string_binding`.
    pub(crate) fn const_index_name(&self, name: &str) -> Option<String> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id)?;
            if let Some(value) = scope.const_index_names.get(name) {
                return Some(value.clone());
            }
            current = scope.parent;
        }
        self.global_scope.const_index_names.get(name).cloned()
    }

    /// The property name a member denotes statically: its own name when the
    /// parser could read it, else the fold of its index child. The checker's
    /// mirror of codegen's `static_member_name` (`emit/computed_member.rs`).
    pub(crate) fn static_member_name_after_fold(
        &self,
        member: &MemberExpression,
    ) -> Option<String> {
        if let Some(name) = member.static_name() {
            return Some(name.to_string());
        }
        let index = member.computed_index.as_deref()?;
        crate::static_analysis::computed_member::fold_nameless_computed_index(index, |name| {
            self.const_index_name(name)
        })
    }

    /// True when a static member name denotes a numeric INDEX rather than a
    /// property name. Decided by round-tripping through `format_js_number` --
    /// the one formatter the fold and HIR both spell a numeric key with -- so
    /// `"1"` and `"1.5"` are indices while `"length"`, and the `"inf"` /
    /// `"infinity"` spellings a bare `f64::from_str` would also accept, stay
    /// property names. Character-for-character the same predicate as codegen's
    /// `static_name_is_numeric_index` (`emit/computed_member.rs`); the two must
    /// not drift or the twins disagree on the string-receiver refusal.
    pub(crate) fn static_name_is_numeric_index(name: &str) -> bool {
        name.parse::<f64>()
            .is_ok_and(|value| format_js_number(value) == name)
    }

    /// Step 1 of spec §4.4: the runtime lanes codegen admits without a name.
    /// Every entry here mirrors a lane in codegen's two-child member arm; a
    /// currently-green fixture that starts refusing at `check` but not at
    /// `run` means a lane is missing HERE — add it, do not loosen the gate.
    ///
    /// The array lanes key on the STRUCTURAL registries
    /// (`is_structural_runtime_array` / `is_growable_array_binding`), which is
    /// exactly what codegen's `dynamic_array_read_base` consults (its
    /// `array_bindings` set, seeded from `repr_table.is_array_binding` for
    /// PARAMS only — mirrored here at function entry, `resolve/function.rs`).
    /// A bare `repr_table.is_array_binding` check does NOT belong here: the
    /// inference over-proves array-ness for any bracket-indexed binding, so it
    /// admitted `const o = {a:1, b:2}; let k = "b"; o[k]` — an object codegen
    /// never registers — and would have made this gate vacuous.
    pub(crate) fn nameless_computed_member_is_admitted_by_a_runtime_lane(
        &self,
        member: &MemberExpression,
    ) -> bool {
        let Some(index) = member.computed_index.as_deref() else {
            return false;
        };
        if let Expression::Identifier(key) = index {
            if self.for_in_key_shape(key).is_some() || self.is_for_in_key_value(key) {
                return true;
            }
        }
        if Self::is_process_argv_member(&member.object) {
            return true;
        }
        match &member.object {
            Expression::Identifier(base) => {
                self.is_structural_runtime_array(base)
                    || self.is_growable_array_binding(base)
                    || self.string_element_array_binding(base)
            }
            Expression::MemberExpression(_) => self
                .growable_i64_field_member_parts(&member.object)
                .is_some(),
            _ => false,
        }
    }

    /// Steps 2–4 of spec §4.4 for a computed member with no static name:
    /// admitted by a runtime lane, folded, or refused with the shared E5506.
    /// Runs for reads AND assignment targets (targets are resolved through
    /// `resolve_member_expression`), so a store gets exactly one diagnostic.
    pub(crate) fn gate_nameless_computed_member(&mut self, member: &MemberExpression) {
        if member.property.is_some() || member.computed_index.is_none() {
            return;
        }
        if self.nameless_computed_member_is_admitted_by_a_runtime_lane(member) {
            return;
        }
        let index = member.computed_index.as_deref().expect("checked above");
        let folded =
            crate::static_analysis::computed_member::fold_nameless_computed_index(index, |name| {
                self.const_index_name(name)
            });
        if folded.is_some() {
            return;
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            kali_common::computed_member_access_unavailable_message().to_string(),
        ));
    }

    /// A statically-known string receiver has no INDEX lane in either
    /// spelling (spec §4.4, "String receivers"); codegen refuses the same way.
    /// Narrowed by the same rule codegen applies (`emit_computed_member` and
    /// the `emit/control_flow.rs` two-child member arm): only a member with NO
    /// static name after the fold, or one whose static name is NUMERIC, is an
    /// index. A static property NAME on a string is dot semantics — `s.length`,
    /// `s["length"]` and `const k = "length"; s[k]` all answer alike, which is
    /// the fold's whole claim. Returns whether it claimed the access, so the
    /// nameless gate does not also fire and report the same defect twice.
    pub(crate) fn gate_static_string_receiver_index(&mut self, member: &MemberExpression) -> bool {
        if member.computed_index.is_none() {
            return false;
        }
        if self
            .resolve_static_string_expression(&member.object)
            .is_none()
        {
            return false;
        }
        if !self
            .static_member_name_after_fold(member)
            .as_deref()
            .is_none_or(Self::static_name_is_numeric_index)
        {
            return false;
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            kali_common::string_index_access_unavailable_message().to_string(),
        ));
        true
    }
}

#[cfg(test)]
#[path = "member_tests.rs"]
mod member_tests;
