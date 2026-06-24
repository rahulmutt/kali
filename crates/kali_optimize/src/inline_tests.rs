use crate::test_support::*;
use crate::*;
use kali_lir::{LirBuilder, LirNodeKind};
use std::time::Instant;

#[test]
fn hot_function_profile_data_expands_inlining_budget() {
    let (mut cold_program, call) = build_hot_add_program();
    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut cold_program);
    assert_eq!(cold_program.nodes[call.0 as usize].kind, LirNodeKind::Call);

    let (mut hot_program, call) = build_hot_add_program();
    Optimizer::new(OptimizationLevel::Release)
        .with_profile_data(ProfileData::new(vec![ProfileSample::new(
            ProfileSampleKind::Function,
            "hot_add",
            8,
        )]))
        .optimize_program(&mut hot_program);

    let optimized_call = &hot_program.nodes[call.0 as usize];
    assert_eq!(optimized_call.kind, LirNodeKind::Value);
    assert_eq!(optimized_call.text.as_deref(), Some("+"));
    assert_eq!(
        optimized_call.children.len(),
        2,
        "inlined hot call should expose the expanded expression tree"
    );
}

#[test]
fn hot_branch_profile_data_unlocks_release_identity_simplification() {
    let (mut cold_program, branch) = build_short_circuit_program("&&", "true", "payload");
    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut cold_program);
    assert_eq!(
        cold_program.nodes[branch.0 as usize].text.as_deref(),
        Some("&&")
    );

    let (mut hot_program, branch) = build_short_circuit_program("&&", "true", "payload");
    Optimizer::new(OptimizationLevel::Release)
        .with_profile_data(ProfileData::new(vec![ProfileSample::new(
            ProfileSampleKind::Branch,
            "branch:payload:then",
            8,
        )]))
        .optimize_program(&mut hot_program);

    let optimized_branch = &hot_program.nodes[branch.0 as usize];
    assert_eq!(optimized_branch.kind, LirNodeKind::Value);
    assert_eq!(optimized_branch.text.as_deref(), Some("payload"));
}

#[test]
fn hot_layout_profile_data_unlocks_release_identity_simplification() {
    let (mut cold_program, branch) = build_short_circuit_program("||", "false", "payload");
    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut cold_program);
    assert_eq!(
        cold_program.nodes[branch.0 as usize].text.as_deref(),
        Some("||")
    );

    let (mut hot_program, branch) = build_short_circuit_program("||", "false", "payload");
    Optimizer::new(OptimizationLevel::Release)
        .with_profile_data(ProfileData::new(vec![ProfileSample::new(
            ProfileSampleKind::Layout,
            "layout:payload-shape",
            8,
        )]))
        .optimize_program(&mut hot_program);

    let optimized_branch = &hot_program.nodes[branch.0 as usize];
    assert_eq!(optimized_branch.kind, LirNodeKind::Value);
    assert_eq!(optimized_branch.text.as_deref(), Some("payload"));
}

#[test]
fn profile_guided_optimization_benchmark_tracks_hot_call_site_reduction_on_a_representative_workload(
) {
    fn build_workload(function_name: &str, literals: &[&str]) -> (LirProgram, LirNodeId) {
        let mut builder = LirBuilder::new();
        let root = builder.alloc(LirNodeKind::Program);

        let hot = builder.alloc_text(LirNodeKind::Instruction, function_name);
        let hot_param = builder.alloc_text(LirNodeKind::Value, "value");
        let hot_block = builder.alloc(LirNodeKind::Block);
        let hot_return = builder.alloc_text(LirNodeKind::Instruction, "return");
        let mut hot_expression = hot_param;

        for literal_text in literals {
            let add = builder.alloc_text(LirNodeKind::Value, "+");
            let literal_node = literal(&mut builder, literal_text);
            builder.node_mut(add).unwrap().children = vec![hot_expression, literal_node];
            hot_expression = add;
        }

        builder.node_mut(hot_return).unwrap().children = vec![hot_expression];
        builder.node_mut(hot_block).unwrap().children = vec![hot_return];
        builder.node_mut(hot).unwrap().children = vec![hot_param, hot_block];

        let call = builder.alloc(LirNodeKind::Call);
        let callee = builder.alloc_text(LirNodeKind::Value, function_name);
        let arg = builder.alloc_text(LirNodeKind::Value, "input");
        builder.node_mut(call).unwrap().children = vec![callee, arg];
        builder.node_mut(root).unwrap().children = vec![hot, call];

        (
            LirProgram {
                root,
                nodes: builder.into_nodes(),
            },
            call,
        )
    }

    let bench_iterations = 64usize;

    for (function_name, literals) in [
        ("hot_add", &["1", "2", "3", "4", "5", "6"][..]),
        ("hot_mix", &["1", "2", "3", "4", "5", "6"][..]),
        ("hot_chain", &["1", "2", "3", "4", "5", "6"][..]),
    ] {
        let release_started = Instant::now();
        for _ in 0..bench_iterations {
            let (mut cold_program, _) = build_workload(function_name, literals);
            Optimizer::new(OptimizationLevel::Release).optimize_program(&mut cold_program);
        }
        let release_elapsed = release_started.elapsed();

        let profile_started = Instant::now();
        for _ in 0..bench_iterations {
            let (mut hot_program, _) = build_workload(function_name, literals);
            Optimizer::new(OptimizationLevel::Release)
                .with_profile_data(ProfileData::new(vec![ProfileSample::new(
                    ProfileSampleKind::Function,
                    function_name,
                    8,
                )]))
                .optimize_program(&mut hot_program);
        }
        let profile_elapsed = profile_started.elapsed();

        eprintln!(
            "profile-guided optimization benchmark for {function_name}: release={}µs profiled={}µs",
            release_elapsed.as_micros(),
            profile_elapsed.as_micros()
        );

        let (mut cold_program, cold_call) = build_workload(function_name, literals);
        Optimizer::new(OptimizationLevel::Release).optimize_program(&mut cold_program);
        assert_eq!(
            cold_program.nodes[cold_call.0 as usize].kind,
            LirNodeKind::Call,
            "baseline release should keep the representative {function_name} call site"
        );

        let (mut hot_program, hot_call) = build_workload(function_name, literals);
        Optimizer::new(OptimizationLevel::Release)
            .with_profile_data(ProfileData::new(vec![ProfileSample::new(
                ProfileSampleKind::Function,
                function_name,
                8,
            )]))
            .optimize_program(&mut hot_program);

        let optimized_call = &hot_program.nodes[hot_call.0 as usize];
        assert_eq!(
            optimized_call.kind,
            LirNodeKind::Value,
            "profile-guided build should inline the hot {function_name} workload"
        );
        assert_eq!(optimized_call.text.as_deref(), Some("+"));
    }
}

#[test]
fn release_inlines_simple_function_calls() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let function = builder.alloc_text(LirNodeKind::Instruction, "add_one");
    let param = builder.alloc_text(LirNodeKind::Value, "x");
    let block = builder.alloc(LirNodeKind::Block);
    let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let expr = builder.alloc_text(LirNodeKind::Value, "+");
    let one = literal(&mut builder, "1");
    let arg = literal(&mut builder, "2");
    builder.node_mut(expr).unwrap().children = vec![param, one];
    builder.node_mut(ret).unwrap().children = vec![expr];
    builder.node_mut(block).unwrap().children = vec![ret];
    builder.node_mut(function).unwrap().children = vec![param, block];
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "add_one");
    builder.node_mut(call).unwrap().children = vec![callee, arg];
    builder.node_mut(root).unwrap().children = vec![function, call];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::Release).optimize_program(&mut program);

    let node = &program.nodes[call.0 as usize];
    assert_eq!(node.kind, LirNodeKind::Literal);
    assert_eq!(node.text.as_deref(), Some("3"));
}

#[test]
fn release_advanced_prunes_dead_inlined_functions() {
    let mut builder = LirBuilder::new();
    let root = builder.alloc(LirNodeKind::Program);
    let function = builder.alloc_text(LirNodeKind::Instruction, "add_one");
    let param = builder.alloc_text(LirNodeKind::Value, "x");
    let block = builder.alloc(LirNodeKind::Block);
    let ret = builder.alloc_text(LirNodeKind::Instruction, "return");
    let expr = builder.alloc_text(LirNodeKind::Value, "+");
    let one = literal(&mut builder, "1");
    let arg = literal(&mut builder, "2");
    builder.node_mut(expr).unwrap().children = vec![param, one];
    builder.node_mut(ret).unwrap().children = vec![expr];
    builder.node_mut(block).unwrap().children = vec![ret];
    builder.node_mut(function).unwrap().children = vec![param, block];
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "add_one");
    builder.node_mut(call).unwrap().children = vec![callee, arg];
    builder.node_mut(root).unwrap().children = vec![function, call];
    let mut program = LirProgram {
        root,
        nodes: builder.into_nodes(),
    };

    Optimizer::new(OptimizationLevel::ReleaseAdvanced).optimize_program(&mut program);

    let node = &program.nodes[call.0 as usize];
    assert_eq!(node.kind, LirNodeKind::Literal);
    assert_eq!(node.text.as_deref(), Some("3"));
    assert_eq!(program.nodes[root.0 as usize].children, vec![call]);
}
