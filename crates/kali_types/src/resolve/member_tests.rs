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
    // Both spellings that carry a static NAME — the literal index the parser
    // reads, and the `const` index that folds — are this gate's, and each
    // refuses exactly once. Review round 1, Important 1: the folded spelling
    // was check-clean while `run` refused it, because the gate handed every
    // nameless index to `gate_nameless_computed_member` and that gate folds.
    for source in [
        "const a = [5, 6]; a[1] = 9; console.log(a[1]);",
        "const a = [5, 6]; const i = 1; a[i] = 9; console.log(a[1]);",
    ] {
        let messages = e5506_messages(source);
        assert!(
            messages
                .iter()
                .any(|m| m.contains("mutating a literal array")),
            "{source}: {messages:?}"
        );
        assert!(
            !messages.iter().any(|m| m.contains(COMPUTED)),
            "{source}: one owner per refusal: {messages:?}"
        );
    }
    // A nameless index that does NOT fold stays with the nameless gate, still
    // exactly once.
    let unfoldable = e5506_messages("const a = [5, 6]; let i = 1; a[i] = 9;");
    assert_eq!(unfoldable.len(), 1, "{unfoldable:?}");
    assert!(unfoldable[0].contains(COMPUTED), "{unfoldable:?}");
    // A literal-array READ with a folded index keeps its lane: measured,
    // `run` prints the element.
    assert!(e5506_messages("const a = [5, 6]; const i = 1; console.log(a[i]);").is_empty());
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

/// The carve-out coordination ruling (2026-09-09): a MATERIALIZED object gives
/// `reject_nonuniform_forin_key_object_access` a shape, and before the
/// carve-out that gate claimed a folded key as a "general dynamic string-keyed
/// access" — so `const o = {…}; o.b = 8; const k = "b"; o[k]` refused at both
/// `check` and `run` even though the fold's whole claim is that it IS the dot
/// spelling. The second half is what keeps the carve-out from becoming a hole:
/// a NON-folding key over the very same materialized shape still refuses.
#[test]
fn a_folded_key_over_a_materialized_object_is_admitted_and_a_dynamic_one_still_refuses() {
    const DYNAMIC: &str = "computed key access `obj[k]` where the key is not a `for..in` key";

    for source in [
        "const o = {a:1, b:2}; o.b = 8; const k = \"b\"; console.log(o[k]);",
        "let o = {a:1, b:2}; const k = \"b\"; o[k] = 8; console.log(o.b);",
        "let o = {a: 6}; const k = \"a\"; o[k] = 7; console.log(o.a);",
    ] {
        assert!(
            e5506_messages(source).is_empty(),
            "{source}: a folded key is a static name, not a dynamic key: {:?}",
            e5506_messages(source)
        );
    }

    for source in [
        "const o = {a:1, b:2}; o.b = 8; let k = \"b\"; console.log(o[k]);",
        "const o = {a:1, b:2}; o.b = 8; let k = \"b\"; o[k] = 9;",
    ] {
        let messages = e5506_messages(source);
        assert!(
            messages.iter().any(|m| m.contains(DYNAMIC)),
            "{source}: a non-folding key over a proven shape still fails closed: {messages:?}"
        );
    }
}

/// Review round 1, Important 1 and 3 (2026-09-09): three admit-list holes, all
/// the same shape — `check` clean while `run` refused, constraint 8's
/// dangerous direction. Each program here was measured in both twins before
/// and after the fix; the readings are in the task report.
///
/// 1-2. `string_element_array_binding` is `repr_table.is_array_binding`
///    narrowed to a `Repr::String` element axis, not a structural mirror, so
///    it admitted an array LITERAL — which codegen's `array_bindings` set
///    never holds. The store half had additionally LOST the refusal
///    `reject_literal_array_unfoldable_mutation` used to give it, since that
///    gate now hands a nameless index to `gate_nameless_computed_member`.
/// 3. `const_index_name`'s walk tunnelled past a shadowing parameter, because
///    only foldable `const`s enter `const_index_names` and the walk looked for
///    an entry rather than for a declaration.
#[test]
fn the_admit_list_does_not_reach_past_the_lanes_codegen_actually_has() {
    for source in [
        // A string-element array LITERAL, read and store: codegen registers no
        // `array_bindings` entry for a literal, so both refuse at `run`.
        "const a = [\"x\", \"y\"]; let i = 0; console.log(a[i]);",
        "const a = [\"x\", \"y\"]; let i = 0; a[i] = \"z\";",
        // A shadowing parameter must not fold the outer `const` of the same
        // name.
        "const k = \"b\"; const o = {a:1, b:2}; function f(k) { return o[k]; }",
        // Nor may the walk leave the function being emitted at all: codegen's
        // `bindings` map is per-`FunctionEmitter`, so a module-scope `const`
        // is invisible to a named function's fold and `run` refuses this.
        "const k = \"b\"; const o = {a:1, b:2}; function f() { return o[k]; } console.log(f());",
    ] {
        let messages = e5506_messages(source);
        assert!(
            messages.iter().any(|m| m.contains(COMPUTED)),
            "{source}: {messages:?}"
        );
    }

    // The lanes codegen DOES have are untouched: a `new Array(n)` binding
    // (structural) still takes a runtime index in both directions, and a
    // string-element array reached through one keeps its lane.
    for source in [
        "const a = new Array(2); a[0] = \"x\"; let i = 0; console.log(a[i]);",
        "const a = new Array(2); let i = 0; a[i] = \"x\";",
    ] {
        let messages = e5506_messages(source);
        assert!(
            !messages.iter().any(|m| m.contains(COMPUTED)),
            "{source}: {messages:?}"
        );
    }

    // The stop is a stop, not a blanket refusal: the same fold still works at
    // module scope, and inside a function when the `const` is declared there.
    for source in [
        "const k = \"b\"; const o = {a:1, b:2}; console.log(o[k]);",
        "function f() { const k = \"b\"; const o = {a:1, b:2}; return o[k]; } console.log(f());",
    ] {
        assert!(
            e5506_messages(source).is_empty(),
            "{source}: {:?}",
            e5506_messages(source)
        );
    }
}

/// Task 6 review, Important (2026-09-09): a name declared twice in one
/// function does not fold. Measured before the fix, both `check`-clean and
/// exit 0 with a SILENT WRONG value: with `const k = "b"; const o = {a:1,b:2};
/// if (true) { const k = "a"; }`, `o[k] = 8; console.log(o.b)` printed `2`
/// (node `8`) — the store landed on field `a` — and `console.log(o[k])`
/// printed `1` (node `2`). This gate's walk is scope-precise and resolved
/// `"b"`, while `repr_infer`'s flat per-function table and codegen's flat
/// per-`FunctionEmitter` `bindings` both resolved the LAST declaration, `"a"`.
/// That is R-59's defining symptom — a name that hits a real, wrong property —
/// so the fold declines instead. Both passes must decline: a gate that
/// admitted here while `repr_infer` declined would leave the store without
/// materialization evidence and make `check` clean where `run` refuses.
#[test]
fn a_name_declared_twice_in_one_function_does_not_fold() {
    for source in [
        // The measured shape: an outer fold name shadowed by a block-scoped
        // `const`, read and store.
        "const k = \"b\"; const o = {a:1, b:2}; if (true) { const k = \"a\"; } console.log(o[k]);",
        "const k = \"b\"; const o = {a:1, b:2}; if (true) { const k = \"a\"; } o[k] = 8;",
        // The shadow need not itself be foldable: a `let`, a `var` or a
        // non-literal `const` makes the flat tables just as ambiguous.
        "const k = \"b\"; const o = {a:1, b:2}; if (true) { let k = 1; } console.log(o[k]);",
        "const k = \"b\"; const o = {a:1, b:2}; if (true) { const k = o; } console.log(o[k]);",
        // Same rule inside a function.
        "function f() { const k = \"b\"; const o = {a:1, b:2}; if (true) { const k = \"a\"; } return o[k]; }",
    ] {
        let messages = e5506_messages(source);
        assert!(
            messages.iter().any(|m| m.contains(COMPUTED)),
            "{source}: a shadowed fold name must refuse: {messages:?}"
        );
    }

    // The poison is keyed on the NAME within one function, not on block
    // nesting: an unshadowed `const` declared in a nested block still folds,
    // and so does a same-named `const` in a DIFFERENT function (each flat
    // table is per-function, so there is no ambiguity to poison).
    for source in [
        "const o = {a:1, b:2}; if (true) { const k = \"b\"; console.log(o[k]); }",
        "const o = {a:1, b:2}; const k = \"b\"; console.log(o[k]);",
        "function f() { const k = \"b\"; const o = {a:1, b:2}; return o[k]; } function g() { const k = \"a\"; return k; } console.log(f());",
    ] {
        let messages = e5506_messages(source);
        assert!(
            !messages.iter().any(|m| m.contains(COMPUTED)),
            "{source}: an unambiguous name still folds: {messages:?}"
        );
    }
}

/// Task 7's end-to-end gate run (2026-09-09): narrowing the admit list to the
/// STRUCTURAL registries was right, but that registry could not see through a
/// method call chained onto `new Array(n)` — the parser hangs the whole chain
/// under one `new` — so a legitimately-registered runtime array fell out of the
/// admit list in that ONE spelling and `check` refused what `run` admits. It
/// killed `tests/fixtures/benchmarks/spectral-norm-benchmark-v1.ts`. Codegen
/// registers it at `emit/control_flow.rs`'s "`const u = new Array(n).fill(v)`"
/// declarator arm; this pins every spelling of that same array against it.
#[test]
fn a_chained_array_allocation_is_the_same_runtime_array_as_the_statement_form() {
    for source in [
        // The chained spellings codegen's declarator arm registers.
        "const u = new Array(3).fill(1); let i = 0; console.log(u[i]);",
        "const u = Array(3).fill(1); let i = 0; console.log(u[i]);",
        "const u = (new Array(3)).fill(1); let i = 0; console.log(u[i]);",
        // The statement spellings, which always worked — kept as the
        // side-by-side the regression was found by.
        "const u = new Array(3); u.fill(1); let i = 0; console.log(u[i]);",
        "const u = new Array(3); u[0] = 1; let i = 0; console.log(u[i]);",
        // `.fill` on an already-structural binding is codegen's other
        // receiver, and the result is an array binding too.
        "const u = new Array(3); const v = u.fill(1); let i = 0; console.log(v[i]);",
    ] {
        assert!(
            e5506_messages(source).is_empty(),
            "{source}: {:?}",
            e5506_messages(source)
        );
    }

    for source in [
        // Negative control: the admit list is still not vacuous. An array
        // LITERAL never enters codegen's `array_bindings`.
        "const a = [\"x\", \"y\"]; let i = 0; console.log(a[i]);",
        // A `.fill` chained onto another `.fill` is NOT an allocation for
        // codegen's receiver test either (`array_fill_call_parts` wants an
        // alloc call or an existing binding), and `run` refuses both
        // spellings — so neither may be admitted here.
        "const u = new Array(3).fill(1).fill(2); let i = 0; console.log(u[i]);",
        "const u = Array(3).fill(1).fill(2); let i = 0; console.log(u[i]);",
    ] {
        let messages = e5506_messages(source);
        assert!(
            messages.iter().any(|m| m.contains(COMPUTED)),
            "{source}: {messages:?}"
        );
    }
}

// Array-return lane (spec 2026-10-02 A1).
#[test]
fn computed_index_on_call_bound_array_is_admitted() {
    let source = "function f() { return [1, 2, 3]; } function main() { const a = f(); let i = 2; console.log(a[i]); } main();";
    assert!(
        e5506_messages(source).is_empty(),
        "{:?}",
        e5506_messages(source)
    );
}

#[test]
fn computed_index_on_direct_array_returning_call_is_admitted() {
    let source = "function f() { return [1, 2, 3]; } function main() { let i = 1; console.log(f()[i]); } main();";
    assert!(
        e5506_messages(source).is_empty(),
        "{:?}",
        e5506_messages(source)
    );
}

#[test]
fn computed_index_on_scalar_call_still_refuses() {
    let source =
        "function f() { return 3; } function main() { let i = 1; console.log(f()[i]); } main();";
    assert!(e5506_messages(source).iter().any(|m| m.contains(COMPUTED)));
}

#[test]
fn computed_index_on_a_shadowed_callee_still_refuses() {
    for source in [
        "function f() { return [1, 2, 3]; } function main(f) { let i = 1; console.log(f()[i]); } main(() => 1);",
        "function f() { return [1, 2, 3]; } function main() { const f = () => 1; let i = 1; console.log(f()[i]); } main();",
    ] {
        let messages = e5506_messages(source);
        assert!(messages.iter().any(|m| m.contains(COMPUTED)), "{source}: {messages:?}");
    }
}

#[test]
fn a_nested_array_returning_declaration_is_admitted_for_both_lanes() {
    for source in [
        "function main() { function f() { return [1, 2, 3]; } let i = 1; console.log(f()[i]); } main();",
        "function main() { function f() { return [1, 2, 3]; } let i = 1; const a = f(); console.log(a[i]); } main();",
    ] {
        let messages = e5506_messages(source);
        assert!(messages.is_empty(), "{source}: {messages:?}");
    }
}

#[test]
fn a_later_shadow_in_the_caller_refuses() {
    for source in [
        "function f(){return [1,2,3];} function main(){ let i=1; console.log(f()[i]); var f = () => [4]; } main();",
        "function f(){return [1,2,3];} function main(){ let i=1; console.log(f()[i]); const f = () => 1; } main();",
    ] {
        let messages = e5506_messages(source);
        assert!(messages.iter().any(|m| m.contains(COMPUTED)), "{source}: {messages:?}");
    }
}

const NEG: &str = "a negative index on a runtime array is unavailable";
const MUT: &str = "on a runtime array is unavailable in the current phase";
const LEN: &str = "assigning to `.length` of a runtime array is unavailable";

fn any_contains(messages: &[String], needle: &str) -> bool {
    messages.iter().any(|message| message.contains(needle))
}

#[test]
fn a_negative_literal_index_on_a_filled_array_refuses() {
    let messages = e5506_messages(
        "function main(){ const a = new Array(3).fill(4); console.log(a[-1]); } main();",
    );
    assert!(any_contains(&messages, NEG), "{messages:?}");
}

#[test]
fn a_negative_literal_index_store_on_a_filled_array_refuses() {
    let messages =
        e5506_messages("function main(){ const a = new Array(3).fill(4); a[-1] = 9; } main();");
    assert!(any_contains(&messages, NEG), "{messages:?}");
}

#[test]
fn each_length_changing_method_on_a_filled_array_refuses() {
    for call in [
        "a.push(4)",
        "a.pop()",
        "a.shift()",
        "a.unshift(1)",
        "a.splice(0, 1)",
    ] {
        let source =
            format!("function main(){{ const a = new Array(3).fill(4); {call}; }} main();");
        let messages = e5506_messages(&source);
        assert!(any_contains(&messages, MUT), "{call}: {messages:?}");
    }
}

#[test]
fn a_length_write_on_a_filled_array_refuses_for_every_operator() {
    for write in ["a.length = 1", "a.length -= 1"] {
        let source =
            format!("function main(){{ const a = new Array(3).fill(4); {write}; }} main();");
        let messages = e5506_messages(&source);
        assert!(any_contains(&messages, LEN), "{write}: {messages:?}");
    }
}

#[test]
fn in_bounds_access_and_a_growable_push_do_not_refuse() {
    for source in [
        "function main(){ const a = new Array(3).fill(4); a[2] = 1; console.log(a[2], a.length); } main();",
        "function main(){ const a = [1,2,3]; a.push(4); console.log(a.length); } main();",
        "function main(){ const a = new Array(3).fill(4); console.log(a[-0]); } main();",
        "function main(){ const a = new Array(3).fill(4); const i = 1; console.log(a[-i]); } main();",
    ] {
        let messages = e5506_messages(source);
        assert!(
            !any_contains(&messages, NEG)
                && !any_contains(&messages, MUT)
                && !any_contains(&messages, LEN),
            "{source}: {messages:?}"
        );
    }
}

#[test]
fn wrapped_receivers_and_a_string_method_key_refuse_like_the_bare_spelling() {
    for (shape, needle) in [
        ("(a).push(4)", MUT),
        ("a?.push(1)", MUT),
        ("a[\"push\"](1)", MUT),
        ("(a as number[]).push(1)", MUT),
        ("(a).pop()", MUT),
        ("(a).length = 1", LEN),
        ("(a)[-1]", NEG),
        ("(a as number[])[-1] = 2", NEG),
    ] {
        let source = format!(
            "function main(){{ const a = new Array(3).fill(4); {shape}; console.log(a.length); }} main();"
        );
        let messages = e5506_messages(&source);
        assert!(any_contains(&messages, needle), "{shape}: {messages:?}");
    }
}

#[test]
fn a_wrapped_growable_receiver_does_not_refuse() {
    for shape in [
        "(a).push(4)",
        "a?.push(4)",
        "a[\"push\"](4)",
        "(a as number[]).push(4)",
    ] {
        let source = format!(
            "function main(){{ const a = [1,2,3]; {shape}; console.log(a.length); }} main();"
        );
        let messages = e5506_messages(&source);
        assert!(!any_contains(&messages, MUT), "{shape}: {messages:?}");
    }
}

const LIT: &str = "on a literal array is unavailable in the current phase";
const LIT_LEN: &str = "assigning to `.length` of a literal array is unavailable";

#[test]
fn each_in_place_mutator_on_a_literal_array_refuses_at_top_level_and_in_a_function() {
    for call in [
        "a.push(4)",
        "a.pop()",
        "a.shift()",
        "a.unshift(0)",
        "a.splice(0, 1)",
        "a.reverse()",
        "a.sort()",
        "a.fill(9)",
        "a.copyWithin(0, 2)",
    ] {
        for source in [
            format!("const a = [1,2,3]; {call}; console.log(a[0]);"),
            format!("let a = [1,2,3]; {call}; console.log(a[0]);"),
            format!("var a = [1,2,3]; {call}; console.log(a[0]);"),
        ] {
            let messages = e5506_messages(&source);
            assert!(any_contains(&messages, LIT), "{source}: {messages:?}");
        }
        // An in-function `push` promotes the literal to the growable lane,
        // which keeps node's output (guarded by the do-not-refuse test).
        if call != "a.push(4)" {
            let source = format!(
                "function main(){{ const a = [1,2,3]; {call}; console.log(a[0]); }} main();"
            );
            let messages = e5506_messages(&source);
            assert!(any_contains(&messages, LIT), "{source}: {messages:?}");
        }
    }
}

#[test]
fn a_length_write_on_a_literal_array_refuses_for_every_operator() {
    for write in ["a.length = 1", "a.length -= 1"] {
        for source in [
            format!("const a = [1,2,3]; {write};"),
            format!("function main(){{ const a = [1,2,3]; {write}; }} main();"),
        ] {
            let messages = e5506_messages(&source);
            assert!(any_contains(&messages, LIT_LEN), "{source}: {messages:?}");
        }
    }
}

#[test]
fn wrapped_nameless_and_captured_literal_receivers_refuse() {
    for source in [
        "const a = [1,2,3]; (a).pop();",
        "const a = [1,2,3]; (a as number[]).pop();",
        "const a = [1,2,3]; a[\"pop\"]();",
        "const a = [1,2,3]; a?.pop();",
        "const a = [1,2,3]; a.push?.(4);",
        "console.log([1,2].push(3));",
        "function main(){ const a = [\"x\",\"y\"]; a.reverse(); } main();",
        "function main(){ const a = [1,2,3]; const f = () => a.pop(); f(); } main();",
    ] {
        let messages = e5506_messages(source);
        assert!(any_contains(&messages, LIT), "{source}: {messages:?}");
    }
}

#[test]
fn the_plain_lane_refuses_the_reorderers_and_an_optional_call_but_not_fill() {
    for call in [
        "a.reverse()",
        "a.sort()",
        "a.copyWithin(0, 2)",
        "a.push?.(1)",
    ] {
        let source =
            format!("function main(){{ const a = new Array(3).fill(4); {call}; }} main();");
        let messages = e5506_messages(&source);
        assert!(any_contains(&messages, MUT), "{call}: {messages:?}");
        assert!(!any_contains(&messages, LIT), "{call}: {messages:?}");
    }
    let messages =
        e5506_messages("function main(){ const a = new Array(3).fill(4); a.fill(5); } main();");
    assert!(
        !any_contains(&messages, MUT) && !any_contains(&messages, LIT),
        "{messages:?}"
    );
}

#[test]
fn an_object_element_literal_gets_exactly_one_mutator_refusal() {
    let messages = e5506_messages("function main(){ const a = [{v:1},{v:2}]; a.pop(); } main();");
    let count = messages
        .iter()
        .filter(|m| m.contains(MUT) || m.contains(LIT))
        .count();
    assert_eq!(count, 1, "{messages:?}");
}

#[test]
fn growable_push_reads_and_non_mutators_on_a_literal_do_not_refuse() {
    for source in [
        "function main(){ const a = [1,2,3]; a.push(4); console.log(a.length); } main();",
        "function main(){ const a = []; for (let i = 0; i < 3; i++) a.push(i); } main();",
        "function main(){ const a = [1,2]; a.push(3); console.log(a.join(\"-\")); } main();",
        "function main(){ const a = [1,2,3]; (a).push(4); } main();",
        "const a = [3,1,2]; const b = a.slice(1); console.log(b[0]);",
        "const a = [3,1,2]; console.log(a.indexOf(2));",
        "const a = [1,2,3]; console.log(a[1]);",
        "const s = { n: 0, push(v){ this.n += v; } }; s.push(2);",
    ] {
        let messages = e5506_messages(source);
        assert!(
            !any_contains(&messages, LIT) && !any_contains(&messages, LIT_LEN),
            "{source}: {messages:?}"
        );
    }
}

#[test]
fn a_push_mixed_with_pop_in_a_function_refuses_the_pop_in_the_resolve_pass() {
    // Spec A-3: the resolve pass reports this before repr_infer's growable reject.
    let messages =
        e5506_messages("function main(){ const a = [1,2]; a.push(3); a.pop(); } main();");
    assert!(
        messages
            .iter()
            .any(|m| m.contains(LIT) && m.contains("`.pop()`")),
        "{messages:?}"
    );
}

#[test]
fn a_parenthesized_optional_call_callee_on_a_literal_refuses() {
    // Final review C1: `(a.pop)?.()` wraps the optional-call member in a
    // `ParenthesizedExpression`, which the optional-chain gate must see through.
    for source in [
        "const a = [1,2,3]; (a.pop)?.(); console.log(a.length);",
        "const a = [1,2,3]; ((a.push))?.(4);",
        "function main(){ const a = [1,2,3]; (a.sort)?.(); } main();",
    ] {
        let messages = e5506_messages(source);
        assert!(any_contains(&messages, LIT), "{source}: {messages:?}");
    }
    let messages =
        e5506_messages("function main(){ const a = new Array(3).fill(4); (a.pop)?.(); } main();");
    assert!(any_contains(&messages, MUT), "{messages:?}");
}

#[test]
fn an_inner_binding_shadowing_an_outer_literal_is_not_a_literal_receiver() {
    // Final review I2: the NEAREST binding of the name decides, so an inner
    // class instance, object, or parameter named like an outer literal array
    // keeps its own method call. (An object literal with a method is not a
    // shape kali lowers inside a function at all, so it is not listed.)
    for source in [
        "const a = [1,2,3]; class Box { sort(){ return 7; } } function main(){ const a = new Box(); console.log(a.sort()); } main();",
        "const a = [1,2,3]; class Box { push(v){ return v + 1; } } function main(){ const a = new Box(); console.log(a.push(4)); } main();",
        "const a = [1,2,3]; class Box { sort(){ return 7; } } function f(a){ return a.sort(); } console.log(f(new Box()));",
        "const a = [1,2,3]; class Box { sort(){ return 7; } } { const a = new Box(); a.sort(); }",
    ] {
        let messages = e5506_messages(source);
        assert!(
            !any_contains(&messages, LIT) && !any_contains(&messages, LIT_LEN),
            "{source}: {messages:?}"
        );
    }
    // The outer literal itself still refuses, and so does an inner literal
    // that shadows an outer non-literal.
    for source in [
        "const a = [1,2,3]; class Box { sort(){ return 7; } } function main(){ const a = new Box(); a.sort(); } main(); a.pop();",
        "const a = 1; function main(){ const a = [1,2,3]; a.pop(); } main();",
    ] {
        let messages = e5506_messages(source);
        assert!(any_contains(&messages, LIT), "{source}: {messages:?}");
    }
}

const UNRES: &str = "the receiver is a value this program built";

fn unres_count(source: &str) -> usize {
    e5506_messages(source).iter().filter(|m| m.contains(UNRES)).count()
}

#[test]
fn check_refuses_a_missing_method_on_a_const_object_literal_or_program_instance() {
    for source in [
        "const o={k:1}; console.log(o.zork(4));",
        "const o={k:1}; console.log(o[\"zork\"](4));",
        "function main(){ const o={k:1}; console.log(o.zork()); } main();",
        "class C{ f(){return 1;} } const c=new C(); console.log(c.g());",
        "class A{ f(){return 1;} } class B extends A{} const b=new B(); console.log(b.g());",
        "const o={k:1}; o.zork?.();",
    ] {
        assert_eq!(unres_count(source), 1, "{source}");
    }
}

#[test]
fn check_stays_quiet_where_it_cannot_know() {
    for source in [
        // members that exist, on the class or its program base
        "class S { push(v){ return v+1; } } const s=new S(); console.log(s.push(1));",
        "class A{ f(){return 4;} } class B extends A{} const b=new B(); console.log(b.f());",
        "class S { n = 0; cb = () => 1; } const s=new S(); console.log(s.cb());",
        // Object.prototype names
        "const o={k:1}; console.log(o.hasOwnProperty(\"k\"));",
        // a name some assignment writes
        "const o={k:1}; o.f = 5; console.log(o.f());",
        "class S { constructor(){ this.cb = 1; } } const s=new S(); console.log(s.cb());",
        // the base leaves the program
        "class X extends EventTarget{} const x=new X(); x.addEventListener(\"t\", ()=>{});",
        // a class name declared twice has no knowable member set
        "function f(){ class A extends EventTarget{} } class A {} const a=new A(); a.zork();",
        // receivers outside the mirror (run-only, spec §1.1)
        "const s=\"abc\"; console.log(s.zork());",
        "const o={k:1}; const p=o; console.log(p.zork());",
        "let o={k:1}; console.log(o.zork());",
        "function g(x){ return x.zork(); }",
    ] {
        assert_eq!(unres_count(source), 0, "{source}");
    }
}

#[test]
fn the_nearest_binding_wins() {
    // Outer: a class instance that has `zork`. Inner: an object literal that does not.
    let inner_lacks = "class S { zork(){ return 1; } } const o=new S(); function main(){ const o={k:1}; console.log(o.zork()); } main();";
    assert_eq!(unres_count(inner_lacks), 1);
    // Outer: an object literal without `zork`. Inner: a class instance that has it.
    let inner_has = "const o={k:1}; class T { zork(){ return 1; } } function main(){ const o=new T(); console.log(o.zork()); } main();";
    assert_eq!(unres_count(inner_has), 0);
}

#[test]
fn a_literal_array_mutator_gets_one_diagnostic_not_two() {
    let messages = e5506_messages("const a=[1,2,3]; a.pop(); console.log(a.length);");
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(!messages[0].contains(UNRES));
}
