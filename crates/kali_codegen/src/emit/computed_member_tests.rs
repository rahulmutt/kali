use crate::test_support::parse_and_lower_lir;
use crate::{lower_lir_to_wasm, CodegenCtx, TargetConfig};
use kali_error::diagnostic::Diagnostic;
use kali_lir::{LirNode, LirNodeKind};

pub(crate) fn diagnostics_for(source: &str) -> Vec<Diagnostic> {
    let program = parse_and_lower_lir(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    lower_lir_to_wasm(&mut ctx, &program).diagnostics
}

pub(crate) fn assert_e5506(diagnostics: &[Diagnostic], needle: &str, context: &str) {
    assert!(
        diagnostics
            .iter()
            .any(|d| d.is_error() && d.code == Some(5506) && d.message.contains(needle)),
        "{context}: expected an E5506 containing {needle:?}, got {diagnostics:?}"
    );
}

#[test]
fn a_nameless_computed_member_lowers_to_its_own_kind() {
    let program = parse_and_lower_lir("const o = {index: 9, i: 7}; let i = 1; o[i]; o[\"b\"];");
    let nameless: Vec<_> = program
        .nodes
        .iter()
        .filter(|node| node.kind == LirNodeKind::ComputedMember)
        .collect();
    assert_eq!(
        nameless.len(),
        1,
        "exactly one nameless computed member: o[i]"
    );
    assert_eq!(nameless[0].text, None);
    assert_eq!(nameless[0].children.len(), 2);
    assert!(
        program
            .nodes
            .iter()
            .any(|node| node.kind == LirNodeKind::Value
                && node.text.as_deref() == Some("b")
                && node.children.len() == 2),
        "o[\"b\"] stays a named two-child Value"
    );
}

#[test]
fn a_nameless_computed_member_read_refuses_instead_of_reading_a_fabricated_name() {
    let diagnostics = diagnostics_for(
        "const o = {index: 9, i: 7}; let i = 1; console.log(o[i]); console.log(o[i + 0]);",
    );
    assert_e5506(
        &diagnostics,
        "computed member access `o[k]` is unavailable",
        "R-59's repro",
    );
}

fn program_prints(source: &str) -> (Vec<Diagnostic>, String) {
    let program = parse_and_lower_lir(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("printable wasm");
    (result.diagnostics, printed)
}

#[test]
fn a_const_string_key_folds_to_the_literal_spelling() {
    // `o[k]` with `const k = "b"` must compile exactly as `o["b"]` does: no
    // diagnostic, and the folded read reaches the object-literal field fold.
    let (folded, _) =
        program_prints("const o = {a: 1, b: 2}; const k = \"b\"; console.log(o[k] + 1);");
    let (literal, _) = program_prints("const o = {a: 1, b: 2}; console.log(o[\"b\"] + 1);");
    assert!(
        folded.iter().all(|d| !d.is_error()),
        "folded read must not error: {folded:?}"
    );
    assert_eq!(
        folded.iter().filter(|d| d.is_error()).count(),
        literal.iter().filter(|d| d.is_error()).count()
    );
}

#[test]
fn a_const_number_key_folds_into_the_static_element_fold() {
    let (diagnostics, _) = program_prints("const a = [5, 6]; const i = 1; console.log(a[i]);");
    assert!(diagnostics.iter().all(|d| !d.is_error()), "{diagnostics:?}");
}

#[test]
fn a_let_key_refuses_even_when_never_reassigned() {
    let diagnostics = diagnostics_for("const o = {a: 1, b: 2}; let k = \"b\"; console.log(o[k]);");
    assert_e5506(
        &diagnostics,
        "computed member access `o[k]` is unavailable",
        "let key",
    );
}

#[test]
fn a_mutable_index_over_an_array_literal_refuses() {
    let diagnostics =
        diagnostics_for("const a = [5, 6, 7]; for (let j = 0; j < 3; j++) console.log(a[j]);");
    assert_e5506(
        &diagnostics,
        "computed member access `o[k]` is unavailable",
        "array literal, mutable index",
    );
}

#[test]
fn a_runtime_array_keeps_its_dynamic_index_lane() {
    let diagnostics = diagnostics_for(
        "const a = new Array(3); for (let i = 0; i < 3; i++) { a[i] = i * 2; } let j = 1; console.log(a[j]);",
    );
    assert!(
        !diagnostics
            .iter()
            .any(|d| d.message.contains("computed member access")),
        "the linear-memory lane must still admit a runtime array: {diagnostics:?}"
    );
}

#[test]
fn a_string_receiver_refuses_in_both_spellings() {
    let literal = diagnostics_for("const s = \"abc\"; console.log(s[1]);");
    assert_e5506(
        &literal,
        "indexing a string `s[i]` is unavailable",
        "literal index on a string",
    );
    let folded = diagnostics_for("const s = \"abc\"; const k = 1; console.log(s[k]);");
    assert_e5506(
        &folded,
        "indexing a string `s[i]` is unavailable",
        "folded index on a string",
    );
}

#[test]
fn a_chained_access_off_a_nameless_member_refuses_rather_than_rendering_the_child_count() {
    // Measured at dc19c3a040: `o[k].length` prints `2` for EVERY string
    // ("xyz" and "xyzwv" both print 2), because `render_length`'s text-less
    // arm returns the member node's CHILD COUNT. It is not a working lane
    // being regressed — it is a silent miscompile, and refusing is strictly
    // better. (The dot spelling `o.a.length` prints `1` by the same arm and
    // is NOT fixed here — see the follow-up recorded in Task 8.)
    for source in [
        "const o = {a: \"xyz\"}; const k = \"a\"; console.log(o[k].length);",
        "const o = {a: \"xyzwv\"}; const k = \"a\"; console.log(o[k].length);",
    ] {
        let diagnostics = diagnostics_for(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.is_error() && d.code == Some(5506)),
            "{source}: by-id consumers see the nameless kind and must decline: {diagnostics:?}"
        );
    }
}

#[test]
fn the_static_renderers_decline_a_nameless_member() {
    // `render_static_value` and `render_length` render a text-less two-child
    // node as its CHILD COUNT — the string "2". Measured at dc19c3a040:
    // `Object[k](o).length` prints 2 for a four-key object, and
    // `o[k].length` prints 2 for every string. A ComputedMember must never
    // reach that arm; it declines, and the access then refuses.
    let program = parse_and_lower_lir("const o = {a: 1}; let k = \"a\"; Object.hasOwn(o, o[k]);");
    let node: &LirNode = program
        .nodes
        .iter()
        .find(|node| node.kind == LirNodeKind::ComputedMember)
        .expect("o[k] with a let key is nameless");
    assert_eq!(node.text, None);

    let diagnostics = diagnostics_for(
        "const o = {a: 1, b: 2, c: 3, d: 4}; const k = \"keys\"; console.log(Object[k](o).length);",
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d.is_error() && d.code == Some(5506)),
        "Object[k] must refuse, not render the child count: {diagnostics:?}"
    );
}

#[test]
fn the_three_spellings_of_a_string_property_name_agree_and_only_an_index_refuses() {
    // A statically-known string has no INDEX lane in any spelling, but a
    // static property NAME on a string is dot semantics like any other fold:
    // `s.length`, `s["length"]` and `const k = "length"; s[k]` must agree.
    // Only the numeric index (`s[1]`, `const k = 1; s[k]`) refuses.
    //
    // Every program forces RUNTIME emission (`n + …` against a `let`) on
    // purpose: a bare `console.log(s["length"])` is claimed by the console
    // static renderer and never reaches the member lane, so it cannot observe
    // the guard at all. Measured with the guard un-narrowed: the console
    // spelling still printed `3` while this runtime spelling refused.
    for source in [
        "const s = \"abc\"; let n = 0; n = n + s.length; console.log(n);",
        "const s = \"abc\"; let n = 0; n = n + s[\"length\"]; console.log(n);",
        "const s = \"abc\"; const k = \"length\"; let n = 0; n = n + s[k]; console.log(n);",
    ] {
        let diagnostics = diagnostics_for(source);
        assert!(
            diagnostics.iter().all(|d| !d.is_error()),
            "{source}: a static property name on a string keeps its lane: {diagnostics:?}"
        );
    }
    for source in [
        "const s = \"abc\"; let n = 0; n = n + s[1]; console.log(n);",
        "const s = \"abc\"; const k = 1; let n = 0; n = n + s[k]; console.log(n);",
    ] {
        let diagnostics = diagnostics_for(source);
        assert_e5506(
            &diagnostics,
            "indexing a string `s[i]` is unavailable",
            source,
        );
    }
}
