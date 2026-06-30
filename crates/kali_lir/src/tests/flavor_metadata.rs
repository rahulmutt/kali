use super::*;

#[test]
fn test_lir_lowering_preserves_function_flavor_metadata() {
    let mir =
        parse_and_lower("async function* outer() { yield 1; } function* inner() { yield 2; }");
    let lir = LirLowerer::new().lower_program(&mir);

    let outer = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("outer"))
        .expect("outer lir node");
    let inner = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("inner"))
        .expect("inner lir node");

    assert_eq!(outer.function_flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.function_flavor, Some(FunctionFlavor::Generator));
}

#[test]
fn test_lir_lowering_preserves_function_flavor_metadata_for_default_export_generator_function_declaration(
) {
    let mir = parse_and_lower("export default function* main() { yield 1; }");
    let lir = LirLowerer::new().lower_program(&mir);

    let default_export = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("main"))
        .expect("default-export generator lir node");

    assert_eq!(
        default_export.function_flavor,
        Some(FunctionFlavor::Generator)
    );
}

#[test]
fn test_lir_lowering_preserves_function_flavor_metadata_for_default_export_anonymous_generator_function_declaration(
) {
    let mir = parse_and_lower("export default function*() { yield* []; }");
    let lir = LirLowerer::new().lower_program(&mir);

    let default_export = lir
        .nodes
        .iter()
        .find(|node| {
            node.kind == LirNodeKind::Instruction
                && node.function_flavor == Some(FunctionFlavor::Generator)
        })
        .expect("anonymous default-export generator lir node");

    assert!(default_export
        .text
        .as_deref()
        .is_some_and(|text| !text.is_empty()));
    assert_eq!(
        default_export.function_flavor,
        Some(FunctionFlavor::Generator)
    );
}

#[test]
fn test_lir_lowering_preserves_function_flavor_metadata_for_default_export_async_generator_function_declaration(
) {
    let mir = parse_and_lower("export default async function* main() { yield 1; }");
    let lir = LirLowerer::new().lower_program(&mir);

    let default_export = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("main"))
        .expect("default-export async generator lir node");

    assert_eq!(
        default_export.function_flavor,
        Some(FunctionFlavor::AsyncGenerator)
    );
}

#[test]
fn test_lir_lowering_preserves_function_flavor_metadata_for_class_methods() {
    let mir = parse_and_lower(
        "class Example { async *outer() { yield 1; } *inner() { yield* other(); } plain() { return 0; } }",
    );
    let lir = LirLowerer::new().lower_program(&mir);

    let outer = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("outer"))
        .expect("outer lir node");
    let inner = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("inner"))
        .expect("inner lir node");
    let plain = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("plain"))
        .expect("plain lir node");

    assert_eq!(outer.function_flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.function_flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.function_flavor, Some(FunctionFlavor::Sync));
}

#[test]
fn test_lir_lowering_preserves_function_flavor_metadata_for_class_expressions() {
    let mir = parse_and_lower(
        "const Example = class NamedExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } };",
    );
    let lir = LirLowerer::new().lower_program(&mir);

    let class_expr = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("NamedExample"))
        .expect("named class expression node");
    assert_eq!(class_expr.function_flavor, None);

    let outer = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("outer"))
        .expect("outer lir class expression node");
    let inner = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("inner"))
        .expect("inner lir class expression node");
    let plain = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("plain"))
        .expect("plain lir class expression node");

    assert_eq!(outer.function_flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.function_flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.function_flavor, Some(FunctionFlavor::Sync));
}

#[test]
fn test_lir_lowering_preserves_function_flavor_metadata_for_default_export_class_expressions() {
    let mir = parse_and_lower(
        "export default (class DefaultExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } });",
    );
    let lir = LirLowerer::new().lower_program(&mir);

    let class_expr = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("DefaultExample"))
        .expect("named default-export class expression node");
    assert_eq!(class_expr.function_flavor, None);

    let outer = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("outer"))
        .expect("outer default-export lir class expression node");
    let inner = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("inner"))
        .expect("inner default-export lir class expression node");
    let plain = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("plain"))
        .expect("plain default-export lir class expression node");

    assert_eq!(outer.function_flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.function_flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.function_flavor, Some(FunctionFlavor::Sync));
}

#[test]
fn test_lir_lowering_preserves_function_flavor_metadata_for_default_export_class_declarations() {
    let mir = parse_and_lower(
        "export default class DefaultDeclExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } }",
    );
    let lir = LirLowerer::new().lower_program(&mir);

    let class_decl = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("DefaultDeclExample"))
        .expect("named default-export class declaration node");
    assert_eq!(class_decl.function_flavor, None);

    let outer = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("outer"))
        .expect("outer default-export lir class declaration node");
    let inner = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("inner"))
        .expect("inner default-export lir class declaration node");
    let plain = lir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("plain"))
        .expect("plain default-export lir class declaration node");

    assert_eq!(outer.function_flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.function_flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.function_flavor, Some(FunctionFlavor::Sync));
}
