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
    let prop_two_key = literal(&mut builder, "2");
    let prop_two_value = literal(&mut builder, "2");
    builder.node_mut(prop_two).unwrap().children = vec![prop_two_key, prop_two_value];

    let prop_one = builder.alloc_text(LirNodeKind::Value, "init");
    let prop_one_key = literal(&mut builder, "1");
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
    let prop_two_key = literal(&mut builder, "2");
    let prop_two_value = literal(&mut builder, "2");
    builder.node_mut(prop_two).unwrap().children = vec![prop_two_key, prop_two_value];

    let prop_one = builder.alloc_text(LirNodeKind::Value, "init");
    let prop_one_key = literal(&mut builder, "1");
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

/// A `hasOwn` fold over an object literal whose only key slot holds the
/// THREE-character property name `"a"` (quote characters included in the name,
/// as `{'"a"': 1}` writes it), probed with the ONE-character name `a`.
///
/// Builds the call directly rather than through `build_object_has_own_call`
/// because that builder's object always contains the probed key.
fn build_quoted_name_object_has_own_call(builder: &mut LirBuilder, probe: &str) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "hasOwn");
    let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
    builder.node_mut(callee).unwrap().children = vec![object_object];

    let object = builder.alloc(LirNodeKind::Value);
    let property = builder.alloc_text(LirNodeKind::Value, "init");
    // The KEY SLOT: the property name is `"a"`, three characters.
    let key_slot = literal(builder, "\"a\"");
    let value = literal(builder, "1");
    builder.node_mut(property).unwrap().children = vec![key_slot, value];
    builder.node_mut(object).unwrap().children = vec![property];

    // The PROBE: an ordinary string literal, which keeps its source quoting.
    let probe = literal(builder, probe);
    builder.node_mut(call).unwrap().children = vec![callee, object, probe];
    call
}

#[test]
fn release_folds_object_has_own_to_false_when_the_key_is_absent() {
    // THE ONLY NEGATIVE ASSERTION ON THIS LANE, and it is load-bearing: the
    // fold runs at Release/ReleaseAdvanced only (`driver.rs`), while `kali run`
    // is Fast mode and `--release` exists only on `kali build`, so NOTHING in
    // the case corpus can reach it. Every other assertion in this file expects
    // `true`, which means a regression flipping the fold to always-`true` --
    // the mirror image of the always-`false` one Task 3 silently introduced
    // here and Task 5 fixed -- would pass the entire test tree.
    //
    // The program is `Object.hasOwn({'"a"': 1}, 'a')`. Node says `false`: the
    // object's one property is named `"a"` with the quote characters IN the
    // name, and `'a'` is a different name. While both sides were un-quoted
    // before comparing, this folded to a wrong `true` -- the fold-lane twin of
    // the member read that invented `p['a']`.
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_quoted_name_object_has_own_call(&mut builder, "'a'");
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Literal);
    assert_eq!(call_node.text.as_deref(), Some("false"));
}

#[test]
fn release_folds_object_has_own_to_true_for_the_quoted_name_itself() {
    // The positive control for the case above, in the same shape: probing the
    // property that IS there (`Object.hasOwn({'"a"': 1}, '"a"')`) must still
    // fold to `true`. Pinning the miss without pinning the hit would admit a
    // "fix" that simply stopped finding anything.
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_quoted_name_object_has_own_call(&mut builder, "'\"a\"'");
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
