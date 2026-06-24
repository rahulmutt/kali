use crate::*;

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
