//! Optimization passes for the Kali compiler.
//!
//! The current implementation focuses on the deterministic, tree-shaped LIR
//! that the rest of the repository already produces. That gives us a safe place
//! to land constant folding, branch elimination, and a handful of algebraic
//! simplifications without needing a full SSA pipeline yet.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

mod profile;

pub use profile::{ProfileData, ProfileSample, ProfileSampleKind, PROFILE_DATA_VERSION};

use kali_lir::{LirNode, LirNodeId, LirNodeKind, LirProgram};
use kali_mir::{LayoutDescriptor, MirBindingKind, MirProgram as MirAnalysisProgram};

/// Minimum recorded weight for a function sample to count as hot in the PGO report.
const HOT_FUNCTION_MINIMUM_WEIGHT: u64 = 8;

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

/// Deterministic summary of one optimization run configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptimizationReport {
    /// Requested optimization level.
    pub level: OptimizationLevel,
    /// Maximum specialization budget configured for the run.
    pub max_specializations: usize,
    /// Whether deterministic profile data was attached to the optimizer.
    pub profile_data_present: bool,
    /// Whether profile data actually contributed any hot-function inlining hints.
    pub profile_data_used_for_inlining: bool,
    /// Hot function keys discovered in the attached profile data.
    pub hot_function_keys: Vec<String>,
}

/// Optimizer context.
#[derive(Clone, Debug)]
pub struct Optimizer {
    level: OptimizationLevel,
    max_specializations: usize,
    profile_data: Option<ProfileData>,
}

impl Optimizer {
    /// Create a new optimizer for the requested level.
    pub fn new(level: OptimizationLevel) -> Self {
        Self {
            level,
            max_specializations: 16,
            profile_data: None,
        }
    }

    /// Override the specialization cap placeholder used by later phases.
    pub fn with_max_specializations(level: OptimizationLevel, max_specializations: usize) -> Self {
        Self {
            level,
            max_specializations,
            profile_data: None,
        }
    }

    /// Return the configured specialization cap.
    pub fn max_specializations(&self) -> usize {
        self.max_specializations
    }

    /// Return the normalized profile data used by the optimizer, if any.
    pub fn profile_data(&self) -> Option<&ProfileData> {
        self.profile_data.as_ref()
    }

    /// Return the current deterministic optimization report.
    pub fn optimization_report(&self) -> OptimizationReport {
        let hot_function_keys = self.profile_data.as_ref().map_or_else(Vec::new, |profile| {
            profile.hot_function_keys(HOT_FUNCTION_MINIMUM_WEIGHT)
        });

        OptimizationReport {
            level: self.level,
            max_specializations: self.max_specializations,
            profile_data_present: self.profile_data.is_some(),
            profile_data_used_for_inlining: !hot_function_keys.is_empty(),
            hot_function_keys,
        }
    }

    /// Attach deterministic profile data to an optimizer.
    pub fn with_profile_data(mut self, profile_data: ProfileData) -> Self {
        self.profile_data = Some(profile_data.normalized());
        self
    }

    /// Optimize a program in place.
    pub fn optimize_program(&self, program: &mut LirProgram) {
        self.optimize_program_internal(program, true);
    }

    /// Optimize a program and return a deterministic optimization report.
    pub fn optimize_program_with_report(&self, program: &mut LirProgram) -> OptimizationReport {
        self.optimize_program(program);
        self.optimization_report()
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

    /// Optimize a program using MIR layout metadata and return a deterministic optimization report.
    pub fn optimize_program_with_mir_and_report(
        &self,
        program: &mut LirProgram,
        mir: &MirAnalysisProgram,
    ) -> OptimizationReport {
        self.optimize_program_with_mir(program, mir);
        self.optimization_report()
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
                let mut specialized_functions = BTreeMap::new();
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
                    &mut specialized_functions,
                );

                if matches!(self.level, OptimizationLevel::ReleaseAdvanced) {
                    self.prune_dead_top_level_functions(program);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn optimize_node(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        plan: &SpecializationPlan,
        tracker: &mut SpecializationTracker,
        owner: String,
        allow_generic_specialization: bool,
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

        for child in snapshot.children.iter().copied() {
            self.optimize_node(
                program,
                child,
                plan,
                tracker,
                next_owner.clone(),
                allow_generic_specialization,
                specialized_functions,
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
            specialized_functions,
        ) {
            self.optimize_node(
                program,
                id,
                plan,
                tracker,
                owner,
                allow_generic_specialization,
                specialized_functions,
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

    #[allow(clippy::too_many_arguments)]
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

    fn array_literal_length(&self, program: &LirProgram, id: LirNodeId) -> Option<usize> {
        if !self.is_array_literal(program, id) {
            return None;
        }

        let node = program.nodes.get(id.0 as usize)?;
        Some(node.children.len())
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
        if node.kind != LirNodeKind::Value || node.text.is_some() {
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
                    }
                    Some(ConstantValue::Boolean(true)) => {
                        program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                    }
                    _ => {}
                }

                match literal_value(program, *right) {
                    Some(ConstantValue::Boolean(false)) => {
                        program.nodes[id.0 as usize] =
                            LirNode::with_text(LirNodeKind::Literal, "false");
                        true
                    }
                    Some(ConstantValue::Boolean(true)) => {
                        program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                        true
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
                    }
                    Some(ConstantValue::Boolean(false)) => {
                        program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                    }
                    _ => {}
                }

                match literal_value(program, *right) {
                    Some(ConstantValue::Boolean(true)) => {
                        program.nodes[id.0 as usize] =
                            LirNode::with_text(LirNodeKind::Literal, "true");
                        true
                    }
                    Some(ConstantValue::Boolean(false)) => {
                        program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn optimize_call_site(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        plan: &SpecializationPlan,
        tracker: &mut SpecializationTracker,
        owner: &str,
        allow_generic_specialization: bool,
        specialized_functions: &mut BTreeMap<String, LirNodeId>,
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
        );

        if let Some(callee) = program.nodes.get_mut(callee_id.0 as usize) {
            callee.text = Some(specialized_name);
        }

        true
    }

    #[allow(clippy::too_many_arguments)]
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
                true,
                specialized_functions,
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

    fn inline_threshold_for_function(&self, callee_name: &str) -> usize {
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

    fn is_hot_function(&self, callee_name: &str) -> bool {
        self.profile_data
            .as_ref()
            .and_then(|profile| profile.sample_weight(ProfileSampleKind::Function, callee_name))
            .is_some_and(|weight| weight >= HOT_FUNCTION_MINIMUM_WEIGHT)
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
    RegExp { pattern: String, flags: String },
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
            ConstantValue::RegExp { .. } => true,
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
        _ => parse_regex_literal(text)
            .map(|(pattern, flags)| ConstantValue::RegExp { pattern, flags })
            .or_else(|| parse_string_literal(text).map(ConstantValue::String))
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
        ConstantValue::RegExp { pattern, flags } => format!("/{pattern}/{flags}"),
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

fn parse_regex_literal(text: &str) -> Option<(String, String)> {
    if !text.starts_with('/') {
        return None;
    }

    let mut escaped = false;
    let mut closing = None;
    for (idx, ch) in text.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '/' => {
                closing = Some(idx);
                break;
            }
            _ => {}
        }
    }

    let closing = closing?;
    if closing == 0 || closing + 1 > text.len() {
        return None;
    }

    let pattern = text[1..closing].to_string();
    let flags = text[closing + 1..].to_string();
    if !flags.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }

    Some((pattern, flags))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
