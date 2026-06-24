use crate::*;
use crate::test_support::*;
use kali_lir::{LirBuilder, LirNodeKind};

#[test]
fn release_folds_object_keys_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_enumeration_call(&mut builder, "keys");
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());
    let keys: Vec<_> = call_node
        .children
        .iter()
        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
        .collect();
    assert_eq!(keys, vec!["\"1\"", "\"2\"", "b"]);
}

#[test]
fn release_folds_object_entries_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_enumeration_call(&mut builder, "entries");
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());
    let entries: Vec<Vec<_>> = call_node
        .children
        .iter()
        .map(|entry_id| {
            program.nodes[entry_id.0 as usize]
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect()
        })
        .collect();
    assert_eq!(
        entries,
        vec![vec!["\"1\"", "4"], vec!["\"2\"", "2"], vec!["b", "1"]]
    );
}

#[test]
fn release_folds_object_from_entries_calls_over_literal_entry_arrays() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_from_entries_call(&mut builder);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());
    let entries: Vec<(String, String)> = call_node
        .children
        .iter()
        .map(|entry_id| {
            let entry_node = &program.nodes[entry_id.0 as usize];
            let key = program.nodes[entry_node.children[0].0 as usize]
                .text
                .as_deref()
                .unwrap()
                .to_string();
            let value = program.nodes[entry_node.children[1].0 as usize]
                .text
                .as_deref()
                .unwrap()
                .to_string();
            (key, value)
        })
        .collect();
    assert_eq!(
        entries,
        vec![
            ("\"b\"".to_string(), "3".to_string()),
            ("\"a\"".to_string(), "2".to_string())
        ]
    );
}

#[test]
fn release_folds_global_this_object_from_entries_calls_over_literal_entry_arrays() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_global_this_object_from_entries_call(&mut builder);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());
    let entries: Vec<(String, String)> = call_node
        .children
        .iter()
        .map(|entry_id| {
            let entry_node = &program.nodes[entry_id.0 as usize];
            let key = program.nodes[entry_node.children[0].0 as usize]
                .text
                .as_deref()
                .unwrap()
                .to_string();
            let value = program.nodes[entry_node.children[1].0 as usize]
                .text
                .as_deref()
                .unwrap()
                .to_string();
            (key, value)
        })
        .collect();
    assert_eq!(
        entries,
        vec![
            ("\"b\"".to_string(), "3".to_string()),
            ("\"a\"".to_string(), "2".to_string())
        ]
    );
}

#[test]
fn release_folds_object_values_calls_over_literal_object_shapes() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_object_enumeration_call(&mut builder, "values");
    builder.node_mut(root).unwrap().children = vec![call];

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
    assert_eq!(values, vec!["4", "2", "1"]);
}

#[test]
fn release_folds_object_enumeration_calls_over_string_literals() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let keys_call = build_object_string_enumeration_call(&mut builder, "keys", "\"ab\"");
    let values_call = build_object_string_enumeration_call(&mut builder, r#"["values"]"#, "\"ab\"");
    let entries_call =
        build_object_string_enumeration_call(&mut builder, r#"["entries"]"#, "\"ab\"");
    builder.node_mut(root).unwrap().children = vec![keys_call, values_call, entries_call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let keys_node = &program.nodes[keys_call.0 as usize];
    assert_eq!(keys_node.kind, LirNodeKind::Value);
    assert_eq!(
        keys_node
            .children
            .iter()
            .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["\"0\"", "\"1\""]
    );

    let values_node = &program.nodes[values_call.0 as usize];
    assert_eq!(values_node.kind, LirNodeKind::Value);
    assert_eq!(
        values_node
            .children
            .iter()
            .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["\"a\"", "\"b\""]
    );

    let entries_node = &program.nodes[entries_call.0 as usize];
    assert_eq!(entries_node.kind, LirNodeKind::Value);
    let entries = entries_node
        .children
        .iter()
        .map(|id| &program.nodes[id.0 as usize])
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 2);
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(entry.kind, LirNodeKind::Value);
        let pair = entry
            .children
            .iter()
            .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(pair[0], format!("\"{}\"", index));
    }
    assert_eq!(
        entries[0]
            .children
            .iter()
            .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["\"0\"", "\"a\""]
    );
    assert_eq!(
        entries[1]
            .children
            .iter()
            .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["\"1\"", "\"b\""]
    );
}

#[test]
fn release_folds_bracketed_global_this_object_enumeration_calls_over_string_literals() {
    for (callee_name, expected) in [
        (r#"["keys"]"#, vec!["\"0\"", "\"1\""]),
        (r#"["values"]"#, vec!["\"a\"", "\"b\""]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let call = build_bracketed_global_this_object_string_enumeration_call(
            &mut builder,
            callee_name,
            "\"ab\"",
        );
        builder.node_mut(root).unwrap().children = vec![call];

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
        assert_eq!(values, expected);
    }

    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_bracketed_global_this_object_string_enumeration_call(
        &mut builder,
        r#"["entries"]"#,
        "\"ab\"",
    );
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());
    let entries: Vec<Vec<_>> = call_node
        .children
        .iter()
        .map(|entry_id| {
            program.nodes[entry_id.0 as usize]
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect()
        })
        .collect();
    assert_eq!(
        entries,
        vec![vec!["\"0\"", "\"a\""], vec!["\"1\"", "\"b\""]]
    );
}

#[test]
fn release_folds_global_this_object_enumeration_calls_over_string_literals() {
    for (callee_name, expected) in [
        (r#"["keys"]"#, vec!["\"0\"", "\"1\""]),
        (r#"["values"]"#, vec!["\"a\"", "\"b\""]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let call =
            build_global_this_object_string_enumeration_call(&mut builder, callee_name, "\"ab\"");
        builder.node_mut(root).unwrap().children = vec![call];

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
        assert_eq!(values, expected);
    }

    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call =
        build_global_this_object_string_enumeration_call(&mut builder, r#"["entries"]"#, "\"ab\"");
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());
    let entries: Vec<Vec<_>> = call_node
        .children
        .iter()
        .map(|entry_id| {
            program.nodes[entry_id.0 as usize]
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect()
        })
        .collect();
    assert_eq!(
        entries,
        vec![vec!["\"0\"", "\"a\""], vec!["\"1\"", "\"b\""]]
    );
}

#[test]
fn release_advanced_folds_global_this_object_enumeration_calls_over_string_literals() {
    for (callee_name, expected) in [
        (r#"["keys"]"#, vec!["\"0\"", "\"1\""]),
        (r#"["values"]"#, vec!["\"a\"", "\"b\""]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let call =
            build_global_this_object_string_enumeration_call(&mut builder, callee_name, "\"ab\"");
        builder.node_mut(root).unwrap().children = vec![call];

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
        assert_eq!(values, expected);
    }

    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call =
        build_global_this_object_string_enumeration_call(&mut builder, r#"["entries"]"#, "\"ab\"");
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());
    let entries: Vec<Vec<_>> = call_node
        .children
        .iter()
        .map(|entry_id| {
            program.nodes[entry_id.0 as usize]
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect()
        })
        .collect();
    assert_eq!(
        entries,
        vec![vec!["\"0\"", "\"a\""], vec!["\"1\"", "\"b\""]]
    );
}

#[test]
fn release_folds_bracketed_global_this_object_enumeration_calls_over_literal_object_shapes() {
    for (callee_name, expected) in [
        (r#"["keys"]"#, vec!["\"1\"", "\"2\"", "b"]),
        (r#"["values"]"#, vec!["4", "2", "1"]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let call = build_bracketed_global_this_object_enumeration_call(&mut builder, callee_name);
        builder.node_mut(root).unwrap().children = vec![call];

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
        assert_eq!(values, expected);
    }

    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_bracketed_global_this_object_enumeration_call(&mut builder, r#"["entries"]"#);
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());
    let entries: Vec<Vec<_>> = call_node
        .children
        .iter()
        .map(|entry_id| {
            program.nodes[entry_id.0 as usize]
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect()
        })
        .collect();
    assert_eq!(
        entries,
        vec![vec!["\"1\"", "4"], vec!["\"2\"", "2"], vec!["b", "1"]]
    );
}

#[test]
fn release_advanced_folds_object_enumeration_calls_over_string_literals() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let keys_call = build_object_string_enumeration_call(&mut builder, "keys", "\"ab\"");
    let values_call = build_object_string_enumeration_call(&mut builder, r#"["values"]"#, "\"ab\"");
    let entries_call =
        build_object_string_enumeration_call(&mut builder, r#"["entries"]"#, "\"ab\"");
    builder.node_mut(root).unwrap().children = vec![keys_call, values_call, entries_call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let keys_node = &program.nodes[keys_call.0 as usize];
    assert_eq!(keys_node.kind, LirNodeKind::Value);
    assert_eq!(
        keys_node
            .children
            .iter()
            .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["\"0\"", "\"1\""]
    );

    let values_node = &program.nodes[values_call.0 as usize];
    assert_eq!(values_node.kind, LirNodeKind::Value);
    assert_eq!(
        values_node
            .children
            .iter()
            .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["\"a\"", "\"b\""]
    );

    let entries_node = &program.nodes[entries_call.0 as usize];
    assert_eq!(entries_node.kind, LirNodeKind::Value);
    let entries = entries_node
        .children
        .iter()
        .map(|id| &program.nodes[id.0 as usize])
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 2);
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(entry.kind, LirNodeKind::Value);
        let pair = entry
            .children
            .iter()
            .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(pair[0], format!("\"{}\"", index));
    }
    assert_eq!(
        entries[0]
            .children
            .iter()
            .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["\"0\"", "\"a\""]
    );
    assert_eq!(
        entries[1]
            .children
            .iter()
            .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["\"1\"", "\"b\""]
    );
}

#[test]
fn release_advanced_folds_bracketed_global_this_object_enumeration_calls_over_string_literals() {
    for (callee_name, expected) in [
        (r#"["keys"]"#, vec!["\"0\"", "\"1\""]),
        (r#"["values"]"#, vec!["\"a\"", "\"b\""]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let call = build_bracketed_global_this_object_string_enumeration_call(
            &mut builder,
            callee_name,
            "\"ab\"",
        );
        builder.node_mut(root).unwrap().children = vec![call];

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
        assert_eq!(values, expected);
    }

    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let call = build_bracketed_global_this_object_string_enumeration_call(
        &mut builder,
        r#"["entries"]"#,
        "\"ab\"",
    );
    builder.node_mut(root).unwrap().children = vec![call];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let call_node = &program.nodes[call.0 as usize];
    assert_eq!(call_node.kind, LirNodeKind::Value);
    assert!(call_node.text.is_none());
    let entries: Vec<Vec<_>> = call_node
        .children
        .iter()
        .map(|entry_id| {
            program.nodes[entry_id.0 as usize]
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect()
        })
        .collect();
    assert_eq!(
        entries,
        vec![vec!["\"0\"", "\"a\""], vec!["\"1\"", "\"b\""]]
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

#[test]
fn fast_folds_object_enumeration_calls_over_literal_object_shapes() {
    for (callee_name, expected) in [
        ("keys", vec!["\"1\"", "\"2\"", "b"]),
        ("entries", vec!["\"1\"", "4", "\"2\"", "2", "b", "1"]),
        ("values", vec!["4", "2", "1"]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let call = build_object_enumeration_call(&mut builder, callee_name);
        builder.node_mut(root).unwrap().children = vec![call];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::Fast).optimize_program(&mut program);

        let call_node = &program.nodes[call.0 as usize];
        assert_eq!(call_node.kind, LirNodeKind::Value);
        assert!(call_node.text.is_none());

        let actual: Vec<_> = match callee_name {
            "entries" => call_node
                .children
                .iter()
                .flat_map(|entry_id| {
                    program.nodes[entry_id.0 as usize]
                        .children
                        .iter()
                        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                        .collect::<Vec<_>>()
                })
                .collect(),
            _ => call_node
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect(),
        };

        assert_eq!(actual, expected);
    }
}

#[test]
fn release_folds_object_enumeration_calls_over_const_bound_literal_object_shapes() {
    for (callee_name, expected) in [
        ("keys", vec!["\"1\"", "\"2\"", "b"]),
        ("entries", vec!["\"1\"", "4", "\"2\"", "2", "b", "1"]),
        ("values", vec!["4", "2", "1"]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let (const_decl, call) =
            build_const_bound_object_enumeration_call(&mut builder, callee_name);
        builder.node_mut(root).unwrap().children = vec![const_decl, call];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

        let call_node = &program.nodes[call.0 as usize];
        assert_eq!(call_node.kind, LirNodeKind::Value);
        assert!(call_node.text.is_none());

        let actual: Vec<_> = match callee_name {
            "entries" => call_node
                .children
                .iter()
                .flat_map(|entry_id| {
                    program.nodes[entry_id.0 as usize]
                        .children
                        .iter()
                        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                        .collect::<Vec<_>>()
                })
                .collect(),
            _ => call_node
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect(),
        };

        assert_eq!(actual, expected);
    }
}

#[test]
fn release_folds_object_enumeration_calls_over_wrapped_const_bound_literal_object_shapes() {
    for (callee_name, expected) in [
        ("keys", vec!["\"1\"", "\"2\"", "b"]),
        ("entries", vec!["\"1\"", "4", "\"2\"", "2", "b", "1"]),
        ("values", vec!["4", "2", "1"]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let (const_decl, call) =
            build_wrapped_const_bound_object_enumeration_call(&mut builder, callee_name);
        builder.node_mut(root).unwrap().children = vec![const_decl, call];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

        let call_node = &program.nodes[call.0 as usize];
        assert_eq!(call_node.kind, LirNodeKind::Value);
        assert!(call_node.text.is_none());

        let actual: Vec<_> = match callee_name {
            "entries" => call_node
                .children
                .iter()
                .flat_map(|entry_id| {
                    program.nodes[entry_id.0 as usize]
                        .children
                        .iter()
                        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                        .collect::<Vec<_>>()
                })
                .collect(),
            _ => call_node
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect(),
        };

        assert_eq!(actual, expected);
    }
}

#[test]
fn release_folds_object_enumeration_calls_over_const_alias_chains() {
    for (callee_name, expected) in [
        ("keys", vec!["\"1\"", "\"2\"", "b"]),
        ("entries", vec!["\"1\"", "4", "\"2\"", "2", "b", "1"]),
        ("values", vec!["4", "2", "1"]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let (const_decl, alias_decl, alias_two_decl, call) =
            build_alias_bound_object_enumeration_call(&mut builder, callee_name);
        builder.node_mut(root).unwrap().children =
            vec![const_decl, alias_decl, alias_two_decl, call];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

        let call_node = &program.nodes[call.0 as usize];
        assert_eq!(call_node.kind, LirNodeKind::Value);
        assert!(call_node.text.is_none());

        let actual: Vec<_> = match callee_name {
            "entries" => call_node
                .children
                .iter()
                .flat_map(|entry_id| {
                    program.nodes[entry_id.0 as usize]
                        .children
                        .iter()
                        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                        .collect::<Vec<_>>()
                })
                .collect(),
            _ => call_node
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect(),
        };

        assert_eq!(actual, expected);
    }
}

#[test]
fn release_advanced_folds_object_enumeration_calls_over_const_alias_chains() {
    for (callee_name, expected) in [
        ("keys", vec!["\"1\"", "\"2\"", "b"]),
        ("entries", vec!["\"1\"", "4", "\"2\"", "2", "b", "1"]),
        ("values", vec!["4", "2", "1"]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let (const_decl, alias_decl, alias_two_decl, call) =
            build_alias_bound_object_enumeration_call(&mut builder, callee_name);
        builder.node_mut(root).unwrap().children =
            vec![const_decl, alias_decl, alias_two_decl, call];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

        let call_node = &program.nodes[call.0 as usize];
        assert_eq!(call_node.kind, LirNodeKind::Value);
        assert!(call_node.text.is_none());

        let actual: Vec<_> = match callee_name {
            "entries" => call_node
                .children
                .iter()
                .flat_map(|entry_id| {
                    program.nodes[entry_id.0 as usize]
                        .children
                        .iter()
                        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                        .collect::<Vec<_>>()
                })
                .collect(),
            _ => call_node
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect(),
        };

        assert_eq!(actual, expected);
    }
}

#[test]
fn release_advanced_folds_object_enumeration_calls_over_const_bound_literal_object_shapes() {
    for (callee_name, expected) in [
        ("keys", vec!["\"1\"", "\"2\"", "b"]),
        ("entries", vec!["\"1\"", "4", "\"2\"", "2", "b", "1"]),
        ("values", vec!["4", "2", "1"]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let (const_decl, call) =
            build_const_bound_object_enumeration_call(&mut builder, callee_name);
        builder.node_mut(root).unwrap().children = vec![const_decl, call];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

        let call_node = &program.nodes[call.0 as usize];
        assert_eq!(call_node.kind, LirNodeKind::Value);
        assert!(call_node.text.is_none());

        let actual: Vec<_> = match callee_name {
            "entries" => call_node
                .children
                .iter()
                .flat_map(|entry_id| {
                    program.nodes[entry_id.0 as usize]
                        .children
                        .iter()
                        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                        .collect::<Vec<_>>()
                })
                .collect(),
            _ => call_node
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect(),
        };

        assert_eq!(actual, expected);
    }
}

#[test]
fn release_advanced_folds_object_enumeration_calls_over_frozen_literal_object_shapes() {
    for (callee_name, expected) in [
        ("keys", vec!["\"1\"", "\"2\"", "b"]),
        ("entries", vec!["\"1\"", "4", "\"2\"", "2", "b", "1"]),
        ("values", vec!["4", "2", "1"]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let call = builder.alloc(LirNodeKind::Call);
        let callee = builder.alloc_text(LirNodeKind::Value, callee_name);
        let object_object = builder.alloc_text(LirNodeKind::Value, "Object");
        builder.node_mut(callee).unwrap().children = vec![object_object];

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
        builder.node_mut(call).unwrap().children = vec![callee, frozen];
        builder.node_mut(root).unwrap().children = vec![call];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

        let call_node = &program.nodes[call.0 as usize];
        assert_eq!(call_node.kind, LirNodeKind::Value);
        assert!(call_node.text.is_none());

        let actual: Vec<_> = match callee_name {
            "entries" => call_node
                .children
                .iter()
                .flat_map(|entry_id| {
                    program.nodes[entry_id.0 as usize]
                        .children
                        .iter()
                        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                        .collect::<Vec<_>>()
                })
                .collect(),
            _ => call_node
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect(),
        };

        assert_eq!(actual, expected);
    }
}

#[test]
fn release_advanced_folds_object_enumeration_calls_over_literal_object_shapes() {
    for (callee_name, expected) in [
        ("keys", vec!["\"1\"", "\"2\"", "b"]),
        ("entries", vec!["\"1\"", "4", "\"2\"", "2", "b", "1"]),
        ("values", vec!["4", "2", "1"]),
    ] {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);
        let call = build_object_enumeration_call(&mut builder, callee_name);
        builder.node_mut(root).unwrap().children = vec![call];

        let mut program = LirProgram {
            root,
            nodes: builder.into_nodes(),
        };

        Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

        let call_node = &program.nodes[call.0 as usize];
        assert_eq!(call_node.kind, LirNodeKind::Value);
        assert!(call_node.text.is_none());

        let actual: Vec<_> = match callee_name {
            "entries" => call_node
                .children
                .iter()
                .flat_map(|entry_id| {
                    program.nodes[entry_id.0 as usize]
                        .children
                        .iter()
                        .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                        .collect::<Vec<_>>()
                })
                .collect(),
            _ => call_node
                .children
                .iter()
                .map(|id| program.nodes[id.0 as usize].text.as_deref().unwrap())
                .collect(),
        };

        assert_eq!(actual, expected);
    }
}
