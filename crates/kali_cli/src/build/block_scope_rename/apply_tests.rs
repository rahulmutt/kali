use crate::build::block_scope_rename::rename_block_scoped_bindings;
use crate::build::block_scope_rename::test_support::parse;

fn renamed_json(source: &str) -> (usize, String) {
    let mut statements = parse(source);
    let outcome = rename_block_scoped_bindings(&mut statements);
    (
        outcome.renamed,
        serde_json::to_string(&statements).expect("serialize"),
    )
}

#[test]
fn a_program_that_shadows_nothing_is_unchanged() {
    let source = "function f(a){ let b=a+1; { let c=b*2; return c; } } console.log(f(3));";
    let before = parse(source);
    let mut after = parse(source);
    assert_eq!(rename_block_scoped_bindings(&mut after).renamed, 0);
    assert_eq!(after, before);
}

#[test]
fn declaration_and_every_reference_in_the_block_are_rewritten() {
    let (n, json) = renamed_json("let x=1; { let x=2; console.log(x); } console.log(x);");
    assert_eq!(n, 1);
    assert_eq!(json.matches("\"x{b0}\"").count(), 2, "{json}");
    assert_eq!(json.matches("\"x\"").count(), 2, "{json}");
}

#[test]
fn a_use_before_the_block_declaration_resolves_to_the_block_binding() {
    let (_, json) =
        renamed_json("let x=1; function g(){} { console.log(x); g(); function g(){} let x=2; }");
    assert_eq!(json.matches("\"x{b").count(), 2, "{json}");
    assert_eq!(json.matches("\"g{b").count(), 2, "{json}");
}

#[test]
fn an_inner_function_keeps_its_capture_when_its_block_shadows() {
    let (_, json) =
        renamed_json("function m(){ let x=1; function g(){ { let x=2; } return x; } return g(); }");
    assert_eq!(json.matches("\"x{b0}\"").count(), 1, "{json}");
}

#[test]
fn a_shorthand_property_keeps_its_key() {
    let (_, json) = renamed_json("let x=1; { const x=2; const o={x}; }");
    assert!(json.contains("\"Identifier\":\"x\""), "{json}");
    assert!(json.contains("\"x{b0}\""), "{json}");
}

#[test]
fn a_member_name_and_a_label_are_never_renamed() {
    let (_, json) = renamed_json("let x=1; { const x={x:1}; console.log(x.x); }");
    assert!(json.contains("\"property\":\"x\""), "{json}");
}

#[test]
fn running_twice_is_a_no_op_the_second_time() {
    let mut statements = parse(
        "let x=1; { let x=2; } function a(){ function h(){} } function b(){ function h(){} }",
    );
    rename_block_scoped_bindings(&mut statements);
    let first = statements.clone();
    assert_eq!(rename_block_scoped_bindings(&mut statements).renamed, 0);
    assert_eq!(statements, first);
}

#[test]
fn all_cases_of_a_switch_share_one_block_scope() {
    let (n, json) = renamed_json(
        "let x=1; switch (x) { case 1: let x=2; console.log(x); break; default: console.log(x); }",
    );
    assert_eq!(n, 1);
    // The discriminant is outside the switch block; the declaration and both case-body uses are inside.
    assert_eq!(json.matches("\"x{b0}\"").count(), 3, "{json}");
    assert_eq!(json.matches("\"x\"").count(), 2, "{json}");
}

#[test]
fn an_exported_module_scope_name_is_untouched() {
    let (n, json) = renamed_json("let x=1; { let x=2; console.log(x); } export { x };");
    assert_eq!(n, 1);
    assert_eq!(json.matches("\"x{b0}\"").count(), 2, "{json}");
    assert!(!json.contains("\"local\":\"x{b"), "{json}");
    assert!(!json.contains("\"id\":\"x{b1}"), "{json}");
}

#[test]
fn an_empty_extends_name_is_never_renamed() {
    let mut statements = parse("class A extends {}");
    assert_eq!(rename_block_scoped_bindings(&mut statements).renamed, 0);
}

/// Spec §5.3: a block-shadowed binding `zq` is referenced from every parsable
/// expression and statement position; each must be rewritten, the outer one not.
/// The parser cannot produce try/catch/finally (E5506), rest params (E5506), object spread
/// (E5506), `export let` (parses as garbage) or JSX (not parsed at all), so those positions are
/// covered by the walk's exhaustive match alone.
#[test]
fn every_reference_position_is_rewritten_and_the_outer_binding_is_not() {
    let body = r#"
        let zq = 1;
        let tag = (s, v) => v;
        {
            const zq = { a: 1 };
            const r1 = zq + 1;
            const r2 = -zq;
            const r3 = f(zq, [zq]);
            const r4 = new C(zq);
            const r5 = zq.a;
            const r6 = o[zq];
            const r7 = [zq, [zq]];
            const r8 = { k: zq };
            const r9 = `a${zq}b${zq}`;
            const r10 = tag`x${zq}`;
            const r11 = zq?.a?.b;
            const r12 = zq ? zq : zq;
            const r13 = (zq, zq);
            const r14 = (zq);
            const r15 = zq || zq;
            const r16 = zq = 3;
            zq++;
            const r17 = typeof zq;
            const r18 = () => zq;
            const r19 = function () { return zq; };
            const r20 = class extends zq { m() { return zq; } p = zq; };
            const r23 = zq as number;
            const r24 = zq satisfies number;
            const r26 = await zq;
            const r27 = zq ?? zq;
            const r28 = [...zq];
            const r29 = import(zq);
            class D extends zq {}
            if (zq) { throw zq; }
            function* genZq() { yield zq; }
            if (zq) { zq; } else { zq; }
            while (zq) { break; }
            do { zq; } while (zq);
            for (let i = zq; i < zq; i += zq) { zq; }
            for (const k in zq) { zq; }
            for (const k2 of zq) { zq; }
            for (zq of [1]) { zq; }
            switch (zq) { case zq: zq; break; default: zq; }
            lbl: for (;;) { zq; break lbl; }
            return zq;
        }
        console.log(zq);
    "#;
    // Everything inside the braces of the block is the shadowing scope: every `zq` there is the inner binding.
    let inner = body.split("console.log(zq);").next().unwrap();
    let inner_count = inner.matches("zq").count() - 1; // minus the outer `let zq = 1;`
    let source = format!("async function outerFn() {{ {body} }}");
    let (n, json) = renamed_json(&source);
    assert_eq!(n, 1, "{json}");
    assert_eq!(
        json.matches("\"zq{b0}\"").count(),
        inner_count,
        "every inner position is rewritten: {json}"
    );
    assert_eq!(
        json.matches("\"zq\"").count(),
        2,
        "the outer declaration and its one reference are untouched: {json}"
    );
}
