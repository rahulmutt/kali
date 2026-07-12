use super::*;

/// `Object.<method>(r)` call node: Call → [Value(method) → [Value("Object")], Value("r")].
fn build_enum_call(builder: &mut LirBuilder, method: &str) -> LirNodeId {
    let callee_obj = builder.alloc_text(LirNodeKind::Value, "Object");
    let callee = builder.alloc_text(LirNodeKind::Value, method);
    builder.node_mut(callee).unwrap().children = vec![callee_obj];
    let arg = builder.alloc_text(LirNodeKind::Value, "r");
    let call = builder.alloc(LirNodeKind::Call);
    builder.node_mut(call).unwrap().children = vec![callee, arg];
    call
}

/// const r = { a: 1, b: 2, c: 3 }; delete r.b; r.b = 4;
/// Object.keys(r); Object.values(r);
/// — LIR shapes exactly as the front end produces them (probe-verified
/// during planning: statement wrappers are Value(None) with one child;
/// const decl = Instruction("const") → Instruction(name) → [Value(name), init]).
/// When `nest_delete_in_branch` is set the `delete` statement is wrapped in a
/// Branch node (an out-of-lane, non-straight-line site) so the timeline lane
/// must decline to fold.
/// Returns (program, del_stmt_id, del_unary_id, keys_call_id, values_call_id).
fn build_delete_reinsert_program(
    nest_delete_in_branch: bool,
) -> (LirProgram, LirNodeId, LirNodeId, LirNodeId, LirNodeId) {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    // object literal { a: 1, b: 2, c: 3 }
    let mut props = Vec::new();
    for (k, v) in [("a", "1"), ("b", "2"), ("c", "3")] {
        let key = builder.alloc_text(LirNodeKind::Literal, k);
        let value = builder.alloc_text(LirNodeKind::Literal, v);
        let p = builder.alloc_text(LirNodeKind::Value, "init");
        builder.node_mut(p).unwrap().children = vec![key, value];
        props.push(p);
    }
    let literal = builder.alloc(LirNodeKind::Value);
    builder.node_mut(literal).unwrap().children = props;
    // const r = <literal>
    let name = builder.alloc_text(LirNodeKind::Value, "r");
    let declarator = builder.alloc_text(LirNodeKind::Instruction, "r");
    builder.node_mut(declarator).unwrap().children = vec![name, literal];
    let decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    builder.node_mut(decl).unwrap().children = vec![declarator];
    // delete r.b;  => Value(None) -> Value("delete") -> Value("b") -> Value("r")
    let del_base = builder.alloc_text(LirNodeKind::Value, "r");
    let del_member = builder.alloc_text(LirNodeKind::Value, "b");
    builder.node_mut(del_member).unwrap().children = vec![del_base];
    let del_unary = builder.alloc_text(LirNodeKind::Value, "delete");
    builder.node_mut(del_unary).unwrap().children = vec![del_member];
    let del_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(del_stmt).unwrap().children = vec![del_unary];
    // Optionally bury the delete statement under a Branch node.
    let del_root_child = if nest_delete_in_branch {
        let branch = builder.alloc(LirNodeKind::Branch);
        builder.node_mut(branch).unwrap().children = vec![del_stmt];
        branch
    } else {
        del_stmt
    };
    // r.b = 4;  => Value(None) -> Value("=") -> [member(b->r), Literal(4)]
    let st_base = builder.alloc_text(LirNodeKind::Value, "r");
    let st_member = builder.alloc_text(LirNodeKind::Value, "b");
    builder.node_mut(st_member).unwrap().children = vec![st_base];
    let four = builder.alloc_text(LirNodeKind::Literal, "4");
    let assign = builder.alloc_text(LirNodeKind::Value, "=");
    builder.node_mut(assign).unwrap().children = vec![st_member, four];
    let st_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(st_stmt).unwrap().children = vec![assign];
    // bare-statement enumeration calls
    let keys_call = build_enum_call(&mut builder, "keys");
    let keys_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(keys_stmt).unwrap().children = vec![keys_call];
    let values_call = build_enum_call(&mut builder, "values");
    let values_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(values_stmt).unwrap().children = vec![values_call];
    builder.node_mut(root).unwrap().children =
        vec![decl, del_root_child, st_stmt, keys_stmt, values_stmt];
    (
        LirProgram {
            root,
            nodes: builder.into_nodes(),
        },
        del_stmt,
        del_unary,
        keys_call,
        values_call,
    )
}

#[test]
fn timeline_folds_delete_then_reinsert_to_node_order() {
    let (mut program, _del_stmt, _del_unary, keys_call, values_call) =
        build_delete_reinsert_program(false);
    let optimizer = Optimizer::new(OptimizationLevel::Release);
    optimizer.fold_object_enumeration_calls_ordered(&mut program);

    let keys_node = &program.nodes[keys_call.0 as usize];
    assert_eq!(keys_node.kind, LirNodeKind::Value);
    assert!(keys_node.text.is_none());
    let keys: Vec<_> = keys_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(keys, vec!["\"a\"", "\"c\"", "\"b\""]);

    let values_node = &program.nodes[values_call.0 as usize];
    assert_eq!(values_node.kind, LirNodeKind::Value);
    assert!(values_node.text.is_none());
    let values: Vec<_> = values_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["1", "3", "4"]);
}

#[test]
fn stale_fold_is_dead_mutated_binding_outside_the_lane_does_not_fold() {
    let (mut program, _del_stmt, del_unary, keys_call, _values_call) =
        build_delete_reinsert_program(true);
    let optimizer = Optimizer::new(OptimizationLevel::Release);
    optimizer.fold_object_enumeration_calls_ordered(&mut program);

    // Out-of-lane (delete buried under a Branch) => binding ineligible =>
    // the enumeration call is left unfolded, never folded stale.
    assert_eq!(program.nodes[keys_call.0 as usize].kind, LirNodeKind::Call);
    // The delete node is left in place for codegen's default-deny arm.
    let del_node = &program.nodes[del_unary.0 as usize];
    assert_eq!(del_node.kind, LirNodeKind::Value);
    assert_eq!(del_node.text.as_deref(), Some("delete"));
}

#[test]
fn consumed_delete_statements_are_erased_to_empty_blocks() {
    let (mut program, del_stmt, _del_unary, _keys_call, _values_call) =
        build_delete_reinsert_program(false);
    let optimizer = Optimizer::new(OptimizationLevel::Release);
    optimizer.fold_object_enumeration_calls_ordered(&mut program);

    let del_node = &program.nodes[del_stmt.0 as usize];
    assert_eq!(del_node.kind, LirNodeKind::Block);
    assert!(del_node.text.is_none());
    assert!(del_node.children.is_empty());
}
