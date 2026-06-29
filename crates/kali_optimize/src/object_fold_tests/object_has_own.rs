use super::*;

#[test]
fn release_folds_object_has_own_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_has_own_call(&mut builder, "hasOwn");
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}

#[test]
fn release_folds_object_has_own_calls_through_optional_chain_wrappers() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_has_own_call(&mut builder, "hasOwn");

    let callee = builder.node_mut(call).unwrap().children[0];
    let object = builder.node_mut(callee).unwrap().children[0];
    builder.node_mut(object).unwrap().text = Some("globalThis?.Object".to_string());

    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}

#[test]
fn release_folds_object_has_own_calls_through_frozen_optional_chain_wrappers() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_has_own_call(&mut builder, "hasOwn");

    let callee = builder.node_mut(call).unwrap().children[0];
    let object = builder.node_mut(callee).unwrap().children[0];
    builder.node_mut(object).unwrap().text = Some("globalThis?.Object".to_string());
    let frozen_callee = build_object_freeze_call(&mut builder, callee);
    builder.node_mut(call).unwrap().children[0] = frozen_callee;

    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}

#[test]
fn release_folds_object_has_own_calls_over_frozen_from_entries_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_has_own_call(&mut builder, "hasOwn");
    let from_entries = build_object_from_entries_call(&mut builder);
    let frozen_from_entries = build_object_freeze_call(&mut builder, from_entries);
    builder.node_mut(call).unwrap().children[1] = frozen_from_entries;
    let key = literal(&mut builder, "\"a\"");
    builder.node_mut(call).unwrap().children[2] = key;
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}

#[test]
fn release_folds_object_has_own_calls_over_frozen_bracketed_from_entries_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_has_own_call(&mut builder, "hasOwn");
    let from_entries =
        build_bracketed_global_this_object_from_entries_call(&mut builder, r#"["fromEntries"]"#);
    let frozen_from_entries = build_object_freeze_call(&mut builder, from_entries);
    builder.node_mut(call).unwrap().children[1] = frozen_from_entries;
    let key = literal(&mut builder, "\"a\"");
    builder.node_mut(call).unwrap().children[2] = key;
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}

#[test]
fn release_folds_object_has_own_calls_through_frozen_callable_wrappers() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = builder.alloc(LirNodeKind::Call);
    let callee = build_object_has_own_callee(&mut builder, "hasOwn");
    let frozen_callee = build_object_freeze_call(&mut builder, callee);

    let object = builder.alloc(LirNodeKind::Value);
    let prop_b = builder.alloc_text(LirNodeKind::Value, "init");
    let prop_b_key = literal(&mut builder, "b");
    let prop_b_value = literal(&mut builder, "1");
    builder.node_mut(prop_b).unwrap().children = vec![prop_b_key, prop_b_value];

    let prop_two = builder.alloc_text(LirNodeKind::Value, "init");
    let prop_two_key = literal(&mut builder, "\"2\"");
    let prop_two_value = literal(&mut builder, "2");
    builder.node_mut(prop_two).unwrap().children = vec![prop_two_key, prop_two_value];

    let prop_one = builder.alloc_text(LirNodeKind::Value, "init");
    let prop_one_key = literal(&mut builder, "\"1\"");
    let prop_one_value = literal(&mut builder, "4");
    builder.node_mut(prop_one).unwrap().children = vec![prop_one_key, prop_one_value];
    builder.node_mut(object).unwrap().children = vec![prop_b, prop_two, prop_one];

    let key = literal(&mut builder, "\"1\"");
    builder.node_mut(call).unwrap().children = vec![frozen_callee, object, key];
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}

#[test]
fn release_advanced_folds_object_has_own_calls_through_frozen_callable_wrappers() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = builder.alloc(LirNodeKind::Call);
    let callee = build_object_has_own_callee(&mut builder, "hasOwn");
    let frozen_callee = build_object_freeze_call(&mut builder, callee);

    let object = builder.alloc(LirNodeKind::Value);
    let prop_b = builder.alloc_text(LirNodeKind::Value, "init");
    let prop_b_key = literal(&mut builder, "b");
    let prop_b_value = literal(&mut builder, "1");
    builder.node_mut(prop_b).unwrap().children = vec![prop_b_key, prop_b_value];

    let prop_two = builder.alloc_text(LirNodeKind::Value, "init");
    let prop_two_key = literal(&mut builder, "\"2\"");
    let prop_two_value = literal(&mut builder, "2");
    builder.node_mut(prop_two).unwrap().children = vec![prop_two_key, prop_two_value];

    let prop_one = builder.alloc_text(LirNodeKind::Value, "init");
    let prop_one_key = literal(&mut builder, "\"1\"");
    let prop_one_value = literal(&mut builder, "4");
    builder.node_mut(prop_one).unwrap().children = vec![prop_one_key, prop_one_value];
    builder.node_mut(object).unwrap().children = vec![prop_b, prop_two, prop_one];

    let key = literal(&mut builder, "\"1\"");
    builder.node_mut(call).unwrap().children = vec![frozen_callee, object, key];
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}

#[test]
fn release_advanced_folds_object_has_own_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_has_own_call(&mut builder, "hasOwn");
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}

#[test]
fn release_advanced_folds_object_has_own_calls_over_frozen_from_entries_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_has_own_call(&mut builder, "hasOwn");
    let from_entries = build_object_from_entries_call(&mut builder);
    let frozen_from_entries = build_object_freeze_call(&mut builder, from_entries);
    builder.node_mut(call).unwrap().children[1] = frozen_from_entries;
    let key = literal(&mut builder, "\"a\"");
    builder.node_mut(call).unwrap().children[2] = key;
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}

#[test]
fn release_advanced_folds_object_has_own_calls_over_frozen_bracketed_from_entries_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_has_own_call(&mut builder, "hasOwn");
    let from_entries =
        build_bracketed_global_this_object_from_entries_call(&mut builder, r#"["fromEntries"]"#);
    let frozen_from_entries = build_object_freeze_call(&mut builder, from_entries);
    builder.node_mut(call).unwrap().children[1] = frozen_from_entries;
    let key = literal(&mut builder, "\"a\"");
    builder.node_mut(call).unwrap().children[2] = key;
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}

#[test]
fn release_advanced_folds_bracketed_object_has_own_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_bracketed_object_has_own_call(&mut builder, r#"["hasOwn"]"#);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}

#[test]
fn release_folds_object_has_own_calls_over_const_bound_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let (const_decl, alias_decl, call) =
        build_const_bound_object_has_own_call(&mut builder, "hasOwn");
    builder.node_mut(root).unwrap().children = vec![const_decl, alias_decl, call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("true"));
}
