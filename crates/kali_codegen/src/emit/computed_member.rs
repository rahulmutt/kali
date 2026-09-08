//! A computed member access with no static property name
//! (`LirNodeKind::ComputedMember`): the one place such a node is emitted.
//!
//! Spec: docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md §4.4.

use crate::*;
use kali_common::computed_member_access_unavailable_message;

impl FunctionEmitter<'_> {
    /// Read gateway for a nameless computed member. Filled in by Task 3 of the
    /// plan; until then every such access refuses, which is already strictly
    /// more honest than the fabricated name it replaces.
    pub(crate) fn emit_computed_member(
        &mut self,
        function: &mut Function,
        _id: LirNodeId,
        _node: &LirNode,
        _want_value: bool,
    ) -> EmittedValue {
        self.deny_e5506(function, computed_member_access_unavailable_message())
    }
}

#[cfg(test)]
#[path = "computed_member_tests.rs"]
mod computed_member_tests;
