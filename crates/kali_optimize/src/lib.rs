//! Optimization passes for the Kali compiler.
//!
//! The current implementation focuses on the deterministic, tree-shaped LIR
//! that the rest of the repository already produces. That gives us a safe place
//! to land constant folding, branch elimination, and a handful of algebraic
//! simplifications without needing a full SSA pipeline yet.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

mod profile;

pub use profile::{ProfileData, ProfileSample, ProfileSampleKind, PROFILE_DATA_VERSION};

mod driver;
pub use driver::{OptimizationLevel, OptimizationReport, Optimizer};

mod constant_fold;
pub(crate) use constant_fold::*;

mod specialize;
pub(crate) use specialize::*;

mod inline;
pub(crate) use inline::*;

use kali_lir::{LirNode, LirNodeId, LirNodeKind, LirProgram};
use kali_mir::{LayoutDescriptor, MirBindingKind, MirProgram as MirAnalysisProgram};

/// Minimum recorded weight for a function sample to count as hot in the PGO report.
const HOT_FUNCTION_MINIMUM_WEIGHT: u64 = 8;

impl Optimizer {
    pub(crate) fn fold_layout_member_access(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        tracker: &mut SpecializationTracker,
        owner: &str,
        env: &BindingEnv,
    ) -> bool {
        let snapshot = program.nodes[id.0 as usize].clone();
        let Some(property) = snapshot.text.as_deref() else {
            return false;
        };
        if snapshot.kind != LirNodeKind::Value || snapshot.children.len() != 1 {
            return false;
        }

        let Some(object_id) = snapshot.children.first().copied() else {
            return false;
        };

        if let Some(field_value) = self.object_literal_field(program, object_id, property) {
            let key = format!(
                "layout-member:{}:{}",
                property,
                node_signature(program, object_id)
            );
            if !tracker.allow(owner, key) {
                return false;
            }

            program.nodes[id.0 as usize] = program.nodes[field_value.0 as usize].clone();
            return true;
        }

        if property == "length" {
            if let Some(length) = self.array_literal_length(program, object_id) {
                let key = format!(
                    "layout-array-length:{}:{}",
                    property,
                    node_signature(program, object_id)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }

                program.nodes[id.0 as usize] = LirNode {
                    kind: LirNodeKind::Literal,
                    text: Some(length.to_string()),
                    children: Vec::new(),
                    function_flavor: None,
                };
                return true;
            }
        }

        let Some(index) = self.constant_array_index(program, env, property) else {
            return false;
        };
        let Some(element_value) = self.array_literal_element(program, object_id, index) else {
            return false;
        };

        let key = format!(
            "layout-array:{}:{}:{}",
            index,
            property,
            node_signature(program, object_id)
        );
        if !tracker.allow(owner, key) {
            return false;
        }

        program.nodes[id.0 as usize] = program.nodes[element_value.0 as usize].clone();
        true
    }

    pub(crate) fn object_literal_field(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        field: &str,
    ) -> Option<LirNodeId> {
        if !self.is_object_literal(program, id) {
            return None;
        }

        let node = program.nodes.get(id.0 as usize)?;
        for property in &node.children {
            let property_node = program.nodes.get(property.0 as usize)?;
            if property_node.children.len() != 2 {
                continue;
            }
            let key_node = program.nodes.get(property_node.children[0].0 as usize)?;
            let key = key_node.text.as_deref()?;
            if key == field {
                return property_node.children.get(1).copied();
            }
        }

        None
    }

    pub(crate) fn array_literal_element(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        index: usize,
    ) -> Option<LirNodeId> {
        if !self.is_array_literal(program, id) {
            return None;
        }

        let node = program.nodes.get(id.0 as usize)?;
        node.children.get(index).copied()
    }

    pub(crate) fn array_literal_length(&self, program: &LirProgram, id: LirNodeId) -> Option<usize> {
        if !self.is_array_literal(program, id) {
            return None;
        }

        let node = program.nodes.get(id.0 as usize)?;
        Some(node.children.len())
    }

    pub(crate) fn constant_array_index(
        &self,
        program: &LirProgram,
        env: &BindingEnv,
        property: &str,
    ) -> Option<usize> {
        property.parse::<usize>().ok().or_else(|| {
            env.bindings
                .get(property)
                .and_then(|bound| literal_value(program, *bound))
                .and_then(|value| match value {
                    ConstantValue::Number(value) if value >= 0 => Some(value as usize),
                    _ => None,
                })
        })
    }

    pub(crate) fn is_object_literal(&self, program: &LirProgram, id: LirNodeId) -> bool {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.is_empty() {
            return false;
        }

        node.children.iter().all(|child| {
            program
                .nodes
                .get(child.0 as usize)
                .is_some_and(|child_node| {
                    matches!(child_node.kind, LirNodeKind::Value)
                        && matches!(
                            child_node.text.as_deref(),
                            Some("init") | Some("get") | Some("set")
                        )
                        && child_node.children.len() == 2
                        && program
                            .nodes
                            .get(child_node.children[0].0 as usize)
                            .is_some_and(|key| key.kind == LirNodeKind::Literal)
                })
        })
    }

    pub(crate) fn is_array_literal(&self, program: &LirProgram, id: LirNodeId) -> bool {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind != LirNodeKind::Value || node.text.is_some() {
            return false;
        }

        !self.is_object_literal(program, id)
    }

    pub(crate) fn fold_object_has_own_call(
        &self,
        program: &mut LirProgram,
        snapshot: &LirNode,
        callee_node: &LirNode,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let callee_name = self.normalized_member_access_name(program, callee_node)?;
        if !matches!(
            callee_name.as_str(),
            "Object.hasOwn"
                | "globalThis.Object.hasOwn"
                | "Object.prototype.hasOwnProperty.call"
                | "globalThis.Object.prototype.hasOwnProperty.call"
                | "Object.hasOwnProperty.call"
                | "globalThis[\"Object\"].prototype.hasOwnProperty[\"call\"]"
                | "globalThis.Object.hasOwnProperty.call"
        ) {
            return None;
        }

        let object_id =
            self.resolve_constant_binding(program, *snapshot.children.get(1)?, bindings)?;
        if !self.is_object_literal(program, object_id) {
            return None;
        }

        let key_id =
            self.resolve_constant_binding(program, *snapshot.children.get(2)?, bindings)?;
        let key = self.constant_property_key(program, key_id)?;
        let has_own = self
            .object_literal_field(program, object_id, &key)
            .is_some();
        Some(self.clone_boolean_literal(program, has_own))
    }

    pub(crate) fn fold_object_enumeration_call(
        &self,
        program: &mut LirProgram,
        snapshot: &LirNode,
        callee_node: &LirNode,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let callee_name = self.normalized_member_access_name(program, callee_node)?;
        let string_mode = match callee_name.as_str() {
            "Object.keys" | "globalThis.Object.keys" => Some("keys"),
            "Object.values" | "globalThis.Object.values" => Some("values"),
            "Object.entries" | "globalThis.Object.entries" => Some("entries"),
            _ => None,
        };
        let is_reflect_own_keys = matches!(
            callee_name.as_str(),
            "Reflect.ownKeys" | "globalThis.Reflect.ownKeys"
        );
        if string_mode.is_none() && !is_reflect_own_keys {
            return None;
        }

        let object_id =
            self.resolve_constant_binding(program, *snapshot.children.get(1)?, bindings)?;
        if let Some(ConstantValue::String(string_text)) = literal_value(program, object_id) {
            if let Some(mode) = string_mode {
                let mut elements = Vec::with_capacity(string_text.chars().count());
                match mode {
                    "keys" => {
                        for (index, _) in string_text.chars().enumerate() {
                            elements.push(
                                self.clone_string_literal(
                                    program,
                                    format!("{:?}", index.to_string()),
                                ),
                            );
                        }
                    }
                    "values" => {
                        for value in string_text.chars() {
                            elements.push(
                                self.clone_string_literal(
                                    program,
                                    format!("{:?}", value.to_string()),
                                ),
                            );
                        }
                    }
                    "entries" => {
                        for (index, value) in string_text.chars().enumerate() {
                            let key_id = self
                                .clone_string_literal(program, format!("{:?}", index.to_string()));
                            let value_id = self
                                .clone_string_literal(program, format!("{:?}", value.to_string()));
                            elements.push(self.push_array_literal(program, vec![key_id, value_id]));
                        }
                    }
                    _ => unreachable!(),
                }
                return Some(self.push_array_literal(program, elements));
            }
        }
        if !self.is_object_literal(program, object_id) {
            return None;
        }

        let properties = self.ordered_object_literal_properties(program, object_id)?;
        match callee_name.as_str() {
            "Object.keys" | "globalThis.Object.keys" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (key, _) in properties {
                    elements.push(self.clone_string_literal(program, key));
                }
                Some(self.push_array_literal(program, elements))
            }
            "Reflect.ownKeys" | "globalThis.Reflect.ownKeys" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (key, _) in properties {
                    elements.push(self.clone_string_literal(program, key));
                }
                Some(self.push_array_literal(program, elements))
            }
            "Object.values" | "globalThis.Object.values" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (_, value) in properties {
                    elements.push(self.clone_subtree_with_substitution(
                        program,
                        value,
                        &BTreeMap::new(),
                        &mut HashMap::new(),
                    ));
                }
                Some(self.push_array_literal(program, elements))
            }
            "Object.entries" | "globalThis.Object.entries" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (key, value) in properties {
                    let key_id = self.clone_string_literal(program, key);
                    let value_id = self.clone_subtree_with_substitution(
                        program,
                        value,
                        &BTreeMap::new(),
                        &mut HashMap::new(),
                    );
                    let pair = self.push_array_literal(program, vec![key_id, value_id]);
                    elements.push(pair);
                }
                Some(self.push_array_literal(program, elements))
            }
            _ => None,
        }
    }

    pub(crate) fn fold_object_from_entries_call(
        &self,
        program: &mut LirProgram,
        snapshot: &LirNode,
        callee_node: &LirNode,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let callee_name = self.normalized_member_access_name(program, callee_node)?;
        if !matches!(
            callee_name.as_str(),
            "Object.fromEntries" | "globalThis.Object.fromEntries"
        ) {
            return None;
        }

        let entries_id =
            self.resolve_constant_binding(program, *snapshot.children.get(1)?, bindings)?;
        if !self.is_array_literal(program, entries_id) {
            return None;
        }

        let entries_node = program.nodes.get(entries_id.0 as usize)?;
        let mut properties: Vec<(String, usize, LirNodeId)> = Vec::new();
        let mut key_positions: HashMap<String, usize> = HashMap::new();
        for (entry_index, entry_id) in entries_node.children.iter().copied().enumerate() {
            let entry_id = self.resolve_constant_binding(program, entry_id, bindings)?;
            if !self.is_array_literal(program, entry_id) {
                return None;
            }

            let entry_node = program.nodes.get(entry_id.0 as usize)?;
            if entry_node.children.len() != 2 {
                return None;
            }

            let key_id =
                self.resolve_constant_binding(program, entry_node.children[0], bindings)?;
            let key = self.constant_property_key(program, key_id)?;
            let value_id =
                self.resolve_constant_binding(program, entry_node.children[1], bindings)?;

            if let Some(position) = key_positions.get(&key).copied() {
                properties[position].2 = value_id;
                continue;
            }

            key_positions.insert(key.clone(), properties.len());
            properties.push((key, entry_index, value_id));
        }

        let object_properties = properties
            .into_iter()
            .map(|(key, _, value)| (key, value))
            .collect::<Vec<_>>();
        Some(self.push_object_literal(program, object_properties))
    }

    pub(crate) fn fold_object_enumeration_calls(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        bindings: &BindingEnv,
    ) {
        let snapshot = program.nodes[id.0 as usize].clone();
        for child in snapshot.children.iter().copied() {
            self.fold_object_enumeration_calls(program, child, bindings);
        }

        if snapshot.kind != LirNodeKind::Call {
            return;
        }

        let Some(callee_id) = snapshot.children.first().copied() else {
            return;
        };
        let Some(callee_node) = program.nodes.get(callee_id.0 as usize).cloned() else {
            return;
        };
        if let Some(folded) =
            self.fold_object_enumeration_call(program, &snapshot, &callee_node, bindings)
        {
            program.nodes[id.0 as usize] = program.nodes[folded.0 as usize].clone();
        }
    }

    pub(crate) fn ordered_object_literal_properties(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> Option<Vec<(String, LirNodeId)>> {
        if !self.is_object_literal(program, id) {
            return None;
        }

        let node = program.nodes.get(id.0 as usize)?;
        let mut properties = Vec::new();
        for (source_index, property) in node.children.iter().copied().enumerate() {
            let property_node = program.nodes.get(property.0 as usize)?;
            if property_node.children.len() != 2 {
                continue;
            }
            let key_node = program.nodes.get(property_node.children[0].0 as usize)?;
            let key = key_node.text.as_deref()?.to_string();
            properties.push((key, source_index, property_node.children[1]));
        }

        properties.sort_by(|(left_key, left_index, _), (right_key, right_index, _)| {
            match (
                Self::object_property_order_key(left_key),
                Self::object_property_order_key(right_key),
            ) {
                (Some(left_order), Some(right_order)) => left_order
                    .cmp(&right_order)
                    .then_with(|| left_index.cmp(right_index)),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => left_index.cmp(right_index),
            }
        });

        Some(
            properties
                .into_iter()
                .map(|(key, _, value)| (key, value))
                .collect(),
        )
    }

    pub(crate) fn resolve_constant_binding(
        &self,
        program: &LirProgram,
        mut id: LirNodeId,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert(id.0) {
                return None;
            }

            let node = program.nodes.get(id.0 as usize)?;
            if node.kind == LirNodeKind::Value
                && node.children.len() == 1
                && node.text.as_deref().is_none_or(|text| text.is_empty())
            {
                id = node.children[0];
                continue;
            }

            if self.is_object_freeze_call(program, node) {
                id = node.children[1];
                continue;
            }

            if node.kind == LirNodeKind::Value
                && node.children.is_empty()
                && node.text.as_deref().is_some()
            {
                let name = node.text.as_deref()?;
                if let Some(bound) = bindings.bindings.get(name).copied() {
                    id = bound;
                    continue;
                }
            }

            return Some(id);
        }
    }

    pub(crate) fn is_object_freeze_call(&self, program: &LirProgram, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call || node.children.len() < 2 {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee_node) = program.nodes.get(callee.0 as usize) else {
            return false;
        };
        matches!(
            self.normalized_member_access_name(program, callee_node)
                .as_deref(),
            Some("Object.freeze") | Some("globalThis.Object.freeze")
        )
    }

    pub(crate) fn collect_constant_bindings(&self, program: &LirProgram, id: LirNodeId) -> BindingEnv {
        let mut env = BindingEnv::default();
        self.collect_constant_bindings_into(program, id, &mut env);
        env
    }

    pub(crate) fn collect_constant_bindings_into(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        env: &mut BindingEnv,
    ) {
        let snapshot = program.nodes[id.0 as usize].clone();
        if let Some((name, init)) = self.extract_const_binding(program, id) {
            let resolved = self
                .resolve_constant_binding(program, init, env)
                .unwrap_or(init);
            if self.is_specializable_binding(program, resolved) {
                env.bindings.insert(name, resolved);
            }
        }

        for child in snapshot.children {
            self.collect_constant_bindings_into(program, child, env);
        }
    }

    pub(crate) fn clone_boolean_literal(&self, program: &mut LirProgram, value: bool) -> LirNodeId {
        let node = LirNode {
            kind: LirNodeKind::Literal,
            text: Some(value.to_string()),
            children: Vec::new(),
            function_flavor: None,
        };
        let new_id = LirNodeId(program.nodes.len() as u32);
        program.nodes.push(node);
        new_id
    }

    pub(crate) fn member_access_name(&self, program: &LirProgram, node: &LirNode) -> Option<String> {
        if self.is_object_freeze_call(program, node) {
            let inner = node.children.get(1).copied()?;
            let inner = program.nodes.get(inner.0 as usize)?;
            return self.member_access_name(program, inner);
        }

        let object = node.children.first().copied()?;
        let object = program.nodes.get(object.0 as usize)?;
        let object_name = match object.text.as_deref() {
            Some(text) => text.to_string(),
            None => self.member_access_name(program, object)?,
        };

        Some(format!("{}.{}", object_name, node.text.as_deref()?))
    }

    pub(crate) fn normalized_member_access_name(
        &self,
        program: &LirProgram,
        node: &LirNode,
    ) -> Option<String> {
        let raw = self.member_access_name(program, node)?;
        Some(Self::canonicalize_optional_chain_member_access_name(
            &Self::canonicalize_bracketed_member_access_name(&raw),
        ))
    }

    pub(crate) fn canonicalize_optional_chain_member_access_name(name: &str) -> String {
        name.replace("globalThis?.", "globalThis.")
    }

    pub(crate) fn canonicalize_bracketed_member_access_name(name: &str) -> String {
        let mut canonical = String::with_capacity(name.len());
        let bytes = name.as_bytes();
        let mut index = 0;

        while index < bytes.len() {
            if bytes[index] == b'[' && index + 2 < bytes.len() {
                let quote = bytes[index + 1];
                if quote == b'"' || quote == b'\'' {
                    let mut end = index + 2;
                    while end < bytes.len() && bytes[end] != quote {
                        end += 1;
                    }

                    if end + 1 < bytes.len() && bytes[end + 1] == b']' {
                        if !canonical.is_empty() && !canonical.ends_with('.') {
                            canonical.push('.');
                        }
                        canonical.push_str(&name[index + 2..end]);
                        index = end + 2;
                        continue;
                    }
                }
            }

            canonical.push(bytes[index] as char);
            index += 1;
        }

        canonical
    }

    pub(crate) fn constant_property_key(&self, program: &LirProgram, id: LirNodeId) -> Option<String> {
        Some(literal_text(literal_value(program, id)?))
    }

    pub(crate) fn clone_string_literal(&self, program: &mut LirProgram, text: String) -> LirNodeId {
        let node = LirNode {
            kind: LirNodeKind::Literal,
            text: Some(text),
            children: Vec::new(),
            function_flavor: None,
        };
        let new_id = LirNodeId(program.nodes.len() as u32);
        program.nodes.push(node);
        new_id
    }

    pub(crate) fn push_array_literal(&self, program: &mut LirProgram, elements: Vec<LirNodeId>) -> LirNodeId {
        let new_id = LirNodeId(program.nodes.len() as u32);
        program.nodes.push(LirNode {
            kind: LirNodeKind::Value,
            text: None,
            children: elements,
            function_flavor: None,
        });
        new_id
    }

    pub(crate) fn push_object_literal(
        &self,
        program: &mut LirProgram,
        properties: Vec<(String, LirNodeId)>,
    ) -> LirNodeId {
        let mut property_nodes = Vec::with_capacity(properties.len());
        for (key, value) in properties {
            let key_id = self.clone_string_literal(program, key);
            let property_id = LirNodeId(program.nodes.len() as u32);
            program.nodes.push(LirNode {
                kind: LirNodeKind::Value,
                text: Some("init".to_string()),
                children: vec![key_id, value],
                function_flavor: None,
            });
            property_nodes.push(property_id);
        }

        let new_id = LirNodeId(program.nodes.len() as u32);
        program.nodes.push(LirNode {
            kind: LirNodeKind::Value,
            text: None,
            children: property_nodes,
            function_flavor: None,
        });
        new_id
    }

    pub(crate) fn object_property_order_key(key: &str) -> Option<u64> {
        let normalized = key.trim_matches('"');
        if normalized.is_empty() || (normalized.len() > 1 && normalized.starts_with('0')) {
            return None;
        }

        let value = normalized.parse::<u64>().ok()?;
        (value < u32::MAX as u64).then_some(value)
    }

}

#[derive(Clone, Debug, Default)]
pub(crate) struct BindingEnv {
    pub(crate) bindings: BTreeMap<String, LirNodeId>,
}

pub(crate) fn node_signature(program: &LirProgram, id: LirNodeId) -> String {
    let Some(node) = program.nodes.get(id.0 as usize) else {
        return "<missing>".to_string();
    };

    let mut signature = format!("{:?}:{:?}", node.kind, node.text);
    if !node.children.is_empty() {
        signature.push('(');
        for child in &node.children {
            signature.push_str(&node_signature(program, *child));
            signature.push(',');
        }
        signature.push(')');
    }
    signature
}

pub(crate) fn literal_signature(prefix: &str, kind: LirNodeKind, text: Option<&str>) -> String {
    match parse_literal_text(text) {
        Some(ConstantValue::Number(value)) => format!(
            "{prefix}:number:{}",
            text.map_or_else(|| value.to_string(), str::to_owned)
        ),
        Some(ConstantValue::BigInt(value)) => format!("{prefix}:bigint:{value}"),
        Some(ConstantValue::Boolean(value)) => {
            format!("{prefix}:boolean:{value}")
        }
        Some(ConstantValue::String(_)) => text
            .and_then(|text| string_literal_signature(prefix, text))
            .unwrap_or_else(|| format!("{prefix}:string:<missing>")),
        Some(ConstantValue::RegExp { pattern, flags }) => {
            format!("{prefix}:regexp:pattern={pattern}:flags={flags}")
        }
        Some(ConstantValue::Null) => format!("{prefix}:null"),
        Some(ConstantValue::Undefined) => format!("{prefix}:undefined"),
        Some(ConstantValue::NegativeZero) => format!("{prefix}:number:-0"),
        Some(ConstantValue::Infinity) => format!("{prefix}:number:Infinity"),
        Some(ConstantValue::NegativeInfinity) => format!("{prefix}:number:-Infinity"),
        Some(ConstantValue::NaN) => format!("{prefix}:number:NaN"),
        None => format!("{:?}:{:?}", kind, text),
    }
}

pub(crate) fn string_literal_signature(prefix: &str, text: &str) -> Option<String> {
    let value = parse_string_literal(text)?;
    let literal_kind = if text.starts_with('`') {
        "template"
    } else {
        "quoted"
    };
    Some(format!("{prefix}:string:{literal_kind}:{value}"))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
