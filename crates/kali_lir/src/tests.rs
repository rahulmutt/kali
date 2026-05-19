use super::*;
use kali_common::FileId;
use kali_hir::HirLowerer;
use kali_lexer::Lexer;
use kali_mir::MirLowerer;
use kali_parser::Parser;

fn parse_and_lower(source: &str) -> MirProgram {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    let statements = parser.parse(None).statements;
    let mut hir_lowerer = HirLowerer::new();
    let hir = hir_lowerer.lower_statements(&statements);
    MirLowerer::new().lower_hir_result(&hir)
}

#[test]
fn test_lir_lowering_preserves_root() {
    let mir = parse_and_lower("function add(a, b) { return a + b; }");
    let lir = LirLowerer::new().lower_program(&mir);

    assert_eq!(lir.nodes[lir.root.0 as usize].kind, LirNodeKind::Program);
    assert_eq!(lir.nodes[lir.root.0 as usize].children.len(), 1);
    assert!(lir.validate().is_ok());
}

#[test]
fn test_lir_lowering_preserves_child_order_and_text_payloads() {
    let mir = parse_and_lower("const answer = 40 + 2; foo(answer);");
    let lir = LirLowerer::new().lower_program(&mir);
    let root = &lir.nodes[lir.root.0 as usize];

    assert_eq!(root.kind, LirNodeKind::Program);
    assert_eq!(root.children.len(), 2);
    assert!(lir
        .nodes
        .iter()
        .any(|node| node.text.as_deref() == Some("answer")));
    assert!(lir
        .nodes
        .iter()
        .any(|node| node.text.as_deref() == Some("foo")));
}

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
fn test_lir_lowering_preserves_function_flavor_metadata_for_class_methods() {
    let mir = parse_and_lower(
        "class Example { async *outer() { yield 1; } *inner() { yield 2; } plain() { return 0; } }",
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
        "const Example = class NamedExample { async *outer() { yield* other(); } *inner() { yield 2; } plain() { return 0; } };",
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
        "export default (class DefaultExample { async *outer() { yield* other(); } *inner() { yield 2; } plain() { return 0; } });",
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
        "export default class DefaultDeclExample { async *outer() { yield* other(); } *inner() { yield 2; } plain() { return 0; } }",
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

#[test]
fn test_lir_validation_rejects_out_of_bounds_children() {
    let lir = LirProgram {
        root: LirNodeId::new(0),
        nodes: vec![LirNode {
            kind: LirNodeKind::Program,
            text: None,
            children: vec![LirNodeId::new(1)],
            function_flavor: None,
        }],
    };

    let error = lir
        .validate()
        .expect_err("invalid LIR should fail validation");
    assert!(error.contains("LIR"), "error: {error}");
    assert!(error.contains("child node id 1"), "error: {error}");
}
