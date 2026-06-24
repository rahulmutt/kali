use crate::*;

impl Optimizer {
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

    pub(crate) fn function_summary(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> Option<FunctionSummary> {
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

    pub(crate) fn extract_inline_body(
        &self,
        program: &LirProgram,
        block_id: LirNodeId,
    ) -> Option<LirNodeId> {
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

    pub(crate) fn contains_call_target(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        target: &str,
    ) -> bool {
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

#[cfg(test)]
#[path = "inline_tests.rs"]
mod inline_tests;
