use super::*;

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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("add_pair".to_string()),
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
fn release_respects_zero_specialization_budget_for_tagged_parameters() {
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("add_pair".to_string()),
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
        }],
    };

    Optimizer::with_max_specializations(OptimizationLevel::Release, 0)
        .optimize_program_with_mir(&mut program, &mir);

    let call_node = &program.nodes[call.0 as usize];
    let callee_name = call_node
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("call target should remain the original function when the specialization budget is zero");
    assert_eq!(callee_name, "add_pair");
    assert!(
        !program.nodes.iter().any(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|text| text.starts_with("add_pair$spec$"))
        }),
        "zero specialization budget should not clone tagged-parameter call sites"
    );
    assert!(
        !program
            .nodes
            .iter()
            .any(|node| node.kind == LirNodeKind::Literal && node.text.as_deref() == Some("33")),
        "zero specialization budget should not create a specialized folded literal"
    );
}

#[test]
fn release_advanced_limits_specialization_to_one_distinct_call_site_after_root_inlining() {
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

    let caller = builder.alloc_text(LirNodeKind::Instruction, "caller");
    let caller_block = builder.alloc(LirNodeKind::Block);

    let call_a = builder.alloc(LirNodeKind::Call);
    let call_a_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let call_a_left = literal(&mut builder, "2");
    let call_a_right = literal(&mut builder, "3");
    builder.node_mut(call_a).unwrap().children = vec![call_a_callee, call_a_left, call_a_right];

    let call_b = builder.alloc(LirNodeKind::Call);
    let call_b_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let call_b_left = literal(&mut builder, "2");
    let call_b_right = literal(&mut builder, "3");
    builder.node_mut(call_b).unwrap().children = vec![call_b_callee, call_b_left, call_b_right];

    let call_c = builder.alloc(LirNodeKind::Call);
    let call_c_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let call_c_left = literal(&mut builder, "4");
    let call_c_right = literal(&mut builder, "5");
    builder.node_mut(call_c).unwrap().children = vec![call_c_callee, call_c_left, call_c_right];

    builder.node_mut(caller_block).unwrap().children = vec![call_a, call_b, call_c];
    builder.node_mut(caller).unwrap().children = vec![caller_block];
    builder.node_mut(root).unwrap().children = vec![function, caller];
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
                bindings: Vec::new(),
            },
            kali_mir::MirFunction {
                name: Some("merge_pair".to_string()),
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

    Optimizer::with_max_specializations(OptimizationLevel::ReleaseAdvanced, 1)
        .optimize_program_with_mir(&mut program, &mir);

    let specialized_name_a = program.nodes[call_a.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("first call should specialize");
    let specialized_name_b = program.nodes[call_b.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("duplicate call should reuse the existing specialization");
    let callee_name_c = program.nodes[call_c.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("third call should keep the original callee when the budget is exhausted");

    assert_eq!(specialized_name_a, "+");
    assert_eq!(specialized_name_b, "+");
    assert!(callee_name_c.starts_with("merge_pair$spec$"));

    let specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|text| text.starts_with("merge_pair$spec$"))
        })
        .count();
    assert_eq!(specialized_count, 1);
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("sum_chain".to_string()),
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
