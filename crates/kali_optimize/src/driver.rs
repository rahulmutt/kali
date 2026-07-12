use crate::*;

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
    /// Whether profile data contributed any hot-branch hints.
    pub profile_data_used_for_branching: bool,
    /// Whether profile data contributed any hot-layout hints.
    pub profile_data_used_for_layout_specialization: bool,
    /// Hot function keys discovered in the attached profile data.
    pub hot_function_keys: Vec<String>,
    /// Hot branch keys discovered in the attached profile data.
    pub hot_branch_keys: Vec<String>,
    /// Hot layout keys discovered in the attached profile data.
    pub hot_layout_keys: Vec<String>,
}

/// Optimizer context.
#[derive(Clone, Debug)]
pub struct Optimizer {
    pub(crate) level: OptimizationLevel,
    pub(crate) max_specializations: usize,
    pub(crate) profile_data: Option<ProfileData>,
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
        let hot_branch_keys = self.profile_data.as_ref().map_or_else(Vec::new, |profile| {
            profile.hot_keys(ProfileSampleKind::Branch, HOT_FUNCTION_MINIMUM_WEIGHT)
        });
        let hot_layout_keys = self.profile_data.as_ref().map_or_else(Vec::new, |profile| {
            profile.hot_keys(ProfileSampleKind::Layout, HOT_FUNCTION_MINIMUM_WEIGHT)
        });

        OptimizationReport {
            level: self.level,
            max_specializations: self.max_specializations,
            profile_data_present: self.profile_data.is_some(),
            profile_data_used_for_inlining: !hot_function_keys.is_empty(),
            profile_data_used_for_branching: !hot_branch_keys.is_empty(),
            profile_data_used_for_layout_specialization: !hot_layout_keys.is_empty(),
            hot_function_keys,
            hot_branch_keys,
            hot_layout_keys,
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

    pub(crate) fn optimize_program_internal(
        &self,
        program: &mut LirProgram,
        allow_generic_specialization: bool,
    ) {
        self.fold_object_enumeration_calls_ordered(program);

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

                let mut constant_bindings = self.collect_constant_bindings(program, program.root);
                let mutated = self.collect_mutated_binding_names(program);
                constant_bindings
                    .bindings
                    .retain(|name, _| !mutated.contains(name));
                self.fold_object_enumeration_calls_ordered(program);

                self.optimize_node(
                    program,
                    program.root,
                    &plan,
                    &mut tracker,
                    "<root>".to_string(),
                    allow_generic_specialization,
                    &mut specialized_functions,
                    &constant_bindings,
                );

                if matches!(self.level, OptimizationLevel::ReleaseAdvanced) {
                    self.prune_dead_top_level_functions(program);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn optimize_node(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        plan: &SpecializationPlan,
        tracker: &mut SpecializationTracker,
        owner: String,
        allow_generic_specialization: bool,
        specialized_functions: &mut BTreeMap<String, LirNodeId>,
        bindings: &BindingEnv,
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
                bindings,
            );
        }

        if matches!(snapshot.kind, LirNodeKind::Program | LirNodeKind::Block) {
            self.optimize_sequence(program, id);
        }

        if self.optimize_constant_expression(program, id, tracker, &owner) {
            return;
        }

        if (matches!(self.level, OptimizationLevel::ReleaseAdvanced)
            || (matches!(self.level, OptimizationLevel::Release)
                && self.profile_has_hot_branch_or_layout_hints()))
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
            bindings,
        ) {
            self.optimize_node(
                program,
                id,
                plan,
                tracker,
                owner,
                allow_generic_specialization,
                specialized_functions,
                bindings,
            );
        }
    }

    pub(crate) fn optimize_sequence(&self, program: &mut LirProgram, id: LirNodeId) {
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

                let mut canonicalized = Vec::with_capacity(flattened.len());
                let mut seen = BTreeMap::new();
                for child in flattened {
                    if self.is_cse_candidate(program, child) {
                        let signature = node_signature(program, child);
                        if let Some(existing) = seen.get(&signature).copied() {
                            canonicalized.push(existing);
                            continue;
                        }
                        seen.insert(signature, child);
                    }
                    canonicalized.push(child);
                }

                program.nodes[id.0 as usize].children = canonicalized;
            }
            _ => {}
        }
    }

    pub(crate) fn is_cse_candidate(&self, program: &LirProgram, id: LirNodeId) -> bool {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return false;
        };

        match node.kind {
            LirNodeKind::Literal => true,
            LirNodeKind::Value => node.text.is_some(),
            _ => false,
        }
    }
}

#[cfg(test)]
#[path = "driver_tests.rs"]
mod driver_tests;
