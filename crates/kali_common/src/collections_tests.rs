use crate::*;

#[test]
fn test_set_constructor_aliases_and_frozen_callable_source_are_canonical() {
    let aliases = set_constructor_aliases();
    let frozen_aliases = set_constructor_frozen_callable_aliases();
    let source = set_constructor_source();
    let iteration_source = set_constructor_iteration_source();
    let frozen_source = set_constructor_frozen_callable_source();

    assert_eq!(
        aliases,
        &[
            "Set",
            "globalThis.Set",
            r#"globalThis["Set"]"#,
            r#"globalThis['Set']"#
        ]
    );
    assert_eq!(
        source,
        "Set; globalThis.Set; globalThis[\"Set\"]; globalThis['Set'];"
    );
    assert_eq!(
        frozen_aliases,
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
    );
    assert_eq!(
        iteration_source,
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
    );
    assert_eq!(
        frozen_source,
        concat!(
            "Object.freeze(Set); Object.freeze((Set)); Object.freeze((null ?? Set)); ",
            "Object.freeze((true && Set)); Object.freeze((false || Set)); Object.freeze(globalThis.Set); ",
            "Object.freeze((globalThis.Set)); Object.freeze((null ?? globalThis.Set)); ",
            "Object.freeze((true && globalThis.Set)); Object.freeze((false || globalThis.Set)); ",
            "Object.freeze(globalThis[\"Set\"]); Object.freeze((globalThis[\"Set\"])); ",
            "Object.freeze((null ?? globalThis[\"Set\"])); Object.freeze((true && globalThis[\"Set\"])); ",
            "Object.freeze((false || globalThis[\"Set\"])); Object.freeze(globalThis['Set']); ",
            "Object.freeze((globalThis['Set'])); Object.freeze((null ?? globalThis['Set'])); ",
            "Object.freeze((true && globalThis['Set'])); Object.freeze((false || globalThis['Set']));"
        )
    );
}

#[test]
fn test_map_constructor_aliases_and_frozen_callable_source_are_canonical() {
    let aliases = map_constructor_aliases();
    let frozen_aliases = map_constructor_frozen_callable_aliases();
    let source = map_constructor_source();
    let iteration_source = map_constructor_iteration_source();
    let frozen_source = map_constructor_frozen_callable_source();

    assert_eq!(
        aliases,
        &[
            "Map",
            "globalThis.Map",
            r#"globalThis["Map"]"#,
            r#"globalThis['Map']"#
        ]
    );
    assert_eq!(
        source,
        "Map; globalThis.Map; globalThis[\"Map\"]; globalThis['Map'];"
    );
    assert_eq!(
        frozen_aliases,
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
    );
    assert_eq!(
        iteration_source,
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
    );
    assert_eq!(
        frozen_source,
        concat!(
            "Object.freeze(Map); Object.freeze((Map)); Object.freeze((null ?? Map)); ",
            "Object.freeze((true && Map)); Object.freeze((false || Map)); Object.freeze(globalThis.Map); ",
            "Object.freeze((globalThis.Map)); Object.freeze((null ?? globalThis.Map)); ",
            "Object.freeze((true && globalThis.Map)); Object.freeze((false || globalThis.Map)); ",
            "Object.freeze(globalThis[\"Map\"]); Object.freeze((globalThis[\"Map\"])); ",
            "Object.freeze((null ?? globalThis[\"Map\"])); Object.freeze((true && globalThis[\"Map\"])); ",
            "Object.freeze((false || globalThis[\"Map\"])); Object.freeze(globalThis['Map']); ",
            "Object.freeze((globalThis['Map'])); Object.freeze((null ?? globalThis['Map'])); ",
            "Object.freeze((true && globalThis['Map'])); Object.freeze((false || globalThis['Map']));"
        )
    );
}
