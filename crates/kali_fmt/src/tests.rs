use super::*;

#[test]
fn formats_simple_function() {
    let formatted = format_source("function add(a,b){return a+b;}");
    assert!(formatted.contains("function add(a, b) {\n"));
    assert!(formatted.contains("return a + b;\n"));
    assert!(formatted.ends_with('\n'));
}

#[test]
fn formatting_is_idempotent() {
    let source = "function add(a,b){return a+b;}";
    let once = format_source(source);
    let twice = format_source(&once);
    assert_eq!(once, twice);
}

#[test]
fn interpolated_template_literals_format_idempotently() {
    let source = "console.log(`v: ${7 / 2} end`);\n";
    let once = format_source(source);
    let twice = format_source(&once);
    assert_eq!(once, twice);
    assert!(once.contains("`v: ${7 / 2} end`"), "formatted: {once:?}");
}
