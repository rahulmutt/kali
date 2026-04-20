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
            node.kind == LirNodeKind::Instruction && node.text.as_deref() == Some(specialized_name)
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
    builder.node_mut(callee_expr1).unwrap().children = vec![callee_param_point, callee_param_value];
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
            node.kind == LirNodeKind::Instruction && node.text.as_deref() == Some(specialized_name)
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
            node.kind == LirNodeKind::Instruction && node.text.as_deref() == Some(specialized_name)
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
            node.kind == LirNodeKind::Instruction && node.text.as_deref() == Some(specialized_name)
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
fn release_specializes_literal_shaped_mir_call_sites_without_layout_metadata() {
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

    let mir = MirAnalysisProgram {
        root: kali_mir::MirNodeId::new(0),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("merge_pair".to_string()),
            kind: kali_mir::MirFunctionKind::Function,
            bindings: Vec::new(),
        }],
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

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
            node.kind == LirNodeKind::Instruction && node.text.as_deref() == Some(specialized_name)
        })
        .expect("specialized function should be inserted without MIR layouts");
    assert_eq!(specialized_function.kind, LirNodeKind::Instruction);
    let literal_thirty_three = program
        .nodes
        .iter()
        .any(|node| node.kind == LirNodeKind::Literal && node.text.as_deref() == Some("33"));
    assert!(
        literal_thirty_three,
        "literal-shaped MIR specialization should still expose the folded literal result"
    );
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
fn release_specializes_identical_generic_call_sites_across_owners_once() {
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

    let caller_one = builder.alloc_text(LirNodeKind::Instruction, "caller_one");
    let caller_one_block = builder.alloc(LirNodeKind::Block);
    let caller_one_call = builder.alloc(LirNodeKind::Call);
    let caller_one_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let caller_one_left = literal(&mut builder, "2");
    let caller_one_right = literal(&mut builder, "3");
    builder.node_mut(caller_one_call).unwrap().children =
        vec![caller_one_callee, caller_one_left, caller_one_right];
    builder.node_mut(caller_one_block).unwrap().children = vec![caller_one_call];
    builder.node_mut(caller_one).unwrap().children = vec![caller_one_block];

    let caller_two = builder.alloc_text(LirNodeKind::Instruction, "caller_two");
    let caller_two_block = builder.alloc(LirNodeKind::Block);
    let caller_two_call = builder.alloc(LirNodeKind::Call);
    let caller_two_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let caller_two_left = literal(&mut builder, "2");
    let caller_two_right = literal(&mut builder, "3");
    builder.node_mut(caller_two_call).unwrap().children =
        vec![caller_two_callee, caller_two_left, caller_two_right];
    builder.node_mut(caller_two_block).unwrap().children = vec![caller_two_call];
    builder.node_mut(caller_two).unwrap().children = vec![caller_two_block];

    builder.node_mut(root).unwrap().children = vec![function, caller_one, caller_two];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let specialized_name_one = program.nodes[caller_one_call.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized call target should exist for the first caller");
    let specialized_name_two = program.nodes[caller_two_call.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized call target should exist for the second caller");

    assert_eq!(specialized_name_one, specialized_name_two);
    assert!(specialized_name_one.starts_with("merge_pair$spec$"));

    let specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node.text.as_deref() == Some(specialized_name_one)
        })
        .count();
    assert_eq!(
        specialized_count, 1,
        "identical generic specializations should be reused across owners"
    );
}

#[test]
fn release_reuses_existing_mir_specializations_after_an_owner_spends_its_budget() {
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

    let caller_one = builder.alloc_text(LirNodeKind::Instruction, "caller_one");
    let caller_one_block = builder.alloc(LirNodeKind::Block);
    let caller_one_call = builder.alloc(LirNodeKind::Call);
    let caller_one_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let caller_one_left = literal(&mut builder, "2");
    let caller_one_right = literal(&mut builder, "3");
    builder.node_mut(caller_one_call).unwrap().children =
        vec![caller_one_callee, caller_one_left, caller_one_right];
    builder.node_mut(caller_one_block).unwrap().children = vec![caller_one_call];
    builder.node_mut(caller_one).unwrap().children = vec![caller_one_block];

    let caller_two = builder.alloc_text(LirNodeKind::Instruction, "caller_two");
    let caller_two_block = builder.alloc(LirNodeKind::Block);
    let caller_two_unique_a_call = builder.alloc(LirNodeKind::Call);
    let caller_two_unique_a_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let caller_two_unique_a_left = literal(&mut builder, "4");
    let caller_two_unique_a_right = literal(&mut builder, "5");
    builder.node_mut(caller_two_unique_a_call).unwrap().children = vec![
        caller_two_unique_a_callee,
        caller_two_unique_a_left,
        caller_two_unique_a_right,
    ];
    let caller_two_unique_b_call = builder.alloc(LirNodeKind::Call);
    let caller_two_unique_b_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let caller_two_unique_b_left = literal(&mut builder, "6");
    let caller_two_unique_b_right = literal(&mut builder, "7");
    builder.node_mut(caller_two_unique_b_call).unwrap().children = vec![
        caller_two_unique_b_callee,
        caller_two_unique_b_left,
        caller_two_unique_b_right,
    ];
    let caller_two_shared_call = builder.alloc(LirNodeKind::Call);
    let caller_two_shared_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let caller_two_shared_left = literal(&mut builder, "2");
    let caller_two_shared_right = literal(&mut builder, "3");
    builder.node_mut(caller_two_shared_call).unwrap().children = vec![
        caller_two_shared_callee,
        caller_two_shared_left,
        caller_two_shared_right,
    ];
    builder.node_mut(caller_two_block).unwrap().children = vec![
        caller_two_unique_a_call,
        caller_two_unique_b_call,
        caller_two_shared_call,
    ];
    builder.node_mut(caller_two).unwrap().children = vec![caller_two_block];

    builder.node_mut(root).unwrap().children = vec![function, caller_one, caller_two];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    let mir = MirAnalysisProgram {
        root: kali_mir::MirNodeId::new(0),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("merge_pair".to_string()),
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

    Optimizer::with_max_specializations(OptimizationLevel::Release, 2)
        .optimize_program_with_mir(&mut program, &mir);

    let caller_one_specialized_name = program.nodes[caller_one_call.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("first caller should specialize under the MIR plan");
    let caller_two_unique_a_name = program.nodes[caller_two_unique_a_call.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("second caller's first unique specialization should be created");
    let caller_two_unique_b_name = program.nodes[caller_two_unique_b_call.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("second caller's second unique specialization should be created when the budget still allows it");
    let caller_two_shared_name = program.nodes[caller_two_shared_call.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect(
            "second caller should reuse the first owner's specialization after its budget is spent",
        );

    assert!(caller_one_specialized_name.starts_with("merge_pair$spec$"));
    assert!(caller_two_unique_a_name.starts_with("merge_pair$spec$"));
    assert_eq!(caller_one_specialized_name, caller_two_shared_name);
    assert_ne!(caller_one_specialized_name, caller_two_unique_a_name);
    assert!(
        caller_two_unique_b_name == "merge_pair"
            || caller_two_unique_b_name.starts_with("merge_pair$spec$")
    );

    let specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|name| name.starts_with("merge_pair$spec$"))
        })
        .count();
    assert!(specialized_count >= 2);
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
fn release_specializes_regex_literal_arguments() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let function = builder.alloc_text(LirNodeKind::Instruction, "consume_pattern");
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

    let call_regex_a = builder.alloc(LirNodeKind::Call);
    let callee_regex_a = builder.alloc_text(LirNodeKind::Value, "consume_pattern");
    let regex_a = literal(&mut builder, "/foo/i");
    builder.node_mut(call_regex_a).unwrap().children = vec![callee_regex_a, regex_a];

    let call_regex_b = builder.alloc(LirNodeKind::Call);
    let callee_regex_b = builder.alloc_text(LirNodeKind::Value, "consume_pattern");
    let regex_b = literal(&mut builder, "/bar/i");
    builder.node_mut(call_regex_b).unwrap().children = vec![callee_regex_b, regex_b];

    let call_regex_a_repeat = builder.alloc(LirNodeKind::Call);
    let callee_regex_a_repeat = builder.alloc_text(LirNodeKind::Value, "consume_pattern");
    let regex_a_repeat = literal(&mut builder, "/foo/i");
    builder.node_mut(call_regex_a_repeat).unwrap().children =
        vec![callee_regex_a_repeat, regex_a_repeat];

    builder.node_mut(root).unwrap().children =
        vec![function, call_regex_a, call_regex_b, call_regex_a_repeat];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    let mir = MirAnalysisProgram {
        root: kali_mir::MirNodeId::new(0),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("consume_pattern".to_string()),
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

    let specialized_name_a = program.nodes[call_regex_a.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized call target should exist for regex literal A");
    let specialized_name_b = program.nodes[call_regex_b.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized call target should exist for regex literal B");
    let specialized_name_a_repeat = program.nodes[call_regex_a_repeat.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized call target should exist for repeated regex literal A");

    assert_eq!(specialized_name_a, specialized_name_a_repeat);
    assert_ne!(specialized_name_a, specialized_name_b);
    assert!(specialized_name_a.starts_with("consume_pattern$spec$"));
    assert!(specialized_name_b.starts_with("consume_pattern$spec$"));
}

#[test]
fn release_specializes_regex_literal_arguments_with_mir_layouts() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let function = builder.alloc_text(LirNodeKind::Instruction, "consume_pattern");
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

    let call_regex_a = builder.alloc(LirNodeKind::Call);
    let callee_regex_a = builder.alloc_text(LirNodeKind::Value, "consume_pattern");
    let regex_a = literal(&mut builder, "/foo/i");
    builder.node_mut(call_regex_a).unwrap().children = vec![callee_regex_a, regex_a];

    let call_regex_b = builder.alloc(LirNodeKind::Call);
    let callee_regex_b = builder.alloc_text(LirNodeKind::Value, "consume_pattern");
    let regex_b = literal(&mut builder, "/bar/i");
    builder.node_mut(call_regex_b).unwrap().children = vec![callee_regex_b, regex_b];

    let call_regex_a_repeat = builder.alloc(LirNodeKind::Call);
    let callee_regex_a_repeat = builder.alloc_text(LirNodeKind::Value, "consume_pattern");
    let regex_a_repeat = literal(&mut builder, "/foo/i");
    builder.node_mut(call_regex_a_repeat).unwrap().children =
        vec![callee_regex_a_repeat, regex_a_repeat];

    builder.node_mut(root).unwrap().children =
        vec![function, call_regex_a, call_regex_b, call_regex_a_repeat];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    let mir = MirAnalysisProgram {
        root: kali_mir::MirNodeId::new(0),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("consume_pattern".to_string()),
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

    let specialized_name_a = program.nodes[call_regex_a.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized call target should exist for regex literal A");
    let specialized_name_b = program.nodes[call_regex_b.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized call target should exist for regex literal B");
    let specialized_name_a_repeat = program.nodes[call_regex_a_repeat.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized call target should exist for repeated regex literal A");

    assert_eq!(specialized_name_a, specialized_name_a_repeat);
    assert_ne!(specialized_name_a, specialized_name_b);
    assert!(specialized_name_a.starts_with("consume_pattern$spec$"));
    assert!(specialized_name_b.starts_with("consume_pattern$spec$"));
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
    let callee_negative_infinity = builder.alloc_text(LirNodeKind::Value, "consume_special_number");
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

    builder.node_mut(root).unwrap().children = vec![function, call_true_a, call_false, call_true_b];
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
    builder.node_mut(call_neg_zero).unwrap().children = vec![call_neg_zero_callee, arg_neg_zero];

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
