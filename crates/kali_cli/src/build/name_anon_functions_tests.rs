//! Exhaustiveness + collision-guard tests for `name_anonymous_functions`.
//!
//! These are built as literal `kali_ast` struct trees (not parsed JS text) so
//! every position — including shapes the hand-written parser may not accept
//! together in one file, like a bare `await`/`yield` outside their normal
//! syntactic contexts — can be exercised directly and deterministically.
//! None of this AST is ever executed; only its shape after
//! `name_anonymous_functions` runs is inspected.

use super::*;
use std::collections::HashSet;

fn lit(n: f64) -> Expression {
    Expression::Literal(LiteralValue::Number(n))
}

fn ident(name: &str) -> Expression {
    Expression::Identifier(name.to_string())
}

fn arrow(body: Expression) -> Expression {
    Expression::ArrowFunctionExpression(Box::new(ArrowFunctionExpression {
        id: None,
        params: vec![],
        body,
        is_async: false,
        returnType: None,
    }))
}

fn func_expr(id: Option<&str>, body: Vec<Statement>) -> Expression {
    Expression::FunctionExpression(Box::new(FunctionExpression {
        id: id.map(|s| s.to_string()),
        params: vec![],
        body: Some(Box::new(BlockStatement { body })),
        is_async: false,
        generator: false,
    }))
}

fn const_decl(name: &str, init: Expression) -> Statement {
    Statement::VariableDeclaration(VariableDeclaration {
        kind: "const".to_string(),
        declarations: vec![VariableDeclarator {
            id: name.to_string(),
            init: Some(init),
        }],
    })
}

fn expr_stmt(e: Expression) -> Statement {
    Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(e),
    })
}

fn return_stmt(e: Expression) -> Statement {
    Statement::ReturnStatement(ReturnStatement { argument: Some(e) })
}

fn block(stmts: Vec<Statement>) -> BlockStatement {
    BlockStatement { body: stmts }
}

/// Pulls every `id` off an arrow/function-expression slot this test placed,
/// panicking if the slot isn't what the fixture put there — a stand-in for a
/// generic re-walking collector, deliberately avoided so this test's
/// trustworthiness doesn't depend on ANOTHER hand-written walk having its own
/// blind spots.
fn arrow_id(e: &Expression) -> Option<String> {
    match e {
        Expression::ArrowFunctionExpression(a) => a.id.clone(),
        other => panic!("expected ArrowFunctionExpression, got {other:?}"),
    }
}

fn func_id(e: &Expression) -> Option<String> {
    match e {
        Expression::FunctionExpression(f) => f.id.clone(),
        other => panic!("expected FunctionExpression, got {other:?}"),
    }
}

/// A fixture with a function-shaped node in many distinct AST positions:
/// nested arrows, an object-literal property value, a class-method body, an
/// IIFE callee, an array element, both ternary branches, a template
/// substitution, a function's return value, callback call-arguments (both
/// arrow and plain-function-expression), if/else branches, try/catch/finally,
/// a for-loop body, a while-loop body, a switch case, an assignment RHS, a
/// logical-operator operand, a `new` argument, an `await` argument, and a
/// `yield` argument. Every slot must receive a DISTINCT, non-`None`
/// `__kali_fn_N` id; a duplicate means one body silently overwrote another.
#[test]
fn every_anonymous_function_shaped_position_gets_a_distinct_id() {
    // 1: top-level arrow
    let s1 = const_decl("a", arrow(lit(1.0)));
    // 2: top-level arrow (sibling — must differ from s1)
    let s2 = const_decl("b", arrow(lit(2.0)));
    // 3/4: arrow nested inside another arrow's body
    let inner_in_outer = arrow(lit(3.0));
    let outer_arrow = arrow(inner_in_outer.clone());
    let s3 = const_decl("c", outer_arrow);
    // 5: object-literal property value
    let s5 = const_decl(
        "obj",
        Expression::ObjectExpression(ObjectExpression {
            properties: vec![ObjectProperty {
                key: PropertyName::Identifier("f".to_string()),
                value: arrow(lit(5.0)),
                kind: ObjectPropertyKind::Init,
            }],
        }),
    );
    // 6: class-method body
    let s6 = Statement::ClassDeclaration(ClassDeclaration {
        name: "C".to_string(),
        body: Box::new(ClassBody {
            methods: vec![MethodDefinition {
                name: "m".to_string(),
                params: vec![],
                body: Some(Box::new(block(vec![const_decl("x", arrow(lit(6.0)))]))),
                is_async: false,
                generator: false,
            }],
        }),
    });
    // 7: IIFE — call callee is a parenthesized anonymous function expression
    let s7 = expr_stmt(Expression::CallExpression(Box::new(CallExpression {
        callee: Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
            expression: Box::new(func_expr(None, vec![return_stmt(lit(7.0))])),
        })),
        args: vec![],
    })));
    // 8: array element
    let s8 = const_decl(
        "arr",
        Expression::ArrayExpression(ArrayExpression {
            elements: vec![Some(ExpressionOrSpread::Expression(arrow(lit(8.0))))],
        }),
    );
    // 9/10: both ternary branches
    let s9 = const_decl(
        "tern",
        Expression::ConditionalExpression(Box::new(ConditionalExpression {
            test: Box::new(ident("cond")),
            consequent: Box::new(arrow(lit(9.0))),
            alternate: Box::new(arrow(lit(10.0))),
        })),
    );
    // 11: template-literal substitution
    let s11 = const_decl(
        "tmpl",
        Expression::TemplateLiteral(TemplateLiteral {
            quasis: vec![
                TemplateElement {
                    value: "".to_string(),
                    tail: false,
                },
                TemplateElement {
                    value: "".to_string(),
                    tail: true,
                },
            ],
            expressions: vec![arrow(lit(11.0))],
        }),
    );
    // 12: return value inside a named function declaration
    let s12 = Statement::FunctionDeclaration(FunctionDeclaration {
        name: "outer".to_string(),
        params: vec![],
        body: Box::new(block(vec![return_stmt(arrow(lit(12.0)))])),
        is_async: false,
        generator: false,
    });
    // 13/14: two callback call-arguments (the Stage 6 motivating shape)
    let s13 = expr_stmt(Expression::CallExpression(Box::new(CallExpression {
        callee: ident("f"),
        args: vec![arrow(lit(13.0)), arrow(lit(14.0))],
    })));
    // 15: a plain FunctionExpression passed as a callback argument
    let s15 = expr_stmt(Expression::CallExpression(Box::new(CallExpression {
        callee: ident("f2"),
        args: vec![func_expr(None, vec![return_stmt(lit(15.0))])],
    })));
    // 16/17: if/else branches
    let s16_17 = Statement::IfStatement(IfStatement {
        test: ident("cond"),
        consequent: Box::new(block(vec![const_decl("x", arrow(lit(16.0)))])),
        alternate: Some(Box::new(block(vec![const_decl("y", arrow(lit(17.0)))]))),
    });
    // 18/19/20: try / catch / finally
    let s18_20 = Statement::TryStatement(TryStatement {
        block: Box::new(block(vec![const_decl("x", arrow(lit(18.0)))])),
        handler: Some(CatchClause {
            param: "e".to_string(),
            body: Box::new(block(vec![const_decl("y", arrow(lit(19.0)))])),
        }),
        finalizer: Some(block(vec![const_decl("z", arrow(lit(20.0)))])),
    });
    // 21: for-loop body
    let s21 = Statement::ForStatement(ForStatement {
        init: None,
        test: None,
        update: None,
        body: Box::new(block(vec![const_decl("x", arrow(lit(21.0)))])),
    });
    // 22: while-loop body
    let s22 = Statement::WhileStatement(WhileStatement {
        test: ident("cond"),
        body: Box::new(block(vec![const_decl("x", arrow(lit(22.0)))])),
    });
    // 23: switch case
    let s23 = Statement::SwitchStatement(SwitchStatement {
        discriminant: ident("v"),
        cases: vec![SwitchCase {
            test: Some(lit(1.0)),
            consequent: vec![const_decl("x", arrow(lit(23.0)))],
        }],
    });
    // 24: assignment RHS
    let s24 = expr_stmt(Expression::AssignmentExpression(Box::new(
        AssignmentExpression {
            operator: AssignmentOperator::Assign,
            left: ident("y"),
            right: arrow(lit(24.0)),
        },
    )));
    // 25: logical-operator operand
    let s25 = const_decl(
        "z",
        Expression::LogicalExpression(Box::new(LogicalExpression {
            operator: LogicalOperator::Or,
            left: Box::new(ident("a")),
            right: Box::new(arrow(lit(25.0))),
        })),
    );
    // 26: `new` argument
    let s26 = const_decl(
        "n",
        Expression::NewExpression(Box::new(NewExpression {
            callee: ident("Foo"),
            args: vec![arrow(lit(26.0))],
        })),
    );
    // 27: `await` argument
    let s27 = Statement::FunctionDeclaration(FunctionDeclaration {
        name: "af".to_string(),
        params: vec![],
        body: Box::new(block(vec![expr_stmt(Expression::AwaitExpression(
            Box::new(AwaitExpression {
                argument: arrow(lit(27.0)),
            }),
        ))])),
        is_async: true,
        generator: false,
    });
    // 28: `yield` argument
    let s28 = Statement::FunctionDeclaration(FunctionDeclaration {
        name: "gen".to_string(),
        params: vec![],
        body: Box::new(block(vec![expr_stmt(Expression::YieldExpression(
            Box::new(YieldExpression {
                delegate: false,
                argument: Some(arrow(lit(28.0))),
            }),
        ))])),
        is_async: false,
        generator: true,
    });
    // 29: named function expression — must KEEP its own name, not be renamed,
    // and must not be double-counted as a fresh slot.
    let s29 = const_decl(
        "named",
        func_expr(Some("myName"), vec![return_stmt(lit(29.0))]),
    );

    let mut statements = vec![
        s1, s2, s3, s5, s6, s7, s8, s9, s11, s12, s13, s15, s16_17, s18_20, s21, s22, s23, s24,
        s25, s26, s27, s28, s29,
    ];

    name_anonymous_functions(&mut statements);

    // Pull every assigned id back out by walking the same fixture shape.
    let mut ids: Vec<String> = Vec::new();

    macro_rules! decl_init {
        ($stmt:expr) => {
            match &$stmt {
                Statement::VariableDeclaration(vd) => vd.declarations[0].init.as_ref().unwrap(),
                other => panic!("expected VariableDeclaration, got {other:?}"),
            }
        };
    }

    // s1, s2
    ids.push(arrow_id(decl_init!(statements[0])).expect("s1 arrow must be named"));
    ids.push(arrow_id(decl_init!(statements[1])).expect("s2 arrow must be named"));
    // s3: outer arrow + nested inner arrow
    {
        let outer = decl_init!(statements[2]);
        ids.push(
            arrow_id(outer)
                .clone()
                .expect("s3 outer arrow must be named"),
        );
        match outer {
            Expression::ArrowFunctionExpression(a) => {
                ids.push(arrow_id(&a.body).expect("s3 inner arrow must be named"));
            }
            other => panic!("expected ArrowFunctionExpression, got {other:?}"),
        }
    }
    // s5: object property value
    {
        let obj = decl_init!(statements[3]);
        match obj {
            Expression::ObjectExpression(o) => {
                ids.push(arrow_id(&o.properties[0].value).expect("s5 must be named"));
            }
            other => panic!("expected ObjectExpression, got {other:?}"),
        }
    }
    // s6: class method body
    match &statements[4] {
        Statement::ClassDeclaration(cd) => {
            match cd.body.methods[0].body.as_ref().unwrap().body[0].clone() {
                Statement::VariableDeclaration(vd) => {
                    ids.push(arrow_id(vd.declarations[0].init.as_ref().unwrap()).expect("s6"));
                }
                other => panic!("expected VariableDeclaration, got {other:?}"),
            }
        }
        other => panic!("expected ClassDeclaration, got {other:?}"),
    }
    // s7: IIFE callee
    match &statements[5] {
        Statement::ExpressionStatement(es) => match es.expression.as_ref() {
            Expression::CallExpression(call) => match &call.callee {
                Expression::ParenthesizedExpression(p) => {
                    ids.push(func_id(&p.expression).expect("s7"));
                }
                other => panic!("expected ParenthesizedExpression, got {other:?}"),
            },
            other => panic!("expected CallExpression, got {other:?}"),
        },
        other => panic!("expected ExpressionStatement, got {other:?}"),
    }
    // s8: array element
    {
        let arr = decl_init!(statements[6]);
        match arr {
            Expression::ArrayExpression(a) => match a.elements[0].as_ref().unwrap() {
                ExpressionOrSpread::Expression(e) => ids.push(arrow_id(e).expect("s8")),
                other => panic!("expected Expression element, got {other:?}"),
            },
            other => panic!("expected ArrayExpression, got {other:?}"),
        }
    }
    // s9/10: ternary branches
    {
        let tern = decl_init!(statements[7]);
        match tern {
            Expression::ConditionalExpression(c) => {
                ids.push(arrow_id(&c.consequent).expect("s9"));
                ids.push(arrow_id(&c.alternate).expect("s10"));
            }
            other => panic!("expected ConditionalExpression, got {other:?}"),
        }
    }
    // s11: template substitution
    {
        let tmpl = decl_init!(statements[8]);
        match tmpl {
            Expression::TemplateLiteral(t) => {
                ids.push(arrow_id(&t.expressions[0]).expect("s11"));
            }
            other => panic!("expected TemplateLiteral, got {other:?}"),
        }
    }
    // s12: return value
    match &statements[9] {
        Statement::FunctionDeclaration(fd) => match &fd.body.body[0] {
            Statement::ReturnStatement(rs) => {
                ids.push(arrow_id(rs.argument.as_ref().unwrap()).expect("s12"));
            }
            other => panic!("expected ReturnStatement, got {other:?}"),
        },
        other => panic!("expected FunctionDeclaration, got {other:?}"),
    }
    // s13/14: callback args
    match &statements[10] {
        Statement::ExpressionStatement(es) => match es.expression.as_ref() {
            Expression::CallExpression(call) => {
                ids.push(arrow_id(&call.args[0]).expect("s13"));
                ids.push(arrow_id(&call.args[1]).expect("s14"));
            }
            other => panic!("expected CallExpression, got {other:?}"),
        },
        other => panic!("expected ExpressionStatement, got {other:?}"),
    }
    // s15: plain function-expression callback arg
    match &statements[11] {
        Statement::ExpressionStatement(es) => match es.expression.as_ref() {
            Expression::CallExpression(call) => {
                ids.push(func_id(&call.args[0]).expect("s15"));
            }
            other => panic!("expected CallExpression, got {other:?}"),
        },
        other => panic!("expected ExpressionStatement, got {other:?}"),
    }
    // s16/17: if/else
    match &statements[12] {
        Statement::IfStatement(is) => {
            match &is.consequent.body[0] {
                Statement::VariableDeclaration(vd) => {
                    ids.push(arrow_id(vd.declarations[0].init.as_ref().unwrap()).expect("s16"));
                }
                other => panic!("expected VariableDeclaration, got {other:?}"),
            }
            match &is.alternate.as_ref().unwrap().body[0] {
                Statement::VariableDeclaration(vd) => {
                    ids.push(arrow_id(vd.declarations[0].init.as_ref().unwrap()).expect("s17"));
                }
                other => panic!("expected VariableDeclaration, got {other:?}"),
            }
        }
        other => panic!("expected IfStatement, got {other:?}"),
    }
    // s18/19/20: try/catch/finally
    match &statements[13] {
        Statement::TryStatement(ts) => {
            match &ts.block.body[0] {
                Statement::VariableDeclaration(vd) => {
                    ids.push(arrow_id(vd.declarations[0].init.as_ref().unwrap()).expect("s18"));
                }
                other => panic!("expected VariableDeclaration, got {other:?}"),
            }
            match &ts.handler.as_ref().unwrap().body.body[0] {
                Statement::VariableDeclaration(vd) => {
                    ids.push(arrow_id(vd.declarations[0].init.as_ref().unwrap()).expect("s19"));
                }
                other => panic!("expected VariableDeclaration, got {other:?}"),
            }
            match &ts.finalizer.as_ref().unwrap().body[0] {
                Statement::VariableDeclaration(vd) => {
                    ids.push(arrow_id(vd.declarations[0].init.as_ref().unwrap()).expect("s20"));
                }
                other => panic!("expected VariableDeclaration, got {other:?}"),
            }
        }
        other => panic!("expected TryStatement, got {other:?}"),
    }
    // s21: for-loop body
    match &statements[14] {
        Statement::ForStatement(fs) => match &fs.body.body[0] {
            Statement::VariableDeclaration(vd) => {
                ids.push(arrow_id(vd.declarations[0].init.as_ref().unwrap()).expect("s21"));
            }
            other => panic!("expected VariableDeclaration, got {other:?}"),
        },
        other => panic!("expected ForStatement, got {other:?}"),
    }
    // s22: while-loop body
    match &statements[15] {
        Statement::WhileStatement(ws) => match &ws.body.body[0] {
            Statement::VariableDeclaration(vd) => {
                ids.push(arrow_id(vd.declarations[0].init.as_ref().unwrap()).expect("s22"));
            }
            other => panic!("expected VariableDeclaration, got {other:?}"),
        },
        other => panic!("expected WhileStatement, got {other:?}"),
    }
    // s23: switch case
    match &statements[16] {
        Statement::SwitchStatement(sw) => match &sw.cases[0].consequent[0] {
            Statement::VariableDeclaration(vd) => {
                ids.push(arrow_id(vd.declarations[0].init.as_ref().unwrap()).expect("s23"));
            }
            other => panic!("expected VariableDeclaration, got {other:?}"),
        },
        other => panic!("expected SwitchStatement, got {other:?}"),
    }
    // s24: assignment RHS
    match &statements[17] {
        Statement::ExpressionStatement(es) => match es.expression.as_ref() {
            Expression::AssignmentExpression(ae) => {
                ids.push(arrow_id(&ae.right).expect("s24"));
            }
            other => panic!("expected AssignmentExpression, got {other:?}"),
        },
        other => panic!("expected ExpressionStatement, got {other:?}"),
    }
    // s25: logical operand
    {
        let z = decl_init!(statements[18]);
        match z {
            Expression::LogicalExpression(le) => {
                ids.push(arrow_id(&le.right).expect("s25"));
            }
            other => panic!("expected LogicalExpression, got {other:?}"),
        }
    }
    // s26: new argument
    {
        let n = decl_init!(statements[19]);
        match n {
            Expression::NewExpression(ne) => {
                ids.push(arrow_id(&ne.args[0]).expect("s26"));
            }
            other => panic!("expected NewExpression, got {other:?}"),
        }
    }
    // s27: await argument
    match &statements[20] {
        Statement::FunctionDeclaration(fd) => match &fd.body.body[0] {
            Statement::ExpressionStatement(es) => match es.expression.as_ref() {
                Expression::AwaitExpression(aw) => {
                    ids.push(arrow_id(&aw.argument).expect("s27"));
                }
                other => panic!("expected AwaitExpression, got {other:?}"),
            },
            other => panic!("expected ExpressionStatement, got {other:?}"),
        },
        other => panic!("expected FunctionDeclaration, got {other:?}"),
    }
    // s28: yield argument
    match &statements[21] {
        Statement::FunctionDeclaration(fd) => match &fd.body.body[0] {
            Statement::ExpressionStatement(es) => match es.expression.as_ref() {
                Expression::YieldExpression(ye) => {
                    ids.push(arrow_id(ye.argument.as_ref().unwrap()).expect("s28"));
                }
                other => panic!("expected YieldExpression, got {other:?}"),
            },
            other => panic!("expected ExpressionStatement, got {other:?}"),
        },
        other => panic!("expected FunctionDeclaration, got {other:?}"),
    }
    // s29: named function expression keeps its own name, untouched.
    {
        let named = decl_init!(statements[22]);
        assert_eq!(func_id(named), Some("myName".to_string()));
    }

    assert_eq!(
        ids.len(),
        28,
        "expected exactly 28 anonymous slots to be named"
    );
    for id in &ids {
        assert!(id.starts_with("__kali_fn_"), "unexpected id shape: {id}");
    }
    let unique: HashSet<&String> = ids.iter().collect();
    assert_eq!(
        unique.len(),
        ids.len(),
        "duplicate __kali_fn_N name assigned — one body would silently overwrite another: {ids:?}"
    );
}

/// A user-declared source function literally named `__kali_fn_0` must not be
/// clobbered, and no anonymous node may be assigned that same name even
/// though it is exactly the first candidate the counter would propose.
#[test]
fn collision_with_user_declared_name_is_avoided() {
    let mut statements = vec![
        // The colliding declaration appears BEFORE the anonymous arrows...
        Statement::FunctionDeclaration(FunctionDeclaration {
            name: "__kali_fn_0".to_string(),
            params: vec![],
            body: Box::new(block(vec![])),
            is_async: false,
            generator: false,
        }),
        const_decl("a", arrow(lit(1.0))),
        const_decl("b", arrow(lit(2.0))),
    ];

    name_anonymous_functions(&mut statements);

    let a_id = arrow_id(match &statements[1] {
        Statement::VariableDeclaration(vd) => vd.declarations[0].init.as_ref().unwrap(),
        other => panic!("expected VariableDeclaration, got {other:?}"),
    })
    .unwrap();
    let b_id = arrow_id(match &statements[2] {
        Statement::VariableDeclaration(vd) => vd.declarations[0].init.as_ref().unwrap(),
        other => panic!("expected VariableDeclaration, got {other:?}"),
    })
    .unwrap();

    assert_ne!(a_id, "__kali_fn_0");
    assert_ne!(b_id, "__kali_fn_0");
    assert_ne!(a_id, b_id);
}

/// Same collision, but the colliding declaration appears AFTER the anonymous
/// nodes in source order — proves collection (pass 1) really does run to
/// completion over the WHOLE tree before any name is assigned (pass 2),
/// independent of textual order.
#[test]
fn collision_guard_is_independent_of_declaration_order() {
    let mut statements = vec![
        const_decl("a", arrow(lit(1.0))),
        const_decl("b", arrow(lit(2.0))),
        // The colliding declaration appears AFTER.
        Statement::FunctionDeclaration(FunctionDeclaration {
            name: "__kali_fn_0".to_string(),
            params: vec![],
            body: Box::new(block(vec![])),
            is_async: false,
            generator: false,
        }),
    ];

    name_anonymous_functions(&mut statements);

    let a_id = arrow_id(match &statements[0] {
        Statement::VariableDeclaration(vd) => vd.declarations[0].init.as_ref().unwrap(),
        other => panic!("expected VariableDeclaration, got {other:?}"),
    })
    .unwrap();
    let b_id = arrow_id(match &statements[1] {
        Statement::VariableDeclaration(vd) => vd.declarations[0].init.as_ref().unwrap(),
        other => panic!("expected VariableDeclaration, got {other:?}"),
    })
    .unwrap();

    assert_ne!(a_id, "__kali_fn_0");
    assert_ne!(b_id, "__kali_fn_0");
    assert_ne!(a_id, b_id);
}

/// Idempotent: running the pass twice must not change any already-assigned
/// id (a re-run must never treat a named node as fresh).
#[test]
fn running_twice_is_a_no_op_the_second_time() {
    let mut statements = vec![
        const_decl("a", arrow(lit(1.0))),
        const_decl("b", arrow(lit(2.0))),
    ];

    name_anonymous_functions(&mut statements);
    let first_pass = statements.clone();
    name_anonymous_functions(&mut statements);

    assert_eq!(statements, first_pass);
}

/// Task 3 (B): `export default function () {}` (anonymous) parses to a
/// `FunctionDeclaration` with `name: ""` — `kali_ast::FunctionDeclaration.name`
/// is a plain `String`, never `Option`, so there is no `None` for this pass
/// to fill via the usual `expr.id` path; it is the literal empty string.
/// `kali_hir` already falls back to a synthetic name for this exact
/// empty-name case (`lowering/statement.rs`'s `FunctionDeclaration` arm,
/// reached via `lower_export_default`'s `Statement::FunctionDeclaration`
/// routing), but `kali_types::resolve_export_default` binds the function's
/// scope directly under the raw (possibly empty) `func.name` with no
/// fallback of its own (`resolve/mod.rs`) — so before this fix the two
/// crates disagreed on the name for this node. The pre-pass must fill
/// `function.name` here too, under the SAME shared counter as every other
/// anonymous node, so both sides agree.
#[test]
fn export_default_anonymous_function_declaration_gets_a_name() {
    let mut statements = vec![Statement::ExportDefault(
        ExportDefaultDeclaration::FunctionDeclaration(FunctionDeclaration {
            name: String::new(),
            params: vec![],
            body: Box::new(block(vec![return_stmt(lit(1.0))])),
            is_async: false,
            generator: false,
        }),
    )];

    name_anonymous_functions(&mut statements);

    let Statement::ExportDefault(ExportDefaultDeclaration::FunctionDeclaration(function)) =
        &statements[0]
    else {
        panic!(
            "expected an ExportDefault(FunctionDeclaration), got {:?}",
            statements[0]
        );
    };
    assert!(
        !function.name.is_empty(),
        "anonymous export-default function must be named by the pre-pass"
    );
    assert!(
        function.name.starts_with("__kali_fn_"),
        "expected the shared synthetic-name convention, got {:?}",
        function.name
    );
}

/// A NAMED `export default function foo() {}` must keep its own name — the
/// fill only applies to the anonymous (`name: ""`) case.
#[test]
fn export_default_named_function_declaration_keeps_its_name() {
    let mut statements = vec![Statement::ExportDefault(
        ExportDefaultDeclaration::FunctionDeclaration(FunctionDeclaration {
            name: "foo".to_string(),
            params: vec![],
            body: Box::new(block(vec![return_stmt(lit(1.0))])),
            is_async: false,
            generator: false,
        }),
    )];

    name_anonymous_functions(&mut statements);

    let Statement::ExportDefault(ExportDefaultDeclaration::FunctionDeclaration(function)) =
        &statements[0]
    else {
        panic!(
            "expected an ExportDefault(FunctionDeclaration), got {:?}",
            statements[0]
        );
    };
    assert_eq!(function.name, "foo");
}
