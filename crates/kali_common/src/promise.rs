/// Canonical browser smoke body for the supported `Promise.allSettled` slice.
pub const fn promise_all_settled_browser_body_source() -> &'static str {
    r#"  const settled = await Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);
  const mixedSettled = await Promise["allSettled"]([Promise.resolve(1), Promise.reject('boom')]);
  const singleMixedSettled = await Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);
  const dottedSettled = await globalThis.Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);
  const mixedDottedSettled = await globalThis.Promise["allSettled"]([Promise.resolve(1), Promise.reject('boom')]);
  const singleDottedSettled = await globalThis.Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);
  const mixedBracketedSettled = await globalThis["Promise"].allSettled([Promise.resolve(1), Promise.reject('boom')]);
  const bracketedSettled = await globalThis["Promise"]["allSettled"]([Promise.resolve(1), Promise.reject('boom')]);
  const singleBracketedSettled = await globalThis['Promise']['allSettled']([Promise.resolve(1), Promise.reject('boom')]);
  const singleMixedBracketedSettled = await globalThis['Promise'].allSettled([Promise.resolve(1), Promise.reject('boom')]);
  const nullishRootSettled = await Object.freeze((null ?? Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const logicalAndRootSettled = await Object.freeze((true && Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const logicalOrRootSettled = await Object.freeze((false || Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const nullishDottedSettled = await Object.freeze((null ?? globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const logicalAndDottedSettled = await Object.freeze((true && globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const logicalOrDottedSettled = await Object.freeze((false || globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const wrappedDottedRootFrozenSettled = await Object.freeze((globalThis.Promise)["allSettled"])([Promise.resolve(1), Promise.reject('boom')]);
  const wrappedBracketedRootFrozenSettled = await Object.freeze((globalThis["Promise"])["allSettled"])([Promise.resolve(1), Promise.reject('boom')]);
  const wrappedBracketedDotRootFrozenSettled = await Object.freeze((globalThis["Promise"]).allSettled)([Promise.resolve(1), Promise.reject('boom')]);
  const wrappedSingleBracketedDotRootFrozenSettled = await Object.freeze((globalThis['Promise']).allSettled)([Promise.resolve(1), Promise.reject('boom')]);
  const frozenBracketedSettled = await Object.freeze(globalThis["Promise"]["allSettled"])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedFrozenBracketedSettled = await Object.freeze((globalThis["Promise"]["allSettled"]))([Promise.resolve(1), Promise.reject('boom')]);
  const singleFrozenBracketedSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedSingleFrozenBracketedSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);
  const mixedBracketedRootFrozenSettled = await Object.freeze(globalThis["Promise"].allSettled)([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedMixedBracketedRootFrozenSettled = await Object.freeze((globalThis["Promise"].allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const singleMixedBracketedRootFrozenSettled = await Object.freeze(globalThis['Promise'].allSettled)([Promise.resolve(1), Promise.reject('boom')]);
  const fullyBracketedSingleRootFrozenSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedFullyBracketedSingleRootFrozenSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedSingleMixedBracketedRootFrozenSettled = await Object.freeze((globalThis['Promise'].allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  const mixedRootFrozenSettled = await Object.freeze(globalThis.Promise["allSettled"])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedMixedRootFrozenSettled = await Object.freeze((globalThis.Promise["allSettled"]))([Promise.resolve(1), Promise.reject('boom')]);
  const singleMixedRootFrozenSettled = await Object.freeze(globalThis.Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedSingleMixedRootFrozenSettled = await Object.freeze((globalThis.Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);
  const bracketedRootFrozenSettled = await Object.freeze(Promise["allSettled"])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedBracketedRootFrozenSettled = await Object.freeze((Promise["allSettled"]))([Promise.resolve(1), Promise.reject('boom')]);
  const singleBracketedRootFrozenSettled = await Object.freeze(Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedSingleBracketedRootFrozenSettled = await Object.freeze((Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);
  const rootFrozenSettled = await Object.freeze(Promise.allSettled)([Promise.resolve(1), Promise.reject('boom')]);
  const parenthesizedRootFrozenSettled = await Object.freeze((Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);
  if (
    settled.length !== 2 ||
    settled[0].status !== 'fulfilled' ||
    settled[0].value !== 1 ||
    settled[1].status !== 'rejected' ||
    settled[1].reason !== 'boom' ||
    mixedSettled.length !== 2 ||
    mixedSettled[0].status !== 'fulfilled' ||
    mixedSettled[0].value !== 1 ||
    mixedSettled[1].status !== 'rejected' ||
    mixedSettled[1].reason !== 'boom' ||
    dottedSettled.length !== 2 ||
    dottedSettled[0].status !== 'fulfilled' ||
    dottedSettled[0].value !== 1 ||
    dottedSettled[1].status !== 'rejected' ||
    dottedSettled[1].reason !== 'boom' ||
    mixedDottedSettled.length !== 2 ||
    mixedDottedSettled[0].status !== 'fulfilled' ||
    mixedDottedSettled[0].value !== 1 ||
    mixedDottedSettled[1].status !== 'rejected' ||
    mixedDottedSettled[1].reason !== 'boom' ||
    mixedBracketedSettled.length !== 2 ||
    mixedBracketedSettled[0].status !== 'fulfilled' ||
    mixedBracketedSettled[0].value !== 1 ||
    mixedBracketedSettled[1].status !== 'rejected' ||
    mixedBracketedSettled[1].reason !== 'boom' ||
    bracketedSettled.length !== 2 ||
    bracketedSettled[0].status !== 'fulfilled' ||
    bracketedSettled[0].value !== 1 ||
    bracketedSettled[1].status !== 'rejected' ||
    bracketedSettled[1].reason !== 'boom' ||
    nullishRootSettled.length !== 2 ||
    nullishRootSettled[0].status !== 'fulfilled' ||
    nullishRootSettled[0].value !== 1 ||
    nullishRootSettled[1].status !== 'rejected' ||
    nullishRootSettled[1].reason !== 'boom' ||
    logicalAndRootSettled.length !== 2 ||
    logicalAndRootSettled[0].status !== 'fulfilled' ||
    logicalAndRootSettled[0].value !== 1 ||
    logicalAndRootSettled[1].status !== 'rejected' ||
    logicalAndRootSettled[1].reason !== 'boom' ||
    logicalOrRootSettled.length !== 2 ||
    logicalOrRootSettled[0].status !== 'fulfilled' ||
    logicalOrRootSettled[0].value !== 1 ||
    logicalOrRootSettled[1].status !== 'rejected' ||
    logicalOrRootSettled[1].reason !== 'boom' ||
    nullishDottedSettled.length !== 2 ||
    nullishDottedSettled[0].status !== 'fulfilled' ||
    nullishDottedSettled[0].value !== 1 ||
    nullishDottedSettled[1].status !== 'rejected' ||
    nullishDottedSettled[1].reason !== 'boom' ||
    logicalAndDottedSettled.length !== 2 ||
    logicalAndDottedSettled[0].status !== 'fulfilled' ||
    logicalAndDottedSettled[0].value !== 1 ||
    logicalAndDottedSettled[1].status !== 'rejected' ||
    logicalAndDottedSettled[1].reason !== 'boom' ||
    logicalOrDottedSettled.length !== 2 ||
    logicalOrDottedSettled[0].status !== 'fulfilled' ||
    logicalOrDottedSettled[0].value !== 1 ||
    logicalOrDottedSettled[1].status !== 'rejected' ||
    logicalOrDottedSettled[1].reason !== 'boom' ||
    wrappedBracketedDotRootFrozenSettled.length !== 2 ||
    wrappedBracketedDotRootFrozenSettled[0].status !== 'fulfilled' ||
    wrappedBracketedDotRootFrozenSettled[0].value !== 1 ||
    wrappedBracketedDotRootFrozenSettled[1].status !== 'rejected' ||
    wrappedBracketedDotRootFrozenSettled[1].reason !== 'boom' ||
    wrappedSingleBracketedDotRootFrozenSettled.length !== 2 ||
    wrappedSingleBracketedDotRootFrozenSettled[0].status !== 'fulfilled' ||
    wrappedSingleBracketedDotRootFrozenSettled[0].value !== 1 ||
    wrappedSingleBracketedDotRootFrozenSettled[1].status !== 'rejected' ||
    wrappedSingleBracketedDotRootFrozenSettled[1].reason !== 'boom' ||
    frozenBracketedSettled.length !== 2 ||
    frozenBracketedSettled[0].status !== 'fulfilled' ||
    frozenBracketedSettled[0].value !== 1 ||
    frozenBracketedSettled[1].status !== 'rejected' ||
    frozenBracketedSettled[1].reason !== 'boom' ||
    parenthesizedFrozenBracketedSettled.length !== 2 ||
    parenthesizedFrozenBracketedSettled[0].status !== 'fulfilled' ||
    parenthesizedFrozenBracketedSettled[0].value !== 1 ||
    parenthesizedFrozenBracketedSettled[1].status !== 'rejected' ||
    parenthesizedFrozenBracketedSettled[1].reason !== 'boom' ||
    mixedBracketedRootFrozenSettled.length !== 2 ||
    mixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    mixedBracketedRootFrozenSettled[0].value !== 1 ||
    mixedBracketedRootFrozenSettled[1].status !== 'rejected' ||
    mixedBracketedRootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedMixedBracketedRootFrozenSettled.length !== 2 ||
    parenthesizedMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedMixedBracketedRootFrozenSettled[0].value !== 1 ||
    parenthesizedMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||
    singleMixedBracketedRootFrozenSettled.length !== 2 ||
    singleMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    singleMixedBracketedRootFrozenSettled[0].value !== 1 ||
    singleMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||
    singleMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||
    fullyBracketedSingleRootFrozenSettled.length !== 2 ||
    fullyBracketedSingleRootFrozenSettled[0].status !== 'fulfilled' ||
    fullyBracketedSingleRootFrozenSettled[0].value !== 1 ||
    fullyBracketedSingleRootFrozenSettled[1].status !== 'rejected' ||
    fullyBracketedSingleRootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedFullyBracketedSingleRootFrozenSettled.length !== 2 ||
    parenthesizedFullyBracketedSingleRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedFullyBracketedSingleRootFrozenSettled[0].value !== 1 ||
    parenthesizedFullyBracketedSingleRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedFullyBracketedSingleRootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedSingleMixedBracketedRootFrozenSettled.length !== 2 ||
    parenthesizedSingleMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedSingleMixedBracketedRootFrozenSettled[0].value !== 1 ||
    parenthesizedSingleMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedSingleMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||
    mixedRootFrozenSettled.length !== 2 ||
    mixedRootFrozenSettled[0].status !== 'fulfilled' ||
    mixedRootFrozenSettled[0].value !== 1 ||
    mixedRootFrozenSettled[1].status !== 'rejected' ||
    mixedRootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedMixedRootFrozenSettled.length !== 2 ||
    parenthesizedMixedRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedMixedRootFrozenSettled[0].value !== 1 ||
    parenthesizedMixedRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedMixedRootFrozenSettled[1].reason !== 'boom' ||
    bracketedRootFrozenSettled.length !== 2 ||
    bracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    bracketedRootFrozenSettled[0].value !== 1 ||
    bracketedRootFrozenSettled[1].status !== 'rejected' ||
    bracketedRootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedBracketedRootFrozenSettled.length !== 2 ||
    parenthesizedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedBracketedRootFrozenSettled[0].value !== 1 ||
    parenthesizedBracketedRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedBracketedRootFrozenSettled[1].reason !== 'boom' ||
    rootFrozenSettled.length !== 2 ||
    rootFrozenSettled[0].status !== 'fulfilled' ||
    rootFrozenSettled[0].value !== 1 ||
    rootFrozenSettled[1].status !== 'rejected' ||
    rootFrozenSettled[1].reason !== 'boom' ||
    parenthesizedRootFrozenSettled.length !== 2 ||
    parenthesizedRootFrozenSettled[0].status !== 'fulfilled' ||
    parenthesizedRootFrozenSettled[0].value !== 1 ||
    parenthesizedRootFrozenSettled[1].status !== 'rejected' ||
    parenthesizedRootFrozenSettled[1].reason !== 'boom'
  ) {
    throw new Error('unexpected Promise.allSettled semantics');
  }
"#
}

/// Canonical browser smoke body for the supported `Promise.race` slice.
pub const fn promise_race_browser_body_source() -> &'static str {
    r#"  const direct = await Promise.race([Promise.resolve(1), Promise.resolve(2)]);
  const mixed = await Promise["race"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleMixed = await Promise['race']([Promise.resolve(1), Promise.resolve(2)]);
  const dotted = await globalThis.Promise.race([Promise.resolve(1), Promise.resolve(2)]);
  const bracketed = await globalThis["Promise"].race([Promise.resolve(1), Promise.resolve(2)]);
  const singleBracketed = await globalThis['Promise'].race([Promise.resolve(1), Promise.resolve(2)]);
  const mixedDotted = await globalThis.Promise["race"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleDotted = await globalThis.Promise['race']([Promise.resolve(1), Promise.resolve(2)]);
  const bracketedBracketed = await globalThis["Promise"]["race"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleBracketedBracketed = await globalThis['Promise']['race']([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedBracketed = await Object.freeze((globalThis["Promise"])["race"])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedSingleBracketed = await Object.freeze((globalThis['Promise'])['race'])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedDottedBracketed = await Object.freeze((globalThis.Promise)["race"])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedSingleDottedBracketed = await Object.freeze((globalThis.Promise)['race'])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedBracketedBracketed = await Object.freeze((globalThis["Promise"]["race"]))([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedSingleBracketedBracketed = await Object.freeze((globalThis['Promise']['race']))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenRoot = await Object.freeze(Promise.race)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenRoot = await Object.freeze((Promise.race))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenBracketed = await Object.freeze(globalThis["Promise"].race)([Promise.resolve(1), Promise.resolve(2)]);
  const frozenSingleBracketed = await Object.freeze(globalThis['Promise'].race)([Promise.resolve(1), Promise.resolve(2)]);
  const frozenDottedBracketed = await Object.freeze(globalThis.Promise["race"])([Promise.resolve(1), Promise.resolve(2)]);
  const frozenSingleDottedBracketed = await Object.freeze(globalThis.Promise['race'])([Promise.resolve(1), Promise.resolve(2)]);
  const frozenBracketedBracketed = await Object.freeze(globalThis["Promise"]["race"])([Promise.resolve(1), Promise.resolve(2)]);
  const frozenSingleBracketedBracketed = await Object.freeze(globalThis['Promise']['race'])([Promise.resolve(1), Promise.resolve(2)]);
  const frozenDotted = await Object.freeze(globalThis.Promise.race)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenDotted = await Object.freeze((globalThis.Promise.race))([Promise.resolve(1), Promise.resolve(2)]);
  if (
    direct !== 1 ||
    mixed !== 1 ||
    singleMixed !== 1 ||
    dotted !== 1 ||
    bracketed !== 1 ||
    singleBracketed !== 1 ||
    mixedDotted !== 1 ||
    singleDotted !== 1 ||
    bracketedBracketed !== 1 ||
    singleBracketedBracketed !== 1 ||
    parenthesizedBracketed !== 1 ||
    parenthesizedSingleBracketed !== 1 ||
    parenthesizedDottedBracketed !== 1 ||
    parenthesizedSingleDottedBracketed !== 1 ||
    parenthesizedBracketedBracketed !== 1 ||
    parenthesizedSingleBracketedBracketed !== 1 ||
    frozenRoot !== 1 ||
    parenthesizedFrozenRoot !== 1 ||
    frozenBracketed !== 1 ||
    frozenSingleBracketed !== 1 ||
    frozenDottedBracketed !== 1 ||
    frozenSingleDottedBracketed !== 1 ||
    frozenBracketedBracketed !== 1 ||
    frozenSingleBracketedBracketed !== 1 ||
    frozenDotted !== 1 ||
    parenthesizedFrozenDotted !== 1
  ) {
    throw new Error('unexpected Promise.race semantics');
  }
"#
}

/// Canonical browser smoke body for the supported `Promise.any` slice.
pub const fn promise_any_browser_body_source() -> &'static str {
    r#"  const direct = await Promise.any([Promise.reject('boom'), Promise.resolve(1)]);
  const mixed = await Promise["any"]([Promise.reject('boom'), Promise.resolve(1)]);
  const singleMixed = await Promise['any']([Promise.reject('boom'), Promise.resolve(1)]);
  const dotted = await globalThis.Promise.any([Promise.reject('boom'), Promise.resolve(1)]);
  const mixedDotted = await globalThis.Promise["any"]([Promise.reject('boom'), Promise.resolve(1)]);
  const singleDotted = await globalThis.Promise['any']([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedDottedBracketed = await Object.freeze((globalThis.Promise)["any"])([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedSingleDottedBracketed = await Object.freeze((globalThis.Promise)['any'])([Promise.reject('boom'), Promise.resolve(1)]);
  const bracketed = await globalThis["Promise"].any([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedBracketed = await Object.freeze((globalThis["Promise"])["any"])([Promise.reject('boom'), Promise.resolve(1)]);
  const mixedBracketed = await globalThis["Promise"]["any"]([Promise.reject('boom'), Promise.resolve(1)]);
  const singleBracketed = await globalThis['Promise']['any']([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedSingleBracketed = await Object.freeze((globalThis['Promise'])['any'])([Promise.reject('boom'), Promise.resolve(1)]);
  const singleMixedBracketed = await globalThis['Promise'].any([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenBracketed = await Object.freeze(globalThis["Promise"].any)([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenSingleBracketed = await Object.freeze(globalThis['Promise'].any)([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenMixedBracketed = await Object.freeze(globalThis["Promise"]["any"])([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenSingleBracketRoot = await Object.freeze(globalThis['Promise']['any'])([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenSingleMixedBracketed = await Object.freeze(globalThis["Promise"]['any'])([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenMixedBracketed = await Object.freeze((globalThis["Promise"]["any"]))([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenSingleBracketRoot = await Object.freeze((globalThis['Promise']['any']))([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenSingleMixedBracketed = await Object.freeze((globalThis["Promise"]['any']))([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenReceiverWrappedDotted = await Object.freeze((globalThis["Promise"]).any)([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenSingleReceiverWrappedDotted = await Object.freeze((globalThis['Promise']).any)([Promise.reject('boom'), Promise.resolve(1)]);
  const nullishRoot = await Object.freeze((null ?? Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);
  const logicalAndRoot = await Object.freeze((true && Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);
  const logicalOrRoot = await Object.freeze((false || Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenRoot = await Object.freeze(Promise.any)([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenRoot = await Object.freeze((Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenDotted = await Object.freeze(globalThis.Promise.any)([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenDottedBracketed = await Object.freeze(globalThis.Promise["any"])([Promise.reject('boom'), Promise.resolve(1)]);
  const frozenSingleDottedBracketed = await Object.freeze(globalThis.Promise['any'])([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenDotted = await Object.freeze((globalThis.Promise.any))([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenDottedBracketed = await Object.freeze((globalThis.Promise)["any"])([Promise.reject('boom'), Promise.resolve(1)]);
  const parenthesizedFrozenSingleDottedBracketed = await Object.freeze((globalThis.Promise)['any'])([Promise.reject('boom'), Promise.resolve(1)]);
  if (
    direct !== 1 ||
    mixed !== 1 ||
    singleMixed !== 1 ||
    dotted !== 1 ||
    mixedDotted !== 1 ||
    singleDotted !== 1 ||
    bracketed !== 1 ||
    parenthesizedBracketed !== 1 ||
    mixedBracketed !== 1 ||
    singleBracketed !== 1 ||
    parenthesizedSingleBracketed !== 1 ||
    singleMixedBracketed !== 1 ||
    frozenBracketed !== 1 ||
    frozenSingleBracketed !== 1 ||
    parenthesizedFrozenMixedBracketed !== 1 ||
    parenthesizedFrozenSingleBracketRoot !== 1 ||
    parenthesizedFrozenReceiverWrappedDotted !== 1 ||
    parenthesizedFrozenSingleReceiverWrappedDotted !== 1 ||
    frozenMixedBracketed !== 1 ||
    frozenSingleBracketRoot !== 1 ||
    nullishRoot !== 1 ||
    logicalAndRoot !== 1 ||
    logicalOrRoot !== 1 ||
    frozenRoot !== 1 ||
    parenthesizedFrozenRoot !== 1 ||
    frozenDotted !== 1 ||
    parenthesizedDottedBracketed !== 1 ||
    parenthesizedSingleDottedBracketed !== 1 ||
    parenthesizedFrozenDotted !== 1
  ) {
    throw new Error('unexpected Promise.any semantics');
  }
"#
}

/// Canonical browser smoke body for the supported `Promise.all` slice.
pub const fn promise_all_browser_body_source() -> &'static str {
    r#"  const direct = await Promise.all([Promise.resolve(1), Promise.resolve(2)]);
  const mixed = await Promise["all"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleMixed = await Promise['all']([Promise.resolve(1), Promise.resolve(2)]);
  const dotted = await globalThis.Promise.all([Promise.resolve(1), Promise.resolve(2)]);
  const mixedDotted = await globalThis.Promise["all"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleDotted = await globalThis.Promise['all']([Promise.resolve(1), Promise.resolve(2)]);
  const bracketed = await globalThis["Promise"].all([Promise.resolve(1), Promise.resolve(2)]);
  const mixedBracketed = await globalThis["Promise"]["all"]([Promise.resolve(1), Promise.resolve(2)]);
  const singleBracketed = await globalThis['Promise']['all']([Promise.resolve(1), Promise.resolve(2)]);
  const singleMixedBracketed = await globalThis['Promise'].all([Promise.resolve(1), Promise.resolve(2)]);
  const nullishRoot = await Object.freeze((null ?? Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const logicalAndRoot = await Object.freeze((true && Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const logicalOrRoot = await Object.freeze((false || Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const nullishDotted = await Object.freeze((null ?? globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const logicalAndDotted = await Object.freeze((true && globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const logicalOrDotted = await Object.freeze((false || globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenRoot = await Object.freeze(Promise.all)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenRoot = await Object.freeze((Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenBracketedRoot = await Object.freeze(Promise["all"])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenBracketedRoot = await Object.freeze((Promise["all"]))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenSingleBracketedRoot = await Object.freeze(Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenSingleBracketedRoot = await Object.freeze((Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);
  const mixedRoot = await Object.freeze(globalThis.Promise["all"])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedMixedRoot = await Object.freeze((globalThis.Promise["all"]))([Promise.resolve(1), Promise.resolve(2)]);
  const singleMixedRoot = await Object.freeze(globalThis.Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedSingleMixedRoot = await Object.freeze((globalThis.Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);
  const bracketedRoot = await Object.freeze(globalThis["Promise"].all)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedBracketedRoot = await Object.freeze((globalThis["Promise"].all))([Promise.resolve(1), Promise.resolve(2)]);
  const mixedBracketedRoot = await Object.freeze(globalThis["Promise"]["all"])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedMixedBracketedRoot = await Object.freeze((globalThis["Promise"]["all"]))([Promise.resolve(1), Promise.resolve(2)]);
  const singleMixedBracketedRoot = await Object.freeze(globalThis['Promise'].all)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedSingleMixedBracketedRoot = await Object.freeze((globalThis['Promise'].all))([Promise.resolve(1), Promise.resolve(2)]);
  const fullyBracketedSingleRoot = await Object.freeze(globalThis['Promise']['all'])([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFullyBracketedSingleRoot = await Object.freeze((globalThis['Promise']['all']))([Promise.resolve(1), Promise.resolve(2)]);
  const frozenGlobal = await Object.freeze(globalThis.Promise.all)([Promise.resolve(1), Promise.resolve(2)]);
  const parenthesizedFrozenGlobal = await Object.freeze((globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);
  if (
    direct.length !== 2 ||
    direct[0] !== 1 ||
    direct[1] !== 2 ||
    mixed.length !== 2 ||
    mixed[0] !== 1 ||
    mixed[1] !== 2 ||
    singleMixed.length !== 2 ||
    singleMixed[0] !== 1 ||
    singleMixed[1] !== 2 ||
    dotted.length !== 2 ||
    dotted[0] !== 1 ||
    dotted[1] !== 2 ||
    mixedDotted.length !== 2 ||
    mixedDotted[0] !== 1 ||
    mixedDotted[1] !== 2 ||
    singleDotted.length !== 2 ||
    singleDotted[0] !== 1 ||
    singleDotted[1] !== 2 ||
    bracketed.length !== 2 ||
    bracketed[0] !== 1 ||
    bracketed[1] !== 2 ||
    mixedBracketed.length !== 2 ||
    mixedBracketed[0] !== 1 ||
    mixedBracketed[1] !== 2 ||
    singleBracketed.length !== 2 ||
    singleBracketed[0] !== 1 ||
    singleBracketed[1] !== 2 ||
    singleMixedBracketed.length !== 2 ||
    singleMixedBracketed[0] !== 1 ||
    singleMixedBracketed[1] !== 2 ||
    nullishRoot.length !== 2 ||
    nullishRoot[0] !== 1 ||
    nullishRoot[1] !== 2 ||
    logicalAndRoot.length !== 2 ||
    logicalAndRoot[0] !== 1 ||
    logicalAndRoot[1] !== 2 ||
    logicalOrRoot.length !== 2 ||
    logicalOrRoot[0] !== 1 ||
    logicalOrRoot[1] !== 2 ||
    nullishDotted.length !== 2 ||
    nullishDotted[0] !== 1 ||
    nullishDotted[1] !== 2 ||
    logicalAndDotted.length !== 2 ||
    logicalAndDotted[0] !== 1 ||
    logicalAndDotted[1] !== 2 ||
    logicalOrDotted.length !== 2 ||
    logicalOrDotted[0] !== 1 ||
    logicalOrDotted[1] !== 2 ||
    frozenRoot.length !== 2 ||
    frozenRoot[0] !== 1 ||
    frozenRoot[1] !== 2 ||
    parenthesizedFrozenRoot.length !== 2 ||
    parenthesizedFrozenRoot[0] !== 1 ||
    parenthesizedFrozenRoot[1] !== 2 ||
    frozenBracketedRoot.length !== 2 ||
    frozenBracketedRoot[0] !== 1 ||
    frozenBracketedRoot[1] !== 2 ||
    parenthesizedFrozenBracketedRoot.length !== 2 ||
    parenthesizedFrozenBracketedRoot[0] !== 1 ||
    parenthesizedFrozenBracketedRoot[1] !== 2 ||
    frozenSingleBracketedRoot.length !== 2 ||
    frozenSingleBracketedRoot[0] !== 1 ||
    frozenSingleBracketedRoot[1] !== 2 ||
    parenthesizedFrozenSingleBracketedRoot.length !== 2 ||
    parenthesizedFrozenSingleBracketedRoot[0] !== 1 ||
    parenthesizedFrozenSingleBracketedRoot[1] !== 2 ||
    frozenGlobal.length !== 2 ||
    frozenGlobal[0] !== 1 ||
    frozenGlobal[1] !== 2 ||
    parenthesizedFrozenGlobal.length !== 2 ||
    parenthesizedFrozenGlobal[0] !== 1 ||
    parenthesizedFrozenGlobal[1] !== 2
  ) {
    throw new Error("unexpected Promise.all results");
  }
"#
}

#[cfg(test)]
#[path = "promise_tests.rs"]
mod promise_tests;
