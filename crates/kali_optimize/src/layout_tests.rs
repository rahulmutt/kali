use crate::test_support::*;
use crate::*;
use kali_lir::{LirBuilder, LirNodeKind};

#[test]
fn release_specializes_const_object_property_access() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let const_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let declarator = builder.alloc_text(LirNodeKind::Instruction, "point");
    let binding_name = builder.alloc_text(LirNodeKind::Value, "point");
    let object = builder.alloc(LirNodeKind::Value);
    let prop_x = builder.alloc_text(LirNodeKind::Value, "init");
    let key_x = literal(&mut builder, "x");
    let value_x = literal(&mut builder, "1");
    let prop_y = builder.alloc_text(LirNodeKind::Value, "init");
    let key_y = literal(&mut builder, "y");
    let value_y = literal(&mut builder, "2");
    let access = builder.alloc_text(LirNodeKind::Value, "y");
    let point_ref = builder.alloc_text(LirNodeKind::Value, "point");

    builder.node_mut(prop_x).unwrap().children = vec![key_x, value_x];
    builder.node_mut(prop_y).unwrap().children = vec![key_y, value_y];
    builder.node_mut(object).unwrap().children = vec![prop_x, prop_y];
    builder.node_mut(declarator).unwrap().children = vec![binding_name, object];
    builder.node_mut(const_decl).unwrap().children = vec![declarator];
    builder.node_mut(access).unwrap().children = vec![point_ref];
    builder.node_mut(root).unwrap().children = vec![const_decl, access];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let node = &program.nodes[access.0 as usize];
    assert_eq!(node.kind, LirNodeKind::Literal);
    assert_eq!(node.text.as_deref(), Some("2"));
}

#[test]
fn release_specializes_object_literal_property_order_canonicalization() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let function = builder.alloc_text(LirNodeKind::Instruction, "consume_point");
    let param_point = builder.alloc_text(LirNodeKind::Value, "point");
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
    builder.node_mut(add1).unwrap().children = vec![param_point, one];
    builder.node_mut(add2).unwrap().children = vec![add1, two];
    builder.node_mut(add3).unwrap().children = vec![add2, three];
    builder.node_mut(add4).unwrap().children = vec![add3, four];
    builder.node_mut(add5).unwrap().children = vec![add4, five];
    builder.node_mut(add6).unwrap().children = vec![add5, six];
    builder.node_mut(add7).unwrap().children = vec![add6, seven];
    builder.node_mut(add8).unwrap().children = vec![add7, eight];
    builder.node_mut(ret).unwrap().children = vec![add8];
    builder.node_mut(block).unwrap().children = vec![ret];
    builder.node_mut(function).unwrap().children = vec![param_point, block];

    let call_a = builder.alloc(LirNodeKind::Call);
    let callee_a = builder.alloc_text(LirNodeKind::Value, "consume_point");
    let object_a = builder.alloc(LirNodeKind::Value);
    let object_a_x = builder.alloc_text(LirNodeKind::Value, "init");
    let object_a_x_key = literal(&mut builder, "x");
    let object_a_x_value = literal(&mut builder, "1");
    let object_a_y = builder.alloc_text(LirNodeKind::Value, "init");
    let object_a_y_key = literal(&mut builder, "y");
    let object_a_y_value = literal(&mut builder, "2");
    builder.node_mut(object_a_x).unwrap().children = vec![object_a_x_key, object_a_x_value];
    builder.node_mut(object_a_y).unwrap().children = vec![object_a_y_key, object_a_y_value];
    builder.node_mut(object_a).unwrap().children = vec![object_a_x, object_a_y];
    builder.node_mut(call_a).unwrap().children = vec![callee_a, object_a];

    let call_b = builder.alloc(LirNodeKind::Call);
    let callee_b = builder.alloc_text(LirNodeKind::Value, "consume_point");
    let object_b = builder.alloc(LirNodeKind::Value);
    let object_b_y = builder.alloc_text(LirNodeKind::Value, "init");
    let object_b_y_key = literal(&mut builder, "y");
    let object_b_y_value = literal(&mut builder, "2");
    let object_b_x = builder.alloc_text(LirNodeKind::Value, "init");
    let object_b_x_key = literal(&mut builder, "x");
    let object_b_x_value = literal(&mut builder, "1");
    builder.node_mut(object_b_y).unwrap().children = vec![object_b_y_key, object_b_y_value];
    builder.node_mut(object_b_x).unwrap().children = vec![object_b_x_key, object_b_x_value];
    builder.node_mut(object_b).unwrap().children = vec![object_b_y, object_b_x];
    builder.node_mut(call_b).unwrap().children = vec![callee_b, object_b];

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
                name: Some("consume_point".to_string()),
                kind: kali_mir::MirFunctionKind::Function,
                function_flavor: None,
                bindings: vec![kali_mir::MirBinding {
                    name: "point".to_string(),
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
                }],
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

    assert_eq!(specialized_name_a, specialized_name_b);
    assert!(specialized_name_a.starts_with("consume_point$spec$"));

    let specialized_count = program
        .nodes
        .iter()
        .filter(|node| {
            node.kind == LirNodeKind::Instruction
                && node.text.as_deref() == Some(specialized_name_a)
        })
        .count();
    assert_eq!(specialized_count, 1);
}

#[test]
fn release_specializes_const_array_element_access() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let index_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let index_binding = builder.alloc_text(LirNodeKind::Instruction, "index");
    let index_name = builder.alloc_text(LirNodeKind::Value, "index");
    let index_value = literal(&mut builder, "1");
    builder.node_mut(index_binding).unwrap().children = vec![index_name, index_value];
    builder.node_mut(index_decl).unwrap().children = vec![index_binding];

    let bag_decl = builder.alloc_text(LirNodeKind::Instruction, "const");
    let bag_binding = builder.alloc_text(LirNodeKind::Instruction, "bag");
    let bag_name = builder.alloc_text(LirNodeKind::Value, "bag");
    let array = builder.alloc(LirNodeKind::Value);
    let first = literal(&mut builder, "10");
    let second = literal(&mut builder, "20");
    builder.node_mut(array).unwrap().children = vec![first, second];
    builder.node_mut(bag_binding).unwrap().children = vec![bag_name, array];
    builder.node_mut(bag_decl).unwrap().children = vec![bag_binding];

    let access = builder.alloc_text(LirNodeKind::Value, "index");
    let bag_ref = builder.alloc_text(LirNodeKind::Value, "bag");
    builder.node_mut(access).unwrap().children = vec![bag_ref];

    builder.node_mut(root).unwrap().children = vec![index_decl, bag_decl, access];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let node = &program.nodes[access.0 as usize];
    assert_eq!(node.kind, LirNodeKind::Literal);
    assert_eq!(node.text.as_deref(), Some("20"));
}

#[test]
fn array_literal_accepts_a_node_whose_children_are_all_literals() {
    let mut builder = LirBuilder::new();
    let array = builder.alloc(LirNodeKind::Value);
    let one = literal(&mut builder, "1");
    let two = literal(&mut builder, "2");
    builder.node_mut(array).unwrap().children = vec![one, two];

    let program = LirProgram {
        root: array,
        nodes: builder.into_nodes(),
    };

    assert!(Optimizer::new(OptimizationLevel::Release).is_array_literal(&program, array));
}

#[test]
fn array_literal_rejects_a_wrapper_around_a_call() {
    // The shape of a lowered `new Array(n).fill(v)` declarator initializer: a
    // text-less `Value` wrapping a `Call`. Under the pre-2026-09-10 definition
    // ("text-less Value that is not an object literal") this returned true, so
    // the binding entered the spec env and every read of the name was
    // overwritten with a clone of this node -- destroying the array identity.
    let mut builder = LirBuilder::new();
    let wrapper = builder.alloc(LirNodeKind::Value);
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "Array");
    let arg = literal(&mut builder, "4");
    builder.node_mut(call).unwrap().children = vec![callee, arg];
    builder.node_mut(wrapper).unwrap().children = vec![call];

    let program = LirProgram {
        root: wrapper,
        nodes: builder.into_nodes(),
    };

    assert!(!Optimizer::new(OptimizationLevel::Release).is_array_literal(&program, wrapper));
}

#[test]
fn array_literal_rejects_an_empty_text_less_value() {
    // A text-less `Value` with no children carries no elements to materialize.
    // Declining costs an optimization; accepting invents an empty array.
    let mut builder = LirBuilder::new();
    let empty = builder.alloc(LirNodeKind::Value);

    let program = LirProgram {
        root: empty,
        nodes: builder.into_nodes(),
    };

    assert!(!Optimizer::new(OptimizationLevel::Release).is_array_literal(&program, empty));
}

#[test]
fn array_literal_accepts_a_nested_array_literal() {
    let mut builder = LirBuilder::new();
    let outer = builder.alloc(LirNodeKind::Value);
    let inner = builder.alloc(LirNodeKind::Value);
    let one = literal(&mut builder, "1");
    let two = literal(&mut builder, "2");
    builder.node_mut(inner).unwrap().children = vec![one];
    builder.node_mut(outer).unwrap().children = vec![inner, two];

    let program = LirProgram {
        root: outer,
        nodes: builder.into_nodes(),
    };

    assert!(Optimizer::new(OptimizationLevel::Release).is_array_literal(&program, outer));
}

#[test]
fn array_literal_accepts_a_value_node_carrying_literal_text() {
    // Covers `is_materializable_element`'s arm 2: a `Value` node with no
    // children whose text parses as a literal (e.g. lowering's numeric-text
    // `Value` shape), distinct from a `LirNodeKind::Literal` node.
    let mut builder = LirBuilder::new();
    let array = builder.alloc(LirNodeKind::Value);
    let value_literal = builder.alloc_text(LirNodeKind::Value, "1");
    builder.node_mut(array).unwrap().children = vec![value_literal];

    let program = LirProgram {
        root: array,
        nodes: builder.into_nodes(),
    };

    assert!(Optimizer::new(OptimizationLevel::Release).is_array_literal(&program, array));
}

#[test]
fn array_literal_rejects_an_identifier_element() {
    // `[x]` -- an identifier read, not a value. Under the pre-2026-09-10
    // predicate this qualified (any text-less Value that isn't an object
    // literal admitted its parent regardless of what the element itself
    // was); now an identifier element is not materializable, so the whole
    // array declines. This is the widest new rejection and the one most
    // likely to change what real programs compile to.
    let mut builder = LirBuilder::new();
    let array = builder.alloc(LirNodeKind::Value);
    let x = builder.alloc_text(LirNodeKind::Value, "x");
    builder.node_mut(array).unwrap().children = vec![x];

    let program = LirProgram {
        root: array,
        nodes: builder.into_nodes(),
    };

    assert!(!Optimizer::new(OptimizationLevel::Release).is_array_literal(&program, array));
}
