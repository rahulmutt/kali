//! `switch` lowering: admittance plan + emit.
//!
//! The plan is built from POSITIVE evidence only. `SwitchPlan::build` returns
//! `Err(reason)` unless it can prove every part of the switch is in the
//! admitted set, and `emit_switch` denies on `Err`. There is deliberately no
//! denylist of bad shapes anywhere in this file: this repository's most
//! repeated lesson is that a denylist of shapes leaks forever and only an
//! allowlist at the choke point closes a class (Spec 4a needed six rounds
//! before a default-deny at the single read site closed the for-in-key class
//! by construction).
//!
//! Extending the admitted set therefore means adding a proof to `build`, never
//! removing a rejection.

use crate::*;

/// How a clause body ends. Only terminators in this enum are admitted; a
/// clause that ends any other way is true fallthrough and is denied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClauseTerminator {
    /// The clause's last statement is `return`.
    Return,
    /// The clause's last statement is an unlabeled `break`.
    Break,
    /// The clause has no statements at all and groups onto the next clause.
    EmptyGroup,
}

/// One admitted clause.
pub(crate) struct SwitchClause {
    /// `None` for the `default` clause.
    pub(crate) test: Option<LirNodeId>,
    pub(crate) body: LirNodeId,
    pub(crate) terminator: ClauseTerminator,
}

/// A switch this emitter has proven it can lower correctly.
pub(crate) struct SwitchPlan {
    pub(crate) discriminant: LirNodeId,
    pub(crate) clauses: Vec<SwitchClause>,
}

impl<'a> FunctionEmitter<'a> {
    /// Build a plan, or explain why this switch is not admitted.
    ///
    /// The allowlist is currently EMPTY: nothing is admitted. Tasks that widen
    /// the admitted set replace the unconditional rejection with a proof.
    pub(crate) fn switch_plan(&self, _node: &LirNode) -> Result<SwitchPlan, String> {
        Err("no switch shape is admitted in the current phase".to_string())
    }

    pub(crate) fn emit_switch(
        &mut self,
        function: &mut Function,
        _id: LirNodeId,
        node: &LirNode,
    ) -> EmittedValue {
        match self.switch_plan(node) {
            Ok(plan) => self.emit_switch_plan(function, plan),
            Err(reason) => {
                let message = format!(
                    "this `switch` is not in the supported lowering set ({reason}); \
                     rewrite it as `if`/`else if` or use a supported switch shape \
                     (fail-closed)"
                );
                self.deny_e5506(function, &message)
            }
        }
    }

    /// Emit an admitted plan. Unreachable until a task admits a shape.
    fn emit_switch_plan(&mut self, function: &mut Function, _plan: SwitchPlan) -> EmittedValue {
        self.deny_e5506(
            function,
            "internal: a switch plan was admitted but no lowering exists for it (fail-closed)",
        )
    }
}
