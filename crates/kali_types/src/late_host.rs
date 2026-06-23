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

    pub(crate) fn resolve_permissions_query_descriptor_name(&self, expr: &Expression) -> Option<String> {
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

    pub(crate) fn resolve_late_permission_escalation_member(&mut self, expr: &MemberExpression) -> bool {
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

    pub(crate) fn resolve_late_env_assignment_mutation(&mut self, expr: &AssignmentExpression) -> bool {
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

    pub(crate) fn resolve_late_process_env_mutation_member(&mut self, member: &MemberExpression) -> bool {
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
