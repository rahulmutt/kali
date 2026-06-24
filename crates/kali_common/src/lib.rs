//! Common utilities shared across all Kali crates.
//!
//! This crate provides:
//! - String interning for identifiers and literals
//! - Source file registry with compact FileId
//! - Span type for source positions
//! - SourceMap for human-readable diagnostics

pub mod interner;
pub mod source_map;
pub mod span;
pub mod template;
mod helpers;

pub use interner::{InternedString, Interner};
pub use span::Span;
pub(crate) use helpers::*;
mod registry;
pub use registry::*;
mod messages;
pub use messages::*;
mod process_kill;
pub use process_kill::*;
mod object;
pub use object::*;
mod number;
pub use number::*;
mod math;
pub use math::*;

/// Canonical source text for the supported late-compat `Object.hasOwn` / `Object.prototype.hasOwnProperty.call` slice.
pub fn late_compat_object_has_own_source(receiver_source: &str, key_source: &str) -> String {
    let source = [
        format!("Object.hasOwn({receiver_source}, {key_source})"),
        format!("globalThis.Object.hasOwn({receiver_source}, {key_source})"),
        format!(r#"globalThis.Object["hasOwn"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].hasOwn({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["hasOwn"]({receiver_source}, {key_source})"#),
        format!(r#"Object["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"Object["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object.prototype.hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"Object.prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"Object["prototype"].hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object.prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object['hasOwnProperty'].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]['hasOwnProperty'].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']['hasOwnProperty'].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype.hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype['hasOwnProperty']['call']({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype['hasOwnProperty'].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']['prototype']['hasOwnProperty']['call']({receiver_source}, {key_source})"#),
        format!("Object.prototype.hasOwnProperty.call({receiver_source}, {key_source})"),
        format!("globalThis.Object.prototype.hasOwnProperty.call({receiver_source}, {key_source})"),
        format!(r#"globalThis.Object.prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["prototype"].hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object.prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].prototype.hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
    ]
    .join("; ");
    format!("{source};")
}

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

/// Canonical late-process-control source prefix that groups the supported
/// Deno and process-control aliases before the shared zero-probe inventory.
const LATE_PROCESS_CONTROL_PREFIX_SEGMENTS: &[&str] = &[
    "Deno.pid",
    "globalThis.Deno.pid",
    "globalThis[\"Deno\"][\"pid\"]",
    "globalThis[\"Deno\"].cwd",
    "globalThis[\"Deno\"].chdir",
    "globalThis[\"Deno\"].exit",
    "Deno[\"pid\"]",
    "globalThis.Deno[\"pid\"]",
    "globalThis.Deno.cwd",
    "globalThis[\"Deno\"][\"cwd\"]",
    "globalThis.Deno[\"cwd\"]",
    "Deno[\"cwd\"]",
    "Deno.chdir",
    "globalThis.Deno.chdir",
    "globalThis[\"Deno\"][\"chdir\"]",
    "globalThis.Deno[\"chdir\"]",
    "Deno[\"chdir\"]",
    "globalThis.Deno.exit",
    "globalThis[\"Deno\"][\"exit\"]",
    "globalThis.Deno[\"exit\"]",
    "Deno[\"exit\"]",
    "process.pid",
    "globalThis.process.pid",
    "globalThis[\"process\"][\"pid\"]",
    "globalThis[\"process\"].pid",
    "process[\"pid\"]",
    "globalThis.process[\"pid\"]",
    "process.cwd",
    "globalThis.process.cwd",
    "globalThis[\"process\"].cwd",
    "globalThis[\"process\"][\"cwd\"]",
    "process[\"cwd\"]",
    "globalThis.process[\"cwd\"]",
    "process.chdir",
    "globalThis.process.chdir",
    "globalThis[\"process\"].chdir",
    "globalThis[\"process\"][\"chdir\"]",
    "process[\"chdir\"]",
    "globalThis.process[\"chdir\"]",
    "process.kill",
    "globalThis.process.kill",
    "globalThis[\"process\"].kill",
    "globalThis[\"process\"][\"kill\"]",
    "process[\"kill\"]",
    "globalThis.process[\"kill\"]",
    "const zero = 0",
    "const zeroAlias = zero",
    "process.kill(zeroAlias)",
];

const LATE_PROCESS_CONTROL_EXIT_SEGMENTS: &[&str] = &[
    "process.exit",
    "globalThis.process.exit",
    "globalThis.process[\"exit\"]",
    "globalThis[\"process\"].exit",
    "globalThis[\"process\"][\"exit\"]",
    "process[\"exit\"]",
];

/// Canonical exit alias inventory for the shared late-process-control slice.
pub const fn late_process_control_exit_aliases() -> &'static [&'static str] {
    LATE_PROCESS_CONTROL_EXIT_SEGMENTS
}

/// Canonical late-process-control exit source text, shared across the
/// browser and runtime late-compat smoke.
pub fn late_process_control_exit_source() -> String {
    join_semicolon_terminated_segments(late_process_control_exit_aliases())
}

/// Canonical late-process-control preamble source text, shared across the
/// browser and runtime late-compat smoke.
pub fn late_process_control_prefix_source() -> String {
    format!(
        "{}; {}",
        join_semicolon_terminated_segments(LATE_PROCESS_CONTROL_PREFIX_SEGMENTS)
            .trim_end_matches(';'),
        late_process_control_exit_source().trim_end()
    )
}

/// Canonical late-process-control source text that embeds the supported Node zero-probe slice.
pub fn late_process_control_source() -> String {
    let process_kill_zero_probe_source = process_kill_zero_probe_alias_inventory_source();
    let parenthesized_receiver_source = process_kill_zero_probe_parenthesized_receiver_source();
    let parenthesized_receiver_freeze_source =
        process_kill_zero_probe_parenthesized_receiver_freeze_source();
    format!(
        "{} {} {} {}",
        late_process_control_prefix_source(),
        parenthesized_receiver_source.trim_end(),
        parenthesized_receiver_freeze_source.trim_end(),
        process_kill_zero_probe_source.trim_end()
    )
}

const LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS: &[&str] = &[
    r#"globalThis['process'].kill(0)"#,
    r#"globalThis['process'].kill(+0)"#,
    r#"globalThis['process']['kill'](0)"#,
    r#"globalThis['process']['kill'](+0)"#,
    r#"process['kill'](0)"#,
    r#"process['kill'](+0)"#,
    r#"process['kill']((0))"#,
    r#"globalThis.process['kill'](0)"#,
    r#"globalThis.process['kill'](+0)"#,
    r#"globalThis.process['kill']((0))"#,
    r#"globalThis['process'].kill((0))"#,
    r#"globalThis['process']['kill']((0))"#,
    r#"globalThis.process['kill']((0))"#,
    r#"Object.freeze(process['kill'])(0)"#,
    r#"Object.freeze(process['kill'])(+0)"#,
    r#"Object.freeze((process['kill']))(0)"#,
    r#"Object.freeze((process['kill']))(+0)"#,
    r#"Object.freeze(globalThis.process['kill'])(0)"#,
    r#"Object.freeze(globalThis.process['kill'])(+0)"#,
    r#"Object.freeze((globalThis.process['kill']))(0)"#,
    r#"Object.freeze((globalThis.process['kill']))(+0)"#,
    r#"Object.freeze(globalThis['process'].kill)(0)"#,
    r#"Object.freeze(globalThis['process'].kill)(+0)"#,
    r#"Object.freeze((globalThis['process']).kill)(0)"#,
    r#"Object.freeze((globalThis['process']).kill)(+0)"#,
    r#"Object.freeze((globalThis['process'])['kill'])(0)"#,
    r#"Object.freeze((globalThis['process'])['kill'])(+0)"#,
    r#"Object.freeze((globalThis['process'].kill))(0)"#,
    r#"Object.freeze((globalThis['process'].kill))(+0)"#,
    r#"Object.freeze((globalThis['process']['kill']))(0)"#,
    r#"Object.freeze((globalThis['process']['kill']))(+0)"#,
    r#"Object.freeze(globalThis['process']['kill'])(0)"#,
    r#"Object.freeze(globalThis['process']['kill'])(+0)"#,
    r#"process['exit'](0)"#,
    r#"process['exit'](+0)"#,
    r#"process['exit']((0))"#,
    r#"Object.freeze(process['exit'])(0)"#,
    r#"Object.freeze(process['exit'])(+0)"#,
    r#"Object.freeze((process['exit']))(0)"#,
    r#"Object.freeze((process['exit']))(+0)"#,
    r#"Object.freeze((process)['exit'])(0)"#,
    r#"Object.freeze((process)['exit'])(+0)"#,
    r#"Object.freeze((globalThis.process)['exit'])(0)"#,
    r#"Object.freeze((globalThis.process)['exit'])(+0)"#,
    r#"Object.freeze((globalThis['process'])['exit'])(0)"#,
    r#"Object.freeze((globalThis['process'])['exit'])(+0)"#,
    r#"globalThis['process'].exit(0)"#,
    r#"globalThis['process'].exit(+0)"#,
    r#"globalThis['process'].exit((0))"#,
    r#"globalThis['process']['exit'](0)"#,
    r#"globalThis['process']['exit'](+0)"#,
    r#"globalThis['process']['exit']((0))"#,
    r#"globalThis.process['exit'](0)"#,
    r#"globalThis.process['exit'](+0)"#,
    r#"globalThis.process['exit']((0))"#,
    r#"Object.freeze(globalThis['process'].exit)(0)"#,
    r#"Object.freeze(globalThis['process'].exit)(+0)"#,
    r#"Object.freeze((globalThis['process'].exit))(0)"#,
    r#"Object.freeze((globalThis['process'].exit))(+0)"#,
    r#"Object.freeze(globalThis['process']['exit'])(0)"#,
    r#"Object.freeze(globalThis['process']['exit'])(+0)"#,
    r#"Object.freeze((globalThis['process']['exit']))(0)"#,
    r#"Object.freeze((globalThis['process']['exit']))(+0)"#,
];

/// Canonical late-process-control aliases for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_kill_aliases() -> &'static [&'static str] {
    &LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS[..33]
}

/// Canonical late-process-control source text for the browser JS single-quoted kill aliases.
pub fn late_process_control_single_quoted_kill_aliases_source() -> String {
    join_semicolon_terminated_segments(late_process_control_single_quoted_kill_aliases())
}

/// Canonical late-process-control aliases for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_exit_aliases() -> &'static [&'static str] {
    &LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS[33..]
}

/// Canonical late-process-control source text for the browser JS single-quoted exit aliases.
pub fn late_process_control_single_quoted_exit_source() -> String {
    join_semicolon_terminated_segments(late_process_control_single_quoted_exit_aliases())
}

/// Canonical late-process-control source text for the browser JS single-quoted exit aliases.
pub fn late_process_control_single_quoted_exit_aliases_source() -> String {
    late_process_control_single_quoted_exit_source()
}

/// Canonical late-process-control source text for the browser JS single-quoted kill aliases.
pub fn late_process_control_single_quoted_kill_source() -> String {
    late_process_control_single_quoted_kill_aliases_source()
}

/// Canonical late-process-control aliases for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_process_aliases() -> &'static [&'static str] {
    LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS
}

/// Canonical late-process-control source text for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_process_aliases_source() -> String {
    format!(
        "{} {}",
        late_process_control_single_quoted_kill_source().trim_end(),
        late_process_control_single_quoted_exit_source().trim_end()
    )
}

/// Canonical late-process-control source text for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_process_source() -> String {
    format!(
        "{} {}",
        late_process_control_source(),
        late_process_control_single_quoted_process_aliases_source().trim_end()
    )
}

const LATE_PROCESS_ENV_MUTATION_SEGMENTS: &[&str] = &[
    r#"process.env = {}"#,
    r#"process.env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis.process.env = {}"#,
    r#"globalThis.process.env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"process["env"] = {}"#,
    r#"process["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"process["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"process['env'] = {}"#,
    r#"process['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"process['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"globalThis.process["env"] = {}"#,
    r#"globalThis.process["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis.process['env'] = {}"#,
    r#"globalThis.process['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis.process['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis.process['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"globalThis["process"].env = {}"#,
    r#"globalThis["process"].env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis["process"]["env"] = {}"#,
    r#"globalThis["process"]["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis["process"]['env'] = {}"#,
    r#"globalThis["process"]['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis["process"]['env']["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis["process"]['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"globalThis['process']["env"] = {}"#,
    r#"globalThis['process']["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"delete globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"]['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"globalThis['process'].env = {}"#,
    r#"globalThis['process'].env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis['process'].env['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"globalThis['process']['env'] = {}"#,
    r#"globalThis['process']['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis['process']['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"delete process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete process['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"delete process.env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis.process['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"delete globalThis.process.env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis['process'].env['KALI_BROWSER_ENV_MUTATION']"#,
    r#"delete globalThis['process']['env']['KALI_BROWSER_ENV_MUTATION']"#,
];

/// Canonical late-process-environment-mutation alias inventory used by the browser and runtime smoke.
pub fn late_process_env_mutation_aliases() -> &'static [&'static str] {
    LATE_PROCESS_ENV_MUTATION_SEGMENTS
}

/// Canonical late-process-environment-mutation source text used by the browser and runtime smoke.
pub fn late_process_env_mutation_source() -> String {
    join_semicolon_terminated_segments(late_process_env_mutation_aliases())
}

const BROADER_INTL_SEGMENTS: &[&str] = &[
    "Intl",
    "globalThis.Intl",
    r#"globalThis["Intl"]"#,
    "globalThis['Intl']",
    "globalThis.Intl.NumberFormat",
    r#"globalThis["Intl"].NumberFormat"#,
    r#"globalThis.Intl["NumberFormat"]"#,
    r#"globalThis['Intl'].NumberFormat"#,
    r#"globalThis['Intl']["NumberFormat"]"#,
    "globalThis.Intl.DateTimeFormat",
    r#"globalThis["Intl"].DateTimeFormat"#,
    r#"globalThis.Intl["DateTimeFormat"]"#,
    r#"globalThis['Intl'].DateTimeFormat"#,
    r#"globalThis['Intl']["DateTimeFormat"]"#,
    r#"globalThis["Intl"]["DateTimeFormat"]"#,
    "globalThis.Intl.PluralRules",
    r#"globalThis["Intl"].PluralRules"#,
    r#"globalThis.Intl["PluralRules"]"#,
    r#"globalThis['Intl'].PluralRules"#,
    r#"globalThis['Intl']["PluralRules"]"#,
    "globalThis.Intl.RelativeTimeFormat",
    r#"globalThis["Intl"].RelativeTimeFormat"#,
    r#"globalThis.Intl["RelativeTimeFormat"]"#,
    r#"globalThis['Intl'].RelativeTimeFormat"#,
    r#"globalThis['Intl']["RelativeTimeFormat"]"#,
    "globalThis.Intl.Collator",
    r#"globalThis["Intl"].Collator"#,
    r#"globalThis.Intl["Collator"]"#,
    r#"globalThis['Intl'].Collator"#,
    r#"globalThis['Intl']["Collator"]"#,
    "globalThis.Intl.DisplayNames",
    r#"globalThis["Intl"].DisplayNames"#,
    r#"globalThis.Intl["DisplayNames"]"#,
    r#"globalThis['Intl'].DisplayNames"#,
    r#"globalThis['Intl']["DisplayNames"]"#,
    "globalThis.Intl.Segmenter",
    r#"globalThis["Intl"].Segmenter"#,
    r#"globalThis.Intl["Segmenter"]"#,
    r#"globalThis['Intl'].Segmenter"#,
    r#"globalThis['Intl']["Segmenter"]"#,
    "globalThis.Intl.Locale",
    r#"globalThis["Intl"].Locale"#,
    r#"globalThis.Intl["Locale"]"#,
    r#"globalThis['Intl'].Locale"#,
    r#"globalThis['Intl']["Locale"]"#,
    "globalThis['Intl']['Segmenter']",
    "globalThis['Intl']['NumberFormat']",
    "globalThis['Intl']['DateTimeFormat']",
    "globalThis['Intl']['PluralRules']",
    "globalThis['Intl']['RelativeTimeFormat']",
    "globalThis['Intl']['Collator']",
    "globalThis['Intl']['DisplayNames']",
    "globalThis['Intl']['Locale']",
    r#"globalThis["Intl"]["NumberFormat"]"#,
    r#"globalThis["Intl"]["PluralRules"]"#,
    r#"globalThis["Intl"]["RelativeTimeFormat"]"#,
    r#"globalThis["Intl"]["Collator"]"#,
    r#"globalThis["Intl"]["DisplayNames"]"#,
    r#"globalThis["Intl"]["Segmenter"]"#,
    r#"globalThis["Intl"]["Locale"]"#,
    "Intl.NumberFormat",
    "Intl.DateTimeFormat",
    "Intl.PluralRules",
    "Intl.RelativeTimeFormat",
    "Intl.Collator",
    "Intl.DisplayNames",
    "Intl.Locale",
];

/// Canonical broader `Intl` aliases used by the browser and runtime smoke.
pub fn broader_intl_aliases() -> &'static [&'static str] {
    BROADER_INTL_SEGMENTS
}

/// Canonical broader `Intl` source text used by the browser and runtime smoke.
pub fn broader_intl_source() -> String {
    join_semicolon_terminated_segments(broader_intl_aliases())
}

const LATE_OBJECT_MODEL_SEGMENTS: &[&str] = &[
    "Proxy",
    "globalThis.Proxy",
    r#"globalThis["Proxy"]"#,
    "globalThis['Proxy']",
    "new Proxy({}, {})",
    "new globalThis.Proxy({}, {})",
    r#"new globalThis["Proxy"]({}, {})"#,
    "new globalThis['Proxy']({}, {})",
    "new WeakMap()",
    "globalThis.WeakMap",
    r#"globalThis["WeakMap"]"#,
    r#"globalThis['WeakMap']"#,
    r#"globalThis["WeakMap"]()"#,
    r#"globalThis['WeakMap']()"#,
    "Object.freeze(new WeakMap())",
    "Object.freeze((new WeakMap()))",
    "Object.freeze(globalThis.WeakMap)",
    "Object.freeze((globalThis.WeakMap))",
    r#"Object.freeze(globalThis["WeakMap"])"#,
    r#"Object.freeze((globalThis["WeakMap"]))"#,
    r#"Object.freeze(globalThis['WeakMap'])"#,
    r#"Object.freeze((globalThis['WeakMap']))"#,
    "new WeakSet()",
    "globalThis.WeakSet",
    r#"globalThis["WeakSet"]"#,
    r#"globalThis['WeakSet']"#,
    r#"globalThis["WeakSet"]()"#,
    r#"globalThis['WeakSet']()"#,
    "Object.freeze(new WeakSet())",
    "Object.freeze((new WeakSet()))",
    "Object.freeze(globalThis.WeakSet)",
    "Object.freeze((globalThis.WeakSet))",
    r#"Object.freeze(globalThis["WeakSet"])"#,
    r#"Object.freeze((globalThis["WeakSet"]))"#,
    r#"Object.freeze(globalThis['WeakSet'])"#,
    r#"Object.freeze((globalThis['WeakSet']))"#,
    "globalThis.WeakRef",
    r#"globalThis["WeakRef"]"#,
    "globalThis['WeakRef']",
    "Object.freeze(globalThis.WeakRef)",
    "Object.freeze((globalThis.WeakRef))",
    r#"Object.freeze(globalThis["WeakRef"])"#,
    r#"Object.freeze((globalThis["WeakRef"]))"#,
    "Object.freeze(globalThis['WeakRef'])",
    "Object.freeze((globalThis['WeakRef']))",
    "new FinalizationRegistry(() => {})",
    "globalThis.FinalizationRegistry",
    r#"globalThis["FinalizationRegistry"](() => {})"#,
    r#"globalThis['FinalizationRegistry'](() => {})"#,
    "Object.freeze(new FinalizationRegistry(() => {}))",
    "Object.freeze((new FinalizationRegistry(() => {})))",
    "Object.freeze(globalThis.FinalizationRegistry)",
    "Object.freeze((globalThis.FinalizationRegistry))",
    r#"Object.freeze(globalThis["FinalizationRegistry"](() => {}))"#,
    r#"Object.freeze((globalThis["FinalizationRegistry"](() => {})))"#,
    r#"Object.freeze(globalThis['FinalizationRegistry'](() => {}))"#,
    r#"Object.freeze((globalThis['FinalizationRegistry'](() => {})))"#,
    r#"Object.freeze(globalThis["FinalizationRegistry"])"#,
    r#"Object.freeze((globalThis["FinalizationRegistry"]))"#,
    r#"Object.freeze(globalThis['FinalizationRegistry'])"#,
    r#"Object.freeze((globalThis['FinalizationRegistry']))"#,
    "Proxy.revocable({}, {})",
    "globalThis.Proxy.revocable({}, {})",
    r#"globalThis["Proxy"]["revocable"]({}, {})"#,
    r#"globalThis['Proxy']['revocable']({}, {})"#,
    r#"globalThis["Proxy"].revocable({}, {})"#,
    r#"globalThis['Proxy'].revocable({}, {})"#,
    r#"globalThis.Proxy["revocable"]({}, {})"#,
    r#"globalThis.Proxy['revocable']({}, {})"#,
    r#"globalThis['Proxy']["revocable"]({}, {})"#,
    r#"globalThis["Proxy"]['revocable']({}, {})"#,
    r#"Object.freeze(globalThis['Proxy']["revocable"])({}, {})"#,
    r#"Object.freeze((globalThis['Proxy']["revocable"]))({}, {})"#,
    r#"Object.freeze((globalThis["Proxy"])["revocable"])({}, {})"#,
    r#"Object.freeze((globalThis['Proxy'])['revocable'])({}, {})"#,
    r#"Object.freeze(globalThis["Proxy"]['revocable'])({}, {})"#,
    r#"Object.freeze((globalThis["Proxy"]['revocable']))({}, {})"#,
    "Object.freeze(Proxy.revocable)({}, {})",
    "Object.freeze((Proxy.revocable))({}, {})",
    "Object.freeze(globalThis.Proxy.revocable)({}, {})",
    "Object.freeze((globalThis.Proxy.revocable))({}, {})",
    r#"Object.freeze(globalThis["Proxy"]["revocable"])({}, {})"#,
    r#"Object.freeze((globalThis["Proxy"]["revocable"]))({}, {})"#,
    r#"Object.freeze(globalThis['Proxy']['revocable'])({}, {})"#,
    r#"Object.freeze((globalThis['Proxy']['revocable']))({}, {})"#,
    r#"Object.freeze(globalThis["Proxy"].revocable)({}, {})"#,
    r#"Object.freeze((globalThis["Proxy"].revocable))({}, {})"#,
    r#"Object.freeze(globalThis['Proxy'].revocable)({}, {})"#,
    r#"Object.freeze((globalThis['Proxy']).revocable)({}, {})"#,
    r#"Object.freeze((globalThis['Proxy'].revocable))({}, {})"#,
    r#"Object.freeze(globalThis.Proxy["revocable"])({}, {})"#,
    r#"Object.freeze((globalThis.Proxy["revocable"]))({}, {})"#,
    r#"Object.freeze(globalThis.Proxy['revocable'])({}, {})"#,
    r#"Object.freeze((globalThis.Proxy['revocable']))({}, {})"#,
];

/// Canonical alias inventory for the shared late-object-model slice.
pub const fn late_object_model_aliases() -> &'static [&'static str] {
    LATE_OBJECT_MODEL_SEGMENTS
}

/// Canonical late-object-model source text used by the browser and runtime smoke.
pub const fn late_object_model_source() -> &'static str {
    r#"Proxy; globalThis.Proxy; globalThis["Proxy"]; globalThis['Proxy']; new Proxy({}, {}); new globalThis.Proxy({}, {}); new globalThis["Proxy"]({}, {}); new globalThis['Proxy']({}, {}); new WeakMap(); globalThis.WeakMap; globalThis["WeakMap"]; globalThis['WeakMap']; globalThis["WeakMap"](); globalThis['WeakMap'](); Object.freeze(new WeakMap()); Object.freeze((new WeakMap())); Object.freeze(globalThis.WeakMap); Object.freeze((globalThis.WeakMap)); Object.freeze(globalThis["WeakMap"]); Object.freeze((globalThis["WeakMap"])); Object.freeze(globalThis['WeakMap']); Object.freeze((globalThis['WeakMap'])); new WeakSet(); globalThis.WeakSet; globalThis["WeakSet"]; globalThis['WeakSet']; globalThis["WeakSet"](); globalThis['WeakSet'](); Object.freeze(new WeakSet()); Object.freeze((new WeakSet())); Object.freeze(globalThis.WeakSet); Object.freeze((globalThis.WeakSet)); Object.freeze(globalThis["WeakSet"]); Object.freeze((globalThis["WeakSet"])); Object.freeze(globalThis['WeakSet']); Object.freeze((globalThis['WeakSet'])); globalThis.WeakRef; globalThis["WeakRef"]; globalThis['WeakRef']; Object.freeze(globalThis.WeakRef); Object.freeze((globalThis.WeakRef)); Object.freeze(globalThis["WeakRef"]); Object.freeze((globalThis["WeakRef"])); Object.freeze(globalThis['WeakRef']); Object.freeze((globalThis['WeakRef'])); new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis["FinalizationRegistry"](() => {}); globalThis['FinalizationRegistry'](() => {}); Object.freeze(new FinalizationRegistry(() => {})); Object.freeze((new FinalizationRegistry(() => {}))); Object.freeze(globalThis.FinalizationRegistry); Object.freeze((globalThis.FinalizationRegistry)); Object.freeze(globalThis["FinalizationRegistry"](() => {})); Object.freeze((globalThis["FinalizationRegistry"](() => {}))); Object.freeze(globalThis['FinalizationRegistry'](() => {})); Object.freeze((globalThis['FinalizationRegistry'](() => {}))); Object.freeze(globalThis["FinalizationRegistry"]); Object.freeze((globalThis["FinalizationRegistry"])); Object.freeze(globalThis['FinalizationRegistry']); Object.freeze((globalThis['FinalizationRegistry'])); Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis["Proxy"]["revocable"]({}, {}); globalThis['Proxy']['revocable']({}, {}); globalThis["Proxy"].revocable({}, {}); globalThis['Proxy'].revocable({}, {}); globalThis.Proxy["revocable"]({}, {}); globalThis.Proxy['revocable']({}, {}); globalThis['Proxy']["revocable"]({}, {}); globalThis["Proxy"]['revocable']({}, {}); Object.freeze(globalThis['Proxy']["revocable"])({}, {}); Object.freeze((globalThis['Proxy']["revocable"]))({}, {}); Object.freeze((globalThis["Proxy"])["revocable"])({}, {}); Object.freeze((globalThis['Proxy'])['revocable'])({}, {}); Object.freeze(globalThis["Proxy"]['revocable'])({}, {}); Object.freeze((globalThis["Proxy"]['revocable']))({}, {}); Object.freeze(Proxy.revocable)({}, {}); Object.freeze((Proxy.revocable))({}, {}); Object.freeze(globalThis.Proxy.revocable)({}, {}); Object.freeze((globalThis.Proxy.revocable))({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze((globalThis["Proxy"]["revocable"]))({}, {}); Object.freeze(globalThis['Proxy']['revocable'])({}, {}); Object.freeze((globalThis['Proxy']['revocable']))({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {}); Object.freeze((globalThis["Proxy"].revocable))({}, {}); Object.freeze(globalThis['Proxy'].revocable)({}, {}); Object.freeze((globalThis['Proxy']).revocable)({}, {}); Object.freeze((globalThis['Proxy'].revocable))({}, {}); Object.freeze(globalThis.Proxy["revocable"])({}, {}); Object.freeze((globalThis.Proxy["revocable"]))({}, {}); Object.freeze(globalThis.Proxy['revocable'])({}, {}); Object.freeze((globalThis.Proxy['revocable']))({}, {});"#
}

const LATE_OBJECT_MODEL_OWN_PROPERTY_SEGMENTS: &[&str] = &[
    r#"Object.hasOwn(globalThis, "a")"#,
    r#"globalThis.Object.hasOwn(globalThis, "a")"#,
    r#"globalThis.Object["hasOwn"](globalThis, "a")"#,
    r#"globalThis["Object"].hasOwn(globalThis, "a")"#,
    r#"globalThis["Object"]["hasOwn"](globalThis, "a")"#,
    r#"Object["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis.Object["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis.Object['hasOwnProperty'].call(globalThis, "a")"#,
    r#"globalThis["Object"]["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis["Object"]['hasOwnProperty'].call(globalThis, "a")"#,
    r#"Object.prototype.hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis.Object.prototype.hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis.Object.prototype.hasOwnProperty["call"](globalThis, "a")"#,
    r#"globalThis.Object["prototype"].hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis.Object["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
    r#"globalThis.Object.prototype["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis["Object"].prototype.hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis["Object"].prototype.hasOwnProperty["call"](globalThis, "a")"#,
    r#"globalThis["Object"].prototype['hasOwnProperty']['call'](globalThis, "a")"#,
    r#"globalThis["Object"].prototype['hasOwnProperty'].call(globalThis, "a")"#,
    r#"globalThis["Object"].prototype["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis["Object"]["prototype"].hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
    r#"globalThis["Object"]["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
    r#"globalThis.Object["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
];

/// Canonical alias inventory for the shared late-object-model own-property slice.
pub fn late_object_model_own_property_aliases() -> &'static [&'static str] {
    LATE_OBJECT_MODEL_OWN_PROPERTY_SEGMENTS
}

/// Canonical late-object-model own-property source text used by the browser and runtime smoke.
pub const fn late_object_model_own_property_source() -> &'static str {
    r#"Object.hasOwn(globalThis, "a"); globalThis.Object.hasOwn(globalThis, "a"); globalThis.Object["hasOwn"](globalThis, "a"); globalThis["Object"].hasOwn(globalThis, "a"); globalThis["Object"]["hasOwn"](globalThis, "a"); Object["hasOwnProperty"].call(globalThis, "a"); globalThis.Object["hasOwnProperty"].call(globalThis, "a"); globalThis.Object['hasOwnProperty'].call(globalThis, "a"); globalThis["Object"]["hasOwnProperty"].call(globalThis, "a"); globalThis["Object"]['hasOwnProperty'].call(globalThis, "a"); Object.prototype.hasOwnProperty.call(globalThis, "a"); globalThis.Object.prototype.hasOwnProperty.call(globalThis, "a"); globalThis.Object.prototype.hasOwnProperty["call"](globalThis, "a"); globalThis.Object["prototype"].hasOwnProperty.call(globalThis, "a"); globalThis.Object["prototype"]["hasOwnProperty"]["call"](globalThis, "a"); globalThis.Object.prototype["hasOwnProperty"].call(globalThis, "a"); globalThis["Object"].prototype.hasOwnProperty.call(globalThis, "a"); globalThis["Object"].prototype.hasOwnProperty["call"](globalThis, "a"); globalThis["Object"].prototype['hasOwnProperty']['call'](globalThis, "a"); globalThis["Object"].prototype['hasOwnProperty'].call(globalThis, "a"); globalThis["Object"].prototype["hasOwnProperty"].call(globalThis, "a"); globalThis["Object"]["prototype"].hasOwnProperty.call(globalThis, "a"); globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](globalThis, "a"); globalThis["Object"]["prototype"].hasOwnProperty["call"](globalThis, "a"); globalThis.Object["prototype"].hasOwnProperty["call"](globalThis, "a");"#
}

const LATE_THREADED_RUNTIME_SEGMENTS: &[&str] = &[
    "SharedArrayBuffer",
    "globalThis.SharedArrayBuffer",
    r#"globalThis["SharedArrayBuffer"]"#,
    "globalThis['SharedArrayBuffer']",
    "Object.freeze(globalThis.SharedArrayBuffer)",
    r#"Object.freeze(globalThis["SharedArrayBuffer"])"#,
    "Object.freeze(globalThis['SharedArrayBuffer'])",
    "Object.freeze(SharedArrayBuffer)",
    "Object.freeze((SharedArrayBuffer))",
    "Object.freeze((globalThis.SharedArrayBuffer))",
    r#"Object.freeze((globalThis["SharedArrayBuffer"]))"#,
    "Object.freeze((globalThis['SharedArrayBuffer']))",
    "Object.freeze((null ?? globalThis.SharedArrayBuffer))",
    "Object.freeze((null ?? globalThis['SharedArrayBuffer']))",
    r#"Object.freeze((true && globalThis["SharedArrayBuffer"]))"#,
    "Object.freeze((true && globalThis['SharedArrayBuffer']))",
    "Object.freeze((true && globalThis.SharedArrayBuffer))",
    r#"Object.freeze((false || globalThis["SharedArrayBuffer"]))"#,
    "Object.freeze((false || globalThis['SharedArrayBuffer']))",
    "Object.freeze((false || globalThis.SharedArrayBuffer))",
    "Atomics",
    "globalThis.Atomics",
    r#"globalThis["Atomics"]"#,
    "globalThis['Atomics']",
    "Object.freeze(globalThis.Atomics)",
    r#"Object.freeze(globalThis["Atomics"])"#,
    "Object.freeze(globalThis['Atomics'])",
    "Object.freeze(Atomics)",
    "Object.freeze((Atomics))",
    "Object.freeze((globalThis.Atomics))",
    r#"Object.freeze((globalThis["Atomics"]))"#,
    "Object.freeze((globalThis['Atomics']))",
    "Object.freeze((null ?? globalThis.Atomics))",
    "Object.freeze((null ?? globalThis['Atomics']))",
    r#"Object.freeze((true && globalThis["Atomics"]))"#,
    "Object.freeze((true && globalThis['Atomics']))",
    "Object.freeze((true && globalThis.Atomics))",
    r#"Object.freeze((false || globalThis["Atomics"]))"#,
    "Object.freeze((false || globalThis['Atomics']))",
    "Object.freeze((false || globalThis.Atomics))",
];

/// Canonical alias inventory for the shared late-threaded-runtime slice.
pub fn late_threaded_runtime_aliases() -> &'static [&'static str] {
    LATE_THREADED_RUNTIME_SEGMENTS
}

/// Canonical late-threaded-runtime source text used by the browser and runtime smoke.
pub const fn late_threaded_runtime_source() -> &'static str {
    "SharedArrayBuffer; globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; Object.freeze(globalThis.SharedArrayBuffer); Object.freeze(globalThis[\"SharedArrayBuffer\"]); Object.freeze(globalThis['SharedArrayBuffer']); Object.freeze(SharedArrayBuffer); Object.freeze((SharedArrayBuffer)); Object.freeze((globalThis.SharedArrayBuffer)); Object.freeze((globalThis[\"SharedArrayBuffer\"])); Object.freeze((globalThis['SharedArrayBuffer'])); Object.freeze((null ?? globalThis.SharedArrayBuffer)); Object.freeze((null ?? globalThis['SharedArrayBuffer'])); Object.freeze((true && globalThis[\"SharedArrayBuffer\"])); Object.freeze((true && globalThis['SharedArrayBuffer'])); Object.freeze((true && globalThis.SharedArrayBuffer)); Object.freeze((false || globalThis[\"SharedArrayBuffer\"])); Object.freeze((false || globalThis['SharedArrayBuffer'])); Object.freeze((false || globalThis.SharedArrayBuffer)); Atomics; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics']; Object.freeze(globalThis.Atomics); Object.freeze(globalThis[\"Atomics\"]); Object.freeze(globalThis['Atomics']); Object.freeze(Atomics); Object.freeze((Atomics)); Object.freeze((globalThis.Atomics)); Object.freeze((globalThis[\"Atomics\"])); Object.freeze((globalThis['Atomics'])); Object.freeze((null ?? globalThis.Atomics)); Object.freeze((null ?? globalThis['Atomics'])); Object.freeze((true && globalThis[\"Atomics\"])); Object.freeze((true && globalThis['Atomics'])); Object.freeze((true && globalThis.Atomics)); Object.freeze((false || globalThis[\"Atomics\"])); Object.freeze((false || globalThis['Atomics'])); Object.freeze((false || globalThis.Atomics));"
}

const LATE_PERMISSION_ESCALATION_SEGMENTS: &[&str] = &[
    "Deno.permissions.request()",
    "Deno.permissions.revoke()",
    r#"Deno.permissions["request"]()"#,
    r#"Deno.permissions["revoke"]()"#,
    "globalThis.Deno.permissions.request()",
    "globalThis.Deno.permissions.revoke()",
    r#"globalThis.Deno.permissions["request"]()"#,
    r#"globalThis.Deno.permissions["revoke"]()"#,
    r#"globalThis["Deno"].permissions["request"]()"#,
    r#"globalThis["Deno"].permissions["revoke"]()"#,
    r#"globalThis["Deno"].permissions.request()"#,
    r#"globalThis["Deno"].permissions.revoke()"#,
    r#"globalThis["Deno"].permissions["request"]()"#,
    r#"globalThis["Deno"]["permissions"]["request"]()"#,
    r#"globalThis["Deno"]["permissions"]["revoke"]()"#,
    r#"globalThis["Deno"]["permissions"].request()"#,
    r#"globalThis["Deno"]["permissions"].revoke()"#,
    r#"globalThis["Deno"].permissions["request"]()"#,
    r#"globalThis["Deno"].permissions["revoke"]()"#,
    r#"globalThis.Deno["permissions"]["request"]()"#,
    r#"globalThis.Deno["permissions"]["revoke"]()"#,
];

/// Canonical late permission-escalation alias inventory used by the browser and runtime smoke.
pub fn late_permission_escalation_aliases() -> &'static [&'static str] {
    LATE_PERMISSION_ESCALATION_SEGMENTS
}

/// Canonical late permission-escalation source text used by the browser and runtime smoke.
pub fn late_permission_escalation_source() -> String {
    join_semicolon_terminated_segments(late_permission_escalation_aliases())
}

/// Canonical late environment-materialization source text used by the browser and runtime smoke.
pub const fn late_env_materialization_source() -> &'static str {
    "Deno.env.toObject(); globalThis.Deno.env.toObject(); Deno.env[\"toObject\"](); Deno[\"env\"][\"toObject\"](); Deno[\"env\"].toObject(); globalThis.Deno.env[\"toObject\"](); globalThis.Deno[\"env\"][\"toObject\"](); globalThis.Deno[\"env\"].toObject(); globalThis[\"Deno\"].env.toObject(); globalThis[\"Deno\"].env[\"toObject\"](); globalThis[\"Deno\"][\"env\"].toObject(); globalThis[\"Deno\"][\"env\"][\"toObject\"](); globalThis.Deno[\"env\"][\"toObject\"](); globalThis[\"Deno\"].env.toObject();"
}

/// Canonical late subprocess source text used by the browser and runtime smoke.
pub const fn late_subprocess_source() -> &'static str {
    "new Deno.Command('sh').spawn(); new Deno[\"Command\"]('sh').spawn(); new globalThis.Deno.Command('sh').spawn(); new globalThis.Deno[\"Command\"]('sh').spawn(); new globalThis[\"Deno\"].Command('sh').spawn(); new globalThis[\"Deno\"][\"Command\"]('sh').spawn();"
}

/// Canonical late network source text used by the browser and runtime smoke.
pub const fn late_network_source() -> &'static str {
    "Deno.connect('127.0.0.1', 1); globalThis.Deno.connect('127.0.0.1', 1); globalThis.Deno[\"connect\"]('127.0.0.1', 1); globalThis[\"Deno\"].connect('127.0.0.1', 1); globalThis[\"Deno\"][\"connect\"]('127.0.0.1', 1); Deno.listen('127.0.0.1', 0); globalThis.Deno.listen('127.0.0.1', 0); globalThis.Deno[\"listen\"]('127.0.0.1', 0); globalThis[\"Deno\"].listen('127.0.0.1', 0); globalThis[\"Deno\"][\"listen\"]('127.0.0.1', 0); Deno.serve('127.0.0.1', 0); globalThis.Deno.serve('127.0.0.1', 0); globalThis.Deno[\"serve\"]('127.0.0.1', 0); globalThis[\"Deno\"].serve('127.0.0.1', 0); globalThis[\"Deno\"][\"serve\"]('127.0.0.1', 0);"
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
