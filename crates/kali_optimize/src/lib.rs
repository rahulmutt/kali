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

use kali_lir::{LirNode, LirNodeId, LirNodeKind, LirProgram};
use kali_mir::{LayoutDescriptor, MirBindingKind, MirProgram as MirAnalysisProgram};

/// Minimum recorded weight for a function sample to count as hot in the PGO report.
const HOT_FUNCTION_MINIMUM_WEIGHT: u64 = 8;

impl Optimizer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn specialize_layout_bindings(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        tracker: &mut SpecializationTracker,
        owner: &str,
        scope: &str,
        plan: &SpecializationPlan,
        env: &mut BindingEnv,
    ) {
        let snapshot = program.nodes[id.0 as usize].clone();
        let is_function_scope = matches!(snapshot.kind, LirNodeKind::Instruction)
            && snapshot
                .text
                .as_deref()
                .is_some_and(|name| plan.functions.contains_key(name));

        match snapshot.kind {
            LirNodeKind::Program | LirNodeKind::Block => {
                let mut local_env = env.clone();
                let next_owner = if is_function_scope {
                    snapshot.text.as_deref().unwrap_or(owner)
                } else {
                    owner
                };
                let next_scope = if is_function_scope {
                    snapshot.text.as_deref().unwrap_or(scope)
                } else {
                    scope
                };
                for child in snapshot.children {
                    self.specialize_layout_bindings(
                        program,
                        child,
                        tracker,
                        next_owner,
                        next_scope,
                        plan,
                        &mut local_env,
                    );

                    if let Some((name, init)) = self.extract_const_binding(program, child) {
                        if self.is_specializable_binding(program, init) {
                            local_env.bindings.insert(name, init);
                        }
                    }
                }
                return;
            }
            LirNodeKind::Instruction if is_function_scope => {
                let mut local_env = env.clone();
                let next_owner = snapshot.text.as_deref().unwrap_or(owner);
                let next_scope = snapshot.text.as_deref().unwrap_or(scope);
                for child in snapshot.children {
                    self.specialize_layout_bindings(
                        program,
                        child,
                        tracker,
                        next_owner,
                        next_scope,
                        plan,
                        &mut local_env,
                    );

                    if let Some((name, init)) = self.extract_const_binding(program, child) {
                        if self.is_specializable_binding(program, init) {
                            local_env.bindings.insert(name, init);
                        }
                    }
                }
                return;
            }
            _ => {}
        }

        if snapshot.kind == LirNodeKind::Value && snapshot.children.is_empty() {
            if let Some(name) = snapshot.text.as_deref() {
                if let Some(bound) = env.bindings.get(name).copied() {
                    let key = format!("bind:{}:{}", name, node_signature(program, bound));
                    if tracker.allow(owner, key) {
                        program.nodes[id.0 as usize] = program.nodes[bound.0 as usize].clone();
                        self.specialize_layout_bindings(
                            program, id, tracker, owner, scope, plan, env,
                        );
                    }
                }
            }
            return;
        }

        for child in snapshot.children {
            self.specialize_layout_bindings(program, child, tracker, owner, scope, plan, env);
        }

        let _ = self.fold_layout_member_access(program, id, tracker, owner, env);
    }

    pub(crate) fn extract_const_binding(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> Option<(String, LirNodeId)> {
        let node = program.nodes.get(id.0 as usize)?;
        if node.kind != LirNodeKind::Instruction {
            return None;
        }
        if node.text.as_deref() != Some("const") {
            return None;
        }

        for declarator in &node.children {
            let declarator_node = program.nodes.get(declarator.0 as usize)?;
            if declarator_node.kind != LirNodeKind::Instruction {
                continue;
            }
            let Some(name) = declarator_node.text.clone() else {
                continue;
            };
            let Some(init) = declarator_node.children.get(1).copied() else {
                continue;
            };
            return Some((name, init));
        }

        None
    }

    pub(crate) fn is_specializable_binding(&self, program: &LirProgram, id: LirNodeId) -> bool {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return false;
        };

        match node.kind {
            LirNodeKind::Literal => true,
            LirNodeKind::Value if node.children.is_empty() => node
                .text
                .as_deref()
                .and_then(|text| parse_literal_text(Some(text)))
                .is_some(),
            LirNodeKind::Value if node.text.is_none() => {
                self.is_object_literal(program, id) || self.is_array_literal(program, id)
            }
            _ => false,
        }
    }

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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn optimize_call_site(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        plan: &SpecializationPlan,
        tracker: &mut SpecializationTracker,
        owner: &str,
        allow_generic_specialization: bool,
        specialized_functions: &mut BTreeMap<String, LirNodeId>,
        bindings: &BindingEnv,
    ) -> bool {
        let snapshot = program.nodes[id.0 as usize].clone();
        if snapshot.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee_id) = snapshot.children.first().copied() else {
            return false;
        };
        let Some(callee_id) = self.resolve_constant_binding(program, callee_id, bindings) else {
            return false;
        };
        let Some(callee_node) = program.nodes.get(callee_id.0 as usize).cloned() else {
            return false;
        };
        let Some(callee_name) = callee_node.text.as_deref() else {
            return false;
        };

        if let Some(folded) =
            self.fold_object_has_own_call(program, &snapshot, &callee_node, bindings)
        {
            let key = format!(
                "object-has-own:{}:{}",
                callee_name,
                self.call_signature(program, &snapshot)
            );
            if tracker.allow(owner, key) {
                program.nodes[id.0 as usize] = program.nodes[folded.0 as usize].clone();
                return true;
            }
        }

        if let Some(folded) =
            self.fold_object_enumeration_call(program, &snapshot, &callee_node, bindings)
        {
            let key = format!(
                "object-enumeration:{}:{}",
                callee_name,
                self.call_signature(program, &snapshot)
            );
            if tracker.allow(owner, key) {
                program.nodes[id.0 as usize] = program.nodes[folded.0 as usize].clone();
                return true;
            }
        }

        if let Some(folded) =
            self.fold_object_from_entries_call(program, &snapshot, &callee_node, bindings)
        {
            let key = format!(
                "object-from-entries:{}:{}",
                callee_name,
                self.call_signature(program, &snapshot)
            );
            if tracker.allow(owner, key) {
                program.nodes[id.0 as usize] = program.nodes[folded.0 as usize].clone();
                return true;
            }
        }

        let Some(summary) = plan.functions.get(callee_name) else {
            return false;
        };
        let args: Vec<LirNodeId> = snapshot.children.iter().skip(1).copied().collect();
        if args.len() != summary.params.len() {
            return false;
        }

        if let Some(inline_body) = summary.inline_body {
            let inline_threshold = self.inline_threshold_for_function(callee_name);
            if summary.node_count <= inline_threshold && !summary.recursive {
                let key = format!(
                    "inline:{}:{}",
                    callee_name,
                    self.call_signature(program, &snapshot)
                );
                if tracker.allow(owner, key) {
                    let cloned_root =
                        self.inline_call_site(program, inline_body, &summary.params, &args);
                    let replacement = program.nodes[cloned_root.0 as usize].clone();
                    program.nodes[id.0 as usize] = replacement;
                    return true;
                }
            }
        }

        if !allow_generic_specialization {
            return false;
        }

        let mut substitutions = BTreeMap::new();
        let mut signature_parts = Vec::new();
        let mut saw_concrete_argument = false;
        for (param, arg) in summary.params.iter().zip(args.iter()) {
            let arg_signature = self.specialization_signature(program, *arg);
            if self.argument_has_concrete_shape(program, *arg) {
                signature_parts.push(format!("generic:{}", arg_signature));
                let cloned_arg = self.clone_subtree_with_substitution(
                    program,
                    *arg,
                    &BTreeMap::new(),
                    &mut HashMap::new(),
                );
                substitutions.insert(param.clone(), cloned_arg);
                saw_concrete_argument = true;
            } else {
                signature_parts.push(arg_signature);
            }
        }

        if !saw_concrete_argument {
            return false;
        }

        let specialization_key =
            format!("specialize:{}:{}", callee_name, signature_parts.join("|"));
        let specialized_name = self.specialized_function_name(callee_name, &signature_parts);
        if specialized_functions.contains_key(&specialized_name) {
            if let Some(callee) = program.nodes.get_mut(callee_id.0 as usize) {
                callee.text = Some(specialized_name);
            }
            return true;
        }
        if !tracker.allow(owner, specialization_key) {
            return false;
        }

        let new_id = self.clone_specialized_function(
            program,
            summary,
            specialized_name.clone(),
            &substitutions,
        );
        specialized_functions.insert(specialized_name.clone(), new_id);
        program.nodes[program.root.0 as usize].children.push(new_id);
        self.specialize_layout_bindings(
            program,
            new_id,
            tracker,
            &specialized_name,
            callee_name,
            plan,
            &mut BindingEnv::default(),
        );
        self.optimize_node(
            program,
            new_id,
            plan,
            tracker,
            specialized_name.clone(),
            allow_generic_specialization,
            specialized_functions,
            &BindingEnv::default(),
        );

        if let Some(callee) = program.nodes.get_mut(callee_id.0 as usize) {
            callee.text = Some(specialized_name);
        }

        true
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn specialize_mir_call_sites(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        plan: &SpecializationPlan,
        mir_plan: &MirSpecializationPlan,
        tracker: &mut SpecializationTracker,
        owner: String,
        scope: String,
        specialized_functions: &mut BTreeMap<String, LirNodeId>,
    ) {
        let snapshot = program.nodes[id.0 as usize].clone();
        let next_owner = match snapshot.kind {
            LirNodeKind::Instruction => snapshot
                .text
                .as_deref()
                .filter(|name| plan.functions.contains_key(*name))
                .map(|name| name.to_string())
                .unwrap_or_else(|| owner.clone()),
            _ => owner.clone(),
        };
        let next_scope = match snapshot.kind {
            LirNodeKind::Instruction => snapshot
                .text
                .as_deref()
                .filter(|name| plan.functions.contains_key(*name))
                .map(|name| name.to_string())
                .unwrap_or_else(|| scope.clone()),
            _ => scope.clone(),
        };

        if let Some((new_function, callee_scope)) = self.specialize_mir_call_site(
            program,
            id,
            plan,
            mir_plan,
            tracker,
            &owner,
            &scope,
            specialized_functions,
        ) {
            let recursive_owner = program.nodes[new_function.0 as usize]
                .text
                .clone()
                .unwrap_or_else(|| next_owner.clone());
            self.optimize_node(
                program,
                new_function,
                plan,
                tracker,
                recursive_owner.clone(),
                true,
                specialized_functions,
                &BindingEnv::default(),
            );
            self.specialize_mir_call_sites(
                program,
                new_function,
                plan,
                mir_plan,
                tracker,
                recursive_owner,
                callee_scope,
                specialized_functions,
            );
        }

        for child in snapshot.children {
            self.specialize_mir_call_sites(
                program,
                child,
                plan,
                mir_plan,
                tracker,
                next_owner.clone(),
                next_scope.clone(),
                specialized_functions,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn specialize_mir_call_site(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        plan: &SpecializationPlan,
        mir_plan: &MirSpecializationPlan,
        tracker: &mut SpecializationTracker,
        owner: &str,
        scope: &str,
        specialized_functions: &mut BTreeMap<String, LirNodeId>,
    ) -> Option<(LirNodeId, String)> {
        let snapshot = program.nodes[id.0 as usize].clone();
        if snapshot.kind != LirNodeKind::Call {
            return None;
        }

        let callee_id = snapshot.children.first().copied()?;
        let callee_node = program.nodes.get(callee_id.0 as usize).cloned()?;
        let callee_name = callee_node.text.as_deref()?;
        let summary = plan.functions.get(callee_name)?;
        if summary.recursive {
            return None;
        }

        let args: Vec<LirNodeId> = snapshot.children.iter().skip(1).copied().collect();
        if args.len() != summary.params.len() {
            return None;
        }

        let mut substitutions = BTreeMap::new();
        let mut signature_parts = Vec::new();
        for (index, (param, arg)) in summary.params.iter().zip(args.iter()).enumerate() {
            let Some(layout) = mir_plan.parameter_layout_any(callee_name, index) else {
                let arg_signature =
                    self.specialization_signature_with_mir(program, *arg, mir_plan, scope);
                if self.argument_has_concrete_shape(program, *arg) {
                    signature_parts.push(format!("generic:{}", arg_signature));
                    let cloned_arg = self.clone_subtree_with_substitution(
                        program,
                        *arg,
                        &BTreeMap::new(),
                        &mut HashMap::new(),
                    );
                    substitutions.insert(param.clone(), cloned_arg);
                } else {
                    signature_parts.push(arg_signature);
                }
                continue;
            };

            let arg_signature =
                self.specialization_signature_with_mir(program, *arg, mir_plan, scope);
            if layout.kind == MirLayoutClass::TaggedVal {
                if !self.argument_has_concrete_layout(program, *arg, mir_plan, scope) {
                    signature_parts.push(arg_signature);
                } else {
                    signature_parts.push(format!("tagged:{}", arg_signature));
                    let cloned_arg = self.clone_subtree_with_substitution(
                        program,
                        *arg,
                        &BTreeMap::new(),
                        &mut HashMap::new(),
                    );
                    substitutions.insert(param.clone(), cloned_arg);
                }
                continue;
            }

            signature_parts.push(format!("{}:{}", layout.kind.as_str(), arg_signature));
            let cloned_arg = self.clone_subtree_with_substitution(
                program,
                *arg,
                &BTreeMap::new(),
                &mut HashMap::new(),
            );
            substitutions.insert(param.clone(), cloned_arg);
        }

        if substitutions.is_empty() {
            return None;
        }

        let specialized_name = self.specialized_function_name(callee_name, &signature_parts);
        if specialized_functions.contains_key(&specialized_name) {
            if let Some(callee) = program.nodes.get_mut(callee_id.0 as usize) {
                callee.text = Some(specialized_name);
            }
            return None;
        }

        let specialization_key =
            format!("specialize:{}:{}", callee_name, signature_parts.join("|"));
        if !tracker.allow(owner, specialization_key) {
            return None;
        }

        let new_id = self.clone_specialized_function(
            program,
            summary,
            specialized_name.clone(),
            &substitutions,
        );
        specialized_functions.insert(specialized_name.clone(), new_id);
        program.nodes[program.root.0 as usize].children.push(new_id);
        self.specialize_layout_bindings(
            program,
            new_id,
            tracker,
            &specialized_name,
            callee_name,
            plan,
            &mut BindingEnv::default(),
        );

        if let Some(callee) = program.nodes.get_mut(callee_id.0 as usize) {
            callee.text = Some(specialized_name);
        }

        Some((new_id, callee_name.to_string()))
    }

    pub(crate) fn clone_specialized_function(
        &self,
        program: &mut LirProgram,
        summary: &FunctionSummary,
        specialized_name: String,
        substitutions: &BTreeMap<String, LirNodeId>,
    ) -> LirNodeId {
        let original = program.nodes[summary.node_id.0 as usize].clone();
        let mut children = original.children.clone();
        let cloned_body = self.clone_subtree_with_substitution(
            program,
            summary.body_block,
            substitutions,
            &mut HashMap::new(),
        );
        if let Some(last_child) = children.last_mut() {
            *last_child = cloned_body;
        }

        let new_id = LirNodeId(program.nodes.len() as u32);
        program.nodes.push(LirNode {
            kind: original.kind,
            text: Some(specialized_name),
            children,
            function_flavor: original.function_flavor,
        });
        new_id
    }

    pub(crate) fn specialized_function_name(&self, callee_name: &str, signature_parts: &[String]) -> String {
        let mut hash = 0xcbf29ce484222325u64;
        let signature = signature_parts.join("|");
        for byte in signature.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("{}$spec${:016x}", callee_name, hash)
    }

    pub(crate) fn argument_has_concrete_layout(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        mir_plan: &MirSpecializationPlan,
        scope: &str,
    ) -> bool {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return false;
        };

        if self.argument_has_concrete_shape(program, id) {
            return true;
        }

        if node.kind == LirNodeKind::Value && node.children.is_empty() {
            if let Some(text) = node.text.as_deref() {
                if let Some(layout) = mir_plan.binding_layout(scope, text) {
                    return layout.kind != MirLayoutClass::TaggedVal;
                }
            }
            return false;
        }

        false
    }

    pub(crate) fn argument_has_concrete_shape(&self, program: &LirProgram, id: LirNodeId) -> bool {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return false;
        };

        if matches!(node.kind, LirNodeKind::Literal) {
            return true;
        }

        if node.kind == LirNodeKind::Value && node.children.is_empty() {
            if let Some(text) = node.text.as_deref() {
                return parse_literal_text(Some(text)).is_some();
            }
            return false;
        }

        if node.kind == LirNodeKind::Value && node.text.is_none() {
            return self.is_object_literal(program, id) || self.is_array_literal(program, id);
        }

        false
    }

    pub(crate) fn specialization_signature_with_mir(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        mir_plan: &MirSpecializationPlan,
        scope: &str,
    ) -> String {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return "<missing>".to_string();
        };

        if node.children.is_empty() {
            if let Some(text) = node.text.as_deref() {
                if let Some(layout) = mir_plan.binding_layout(scope, text) {
                    return format!("binding:{}", layout.key());
                }
            }
        }

        let mut signature = match node.kind {
            LirNodeKind::Literal => {
                literal_signature("Literal", node.kind.clone(), node.text.as_deref())
            }
            LirNodeKind::Value if node.children.is_empty() => {
                literal_signature("Value", node.kind.clone(), node.text.as_deref())
            }
            LirNodeKind::Value if self.is_object_literal(program, id) => {
                self.object_literal_signature(program, id, mir_plan, scope)
            }
            LirNodeKind::Value if self.is_array_literal(program, id) => {
                self.array_literal_signature(program, id, mir_plan, scope)
            }
            _ => format!("{:?}:{:?}", node.kind, node.text),
        };

        if !node.children.is_empty() && !matches!(node.kind, LirNodeKind::Value)
            || (matches!(node.kind, LirNodeKind::Value)
                && !self.is_object_literal(program, id)
                && !self.is_array_literal(program, id))
        {
            signature.push('(');
            for child in &node.children {
                signature.push_str(
                    &self.specialization_signature_with_mir(program, *child, mir_plan, scope),
                );
                signature.push(',');
            }
            signature.push(')');
        }

        signature
    }

    pub(crate) fn object_literal_signature(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        mir_plan: &MirSpecializationPlan,
        scope: &str,
    ) -> String {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return "<missing>".to_string();
        };

        let mut property_signatures = Vec::with_capacity(node.children.len());
        for child in &node.children {
            property_signatures
                .push(self.object_property_signature(program, *child, mir_plan, scope));
        }
        property_signatures.sort();

        let mut signature = format!("Value:object:len={}", node.children.len());
        signature.push('(');
        for property_signature in property_signatures {
            signature.push_str(&property_signature);
            signature.push(',');
        }
        signature.push(')');
        signature
    }

    pub(crate) fn array_literal_signature(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        mir_plan: &MirSpecializationPlan,
        scope: &str,
    ) -> String {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return "<missing>".to_string();
        };

        let mut signature = format!("Value:array:len={}", node.children.len());
        signature.push('(');
        for child in &node.children {
            signature.push_str(
                &self.specialization_signature_with_mir(program, *child, mir_plan, scope),
            );
            signature.push(',');
        }
        signature.push(')');
        signature
    }

    pub(crate) fn object_property_signature(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        mir_plan: &MirSpecializationPlan,
        scope: &str,
    ) -> String {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return "<missing>".to_string();
        };

        if node.kind == LirNodeKind::Value
            && matches!(
                node.text.as_deref(),
                Some("init") | Some("get") | Some("set")
            )
            && node.children.len() == 2
        {
            let key = program
                .nodes
                .get(node.children[0].0 as usize)
                .and_then(|key| key.text.as_deref())
                .unwrap_or("<key>");
            let value =
                self.specialization_signature_with_mir(program, node.children[1], mir_plan, scope);
            return format!("{key}:{value}");
        }

        self.specialization_signature_with_mir(program, id, mir_plan, scope)
    }

    pub(crate) fn build_specialization_plan(&self, program: &LirProgram) -> SpecializationPlan {
        let mut plan = SpecializationPlan::default();
        let mut visited = HashSet::new();
        self.collect_specialization_plan(program, program.root, &mut visited, &mut plan);
        plan
    }

    pub(crate) fn collect_specialization_plan(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        visited: &mut HashSet<LirNodeId>,
        plan: &mut SpecializationPlan,
    ) {
        if !visited.insert(id) {
            return;
        }

        if let Some(summary) = self.function_summary(program, id) {
            plan.functions.insert(summary.name.clone(), summary);
        }

        let children = program
            .nodes
            .get(id.0 as usize)
            .map(|node| node.children.clone())
            .unwrap_or_default();
        for child in children {
            self.collect_specialization_plan(program, child, visited, plan);
        }
    }

    pub(crate) fn function_summary(&self, program: &LirProgram, id: LirNodeId) -> Option<FunctionSummary> {
        let node = program.nodes.get(id.0 as usize)?;
        if node.kind != LirNodeKind::Instruction {
            return None;
        }

        let name = node.text.clone()?;
        if node.children.len() < 2 {
            return None;
        }

        let block_id = *node.children.last()?;
        let block = program.nodes.get(block_id.0 as usize)?;
        if block.kind != LirNodeKind::Block {
            return None;
        }

        let mut params = Vec::new();
        for child in node.children.iter().take(node.children.len() - 1) {
            let child_node = program.nodes.get(child.0 as usize)?;
            if let Some(text) = &child_node.text {
                params.push(text.clone());
            }
        }

        let inline_body = self.extract_inline_body(program, block_id);
        let node_count = inline_body
            .map(|body| self.count_subtree_nodes(program, body))
            .unwrap_or(0);
        let recursive = inline_body
            .map(|body| self.contains_call_target(program, body, &name))
            .unwrap_or(false);

        Some(FunctionSummary {
            node_id: id,
            name,
            params,
            body_block: block_id,
            inline_body,
            node_count,
            recursive,
        })
    }

    pub(crate) fn extract_inline_body(&self, program: &LirProgram, block_id: LirNodeId) -> Option<LirNodeId> {
        let block = program.nodes.get(block_id.0 as usize)?;
        if block.kind != LirNodeKind::Block || block.children.len() != 1 {
            return None;
        }

        let child_id = block.children[0];
        let child = program.nodes.get(child_id.0 as usize)?;
        match child.kind {
            LirNodeKind::Instruction if child.text.as_deref() == Some("return") => {
                child.children.first().copied()
            }
            LirNodeKind::Literal | LirNodeKind::Value | LirNodeKind::Call | LirNodeKind::Branch => {
                Some(child_id)
            }
            _ => None,
        }
    }

    pub(crate) fn count_subtree_nodes(&self, program: &LirProgram, id: LirNodeId) -> usize {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return 0;
        };

        let mut count = 1;
        for child in &node.children {
            count += self.count_subtree_nodes(program, *child);
        }
        count
    }

    pub(crate) fn contains_call_target(&self, program: &LirProgram, id: LirNodeId, target: &str) -> bool {
        let mut targets = BTreeSet::new();
        self.collect_call_targets(program, id, &mut targets);
        targets.contains(target)
    }

    pub(crate) fn collect_call_targets(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        targets: &mut BTreeSet<String>,
    ) {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return;
        };

        if node.kind == LirNodeKind::Call {
            if let Some(callee) = node.children.first().copied() {
                if let Some(callee_node) = program.nodes.get(callee.0 as usize) {
                    if let Some(name) = callee_node.text.as_deref() {
                        targets.insert(name.to_string());
                    }
                }
            }
        }

        for child in &node.children {
            self.collect_call_targets(program, *child, targets);
        }
    }

    pub(crate) fn prune_dead_top_level_functions(&self, program: &mut LirProgram) {
        let root_id = program.root;
        let root_children = program.nodes[root_id.0 as usize].children.clone();
        let mut top_level_functions = BTreeMap::<String, FunctionSummary>::new();
        for child in &root_children {
            if let Some(summary) = self.function_summary(program, *child) {
                top_level_functions.insert(summary.name.clone(), summary);
            }
        }

        let mut live = BTreeSet::new();
        let mut worklist = Vec::new();
        for child in &root_children {
            if self.function_summary(program, *child).is_none() {
                let mut targets = BTreeSet::new();
                self.collect_call_targets(program, *child, &mut targets);
                for target in targets {
                    if top_level_functions.contains_key(&target) {
                        worklist.push(target);
                    }
                }
            }
        }

        while let Some(name) = worklist.pop() {
            if !live.insert(name.clone()) {
                continue;
            }

            let Some(summary) = top_level_functions.get(&name) else {
                continue;
            };
            let mut targets = BTreeSet::new();
            self.collect_call_targets(program, summary.body_block, &mut targets);
            for target in targets {
                if top_level_functions.contains_key(&target) && !live.contains(&target) {
                    worklist.push(target);
                }
            }
        }

        let mut filtered = Vec::with_capacity(root_children.len());
        for child in root_children {
            if let Some(summary) = self.function_summary(program, child) {
                if live.contains(&summary.name) {
                    filtered.push(child);
                }
            } else {
                filtered.push(child);
            }
        }

        program.nodes[root_id.0 as usize].children = filtered;
    }

    pub(crate) fn inline_call_site(
        &self,
        program: &mut LirProgram,
        body_root: LirNodeId,
        params: &[String],
        args: &[LirNodeId],
    ) -> LirNodeId {
        let substitutions: BTreeMap<String, LirNodeId> =
            params.iter().cloned().zip(args.iter().copied()).collect();
        let mut memo = HashMap::new();
        self.clone_subtree_with_substitution(program, body_root, &substitutions, &mut memo)
    }

    pub(crate) fn clone_subtree_with_substitution(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        substitutions: &BTreeMap<String, LirNodeId>,
        memo: &mut HashMap<LirNodeId, LirNodeId>,
    ) -> LirNodeId {
        let snapshot = program.nodes[id.0 as usize].clone();
        if snapshot.kind == LirNodeKind::Value && snapshot.children.is_empty() {
            if let Some(name) = snapshot.text.as_deref() {
                if let Some(&replacement) = substitutions.get(name) {
                    return replacement;
                }
            }
        }

        if let Some(existing) = memo.get(&id).copied() {
            return existing;
        }

        let mut children = Vec::with_capacity(snapshot.children.len());
        for child in snapshot.children {
            children.push(self.clone_subtree_with_substitution(
                program,
                child,
                substitutions,
                memo,
            ));
        }

        let new_id = LirNodeId(program.nodes.len() as u32);
        program.nodes.push(LirNode {
            kind: snapshot.kind,
            text: snapshot.text,
            children,
            function_flavor: snapshot.function_flavor,
        });
        memo.insert(id, new_id);
        new_id
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

    pub(crate) fn inline_threshold_for_function(&self, callee_name: &str) -> usize {
        let base_threshold: usize = match self.level {
            OptimizationLevel::Release => 12,
            OptimizationLevel::ReleaseAdvanced => 24,
            _ => 0,
        };

        if base_threshold == 0 {
            return 0;
        }

        if self.is_hot_function(callee_name) {
            base_threshold.saturating_mul(2)
        } else {
            base_threshold
        }
    }

    pub(crate) fn is_hot_function(&self, callee_name: &str) -> bool {
        self.profile_data
            .as_ref()
            .and_then(|profile| profile.sample_weight(ProfileSampleKind::Function, callee_name))
            .is_some_and(|weight| weight >= HOT_FUNCTION_MINIMUM_WEIGHT)
    }

    pub(crate) fn profile_has_hot_branch_or_layout_hints(&self) -> bool {
        self.profile_data.as_ref().is_some_and(|profile| {
            !profile
                .hot_keys(ProfileSampleKind::Branch, HOT_FUNCTION_MINIMUM_WEIGHT)
                .is_empty()
                || !profile
                    .hot_keys(ProfileSampleKind::Layout, HOT_FUNCTION_MINIMUM_WEIGHT)
                    .is_empty()
        })
    }

    pub(crate) fn call_signature(&self, program: &LirProgram, node: &LirNode) -> String {
        let callee = node
            .children
            .first()
            .and_then(|child| program.nodes.get(child.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .unwrap_or("<unknown>");

        let mut signature = String::from(callee);
        signature.push('(');
        for child in node.children.iter().skip(1) {
            signature.push_str(&self.specialization_signature(program, *child));
            signature.push(',');
        }
        signature.push(')');
        signature
    }

    pub(crate) fn specialization_signature(&self, program: &LirProgram, id: LirNodeId) -> String {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return "<missing>".to_string();
        };

        let mut signature = match node.kind {
            LirNodeKind::Literal => {
                literal_signature("Literal", node.kind.clone(), node.text.as_deref())
            }
            LirNodeKind::Value if node.children.is_empty() => {
                literal_signature("Value", node.kind.clone(), node.text.as_deref())
            }
            _ => format!("{:?}:{:?}", node.kind, node.text),
        };

        if !node.children.is_empty() {
            signature.push('(');
            for child in &node.children {
                signature.push_str(&self.specialization_signature(program, *child));
                signature.push(',');
            }
            signature.push(')');
        }

        signature
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MirLayoutClass {
    Scalar,
    Struct,
    Array,
    Closure,
    TaggedVal,
}

impl MirLayoutClass {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            MirLayoutClass::Scalar => "scalar",
            MirLayoutClass::Struct => "struct",
            MirLayoutClass::Array => "array",
            MirLayoutClass::Closure => "closure",
            MirLayoutClass::TaggedVal => "tagged",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MirLayoutSignature {
    pub(crate) kind: MirLayoutClass,
    pub(crate) fingerprint: String,
}

impl MirLayoutSignature {
    pub(crate) fn from_descriptor(descriptor: &LayoutDescriptor) -> Self {
        Self {
            kind: MirLayoutClass::from_descriptor(descriptor),
            fingerprint: descriptor.fingerprint(),
        }
    }

    pub(crate) fn key(&self) -> String {
        format!("{}:{}", self.kind.as_str(), self.fingerprint)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MirSpecializationPlan {
    pub(crate) scoped_binding_layouts: BTreeMap<String, BTreeMap<String, MirLayoutSignature>>,
    pub(crate) parameter_layouts: BTreeMap<String, Vec<MirLayoutSignature>>,
}

impl MirSpecializationPlan {
    pub(crate) fn from_program(mir: &MirAnalysisProgram) -> Self {
        let mut scoped_binding_layouts = BTreeMap::new();
        let mut parameter_layouts = BTreeMap::new();

        for function in &mir.functions {
            let scope = function.name.as_deref().unwrap_or("<module>").to_string();
            let binding_layouts = scoped_binding_layouts.entry(scope.clone()).or_default();
            let mut params = Vec::new();

            for binding in &function.bindings {
                let layout = MirLayoutSignature::from_descriptor(&binding.layout);
                Self::record_layout(binding_layouts, &binding.name, layout.clone());

                if binding.kind == MirBindingKind::Parameter {
                    params.push(layout);
                }
            }

            if function.name.is_some() {
                parameter_layouts.insert(scope, params);
            }
        }

        Self {
            scoped_binding_layouts,
            parameter_layouts,
        }
    }

    pub(crate) fn binding_layout(&self, scope: &str, name: &str) -> Option<MirLayoutSignature> {
        self.scoped_binding_layouts
            .get(scope)
            .and_then(|bindings| bindings.get(name))
            .cloned()
            .or_else(|| {
                self.scoped_binding_layouts
                    .get("<module>")
                    .and_then(|bindings| bindings.get(name))
                    .cloned()
            })
    }

    pub(crate) fn parameter_layout_any(&self, function: &str, index: usize) -> Option<MirLayoutSignature> {
        self.parameter_layouts
            .get(function)
            .and_then(|layouts| layouts.get(index).cloned())
    }

    pub(crate) fn record_layout(
        binding_layouts: &mut BTreeMap<String, MirLayoutSignature>,
        binding_name: &str,
        layout: MirLayoutSignature,
    ) {
        binding_layouts
            .entry(binding_name.to_string())
            .and_modify(|existing| {
                if *existing != layout {
                    *existing = Self::tagged_layout_signature();
                }
            })
            .or_insert(layout);
    }

    pub(crate) fn tagged_layout_signature() -> MirLayoutSignature {
        MirLayoutSignature {
            kind: MirLayoutClass::TaggedVal,
            fingerprint: LayoutDescriptor::TaggedVal.fingerprint(),
        }
    }
}

impl MirLayoutClass {
    pub(crate) fn from_descriptor(descriptor: &LayoutDescriptor) -> Self {
        match descriptor {
            LayoutDescriptor::Scalar(_) => MirLayoutClass::Scalar,
            LayoutDescriptor::Struct { .. } => MirLayoutClass::Struct,
            LayoutDescriptor::Array { .. } => MirLayoutClass::Array,
            LayoutDescriptor::Closure { .. } => MirLayoutClass::Closure,
            LayoutDescriptor::TaggedVal => MirLayoutClass::TaggedVal,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SpecializationPlan {
    pub(crate) functions: BTreeMap<String, FunctionSummary>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BindingEnv {
    pub(crate) bindings: BTreeMap<String, LirNodeId>,
}

#[derive(Clone, Debug)]
pub(crate) struct FunctionSummary {
    pub(crate) node_id: LirNodeId,
    pub(crate) name: String,
    pub(crate) params: Vec<String>,
    pub(crate) body_block: LirNodeId,
    pub(crate) inline_body: Option<LirNodeId>,
    pub(crate) node_count: usize,
    pub(crate) recursive: bool,
}

#[derive(Debug)]
pub(crate) struct SpecializationTracker {
    pub(crate) max_specializations: usize,
    pub(crate) seen: BTreeMap<String, BTreeSet<String>>,
}

impl SpecializationTracker {
    pub(crate) fn new(max_specializations: usize) -> Self {
        Self {
            max_specializations,
            seen: BTreeMap::new(),
        }
    }

    pub(crate) fn allow(&mut self, owner: impl Into<String>, key: String) -> bool {
        let owner = owner.into();
        let seen = self.seen.entry(owner).or_default();
        if seen.contains(&key) {
            return true;
        }

        if seen.len() >= self.max_specializations {
            return false;
        }

        seen.insert(key);
        true
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
