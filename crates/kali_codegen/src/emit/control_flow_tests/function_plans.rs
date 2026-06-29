use super::*;

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
