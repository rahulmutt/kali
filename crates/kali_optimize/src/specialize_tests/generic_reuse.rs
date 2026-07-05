use super::*;

#[test]
fn release_allows_generic_specialization_inside_mir_specialized_clones() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let inner = builder.alloc_text(LirNodeKind::Instruction, "merge_pair");
    let inner_param_left = builder.alloc_text(LirNodeKind::Value, "left");
    let inner_param_right = builder.alloc_text(LirNodeKind::Value, "right");
    let inner_block = builder.alloc(LirNodeKind::Block);
    let inner_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
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
    let _eight = literal(&mut builder, "8");
    builder.node_mut(add1).unwrap().children = vec![inner_param_left, inner_param_right];
    builder.node_mut(add2).unwrap().children = vec![add1, one];
    builder.node_mut(add3).unwrap().children = vec![add2, two];
    builder.node_mut(add4).unwrap().children = vec![add3, three];
    builder.node_mut(add5).unwrap().children = vec![add4, four];
    builder.node_mut(add6).unwrap().children = vec![add5, five];
    builder.node_mut(add7).unwrap().children = vec![add6, six];
    builder.node_mut(add8).unwrap().children = vec![add7, seven];
    builder.node_mut(inner_ret).unwrap().children = vec![add8];
    builder.node_mut(inner_block).unwrap().children = vec![inner_ret];
    builder.node_mut(inner).unwrap().children =
        vec![inner_param_left, inner_param_right, inner_block];

    let outer = builder.alloc_text(LirNodeKind::Instruction, "wrap_sum");
    let outer_param_left = builder.alloc_text(LirNodeKind::Value, "left");
    let outer_param_right = builder.alloc_text(LirNodeKind::Value, "right");
    let outer_block = builder.alloc(LirNodeKind::Block);
    let outer_call = builder.alloc(LirNodeKind::Call);
    let outer_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let outer_add1 = builder.alloc_text(LirNodeKind::Value, "+");
    let outer_add2 = builder.alloc_text(LirNodeKind::Value, "+");
    let outer_add3 = builder.alloc_text(LirNodeKind::Value, "+");
    let outer_add4 = builder.alloc_text(LirNodeKind::Value, "+");
    let outer_add5 = builder.alloc_text(LirNodeKind::Value, "+");
    let outer_add6 = builder.alloc_text(LirNodeKind::Value, "+");
    let outer_add7 = builder.alloc_text(LirNodeKind::Value, "+");
    let outer_add8 = builder.alloc_text(LirNodeKind::Value, "+");
    let oone = literal(&mut builder, "1");
    let otwo = literal(&mut builder, "2");
    let othree = literal(&mut builder, "3");
    let ofour = literal(&mut builder, "4");
    let ofive = literal(&mut builder, "5");
    let osix = literal(&mut builder, "6");
    let oseven = literal(&mut builder, "7");
    let oeight = literal(&mut builder, "8");
    builder.node_mut(outer_call).unwrap().children =
        vec![outer_callee, outer_param_left, outer_param_right];
    builder.node_mut(outer_add1).unwrap().children = vec![outer_call, oone];
    builder.node_mut(outer_add2).unwrap().children = vec![outer_add1, otwo];
    builder.node_mut(outer_add3).unwrap().children = vec![outer_add2, othree];
    builder.node_mut(outer_add4).unwrap().children = vec![outer_add3, ofour];
    builder.node_mut(outer_add5).unwrap().children = vec![outer_add4, ofive];
    builder.node_mut(outer_add6).unwrap().children = vec![outer_add5, osix];
    builder.node_mut(outer_add7).unwrap().children = vec![outer_add6, oseven];
    builder.node_mut(outer_add8).unwrap().children = vec![outer_add7, oeight];
    builder.node_mut(outer_block).unwrap().children = vec![outer_add8];
    builder.node_mut(outer).unwrap().children =
        vec![outer_param_left, outer_param_right, outer_block];

    let root_call = builder.alloc(LirNodeKind::Call);
    let root_callee = builder.alloc_text(LirNodeKind::Value, "wrap_sum");
    let root_left = literal(&mut builder, "2");
    let root_right = literal(&mut builder, "3");
    builder.node_mut(root_call).unwrap().children = vec![root_callee, root_left, root_right];

    builder.node_mut(root).unwrap().children = vec![inner, outer, root_call];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    let mir = MirAnalysisProgram {
        root: kali_mir::MirNodeId::new(0),
        arena_facts: Vec::new(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("wrap_sum".to_string()),
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

    let root_call_node = &program.nodes[root_call.0 as usize];
    let outer_specialized_name = root_call_node
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized wrap_sum call target should exist");
    assert!(outer_specialized_name.starts_with("wrap_sum$spec$"));

    let outer_specialized_function = program
        .nodes
        .iter()
        .find(|node| {
            node.kind == LirNodeKind::Instruction
                && node.text.as_deref() == Some(outer_specialized_name)
        })
        .expect("specialized wrap_sum function should be inserted");
    let outer_body_block_id = outer_specialized_function
        .children
        .last()
        .copied()
        .expect("specialized wrap_sum should still have a body block");
    let outer_body_block = &program.nodes[outer_body_block_id.0 as usize];
    assert!(
        !outer_body_block.children.is_empty(),
        "specialized wrap_sum body should still contain the nested add chain"
    );

    let nested_specialized_count = program
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
    assert_eq!(nested_specialized_count, 1);

    let literal_thirty_three = program
        .nodes
        .iter()
        .any(|node| node.kind == LirNodeKind::Literal && node.text.as_deref() == Some("33"));
    assert!(
        literal_thirty_three,
        "layout-specialized clones should still allow nested generic specialization to fold the inner call"
    );
}

#[test]
fn release_advanced_allows_generic_specialization_inside_mir_specialized_clones() {
    fn append_literal_chain(
        builder: &mut LirBuilder,
        mut current: LirNodeId,
        start: u32,
        end: u32,
    ) -> LirNodeId {
        for value in start..=end {
            let next = builder.alloc_text(LirNodeKind::Value, "+");
            let literal_node = literal(builder, &value.to_string());
            builder.node_mut(next).unwrap().children = vec![current, literal_node];
            current = next;
        }
        current
    }

    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let inner = builder.alloc_text(LirNodeKind::Instruction, "merge_pair");
    let inner_param_left = builder.alloc_text(LirNodeKind::Value, "left");
    let inner_param_right = builder.alloc_text(LirNodeKind::Value, "right");
    let inner_block = builder.alloc(LirNodeKind::Block);
    let inner_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let inner_head = builder.alloc_text(LirNodeKind::Value, "+");
    builder.node_mut(inner_head).unwrap().children = vec![inner_param_left, inner_param_right];
    let inner_result = append_literal_chain(&mut builder, inner_head, 1, 15);
    builder.node_mut(inner_ret).unwrap().children = vec![inner_result];
    builder.node_mut(inner_block).unwrap().children = vec![inner_ret];
    builder.node_mut(inner).unwrap().children =
        vec![inner_param_left, inner_param_right, inner_block];

    let outer = builder.alloc_text(LirNodeKind::Instruction, "wrap_sum");
    let outer_param_left = builder.alloc_text(LirNodeKind::Value, "left");
    let outer_param_right = builder.alloc_text(LirNodeKind::Value, "right");
    let outer_block = builder.alloc(LirNodeKind::Block);
    let outer_call = builder.alloc(LirNodeKind::Call);
    let outer_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    builder.node_mut(outer_call).unwrap().children =
        vec![outer_callee, outer_param_left, outer_param_right];
    let outer_result = append_literal_chain(&mut builder, outer_call, 1, 15);
    builder.node_mut(outer_block).unwrap().children = vec![outer_result];
    builder.node_mut(outer).unwrap().children =
        vec![outer_param_left, outer_param_right, outer_block];

    let root_call = builder.alloc(LirNodeKind::Call);
    let root_callee = builder.alloc_text(LirNodeKind::Value, "wrap_sum");
    let root_left = literal(&mut builder, "2");
    let root_right = literal(&mut builder, "3");
    builder.node_mut(root_call).unwrap().children = vec![root_callee, root_left, root_right];

    builder.node_mut(root).unwrap().children = vec![inner, outer, root_call];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    let mir = MirAnalysisProgram {
        root: kali_mir::MirNodeId::new(0),
        arena_facts: Vec::new(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("wrap_sum".to_string()),
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

    Optimizer::new(OptimizationLevel::ReleaseAdvanced)
        .optimize_program_with_mir(&mut program, &mir);

    let root_call_node = &program.nodes[root_call.0 as usize];
    let outer_specialized_name = root_call_node
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized wrap_sum call target should exist");
    assert!(outer_specialized_name.starts_with("wrap_sum$spec$"));

    let nested_specialized_count = program
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
    assert_eq!(nested_specialized_count, 1);

    let expected_total = 2 + 3 + (1..=15).sum::<u32>();
    let expected_literal = expected_total.to_string();
    let literal_values: Vec<_> = program
        .nodes
        .iter()
        .filter(|node| node.kind == LirNodeKind::Literal)
        .map(|node| node.text.clone())
        .collect();
    let literal_total = literal_values
        .iter()
        .any(|node| node.as_deref() == Some(expected_literal.as_str()));
    assert!(
        literal_total,
        "release-advanced layout-specialized clones should still allow nested generic specialization to fold the inner call; literals={literal_values:?}"
    );
}

#[test]
fn release_reuses_generic_specializations_across_layout_specialized_owners() {
    fn append_literal_chain(
        builder: &mut LirBuilder,
        mut current: LirNodeId,
        start: u32,
        end: u32,
    ) -> LirNodeId {
        for value in start..=end {
            let next = builder.alloc_text(LirNodeKind::Value, "+");
            let literal_node = literal(builder, &value.to_string());
            builder.node_mut(next).unwrap().children = vec![current, literal_node];
            current = next;
        }
        current
    }

    fn build_object(builder: &mut LirBuilder, first_key: &str, second_key: &str) -> LirNodeId {
        let object = builder.alloc(LirNodeKind::Value);
        let init_a = builder.alloc_text(LirNodeKind::Value, "init");
        let key_a = literal(builder, first_key);
        let value_a = literal(builder, "1");
        builder.node_mut(init_a).unwrap().children = vec![key_a, value_a];

        let init_b = builder.alloc_text(LirNodeKind::Value, "init");
        let key_b = literal(builder, second_key);
        let value_b = literal(builder, "2");
        builder.node_mut(init_b).unwrap().children = vec![key_b, value_b];

        builder.node_mut(object).unwrap().children = vec![init_a, init_b];
        object
    }

    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let merge_pair = builder.alloc_text(LirNodeKind::Instruction, "merge_pair");
    let merge_left = builder.alloc_text(LirNodeKind::Value, "left");
    let merge_right = builder.alloc_text(LirNodeKind::Value, "right");
    let merge_block = builder.alloc(LirNodeKind::Block);
    let merge_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let merge_head = builder.alloc_text(LirNodeKind::Value, "+");
    builder.node_mut(merge_head).unwrap().children = vec![merge_left, merge_right];
    let merge_result = append_literal_chain(&mut builder, merge_head, 1, 8);
    builder.node_mut(merge_ret).unwrap().children = vec![merge_result];
    builder.node_mut(merge_block).unwrap().children = vec![merge_ret];
    builder.node_mut(merge_pair).unwrap().children = vec![merge_left, merge_right, merge_block];

    let wrapper_a = builder.alloc_text(LirNodeKind::Instruction, "wrapper_a");
    let wrapper_a_param = builder.alloc_text(LirNodeKind::Value, "payload");
    let wrapper_a_block = builder.alloc(LirNodeKind::Block);
    let wrapper_a_call = builder.alloc(LirNodeKind::Call);
    let wrapper_a_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let wrapper_a_left = literal(&mut builder, "2");
    let wrapper_a_right = literal(&mut builder, "3");
    builder.node_mut(wrapper_a_call).unwrap().children =
        vec![wrapper_a_callee, wrapper_a_left, wrapper_a_right];
    let wrapper_a_result = append_literal_chain(&mut builder, wrapper_a_call, 1, 8);
    builder.node_mut(wrapper_a_block).unwrap().children = vec![wrapper_a_result];
    builder.node_mut(wrapper_a).unwrap().children = vec![wrapper_a_param, wrapper_a_block];

    let wrapper_b = builder.alloc_text(LirNodeKind::Instruction, "wrapper_b");
    let wrapper_b_param = builder.alloc_text(LirNodeKind::Value, "payload");
    let wrapper_b_block = builder.alloc(LirNodeKind::Block);
    let wrapper_b_call = builder.alloc(LirNodeKind::Call);
    let wrapper_b_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let wrapper_b_left = literal(&mut builder, "2");
    let wrapper_b_right = literal(&mut builder, "3");
    builder.node_mut(wrapper_b_call).unwrap().children =
        vec![wrapper_b_callee, wrapper_b_left, wrapper_b_right];
    let wrapper_b_result = append_literal_chain(&mut builder, wrapper_b_call, 1, 8);
    builder.node_mut(wrapper_b_block).unwrap().children = vec![wrapper_b_result];
    builder.node_mut(wrapper_b).unwrap().children = vec![wrapper_b_param, wrapper_b_block];

    let root_call_a = builder.alloc(LirNodeKind::Call);
    let root_call_a_callee = builder.alloc_text(LirNodeKind::Value, "wrapper_a");
    let root_call_a_arg = build_object(&mut builder, "x", "y");
    builder.node_mut(root_call_a).unwrap().children = vec![root_call_a_callee, root_call_a_arg];

    let root_call_b = builder.alloc(LirNodeKind::Call);
    let root_call_b_callee = builder.alloc_text(LirNodeKind::Value, "wrapper_b");
    let root_call_b_arg = build_object(&mut builder, "x", "z");
    builder.node_mut(root_call_b).unwrap().children = vec![root_call_b_callee, root_call_b_arg];

    builder.node_mut(root).unwrap().children =
        vec![merge_pair, wrapper_a, wrapper_b, root_call_a, root_call_b];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    let wrapper_a_layout = LayoutDescriptor::Struct {
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
    let wrapper_b_layout = LayoutDescriptor::Struct {
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
        nodes: Vec::new(),
        functions: vec![
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
            kali_mir::MirFunction {
                name: Some("wrapper_a".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
                bindings: vec![kali_mir::MirBinding {
                    name: "payload".to_string(),
                    kind: MirBindingKind::Parameter,
                    ownership: kali_mir::OwnershipClass::Borrowed,
                    layout: wrapper_a_layout,
                    escapes: false,
                    captured_by: Vec::new(),
                }],
            },
            kali_mir::MirFunction {
                name: Some("wrapper_b".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
                bindings: vec![kali_mir::MirBinding {
                    name: "payload".to_string(),
                    kind: MirBindingKind::Parameter,
                    ownership: kali_mir::OwnershipClass::Borrowed,
                    layout: wrapper_b_layout,
                    escapes: false,
                    captured_by: Vec::new(),
                }],
            },
        ],
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program_with_mir(&mut program, &mir);

    let wrapper_a_specialized_name = program.nodes[root_call_a.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized wrapper_a call target should exist");
    let wrapper_b_specialized_name = program.nodes[root_call_b.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized wrapper_b call target should exist");

    assert!(wrapper_a_specialized_name.starts_with("wrapper_a$spec$"));
    assert!(wrapper_b_specialized_name.starts_with("wrapper_b$spec$"));
    assert_ne!(
        wrapper_a_specialized_name, wrapper_b_specialized_name,
        "distinct module-like owners should still produce distinct layout-specialized wrappers"
    );

    let merge_pair_specialized_name = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|text| text.starts_with("merge_pair$spec$"))
        })
        .map(|node| {
            node.text
                .clone()
                .expect("specialized merge_pair should have a name")
        })
        .next()
        .expect("nested generic specialization should be created exactly once");

    let merge_pair_specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node.text.as_deref() == Some(merge_pair_specialized_name.as_str())
        })
        .count();
    assert_eq!(
        merge_pair_specialized_count, 1,
        "identical generic specializations should be reused across layout-specialized owners"
    );

    let merge_pair_call_count = program
        .nodes
        .iter()
        .filter(|node| node.kind == LirNodeKind::Call)
        .filter(|node| {
            node.children
                .first()
                .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
                .and_then(|callee| callee.text.as_deref())
                == Some(merge_pair_specialized_name.as_str())
        })
        .count();
    assert!(
        merge_pair_call_count >= 2,
        "layout-specialized wrappers should both retarget to the same nested generic specialization"
    );
}

#[test]
fn release_advanced_reuses_generic_specializations_across_layout_specialized_owners() {
    fn append_literal_chain(
        builder: &mut LirBuilder,
        mut current: LirNodeId,
        start: u32,
        end: u32,
    ) -> LirNodeId {
        for value in start..=end {
            let next = builder.alloc_text(LirNodeKind::Value, "+");
            let literal_node = literal(builder, &value.to_string());
            builder.node_mut(next).unwrap().children = vec![current, literal_node];
            current = next;
        }
        current
    }

    fn build_object(builder: &mut LirBuilder, first_key: &str, second_key: &str) -> LirNodeId {
        let object = builder.alloc(LirNodeKind::Value);
        let init_a = builder.alloc_text(LirNodeKind::Value, "init");
        let key_a = literal(builder, first_key);
        let value_a = literal(builder, "1");
        builder.node_mut(init_a).unwrap().children = vec![key_a, value_a];

        let init_b = builder.alloc_text(LirNodeKind::Value, "init");
        let key_b = literal(builder, second_key);
        let value_b = literal(builder, "2");
        builder.node_mut(init_b).unwrap().children = vec![key_b, value_b];

        builder.node_mut(object).unwrap().children = vec![init_a, init_b];
        object
    }

    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let merge_pair = builder.alloc_text(LirNodeKind::Instruction, "merge_pair");
    let merge_left = builder.alloc_text(LirNodeKind::Value, "left");
    let merge_right = builder.alloc_text(LirNodeKind::Value, "right");
    let merge_block = builder.alloc(LirNodeKind::Block);
    let merge_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let merge_head = builder.alloc_text(LirNodeKind::Value, "+");
    builder.node_mut(merge_head).unwrap().children = vec![merge_left, merge_right];
    let merge_result = append_literal_chain(&mut builder, merge_head, 1, 15);
    builder.node_mut(merge_ret).unwrap().children = vec![merge_result];
    builder.node_mut(merge_block).unwrap().children = vec![merge_ret];
    builder.node_mut(merge_pair).unwrap().children = vec![merge_left, merge_right, merge_block];

    let wrapper_a = builder.alloc_text(LirNodeKind::Instruction, "wrapper_a");
    let wrapper_a_param = builder.alloc_text(LirNodeKind::Value, "payload");
    let wrapper_a_block = builder.alloc(LirNodeKind::Block);
    let wrapper_a_call = builder.alloc(LirNodeKind::Call);
    let wrapper_a_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let wrapper_a_left = literal(&mut builder, "2");
    let wrapper_a_right = literal(&mut builder, "3");
    builder.node_mut(wrapper_a_call).unwrap().children =
        vec![wrapper_a_callee, wrapper_a_left, wrapper_a_right];
    let wrapper_a_result = append_literal_chain(&mut builder, wrapper_a_call, 1, 15);
    builder.node_mut(wrapper_a_block).unwrap().children = vec![wrapper_a_result];
    builder.node_mut(wrapper_a).unwrap().children = vec![wrapper_a_param, wrapper_a_block];

    let wrapper_b = builder.alloc_text(LirNodeKind::Instruction, "wrapper_b");
    let wrapper_b_param = builder.alloc_text(LirNodeKind::Value, "payload");
    let wrapper_b_block = builder.alloc(LirNodeKind::Block);
    let wrapper_b_call = builder.alloc(LirNodeKind::Call);
    let wrapper_b_callee = builder.alloc_text(LirNodeKind::Value, "merge_pair");
    let wrapper_b_left = literal(&mut builder, "2");
    let wrapper_b_right = literal(&mut builder, "3");
    builder.node_mut(wrapper_b_call).unwrap().children =
        vec![wrapper_b_callee, wrapper_b_left, wrapper_b_right];
    let wrapper_b_result = append_literal_chain(&mut builder, wrapper_b_call, 1, 15);
    builder.node_mut(wrapper_b_block).unwrap().children = vec![wrapper_b_result];
    builder.node_mut(wrapper_b).unwrap().children = vec![wrapper_b_param, wrapper_b_block];

    let root_call_a = builder.alloc(LirNodeKind::Call);
    let root_call_a_callee = builder.alloc_text(LirNodeKind::Value, "wrapper_a");
    let root_call_a_arg = build_object(&mut builder, "x", "y");
    builder.node_mut(root_call_a).unwrap().children = vec![root_call_a_callee, root_call_a_arg];

    let root_call_b = builder.alloc(LirNodeKind::Call);
    let root_call_b_callee = builder.alloc_text(LirNodeKind::Value, "wrapper_b");
    let root_call_b_arg = build_object(&mut builder, "x", "z");
    builder.node_mut(root_call_b).unwrap().children = vec![root_call_b_callee, root_call_b_arg];

    builder.node_mut(root).unwrap().children =
        vec![merge_pair, wrapper_a, wrapper_b, root_call_a, root_call_b];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    let wrapper_a_layout = LayoutDescriptor::Struct {
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
    let wrapper_b_layout = LayoutDescriptor::Struct {
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
        nodes: Vec::new(),
        functions: vec![
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
            kali_mir::MirFunction {
                name: Some("wrapper_a".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
                bindings: vec![kali_mir::MirBinding {
                    name: "payload".to_string(),
                    kind: MirBindingKind::Parameter,
                    ownership: kali_mir::OwnershipClass::Borrowed,
                    layout: wrapper_a_layout,
                    escapes: false,
                    captured_by: Vec::new(),
                }],
            },
            kali_mir::MirFunction {
                name: Some("wrapper_b".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
                bindings: vec![kali_mir::MirBinding {
                    name: "payload".to_string(),
                    kind: MirBindingKind::Parameter,
                    ownership: kali_mir::OwnershipClass::Borrowed,
                    layout: wrapper_b_layout,
                    escapes: false,
                    captured_by: Vec::new(),
                }],
            },
        ],
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced)
        .optimize_program_with_mir(&mut program, &mir);

    let wrapper_a_specialized_name = program.nodes[root_call_a.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized wrapper_a call target should exist");
    let wrapper_b_specialized_name = program.nodes[root_call_b.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized wrapper_b call target should exist");

    assert!(wrapper_a_specialized_name.starts_with("wrapper_a$spec$"));
    assert!(wrapper_b_specialized_name.starts_with("wrapper_b$spec$"));
    assert_ne!(
        wrapper_a_specialized_name, wrapper_b_specialized_name,
        "distinct module-like owners should still produce distinct layout-specialized wrappers"
    );

    let merge_pair_specialized_name = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|text| text.starts_with("merge_pair$spec$"))
        })
        .map(|node| {
            node.text
                .clone()
                .expect("specialized merge_pair should have a name")
        })
        .next()
        .expect("nested generic specialization should be created exactly once");

    let merge_pair_specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node.text.as_deref() == Some(merge_pair_specialized_name.as_str())
        })
        .count();
    assert_eq!(
        merge_pair_specialized_count, 1,
        "identical generic specializations should be reused across layout-specialized owners"
    );

    let merge_pair_call_count = program
        .nodes
        .iter()
        .filter(|node| node.kind == LirNodeKind::Call)
        .filter(|node| {
            node.children
                .first()
                .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
                .and_then(|callee| callee.text.as_deref())
                == Some(merge_pair_specialized_name.as_str())
        })
        .count();
    assert!(
        merge_pair_call_count >= 2,
        "layout-specialized wrappers should both retarget to the same nested generic specialization"
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
fn release_reuses_generic_specializations_across_reexport_chain() {
    fn append_literal_chain(
        builder: &mut LirBuilder,
        mut current: LirNodeId,
        start: u32,
        end: u32,
    ) -> LirNodeId {
        for value in start..=end {
            let next = builder.alloc_text(LirNodeKind::Value, "+");
            let literal_node = literal(builder, &value.to_string());
            builder.node_mut(next).unwrap().children = vec![current, literal_node];
            current = next;
        }
        current
    }

    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let math_add = builder.alloc_text(LirNodeKind::Instruction, "math_add");
    let math_left = builder.alloc_text(LirNodeKind::Value, "left");
    let math_right = builder.alloc_text(LirNodeKind::Value, "right");
    let math_block = builder.alloc(LirNodeKind::Block);
    let math_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let math_head = builder.alloc_text(LirNodeKind::Value, "+");
    builder.node_mut(math_head).unwrap().children = vec![math_left, math_right];
    let math_result = append_literal_chain(&mut builder, math_head, 1, 8);
    builder.node_mut(math_ret).unwrap().children = vec![math_result];
    builder.node_mut(math_block).unwrap().children = vec![math_ret];
    builder.node_mut(math_add).unwrap().children = vec![math_left, math_right, math_block];

    let module_helper = builder.alloc_text(LirNodeKind::Instruction, "module_helper");
    let helper_left = builder.alloc_text(LirNodeKind::Value, "left");
    let helper_right = builder.alloc_text(LirNodeKind::Value, "right");
    let helper_block = builder.alloc(LirNodeKind::Block);
    let helper_call = builder.alloc(LirNodeKind::Call);
    let helper_callee = builder.alloc_text(LirNodeKind::Value, "math_add");
    builder.node_mut(helper_call).unwrap().children =
        vec![helper_callee, helper_left, helper_right];
    let helper_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let helper_result = append_literal_chain(&mut builder, helper_call, 1, 8);
    builder.node_mut(helper_ret).unwrap().children = vec![helper_result];
    builder.node_mut(helper_block).unwrap().children = vec![helper_ret];
    builder.node_mut(module_helper).unwrap().children =
        vec![helper_left, helper_right, helper_block];

    let bridge = builder.alloc_text(LirNodeKind::Instruction, "bridge");
    let bridge_left = builder.alloc_text(LirNodeKind::Value, "left");
    let bridge_right = builder.alloc_text(LirNodeKind::Value, "right");
    let bridge_block = builder.alloc(LirNodeKind::Block);
    let bridge_call = builder.alloc(LirNodeKind::Call);
    let bridge_callee = builder.alloc_text(LirNodeKind::Value, "module_helper");
    builder.node_mut(bridge_call).unwrap().children =
        vec![bridge_callee, bridge_left, bridge_right];
    let bridge_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let bridge_result = append_literal_chain(&mut builder, bridge_call, 1, 8);
    builder.node_mut(bridge_ret).unwrap().children = vec![bridge_result];
    builder.node_mut(bridge_block).unwrap().children = vec![bridge_ret];
    builder.node_mut(bridge).unwrap().children = vec![bridge_left, bridge_right, bridge_block];

    let public_a = builder.alloc_text(LirNodeKind::Instruction, "public_a");
    let public_a_left = builder.alloc_text(LirNodeKind::Value, "left");
    let public_a_right = builder.alloc_text(LirNodeKind::Value, "right");
    let public_a_block = builder.alloc(LirNodeKind::Block);
    let public_a_call = builder.alloc(LirNodeKind::Call);
    let public_a_callee = builder.alloc_text(LirNodeKind::Value, "bridge");
    builder.node_mut(public_a_call).unwrap().children =
        vec![public_a_callee, public_a_left, public_a_right];
    let public_a_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let public_a_result = append_literal_chain(&mut builder, public_a_call, 1, 8);
    builder.node_mut(public_a_ret).unwrap().children = vec![public_a_result];
    builder.node_mut(public_a_block).unwrap().children = vec![public_a_ret];
    builder.node_mut(public_a).unwrap().children =
        vec![public_a_left, public_a_right, public_a_block];

    let public_b = builder.alloc_text(LirNodeKind::Instruction, "public_b");
    let public_b_left = builder.alloc_text(LirNodeKind::Value, "left");
    let public_b_right = builder.alloc_text(LirNodeKind::Value, "right");
    let public_b_block = builder.alloc(LirNodeKind::Block);
    let public_b_call = builder.alloc(LirNodeKind::Call);
    let public_b_callee = builder.alloc_text(LirNodeKind::Value, "bridge");
    builder.node_mut(public_b_call).unwrap().children =
        vec![public_b_callee, public_b_left, public_b_right];
    let public_b_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let public_b_result = append_literal_chain(&mut builder, public_b_call, 1, 8);
    builder.node_mut(public_b_ret).unwrap().children = vec![public_b_result];
    builder.node_mut(public_b_block).unwrap().children = vec![public_b_ret];
    builder.node_mut(public_b).unwrap().children =
        vec![public_b_left, public_b_right, public_b_block];

    let root_call_a = builder.alloc(LirNodeKind::Call);
    let root_call_a_callee = builder.alloc_text(LirNodeKind::Value, "public_a");
    let root_call_a_left = literal(&mut builder, "2");
    let root_call_a_right = literal(&mut builder, "3");
    builder.node_mut(root_call_a).unwrap().children =
        vec![root_call_a_callee, root_call_a_left, root_call_a_right];

    let root_call_b = builder.alloc(LirNodeKind::Call);
    let root_call_b_callee = builder.alloc_text(LirNodeKind::Value, "public_b");
    let root_call_b_left = literal(&mut builder, "2");
    let root_call_b_right = literal(&mut builder, "3");
    builder.node_mut(root_call_b).unwrap().children =
        vec![root_call_b_callee, root_call_b_left, root_call_b_right];

    builder.node_mut(root).unwrap().children = vec![
        math_add,
        module_helper,
        bridge,
        public_a,
        public_b,
        root_call_a,
        root_call_b,
    ];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let public_a_specialized_name = program.nodes[root_call_a.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized public_a call target should exist");
    let public_b_specialized_name = program.nodes[root_call_b.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized public_b call target should exist");
    assert!(public_a_specialized_name.starts_with("public_a$spec$"));
    assert!(public_b_specialized_name.starts_with("public_b$spec$"));

    let bridge_specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|text| text.starts_with("bridge$spec$"))
        })
        .count();
    assert_eq!(
        bridge_specialized_count, 1,
        "re-export chain wrappers should still reuse the same bridge specialization"
    );

    let helper_specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|text| text.starts_with("module_helper$spec$"))
        })
        .count();
    assert_eq!(
        helper_specialized_count, 1,
        "re-export chain wrappers should still reuse the same helper specialization"
    );

    let math_specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|text| text.starts_with("math_add$spec$"))
        })
        .count();
    assert_eq!(
        math_specialized_count, 1,
        "cross-module-style re-export chains should still reuse the same generic specialization"
    );
}

#[test]
fn release_advanced_partially_specializes_reexport_chain() {
    fn append_literal_chain(
        builder: &mut LirBuilder,
        mut current: LirNodeId,
        start: u32,
        end: u32,
    ) -> LirNodeId {
        for value in start..=end {
            let next = builder.alloc_text(LirNodeKind::Value, "+");
            let literal_node = literal(builder, &value.to_string());
            builder.node_mut(next).unwrap().children = vec![current, literal_node];
            current = next;
        }
        current
    }

    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let math_add = builder.alloc_text(LirNodeKind::Instruction, "math_add");
    let math_left = builder.alloc_text(LirNodeKind::Value, "left");
    let math_right = builder.alloc_text(LirNodeKind::Value, "right");
    let math_block = builder.alloc(LirNodeKind::Block);
    let math_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let math_head = builder.alloc_text(LirNodeKind::Value, "+");
    builder.node_mut(math_head).unwrap().children = vec![math_left, math_right];
    let math_result = append_literal_chain(&mut builder, math_head, 1, 8);
    builder.node_mut(math_ret).unwrap().children = vec![math_result];
    builder.node_mut(math_block).unwrap().children = vec![math_ret];
    builder.node_mut(math_add).unwrap().children = vec![math_left, math_right, math_block];

    let module_helper = builder.alloc_text(LirNodeKind::Instruction, "module_helper");
    let helper_left = builder.alloc_text(LirNodeKind::Value, "left");
    let helper_right = builder.alloc_text(LirNodeKind::Value, "right");
    let helper_block = builder.alloc(LirNodeKind::Block);
    let helper_call = builder.alloc(LirNodeKind::Call);
    let helper_callee = builder.alloc_text(LirNodeKind::Value, "math_add");
    builder.node_mut(helper_call).unwrap().children =
        vec![helper_callee, helper_left, helper_right];
    let helper_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let helper_result = append_literal_chain(&mut builder, helper_call, 1, 8);
    builder.node_mut(helper_ret).unwrap().children = vec![helper_result];
    builder.node_mut(helper_block).unwrap().children = vec![helper_ret];
    builder.node_mut(module_helper).unwrap().children =
        vec![helper_left, helper_right, helper_block];

    let bridge = builder.alloc_text(LirNodeKind::Instruction, "bridge");
    let bridge_left = builder.alloc_text(LirNodeKind::Value, "left");
    let bridge_right = builder.alloc_text(LirNodeKind::Value, "right");
    let bridge_block = builder.alloc(LirNodeKind::Block);
    let bridge_call = builder.alloc(LirNodeKind::Call);
    let bridge_callee = builder.alloc_text(LirNodeKind::Value, "module_helper");
    builder.node_mut(bridge_call).unwrap().children =
        vec![bridge_callee, bridge_left, bridge_right];
    let bridge_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let bridge_result = append_literal_chain(&mut builder, bridge_call, 1, 8);
    builder.node_mut(bridge_ret).unwrap().children = vec![bridge_result];
    builder.node_mut(bridge_block).unwrap().children = vec![bridge_ret];
    builder.node_mut(bridge).unwrap().children = vec![bridge_left, bridge_right, bridge_block];

    let public_a = builder.alloc_text(LirNodeKind::Instruction, "public_a");
    let public_a_left = builder.alloc_text(LirNodeKind::Value, "left");
    let public_a_right = builder.alloc_text(LirNodeKind::Value, "right");
    let public_a_block = builder.alloc(LirNodeKind::Block);
    let public_a_call = builder.alloc(LirNodeKind::Call);
    let public_a_callee = builder.alloc_text(LirNodeKind::Value, "bridge");
    builder.node_mut(public_a_call).unwrap().children =
        vec![public_a_callee, public_a_left, public_a_right];
    let public_a_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let public_a_result = append_literal_chain(&mut builder, public_a_call, 1, 8);
    builder.node_mut(public_a_ret).unwrap().children = vec![public_a_result];
    builder.node_mut(public_a_block).unwrap().children = vec![public_a_ret];
    builder.node_mut(public_a).unwrap().children =
        vec![public_a_left, public_a_right, public_a_block];

    let public_b = builder.alloc_text(LirNodeKind::Instruction, "public_b");
    let public_b_left = builder.alloc_text(LirNodeKind::Value, "left");
    let public_b_right = builder.alloc_text(LirNodeKind::Value, "right");
    let public_b_block = builder.alloc(LirNodeKind::Block);
    let public_b_call = builder.alloc(LirNodeKind::Call);
    let public_b_callee = builder.alloc_text(LirNodeKind::Value, "bridge");
    builder.node_mut(public_b_call).unwrap().children =
        vec![public_b_callee, public_b_left, public_b_right];
    let public_b_ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let public_b_result = append_literal_chain(&mut builder, public_b_call, 1, 8);
    builder.node_mut(public_b_ret).unwrap().children = vec![public_b_result];
    builder.node_mut(public_b_block).unwrap().children = vec![public_b_ret];
    builder.node_mut(public_b).unwrap().children =
        vec![public_b_left, public_b_right, public_b_block];

    let root_call_a = builder.alloc(LirNodeKind::Call);
    let root_call_a_callee = builder.alloc_text(LirNodeKind::Value, "public_a");
    let root_call_a_left = literal(&mut builder, "2");
    let root_call_a_right = literal(&mut builder, "3");
    builder.node_mut(root_call_a).unwrap().children =
        vec![root_call_a_callee, root_call_a_left, root_call_a_right];

    let root_call_b = builder.alloc(LirNodeKind::Call);
    let root_call_b_callee = builder.alloc_text(LirNodeKind::Value, "public_b");
    let root_call_b_left = literal(&mut builder, "2");
    let root_call_b_right = literal(&mut builder, "3");
    builder.node_mut(root_call_b).unwrap().children =
        vec![root_call_b_callee, root_call_b_left, root_call_b_right];

    builder.node_mut(root).unwrap().children = vec![
        math_add,
        module_helper,
        bridge,
        public_a,
        public_b,
        root_call_a,
        root_call_b,
    ];
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
                name: Some("module_helper".to_string()),
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
                name: Some("bridge".to_string()),
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
                name: Some("public_a".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
                bindings: vec![
                    kali_mir::MirBinding {
                        name: "left".to_string(),
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
                    },
                    kali_mir::MirBinding {
                        name: "right".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::Struct {
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
                        },
                        escapes: false,
                        captured_by: Vec::new(),
                    },
                ],
            },
            kali_mir::MirFunction {
                name: Some("public_b".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
                bindings: vec![
                    kali_mir::MirBinding {
                        name: "left".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::Struct {
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
                        },
                        escapes: false,
                        captured_by: Vec::new(),
                    },
                    kali_mir::MirBinding {
                        name: "right".to_string(),
                        kind: MirBindingKind::Parameter,
                        ownership: kali_mir::OwnershipClass::Borrowed,
                        layout: LayoutDescriptor::Struct {
                            fields: vec![
                                (
                                    "x".to_string(),
                                    Box::new(LayoutDescriptor::Scalar("number".to_string())),
                                ),
                                (
                                    "w".to_string(),
                                    Box::new(LayoutDescriptor::Scalar("number".to_string())),
                                ),
                            ],
                        },
                        escapes: false,
                        captured_by: Vec::new(),
                    },
                ],
            },
        ],
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced)
        .optimize_program_with_mir(&mut program, &mir);

    let public_a_specialized_name = program.nodes[root_call_a.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized public_a call target should exist");
    let public_b_specialized_name = program.nodes[root_call_b.0 as usize]
        .children
        .first()
        .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
        .and_then(|callee| callee.text.as_deref())
        .expect("specialized public_b call target should exist");

    assert_eq!(public_a_specialized_name, "+");
    assert!(public_b_specialized_name.starts_with("public_b$spec$"));
    assert_ne!(
        public_a_specialized_name, public_b_specialized_name,
        "the advanced chain should still distinguish the folded and specialized branches"
    );

    let public_b_specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|text| text.starts_with("public_b$spec$"))
        })
        .count();
    assert_eq!(
        public_b_specialized_count, 1,
        "release-advanced should still materialize the specialized public_b wrapper once"
    );

    let bridge_specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|text| text.starts_with("bridge$spec$"))
        })
        .count();
    assert_eq!(
        bridge_specialized_count, 0,
        "bridge should stay folded in this advanced chain"
    );

    let helper_specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|text| text.starts_with("module_helper$spec$"))
        })
        .count();
    assert_eq!(
        helper_specialized_count, 0,
        "module_helper should stay folded in this advanced chain"
    );

    let math_specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node
                    .text
                    .as_deref()
                    .is_some_and(|text| text.starts_with("math_add$spec$"))
        })
        .count();
    assert_eq!(
        math_specialized_count, 0,
        "math_add should stay folded in this advanced chain"
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
        arena_facts: Vec::new(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
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
