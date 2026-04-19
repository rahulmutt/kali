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
        self.optimize_program_internal(program, true);
    }

    /// Optimize a program using MIR layout metadata to drive additional call-site specialization.
    pub fn optimize_program_with_mir(&self, program: &mut LirProgram, mir: &MirAnalysisProgram) {
        self.optimize_program_internal(program, false);

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
            "<root>".to_string(),
            &mut specialized_functions,
        );

        if matches!(self.level, OptimizationLevel::ReleaseAdvanced) {
            self.prune_dead_top_level_functions(program);
        }
    }

    fn optimize_program_internal(
        &self,
        program: &mut LirProgram,
        allow_generic_specialization: bool,
    ) {
        match self.level {
            OptimizationLevel::Fast | OptimizationLevel::Default => {}
            OptimizationLevel::Release | OptimizationLevel::ReleaseAdvanced => {
                let plan = self.build_specialization_plan(program);
                let mut tracker = SpecializationTracker::new(self.max_specializations);
                let mut binding_env = BindingEnv::default();
                self.specialize_layout_bindings(
                    program,
                    program.root,
                    &mut tracker,
                    "<root>",
                    "<root>",
                    &plan,
                    &mut binding_env,
                );

                self.optimize_node(
                    program,
                    program.root,
                    &plan,
                    &mut tracker,
                    "<root>".to_string(),
                    allow_generic_specialization,
                );

                if matches!(self.level, OptimizationLevel::ReleaseAdvanced) {
                    self.prune_dead_top_level_functions(program);
                }
            }
        }
    }

    fn optimize_node(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        plan: &SpecializationPlan,
        tracker: &mut SpecializationTracker,
        owner: String,
        allow_generic_specialization: bool,
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
            self.optimize_node(
                program,
                child,
                plan,
                tracker,
                next_owner.clone(),
                allow_generic_specialization,
            );
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

        if self.optimize_call_site(
            program,
            id,
            plan,
            tracker,
            &owner,
            allow_generic_specialization,
        ) {
            self.optimize_node(
                program,
                id,
                plan,
                tracker,
                owner,
                allow_generic_specialization,
            );
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
                if is_zero_constant(literal_value(program, *left)) {
                    program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                    return true;
                }
                if is_zero_constant(literal_value(program, *right)) {
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
                if is_zero_constant(literal_value(program, *right)) {
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
                if is_zero_constant(literal_value(program, *left))
                    || is_zero_constant(literal_value(program, *right))
                {
                    program.nodes[id.0 as usize] = LirNode::with_text(LirNodeKind::Literal, "0");
                    return true;
                }
                if is_one_constant(literal_value(program, *left)) {
                    program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                    return true;
                }
                if is_one_constant(literal_value(program, *right)) {
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
        allow_generic_specialization: bool,
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
        let args: Vec<LirNodeId> = snapshot.children.iter().skip(1).copied().collect();
        if args.len() != summary.params.len() {
            return false;
        }

        if let Some(inline_body) = summary.inline_body {
            let inline_threshold = match self.level {
                OptimizationLevel::Release => 12,
                OptimizationLevel::ReleaseAdvanced => 24,
                _ => 0,
            };
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
        if !tracker.allow(owner, specialization_key) {
            return false;
        }

        let specialized_name = self.specialized_function_name(callee_name, &signature_parts);
        let new_id = self.clone_specialized_function(
            program,
            summary,
            specialized_name.clone(),
            &substitutions,
        );
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
        );

        if let Some(callee) = program.nodes.get_mut(callee_id.0 as usize) {
            callee.text = Some(specialized_name);
        }

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
                false,
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

    fn specialize_mir_call_site(
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
            let Some(layout) = mir_plan.parameter_layout_any(callee_name, index) else {
                signature_parts.push(self.specialization_signature(program, *arg));
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
            callee_name,
            plan,
            &mut BindingEnv::default(),
        );

        if let Some(callee) = program.nodes.get_mut(callee_id.0 as usize) {
            callee.text = Some(specialized_name);
        }

        Some((new_id, callee_name.to_string()))
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

    fn argument_has_concrete_layout(
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

    fn argument_has_concrete_shape(&self, program: &LirProgram, id: LirNodeId) -> bool {
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

    fn specialization_signature_with_mir(
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

    fn object_literal_signature(
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

    fn array_literal_signature(
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

    fn object_property_signature(
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
    scoped_binding_layouts: BTreeMap<String, BTreeMap<String, MirLayoutSignature>>,
    parameter_layouts: BTreeMap<String, Vec<MirLayoutSignature>>,
}

impl MirSpecializationPlan {
    fn from_program(mir: &MirAnalysisProgram) -> Self {
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

    fn binding_layout(&self, scope: &str, name: &str) -> Option<MirLayoutSignature> {
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

    fn parameter_layout_any(&self, function: &str, index: usize) -> Option<MirLayoutSignature> {
        self.parameter_layouts
            .get(function)
            .and_then(|layouts| layouts.get(index).cloned())
    }

    fn record_layout(
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

    fn tagged_layout_signature() -> MirLayoutSignature {
        MirLayoutSignature {
            kind: MirLayoutClass::TaggedVal,
            fingerprint: layout_descriptor_signature(&LayoutDescriptor::TaggedVal),
        }
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
        LayoutDescriptor::Closure { captures } => {
            let mut captures = captures.clone();
            captures.sort();
            format!("Closure(captures={})", captures.join("|"))
        }
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum ConstantValue {
    Number(i64),
    BigInt(i64),
    Boolean(bool),
    String(String),
    Null,
    Undefined,
    NegativeZero,
    Infinity,
    NegativeInfinity,
    NaN,
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
            ConstantValue::Number(value) | ConstantValue::BigInt(value) => value != 0,
            ConstantValue::Boolean(value) => value,
            ConstantValue::String(value) => !value.is_empty(),
            ConstantValue::Null
            | ConstantValue::Undefined
            | ConstantValue::NegativeZero
            | ConstantValue::NaN => false,
            ConstantValue::Infinity | ConstantValue::NegativeInfinity => true,
        }
    }
}

fn is_zero_constant(value: Option<ConstantValue>) -> bool {
    matches!(
        value,
        Some(ConstantValue::Number(0) | ConstantValue::BigInt(0) | ConstantValue::NegativeZero)
    )
}

fn is_one_constant(value: Option<ConstantValue>) -> bool {
    matches!(
        value,
        Some(ConstantValue::Number(1) | ConstantValue::BigInt(1))
    )
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
        "null" => Some(ConstantValue::Null),
        "undefined" => Some(ConstantValue::Undefined),
        "-0" => Some(ConstantValue::NegativeZero),
        "Infinity" => Some(ConstantValue::Infinity),
        "-Infinity" => Some(ConstantValue::NegativeInfinity),
        "NaN" => Some(ConstantValue::NaN),
        _ => parse_string_literal(text)
            .map(ConstantValue::String)
            .or_else(|| {
                if let Some(stripped) = text.strip_suffix('n') {
                    stripped.parse::<i64>().ok().map(ConstantValue::BigInt)
                } else {
                    parse_number_literal(text).map(ConstantValue::Number)
                }
            }),
    }
}

fn literal_signature(prefix: &str, kind: LirNodeKind, text: Option<&str>) -> String {
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
        Some(ConstantValue::Null) => format!("{prefix}:null"),
        Some(ConstantValue::Undefined) => format!("{prefix}:undefined"),
        Some(ConstantValue::NegativeZero) => format!("{prefix}:number:-0"),
        Some(ConstantValue::Infinity) => format!("{prefix}:number:Infinity"),
        Some(ConstantValue::NegativeInfinity) => format!("{prefix}:number:-Infinity"),
        Some(ConstantValue::NaN) => format!("{prefix}:number:NaN"),
        None => format!("{:?}:{:?}", kind, text),
    }
}

fn string_literal_signature(prefix: &str, text: &str) -> Option<String> {
    let value = parse_string_literal(text)?;
    let literal_kind = if text.starts_with('`') {
        "template"
    } else {
        "quoted"
    };
    Some(format!("{prefix}:string:{literal_kind}:{value}"))
}

fn fold_unary(op: &str, value: ConstantValue) -> Option<ConstantValue> {
    match (op, value) {
        ("-", ConstantValue::Number(0)) => Some(ConstantValue::NegativeZero),
        ("-", ConstantValue::NegativeZero) => Some(ConstantValue::Number(0)),
        ("-", ConstantValue::Number(value)) => value.checked_neg().map(ConstantValue::Number),
        ("-", ConstantValue::BigInt(value)) => value.checked_neg().map(ConstantValue::BigInt),
        ("-", ConstantValue::Infinity) => Some(ConstantValue::NegativeInfinity),
        ("-", ConstantValue::NegativeInfinity) => Some(ConstantValue::Infinity),
        ("-", ConstantValue::NaN) => Some(ConstantValue::NaN),
        ("!", value) => Some(ConstantValue::Boolean(!value.truthy())),
        _ => None,
    }
}

fn fold_binary(op: &str, left: ConstantValue, right: ConstantValue) -> Option<ConstantValue> {
    fn as_number(value: ConstantValue) -> Option<i64> {
        match value {
            ConstantValue::Number(value) => Some(value),
            ConstantValue::NegativeZero => Some(0),
            _ => None,
        }
    }

    match (op, left, right) {
        ("+", ConstantValue::BigInt(left), ConstantValue::BigInt(right)) => {
            left.checked_add(right).map(ConstantValue::BigInt)
        }
        ("-", ConstantValue::BigInt(left), ConstantValue::BigInt(right)) => {
            left.checked_sub(right).map(ConstantValue::BigInt)
        }
        ("*", ConstantValue::BigInt(left), ConstantValue::BigInt(right)) => {
            left.checked_mul(right).map(ConstantValue::BigInt)
        }
        ("/", ConstantValue::BigInt(left), ConstantValue::BigInt(right)) => {
            if right == 0 {
                None
            } else {
                Some(ConstantValue::BigInt(left / right))
            }
        }
        ("==", ConstantValue::BigInt(left), ConstantValue::BigInt(right)) => {
            Some(ConstantValue::Boolean(left == right))
        }
        ("+", left, right) => match (as_number(left), as_number(right)) {
            (Some(left), Some(right)) => left.checked_add(right).map(ConstantValue::Number),
            _ => None,
        },
        ("-", left, right) => match (as_number(left), as_number(right)) {
            (Some(left), Some(right)) => left.checked_sub(right).map(ConstantValue::Number),
            _ => None,
        },
        ("*", left, right) => match (as_number(left), as_number(right)) {
            (Some(left), Some(right)) => left.checked_mul(right).map(ConstantValue::Number),
            _ => None,
        },
        ("/", left, right) => match (as_number(left), as_number(right)) {
            (Some(left), Some(right)) => {
                if right == 0 {
                    None
                } else {
                    Some(ConstantValue::Number(left / right))
                }
            }
            _ => None,
        },
        ("==", left, right) => match (left, right) {
            (ConstantValue::Number(left), ConstantValue::Number(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::Number(left), ConstantValue::NegativeZero)
            | (ConstantValue::NegativeZero, ConstantValue::Number(left)) => {
                Some(ConstantValue::Boolean(left == 0))
            }
            (ConstantValue::NegativeZero, ConstantValue::NegativeZero) => {
                Some(ConstantValue::Boolean(true))
            }
            (ConstantValue::Boolean(left), ConstantValue::Boolean(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::String(left), ConstantValue::String(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::Null, ConstantValue::Null)
            | (ConstantValue::Undefined, ConstantValue::Undefined)
            | (ConstantValue::Null, ConstantValue::Undefined)
            | (ConstantValue::Undefined, ConstantValue::Null) => Some(ConstantValue::Boolean(true)),
            _ => None,
        },
        ("&&", left, right) => Some(ConstantValue::Boolean(left.truthy() && right.truthy())),
        ("||", left, right) => Some(ConstantValue::Boolean(left.truthy() || right.truthy())),
        _ => None,
    }
}

fn literal_text(value: ConstantValue) -> String {
    match value {
        ConstantValue::Number(value) => value.to_string(),
        ConstantValue::BigInt(value) => format!("{value}n"),
        ConstantValue::Boolean(value) => value.to_string(),
        ConstantValue::String(value) => {
            format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
        }
        ConstantValue::Null => "null".to_string(),
        ConstantValue::Undefined => "undefined".to_string(),
        ConstantValue::NegativeZero => "-0".to_string(),
        ConstantValue::Infinity => "Infinity".to_string(),
        ConstantValue::NegativeInfinity => "-Infinity".to_string(),
        ConstantValue::NaN => "NaN".to_string(),
    }
}

fn parse_number_literal(text: &str) -> Option<i64> {
    if let Some(stripped) = text.strip_suffix('n') {
        return stripped.parse::<i64>().ok();
    }
    text.parse::<i64>().ok()
}

fn parse_string_literal(text: &str) -> Option<String> {
    let (inner, is_template) = text
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(|inner| (inner, false))
        .or_else(|| {
            text.strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
                .map(|inner| (inner, false))
        })
        .or_else(|| {
            text.strip_prefix('`')
                .and_then(|value| value.strip_suffix('`'))
                .map(|inner| (inner, true))
        })?;
    let mut value = inner
        .replace("\\\\", "\\")
        .replace("\\\"", "\"")
        .replace("\\'", "'");
    if is_template {
        value = value.replace("\\`", "`");
    }
    Some(value)
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
    fn release_specializes_object_literal_property_order_canonicalization() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_point");
        let param_point = builder.alloc_text(LirNodeKind::Value, "point");
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
        builder.node_mut(add1).unwrap().children = vec![param_point, one];
        builder.node_mut(add2).unwrap().children = vec![add1, two];
        builder.node_mut(add3).unwrap().children = vec![add2, three];
        builder.node_mut(add4).unwrap().children = vec![add3, four];
        builder.node_mut(add5).unwrap().children = vec![add4, five];
        builder.node_mut(add6).unwrap().children = vec![add5, six];
        builder.node_mut(add7).unwrap().children = vec![add6, seven];
        builder.node_mut(add8).unwrap().children = vec![add7, eight];
        builder.node_mut(ret).unwrap().children = vec![add8];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_point, block];

        let call_a = builder.alloc(LirNodeKind::Call);
        let callee_a = builder.alloc_text(LirNodeKind::Value, "consume_point");
        let object_a = builder.alloc(LirNodeKind::Value);
        let object_a_x = builder.alloc_text(LirNodeKind::Value, "init");
        let object_a_x_key = literal(&mut builder, "x");
        let object_a_x_value = literal(&mut builder, "1");
        let object_a_y = builder.alloc_text(LirNodeKind::Value, "init");
        let object_a_y_key = literal(&mut builder, "y");
        let object_a_y_value = literal(&mut builder, "2");
        builder.node_mut(object_a_x).unwrap().children = vec![object_a_x_key, object_a_x_value];
        builder.node_mut(object_a_y).unwrap().children = vec![object_a_y_key, object_a_y_value];
        builder.node_mut(object_a).unwrap().children = vec![object_a_x, object_a_y];
        builder.node_mut(call_a).unwrap().children = vec![callee_a, object_a];

        let call_b = builder.alloc(LirNodeKind::Call);
        let callee_b = builder.alloc_text(LirNodeKind::Value, "consume_point");
        let object_b = builder.alloc(LirNodeKind::Value);
        let object_b_y = builder.alloc_text(LirNodeKind::Value, "init");
        let object_b_y_key = literal(&mut builder, "y");
        let object_b_y_value = literal(&mut builder, "2");
        let object_b_x = builder.alloc_text(LirNodeKind::Value, "init");
        let object_b_x_key = literal(&mut builder, "x");
        let object_b_x_value = literal(&mut builder, "1");
        builder.node_mut(object_b_y).unwrap().children = vec![object_b_y_key, object_b_y_value];
        builder.node_mut(object_b_x).unwrap().children = vec![object_b_x_key, object_b_x_value];
        builder.node_mut(object_b).unwrap().children = vec![object_b_y, object_b_x];
        builder.node_mut(call_b).unwrap().children = vec![callee_b, object_b];

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
                    bindings: Vec::new(),
                },
                kali_mir::MirFunction {
                    name: Some("consume_point".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![kali_mir::MirBinding {
                        name: "point".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::Struct {
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
                        },
                        escapes: false,
                        captured_by: Vec::new(),
                    }],
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
        assert!(specialized_name_a.starts_with("consume_point$spec$"));

        let specialized_count = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_a)
            })
            .count();
        assert_eq!(specialized_count, 1);
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
    fn release_specializes_array_literal_arguments_by_shape() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_array");
        let param_items = builder.alloc_text(LirNodeKind::Value, "items");
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
        builder.node_mut(function).unwrap().children = vec![param_items, param_value, block];

        let call_a = builder.alloc(LirNodeKind::Call);
        let callee_a = builder.alloc_text(LirNodeKind::Value, "consume_array");
        let array_a = builder.alloc(LirNodeKind::Value);
        let array_a_first = literal(&mut builder, "1");
        let array_a_second = literal(&mut builder, "2");
        builder.node_mut(array_a).unwrap().children = vec![array_a_first, array_a_second];
        let value_a = literal(&mut builder, "1");
        builder.node_mut(call_a).unwrap().children = vec![callee_a, array_a, value_a];

        let call_b = builder.alloc(LirNodeKind::Call);
        let callee_b = builder.alloc_text(LirNodeKind::Value, "consume_array");
        let array_b = builder.alloc(LirNodeKind::Value);
        let array_b_first = literal(&mut builder, "1");
        let array_b_second = literal(&mut builder, "2");
        let array_b_third = literal(&mut builder, "3");
        builder.node_mut(array_b).unwrap().children =
            vec![array_b_first, array_b_second, array_b_third];
        let value_b = literal(&mut builder, "1");
        builder.node_mut(call_b).unwrap().children = vec![callee_b, array_b, value_b];

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
                    bindings: Vec::new(),
                },
                kali_mir::MirFunction {
                    name: Some("consume_array".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "items".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::TaggedVal,
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "value".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::TaggedVal,
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
        assert!(specialized_name_a.starts_with("consume_array$spec$"));
        assert!(specialized_name_b.starts_with("consume_array$spec$"));

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
    fn release_recursively_specializes_nested_mir_call_sites() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let inner = builder.alloc_text(LirNodeKind::Instruction, "sum_pair");
        let inner_left = builder.alloc_text(LirNodeKind::Value, "left");
        let inner_right = builder.alloc_text(LirNodeKind::Value, "right");
        let inner_block = builder.alloc(LirNodeKind::Block);
        let inner_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let inner_add1 = builder.alloc_text(LirNodeKind::Value, "+");
        let inner_add2 = builder.alloc_text(LirNodeKind::Value, "+");
        let inner_add3 = builder.alloc_text(LirNodeKind::Value, "+");
        let inner_add4 = builder.alloc_text(LirNodeKind::Value, "+");
        let inner_add5 = builder.alloc_text(LirNodeKind::Value, "+");
        let inner_add6 = builder.alloc_text(LirNodeKind::Value, "+");
        let inner_add7 = builder.alloc_text(LirNodeKind::Value, "+");
        let inner_add8 = builder.alloc_text(LirNodeKind::Value, "+");
        let inner_one = literal(&mut builder, "1");
        let inner_two = literal(&mut builder, "2");
        let inner_three = literal(&mut builder, "3");
        let inner_four = literal(&mut builder, "4");
        let inner_five = literal(&mut builder, "5");
        let inner_six = literal(&mut builder, "6");
        let inner_seven = literal(&mut builder, "7");
        let inner_eight = literal(&mut builder, "8");
        builder.node_mut(inner_add1).unwrap().children = vec![inner_left, inner_right];
        builder.node_mut(inner_add2).unwrap().children = vec![inner_add1, inner_one];
        builder.node_mut(inner_add3).unwrap().children = vec![inner_add2, inner_two];
        builder.node_mut(inner_add4).unwrap().children = vec![inner_add3, inner_three];
        builder.node_mut(inner_add5).unwrap().children = vec![inner_add4, inner_four];
        builder.node_mut(inner_add6).unwrap().children = vec![inner_add5, inner_five];
        builder.node_mut(inner_add7).unwrap().children = vec![inner_add6, inner_six];
        builder.node_mut(inner_add8).unwrap().children = vec![inner_add7, inner_seven];
        builder.node_mut(inner_ret).unwrap().children = vec![inner_add8, inner_eight];
        builder.node_mut(inner_block).unwrap().children = vec![inner_ret];
        builder.node_mut(inner).unwrap().children = vec![inner_left, inner_right, inner_block];

        let outer = builder.alloc_text(LirNodeKind::Instruction, "use_sum_pair");
        let outer_left = builder.alloc_text(LirNodeKind::Value, "left");
        let outer_right = builder.alloc_text(LirNodeKind::Value, "right");
        let outer_block = builder.alloc(LirNodeKind::Block);
        let outer_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let outer_add1 = builder.alloc_text(LirNodeKind::Value, "+");
        let outer_add2 = builder.alloc_text(LirNodeKind::Value, "+");
        let outer_add3 = builder.alloc_text(LirNodeKind::Value, "+");
        let outer_add4 = builder.alloc_text(LirNodeKind::Value, "+");
        let outer_add5 = builder.alloc_text(LirNodeKind::Value, "+");
        let outer_add6 = builder.alloc_text(LirNodeKind::Value, "+");
        let outer_add7 = builder.alloc_text(LirNodeKind::Value, "+");
        let outer_add8 = builder.alloc_text(LirNodeKind::Value, "+");
        let outer_one = literal(&mut builder, "9");
        let outer_two = literal(&mut builder, "10");
        let outer_three = literal(&mut builder, "11");
        let outer_four = literal(&mut builder, "12");
        let outer_five = literal(&mut builder, "13");
        let outer_six = literal(&mut builder, "14");
        let outer_seven = literal(&mut builder, "15");
        let outer_eight = literal(&mut builder, "16");
        let nested_call = builder.alloc(LirNodeKind::Call);
        let nested_call_callee = builder.alloc_text(LirNodeKind::Value, "sum_pair");
        builder.node_mut(nested_call).unwrap().children =
            vec![nested_call_callee, outer_left, outer_right];
        builder.node_mut(outer_add1).unwrap().children = vec![nested_call, outer_one];
        builder.node_mut(outer_add2).unwrap().children = vec![outer_add1, outer_two];
        builder.node_mut(outer_add3).unwrap().children = vec![outer_add2, outer_three];
        builder.node_mut(outer_add4).unwrap().children = vec![outer_add3, outer_four];
        builder.node_mut(outer_add5).unwrap().children = vec![outer_add4, outer_five];
        builder.node_mut(outer_add6).unwrap().children = vec![outer_add5, outer_six];
        builder.node_mut(outer_add7).unwrap().children = vec![outer_add6, outer_seven];
        builder.node_mut(outer_add8).unwrap().children = vec![outer_add7, outer_eight];
        builder.node_mut(outer_ret).unwrap().children = vec![outer_add8];
        builder.node_mut(outer_block).unwrap().children = vec![outer_ret];
        builder.node_mut(outer).unwrap().children = vec![outer_left, outer_right, outer_block];

        let call = builder.alloc(LirNodeKind::Call);
        let callee = builder.alloc_text(LirNodeKind::Value, "use_sum_pair");
        let two = literal(&mut builder, "2");
        let three = literal(&mut builder, "3");
        builder.node_mut(call).unwrap().children = vec![callee, two, three];

        builder.node_mut(root).unwrap().children = vec![inner, outer, call];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![
                kali_mir::MirFunction {
                    name: Some("sum_pair".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "left".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::TaggedVal,
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "right".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::TaggedVal,
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                    ],
                },
                kali_mir::MirFunction {
                    name: Some("use_sum_pair".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "left".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::TaggedVal,
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "right".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::TaggedVal,
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                    ],
                },
            ],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let outer_specialized_name = program.nodes[call.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized outer call target should exist");
        assert!(outer_specialized_name.starts_with("use_sum_pair$spec$"));

        let inner_specialized_name = program
            .nodes
            .iter()
            .find(|node| {
                node.kind == LirNodeKind::Instruction
                    && node
                        .text
                        .as_deref()
                        .is_some_and(|text| text.starts_with("sum_pair$spec$"))
            })
            .and_then(|node| node.text.as_deref())
            .expect("nested specialized inner call target should exist");
        assert!(inner_specialized_name.starts_with("sum_pair$spec$"));

        let inner_specialized_count = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(inner_specialized_name)
            })
            .count();
        assert_eq!(inner_specialized_count, 1);
    }

    #[test]
    fn release_specializes_same_binding_name_in_distinct_function_scopes() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let callee = builder.alloc_text(LirNodeKind::Instruction, "consume_point");
        let callee_param_point = builder.alloc_text(LirNodeKind::Value, "point");
        let callee_param_value = builder.alloc_text(LirNodeKind::Value, "value");
        let callee_block = builder.alloc(LirNodeKind::Block);
        let callee_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let callee_expr1 = builder.alloc_text(LirNodeKind::Value, "+");
        let callee_expr2 = builder.alloc_text(LirNodeKind::Value, "+");
        let callee_expr3 = builder.alloc_text(LirNodeKind::Value, "+");
        let callee_expr4 = builder.alloc_text(LirNodeKind::Value, "+");
        let callee_expr5 = builder.alloc_text(LirNodeKind::Value, "+");
        let callee_expr6 = builder.alloc_text(LirNodeKind::Value, "+");
        let callee_expr7 = builder.alloc_text(LirNodeKind::Value, "+");
        let callee_expr8 = builder.alloc_text(LirNodeKind::Value, "+");
        let callee_rhs = literal(&mut builder, "2");
        let callee_rhs2 = literal(&mut builder, "3");
        let callee_rhs3 = literal(&mut builder, "4");
        let callee_rhs4 = literal(&mut builder, "5");
        let callee_rhs5 = literal(&mut builder, "6");
        let callee_rhs6 = literal(&mut builder, "7");
        let callee_rhs7 = literal(&mut builder, "8");
        let callee_rhs8 = literal(&mut builder, "9");
        builder.node_mut(callee_expr1).unwrap().children =
            vec![callee_param_point, callee_param_value];
        builder.node_mut(callee_expr2).unwrap().children = vec![callee_expr1, callee_rhs];
        builder.node_mut(callee_expr3).unwrap().children = vec![callee_expr2, callee_rhs2];
        builder.node_mut(callee_expr4).unwrap().children = vec![callee_expr3, callee_rhs3];
        builder.node_mut(callee_expr5).unwrap().children = vec![callee_expr4, callee_rhs4];
        builder.node_mut(callee_expr6).unwrap().children = vec![callee_expr5, callee_rhs5];
        builder.node_mut(callee_expr7).unwrap().children = vec![callee_expr6, callee_rhs6];
        builder.node_mut(callee_expr8).unwrap().children = vec![callee_expr7, callee_rhs7];
        builder.node_mut(callee_ret).unwrap().children = vec![callee_expr8, callee_rhs8];
        builder.node_mut(callee_block).unwrap().children = vec![callee_ret];
        builder.node_mut(callee).unwrap().children =
            vec![callee_param_point, callee_param_value, callee_block];

        let caller_a = builder.alloc_text(LirNodeKind::Instruction, "use_a");
        let caller_a_point = builder.alloc_text(LirNodeKind::Value, "point");
        let caller_a_value = builder.alloc_text(LirNodeKind::Value, "value");
        let caller_a_block = builder.alloc(LirNodeKind::Block);
        let caller_a_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let call_a = builder.alloc(LirNodeKind::Call);
        let call_a_callee = builder.alloc_text(LirNodeKind::Value, "consume_point");
        builder.node_mut(call_a).unwrap().children =
            vec![call_a_callee, caller_a_point, caller_a_value];
        builder.node_mut(caller_a_ret).unwrap().children = vec![call_a];
        builder.node_mut(caller_a_block).unwrap().children = vec![caller_a_ret];
        builder.node_mut(caller_a).unwrap().children =
            vec![caller_a_point, caller_a_value, caller_a_block];

        let caller_b = builder.alloc_text(LirNodeKind::Instruction, "use_b");
        let caller_b_point = builder.alloc_text(LirNodeKind::Value, "point");
        let caller_b_value = builder.alloc_text(LirNodeKind::Value, "value");
        let caller_b_block = builder.alloc(LirNodeKind::Block);
        let caller_b_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let call_b = builder.alloc(LirNodeKind::Call);
        let call_b_callee = builder.alloc_text(LirNodeKind::Value, "consume_point");
        builder.node_mut(call_b).unwrap().children =
            vec![call_b_callee, caller_b_point, caller_b_value];
        builder.node_mut(caller_b_ret).unwrap().children = vec![call_b];
        builder.node_mut(caller_b_block).unwrap().children = vec![caller_b_ret];
        builder.node_mut(caller_b).unwrap().children =
            vec![caller_b_point, caller_b_value, caller_b_block];

        builder.node_mut(root).unwrap().children = vec![callee, caller_a, caller_b];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let point_layout_a = LayoutDescriptor::Struct {
            fields: vec![(
                "x".to_string(),
                Box::new(LayoutDescriptor::Scalar("number".to_string())),
            )],
        };
        let point_layout_b = LayoutDescriptor::Struct {
            fields: vec![(
                "y".to_string(),
                Box::new(LayoutDescriptor::Scalar("number".to_string())),
            )],
        };
        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![
                kali_mir::MirFunction {
                    name: Some("consume_point".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "point".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: point_layout_a.clone(),
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
                kali_mir::MirFunction {
                    name: Some("use_a".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "point".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: point_layout_a,
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
                kali_mir::MirFunction {
                    name: Some("use_b".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "point".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: point_layout_b,
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

        let specialized_name_a = program.nodes[call_a.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for caller_a");
        let specialized_name_b = program.nodes[call_b.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for caller_b");

        assert_ne!(specialized_name_a, specialized_name_b);
        assert!(specialized_name_a.starts_with("consume_point$spec$"));
        assert!(specialized_name_b.starts_with("consume_point$spec$"));
    }

    #[test]
    fn release_specializes_tagged_parameters_from_concrete_arguments() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "add_pair");
        let param_left = builder.alloc_text(LirNodeKind::Value, "left");
        let param_right = builder.alloc_text(LirNodeKind::Value, "right");
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
        builder.node_mut(add1).unwrap().children = vec![param_left, param_right];
        builder.node_mut(add2).unwrap().children = vec![add1, one];
        builder.node_mut(add3).unwrap().children = vec![add2, two];
        builder.node_mut(add4).unwrap().children = vec![add3, three];
        builder.node_mut(add5).unwrap().children = vec![add4, four];
        builder.node_mut(add6).unwrap().children = vec![add5, five];
        builder.node_mut(add7).unwrap().children = vec![add6, six];
        builder.node_mut(add8).unwrap().children = vec![add7, seven];
        builder.node_mut(ret).unwrap().children = vec![add8, eight];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_left, param_right, block];

        let call = builder.alloc(LirNodeKind::Call);
        let callee = builder.alloc_text(LirNodeKind::Value, "add_pair");
        let left = literal(&mut builder, "2");
        let right = literal(&mut builder, "3");
        builder.node_mut(call).unwrap().children = vec![callee, left, right];

        builder.node_mut(root).unwrap().children = vec![function, call];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![kali_mir::MirFunction {
                name: Some("add_pair".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                bindings: vec![
                    kali_mir::MirBinding {
                        name: "left".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::TaggedVal,
                        escapes: false,
                        captured_by: Vec::new(),
                    },
                    kali_mir::MirBinding {
                        name: "right".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::TaggedVal,
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
            .expect("specialized call target should exist for tagged parameters");
        assert!(specialized_name.starts_with("add_pair$spec$"));

        let specialized_function = program
            .nodes
            .iter()
            .find(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name)
            })
            .expect("specialized function should be inserted for tagged parameters");
        let literal_thirty_three = program
            .nodes
            .iter()
            .any(|node| node.kind == LirNodeKind::Literal && node.text.as_deref() == Some("33"));
        assert!(
            literal_thirty_three,
            "tagged-parameter specialization should still expose the folded literal result"
        );
        assert_eq!(specialized_function.kind, LirNodeKind::Instruction);
    }

    #[test]
    fn release_specializes_tagged_parameters_for_non_inlined_functions() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "sum_chain");
        let param_left = builder.alloc_text(LirNodeKind::Value, "left");
        let param_right = builder.alloc_text(LirNodeKind::Value, "right");
        let block = builder.alloc(LirNodeKind::Block);
        let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let add1 = builder.alloc_text(LirNodeKind::Value, "+");
        let add2 = builder.alloc_text(LirNodeKind::Value, "+");
        let add3 = builder.alloc_text(LirNodeKind::Value, "+");
        let add4 = builder.alloc_text(LirNodeKind::Value, "+");
        let add5 = builder.alloc_text(LirNodeKind::Value, "+");
        let add6 = builder.alloc_text(LirNodeKind::Value, "+");
        let one = literal(&mut builder, "1");
        let two = literal(&mut builder, "2");
        let three = literal(&mut builder, "3");
        let four = literal(&mut builder, "4");
        let five = literal(&mut builder, "5");
        builder.node_mut(add1).unwrap().children = vec![param_left, param_right];
        builder.node_mut(add2).unwrap().children = vec![add1, one];
        builder.node_mut(add3).unwrap().children = vec![add2, two];
        builder.node_mut(add4).unwrap().children = vec![add3, three];
        builder.node_mut(add5).unwrap().children = vec![add4, four];
        builder.node_mut(add6).unwrap().children = vec![add5, five];
        builder.node_mut(ret).unwrap().children = vec![add6];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_left, param_right, block];

        let call = builder.alloc(LirNodeKind::Call);
        let callee = builder.alloc_text(LirNodeKind::Value, "sum_chain");
        let left = literal(&mut builder, "2");
        let right = literal(&mut builder, "3");
        builder.node_mut(call).unwrap().children = vec![callee, left, right];

        builder.node_mut(root).unwrap().children = vec![function, call];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![kali_mir::MirFunction {
                name: Some("sum_chain".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                bindings: vec![
                    kali_mir::MirBinding {
                        name: "left".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::TaggedVal,
                        escapes: false,
                        captured_by: Vec::new(),
                    },
                    kali_mir::MirBinding {
                        name: "right".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::TaggedVal,
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
            .expect("specialized call target should exist for non-inlined tagged parameters");
        assert!(specialized_name.starts_with("sum_chain$spec$"));

        let specialized_function = program
            .nodes
            .iter()
            .find(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name)
            })
            .expect("specialized function should be inserted for non-inlined tagged parameters");
        let literal_twenty = program
            .nodes
            .iter()
            .any(|node| node.kind == LirNodeKind::Literal && node.text.as_deref() == Some("20"));
        assert!(
            literal_twenty,
            "non-inlined tagged-parameter specialization should still expose the folded literal result"
        );
        assert_eq!(specialized_function.kind, LirNodeKind::Instruction);
    }

    #[test]
    fn release_specializes_concrete_arguments_without_mir_layouts() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "merge_pair");
        let param_left = builder.alloc_text(LirNodeKind::Value, "left");
        let param_right = builder.alloc_text(LirNodeKind::Value, "right");
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
        builder.node_mut(add1).unwrap().children = vec![param_left, param_right];
        builder.node_mut(add2).unwrap().children = vec![add1, one];
        builder.node_mut(add3).unwrap().children = vec![add2, two];
        builder.node_mut(add4).unwrap().children = vec![add3, three];
        builder.node_mut(add5).unwrap().children = vec![add4, four];
        builder.node_mut(add6).unwrap().children = vec![add5, five];
        builder.node_mut(add7).unwrap().children = vec![add6, six];
        builder.node_mut(add8).unwrap().children = vec![add7, seven];
        builder.node_mut(ret).unwrap().children = vec![add8, eight];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_left, param_right, block];

        let call = builder.alloc(LirNodeKind::Call);
        let callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
        let left = literal(&mut builder, "2");
        let right = literal(&mut builder, "3");
        builder.node_mut(call).unwrap().children = vec![callee, left, right];

        builder.node_mut(root).unwrap().children = vec![function, call];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

        let call_node = &program.nodes[call.0 as usize];
        let specialized_name = call_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist without MIR layouts");
        assert!(specialized_name.starts_with("merge_pair$spec$"));

        let specialized_function = program
            .nodes
            .iter()
            .find(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name)
            })
            .expect("specialized function should be inserted without MIR layouts");
        assert_eq!(specialized_function.kind, LirNodeKind::Instruction);
        assert!(
            program
                .nodes
                .iter()
                .filter(|node| {
                    node.kind == LirNodeKind::Instruction
                        && node.text.as_deref() == Some(specialized_name)
                })
                .count()
                == 1,
            "specialized function should only be cloned once"
        );
    }

    #[test]
    fn release_specializes_string_literal_arguments() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "echo_text");
        let param_text = builder.alloc_text(LirNodeKind::Value, "text");
        let block = builder.alloc(LirNodeKind::Block);
        let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let add1 = builder.alloc_text(LirNodeKind::Value, "+");
        let add2 = builder.alloc_text(LirNodeKind::Value, "+");
        let add3 = builder.alloc_text(LirNodeKind::Value, "+");
        let add4 = builder.alloc_text(LirNodeKind::Value, "+");
        let add5 = builder.alloc_text(LirNodeKind::Value, "+");
        let add6 = builder.alloc_text(LirNodeKind::Value, "+");
        let one = literal(&mut builder, "1");
        let two = literal(&mut builder, "2");
        let three = literal(&mut builder, "3");
        let four = literal(&mut builder, "4");
        let five = literal(&mut builder, "5");
        let six = literal(&mut builder, "6");
        builder.node_mut(add1).unwrap().children = vec![param_text, one];
        builder.node_mut(add2).unwrap().children = vec![add1, two];
        builder.node_mut(add3).unwrap().children = vec![add2, three];
        builder.node_mut(add4).unwrap().children = vec![add3, four];
        builder.node_mut(add5).unwrap().children = vec![add4, five];
        builder.node_mut(add6).unwrap().children = vec![add5, six];
        builder.node_mut(ret).unwrap().children = vec![add6];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_text, block];

        let call_a = builder.alloc(LirNodeKind::Call);
        let call_a_callee = builder.alloc_text(LirNodeKind::Value, "echo_text");
        let arg_a = literal(&mut builder, "\"alpha\"");
        builder.node_mut(call_a).unwrap().children = vec![call_a_callee, arg_a];

        let call_b = builder.alloc(LirNodeKind::Call);
        let call_b_callee = builder.alloc_text(LirNodeKind::Value, "echo_text");
        let arg_b = literal(&mut builder, "\"beta\"");
        builder.node_mut(call_b).unwrap().children = vec![call_b_callee, arg_b];

        builder.node_mut(root).unwrap().children = vec![function, call_a, call_b];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![kali_mir::MirFunction {
                name: Some("echo_text".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                bindings: vec![kali_mir::MirBinding {
                    name: "text".to_string(),
                    kind: MirBindingKind::Parameter,
                    ownership: kali_mir::OwnershipClass::Borrowed,
                    layout: LayoutDescriptor::TaggedVal,
                    escapes: false,
                    captured_by: Vec::new(),
                }],
            }],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let specialized_name_a = program.nodes[call_a.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for string literal A");
        let specialized_name_b = program.nodes[call_b.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for string literal B");

        assert_ne!(specialized_name_a, specialized_name_b);
        assert!(specialized_name_a.starts_with("echo_text$spec$"));
        assert!(specialized_name_b.starts_with("echo_text$spec$"));

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

    #[test]
    fn release_specializes_quoted_string_and_template_literal_arguments_distinctly() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "echo_text_variant");
        let param_text = builder.alloc_text(LirNodeKind::Value, "text");
        let block = builder.alloc(LirNodeKind::Block);
        let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
        let add1 = builder.alloc_text(LirNodeKind::Value, "+");
        let add2 = builder.alloc_text(LirNodeKind::Value, "+");
        let add3 = builder.alloc_text(LirNodeKind::Value, "+");
        let add4 = builder.alloc_text(LirNodeKind::Value, "+");
        let add5 = builder.alloc_text(LirNodeKind::Value, "+");
        let add6 = builder.alloc_text(LirNodeKind::Value, "+");
        let one = literal(&mut builder, "1");
        let two = literal(&mut builder, "2");
        let three = literal(&mut builder, "3");
        let four = literal(&mut builder, "4");
        let five = literal(&mut builder, "5");
        let six = literal(&mut builder, "6");
        builder.node_mut(add1).unwrap().children = vec![param_text, one];
        builder.node_mut(add2).unwrap().children = vec![add1, two];
        builder.node_mut(add3).unwrap().children = vec![add2, three];
        builder.node_mut(add4).unwrap().children = vec![add3, four];
        builder.node_mut(add5).unwrap().children = vec![add4, five];
        builder.node_mut(add6).unwrap().children = vec![add5, six];
        builder.node_mut(ret).unwrap().children = vec![add6];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_text, block];

        let call_quoted = builder.alloc(LirNodeKind::Call);
        let call_quoted_callee = builder.alloc_text(LirNodeKind::Value, "echo_text_variant");
        let quoted = literal(&mut builder, "\"alpha\"");
        builder.node_mut(call_quoted).unwrap().children = vec![call_quoted_callee, quoted];

        let call_template = builder.alloc(LirNodeKind::Call);
        let call_template_callee = builder.alloc_text(LirNodeKind::Value, "echo_text_variant");
        let template = literal(&mut builder, "`alpha`");
        builder.node_mut(call_template).unwrap().children = vec![call_template_callee, template];

        let call_quoted_b = builder.alloc(LirNodeKind::Call);
        let call_quoted_b_callee = builder.alloc_text(LirNodeKind::Value, "echo_text_variant");
        let quoted_b = literal(&mut builder, "\"alpha\"");
        builder.node_mut(call_quoted_b).unwrap().children = vec![call_quoted_b_callee, quoted_b];

        builder.node_mut(root).unwrap().children =
            vec![function, call_quoted, call_template, call_quoted_b];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![kali_mir::MirFunction {
                name: Some("echo_text_variant".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                bindings: vec![kali_mir::MirBinding {
                    name: "text".to_string(),
                    kind: MirBindingKind::Parameter,
                    ownership: kali_mir::OwnershipClass::Borrowed,
                    layout: LayoutDescriptor::TaggedVal,
                    escapes: false,
                    captured_by: Vec::new(),
                }],
            }],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let quoted_name = program.nodes[call_quoted.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for quoted string literal");
        let template_name = program.nodes[call_template.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for template literal");
        let quoted_name_b = program.nodes[call_quoted_b.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for repeated quoted string literal");

        assert_eq!(quoted_name, quoted_name_b);
        assert_ne!(quoted_name, template_name);
        assert!(quoted_name.starts_with("echo_text_variant$spec$"));
        assert!(template_name.starts_with("echo_text_variant$spec$"));
    }

    #[test]
    fn release_specializes_nullish_literal_arguments() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_value");
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
        builder.node_mut(function).unwrap().children = vec![param_value, block];

        let call_null_a = builder.alloc(LirNodeKind::Call);
        let callee_null_a = builder.alloc_text(LirNodeKind::Value, "consume_value");
        let null_a = literal(&mut builder, "null");
        builder.node_mut(call_null_a).unwrap().children = vec![callee_null_a, null_a];

        let call_undefined = builder.alloc(LirNodeKind::Call);
        let callee_undefined = builder.alloc_text(LirNodeKind::Value, "consume_value");
        let undefined = literal(&mut builder, "undefined");
        builder.node_mut(call_undefined).unwrap().children = vec![callee_undefined, undefined];

        let call_null_b = builder.alloc(LirNodeKind::Call);
        let callee_null_b = builder.alloc_text(LirNodeKind::Value, "consume_value");
        let null_b = literal(&mut builder, "null");
        builder.node_mut(call_null_b).unwrap().children = vec![callee_null_b, null_b];

        builder.node_mut(root).unwrap().children =
            vec![function, call_null_a, call_undefined, call_null_b];
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
                    bindings: Vec::new(),
                },
                kali_mir::MirFunction {
                    name: Some("consume_value".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![kali_mir::MirBinding {
                        name: "value".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::TaggedVal,
                        escapes: false,
                        captured_by: Vec::new(),
                    }],
                },
            ],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let call_null_a_node = &program.nodes[call_null_a.0 as usize];
        let call_undefined_node = &program.nodes[call_undefined.0 as usize];
        let call_null_b_node = &program.nodes[call_null_b.0 as usize];
        let specialized_name_null_a = call_null_a_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for null_a");
        let specialized_name_undefined = call_undefined_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for undefined");
        let specialized_name_null_b = call_null_b_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for null_b");

        assert_eq!(specialized_name_null_a, specialized_name_null_b);
        assert_ne!(specialized_name_null_a, specialized_name_undefined);
        assert!(specialized_name_null_a.starts_with("consume_value$spec$"));
        assert!(specialized_name_undefined.starts_with("consume_value$spec$"));

        let specialized_count_null = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_null_a)
            })
            .count();
        let specialized_count_undefined = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_undefined)
            })
            .count();
        assert_eq!(specialized_count_null, 1);
        assert_eq!(specialized_count_undefined, 1);
    }

    #[test]
    fn release_specializes_infinity_and_nan_literal_arguments() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_special_number");
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
        builder.node_mut(function).unwrap().children = vec![param_value, block];

        let call_infinity_a = builder.alloc(LirNodeKind::Call);
        let callee_infinity_a = builder.alloc_text(LirNodeKind::Value, "consume_special_number");
        let infinity_a = literal(&mut builder, "Infinity");
        builder.node_mut(call_infinity_a).unwrap().children = vec![callee_infinity_a, infinity_a];

        let call_nan = builder.alloc(LirNodeKind::Call);
        let callee_nan = builder.alloc_text(LirNodeKind::Value, "consume_special_number");
        let nan = literal(&mut builder, "NaN");
        builder.node_mut(call_nan).unwrap().children = vec![callee_nan, nan];

        let call_negative_infinity = builder.alloc(LirNodeKind::Call);
        let callee_negative_infinity =
            builder.alloc_text(LirNodeKind::Value, "consume_special_number");
        let negative_infinity = literal(&mut builder, "-Infinity");
        builder.node_mut(call_negative_infinity).unwrap().children =
            vec![callee_negative_infinity, negative_infinity];

        let call_infinity_b = builder.alloc(LirNodeKind::Call);
        let callee_infinity_b = builder.alloc_text(LirNodeKind::Value, "consume_special_number");
        let infinity_b = literal(&mut builder, "Infinity");
        builder.node_mut(call_infinity_b).unwrap().children = vec![callee_infinity_b, infinity_b];

        builder.node_mut(root).unwrap().children = vec![
            function,
            call_infinity_a,
            call_nan,
            call_negative_infinity,
            call_infinity_b,
        ];
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
                    bindings: Vec::new(),
                },
                kali_mir::MirFunction {
                    name: Some("consume_special_number".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![kali_mir::MirBinding {
                        name: "value".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::TaggedVal,
                        escapes: false,
                        captured_by: Vec::new(),
                    }],
                },
            ],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let specialized_name_infinity_a = program.nodes[call_infinity_a.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for infinity_a");
        let specialized_name_nan = program.nodes[call_nan.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for nan");
        let specialized_name_negative_infinity = program.nodes[call_negative_infinity.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for negative_infinity");
        let specialized_name_infinity_b = program.nodes[call_infinity_b.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for infinity_b");

        assert_eq!(specialized_name_infinity_a, specialized_name_infinity_b);
        assert_ne!(specialized_name_infinity_a, specialized_name_nan);
        assert_ne!(
            specialized_name_infinity_a,
            specialized_name_negative_infinity
        );
        assert_ne!(specialized_name_nan, specialized_name_negative_infinity);
        assert!(specialized_name_infinity_a.starts_with("consume_special_number$spec$"));
        assert!(specialized_name_nan.starts_with("consume_special_number$spec$"));
        assert!(specialized_name_negative_infinity.starts_with("consume_special_number$spec$"));

        let specialized_count_infinity = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_infinity_a)
            })
            .count();
        let specialized_count_nan = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_nan)
            })
            .count();
        let specialized_count_negative_infinity = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_negative_infinity)
            })
            .count();
        assert_eq!(specialized_count_infinity, 1);
        assert_eq!(specialized_count_nan, 1);
        assert_eq!(specialized_count_negative_infinity, 1);
    }

    #[test]
    fn release_specializes_boolean_literal_arguments() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_flag");
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
        builder.node_mut(function).unwrap().children = vec![param_value, block];

        let call_true_a = builder.alloc(LirNodeKind::Call);
        let callee_true_a = builder.alloc_text(LirNodeKind::Value, "consume_flag");
        let true_a = literal(&mut builder, "true");
        builder.node_mut(call_true_a).unwrap().children = vec![callee_true_a, true_a];

        let call_false = builder.alloc(LirNodeKind::Call);
        let callee_false = builder.alloc_text(LirNodeKind::Value, "consume_flag");
        let false_lit = literal(&mut builder, "false");
        builder.node_mut(call_false).unwrap().children = vec![callee_false, false_lit];

        let call_true_b = builder.alloc(LirNodeKind::Call);
        let callee_true_b = builder.alloc_text(LirNodeKind::Value, "consume_flag");
        let true_b = literal(&mut builder, "true");
        builder.node_mut(call_true_b).unwrap().children = vec![callee_true_b, true_b];

        builder.node_mut(root).unwrap().children =
            vec![function, call_true_a, call_false, call_true_b];
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
                    bindings: Vec::new(),
                },
                kali_mir::MirFunction {
                    name: Some("consume_flag".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![kali_mir::MirBinding {
                        name: "value".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::TaggedVal,
                        escapes: false,
                        captured_by: Vec::new(),
                    }],
                },
            ],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let call_true_a_node = &program.nodes[call_true_a.0 as usize];
        let call_false_node = &program.nodes[call_false.0 as usize];
        let call_true_b_node = &program.nodes[call_true_b.0 as usize];
        let specialized_name_true_a = call_true_a_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for true_a");
        let specialized_name_false = call_false_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for false");
        let specialized_name_true_b = call_true_b_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for true_b");

        assert_eq!(specialized_name_true_a, specialized_name_true_b);
        assert_ne!(specialized_name_true_a, specialized_name_false);
        assert!(specialized_name_true_a.starts_with("consume_flag$spec$"));
        assert!(specialized_name_false.starts_with("consume_flag$spec$"));

        let specialized_count_true = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_true_a)
            })
            .count();
        let specialized_count_false = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_false)
            })
            .count();
        assert_eq!(specialized_count_true, 1);
        assert_eq!(specialized_count_false, 1);
    }

    #[test]
    fn release_specializes_numeric_literal_arguments() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_number");
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
        builder.node_mut(function).unwrap().children = vec![param_value, block];

        let call_one_a = builder.alloc(LirNodeKind::Call);
        let call_one_a_callee = builder.alloc_text(LirNodeKind::Value, "consume_number");
        let arg_one_a = literal(&mut builder, "1");
        builder.node_mut(call_one_a).unwrap().children = vec![call_one_a_callee, arg_one_a];

        let call_two = builder.alloc(LirNodeKind::Call);
        let call_two_callee = builder.alloc_text(LirNodeKind::Value, "consume_number");
        let arg_two = literal(&mut builder, "2");
        builder.node_mut(call_two).unwrap().children = vec![call_two_callee, arg_two];

        let call_one_b = builder.alloc(LirNodeKind::Call);
        let call_one_b_callee = builder.alloc_text(LirNodeKind::Value, "consume_number");
        let arg_one_b = literal(&mut builder, "1");
        builder.node_mut(call_one_b).unwrap().children = vec![call_one_b_callee, arg_one_b];

        builder.node_mut(root).unwrap().children = vec![function, call_one_a, call_two, call_one_b];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![kali_mir::MirFunction {
                name: Some("consume_number".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                bindings: vec![kali_mir::MirBinding {
                    name: "value".to_string(),
                    kind: MirBindingKind::Parameter,
                    ownership: kali_mir::OwnershipClass::Borrowed,
                    layout: LayoutDescriptor::TaggedVal,
                    escapes: false,
                    captured_by: Vec::new(),
                }],
            }],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let specialized_name_one_a = program.nodes[call_one_a.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for one_a");
        let specialized_name_two = program.nodes[call_two.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for two");
        let specialized_name_one_b = program.nodes[call_one_b.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for one_b");

        assert_eq!(specialized_name_one_a, specialized_name_one_b);
        assert_ne!(specialized_name_one_a, specialized_name_two);
        assert!(specialized_name_one_a.starts_with("consume_number$spec$"));
        assert!(specialized_name_two.starts_with("consume_number$spec$"));

        let specialized_count_one = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_one_a)
            })
            .count();
        let specialized_count_two = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_two)
            })
            .count();
        assert_eq!(specialized_count_one, 1);
        assert_eq!(specialized_count_two, 1);
    }

    #[test]
    fn release_specializes_negative_zero_literal_arguments() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_zero");
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
        let zero = literal(&mut builder, "0");
        let neg_zero_a = literal(&mut builder, "-0");
        let neg_zero_b = literal(&mut builder, "-0");
        let neg_zero_c = literal(&mut builder, "-0");
        let neg_zero_d = literal(&mut builder, "-0");
        let neg_zero_e = literal(&mut builder, "-0");
        let neg_zero_f = literal(&mut builder, "-0");
        let neg_zero_g = literal(&mut builder, "-0");
        builder.node_mut(add1).unwrap().children = vec![param_value, zero];
        builder.node_mut(add2).unwrap().children = vec![add1, neg_zero_a];
        builder.node_mut(add3).unwrap().children = vec![add2, neg_zero_b];
        builder.node_mut(add4).unwrap().children = vec![add3, neg_zero_c];
        builder.node_mut(add5).unwrap().children = vec![add4, neg_zero_d];
        builder.node_mut(add6).unwrap().children = vec![add5, neg_zero_e];
        builder.node_mut(add7).unwrap().children = vec![add6, neg_zero_f];
        builder.node_mut(add8).unwrap().children = vec![add7, neg_zero_g];
        builder.node_mut(ret).unwrap().children = vec![add8];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_value, block];

        let call_zero_a = builder.alloc(LirNodeKind::Call);
        let call_zero_a_callee = builder.alloc_text(LirNodeKind::Value, "consume_zero");
        let arg_zero_a = literal(&mut builder, "0");
        builder.node_mut(call_zero_a).unwrap().children = vec![call_zero_a_callee, arg_zero_a];

        let call_neg_zero = builder.alloc(LirNodeKind::Call);
        let call_neg_zero_callee = builder.alloc_text(LirNodeKind::Value, "consume_zero");
        let arg_neg_zero = literal(&mut builder, "-0");
        builder.node_mut(call_neg_zero).unwrap().children =
            vec![call_neg_zero_callee, arg_neg_zero];

        let call_zero_b = builder.alloc(LirNodeKind::Call);
        let call_zero_b_callee = builder.alloc_text(LirNodeKind::Value, "consume_zero");
        let arg_zero_b = literal(&mut builder, "0");
        builder.node_mut(call_zero_b).unwrap().children = vec![call_zero_b_callee, arg_zero_b];

        builder.node_mut(root).unwrap().children =
            vec![function, call_zero_a, call_neg_zero, call_zero_b];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![kali_mir::MirFunction {
                name: Some("consume_zero".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                bindings: vec![kali_mir::MirBinding {
                    name: "value".to_string(),
                    kind: MirBindingKind::Parameter,
                    ownership: kali_mir::OwnershipClass::Borrowed,
                    layout: LayoutDescriptor::TaggedVal,
                    escapes: false,
                    captured_by: Vec::new(),
                }],
            }],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let specialized_name_zero_a = program.nodes[call_zero_a.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for zero_a");
        let specialized_name_neg_zero = program.nodes[call_neg_zero.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for neg_zero");
        let specialized_name_zero_b = program.nodes[call_zero_b.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for zero_b");

        assert_eq!(specialized_name_zero_a, specialized_name_zero_b);
        assert_ne!(specialized_name_zero_a, specialized_name_neg_zero);
        assert!(specialized_name_zero_a.starts_with("consume_zero$spec$"));
        assert!(specialized_name_neg_zero.starts_with("consume_zero$spec$"));

        let specialized_count_zero = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_zero_a)
            })
            .count();
        let specialized_count_neg_zero = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_neg_zero)
            })
            .count();
        assert_eq!(specialized_count_zero, 1);
        assert_eq!(specialized_count_neg_zero, 1);
    }

    #[test]
    fn release_specializes_bigint_literal_arguments() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_bigint");
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
        let one = literal(&mut builder, "1n");
        let two = literal(&mut builder, "2n");
        let three = literal(&mut builder, "3n");
        let four = literal(&mut builder, "4n");
        let five = literal(&mut builder, "5n");
        let six = literal(&mut builder, "6n");
        let seven = literal(&mut builder, "7n");
        let eight = literal(&mut builder, "8n");
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
        builder.node_mut(function).unwrap().children = vec![param_value, block];

        let call_one_a = builder.alloc(LirNodeKind::Call);
        let call_one_a_callee = builder.alloc_text(LirNodeKind::Value, "consume_bigint");
        let arg_one_a = literal(&mut builder, "1n");
        builder.node_mut(call_one_a).unwrap().children = vec![call_one_a_callee, arg_one_a];

        let call_two = builder.alloc(LirNodeKind::Call);
        let call_two_callee = builder.alloc_text(LirNodeKind::Value, "consume_bigint");
        let arg_two = literal(&mut builder, "2n");
        builder.node_mut(call_two).unwrap().children = vec![call_two_callee, arg_two];

        let call_one_b = builder.alloc(LirNodeKind::Call);
        let call_one_b_callee = builder.alloc_text(LirNodeKind::Value, "consume_bigint");
        let arg_one_b = literal(&mut builder, "1n");
        builder.node_mut(call_one_b).unwrap().children = vec![call_one_b_callee, arg_one_b];

        builder.node_mut(root).unwrap().children = vec![function, call_one_a, call_two, call_one_b];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![kali_mir::MirFunction {
                name: Some("consume_bigint".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                bindings: vec![kali_mir::MirBinding {
                    name: "value".to_string(),
                    kind: MirBindingKind::Parameter,
                    ownership: kali_mir::OwnershipClass::Borrowed,
                    layout: LayoutDescriptor::TaggedVal,
                    escapes: false,
                    captured_by: Vec::new(),
                }],
            }],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let specialized_name_one_a = program.nodes[call_one_a.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for one_a");
        let specialized_name_two = program.nodes[call_two.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for two");
        let specialized_name_one_b = program.nodes[call_one_b.0 as usize]
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("specialized call target should exist for one_b");

        assert_eq!(specialized_name_one_a, specialized_name_one_b);
        assert_ne!(specialized_name_one_a, specialized_name_two);
        assert!(specialized_name_one_a.starts_with("consume_bigint$spec$"));
        assert!(specialized_name_two.starts_with("consume_bigint$spec$"));

        let specialized_count_one = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_one_a)
            })
            .count();
        let specialized_count_two = program
            .nodes
            .iter()
            .filter(|node| {
                node.kind == LirNodeKind::Instruction
                    && node.text.as_deref() == Some(specialized_name_two)
            })
            .count();
        assert_eq!(specialized_count_one, 1);
        assert_eq!(specialized_count_two, 1);
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
                                captures: vec!["scope_shared".to_string()],
                            },
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "handler_b".to_string(),
                            kind: MirBindingKind::Local,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: LayoutDescriptor::Closure {
                                captures: vec!["scope_shared".to_string()],
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
    fn release_specializes_distinct_closure_capture_bindings() {
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
        assert_ne!(specialized_name_a, specialized_name_b);
        assert!(specialized_name_a.starts_with("consume_handler$spec$"));
        assert!(specialized_name_b.starts_with("consume_handler$spec$"));

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

    #[test]
    fn release_specializes_nested_mir_bound_bindings_inside_object_literals() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_point");
        let param_point = builder.alloc_text(LirNodeKind::Value, "point");
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
        builder.node_mut(add1).unwrap().children = vec![param_point, one];
        builder.node_mut(add2).unwrap().children = vec![add1, two];
        builder.node_mut(add3).unwrap().children = vec![add2, three];
        builder.node_mut(add4).unwrap().children = vec![add3, four];
        builder.node_mut(add5).unwrap().children = vec![add4, five];
        builder.node_mut(add6).unwrap().children = vec![add5, six];
        builder.node_mut(add7).unwrap().children = vec![add6, seven];
        builder.node_mut(add8).unwrap().children = vec![add7, eight];
        builder.node_mut(ret).unwrap().children = vec![add8];
        builder.node_mut(block).unwrap().children = vec![ret];
        builder.node_mut(function).unwrap().children = vec![param_point, block];

        let use_a = builder.alloc_text(LirNodeKind::Instruction, "use_a");
        let shared_a = builder.alloc_text(LirNodeKind::Value, "shared");
        let block_a = builder.alloc(LirNodeKind::Block);
        let ret_a = builder.alloc_text(LirNodeKind::Instruction, "return");
        let call_a = builder.alloc(LirNodeKind::Call);
        let callee_a = builder.alloc_text(LirNodeKind::Value, "consume_point");
        let object_a = builder.alloc(LirNodeKind::Value);
        let init_a = builder.alloc_text(LirNodeKind::Value, "init");
        let key_a = literal(&mut builder, "x");
        builder.node_mut(init_a).unwrap().children = vec![key_a, shared_a];
        builder.node_mut(object_a).unwrap().children = vec![init_a];
        builder.node_mut(call_a).unwrap().children = vec![callee_a, object_a];
        builder.node_mut(ret_a).unwrap().children = vec![call_a];
        builder.node_mut(block_a).unwrap().children = vec![ret_a];
        builder.node_mut(use_a).unwrap().children = vec![shared_a, block_a];

        let use_b = builder.alloc_text(LirNodeKind::Instruction, "use_b");
        let shared_b = builder.alloc_text(LirNodeKind::Value, "shared");
        let block_b = builder.alloc(LirNodeKind::Block);
        let ret_b = builder.alloc_text(LirNodeKind::Instruction, "return");
        let call_b = builder.alloc(LirNodeKind::Call);
        let callee_b = builder.alloc_text(LirNodeKind::Value, "consume_point");
        let object_b = builder.alloc(LirNodeKind::Value);
        let init_b = builder.alloc_text(LirNodeKind::Value, "init");
        let key_b = literal(&mut builder, "x");
        builder.node_mut(init_b).unwrap().children = vec![key_b, shared_b];
        builder.node_mut(object_b).unwrap().children = vec![init_b];
        builder.node_mut(call_b).unwrap().children = vec![callee_b, object_b];
        builder.node_mut(ret_b).unwrap().children = vec![call_b];
        builder.node_mut(block_b).unwrap().children = vec![ret_b];
        builder.node_mut(use_b).unwrap().children = vec![shared_b, block_b];

        builder.node_mut(root).unwrap().children = vec![function, use_a, use_b];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let shared_a_layout = LayoutDescriptor::Struct {
            fields: vec![(
                "left".to_string(),
                Box::new(LayoutDescriptor::Scalar("number".to_string())),
            )],
        };
        let shared_b_layout = LayoutDescriptor::Struct {
            fields: vec![(
                "right".to_string(),
                Box::new(LayoutDescriptor::Scalar("number".to_string())),
            )],
        };
        let point_layout = LayoutDescriptor::Struct {
            fields: vec![(
                "x".to_string(),
                Box::new(LayoutDescriptor::Scalar("number".to_string())),
            )],
        };
        let mir = MirAnalysisProgram {
            root: kali_mir::MirNodeId::new(0),
            nodes: Vec::new(),
            functions: vec![
                kali_mir::MirFunction {
                    name: None,
                    kind: kali_mir::MirFunctionKind::Module,
                    bindings: Vec::new(),
                },
                kali_mir::MirFunction {
                    name: Some("consume_point".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![kali_mir::MirBinding {
                        name: "point".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: point_layout,
                        escapes: false,
                        captured_by: Vec::new(),
                    }],
                },
                kali_mir::MirFunction {
                    name: Some("use_a".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![kali_mir::MirBinding {
                        name: "shared".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: shared_a_layout,
                        escapes: false,
                        captured_by: Vec::new(),
                    }],
                },
                kali_mir::MirFunction {
                    name: Some("use_b".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![kali_mir::MirBinding {
                        name: "shared".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: shared_b_layout,
                        escapes: false,
                        captured_by: Vec::new(),
                    }],
                },
            ],
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

        let specialized_names: BTreeSet<_> = program
            .nodes
            .iter()
            .filter_map(|node| {
                (node.kind == LirNodeKind::Instruction)
                    .then_some(node.text.as_deref())
                    .flatten()
            })
            .filter(|name| name.starts_with("consume_point$spec$"))
            .collect();
        assert_eq!(
            specialized_names.len(),
            2,
            "nested MIR-bound bindings inside object literals should drive distinct specializations"
        );
        assert!(specialized_names
            .iter()
            .all(|name| name.starts_with("consume_point$spec$")));
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

    #[test]
    fn release_specializes_distinct_array_layout_bindings() {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let function = builder.alloc_text(LirNodeKind::Instruction, "consume_array");
        let param_items = builder.alloc_text(LirNodeKind::Value, "items");
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
        builder.node_mut(function).unwrap().children = vec![param_items, param_value, block];

        let call_a = builder.alloc(LirNodeKind::Call);
        let callee_a = builder.alloc_text(LirNodeKind::Value, "consume_array");
        let items_a = builder.alloc_text(LirNodeKind::Value, "items_a");
        let value_a = literal(&mut builder, "1");
        builder.node_mut(call_a).unwrap().children = vec![callee_a, items_a, value_a];

        let call_b = builder.alloc(LirNodeKind::Call);
        let callee_b = builder.alloc_text(LirNodeKind::Value, "consume_array");
        let items_b = builder.alloc_text(LirNodeKind::Value, "items_b");
        let value_b = literal(&mut builder, "1");
        builder.node_mut(call_b).unwrap().children = vec![callee_b, items_b, value_b];

        builder.node_mut(root).unwrap().children = vec![function, call_a, call_b];
        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        let array_layout_a = LayoutDescriptor::Array {
            element: Box::new(LayoutDescriptor::Scalar("number".to_string())),
            length: Some(2),
        };
        let array_layout_b = LayoutDescriptor::Array {
            element: Box::new(LayoutDescriptor::Scalar("number".to_string())),
            length: Some(3),
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
                            name: "items_a".to_string(),
                            kind: MirBindingKind::Local,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: array_layout_a.clone(),
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                        kali_mir::MirBinding {
                            name: "items_b".to_string(),
                            kind: MirBindingKind::Local,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: array_layout_b.clone(),
                            escapes: false,
                            captured_by: Vec::new(),
                        },
                    ],
                },
                kali_mir::MirFunction {
                    name: Some("consume_array".to_string()),
                    kind: kali_mir::MirFunctionKind::Function,
                    bindings: vec![
                        kali_mir::MirBinding {
                            name: "items".to_string(),
                            kind: MirBindingKind::Parameter,
                            ownership: kali_mir::OwnershipClass::Borrowed,
                            layout: array_layout_a,
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
        assert!(specialized_name_a.starts_with("consume_array$spec$"));
        assert!(specialized_name_b.starts_with("consume_array$spec$"));

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
