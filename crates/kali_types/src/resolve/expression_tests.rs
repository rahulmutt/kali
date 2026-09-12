use crate::test_support::*;
use crate::*;
use kali_ast::{
    AssignmentExpression, AssignmentOperator, BinaryExpression, BlockStatement, CallExpression,
    DecoratedExpression, ExportDefaultDeclaration, ExportNamedDeclaration, ExportSpecifier,
    Expression, ExpressionStatement, FunctionDeclaration, LiteralValue, LogicalExpression,
    LogicalOperator, MemberExpression, ObjectExpression, ObjectProperty, ObjectPropertyKind,
    OptionalChainExpression, OptionalChainInner, ParenthesizedExpression, PropertyName,
    TemplateElement, TemplateLiteral, UnaryExpression, UpdateExpression, UpdateOperator,
    VariableDeclaration, VariableDeclarator,
};
use kali_error::_error_codes::{e3, e5};
use kali_test_support::fixtures;
use std::fs;

#[path = "expression_tests/exports.rs"]
mod exports;

#[path = "expression_tests/operators.rs"]
mod operators;

#[path = "expression_tests/dynamic_import.rs"]
mod dynamic_import;

#[test]
fn a_one_element_array_literal_of_an_allocation_is_refused() {
    // `[new Array(3)]` lowers to the same LIR node as `new Array(3)` itself
    // (`crates/kali_mir/src/lower.rs:108`, `:116` erase the HIR distinction), so
    // codegen reads the literal as the allocation and answers its length.
    let statements = vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "const".to_string(),
        declarations: vec![VariableDeclarator {
            id: "xs".to_string(),
            init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::NewExpression(Box::new(kali_ast::NewExpression {
                        callee: Expression::CallExpression(Box::new(CallExpression {
                            callee: Expression::Identifier("Array".to_string()),
                            args: vec![Expression::Literal(LiteralValue::Number(3.0))],
                        })),
                        args: vec![],
                    })),
                ))],
            })),
        }],
    })];

    let result = assert_resolution!(statements, diagnostics: 1);
    assert!(
        result.diagnostics[0]
            .message
            .contains("lowers to the same node as the allocation itself"),
        "unexpected message: {}",
        result.diagnostics[0].message
    );
}

#[test]
fn a_two_element_array_literal_of_allocations_is_admitted() {
    // Only the ONE-element literal collides: a two-child text-less `Value` is not
    // a shape `resolve_array_alloc_call` accepts.
    let allocation = || {
        Expression::NewExpression(Box::new(kali_ast::NewExpression {
            callee: Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::Identifier("Array".to_string()),
                args: vec![Expression::Literal(LiteralValue::Number(3.0))],
            })),
            args: vec![],
        }))
    };
    let statements = vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "const".to_string(),
        declarations: vec![VariableDeclarator {
            id: "xs".to_string(),
            init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![
                    Some(kali_ast::ExpressionOrSpread::Expression(allocation())),
                    Some(kali_ast::ExpressionOrSpread::Expression(allocation())),
                ],
            })),
        }],
    })];

    assert_resolution!(statements, diagnostics: 0);
}

#[test]
fn a_one_element_array_literal_of_a_non_allocation_is_admitted() {
    // The over-refusal direction: a one-element literal whose element is not
    // an array allocation at all must stay silent on this gate.
    let statements = vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "const".to_string(),
        declarations: vec![VariableDeclarator {
            id: "xs".to_string(),
            init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(1.0)),
                ))],
            })),
        }],
    })];

    assert_resolution!(statements, diagnostics: 0);
}

/// `const xs = [<element>]; ` as a `VariableDeclaration` statement, for the
/// one-element-literal-gate tests below — every one of them differs only in
/// the element expression.
fn one_element_literal_statements(element: Expression) -> Vec<Statement> {
    vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "const".to_string(),
        declarations: vec![VariableDeclarator {
            id: "xs".to_string(),
            init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Expression(element))],
            })),
        }],
    })]
}

fn parenthesized(expr: Expression) -> Expression {
    Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
        expression: Box::new(expr),
    }))
}

#[test]
fn a_one_element_literal_of_a_paren_wrapped_callee_allocation_is_refused() {
    // `[new (Array)(3)]` — HIR erases the parenthesized CALLEE exactly as it
    // erases a parenthesized whole expression, so this collides with the
    // allocation the same way `[new Array(3)]` does. Round 1's fix only
    // unwrapped wrappers at the expression root and missed this.
    let allocation = Expression::NewExpression(Box::new(kali_ast::NewExpression {
        callee: Expression::CallExpression(Box::new(CallExpression {
            callee: parenthesized(Expression::Identifier("Array".to_string())),
            args: vec![Expression::Literal(LiteralValue::Number(3.0))],
        })),
        args: vec![],
    }));

    let result = assert_resolution!(one_element_literal_statements(allocation), diagnostics: 1);
    assert!(
        result.diagnostics[0]
            .message
            .contains("lowers to the same node as the allocation itself"),
        "unexpected message: {}",
        result.diagnostics[0].message
    );
}

#[test]
fn a_one_element_literal_of_a_paren_wrapped_global_this_uint8array_callee_is_refused() {
    // `[new (globalThis.Uint8Array)(3)]` — the whole member callee is
    // parenthesized, not just the object; `unwrap_transparent` at the callee
    // position peels it before `is_global_this_uint8array` ever looks at it.
    let allocation = Expression::NewExpression(Box::new(kali_ast::NewExpression {
        callee: Expression::CallExpression(Box::new(CallExpression {
            callee: parenthesized(Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("globalThis".to_string()),
                property: Some("Uint8Array".to_string()),
                computed_index: None,
            }))),
            args: vec![Expression::Literal(LiteralValue::Number(3.0))],
        })),
        args: vec![],
    }));

    let result = assert_resolution!(one_element_literal_statements(allocation), diagnostics: 1);
    assert!(
        result.diagnostics[0]
            .message
            .contains("lowers to the same node as the allocation itself"),
        "unexpected message: {}",
        result.diagnostics[0].message
    );
}

#[test]
fn a_one_element_literal_of_a_paren_wrapped_global_this_object_is_refused() {
    // `[new (globalThis).Uint8Array(3)]` — only the OBJECT half of the member
    // is parenthesized; `is_global_this_uint8array` unwraps `member.object`
    // itself to reach it.
    let allocation = Expression::NewExpression(Box::new(kali_ast::NewExpression {
        callee: Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: parenthesized(Expression::Identifier("globalThis".to_string())),
                property: Some("Uint8Array".to_string()),
                computed_index: None,
            })),
            args: vec![Expression::Literal(LiteralValue::Number(3.0))],
        })),
        args: vec![],
    }));

    let result = assert_resolution!(one_element_literal_statements(allocation), diagnostics: 1);
    assert!(
        result.diagnostics[0]
            .message
            .contains("lowers to the same node as the allocation itself"),
        "unexpected message: {}",
        result.diagnostics[0].message
    );
}

#[test]
fn a_one_element_literal_of_a_parenless_new_array_is_admitted() {
    // `[new Array]` — no `()` at all, so the parser never wraps the callee in
    // a `CallExpression`; codegen's `resolve_array_alloc_call` requires an
    // actual `Call` node and declines this (`emit/call.rs:5462-5475`), so
    // there is no collision behind a refusal here. Over-refusing this was a
    // round-2 finding: an earlier version matched a bare `Identifier` callee
    // directly and refused this legitimate program.
    let parenless_new_array = Expression::NewExpression(Box::new(kali_ast::NewExpression {
        callee: Expression::Identifier("Array".to_string()),
        args: vec![],
    }));

    assert_resolution!(
        one_element_literal_statements(parenless_new_array),
        diagnostics: 0
    );
}

#[test]
fn a_one_element_literal_of_a_parenless_new_global_this_uint8array_is_admitted() {
    // `[new globalThis.Uint8Array]` — same reasoning as the plain-identifier
    // parenless form above, for the member-callee spelling.
    let parenless_new_uint8array = Expression::NewExpression(Box::new(kali_ast::NewExpression {
        callee: Expression::MemberExpression(Box::new(MemberExpression {
            object: Expression::Identifier("globalThis".to_string()),
            property: Some("Uint8Array".to_string()),
            computed_index: None,
        })),
        args: vec![],
    }));

    assert_resolution!(
        one_element_literal_statements(parenless_new_uint8array),
        diagnostics: 0
    );
}

/// Asserts `statements` resolves to exactly one diagnostic and that its
/// message contains the `ALLOC_IN_LITERAL` needle
/// (`crates/kali_cli/tests/cases/runtime/inline_allocation_value_position.toml`'s
/// `[constants]` block), shared by every regression test below so each one
/// states only its own AST shape.
fn assert_refused_by_the_allocation_literal_gate(statements: Vec<Statement>) {
    let result = assert_resolution!(statements, diagnostics: 1);
    assert!(
        result.diagnostics[0]
            .message
            .contains("lowers to the same node as the allocation itself"),
        "unexpected message: {}",
        result.diagnostics[0].message
    );
}

// --- Round 1 and round 2 regression coverage ---
//
// Rounds 1-2 each closed a live miscompile in `expression_is_array_allocation`
// / `unwrap_transparent` that had NO unit test of its own (only the round-2
// paren spellings above got one at the time). Per the round-3 ruling: an
// untested guarantee is not established, so every spelling closed in rounds
// 1-2 gets its own test here, plus one admitted control that shares the same
// `globalThis`-qualified machinery but must NOT refuse.

#[test]
fn a_one_element_literal_of_a_global_this_uint8array_allocation_is_refused() {
    // `[new globalThis.Uint8Array(3)]` — round 1's first closure.
    // `is_array_like_constructor`'s own second, `globalThis`-qualified branch
    // (`emit/call.rs:5510-5521`).
    let allocation = Expression::NewExpression(Box::new(kali_ast::NewExpression {
        callee: Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("globalThis".to_string()),
                property: Some("Uint8Array".to_string()),
                computed_index: None,
            })),
            args: vec![Expression::Literal(LiteralValue::Number(3.0))],
        })),
        args: vec![],
    }));

    assert_refused_by_the_allocation_literal_gate(one_element_literal_statements(allocation));
}

#[test]
fn a_one_element_literal_of_a_bracket_global_this_uint8array_call_is_refused() {
    // `[globalThis["Uint8Array"](3)]` — the bracket-literal spelling of the
    // same `globalThis`-qualified member, as a bare call (no `new`).
    // `MemberExpression::property`'s own doc comment: a computed access is
    // populated with `Some(name)` too when the parser can read the index
    // statically, which is exactly what `is_global_this_uint8array` relies on
    // to cover both spellings with one check.
    let allocation = Expression::CallExpression(Box::new(CallExpression {
        callee: Expression::MemberExpression(Box::new(MemberExpression {
            object: Expression::Identifier("globalThis".to_string()),
            property: Some("Uint8Array".to_string()),
            computed_index: Some(Box::new(Expression::Literal(LiteralValue::String(
                "Uint8Array".to_string(),
            )))),
        })),
        args: vec![Expression::Literal(LiteralValue::Number(3.0))],
    }));

    assert_refused_by_the_allocation_literal_gate(one_element_literal_statements(allocation));
}

#[test]
fn a_one_element_literal_of_an_allocation_wrapped_in_as_is_refused() {
    // `[new Array(3) as number[]]` — round 1's second closure.
    // `crates/kali_hir/src/lowering/expression.rs:210` erases `TypeAssertion`
    // outright, and the parser accepts `as` in a plain `.js` file too.
    let allocation = Expression::TypeAssertion(Box::new(kali_ast::TypeAssertion {
        type_name: "number[]".to_string(),
        expression: Box::new(Expression::NewExpression(Box::new(
            kali_ast::NewExpression {
                callee: Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::Identifier("Array".to_string()),
                    args: vec![Expression::Literal(LiteralValue::Number(3.0))],
                })),
                args: vec![],
            },
        ))),
    }));

    assert_refused_by_the_allocation_literal_gate(one_element_literal_statements(allocation));
}

#[test]
fn a_one_element_literal_of_an_allocation_wrapped_in_satisfies_is_refused() {
    // `[new Array(3) satisfies unknown]` — round 1's third closure.
    // `crates/kali_hir/src/lowering/expression.rs:211` erases
    // `SatisfiesExpression` outright, same as `TypeAssertion`.
    let allocation = Expression::SatisfiesExpression(Box::new(kali_ast::SatisfiesExpression {
        type_name: "unknown".to_string(),
        expression: Box::new(Expression::NewExpression(Box::new(
            kali_ast::NewExpression {
                callee: Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::Identifier("Array".to_string()),
                    args: vec![Expression::Literal(LiteralValue::Number(3.0))],
                })),
                args: vec![],
            },
        ))),
    }));

    assert_refused_by_the_allocation_literal_gate(one_element_literal_statements(allocation));
}

#[test]
fn a_one_element_literal_of_a_paren_wrapped_optional_chain_global_this_uint8array_is_refused() {
    // `[new (globalThis?.Uint8Array)(3)]` — the optional-chain escape found
    // and closed while verifying round 2: `unwrap_transparent`'s
    // `OptionalChainExpression` arm falls out of the same generic recursion
    // that closes the plain-paren forms, since `new` requires the chain to be
    // parenthesized to be syntactically valid at all.
    let optional_chain_global_this_uint8array =
        Expression::OptionalChainExpression(Box::new(OptionalChainExpression {
            inner: Box::new(OptionalChainInner::NonNull {
                object: Box::new(Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: Some("Uint8Array".to_string()),
                    computed_index: None,
                }))),
                optional: true,
            }),
        }));
    let allocation = Expression::NewExpression(Box::new(kali_ast::NewExpression {
        callee: Expression::CallExpression(Box::new(CallExpression {
            callee: parenthesized(optional_chain_global_this_uint8array),
            args: vec![Expression::Literal(LiteralValue::Number(3.0))],
        })),
        args: vec![],
    }));

    assert_refused_by_the_allocation_literal_gate(one_element_literal_statements(allocation));
}

#[test]
fn a_one_element_literal_of_a_global_this_array_call_is_admitted() {
    // `[globalThis.Array(3)]` — the boundary `is_global_this_uint8array` pins:
    // codegen's `is_array_like_constructor` special-cases a `globalThis`
    // qualifier ONLY for `Uint8Array` (`emit/call.rs:5510-5521`); a
    // `globalThis`-qualified `Array` has no such branch and is not an
    // allocation shape at all, so this must stay silent even though it shares
    // every other piece of the `globalThis`-qualified machinery.
    let non_allocation = Expression::CallExpression(Box::new(CallExpression {
        callee: Expression::MemberExpression(Box::new(MemberExpression {
            object: Expression::Identifier("globalThis".to_string()),
            property: Some("Array".to_string()),
            computed_index: None,
        })),
        args: vec![Expression::Literal(LiteralValue::Number(3.0))],
    }));

    assert_resolution!(one_element_literal_statements(non_allocation), diagnostics: 0);
}
