use crate::lower::collect_functions;
use crate::test_support::*;
use crate::*;
use kali_test_support::fixtures::{tempdir, write_file};
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
fn for_of_object_enumeration_lowers_for_single_quoted_bracketed_aliases_over_frozen_from_entries_operands(
) {
    let program = parse_and_lower_lir(
        "for (const value of [...globalThis['Object']['values'](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(value); } for (const key of [...globalThis['Object']['keys'](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(key); } for (const entry of [...globalThis['Object']['entries'](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_mixed_quote_bracketed_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const entry of [...globalThis[\"Object\"][\"entries\"](Object.freeze(globalThis[\"Object\"]['fromEntries']([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(entry[0]); console.log(entry[1]); } for (const entry of [...globalThis[\"Object\"][\"entries\"](Object.freeze(globalThis['Object'][\"fromEntries\"]([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_await_spread_of_object_enumeration_lowers_for_remaining_global_this_object_spellings_over_frozen_from_entries_operands(
) {
    let program = parse_and_lower_lir(
        "for await (const value of [...globalThis[\"Object\"].values(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(value); } for await (const value of [...globalThis.Object.values(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(value); } for await (const key of [...globalThis.Object[\"keys\"](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(key); } for await (const entry of [...globalThis[\"Object\"][\"entries\"](Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const key of Object.keys(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_bracketed_object_spelling_variants() {
    let program = parse_and_lower_lir(
        "for (const key of Object[\"keys\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(key); } for (const entry of globalThis.Object[\"entries\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_object_freeze_wrapped_object_roots() {
    let program = parse_and_lower_lir(
        "for (const key of Object.freeze(Object).keys({ b: 1, a: 2 })) { console.log(key); } for (const value of Object.freeze(globalThis[\"Object\"]).values({ b: 1, a: 2 })) { console.log(value); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_bracketed_global_this_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const key of Object.keys(globalThis[\"Object\"][\"fromEntries\"]([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(key); } for (const entry of Object.entries(globalThis[\"Object\"][\"fromEntries\"]([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_reflect_own_keys_lowers_for_frozen_static_object_literals() {
    let program = parse_and_lower_lir(
        "const object = Object.freeze({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 }); for (const key of Reflect.ownKeys(object)) { console.log(key); } for await (const key of globalThis[\"Reflect\"][\"ownKeys\"](object)) { console.log(key); } for await (const key of globalThis[\"Reflect\"].ownKeys(object)) { console.log(key); } for await (const key of globalThis[\"Reflect\"][\"ownKeys\"](object)) { console.log(key); } for await (const key of globalThis[\"Reflect\"].ownKeys(object)) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_reflect_own_keys_lowers_for_frozen_static_object_literal_aliases() {
    let program = parse_and_lower_lir(
        "const frozen = Object.freeze({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 }); for (const key of Reflect.ownKeys(frozen)) { console.log(key); } for await (const key of globalThis['Reflect']['ownKeys'](frozen)) { console.log(key); } for await (const key of globalThis['Reflect'].ownKeys(frozen)) { console.log(key); } for await (const key of globalThis[\"Reflect\"][\"ownKeys\"](frozen)) { console.log(key); } for await (const key of globalThis[\"Reflect\"].ownKeys(frozen)) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_mixed_quote_bracket_root_aliases() {
    let program = parse_and_lower_lir(
        r#"const object = Object.freeze({ "b": 1, "2": 2, "a": 3, "1": 4 }); for (const key of globalThis["Object"]['keys'](object)) { console.log(key); } for (const value of globalThis['Object']["values"](object)) { console.log(value); } for (const entry of globalThis["Object"]['entries'](object)) { console.log(entry[0]); console.log(entry[1]); } for (const key of globalThis["Reflect"]['ownKeys'](object)) { console.log(key); } for (const key of globalThis['Reflect']["ownKeys"](object)) { console.log(key); }"#,
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_dot_root_bracket_aliases() {
    let program = parse_and_lower_lir(
        "const object = Object.freeze({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 }); for (const key of globalThis[\"Object\"].keys(object)) { console.log(key); } for (const value of globalThis[\"Object\"].values(object)) { console.log(value); } for (const entry of globalThis[\"Object\"].entries(object)) { console.log(entry[0]); console.log(entry[1]); } for (const key of globalThis['Object'].keys(object)) { console.log(key); } for (const value of globalThis['Object'].values(object)) { console.log(value); } for (const entry of globalThis['Object'].entries(object)) { console.log(entry[0]); console.log(entry[1]); } for (const key of globalThis[\"Reflect\"].ownKeys(object)) { console.log(key); } for (const key of globalThis['Reflect'][\"ownKeys\"](object)) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_values_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const value of Object.values(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(value); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_entries_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const entry of Object.entries(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_mixed_bracket_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const value of [...globalThis.Object[\"values\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(value); } for (const key of [...globalThis.Object[\"keys\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); } for (const key of [...globalThis[\"Object\"].keys(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); } for (const entry of [...globalThis[\"Object\"][\"entries\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_frozen_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const key of Object.keys(Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])))) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_object_enumeration_lowers_for_frozen_static_object_literal_values() {
    let program = parse_and_lower_lir(
        "for (const value of Object.values(Object.freeze({ \"b\": 1, \"a\": 2 }))) { console.log(value); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_of_spread_of_object_enumeration_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for (const key of [...Object.keys(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn for_await_spread_of_object_enumeration_lowers_for_static_object_from_entries_operands() {
    let program = parse_and_lower_lir(
        "for await (const value of [...Object.values(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(value); } for await (const key of [...Object.keys(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"zed\", 1], [\"alpha\", 2], [\"zed\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn bitwise_not_unary_lowers_integer_operands() {
    let program = parse_and_lower_lir("console.log(~1); console.log(~(-2));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.sub"), "{printed}");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn mutable_local_reassignment_keeps_runtime_reads() {
    let program = parse_and_lower_lir("let value = 1; value = 3; console.log(value);");
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
    assert!(printed.contains("local.set"), "{printed}");
    assert!(printed.contains("local.get"), "{printed}");
}

#[test]
fn mutable_local_compound_assignment_accepts_wrapper_targets() {
    let program = parse_and_lower_lir("let value = 1; ((value)) += 2; console.log(value);");
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
    assert!(printed.contains("local.set"), "{printed}");
    assert!(printed.contains("local.get"), "{printed}");
}

#[test]
fn compound_assignment_on_non_local_targets_reports_feature_unavailable() {
    let program = parse_and_lower_lir("let target = { value: 1 }; target.value += 2;");
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
                    .contains("compound assignment lowering is unavailable")
        }),
        "expected a compound-assignment diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn compound_assignment_on_immutable_bindings_reports_feature_unavailable() {
    let program = parse_and_lower_lir("const value = 1; value += 2;");
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
                    .contains("compound assignment lowering is unavailable for binding 'value'")
        }),
        "expected an immutable-binding compound-assignment diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn update_expression_lowering_keeps_prefix_and_postfix_local_reads() {
    let program = parse_and_lower_lir(
        "let value = 1; console.log(++value); console.log(value--); console.log(value);",
    );
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
    assert!(printed.contains("local.set"), "{printed}");
    assert!(printed.contains("local.get"), "{printed}");
    assert!(printed.contains("i64.add"), "{printed}");
    assert!(printed.contains("i64.sub"), "{printed}");
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

#[test]
fn supported_for_of_array_iteration_accepts_static_predicate_filter_calls() {
    let program = parse_and_lower_lir(
        "for (const item of [1, 2, 3].filter((value) => value > 1)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unsupported_array_callback_iteration_lowering_reports_feature_unavailable() {
    for source in [
        "const values = [1, 2]; for (const item of values.find((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.findIndex((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.findLast((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.findLastIndex((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.some((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.every((value) => value > 1)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.reduce((acc, value) => acc + value, 0)) { console.log(item); }",
        "const values = [1, 2]; for (const item of values.reduceRight((acc, value) => acc + value, 0)) { console.log(item); }",
    ] {
        let program = parse_and_lower_lir(source);
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
                        .contains("for-of array iteration lowering is unavailable")
            }),
            "expected an unavailable array-callback diagnostic: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn supported_exponentiation_operator_lowering_is_available_for_integer_literals() {
    let program = parse_and_lower_lir("console.log(2 ** 3);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("call 16"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
    assert!(printed.contains("i64.const 3"), "{printed}");
}

#[test]
fn supported_remainder_operator_lowering_is_available_for_integer_literals() {
    let program = parse_and_lower_lir("console.log(7 % 3);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error()),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.rem_s"), "{printed}");
}

#[test]
fn unsupported_exponentiation_operator_rejects_negative_exponents() {
    let program = parse_and_lower_lir("console.log(2 ** -1);");
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
                && diagnostic.message.contains("negative numeric literals")
        }),
        "expected a negative-exponentiation diagnostic: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_const_alias_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; const values = ([1, (value)]); for (const item of (values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_binding_wrappers() {
    let program =
        parse_and_lower_lir("let item = 0; for ((item) of [1, 2]) { console.log(item); }");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_as_const_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for (const item of ([1, (value)] as const)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_satisfies_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for (const item of ([1, (value)] satisfies readonly [1, 2])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_array_from_wrappers() {
    let program =
        parse_and_lower_lir("for (const item of Array.from([1, 2])) { console.log(item); }");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_nullish_wrapped_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of Object.freeze((null ?? Array.from))([1, 2])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_and_wrapped_bracketed_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of Object.freeze((true && globalThis[\"Array\"].from))([1, 2])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_logical_or_wrapped_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of Object.freeze((false || Array.from))([1, 2])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_nullish_wrapped_fully_bracketed_global_this_array_from_calls(
) {
    let program = parse_and_lower_lir(
        "for (const item of Object.freeze((null ?? globalThis[\"Array\"][\"from\"]))([1, 2])) { console.log(item); }\nfor (const item of Object.freeze((null ?? globalThis['Array']['from']))([1, 2])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_direct_array_from_calls() {
    let program =
        parse_and_lower_lir("for (const item of Array.from([1, 2])) { console.log(item); }");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of globalThis.Array.from([1, 2])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_bracketed_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of globalThis[\"Array\"].from([1, 2])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_single_quoted_bracketed_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of globalThis['Array'].from([1, 2])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_bracketed_global_this_array_bracket_from_calls() {
    let program = parse_and_lower_lir(
        "for (const item of globalThis[\"Array\"][\"from\"]([1, 2])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_single_quoted_bracketed_global_this_array_bracket_from_calls(
) {
    let program = parse_and_lower_lir(
        "for (const item of globalThis['Array']['from']([1, 2])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_single_quoted_array_from_calls() {
    let program =
        parse_and_lower_lir("for (const item of Array['from']([1, 2])) { console.log(item); }");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze(Array.from)(values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_bracketed_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze(globalThis[\"Array\"].from)(values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_bracketed_and_single_quoted_frozen_global_this_array_from_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis[\"Array\"])[\"from\"])(values)) { console.log(item); } for (const item of Object.freeze((globalThis['Array'])['from'])(values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_mixed_quoted_bracket_root_frozen_global_this_array_from_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis[\"Array\"])[\"from\"])(values)) { console.log(item); } for (const item of Object.freeze((globalThis['Array'])['from'])(values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze(globalThis.Array.from)(values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_nullish_and_logical_wrapped_global_this_array_from_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((null ?? globalThis.Array.from))(values)) { console.log(item); } for (const item of Object.freeze((true && globalThis.Array.from))(values)) { console.log(item); } for (const item of Object.freeze((false || globalThis.Array.from))(values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_parenthesized_global_this_array_bracket_from_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis.Array))[\"from\"](values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_bracketed_global_this_array_receiver_freeze_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis[\"Array\"]))[\"from\"](values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_single_quoted_global_this_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze(globalThis['Array']['from'])(values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_single_quoted_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze(Array['from'])(values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_single_quoted_global_this_array_receiver_freeze_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis['Array']).from)(values)) { console.log(item); } for (const item of Object.freeze((globalThis['Array'])[\"from\"])(values)) { console.log(item); } for (const item of Object.freeze((globalThis['Array']))[\"from\"](values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_frozen_parenthesized_global_this_array_single_quoted_from_calls(
) {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((globalThis.Array))['from'](values)) { console.log(item); } for (const item of Object.freeze((globalThis[\"Array\"]))['from'](values)) { console.log(item); } for (const item of Object.freeze((globalThis['Array']))[\"from\"](values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_parenthesized_frozen_array_from_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of Object.freeze((Array.from))(values)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_identity_map_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of values.map((value) => value)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn static_identity_strict_equality_uses_javascript_number_semantics() {
    assert!(
        StaticObjectIdentityValue::Number(0.0).strict_eq(&StaticObjectIdentityValue::Number(-0.0))
    );
    assert!(!StaticObjectIdentityValue::Number(f64::NAN)
        .strict_eq(&StaticObjectIdentityValue::Number(f64::NAN)));
}

#[test]
fn static_identity_same_value_zero_uses_array_includes_number_semantics() {
    assert!(StaticObjectIdentityValue::Number(0.0)
        .same_value_zero(&StaticObjectIdentityValue::Number(-0.0)));
    assert!(StaticObjectIdentityValue::Number(f64::NAN)
        .same_value_zero(&StaticObjectIdentityValue::Number(f64::NAN)));
}

#[test]
fn supported_for_of_array_iteration_accepts_truthy_identity_filter_calls() {
    let program = parse_and_lower_lir(
        "for (const item of [1, 2].filter((value) => value)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_identity_flat_map_calls() {
    let program = parse_and_lower_lir(
        "for (const item of [1, 2].flatMap((value) => [value])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_as_const_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for await (const item of ([1, (value)] as const)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_array_from_wrappers() {
    let program =
        parse_and_lower_lir("for await (const item of Array.from([1, 2])) { console.log(item); }");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_satisfies_wrappers() {
    let program = parse_and_lower_lir(
        "const value = 2; for await (const item of ([1, (value)] satisfies readonly [1, 2])) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_parenthesized_binding_wrappers() {
    let program =
        parse_and_lower_lir("let item = 0; for await ((item) of [1, 2]) { console.log(item); }");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_identity_map_calls() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for await (const item of values.map((value) => value)) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_await_wrappers() {
    let program =
        parse_and_lower_lir("for await (const value of await [1, 2, 3]) { console.log(value); }");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_object_entries_iteration_accepts_static_object_literals() {
    let program = parse_and_lower_lir(
        "for await (const entry of Object.entries({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_object_entries_iteration_accepts_parenthesized_frozen_bracket_root_wrappers()
{
    let program = parse_and_lower_lir(
        "for await (const entry of Object.freeze((globalThis[\"Object\"]))[\"entries\"]({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_object_entries_iteration_accepts_static_object_literals() {
    let program = parse_and_lower_lir(
        "for (const entry of Object.entries({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_object_keys_iteration_accepts_parenthesized_frozen_callable_wrappers() {
    let program = parse_and_lower_lir(
        "for (const key of (Object.freeze(Object.keys))({ \"b\": 1, \"a\": 2 })) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_object_keys_iteration_accepts_parenthesized_global_this_frozen_callable_wrappers(
) {
    let program = parse_and_lower_lir(
        "for (const key of (Object.freeze((globalThis.Object.keys)))({ \"b\": 1, \"a\": 2 })) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_object_keys_iteration_accepts_logical_and_or_and_single_quoted_bracket_root_wrappers(
) {
    let program = parse_and_lower_lir(
        "for await (const key of Object.freeze((true && Object.keys))({ \"b\": 1, \"a\": 2 })) { console.log(key); } for await (const key of Object.freeze((false || Object.keys))({ \"b\": 1, \"a\": 2 })) { console.log(key); } for await (const key of Object.freeze((globalThis['Object'])['keys'])({ \"b\": 1, \"a\": 2 })) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_object_values_iteration_accepts_parenthesized_frozen_callable_wrappers() {
    let program = parse_and_lower_lir(
        "for await (const value of (Object.freeze(Object.values))({ \"b\": 1, \"a\": 2 })) { console.log(value); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_object_entries_iteration_accepts_parenthesized_global_this_frozen_callable_wrappers(
) {
    let program = parse_and_lower_lir(
        "for await (const entry of (Object.freeze((globalThis.Object.entries)))({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_reflect_own_keys_iteration_accepts_static_object_literals() {
    let program = parse_and_lower_lir(
        "for (const key of Reflect.ownKeys({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 })) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_reflect_own_keys_iteration_accepts_static_object_literals() {
    let program = parse_and_lower_lir(
        "for await (const key of Reflect.ownKeys({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 })) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_reflect_own_keys_iteration_accepts_sequence_expression_wrappers() {
    let program = parse_and_lower_lir(
        "for (const key of (0, Reflect.ownKeys({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 }))) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_reflect_own_keys_iteration_accepts_nullish_wrapped_callable_targets() {
    let program = parse_and_lower_lir(
        "for (const key of Object.freeze((null ?? Reflect.ownKeys))({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 })) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_reflect_own_keys_iteration_accepts_logical_wrapped_callable_targets() {
    let program = parse_and_lower_lir(
        "for (const key of Object.freeze((true && Reflect.ownKeys))({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 })) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_reflect_own_keys_iteration_accepts_logical_or_wrapped_callable_targets() {
    let program = parse_and_lower_lir(
        "for (const key of Object.freeze((false || Reflect.ownKeys))({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 })) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_reflect_own_keys_iteration_accepts_sequence_expression_wrappers() {
    let program = parse_and_lower_lir(
        "for await (const key of (0, Reflect.ownKeys({ \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 }))) { console.log(key); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_object_enumeration_accepts_string_literals() {
    let program = parse_and_lower_lir(
        "for (const key of Object.keys('ab')) { console.log(key); } for (const value of Object.values('ab')) { console.log(value); } for (const entry of Object.entries('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_object_entries_string_literals_accept_bracketed_global_this_alias() {
    let program = parse_and_lower_lir(
        "for (const entry of globalThis[\"Object\"][\"entries\"]('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_object_enumeration_accepts_string_literals() {
    let program = parse_and_lower_lir(
        "for await (const key of Object.keys('ab')) { console.log(key); } for await (const value of Object.values('ab')) { console.log(value); } for await (const entry of Object.entries('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_object_enumeration_accepts_nullish_and_logical_wrapped_callable_aliases() {
    let program = parse_and_lower_lir(
        "for await (const key of Object.freeze((null ?? Object.keys))('ab')) { console.log(key); } for await (const value of Object.freeze((true && Object.values))('ab')) { console.log(value); } for await (const entry of Object.freeze((false || Object.entries))('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_object_enumeration_accepts_parenthesized_receiver_wrapped_bracketed_aliases()
{
    let program = parse_and_lower_lir(
        "for await (const key of Object.freeze((globalThis.Object)[\"keys\"])('ab')) { console.log(key); } for await (const value of Object.freeze((globalThis[\"Object\"])[\"values\"])('ab')) { console.log(value); } for await (const entry of Object.freeze((globalThis[\"Object\"])[\"entries\"])('ab')) { console.log(entry[0]); console.log(entry[1]); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_string_concatenation_iteration_accepts_static_string_operands() {
    let program = parse_and_lower_lir(
        "const prefix = 'he'; const suffix = 'llo'; for await (const ch of prefix + suffix) { console.log(ch); } for await (const ch of (`${prefix}${suffix}`)) { console.log(ch); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_spread_of_const_bound_literal_arrays() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for await (const item of [...values]) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_await_array_iteration_accepts_spread_of_parenthesized_const_bound_literal_arrays()
{
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for await (const item of [...(values)]) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_spread_of_const_bound_literal_arrays() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of [...values]) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn supported_for_of_array_iteration_accepts_spread_of_parenthesized_const_bound_literal_arrays() {
    let program = parse_and_lower_lir(
        "const values = [1, 2]; for (const item of [...(values)]) { console.log(item); }",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn nullish_coalescing_lowers_for_supported_input_shapes() {
    assert_nullish_coalescing_lowers("console.log(null ?? 1);");
    assert_nullish_coalescing_lowers("console.log(undefined ?? 1);");
}

#[test]
fn nullish_assignment_lowers_for_wrapped_mutable_local_binding_targets() {
    assert_nullish_assignment_lowers("let value = null; ((value)) ??= 1; console.log(value);");
}

#[test]
fn logical_assignment_lowers_for_wrapped_mutable_local_binding_targets() {
    assert_logical_assignment_lowers("let left = 0; ((left)) ||= 1; console.log(left); let right = 1; ((right)) &&= 2; console.log(right);");
}

#[test]
fn object_enumeration_helper_iteration_lowers_via_frozen_object_entries_call_without_diagnostics() {
    let program = parse_and_lower_lir("for await (const entry of (Object.freeze(Object.entries))({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }");
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
}

#[test]
fn unresolved_identifier_lowering_attaches_a_guidance_note() {
    let mut program = sample_program();
    program.nodes[7].text = Some("missing_value".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.source_path = Some(std::path::PathBuf::from("src/missing_value.ts"));
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_warning()
                && diagnostic.code
                    == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic
                    .message
                    .contains("zero placeholder compatibility fallback")
                && diagnostic.context.as_deref().is_some_and(|context| {
                    context.origin == kali_error::DiagnosticContextOrigin::Source
                        && context.requested_value.as_deref() == Some("missing_value")
                        && context.effective_value.as_deref()
                            == Some("zero placeholder compatibility fallback")
                })
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("fallback emits a zero placeholder"))
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("source path: "))
        }),
        "expected an unresolved-identifier diagnostic on the lowering path: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn unresolved_call_target_lowering_attaches_a_guidance_note() {
    let mut program = sample_program();
    program.nodes[10].text = Some("missing_fn".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.source_path = Some(std::path::PathBuf::from("src/missing_fn.ts"));
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.is_warning()
                && diagnostic.code
                    == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic
                    .message
                    .contains("zero placeholder compatibility fallback")
                && diagnostic.context.as_deref().is_some_and(|context| {
                    context.origin == kali_error::DiagnosticContextOrigin::Source
                        && context.requested_value.as_deref() == Some("missing_fn")
                        && context.effective_value.as_deref()
                            == Some("zero placeholder compatibility fallback")
                })
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("fallback emits a zero placeholder"))
                && diagnostic
                    .notes
                    .iter()
                    .any(|note| note.contains("source path: "))
        }),
        "expected an unresolved-call diagnostic on the lowering path: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn duplicate_unresolved_identifier_lowering_reports_one_guidance_note() {
    let mut program = sample_program();
    program.nodes[7].text = Some("missing_value".to_string());
    program.nodes[8].text = Some("missing_value".to_string());

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    let matching_diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == Some(kali_error::_error_codes::e3::UNDEFINED_IDENTIFIER as u32)
                && diagnostic.message.contains("missing_value")
        })
        .count();
    assert_eq!(
        matching_diagnostics, 1,
        "expected the repeated unresolved identifier to be reported once: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
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

#[test]
fn source_path_in_temp_dir_attaches_to_unresolved_identifier_diagnostics() {
    // Use kali_test_support::fixtures to create a real temp directory with a
    // source file alongside a package.json; verify the source_path is threaded
    // through to diagnostics so downstream tooling knows where the file lives.
    let dir = tempdir();
    let src = write_file(dir.path(), "index.ts", "missing_var;");
    let _pkg = write_file(
        dir.path(),
        "package.json",
        r#"{"name":"smoke","version":"1.2.3"}"#,
    );

    let program = parse_and_lower_lir("missing_var;");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.source_path = Some(src.clone());
    let result = lower_lir_to_wasm(&mut ctx, &program);

    // The unresolved identifier should produce a warning that embeds the
    // source path from the temp directory.
    let src_str = src.to_string_lossy();
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| { d.notes.iter().any(|note| note.contains(src_str.as_ref())) }),
        "expected a diagnostic containing the tempdir source path {:?}; got: {:?}",
        src,
        result.diagnostics
    );
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}
