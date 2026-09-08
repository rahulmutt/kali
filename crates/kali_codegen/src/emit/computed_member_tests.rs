use crate::test_support::parse_and_lower_lir;
use crate::{lower_lir_to_wasm, CodegenCtx, TargetConfig};
use kali_error::diagnostic::Diagnostic;
use kali_lir::LirNodeKind;

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
