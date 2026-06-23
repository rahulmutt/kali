//! Member-expression resolution.
use crate::*;

impl TypeContext {
    pub(crate) fn resolve_member_expression(&mut self, expr: &MemberExpression) {
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

        Some(format!("{}.{}", object_name, expr.property))
    }

    pub(crate) fn is_runtime_args_slice_member(expr: &MemberExpression) -> bool {
        if expr.property != "slice" {
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

        Some(format!("{}[\"{}\"]", object_name, expr.property))
    }

    pub(crate) fn member_access_name_single_quoted(expr: &MemberExpression) -> Option<String> {
        let object_name = Self::member_access_single_quoted_root_name(&expr.object)?;

        Some(format!("{}['{}']", object_name, expr.property))
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
                Some(member.property.clone())
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

}

#[cfg(test)]
#[path = "member_tests.rs"]
mod member_tests;
