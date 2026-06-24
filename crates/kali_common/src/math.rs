use crate::*;

/// Canonical frozen callable aliases for the supported `Math.abs` / `Math.sign` helper slice.
pub const fn math_abs_sign_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["abs"])"#,
        r#"Object.freeze((globalThis.Math["abs"]))"#,
        r#"Object.freeze(globalThis.Math['abs'])"#,
        r#"Object.freeze((globalThis.Math['abs']))"#,
        r#"Object.freeze(globalThis.Math.abs)"#,
        r#"Object.freeze((globalThis.Math.abs))"#,
        r#"Object.freeze(globalThis["Math"]["abs"])"#,
        r#"Object.freeze((globalThis["Math"]["abs"]))"#,
        r#"Object.freeze(globalThis["Math"]['abs'])"#,
        r#"Object.freeze((globalThis["Math"]['abs']))"#,
        r#"Object.freeze(globalThis["Math"].abs)"#,
        r#"Object.freeze((globalThis["Math"].abs))"#,
        r#"Object.freeze(globalThis['Math']['abs'])"#,
        r#"Object.freeze((globalThis['Math']['abs']))"#,
        r#"Object.freeze(globalThis['Math'].abs)"#,
        r#"Object.freeze((globalThis['Math'].abs))"#,
        r#"Object.freeze(Math.abs)"#,
        r#"Object.freeze((Math.abs))"#,
        r#"Object.freeze(Math["abs"])"#,
        r#"Object.freeze((Math["abs"]))"#,
        r#"Object.freeze(Math['abs'])"#,
        r#"Object.freeze((Math['abs']))"#,
        r#"Object.freeze(globalThis.Math["sign"])"#,
        r#"Object.freeze((globalThis.Math["sign"]))"#,
        r#"Object.freeze(globalThis.Math['sign'])"#,
        r#"Object.freeze((globalThis.Math['sign']))"#,
        r#"Object.freeze(globalThis.Math.sign)"#,
        r#"Object.freeze((globalThis.Math.sign))"#,
        r#"Object.freeze(globalThis["Math"]["sign"])"#,
        r#"Object.freeze((globalThis["Math"]["sign"]))"#,
        r#"Object.freeze(globalThis["Math"]['sign'])"#,
        r#"Object.freeze((globalThis["Math"]['sign']))"#,
        r#"Object.freeze(globalThis["Math"].sign)"#,
        r#"Object.freeze((globalThis["Math"].sign))"#,
        r#"Object.freeze(globalThis['Math']['sign'])"#,
        r#"Object.freeze((globalThis['Math']['sign']))"#,
        r#"Object.freeze(globalThis['Math'].sign)"#,
        r#"Object.freeze((globalThis['Math'].sign))"#,
        r#"Object.freeze(Math.sign)"#,
        r#"Object.freeze((Math.sign))"#,
        r#"Object.freeze(Math["sign"])"#,
        r#"Object.freeze((Math["sign"]))"#,
        r#"Object.freeze(Math['sign'])"#,
        r#"Object.freeze((Math['sign']))"#,
    ]
}

/// Canonical source text for the supported `Math.abs` / `Math.sign` frozen callable aliases.
pub fn math_abs_sign_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_abs_sign_frozen_callable_aliases())
}

/// Canonical `console.log(...)` invocation lines for the supported `Math.abs` / `Math.sign` frozen callable aliases.
pub fn math_abs_sign_frozen_callable_invocation_lines(indentation: &str) -> String {
    math_abs_sign_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}console.log({alias}(alias));"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical `console.log(...)` invocation text for the supported `Math.abs` / `Math.sign` frozen callable aliases.
pub fn math_abs_sign_frozen_callable_invocation_source() -> String {
    math_abs_sign_frozen_callable_invocation_lines("")
}

/// Canonical `return [...]` entry text for the supported `Math.abs` / `Math.sign` frozen callable aliases.
pub fn math_abs_sign_frozen_callable_entries(indentation: &str) -> String {
    math_abs_sign_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}{alias}(alias)"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Canonical `return [...]` entry text for the supported `Math.abs` / `Math.sign` frozen callable aliases.
pub fn math_abs_sign_frozen_callable_entries_source() -> String {
    math_abs_sign_frozen_callable_entries("")
}

/// Canonical frozen callable aliases for the supported `Math.floor` / `Math.trunc` / `Math.ceil` helper slice.
pub const fn math_floor_trunc_ceil_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["floor"])"#,
        r#"Object.freeze((globalThis.Math["floor"]))"#,
        r#"Object.freeze(globalThis.Math['floor'])"#,
        r#"Object.freeze((globalThis.Math['floor']))"#,
        r#"Object.freeze(globalThis.Math.floor)"#,
        r#"Object.freeze((globalThis.Math.floor))"#,
        r#"Object.freeze(globalThis["Math"]["floor"])"#,
        r#"Object.freeze((globalThis["Math"]["floor"]))"#,
        r#"Object.freeze((globalThis["Math"]))["floor"]"#,
        r#"Object.freeze((globalThis["Math"]))['floor']"#,
        r#"Object.freeze((globalThis.Math))["floor"]"#,
        r#"Object.freeze((globalThis.Math))['floor']"#,
        r#"Object.freeze((globalThis['Math']))["floor"]"#,
        r#"Object.freeze((globalThis['Math']))['floor']"#,
        r#"Object.freeze(globalThis["Math"]['floor'])"#,
        r#"Object.freeze((globalThis["Math"]['floor']))"#,
        r#"Object.freeze(globalThis["Math"].floor)"#,
        r#"Object.freeze((globalThis["Math"])["floor"])"#,
        r#"Object.freeze((globalThis['Math'])['floor'])"#,
        r#"Object.freeze(globalThis['Math'].floor)"#,
        r#"Object.freeze((globalThis['Math']).floor)"#,
        r#"Object.freeze((globalThis["Math"]).floor)"#,
        r#"Object.freeze((globalThis["Math"].floor))"#,
        r#"Object.freeze(Math["floor"])"#,
        r#"Object.freeze((Math["floor"]))"#,
        r#"Object.freeze(Math['floor'])"#,
        r#"Object.freeze((Math['floor']))"#,
        r#"Object.freeze(globalThis.Math["trunc"])"#,
        r#"Object.freeze((globalThis.Math["trunc"]))"#,
        r#"Object.freeze(globalThis.Math['trunc'])"#,
        r#"Object.freeze((globalThis.Math['trunc']))"#,
        r#"Object.freeze(globalThis.Math.trunc)"#,
        r#"Object.freeze((globalThis.Math.trunc))"#,
        r#"Object.freeze(globalThis["Math"]["trunc"])"#,
        r#"Object.freeze((globalThis["Math"]["trunc"]))"#,
        r#"Object.freeze((globalThis["Math"]))["trunc"]"#,
        r#"Object.freeze((globalThis["Math"]))['trunc']"#,
        r#"Object.freeze((globalThis.Math))["trunc"]"#,
        r#"Object.freeze((globalThis.Math))['trunc']"#,
        r#"Object.freeze((globalThis['Math']))["trunc"]"#,
        r#"Object.freeze((globalThis['Math']))['trunc']"#,
        r#"Object.freeze(globalThis["Math"]['trunc'])"#,
        r#"Object.freeze((globalThis["Math"]['trunc']))"#,
        r#"Object.freeze(globalThis["Math"].trunc)"#,
        r#"Object.freeze((globalThis["Math"])["trunc"])"#,
        r#"Object.freeze((globalThis['Math'])['trunc'])"#,
        r#"Object.freeze(globalThis['Math'].trunc)"#,
        r#"Object.freeze((globalThis['Math']).trunc)"#,
        r#"Object.freeze((globalThis["Math"]).trunc)"#,
        r#"Object.freeze((globalThis["Math"].trunc))"#,
        r#"Object.freeze(Math["trunc"])"#,
        r#"Object.freeze((Math["trunc"]))"#,
        r#"Object.freeze(Math['trunc'])"#,
        r#"Object.freeze((Math['trunc']))"#,
        r#"Object.freeze(globalThis.Math["ceil"])"#,
        r#"Object.freeze((globalThis.Math["ceil"]))"#,
        r#"Object.freeze(globalThis.Math['ceil'])"#,
        r#"Object.freeze((globalThis.Math['ceil']))"#,
        r#"Object.freeze(globalThis.Math.ceil)"#,
        r#"Object.freeze((globalThis.Math.ceil))"#,
        r#"Object.freeze(globalThis["Math"]["ceil"])"#,
        r#"Object.freeze((globalThis["Math"]["ceil"]))"#,
        r#"Object.freeze((globalThis["Math"]))["ceil"]"#,
        r#"Object.freeze((globalThis["Math"]))['ceil']"#,
        r#"Object.freeze((globalThis.Math))["ceil"]"#,
        r#"Object.freeze((globalThis.Math))['ceil']"#,
        r#"Object.freeze((globalThis['Math']))["ceil"]"#,
        r#"Object.freeze((globalThis['Math']))['ceil']"#,
        r#"Object.freeze(globalThis["Math"]['ceil'])"#,
        r#"Object.freeze((globalThis["Math"]['ceil']))"#,
        r#"Object.freeze(globalThis["Math"].ceil)"#,
        r#"Object.freeze((globalThis["Math"])["ceil"])"#,
        r#"Object.freeze((globalThis['Math'])['ceil'])"#,
        r#"Object.freeze(globalThis['Math'].ceil)"#,
        r#"Object.freeze((globalThis['Math']).ceil)"#,
        r#"Object.freeze((globalThis["Math"]).ceil)"#,
        r#"Object.freeze((globalThis["Math"].ceil))"#,
        r#"Object.freeze(Math["ceil"])"#,
        r#"Object.freeze((Math["ceil"]))"#,
        r#"Object.freeze(Math['ceil'])"#,
        r#"Object.freeze((Math['ceil']))"#,
    ]
}

/// Canonical source text for the supported `Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.
pub fn math_floor_trunc_ceil_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_floor_trunc_ceil_frozen_callable_aliases())
}

/// Canonical `console.log(...)` invocation lines for the supported `Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.
pub fn math_floor_trunc_ceil_frozen_callable_invocation_lines(indentation: &str) -> String {
    math_floor_trunc_ceil_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}console.log({alias}(alias));"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical `console.log(...)` invocation text for the supported `Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.
pub fn math_floor_trunc_ceil_frozen_callable_invocation_source() -> String {
    math_floor_trunc_ceil_frozen_callable_invocation_lines("")
}

/// Canonical `return [...]` entry text for the supported `Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.
pub fn math_floor_trunc_ceil_frozen_callable_entries(indentation: &str) -> String {
    math_floor_trunc_ceil_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}{alias}(alias)"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Canonical `return [...]` entry text for the supported `Math.floor` / `Math.trunc` / `Math.ceil` frozen callable aliases.
pub fn math_floor_trunc_ceil_frozen_callable_entries_source() -> String {
    math_floor_trunc_ceil_frozen_callable_entries("")
}

/// Canonical frozen callable aliases for the supported `Math.round` helper slice.
pub const fn math_round_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["round"])"#,
        r#"Object.freeze((globalThis.Math["round"]))"#,
        r#"Object.freeze(globalThis.Math['round'])"#,
        r#"Object.freeze((globalThis.Math['round']))"#,
        r#"Object.freeze(globalThis.Math.round)"#,
        r#"Object.freeze((globalThis.Math.round))"#,
        r#"Object.freeze(globalThis?.Math.round)"#,
        r#"Object.freeze((globalThis?.Math.round))"#,
        r#"Object.freeze(globalThis["Math"]["round"])"#,
        r#"Object.freeze((globalThis["Math"]["round"]))"#,
        r#"Object.freeze(globalThis["Math"]['round'])"#,
        r#"Object.freeze((globalThis["Math"]['round']))"#,
        r#"Object.freeze(globalThis["Math"].round)"#,
        r#"Object.freeze((globalThis["Math"]).round)"#,
        r#"Object.freeze((globalThis["Math"].round))"#,
        r#"Object.freeze((globalThis["Math"])["round"])"#,
        r#"Object.freeze((globalThis['Math'])['round'])"#,
        r#"Object.freeze((globalThis['Math'])["round"])"#,
        r#"Object.freeze(globalThis['Math']['round'])"#,
        r#"Object.freeze((globalThis['Math']['round']))"#,
        r#"Object.freeze(globalThis['Math'].round)"#,
        r#"Object.freeze((globalThis['Math']).round)"#,
        r#"Object.freeze((globalThis['Math'].round))"#,
        r#"Object.freeze(Math.round)"#,
        r#"Object.freeze((Math.round))"#,
        r#"Object.freeze(Math["round"])"#,
        r#"Object.freeze((Math["round"]))"#,
        r#"Object.freeze(Math['round'])"#,
        r#"Object.freeze((Math['round']))"#,
        r#"Object.freeze((null ?? Math.round))"#,
        r#"Object.freeze((true && globalThis.Math.round))"#,
        r#"Object.freeze((false || globalThis["Math"]["round"]))"#,
    ]
}

/// Canonical source text for the supported `Math.round` frozen callable aliases.
pub fn math_round_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_round_frozen_callable_aliases())
}

/// Canonical `console.log(...)` invocation lines for the supported `Math.round` frozen callable aliases.
pub fn math_round_frozen_callable_invocation_lines(indentation: &str) -> String {
    math_round_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}console.log({alias}(value));"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical `console.log(...)` invocation text for the supported `Math.round` frozen callable aliases.
pub fn math_round_frozen_callable_invocation_source() -> String {
    math_round_frozen_callable_invocation_lines("")
}

/// Canonical `return [...]` entry text for the supported `Math.round` frozen callable aliases.
pub fn math_round_frozen_callable_entries(indentation: &str) -> String {
    math_round_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("{indentation}{alias}(value)"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Canonical `return [...]` entry text for the supported `Math.round` frozen callable aliases.
pub fn math_round_frozen_callable_entries_source() -> String {
    math_round_frozen_callable_entries("")
}

/// Canonical direct aliases for the supported `Math.pow` helper slice.
pub const fn math_pow_aliases() -> &'static [&'static str] {
    &[
        "Math.pow",
        r#"Math['pow']"#,
        r#"Math["pow"]"#,
        "globalThis.Math.pow",
        r#"globalThis.Math['pow']"#,
        r#"globalThis.Math["pow"]"#,
        r#"globalThis['Math'].pow"#,
        r#"globalThis['Math']['pow']"#,
        r#"globalThis['Math']["pow"]"#,
        r#"globalThis["Math"].pow"#,
        r#"globalThis["Math"]["pow"]"#,
        r#"globalThis["Math"]['pow']"#,
    ]
}

/// Canonical source text for the supported `Math.pow` helper aliases.
pub fn math_pow_source() -> String {
    join_semicolon_terminated_segments(math_pow_aliases())
}

/// Canonical source text for the supported `Math.pow` alias inventory.
pub fn math_pow_alias_inventory_source() -> String {
    format!(
        "{} {}",
        math_pow_source().trim_end(),
        math_pow_frozen_callable_source().trim_end()
    )
}

/// Canonical browser alias inventory for the supported `Math.pow` helper slice.
pub fn math_pow_browser_alias_inventory_aliases() -> Vec<&'static str> {
    let frozen_aliases = math_pow_frozen_callable_aliases();
    ordered_unique_union(&[
        math_pow_aliases(),
        frozen_aliases.as_slice(),
        math_pow_bracketed_frozen_callable_aliases(),
    ])
}

/// Canonical browser source text for the supported `Math.pow` alias inventory.
pub fn math_pow_browser_alias_inventory_source() -> String {
    join_semicolon_terminated_segments(&math_pow_browser_alias_inventory_aliases())
}

/// Canonical browser-invocation lines for the supported `Math.pow` browser alias inventory.
pub fn math_pow_browser_alias_inventory_invocation_lines(indentation: &str) -> String {
    math_pow_invocation_lines_for_aliases(
        math_pow_browser_alias_inventory_aliases().as_slice(),
        "2",
        "alias",
        indentation,
    )
}

/// Canonical browser-source invocation text for the supported `Math.pow` browser alias inventory.
pub fn math_pow_browser_alias_inventory_invocation_source() -> String {
    format!(
        "const exponent = 3; const alias = exponent;\n{}\n",
        math_pow_browser_alias_inventory_invocation_lines("")
    )
}

/// Canonical browser-bundle source text for the supported bracketed `globalThis["Math"].pow` alias chain.
pub const fn math_pow_bracketed_global_this_alias_chain_source() -> &'static str {
    r##"// kali-tree-shake: bracketedGlobalThisMathPowAliasChain
function bracketedGlobalThisMathPowAliasChain() {
  const exponent = 3;
  const alias = exponent;
  console.log(globalThis["Math"].pow(2, alias));
  return globalThis["Math"].pow(2, alias);
}
"##
}

/// Canonical `console.log(...)` invocation lines for the supported `Math.pow` helper slice.
pub fn math_pow_invocation_lines(source: &str, indentation: &str) -> String {
    source
        .trim_end_matches(';')
        .split("; ")
        .map(|alias| format!("{indentation}console.log({alias}(2, alias));"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical `console.log(...)` invocation lines for an arbitrary `Math.pow` alias inventory.
pub fn math_pow_invocation_lines_for_aliases(
    aliases: &[&str],
    base: &str,
    argument: &str,
    indentation: &str,
) -> String {
    aliases
        .iter()
        .map(|alias| format!("{indentation}console.log({alias}({base}, {argument}));"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical `return [...]` invocation entries for an arbitrary `Math.pow` alias inventory.
pub fn math_pow_invocation_entries_for_aliases(
    aliases: &[&str],
    base: &str,
    argument: &str,
    indentation: &str,
) -> String {
    aliases
        .iter()
        .map(|alias| format!("{indentation}{alias}({base}, {argument}),"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Canonical direct frozen callable aliases for the supported `Math.pow` helper slice.
pub const fn math_pow_frozen_callable_direct_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math['pow'])"#,
        r#"Object.freeze(globalThis.Math["pow"])"#,
        r#"Object.freeze(globalThis['Math']['pow'])"#,
        r#"Object.freeze(globalThis['Math']["pow"])"#,
        r#"Object.freeze(globalThis["Math"]["pow"])"#,
        r#"Object.freeze(globalThis["Math"]['pow'])"#,
        r#"Object.freeze(globalThis.Math.pow)"#,
        r#"Object.freeze(globalThis['Math'].pow)"#,
        r#"Object.freeze(globalThis["Math"].pow)"#,
        r#"Object.freeze(Math.pow)"#,
        r#"Object.freeze(Math['pow'])"#,
        r#"Object.freeze(Math["pow"])"#,
    ]
}

/// Canonical parenthesized frozen callable aliases for the supported `Math.pow` helper slice.
pub const fn math_pow_frozen_callable_parenthesized_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze((globalThis.Math['pow']))"#,
        r#"Object.freeze((globalThis.Math["pow"]))"#,
        r#"Object.freeze((globalThis['Math']['pow']))"#,
        r#"Object.freeze((globalThis['Math']["pow"]))"#,
        r#"Object.freeze((globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((globalThis["Math"]['pow']))"#,
        r#"Object.freeze((globalThis.Math.pow))"#,
        r#"Object.freeze((globalThis['Math'].pow))"#,
        r#"Object.freeze((globalThis["Math"].pow))"#,
        r#"Object.freeze((Math.pow))"#,
        r#"Object.freeze((Math['pow']))"#,
        r#"Object.freeze((Math["pow"]))"#,
    ]
}

/// Canonical nullish/logical frozen callable aliases for the supported `Math.pow` helper slice.
pub const fn math_pow_frozen_callable_nullish_logical_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze((null ?? Math.pow))"#,
        r#"Object.freeze((true && Math.pow))"#,
        r#"Object.freeze((false || Math.pow))"#,
        r#"Object.freeze((null ?? globalThis.Math.pow))"#,
        r#"Object.freeze((true && globalThis.Math.pow))"#,
        r#"Object.freeze((false || globalThis.Math.pow))"#,
        r#"Object.freeze((null ?? globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((true && globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((false || globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((null ?? globalThis['Math']['pow']))"#,
        r#"Object.freeze((true && globalThis['Math']['pow']))"#,
        r#"Object.freeze((false || globalThis['Math']['pow']))"#,
    ]
}

/// Canonical bracketed-root frozen callable aliases for the supported `Math.pow` helper slice.
pub const fn math_pow_bracketed_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze((globalThis.Math))["pow"]"#,
        r#"Object.freeze((globalThis.Math))['pow']"#,
        r#"Object.freeze((globalThis.Math).pow)"#,
        r#"Object.freeze((globalThis.Math)['pow'])"#,
        r#"Object.freeze((globalThis["Math"]))["pow"]"#,
        r#"Object.freeze((globalThis['Math']))['pow']"#,
        r#"Object.freeze((globalThis['Math'])["pow"])"#,
        r#"Object.freeze((globalThis['Math'])['pow'])"#,
        r#"Object.freeze((globalThis["Math"]).pow)"#,
        r#"Object.freeze((globalThis['Math']).pow)"#,
    ]
}

/// Canonical source text for the supported `Math.pow` bracketed-root frozen callable aliases.
pub fn math_pow_bracketed_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_pow_bracketed_frozen_callable_aliases())
}

/// Canonical `console.log(...)` invocation lines for the supported bracketed-root frozen `Math.pow` aliases.
pub fn math_pow_bracketed_frozen_callable_invocation_lines(indentation: &str) -> String {
    math_pow_invocation_lines_for_aliases(
        math_pow_bracketed_frozen_callable_aliases(),
        "2",
        "alias",
        indentation,
    )
}

/// Canonical `return [...]` invocation entries for the supported bracketed-root frozen `Math.pow` aliases.
pub fn math_pow_bracketed_frozen_callable_invocation_entries(indentation: &str) -> String {
    math_pow_invocation_entries_for_aliases(
        math_pow_bracketed_frozen_callable_aliases(),
        "2",
        "alias",
        indentation,
    )
}

/// Canonical frozen callable aliases for the supported `Math.pow` helper slice.
pub fn math_pow_frozen_callable_aliases() -> Vec<&'static str> {
    ordered_unique_union(&[
        math_pow_frozen_callable_direct_aliases(),
        math_pow_frozen_callable_parenthesized_aliases(),
        math_pow_frozen_callable_nullish_logical_aliases(),
    ])
}

/// Canonical source text for the supported `Math.pow` frozen callable aliases.
pub fn math_pow_frozen_callable_source() -> String {
    let aliases = math_pow_frozen_callable_aliases();
    join_semicolon_terminated_segments(&aliases)
}

/// Canonical frozen callable aliases for the supported `Math.cbrt` helper slice.
pub const fn math_cbrt_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["cbrt"])"#,
        r#"Object.freeze((globalThis.Math["cbrt"]))"#,
        r#"Object.freeze(globalThis.Math['cbrt'])"#,
        r#"Object.freeze((globalThis.Math['cbrt']))"#,
        r#"Object.freeze(globalThis.Math.cbrt)"#,
        r#"Object.freeze((globalThis.Math.cbrt))"#,
        r#"Object.freeze((globalThis.Math)["cbrt"])"#,
        r#"Object.freeze((globalThis.Math)['cbrt'])"#,
        r#"Object.freeze(globalThis["Math"]["cbrt"])"#,
        r#"Object.freeze((globalThis["Math"]["cbrt"]))"#,
        r#"Object.freeze(globalThis["Math"]['cbrt'])"#,
        r#"Object.freeze((globalThis["Math"]['cbrt']))"#,
        r#"Object.freeze((globalThis["Math"]))["cbrt"]"#,
        r#"Object.freeze((globalThis["Math"]))['cbrt']"#,
        r#"Object.freeze((globalThis.Math))["cbrt"]"#,
        r#"Object.freeze((globalThis.Math))['cbrt']"#,
        r#"Object.freeze((globalThis["Math"]).cbrt)"#,
        r#"Object.freeze((globalThis["Math"])["cbrt"])"#,
        r#"Object.freeze(globalThis["Math"].cbrt)"#,
        r#"Object.freeze((globalThis["Math"].cbrt))"#,
        r#"Object.freeze((globalThis['Math'])["cbrt"])"#,
        r#"Object.freeze((globalThis['Math'])['cbrt'])"#,
        r#"Object.freeze((globalThis['Math']))["cbrt"]"#,
        r#"Object.freeze((globalThis['Math']))['cbrt']"#,
        r#"Object.freeze(globalThis['Math'].cbrt)"#,
        r#"Object.freeze((globalThis['Math'].cbrt))"#,
        r#"Object.freeze(Math.cbrt)"#,
        r#"Object.freeze((Math.cbrt))"#,
        r#"Object.freeze(Math["cbrt"])"#,
        r#"Object.freeze((Math["cbrt"]))"#,
        r#"Object.freeze(Math['cbrt'])"#,
        r#"Object.freeze((Math['cbrt']))"#,
    ]
}

/// Canonical source text for the supported `Math.cbrt` frozen callable aliases.
pub fn math_cbrt_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_cbrt_frozen_callable_aliases())
}

/// Canonical frozen callable aliases for the supported `Math.hypot` helper slice.
pub const fn math_hypot_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["hypot"])"#,
        r#"Object.freeze((globalThis.Math["hypot"]))"#,
        r#"Object.freeze(globalThis.Math['hypot'])"#,
        r#"Object.freeze((globalThis.Math['hypot']))"#,
        r#"Object.freeze(globalThis.Math.hypot)"#,
        r#"Object.freeze((globalThis.Math.hypot))"#,
        r#"Object.freeze(globalThis["Math"]["hypot"])"#,
        r#"Object.freeze((globalThis["Math"]["hypot"]))"#,
        r#"Object.freeze(globalThis["Math"]['hypot'])"#,
        r#"Object.freeze((globalThis["Math"]['hypot']))"#,
        r#"Object.freeze((globalThis["Math"]).hypot)"#,
        r#"Object.freeze((globalThis["Math"])["hypot"])"#,
        r#"Object.freeze((globalThis["Math"])['hypot'])"#,
        r#"Object.freeze(globalThis["Math"].hypot)"#,
        r#"Object.freeze((globalThis["Math"].hypot))"#,
        r#"Object.freeze(globalThis['Math']['hypot'])"#,
        r#"Object.freeze((globalThis['Math']['hypot']))"#,
        r#"Object.freeze((globalThis['Math']).hypot)"#,
        r#"Object.freeze((globalThis['Math'])["hypot"])"#,
        r#"Object.freeze((globalThis['Math'])['hypot'])"#,
        r#"Object.freeze((globalThis["Math"]))["hypot"]"#,
        r#"Object.freeze((globalThis['Math']))["hypot"]"#,
        r#"Object.freeze((globalThis.Math))["hypot"]"#,
        r#"Object.freeze((globalThis.Math))['hypot']"#,
        r#"Object.freeze(globalThis['Math'].hypot)"#,
        r#"Object.freeze((globalThis['Math'].hypot))"#,
        r#"Object.freeze(Math.hypot)"#,
        r#"Object.freeze((Math.hypot))"#,
        r#"Object.freeze(Math["hypot"])"#,
        r#"Object.freeze((Math["hypot"]))"#,
        r#"Object.freeze(Math['hypot'])"#,
        r#"Object.freeze((Math['hypot']))"#,
    ]
}

/// Canonical source text for the supported `Math.hypot` frozen callable aliases.
pub fn math_hypot_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_hypot_frozen_callable_aliases())
}

/// Canonical frozen callable aliases for the supported `Math.exp2` helper slice.
pub const fn math_exp2_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Math["exp2"])"#,
        r#"Object.freeze((globalThis.Math["exp2"]))"#,
        r#"Object.freeze(globalThis.Math['exp2'])"#,
        r#"Object.freeze((globalThis.Math['exp2']))"#,
        r#"Object.freeze(globalThis.Math.exp2)"#,
        r#"Object.freeze((globalThis.Math.exp2))"#,
        r#"Object.freeze(globalThis?.Math.exp2)"#,
        r#"Object.freeze((globalThis?.Math.exp2))"#,
        r#"Object.freeze(globalThis["Math"]["exp2"])"#,
        r#"Object.freeze((globalThis["Math"]["exp2"]))"#,
        r#"Object.freeze(globalThis["Math"]['exp2'])"#,
        r#"Object.freeze((globalThis["Math"]['exp2']))"#,
        r#"Object.freeze(globalThis["Math"].exp2)"#,
        r#"Object.freeze((globalThis["Math"]).exp2)"#,
        r#"Object.freeze((globalThis["Math"].exp2))"#,
        r#"Object.freeze((globalThis["Math"])["exp2"])"#,
        r#"Object.freeze((globalThis['Math'])['exp2'])"#,
        r#"Object.freeze((globalThis['Math'])["exp2"])"#,
        r#"Object.freeze(globalThis['Math']['exp2'])"#,
        r#"Object.freeze((globalThis['Math']['exp2']))"#,
        r#"Object.freeze(globalThis['Math'].exp2)"#,
        r#"Object.freeze((globalThis['Math']).exp2)"#,
        r#"Object.freeze((globalThis['Math'].exp2))"#,
        r#"Object.freeze(Math.exp2)"#,
        r#"Object.freeze((Math.exp2))"#,
        r#"Object.freeze(Math["exp2"])"#,
        r#"Object.freeze((Math["exp2"]))"#,
        r#"Object.freeze(Math['exp2'])"#,
        r#"Object.freeze((Math['exp2']))"#,
        r#"Object.freeze((null ?? globalThis.Math["exp2"]))"#,
        r#"Object.freeze((true && globalThis.Math["exp2"]))"#,
        r#"Object.freeze((false || globalThis.Math["exp2"]))"#,
        r#"Object.freeze((null ?? globalThis["Math"].exp2))"#,
        r#"Object.freeze((true && globalThis["Math"].exp2))"#,
        r#"Object.freeze((false || globalThis["Math"].exp2))"#,
        r#"Object.freeze((null ?? Math.exp2))"#,
        r#"Object.freeze((true && globalThis.Math.exp2))"#,
        r#"Object.freeze((false || globalThis.Math.exp2))"#,
        r#"Object.freeze((null ?? globalThis["Math"]["exp2"]))"#,
        r#"Object.freeze((true && globalThis["Math"]["exp2"]))"#,
        r#"Object.freeze((false || globalThis["Math"]["exp2"]))"#,
    ]
}

/// Canonical source text for the supported `Math.exp2` frozen callable aliases.
pub fn math_exp2_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(math_exp2_frozen_callable_aliases())
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod math_tests;
