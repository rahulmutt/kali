use super::*;
use kali_ast::{UpdateExpression, UpdateOperator, AST};
use kali_common::FileId;
use kali_lexer::Lexer;
use kali_parser::Parser;

fn parse(source: &str) -> Vec<Statement> {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    parser.parse(None).statements
}

#[test]
fn test_hir_builder() {
    let mut builder = HirBuilder::new();
    let root = builder.alloc(HirNodeKind::Program, None);
    assert_eq!(root.0, 0);
    assert_eq!(builder.next_id.0, 1);
}

#[test]
fn test_lower_statements_to_hir() {
    let statements = parse("const answer = 40 + 2; function add(a, b) { return a + b; }");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    assert!(result.diagnostics.is_empty());
    assert_eq!(
        result.nodes[result.root.0 as usize].kind,
        HirNodeKind::Program
    );
    assert_eq!(result.nodes[result.root.0 as usize].children.len(), 2);
    assert!(result.validate().is_ok());

    let var_decl = &result.nodes[result.nodes[result.root.0 as usize].children[0].0 as usize];
    assert_eq!(var_decl.kind, HirNodeKind::VarDecl);
    assert_eq!(var_decl.text.as_deref(), Some("const"));

    let func_decl = &result.nodes[result.nodes[result.root.0 as usize].children[1].0 as usize];
    assert_eq!(func_decl.kind, HirNodeKind::FunctionDecl);
    assert_eq!(func_decl.text.as_deref(), Some("add"));
}

#[test]
fn test_lower_program_from_ast_matches_statement_lowering_for_empty_ast_shell() {
    let statements = parse("const answer = 40 + 2; function add(a, b) { return a + b; }");
    let mut lowerer = HirLowerer::new();
    let ast = AST::empty();

    let from_ast = lowerer.lower_program_from_ast(&ast, &statements);
    let from_statements = lowerer.lower_statements(&statements);

    assert!(from_ast.diagnostics.is_empty());
    assert_eq!(from_ast.root, from_statements.root);
    assert_eq!(from_ast.nodes, from_statements.nodes);
}

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
fn test_lower_statements_records_function_flavor_metadata_for_class_methods() {
    let statements = parse(
        "class Example { async *outer() { yield 1; } *inner() { yield 2; } plain() { return 0; } }",
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
        "const Example = class NamedExample { async *outer() { yield 1; } *inner() { yield 2; } plain() { return 0; } };",
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
fn test_lower_statements_records_export_all_nodes() {
    let statements = parse("export * from './helper.ts';");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_statements(&statements);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert_eq!(result.nodes[result.root.0 as usize].children.len(), 1);
    let export_decl = &result.nodes[result.nodes[result.root.0 as usize].children[0].0 as usize];
    assert_eq!(export_decl.kind, HirNodeKind::ExportDecl);
    assert_eq!(export_decl.text.as_deref(), Some("./helper.ts"));
}

#[test]
fn test_object_literal_lowers_to_stable_property_shape() {
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_expression(&Expression::ObjectExpression(ObjectExpression {
        properties: vec![ObjectProperty {
            key: PropertyName::Identifier("answer".to_string()),
            value: Expression::Identifier("value".to_string()),
            kind: ObjectPropertyKind::Init,
        }],
    }));

    let root = &lowerer.builder.nodes[result.0 as usize];
    assert_eq!(root.kind, HirNodeKind::ObjectExpr);
    assert_eq!(root.children.len(), 1);

    let property = &lowerer.builder.nodes[root.children[0].0 as usize];
    assert_eq!(property.kind, HirNodeKind::ObjectProperty);
    assert_eq!(property.text.as_deref(), Some("init"));
    assert_eq!(property.children.len(), 2);

    let key = &lowerer.builder.nodes[property.children[0].0 as usize];
    assert_eq!(key.kind, HirNodeKind::Literal);
    assert_eq!(key.text.as_deref(), Some("answer"));

    let value = &lowerer.builder.nodes[property.children[1].0 as usize];
    assert_eq!(value.kind, HirNodeKind::Ident);
    assert_eq!(value.text.as_deref(), Some("value"));
}

#[test]
fn test_numeric_object_property_names_lower_as_string_literals() {
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_expression(&Expression::ObjectExpression(ObjectExpression {
        properties: vec![ObjectProperty {
            key: PropertyName::Number(3.0),
            value: Expression::Identifier("value".to_string()),
            kind: ObjectPropertyKind::Init,
        }],
    }));

    let root = &lowerer.builder.nodes[result.0 as usize];
    assert_eq!(root.kind, HirNodeKind::ObjectExpr);
    assert_eq!(root.children.len(), 1);

    let property = &lowerer.builder.nodes[root.children[0].0 as usize];
    assert_eq!(property.kind, HirNodeKind::ObjectProperty);

    let key = &lowerer.builder.nodes[property.children[0].0 as usize];
    assert_eq!(key.kind, HirNodeKind::Literal);
    assert_eq!(key.text.as_deref(), Some("\"3\""));
}

#[test]
fn test_numeric_object_property_names_lower_from_parsed_source_as_string_literals() {
    let statements = parse("const obj = { 3: value };\n");
    let kali_ast::Statement::VariableDeclaration(vd) = &statements[0] else {
        panic!("Expected VariableDeclaration, got {:?}", statements[0]);
    };
    let init = vd.declarations[0].init.as_ref().expect("initializer");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_expression(init);

    let root = &lowerer.builder.nodes[result.0 as usize];
    assert_eq!(root.kind, HirNodeKind::ObjectExpr);
    assert_eq!(root.children.len(), 1);

    let property = &lowerer.builder.nodes[root.children[0].0 as usize];
    assert_eq!(property.kind, HirNodeKind::ObjectProperty);

    let key = &lowerer.builder.nodes[property.children[0].0 as usize];
    assert_eq!(key.kind, HirNodeKind::Literal);
    assert_eq!(key.text.as_deref(), Some("\"3\""));

    let value = &lowerer.builder.nodes[property.children[1].0 as usize];
    assert_eq!(value.kind, HirNodeKind::Ident);
    assert_eq!(value.text.as_deref(), Some("value"));
}

#[test]
fn test_numeric_object_property_names_lower_negative_zero_as_string_literal_zero() {
    let statements = parse("const obj = { [-0]: value };\n");
    let kali_ast::Statement::VariableDeclaration(vd) = &statements[0] else {
        panic!("Expected VariableDeclaration, got {:?}", statements[0]);
    };
    let init = vd.declarations[0].init.as_ref().expect("initializer");
    let mut lowerer = HirLowerer::new();
    let result = lowerer.lower_expression(init);

    let root = &lowerer.builder.nodes[result.0 as usize];
    assert_eq!(root.kind, HirNodeKind::ObjectExpr);
    assert_eq!(root.children.len(), 1);

    let property = &lowerer.builder.nodes[root.children[0].0 as usize];
    assert_eq!(property.kind, HirNodeKind::ObjectProperty);

    let key = &lowerer.builder.nodes[property.children[0].0 as usize];
    assert_eq!(key.kind, HirNodeKind::Literal);
    assert_eq!(key.text.as_deref(), Some("\"0\""));

    let value = &lowerer.builder.nodes[property.children[1].0 as usize];
    assert_eq!(value.kind, HirNodeKind::Ident);
    assert_eq!(value.text.as_deref(), Some("value"));
}

#[test]
fn test_update_expression_lowers_prefix_and_postfix_forms() {
    let mut lowerer = HirLowerer::new();

    let prefix =
        lowerer.lower_expression(&Expression::UpdateExpression(Box::new(UpdateExpression {
            operator: UpdateOperator::Increment,
            argument: Expression::Identifier("value".to_string()),
            prefix: true,
        })));
    let prefix_node = &lowerer.builder.nodes[prefix.0 as usize];
    assert_eq!(prefix_node.kind, HirNodeKind::UpdateExpr);
    assert_eq!(prefix_node.text.as_deref(), Some("prefix++"));
    assert_eq!(prefix_node.children.len(), 1);
    let prefix_arg = &lowerer.builder.nodes[prefix_node.children[0].0 as usize];
    assert_eq!(prefix_arg.kind, HirNodeKind::Ident);
    assert_eq!(prefix_arg.text.as_deref(), Some("value"));

    let postfix =
        lowerer.lower_expression(&Expression::UpdateExpression(Box::new(UpdateExpression {
            operator: UpdateOperator::Decrement,
            argument: Expression::Identifier("value".to_string()),
            prefix: false,
        })));
    let postfix_node = &lowerer.builder.nodes[postfix.0 as usize];
    assert_eq!(postfix_node.kind, HirNodeKind::UpdateExpr);
    assert_eq!(postfix_node.text.as_deref(), Some("postfix--"));
    assert_eq!(postfix_node.children.len(), 1);
    let postfix_arg = &lowerer.builder.nodes[postfix_node.children[0].0 as usize];
    assert_eq!(postfix_arg.kind, HirNodeKind::Ident);
    assert_eq!(postfix_arg.text.as_deref(), Some("value"));
}

#[test]
fn test_hir_validation_rejects_out_of_bounds_children() {
    let hir = LoweringResult {
        root: HirNodeId::new(0),
        nodes: vec![HirNode {
            kind: HirNodeKind::Program,
            span: None,
            text: None,
            children: vec![HirNodeId::new(1)],
        }],
        function_flavors: Vec::new(),
        diagnostics: Vec::new(),
    };

    let error = hir
        .validate()
        .expect_err("invalid HIR should fail validation");
    assert!(error.contains("HIR"), "error: {error}");
    assert!(error.contains("child node id 1"), "error: {error}");
}
