use super::*;

#[test]
fn test_promise_race_browser_body_source_includes_the_shared_freeze_wrapper_aliases() {
    let body = promise_race_browser_body_source();

    assert!(
        body.contains(
            "const mixed = await Promise[\"race\"]([Promise.resolve(1), Promise.resolve(2)]);"
        ),
        "body: {body}"
    );
    assert!(
        body.contains(
            "const singleMixed = await Promise['race']([Promise.resolve(1), Promise.resolve(2)]);"
        ),
        "body: {body}"
    );
    assert!(
        body.contains("const bracketed = await globalThis[\"Promise\"].race([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleBracketed = await globalThis['Promise'].race([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const mixedDotted = await globalThis.Promise[\"race\"]([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleDotted = await globalThis.Promise['race']([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const bracketedBracketed = await globalThis[\"Promise\"][\"race\"]([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleBracketedBracketed = await globalThis['Promise']['race']([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedDottedBracketed = await Object.freeze((globalThis.Promise)[\"race\"])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleDottedBracketed = await Object.freeze((globalThis.Promise)['race'])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedBracketedBracketed = await Object.freeze((globalThis[\"Promise\"][\"race\"]))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleBracketedBracketed = await Object.freeze((globalThis['Promise']['race']))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenRoot = await Object.freeze(Promise.race)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenRoot = await Object.freeze((Promise.race))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenBracketed = await Object.freeze(globalThis[\"Promise\"].race)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleBracketed = await Object.freeze(globalThis['Promise'].race)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const frozenDottedBracketed = await Object.freeze(globalThis.Promise["race"])([Promise.resolve(1), Promise.resolve(2)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleDottedBracketed = await Object.freeze(globalThis.Promise['race'])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenBracketedBracketed = await Object.freeze(globalThis[\"Promise\"][\"race\"])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleBracketedBracketed = await Object.freeze(globalThis['Promise']['race'])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenDotted = await Object.freeze(globalThis.Promise.race)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenDotted = await Object.freeze((globalThis.Promise.race))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("throw new Error('unexpected Promise.race semantics');"),
        "body: {body}"
    );
    assert!(body.contains("  }\n"), "body: {body}");
}

#[test]
fn test_promise_any_browser_body_source_includes_the_shared_freeze_wrapper_aliases() {
    let body = promise_any_browser_body_source();

    assert!(
        body.contains(
            "const direct = await Promise.any([Promise.reject('boom'), Promise.resolve(1)]);"
        ),
        "body: {body}"
    );
    assert!(
        body.contains(
            "const mixed = await Promise[\"any\"]([Promise.reject('boom'), Promise.resolve(1)]);"
        ),
        "body: {body}"
    );
    assert!(
        body.contains("const singleMixed = await Promise['any']([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const bracketed = await globalThis[\"Promise\"].any([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedBracketed = await Object.freeze((globalThis[\"Promise\"])[\"any\"])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleBracketed = await globalThis['Promise']['any']([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedDottedBracketed = await Object.freeze((globalThis.Promise)[\"any\"])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleDottedBracketed = await Object.freeze((globalThis.Promise)['any'])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleBracketed = await Object.freeze((globalThis['Promise'])['any'])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleMixedBracketed = await globalThis['Promise'].any([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const frozenBracketed = await Object.freeze(globalThis["Promise"].any)([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleBracketed = await Object.freeze(globalThis['Promise'].any)([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const frozenMixedBracketed = await Object.freeze(globalThis["Promise"]["any"])([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleBracketRoot = await Object.freeze(globalThis['Promise']['any'])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const frozenSingleMixedBracketed = await Object.freeze(globalThis["Promise"]['any'])([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const parenthesizedFrozenMixedBracketed = await Object.freeze((globalThis["Promise"]["any"]))([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenSingleBracketRoot = await Object.freeze((globalThis['Promise']['any']))([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const parenthesizedFrozenSingleMixedBracketed = await Object.freeze((globalThis["Promise"]['any']))([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const parenthesizedFrozenReceiverWrappedDotted = await Object.freeze((globalThis["Promise"]).any)([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenSingleReceiverWrappedDotted = await Object.freeze((globalThis['Promise']).any)([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const nullishRoot = await Object.freeze((null ?? Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalOrRoot = await Object.freeze((false || Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenRoot = await Object.freeze(Promise.any)([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains(r#"const parenthesizedFrozenDottedBracketed = await Object.freeze((globalThis.Promise)["any"])([Promise.reject('boom'), Promise.resolve(1)]);"#),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenSingleDottedBracketed = await Object.freeze((globalThis.Promise)['any'])([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenDotted = await Object.freeze((globalThis.Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("throw new Error('unexpected Promise.any semantics');"),
        "body: {body}"
    );
    assert!(body.contains("  }\n"), "body: {body}");
}
