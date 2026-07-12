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

// --- Fix wave: eligibility boundary (Lane C review fixes) ---

/// const r = { a: 1, b: 2 };
/// { r.b = 9; }              // store nested inside a bare Block (out-of-lane)
/// Object.values(r);
/// The store inside the Block is counted globally but must NOT be credited as a
/// permitted occurrence, so `r` is ineligible → the enumeration is NOT folded.
fn build_block_nested_store_program() -> (LirProgram, LirNodeId) {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    // { a: 1, b: 2 }
    let mut props = Vec::new();
    for (k, v) in [("a", "1"), ("b", "2")] {
        let key = builder.alloc_text(LirNodeKind::Literal, k);
        let value = builder.alloc_text(LirNodeKind::Literal, v);
        let p = builder.alloc_text(LirNodeKind::Value, "init");
        builder.node_mut(p).unwrap().children = vec![key, value];
        props.push(p);
    }
    let literal = builder.alloc(LirNodeKind::Value);
    builder.node_mut(literal).unwrap().children = props;
    let name = builder.alloc_text(LirNodeKind::Value, "r");
    let declarator = builder.alloc_text(LirNodeKind::Instruction, "r");
    builder.node_mut(declarator).unwrap().children = vec![name, literal];
    let decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    builder.node_mut(decl).unwrap().children = vec![declarator];
    // r.b = 9;  wrapped in a statement wrapper, buried inside a Block.
    let st_base = builder.alloc_text(LirNodeKind::Value, "r");
    let st_member = builder.alloc_text(LirNodeKind::Value, "b");
    builder.node_mut(st_member).unwrap().children = vec![st_base];
    let nine = builder.alloc_text(LirNodeKind::Literal, "9");
    let assign = builder.alloc_text(LirNodeKind::Value, "=");
    builder.node_mut(assign).unwrap().children = vec![st_member, nine];
    let st_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(st_stmt).unwrap().children = vec![assign];
    let block = builder.alloc(LirNodeKind::Block);
    builder.node_mut(block).unwrap().children = vec![st_stmt];
    // Object.values(r)
    let values_call = build_enum_call(&mut builder, "values");
    let values_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(values_stmt).unwrap().children = vec![values_call];
    builder.node_mut(root).unwrap().children = vec![decl, block, values_stmt];
    (
        LirProgram {
            root,
            nodes: builder.into_nodes(),
        },
        values_call,
    )
}

#[test]
fn block_nested_store_disqualifies_binding_no_stale_fold() {
    let (mut program, values_call) = build_block_nested_store_program();
    let optimizer = Optimizer::new(OptimizationLevel::Release);
    optimizer.fold_object_enumeration_calls_ordered(&mut program);

    // Store hidden in a bare Block → r ineligible → enumeration left unfolded
    // (fail-closed; codegen's E5506 backstop rejects the unfolded call).
    assert_eq!(
        program.nodes[values_call.0 as usize].kind,
        LirNodeKind::Call
    );
}

/// const r = { a: 1 };
/// r["a"] = 2;               // computed store: member node has 2 children
/// Object.values(r);
/// The computed store must mark `r` mutated (else the whole program takes the
/// flat, order-blind path and folds the stale pre-store shape). It is an
/// out-of-lane mutation, so `r` is ineligible → the enumeration is NOT folded.
fn build_computed_store_program() -> (LirProgram, LirNodeId) {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    // { a: 1 }
    let key = builder.alloc_text(LirNodeKind::Literal, "a");
    let value = builder.alloc_text(LirNodeKind::Literal, "1");
    let p = builder.alloc_text(LirNodeKind::Value, "init");
    builder.node_mut(p).unwrap().children = vec![key, value];
    let literal = builder.alloc(LirNodeKind::Value);
    builder.node_mut(literal).unwrap().children = vec![p];
    let name = builder.alloc_text(LirNodeKind::Value, "r");
    let declarator = builder.alloc_text(LirNodeKind::Instruction, "r");
    builder.node_mut(declarator).unwrap().children = vec![name, literal];
    let decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    builder.node_mut(decl).unwrap().children = vec![declarator];
    // r["a"] = 2;  computed member = Value with 2 children [base r, index "a"].
    let st_base = builder.alloc_text(LirNodeKind::Value, "r");
    let index = builder.alloc_text(LirNodeKind::Literal, "a");
    let st_member = builder.alloc_text(LirNodeKind::Value, "a");
    builder.node_mut(st_member).unwrap().children = vec![st_base, index];
    let two = builder.alloc_text(LirNodeKind::Literal, "2");
    let assign = builder.alloc_text(LirNodeKind::Value, "=");
    builder.node_mut(assign).unwrap().children = vec![st_member, two];
    let st_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(st_stmt).unwrap().children = vec![assign];
    // Object.values(r)
    let values_call = build_enum_call(&mut builder, "values");
    let values_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(values_stmt).unwrap().children = vec![values_call];
    builder.node_mut(root).unwrap().children = vec![decl, st_stmt, values_stmt];
    (
        LirProgram {
            root,
            nodes: builder.into_nodes(),
        },
        values_call,
    )
}

#[test]
fn computed_member_store_marks_binding_mutated_no_stale_fold() {
    let (mut program, values_call) = build_computed_store_program();
    let optimizer = Optimizer::new(OptimizationLevel::Release);
    optimizer.fold_object_enumeration_calls_ordered(&mut program);

    // Computed store makes `r` mutated + out-of-lane → ineligible → unfolded.
    assert_eq!(
        program.nodes[values_call.0 as usize].kind,
        LirNodeKind::Call
    );
}

/// const r = { a: 1 };
/// r.b = 2;
/// const s = r;              // alias of a mutated binding
/// Object.keys(s);           // must NOT fold stale via the release inline path
/// Exercises the FULL release driver path (optimize_program): the ordered pass
/// already drops alias ids, but the inline-time `constant_bindings` env must
/// too, or inline.rs folds `Object.keys(s)` against r's stale snapshot.
fn build_alias_of_mutated_program() -> (LirProgram, LirNodeId) {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    // { a: 1 }
    let key = builder.alloc_text(LirNodeKind::Literal, "a");
    let value = builder.alloc_text(LirNodeKind::Literal, "1");
    let p = builder.alloc_text(LirNodeKind::Value, "init");
    builder.node_mut(p).unwrap().children = vec![key, value];
    let literal = builder.alloc(LirNodeKind::Value);
    builder.node_mut(literal).unwrap().children = vec![p];
    let name = builder.alloc_text(LirNodeKind::Value, "r");
    let declarator = builder.alloc_text(LirNodeKind::Instruction, "r");
    builder.node_mut(declarator).unwrap().children = vec![name, literal];
    let decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    builder.node_mut(decl).unwrap().children = vec![declarator];
    // r.b = 2;
    let st_base = builder.alloc_text(LirNodeKind::Value, "r");
    let st_member = builder.alloc_text(LirNodeKind::Value, "b");
    builder.node_mut(st_member).unwrap().children = vec![st_base];
    let two = builder.alloc_text(LirNodeKind::Literal, "2");
    let assign = builder.alloc_text(LirNodeKind::Value, "=");
    builder.node_mut(assign).unwrap().children = vec![st_member, two];
    let st_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(st_stmt).unwrap().children = vec![assign];
    // const s = r;
    let alias_init = builder.alloc_text(LirNodeKind::Value, "r");
    let s_name = builder.alloc_text(LirNodeKind::Value, "s");
    let s_declarator = builder.alloc_text(LirNodeKind::Instruction, "s");
    builder.node_mut(s_declarator).unwrap().children = vec![s_name, alias_init];
    let s_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    builder.node_mut(s_decl).unwrap().children = vec![s_declarator];
    // Object.keys(s)
    let callee_obj = builder.alloc_text(LirNodeKind::Value, "Object");
    let callee = builder.alloc_text(LirNodeKind::Value, "keys");
    builder.node_mut(callee).unwrap().children = vec![callee_obj];
    let arg = builder.alloc_text(LirNodeKind::Value, "s");
    let keys_call = builder.alloc(LirNodeKind::Call);
    builder.node_mut(keys_call).unwrap().children = vec![callee, arg];
    let keys_stmt = builder.alloc(LirNodeKind::Value);
    builder.node_mut(keys_stmt).unwrap().children = vec![keys_call];
    builder.node_mut(root).unwrap().children = vec![decl, st_stmt, s_decl, keys_stmt];
    (
        LirProgram {
            root,
            nodes: builder.into_nodes(),
        },
        keys_call,
    )
}

// NOTE (I3): the full release DRIVER harness does not cleanly isolate this
// bug. `specialize_layout_bindings` runs before the driver's inline-path env is
// built and inlines the const `r`'s literal into the store's member base, so by
// that point `collect_mutated_binding_names` finds no bare-identifier base and
// returns empty — the strip has nothing to act on (a synthetic-LIR artifact;
// real front-end programs like Probe 3 retain the base and the alias e2e case
// fails closed via the field-store / join E5506 backstops). So I3 is covered
// here by a UNIT test on the SHARED `strip_mutated_bindings` env-strip that both
// the ordered pass and the release inline path call — asserting an alias of a
// mutated binding is dropped from the fold env.
#[test]
fn strip_mutated_bindings_drops_names_and_aliases() {
    let (program, _keys_call) = build_alias_of_mutated_program();
    let optimizer = Optimizer::new(OptimizationLevel::Release);

    // The flat env resolves both `r` and its alias `s` to r's literal id.
    let mut env = optimizer.collect_constant_bindings(&program, program.root);
    assert!(env.bindings.contains_key("r"));
    assert!(env.bindings.contains_key("s"));
    assert_eq!(env.bindings["r"], env.bindings["s"]);

    let mutated = optimizer.collect_mutated_binding_names(&program);
    assert!(mutated.contains("r"), "r.b = 2 must mark r mutated");

    optimizer.strip_mutated_bindings(&mut env, &mutated);

    // Both the mutated name AND its alias (same resolved literal id) are gone;
    // neither can fold `Object.keys(...)` against r's stale pre-store shape.
    assert!(!env.bindings.contains_key("r"), "mutated name r dropped");
    assert!(
        !env.bindings.contains_key("s"),
        "alias s (resolves to r's literal id) dropped"
    );
}
