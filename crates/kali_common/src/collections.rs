use crate::*;

/// Canonical root aliases for the supported `Set` constructor slice.
pub const fn set_constructor_aliases() -> &'static [&'static str] {
    &[
        "Set",
        "globalThis.Set",
        r#"globalThis["Set"]"#,
        r#"globalThis['Set']"#,
    ]
}

/// Canonical frozen callable aliases for the supported `Set` constructor slice.
pub const fn set_constructor_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(Set)"#,
        r#"Object.freeze((Set))"#,
        r#"Object.freeze((null ?? Set))"#,
        r#"Object.freeze((true && Set))"#,
        r#"Object.freeze((false || Set))"#,
        r#"Object.freeze(globalThis.Set)"#,
        r#"Object.freeze((globalThis.Set))"#,
        r#"Object.freeze((null ?? globalThis.Set))"#,
        r#"Object.freeze((true && globalThis.Set))"#,
        r#"Object.freeze((false || globalThis.Set))"#,
        r#"Object.freeze(globalThis["Set"])"#,
        r#"Object.freeze((globalThis["Set"]))"#,
        r#"Object.freeze((null ?? globalThis["Set"]))"#,
        r#"Object.freeze((true && globalThis["Set"]))"#,
        r#"Object.freeze((false || globalThis["Set"]))"#,
        r#"Object.freeze(globalThis['Set'])"#,
        r#"Object.freeze((globalThis['Set']))"#,
        r#"Object.freeze((null ?? globalThis['Set']))"#,
        r#"Object.freeze((true && globalThis['Set']))"#,
        r#"Object.freeze((false || globalThis['Set']))"#,
    ]
}

/// Canonical source text for the supported `Set` constructor aliases.
pub fn set_constructor_source() -> String {
    join_semicolon_terminated_segments(set_constructor_aliases())
}

/// Canonical source text for the supported `Set` constructor iteration smoke.
pub fn set_constructor_iteration_source() -> String {
    concat!(
        "for (const value of new Set([1, 2, 1])) { console.log(value); } ",
        "for (const value of new Set(Object.freeze([1, 2, 1]))) { console.log(value); } ",
        "for (const value of new globalThis.Set([1, 2, 1])) { console.log(value); } ",
        "for (const value of new globalThis[\"Set\"]([1, 2, 1])) { console.log(value); } ",
        "for (const value of new globalThis['Set']([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (globalThis['Set'])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new globalThis['Set'](Object.freeze([1, 2, 1]))) { console.log(value); } ",
        "for (const value of new (Object.freeze((Set)))([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (Object.freeze((globalThis.Set)))([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (Object.freeze((globalThis[\"Set\"])))([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (Object.freeze((globalThis['Set'])))([1, 2, 1])) { console.log(value); } ",
        "for (const value of Object.freeze(new Set([1, 2, 1]))) { console.log(value); } ",
        "for (const value of Object.freeze((new Set([1, 2, 1])))) { console.log(value); } ",
        "for (const value of Object.freeze((null ?? new Set([1, 2, 1])))) { console.log(value); } ",
        "for (const value of Object.freeze((true && new Set([1, 2, 1])))) { console.log(value); } ",
        "for (const value of Object.freeze((false || new Set([1, 2, 1])))) { console.log(value); } ",
        "for (const value of Object.freeze(new globalThis[\"Set\"]([1, 2, 1]))) { console.log(value); } ",
        "for (const value of Object.freeze((new globalThis[\"Set\"]([1, 2, 1])))) { console.log(value); } ",
        "for (const value of new (null ?? Set)([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (true && Set)([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (false || Set)([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (null ?? globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (true && globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (false || globalThis[\"Set\"])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (null ?? globalThis['Set'])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (true && globalThis['Set'])([1, 2, 1])) { console.log(value); } ",
        "for (const value of new (false || globalThis['Set'])([1, 2, 1])) { console.log(value); }"
    )
    .to_string()
}

/// Canonical source text for the supported `Set` frozen callable aliases.
pub fn set_constructor_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(set_constructor_frozen_callable_aliases())
}

/// Canonical root aliases for the supported `Map` constructor slice.
pub const fn map_constructor_aliases() -> &'static [&'static str] {
    &[
        "Map",
        "globalThis.Map",
        r#"globalThis["Map"]"#,
        r#"globalThis['Map']"#,
    ]
}

/// Canonical frozen callable aliases for the supported `Map` constructor slice.
pub const fn map_constructor_frozen_callable_aliases() -> &'static [&'static str] {
    &[
        r#"Object.freeze(Map)"#,
        r#"Object.freeze((Map))"#,
        r#"Object.freeze((null ?? Map))"#,
        r#"Object.freeze((true && Map))"#,
        r#"Object.freeze((false || Map))"#,
        r#"Object.freeze(globalThis.Map)"#,
        r#"Object.freeze((globalThis.Map))"#,
        r#"Object.freeze((null ?? globalThis.Map))"#,
        r#"Object.freeze((true && globalThis.Map))"#,
        r#"Object.freeze((false || globalThis.Map))"#,
        r#"Object.freeze(globalThis["Map"])"#,
        r#"Object.freeze((globalThis["Map"]))"#,
        r#"Object.freeze((null ?? globalThis["Map"]))"#,
        r#"Object.freeze((true && globalThis["Map"]))"#,
        r#"Object.freeze((false || globalThis["Map"]))"#,
        r#"Object.freeze(globalThis['Map'])"#,
        r#"Object.freeze((globalThis['Map']))"#,
        r#"Object.freeze((null ?? globalThis['Map']))"#,
        r#"Object.freeze((true && globalThis['Map']))"#,
        r#"Object.freeze((false || globalThis['Map']))"#,
    ]
}

/// Canonical source text for the supported `Map` constructor aliases.
pub fn map_constructor_source() -> String {
    join_semicolon_terminated_segments(map_constructor_aliases())
}

/// Canonical source text for the supported `Map` constructor iteration smoke.
pub fn map_constructor_iteration_source() -> String {
    concat!(
        "for (const entry of new Map([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new Map(Object.freeze([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new globalThis.Map([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new globalThis[\"Map\"]([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new globalThis['Map']([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new globalThis['Map'](Object.freeze([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (Object.freeze((Map)))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (Object.freeze((globalThis.Map)))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (Object.freeze((globalThis[\"Map\"])))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (Object.freeze((globalThis['Map'])))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze(new Map([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze((new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze((null ?? new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze((true && new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze((false || new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze(new globalThis[\"Map\"]([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of Object.freeze((new globalThis[\"Map\"]([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (null ?? Map)([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (true && Map)([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (false || Map)([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (null ?? globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (true && globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (false || globalThis[\"Map\"])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (null ?? globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (true && globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
        "for (const entry of new (false || globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); }"
    )
    .to_string()
}

/// Canonical source text for the supported `Map` frozen callable aliases.
pub fn map_constructor_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(map_constructor_frozen_callable_aliases())
}

#[cfg(test)]
#[path = "collections_tests.rs"]
mod collections_tests;
