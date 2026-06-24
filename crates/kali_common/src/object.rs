use crate::*;

/// Canonical frozen callable aliases for the supported `Object.hasOwn` helper slice.
pub const fn object_has_own_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(Object.hasOwn)"#,
        r#"Object.freeze((Object.hasOwn))"#,
        r#"Object.freeze(Object["hasOwn"])"#,
        r#"Object.freeze((Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis.Object.hasOwn)"#,
        r#"Object.freeze((globalThis.Object.hasOwn))"#,
        r#"Object.freeze(globalThis.Object["hasOwn"])"#,
        r#"Object.freeze(globalThis.Object['hasOwn'])"#,
        r#"Object.freeze((globalThis.Object)["hasOwn"])"#,
        r#"Object.freeze((globalThis.Object).hasOwn)"#,
        r#"Object.freeze((globalThis.Object)['hasOwn'])"#,
        r#"Object.freeze((globalThis.Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis?.Object.hasOwn)"#,
        r#"Object.freeze((globalThis?.Object.hasOwn))"#,
        r#"Object.freeze((globalThis?.Object).hasOwn)"#,
        r#"Object.freeze((globalThis?.Object)["hasOwn"])"#,
        r#"Object.freeze(globalThis?.Object["hasOwn"])"#,
        r#"Object.freeze((globalThis?.Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis["Object"].hasOwn)"#,
        r#"Object.freeze((globalThis["Object"].hasOwn))"#,
        r#"Object.freeze((globalThis["Object"]).hasOwn)"#,
        r#"Object.freeze((globalThis["Object"])["hasOwn"])"#,
        r#"Object.freeze(globalThis["Object"]["hasOwn"])"#,
        r#"Object.freeze((globalThis["Object"]["hasOwn"]))"#,
        r#"Object.freeze((globalThis["Object"]))["hasOwn"]"#,
        r#"Object.freeze((globalThis["Object"]))['hasOwn']"#,
        r#"Object.freeze((globalThis['Object']))["hasOwn"]"#,
        r#"Object.freeze((globalThis['Object']))['hasOwn']"#,
        r#"Object.freeze(globalThis['Object'].hasOwn)"#,
        r#"Object.freeze((globalThis['Object'].hasOwn))"#,
        r#"Object.freeze((globalThis['Object']).hasOwn)"#,
        r#"Object.freeze((globalThis['Object'])['hasOwn'])"#,
        r#"Object.freeze(globalThis['Object']['hasOwn'])"#,
        r#"Object.freeze((globalThis['Object']['hasOwn']))"#,
        r#"Object.freeze((null ?? Object.hasOwn))"#,
        r#"Object.freeze((true && Object.hasOwn))"#,
        r#"Object.freeze((false || Object.hasOwn))"#,
    ]
}

/// Canonical source text for the supported `Object.hasOwn` frozen callable aliases.
pub fn object_has_own_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(object_has_own_frozen_callable_aliases())
}

/// Canonical frozen callable aliases for the supported `Reflect.ownKeys` helper slice.
pub const fn reflect_own_keys_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(Reflect.ownKeys)"#,
        r#"Object.freeze((Reflect.ownKeys))"#,
        r#"Object.freeze(globalThis.Reflect.ownKeys)"#,
        r#"Object.freeze((null ?? globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze((true && globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze((false || globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze(globalThis.Reflect["ownKeys"])"#,
        r#"Object.freeze(globalThis.Reflect['ownKeys'])"#,
        r#"Object.freeze((globalThis.Reflect['ownKeys']))"#,
        r#"Object.freeze(globalThis["Reflect"].ownKeys)"#,
        r#"Object.freeze(globalThis["Reflect"]['ownKeys'])"#,
        r#"Object.freeze(globalThis['Reflect']["ownKeys"])"#,
        r#"Object.freeze((globalThis["Reflect"]['ownKeys']))"#,
        r#"Object.freeze((globalThis['Reflect']["ownKeys"]))"#,
        r#"Object.freeze((globalThis.Reflect)["ownKeys"])"#,
        r#"Object.freeze((globalThis["Reflect"]).ownKeys)"#,
        r#"Object.freeze((globalThis['Reflect']).ownKeys)"#,
        r#"Object.freeze((globalThis["Reflect"])["ownKeys"])"#,
        r#"Object.freeze(globalThis["Reflect"]["ownKeys"])"#,
        r#"Object.freeze((globalThis["Reflect"].ownKeys))"#,
        r#"Object.freeze((globalThis["Reflect"]["ownKeys"]))"#,
        r#"Object.freeze((globalThis.Reflect["ownKeys"]))"#,
        r#"Object.freeze((globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze((globalThis['Reflect'].ownKeys))"#,
        r#"Object.freeze((globalThis['Reflect']['ownKeys']))"#,
        r#"Object.freeze((globalThis['Reflect'])['ownKeys'])"#,
        r#"Object.freeze(globalThis['Reflect'].ownKeys)"#,
        r#"Object.freeze(globalThis['Reflect']['ownKeys'])"#,
        r#"Object.freeze((null ?? Reflect.ownKeys))"#,
        r#"Object.freeze((true && Reflect.ownKeys))"#,
        r#"Object.freeze((false || Reflect.ownKeys))"#,
        r#"Object.freeze((true ? Reflect.ownKeys : Reflect.ownKeys))"#,
        r#"Object.freeze((true ? globalThis.Reflect.ownKeys : globalThis.Reflect.ownKeys))"#,
        r#"Object.freeze((null ?? globalThis["Reflect"]["ownKeys"]))"#,
        r#"Object.freeze((true && globalThis["Reflect"]["ownKeys"]))"#,
        r#"Object.freeze((false || globalThis["Reflect"]["ownKeys"]))"#,
    ]
}

/// Canonical source text for the supported `Reflect.ownKeys` frozen callable aliases.
pub fn reflect_own_keys_frozen_callable_source(object_source: &str) -> String {
    let statements = [
        format!("const frozenBareCallableKeys = Object.freeze(Reflect.ownKeys)({object_source})"),
        format!("const parenthesizedFrozenBareCallableKeys = Object.freeze((Reflect.ownKeys))({object_source})"),
        format!("const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)({object_source})"),
        format!(r#"const mixedBracketedRootKeys = Object.freeze(globalThis["Reflect"]['ownKeys'])({object_source})"#),
        format!(r#"const parenthesizedMixedBracketedRootKeys = Object.freeze((globalThis["Reflect"]['ownKeys']))({object_source})"#),
        format!(r#"const mixedSingleQuotedRootKeys = Object.freeze(globalThis['Reflect']["ownKeys"])({object_source})"#),
        format!(r#"const parenthesizedMixedSingleQuotedRootKeys = Object.freeze((globalThis['Reflect']["ownKeys"]))({object_source})"#),
        format!(r#"const frozenMixedBracketedKeys = Object.freeze(globalThis.Reflect["ownKeys"])({object_source})"#),
        format!(r#"const frozenSingleQuotedMixedBracketedKeys = Object.freeze(globalThis.Reflect['ownKeys'])({object_source})"#),
        format!(r#"const parenthesizedFrozenSingleQuotedMixedBracketedKeys = Object.freeze((globalThis.Reflect['ownKeys']))({object_source})"#),
        format!(r#"const nullishFrozenCallableKeys = Object.freeze((null ?? globalThis.Reflect.ownKeys))({object_source})"#),
        format!(r#"const logicalAndFrozenCallableKeys = Object.freeze((true && globalThis.Reflect.ownKeys))({object_source})"#),
        format!(r#"const logicalOrFrozenCallableKeys = Object.freeze((false || globalThis.Reflect.ownKeys))({object_source})"#),
        format!(r#"const frozenMixedRootKeys = Object.freeze(globalThis["Reflect"].ownKeys)({object_source})"#),
        format!(r#"const parenthesizedFrozenDotRootBracketedKeys = Object.freeze((globalThis.Reflect)["ownKeys"])({object_source})"#),
        format!(r#"const frozenParenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"]).ownKeys)({object_source})"#),
        format!(r#"const frozenParenthesizedSingleQuotedBracketRootKeys = Object.freeze((globalThis['Reflect']).ownKeys)({object_source})"#),
        format!(r#"const frozenParenthesizedBracketedRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])({object_source})"#),
        format!(r#"const frozenParenthesizedMixedRootKeys = Object.freeze((globalThis["Reflect"].ownKeys))({object_source})"#),
        format!(r#"const frozenBracketedKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])({object_source})"#),
        format!(r#"const parenthesizedFrozenMixedBracketedKeys = Object.freeze((globalThis.Reflect["ownKeys"]))({object_source})"#),
        format!(r#"const parenthesizedFrozenBracketedKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))({object_source})"#),
        format!("const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))({object_source})"),
        format!(r#"const frozenSingleQuotedRootKeys = Object.freeze(globalThis['Reflect'].ownKeys)({object_source})"#),
        format!(r#"const nullishFrozenBracketedKeys = Object.freeze((null ?? globalThis["Reflect"]["ownKeys"]))({object_source})"#),
        format!(r#"const logicalAndFrozenBracketedKeys = Object.freeze((true && globalThis["Reflect"]["ownKeys"]))({object_source})"#),
        format!(r#"const logicalOrFrozenBracketedKeys = Object.freeze((false || globalThis["Reflect"]["ownKeys"]))({object_source})"#),
        format!(r#"const frozenParenthesizedSingleQuotedRootKeys = Object.freeze((globalThis['Reflect']).ownKeys)({object_source})"#),
        format!(r#"const frozenParenthesizedSingleQuotedBracketedKeys = Object.freeze((globalThis['Reflect'])['ownKeys'])({object_source})"#),
        format!(r#"const frozenSingleQuotedBracketedKeys = Object.freeze(globalThis['Reflect']['ownKeys'])({object_source})"#),
        format!(r#"const parenthesizedFrozenSingleQuotedRootKeys = Object.freeze((globalThis['Reflect'].ownKeys))({object_source})"#),
        format!(r#"const parenthesizedFrozenSingleQuotedBracketedKeys = Object.freeze((globalThis['Reflect']['ownKeys']))({object_source})"#),
        format!("const frozenNullishCallableKeys = Object.freeze((null ?? Reflect.ownKeys))({object_source})"),
        format!("const frozenLogicalAndCallableKeys = Object.freeze((true && Reflect.ownKeys))({object_source})"),
        format!("const frozenLogicalOrCallableKeys = Object.freeze((false || Reflect.ownKeys))({object_source})"),
        format!("const conditionalFrozenCallableKeys = Object.freeze((true ? Reflect.ownKeys : Reflect.ownKeys))({object_source})"),
        format!("const conditionalFrozenGlobalCallableKeys = Object.freeze((true ? globalThis.Reflect.ownKeys : globalThis.Reflect.ownKeys))({object_source})"),
    ];
    join_semicolon_terminated_segments(&statements.iter().map(String::as_str).collect::<Vec<_>>())
}

/// Canonical frozen callable aliases for the supported `Object.keys` / `Object.values` / `Object.entries` helper slice.
pub const fn object_enumeration_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze((globalThis["Object"]).keys)"#,
        r#"Object.freeze((globalThis["Object"])["keys"])"#,
        r#"Object.freeze((globalThis["Object"]).values)"#,
        r#"Object.freeze((globalThis["Object"])["values"])"#,
        r#"Object.freeze((globalThis["Object"]).entries)"#,
        r#"Object.freeze((globalThis["Object"])["entries"])"#,
        r#"Object.freeze((globalThis["Object"]["keys"]))"#,
        r#"Object.freeze((globalThis["Object"]))["keys"]"#,
        r#"Object.freeze((globalThis['Object']))['keys']"#,
        r#"Object.freeze((globalThis["Object"]["values"]))"#,
        r#"Object.freeze((globalThis["Object"]))["values"]"#,
        r#"Object.freeze((globalThis['Object']))['values']"#,
        r#"Object.freeze((globalThis["Object"]["entries"]))"#,
        r#"Object.freeze((globalThis["Object"]))["entries"]"#,
        r#"Object.freeze((globalThis['Object']))['entries']"#,
        r#"Object.freeze((globalThis['Object']).keys)"#,
        r#"Object.freeze((globalThis['Object'])['keys'])"#,
        r#"Object.freeze((globalThis['Object'])["keys"])"#,
        r#"Object.freeze((globalThis['Object']).values)"#,
        r#"Object.freeze((globalThis['Object'])['values'])"#,
        r#"Object.freeze((globalThis['Object'])["values"])"#,
        r#"Object.freeze((globalThis['Object']).entries)"#,
        r#"Object.freeze((globalThis['Object'])['entries'])"#,
        r#"Object.freeze((globalThis['Object'])["entries"])"#,
        r#"Object.freeze(globalThis["Object"]["keys"])"#,
        r#"Object.freeze(globalThis["Object"]['keys'])"#,
        r#"Object.freeze(globalThis["Object"]['values'])"#,
        r#"Object.freeze(globalThis["Object"]['entries'])"#,
        r#"Object.freeze(globalThis["Object"]["values"])"#,
        r#"Object.freeze(globalThis["Object"]["entries"])"#,
        r#"Object.freeze(globalThis.Object['keys'])"#,
        r#"Object.freeze(globalThis.Object['values'])"#,
        r#"Object.freeze(globalThis.Object['entries'])"#,
        r#"Object.freeze(globalThis.Object["keys"])"#,
        r#"Object.freeze(globalThis.Object["values"])"#,
        r#"Object.freeze(globalThis.Object["entries"])"#,
        r#"Object.freeze((globalThis.Object["keys"]))"#,
        r#"Object.freeze((globalThis.Object)['keys'])"#,
        r#"Object.freeze((globalThis.Object['keys']))"#,
        r#"Object.freeze((globalThis.Object["values"]))"#,
        r#"Object.freeze((globalThis.Object)['values'])"#,
        r#"Object.freeze((globalThis.Object['values']))"#,
        r#"Object.freeze((globalThis.Object["entries"]))"#,
        r#"Object.freeze((globalThis.Object)['entries'])"#,
        r#"Object.freeze((globalThis.Object['entries']))"#,
        r#"Object.freeze(globalThis['Object']["keys"])"#,
        r#"Object.freeze(globalThis['Object']["values"])"#,
        r#"Object.freeze(globalThis['Object']["entries"])"#,
    ]
}

/// Canonical source text for the supported `Object.keys` / `Object.values` / `Object.entries` helper slice.
pub fn object_enumeration_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(object_enumeration_frozen_callable_aliases())
}

/// Canonical boolean-check source for the supported `Object.hasOwn` frozen callable aliases.
pub fn object_has_own_frozen_callable_condition_source(
    receiver_source: &str,
    key_source: &str,
) -> String {
    object_has_own_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("!{alias}({receiver_source}, {key_source})"))
        .collect::<Vec<_>>()
        .join(" || ")
}

/// Canonical frozen callable aliases for the supported `Object.prototype.hasOwnProperty.call` helper slice.
pub const fn object_has_own_property_call_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(globalThis.Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))"#,
        r#"Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"]))"#,
        r#"Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call)"#,
        r#"Object.freeze((globalThis['Object']).prototype['hasOwnProperty'].call)"#,
        r#"Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"]))"#,
        r#"Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call)"#,
        r#"Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call))"#,
        r#"Object.freeze(globalThis['Object'].prototype['hasOwnProperty']['call'])"#,
        r#"Object.freeze((globalThis['Object'].prototype['hasOwnProperty']['call']))"#,
        r#"Object.freeze((globalThis['Object']).prototype['hasOwnProperty']['call'])"#,
        r#"Object.freeze((globalThis['Object'])['prototype']['hasOwnProperty']['call'])"#,
        r#"Object.freeze(globalThis['Object'].prototype['hasOwnProperty'].call)"#,
        r#"Object.freeze((globalThis['Object'].prototype['hasOwnProperty'].call))"#,
        r#"Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(globalThis["Object"].hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis["Object"].hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(Object.prototype["hasOwnProperty"].call)"#,
        r#"Object.freeze((Object.prototype["hasOwnProperty"].call))"#,
        r#"Object.freeze(Object["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze((Object["prototype"].hasOwnProperty.call))"#,
        r#"Object.freeze(Object["prototype"]["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((Object["prototype"]["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze((null ?? Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((true && Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((false || Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(Object.prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((Object.prototype.hasOwnProperty["call"]))"#,
        r#"Object.freeze(Object["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze((Object["prototype"].hasOwnProperty["call"]))"#,
    ]
}

/// Canonical source text for the supported `Object.prototype.hasOwnProperty.call` frozen callable aliases.
pub fn object_has_own_property_call_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(object_has_own_property_call_frozen_callable_aliases())
}

/// Canonical boolean-check source for the supported `Object.prototype.hasOwnProperty.call` frozen callable aliases.
pub fn object_has_own_property_call_frozen_callable_condition_source(
    receiver_source: &str,
    key_source: &str,
) -> String {
    object_has_own_property_call_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("!{alias}({receiver_source}, {key_source})"))
        .collect::<Vec<_>>()
        .join(" || ")
}

/// Canonical combined boolean-check source for the supported `Object.hasOwn` helper slice.
pub fn object_has_own_combined_frozen_callable_condition_source(
    receiver_source: &str,
    key_source: &str,
) -> String {
    format!(
        "{} || {}",
        object_has_own_frozen_callable_condition_source(receiver_source, key_source),
        object_has_own_property_call_frozen_callable_condition_source(receiver_source, key_source)
    )
}

/// Canonical source text for the supported `Object.prototype.hasOwnProperty.call` helper.
pub const fn object_has_own_property_call_source() -> &'static str {
    "Object.prototype.hasOwnProperty.call"
}

/// Canonical binding source for the supported `Object.prototype.hasOwnProperty.call` helper.
pub fn object_has_own_property_call_binding_source(binding_name: &str) -> String {
    format!(
        "const {binding_name} = {};",
        object_has_own_property_call_source()
    )
}

#[cfg(test)]
#[path = "object_tests.rs"]
mod object_tests;
