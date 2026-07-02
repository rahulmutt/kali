use super::*;

#[test]
fn split_returns_none_for_non_backtick_text() {
    assert!(split_template_literal("\"v: ${x}\"").is_none());
    assert!(split_template_literal("plain").is_none());
}

#[test]
fn split_handles_template_without_interpolation() {
    let segments = split_template_literal("`hello`").expect("split");
    assert_eq!(segments.quasis, vec!["hello".to_string()]);
    assert!(segments.expressions.is_empty());
}

#[test]
fn split_extracts_quasis_and_expressions() {
    let segments = split_template_literal("`v: ${7 / 2} end`").expect("split");
    assert_eq!(segments.quasis, vec!["v: ".to_string(), " end".to_string()]);
    assert_eq!(segments.expressions, vec!["7 / 2".to_string()]);
}

#[test]
fn split_handles_adjacent_interpolations_and_edges() {
    let segments = split_template_literal("`${a}${b}`").expect("split");
    assert_eq!(
        segments.quasis,
        vec!["".to_string(), "".to_string(), "".to_string()]
    );
    assert_eq!(segments.expressions, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn split_respects_nested_braces_and_strings_in_expressions() {
    let segments = split_template_literal("`v: ${fn({ a: '}' })}`").expect("split");
    assert_eq!(segments.quasis, vec!["v: ".to_string(), "".to_string()]);
    assert_eq!(segments.expressions, vec!["fn({ a: '}' })".to_string()]);
}

#[test]
fn split_returns_none_for_unterminated_interpolation() {
    assert!(split_template_literal("`v: ${7`").is_none());
}

#[test]
fn resolve_still_renders_via_segments() {
    let rendered = resolve_interpolated_template_literal("`v: ${x} end`", |segment| {
        (segment == "x").then(|| "3.5".to_string())
    })
    .expect("render");
    assert_eq!(rendered, "v: 3.5 end");
}

#[test]
fn resolve_still_passes_through_plain_templates() {
    let rendered = resolve_interpolated_template_literal("`hello`", |_| None).expect("render");
    assert_eq!(rendered, "hello");
}
