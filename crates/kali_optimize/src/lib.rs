//! Optimization passes for the Kali compiler.
//!
//! The current implementation focuses on the deterministic, tree-shaped LIR
//! that the rest of the repository already produces. That gives us a safe place
//! to land constant folding, branch elimination, and a handful of algebraic
//! simplifications without needing a full SSA pipeline yet.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use kali_lir::{LirNode, LirNodeId, LirNodeKind, LirProgram};
use kali_mir::{LayoutDescriptor, MirBindingKind, MirProgram as MirAnalysisProgram};

/// Optimization level.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// Skip optimization work.
    Fast,
    /// Apply the baseline optimization set.
    Release,
    /// Apply the baseline set plus more aggressive algebraic simplifications.
    ReleaseAdvanced,

    #[default]
    Default,
}

/// Optimizer context.
#[derive(Clone, Debug)]
pub struct Optimizer {
    level: OptimizationLevel,
    max_specializations: usize,
}

impl Optimizer {
    /// Create a new optimizer for the requested level.
    pub fn new(level: OptimizationLevel) -> Self {
        Self {
            level,
            max_specializations: 16,
        }
    }

    /// Override the specialization cap placeholder used by later phases.
    pub fn with_max_specializations(level: OptimizationLevel, max_specializations: usize) -> Self {
        Self {
            level,
            max_specializations,
        }
    }

    /// Return the configured specialization cap.
    pub fn max_specializations(&self) -> usize {
        self.max_specializations
    }

    /// Optimize a program in place.
    pub fn optimize_program(&self, program: &mut LirProgram) {
        match self.level {
            OptimizationLevel::Fast | OptimizationLevel::Default => {}
            OptimizationLevel::Release | OptimizationLevel::ReleaseAdvanced => {
                let mut tracker = SpecializationTracker::new(self.max_specializations);
                let mut binding_env = BindingEnv::default();
                self.specialize_layout_bindings(
                    program,
                    program.root,
                    &mut tracker,
                    "<root>",
                    &mut binding_env,
                );

                let plan = self.build_specialization_plan(program);
                self.optimize_node(
                    program,
                    program.root,
                    &plan,
                    &mut tracker,
                    "<root>".to_string(),
                );

                if matches!(self.level, OptimizationLevel::ReleaseAdvanced) {
                    self.prune_dead_top_level_functions(program);
                }
            }
        }
    }

    /// Optimize a program using MIR layout metadata to drive additional call-site specialization.
    pub fn optimize_program_with_mir(&self, program: &mut LirProgram, mir: &MirAnalysisProgram) {
        self.optimize_program(program);

        if matches!(
            self.level,
            OptimizationLevel::Fast | OptimizationLevel::Default
        ) {
            return;
        }

        let plan = self.build_specialization_plan(program);
        let mir_plan = MirSpecializationPlan::from_program(mir);
        let mut tracker = SpecializationTracker::new(self.max_specializations);
        let mut specialized_functions = BTreeMap::new();
        self.specialize_mir_call_sites(
            program,
            program.root,
            &plan,
            &mir_plan,
            &mut tracker,
            "<root>".to_string(),
            &mut specialized_functions,
        );

        if matches!(self.level, OptimizationLevel::ReleaseAdvanced) {
            self.prune_dead_top_level_functions(program);
        }
    }

    fn optimize_node(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        plan: &SpecializationPlan,
        tracker: &mut SpecializationTracker,
        owner: String,
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

        for child in snapshot.children.iter().copied() {
            self.optimize_node(program, child, plan, tracker, next_owner.clone());
        }

        if matches!(snapshot.kind, LirNodeKind::Program | LirNodeKind::Block) {
            self.optimize_sequence(program, id);
        }

        if self.optimize_constant_expression(program, id, tracker, &owner) {
            return;
        }

        if matches!(self.level, OptimizationLevel::ReleaseAdvanced)
            && self.optimize_algebraic_identity(program, id, tracker, &owner)
        {
            return;
        }

        if self.optimize_call_site(program, id, plan, tracker, &owner) {
            self.optimize_node(program, id, plan, tracker, owner);
        }
    }

    fn optimize_sequence(&self, program: &mut LirProgram, id: LirNodeId) {
        let snapshot = program.nodes[id.0 as usize].clone();
        match snapshot.kind {
            LirNodeKind::Program | LirNodeKind::Block => {
                let mut flattened = Vec::with_capacity(snapshot.children.len());
                for child in snapshot.children {
                    let child_node = &program.nodes[child.0 as usize];
                    if matches!(child_node.kind, LirNodeKind::Program | LirNodeKind::Block)
                        && child_node.text.is_none()
                    {
                        flattened.extend(child_node.children.iter().copied());
                    } else {
                        flattened.push(child);
                    }
                }
                program.nodes[id.0 as usize].children = flattened;
            }
            _ => {}
        }
    }

    fn specialize_layout_bindings(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        tracker: &mut SpecializationTracker,
        owner: &str,
        env: &mut BindingEnv,
    ) {
        let snapshot = program.nodes[id.0 as usize].clone();
        match snapshot.kind {
            LirNodeKind::Program | LirNodeKind::Block => {
                let mut local_env = env.clone();
                for child in snapshot.children {
                    let child_node = program.nodes[child.0 as usize].clone();
                    if matches!(child_node.kind, LirNodeKind::Program | LirNodeKind::Block) {
                        self.specialize_layout_bindings(
                            program,
                            child,
                            tracker,
                            owner,
                            &mut local_env,
                        );
                    } else {
                        self.specialize_layout_bindings(
                            program,
                            child,
                            tracker,
                            owner,
                            &mut local_env,
                        );
                    }

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
                        self.specialize_layout_bindings(program, id, tracker, owner, env);
                    }
                }
            }
            return;
        }

        for child in snapshot.children {
            self.specialize_layout_bindings(program, child, tracker, owner, env);
        }

        let _ = self.fold_layout_member_access(program, id, tracker, owner, env);
    }

    fn extract_const_binding(
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

    fn is_specializable_binding(&self, program: &LirProgram, id: LirNodeId) -> bool {
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

    fn fold_layout_member_access(
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

    fn object_literal_field(
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

    fn array_literal_element(
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

    fn constant_array_index(
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

    fn is_object_literal(&self, program: &LirProgram, id: LirNodeId) -> bool {
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

    fn is_array_literal(&self, program: &LirProgram, id: LirNodeId) -> bool {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.is_empty() {
            return false;
        }

        !self.is_object_literal(program, id)
    }

    fn optimize_constant_expression(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        tracker: &mut SpecializationTracker,
        owner: &str,
    ) -> bool {
        let snapshot = program.nodes[id.0 as usize].clone();
        match snapshot.kind {
            LirNodeKind::Literal => false,
            LirNodeKind::Value => {
                let Some(op) = snapshot.text.as_deref() else {
                    return false;
                };

                match snapshot.children.len() {
                    1 => {
                        let Some(value) = literal_value(program, snapshot.children[0]) else {
                            return false;
                        };
                        if let Some(folded) = fold_unary(op, value) {
                            let key = format!(
                                "unary:{}:{}",
                                op,
                                node_signature(program, snapshot.children[0])
                            );
                            if !tracker.allow(owner, key) {
                                return false;
                            }
                            program.nodes[id.0 as usize] =
                                LirNode::with_text(LirNodeKind::Literal, literal_text(folded));
                            return true;
                        }
                    }
                    2 => {
                        let left = literal_value(program, snapshot.children[0]);
                        let right = literal_value(program, snapshot.children[1]);
                        if let (Some(left), Some(right)) = (left, right) {
                            if let Some(folded) = fold_binary(op, left, right) {
                                let key = format!(
                                    "binary:{}:{}:{}",
                                    op,
                                    node_signature(program, snapshot.children[0]),
                                    node_signature(program, snapshot.children[1])
                                );
                                if !tracker.allow(owner, key) {
                                    return false;
                                }
                                program.nodes[id.0 as usize] =
                                    LirNode::with_text(LirNodeKind::Literal, literal_text(folded));
                                return true;
                            }
                        }
                    }
                    _ => {}
                }
                false
            }
            LirNodeKind::Branch => {
                let Some(cond_id) = snapshot.children.first().copied() else {
                    return false;
                };
                let Some(condition) = literal_value(program, cond_id) else {
                    return false;
                };
                let truthy = condition.truthy();
                let chosen = if truthy {
                    snapshot.children.get(1).copied()
                } else {
                    snapshot.children.get(2).copied()
                };

                let Some(chosen) = chosen else {
                    let key = format!("branch:{}", node_signature(program, cond_id));
                    if !tracker.allow(owner, key) {
                        return false;
                    }
                    program.nodes[id.0 as usize] =
                        LirNode::with_text(LirNodeKind::Literal, if truthy { "1" } else { "0" });
                    return true;
                };

                let key = format!("branch:{}:{}", node_signature(program, cond_id), truthy);
                if !tracker.allow(owner, key) {
                    return false;
                }
                program.nodes[id.0 as usize] = program.nodes[chosen.0 as usize].clone();
                true
            }
            _ => false,
        }
    }

    fn optimize_algebraic_identity(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        tracker: &mut SpecializationTracker,
        owner: &str,
    ) -> bool {
        let snapshot = program.nodes[id.0 as usize].clone();
        let Some(op) = snapshot.text.as_deref() else {
            return false;
        };

        match (op, snapshot.children.as_slice()) {
            ("+", [left, right]) => {
                let key = format!(
                    "identity:+:{}:{}",
                    node_signature(program, *left),
                    node_signature(program, *right)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }
                if literal_value(program, *left) == Some(ConstantValue::Number(0)) {
                    program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                    return true;
                }
                if literal_value(program, *right) == Some(ConstantValue::Number(0)) {
                    program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                    return true;
                }
                false
            }
            ("-", [left, right]) => {
                let key = format!(
                    "identity:-:{}:{}",
                    node_signature(program, *left),
                    node_signature(program, *right)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }
                if literal_value(program, *right) == Some(ConstantValue::Number(0)) {
                    program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                    return true;
                }
                false
            }
            ("*", [left, right]) => {
                let key = format!(
                    "identity:*:{}:{}",
                    node_signature(program, *left),
                    node_signature(program, *right)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }
                if literal_value(program, *left) == Some(ConstantValue::Number(0))
                    || literal_value(program, *right) == Some(ConstantValue::Number(0))
                {
                    program.nodes[id.0 as usize] = LirNode::with_text(LirNodeKind::Literal, "0");
                    return true;
                }
                if literal_value(program, *left) == Some(ConstantValue::Number(1)) {
                    program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                    return true;
                }
                if literal_value(program, *right) == Some(ConstantValue::Number(1)) {
                    program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                    return true;
                }
                false
            }
            ("&&", [left, right]) => {
                let key = format!(
                    "identity:&&:{}:{}",
                    node_signature(program, *left),
                    node_signature(program, *right)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }
                match literal_value(program, *left) {
                    Some(ConstantValue::Boolean(false)) => {
                        program.nodes[id.0 as usize] =
                            LirNode::with_text(LirNodeKind::Literal, "false");
                        return true;
                    }
                    Some(ConstantValue::Boolean(true)) => {
                        program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                        return true;
                    }
                    _ => {}
                }

                match literal_value(program, *right) {
                    Some(ConstantValue::Boolean(false)) => {
                        program.nodes[id.0 as usize] =
                            LirNode::with_text(LirNodeKind::Literal, "false");
                        return true;
                    }
                    Some(ConstantValue::Boolean(true)) => {
                        program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                        return true;
                    }
                    _ => false,
                }
            }
            ("||", [left, right]) => {
                let key = format!(
                    "identity:||:{}:{}",
                    node_signature(program, *left),
                    node_signature(program, *right)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }
                match literal_value(program, *left) {
                    Some(ConstantValue::Boolean(true)) => {
                        program.nodes[id.0 as usize] =
                            LirNode::with_text(LirNodeKind::Literal, "true");
                        return true;
                    }
                    Some(ConstantValue::Boolean(false)) => {
                        program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                        return true;
                    }
                    _ => {}
                }

                match literal_value(program, *right) {
                    Some(ConstantValue::Boolean(true)) => {
                        program.nodes[id.0 as usize] =
                            LirNode::with_text(LirNodeKind::Literal, "true");
                        return true;
                    }
                    Some(ConstantValue::Boolean(false)) => {
                        program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                        return true;
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
    fn optimize_call_site(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        plan: &SpecializationPlan,
        tracker: &mut SpecializationTracker,
        owner: &str,
    ) -> bool {
        let snapshot = program.nodes[id.0 as usize].clone();
        if snapshot.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee_id) = snapshot.children.first().copied() else {
            return false;
        };
        let Some(callee_node) = program.nodes.get(callee_id.0 as usize).cloned() else {
            return false;
        };
        let Some(callee_name) = callee_node.text.as_deref() else {
            return false;
        };
        let Some(summary) = plan.functions.get(callee_name) else {
            return false;
        };
        let Some(inline_body) = summary.inline_body else {
            return false;
        };

        let inline_threshold = match self.level {
            OptimizationLevel::Release => 12,
            OptimizationLevel::ReleaseAdvanced => 24,
            _ => 0,
        };
        if summary.node_count > inline_threshold || summary.recursive {
            return false;
        }

        let args: Vec<LirNodeId> = snapshot.children.iter().skip(1).copied().collect();
        if args.len() != summary.params.len() {
            return false;
        }

        let key = format!(
            "inline:{}:{}",
            callee_name,
            self.call_signature(program, &snapshot)
        );
        if !tracker.allow(owner, key) {
            return false;
        }

        let cloned_root = self.inline_call_site(program, inline_body, &summary.params, &args);
        let replacement = program.nodes[cloned_root.0 as usize].clone();
        program.nodes[id.0 as usize] = replacement;
        true
    }

    fn specialize_mir_call_sites(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        plan: &SpecializationPlan,
        mir_plan: &MirSpecializationPlan,
        tracker: &mut SpecializationTracker,
        owner: String,
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

        if let Some(new_function) = self.specialize_mir_call_site(
            program,
            id,
            plan,
            mir_plan,
            tracker,
            &owner,
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
            );
            self.specialize_mir_call_sites(
                program,
                new_function,
                plan,
                mir_plan,
                tracker,
                recursive_owner,
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
                specialized_functions,
            );
        }
    }

    fn specialize_mir_call_site(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        plan: &SpecializationPlan,
        mir_plan: &MirSpecializationPlan,
        tracker: &mut SpecializationTracker,
        owner: &str,
        specialized_functions: &mut BTreeMap<String, LirNodeId>,
    ) -> Option<LirNodeId> {
        let snapshot = program.nodes[id.0 as usize].clone();
        if snapshot.kind != LirNodeKind::Call {
            return None;
        }

        let Some(callee_id) = snapshot.children.first().copied() else {
            return None;
        };
        let Some(callee_node) = program.nodes.get(callee_id.0 as usize).cloned() else {
            return None;
        };
        let Some(callee_name) = callee_node.text.as_deref() else {
            return None;
        };
        let Some(summary) = plan.functions.get(callee_name) else {
            return None;
        };
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
            let Some(layout) = mir_plan.parameter_layout(callee_name, index) else {
                signature_parts.push(self.specialization_signature(program, *arg));
                continue;
            };

            if layout.kind == MirLayoutClass::TaggedVal {
                signature_parts.push(self.specialization_signature(program, *arg));
                continue;
            }

            let arg_signature = self.specialization_signature_with_mir(program, *arg, mir_plan);
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

        let specialization_key =
            format!("specialize:{}:{}", callee_name, signature_parts.join("|"));
        if !tracker.allow(owner, specialization_key) {
            return None;
        }

        let specialized_name = self.specialized_function_name(callee_name, &signature_parts);
        if specialized_functions.contains_key(&specialized_name) {
            if let Some(callee) = program.nodes.get_mut(callee_id.0 as usize) {
                callee.text = Some(specialized_name);
            }
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
            &mut BindingEnv::default(),
        );

        if let Some(callee) = program.nodes.get_mut(callee_id.0 as usize) {
            callee.text = Some(specialized_name);
        }

        Some(new_id)
    }

    fn clone_specialized_function(
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
        });
        new_id
    }

    fn specialized_function_name(&self, callee_name: &str, signature_parts: &[String]) -> String {
        let mut hash = 0xcbf29ce484222325u64;
        let signature = signature_parts.join("|");
        for byte in signature.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("{}$spec${:016x}", callee_name, hash)
    }

    fn specialization_signature_with_mir(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        mir_plan: &MirSpecializationPlan,
    ) -> String {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return "<missing>".to_string();
        };

        if node.children.is_empty() {
            if let Some(text) = node.text.as_deref() {
                if let Some(layout) = mir_plan.binding_layout(text) {
                    return format!("binding:{}", layout.key());
                }
            }
        }

        self.specialization_signature(program, id)
    }

    fn build_specialization_plan(&self, program: &LirProgram) -> SpecializationPlan {
        let mut plan = SpecializationPlan::default();
        let mut visited = HashSet::new();
        self.collect_specialization_plan(program, program.root, &mut visited, &mut plan);
        plan
    }

    fn collect_specialization_plan(
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

    fn function_summary(&self, program: &LirProgram, id: LirNodeId) -> Option<FunctionSummary> {
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

    fn extract_inline_body(&self, program: &LirProgram, block_id: LirNodeId) -> Option<LirNodeId> {
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

    fn count_subtree_nodes(&self, program: &LirProgram, id: LirNodeId) -> usize {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return 0;
        };

        let mut count = 1;
        for child in &node.children {
            count += self.count_subtree_nodes(program, *child);
        }
        count
    }

    fn contains_call_target(&self, program: &LirProgram, id: LirNodeId, target: &str) -> bool {
        let mut targets = BTreeSet::new();
        self.collect_call_targets(program, id, &mut targets);
        targets.contains(target)
    }

    fn collect_call_targets(
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

    fn prune_dead_top_level_functions(&self, program: &mut LirProgram) {
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

    fn inline_call_site(
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

    fn clone_subtree_with_substitution(
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
        });
        memo.insert(id, new_id);
        new_id
    }

    fn call_signature(&self, program: &LirProgram, node: &LirNode) -> String {
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

    fn specialization_signature(&self, program: &LirProgram, id: LirNodeId) -> String {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return "<missing>".to_string();
        };

        let mut signature = match node.kind {
            LirNodeKind::Literal => match parse_literal_text(node.text.as_deref()) {
                Some(ConstantValue::Number(_)) => "Literal:number".to_string(),
                Some(ConstantValue::Boolean(_)) => "Literal:boolean".to_string(),
                None => format!("{:?}:{:?}", node.kind, node.text),
            },
            LirNodeKind::Value if node.children.is_empty() => match node.text.as_deref() {
                Some(text) if parse_literal_text(Some(text)).is_some() => {
                    if matches!(
                        parse_literal_text(Some(text)),
                        Some(ConstantValue::Boolean(_))
                    ) {
                        "Value:boolean".to_string()
                    } else {
                        "Value:number".to_string()
                    }
                }
                _ => format!("{:?}:{:?}", node.kind, node.text),
            },
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
enum MirLayoutClass {
    Scalar,
    Struct,
    Array,
    Closure,
    TaggedVal,
}

impl MirLayoutClass {
    fn as_str(self) -> &'static str {
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
struct MirLayoutSignature {
    kind: MirLayoutClass,
    fingerprint: String,
}

impl MirLayoutSignature {
    fn from_descriptor(descriptor: &LayoutDescriptor) -> Self {
        Self {
            kind: MirLayoutClass::from_descriptor(descriptor),
            fingerprint: layout_descriptor_signature(descriptor),
        }
    }

    fn key(&self) -> String {
        format!("{}:{}", self.kind.as_str(), self.fingerprint)
    }
}

#[derive(Clone, Debug, Default)]
struct MirSpecializationPlan {
    binding_layouts: BTreeMap<String, MirLayoutSignature>,
    parameter_layouts: BTreeMap<String, Vec<MirLayoutSignature>>,
}

impl MirSpecializationPlan {
    fn from_program(mir: &MirAnalysisProgram) -> Self {
        let mut binding_layouts = BTreeMap::new();
        let mut parameter_layouts = BTreeMap::new();

        for function in &mir.functions {
            if let Some(name) = function.name.as_deref() {
                let mut params = Vec::new();
                for binding in &function.bindings {
                    let layout = MirLayoutSignature::from_descriptor(&binding.layout);
                    binding_layouts
                        .entry(binding.name.clone())
                        .and_modify(|existing| {
                            if *existing != layout {
                                *existing = MirLayoutSignature {
                                    kind: MirLayoutClass::TaggedVal,
                                    fingerprint: layout_descriptor_signature(
                                        &LayoutDescriptor::TaggedVal,
                                    ),
                                };
                            }
                        })
                        .or_insert(layout.clone());

                    if binding.kind == MirBindingKind::Parameter {
                        params.push(layout);
                    }
                }
                parameter_layouts.insert(name.to_string(), params);
            } else {
                for binding in &function.bindings {
                    let layout = MirLayoutSignature::from_descriptor(&binding.layout);
                    binding_layouts
                        .entry(binding.name.clone())
                        .and_modify(|existing| {
                            if *existing != layout {
                                *existing = MirLayoutSignature {
                                    kind: MirLayoutClass::TaggedVal,
                                    fingerprint: layout_descriptor_signature(
                                        &LayoutDescriptor::TaggedVal,
                                    ),
                                };
                            }
                        })
                        .or_insert(layout);
                }
            }
        }

        Self {
            binding_layouts,
            parameter_layouts,
        }
    }

    fn binding_layout(&self, name: &str) -> Option<MirLayoutSignature> {
        self.binding_layouts
            .get(name)
            .cloned()
            .filter(|layout| layout.kind != MirLayoutClass::TaggedVal)
    }

    fn parameter_layout(&self, function: &str, index: usize) -> Option<MirLayoutSignature> {
        self.parameter_layouts
            .get(function)
            .and_then(|layouts| layouts.get(index).cloned())
            .filter(|layout| layout.kind != MirLayoutClass::TaggedVal)
    }
}

impl MirLayoutClass {
    fn from_descriptor(descriptor: &LayoutDescriptor) -> Self {
        match descriptor {
            LayoutDescriptor::Scalar(_) => MirLayoutClass::Scalar,
            LayoutDescriptor::Struct { .. } => MirLayoutClass::Struct,
            LayoutDescriptor::Array { .. } => MirLayoutClass::Array,
            LayoutDescriptor::Closure { .. } => MirLayoutClass::Closure,
            LayoutDescriptor::TaggedVal => MirLayoutClass::TaggedVal,
        }
    }
}

fn layout_descriptor_signature(descriptor: &LayoutDescriptor) -> String {
    match descriptor {
        LayoutDescriptor::Scalar(name) => format!("Scalar({name})"),
        LayoutDescriptor::Struct { fields } => {
            let mut parts = Vec::with_capacity(fields.len());
            for (field, layout) in fields {
                parts.push(format!("{}:{}", field, layout_descriptor_signature(layout)));
            }
            format!("Struct({})", parts.join(","))
        }
        LayoutDescriptor::Array { element, length } => format!(
            "Array(length={:?},element={})",
            length,
            layout_descriptor_signature(element)
        ),
        LayoutDescriptor::Closure { captures } => format!("Closure(len={})", captures.len()),
        LayoutDescriptor::TaggedVal => "TaggedVal".to_string(),
    }
}

#[derive(Clone, Debug, Default)]
struct SpecializationPlan {
    functions: BTreeMap<String, FunctionSummary>,
}

#[derive(Clone, Debug, Default)]
struct BindingEnv {
    bindings: BTreeMap<String, LirNodeId>,
}

#[derive(Clone, Debug)]
struct FunctionSummary {
    node_id: LirNodeId,
    name: String,
    params: Vec<String>,
    body_block: LirNodeId,
    inline_body: Option<LirNodeId>,
    node_count: usize,
    recursive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConstantValue {
    Number(i64),
    Boolean(bool),
}

#[derive(Debug)]
struct SpecializationTracker {
    max_specializations: usize,
    seen: BTreeMap<String, BTreeSet<String>>,
}

impl SpecializationTracker {
    fn new(max_specializations: usize) -> Self {
        Self {
            max_specializations,
            seen: BTreeMap::new(),
        }
    }

    fn allow(&mut self, owner: impl Into<String>, key: String) -> bool {
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

impl ConstantValue {
    fn truthy(self) -> bool {
        match self {
            ConstantValue::Number(value) => value != 0,
            ConstantValue::Boolean(value) => value,
        }
    }
}

fn literal_value(program: &LirProgram, id: LirNodeId) -> Option<ConstantValue> {
    let node = program.nodes.get(id.0 as usize)?;
    match node.kind {
        LirNodeKind::Literal => parse_literal_text(node.text.as_deref()),
        LirNodeKind::Value if node.children.is_empty() => parse_literal_text(node.text.as_deref()),
        _ => None,
    }
}

fn node_signature(program: &LirProgram, id: LirNodeId) -> String {
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

fn parse_literal_text(text: Option<&str>) -> Option<ConstantValue> {
    let text = text?;
    match text {
        "true" => Some(ConstantValue::Boolean(true)),
        "false" => Some(ConstantValue::Boolean(false)),
        "null" | "undefined" => Some(ConstantValue::Number(0)),
        _ => parse_number_literal(text).map(ConstantValue::Number),
    }
}

fn fold_unary(op: &str, value: ConstantValue) -> Option<ConstantValue> {
    match (op, value) {
        ("-", ConstantValue::Number(value)) => value.checked_neg().map(ConstantValue::Number),
        ("!", value) => Some(ConstantValue::Boolean(!value.truthy())),
        _ => None,
    }
}

fn fold_binary(op: &str, left: ConstantValue, right: ConstantValue) -> Option<ConstantValue> {
    match (op, left, right) {
        ("+", ConstantValue::Number(left), ConstantValue::Number(right)) => {
            left.checked_add(right).map(ConstantValue::Number)
        }
        ("-", ConstantValue::Number(left), ConstantValue::Number(right)) => {
            left.checked_sub(right).map(ConstantValue::Number)
        }
        ("*", ConstantValue::Number(left), ConstantValue::Number(right)) => {
            left.checked_mul(right).map(ConstantValue::Number)
        }
        ("/", ConstantValue::Number(left), ConstantValue::Number(right)) => {
            if right == 0 {
                None
            } else {
                Some(ConstantValue::Number(left / right))
            }
        }
        ("==", left, right) => Some(ConstantValue::Boolean(match (left, right) {
            (ConstantValue::Number(left), ConstantValue::Number(right)) => left == right,
            (ConstantValue::Boolean(left), ConstantValue::Boolean(right)) => left == right,
            _ => left.truthy() == right.truthy(),
        })),
        ("&&", left, right) => Some(ConstantValue::Boolean(left.truthy() && right.truthy())),
        ("||", left, right) => Some(ConstantValue::Boolean(left.truthy() || right.truthy())),
        _ => None,
    }
}

fn literal_text(value: ConstantValue) -> String {
    match value {
        ConstantValue::Number(value) => value.to_string(),
        ConstantValue::Boolean(value) => value.to_string(),
    }
}

fn parse_number_literal(text: &str) -> Option<i64> {
    if let Some(stripped) = text.strip_suffix('n') {
        return stripped.parse::<i64>().ok();
    }
    text.parse::<i64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kali_lir::{LirBuilder, LirNodeKind};

    fn literal(builder: &mut LirBuilder, value: &str) -> LirNodeId {
        builder.alloc_text(LirNodeKind::Literal, value)
    }

    #[test]
    fn release_constant_folds_binary_expressions() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let add = builder.alloc_text(LirNodeKind::Value, "+");
        let lhs = literal(&mut builder, "1");
        let rhs = literal(&mut builder, "2");
        builder.node_mut(add).unwrap().children = vec![lhs, rhs];
        builder.node_mut(root).unwrap().children = vec![add];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

        let node = &program.nodes[add.0 as usize];
        assert_eq!(node.kind, LirNodeKind::Literal);
        assert_eq!(node.text.as_deref(), Some("3"));
    }

    #[test]
    fn specialization_cap_limits_distinct_constant_folds() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let first = builder.alloc_text(LirNodeKind::Value, "+");
        let second = builder.alloc_text(LirNodeKind::Value, "+");
        let first_left = literal(&mut builder, "1");
        let first_right = literal(&mut builder, "2");
        let second_left = literal(&mut builder, "3");
        let second_right = literal(&mut builder, "4");
        builder.node_mut(first).unwrap().children = vec![first_left, first_right];
        builder.node_mut(second).unwrap().children = vec![second_left, second_right];
        builder.node_mut(root).unwrap().children = vec![first, second];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::with_max_specializations(OptimizationLevel::Release, 1)
            .optimize_program(&mut program);

        let first_node = &program.nodes[first.0 as usize];
        let second_node = &program.nodes[second.0 as usize];
        assert_eq!(first_node.kind, LirNodeKind::Literal);
        assert_eq!(first_node.text.as_deref(), Some("3"));
        assert_eq!(second_node.kind, LirNodeKind::Value);
        assert_eq!(second_node.text.as_deref(), Some("+"));
    }

    #[test]
    fn specialization_cap_is_scoped_per_function() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let first_function = builder.alloc_text(LirNodeKind::Instruction, "first");
        let first_param = builder.alloc_text(LirNodeKind::Value, "x");
        let first_block = builder.alloc(LirNodeKind::Block);
        let first_return = builder.alloc_text(LirNodeKind::Instruction, "return");
        let first_expr = builder.alloc_text(LirNodeKind::Value, "+");
        let first_left = literal(&mut builder, "1");
        let first_right = literal(&mut builder, "2");
        builder.node_mut(first_expr).unwrap().children = vec![first_left, first_right];
        builder.node_mut(first_return).unwrap().children = vec![first_expr];
        builder.node_mut(first_block).unwrap().children = vec![first_return];
        builder.node_mut(first_function).unwrap().children = vec![first_param, first_block];

        let second_function = builder.alloc_text(LirNodeKind::Instruction, "second");
        let second_param = builder.alloc_text(LirNodeKind::Value, "y");
        let second_block = builder.alloc(LirNodeKind::Block);
        let second_return = builder.alloc_text(LirNodeKind::Instruction, "return");
        let second_expr = builder.alloc_text(LirNodeKind::Value, "+");
        let second_left = literal(&mut builder, "3");
        let second_right = literal(&mut builder, "4");
        builder.node_mut(second_expr).unwrap().children = vec![second_left, second_right];
        builder.node_mut(second_return).unwrap().children = vec![second_expr];
        builder.node_mut(second_block).unwrap().children = vec![second_return];
        builder.node_mut(second_function).unwrap().children = vec![second_param, second_block];

        builder.node_mut(root).unwrap().children = vec![first_function, second_function];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::with_max_specializations(OptimizationLevel::Release, 1)
            .optimize_program(&mut program);

        assert_eq!(
            program.nodes[first_expr.0 as usize].kind,
            LirNodeKind::Literal
        );
        assert_eq!(
            program.nodes[first_expr.0 as usize].text.as_deref(),
            Some("3")
        );
        assert_eq!(
            program.nodes[second_expr.0 as usize].kind,
            LirNodeKind::Literal
        );
        assert_eq!(
            program.nodes[second_expr.0 as usize].text.as_deref(),
            Some("7")
        );
    }

    #[test]
    fn release_advanced_eliminates_algebraic_identities() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let add = builder.alloc_text(LirNodeKind::Value, "+");
        let ident = builder.alloc_text(LirNodeKind::Value, "x");
        let zero = literal(&mut builder, "0");
        builder.node_mut(add).unwrap().children = vec![ident, zero];
        builder.node_mut(root).unwrap().children = vec![add];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

        let node = &program.nodes[add.0 as usize];
        assert_eq!(node.kind, LirNodeKind::Value);
        assert_eq!(node.text.as_deref(), Some("x"));
        assert!(node.children.is_empty());
    }

    #[test]
    fn release_inlines_simple_function_calls() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let function = builder.alloc_text(LirNodeKind::Instruction, "add_one");
        let param = builder.alloc_text(LirNodeKind::Value, "x");
        let block = builder.alloc(LirNodeKind::Block);
        let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let expr = builder.alloc_text(LirNodeKind::Value, "+");
        let one = literal(&mut builder, "1");
        let arg = literal(&mut builder, "2");
        builder.node_mut(expr).unwrap().children = vec![param, one];
        builder.node_mut(ret).unwrap().children = vec![expr];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param, block];
        let call = builder.alloc(LirNodeKind::Call);
        let callee = builder.alloc_text(LirNodeKind::Value, "add_one");
        builder.node_mut(call).unwrap().children = vec![callee, arg];
        builder.node_mut(root).unwrap().children = vec![function, call];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

        let node = &program.nodes[call.0 as usize];
        assert_eq!(node.kind, LirNodeKind::Literal);
        assert_eq!(node.text.as_deref(), Some("3"));
    }

    #[test]
    fn release_advanced_prunes_dead_inlined_functions() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let function = builder.alloc_text(LirNodeKind::Instruction, "add_one");
        let param = builder.alloc_text(LirNodeKind::Value, "x");
        let block = builder.alloc(LirNodeKind::Block);
        let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let expr = builder.alloc_text(LirNodeKind::Value, "+");
        let one = literal(&mut builder, "1");
        let arg = literal(&mut builder, "2");
        builder.node_mut(expr).unwrap().children = vec![param, one];
        builder.node_mut(ret).unwrap().children = vec![expr];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param, block];
        let call = builder.alloc(LirNodeKind::Call);
        let callee = builder.alloc_text(LirNodeKind::Value, "add_one");
        builder.node_mut(call).unwrap().children = vec![callee, arg];
        builder.node_mut(root).unwrap().children = vec![function, call];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

        let node = &program.nodes[call.0 as usize];
        assert_eq!(node.kind, LirNodeKind::Literal);
        assert_eq!(node.text.as_deref(), Some("3"));
        assert_eq!(program.nodes[root.0 as usize].children, vec![call]);
    }

    #[test]
    fn release_eliminates_constant_branches() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let branch = builder.alloc(LirNodeKind::Branch);
        let cond = literal(&mut builder, "false");
        let then_lit = literal(&mut builder, "1");
        let else_lit = literal(&mut builder, "2");
        builder.node_mut(branch).unwrap().children = vec![cond, then_lit, else_lit];
        builder.node_mut(root).unwrap().children = vec![branch];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

        let node = &program.nodes[branch.0 as usize];
        assert_eq!(node.kind, LirNodeKind::Literal);
        assert_eq!(node.text.as_deref(), Some("2"));
    }

    #[test]
    fn release_specializes_const_object_property_access() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let const_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
        let declarator = builder.alloc_text(LirNodeKind::Instruction, "point");
        let binding_name = builder.alloc_text(LirNodeKind::Value, "point");
        let object = builder.alloc(LirNodeKind::Value);
        let prop_x = builder.alloc_text(LirNodeKind::Value, "init");
        let key_x = literal(&mut builder, "x");
        let value_x = literal(&mut builder, "1");
        let prop_y = builder.alloc_text(LirNodeKind::Value, "init");
        let key_y = literal(&mut builder, "y");
        let value_y = literal(&mut builder, "2");
        let access = builder.alloc_text(LirNodeKind::Value, "y");
        let point_ref = builder.alloc_text(LirNodeKind::Value, "point");

        builder.node_mut(prop_x).unwrap().children = vec![key_x, value_x];
        builder.node_mut(prop_y).unwrap().children = vec![key_y, value_y];
        builder.node_mut(object).unwrap().children = vec![prop_x, prop_y];
        builder.node_mut(declarator).unwrap().children = vec![binding_name, object];
        builder.node_mut(const_decl).unwrap().children = vec![declarator];
        builder.node_mut(access).unwrap().children = vec![point_ref];
        builder.node_mut(root).unwrap().children = vec![const_decl, access];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

        let node = &program.nodes[access.0 as usize];
        assert_eq!(node.kind, LirNodeKind::Literal);
        assert_eq!(node.text.as_deref(), Some("2"));
    }

    #[test]
    fn release_specializes_const_array_element_access() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let index_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
        let index_binding = builder.alloc_text(LirNodeKind::Instruction, "index");
        let index_name = builder.alloc_text(LirNodeKind::Value, "index");
        let index_value = literal(&mut builder, "1");
        builder.node_mut(index_binding).unwrap().children = vec![index_name, index_value];
        builder.node_mut(index_decl).unwrap().children = vec![index_binding];

        let bag_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
        let bag_binding = builder.alloc_text(LirNodeKind::Instruction, "bag");
        let bag_name = builder.alloc_text(LirNodeKind::Value, "bag");
        let array = builder.alloc(LirNodeKind::Value);
        let first = literal(&mut builder, "10");
        let second = literal(&mut builder, "20");
        builder.node_mut(array).unwrap().children = vec![first, second];
        builder.node_mut(bag_binding).unwrap().children = vec![bag_name, array];
        builder.node_mut(bag_decl).unwrap().children = vec![bag_binding];

        let access = builder.alloc_text(LirNodeKind::Value, "index");
        let bag_ref = builder.alloc_text(LirNodeKind::Value, "bag");
        builder.node_mut(access).unwrap().children = vec![bag_ref];

        builder.node_mut(root).unwrap().children = vec![index_decl, bag_decl, access];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

        let node = &program.nodes[access.0 as usize];
        assert_eq!(node.kind, LirNodeKind::Literal);
        assert_eq!(node.text.as_deref(), Some("20"));
    }

    #[test]
    fn release_specializes_large_function_using_mir_layouts() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "sum_many");
        let param_x = builder.alloc_text(LirNodeKind::Value, "x");
        let param_y = builder.alloc_text(LirNodeKind::Value, "y");
        let block = builder.alloc(LirNodeKind::Block);
        let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let outer_add = builder.alloc_text(LirNodeKind::Value, "+");
        let left_add = builder.alloc_text(LirNodeKind::Value, "+");
        let right_add = builder.alloc_text(LirNodeKind::Value, "+");
        let left_left = builder.alloc_text(LirNodeKind::Value, "+");
        let left_right = builder.alloc_text(LirNodeKind::Value, "+");
        let right_left = builder.alloc_text(LirNodeKind::Value, "+");
        let right_right = builder.alloc_text(LirNodeKind::Value, "+");
        builder.node_mut(left_left).unwrap().children = vec![param_x, param_y];
        builder.node_mut(left_right).unwrap().children = vec![param_x, param_y];
        builder.node_mut(right_left).unwrap().children = vec![param_x, param_y];
        builder.node_mut(right_right).unwrap().children = vec![param_x, param_y];
        builder.node_mut(left_add).unwrap().children = vec![left_left, left_right];
        builder.node_mut(right_add).unwrap().children = vec![right_left, right_right];
        builder.node_mut(outer_add).unwrap().children = vec![left_add, right_add];
        builder.node_mut(ret).unwrap().children = vec![outer_add];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_x, param_y, block];

        let call = builder.alloc(LirNodeKind::Call);
        let callee = builder.alloc_text(LirNodeKind::Value, "sum_many");
        let one = literal(&mut builder, "1");
        let two = literal(&mut builder, "2");
        builder.node_mut(call).unwrap().children = vec![callee, one, two];

        builder.node_mut(root).unwrap().children = vec![function, call];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![kali_mir::MirFunction {
                name: Some("sum_many".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                bindings: vec![
                    kali_mir::MirBinding {
                        name: "x".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::Scalar("number".to_string()),
                        escapes: false,
                        captured_by: Vec::new(),
                    },
                    kali_mir::MirBinding {
                        name: "y".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::Scalar("number".to_string()),
                        escapes: false,
                        captured_by: Vec::new(),
                    },
                ],
            }],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let call_node = &program.nodes[call.0 as usize];
        let specialized_name = call_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist");
        assert!(specialized_name.starts_with("sum_many$spec$"));

        let specialized_function = program
            .nodes
            .iter()
            .find(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name)
            })
            .expect("specialized function should be inserted");
        let literal_twelve = program
            .nodes
            .iter()
            .any(|node| node.kind == LirNodeKind::Literal && node.text.as_deref() == Some("12"));
        assert!(
            literal_twelve,
            "specialized clone should fold the repeated literals"
        );
        assert_eq!(specialized_function.kind, LirNodeKind::Instruction);
    }

    #[test]
    fn release_specializes_shared_closure_layout_bindings() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_handler");
        let param_handler = builder.alloc_text(LirNodeKind::Value, "handler");
        let param_value = builder.alloc_text(LirNodeKind::Value, "value");
        let block = builder.alloc(LirNodeKind::Block);
        let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let add1 = builder.alloc_text(LirNodeKind::Value, "+");
        let add2 = builder.alloc_text(LirNodeKind::Value, "+");
        let add3 = builder.alloc_text(LirNodeKind::Value, "+");
        let add4 = builder.alloc_text(LirNodeKind::Value, "+");
        let add5 = builder.alloc_text(LirNodeKind::Value, "+");
        let add6 = builder.alloc_text(LirNodeKind::Value, "+");
        let add7 = builder.alloc_text(LirNodeKind::Value, "+");
        let add8 = builder.alloc_text(LirNodeKind::Value, "+");
        let one = literal(&mut builder, "1");
        let two = literal(&mut builder, "2");
        let three = literal(&mut builder, "3");
        let four = literal(&mut builder, "4");
        let five = literal(&mut builder, "5");
        let six = literal(&mut builder, "6");
        let seven = literal(&mut builder, "7");
        let eight = literal(&mut builder, "8");
        builder.node_mut(add1).unwrap().children = vec![param_value, one];
        builder.node_mut(add2).unwrap().children = vec![add1, two];
        builder.node_mut(add3).unwrap().children = vec![add2, three];
        builder.node_mut(add4).unwrap().children = vec![add3, four];
        builder.node_mut(add5).unwrap().children = vec![add4, five];
        builder.node_mut(add6).unwrap().children = vec![add5, six];
        builder.node_mut(add7).unwrap().children = vec![add6, seven];
        builder.node_mut(add8).unwrap().children = vec![add7, eight];
        builder.node_mut(ret).unwrap().children = vec![add8];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_handler, param_value, block];

        let call_a = builder.alloc(LirNodeKind::Call);
        let callee_a = builder.alloc_text(LirNodeKind::Value, "consume_handler");
        let handler_a = builder.alloc_text(LirNodeKind::Value, "handler_a");
        let one_a = literal(&mut builder, "1");
        builder.node_mut(call_a).unwrap().children = vec![callee_a, handler_a, one_a];

        let call_b = builder.alloc(LirNodeKind::Call);
        let callee_b = builder.alloc_text(LirNodeKind::Value, "consume_handler");
        let handler_b = builder.alloc_text(LirNodeKind::Value, "handler_b");
        let one_b = literal(&mut builder, "1");
        builder.node_mut(call_b).unwrap().children = vec![callee_b, handler_b, one_b];

        builder.node_mut(root).unwrap().children = vec![function, call_a, call_b];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![
                kali_mir::MirFunction {
                    name: None,
                    kind: kali_mir::MirFunctionKind::Module,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "handler_a".to_string(),
                            kind: MirBindingKind::Local,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::Closure {
                                captures: vec!["scope_a".to_string()],
                            },
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "handler_b".to_string(),
                            kind: MirBindingKind::Local,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::Closure {
                                captures: vec!["scope_b".to_string()],
                            },
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                    ],
                },
                kali_mir::MirFunction {
                    name: Some("consume_handler".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "handler".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::Closure {
                                captures: vec!["scope".to_string()],
                            },
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "value".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::Scalar("number".to_string()),
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                    ],
                },
            ],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let call_a_node = &program.nodes[call_a.0 as usize];
        let call_b_node = &program.nodes[call_b.0 as usize];
        let specialized_name_a = call_a_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for call_a");
        let specialized_name_b = call_b_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for call_b");
        assert_eq!(specialized_name_a, specialized_name_b);
        assert!(specialized_name_a.starts_with("consume_handler$spec$"));

        let specialized_count = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_a)
            })
            .count();
        assert_eq!(
            specialized_count, 1,
            "closure-layout specialization should be shared"
        );
    }

    #[test]
    fn release_specializes_shared_struct_layout_bindings() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_point");
        let param_point = builder.alloc_text(LirNodeKind::Value, "point");
        let param_value = builder.alloc_text(LirNodeKind::Value, "value");
        let block = builder.alloc(LirNodeKind::Block);
        let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let add1 = builder.alloc_text(LirNodeKind::Value, "+");
        let add2 = builder.alloc_text(LirNodeKind::Value, "+");
        let add3 = builder.alloc_text(LirNodeKind::Value, "+");
        let add4 = builder.alloc_text(LirNodeKind::Value, "+");
        let add5 = builder.alloc_text(LirNodeKind::Value, "+");
        let add6 = builder.alloc_text(LirNodeKind::Value, "+");
        let add7 = builder.alloc_text(LirNodeKind::Value, "+");
        let add8 = builder.alloc_text(LirNodeKind::Value, "+");
        let one = literal(&mut builder, "1");
        let two = literal(&mut builder, "2");
        let three = literal(&mut builder, "3");
        let four = literal(&mut builder, "4");
        let five = literal(&mut builder, "5");
        let six = literal(&mut builder, "6");
        let seven = literal(&mut builder, "7");
        let eight = literal(&mut builder, "8");
        builder.node_mut(add1).unwrap().children = vec![param_value, one];
        builder.node_mut(add2).unwrap().children = vec![add1, two];
        builder.node_mut(add3).unwrap().children = vec![add2, three];
        builder.node_mut(add4).unwrap().children = vec![add3, four];
        builder.node_mut(add5).unwrap().children = vec![add4, five];
        builder.node_mut(add6).unwrap().children = vec![add5, six];
        builder.node_mut(add7).unwrap().children = vec![add6, seven];
        builder.node_mut(add8).unwrap().children = vec![add7, eight];
        builder.node_mut(ret).unwrap().children = vec![add8];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_point, param_value, block];

        let call_a = builder.alloc(LirNodeKind::Call);
        let callee_a = builder.alloc_text(LirNodeKind::Value, "consume_point");
        let point_a = builder.alloc_text(LirNodeKind::Value, "point_a");
        let value_a = literal(&mut builder, "1");
        builder.node_mut(call_a).unwrap().children = vec![callee_a, point_a, value_a];

        let call_b = builder.alloc(LirNodeKind::Call);
        let callee_b = builder.alloc_text(LirNodeKind::Value, "consume_point");
        let point_b = builder.alloc_text(LirNodeKind::Value, "point_b");
        let value_b = literal(&mut builder, "1");
        builder.node_mut(call_b).unwrap().children = vec![callee_b, point_b, value_b];

        let call_c = builder.alloc(LirNodeKind::Call);
        let callee_c = builder.alloc_text(LirNodeKind::Value, "consume_point");
        let point_c = builder.alloc_text(LirNodeKind::Value, "point_c");
        let value_c = literal(&mut builder, "1");
        builder.node_mut(call_c).unwrap().children = vec![callee_c, point_c, value_c];

        builder.node_mut(root).unwrap().children = vec![function, call_a, call_b, call_c];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let struct_layout = LayoutDescriptor::Struct {
            fields: vec![
                (
                    "x".to_string(),
                    Box::new(LayoutDescriptor::Scalar("number".to_string())),
                ),
                (
                    "y".to_string(),
                    Box::new(LayoutDescriptor::Scalar("number".to_string())),
                ),
            ],
        };
        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![
                kali_mir::MirFunction {
                    name: None,
                    kind: kali_mir::MirFunctionKind::Module,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "point_a".to_string(),
                            kind: MirBindingKind::Local,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: struct_layout.clone(),
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "point_b".to_string(),
                            kind: MirBindingKind::Local,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: struct_layout.clone(),
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "point_c".to_string(),
                            kind: MirBindingKind::Local,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: struct_layout.clone(),
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                    ],
                },
                kali_mir::MirFunction {
                    name: Some("consume_point".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "point".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: struct_layout,
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "value".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::Scalar("number".to_string()),
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                    ],
                },
            ],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let call_a_node = &program.nodes[call_a.0 as usize];
        let call_b_node = &program.nodes[call_b.0 as usize];
        let call_c_node = &program.nodes[call_c.0 as usize];
        let specialized_name_a = call_a_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for call_a");
        let specialized_name_b = call_b_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for call_b");
        let specialized_name_c = call_c_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for call_c");
        assert_eq!(specialized_name_a, specialized_name_b);
        assert_eq!(specialized_name_a, specialized_name_c);
        assert!(specialized_name_a.starts_with("consume_point$spec$"));

        let specialized_count = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_a)
            })
            .count();
        assert_eq!(
            specialized_count, 1,
            "struct-layout specialization should be shared across identical bindings"
        );
    }

    #[test]
    fn release_specializes_distinct_struct_layout_bindings() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_point");
        let param_point = builder.alloc_text(LirNodeKind::Value, "point");
        let param_value = builder.alloc_text(LirNodeKind::Value, "value");
        let block = builder.alloc(LirNodeKind::Block);
        let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let add1 = builder.alloc_text(LirNodeKind::Value, "+");
        let add2 = builder.alloc_text(LirNodeKind::Value, "+");
        let add3 = builder.alloc_text(LirNodeKind::Value, "+");
        let add4 = builder.alloc_text(LirNodeKind::Value, "+");
        let add5 = builder.alloc_text(LirNodeKind::Value, "+");
        let add6 = builder.alloc_text(LirNodeKind::Value, "+");
        let add7 = builder.alloc_text(LirNodeKind::Value, "+");
        let add8 = builder.alloc_text(LirNodeKind::Value, "+");
        let one = literal(&mut builder, "1");
        let two = literal(&mut builder, "2");
        let three = literal(&mut builder, "3");
        let four = literal(&mut builder, "4");
        let five = literal(&mut builder, "5");
        let six = literal(&mut builder, "6");
        let seven = literal(&mut builder, "7");
        let eight = literal(&mut builder, "8");
        builder.node_mut(add1).unwrap().children = vec![param_value, one];
        builder.node_mut(add2).unwrap().children = vec![add1, two];
        builder.node_mut(add3).unwrap().children = vec![add2, three];
        builder.node_mut(add4).unwrap().children = vec![add3, four];
        builder.node_mut(add5).unwrap().children = vec![add4, five];
        builder.node_mut(add6).unwrap().children = vec![add5, six];
        builder.node_mut(add7).unwrap().children = vec![add6, seven];
        builder.node_mut(add8).unwrap().children = vec![add7, eight];
        builder.node_mut(ret).unwrap().children = vec![add8];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_point, param_value, block];

        let call_a = builder.alloc(LirNodeKind::Call);
        let callee_a = builder.alloc_text(LirNodeKind::Value, "consume_point");
        let point_a = builder.alloc_text(LirNodeKind::Value, "point_a");
        let value_a = literal(&mut builder, "1");
        builder.node_mut(call_a).unwrap().children = vec![callee_a, point_a, value_a];

        let call_b = builder.alloc(LirNodeKind::Call);
        let callee_b = builder.alloc_text(LirNodeKind::Value, "consume_point");
        let point_b = builder.alloc_text(LirNodeKind::Value, "point_b");
        let value_b = literal(&mut builder, "1");
        builder.node_mut(call_b).unwrap().children = vec![callee_b, point_b, value_b];

        builder.node_mut(root).unwrap().children = vec![function, call_a, call_b];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let struct_layout_a = LayoutDescriptor::Struct {
            fields: vec![
                (
                    "x".to_string(),
                    Box::new(LayoutDescriptor::Scalar("number".to_string())),
                ),
                (
                    "y".to_string(),
                    Box::new(LayoutDescriptor::Scalar("number".to_string())),
                ),
            ],
        };
        let struct_layout_b = LayoutDescriptor::Struct {
            fields: vec![
                (
                    "x".to_string(),
                    Box::new(LayoutDescriptor::Scalar("number".to_string())),
                ),
                (
                    "z".to_string(),
                    Box::new(LayoutDescriptor::Scalar("number".to_string())),
                ),
            ],
        };
        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![
                kali_mir::MirFunction {
                    name: None,
                    kind: kali_mir::MirFunctionKind::Module,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "point_a".to_string(),
                            kind: MirBindingKind::Local,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: struct_layout_a.clone(),
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "point_b".to_string(),
                            kind: MirBindingKind::Local,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: struct_layout_b.clone(),
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                    ],
                },
                kali_mir::MirFunction {
                    name: Some("consume_point".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "point".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: struct_layout_a,
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "value".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::Scalar("number".to_string()),
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                    ],
                },
            ],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let call_a_node = &program.nodes[call_a.0 as usize];
        let call_b_node = &program.nodes[call_b.0 as usize];
        let specialized_name_a = call_a_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for call_a");
        let specialized_name_b = call_b_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for call_b");

        assert_ne!(specialized_name_a, specialized_name_b);

        let specialized_count_a = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_a)
            })
            .count();
        let specialized_count_b = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_b)
            })
            .count();

        assert_eq!(specialized_count_a, 1);
        assert_eq!(specialized_count_b, 1);
    }
}
