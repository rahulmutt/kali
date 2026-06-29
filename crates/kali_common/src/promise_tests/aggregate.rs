use super::*;

#[test]
fn test_promise_all_settled_browser_body_source_includes_the_shared_freeze_wrapper_aliases() {
    let body = promise_all_settled_browser_body_source();

    assert!(
        body.contains("const singleMixedSettled = await Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleDottedSettled = await globalThis.Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleBracketedSettled = await globalThis['Promise']['allSettled']([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleMixedBracketedSettled = await globalThis['Promise'].allSettled([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const nullishRootSettled = await Object.freeze((null ?? Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalAndRootSettled = await Object.freeze((true && Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalOrRootSettled = await Object.freeze((false || Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const nullishDottedSettled = await Object.freeze((null ?? globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalAndDottedSettled = await Object.freeze((true && globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalOrDottedSettled = await Object.freeze((false || globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const wrappedDottedRootFrozenSettled = await Object.freeze((globalThis.Promise)[\"allSettled\"])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const wrappedBracketedRootFrozenSettled = await Object.freeze((globalThis[\"Promise\"])[\"allSettled\"])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const wrappedBracketedDotRootFrozenSettled = await Object.freeze((globalThis[\"Promise\"]).allSettled)([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const wrappedSingleBracketedDotRootFrozenSettled = await Object.freeze((globalThis['Promise']).allSettled)([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenBracketedSettled = await Object.freeze(globalThis[\"Promise\"][\"allSettled\"])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenBracketedSettled = await Object.freeze((globalThis[\"Promise\"][\"allSettled\"]))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleFrozenBracketedSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleFrozenBracketedSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const mixedBracketedRootFrozenSettled = await Object.freeze(globalThis[\"Promise\"].allSettled)([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedMixedBracketedRootFrozenSettled = await Object.freeze((globalThis[\"Promise\"].allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleMixedBracketedRootFrozenSettled = await Object.freeze(globalThis['Promise'].allSettled)([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const fullyBracketedSingleRootFrozenSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFullyBracketedSingleRootFrozenSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleMixedBracketedRootFrozenSettled = await Object.freeze((globalThis['Promise'].allSettled))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const mixedRootFrozenSettled = await Object.freeze(globalThis.Promise[\"allSettled\"])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedMixedRootFrozenSettled = await Object.freeze((globalThis.Promise[\"allSettled\"]))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleMixedRootFrozenSettled = await Object.freeze(globalThis.Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleMixedRootFrozenSettled = await Object.freeze((globalThis.Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const bracketedRootFrozenSettled = await Object.freeze(Promise[\"allSettled\"])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedBracketedRootFrozenSettled = await Object.freeze((Promise[\"allSettled\"]))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const singleBracketedRootFrozenSettled = await Object.freeze(Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedSingleBracketedRootFrozenSettled = await Object.freeze((Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("parenthesizedFrozenBracketedSettled.length !== 2"),
        "body: {body}"
    );
    assert!(
        body.contains("throw new Error('unexpected Promise.allSettled semantics');"),
        "body: {body}"
    );
    assert!(body.contains("  }\n"), "body: {body}");
}

#[test]
fn test_promise_all_browser_body_source_includes_the_shared_freeze_wrapper_aliases() {
    let body = promise_all_browser_body_source();

    assert!(
        body.contains(
            "const singleMixed = await Promise['all']([Promise.resolve(1), Promise.resolve(2)]);"
        ),
        "body: {body}"
    );
    assert!(
        body.contains("const nullishRoot = await Object.freeze((null ?? Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalAndRoot = await Object.freeze((true && Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const nullishDotted = await Object.freeze((null ?? globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const logicalOrDotted = await Object.freeze((false || globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenRoot = await Object.freeze(Promise.all)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenRoot = await Object.freeze((Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenBracketedRoot = await Object.freeze(Promise[\"all\"])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenBracketedRoot = await Object.freeze((Promise[\"all\"]))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenSingleBracketedRoot = await Object.freeze(Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenSingleBracketedRoot = await Object.freeze((Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const mixedRoot = await Object.freeze(globalThis.Promise[\"all\"])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const mixedBracketedRoot = await Object.freeze(globalThis[\"Promise\"][\"all\"])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const fullyBracketedSingleRoot = await Object.freeze(globalThis['Promise']['all'])([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const frozenGlobal = await Object.freeze(globalThis.Promise.all)([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenGlobal = await Object.freeze((globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);"),
        "body: {body}"
    );
    assert!(
        body.contains("throw new Error(\"unexpected Promise.all results\");"),
        "body: {body}"
    );
}
