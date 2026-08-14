"""Rule-8 / rule-9 captured fixture texts for Task 18 batch 8A.

Every constant here is the BYTE-EXACT OUTPUT OF EXECUTING THE REAL CODE, never
a hand-applied `format!` substitution, a hand-applied `str::replace`, a
hand-applied indentation, or a retyped approximation. Rule 8 forbids
hand-simulating a `format!`; rule 9 extends the same discipline to a fixture
built one level removed inside a library crate (`kali_common::`) -- this batch
has both shapes, plus a third (`indent_source`, a `.lines()`/`format!`/`join`
re-indentation) that is the same trap wearing different clothes.

HOW THEY WERE CAPTURED, so they can be re-derived.

A temporary target `crates/kali_cli/tests/zz_b8a_dump.rs`, deleted in the same
session, with one `mod` per source that `include!`d the shipped `.rs` and a
`#[test] fn zz_dump_*` inside that `mod` (the fixture builders are private, so
the dump has to live in the module that includes them). Run as

    ZZ_B8A_OUT=<dir> cargo test -p kali_cli --test zz_b8a_dump -- zz_dump \
        --test-threads=1

`include!` rather than a retyped copy, so the executed `format!` /
`indent_source` / `kali_common` call is literally the one in the shipped
source. Every constant below came from that one run; none was edited
afterwards.

WHY EACH ONE IS HERE (i.e. why it is not a plain string literal the lexer could
have pulled straight out of the `.rs`):

  * CAP_PROMISE_ALL_* / CAP_PROMISE_ALL_SETTLED_* / CAP_PROMISE_RACE_* -- an
    inline `format!` with `{{`/`}}` brace-collapse whose body argument comes
    from `kali_common::promise`. Rule 9's "one level removed inside a library
    crate" case, on top of rule 8's brace-collapse case.
  * CAP_STRING_CONCAT_* -- a `format!` per command over an inline body. The
    `test` arm wraps the body in `Kali.test(...)` WITHOUT re-indenting it, so
    the resulting program has an unindented body inside the callback. Hand-
    deriving that (and "helpfully" indenting it) is exactly the trap.
  * CAP_TLSI_* -- `indent_source(kali_common::browser_template_literal_string_
    iteration_body_source(), "  ")` and then a `format!`. Two levels of
    transformation, and the `test` arm's indentation differs from the `run`
    arm's trailing-newline handling.
  * CAP_SET_ITERATION_* -- plain `&'static str` literals, captured through the
    same run anyway so that the run/test pair is proved to differ by exactly
    the wrapper rather than assumed to.
  * CAP_REFLECT_RUN -- a `format!` whose `{frozen_callable_lines}` argument is
    `kali_common::reflect_own_keys_frozen_callable_source("obj")`, i.e. rule
    9's library-crate case again. CAP_REFLECT_TEST / CAP_REFLECT_BUNDLE are
    plain literals, captured through the same run for the same reason as
    CAP_SET_ITERATION_*.

They are embedded here rather than read from a dump file so this module runs
from a clean checkout with no uncommitted inputs -- the defect that got the
pilot's per-file generators deleted (see README). `gen_batch8a.py` re-checks
each one against its own `.rs` before emitting it (`check_captured`), so a
stale capture taken before a source edit fails the generator rather than
shipping a program that is no longer the program under test.
"""


CAP_PROMISE_ALL_BUNDLE = (
    '// kali-tree-shake: browserPromiseAll\n'
    'async function browserPromiseAll() {\n'
    '  const direct = await Promise.all([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixed = await Promise["all"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixed = await Promise['all']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const dotted = await globalThis.Promise.all([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixedDotted = await globalThis.Promise["all"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleDotted = await globalThis.Promise['all']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const bracketed = await globalThis["Promise"].all([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixedBracketed = await globalThis["Promise"]["all"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleBracketed = await globalThis['Promise']['all']([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const singleMixedBracketed = await globalThis['Promise'].all([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const nullishRoot = await Object.freeze((null ?? Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalAndRoot = await Object.freeze((true && Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalOrRoot = await Object.freeze((false || Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const nullishDotted = await Object.freeze((null ?? globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalAndDotted = await Object.freeze((true && globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalOrDotted = await Object.freeze((false || globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const frozenRoot = await Object.freeze(Promise.all)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenRoot = await Object.freeze((Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const frozenBracketedRoot = await Object.freeze(Promise["all"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenBracketedRoot = await Object.freeze((Promise["all"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleBracketedRoot = await Object.freeze(Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedFrozenSingleBracketedRoot = await Object.freeze((Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const mixedRoot = await Object.freeze(globalThis.Promise["all"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedMixedRoot = await Object.freeze((globalThis.Promise["all"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixedRoot = await Object.freeze(globalThis.Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedSingleMixedRoot = await Object.freeze((globalThis.Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const bracketedRoot = await Object.freeze(globalThis["Promise"].all)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedBracketedRoot = await Object.freeze((globalThis["Promise"].all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixedBracketedRoot = await Object.freeze(globalThis["Promise"]["all"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedMixedBracketedRoot = await Object.freeze((globalThis["Promise"]["all"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixedBracketedRoot = await Object.freeze(globalThis['Promise'].all)([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedSingleMixedBracketedRoot = await Object.freeze((globalThis['Promise'].all))([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const fullyBracketedSingleRoot = await Object.freeze(globalThis['Promise']['all'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedFullyBracketedSingleRoot = await Object.freeze((globalThis['Promise']['all']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenGlobal = await Object.freeze(globalThis.Promise.all)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenGlobal = await Object.freeze((globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  if (\n'
    '    direct.length !== 2 ||\n'
    '    direct[0] !== 1 ||\n'
    '    direct[1] !== 2 ||\n'
    '    mixed.length !== 2 ||\n'
    '    mixed[0] !== 1 ||\n'
    '    mixed[1] !== 2 ||\n'
    '    singleMixed.length !== 2 ||\n'
    '    singleMixed[0] !== 1 ||\n'
    '    singleMixed[1] !== 2 ||\n'
    '    dotted.length !== 2 ||\n'
    '    dotted[0] !== 1 ||\n'
    '    dotted[1] !== 2 ||\n'
    '    mixedDotted.length !== 2 ||\n'
    '    mixedDotted[0] !== 1 ||\n'
    '    mixedDotted[1] !== 2 ||\n'
    '    singleDotted.length !== 2 ||\n'
    '    singleDotted[0] !== 1 ||\n'
    '    singleDotted[1] !== 2 ||\n'
    '    bracketed.length !== 2 ||\n'
    '    bracketed[0] !== 1 ||\n'
    '    bracketed[1] !== 2 ||\n'
    '    mixedBracketed.length !== 2 ||\n'
    '    mixedBracketed[0] !== 1 ||\n'
    '    mixedBracketed[1] !== 2 ||\n'
    '    singleBracketed.length !== 2 ||\n'
    '    singleBracketed[0] !== 1 ||\n'
    '    singleBracketed[1] !== 2 ||\n'
    '    singleMixedBracketed.length !== 2 ||\n'
    '    singleMixedBracketed[0] !== 1 ||\n'
    '    singleMixedBracketed[1] !== 2 ||\n'
    '    nullishRoot.length !== 2 ||\n'
    '    nullishRoot[0] !== 1 ||\n'
    '    nullishRoot[1] !== 2 ||\n'
    '    logicalAndRoot.length !== 2 ||\n'
    '    logicalAndRoot[0] !== 1 ||\n'
    '    logicalAndRoot[1] !== 2 ||\n'
    '    logicalOrRoot.length !== 2 ||\n'
    '    logicalOrRoot[0] !== 1 ||\n'
    '    logicalOrRoot[1] !== 2 ||\n'
    '    nullishDotted.length !== 2 ||\n'
    '    nullishDotted[0] !== 1 ||\n'
    '    nullishDotted[1] !== 2 ||\n'
    '    logicalAndDotted.length !== 2 ||\n'
    '    logicalAndDotted[0] !== 1 ||\n'
    '    logicalAndDotted[1] !== 2 ||\n'
    '    logicalOrDotted.length !== 2 ||\n'
    '    logicalOrDotted[0] !== 1 ||\n'
    '    logicalOrDotted[1] !== 2 ||\n'
    '    frozenRoot.length !== 2 ||\n'
    '    frozenRoot[0] !== 1 ||\n'
    '    frozenRoot[1] !== 2 ||\n'
    '    parenthesizedFrozenRoot.length !== 2 ||\n'
    '    parenthesizedFrozenRoot[0] !== 1 ||\n'
    '    parenthesizedFrozenRoot[1] !== 2 ||\n'
    '    frozenBracketedRoot.length !== 2 ||\n'
    '    frozenBracketedRoot[0] !== 1 ||\n'
    '    frozenBracketedRoot[1] !== 2 ||\n'
    '    parenthesizedFrozenBracketedRoot.length !== 2 ||\n'
    '    parenthesizedFrozenBracketedRoot[0] !== 1 ||\n'
    '    parenthesizedFrozenBracketedRoot[1] !== 2 ||\n'
    '    frozenSingleBracketedRoot.length !== 2 ||\n'
    '    frozenSingleBracketedRoot[0] !== 1 ||\n'
    '    frozenSingleBracketedRoot[1] !== 2 ||\n'
    '    parenthesizedFrozenSingleBracketedRoot.length !== 2 ||\n'
    '    parenthesizedFrozenSingleBracketedRoot[0] !== 1 ||\n'
    '    parenthesizedFrozenSingleBracketedRoot[1] !== 2 ||\n'
    '    frozenGlobal.length !== 2 ||\n'
    '    frozenGlobal[0] !== 1 ||\n'
    '    frozenGlobal[1] !== 2 ||\n'
    '    parenthesizedFrozenGlobal.length !== 2 ||\n'
    '    parenthesizedFrozenGlobal[0] !== 1 ||\n'
    '    parenthesizedFrozenGlobal[1] !== 2\n'
    '  ) {\n'
    '    throw new Error("unexpected Promise.all results");\n'
    '  }\n'
    '\n'
    '}\n'
)

CAP_PROMISE_ALL_RUN = (
    'async function browserPromiseAll() {\n'
    '  const direct = await Promise.all([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixed = await Promise["all"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixed = await Promise['all']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const dotted = await globalThis.Promise.all([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixedDotted = await globalThis.Promise["all"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleDotted = await globalThis.Promise['all']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const bracketed = await globalThis["Promise"].all([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixedBracketed = await globalThis["Promise"]["all"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleBracketed = await globalThis['Promise']['all']([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const singleMixedBracketed = await globalThis['Promise'].all([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const nullishRoot = await Object.freeze((null ?? Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalAndRoot = await Object.freeze((true && Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalOrRoot = await Object.freeze((false || Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const nullishDotted = await Object.freeze((null ?? globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalAndDotted = await Object.freeze((true && globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalOrDotted = await Object.freeze((false || globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const frozenRoot = await Object.freeze(Promise.all)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenRoot = await Object.freeze((Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const frozenBracketedRoot = await Object.freeze(Promise["all"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenBracketedRoot = await Object.freeze((Promise["all"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleBracketedRoot = await Object.freeze(Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedFrozenSingleBracketedRoot = await Object.freeze((Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const mixedRoot = await Object.freeze(globalThis.Promise["all"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedMixedRoot = await Object.freeze((globalThis.Promise["all"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixedRoot = await Object.freeze(globalThis.Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedSingleMixedRoot = await Object.freeze((globalThis.Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const bracketedRoot = await Object.freeze(globalThis["Promise"].all)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedBracketedRoot = await Object.freeze((globalThis["Promise"].all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixedBracketedRoot = await Object.freeze(globalThis["Promise"]["all"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedMixedBracketedRoot = await Object.freeze((globalThis["Promise"]["all"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixedBracketedRoot = await Object.freeze(globalThis['Promise'].all)([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedSingleMixedBracketedRoot = await Object.freeze((globalThis['Promise'].all))([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const fullyBracketedSingleRoot = await Object.freeze(globalThis['Promise']['all'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedFullyBracketedSingleRoot = await Object.freeze((globalThis['Promise']['all']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenGlobal = await Object.freeze(globalThis.Promise.all)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenGlobal = await Object.freeze((globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  if (\n'
    '    direct.length !== 2 ||\n'
    '    direct[0] !== 1 ||\n'
    '    direct[1] !== 2 ||\n'
    '    mixed.length !== 2 ||\n'
    '    mixed[0] !== 1 ||\n'
    '    mixed[1] !== 2 ||\n'
    '    singleMixed.length !== 2 ||\n'
    '    singleMixed[0] !== 1 ||\n'
    '    singleMixed[1] !== 2 ||\n'
    '    dotted.length !== 2 ||\n'
    '    dotted[0] !== 1 ||\n'
    '    dotted[1] !== 2 ||\n'
    '    mixedDotted.length !== 2 ||\n'
    '    mixedDotted[0] !== 1 ||\n'
    '    mixedDotted[1] !== 2 ||\n'
    '    singleDotted.length !== 2 ||\n'
    '    singleDotted[0] !== 1 ||\n'
    '    singleDotted[1] !== 2 ||\n'
    '    bracketed.length !== 2 ||\n'
    '    bracketed[0] !== 1 ||\n'
    '    bracketed[1] !== 2 ||\n'
    '    mixedBracketed.length !== 2 ||\n'
    '    mixedBracketed[0] !== 1 ||\n'
    '    mixedBracketed[1] !== 2 ||\n'
    '    singleBracketed.length !== 2 ||\n'
    '    singleBracketed[0] !== 1 ||\n'
    '    singleBracketed[1] !== 2 ||\n'
    '    singleMixedBracketed.length !== 2 ||\n'
    '    singleMixedBracketed[0] !== 1 ||\n'
    '    singleMixedBracketed[1] !== 2 ||\n'
    '    nullishRoot.length !== 2 ||\n'
    '    nullishRoot[0] !== 1 ||\n'
    '    nullishRoot[1] !== 2 ||\n'
    '    logicalAndRoot.length !== 2 ||\n'
    '    logicalAndRoot[0] !== 1 ||\n'
    '    logicalAndRoot[1] !== 2 ||\n'
    '    logicalOrRoot.length !== 2 ||\n'
    '    logicalOrRoot[0] !== 1 ||\n'
    '    logicalOrRoot[1] !== 2 ||\n'
    '    nullishDotted.length !== 2 ||\n'
    '    nullishDotted[0] !== 1 ||\n'
    '    nullishDotted[1] !== 2 ||\n'
    '    logicalAndDotted.length !== 2 ||\n'
    '    logicalAndDotted[0] !== 1 ||\n'
    '    logicalAndDotted[1] !== 2 ||\n'
    '    logicalOrDotted.length !== 2 ||\n'
    '    logicalOrDotted[0] !== 1 ||\n'
    '    logicalOrDotted[1] !== 2 ||\n'
    '    frozenRoot.length !== 2 ||\n'
    '    frozenRoot[0] !== 1 ||\n'
    '    frozenRoot[1] !== 2 ||\n'
    '    parenthesizedFrozenRoot.length !== 2 ||\n'
    '    parenthesizedFrozenRoot[0] !== 1 ||\n'
    '    parenthesizedFrozenRoot[1] !== 2 ||\n'
    '    frozenBracketedRoot.length !== 2 ||\n'
    '    frozenBracketedRoot[0] !== 1 ||\n'
    '    frozenBracketedRoot[1] !== 2 ||\n'
    '    parenthesizedFrozenBracketedRoot.length !== 2 ||\n'
    '    parenthesizedFrozenBracketedRoot[0] !== 1 ||\n'
    '    parenthesizedFrozenBracketedRoot[1] !== 2 ||\n'
    '    frozenSingleBracketedRoot.length !== 2 ||\n'
    '    frozenSingleBracketedRoot[0] !== 1 ||\n'
    '    frozenSingleBracketedRoot[1] !== 2 ||\n'
    '    parenthesizedFrozenSingleBracketedRoot.length !== 2 ||\n'
    '    parenthesizedFrozenSingleBracketedRoot[0] !== 1 ||\n'
    '    parenthesizedFrozenSingleBracketedRoot[1] !== 2 ||\n'
    '    frozenGlobal.length !== 2 ||\n'
    '    frozenGlobal[0] !== 1 ||\n'
    '    frozenGlobal[1] !== 2 ||\n'
    '    parenthesizedFrozenGlobal.length !== 2 ||\n'
    '    parenthesizedFrozenGlobal[0] !== 1 ||\n'
    '    parenthesizedFrozenGlobal[1] !== 2\n'
    '  ) {\n'
    '    throw new Error("unexpected Promise.all results");\n'
    '  }\n'
    '\n'
    '}\n'
    '\n'
    'async function main() {\n'
    '  await browserPromiseAll();\n'
    "  console.log('browser promise all ok');\n"
    '}\n'
    '\n'
    'main();\n'
)

CAP_PROMISE_ALL_TEST = (
    'async function browserPromiseAll() {\n'
    '  const direct = await Promise.all([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixed = await Promise["all"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixed = await Promise['all']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const dotted = await globalThis.Promise.all([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixedDotted = await globalThis.Promise["all"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleDotted = await globalThis.Promise['all']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const bracketed = await globalThis["Promise"].all([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixedBracketed = await globalThis["Promise"]["all"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleBracketed = await globalThis['Promise']['all']([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const singleMixedBracketed = await globalThis['Promise'].all([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const nullishRoot = await Object.freeze((null ?? Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalAndRoot = await Object.freeze((true && Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalOrRoot = await Object.freeze((false || Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const nullishDotted = await Object.freeze((null ?? globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalAndDotted = await Object.freeze((true && globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const logicalOrDotted = await Object.freeze((false || globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const frozenRoot = await Object.freeze(Promise.all)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenRoot = await Object.freeze((Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const frozenBracketedRoot = await Object.freeze(Promise["all"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenBracketedRoot = await Object.freeze((Promise["all"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleBracketedRoot = await Object.freeze(Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedFrozenSingleBracketedRoot = await Object.freeze((Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const mixedRoot = await Object.freeze(globalThis.Promise["all"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedMixedRoot = await Object.freeze((globalThis.Promise["all"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixedRoot = await Object.freeze(globalThis.Promise['all'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedSingleMixedRoot = await Object.freeze((globalThis.Promise['all']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const bracketedRoot = await Object.freeze(globalThis["Promise"].all)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedBracketedRoot = await Object.freeze((globalThis["Promise"].all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixedBracketedRoot = await Object.freeze(globalThis["Promise"]["all"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedMixedBracketedRoot = await Object.freeze((globalThis["Promise"]["all"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixedBracketedRoot = await Object.freeze(globalThis['Promise'].all)([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedSingleMixedBracketedRoot = await Object.freeze((globalThis['Promise'].all))([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const fullyBracketedSingleRoot = await Object.freeze(globalThis['Promise']['all'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    "  const parenthesizedFullyBracketedSingleRoot = await Object.freeze((globalThis['Promise']['all']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenGlobal = await Object.freeze(globalThis.Promise.all)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenGlobal = await Object.freeze((globalThis.Promise.all))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  if (\n'
    '    direct.length !== 2 ||\n'
    '    direct[0] !== 1 ||\n'
    '    direct[1] !== 2 ||\n'
    '    mixed.length !== 2 ||\n'
    '    mixed[0] !== 1 ||\n'
    '    mixed[1] !== 2 ||\n'
    '    singleMixed.length !== 2 ||\n'
    '    singleMixed[0] !== 1 ||\n'
    '    singleMixed[1] !== 2 ||\n'
    '    dotted.length !== 2 ||\n'
    '    dotted[0] !== 1 ||\n'
    '    dotted[1] !== 2 ||\n'
    '    mixedDotted.length !== 2 ||\n'
    '    mixedDotted[0] !== 1 ||\n'
    '    mixedDotted[1] !== 2 ||\n'
    '    singleDotted.length !== 2 ||\n'
    '    singleDotted[0] !== 1 ||\n'
    '    singleDotted[1] !== 2 ||\n'
    '    bracketed.length !== 2 ||\n'
    '    bracketed[0] !== 1 ||\n'
    '    bracketed[1] !== 2 ||\n'
    '    mixedBracketed.length !== 2 ||\n'
    '    mixedBracketed[0] !== 1 ||\n'
    '    mixedBracketed[1] !== 2 ||\n'
    '    singleBracketed.length !== 2 ||\n'
    '    singleBracketed[0] !== 1 ||\n'
    '    singleBracketed[1] !== 2 ||\n'
    '    singleMixedBracketed.length !== 2 ||\n'
    '    singleMixedBracketed[0] !== 1 ||\n'
    '    singleMixedBracketed[1] !== 2 ||\n'
    '    nullishRoot.length !== 2 ||\n'
    '    nullishRoot[0] !== 1 ||\n'
    '    nullishRoot[1] !== 2 ||\n'
    '    logicalAndRoot.length !== 2 ||\n'
    '    logicalAndRoot[0] !== 1 ||\n'
    '    logicalAndRoot[1] !== 2 ||\n'
    '    logicalOrRoot.length !== 2 ||\n'
    '    logicalOrRoot[0] !== 1 ||\n'
    '    logicalOrRoot[1] !== 2 ||\n'
    '    nullishDotted.length !== 2 ||\n'
    '    nullishDotted[0] !== 1 ||\n'
    '    nullishDotted[1] !== 2 ||\n'
    '    logicalAndDotted.length !== 2 ||\n'
    '    logicalAndDotted[0] !== 1 ||\n'
    '    logicalAndDotted[1] !== 2 ||\n'
    '    logicalOrDotted.length !== 2 ||\n'
    '    logicalOrDotted[0] !== 1 ||\n'
    '    logicalOrDotted[1] !== 2 ||\n'
    '    frozenRoot.length !== 2 ||\n'
    '    frozenRoot[0] !== 1 ||\n'
    '    frozenRoot[1] !== 2 ||\n'
    '    parenthesizedFrozenRoot.length !== 2 ||\n'
    '    parenthesizedFrozenRoot[0] !== 1 ||\n'
    '    parenthesizedFrozenRoot[1] !== 2 ||\n'
    '    frozenBracketedRoot.length !== 2 ||\n'
    '    frozenBracketedRoot[0] !== 1 ||\n'
    '    frozenBracketedRoot[1] !== 2 ||\n'
    '    parenthesizedFrozenBracketedRoot.length !== 2 ||\n'
    '    parenthesizedFrozenBracketedRoot[0] !== 1 ||\n'
    '    parenthesizedFrozenBracketedRoot[1] !== 2 ||\n'
    '    frozenSingleBracketedRoot.length !== 2 ||\n'
    '    frozenSingleBracketedRoot[0] !== 1 ||\n'
    '    frozenSingleBracketedRoot[1] !== 2 ||\n'
    '    parenthesizedFrozenSingleBracketedRoot.length !== 2 ||\n'
    '    parenthesizedFrozenSingleBracketedRoot[0] !== 1 ||\n'
    '    parenthesizedFrozenSingleBracketedRoot[1] !== 2 ||\n'
    '    frozenGlobal.length !== 2 ||\n'
    '    frozenGlobal[0] !== 1 ||\n'
    '    frozenGlobal[1] !== 2 ||\n'
    '    parenthesizedFrozenGlobal.length !== 2 ||\n'
    '    parenthesizedFrozenGlobal[0] !== 1 ||\n'
    '    parenthesizedFrozenGlobal[1] !== 2\n'
    '  ) {\n'
    '    throw new Error("unexpected Promise.all results");\n'
    '  }\n'
    '\n'
    '}\n'
    '\n'
    "Kali.test('browser promise all', () => browserPromiseAll());\n"
)

CAP_PROMISE_ALL_SETTLED_BUNDLE = (
    '// kali-tree-shake: browserPromiseAllSettled\n'
    'async function browserPromiseAllSettled() {\n'
    "  const settled = await Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedSettled = await Promise["allSettled"]([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleMixedSettled = await Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const dottedSettled = await globalThis.Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedDottedSettled = await globalThis.Promise["allSettled"]([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleDottedSettled = await globalThis.Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedBracketedSettled = await globalThis["Promise"].allSettled([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const bracketedSettled = await globalThis["Promise"]["allSettled"]([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleBracketedSettled = await globalThis['Promise']['allSettled']([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const singleMixedBracketedSettled = await globalThis['Promise'].allSettled([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const nullishRootSettled = await Object.freeze((null ?? Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalAndRootSettled = await Object.freeze((true && Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalOrRootSettled = await Object.freeze((false || Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const nullishDottedSettled = await Object.freeze((null ?? globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalAndDottedSettled = await Object.freeze((true && globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalOrDottedSettled = await Object.freeze((false || globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const wrappedDottedRootFrozenSettled = await Object.freeze((globalThis.Promise)["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const wrappedBracketedRootFrozenSettled = await Object.freeze((globalThis["Promise"])["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const wrappedBracketedDotRootFrozenSettled = await Object.freeze((globalThis["Promise"]).allSettled)([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const wrappedSingleBracketedDotRootFrozenSettled = await Object.freeze((globalThis['Promise']).allSettled)([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const frozenBracketedSettled = await Object.freeze(globalThis["Promise"]["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedFrozenBracketedSettled = await Object.freeze((globalThis["Promise"]["allSettled"]))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleFrozenBracketedSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleFrozenBracketedSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedBracketedRootFrozenSettled = await Object.freeze(globalThis["Promise"].allSettled)([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedMixedBracketedRootFrozenSettled = await Object.freeze((globalThis["Promise"].allSettled))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleMixedBracketedRootFrozenSettled = await Object.freeze(globalThis['Promise'].allSettled)([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const fullyBracketedSingleRootFrozenSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedFullyBracketedSingleRootFrozenSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleMixedBracketedRootFrozenSettled = await Object.freeze((globalThis['Promise'].allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedRootFrozenSettled = await Object.freeze(globalThis.Promise["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedMixedRootFrozenSettled = await Object.freeze((globalThis.Promise["allSettled"]))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleMixedRootFrozenSettled = await Object.freeze(globalThis.Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleMixedRootFrozenSettled = await Object.freeze((globalThis.Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const bracketedRootFrozenSettled = await Object.freeze(Promise["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedBracketedRootFrozenSettled = await Object.freeze((Promise["allSettled"]))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleBracketedRootFrozenSettled = await Object.freeze(Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleBracketedRootFrozenSettled = await Object.freeze((Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const rootFrozenSettled = await Object.freeze(Promise.allSettled)([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedRootFrozenSettled = await Object.freeze((Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  if (\n'
    '    settled.length !== 2 ||\n'
    "    settled[0].status !== 'fulfilled' ||\n"
    '    settled[0].value !== 1 ||\n'
    "    settled[1].status !== 'rejected' ||\n"
    "    settled[1].reason !== 'boom' ||\n"
    '    mixedSettled.length !== 2 ||\n'
    "    mixedSettled[0].status !== 'fulfilled' ||\n"
    '    mixedSettled[0].value !== 1 ||\n'
    "    mixedSettled[1].status !== 'rejected' ||\n"
    "    mixedSettled[1].reason !== 'boom' ||\n"
    '    dottedSettled.length !== 2 ||\n'
    "    dottedSettled[0].status !== 'fulfilled' ||\n"
    '    dottedSettled[0].value !== 1 ||\n'
    "    dottedSettled[1].status !== 'rejected' ||\n"
    "    dottedSettled[1].reason !== 'boom' ||\n"
    '    mixedDottedSettled.length !== 2 ||\n'
    "    mixedDottedSettled[0].status !== 'fulfilled' ||\n"
    '    mixedDottedSettled[0].value !== 1 ||\n'
    "    mixedDottedSettled[1].status !== 'rejected' ||\n"
    "    mixedDottedSettled[1].reason !== 'boom' ||\n"
    '    mixedBracketedSettled.length !== 2 ||\n'
    "    mixedBracketedSettled[0].status !== 'fulfilled' ||\n"
    '    mixedBracketedSettled[0].value !== 1 ||\n'
    "    mixedBracketedSettled[1].status !== 'rejected' ||\n"
    "    mixedBracketedSettled[1].reason !== 'boom' ||\n"
    '    bracketedSettled.length !== 2 ||\n'
    "    bracketedSettled[0].status !== 'fulfilled' ||\n"
    '    bracketedSettled[0].value !== 1 ||\n'
    "    bracketedSettled[1].status !== 'rejected' ||\n"
    "    bracketedSettled[1].reason !== 'boom' ||\n"
    '    nullishRootSettled.length !== 2 ||\n'
    "    nullishRootSettled[0].status !== 'fulfilled' ||\n"
    '    nullishRootSettled[0].value !== 1 ||\n'
    "    nullishRootSettled[1].status !== 'rejected' ||\n"
    "    nullishRootSettled[1].reason !== 'boom' ||\n"
    '    logicalAndRootSettled.length !== 2 ||\n'
    "    logicalAndRootSettled[0].status !== 'fulfilled' ||\n"
    '    logicalAndRootSettled[0].value !== 1 ||\n'
    "    logicalAndRootSettled[1].status !== 'rejected' ||\n"
    "    logicalAndRootSettled[1].reason !== 'boom' ||\n"
    '    logicalOrRootSettled.length !== 2 ||\n'
    "    logicalOrRootSettled[0].status !== 'fulfilled' ||\n"
    '    logicalOrRootSettled[0].value !== 1 ||\n'
    "    logicalOrRootSettled[1].status !== 'rejected' ||\n"
    "    logicalOrRootSettled[1].reason !== 'boom' ||\n"
    '    nullishDottedSettled.length !== 2 ||\n'
    "    nullishDottedSettled[0].status !== 'fulfilled' ||\n"
    '    nullishDottedSettled[0].value !== 1 ||\n'
    "    nullishDottedSettled[1].status !== 'rejected' ||\n"
    "    nullishDottedSettled[1].reason !== 'boom' ||\n"
    '    logicalAndDottedSettled.length !== 2 ||\n'
    "    logicalAndDottedSettled[0].status !== 'fulfilled' ||\n"
    '    logicalAndDottedSettled[0].value !== 1 ||\n'
    "    logicalAndDottedSettled[1].status !== 'rejected' ||\n"
    "    logicalAndDottedSettled[1].reason !== 'boom' ||\n"
    '    logicalOrDottedSettled.length !== 2 ||\n'
    "    logicalOrDottedSettled[0].status !== 'fulfilled' ||\n"
    '    logicalOrDottedSettled[0].value !== 1 ||\n'
    "    logicalOrDottedSettled[1].status !== 'rejected' ||\n"
    "    logicalOrDottedSettled[1].reason !== 'boom' ||\n"
    '    wrappedBracketedDotRootFrozenSettled.length !== 2 ||\n'
    "    wrappedBracketedDotRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    wrappedBracketedDotRootFrozenSettled[0].value !== 1 ||\n'
    "    wrappedBracketedDotRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    wrappedBracketedDotRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    wrappedSingleBracketedDotRootFrozenSettled.length !== 2 ||\n'
    "    wrappedSingleBracketedDotRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    wrappedSingleBracketedDotRootFrozenSettled[0].value !== 1 ||\n'
    "    wrappedSingleBracketedDotRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    wrappedSingleBracketedDotRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    frozenBracketedSettled.length !== 2 ||\n'
    "    frozenBracketedSettled[0].status !== 'fulfilled' ||\n"
    '    frozenBracketedSettled[0].value !== 1 ||\n'
    "    frozenBracketedSettled[1].status !== 'rejected' ||\n"
    "    frozenBracketedSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedFrozenBracketedSettled.length !== 2 ||\n'
    "    parenthesizedFrozenBracketedSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedFrozenBracketedSettled[0].value !== 1 ||\n'
    "    parenthesizedFrozenBracketedSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedFrozenBracketedSettled[1].reason !== 'boom' ||\n"
    '    mixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    mixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    mixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    mixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    mixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedMixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedMixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    singleMixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    singleMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    singleMixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    singleMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    singleMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    fullyBracketedSingleRootFrozenSettled.length !== 2 ||\n'
    "    fullyBracketedSingleRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    fullyBracketedSingleRootFrozenSettled[0].value !== 1 ||\n'
    "    fullyBracketedSingleRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    fullyBracketedSingleRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedFullyBracketedSingleRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedFullyBracketedSingleRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedFullyBracketedSingleRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedFullyBracketedSingleRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedFullyBracketedSingleRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedSingleMixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedSingleMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedSingleMixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedSingleMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedSingleMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    mixedRootFrozenSettled.length !== 2 ||\n'
    "    mixedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    mixedRootFrozenSettled[0].value !== 1 ||\n'
    "    mixedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    mixedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedMixedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedMixedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedMixedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedMixedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedMixedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    bracketedRootFrozenSettled.length !== 2 ||\n'
    "    bracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    bracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    bracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    bracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    rootFrozenSettled.length !== 2 ||\n'
    "    rootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    rootFrozenSettled[0].value !== 1 ||\n'
    "    rootFrozenSettled[1].status !== 'rejected' ||\n"
    "    rootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedRootFrozenSettled[1].reason !== 'boom'\n"
    '  ) {\n'
    "    throw new Error('unexpected Promise.allSettled semantics');\n"
    '  }\n'
    '\n'
    '}\n'
)

CAP_PROMISE_ALL_SETTLED_RUN = (
    'async function browserPromiseAllSettled() {\n'
    "  const settled = await Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedSettled = await Promise["allSettled"]([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleMixedSettled = await Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const dottedSettled = await globalThis.Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedDottedSettled = await globalThis.Promise["allSettled"]([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleDottedSettled = await globalThis.Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedBracketedSettled = await globalThis["Promise"].allSettled([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const bracketedSettled = await globalThis["Promise"]["allSettled"]([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleBracketedSettled = await globalThis['Promise']['allSettled']([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const singleMixedBracketedSettled = await globalThis['Promise'].allSettled([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const nullishRootSettled = await Object.freeze((null ?? Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalAndRootSettled = await Object.freeze((true && Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalOrRootSettled = await Object.freeze((false || Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const nullishDottedSettled = await Object.freeze((null ?? globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalAndDottedSettled = await Object.freeze((true && globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalOrDottedSettled = await Object.freeze((false || globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const wrappedDottedRootFrozenSettled = await Object.freeze((globalThis.Promise)["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const wrappedBracketedRootFrozenSettled = await Object.freeze((globalThis["Promise"])["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const wrappedBracketedDotRootFrozenSettled = await Object.freeze((globalThis["Promise"]).allSettled)([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const wrappedSingleBracketedDotRootFrozenSettled = await Object.freeze((globalThis['Promise']).allSettled)([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const frozenBracketedSettled = await Object.freeze(globalThis["Promise"]["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedFrozenBracketedSettled = await Object.freeze((globalThis["Promise"]["allSettled"]))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleFrozenBracketedSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleFrozenBracketedSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedBracketedRootFrozenSettled = await Object.freeze(globalThis["Promise"].allSettled)([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedMixedBracketedRootFrozenSettled = await Object.freeze((globalThis["Promise"].allSettled))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleMixedBracketedRootFrozenSettled = await Object.freeze(globalThis['Promise'].allSettled)([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const fullyBracketedSingleRootFrozenSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedFullyBracketedSingleRootFrozenSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleMixedBracketedRootFrozenSettled = await Object.freeze((globalThis['Promise'].allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedRootFrozenSettled = await Object.freeze(globalThis.Promise["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedMixedRootFrozenSettled = await Object.freeze((globalThis.Promise["allSettled"]))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleMixedRootFrozenSettled = await Object.freeze(globalThis.Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleMixedRootFrozenSettled = await Object.freeze((globalThis.Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const bracketedRootFrozenSettled = await Object.freeze(Promise["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedBracketedRootFrozenSettled = await Object.freeze((Promise["allSettled"]))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleBracketedRootFrozenSettled = await Object.freeze(Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleBracketedRootFrozenSettled = await Object.freeze((Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const rootFrozenSettled = await Object.freeze(Promise.allSettled)([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedRootFrozenSettled = await Object.freeze((Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  if (\n'
    '    settled.length !== 2 ||\n'
    "    settled[0].status !== 'fulfilled' ||\n"
    '    settled[0].value !== 1 ||\n'
    "    settled[1].status !== 'rejected' ||\n"
    "    settled[1].reason !== 'boom' ||\n"
    '    mixedSettled.length !== 2 ||\n'
    "    mixedSettled[0].status !== 'fulfilled' ||\n"
    '    mixedSettled[0].value !== 1 ||\n'
    "    mixedSettled[1].status !== 'rejected' ||\n"
    "    mixedSettled[1].reason !== 'boom' ||\n"
    '    dottedSettled.length !== 2 ||\n'
    "    dottedSettled[0].status !== 'fulfilled' ||\n"
    '    dottedSettled[0].value !== 1 ||\n'
    "    dottedSettled[1].status !== 'rejected' ||\n"
    "    dottedSettled[1].reason !== 'boom' ||\n"
    '    mixedDottedSettled.length !== 2 ||\n'
    "    mixedDottedSettled[0].status !== 'fulfilled' ||\n"
    '    mixedDottedSettled[0].value !== 1 ||\n'
    "    mixedDottedSettled[1].status !== 'rejected' ||\n"
    "    mixedDottedSettled[1].reason !== 'boom' ||\n"
    '    mixedBracketedSettled.length !== 2 ||\n'
    "    mixedBracketedSettled[0].status !== 'fulfilled' ||\n"
    '    mixedBracketedSettled[0].value !== 1 ||\n'
    "    mixedBracketedSettled[1].status !== 'rejected' ||\n"
    "    mixedBracketedSettled[1].reason !== 'boom' ||\n"
    '    bracketedSettled.length !== 2 ||\n'
    "    bracketedSettled[0].status !== 'fulfilled' ||\n"
    '    bracketedSettled[0].value !== 1 ||\n'
    "    bracketedSettled[1].status !== 'rejected' ||\n"
    "    bracketedSettled[1].reason !== 'boom' ||\n"
    '    nullishRootSettled.length !== 2 ||\n'
    "    nullishRootSettled[0].status !== 'fulfilled' ||\n"
    '    nullishRootSettled[0].value !== 1 ||\n'
    "    nullishRootSettled[1].status !== 'rejected' ||\n"
    "    nullishRootSettled[1].reason !== 'boom' ||\n"
    '    logicalAndRootSettled.length !== 2 ||\n'
    "    logicalAndRootSettled[0].status !== 'fulfilled' ||\n"
    '    logicalAndRootSettled[0].value !== 1 ||\n'
    "    logicalAndRootSettled[1].status !== 'rejected' ||\n"
    "    logicalAndRootSettled[1].reason !== 'boom' ||\n"
    '    logicalOrRootSettled.length !== 2 ||\n'
    "    logicalOrRootSettled[0].status !== 'fulfilled' ||\n"
    '    logicalOrRootSettled[0].value !== 1 ||\n'
    "    logicalOrRootSettled[1].status !== 'rejected' ||\n"
    "    logicalOrRootSettled[1].reason !== 'boom' ||\n"
    '    nullishDottedSettled.length !== 2 ||\n'
    "    nullishDottedSettled[0].status !== 'fulfilled' ||\n"
    '    nullishDottedSettled[0].value !== 1 ||\n'
    "    nullishDottedSettled[1].status !== 'rejected' ||\n"
    "    nullishDottedSettled[1].reason !== 'boom' ||\n"
    '    logicalAndDottedSettled.length !== 2 ||\n'
    "    logicalAndDottedSettled[0].status !== 'fulfilled' ||\n"
    '    logicalAndDottedSettled[0].value !== 1 ||\n'
    "    logicalAndDottedSettled[1].status !== 'rejected' ||\n"
    "    logicalAndDottedSettled[1].reason !== 'boom' ||\n"
    '    logicalOrDottedSettled.length !== 2 ||\n'
    "    logicalOrDottedSettled[0].status !== 'fulfilled' ||\n"
    '    logicalOrDottedSettled[0].value !== 1 ||\n'
    "    logicalOrDottedSettled[1].status !== 'rejected' ||\n"
    "    logicalOrDottedSettled[1].reason !== 'boom' ||\n"
    '    wrappedBracketedDotRootFrozenSettled.length !== 2 ||\n'
    "    wrappedBracketedDotRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    wrappedBracketedDotRootFrozenSettled[0].value !== 1 ||\n'
    "    wrappedBracketedDotRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    wrappedBracketedDotRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    wrappedSingleBracketedDotRootFrozenSettled.length !== 2 ||\n'
    "    wrappedSingleBracketedDotRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    wrappedSingleBracketedDotRootFrozenSettled[0].value !== 1 ||\n'
    "    wrappedSingleBracketedDotRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    wrappedSingleBracketedDotRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    frozenBracketedSettled.length !== 2 ||\n'
    "    frozenBracketedSettled[0].status !== 'fulfilled' ||\n"
    '    frozenBracketedSettled[0].value !== 1 ||\n'
    "    frozenBracketedSettled[1].status !== 'rejected' ||\n"
    "    frozenBracketedSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedFrozenBracketedSettled.length !== 2 ||\n'
    "    parenthesizedFrozenBracketedSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedFrozenBracketedSettled[0].value !== 1 ||\n'
    "    parenthesizedFrozenBracketedSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedFrozenBracketedSettled[1].reason !== 'boom' ||\n"
    '    mixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    mixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    mixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    mixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    mixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedMixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedMixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    singleMixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    singleMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    singleMixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    singleMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    singleMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    fullyBracketedSingleRootFrozenSettled.length !== 2 ||\n'
    "    fullyBracketedSingleRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    fullyBracketedSingleRootFrozenSettled[0].value !== 1 ||\n'
    "    fullyBracketedSingleRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    fullyBracketedSingleRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedFullyBracketedSingleRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedFullyBracketedSingleRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedFullyBracketedSingleRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedFullyBracketedSingleRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedFullyBracketedSingleRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedSingleMixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedSingleMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedSingleMixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedSingleMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedSingleMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    mixedRootFrozenSettled.length !== 2 ||\n'
    "    mixedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    mixedRootFrozenSettled[0].value !== 1 ||\n'
    "    mixedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    mixedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedMixedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedMixedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedMixedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedMixedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedMixedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    bracketedRootFrozenSettled.length !== 2 ||\n'
    "    bracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    bracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    bracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    bracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    rootFrozenSettled.length !== 2 ||\n'
    "    rootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    rootFrozenSettled[0].value !== 1 ||\n'
    "    rootFrozenSettled[1].status !== 'rejected' ||\n"
    "    rootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedRootFrozenSettled[1].reason !== 'boom'\n"
    '  ) {\n'
    "    throw new Error('unexpected Promise.allSettled semantics');\n"
    '  }\n'
    '\n'
    '}\n'
    '\n'
    'async function main() {\n'
    '  await browserPromiseAllSettled();\n'
    "  console.log('browser promise allSettled ok');\n"
    '}\n'
    '\n'
    'main();\n'
)

CAP_PROMISE_ALL_SETTLED_TEST = (
    'async function browserPromiseAllSettled() {\n'
    "  const settled = await Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedSettled = await Promise["allSettled"]([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleMixedSettled = await Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const dottedSettled = await globalThis.Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedDottedSettled = await globalThis.Promise["allSettled"]([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleDottedSettled = await globalThis.Promise['allSettled']([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedBracketedSettled = await globalThis["Promise"].allSettled([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const bracketedSettled = await globalThis["Promise"]["allSettled"]([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleBracketedSettled = await globalThis['Promise']['allSettled']([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const singleMixedBracketedSettled = await globalThis['Promise'].allSettled([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const nullishRootSettled = await Object.freeze((null ?? Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalAndRootSettled = await Object.freeze((true && Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalOrRootSettled = await Object.freeze((false || Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const nullishDottedSettled = await Object.freeze((null ?? globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalAndDottedSettled = await Object.freeze((true && globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const logicalOrDottedSettled = await Object.freeze((false || globalThis.Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const wrappedDottedRootFrozenSettled = await Object.freeze((globalThis.Promise)["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const wrappedBracketedRootFrozenSettled = await Object.freeze((globalThis["Promise"])["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const wrappedBracketedDotRootFrozenSettled = await Object.freeze((globalThis["Promise"]).allSettled)([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const wrappedSingleBracketedDotRootFrozenSettled = await Object.freeze((globalThis['Promise']).allSettled)([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const frozenBracketedSettled = await Object.freeze(globalThis["Promise"]["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedFrozenBracketedSettled = await Object.freeze((globalThis["Promise"]["allSettled"]))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleFrozenBracketedSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleFrozenBracketedSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedBracketedRootFrozenSettled = await Object.freeze(globalThis["Promise"].allSettled)([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedMixedBracketedRootFrozenSettled = await Object.freeze((globalThis["Promise"].allSettled))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleMixedBracketedRootFrozenSettled = await Object.freeze(globalThis['Promise'].allSettled)([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const fullyBracketedSingleRootFrozenSettled = await Object.freeze(globalThis['Promise']['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedFullyBracketedSingleRootFrozenSettled = await Object.freeze((globalThis['Promise']['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleMixedBracketedRootFrozenSettled = await Object.freeze((globalThis['Promise'].allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const mixedRootFrozenSettled = await Object.freeze(globalThis.Promise["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedMixedRootFrozenSettled = await Object.freeze((globalThis.Promise["allSettled"]))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleMixedRootFrozenSettled = await Object.freeze(globalThis.Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleMixedRootFrozenSettled = await Object.freeze((globalThis.Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  const bracketedRootFrozenSettled = await Object.freeze(Promise["allSettled"])([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    '  const parenthesizedBracketedRootFrozenSettled = await Object.freeze((Promise["allSettled"]))([Promise.resolve(1), Promise.reject(\'boom\')]);\n'
    "  const singleBracketedRootFrozenSettled = await Object.freeze(Promise['allSettled'])([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedSingleBracketedRootFrozenSettled = await Object.freeze((Promise['allSettled']))([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const rootFrozenSettled = await Object.freeze(Promise.allSettled)([Promise.resolve(1), Promise.reject('boom')]);\n"
    "  const parenthesizedRootFrozenSettled = await Object.freeze((Promise.allSettled))([Promise.resolve(1), Promise.reject('boom')]);\n"
    '  if (\n'
    '    settled.length !== 2 ||\n'
    "    settled[0].status !== 'fulfilled' ||\n"
    '    settled[0].value !== 1 ||\n'
    "    settled[1].status !== 'rejected' ||\n"
    "    settled[1].reason !== 'boom' ||\n"
    '    mixedSettled.length !== 2 ||\n'
    "    mixedSettled[0].status !== 'fulfilled' ||\n"
    '    mixedSettled[0].value !== 1 ||\n'
    "    mixedSettled[1].status !== 'rejected' ||\n"
    "    mixedSettled[1].reason !== 'boom' ||\n"
    '    dottedSettled.length !== 2 ||\n'
    "    dottedSettled[0].status !== 'fulfilled' ||\n"
    '    dottedSettled[0].value !== 1 ||\n'
    "    dottedSettled[1].status !== 'rejected' ||\n"
    "    dottedSettled[1].reason !== 'boom' ||\n"
    '    mixedDottedSettled.length !== 2 ||\n'
    "    mixedDottedSettled[0].status !== 'fulfilled' ||\n"
    '    mixedDottedSettled[0].value !== 1 ||\n'
    "    mixedDottedSettled[1].status !== 'rejected' ||\n"
    "    mixedDottedSettled[1].reason !== 'boom' ||\n"
    '    mixedBracketedSettled.length !== 2 ||\n'
    "    mixedBracketedSettled[0].status !== 'fulfilled' ||\n"
    '    mixedBracketedSettled[0].value !== 1 ||\n'
    "    mixedBracketedSettled[1].status !== 'rejected' ||\n"
    "    mixedBracketedSettled[1].reason !== 'boom' ||\n"
    '    bracketedSettled.length !== 2 ||\n'
    "    bracketedSettled[0].status !== 'fulfilled' ||\n"
    '    bracketedSettled[0].value !== 1 ||\n'
    "    bracketedSettled[1].status !== 'rejected' ||\n"
    "    bracketedSettled[1].reason !== 'boom' ||\n"
    '    nullishRootSettled.length !== 2 ||\n'
    "    nullishRootSettled[0].status !== 'fulfilled' ||\n"
    '    nullishRootSettled[0].value !== 1 ||\n'
    "    nullishRootSettled[1].status !== 'rejected' ||\n"
    "    nullishRootSettled[1].reason !== 'boom' ||\n"
    '    logicalAndRootSettled.length !== 2 ||\n'
    "    logicalAndRootSettled[0].status !== 'fulfilled' ||\n"
    '    logicalAndRootSettled[0].value !== 1 ||\n'
    "    logicalAndRootSettled[1].status !== 'rejected' ||\n"
    "    logicalAndRootSettled[1].reason !== 'boom' ||\n"
    '    logicalOrRootSettled.length !== 2 ||\n'
    "    logicalOrRootSettled[0].status !== 'fulfilled' ||\n"
    '    logicalOrRootSettled[0].value !== 1 ||\n'
    "    logicalOrRootSettled[1].status !== 'rejected' ||\n"
    "    logicalOrRootSettled[1].reason !== 'boom' ||\n"
    '    nullishDottedSettled.length !== 2 ||\n'
    "    nullishDottedSettled[0].status !== 'fulfilled' ||\n"
    '    nullishDottedSettled[0].value !== 1 ||\n'
    "    nullishDottedSettled[1].status !== 'rejected' ||\n"
    "    nullishDottedSettled[1].reason !== 'boom' ||\n"
    '    logicalAndDottedSettled.length !== 2 ||\n'
    "    logicalAndDottedSettled[0].status !== 'fulfilled' ||\n"
    '    logicalAndDottedSettled[0].value !== 1 ||\n'
    "    logicalAndDottedSettled[1].status !== 'rejected' ||\n"
    "    logicalAndDottedSettled[1].reason !== 'boom' ||\n"
    '    logicalOrDottedSettled.length !== 2 ||\n'
    "    logicalOrDottedSettled[0].status !== 'fulfilled' ||\n"
    '    logicalOrDottedSettled[0].value !== 1 ||\n'
    "    logicalOrDottedSettled[1].status !== 'rejected' ||\n"
    "    logicalOrDottedSettled[1].reason !== 'boom' ||\n"
    '    wrappedBracketedDotRootFrozenSettled.length !== 2 ||\n'
    "    wrappedBracketedDotRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    wrappedBracketedDotRootFrozenSettled[0].value !== 1 ||\n'
    "    wrappedBracketedDotRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    wrappedBracketedDotRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    wrappedSingleBracketedDotRootFrozenSettled.length !== 2 ||\n'
    "    wrappedSingleBracketedDotRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    wrappedSingleBracketedDotRootFrozenSettled[0].value !== 1 ||\n'
    "    wrappedSingleBracketedDotRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    wrappedSingleBracketedDotRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    frozenBracketedSettled.length !== 2 ||\n'
    "    frozenBracketedSettled[0].status !== 'fulfilled' ||\n"
    '    frozenBracketedSettled[0].value !== 1 ||\n'
    "    frozenBracketedSettled[1].status !== 'rejected' ||\n"
    "    frozenBracketedSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedFrozenBracketedSettled.length !== 2 ||\n'
    "    parenthesizedFrozenBracketedSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedFrozenBracketedSettled[0].value !== 1 ||\n'
    "    parenthesizedFrozenBracketedSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedFrozenBracketedSettled[1].reason !== 'boom' ||\n"
    '    mixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    mixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    mixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    mixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    mixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedMixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedMixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    singleMixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    singleMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    singleMixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    singleMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    singleMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    fullyBracketedSingleRootFrozenSettled.length !== 2 ||\n'
    "    fullyBracketedSingleRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    fullyBracketedSingleRootFrozenSettled[0].value !== 1 ||\n'
    "    fullyBracketedSingleRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    fullyBracketedSingleRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedFullyBracketedSingleRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedFullyBracketedSingleRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedFullyBracketedSingleRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedFullyBracketedSingleRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedFullyBracketedSingleRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedSingleMixedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedSingleMixedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedSingleMixedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedSingleMixedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedSingleMixedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    mixedRootFrozenSettled.length !== 2 ||\n'
    "    mixedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    mixedRootFrozenSettled[0].value !== 1 ||\n'
    "    mixedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    mixedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedMixedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedMixedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedMixedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedMixedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedMixedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    bracketedRootFrozenSettled.length !== 2 ||\n'
    "    bracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    bracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    bracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    bracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedBracketedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedBracketedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedBracketedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedBracketedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedBracketedRootFrozenSettled[1].reason !== 'boom' ||\n"
    '    rootFrozenSettled.length !== 2 ||\n'
    "    rootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    rootFrozenSettled[0].value !== 1 ||\n'
    "    rootFrozenSettled[1].status !== 'rejected' ||\n"
    "    rootFrozenSettled[1].reason !== 'boom' ||\n"
    '    parenthesizedRootFrozenSettled.length !== 2 ||\n'
    "    parenthesizedRootFrozenSettled[0].status !== 'fulfilled' ||\n"
    '    parenthesizedRootFrozenSettled[0].value !== 1 ||\n'
    "    parenthesizedRootFrozenSettled[1].status !== 'rejected' ||\n"
    "    parenthesizedRootFrozenSettled[1].reason !== 'boom'\n"
    '  ) {\n'
    "    throw new Error('unexpected Promise.allSettled semantics');\n"
    '  }\n'
    '\n'
    '}\n'
    '\n'
    "Kali.test('browser promise allSettled', () => browserPromiseAllSettled());\n"
)

CAP_PROMISE_RACE_BUNDLE = (
    '// kali-tree-shake: browserPromiseRace\n'
    'async function browserPromiseRace() {\n'
    '  const direct = await Promise.race([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixed = await Promise["race"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixed = await Promise['race']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const dotted = await globalThis.Promise.race([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const bracketed = await globalThis["Promise"].race([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleBracketed = await globalThis['Promise'].race([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const mixedDotted = await globalThis.Promise["race"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleDotted = await globalThis.Promise['race']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const bracketedBracketed = await globalThis["Promise"]["race"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleBracketedBracketed = await globalThis['Promise']['race']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const parenthesizedBracketed = await Object.freeze((globalThis["Promise"])["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const parenthesizedSingleBracketed = await Object.freeze((globalThis['Promise'])['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const parenthesizedDottedBracketed = await Object.freeze((globalThis.Promise)["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const parenthesizedSingleDottedBracketed = await Object.freeze((globalThis.Promise)['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const parenthesizedBracketedBracketed = await Object.freeze((globalThis["Promise"]["race"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const parenthesizedSingleBracketedBracketed = await Object.freeze((globalThis['Promise']['race']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenRoot = await Object.freeze(Promise.race)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenRoot = await Object.freeze((Promise.race))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const frozenBracketed = await Object.freeze(globalThis["Promise"].race)([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleBracketed = await Object.freeze(globalThis['Promise'].race)([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenDottedBracketed = await Object.freeze(globalThis.Promise["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleDottedBracketed = await Object.freeze(globalThis.Promise['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenBracketedBracketed = await Object.freeze(globalThis["Promise"]["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleBracketedBracketed = await Object.freeze(globalThis['Promise']['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenDotted = await Object.freeze(globalThis.Promise.race)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenDotted = await Object.freeze((globalThis.Promise.race))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  if (\n'
    '    direct !== 1 ||\n'
    '    mixed !== 1 ||\n'
    '    singleMixed !== 1 ||\n'
    '    dotted !== 1 ||\n'
    '    bracketed !== 1 ||\n'
    '    singleBracketed !== 1 ||\n'
    '    mixedDotted !== 1 ||\n'
    '    singleDotted !== 1 ||\n'
    '    bracketedBracketed !== 1 ||\n'
    '    singleBracketedBracketed !== 1 ||\n'
    '    parenthesizedBracketed !== 1 ||\n'
    '    parenthesizedSingleBracketed !== 1 ||\n'
    '    parenthesizedDottedBracketed !== 1 ||\n'
    '    parenthesizedSingleDottedBracketed !== 1 ||\n'
    '    parenthesizedBracketedBracketed !== 1 ||\n'
    '    parenthesizedSingleBracketedBracketed !== 1 ||\n'
    '    frozenRoot !== 1 ||\n'
    '    parenthesizedFrozenRoot !== 1 ||\n'
    '    frozenBracketed !== 1 ||\n'
    '    frozenSingleBracketed !== 1 ||\n'
    '    frozenDottedBracketed !== 1 ||\n'
    '    frozenSingleDottedBracketed !== 1 ||\n'
    '    frozenBracketedBracketed !== 1 ||\n'
    '    frozenSingleBracketedBracketed !== 1 ||\n'
    '    frozenDotted !== 1 ||\n'
    '    parenthesizedFrozenDotted !== 1\n'
    '  ) {\n'
    "    throw new Error('unexpected Promise.race semantics');\n"
    '  }\n'
    '\n'
    '}\n'
)

CAP_PROMISE_RACE_RUN = (
    'async function browserPromiseRace() {\n'
    '  const direct = await Promise.race([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixed = await Promise["race"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixed = await Promise['race']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const dotted = await globalThis.Promise.race([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const bracketed = await globalThis["Promise"].race([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleBracketed = await globalThis['Promise'].race([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const mixedDotted = await globalThis.Promise["race"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleDotted = await globalThis.Promise['race']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const bracketedBracketed = await globalThis["Promise"]["race"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleBracketedBracketed = await globalThis['Promise']['race']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const parenthesizedBracketed = await Object.freeze((globalThis["Promise"])["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const parenthesizedSingleBracketed = await Object.freeze((globalThis['Promise'])['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const parenthesizedDottedBracketed = await Object.freeze((globalThis.Promise)["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const parenthesizedSingleDottedBracketed = await Object.freeze((globalThis.Promise)['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const parenthesizedBracketedBracketed = await Object.freeze((globalThis["Promise"]["race"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const parenthesizedSingleBracketedBracketed = await Object.freeze((globalThis['Promise']['race']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenRoot = await Object.freeze(Promise.race)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenRoot = await Object.freeze((Promise.race))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const frozenBracketed = await Object.freeze(globalThis["Promise"].race)([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleBracketed = await Object.freeze(globalThis['Promise'].race)([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenDottedBracketed = await Object.freeze(globalThis.Promise["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleDottedBracketed = await Object.freeze(globalThis.Promise['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenBracketedBracketed = await Object.freeze(globalThis["Promise"]["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleBracketedBracketed = await Object.freeze(globalThis['Promise']['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenDotted = await Object.freeze(globalThis.Promise.race)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenDotted = await Object.freeze((globalThis.Promise.race))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  if (\n'
    '    direct !== 1 ||\n'
    '    mixed !== 1 ||\n'
    '    singleMixed !== 1 ||\n'
    '    dotted !== 1 ||\n'
    '    bracketed !== 1 ||\n'
    '    singleBracketed !== 1 ||\n'
    '    mixedDotted !== 1 ||\n'
    '    singleDotted !== 1 ||\n'
    '    bracketedBracketed !== 1 ||\n'
    '    singleBracketedBracketed !== 1 ||\n'
    '    parenthesizedBracketed !== 1 ||\n'
    '    parenthesizedSingleBracketed !== 1 ||\n'
    '    parenthesizedDottedBracketed !== 1 ||\n'
    '    parenthesizedSingleDottedBracketed !== 1 ||\n'
    '    parenthesizedBracketedBracketed !== 1 ||\n'
    '    parenthesizedSingleBracketedBracketed !== 1 ||\n'
    '    frozenRoot !== 1 ||\n'
    '    parenthesizedFrozenRoot !== 1 ||\n'
    '    frozenBracketed !== 1 ||\n'
    '    frozenSingleBracketed !== 1 ||\n'
    '    frozenDottedBracketed !== 1 ||\n'
    '    frozenSingleDottedBracketed !== 1 ||\n'
    '    frozenBracketedBracketed !== 1 ||\n'
    '    frozenSingleBracketedBracketed !== 1 ||\n'
    '    frozenDotted !== 1 ||\n'
    '    parenthesizedFrozenDotted !== 1\n'
    '  ) {\n'
    "    throw new Error('unexpected Promise.race semantics');\n"
    '  }\n'
    '\n'
    '}\n'
    '\n'
    'async function main() {\n'
    '  await browserPromiseRace();\n'
    "  console.log('browser promise race ok');\n"
    '}\n'
    '\n'
    'main();\n'
)

CAP_PROMISE_RACE_TEST = (
    'async function browserPromiseRace() {\n'
    '  const direct = await Promise.race([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const mixed = await Promise["race"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleMixed = await Promise['race']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const dotted = await globalThis.Promise.race([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const bracketed = await globalThis["Promise"].race([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleBracketed = await globalThis['Promise'].race([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const mixedDotted = await globalThis.Promise["race"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleDotted = await globalThis.Promise['race']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const bracketedBracketed = await globalThis["Promise"]["race"]([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const singleBracketedBracketed = await globalThis['Promise']['race']([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const parenthesizedBracketed = await Object.freeze((globalThis["Promise"])["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const parenthesizedSingleBracketed = await Object.freeze((globalThis['Promise'])['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const parenthesizedDottedBracketed = await Object.freeze((globalThis.Promise)["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const parenthesizedSingleDottedBracketed = await Object.freeze((globalThis.Promise)['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const parenthesizedBracketedBracketed = await Object.freeze((globalThis["Promise"]["race"]))([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const parenthesizedSingleBracketedBracketed = await Object.freeze((globalThis['Promise']['race']))([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenRoot = await Object.freeze(Promise.race)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenRoot = await Object.freeze((Promise.race))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const frozenBracketed = await Object.freeze(globalThis["Promise"].race)([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleBracketed = await Object.freeze(globalThis['Promise'].race)([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenDottedBracketed = await Object.freeze(globalThis.Promise["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleDottedBracketed = await Object.freeze(globalThis.Promise['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenBracketedBracketed = await Object.freeze(globalThis["Promise"]["race"])([Promise.resolve(1), Promise.resolve(2)]);\n'
    "  const frozenSingleBracketedBracketed = await Object.freeze(globalThis['Promise']['race'])([Promise.resolve(1), Promise.resolve(2)]);\n"
    '  const frozenDotted = await Object.freeze(globalThis.Promise.race)([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  const parenthesizedFrozenDotted = await Object.freeze((globalThis.Promise.race))([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  if (\n'
    '    direct !== 1 ||\n'
    '    mixed !== 1 ||\n'
    '    singleMixed !== 1 ||\n'
    '    dotted !== 1 ||\n'
    '    bracketed !== 1 ||\n'
    '    singleBracketed !== 1 ||\n'
    '    mixedDotted !== 1 ||\n'
    '    singleDotted !== 1 ||\n'
    '    bracketedBracketed !== 1 ||\n'
    '    singleBracketedBracketed !== 1 ||\n'
    '    parenthesizedBracketed !== 1 ||\n'
    '    parenthesizedSingleBracketed !== 1 ||\n'
    '    parenthesizedDottedBracketed !== 1 ||\n'
    '    parenthesizedSingleDottedBracketed !== 1 ||\n'
    '    parenthesizedBracketedBracketed !== 1 ||\n'
    '    parenthesizedSingleBracketedBracketed !== 1 ||\n'
    '    frozenRoot !== 1 ||\n'
    '    parenthesizedFrozenRoot !== 1 ||\n'
    '    frozenBracketed !== 1 ||\n'
    '    frozenSingleBracketed !== 1 ||\n'
    '    frozenDottedBracketed !== 1 ||\n'
    '    frozenSingleDottedBracketed !== 1 ||\n'
    '    frozenBracketedBracketed !== 1 ||\n'
    '    frozenSingleBracketedBracketed !== 1 ||\n'
    '    frozenDotted !== 1 ||\n'
    '    parenthesizedFrozenDotted !== 1\n'
    '  ) {\n'
    "    throw new Error('unexpected Promise.race semantics');\n"
    '  }\n'
    '\n'
    '}\n'
    '\n'
    "Kali.test('browser promise race', () => browserPromiseRace());\n"
)

CAP_STRING_CONCAT_BUNDLE = (
    '// kali-tree-shake: browserStringConcatenation\n'
    'export async function browserStringConcatenation() {\n'
    '  const prefix = "he";\n'
    '  const suffix = "llo";\n'
    '  const syncChars = [];\n'
    '  for (const item of prefix + suffix) {\n'
    '    syncChars.push(item);\n'
    '  }\n'
    '  const asyncChars = [];\n'
    '  for await (const item of prefix + suffix) {\n'
    '    asyncChars.push(item);\n'
    '  }\n'
    '  if (syncChars.join("") !== "hello" || asyncChars.join("") !== "hello") {\n'
    "    throw new Error('unexpected string concatenation iteration semantics');\n"
    '  }\n'
    "  console.log('browser string concatenation ok');\n"
    '}\n'
)

CAP_STRING_CONCAT_RUN = (
    'const prefix = "he";\n'
    'const suffix = "llo";\n'
    'const syncChars = [];\n'
    'for (const item of prefix + suffix) {\n'
    '  syncChars.push(item);\n'
    '}\n'
    'const asyncChars = [];\n'
    'for await (const item of prefix + suffix) {\n'
    '  asyncChars.push(item);\n'
    '}\n'
    'if (syncChars.join("") !== "hello" || asyncChars.join("") !== "hello") {\n'
    "  throw new Error('unexpected string concatenation iteration semantics');\n"
    '}\n'
    "console.log('browser string concatenation ok');\n"
)

CAP_STRING_CONCAT_TEST = (
    "Kali.test('browser string concatenation', () => {\n"
    'const prefix = "he";\n'
    'const suffix = "llo";\n'
    'const syncChars = [];\n'
    'for (const item of prefix + suffix) {\n'
    '  syncChars.push(item);\n'
    '}\n'
    'const asyncChars = [];\n'
    'for await (const item of prefix + suffix) {\n'
    '  asyncChars.push(item);\n'
    '}\n'
    'if (syncChars.join("") !== "hello" || asyncChars.join("") !== "hello") {\n'
    "  throw new Error('unexpected string concatenation iteration semantics');\n"
    '}\n'
    '});\n'
)

CAP_TLSI_BUNDLE = (
    '// kali-tree-shake: browserTemplateLiteralStringIteration\n'
    'export async function browserTemplateLiteralStringIteration() {\n'
    '  const prefix = "he";\n'
    '  const suffix = "llo";\n'
    '  const syncChars = [];\n'
    '  for (const item of `${prefix}${suffix}`) {\n'
    '    syncChars.push(item);\n'
    '  }\n'
    '  const asyncChars = [];\n'
    '  for await (const item of `${prefix}${suffix}`) {\n'
    '    asyncChars.push(item);\n'
    '  }\n'
    '  if (syncChars.join("") !== "hello" || asyncChars.join("") !== "hello") {\n'
    "    throw new Error('unexpected template literal iteration semantics');\n"
    '  }\n'
    "  console.log('browser template literal iteration ok');\n"
    '}\n'
)

CAP_TLSI_RUN = (
    '  const prefix = "he";\n'
    '  const suffix = "llo";\n'
    '  const syncChars = [];\n'
    '  for (const item of `${prefix}${suffix}`) {\n'
    '    syncChars.push(item);\n'
    '  }\n'
    '  const asyncChars = [];\n'
    '  for await (const item of `${prefix}${suffix}`) {\n'
    '    asyncChars.push(item);\n'
    '  }\n'
    '  if (syncChars.join("") !== "hello" || asyncChars.join("") !== "hello") {\n'
    "    throw new Error('unexpected template literal iteration semantics');\n"
    '  }\n'
    "console.log('browser template literal iteration ok');\n"
)

CAP_TLSI_TEST = (
    "Kali.test('browser template literal iteration', () => {\n"
    '  const prefix = "he";\n'
    '  const suffix = "llo";\n'
    '  const syncChars = [];\n'
    '  for (const item of `${prefix}${suffix}`) {\n'
    '    syncChars.push(item);\n'
    '  }\n'
    '  const asyncChars = [];\n'
    '  for await (const item of `${prefix}${suffix}`) {\n'
    '    asyncChars.push(item);\n'
    '  }\n'
    '  if (syncChars.join("") !== "hello" || asyncChars.join("") !== "hello") {\n'
    "    throw new Error('unexpected template literal iteration semantics');\n"
    '  }\n'
    '});\n'
)

CAP_SET_ITERATION_RUN = (
    'function browserSetIteration() {\n'
    '  function assertSetIteration(values) {\n'
    '    if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {\n'
    "      throw new Error('unexpected Set constructor iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const values = [1, 2, 1];\n'
    '  const setAlias = Set;\n'
    '  const wrappedSetAlias = (setAlias);\n'
    '  const aliasValues = (values);\n'
    '  const direct = [];\n'
    '  for (const value of new Set(values)) {\n'
    '    direct.push(value);\n'
    '  }\n'
    '  const alias = [];\n'
    '  for (const value of new setAlias(aliasValues)) {\n'
    '    alias.push(value);\n'
    '  }\n'
    '  const wrappedAlias = [];\n'
    '  for (const value of new (wrappedSetAlias)(aliasValues)) {\n'
    '    wrappedAlias.push(value);\n'
    '  }\n'
    '  const globalDirect = [];\n'
    '  for (const value of new globalThis.Set(values)) {\n'
    '    globalDirect.push(value);\n'
    '  }\n'
    '  const parenthesizedBracketed = [];\n'
    '  for (const value of new (globalThis["Set"])(values)) {\n'
    '    parenthesizedBracketed.push(value);\n'
    '  }\n'
    '  const parenthesizedSingleBracketed = [];\n'
    "  for (const value of new (globalThis['Set'])(values)) {\n"
    '    parenthesizedSingleBracketed.push(value);\n'
    '  }\n'
    '  const bracketed = [];\n'
    '  for (const value of new globalThis["Set"](values)) {\n'
    '    bracketed.push(value);\n'
    '  }\n'
    '  const singleBracketed = [];\n'
    "  for (const value of new globalThis['Set'](values)) {\n"
    '    singleBracketed.push(value);\n'
    '  }\n'
    '  const nullishValues = [];\n'
    '  for (const value of new (null ?? Set)(aliasValues)) {\n'
    '    nullishValues.push(value);\n'
    '  }\n'
    '  const logicalOrValues = [];\n'
    '  for (const value of new (false || Set)(aliasValues)) {\n'
    '    logicalOrValues.push(value);\n'
    '  }\n'
    '  assertSetIteration(nullishValues);\n'
    '  assertSetIteration(logicalOrValues);\n'
    '  const frozenValues = Object.freeze(aliasValues);\n'
    '  const frozenSet = Object.freeze(Set);\n'
    '  const frozenGlobalThisSet = Object.freeze(globalThis.Set);\n'
    '  const frozenGlobalThisBracketedSet = Object.freeze(globalThis["Set"]);\n'
    '  const wrappedFrozenSet = Object.freeze((Set));\n'
    '  const wrappedFrozenGlobalThisSet = Object.freeze((globalThis.Set));\n'
    '  const wrappedFrozenGlobalThisBracketedSet = Object.freeze((globalThis["Set"]));\n'
    '  const frozenDirect = [];\n'
    '  for (const value of new Set(frozenValues)) {\n'
    '    frozenDirect.push(value);\n'
    '  }\n'
    '  const frozenAlias = [];\n'
    '  for (const value of new (frozenSet)(values)) {\n'
    '    frozenAlias.push(value);\n'
    '  }\n'
    '  const frozenGlobalDirect = [];\n'
    '  for (const value of new (frozenGlobalThisSet)(values)) {\n'
    '    frozenGlobalDirect.push(value);\n'
    '  }\n'
    '  const frozenGlobalBracketed = [];\n'
    '  for (const value of new (frozenGlobalThisBracketedSet)(values)) {\n'
    '    frozenGlobalBracketed.push(value);\n'
    '  }\n'
    '  const wrappedFrozenDirect = [];\n'
    '  for (const value of new (wrappedFrozenSet)(values)) {\n'
    '    wrappedFrozenDirect.push(value);\n'
    '  }\n'
    '  const wrappedFrozenGlobalDirect = [];\n'
    '  for (const value of new (wrappedFrozenGlobalThisSet)(values)) {\n'
    '    wrappedFrozenGlobalDirect.push(value);\n'
    '  }\n'
    '  const wrappedFrozenGlobalBracketed = [];\n'
    '  for (const value of new (wrappedFrozenGlobalThisBracketedSet)(values)) {\n'
    '    wrappedFrozenGlobalBracketed.push(value);\n'
    '  }\n'
    '\n'
    '  let returnFinally = false;\n'
    '  function setReturnProbe() {\n'
    '    try {\n'
    '      for (const value of new Set(values)) {\n'
    '        return value;\n'
    '      }\n'
    "      throw new Error('unexpected empty Set constructor iteration');\n"
    '    } finally {\n'
    '      returnFinally = true;\n'
    '    }\n'
    '  }\n'
    '  const returnValue = setReturnProbe();\n'
    '  if (returnValue !== 1 || !returnFinally) {\n'
    "    throw new Error('unexpected Set constructor return/finally semantics');\n"
    '  }\n'
    '\n'
    '  let throwFinally = false;\n'
    '  function setThrowProbe() {\n'
    '    try {\n'
    '      for (const value of new Set(values)) {\n'
    '        if (value === 1) {\n'
    "          throw new Error('boom');\n"
    '        }\n'
    '      }\n'
    "      throw new Error('unexpected empty Set constructor iteration');\n"
    '    } finally {\n'
    '      throwFinally = true;\n'
    '    }\n'
    '  }\n'
    '  let threw = false;\n'
    '  try {\n'
    '    setThrowProbe();\n'
    '  } catch {\n'
    '    threw = true;\n'
    '  }\n'
    '  if (!threw || !throwFinally) {\n'
    "    throw new Error('unexpected Set constructor throw/finally semantics');\n"
    '  }\n'
    '\n'
    '  assertSetIteration(direct);\n'
    '  assertSetIteration(alias);\n'
    '  assertSetIteration(wrappedAlias);\n'
    '  assertSetIteration(globalDirect);\n'
    '  assertSetIteration(parenthesizedBracketed);\n'
    '  assertSetIteration(parenthesizedSingleBracketed);\n'
    '  assertSetIteration(bracketed);\n'
    '  assertSetIteration(singleBracketed);\n'
    '  assertSetIteration(frozenDirect);\n'
    '  assertSetIteration(frozenAlias);\n'
    '  assertSetIteration(frozenGlobalDirect);\n'
    '  assertSetIteration(frozenGlobalBracketed);\n'
    '  assertSetIteration(wrappedFrozenDirect);\n'
    '  assertSetIteration(wrappedFrozenGlobalDirect);\n'
    '  assertSetIteration(wrappedFrozenGlobalBracketed);\n'
    "  console.log('browser set constructor iteration ok');\n"
    '}\n'
    '\n'
    'browserSetIteration();\n'
)

CAP_SET_ITERATION_TEST = (
    'function setConstructorIterationCheck() {\n'
    '  function assertSetIteration(values) {\n'
    '    if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {\n'
    "      throw new Error('unexpected Set constructor iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const values = [1, 2, 1];\n'
    '  const setAlias = Set;\n'
    '  const wrappedSetAlias = (setAlias);\n'
    '  const aliasValues = (values);\n'
    '  const direct = [];\n'
    '  for (const value of new Set(values)) {\n'
    '    direct.push(value);\n'
    '  }\n'
    '  const alias = [];\n'
    '  for (const value of new setAlias(aliasValues)) {\n'
    '    alias.push(value);\n'
    '  }\n'
    '  const wrappedAlias = [];\n'
    '  for (const value of new (wrappedSetAlias)(aliasValues)) {\n'
    '    wrappedAlias.push(value);\n'
    '  }\n'
    '  const globalDirect = [];\n'
    '  for (const value of new globalThis.Set(values)) {\n'
    '    globalDirect.push(value);\n'
    '  }\n'
    '  const parenthesizedBracketed = [];\n'
    '  for (const value of new (globalThis["Set"])(values)) {\n'
    '    parenthesizedBracketed.push(value);\n'
    '  }\n'
    '  const parenthesizedSingleBracketed = [];\n'
    "  for (const value of new (globalThis['Set'])(values)) {\n"
    '    parenthesizedSingleBracketed.push(value);\n'
    '  }\n'
    '  const bracketed = [];\n'
    '  for (const value of new globalThis["Set"](values)) {\n'
    '    bracketed.push(value);\n'
    '  }\n'
    '  const singleBracketed = [];\n'
    "  for (const value of new globalThis['Set'](values)) {\n"
    '    singleBracketed.push(value);\n'
    '  }\n'
    '  const nullishValues = [];\n'
    '  for (const value of new (null ?? Set)(aliasValues)) {\n'
    '    nullishValues.push(value);\n'
    '  }\n'
    '  const logicalOrValues = [];\n'
    '  for (const value of new (false || Set)(aliasValues)) {\n'
    '    logicalOrValues.push(value);\n'
    '  }\n'
    '  assertSetIteration(nullishValues);\n'
    '  assertSetIteration(logicalOrValues);\n'
    '  const frozenValues = Object.freeze(aliasValues);\n'
    '  const frozenSet = Object.freeze(Set);\n'
    '  const frozenGlobalThisSet = Object.freeze(globalThis.Set);\n'
    '  const frozenGlobalThisBracketedSet = Object.freeze(globalThis["Set"]);\n'
    '  const wrappedFrozenSet = Object.freeze((Set));\n'
    '  const wrappedFrozenGlobalThisSet = Object.freeze((globalThis.Set));\n'
    '  const wrappedFrozenGlobalThisBracketedSet = Object.freeze((globalThis["Set"]));\n'
    '  const frozenDirect = [];\n'
    '  for (const value of new Set(frozenValues)) {\n'
    '    frozenDirect.push(value);\n'
    '  }\n'
    '  const frozenAlias = [];\n'
    '  for (const value of new (frozenSet)(values)) {\n'
    '    frozenAlias.push(value);\n'
    '  }\n'
    '  const frozenGlobalDirect = [];\n'
    '  for (const value of new (frozenGlobalThisSet)(values)) {\n'
    '    frozenGlobalDirect.push(value);\n'
    '  }\n'
    '  const frozenGlobalBracketed = [];\n'
    '  for (const value of new (frozenGlobalThisBracketedSet)(values)) {\n'
    '    frozenGlobalBracketed.push(value);\n'
    '  }\n'
    '  const wrappedFrozenDirect = [];\n'
    '  for (const value of new (wrappedFrozenSet)(values)) {\n'
    '    wrappedFrozenDirect.push(value);\n'
    '  }\n'
    '  const wrappedFrozenGlobalDirect = [];\n'
    '  for (const value of new (wrappedFrozenGlobalThisSet)(values)) {\n'
    '    wrappedFrozenGlobalDirect.push(value);\n'
    '  }\n'
    '  const wrappedFrozenGlobalBracketed = [];\n'
    '  for (const value of new (wrappedFrozenGlobalThisBracketedSet)(values)) {\n'
    '    wrappedFrozenGlobalBracketed.push(value);\n'
    '  }\n'
    '\n'
    '  let returnFinally = false;\n'
    '  function setReturnProbe() {\n'
    '    try {\n'
    '      for (const value of new Set(values)) {\n'
    '        return value;\n'
    '      }\n'
    "      throw new Error('unexpected empty Set constructor iteration');\n"
    '    } finally {\n'
    '      returnFinally = true;\n'
    '    }\n'
    '  }\n'
    '  const returnValue = setReturnProbe();\n'
    '  if (returnValue !== 1 || !returnFinally) {\n'
    "    throw new Error('unexpected Set constructor return/finally semantics');\n"
    '  }\n'
    '\n'
    '  let throwFinally = false;\n'
    '  function setThrowProbe() {\n'
    '    try {\n'
    '      for (const value of new Set(values)) {\n'
    '        if (value === 1) {\n'
    "          throw new Error('boom');\n"
    '        }\n'
    '      }\n'
    "      throw new Error('unexpected empty Set constructor iteration');\n"
    '    } finally {\n'
    '      throwFinally = true;\n'
    '    }\n'
    '  }\n'
    '  let threw = false;\n'
    '  try {\n'
    '    setThrowProbe();\n'
    '  } catch {\n'
    '    threw = true;\n'
    '  }\n'
    '  if (!threw || !throwFinally) {\n'
    "    throw new Error('unexpected Set constructor throw/finally semantics');\n"
    '  }\n'
    '\n'
    '  assertSetIteration(direct);\n'
    '  assertSetIteration(alias);\n'
    '  assertSetIteration(wrappedAlias);\n'
    '  assertSetIteration(globalDirect);\n'
    '  assertSetIteration(parenthesizedBracketed);\n'
    '  assertSetIteration(parenthesizedSingleBracketed);\n'
    '  assertSetIteration(bracketed);\n'
    '  assertSetIteration(singleBracketed);\n'
    '  assertSetIteration(frozenDirect);\n'
    '  assertSetIteration(frozenAlias);\n'
    '  assertSetIteration(frozenGlobalDirect);\n'
    '  assertSetIteration(frozenGlobalBracketed);\n'
    '  assertSetIteration(wrappedFrozenDirect);\n'
    '  assertSetIteration(wrappedFrozenGlobalDirect);\n'
    '  assertSetIteration(wrappedFrozenGlobalBracketed);\n'
    "  console.log('browser set constructor iteration ok');\n"
    '}\n'
    '\n'
    "Kali.test('set constructor iteration', () => {\n"
    '  setConstructorIterationCheck();\n'
    '});\n'
)

CAP_REFLECT_RUN = (
    'const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };\n'
    'const frozenObj = Object.freeze(obj);\n'
    'const keys = globalThis.Reflect.ownKeys(obj);\n'
    'const frozenKeys = globalThis.Reflect.ownKeys(frozenObj);\n'
    'const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);\n'
    'const mixedBracketedDirectKeys = globalThis["Reflect"][\'ownKeys\'](obj);\n'
    'const mixedSingleQuotedDirectKeys = globalThis[\'Reflect\']["ownKeys"](obj);\n'
    'const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);\n'
    "const singleQuotedPropertyKeys = globalThis['Reflect'].ownKeys(obj);\n"
    'const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);\n'
    'const fullyBracketedKeys = globalThis["Reflect"]["ownKeys"](obj);\n'
    'const parenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj);\n'
    'const frozenBracketRootKeys = Object.freeze((globalThis["Reflect"]))["ownKeys"](obj);\n'
    "const singleQuotedKeys = globalThis['Reflect']['ownKeys'](obj);\n"
    "const singleQuotedMixedBracketedKeys = globalThis.Reflect['ownKeys'](obj);\n"
    "const frozenSingleQuotedKeys = globalThis['Reflect']['ownKeys'](frozenObj);\n"
    "const parenthesizedFrozenSingleQuotedKeys = Object.freeze((globalThis['Reflect']['ownKeys']))(obj);\n"
    "const parenthesizedFrozenSingleQuotedFrozenKeys = Object.freeze((globalThis['Reflect']['ownKeys']))(frozenObj);\n"
    'const frozenBareCallableKeys = Object.freeze(Reflect.ownKeys)(obj); const parenthesizedFrozenBareCallableKeys = Object.freeze((Reflect.ownKeys))(obj); const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)(obj); const mixedBracketedRootKeys = Object.freeze(globalThis["Reflect"][\'ownKeys\'])(obj); const parenthesizedMixedBracketedRootKeys = Object.freeze((globalThis["Reflect"][\'ownKeys\']))(obj); const mixedSingleQuotedRootKeys = Object.freeze(globalThis[\'Reflect\']["ownKeys"])(obj); const parenthesizedMixedSingleQuotedRootKeys = Object.freeze((globalThis[\'Reflect\']["ownKeys"]))(obj); const frozenMixedBracketedKeys = Object.freeze(globalThis.Reflect["ownKeys"])(obj); const frozenSingleQuotedMixedBracketedKeys = Object.freeze(globalThis.Reflect[\'ownKeys\'])(obj); const parenthesizedFrozenSingleQuotedMixedBracketedKeys = Object.freeze((globalThis.Reflect[\'ownKeys\']))(obj); const nullishFrozenCallableKeys = Object.freeze((null ?? globalThis.Reflect.ownKeys))(obj); const logicalAndFrozenCallableKeys = Object.freeze((true && globalThis.Reflect.ownKeys))(obj); const logicalOrFrozenCallableKeys = Object.freeze((false || globalThis.Reflect.ownKeys))(obj); const frozenMixedRootKeys = Object.freeze(globalThis["Reflect"].ownKeys)(obj); const parenthesizedFrozenDotRootBracketedKeys = Object.freeze((globalThis.Reflect)["ownKeys"])(obj); const frozenParenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"]).ownKeys)(obj); const frozenParenthesizedSingleQuotedBracketRootKeys = Object.freeze((globalThis[\'Reflect\']).ownKeys)(obj); const frozenParenthesizedBracketedRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj); const frozenParenthesizedMixedRootKeys = Object.freeze((globalThis["Reflect"].ownKeys))(obj); const frozenBracketedKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])(obj); const parenthesizedFrozenMixedBracketedKeys = Object.freeze((globalThis.Reflect["ownKeys"]))(obj); const parenthesizedFrozenBracketedKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))(obj); const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))(obj); const frozenSingleQuotedRootKeys = Object.freeze(globalThis[\'Reflect\'].ownKeys)(obj); const nullishFrozenBracketedKeys = Object.freeze((null ?? globalThis["Reflect"]["ownKeys"]))(obj); const logicalAndFrozenBracketedKeys = Object.freeze((true && globalThis["Reflect"]["ownKeys"]))(obj); const logicalOrFrozenBracketedKeys = Object.freeze((false || globalThis["Reflect"]["ownKeys"]))(obj); const frozenParenthesizedSingleQuotedRootKeys = Object.freeze((globalThis[\'Reflect\']).ownKeys)(obj); const frozenParenthesizedSingleQuotedBracketedKeys = Object.freeze((globalThis[\'Reflect\'])[\'ownKeys\'])(obj); const frozenSingleQuotedBracketedKeys = Object.freeze(globalThis[\'Reflect\'][\'ownKeys\'])(obj); const parenthesizedFrozenSingleQuotedRootKeys = Object.freeze((globalThis[\'Reflect\'].ownKeys))(obj); const parenthesizedFrozenSingleQuotedBracketedKeys = Object.freeze((globalThis[\'Reflect\'][\'ownKeys\']))(obj); const frozenNullishCallableKeys = Object.freeze((null ?? Reflect.ownKeys))(obj); const frozenLogicalAndCallableKeys = Object.freeze((true && Reflect.ownKeys))(obj); const frozenLogicalOrCallableKeys = Object.freeze((false || Reflect.ownKeys))(obj); const conditionalFrozenCallableKeys = Object.freeze((true ? Reflect.ownKeys : Reflect.ownKeys))(obj); const conditionalFrozenGlobalCallableKeys = Object.freeze((true ? globalThis.Reflect.ownKeys : globalThis.Reflect.ownKeys))(obj);\n'
    'let syncCount = 0;\n'
    'for (const key of globalThis.Reflect.ownKeys(obj)) {\n'
    '  syncCount += 1;\n'
    '}\n'
    'let frozenSyncCount = 0;\n'
    'for (const key of globalThis.Reflect.ownKeys(frozenObj)) {\n'
    '  frozenSyncCount += 1;\n'
    '}\n'
    'let sequenceCount = 0;\n'
    'for (const key of (0, globalThis.Reflect.ownKeys(obj))) {\n'
    '  sequenceCount += 1;\n'
    '}\n'
    'let frozenSequenceCount = 0;\n'
    'for (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {\n'
    '  frozenSequenceCount += 1;\n'
    '}\n'
    'let asyncCount = 0;\n'
    'for await (const key of globalThis.Reflect.ownKeys(obj)) {\n'
    '  asyncCount += 1;\n'
    '}\n'
    'let frozenAsyncCount = 0;\n'
    'for await (const key of globalThis.Reflect.ownKeys(frozenObj)) {\n'
    '  frozenAsyncCount += 1;\n'
    '}\n'
    'let asyncSequenceCount = 0;\n'
    'for await (const key of (0, globalThis.Reflect.ownKeys(obj))) {\n'
    '  asyncSequenceCount += 1;\n'
    '}\n'
    'let frozenAsyncSequenceCount = 0;\n'
    'for await (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {\n'
    '  frozenAsyncSequenceCount += 1;\n'
    '}\n'
    'let breakContinueCount = 0;\n'
    'for (const key of globalThis.Reflect.ownKeys(obj)) {\n'
    "  if (key === '1') {\n"
    '    continue;\n'
    '  }\n'
    '  breakContinueCount += 1;\n'
    '  break;\n'
    '}\n'
    'if (\n'
    '  keys.length !== 4 ||\n'
    "  keys[0] !== '1' ||\n"
    "  keys[1] !== '2' ||\n"
    "  keys[2] !== 'b' ||\n"
    "  keys[3] !== 'a' ||\n"
    '  frozenKeys.length !== 4 ||\n'
    "  frozenKeys[0] !== '1' ||\n"
    "  frozenKeys[1] !== '2' ||\n"
    "  frozenKeys[2] !== 'b' ||\n"
    "  frozenKeys[3] !== 'a' ||\n"
    '  mixedRootKeys.length !== 4 ||\n'
    "  mixedRootKeys[0] !== '1' ||\n"
    "  mixedRootKeys[1] !== '2' ||\n"
    "  mixedRootKeys[2] !== 'b' ||\n"
    "  mixedRootKeys[3] !== 'a' ||\n"
    '  mixedBracketedDirectKeys.length !== 4 ||\n'
    "  mixedBracketedDirectKeys[0] !== '1' ||\n"
    "  mixedBracketedDirectKeys[1] !== '2' ||\n"
    "  mixedBracketedDirectKeys[2] !== 'b' ||\n"
    "  mixedBracketedDirectKeys[3] !== 'a' ||\n"
    '  mixedSingleQuotedDirectKeys.length !== 4 ||\n'
    "  mixedSingleQuotedDirectKeys[0] !== '1' ||\n"
    "  mixedSingleQuotedDirectKeys[1] !== '2' ||\n"
    "  mixedSingleQuotedDirectKeys[2] !== 'b' ||\n"
    "  mixedSingleQuotedDirectKeys[3] !== 'a' ||\n"
    '  mixedBracketedKeys.length !== 4 ||\n'
    "  mixedBracketedKeys[0] !== '1' ||\n"
    "  mixedBracketedKeys[1] !== '2' ||\n"
    "  mixedBracketedKeys[2] !== 'b' ||\n"
    "  mixedBracketedKeys[3] !== 'a' ||\n"
    '  singleQuotedPropertyKeys.length !== 4 ||\n'
    "  singleQuotedPropertyKeys[0] !== '1' ||\n"
    "  singleQuotedPropertyKeys[1] !== '2' ||\n"
    "  singleQuotedPropertyKeys[2] !== 'b' ||\n"
    "  singleQuotedPropertyKeys[3] !== 'a' ||\n"
    '  bracketedKeys.length !== 4 ||\n'
    "  bracketedKeys[0] !== '1' ||\n"
    "  bracketedKeys[1] !== '2' ||\n"
    "  bracketedKeys[2] !== 'b' ||\n"
    "  bracketedKeys[3] !== 'a' ||\n"
    '  fullyBracketedKeys.length !== 4 ||\n'
    "  fullyBracketedKeys[0] !== '1' ||\n"
    "  fullyBracketedKeys[1] !== '2' ||\n"
    "  fullyBracketedKeys[2] !== 'b' ||\n"
    "  fullyBracketedKeys[3] !== 'a' ||\n"
    '    parenthesizedBracketRootKeys.length !== 4 ||\n'
    "    parenthesizedBracketRootKeys[0] !== '1' ||\n"
    "    parenthesizedBracketRootKeys[1] !== '2' ||\n"
    "    parenthesizedBracketRootKeys[2] !== 'b' ||\n"
    "    parenthesizedBracketRootKeys[3] !== 'a' ||\n"
    '    frozenBracketRootKeys.length !== 4 ||\n'
    "    frozenBracketRootKeys[0] !== '1' ||\n"
    "    frozenBracketRootKeys[1] !== '2' ||\n"
    "    frozenBracketRootKeys[2] !== 'b' ||\n"
    "    frozenBracketRootKeys[3] !== 'a' ||\n"
    '    singleQuotedKeys.length !== 4 ||\n'
    "    singleQuotedKeys[0] !== '1' ||\n"
    "    singleQuotedKeys[1] !== '2' ||\n"
    "    singleQuotedKeys[2] !== 'b' ||\n"
    "    singleQuotedKeys[3] !== 'a' ||\n"
    '    frozenSingleQuotedKeys.length !== 4 ||\n'
    "    frozenSingleQuotedKeys[0] !== '1' ||\n"
    "    frozenSingleQuotedKeys[1] !== '2' ||\n"
    "    frozenSingleQuotedKeys[2] !== 'b' ||\n"
    "    frozenSingleQuotedKeys[3] !== 'a' ||\n"
    '  singleQuotedMixedBracketedKeys.length !== 4 ||\n'
    "  singleQuotedMixedBracketedKeys[0] !== '1' ||\n"
    "  singleQuotedMixedBracketedKeys[1] !== '2' ||\n"
    "  singleQuotedMixedBracketedKeys[2] !== 'b' ||\n"
    "  singleQuotedMixedBracketedKeys[3] !== 'a' ||\n"
    '  parenthesizedFrozenSingleQuotedKeys.length !== 4 ||\n'
    "  parenthesizedFrozenSingleQuotedKeys[0] !== '1' ||\n"
    "  parenthesizedFrozenSingleQuotedKeys[1] !== '2' ||\n"
    "  parenthesizedFrozenSingleQuotedKeys[2] !== 'b' ||\n"
    "  parenthesizedFrozenSingleQuotedKeys[3] !== 'a' ||\n"
    '  parenthesizedFrozenSingleQuotedFrozenKeys.length !== 4 ||\n'
    "  parenthesizedFrozenSingleQuotedFrozenKeys[0] !== '1' ||\n"
    "  parenthesizedFrozenSingleQuotedFrozenKeys[1] !== '2' ||\n"
    "  parenthesizedFrozenSingleQuotedFrozenKeys[2] !== 'b' ||\n"
    "  parenthesizedFrozenSingleQuotedFrozenKeys[3] !== 'a' ||\n"
    '  parenthesizedFrozenMixedBracketedKeys.length !== 4 ||\n'
    "  parenthesizedFrozenMixedBracketedKeys[0] !== '1' ||\n"
    "  parenthesizedFrozenMixedBracketedKeys[1] !== '2' ||\n"
    "  parenthesizedFrozenMixedBracketedKeys[2] !== 'b' ||\n"
    "  parenthesizedFrozenMixedBracketedKeys[3] !== 'a' ||\n"
    '  frozenKeys.length !== 4 ||\n'
    '  frozenBareCallableKeys.length !== 4 ||\n'
    '  parenthesizedFrozenBareCallableKeys.length !== 4 ||\n'
    '  frozenCallableKeys.length !== 4 ||\n'
    '  frozenMixedBracketedKeys.length !== 4 ||\n'
    '  frozenBracketedKeys.length !== 4 ||\n'
    '  parenthesizedFrozenBracketedKeys.length !== 4 ||\n'
    "  frozenKeys[0] !== '1' ||\n"
    "  frozenKeys[1] !== '2' ||\n"
    "  frozenKeys[2] !== 'b' ||\n"
    "  frozenKeys[3] !== 'a' ||\n"
    "  frozenBareCallableKeys[0] !== '1' ||\n"
    "  frozenBareCallableKeys[1] !== '2' ||\n"
    "  frozenBareCallableKeys[2] !== 'b' ||\n"
    "  frozenBareCallableKeys[3] !== 'a' ||\n"
    "  parenthesizedFrozenBareCallableKeys[0] !== '1' ||\n"
    "  parenthesizedFrozenBareCallableKeys[1] !== '2' ||\n"
    "  parenthesizedFrozenBareCallableKeys[2] !== 'b' ||\n"
    "  parenthesizedFrozenBareCallableKeys[3] !== 'a' ||\n"
    "  frozenCallableKeys[0] !== '1' ||\n"
    "  frozenCallableKeys[1] !== '2' ||\n"
    "  frozenCallableKeys[2] !== 'b' ||\n"
    "  frozenCallableKeys[3] !== 'a' ||\n"
    '  syncCount !== 4 ||\n'
    '  frozenSyncCount !== 4 ||\n'
    '  sequenceCount !== 4 ||\n'
    '  frozenSequenceCount !== 4 ||\n'
    '  asyncCount !== 4 ||\n'
    '  frozenAsyncCount !== 4 ||\n'
    '  asyncSequenceCount !== 4 ||\n'
    '  frozenAsyncSequenceCount !== 4 ||\n'
    '  breakContinueCount !== 1\n'
    ') {\n'
    "  throw new Error('unexpected Reflect.ownKeys ordering');\n"
    '}\n'
    "console.log('reflect ownKeys ok');\n"
)

CAP_REFLECT_TEST = (
    "Kali.test('reflect ownKeys', () => {\n"
    '  const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };\n'
    '  const frozenObj = Object.freeze(obj);\n'
    '  const keys = globalThis.Reflect.ownKeys(obj);\n'
    '  const frozenKeys = globalThis.Reflect.ownKeys(frozenObj);\n'
    '  const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);\n'
    '  const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);\n'
    '  const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);\n'
    '  const fullyBracketedKeys = globalThis["Reflect"]["ownKeys"](obj);\n'
    '  const parenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj);\n'
    '  const frozenBracketRootKeys = Object.freeze((globalThis["Reflect"]))["ownKeys"](obj);\n'
    "  const singleQuotedKeys = globalThis['Reflect']['ownKeys'](obj);\n"
    "const singleQuotedMixedBracketedKeys = globalThis.Reflect['ownKeys'](obj);\n"
    "  const frozenSingleQuotedKeys = globalThis['Reflect']['ownKeys'](frozenObj);\n"
    '  const parenthesizedFrozenMixedBracketedKeys = Object.freeze((globalThis.Reflect["ownKeys"]))(obj);\n'
    'const frozenBareCallableKeys = Object.freeze(Reflect.ownKeys)(obj);\n'
    'const parenthesizedFrozenBareCallableKeys = Object.freeze((Reflect.ownKeys))(obj);\n'
    'const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)(obj);\n'
    'const frozenMixedBracketedKeys = Object.freeze(globalThis.Reflect["ownKeys"])(obj);\n'
    'const frozenBracketedKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])(obj);\n'
    'const parenthesizedFrozenBracketedKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))(obj);\n'
    'const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))(obj);\n'
    'const frozenNullishCallableKeys = Object.freeze((null ?? Reflect.ownKeys))(obj);\n'
    'const frozenLogicalAndCallableKeys = Object.freeze((true && Reflect.ownKeys))(obj);\n'
    'const frozenLogicalOrCallableKeys = Object.freeze((false || Reflect.ownKeys))(obj);\n'
    '  let syncCount = 0;\n'
    '  for (const key of globalThis.Reflect.ownKeys(obj)) {\n'
    '    syncCount += 1;\n'
    '  }\n'
    '  let frozenSyncCount = 0;\n'
    '  for (const key of globalThis.Reflect.ownKeys(frozenObj)) {\n'
    '    frozenSyncCount += 1;\n'
    '  }\n'
    '  let sequenceCount = 0;\n'
    '  for (const key of (0, globalThis.Reflect.ownKeys(obj))) {\n'
    '    sequenceCount += 1;\n'
    '  }\n'
    '  let frozenSequenceCount = 0;\n'
    '  for (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {\n'
    '    frozenSequenceCount += 1;\n'
    '  }\n'
    '  let asyncCount = 0;\n'
    '  for await (const key of globalThis.Reflect.ownKeys(obj)) {\n'
    '    asyncCount += 1;\n'
    '  }\n'
    '  let frozenAsyncCount = 0;\n'
    '  for await (const key of globalThis.Reflect.ownKeys(frozenObj)) {\n'
    '    frozenAsyncCount += 1;\n'
    '  }\n'
    '  let asyncSequenceCount = 0;\n'
    '  for await (const key of (0, globalThis.Reflect.ownKeys(obj))) {\n'
    '    asyncSequenceCount += 1;\n'
    '  }\n'
    '  let frozenAsyncSequenceCount = 0;\n'
    '  for await (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {\n'
    '    frozenAsyncSequenceCount += 1;\n'
    '  }\n'
    '  let breakContinueCount = 0;\n'
    '  for (const key of globalThis.Reflect.ownKeys(obj)) {\n'
    "    if (key === '1') {\n"
    '      continue;\n'
    '    }\n'
    '    breakContinueCount += 1;\n'
    '    break;\n'
    '  }\n'
    '  if (\n'
    '    keys.length !== 4 ||\n'
    "    keys[0] !== '1' ||\n"
    "    keys[1] !== '2' ||\n"
    "    keys[2] !== 'b' ||\n"
    "    keys[3] !== 'a' ||\n"
    '    frozenKeys.length !== 4 ||\n'
    "    frozenKeys[0] !== '1' ||\n"
    "    frozenKeys[1] !== '2' ||\n"
    "    frozenKeys[2] !== 'b' ||\n"
    "    frozenKeys[3] !== 'a' ||\n"
    '    mixedRootKeys.length !== 4 ||\n'
    "    mixedRootKeys[0] !== '1' ||\n"
    "    mixedRootKeys[1] !== '2' ||\n"
    "    mixedRootKeys[2] !== 'b' ||\n"
    "    mixedRootKeys[3] !== 'a' ||\n"
    '    mixedBracketedKeys.length !== 4 ||\n'
    "    mixedBracketedKeys[0] !== '1' ||\n"
    "    mixedBracketedKeys[1] !== '2' ||\n"
    "    mixedBracketedKeys[2] !== 'b' ||\n"
    "    mixedBracketedKeys[3] !== 'a' ||\n"
    '    bracketedKeys.length !== 4 ||\n'
    "    bracketedKeys[0] !== '1' ||\n"
    "    bracketedKeys[1] !== '2' ||\n"
    "    bracketedKeys[2] !== 'b' ||\n"
    "    bracketedKeys[3] !== 'a' ||\n"
    '  fullyBracketedKeys.length !== 4 ||\n'
    "  fullyBracketedKeys[0] !== '1' ||\n"
    "  fullyBracketedKeys[1] !== '2' ||\n"
    "  fullyBracketedKeys[2] !== 'b' ||\n"
    "  fullyBracketedKeys[3] !== 'a' ||\n"
    '    singleQuotedKeys.length !== 4 ||\n'
    "    singleQuotedKeys[0] !== '1' ||\n"
    "    singleQuotedKeys[1] !== '2' ||\n"
    "    singleQuotedKeys[2] !== 'b' ||\n"
    "    singleQuotedKeys[3] !== 'a' ||\n"
    '    frozenSingleQuotedKeys.length !== 4 ||\n'
    "    frozenSingleQuotedKeys[0] !== '1' ||\n"
    "    frozenSingleQuotedKeys[1] !== '2' ||\n"
    "    frozenSingleQuotedKeys[2] !== 'b' ||\n"
    "    frozenSingleQuotedKeys[3] !== 'a' ||\n"
    '    parenthesizedFrozenMixedBracketedKeys.length !== 4 ||\n'
    "    parenthesizedFrozenMixedBracketedKeys[0] !== '1' ||\n"
    "    parenthesizedFrozenMixedBracketedKeys[1] !== '2' ||\n"
    "    parenthesizedFrozenMixedBracketedKeys[2] !== 'b' ||\n"
    "    parenthesizedFrozenMixedBracketedKeys[3] !== 'a' ||\n"
    '    syncCount !== 4 ||\n'
    '    frozenSyncCount !== 4 ||\n'
    '    sequenceCount !== 4 ||\n'
    '    frozenSequenceCount !== 4 ||\n'
    '    asyncCount !== 4 ||\n'
    '    frozenAsyncCount !== 4 ||\n'
    '    asyncSequenceCount !== 4 ||\n'
    '    frozenAsyncSequenceCount !== 4 ||\n'
    '    breakContinueCount !== 1\n'
    '  ) {\n'
    "    throw new Error('unexpected Reflect.ownKeys ordering');\n"
    '  }\n'
    '});\n'
)

CAP_REFLECT_BUNDLE = (
    '// kali-tree-shake: reflectOwnKeysSmoke\n'
    'async function reflectOwnKeysSmoke(left, right) {\n'
    '  const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };\n'
    '  const frozenObj = Object.freeze(obj);\n'
    '  const keys = Reflect.ownKeys(obj);\n'
    '  const frozenKeys = Reflect.ownKeys(frozenObj);\n'
    '  const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);\n'
    '  const mixedBracketedDirectKeys = globalThis["Reflect"][\'ownKeys\'](obj);\n'
    '  const mixedSingleQuotedDirectKeys = globalThis[\'Reflect\']["ownKeys"](obj);\n'
    '  const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);\n'
    '  const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);\n'
    '  const fullyBracketedKeys = globalThis["Reflect"]["ownKeys"](obj);\n'
    '  const parenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj);\n'
    '  const frozenBracketRootKeys = Object.freeze((globalThis["Reflect"]))["ownKeys"](obj);\n'
    "  const singleQuotedKeys = globalThis['Reflect']['ownKeys'](obj);\n"
    "const singleQuotedMixedBracketedKeys = globalThis.Reflect['ownKeys'](obj);\n"
    "  const frozenSingleQuotedKeys = globalThis['Reflect']['ownKeys'](frozenObj);\n"
    '  const parenthesizedFrozenMixedBracketedKeys = Object.freeze((globalThis.Reflect["ownKeys"]))(obj);\n'
    'const frozenBareCallableKeys = Object.freeze(Reflect.ownKeys)(obj);\n'
    'const parenthesizedFrozenBareCallableKeys = Object.freeze((Reflect.ownKeys))(obj);\n'
    'const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)(obj);\n'
    'const frozenMixedBracketedKeys = Object.freeze(globalThis.Reflect["ownKeys"])(obj);\n'
    'const frozenBracketedKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])(obj);\n'
    'const parenthesizedFrozenBracketedKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))(obj);\n'
    'const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))(obj);\n'
    'const frozenNullishCallableKeys = Object.freeze((null ?? Reflect.ownKeys))(obj);\n'
    'const frozenLogicalAndCallableKeys = Object.freeze((true && Reflect.ownKeys))(obj);\n'
    'const frozenLogicalOrCallableKeys = Object.freeze((false || Reflect.ownKeys))(obj);\n'
    '  let syncCount = 0;\n'
    '  for (const key of Reflect.ownKeys(obj)) {\n'
    '    syncCount += 1;\n'
    '  }\n'
    '  let frozenSyncCount = 0;\n'
    '  for (const key of Reflect.ownKeys(frozenObj)) {\n'
    '    frozenSyncCount += 1;\n'
    '  }\n'
    '  let sequenceCount = 0;\n'
    '  for (const key of (0, Reflect.ownKeys(obj))) {\n'
    '    sequenceCount += 1;\n'
    '  }\n'
    '  let frozenSequenceCount = 0;\n'
    '  for (const key of (0, Reflect.ownKeys(frozenObj))) {\n'
    '    frozenSequenceCount += 1;\n'
    '  }\n'
    '  let asyncCount = 0;\n'
    '  for await (const key of Reflect.ownKeys(obj)) {\n'
    '    asyncCount += 1;\n'
    '  }\n'
    '  let frozenAsyncCount = 0;\n'
    '  for await (const key of Reflect.ownKeys(frozenObj)) {\n'
    '    frozenAsyncCount += 1;\n'
    '  }\n'
    '  let asyncSequenceCount = 0;\n'
    '  for await (const key of (0, Reflect.ownKeys(obj))) {\n'
    '    asyncSequenceCount += 1;\n'
    '  }\n'
    '  let frozenAsyncSequenceCount = 0;\n'
    '  for await (const key of (0, Reflect.ownKeys(frozenObj))) {\n'
    '    frozenAsyncSequenceCount += 1;\n'
    '  }\n'
    '  let breakContinueCount = 0;\n'
    '  for (const key of Reflect.ownKeys(obj)) {\n'
    "    if (key === '1') {\n"
    '      continue;\n'
    '    }\n'
    '    breakContinueCount += 1;\n'
    '    break;\n'
    '  }\n'
    '  if (\n'
    '    keys.length !== 4 ||\n'
    "    keys[0] !== '1' ||\n"
    "    keys[1] !== '2' ||\n"
    "    keys[2] !== 'b' ||\n"
    "    keys[3] !== 'a' ||\n"
    '    frozenKeys.length !== 4 ||\n'
    "    frozenKeys[0] !== '1' ||\n"
    "    frozenKeys[1] !== '2' ||\n"
    "    frozenKeys[2] !== 'b' ||\n"
    "    frozenKeys[3] !== 'a' ||\n"
    '    mixedRootKeys.length !== 4 ||\n'
    "    mixedRootKeys[0] !== '1' ||\n"
    "    mixedRootKeys[1] !== '2' ||\n"
    "    mixedRootKeys[2] !== 'b' ||\n"
    "    mixedRootKeys[3] !== 'a' ||\n"
    '    mixedBracketedDirectKeys.length !== 4 ||\n'
    "    mixedBracketedDirectKeys[0] !== '1' ||\n"
    "    mixedBracketedDirectKeys[1] !== '2' ||\n"
    "    mixedBracketedDirectKeys[2] !== 'b' ||\n"
    "    mixedBracketedDirectKeys[3] !== 'a' ||\n"
    '    mixedSingleQuotedDirectKeys.length !== 4 ||\n'
    "    mixedSingleQuotedDirectKeys[0] !== '1' ||\n"
    "    mixedSingleQuotedDirectKeys[1] !== '2' ||\n"
    "    mixedSingleQuotedDirectKeys[2] !== 'b' ||\n"
    "    mixedSingleQuotedDirectKeys[3] !== 'a' ||\n"
    '    mixedBracketedKeys.length !== 4 ||\n'
    "    mixedBracketedKeys[0] !== '1' ||\n"
    "    mixedBracketedKeys[1] !== '2' ||\n"
    "    mixedBracketedKeys[2] !== 'b' ||\n"
    "    mixedBracketedKeys[3] !== 'a' ||\n"
    '    bracketedKeys.length !== 4 ||\n'
    "    bracketedKeys[0] !== '1' ||\n"
    "    bracketedKeys[1] !== '2' ||\n"
    "    bracketedKeys[2] !== 'b' ||\n"
    "    bracketedKeys[3] !== 'a' ||\n"
    '  fullyBracketedKeys.length !== 4 ||\n'
    "  fullyBracketedKeys[0] !== '1' ||\n"
    "  fullyBracketedKeys[1] !== '2' ||\n"
    "  fullyBracketedKeys[2] !== 'b' ||\n"
    "  fullyBracketedKeys[3] !== 'a' ||\n"
    '    parenthesizedBracketRootKeys.length !== 4 ||\n'
    "    parenthesizedBracketRootKeys[0] !== '1' ||\n"
    "    parenthesizedBracketRootKeys[1] !== '2' ||\n"
    "    parenthesizedBracketRootKeys[2] !== 'b' ||\n"
    "    parenthesizedBracketRootKeys[3] !== 'a' ||\n"
    '    frozenBracketRootKeys.length !== 4 ||\n'
    "    frozenBracketRootKeys[0] !== '1' ||\n"
    "    frozenBracketRootKeys[1] !== '2' ||\n"
    "    frozenBracketRootKeys[2] !== 'b' ||\n"
    "    frozenBracketRootKeys[3] !== 'a' ||\n"
    '    singleQuotedKeys.length !== 4 ||\n'
    "    singleQuotedKeys[0] !== '1' ||\n"
    "    singleQuotedKeys[1] !== '2' ||\n"
    "    singleQuotedKeys[2] !== 'b' ||\n"
    "    singleQuotedKeys[3] !== 'a' ||\n"
    '    frozenSingleQuotedKeys.length !== 4 ||\n'
    "    frozenSingleQuotedKeys[0] !== '1' ||\n"
    "    frozenSingleQuotedKeys[1] !== '2' ||\n"
    "    frozenSingleQuotedKeys[2] !== 'b' ||\n"
    "    frozenSingleQuotedKeys[3] !== 'a' ||\n"
    '    parenthesizedFrozenMixedBracketedKeys.length !== 4 ||\n'
    "    parenthesizedFrozenMixedBracketedKeys[0] !== '1' ||\n"
    "    parenthesizedFrozenMixedBracketedKeys[1] !== '2' ||\n"
    "    parenthesizedFrozenMixedBracketedKeys[2] !== 'b' ||\n"
    "    parenthesizedFrozenMixedBracketedKeys[3] !== 'a' ||\n"
    '    frozenNullishCallableKeys.length !== 4 ||\n'
    "    frozenNullishCallableKeys[0] !== '1' ||\n"
    "    frozenNullishCallableKeys[1] !== '2' ||\n"
    "    frozenNullishCallableKeys[2] !== 'b' ||\n"
    "    frozenNullishCallableKeys[3] !== 'a' ||\n"
    '    frozenLogicalAndCallableKeys.length !== 4 ||\n'
    "    frozenLogicalAndCallableKeys[0] !== '1' ||\n"
    "    frozenLogicalAndCallableKeys[1] !== '2' ||\n"
    "    frozenLogicalAndCallableKeys[2] !== 'b' ||\n"
    "    frozenLogicalAndCallableKeys[3] !== 'a' ||\n"
    '    frozenLogicalOrCallableKeys.length !== 4 ||\n'
    "    frozenLogicalOrCallableKeys[0] !== '1' ||\n"
    "    frozenLogicalOrCallableKeys[1] !== '2' ||\n"
    "    frozenLogicalOrCallableKeys[2] !== 'b' ||\n"
    "    frozenLogicalOrCallableKeys[3] !== 'a' ||\n"
    '    syncCount !== 4 ||\n'
    '    frozenSyncCount !== 4 ||\n'
    '    sequenceCount !== 4 ||\n'
    '    frozenSequenceCount !== 4 ||\n'
    '    asyncCount !== 4 ||\n'
    '    frozenAsyncCount !== 4 ||\n'
    '    asyncSequenceCount !== 4 ||\n'
    '    frozenAsyncSequenceCount !== 4 ||\n'
    '    breakContinueCount !== 1\n'
    '  ) {\n'
    "    throw new Error('unexpected Reflect.ownKeys ordering');\n"
    '  }\n'
    '  return left - left + right - right;\n'
    '}\n'
)
