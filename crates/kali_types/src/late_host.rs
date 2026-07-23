//! Late host/runtime/network/env/permission analysis methods for `TypeContext`.

use crate::*;

impl TypeContext {
    pub(crate) fn resolve_permission_query_call(&mut self, expr: &CallExpression) {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return;
        };

        if !matches!(
            callee_name.as_str(),
            "Deno.permissions.query" | "globalThis.Deno.permissions.query"
        ) {
            return;
        }

        let Some(descriptor_name) = expr
            .args
            .first()
            .and_then(|expr| self.resolve_permissions_query_descriptor_name(expr))
        else {
            return;
        };

        if matches!(descriptor_name.as_str(), "read" | "write" | "net" | "env") {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "permission query descriptor '{}' is unavailable in the Phase-1 Deno permission facade",
                descriptor_name
            ),
        ));
    }

    pub(crate) fn resolve_process_kill_call(&mut self, expr: &CallExpression) {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return;
        };

        if !matches!(
            callee_name.as_str(),
            "process.kill" | "globalThis.process.kill"
        ) {
            return;
        }

        if self.api_surface != "node" {
            return;
        }

        let Some(first_arg) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::process_kill_zero_probe_unavailable_message(),
            ));
            return;
        };

        let Some(first_value) = self.resolve_static_numeric_literal_value(first_arg) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::process_kill_zero_probe_unavailable_message(),
            ));
            return;
        };

        if first_value != 0.0 || expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::process_kill_zero_probe_unavailable_message(),
            ));
        }
    }

    /// throw-fallout Stage 3 bucket #5: admit `performance.now()` (no args) and
    /// reject unsupported argument shapes with `FEATURE_UNAVAILABLE`, symmetric
    /// with the codegen recognizer (`FunctionEmitter::performance_now_import_index`
    /// and its `emit_call` arm). `performance` is already a baseline browser host
    /// global (see `builtins.rs`), so the callee resolves; this arm only guards
    /// the argument shape.
    pub(crate) fn resolve_performance_now_call(&mut self, expr: &CallExpression) {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return;
        };

        if !matches!(
            callee_name.as_str(),
            "performance.now" | "globalThis.performance.now"
        ) {
            return;
        }

        if !expr.args.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "performance.now() does not accept arguments in the current phase".to_string(),
            ));
        }
    }

    /// throw-fallout Stage 3 bucket #6: admit `crypto.getRandomValues(<buffer>)`
    /// (exactly one argument), `crypto.randomUUID()` (no arguments),
    /// `crypto.subtle.digest(<string>, <buffer>)`, and
    /// `new TextEncoder().encode(<string>)`, rejecting the unsupported
    /// argument/algorithm shapes with `FEATURE_UNAVAILABLE`. Symmetric with the
    /// codegen recognizers (`crypto_get_random_values_import_index` /
    /// `crypto_random_uuid_import_index` / `crypto_subtle_digest_import_index` /
    /// `is_text_encoder_encode` and their `emit_call` arms). `crypto` /
    /// `TextEncoder` are baseline host globals (see `builtins.rs`), so the callee
    /// resolves; this arm only guards the argument shape.
    pub(crate) fn resolve_crypto_call(&mut self, expr: &CallExpression) {
        // `crypto.subtle.digest(algo, bytes)` and `new TextEncoder().encode(str)`
        // are recognized STRUCTURALLY (their callee objects — a `crypto.subtle`
        // member chain and a `new TextEncoder()` construction — are not static
        // references `resolve_static_callable_name` names). Codegen accepts a
        // string algorithm/argument and a string-backed byte buffer; reject
        // everything else symmetrically.
        if let Expression::MemberExpression(member) = &expr.callee {
            if member.computed_index.is_none() {
                match member.property.as_str() {
                    "digest" if Self::is_crypto_subtle_object(&member.object) => {
                        // Mirror codegen's `crypto_subtle_digest` arm EXACTLY: it
                        // recognizes-and-lowers any structural `crypto.subtle.digest`
                        // and rejects ONLY on arity (missing / extra arguments), never
                        // on operand string-ness. A previous string-typed gate here
                        // HARD-REJECTED (E5506) well-formed-but-not-statically-string
                        // shapes — e.g. `crypto.subtle.digest('SHA-256', encode(f(x)))`
                        // where the input flows from an imported/runtime call — which
                        // codegen tolerantly lowers and which the pre-recognizer phase
                        // compiled as an unrecognized placeholder that checked, built,
                        // and deployed. Those unsupported-but-well-formed shapes now
                        // fall through here (no E5506) so deployability is preserved;
                        // only an arity mismatch (which codegen ALSO rejects) errors.
                        if expr.args.len() != 2 {
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                "crypto.subtle.digest requires exactly a string algorithm name and an input buffer in the current phase"
                                    .to_string(),
                            ));
                        }
                        return;
                    }
                    "encode" if Self::is_new_text_encoder(&member.object) => {
                        // Mirror codegen's `is_text_encoder_encode` arm EXACTLY: it
                        // reinterprets any structural `new TextEncoder().encode(x)` and
                        // rejects ONLY on arity (missing / extra arguments), never on
                        // operand string-ness. See the `digest` arm above — the prior
                        // string-typed gate over-rejected runtime/imported-call inputs
                        // (`encode(describe(count))`) that codegen tolerantly lowers and
                        // that previously deployed as a placeholder. Fall through for
                        // those; error only on an arity mismatch codegen also rejects.
                        if expr.args.len() != 1 {
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                "TextEncoder().encode requires exactly a single argument in the current phase"
                                    .to_string(),
                            ));
                        }
                        return;
                    }
                    "decode" if Self::is_new_text_decoder(&member.object) => {
                        // Stage P5 Task 4, mirroring the `encode` arm above: the
                        // codegen decode arm rejects ONLY on arity here (its
                        // ARGUMENT-provenance gate is a separate, structural
                        // fail-closed deny that this pass cannot see, and which
                        // errors on its own), so reject the same arity mismatch
                        // symmetrically and let every other shape fall through.
                        if expr.args.len() != 1 {
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                "TextDecoder().decode requires exactly a single argument in the current phase"
                                    .to_string(),
                            ));
                        }
                        return;
                    }
                    _ => {}
                }
            }
        }

        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return;
        };

        match callee_name.as_str() {
            "crypto.getRandomValues" | "globalThis.crypto.getRandomValues"
                if expr.args.is_empty() =>
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "crypto.getRandomValues requires a typed-array buffer argument in the current phase"
                        .to_string(),
                ));
            }
            "crypto.randomUUID" | "globalThis.crypto.randomUUID" if !expr.args.is_empty() => {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "crypto.randomUUID() does not accept arguments in the current phase"
                        .to_string(),
                ));
            }
            _ => {}
        }
    }

    /// True when `expr` is the `crypto.subtle` object (member `subtle` off the
    /// `crypto` identifier). Structural mirror of `repr_infer::is_crypto_subtle_object`.
    fn is_crypto_subtle_object(expr: &Expression) -> bool {
        matches!(
            expr,
            Expression::MemberExpression(member)
                if member.computed_index.is_none()
                    && member.property.as_str() == "subtle"
                    && matches!(&member.object, Expression::Identifier(name) if name == "crypto")
        )
    }

    /// True when `expr` invokes the `TextEncoder` constructor — `new TextEncoder()`
    /// or the bare `TextEncoder()` call the parser leaves as the `.encode` object
    /// when it hoists the `new`. Structural mirror of `repr_infer::is_text_encoder_ctor`.
    fn is_new_text_encoder(expr: &Expression) -> bool {
        match expr {
            Expression::NewExpression(new_expr) => {
                matches!(&new_expr.callee, Expression::Identifier(name) if name == "TextEncoder")
            }
            Expression::CallExpression(call) => {
                matches!(&call.callee, Expression::Identifier(name) if name == "TextEncoder")
            }
            _ => false,
        }
    }

    /// True when `expr` invokes the `TextDecoder` constructor — `new TextDecoder()`
    /// or the bare `TextDecoder()` call the parser leaves as the `.decode` object
    /// when it hoists the `new`. Structural mirror of `repr_infer::is_text_decoder_ctor`
    /// and the twin of `is_new_text_encoder`.
    fn is_new_text_decoder(expr: &Expression) -> bool {
        match expr {
            Expression::NewExpression(new_expr) => {
                matches!(&new_expr.callee, Expression::Identifier(name) if name == "TextDecoder")
            }
            Expression::CallExpression(call) => {
                matches!(&call.callee, Expression::Identifier(name) if name == "TextDecoder")
            }
            _ => false,
        }
    }

    pub(crate) fn resolve_permissions_query_descriptor_name(
        &self,
        expr: &Expression,
    ) -> Option<String> {
        match expr {
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_permissions_query_descriptor_name(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_permissions_query_descriptor_name(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_permissions_query_descriptor_name(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_permissions_query_descriptor_name(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_permissions_query_descriptor_name(&expr.expression)
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                for property in properties {
                    if !matches!(property.kind, ObjectPropertyKind::Init) {
                        continue;
                    }

                    let key_name = match &property.key {
                        PropertyName::Identifier(name) | PropertyName::String(name) => {
                            name.as_str()
                        }
                        PropertyName::Number(_) => continue,
                    };

                    if key_name != "name" {
                        continue;
                    }

                    return self.resolve_static_string_expression(&property.value);
                }

                None
            }
            _ => None,
        }
    }

    pub(crate) fn resolve_threaded_runtime_member(&mut self, expr: &MemberExpression) {
        let Expression::Identifier(object_name) = &expr.object else {
            return;
        };

        if object_name != "globalThis" {
            return;
        }

        if !matches!(expr.property.as_str(), "SharedArrayBuffer" | "Atomics") {
            return;
        }

        if self.has_threaded_runtime_profile() {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "threaded runtime global 'globalThis.{}' is unavailable until the WASM-threaded profile is enabled",
                expr.property
            ),
        ));
    }

    pub(crate) fn resolve_late_host_control_member(&mut self, expr: &MemberExpression) {
        if !matches!(
            expr.property.as_str(),
            "pid" | "cwd" | "chdir" | "exit" | "kill"
        ) {
            return;
        }

        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return;
        };

        if expr.property == "pid" && object_name == "Deno" && self.api_surface == "deno" {
            return;
        }

        if expr.property == "exit" && object_name == "Deno" && self.api_surface == "deno" {
            return;
        }

        if expr.property == "cwd" && object_name == "Deno" && self.api_surface == "deno" {
            return;
        }

        if expr.property == "chdir" && object_name == "Deno" && self.api_surface == "deno" {
            return;
        }

        if expr.property == "cwd" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if expr.property == "chdir" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if expr.property == "pid" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if expr.property == "exit" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if expr.property == "kill" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if !matches!(object_name.as_str(), "Deno" | "process") {
            return;
        }

        let dotted = Self::member_access_name(expr)
            .unwrap_or_else(|| format!("{}.{}", object_name, expr.property));
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());
        let extra_alias = if object_name == "Deno"
            && matches!(expr.property.as_str(), "cwd" | "chdir" | "exit")
        {
            Some(format!("globalThis[\"Deno\"].{}", expr.property))
        } else if object_name == "process"
            && matches!(
                expr.property.as_str(),
                "pid" | "cwd" | "chdir" | "exit" | "kill"
            )
        {
            let mut aliases = vec![
                format!("globalThis[\"process\"].{}", expr.property),
                format!("globalThis.process[\"{}\"]", expr.property),
                format!("globalThis[\"process\"][\"{}\"]", expr.property),
            ];
            if expr.property == "kill" {
                aliases.extend(
                    late_process_control_single_quoted_kill_aliases()
                        .iter()
                        .copied()
                        .map(String::from),
                );
                aliases.extend(
                    process_kill_zero_probe_wrapped_zero_aliases()
                        .iter()
                        .copied()
                        .map(String::from),
                );
            } else if expr.property == "exit" {
                aliases.extend(
                    late_process_control_single_quoted_exit_aliases()
                        .iter()
                        .copied()
                        .map(String::from),
                );
            }
            Some(aliases.join(", "))
        } else {
            None
        };

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "late host-control API '{}' (aka {}{}) is unavailable until the later host-control compatibility path is enabled",
                dotted,
                bracketed,
                extra_alias
                    .as_deref()
                    .map(|alias| format!(", {alias}"))
                    .unwrap_or_default()
            ),
        ));
    }

    pub(crate) fn resolve_late_subprocess_member(&mut self, expr: &MemberExpression) -> bool {
        if self.sandbox_policy_attached {
            return false;
        }

        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return false;
        };

        if object_name != "Deno" || expr.property != "Command" {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "subprocess spawning API '{}' (aka {}) is unavailable until the later subprocess compatibility path is enabled",
                dotted, bracketed
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_network_member(&mut self, expr: &MemberExpression) -> bool {
        if self.sandbox_policy_attached {
            return false;
        }

        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return false;
        };

        if object_name != "Deno"
            || !matches!(expr.property.as_str(), "connect" | "listen" | "serve")
        {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "socket/listener networking API '{}' (aka {}) is unavailable until the later network compatibility path is enabled",
                dotted, bracketed
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_permission_escalation_member(
        &mut self,
        expr: &MemberExpression,
    ) -> bool {
        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        if !matches!(
            dotted.as_str(),
            "Deno.permissions.request"
                | "Deno.permissions.revoke"
                | "globalThis.Deno.permissions.request"
                | "globalThis.Deno.permissions.revoke"
        ) && !matches!(
            bracketed.as_str(),
            r#"Deno["permissions"]["request"]"#
                | r#"Deno["permissions"]["revoke"]"#
                | r#"globalThis["Deno"]["permissions"]["request"]"#
                | r#"globalThis["Deno"]["permissions"]["revoke"]"#
        ) {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "permission escalation API '{}' (aka {}) is unavailable in the Phase-1 Deno permission facade",
                dotted, bracketed
            ),
        ));
        true
    }

    pub(crate) fn resolve_deno_args_member(&mut self, expr: &MemberExpression) -> bool {
        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return false;
        };

        if object_name != "Deno" || expr.property != "args" {
            return false;
        }

        if self.api_surface == "deno" {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "invocation arguments API '{}' (aka {}) is unavailable on the {} API surface until the Deno runtime surface is enabled",
                dotted, bracketed, self.api_surface
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_env_object_member(&mut self, expr: &MemberExpression) -> bool {
        if self.api_surface == "deno" {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        if !matches!(
            dotted.as_str(),
            "Deno.env.toObject"
                | "Deno.env[\"toObject\"]"
                | "globalThis.Deno.env.toObject"
                | "globalThis.Deno.env[\"toObject\"]"
                | "Deno[\"env\"].toObject"
                | "Deno[\"env\"][\"toObject\"]"
                | "globalThis.Deno[\"env\"].toObject"
                | "globalThis.Deno[\"env\"][\"toObject\"]"
                | "globalThis[\"Deno\"].env.toObject"
                | "globalThis[\"Deno\"].env[\"toObject\"]"
                | "globalThis[\"Deno\"][\"env\"].toObject"
                | "globalThis[\"Deno\"][\"env\"][\"toObject\"]"
        ) && !matches!(
            bracketed.as_str(),
            r#"Deno["env"]["toObject"]"#
                | r#"globalThis["Deno"]["env"]["toObject"]"#
                | r#"globalThis.Deno["env"]["toObject"]"#
        ) {
            return false;
        }

        let aliases = [
            bracketed.as_str(),
            "Deno.env[\"toObject\"]",
            "Deno[\"env\"].toObject",
            "Deno[\"env\"][\"toObject\"]",
            "globalThis.Deno.env[\"toObject\"]",
            "globalThis.Deno[\"env\"].toObject",
            "globalThis.Deno[\"env\"][\"toObject\"]",
            "globalThis[\"Deno\"].env.toObject",
            "globalThis[\"Deno\"].env[\"toObject\"]",
            "globalThis[\"Deno\"][\"env\"].toObject",
            "globalThis[\"Deno\"][\"env\"][\"toObject\"]",
            "globalThis[\"Deno\"].env[\"toObject\"]",
            "globalThis.Deno[\"env\"][\"toObject\"]",
            "globalThis[\"Deno\"].env[\"toObject\"]",
        ];

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "environment snapshot materialization API '{}' (aka {}) is unavailable until the later env-object materialization and object-aggregate lowering path is enabled",
                dotted,
                aliases.join(", "),
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_env_mutation_member(&mut self, expr: &MemberExpression) -> bool {
        if self.api_surface == "deno" {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        if !matches!(
            dotted.as_str(),
            "Deno.env.set"
                | "Deno.env.delete"
                | "globalThis.Deno.env.set"
                | "globalThis.Deno.env.delete"
        ) {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                dotted, bracketed, self.api_surface
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_env_assignment_mutation(
        &mut self,
        expr: &AssignmentExpression,
    ) -> bool {
        let Expression::MemberExpression(member) = &expr.left else {
            return false;
        };

        let dotted = Self::member_access_name(member).unwrap_or_else(|| member.property.clone());
        let bracketed =
            Self::member_access_name_bracketed(member).unwrap_or_else(|| dotted.clone());

        if Self::is_process_env_root_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        if self.api_surface != "node" && Self::is_process_env_mutation_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        false
    }

    pub(crate) fn resolve_late_process_env_mutation_member(
        &mut self,
        member: &MemberExpression,
    ) -> bool {
        let dotted = Self::member_access_name(member).unwrap_or_else(|| member.property.clone());
        let bracketed =
            Self::member_access_name_bracketed(member).unwrap_or_else(|| dotted.clone());

        if Self::is_process_env_root_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        if self.api_surface != "node" && Self::is_process_env_mutation_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        false
    }

    pub(crate) fn is_process_env_root_path(path: &str) -> bool {
        matches!(path, "process.env" | "globalThis.process.env")
    }

    pub(crate) fn is_process_env_mutation_path(path: &str) -> bool {
        Self::is_process_env_root_path(path)
            || path.starts_with("process.env.")
            || path.starts_with("process.env[")
            || path.starts_with("globalThis.process.env.")
            || path.starts_with("globalThis.process.env[")
    }

    pub(crate) fn resolve_late_intl_member(&mut self, expr: &MemberExpression) -> bool {
        let is_intl_root = matches!(&expr.object, Expression::Identifier(name) if name == "Intl")
            || matches!(
                &expr.object,
                Expression::Identifier(name) if name == "globalThis" && expr.property == "Intl"
            )
            || matches!(
                &expr.object,
                Expression::MemberExpression(member)
                    if matches!(&member.object, Expression::Identifier(name) if name == "globalThis")
                        && member.property == "Intl"
            );

        if !is_intl_root {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr)
            .unwrap_or_else(|| format!("globalThis[\"{}\"]", expr.property));
        let single_quoted = Self::member_access_name_single_quoted(expr)
            .unwrap_or_else(|| format!("globalThis['{}']", expr.property));
        let single_quoted_root_dotted = Self::member_access_single_quoted_root_name(&expr.object)
            .map(|root| format!("{}.{}", root, expr.property))
            .unwrap_or_else(|| single_quoted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "broader Intl support via '{}' (aka {}, {}, {}) is unavailable until the later web/Intl compatibility path is enabled",
                dotted, bracketed, single_quoted, single_quoted_root_dotted
            ),
        ));
        true
    }

    pub(crate) fn resolve_late_object_model_member(&mut self, expr: &MemberExpression) -> bool {
        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());
        let single_quoted = Self::member_access_name_single_quoted(expr).unwrap_or_else(|| {
            format!(
                "{}['{}']",
                dotted
                    .rsplit_once('.')
                    .map(|(root, _)| root)
                    .unwrap_or(&dotted),
                expr.property
            )
        });
        let single_quoted_root_dotted = Self::member_access_single_quoted_root_name(&expr.object)
            .map(|root| format!("{}.{}", root, expr.property))
            .unwrap_or_else(|| single_quoted.clone());

        if self.api_surface != "node"
            && self.is_supported_static_callable_member_name(&dotted, &bracketed)
        {
            return false;
        }

        if matches!(
            dotted.as_str(),
            "Proxy.revocable" | "globalThis.Proxy.revocable"
        ) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "late object-model API '{}' (aka {}, {}, {}) is unavailable until the later object-model compatibility path is enabled",
                    dotted, bracketed, single_quoted, single_quoted_root_dotted
                ),
            ));
            return true;
        }

        if matches!(
            dotted.as_str(),
            "Object.hasOwn"
                | "globalThis.Object.hasOwn"
                | "Object.prototype.hasOwnProperty.call"
                | "globalThis.Object.prototype.hasOwnProperty.call"
                | "Object.hasOwnProperty.call"
                | "globalThis[\"Object\"].prototype.hasOwnProperty[\"call\"]"
                | "globalThis.Object.hasOwnProperty.call"
        ) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "late object-model API '{}' (aka {}) is unavailable until the later object-model compatibility path is enabled",
                    dotted, bracketed
                ),
            ));
            return true;
        }

        if !matches!(
            expr.property.as_str(),
            "Proxy" | "WeakMap" | "WeakSet" | "WeakRef" | "FinalizationRegistry"
        ) {
            return false;
        }

        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return false;
        };

        if object_name != "globalThis" {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "late object-model API '{}' (aka {}) is unavailable until the later object-model compatibility path is enabled",
                dotted, bracketed
            ),
        ));
        true
    }
}

#[cfg(test)]
#[path = "late_host_tests.rs"]
mod late_host_tests;
