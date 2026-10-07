use crate::*;

#[test]
fn test_function_kind_metadata_survives_serde_roundtrip() {
    let function = FunctionDeclaration {
        name: "generatorFn".to_string(),
        params: vec!["value".to_string()],
        defaults: Vec::new(),
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
        ..Default::default()
    };
    let class = ClassExpression {
        super_class: None,
        id: Some("Example".to_string()),
        body: Box::new(ClassBody {
            field_names: Vec::new(),
            has_computed_members: false,
            methods: vec![
                MethodDefinition {
                    name: "outer".to_string(),
                    params: vec!["value".to_string()],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: true,
                    generator: true,
                    ..Default::default()
                },
                MethodDefinition {
                    name: "inner".to_string(),
                    params: vec![],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: false,
                    generator: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        }),
    };
    let class_decl = ClassDeclaration {
        super_class: None,
        name: "DeclExample".to_string(),
        body: Box::new(ClassBody {
            field_names: Vec::new(),
            has_computed_members: false,
            methods: vec![
                MethodDefinition {
                    name: "outer".to_string(),
                    params: vec!["value".to_string()],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: true,
                    generator: true,
                    ..Default::default()
                },
                MethodDefinition {
                    name: "inner".to_string(),
                    params: vec![],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: false,
                    generator: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        }),
    };
    let default_export_class = Statement::ExportDefault(ExportDefaultDeclaration::Expression(
        Expression::ClassExpression(Box::new(ClassExpression {
            super_class: None,
            id: Some("DefaultExample".to_string()),
            body: Box::new(ClassBody {
                field_names: Vec::new(),
                has_computed_members: false,
                methods: vec![
                    MethodDefinition {
                        name: "outer".to_string(),
                        params: vec![],
                        body: Some(Box::new(BlockStatement { body: vec![] })),
                        is_async: true,
                        generator: true,
                        ..Default::default()
                    },
                    MethodDefinition {
                        name: "inner".to_string(),
                        params: vec![],
                        body: Some(Box::new(BlockStatement { body: vec![] })),
                        is_async: false,
                        generator: true,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
        })),
    ));
    let default_export_class_decl = Statement::ExportDefault(
        ExportDefaultDeclaration::ClassDeclaration(ClassDeclaration {
            super_class: None,
            name: "DefaultDeclExample".to_string(),
            body: Box::new(ClassBody {
                field_names: Vec::new(),
                has_computed_members: false,
                methods: vec![
                    MethodDefinition {
                        name: "outer".to_string(),
                        params: vec![],
                        body: Some(Box::new(BlockStatement { body: vec![] })),
                        is_async: true,
                        generator: true,
                        ..Default::default()
                    },
                    MethodDefinition {
                        name: "inner".to_string(),
                        params: vec![],
                        body: Some(Box::new(BlockStatement { body: vec![] })),
                        is_async: false,
                        generator: true,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }),
        }),
    );
    let default_export_async_generator = Statement::ExportDefault(
        ExportDefaultDeclaration::FunctionDeclaration(FunctionDeclaration {
            name: "DefaultAsyncGenerator".to_string(),
            params: vec![],
            defaults: Vec::new(),
            body: Box::new(BlockStatement { body: vec![] }),
            is_async: true,
            generator: true,
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
    let round_tripped_default_export_async_generator: Statement =
        serde_json::from_str(&serde_json::to_string(&default_export_async_generator).unwrap())
            .unwrap();

    assert_eq!(round_tripped_function, function);
    assert_eq!(round_tripped_function_expr, function_expr);
    assert_eq!(round_tripped_class, class);
    assert_eq!(round_tripped_class_decl, class_decl);
    assert_eq!(round_tripped_default_export_class, default_export_class);
    assert_eq!(
        round_tripped_default_export_class_decl,
        default_export_class_decl
    );
    assert_eq!(
        round_tripped_default_export_async_generator,
        default_export_async_generator
    );
}

#[test]
fn test_ast_roundtrips_default_export_anonymous_generator_function_declaration() {
    let default_export_generator = Statement::ExportDefault(
        ExportDefaultDeclaration::FunctionDeclaration(FunctionDeclaration {
            name: "".to_string(),
            params: vec![],
            defaults: Vec::new(),
            body: Box::new(BlockStatement { body: vec![] }),
            is_async: false,
            generator: true,
        }),
    );

    let round_tripped_default_export_generator: Statement =
        serde_json::from_str(&serde_json::to_string(&default_export_generator).unwrap()).unwrap();

    assert_eq!(
        round_tripped_default_export_generator,
        default_export_generator
    );
}

#[test]
fn a_function_declaration_without_defaults_serializes_without_the_field() {
    let declaration = FunctionDeclaration {
        name: "f".to_string(),
        params: vec!["a".to_string()],
        defaults: Vec::new(),
        body: Box::new(crate::BlockStatement { body: Vec::new() }),
        is_async: false,
        generator: false,
    };
    let json = serde_json::to_string(&declaration).expect("serializes");
    assert!(!json.contains("defaults"), "{json}");
    let back: FunctionDeclaration = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, declaration);
}

#[test]
fn a_function_declaration_with_a_default_round_trips() {
    let declaration = FunctionDeclaration {
        name: "f".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        defaults: vec![
            None,
            Some(Box::new(crate::Expression::Literal(crate::LiteralValue::Number(2.0)))),
        ],
        body: Box::new(crate::BlockStatement { body: Vec::new() }),
        is_async: false,
        generator: false,
    };
    let json = serde_json::to_string(&declaration).expect("serializes");
    let back: FunctionDeclaration = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, declaration);
}
