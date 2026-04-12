//! Optimization passes for the Kali compiler.
//!
//! The current implementation focuses on the deterministic, tree-shaped LIR
//! that the rest of the repository already produces. That gives us a safe place
//! to land constant folding, branch elimination, and a handful of algebraic
//! simplifications without needing a full SSA pipeline yet.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use kali_lir::{LirNode, LirNodeId, LirNodeKind, LirProgram};

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
                let plan = self.build_specialization_plan(program);
                let mut tracker = SpecializationTracker::new(self.max_specializations);
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

#[derive(Clone, Debug, Default)]
struct SpecializationPlan {
    functions: BTreeMap<String, FunctionSummary>,
}

#[derive(Clone, Debug)]
struct FunctionSummary {
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
}
