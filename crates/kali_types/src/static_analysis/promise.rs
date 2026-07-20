//! Promise static-analysis helpers.
use crate::*;

impl TypeContext {
    /// `Promise.resolve(v)` (and `globalThis.Promise.resolve(v)`) synchronously
    /// settles to `v`; codegen passes that value straight through the enclosing
    /// `await` (throw-fallout Stage 3 Task 4). Mirror that admission here by
    /// resolving the inner argument so its facts (repr, static-string provenance)
    /// propagate to the enclosing `await`, which resolves via its argument. This
    /// keeps the kali_types admission symmetric with the codegen recognizer — no
    /// admit/emit desync — without demanding the full async machinery (Stage 7).
    pub(crate) fn resolve_promise_member_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };
        if member.property.as_str() != "resolve" {
            return;
        }
        if !Self::is_promise_root_expression(&member.object) {
            return;
        }
        if let Some(argument) = expr.args.first() {
            self.resolve_expression(argument);
        }
    }

    /// The `Promise` global: the bare `Promise` identifier or `globalThis.Promise`.
    fn is_promise_root_expression(expression: &Expression) -> bool {
        match expression {
            Expression::Identifier(name) => name == "Promise",
            Expression::MemberExpression(member) => {
                member.property.as_str() == "Promise"
                    && matches!(&member.object, Expression::Identifier(name) if name == "globalThis")
            }
            Expression::ParenthesizedExpression(expr) => {
                Self::is_promise_root_expression(&expr.expression)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
#[path = "promise_tests.rs"]
mod promise_tests;
