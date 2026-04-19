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
