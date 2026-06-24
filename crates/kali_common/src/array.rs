use crate::*;

/// Canonical aliases for the supported `Array.from` helper slice.
pub const fn array_from_aliases() -> &'static [&'static str] {
    &[
        "Array.from",
        "globalThis.Array.from",
        r#"globalThis["Array"].from"#,
        r#"globalThis["Array"]["from"]"#,
        r#"globalThis["Array"]['from']"#,
        r#"globalThis['Array'].from"#,
        r#"globalThis['Array']['from']"#,
        r#"globalThis['Array']["from"]"#,
        r#"Array["from"]"#,
        r#"Array['from']"#,
        r#"globalThis.Array["from"]"#,
        r#"globalThis.Array['from']"#,
    ]
}

/// Canonical source text for the supported `Array.from` helper aliases.
pub fn array_from_source() -> String {
    join_semicolon_terminated_segments(array_from_aliases())
}

/// Canonical frozen callable aliases for the supported `Array.from` helper slice.
pub const fn array_from_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(Array.from)"#,
        r#"Object.freeze((Array.from))"#,
        r#"Object.freeze(globalThis.Array.from)"#,
        r#"Object.freeze((globalThis.Array.from))"#,
        r#"Object.freeze(globalThis["Array"].from)"#,
        r#"Object.freeze((globalThis["Array"].from))"#,
        r#"Object.freeze((globalThis["Array"]).from)"#,
        r#"Object.freeze((globalThis["Array"])["from"])"#,
        r#"Object.freeze((globalThis["Array"])['from'])"#,
        r#"Object.freeze(globalThis["Array"]["from"])"#,
        r#"Object.freeze((globalThis["Array"]["from"]))"#,
        r#"Object.freeze(globalThis['Array'].from)"#,
        r#"Object.freeze((globalThis['Array'].from))"#,
        r#"Object.freeze((globalThis['Array']).from)"#,
        r#"Object.freeze((globalThis['Array'])["from"])"#,
        r#"Object.freeze((globalThis["Array"]))["from"]"#,
        r#"Object.freeze((globalThis["Array"]))['from']"#,
        r#"Object.freeze((globalThis['Array']))["from"]"#,
        r#"Object.freeze((globalThis['Array']))['from']"#,
        r#"Object.freeze(globalThis['Array']['from'])"#,
        r#"Object.freeze((globalThis['Array']['from']))"#,
        r#"Object.freeze(globalThis["Array"]['from'])"#,
        r#"Object.freeze((globalThis["Array"]['from']))"#,
        r#"Object.freeze((globalThis['Array'])['from'])"#,
        r#"Object.freeze(globalThis['Array']["from"])"#,
        r#"Object.freeze((globalThis['Array']["from"]))"#,
        r#"Object.freeze(Array['from'])"#,
        r#"Object.freeze((Array['from']))"#,
        r#"Object.freeze(Array["from"])"#,
        r#"Object.freeze((Array["from"]))"#,
        r#"Object.freeze(globalThis.Array['from'])"#,
        r#"Object.freeze((globalThis.Array['from']))"#,
        r#"Object.freeze(globalThis.Array["from"])"#,
        r#"Object.freeze((null ?? globalThis.Array["from"]))"#,
        r#"Object.freeze((true && globalThis.Array["from"]))"#,
        r#"Object.freeze((false || globalThis.Array["from"]))"#,
        r#"Object.freeze((globalThis.Array["from"]))"#,
        r#"Object.freeze((globalThis.Array).from)"#,
        r#"Object.freeze((globalThis.Array)["from"])"#,
        r#"Object.freeze((globalThis.Array))["from"]"#,
        r#"Object.freeze((globalThis.Array))['from']"#,
        r#"Object.freeze((globalThis.Array)['from'])"#,
        r#"Object.freeze((null ?? globalThis.Array.from))"#,
        r#"Object.freeze((true && globalThis.Array.from))"#,
        r#"Object.freeze((false || globalThis.Array.from))"#,
        r#"Object.freeze((Array.from, Array.from))"#,
        r#"Object.freeze((globalThis.Array.from, globalThis.Array.from))"#,
        r#"Object.freeze((globalThis["Array"].from, globalThis["Array"].from))"#,
        r#"Object.freeze((globalThis['Array'].from, globalThis['Array'].from))"#,
        r#"Object.freeze((null ?? Array.from))"#,
        r#"Object.freeze((true && Array.from))"#,
        r#"Object.freeze((false || Array.from))"#,
        r#"Object.freeze((null ?? globalThis["Array"].from))"#,
        r#"Object.freeze((true && globalThis["Array"].from))"#,
        r#"Object.freeze((false || globalThis["Array"].from))"#,
        r#"Object.freeze((null ?? globalThis["Array"]["from"]))"#,
        r#"Object.freeze((true && globalThis["Array"]["from"]))"#,
        r#"Object.freeze((false || globalThis["Array"]["from"]))"#,
        r#"Object.freeze((null ?? globalThis['Array']['from']))"#,
        r#"Object.freeze((true && globalThis['Array']['from']))"#,
        r#"Object.freeze((false || globalThis['Array']['from']))"#,
        r#"Object.freeze((null ?? globalThis['Array'].from))"#,
        r#"Object.freeze((true && globalThis['Array'].from))"#,
        r#"Object.freeze((false || globalThis['Array'].from))"#,
        r#"Object.freeze((null ?? globalThis["Array"]['from']))"#,
        r#"Object.freeze((true && globalThis["Array"]['from']))"#,
        r#"Object.freeze((false || globalThis["Array"]['from']))"#,
        r#"Object.freeze((null ?? globalThis.Array['from']))"#,
        r#"Object.freeze((true && globalThis.Array['from']))"#,
        r#"Object.freeze((false || globalThis.Array['from']))"#,
    ]
}

/// Canonical source text for the supported `Array.from` frozen callable aliases.
pub fn array_from_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(array_from_frozen_callable_aliases())
}

/// Canonical source text for the supported `Array.from` alias inventory.
pub fn array_from_alias_inventory_source() -> String {
    format!(
        "{} {}",
        array_from_source().trim_end(),
        array_from_frozen_callable_source().trim_end()
    )
}

/// Canonical `for`/`for await` loop lines for the supported `Array.from` helper slice.
pub fn array_from_loop_lines(source: &str, loop_header: &str, indentation: &str) -> String {
    source
        .trim_end_matches(';')
        .split("; ")
        .map(|alias| {
            format!(
                "{indentation}{loop_header}{alias}(values)) {{\n{indentation}  console.log(value);\n{indentation}}}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
#[path = "array_tests.rs"]
mod array_tests;
