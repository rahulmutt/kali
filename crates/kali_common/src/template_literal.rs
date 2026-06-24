/// Canonical body for the supported template-literal string iteration slice.
pub const fn template_literal_string_iteration_body_source() -> &'static str {
    r#"for (const ch of `hello`) { console.log(ch); }"#
}

/// Canonical browser body for the supported template-literal string iteration slice.
pub const fn browser_template_literal_string_iteration_body_source() -> &'static str {
    r#"const prefix = "he";
const suffix = "llo";
const syncChars = [];
for (const item of `${prefix}${suffix}`) {
  syncChars.push(item);
}
const asyncChars = [];
for await (const item of `${prefix}${suffix}`) {
  asyncChars.push(item);
}
if (syncChars.join("") !== "hello" || asyncChars.join("") !== "hello") {
  throw new Error('unexpected template literal iteration semantics');
}"#
}

#[cfg(test)]
#[path = "template_literal_tests.rs"]
mod template_literal_tests;
