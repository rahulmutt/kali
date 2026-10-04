use super::*;

#[test]
fn test_parse_generator_class_method_preserves_generator_flag() {
    assert_parse_class_method_modifiers_are_preserved(
        "class Example { *main() { yield 1; } }",
        false,
        true,
    );
}

#[test]
fn test_parse_generator_class_method_delegating_yield_expression() {
    let tokens = lex("class Example { *main() { yield* other(); } }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ClassDeclaration(class_decl) => {
            assert_eq!(class_decl.body.methods.len(), 1);
            let method = &class_decl.body.methods[0];
            assert!(method.generator, "expected generator flag to be preserved");
            let body = method.body.as_ref().expect("method body");
            assert_eq!(body.body.len(), 1);
            match &body.body[0] {
                Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
                    Expression::YieldExpression(yield_expr) => {
                        assert!(
                            yield_expr.delegate,
                            "expected yield* delegation to be preserved"
                        );
                        let argument = yield_expr.argument.as_ref().expect("yield argument");
                        match argument {
                            Expression::CallExpression(call_expr) => {
                                assert_eq!(call_expr.args.len(), 0)
                            }
                            other => panic!("unexpected yield* argument: {other:?}"),
                        }
                    }
                    other => panic!("Expected YieldExpression, got {other:?}"),
                },
                other => panic!("Expected ExpressionStatement, got {other:?}"),
            }
        }
        other => panic!("Expected ClassDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_generator_class_method_preserves_generator_flags() {
    assert_parse_class_method_modifiers_are_preserved(
        "class Example { async *main() { yield 1; } }",
        true,
        true,
    );
}

#[test]
fn test_parse_class_expression_preserves_method_modifiers() {
    let tokens = lex("const Example = class NamedExample { async *main() { yield* other(); } };");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(decl) => {
            let init = decl.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::ClassExpression(class_expr) => {
                    assert_eq!(class_expr.id.as_deref(), Some("NamedExample"));
                    assert_eq!(class_expr.body.methods.len(), 1);
                    let method = &class_expr.body.methods[0];
                    assert_eq!(method.name, "main");
                    assert!(method.is_async, "expected async flag to be preserved");
                    assert!(method.generator, "expected generator flag to be preserved");
                    let body = method.body.as_ref().expect("method body");
                    assert_eq!(body.body.len(), 1);
                    match &body.body[0] {
                        Statement::ExpressionStatement(expr_stmt) => {
                            match expr_stmt.expression.as_ref() {
                                Expression::YieldExpression(yield_expr) => {
                                    assert!(
                                        yield_expr.delegate,
                                        "expected yield* delegation to be preserved"
                                    );
                                    let argument =
                                        yield_expr.argument.as_ref().expect("yield argument");
                                    match argument {
                                        Expression::CallExpression(call_expr) => {
                                            assert_eq!(call_expr.args.len(), 0)
                                        }
                                        other => panic!("unexpected yield* argument: {other:?}"),
                                    }
                                }
                                other => panic!("Expected YieldExpression, got {other:?}"),
                            }
                        }
                        other => panic!("Expected ExpressionStatement, got {other:?}"),
                    }
                }
                other => panic!("Expected ClassExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_default_export_class_expression_preserves_method_modifiers() {
    let tokens = lex("export default (class NamedExample { async *main() { yield* other(); } });");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportDefault(decl) => match decl {
            kali_ast::ExportDefaultDeclaration::Expression(expr) => {
                let mut expr = expr;
                loop {
                    match expr {
                        Expression::ParenthesizedExpression(parenthesized) => {
                            expr = parenthesized.expression.as_ref();
                        }
                        Expression::ClassExpression(class_expr) => {
                            assert_eq!(class_expr.id.as_deref(), Some("NamedExample"));
                            assert_eq!(class_expr.body.methods.len(), 1);
                            let method = &class_expr.body.methods[0];
                            assert_eq!(method.name, "main");
                            assert!(method.is_async, "expected async flag to be preserved");
                            assert!(method.generator, "expected generator flag to be preserved");
                            let body = method.body.as_ref().expect("method body");
                            assert_eq!(body.body.len(), 1);
                            match &body.body[0] {
                                Statement::ExpressionStatement(expr_stmt) => {
                                    match expr_stmt.expression.as_ref() {
                                        Expression::YieldExpression(yield_expr) => {
                                            assert!(
                                                yield_expr.delegate,
                                                "expected yield* delegation to be preserved"
                                            );
                                            let argument = yield_expr
                                                .argument
                                                .as_ref()
                                                .expect("yield argument");
                                            match argument {
                                                Expression::CallExpression(call_expr) => {
                                                    assert_eq!(call_expr.args.len(), 0)
                                                }
                                                other => {
                                                    panic!("unexpected yield* argument: {other:?}")
                                                }
                                            }
                                        }
                                        other => panic!("Expected YieldExpression, got {other:?}"),
                                    }
                                }
                                other => panic!("Expected ExpressionStatement, got {other:?}"),
                            }
                            break;
                        }
                        other => panic!("Expected default-export class expression, got {other:?}"),
                    }
                }
            }
            other => panic!("Expected default-export class expression, got {other:?}"),
        },
        other => panic!("Expected ExportDefaultDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_default_export_class_declaration_preserves_method_modifiers() {
    let tokens = lex("export default class NamedDeclExample { async *main() { yield* other(); } }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportDefault(decl) => match decl {
            kali_ast::ExportDefaultDeclaration::ClassDeclaration(class_decl) => {
                assert_eq!(class_decl.name, "NamedDeclExample");
                assert_eq!(class_decl.body.methods.len(), 1);
                let method = &class_decl.body.methods[0];
                assert_eq!(method.name, "main");
                assert!(method.is_async, "expected async flag to be preserved");
                assert!(method.generator, "expected generator flag to be preserved");
                let body = method.body.as_ref().expect("method body");
                assert_eq!(body.body.len(), 1);
                match &body.body[0] {
                    Statement::ExpressionStatement(expr_stmt) => {
                        match expr_stmt.expression.as_ref() {
                            Expression::YieldExpression(yield_expr) => {
                                assert!(
                                    yield_expr.delegate,
                                    "expected yield* delegation to be preserved"
                                );
                                let argument =
                                    yield_expr.argument.as_ref().expect("yield argument");
                                match argument {
                                    Expression::CallExpression(call_expr) => {
                                        assert_eq!(call_expr.args.len(), 0)
                                    }
                                    other => panic!("unexpected yield* argument: {other:?}"),
                                }
                            }
                            other => panic!("Expected YieldExpression, got {other:?}"),
                        }
                    }
                    other => panic!("Expected ExpressionStatement, got {other:?}"),
                }
            }
            other => panic!("Expected default-export class declaration, got {other:?}"),
        },
        other => panic!("Expected ExportDefaultDeclaration, got {other:?}"),
    }
}

fn parse_single_class(source: &str) -> kali_ast::ClassDeclaration {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    match output.statements.into_iter().next() {
        Some(Statement::ClassDeclaration(class_decl)) => class_decl,
        other => panic!("expected a class declaration, got {other:?}"),
    }
}

#[test]
fn a_class_keeps_its_base_name() {
    assert_eq!(
        parse_single_class("class B extends A { f(){} }")
            .super_class
            .as_deref(),
        Some("A")
    );
    assert_eq!(
        parse_single_class("class B extends ns.A {}")
            .super_class
            .as_deref(),
        Some("")
    );
    assert_eq!(
        parse_single_class("class B extends mixin(A) {}")
            .super_class
            .as_deref(),
        Some("")
    );
    assert_eq!(
        parse_single_class("class B<T> extends A<T> {}")
            .super_class
            .as_deref(),
        Some("A")
    );
    assert_eq!(
        parse_single_class("class B extends A implements I {}")
            .super_class
            .as_deref(),
        Some("A")
    );
    assert_eq!(parse_single_class("class B { f(){} }").super_class, None);
    assert_eq!(
        parse_single_class("class B implements I { f(){} }").super_class,
        None
    );
}

#[test]
fn a_member_expression_base_leaves_the_program() {
    // `A.Inner` is not the class `A`: recording `A` made `B` look like a
    // program class whose chain stays in the program.
    assert_eq!(
        parse_single_class("class B extends A.Inner {}")
            .super_class
            .as_deref(),
        Some("")
    );
    assert_eq!(
        parse_single_class("class B extends (A) {}")
            .super_class
            .as_deref(),
        Some("")
    );
}

#[test]
fn an_extends_inside_type_parameters_is_not_the_base() {
    assert_eq!(
        parse_single_class("class B<T extends Foo> {}").super_class,
        None
    );
    assert_eq!(
        parse_single_class("class B<T extends Foo<U>, U> extends A<T> {}")
            .super_class
            .as_deref(),
        Some("A")
    );
    assert_eq!(
        parse_single_class("class B<T extends Map<string, Set<U>>> extends A {}")
            .super_class
            .as_deref(),
        Some("A")
    );
}

#[test]
fn a_class_with_a_base_keeps_its_methods() {
    let class_decl = parse_single_class("class B extends A { f(){ return 1; } g(){} }");
    let names: Vec<_> = class_decl
        .body
        .methods
        .iter()
        .map(|m| m.name.as_str())
        .collect();
    assert_eq!(names, ["f", "g"]);
}

#[test]
fn a_class_keeps_its_field_names() {
    let class_decl = parse_single_class(
        "class S { n = 0; cb = () => 1; label: string; maybe?: number; done!: boolean; f(){} }",
    );
    assert_eq!(
        class_decl.body.field_names,
        ["n", "cb", "label", "maybe", "done"]
    );
    assert_eq!(class_decl.body.methods.len(), 1);
}

#[test]
fn a_class_expression_keeps_its_base_name() {
    let tokens = lex("const K = class extends EventTarget {};");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    let Some(Statement::VariableDeclaration(decl)) = output.statements.first() else {
        panic!("expected a declaration, got {:?}", output.statements);
    };
    let Some(Expression::ClassExpression(class_expr)) = &decl.declarations[0].init else {
        panic!("expected a class expression");
    };
    assert_eq!(class_expr.super_class.as_deref(), Some("EventTarget"));
    assert_eq!(class_expr.id, None);
}

#[test]
fn a_computed_member_key_marks_the_member_set_unknown() {
    for source in [
        "class C { [\"foo\"](){ return 1; } }",
        "class C { static [k] = 1; f(){} }",
        "class C { f(){} get [k](){ return 1; } }",
        "class C { *[Symbol.iterator](){} }",
        "class C { n = 0; [k] = 1; }",
    ] {
        assert!(
            parse_single_class(source).body.has_computed_members,
            "{source}"
        );
    }
    for source in [
        "class C { f(){ return [1]; } }",
        "class C { xs: number[] = []; f(){} }",
        "class C { n = 0; }",
    ] {
        assert!(
            !parse_single_class(source).body.has_computed_members,
            "{source}"
        );
    }
}

fn parse_class(source: &str) -> kali_ast::ClassBody {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    match &output.statements[0] {
        Statement::ClassDeclaration(class_decl) => (*class_decl.body).clone(),
        other => panic!("expected a class, got {other:?}"),
    }
}

#[test]
fn a_field_keeps_its_initializer() {
    let body = parse_class("class C { n = 1 + 2; label: string = \"x\"; m; }");
    let names: Vec<_> = body.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["n", "label", "m"]);
    assert!(matches!(
        body.fields[0].value,
        Some(Expression::BinaryExpression(_))
    ));
    assert!(matches!(body.fields[1].value, Some(Expression::Literal(_))));
    assert_eq!(body.fields[2].value, None);
    assert_eq!(body.field_names, ["n", "label", "m"]);
}

#[test]
fn fields_without_semicolons_end_where_the_expression_ends() {
    let body = parse_class("class C { n = 0\n m = 1\n f(){ return 2; } }");
    let names: Vec<_> = body.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["n", "m"]);
    assert_eq!(body.methods.len(), 1);
    assert_eq!(body.methods[0].name, "f");
}

#[test]
fn an_annotated_field_without_semicolon_stops_before_the_next_member() {
    let body = parse_class("class C { label: string\n m = 1 }");
    assert_eq!(body.fields[0].name, "label");
    assert_eq!(body.fields[0].value, None);
    assert_eq!(body.fields[1].name, "m");
}

#[test]
fn accessors_and_statics_are_marked() {
    let body = parse_class("class C { get v(){ return 1; } set v(x){} static make(){ return 2; } static k = 3; f(){} }");
    let kinds: Vec<_> = body
        .methods
        .iter()
        .map(|m| (m.name.as_str(), m.kind.clone(), m.is_static))
        .collect();
    assert_eq!(
        kinds,
        [
            ("v", MethodKind::Get, false),
            ("v", MethodKind::Set, false),
            ("make", MethodKind::Method, true),
            ("f", MethodKind::Method, false),
        ]
    );
    assert_eq!(body.fields[0].name, "k");
    assert!(body.fields[0].is_static);
}

#[test]
fn a_method_named_like_a_modifier_is_a_plain_method() {
    let body = parse_class("class C { get(){ return 1; } static(){ return 2; } }");
    let names: Vec<_> = body
        .methods
        .iter()
        .map(|m| (m.name.as_str(), m.kind.clone(), m.is_static))
        .collect();
    assert_eq!(
        names,
        [
            ("get", MethodKind::Method, false),
            ("static", MethodKind::Method, false)
        ]
    );
}

#[test]
fn private_members_and_static_blocks_are_flagged() {
    let body = parse_class("class C { #x = 1; #m(){} static { init(); } f(){} }");
    assert!(body.has_private_members);
    assert!(body.has_static_block);
    assert_eq!(body.methods.len(), 1);
    assert_eq!(body.methods[0].name, "f");
}

#[test]
fn a_block_arrow_is_marked_and_a_function_expression_is_not() {
    let tokens = lex("const f = () => { return 1; }; const g = function(){ return 2; };");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    let flags: Vec<bool> = output
        .statements
        .iter()
        .map(|s| match s {
            Statement::VariableDeclaration(d) => match &d.declarations[0].init {
                Some(Expression::FunctionExpression(f)) => f.is_arrow,
                other => panic!("expected a function expression, got {other:?}"),
            },
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(flags, [true, false]);
}
