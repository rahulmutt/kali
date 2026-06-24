use crate::test_support::*;
use crate::*;
use kali_lir::{LirBuilder, LirNodeKind};

#[test]
fn fast_keeps_binary_expressions_opaque() {
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

    Optimizer::new(OptimizationLevel::Fast).optimize_program(&mut program);

    let node = &program.nodes[add.0 as usize];
    assert_eq!(node.kind, LirNodeKind::Value);
    assert_eq!(node.text.as_deref(), Some("+"));
    assert_eq!(node.children.len(), 2);
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
fn release_constant_folds_remainder_expressions() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let rem = builder.alloc_text(LirNodeKind::Value, "%");
    let lhs = literal(&mut builder, "7");
    let rhs = literal(&mut builder, "4");
    builder.node_mut(rem).unwrap().children = vec![lhs, rhs];
    builder.node_mut(root).unwrap().children = vec![rem];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let node = &program.nodes[rem.0 as usize];
    assert_eq!(node.kind, LirNodeKind::Literal);
    assert_eq!(node.text.as_deref(), Some("3"));
}

#[test]
fn release_constant_folds_string_concatenation() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let concat = builder.alloc_text(LirNodeKind::Value, "+");
    let lhs = literal(&mut builder, "\"hello \"");
    let rhs = literal(&mut builder, "\"world\"");
    builder.node_mut(concat).unwrap().children = vec![lhs, rhs];
    builder.node_mut(root).unwrap().children = vec![concat];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let node = &program.nodes[concat.0 as usize];
    assert_eq!(node.kind, LirNodeKind::Literal);
    assert_eq!(node.text.as_deref(), Some("\"hello world\""));
}

#[test]
fn release_constant_folds_bigint_addition_chain() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let add1 = builder.alloc_text(LirNodeKind::Value, "+");
    let add2 = builder.alloc_text(LirNodeKind::Value, "+");
    let add3 = builder.alloc_text(LirNodeKind::Value, "+");
    let add4 = builder.alloc_text(LirNodeKind::Value, "+");
    let one = literal(&mut builder, "1n");
    let two = literal(&mut builder, "2n");
    let three = literal(&mut builder, "3n");
    let four = literal(&mut builder, "4n");
    let five = literal(&mut builder, "5n");
    let six = literal(&mut builder, "6n");
    let seven = literal(&mut builder, "7n");
    let eight = literal(&mut builder, "8n");
    builder.node_mut(add1).unwrap().children = vec![one, two];
    builder.node_mut(add2).unwrap().children = vec![add1, three];
    builder.node_mut(add3).unwrap().children = vec![add2, four];
    builder.node_mut(add4).unwrap().children = vec![add3, five];
    let add5 = builder.alloc_text(LirNodeKind::Value, "+");
    builder.node_mut(add5).unwrap().children = vec![add4, six];
    let add6 = builder.alloc_text(LirNodeKind::Value, "+");
    builder.node_mut(add6).unwrap().children = vec![add5, seven];
    let add7 = builder.alloc_text(LirNodeKind::Value, "+");
    builder.node_mut(add7).unwrap().children = vec![add6, eight];
    builder.node_mut(root).unwrap().children = vec![add7];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let node = &program.nodes[add7.0 as usize];
    assert_eq!(node.kind, LirNodeKind::Literal);
    assert_eq!(node.text.as_deref(), Some("36n"));
}

#[test]
fn release_constant_folds_bigint_multiplication_chain() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let add1 = builder.alloc_text(LirNodeKind::Value, "+");
    let add2 = builder.alloc_text(LirNodeKind::Value, "+");
    let add3 = builder.alloc_text(LirNodeKind::Value, "+");
    let add4 = builder.alloc_text(LirNodeKind::Value, "+");
    let mul = builder.alloc_text(LirNodeKind::Value, "*");
    let one = literal(&mut builder, "1n");
    let two = literal(&mut builder, "2n");
    let three = literal(&mut builder, "3n");
    let four = literal(&mut builder, "4n");
    let five = literal(&mut builder, "5n");
    let six = literal(&mut builder, "6n");
    let seven = literal(&mut builder, "7n");
    let eight = literal(&mut builder, "8n");
    let identity = literal(&mut builder, "1n");
    builder.node_mut(add1).unwrap().children = vec![one, two];
    builder.node_mut(add2).unwrap().children = vec![add1, three];
    builder.node_mut(add3).unwrap().children = vec![add2, four];
    builder.node_mut(add4).unwrap().children = vec![add3, five];
    builder.node_mut(mul).unwrap().children = vec![add4, identity];
    let add5 = builder.alloc_text(LirNodeKind::Value, "+");
    builder.node_mut(add5).unwrap().children = vec![mul, six];
    let add6 = builder.alloc_text(LirNodeKind::Value, "+");
    builder.node_mut(add6).unwrap().children = vec![add5, seven];
    let add7 = builder.alloc_text(LirNodeKind::Value, "+");
    builder.node_mut(add7).unwrap().children = vec![add6, eight];
    builder.node_mut(root).unwrap().children = vec![add7];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let node = &program.nodes[add7.0 as usize];
    assert_eq!(node.kind, LirNodeKind::Literal);
    assert_eq!(node.text.as_deref(), Some("36n"));
}

#[test]
fn release_constant_folds_bigint_remainder_expression() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let rem = builder.alloc_text(LirNodeKind::Value, "%");
    let lhs = literal(&mut builder, "7n");
    let rhs = literal(&mut builder, "4n");
    builder.node_mut(rem).unwrap().children = vec![lhs, rhs];
    builder.node_mut(root).unwrap().children = vec![rem];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let node = &program.nodes[rem.0 as usize];
    assert_eq!(node.kind, LirNodeKind::Literal);
    assert_eq!(node.text.as_deref(), Some("3n"));
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
fn release_advanced_eliminates_division_by_one() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let div = builder.alloc_text(LirNodeKind::Value, "/");
    let ident = builder.alloc_text(LirNodeKind::Value, "x");
    let one = literal(&mut builder, "1");
    builder.node_mut(div).unwrap().children = vec![ident, one];
    builder.node_mut(root).unwrap().children = vec![div];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let node = &program.nodes[div.0 as usize];
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

#[test]
fn release_eliminates_duplicate_pure_expressions_within_basic_blocks() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let left = builder.alloc_text(LirNodeKind::Value, "+");
    let right = builder.alloc_text(LirNodeKind::Value, "+");
    let left_lhs = builder.alloc_text(LirNodeKind::Value, "x");
    let left_rhs = builder.alloc_text(LirNodeKind::Value, "y");
    let right_lhs = builder.alloc_text(LirNodeKind::Value, "x");
    let right_rhs = builder.alloc_text(LirNodeKind::Value, "y");
    builder.node_mut(left).unwrap().children = vec![left_lhs, left_rhs];
    builder.node_mut(right).unwrap().children = vec![right_lhs, right_rhs];
    builder.node_mut(root).unwrap().children = vec![left, right];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let root_children = &program.nodes[root.0 as usize].children;
    assert_eq!(root_children.len(), 2);
    assert_eq!(root_children[0], root_children[1]);
    let canonical = &program.nodes[root_children[0].0 as usize];
    assert_eq!(canonical.kind, LirNodeKind::Value);
    assert_eq!(canonical.text.as_deref(), Some("+"));
}

#[test]
fn release_eliminates_duplicate_literals_within_basic_blocks() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let first = literal(&mut builder, "42");
    let second = literal(&mut builder, "42");
    builder.node_mut(root).unwrap().children = vec![first, second];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let root_children = &program.nodes[root.0 as usize].children;
    assert_eq!(root_children.len(), 2);
    assert_eq!(root_children[0], root_children[1]);
    let canonical = &program.nodes[root_children[0].0 as usize];
    assert_eq!(canonical.kind, LirNodeKind::Literal);
    assert_eq!(canonical.text.as_deref(), Some("42"));
}
