//! A computed member access with no static property name
//! (`LirNodeKind::ComputedMember`): the one place such a node is emitted.
//!
//! Spec: docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md §4.4.

use crate::*;
use kali_common::js_number::format_js_number;
use kali_common::{
    computed_member_access_unavailable_message, string_index_access_unavailable_message,
};

impl FunctionEmitter<'_> {
    /// The property name a member node denotes statically: its text when it
    /// has one, else the fold of its index child. The fold admits exactly a
    /// childless `Value` (a bare identifier) whose `const` binding resolves
    /// to a static string or number — `bindings` is `const`-only by
    /// construction, and a `let`/`var` lives in `locals` and never resolves
    /// here. A number is rendered by `format_js_number`, the same formatter
    /// HIR stores a numeric key with, so probe and key stay one currency.
    pub(crate) fn static_member_name(&self, node: &LirNode) -> Option<String> {
        if let Some(text) = node.text.as_deref() {
            return Some(text.to_string());
        }
        let index = *node.children.get(1)?;
        let index_node = self.node(index);
        if index_node.kind != LirNodeKind::Value || !index_node.children.is_empty() {
            return None;
        }
        match self.resolve_static_object_identity_value(index)? {
            StaticObjectIdentityValue::String(value) => Some(value),
            StaticObjectIdentityValue::Number(value) => Some(format_js_number(value)),
            _ => None,
        }
    }

    /// `node` as the ordinary `Value`-shaped member the rest of codegen
    /// understands, carrying `name` as its text. Used to re-dispatch a folded
    /// access through the lanes the literal spelling takes; the original node
    /// is not changed (the emitter borrows the program immutably), which is
    /// why by-id consumers keep seeing the nameless kind and decline.
    pub(crate) fn named_twin(node: &LirNode, name: String) -> LirNode {
        let mut twin = node.clone();
        twin.kind = LirNodeKind::Value;
        twin.text = Some(name);
        twin
    }

    /// The store target as the `Value`-shaped node the store arms match on.
    /// A nameless computed member (`LirNodeKind::ComputedMember`) is presented
    /// as a text-less two-child `Value` so the for-in ordinal and runtime
    /// array arms — which never read the text for a name — keep admitting it.
    pub(crate) fn store_target_node(&self, left: LirNodeId) -> LirNode {
        let mut target = self.node(left).clone();
        if target.kind == LirNodeKind::ComputedMember {
            target.kind = LirNodeKind::Value;
        }
        target
    }

    /// True when `id` resolves to a statically-known string, the receiver
    /// shape that has no admitting INDEX lane in either spelling.
    pub(crate) fn is_static_string_receiver(&self, id: LirNodeId) -> bool {
        matches!(
            self.resolve_static_object_identity_value(id),
            Some(StaticObjectIdentityValue::String(_))
        )
    }

    /// True when a static member name denotes a numeric INDEX rather than a
    /// property name. Decided by round-tripping through `format_js_number` --
    /// the one formatter the fold and HIR both spell a numeric key with -- so
    /// `"1"` and `"1.5"` are indices while `"length"`, and the `"inf"` /
    /// `"infinity"` spellings a bare `f64::from_str` would also accept, stay
    /// property names. No second formatter, no second parse rule.
    pub(crate) fn static_name_is_numeric_index(name: &str) -> bool {
        name.parse::<f64>()
            .is_ok_and(|value| format_js_number(value) == name)
    }

    /// Read gateway for a nameless computed member (spec §4.4, in order):
    /// 1. the runtime lanes that never read the text for a name, offered a
    ///    `Value`-shaped text-less probe;
    /// 2. the fold;
    /// 3. dot semantics — the named twin re-dispatched through `emit_value`;
    /// 4. refuse.
    pub(crate) fn emit_computed_member(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        node: &LirNode,
        want_value: bool,
    ) -> EmittedValue {
        let mut probe = node.clone();
        probe.kind = LirNodeKind::Value;
        probe.text = None;

        if let Some((base, index, elem)) = self.computed_forin_object_access(&probe) {
            return self.emit_object_field_read_dynamic(function, base, index, elem);
        }
        if self.growable_array_read_base(&probe).is_some() || self.growable_field_read_base(&probe)
        {
            return self.emit_growable_index_read(function, node.children[0], node.children[1]);
        }
        if let Some(base_name) = self.dynamic_array_read_base(&probe) {
            return self.emit_dynamic_array_read_node(
                function,
                node.children[0],
                node.children[1],
                &base_name,
            );
        }
        // The fold decides the SHAPE of the string-receiver refusal, so it is
        // resolved before the refusal is taken. A statically-known string has
        // no INDEX lane in either spelling (`s[1]` was a silent `0`; `s[k]`
        // with a numeric `k` folds onto that same lane), but a folded PROPERTY
        // NAME is dot semantics like any other fold: `s.length`, `s["length"]`
        // and `const k = "length"; s[k]` must all answer `3`, which is the
        // fold's whole claim (spec §4.4 step 3, "Dot semantics from here").
        // Spec §5's string row is about an INDEX (`s[1]`, `s[k]`), and §2.5's
        // controls do not license breaking property-name access on a string.
        let name = self.static_member_name(node);
        if self.is_static_string_receiver(node.children[0])
            && name
                .as_deref()
                .is_none_or(Self::static_name_is_numeric_index)
        {
            return self.deny_e5506(function, string_index_access_unavailable_message());
        }
        let Some(name) = name else {
            return self.deny_e5506(function, computed_member_access_unavailable_message());
        };
        let twin = Self::named_twin(node, name);
        self.emit_value(function, id, &twin, want_value)
    }
}

#[cfg(test)]
#[path = "computed_member_tests.rs"]
mod computed_member_tests;
