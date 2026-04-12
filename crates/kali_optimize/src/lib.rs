//! Optimization passes for the Kali compiler.
//!
//! The current implementation focuses on the deterministic, tree-shaped LIR
//! that the rest of the repository already produces. That gives us a safe place
//! to land constant folding, branch elimination, and a handful of algebraic
//! simplifications without needing a full SSA pipeline yet.

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
            OptimizationLevel::Release => {
                self.optimize_node(program, program.root, true);
            }
            OptimizationLevel::ReleaseAdvanced => {
                self.optimize_node(program, program.root, true);
            }
        }
    }

    fn optimize_node(&self, program: &mut LirProgram, id: LirNodeId, is_root: bool) {
        let children = program.nodes[id.0 as usize].children.clone();
        for child in children {
            self.optimize_node(program, child, false);
        }

        if is_root {
            self.optimize_sequence(program, id);
            return;
        }

        if self.optimize_constant_expression(program, id) {
            return;
        }

        if matches!(self.level, OptimizationLevel::ReleaseAdvanced)
            && self.optimize_algebraic_identity(program, id)
        {
            return;
        }

        if matches!(self.level, OptimizationLevel::ReleaseAdvanced) {
            self.optimize_sequence(program, id);
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

    fn optimize_constant_expression(&self, program: &mut LirProgram, id: LirNodeId) -> bool {
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
                    program.nodes[id.0 as usize] =
                        LirNode::with_text(LirNodeKind::Literal, if truthy { "1" } else { "0" });
                    return true;
                };

                program.nodes[id.0 as usize] = program.nodes[chosen.0 as usize].clone();
                true
            }
            _ => false,
        }
    }

    fn optimize_algebraic_identity(&self, program: &mut LirProgram, id: LirNodeId) -> bool {
        let snapshot = program.nodes[id.0 as usize].clone();
        let Some(op) = snapshot.text.as_deref() else {
            return false;
        };

        match (op, snapshot.children.as_slice()) {
            ("+", [left, right]) => {
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
                if literal_value(program, *right) == Some(ConstantValue::Number(0)) {
                    program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                    return true;
                }
                false
            }
            ("*", [left, right]) => {
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConstantValue {
    Number(i64),
    Boolean(bool),
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
