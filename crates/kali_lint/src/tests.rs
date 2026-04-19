use super::*;

#[test]
fn reports_basic_lint_issues() {
    let diagnostics = lint("var x = 1; let y = 2; debugger; if (x == y) { }");
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(w2::NO_VAR as u32)));
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(w2::PREFER_CONST as u32)));
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(w2::DEBUGGER as u32)));
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(w2::EQEQEQ as u32)));
}

#[test]
fn fix_mode_applies_basic_safe_rewrites() {
    let result = lint_with_options("var x = 1; debugger; if (x == 1) { }", true);
    let fixed = result.fixed_source.expect("fixed source");
    assert!(fixed.contains("let x = 1;"));
    assert!(!fixed.contains("debugger"));
    assert!(fixed.contains("==="));
}
