use crate::lower::collect_functions;
use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

fn legacy_phase1_baseline(program: &LirProgram, mir: &kali_mir::MirProgram) -> LirProgram {
    let mut nodes = program.nodes.clone();
    let mut extra_nodes = Vec::new();
    let mut insertions = Vec::new();

    let mut ownership_by_name = std::collections::BTreeMap::new();
    for function in &mir.functions {
        for binding in &function.bindings {
            if binding.kind == kali_mir::MirBindingKind::Local {
                ownership_by_name
                    .entry(binding.name.clone())
                    .or_insert(binding.ownership);
            }
        }
    }

    insertions.push((
        program.root.0 as usize,
        vec!["phase1.alloc", "phase1.decref"],
    ));

    for (index, node) in program.nodes.iter().enumerate() {
        if node.kind != LirNodeKind::Instruction {
            continue;
        }

        let Some(name) = node.text.as_deref() else {
            continue;
        };

        if let Some(last_child) = node.children.last().copied() {
            if program
                .nodes
                .get(last_child.0 as usize)
                .is_some_and(|child| child.kind == LirNodeKind::Block)
            {
                insertions.push((last_child.0 as usize, vec!["phase1.alloc", "phase1.decref"]));
                continue;
            }
        }

        let Some(ownership) = ownership_by_name.get(name).copied() else {
            continue;
        };

        let markers: Vec<&'static str> = match ownership {
            kali_mir::OwnershipClass::OwnedHeap => vec!["phase1.alloc", "phase1.decref"],
            kali_mir::OwnershipClass::SharedHeap => {
                vec!["phase1.alloc", "phase1.incref", "phase1.decref"]
            }
            kali_mir::OwnershipClass::Stack | kali_mir::OwnershipClass::Borrowed => Vec::new(),
        };

        if markers.is_empty() {
            continue;
        }

        insertions.push((index, markers));
    }

    for (index, markers) in insertions {
        let mut synthetic_children = Vec::with_capacity(markers.len());
        for marker in markers {
            let id = LirNodeId((nodes.len() + extra_nodes.len()) as u32);
            extra_nodes.push(LirNode::with_text(LirNodeKind::Literal, marker));
            synthetic_children.push(id);
        }
        nodes[index].children.extend(synthetic_children);
    }

    nodes.extend(extra_nodes);
    LirProgram {
        root: program.root,
        nodes,
    }
}

#[path = "control_flow_tests/function_plans.rs"]
mod function_plans;

#[path = "control_flow_tests/unsupported_generators.rs"]
mod unsupported_generators;

#[path = "control_flow_tests/pipeline_basics.rs"]
mod pipeline_basics;

/// Compiles `source` with `main`'s `b` marked call-bound (array-return
/// project, spec 2026-10-02 §3.3), optionally admitting `f` as
/// array-returning, and returns the E5506 messages codegen raised.
fn call_bound_e5506_messages(source: &str, admit_f: bool) -> Vec<String> {
    let program = parse_and_lower_lir(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.repr_table.set_call_bound_array_binding("main", "b");
    if admit_f {
        ctx.repr_table.set_array_return("f", kali_common::Repr::I64);
    }
    let result = lower_lir_to_wasm(&mut ctx, &program);
    result
        .diagnostics
        .iter()
        .filter(|d| d.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32))
        .map(|d| d.message.clone())
        .collect()
}

#[test]
fn call_bound_registration_requires_a_call_init() {
    let computed = kali_common::computed_member_access_unavailable_message();
    // The call-bound fact alone does not register `b`: without `f` admitted
    // as array-returning, the declarator lane declines and `b[i]` refuses.
    let not_admitted = call_bound_e5506_messages(
        "function f() { return [1, 2, 3]; } function main() { const b = f(); let i = 1; console.log(b[i]); } main();",
        false,
    );
    assert!(
        not_admitted.iter().any(|m| m == computed),
        "{not_admitted:?}"
    );
    // An init that is no longer the call (an optimizer rewrite, or any other
    // expression) does not register `b` either, even with `f` admitted.
    let rewritten = call_bound_e5506_messages(
        "function f() { return [1, 2, 3]; } function main() { let n = 1; const b = n + 1; let i = 1; console.log(b[i]); } main();",
        true,
    );
    assert!(rewritten.iter().any(|m| m == computed), "{rewritten:?}");
    // Control: the call init with `f` admitted registers `b`, so the read
    // compiles.
    let admitted = call_bound_e5506_messages(
        "function f() { return [1, 2, 3]; } function main() { const b = f(); let i = 1; console.log(b[i]); } main();",
        true,
    );
    assert!(admitted.is_empty(), "{admitted:?}");
}

/// Compiles `source`, optionally admitting the first anonymous function
/// (`__kali_fn_0`, HIR's synthetic name when the CLI pre-pass has not run) as
/// array-returning, and returns the E5506 messages codegen raised
/// (anon-array-return spec §3.2).
fn anon_array_return_e5506_messages(source: &str, admit: bool) -> Vec<String> {
    let program = parse_and_lower_lir(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    if admit {
        ctx.repr_table
            .set_array_return("__kali_fn_0", kali_common::Repr::I64);
    }
    let result = lower_lir_to_wasm(&mut ctx, &program);
    result
        .diagnostics
        .iter()
        .filter(|d| d.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32))
        .map(|d| d.message.clone())
        .collect()
}

#[test]
fn anon_alias_direct_index_reads_the_admitted_return() {
    let src = "const f = () => [1, 2, 3]; console.log(f()[0]);";
    let refused = anon_array_return_e5506_messages(src, false);
    assert!(
        refused.iter().any(|m| m.contains("indexed read")),
        "not admitted: backstop 1 refuses ({refused:?})"
    );
    let admitted = anon_array_return_e5506_messages(src, true);
    assert!(admitted.is_empty(), "admitted: {admitted:?}");
}

#[test]
fn anon_iife_length_reads_the_admitted_return() {
    let src = "console.log((() => [1, 2, 3])().length);";
    assert!(!anon_array_return_e5506_messages(src, false).is_empty());
    assert!(anon_array_return_e5506_messages(src, true).is_empty());
}

#[test]
fn anon_let_alias_is_not_resolved() {
    let src = "let f = () => [1, 2, 3]; console.log(f()[0]);";
    assert!(!anon_array_return_e5506_messages(src, true).is_empty());
}
