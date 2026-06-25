use crate::test_support::parse;
use crate::*;

#[test]
fn test_lower_statements_records_function_flavor_metadata() {
    let statements = parse("async function* outer() { yield 1; } function* inner() { yield 2; }");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    let outer = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("outer")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("outer function node");
    let inner = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("inner")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("inner function node");

    assert_eq!(
        result.function_flavor(outer),
        Some(FunctionFlavor::AsyncGenerator)
    );
    assert_eq!(
        result.function_flavor(inner),
        Some(FunctionFlavor::Generator)
    );
}

#[test]
fn test_lower_statements_records_function_flavor_metadata_for_function_expressions() {
    let statements = parse("const syncExpr = function syncExpr() { return 1; }; const asyncExpr = async function asyncExpr() { return 1; }; const generatorExpr = function* generatorExpr() { yield 1; }; const asyncGeneratorExpr = async function* asyncGeneratorExpr() { yield 1; };");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    let sync = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionExpr && node.text.as_deref() == Some("syncExpr")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("sync function expression node");
    let async_expr = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionExpr && node.text.as_deref() == Some("asyncExpr")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("async function expression node");
    let generator = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionExpr && node.text.as_deref() == Some("generatorExpr")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("generator function expression node");
    let async_generator = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionExpr
                && node.text.as_deref() == Some("asyncGeneratorExpr")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("async generator function expression node");

    assert_eq!(result.function_flavor(sync), Some(FunctionFlavor::Sync));
    assert_eq!(
        result.function_flavor(async_expr),
        Some(FunctionFlavor::Async)
    );
    assert_eq!(
        result.function_flavor(generator),
        Some(FunctionFlavor::Generator)
    );
    assert_eq!(
        result.function_flavor(async_generator),
        Some(FunctionFlavor::AsyncGenerator)
    );
}

#[test]
fn test_lower_statements_records_function_flavor_metadata_for_default_export_generator_function_declaration(
) {
    let statements = parse("export default function* main() { yield 1; }");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    let default_export = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("main")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("default-export generator function node");

    assert_eq!(
        result.function_flavor(default_export),
        Some(FunctionFlavor::Generator)
    );
}

#[test]
fn test_lower_statements_records_function_flavor_metadata_for_default_export_async_generator_function_declaration(
) {
    let statements = parse("export default async function* main() { yield 1; }");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    let default_export = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("main")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("default-export async generator function node");

    assert_eq!(
        result.function_flavor(default_export),
        Some(FunctionFlavor::AsyncGenerator)
    );
}

#[test]
fn test_lower_statements_records_function_flavor_metadata_for_default_export_anonymous_async_generator_function_declaration(
) {
    let statements = parse("export default async function*() { yield 1; }");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    let (index, node) = result
        .nodes
        .iter()
        .enumerate()
        .find(|(index, node)| {
            node.kind == HirNodeKind::FunctionDecl
                && result.function_flavor(HirNodeId::new(*index as u32))
                    == Some(FunctionFlavor::AsyncGenerator)
        })
        .expect("anonymous default-export async generator function node");

    assert!(node.text.as_deref().is_some_and(|text| !text.is_empty()));
    assert_eq!(
        result.function_flavor(HirNodeId::new(index as u32)),
        Some(FunctionFlavor::AsyncGenerator)
    );
}

#[test]
fn test_lower_statements_records_function_flavor_metadata_for_default_export_anonymous_generator_function_declaration(
) {
    let statements = parse("export default function*() { yield* []; }");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    let (index, node) = result
        .nodes
        .iter()
        .enumerate()
        .find(|(index, node)| {
            node.kind == HirNodeKind::FunctionDecl
                && result.function_flavor(HirNodeId::new(*index as u32))
                    == Some(FunctionFlavor::Generator)
        })
        .expect("anonymous default-export generator function node");

    assert!(node.text.as_deref().is_some_and(|text| !text.is_empty()));
    assert_eq!(
        result.function_flavor(HirNodeId::new(index as u32)),
        Some(FunctionFlavor::Generator)
    );
}

#[test]
fn test_lower_statements_records_function_flavor_metadata_for_class_methods() {
    let statements = parse(
        "class Example { async *outer() { yield 1; } *inner() { yield* other(); } plain() { return 0; } }",
    );
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    let outer = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("outer")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("async generator class method node");
    let inner = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("inner")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("generator class method node");
    let plain = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("plain")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("plain class method node");

    assert_eq!(
        result.function_flavor(outer),
        Some(FunctionFlavor::AsyncGenerator)
    );
    assert_eq!(
        result.function_flavor(inner),
        Some(FunctionFlavor::Generator)
    );
    assert_eq!(result.function_flavor(plain), Some(FunctionFlavor::Sync));
}

#[test]
fn test_lower_statements_records_function_flavor_metadata_for_class_expressions() {
    let statements = parse(
        "const Example = class NamedExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } };",
    );
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    let class_expr = result
        .nodes
        .iter()
        .find(|node| {
            node.kind == HirNodeKind::ClassExpr && node.text.as_deref() == Some("NamedExample")
        })
        .expect("named class expression node");
    assert_eq!(class_expr.kind, HirNodeKind::ClassExpr);

    let outer = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("outer")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("async generator class expression method node");
    let inner = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("inner")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("generator class expression method node");
    let plain = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("plain")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("plain class expression method node");

    assert_eq!(
        result.function_flavor(outer),
        Some(FunctionFlavor::AsyncGenerator)
    );
    assert_eq!(
        result.function_flavor(inner),
        Some(FunctionFlavor::Generator)
    );
    assert_eq!(result.function_flavor(plain), Some(FunctionFlavor::Sync));
}

#[test]
fn test_lower_statements_records_function_flavor_metadata_for_default_export_class_expressions() {
    let statements = parse(
        "export default (class DefaultExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } });",
    );
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    let class_expr = result
        .nodes
        .iter()
        .find(|node| {
            node.kind == HirNodeKind::ClassExpr && node.text.as_deref() == Some("DefaultExample")
        })
        .expect("named default-export class expression node");
    assert_eq!(class_expr.kind, HirNodeKind::ClassExpr);

    let outer = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("outer")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("async generator default-export class expression method node");
    let inner = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("inner")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("generator default-export class expression method node");
    let plain = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("plain")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("plain default-export class expression method node");

    assert_eq!(
        result.function_flavor(outer),
        Some(FunctionFlavor::AsyncGenerator)
    );
    assert_eq!(
        result.function_flavor(inner),
        Some(FunctionFlavor::Generator)
    );
    assert_eq!(result.function_flavor(plain), Some(FunctionFlavor::Sync));
}

#[test]
fn test_lower_statements_records_function_flavor_metadata_for_default_export_class_declarations() {
    let statements = parse(
        "export default class DefaultDeclExample { async *outer() { yield* other(); } *inner() { yield* other(); } plain() { return 0; } }",
    );
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    let class_decl = result
        .nodes
        .iter()
        .find(|node| {
            node.kind == HirNodeKind::ClassDecl
                && node.text.as_deref() == Some("DefaultDeclExample")
        })
        .expect("named default-export class declaration node");
    assert_eq!(class_decl.kind, HirNodeKind::ClassDecl);

    let outer = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("outer")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("async generator default-export class declaration method node");
    let inner = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("inner")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("generator default-export class declaration method node");
    let plain = result
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| {
            node.kind == HirNodeKind::FunctionDecl && node.text.as_deref() == Some("plain")
        })
        .map(|(index, _)| HirNodeId::new(index as u32))
        .expect("plain default-export class declaration method node");

    assert_eq!(
        result.function_flavor(outer),
        Some(FunctionFlavor::AsyncGenerator)
    );
    assert_eq!(
        result.function_flavor(inner),
        Some(FunctionFlavor::Generator)
    );
    assert_eq!(result.function_flavor(plain), Some(FunctionFlavor::Sync));
}
