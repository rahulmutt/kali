use super::*;

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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![
            kali_mir::MirFunction {
                name: None,
                kind: kali_mir::MirFunctionKind::Module,
                function_flavor: None,
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
                function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![
            kali_mir::MirFunction {
                name: None,
                kind: kali_mir::MirFunctionKind::Module,
                function_flavor: None,
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
                function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![
            kali_mir::MirFunction {
                name: None,
                kind: kali_mir::MirFunctionKind::Module,
                function_flavor: None,
                bindings: Vec::new(),
            },
            kali_mir::MirFunction {
                name: Some("consume_point".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
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
                function_flavor: None,
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
                function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![
            kali_mir::MirFunction {
                name: None,
                kind: kali_mir::MirFunctionKind::Module,
                function_flavor: None,
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
                function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![
            kali_mir::MirFunction {
                name: None,
                kind: kali_mir::MirFunctionKind::Module,
                function_flavor: None,
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
                function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![
            kali_mir::MirFunction {
                name: None,
                kind: kali_mir::MirFunctionKind::Module,
                function_flavor: None,
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
                function_flavor: None,
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
