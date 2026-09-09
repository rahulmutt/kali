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
    /// chain like `resolve_static_string_binding` — but STOPPING at any scope
    /// that declares the name at all. `const_index_names` holds only the
    /// foldable subset (`const` + string/number literal), so a shadowing
    /// `let`/`var`/parameter/computed-`const` leaves no entry, and a walk that
    /// merely looked for an entry would tunnel past the shadow and fold the
    /// OUTER binding's literal — admitting `const k = "b"; const o = {a:1,b:2};
    /// function f(k) { return o[k]; }`, which codegen refuses.
    /// `resolve_static_string_binding` needs no such stop: `static_values`
    /// records every declaration kind, so a shadow stops it by construction.
    ///
    /// The walk is also FUNCTION-BOUNDED, exactly like
    /// `is_structural_runtime_array` and for the same reason: codegen's
    /// `bindings` map is per-`FunctionEmitter`, so a module-scope `const` is
    /// simply not visible while a named function is emitted and its fold
    /// declines there. Measured: `const k = "b"; const o = {a:1,b:2};
    /// function f() { return o[k]; }` is refused by `run`, so admitting it
    /// here would kill nothing but would leave `check` claiming a lane that
    /// does not exist. Module/global scope is therefore reachable only under
    /// `_start` (no tracked function).
    pub(crate) fn const_index_name(&self, name: &str) -> Option<String> {
        let tracked_scope = self.current_function_scope();
        // Task 6 review follow-up: a name declared more than once in this
        // function does not fold, in EITHER checker pass. This walk is
        // scope-precise, but the tables the fold ultimately has to agree with
        // are not — `repr_infer`'s `const_index_names` is flat per function
        // and codegen's `bindings` is flat per `FunctionEmitter`, both
        // last-write-wins. Resolving `"b"` here while those resolve `"a"`
        // admitted a store and a read that landed on a DIFFERENT real field:
        // `const k = "b"; const o = {a:1,b:2}; if (true) { const k = "a"; }`
        // then `o[k] = 8` / `o[k]` wrote and read `a`, silently, exit 0 —
        // R-59's symptom on the lane this project opened. Declining is the
        // honest answer, and it must happen HERE too: if only `repr_infer`
        // declined, this gate would keep admitting while the store lost its
        // materialization evidence, and `check` would be clean where `run`
        // refuses (spec §2.6, §8's dangerous direction).
        if self
            .shadowed_index_names
            .contains(&(tracked_scope, name.to_string()))
        {
            return None;
        }
        let mut current = self.current_scope_id();
        loop {
            let Some(scope_id) = current else {
                return self.global_scope.const_index_names.get(name).cloned();
            };
            // `?` rather than a `let ... else { return None }`: clippy's
            // `question_mark` lint refuses the longer spelling under
            // `-D warnings`, and the two are the same fail-closed answer (a
            // scope id with no scope resolves no name). The `bool`-returning
            // twins in `resolve/expression.rs` keep the `let ... else` form
            // because `?` does not apply to their return type.
            let scope = self.scopes.get(&scope_id)?;
            if scope.scope_type == ScopeType::Function && Some(scope_id) != tracked_scope {
                // Crossed into a function `current_function_name()` does not
                // name — fail closed rather than guess.
                return None;
            }
            if let Some(value) = scope.const_index_names.get(name) {
                return Some(value.clone());
            }
            if scope.bindings.contains_key(name) {
                return None;
            }
            if scope.scope_type == ScopeType::Function {
                // The tracked function's own top scope, no hit: a free module
                // reference codegen's emitter for this function cannot fold.
                return None;
            }
            current = scope.parent;
        }
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
    /// NO repr-table proof belongs here, in any spelling: the inference
    /// over-proves array-ness for any bracket-indexed binding, so a bare
    /// `repr_table.is_array_binding` admitted `const o = {a:1, b:2};
    /// let k = "b"; o[k]` (an object codegen never registers), and
    /// `string_element_array_binding` — which IS that proof narrowed to a
    /// `Repr::String` element axis — admitted a literal array in both
    /// directions: `const a = ["x","y"]; let i = 0; a[i]` and `a[i] = "z"`
    /// were check-clean while `run` refused, the store having additionally
    /// LOST the refusal `reject_literal_array_unfoldable_mutation` used to
    /// give it. Measured, then deleted.
    ///
    /// There is no `process.argv` entry either. Codegen's argv element lane
    /// (`intrinsics/host.rs`, `is_process_argv_element`) requires the INDEX
    /// CHILD's text to parse as a non-negative integer literal, and
    /// `emit_computed_member` has no argv lane at all — so the whole live
    /// domain of such an entry (`process.argv[i]`, `process.argv[i + 1]`, the
    /// literal index having already returned early with a name) is what
    /// codegen refuses. Measured under `--api node`: both refuse.
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
        match &member.object {
            Expression::Identifier(base) => {
                self.is_structural_runtime_array(base) || self.is_growable_array_binding(base)
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
