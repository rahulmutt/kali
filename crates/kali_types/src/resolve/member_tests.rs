use crate::test_support::*;
use crate::*;
use kali_ast::{DecoratedExpression, Expression, LiteralValue, ParenthesizedExpression};
use kali_error::_error_codes::e5;

#[test]
fn test_member_access_bracketed_name_for_env_snapshot_materialization() {
    let expr = kali_ast::MemberExpression {
        computed_index: None,
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            computed_index: None,
            object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                computed_index: None,
                object: Expression::Identifier("globalThis".to_string()),
                property: Some("Deno".to_string()),
            })),
            property: Some("env".to_string()),
        })),
        property: Some("toObject".to_string()),
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
        computed_index: None,
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            computed_index: None,
            object: Expression::Identifier("Deno".to_string()),
            property: Some("env".to_string()),
        })),
        property: Some("toObject".to_string()),
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
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: Some("Deno".to_string()),
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
        computed_index: None,
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            computed_index: None,
            object: sequence_wrapped_object,
            property: Some("env".to_string()),
        })),
        property: Some("toObject".to_string()),
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
        computed_index: None,
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            computed_index: None,
            object: Expression::Identifier("Deno".to_string()),
            property: Some("permissions".to_string()),
        })),
        property: Some("request".to_string()),
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

fn e5506_messages(source: &str) -> Vec<String> {
    let statements = parse_statements(source);
    let mut ctx = TypeContext::new();
    ctx.resolve_statements(&statements)
        .diagnostics
        .into_iter()
        .filter(|d| d.code == Some(e5::FEATURE_UNAVAILABLE as u32))
        .map(|d| d.message)
        .collect()
}

const COMPUTED: &str = "computed member access `o[k]` is unavailable";
const STRING: &str = "indexing a string `s[i]` is unavailable";

#[test]
fn a_const_literal_key_folds_and_the_checker_admits_the_read_and_the_store() {
    assert!(e5506_messages("const o = {a:1, b:2}; const k = \"b\"; console.log(o[k]);").is_empty());
    assert!(
        e5506_messages("const o = {a:1, b:2}; const k = \"b\"; o[k] = 8; console.log(o.b);")
            .is_empty()
    );
    assert!(e5506_messages("const o = {1: \"one\"}; const k = 1; console.log(o[k]);").is_empty());
}

#[test]
fn a_let_var_or_parameter_key_refuses_once_with_the_shared_message() {
    for source in [
        "const o = {a:1, b:2}; let k = \"b\"; console.log(o[k]);",
        "const o = {a:1, b:2}; var k = \"b\"; console.log(o[k]);",
        "const o = {index: 9, i: 7}; let i = 1; console.log(o[i]);",
        "const o = {index: 9, i: 7}; let i = 1; console.log(o[i + 0]);",
        "const o = {a:1, b:2}; let k = \"b\"; o[k] = 8;",
        // A PARAMETER key over a module-scope object: the receiver is a free
        // module reference `is_structural_runtime_array` fails closed on, and
        // `k` is no `const`. Measured on this branch: `check` was silent
        // before this task and `run` already refused with this same message —
        // the twin gap this gate closes.
        "const o = {a: 1, b: 2}; function f(k) { return o[k]; } console.log(f(\"b\"));",
    ] {
        let messages = e5506_messages(source);
        assert_eq!(
            messages.len(),
            1,
            "{source}: exactly one refusal, got {messages:?}"
        );
        assert!(messages[0].contains(COMPUTED), "{source}: {messages:?}");
    }
}

#[test]
fn a_string_receiver_refuses_in_both_spellings() {
    let literal = e5506_messages("const s = \"abc\"; console.log(s[1]);");
    assert!(literal.iter().any(|m| m.contains(STRING)), "{literal:?}");
    let folded = e5506_messages("const s = \"abc\"; const k = 1; console.log(s[k]);");
    assert!(folded.iter().any(|m| m.contains(STRING)), "{folded:?}");
}

#[test]
fn an_array_literal_element_store_refuses_even_with_a_literal_index() {
    let messages = e5506_messages("const a = [5, 6]; a[1] = 9; console.log(a[1]);");
    assert!(
        messages
            .iter()
            .any(|m| m.contains("mutating a literal array")),
        "{messages:?}"
    );
    assert!(
        !messages.iter().any(|m| m.contains(COMPUTED)),
        "one owner per refusal: {messages:?}"
    );
}

#[test]
fn the_runtime_lanes_are_still_admitted() {
    for source in [
        "const a = new Array(3); let i = 1; console.log(a[i]);",
        "const a = new Array(3); let i = 1; a[i] = 4;",
        "const o = {a: 1, b: 2}; for (const k in o) { console.log(o[k]); }",
        // 2026-09-09 re-pin: the brief listed this among the REFUSALS, but a
        // subscripted parameter is proven an array binding by `repr_infer`
        // and registered structurally at function entry — exactly the set
        // codegen's emitter registers from the same proof — so codegen emits
        // a dynamic array read for it. Measured on this branch:
        // `function f(a, k) { return a[k]; }` called with a `new Array(2)`
        // prints its element under `kali run` (6), and this dead-function
        // source runs clean. Refusing it at `check` would kill a lane `run`
        // admits (binding constraint 8's defect direction). The OBJECT
        // spelling is refused, but by the pre-existing
        // `reject_nonuniform_forin_key_object_access` gate at the call site.
        "function f(o, k) { return o[k]; }",
    ] {
        let messages = e5506_messages(source);
        assert!(
            !messages.iter().any(|m| m.contains(COMPUTED)),
            "{source}: {messages:?}"
        );
    }
}
