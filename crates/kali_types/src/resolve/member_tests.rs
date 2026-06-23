use crate::test_support::*;
use crate::*;
use kali_ast::{DecoratedExpression, Expression, LiteralValue, ParenthesizedExpression};

#[test]
fn test_member_access_bracketed_name_for_env_snapshot_materialization() {
    let expr = kali_ast::MemberExpression {
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                object: Expression::Identifier("globalThis".to_string()),
                property: "Deno".to_string(),
            })),
            property: "env".to_string(),
        })),
        property: "toObject".to_string(),
    };

    assert_eq!(
        TypeContext::member_access_name(&expr).as_deref(),
        Some("globalThis.Deno.env.toObject")
    );
    assert_eq!(
        TypeContext::member_access_name_bracketed(&expr).as_deref(),
        Some(r#"globalThis["Deno"]["env"]["toObject"]"#)
    );

    let mixed_expr = kali_ast::MemberExpression {
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: Expression::Identifier("Deno".to_string()),
            property: "env".to_string(),
        })),
        property: "toObject".to_string(),
    };

    assert_eq!(
        TypeContext::member_access_name(&mixed_expr).as_deref(),
        Some("Deno.env.toObject")
    );
    assert_eq!(
        TypeContext::member_access_name_bracketed(&mixed_expr).as_deref(),
        Some(r#"Deno["env"]["toObject"]"#)
    );

    let wrapped_object = Expression::DecoratedExpression(DecoratedExpression {
        expression: Box::new(Expression::ParenthesizedExpression(Box::new(
            ParenthesizedExpression {
                expression: Box::new(Expression::MemberExpression(Box::new(
                    kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    },
                ))),
            },
        ))),
    });
    let sequence_wrapped_object = sequence_expression(vec![
        Expression::Literal(LiteralValue::Number(0.0)),
        wrapped_object,
    ]);

    assert_eq!(
        TypeContext::member_object_name(&sequence_wrapped_object).as_deref(),
        Some("Deno")
    );

    let wrapped_expr = kali_ast::MemberExpression {
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: sequence_wrapped_object,
            property: "env".to_string(),
        })),
        property: "toObject".to_string(),
    };

    assert_eq!(
        TypeContext::member_access_name(&wrapped_expr).as_deref(),
        Some("globalThis.Deno.env.toObject")
    );
    assert_eq!(
        TypeContext::member_access_name_bracketed(&wrapped_expr).as_deref(),
        Some(r#"globalThis["Deno"]["env"]["toObject"]"#)
    );
}

#[test]
fn test_member_access_bracketed_name_for_permission_escalation() {
    let expr = kali_ast::MemberExpression {
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: Expression::Identifier("Deno".to_string()),
            property: "permissions".to_string(),
        })),
        property: "request".to_string(),
    };

    assert_eq!(
        TypeContext::member_access_name(&expr).as_deref(),
        Some("Deno.permissions.request")
    );
    assert_eq!(
        TypeContext::member_access_name_bracketed(&expr).as_deref(),
        Some(r#"Deno["permissions"]["request"]"#)
    );
}
