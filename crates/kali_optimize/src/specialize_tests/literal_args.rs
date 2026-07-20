use super::*;

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
                name: Some("consume_array".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("echo_text".to_string()),
            kind: kali_mir::MirFunctionKind::Function,
            function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("echo_text_variant".to_string()),
            kind: kali_mir::MirFunctionKind::Function,
            function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("consume_pattern".to_string()),
            kind: kali_mir::MirFunctionKind::Function,
            function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("consume_pattern".to_string()),
            kind: kali_mir::MirFunctionKind::Function,
            function_flavor: None,
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
                name: Some("consume_value".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
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
fn release_advanced_specializes_nullish_literal_arguments() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let function = builder.alloc_text(LirNodeKind::Instruction, "consume_value");
    let param_value = builder.alloc_text(LirNodeKind::Value, "value");
    let block = builder.alloc(LirNodeKind::Block);
    let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let mut previous = param_value;
    for value in 1..=32 {
        let add = builder.alloc_text(LirNodeKind::Value, "+");
        let literal = literal(&mut builder, &value.to_string());
        builder.node_mut(add).unwrap().children = vec![previous, literal];
        previous = add;
    }
    builder.node_mut(ret).unwrap().children = vec![previous];
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
                name: Some("consume_value".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
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

    Optimizer::new(OptimizationLevel::ReleaseAdvanced)
        .optimize_program_with_mir(&mut program, &mir);

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
fn fast_keeps_nullish_literal_arguments_unspecialized() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let function = builder.alloc_text(LirNodeKind::Instruction, "consume_value");
    let param_value = builder.alloc_text(LirNodeKind::Value, "value");
    let block = builder.alloc(LirNodeKind::Block);
    let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let mut previous = param_value;
    for value in 1..=32 {
        let add = builder.alloc_text(LirNodeKind::Value, "+");
        let literal = literal(&mut builder, &value.to_string());
        builder.node_mut(add).unwrap().children = vec![previous, literal];
        previous = add;
    }
    builder.node_mut(ret).unwrap().children = vec![previous];
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
                name: Some("consume_value".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
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

    Optimizer::new(OptimizationLevel::Fast).optimize_program_with_mir(&mut program, &mir);

    for call_id in [call_null_a, call_undefined, call_null_b] {
        let call_node = &program.nodes[call_id.0 as usize];
        assert_eq!(call_node.kind, LirNodeKind::Call);
        let callee_name = call_node
            .children
            .first()
            .and_then(|callee_id| program.nodes.get(callee_id.0 as usize))
            .and_then(|callee| callee.text.as_deref())
            .expect("call target should remain the original function in fast mode");
        assert_eq!(callee_name, "consume_value");
    }

    let specialized_names: Vec<_> = program
        .nodes
        .iter()
        .filter_map(|node| {
            (node.kind == LirNodeKind::Instruction)
                .then_some(node.text.as_deref())
                .flatten()
        })
        .filter(|name| name.starts_with("consume_value$spec$"))
        .collect();
    assert!(
        specialized_names.is_empty(),
        "unexpected specializations: {specialized_names:?}"
    );
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
                name: Some("consume_special_number".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
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
                name: Some("consume_flag".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("consume_number".to_string()),
            kind: kali_mir::MirFunctionKind::Function,
            function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("consume_zero".to_string()),
            kind: kali_mir::MirFunctionKind::Function,
            function_flavor: None,
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
        arena_facts: Vec::new(),
        parent_labels: Default::default(),
        nodes: Vec::new(),
        functions: vec![kali_mir::MirFunction {
            name: Some("consume_bigint".to_string()),
            kind: kali_mir::MirFunctionKind::Function,
            function_flavor: None,
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
fn release_advanced_specializes_bigint_literal_arguments() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let function = builder.alloc_text(LirNodeKind::Instruction, "consume_bigint");
    let param_value = builder.alloc_text(LirNodeKind::Value, "value");
    let block = builder.alloc(LirNodeKind::Block);
    let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let mut previous = param_value;
    for value in 1..=32 {
        let add = builder.alloc_text(LirNodeKind::Value, "+");
        let literal = literal(&mut builder, &format!("{value}n"));
        builder.node_mut(add).unwrap().children = vec![previous, literal];
        previous = add;
    }
    builder.node_mut(ret).unwrap().children = vec![previous];
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
                name: Some("consume_bigint".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
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

    Optimizer::new(OptimizationLevel::ReleaseAdvanced)
        .optimize_program_with_mir(&mut program, &mir);

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
