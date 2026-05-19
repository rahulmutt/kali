use super::*;

#[test]
fn test_node_id() {
    let id = NodeId::new(42);
    assert_eq!(id.as_u32(), 42);
    assert_eq!(id.to_string(), "n42");
}

#[test]
fn test_ast_builder() {
    let mut builder = ASTBuilder::new();
    let root_id = builder.new_node(NodeKind::Program, None);
    builder.set_root(root_id);

    let root = builder.get_node(root_id).unwrap();
    assert_eq!(root.kind, NodeKind::Program);

    assert!(builder.root().is_some());
}

#[test]
fn test_ast_conversion() {
    let mut builder = ASTBuilder::new();
    let root_id = builder.new_node(NodeKind::Program, None);
    builder.set_root(root_id);

    let ast: AST = builder.into();
    assert!(ast.root().is_some());
}

#[test]
fn test_function_kind_metadata_survives_serde_roundtrip() {
    let function = FunctionDeclaration {
        name: "generatorFn".to_string(),
        params: vec!["value".to_string()],
        body: Box::new(BlockStatement { body: vec![] }),
        is_async: true,
        generator: true,
    };
    let function_expr = FunctionExpression {
        id: Some("asyncGeneratorExpr".to_string()),
        params: vec![FunctionParam {
            name: "value".to_string(),
        }],
        body: Some(Box::new(BlockStatement { body: vec![] })),
        is_async: true,
        generator: true,
    };
    let class = ClassExpression {
        id: Some("Example".to_string()),
        body: Box::new(ClassBody {
            methods: vec![
                MethodDefinition {
                    name: "outer".to_string(),
                    params: vec!["value".to_string()],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: true,
                    generator: true,
                },
                MethodDefinition {
                    name: "inner".to_string(),
                    params: vec![],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: false,
                    generator: true,
                },
            ],
        }),
    };
    let class_decl = ClassDeclaration {
        name: "DeclExample".to_string(),
        body: Box::new(ClassBody {
            methods: vec![
                MethodDefinition {
                    name: "outer".to_string(),
                    params: vec!["value".to_string()],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: true,
                    generator: true,
                },
                MethodDefinition {
                    name: "inner".to_string(),
                    params: vec![],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: false,
                    generator: true,
                },
            ],
        }),
    };
    let default_export_class = Statement::ExportDefault(ExportDefaultDeclaration::Expression(
        Expression::ClassExpression(Box::new(ClassExpression {
            id: Some("DefaultExample".to_string()),
            body: Box::new(ClassBody {
                methods: vec![
                    MethodDefinition {
                        name: "outer".to_string(),
                        params: vec![],
                        body: Some(Box::new(BlockStatement { body: vec![] })),
                        is_async: true,
                        generator: true,
                    },
                    MethodDefinition {
                        name: "inner".to_string(),
                        params: vec![],
                        body: Some(Box::new(BlockStatement { body: vec![] })),
                        is_async: false,
                        generator: true,
                    },
                ],
            }),
        })),
    ));
    let default_export_class_decl = Statement::ExportDefault(
        ExportDefaultDeclaration::ClassDeclaration(ClassDeclaration {
            name: "DefaultDeclExample".to_string(),
            body: Box::new(ClassBody {
                methods: vec![
                    MethodDefinition {
                        name: "outer".to_string(),
                        params: vec![],
                        body: Some(Box::new(BlockStatement { body: vec![] })),
                        is_async: true,
                        generator: true,
                    },
                    MethodDefinition {
                        name: "inner".to_string(),
                        params: vec![],
                        body: Some(Box::new(BlockStatement { body: vec![] })),
                        is_async: false,
                        generator: true,
                    },
                ],
            }),
        }),
    );

    let round_tripped_function: FunctionDeclaration =
        serde_json::from_str(&serde_json::to_string(&function).unwrap()).unwrap();
    let round_tripped_function_expr: FunctionExpression =
        serde_json::from_str(&serde_json::to_string(&function_expr).unwrap()).unwrap();
    let round_tripped_class: ClassExpression =
        serde_json::from_str(&serde_json::to_string(&class).unwrap()).unwrap();
    let round_tripped_class_decl: ClassDeclaration =
        serde_json::from_str(&serde_json::to_string(&class_decl).unwrap()).unwrap();
    let round_tripped_default_export_class: Statement =
        serde_json::from_str(&serde_json::to_string(&default_export_class).unwrap()).unwrap();
    let round_tripped_default_export_class_decl: Statement =
        serde_json::from_str(&serde_json::to_string(&default_export_class_decl).unwrap()).unwrap();

    assert_eq!(round_tripped_function, function);
    assert_eq!(round_tripped_function_expr, function_expr);
    assert_eq!(round_tripped_class, class);
    assert_eq!(round_tripped_class_decl, class_decl);
    assert_eq!(round_tripped_default_export_class, default_export_class);
    assert_eq!(
        round_tripped_default_export_class_decl,
        default_export_class_decl
    );
}
