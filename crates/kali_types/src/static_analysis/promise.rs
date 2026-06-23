//! Promise static-analysis helpers.
use crate::*;

impl TypeContext {
    pub(crate) fn resolve_promise_member_call(&mut self, _expr: &CallExpression) {}
}
