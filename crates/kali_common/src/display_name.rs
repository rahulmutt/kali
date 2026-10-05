//! The written spelling of a binding the block-scope rename pass
//! (`kali_cli::build::block_scope_rename`) renamed to `<name>{b<N>}`,
//! and a captured parameter renamed to `<name>{p}`.
//! `{` and `}` are not identifier characters, so the suffix never occurs in a
//! source name; it is stripped wherever kali shows a name to a person.

use std::borrow::Cow;

/// `text` with every `{b<digits>}` and `{p}` that directly follow an identifier
/// character removed. Borrowed when there is nothing to strip.
pub fn display_names_in(text: &str) -> Cow<'_, str> {
    if !text.contains("{b") && !text.contains("{p}") {
        return Cow::Borrowed(text);
    }
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut changed = false;
    while i < bytes.len() {
        if let Some(len) = suffix_len_at(bytes, i) {
            changed = true;
            i += len;
            continue;
        }
        let ch = text[i..].chars().next().expect("in bounds");
        out.push(ch);
        i += ch.len_utf8();
    }
    if changed {
        Cow::Owned(out)
    } else {
        Cow::Borrowed(text)
    }
}

fn suffix_len_at(bytes: &[u8], i: usize) -> Option<usize> {
    if i == 0 || bytes.get(i) != Some(&b'{') {
        return None;
    }
    let prev = bytes[i - 1];
    if !(prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'$') {
        return None;
    }
    // Captured-bindings spec §3.2: a rewritten parameter is `<name>{p}`.
    if bytes.get(i + 1) == Some(&b'p') && bytes.get(i + 2) == Some(&b'}') {
        return Some(3);
    }
    if bytes.get(i + 1) != Some(&b'b') {
        return None;
    }
    let digits = bytes[i + 2..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();
    if digits == 0 || bytes.get(i + 2 + digits) != Some(&b'}') {
        return None;
    }
    Some(3 + digits)
}

/// The spelling the captured-parameter rewrite gives parameter `name`
/// (captured-bindings spec §3.2).
pub fn captured_param_spelling(name: &str) -> String {
    format!("{name}{{p}}")
}

#[cfg(test)]
#[path = "display_name_tests.rs"]
mod display_name_tests;
