use super::*;

#[test]
fn split_command_spec_supports_shell_like_quoting() {
    let parts = split_command_spec(
        r#"browser-wrapper --headless --profile "real browser" 'wrapped runner' escaped\ space"#,
    )
    .expect("split valid browser harness command");

    assert_eq!(
        parts,
        vec![
            "browser-wrapper".to_string(),
            "--headless".to_string(),
            "--profile".to_string(),
            "real browser".to_string(),
            "wrapped runner".to_string(),
            "escaped space".to_string(),
        ]
    );
}

#[test]
fn split_command_spec_rejects_malformed_inputs() {
    assert_eq!(split_command_spec("   "), None);
    assert_eq!(split_command_spec(r#"" --flag"#), None);
    assert_eq!(split_command_spec(r#"browser-wrapper "unterminated"#), None);
    assert_eq!(split_command_spec(r#"browser-wrapper \"#), None);
}
