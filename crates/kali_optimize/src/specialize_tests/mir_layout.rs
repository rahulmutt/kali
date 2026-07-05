use super::*;

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
        arena_facts: Vec::new(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("sum_many".to_string()),
            kind: kali_mir::MirFunctionKind::Function,
            function_flavor: None,
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
        arena_facts: Vec::new(),
        nodes: Vec::new(),
        functions: vec![
            kali_mir::MirFunction {
                name: Some("sum_pair".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
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
                function_flavor: None,
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
        arena_facts: Vec::new(),
        nodes: Vec::new(),
        functions: vec![
            kali_mir::MirFunction {
                name: Some("consume_point".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
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
                function_flavor: None,
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
                function_flavor: None,
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
        arena_facts: Vec::new(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("merge_pair".to_string()),
            kind: kali_mir::MirFunctionKind::Function,
            function_flavor: None,
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
