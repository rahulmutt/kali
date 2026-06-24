use crate::test_support::*;
use crate::*;
use kali_lir::{LirBuilder, LirNodeKind};

#[test]
fn optimizer_carries_normalized_profile_data() {
    let optimizer =
        Optimizer::new(OptimizationLevel::Release).with_profile_data(ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Function, " hot-path ", 1),
            ProfileSample::new(ProfileSampleKind::Function, "hot-path", 2),
            ProfileSample::new(ProfileSampleKind::Branch, "branch:if-true", 4),
        ]));

    let profile = optimizer.profile_data().expect("profile data");
    assert!(profile.is_current_version());
    assert_eq!(
        profile.samples,
        vec![
            ProfileSample::new(ProfileSampleKind::Function, "hot-path", 3),
            ProfileSample::new(ProfileSampleKind::Branch, "branch:if-true", 4),
        ]
    );
    assert_eq!(profile.hot_function_keys(3), vec!["hot-path".to_string()]);
}

#[test]
fn optimization_report_distinguishes_profile_usage_states() {
    let no_profile = Optimizer::new(OptimizationLevel::Fast).optimization_report();
    assert_eq!(no_profile.level, OptimizationLevel::Fast);
    assert_eq!(no_profile.max_specializations, 16);
    assert!(!no_profile.profile_data_present);
    assert!(!no_profile.profile_data_used_for_inlining);
    assert!(no_profile.hot_function_keys.is_empty());

    let cold_profile = Optimizer::new(OptimizationLevel::Release)
        .with_profile_data(ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Function, "cold-path", 7),
            ProfileSample::new(ProfileSampleKind::Branch, "branch:cold", 7),
            ProfileSample::new(ProfileSampleKind::Layout, "layout:cold", 7),
        ]))
        .optimization_report();
    assert_eq!(cold_profile.level, OptimizationLevel::Release);
    assert!(cold_profile.profile_data_present);
    assert!(!cold_profile.profile_data_used_for_inlining);
    assert!(!cold_profile.profile_data_used_for_branching);
    assert!(!cold_profile.profile_data_used_for_layout_specialization);
    assert!(cold_profile.hot_function_keys.is_empty());
    assert!(cold_profile.hot_branch_keys.is_empty());
    assert!(cold_profile.hot_layout_keys.is_empty());

    let hot_profile = Optimizer::with_max_specializations(OptimizationLevel::ReleaseAdvanced, 32)
        .with_profile_data(ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Function, "beta-hot", 8),
            ProfileSample::new(ProfileSampleKind::Function, "alpha-hot", 9),
            ProfileSample::new(ProfileSampleKind::Function, "beta-hot", 1),
            ProfileSample::new(ProfileSampleKind::Branch, "branch:alpha-hot:then", 8),
            ProfileSample::new(ProfileSampleKind::Layout, "layout:point", 9),
        ]))
        .optimization_report();
    assert_eq!(hot_profile.level, OptimizationLevel::ReleaseAdvanced);
    assert_eq!(hot_profile.max_specializations, 32);
    assert!(hot_profile.profile_data_present);
    assert!(hot_profile.profile_data_used_for_inlining);
    assert!(hot_profile.profile_data_used_for_branching);
    assert!(hot_profile.profile_data_used_for_layout_specialization);
    assert_eq!(
        hot_profile.hot_function_keys,
        vec!["alpha-hot".to_string(), "beta-hot".to_string()]
    );
    assert_eq!(
        hot_profile.hot_branch_keys,
        vec!["branch:alpha-hot:then".to_string()]
    );
    assert_eq!(
        hot_profile.hot_layout_keys,
        vec!["layout:point".to_string()]
    );
}

#[test]
fn specialization_cap_limits_distinct_constant_folds() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let first = builder.alloc_text(LirNodeKind::Value, "+");
    let second = builder.alloc_text(LirNodeKind::Value, "+");
    let first_left = literal(&mut builder, "1");
    let first_right = literal(&mut builder, "2");
    let second_left = literal(&mut builder, "3");
    let second_right = literal(&mut builder, "4");
    builder.node_mut(first).unwrap().children = vec![first_left, first_right];
    builder.node_mut(second).unwrap().children = vec![second_left, second_right];
    builder.node_mut(root).unwrap().children = vec![first, second];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::with_max_specializations(OptimizationLevel::Release, 1)
        .optimize_program(&mut program);

    let first_node = &program.nodes[first.0 as usize];
    let second_node = &program.nodes[second.0 as usize];
    assert_eq!(first_node.kind, LirNodeKind::Literal);
    assert_eq!(first_node.text.as_deref(), Some("3"));
    assert_eq!(second_node.kind, LirNodeKind::Value);
    assert_eq!(second_node.text.as_deref(), Some("+"));
}

#[test]
fn specialization_cap_is_scoped_per_function() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);

    let first_function = builder.alloc_text(LirNodeKind::Instruction, "first");
    let first_param = builder.alloc_text(LirNodeKind::Value, "x");
    let first_block = builder.alloc(LirNodeKind::Block);
    let first_return = builder.alloc_text(LirNodeKind::Instruction, "return");
    let first_expr = builder.alloc_text(LirNodeKind::Value, "+");
    let first_left = literal(&mut builder, "1");
    let first_right = literal(&mut builder, "2");
    builder.node_mut(first_expr).unwrap().children = vec![first_left, first_right];
    builder.node_mut(first_return).unwrap().children = vec![first_expr];
    builder.node_mut(first_block).unwrap().children = vec![first_return];
    builder.node_mut(first_function).unwrap().children = vec![first_param, first_block];

    let second_function = builder.alloc_text(LirNodeKind::Instruction, "second");
    let second_param = builder.alloc_text(LirNodeKind::Value, "y");
    let second_block = builder.alloc(LirNodeKind::Block);
    let second_return = builder.alloc_text(LirNodeKind::Instruction, "return");
    let second_expr = builder.alloc_text(LirNodeKind::Value, "+");
    let second_left = literal(&mut builder, "3");
    let second_right = literal(&mut builder, "4");
    builder.node_mut(second_expr).unwrap().children = vec![second_left, second_right];
    builder.node_mut(second_return).unwrap().children = vec![second_expr];
    builder.node_mut(second_block).unwrap().children = vec![second_return];
    builder.node_mut(second_function).unwrap().children = vec![second_param, second_block];

    builder.node_mut(root).unwrap().children = vec![first_function, second_function];

    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::with_max_specializations(OptimizationLevel::Release, 1)
        .optimize_program(&mut program);

    assert_eq!(
        program.nodes[first_expr.0 as usize].kind,
        LirNodeKind::Literal
    );
    assert_eq!(
        program.nodes[first_expr.0 as usize].text.as_deref(),
        Some("3")
    );
    assert_eq!(
        program.nodes[second_expr.0 as usize].kind,
        LirNodeKind::Literal
    );
    assert_eq!(
        program.nodes[second_expr.0 as usize].text.as_deref(),
        Some("7")
    );
}
