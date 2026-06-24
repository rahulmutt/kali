//! kali_optimize-specific test builders (compiled under cfg(test)).
use crate::*;
use kali_lir::{LirBuilder, LirNodeKind};

pub(crate) fn literal(builder: &mut LirBuilder, value: &str) -> LirNodeId {
    builder.alloc_text(LirNodeKind::Literal, value)
}

pub(crate) fn build_hot_add_program() -> (LirProgram, LirNodeId) {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let function = builder.alloc_text(LirNodeKind::Instruction, "hot_add");
    let param = builder.alloc_text(LirNodeKind::Value, "value");
    let block = builder.alloc(LirNodeKind::Block);
    let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let mut expression = param;

    for literal_text in ["1", "2", "3", "4", "5", "6"] {
        let add = builder.alloc_text(LirNodeKind::Value, "+");
        let literal = literal(&mut builder, literal_text);
        builder.node_mut(add).unwrap().children = vec![expression, literal];
        expression = add;
    }

    builder.node_mut(ret).unwrap().children = vec![expression];
    builder.node_mut(block).unwrap().children = vec![ret];
    builder.node_mut(function).unwrap().children = vec![param, block];

    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "hot_add");
    let argument = builder.alloc_text(LirNodeKind::Value, "input");
    builder.node_mut(call).unwrap().children = vec![callee, argument];
    builder.node_mut(root).unwrap().children = vec![function, call];

    (
        LirProgram {
            root,
            nodes: builder.into_nodes(),
        },
        call,
    )
}

pub(crate) fn build_short_circuit_program(operator: &str, left: &str, right: &str) -> (LirProgram, LirNodeId) {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let node = builder.alloc_text(LirNodeKind::Value, operator);
    let lhs = literal(&mut builder, left);
    let rhs = builder.alloc_text(LirNodeKind::Value, right);
    builder.node_mut(node).unwrap().children = vec![lhs, rhs];
    builder.node_mut(root).unwrap().children = vec![node];

    (
        LirProgram {
            root,
            nodes: builder.into_nodes(),
        },
        node,
    )
}

pub(crate) fn build_object_enumeration_call(builder: &mut LirBuilder, callee_name: &str) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
    builder.node_mut(callee).unwrap().children = vec![object_object];

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
    builder.node_mut(call).unwrap().children = vec![callee, object];
    call
}

pub(crate) fn build_object_string_enumeration_call(
    builder: &mut LirBuilder,
    callee_name: &str,
    string_value: &str,
) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
    builder.node_mut(callee).unwrap().children = vec![object_object];
    let string_literal = literal(builder, string_value);
    builder.node_mut(call).unwrap().children = vec![callee, string_literal];
    call
}

pub(crate) fn build_bracketed_global_this_object_string_enumeration_call(
    builder: &mut LirBuilder,
    callee_name: &str,
    string_value: &str,
) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, r#"["Object"]"#);
    let global_this = builder.alloc_text(LirNodeKind::Value, "globalThis");
    builder.node_mut(object_object).unwrap().children = vec![global_this];
    builder.node_mut(callee).unwrap().children = vec![object_object];
    let string_literal = literal(builder, string_value);
    builder.node_mut(call).unwrap().children = vec![callee, string_literal];
    call
}

pub(crate) fn build_global_this_object_string_enumeration_call(
    builder: &mut LirBuilder,
    callee_name: &str,
    string_value: &str,
) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
    let global_this = builder.alloc_text(LirNodeKind::Value, "globalThis");
    builder.node_mut(object_object).unwrap().children = vec![global_this];
    builder.node_mut(callee).unwrap().children = vec![object_object];
    let string_literal = literal(builder, string_value);
    builder.node_mut(call).unwrap().children = vec![callee, string_literal];
    call
}

pub(crate) fn build_object_from_entries_call(builder: &mut LirBuilder) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "fromEntries");
    let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
    builder.node_mut(callee).unwrap().children = vec![object_object];

    let entries = builder.alloc(LirNodeKind::Value);
    let entry_b = builder.alloc(LirNodeKind::Value);
    let entry_b_key = literal(builder, "\"b\"");
    let entry_b_value = literal(builder, "1");
    builder.node_mut(entry_b).unwrap().children = vec![entry_b_key, entry_b_value];

    let entry_a = builder.alloc(LirNodeKind::Value);
    let entry_a_key = literal(builder, "\"a\"");
    let entry_a_value = literal(builder, "2");
    builder.node_mut(entry_a).unwrap().children = vec![entry_a_key, entry_a_value];

    let entry_b_overwrite = builder.alloc(LirNodeKind::Value);
    let entry_b_overwrite_key = literal(builder, "\"b\"");
    let entry_b_overwrite_value = literal(builder, "3");
    builder.node_mut(entry_b_overwrite).unwrap().children =
        vec![entry_b_overwrite_key, entry_b_overwrite_value];

    builder.node_mut(entries).unwrap().children = vec![entry_b, entry_a, entry_b_overwrite];
    builder.node_mut(call).unwrap().children = vec![callee, entries];
    call
}

pub(crate) fn build_global_this_object_from_entries_call(builder: &mut LirBuilder) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "fromEntries");
    let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
    let global_this = builder.alloc_text(LirNodeKind::Value, "globalThis");
    builder.node_mut(object_object).unwrap().children = vec![global_this];
    builder.node_mut(callee).unwrap().children = vec![object_object];

    let entries = builder.alloc(LirNodeKind::Value);
    let entry_b = builder.alloc(LirNodeKind::Value);
    let entry_b_key = literal(builder, "\"b\"");
    let entry_b_value = literal(builder, "1");
    builder.node_mut(entry_b).unwrap().children = vec![entry_b_key, entry_b_value];

    let entry_a = builder.alloc(LirNodeKind::Value);
    let entry_a_key = literal(builder, "\"a\"");
    let entry_a_value = literal(builder, "2");
    builder.node_mut(entry_a).unwrap().children = vec![entry_a_key, entry_a_value];

    let entry_b_overwrite = builder.alloc(LirNodeKind::Value);
    let entry_b_overwrite_key = literal(builder, "\"b\"");
    let entry_b_overwrite_value = literal(builder, "3");
    builder.node_mut(entry_b_overwrite).unwrap().children =
        vec![entry_b_overwrite_key, entry_b_overwrite_value];

    builder.node_mut(entries).unwrap().children = vec![entry_b, entry_a, entry_b_overwrite];
    builder.node_mut(call).unwrap().children = vec![callee, entries];
    call
}

pub(crate) fn build_bracketed_global_this_object_from_entries_call(
    builder: &mut LirBuilder,
    callee_name: &str,
) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, r#"["Object"]"#);
    let global_this = builder.alloc_text(LirNodeKind::Value, "globalThis");
    builder.node_mut(object_object).unwrap().children = vec![global_this];
    builder.node_mut(callee).unwrap().children = vec![object_object];

    let entries = builder.alloc(LirNodeKind::Value);
    let entry_b = builder.alloc(LirNodeKind::Value);
    let entry_b_key = literal(builder, "\"b\"");
    let entry_b_value = literal(builder, "1");
    builder.node_mut(entry_b).unwrap().children = vec![entry_b_key, entry_b_value];

    let entry_a = builder.alloc(LirNodeKind::Value);
    let entry_a_key = literal(builder, "\"a\"");
    let entry_a_value = literal(builder, "2");
    builder.node_mut(entry_a).unwrap().children = vec![entry_a_key, entry_a_value];

    let entry_b_overwrite = builder.alloc(LirNodeKind::Value);
    let entry_b_overwrite_key = literal(builder, "\"b\"");
    let entry_b_overwrite_value = literal(builder, "3");
    builder.node_mut(entry_b_overwrite).unwrap().children =
        vec![entry_b_overwrite_key, entry_b_overwrite_value];

    builder.node_mut(entries).unwrap().children = vec![entry_b, entry_a, entry_b_overwrite];
    builder.node_mut(call).unwrap().children = vec![callee, entries];
    call
}

pub(crate) fn build_bracketed_global_this_object_enumeration_call(
    builder: &mut LirBuilder,
    callee_name: &str,
) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, r#"["Object"]"#);
    let global_this = builder.alloc_text(LirNodeKind::Value, "globalThis");
    builder.node_mut(object_object).unwrap().children = vec![global_this];
    builder.node_mut(callee).unwrap().children = vec![object_object];

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
    builder.node_mut(call).unwrap().children = vec![callee, object];
    call
}

pub(crate) fn build_reflect_own_keys_call(builder: &mut LirBuilder) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "ownKeys");
    let object_object = builder.alloc_text(LirNodeKind::Value, "Reflect");
    builder.node_mut(callee).unwrap().children = vec![object_object];

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
    builder.node_mut(call).unwrap().children = vec![callee, object];
    call
}

pub(crate) fn build_bracketed_reflect_own_keys_call(builder: &mut LirBuilder, callee_name: &str) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let reflect = builder.alloc_text(LirNodeKind::Value, r#"["Reflect"]"#);
    let global_this = builder.alloc_text(LirNodeKind::Value, "globalThis");
    builder.node_mut(reflect).unwrap().children = vec![global_this];
    builder.node_mut(callee).unwrap().children = vec![reflect];

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
    builder.node_mut(call).unwrap().children = vec![callee, object];
    call
}

pub(crate) fn build_global_this_reflect_own_keys_call(
    builder: &mut LirBuilder,
    callee_name: &str,
) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let reflect = builder.alloc_text(LirNodeKind::Value, "Reflect");
    let global_this = builder.alloc_text(LirNodeKind::Value, "globalThis");
    builder.node_mut(reflect).unwrap().children = vec![global_this];
    builder.node_mut(callee).unwrap().children = vec![reflect];

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
    builder.node_mut(call).unwrap().children = vec![callee, object];
    call
}

pub(crate) fn build_object_freeze_call(builder: &mut LirBuilder, argument: LirNodeId) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "freeze");
    let object = builder.alloc_text(LirNodeKind::Value, "Object");
    builder.node_mut(callee).unwrap().children = vec![object];
    builder.node_mut(call).unwrap().children = vec![callee, argument];
    call
}

pub(crate) fn build_object_has_own_callee(builder: &mut LirBuilder, callee_name: &str) -> LirNodeId {
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
    builder.node_mut(callee).unwrap().children = vec![object_object];
    callee
}

pub(crate) fn build_object_has_own_call(builder: &mut LirBuilder, callee_name: &str) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
    builder.node_mut(callee).unwrap().children = vec![object_object];

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
    let key = literal(builder, "\"1\"");
    builder.node_mut(call).unwrap().children = vec![callee, object, key];
    call
}

pub(crate) fn build_bracketed_object_has_own_call(builder: &mut LirBuilder, callee_name: &str) -> LirNodeId {
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, r#"["Object"]"#);
    let global_this = builder.alloc_text(LirNodeKind::Value, "globalThis");
    builder.node_mut(object_object).unwrap().children = vec![global_this];
    builder.node_mut(callee).unwrap().children = vec![object_object];

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
    let key = literal(builder, "\"1\"");
    builder.node_mut(call).unwrap().children = vec![callee, object, key];
    call
}

pub(crate) fn build_const_bound_object_has_own_call(
    builder: &mut LirBuilder,
    callee_name: &str,
) -> (LirNodeId, LirNodeId, LirNodeId) {
    let const_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let declarator = builder.alloc_text(LirNodeKind::Instruction, "point");
    let binding_name = builder.alloc_text(LirNodeKind::Value, "point");
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
    builder.node_mut(declarator).unwrap().children = vec![binding_name, object];
    builder.node_mut(const_decl).unwrap().children = vec![declarator];

    let alias_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let alias_declarator = builder.alloc_text(LirNodeKind::Instruction, "alias");
    let alias_name = builder.alloc_text(LirNodeKind::Value, "alias");
    let alias_binding = builder.alloc_text(LirNodeKind::Value, "point");
    builder.node_mut(alias_declarator).unwrap().children = vec![alias_name, alias_binding];
    builder.node_mut(alias_decl).unwrap().children = vec![alias_declarator];

    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
    builder.node_mut(callee).unwrap().children = vec![object_object];
    let alias_ref = builder.alloc_text(LirNodeKind::Value, "alias");
    let key = literal(builder, "\"1\"");
    builder.node_mut(call).unwrap().children = vec![callee, alias_ref, key];
    (const_decl, alias_decl, call)
}

pub(crate) fn build_const_bound_reflect_own_keys_call(builder: &mut LirBuilder) -> (LirNodeId, LirNodeId) {
    let const_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let declarator = builder.alloc_text(LirNodeKind::Instruction, "point");
    let binding_name = builder.alloc_text(LirNodeKind::Value, "point");
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
    builder.node_mut(declarator).unwrap().children = vec![binding_name, object];
    builder.node_mut(const_decl).unwrap().children = vec![declarator];

    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "ownKeys");
    let object_object = builder.alloc_text(LirNodeKind::Value, "Reflect");
    builder.node_mut(callee).unwrap().children = vec![object_object];
    let point_ref = builder.alloc_text(LirNodeKind::Value, "point");
    builder.node_mut(call).unwrap().children = vec![callee, point_ref];
    (const_decl, call)
}

pub(crate) fn build_alias_bound_reflect_own_keys_call(
    builder: &mut LirBuilder,
) -> (LirNodeId, LirNodeId, LirNodeId, LirNodeId) {
    let const_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let declarator = builder.alloc_text(LirNodeKind::Instruction, "point");
    let binding_name = builder.alloc_text(LirNodeKind::Value, "point");
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
    builder.node_mut(declarator).unwrap().children = vec![binding_name, object];
    builder.node_mut(const_decl).unwrap().children = vec![declarator];

    let alias_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let alias_declarator = builder.alloc_text(LirNodeKind::Instruction, "alias");
    let alias_name = builder.alloc_text(LirNodeKind::Value, "alias");
    let alias_binding = builder.alloc_text(LirNodeKind::Value, "point");
    builder.node_mut(alias_declarator).unwrap().children = vec![alias_name, alias_binding];
    builder.node_mut(alias_decl).unwrap().children = vec![alias_declarator];

    let alias_two_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let alias_two_declarator = builder.alloc_text(LirNodeKind::Instruction, "alias2");
    let alias_two_name = builder.alloc_text(LirNodeKind::Value, "alias2");
    let alias_two_binding = builder.alloc_text(LirNodeKind::Value, "alias");
    builder.node_mut(alias_two_declarator).unwrap().children =
        vec![alias_two_name, alias_two_binding];
    builder.node_mut(alias_two_decl).unwrap().children = vec![alias_two_declarator];

    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "ownKeys");
    let object_object = builder.alloc_text(LirNodeKind::Value, "Reflect");
    builder.node_mut(callee).unwrap().children = vec![object_object];
    let alias_ref = builder.alloc_text(LirNodeKind::Value, "alias2");
    builder.node_mut(call).unwrap().children = vec![callee, alias_ref];
    (const_decl, alias_decl, alias_two_decl, call)
}

pub(crate) fn build_const_bound_object_enumeration_call(
    builder: &mut LirBuilder,
    callee_name: &str,
) -> (LirNodeId, LirNodeId) {
    let const_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let declarator = builder.alloc_text(LirNodeKind::Instruction, "point");
    let binding_name = builder.alloc_text(LirNodeKind::Value, "point");
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
    builder.node_mut(declarator).unwrap().children = vec![binding_name, object];
    builder.node_mut(const_decl).unwrap().children = vec![declarator];

    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
    builder.node_mut(callee).unwrap().children = vec![object_object];
    let point_ref = builder.alloc_text(LirNodeKind::Value, "point");
    builder.node_mut(call).unwrap().children = vec![callee, point_ref];
    (const_decl, call)
}

pub(crate) fn build_wrapped_const_bound_object_enumeration_call(
    builder: &mut LirBuilder,
    callee_name: &str,
) -> (LirNodeId, LirNodeId) {
    let (const_decl, call) = build_const_bound_object_enumeration_call(builder, callee_name);
    let object_ref = builder.node_mut(call).unwrap().children[1];
    let wrapper = builder.alloc(LirNodeKind::Value);
    builder.node_mut(wrapper).unwrap().children = vec![object_ref];
    builder.node_mut(call).unwrap().children[1] = wrapper;
    (const_decl, call)
}

pub(crate) fn build_alias_bound_object_enumeration_call(
    builder: &mut LirBuilder,
    callee_name: &str,
) -> (LirNodeId, LirNodeId, LirNodeId, LirNodeId) {
    let const_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let declarator = builder.alloc_text(LirNodeKind::Instruction, "point");
    let binding_name = builder.alloc_text(LirNodeKind::Value, "point");
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
    builder.node_mut(declarator).unwrap().children = vec![binding_name, object];
    builder.node_mut(const_decl).unwrap().children = vec![declarator];

    let alias_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let alias_declarator = builder.alloc_text(LirNodeKind::Instruction, "alias");
    let alias_name = builder.alloc_text(LirNodeKind::Value, "alias");
    let alias_binding = builder.alloc_text(LirNodeKind::Value, "point");
    builder.node_mut(alias_declarator).unwrap().children = vec![alias_name, alias_binding];
    builder.node_mut(alias_decl).unwrap().children = vec![alias_declarator];

    let alias_two_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let alias_two_declarator = builder.alloc_text(LirNodeKind::Instruction, "alias2");
    let alias_two_name = builder.alloc_text(LirNodeKind::Value, "alias2");
    let alias_two_binding = builder.alloc_text(LirNodeKind::Value, "alias");
    builder.node_mut(alias_two_declarator).unwrap().children =
        vec![alias_two_name, alias_two_binding];
    builder.node_mut(alias_two_decl).unwrap().children = vec![alias_two_declarator];

    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
    let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
    builder.node_mut(callee).unwrap().children = vec![object_object];
    let alias_ref = builder.alloc_text(LirNodeKind::Value, "alias2");
    builder.node_mut(call).unwrap().children = vec![callee, alias_ref];
    (const_decl, alias_decl, alias_two_decl, call)
}

pub(crate) fn assert_mixed_bracketed_reflect_own_keys_folds(level: OptimizationLevel) {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_bracketed_reflect_own_keys_call(&mut builder, "ownKeys");
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(level).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}

pub(crate) fn assert_global_this_reflect_bracketed_own_keys_folds(level: OptimizationLevel) {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_global_this_reflect_own_keys_call(&mut builder, r#"["ownKeys"]"#);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(level).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    let values: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(values, vec!["\"1\"", "\"2\"", "b"]);
}
