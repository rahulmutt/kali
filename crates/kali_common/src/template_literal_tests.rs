use crate::*;

#[test]
fn test_template_literal_string_iteration_body_source_is_canonical() {
    assert_eq!(
        template_literal_string_iteration_body_source(),
        "for (const ch of `hello`) { console.log(ch); }"
    );
}

#[test]
fn test_browser_template_literal_string_iteration_body_source_is_canonical() {
    assert_eq!(
        browser_template_literal_string_iteration_body_source(),
        concat!(
            "const prefix = \"he\";\n",
            "const suffix = \"llo\";\n",
            "const syncChars = [];\n",
            "for (const item of `${prefix}${suffix}`) {\n",
            "  syncChars.push(item);\n",
            "}\n",
            "const asyncChars = [];\n",
            "for await (const item of `${prefix}${suffix}`) {\n",
            "  asyncChars.push(item);\n",
            "}\n",
            "if (syncChars.join(\"\") !== \"hello\" || asyncChars.join(\"\") !== \"hello\") {\n",
            "  throw new Error('unexpected template literal iteration semantics');\n",
            "}"
        )
    );
}
