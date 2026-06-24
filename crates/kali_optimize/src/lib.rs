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

mod object_fold;
pub(crate) use object_fold::*;

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
