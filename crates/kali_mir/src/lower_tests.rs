use crate::test_support::*;
use crate::*;
use kali_hir::FunctionFlavor;

#[test]
fn test_mir_lowering_preserves_program_shape() {
    let hir = parse_and_lower_hir("const answer = 40 + 2;");
    let mir = MirLowerer::new().lower_hir_result(&hir);

    assert_eq!(mir.nodes[mir.root.0 as usize].kind, MirNodeKind::Program);
    assert_eq!(mir.nodes[mir.root.0 as usize].children.len(), 1);
    assert_eq!(
        mir.nodes[mir.nodes[mir.root.0 as usize].children[0].0 as usize].kind,
        MirNodeKind::Decl
    );
    assert!(mir.validate().is_ok());
}

#[test]
fn test_mir_lowering_preserves_function_nodes_with_flavor_metadata() {
    let hir =
        parse_and_lower_hir("async function* outer() { yield 1; } function* inner() { yield 2; }");
    let mir = MirLowerer::new().lower_hir_result(&hir);

    let outer = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("outer"))
        .expect("outer MIR node");
    let inner = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("inner"))
        .expect("inner MIR node");

    assert_eq!(outer.function_flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.function_flavor, Some(FunctionFlavor::Generator));
}

#[test]
fn test_mir_lowering_preserves_function_flavor_metadata() {
    let hir =
        parse_and_lower_hir("async function* outer() { yield 1; } function* inner() { yield 2; }");
    let mir = MirLowerer::new().lower_hir_result(&hir);

    let outer = mir.function("outer").expect("outer function");
    let inner = mir.function("inner").expect("inner function");

    assert_eq!(outer.function_flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.function_flavor, Some(FunctionFlavor::Generator));
}

#[test]
fn test_mir_lowering_preserves_function_flavor_metadata_for_function_expressions() {
    let hir = parse_and_lower_hir(
        "const syncExpr = function syncExpr() { return 1; }; const asyncExpr = async function asyncExpr() { return 1; }; const generatorExpr = function* generatorExpr() { yield 1; }; const asyncGeneratorExpr = async function* asyncGeneratorExpr() { yield 1; };",
    );
    let mir = MirLowerer::new().lower_hir_result(&hir);

    let sync = mir.function("syncExpr").expect("sync expression");
    let async_expr = mir.function("asyncExpr").expect("async expression");
    let generator = mir.function("generatorExpr").expect("generator expression");
    let async_generator = mir
        .function("asyncGeneratorExpr")
        .expect("async generator expression");

    assert_eq!(sync.function_flavor, Some(FunctionFlavor::Sync));
    assert_eq!(async_expr.function_flavor, Some(FunctionFlavor::Async));
    assert_eq!(generator.function_flavor, Some(FunctionFlavor::Generator));
    assert_eq!(
        async_generator.function_flavor,
        Some(FunctionFlavor::AsyncGenerator)
    );
}

#[test]
fn test_mir_lowering_preserves_function_flavor_metadata_for_class_methods() {
    let hir = parse_and_lower_hir(
        "class Example { async *outer() { yield 1; } *inner() { yield* other(); } plain() { return 0; } }",
    );
    let mir = MirLowerer::new().lower_hir_result(&hir);

    let outer = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("outer"))
        .expect("outer class method node");
    let inner = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("inner"))
        .expect("inner class method node");
    let plain = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("plain"))
        .expect("plain class method node");

    assert_eq!(outer.function_flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.function_flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.function_flavor, Some(FunctionFlavor::Sync));
}

#[test]
fn test_mir_lowering_preserves_function_flavor_metadata_for_class_expressions() {
    let hir = parse_and_lower_hir(
        "const Example = class NamedExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } };",
    );
    let mir = MirLowerer::new().lower_hir_result(&hir);

    let class_expr = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("NamedExample"))
        .expect("named class expression node");
    assert_eq!(class_expr.function_flavor, None);

    let outer = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("outer"))
        .expect("outer class expression method node");
    let inner = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("inner"))
        .expect("inner class expression method node");
    let plain = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("plain"))
        .expect("plain class expression method node");

    assert_eq!(outer.function_flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.function_flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.function_flavor, Some(FunctionFlavor::Sync));
}

#[test]
fn test_mir_lowering_preserves_function_flavor_metadata_for_default_export_generator_function_declaration(
) {
    let hir = parse_and_lower_hir("export default function* main() { yield 1; }");
    let mir = MirLowerer::new().lower_hir_result(&hir);

    let default_export = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("main"))
        .expect("default-export generator function node");
    assert_eq!(
        default_export.function_flavor,
        Some(FunctionFlavor::Generator)
    );
}

#[test]
fn test_mir_lowering_preserves_function_flavor_metadata_for_default_export_anonymous_generator_function_declaration(
) {
    let hir = parse_and_lower_hir("export default function*() { yield* []; }");
    let mir = MirLowerer::new().lower_hir_result(&hir);

    let default_export = mir
        .nodes
        .iter()
        .find(|node| {
            node.kind == MirNodeKind::Function
                && node.function_flavor == Some(FunctionFlavor::Generator)
        })
        .expect("anonymous default-export generator function node");
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
fn test_mir_lowering_preserves_function_flavor_metadata_for_default_export_class_expressions() {
    let hir = parse_and_lower_hir(
        "export default (class DefaultExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } });",
    );
    let mir = MirLowerer::new().lower_hir_result(&hir);

    let class_expr = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("DefaultExample"))
        .expect("named default-export class expression node");
    assert_eq!(class_expr.function_flavor, None);

    let outer = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("outer"))
        .expect("outer default-export class expression method node");
    let inner = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("inner"))
        .expect("inner default-export class expression method node");
    let plain = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("plain"))
        .expect("plain default-export class expression method node");

    assert_eq!(outer.function_flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.function_flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.function_flavor, Some(FunctionFlavor::Sync));
}

#[test]
fn test_mir_lowering_preserves_function_flavor_metadata_for_default_export_class_declarations() {
    let hir = parse_and_lower_hir(
        "export default class DefaultDeclExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } }",
    );
    let mir = MirLowerer::new().lower_hir_result(&hir);

    let class_decl = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("DefaultDeclExample"))
        .expect("named default-export class declaration node");
    assert_eq!(class_decl.function_flavor, None);

    let outer = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("outer"))
        .expect("outer default-export class declaration method node");
    let inner = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("inner"))
        .expect("inner default-export class declaration method node");
    let plain = mir
        .nodes
        .iter()
        .find(|node| node.text.as_deref() == Some("plain"))
        .expect("plain default-export class declaration method node");

    assert_eq!(outer.function_flavor, Some(FunctionFlavor::AsyncGenerator));
    assert_eq!(inner.function_flavor, Some(FunctionFlavor::Generator));
    assert_eq!(plain.function_flavor, Some(FunctionFlavor::Sync));
}

#[test]
fn test_call_expressions_lower_to_call_nodes() {
    let hir = parse_and_lower_hir("foo(bar, 1);");
    let mir = MirLowerer::new().lower_hir_result(&hir);
    let expr_stmt = &mir.nodes[mir.nodes[mir.root.0 as usize].children[0].0 as usize];
    let call = expr_stmt
        .children
        .iter()
        .map(|child| &mir.nodes[child.0 as usize])
        .find(|node| node.kind == MirNodeKind::Call)
        .expect("call node");
    assert_eq!(call.children.len(), 3);
}

#[test]
fn test_mir_validation_rejects_out_of_bounds_children() {
    let mir = MirProgram {
        root: MirNodeId::new(0),
        nodes: vec![MirNode {
            kind: MirNodeKind::Program,
            text: None,
            children: vec![MirNodeId::new(1)],
            function_flavor: None,
        }],
        functions: Vec::new(),
    };

    let error = mir
        .validate()
        .expect_err("invalid MIR should fail validation");
    assert!(error.contains("MIR"), "error: {error}");
    assert!(error.contains("child node id 1"), "error: {error}");
}
