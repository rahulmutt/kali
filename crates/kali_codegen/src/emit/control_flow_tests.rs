use crate::lower::collect_functions;
use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

#[test]
fn generates_valid_wasm_for_simple_programs() {
    let program = sample_program();
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.add"));
    assert!(printed.contains("call"));
}

#[test]
fn function_plans_are_detected_from_instruction_shape() {
    let program = sample_program();
    let plans = collect_functions(&program);
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].name, "add");
    assert_eq!(plans[0].params, vec!["a", "b"]);
}

#[test]
fn function_plans_preserve_generator_flavor_metadata_for_class_methods() {
    let program = parse_and_lower_lir(
        "class Example { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } }",
    );
    let plans = collect_functions(&program);

    let outer = plans
        .iter()
        .find(|plan| plan.name == "outer")
        .expect("outer function plan");
    let inner = plans
        .iter()
        .find(|plan| plan.name == "inner")
        .expect("inner function plan");
    let plain = plans
        .iter()
        .find(|plan| plan.name == "plain")
        .expect("plain function plan");

    assert_eq!(outer.flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.flavor, Some(FunctionFlavor::Sync));
}

#[test]
fn function_plans_preserve_generator_flavor_metadata_for_class_expressions() {
    let program = parse_and_lower_lir(
        "const Example = class NamedExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } };",
    );
    let plans = collect_functions(&program);

    let named = plans
        .iter()
        .find(|plan| plan.name == "NamedExample")
        .expect("named class expression function plan");
    let outer = plans
        .iter()
        .find(|plan| plan.name == "outer")
        .expect("outer class expression function plan");
    let inner = plans
        .iter()
        .find(|plan| plan.name == "inner")
        .expect("inner class expression function plan");
    let plain = plans
        .iter()
        .find(|plan| plan.name == "plain")
        .expect("plain class expression function plan");

    assert_eq!(named.flavor, None);
    assert_eq!(outer.flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.flavor, Some(FunctionFlavor::Sync));
}

#[test]
fn function_plans_preserve_generator_flavor_metadata_for_default_export_class_expressions() {
    let program = parse_and_lower_lir(
        "export default (class NamedExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } });",
    );
    let plans = collect_functions(&program);

    let named = plans
        .iter()
        .find(|plan| plan.name == "NamedExample")
        .expect("named default-export class expression function plan");
    let outer = plans
        .iter()
        .find(|plan| plan.name == "outer")
        .expect("outer default-export class expression function plan");
    let inner = plans
        .iter()
        .find(|plan| plan.name == "inner")
        .expect("inner default-export class expression function plan");
    let plain = plans
        .iter()
        .find(|plan| plan.name == "plain")
        .expect("plain default-export class expression function plan");

    assert_eq!(named.flavor, None);
    assert_eq!(outer.flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.flavor, Some(FunctionFlavor::Sync));
}

#[test]
fn function_plans_preserve_generator_flavor_metadata_for_default_export_class_declarations() {
    let program = parse_and_lower_lir(
        "export default class NamedDeclExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } }",
    );
    let plans = collect_functions(&program);

    let named = plans
        .iter()
        .find(|plan| plan.name == "NamedDeclExample")
        .expect("named default-export class declaration function plan");
    let outer = plans
        .iter()
        .find(|plan| plan.name == "outer")
        .expect("outer default-export class declaration function plan");
    let inner = plans
        .iter()
        .find(|plan| plan.name == "inner")
        .expect("inner default-export class declaration function plan");
    let plain = plans
        .iter()
        .find(|plan| plan.name == "plain")
        .expect("plain default-export class declaration function plan");

    assert_eq!(named.flavor, None);
    assert_eq!(outer.flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.flavor, Some(FunctionFlavor::Sync));
}

#[test]
fn function_plans_preserve_generator_flavor_metadata_for_default_export_generator_function_declarations(
) {
    let program = parse_and_lower_lir("export default function* main() { yield* []; }\nmain();");
    let plans = collect_functions(&program);

    let main = plans
        .iter()
        .find(|plan| plan.name == "main")
        .expect("default-export generator function plan");

    assert_eq!(main.flavor, Some(FunctionFlavor::Generator));
}

#[test]
fn function_plans_preserve_generator_flavor_metadata_for_default_export_anonymous_generator_function_declarations(
) {
    let program = parse_and_lower_lir("export default function*() { yield* []; }\n");
    let plans = collect_functions(&program);

    let main = plans
        .iter()
        .find(|plan| plan.flavor == Some(FunctionFlavor::Generator))
        .expect("anonymous default-export generator function plan");

    assert!(!main.name.is_empty());
    assert_eq!(main.flavor, Some(FunctionFlavor::Generator));
}

#[test]
fn function_plans_preserve_generator_flavor_metadata_for_default_export_async_generator_function_declarations(
) {
    let program =
        parse_and_lower_lir("export default async function* main() { yield 1; }\nmain();");
    let plans = collect_functions(&program);

    let main = plans
        .iter()
        .find(|plan| plan.name == "main")
        .expect("default-export async generator function plan");

    assert_eq!(main.flavor, Some(FunctionFlavor::AsyncGenerator));
}

#[test]
fn function_plans_preserve_generator_flavor_metadata_for_default_export_anonymous_async_generator_function_declarations(
) {
    let program = parse_and_lower_lir("export default async function*() { yield 1; }\n");
    let plans = collect_functions(&program);

    let main = plans
        .iter()
        .find(|plan| plan.flavor == Some(FunctionFlavor::AsyncGenerator))
        .expect("anonymous default-export async generator function plan");

    assert!(!main.name.is_empty());
    assert_eq!(main.flavor, Some(FunctionFlavor::AsyncGenerator));
}

#[test]
fn boolean_branches_use_the_layout_fast_path() {
    let program = parse_and_lower_lir("if (1 == 1) { 7; } else { 9; }");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty());
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i32.wrap_i64"));
    assert!(!printed.contains("i64.eqz"));
}

#[test]
fn unsupported_generator_function_lowering_reports_feature_unavailable() {
    let program = parse_and_lower_lir("function* main() { yield* []; }\nmain();");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("generator function lowering")
        }),
        "expected an unavailable generator diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn unsupported_async_generator_function_lowering_reports_feature_unavailable() {
    let program = parse_and_lower_lir("async function* main() { yield 1; }\nmain();");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("async-generator function lowering")
        }),
        "expected an unavailable async-generator diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn unsupported_generator_default_export_function_lowering_reports_feature_unavailable() {
    let program = parse_and_lower_lir("export default function* main() { yield* []; }\nmain();");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("generator function lowering")
        }),
        "expected an unavailable generator default-export diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn unsupported_async_generator_default_export_function_lowering_reports_feature_unavailable() {
    let program =
        parse_and_lower_lir("export default async function* main() { yield 1; }\nmain();");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("async-generator function lowering")
        }),
        "expected an unavailable async-generator default-export diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn unsupported_generator_class_method_lowering_reports_feature_unavailable() {
    let program = parse_and_lower_lir("class Example { *main() { yield* []; } }\nnew Example();");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("generator function lowering")
        }),
        "expected an unavailable generator class-method diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn unsupported_async_generator_class_method_lowering_reports_feature_unavailable() {
    let program =
        parse_and_lower_lir("class Example { async *main() { yield 1; } }\nnew Example();");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("async-generator function lowering")
        }),
        "expected an unavailable async-generator class-method diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn mixed_generator_and_async_generator_lowering_reports_feature_unavailable() {
    let program = parse_and_lower_lir(
        "function* main() { yield 1; }\nasync function* nested() { yield 2; }\nmain();\nnested();",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("generator and async-generator function lowering")
        }),
        "expected a combined generator diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn mixed_generator_and_async_generator_class_method_lowering_reports_feature_unavailable() {
    let program = parse_and_lower_lir(
        "class Example { *syncGen() { yield* []; } async *asyncGen() { yield 1; } }\nnew Example();",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("generator and async-generator function lowering")
        }),
        "expected a combined generator class-method diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn mixed_generator_and_async_generator_class_expression_lowering_reports_feature_unavailable() {
    let program = parse_and_lower_lir(
        "const Example = class NamedExample { *syncGen() { yield* []; } async *asyncGen() { yield 1; } };\nnew Example();",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic
                    .message
                    .contains("generator and async-generator function lowering")
        }),
        "expected a combined generator class-expression diagnostic: {:?}",
        result.diagnostics
    );
}

#[test]
fn generator_function_without_yield_still_remains_feature_unavailable() {
    let program = parse_and_lower_lir("function* main() { return 1; }\nmain();");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_error()
                && diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("generator function lowering")
        }),
        "expected an unavailable generator diagnostic: {:?}",
        result.diagnostics
    );
}

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

#[test]
fn mir_backed_pipeline_reduces_legacy_overhead_on_escaping_locals() {
    let current_lir = sample_program();
    let mir = kali_mir::MirProgram {
        root: kali_mir::MirNodeId::new(0),
        nodes: Vec::new(),
        functions: Vec::new(),
    };
    let baseline_lir = legacy_phase1_baseline(&current_lir, &mir);

    let current_trace = current_lir
        .nodes
        .iter()
        .filter_map(|node| node.text.as_deref())
        .collect::<Vec<_>>();
    let baseline_trace = baseline_lir
        .nodes
        .iter()
        .filter_map(|node| node.text.as_deref())
        .collect::<Vec<_>>();

    assert!(!current_trace.contains(&"phase1.alloc"));
    assert!(!current_trace.contains(&"phase1.incref"));
    assert!(!current_trace.contains(&"phase1.decref"));
    assert!(baseline_trace.contains(&"phase1.alloc"));
    assert!(baseline_trace.contains(&"phase1.decref"));

    let (current_bytes, current_instructions) = compile_and_measure(&current_lir);
    let (baseline_bytes, baseline_instructions) = compile_and_measure(&baseline_lir);

    assert!(
        current_bytes.len() < baseline_bytes.len(),
        "MIR-backed pipeline should produce smaller WASM than the legacy baseline"
    );
    assert!(
        current_instructions < baseline_instructions,
        "MIR-backed pipeline should emit fewer instructions than the legacy baseline"
    );
}
