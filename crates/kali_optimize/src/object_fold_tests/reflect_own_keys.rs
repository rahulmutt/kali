use super::*;

#[test]
fn folded_keys_are_canonical_quoted_string_literals() {
    // Front-end provenance: LIR property-key text is UNQUOTED for both
    // `{ a: 1 }` and `{ "a": 1 }` (they are identical by HIR). The folded
    // enumeration array must emit its key elements as CANONICAL QUOTED
    // string-literal text (the same `format!("{:?}", ...)` encoding the
    // string-mode fold branch uses), or downstream length/element reads
    // see non-string literals (throw-fallout Stage 2 Lane D).
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    // Object.keys({ b: 1, "2": 2 }) with UNQUOTED key text, as the real
    // front end produces it:
    let callee_obj = builder.alloc_text(LirNodeKind::Value, "Object");
    let callee = builder.alloc_text(LirNodeKind::Value, "keys");
    builder.node_mut(callee).unwrap().children = vec![callee_obj];
    let k1 = builder.alloc_text(LirNodeKind::Literal, "b");
    let v1 = builder.alloc_text(LirNodeKind::Literal, "1");
    let p1 = builder.alloc_text(LirNodeKind::Value, "init");
    builder.node_mut(p1).unwrap().children = vec![k1, v1];
    let k2 = builder.alloc_text(LirNodeKind::Literal, "2");
    let v2 = builder.alloc_text(LirNodeKind::Literal, "2");
    let p2 = builder.alloc_text(LirNodeKind::Value, "init");
    builder.node_mut(p2).unwrap().children = vec![k2, v2];
    let object = builder.alloc(LirNodeKind::Value);
    builder.node_mut(object).unwrap().children = vec![p1, p2];
    let call = builder.alloc(LirNodeKind::Call);
    builder.node_mut(call).unwrap().children = vec![callee, object];
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };
    Optimizer::new(OptimizationLevel::Fast).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    let texts: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    // ES order (index-like "2" first), canonical quoted encoding:
    assert_eq!(texts, vec!["\"2\"", "\"b\""]);
}

#[test]
fn does_not_fold_object_keys_over_a_proto_keyed_literal() {
    // Named requirement (Task 3 review carve-over): `__proto__` is JS's
    // prototype setter, not an own property — node's
    // `Object.keys({ "__proto__": 1, "a": 2 })` is `["a"]`, never
    // `["__proto__", "a"]`. The enumeration fold reads LIR property text
    // directly (it never consults repr shapes), so it must refuse to fold
    // rather than ever emit the phantom `__proto__` key. Leaving the call
    // unfolded routes it to the reject/backstop lane (fail-closed).
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let callee_obj = builder.alloc_text(LirNodeKind::Value, "Object");
    let callee = builder.alloc_text(LirNodeKind::Value, "keys");
    builder.node_mut(callee).unwrap().children = vec![callee_obj];
    let k1 = builder.alloc_text(LirNodeKind::Literal, "\"__proto__\"");
    let v1 = builder.alloc_text(LirNodeKind::Literal, "1");
    let p1 = builder.alloc_text(LirNodeKind::Value, "init");
    builder.node_mut(p1).unwrap().children = vec![k1, v1];
    let k2 = builder.alloc_text(LirNodeKind::Literal, "\"a\"");
    let v2 = builder.alloc_text(LirNodeKind::Literal, "2");
    let p2 = builder.alloc_text(LirNodeKind::Value, "init");
    builder.node_mut(p2).unwrap().children = vec![k2, v2];
    let object = builder.alloc(LirNodeKind::Value);
    builder.node_mut(object).unwrap().children = vec![p1, p2];
    let call = builder.alloc(LirNodeKind::Call);
    builder.node_mut(call).unwrap().children = vec![callee, object];
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };
    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    // Must remain an unfolded Call — never a folded array literal that
    // would carry the phantom `__proto__` key.
    assert_eq!(
        call_node.kind,
        LirNodeKind::Call,
        "Object.keys over a __proto__-keyed literal must not fold"
    );
}

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
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
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
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
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
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
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
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
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
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
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
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
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
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
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
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
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
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
}

/// Build `Object.freeze(<selection>)(<{ b:1, "2":2, "1":4 }>)` where
/// `<selection>` is the caller-supplied frozen-callable-selection node (a
/// `??`/`&&`/`||` logical or a `?` ternary over `Reflect.ownKeys` spellings).
fn build_frozen_selection_own_keys_call(
    builder: &mut LirBuilder,
    selection: LirNodeId,
) -> LirNodeId {
    let frozen = build_object_freeze_call(builder, selection);

    let object = builder.alloc(LirNodeKind::Value);
    let prop_b = builder.alloc_text(LirNodeKind::Value, "init");
    let prop_b_key = literal(builder, "b");
    let prop_b_value = literal(builder, "1");
    builder.node_mut(prop_b).unwrap().children = vec![prop_b_key, prop_b_value];

    let prop_two = builder.alloc_text(LirNodeKind::Value, "init");
    let prop_two_key = literal(builder, "\"2\"");
    let prop_two_value = literal(builder, "2");
    builder.node_mut(prop_two).unwrap().children = vec![prop_two_key, prop_two_value];

    let prop_one = builder.alloc_text(LirNodeKind::Value, "init");
    let prop_one_key = literal(builder, "\"1\"");
    let prop_one_value = literal(builder, "4");
    builder.node_mut(prop_one).unwrap().children = vec![prop_one_key, prop_one_value];

    builder.node_mut(object).unwrap().children = vec![prop_b, prop_two, prop_one];

    let call = builder.alloc(LirNodeKind::Call);
    builder.node_mut(call).unwrap().children = vec![frozen, object];
    call
}

/// `Reflect.ownKeys` member node (`ownKeys` with `Reflect` object child).
fn build_reflect_own_keys_member(builder: &mut LirBuilder) -> LirNodeId {
    let callee = builder.alloc_text(LirNodeKind::Value, "ownKeys");
    let reflect = builder.alloc_text(LirNodeKind::Value, "Reflect");
    builder.node_mut(callee).unwrap().children = vec![reflect];
    callee
}

#[test]
fn folds_reflect_own_keys_over_nullish_frozen_callable_selection() {
    // `Object.freeze((null ?? Reflect.ownKeys))(obj)` — the `??` short-circuit
    // frozen-callable selection must fold identically to the bare
    // `Object.freeze(Reflect.ownKeys)(obj)` form (throw-fallout Stage 2 drain).
    // The enumeration fold's callee resolver must see through the selection via
    // the same both-branches-same-target discipline the shared static oracle
    // (`resolve_static_callable_name` / codegen's
    // `resolve_transparent_callable_node`) already applies.
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let own_keys = build_reflect_own_keys_member(&mut builder);
    let null_lit = literal(&mut builder, "null");
    let selection = builder.alloc_text(LirNodeKind::Value, "??");
    builder.node_mut(selection).unwrap().children = vec![null_lit, own_keys];
    let call = build_frozen_selection_own_keys_call(&mut builder, selection);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };
    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(
        call_node.kind,
        LirNodeKind::Value,
        "nullish frozen-callable selection must fold to an array literal"
    );
    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
}

#[test]
fn folds_reflect_own_keys_over_identical_ternary_frozen_callable_selection() {
    // `Object.freeze((true ? Reflect.ownKeys : Reflect.ownKeys))(obj)` — the
    // ternary selects between two names that resolve to the SAME callable, so
    // it folds (throw-fallout Stage 2 drain).
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let consequent = build_reflect_own_keys_member(&mut builder);
    let alternate = build_reflect_own_keys_member(&mut builder);
    let test = literal(&mut builder, "true");
    let selection = builder.alloc_text(LirNodeKind::Value, "?");
    builder.node_mut(selection).unwrap().children = vec![test, consequent, alternate];
    let call = build_frozen_selection_own_keys_call(&mut builder, selection);
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
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
}

#[test]
fn does_not_fold_frozen_callable_selection_between_two_different_callables() {
    // Fail-closed: a ternary selecting between two DIFFERENT callables
    // (`Object.keys` vs `Reflect.ownKeys`) must NOT fold — the both-branches-
    // must-name-the-SAME-target rule forbids it. The call stays an unfolded
    // Call so it routes to codegen's E5506 backstop (honest rejection, never a
    // miscompiled enumeration of the wrong operator).
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let object_keys = builder.alloc_text(LirNodeKind::Value, "keys");
    let object_obj = builder.alloc_text(LirNodeKind::Value, "Object");
    builder.node_mut(object_keys).unwrap().children = vec![object_obj];
    let reflect_own_keys = build_reflect_own_keys_member(&mut builder);
    let test = builder.alloc_text(LirNodeKind::Value, "cond");
    let selection = builder.alloc_text(LirNodeKind::Value, "?");
    builder.node_mut(selection).unwrap().children = vec![test, object_keys, reflect_own_keys];
    let call = build_frozen_selection_own_keys_call(&mut builder, selection);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };
    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(
        call_node.kind,
        LirNodeKind::Call,
        "selection between two different callables must not fold"
    );
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
    assert_eq!(values, vec!["\"1\"", "\"2\"", "\"b\""]);
}
