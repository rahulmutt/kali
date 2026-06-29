use super::*;

#[test]
fn fast_folds_reflect_own_keys_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_reflect_own_keys_call(&mut builder);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Fast).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}

#[test]
fn release_folds_reflect_own_keys_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_reflect_own_keys_call(&mut builder);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}

#[test]
fn fast_folds_bracketed_reflect_own_keys_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_bracketed_reflect_own_keys_call(&mut builder, r#"["ownKeys"]"#);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Fast).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}

#[test]
fn release_folds_bracketed_reflect_own_keys_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_bracketed_reflect_own_keys_call(&mut builder, r#"["ownKeys"]"#);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}

#[test]
fn release_advanced_folds_reflect_own_keys_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_reflect_own_keys_call(&mut builder);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}

#[test]
fn release_advanced_folds_bracketed_reflect_own_keys_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_bracketed_reflect_own_keys_call(&mut builder, r#"["ownKeys"]"#);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}

#[test]
fn fast_folds_mixed_bracketed_reflect_own_keys_calls_over_literal_object_shapes() {
    assert_mixed_bracketed_reflect_own_keys_folds(OptimizationLevel::Fast);
}

#[test]
fn release_folds_mixed_bracketed_reflect_own_keys_calls_over_literal_object_shapes() {
    assert_mixed_bracketed_reflect_own_keys_folds(OptimizationLevel::Release);
}

#[test]
fn release_advanced_folds_mixed_bracketed_reflect_own_keys_calls_over_literal_object_shapes() {
    assert_mixed_bracketed_reflect_own_keys_folds(OptimizationLevel::ReleaseAdvanced);
}

#[test]
fn fast_folds_global_this_reflect_bracketed_own_keys_calls_over_literal_object_shapes() {
    assert_global_this_reflect_bracketed_own_keys_folds(OptimizationLevel::Fast);
}

#[test]
fn release_folds_global_this_reflect_bracketed_own_keys_calls_over_literal_object_shapes() {
    assert_global_this_reflect_bracketed_own_keys_folds(OptimizationLevel::Release);
}

#[test]
fn release_advanced_folds_global_this_reflect_bracketed_own_keys_calls_over_literal_object_shapes()
{
    assert_global_this_reflect_bracketed_own_keys_folds(OptimizationLevel::ReleaseAdvanced);
}

#[test]
fn release_folds_reflect_own_keys_calls_over_frozen_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
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

    let frozen = build_object_freeze_call(&mut builder, object);
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "ownKeys");
    let reflect = builder.alloc_text(LirNodeKind::Value, "Reflect");
    builder.node_mut(callee).unwrap().children = vec![reflect];
    builder.node_mut(call).unwrap().children = vec![callee, frozen];
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}

#[test]
fn release_advanced_folds_reflect_own_keys_calls_over_const_bound_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let (const_decl, call) = build_const_bound_reflect_own_keys_call(&mut builder);
    builder.node_mut(root).unwrap().children = vec![const_decl, call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());

    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}

#[test]
fn release_folds_reflect_own_keys_calls_over_const_alias_chains() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let (const_decl, alias_decl, alias_two_decl, call) =
        build_alias_bound_reflect_own_keys_call(&mut builder);
    builder.node_mut(root).unwrap().children = vec![const_decl, alias_decl, alias_two_decl, call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());

    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}

#[test]
fn release_advanced_folds_reflect_own_keys_calls_over_const_alias_chains() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let (const_decl, alias_decl, alias_two_decl, call) =
        build_alias_bound_reflect_own_keys_call(&mut builder);
    builder.node_mut(root).unwrap().children = vec![const_decl, alias_decl, alias_two_decl, call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());

    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}
