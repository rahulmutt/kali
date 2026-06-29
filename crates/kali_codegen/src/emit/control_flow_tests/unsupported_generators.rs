use super::*;

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
