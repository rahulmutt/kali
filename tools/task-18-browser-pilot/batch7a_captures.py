"""Rule-8 / rule-9 captured fixture texts for Task 18 batch 7A.

Every constant here is the BYTE-EXACT OUTPUT OF EXECUTING THE REAL CODE, never
a hand-applied `format!` substitution, a hand-applied `str::replace`, or a
retyped approximation. Rule 8 forbids hand-simulating a `format!`; rule 9
extends the same discipline to a fixture built one level removed inside a
library crate (`kali_common::`), and to a `str::replace` whose needle carries
significant leading whitespace -- this batch has all three shapes.

HOW THEY WERE CAPTURED, so they can be re-derived.

A temporary target `crates/kali_cli/tests/zz_b7a_dump.rs`, deleted in the same
session, with one `mod` per source that `include!`d the shipped `.rs` and a
`#[test] fn zz_dump_*` inside that `mod` (the fixture builders are private, so
the dump has to live in the module that includes them). Run as

    ZZ_B7A_OUT=<dir> cargo test -p kali_cli --test zz_b7a_dump -- zz_dump \
        --test-threads=1

`include!` rather than a retyped copy, so the executed `format!` / `replace` /
`kali_common` call is literally the one in the shipped source. Every constant
below came from that one run; none was edited afterwards.

WHY EACH ONE IS HERE (i.e. why it is not a plain string literal the lexer could
have pulled straight out of the `.rs`):

  * CAP_FROM_ENTRIES_HARNESS_* -- built by `source.replace("  __TS_ONLY__", ...)`
    where the needle CARRIES TWO LEADING SPACES and the replacement is either
    "" or a five-line block. Hand-deriving the resulting indentation is exactly
    the trap rule 8 exists to prevent.
  * CAP_HAS_OWN_BUNDLE_* / CAP_HAS_OWN_HARNESS_* -- an inline `format!` with
    `{{`/`}}` brace-collapse whose three arguments come from FOUR
    `kali_common::object` helpers, each of which itself joins a 30+ entry alias
    table. Rule 9's "one level removed inside a library crate" case.
  * CAP_HOFE_*_FROZEN -- built by `str::replace` off the plain text.
  * CAP_HOFE_*_PLAIN -- plain `&'static str` literals, captured through the same
    run anyway so that the frozen/plain pair is proved to differ by exactly the
    `Object.freeze(` wrap rather than assumed to.
  * CAP_IS_ALIAS_CHAIN_* -- one `format!` per command, wrapping a shared body in
    `Kali.test(...)` for `test` and appending a `console.log` for everything
    else.

They are embedded here rather than read from a dump file so this module runs
from a clean checkout with no uncommitted inputs -- the defect that got the
pilot's per-file generators deleted (see README). `gen_batch7a.py` re-checks
each one against its own `.rs` before emitting it (`check_captured`), so a
stale capture taken before a source edit fails the generator rather than
shipping a program that is no longer the program under test.
"""

CAP_FROM_ENTRIES_HARNESS_RUN_PLAIN = (
    'function assertFromEntriesShape(fromEntries) {\n'
    '  const keys = Object.keys(fromEntries);\n'
    '  const entries = Object.entries(fromEntries);\n'
    '  const values = Object.values(fromEntries);\n'
    '  if (\n'
    '    keys.length !== 2 ||\n'
    "    keys[0] !== 'b' ||\n"
    "    keys[1] !== 'a' ||\n"
    '    entries.length !== 2 ||\n'
    "    entries[0][0] !== 'b' ||\n"
    '    entries[0][1] !== 1 ||\n'
    "    entries[1][0] !== 'a' ||\n"
    '    entries[1][1] !== 2 ||\n'
    '    values.length !== 2 ||\n'
    '    values[0] !== 1 ||\n'
    '    values[1] !== 2\n'
    '  ) {\n'
    "    throw new Error('unexpected Object.fromEntries semantics');\n"
    '  }\n'
    '}\n'
    '\n'
    'const wrappedEntries = ([["b", 1], ["a", 2]]);\n'
    'const frozenEntries = Object.freeze([["b", 1], ["a", 2]]);\n'
    'const conditionalEntries = (true ? [["b", 1], ["a", 2]] : [["x", 9]]);\n'
    'const directFromEntries = Object.fromEntries([["b", 1], ["a", 2]]);\n'
    'const wrappedFromEntries = Object.fromEntries(wrappedEntries);\n'
    'const frozenFromEntries = Object.fromEntries(frozenEntries);\n'
    'const dottedFromEntries = globalThis.Object.fromEntries([["b", 1], ["a", 2]]);\n'
    'const mixedDottedFromEntries = globalThis.Object["fromEntries"]([["b", 1], ["a", 2]]);\n'
    'const mixedBracketedFromEntries = globalThis["Object"].fromEntries([["b", 1], ["a", 2]]);\n'
    'const mixedBracketedQuotedFromEntries = globalThis["Object"][\'fromEntries\']([["b", 1], ["a", 2]]);\n'
    'const mixedSingleQuotedFromEntries = globalThis[\'Object\']["fromEntries"]([["b", 1], ["a", 2]]);\n'
    'const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2]]);\n'
    'assertFromEntriesShape(directFromEntries);\n'
    'assertFromEntriesShape(wrappedFromEntries);\n'
    'assertFromEntriesShape(frozenFromEntries);\n'
    'assertFromEntriesShape(Object.fromEntries(conditionalEntries));\n'
    'assertFromEntriesShape(dottedFromEntries);\n'
    'assertFromEntriesShape(mixedDottedFromEntries);\n'
    'assertFromEntriesShape(mixedBracketedFromEntries);\n'
    'assertFromEntriesShape(mixedBracketedQuotedFromEntries);\n'
    'assertFromEntriesShape(mixedSingleQuotedFromEntries);\n'
    'assertFromEntriesShape(bracketedFromEntries);\n'
    "console.log('browser object fromEntries ok');\n"
)

CAP_FROM_ENTRIES_HARNESS_RUN_TS = (
    'function assertFromEntriesShape(fromEntries) {\n'
    '  const keys = Object.keys(fromEntries);\n'
    '  const entries = Object.entries(fromEntries);\n'
    '  const values = Object.values(fromEntries);\n'
    '  if (\n'
    '    keys.length !== 2 ||\n'
    "    keys[0] !== 'b' ||\n"
    "    keys[1] !== 'a' ||\n"
    '    entries.length !== 2 ||\n'
    "    entries[0][0] !== 'b' ||\n"
    '    entries[0][1] !== 1 ||\n'
    "    entries[1][0] !== 'a' ||\n"
    '    entries[1][1] !== 2 ||\n'
    '    values.length !== 2 ||\n'
    '    values[0] !== 1 ||\n'
    '    values[1] !== 2\n'
    '  ) {\n'
    "    throw new Error('unexpected Object.fromEntries semantics');\n"
    '  }\n'
    '}\n'
    '\n'
    'const wrappedEntries = ([["b", 1], ["a", 2]]);\n'
    'const frozenEntries = Object.freeze([["b", 1], ["a", 2]]);\n'
    'const conditionalEntries = (true ? [["b", 1], ["a", 2]] : [["x", 9]]);\n'
    '  const wrappedEntriesConst = ([["b", 1], ["a", 2]] as const);\n'
    '  const wrappedFromEntriesConst = Object.fromEntries(wrappedEntriesConst);\n'
    '  assertFromEntriesShape(wrappedFromEntriesConst);\n'
    '  const wrappedEntriesSatisfies = ([["b", 1], ["a", 2]] satisfies unknown);\n'
    '  const wrappedFromEntriesSatisfies = Object.fromEntries(wrappedEntriesSatisfies);\n'
    '  assertFromEntriesShape(wrappedFromEntriesSatisfies);\n'
    'const directFromEntries = Object.fromEntries([["b", 1], ["a", 2]]);\n'
    'const wrappedFromEntries = Object.fromEntries(wrappedEntries);\n'
    'const frozenFromEntries = Object.fromEntries(frozenEntries);\n'
    'const dottedFromEntries = globalThis.Object.fromEntries([["b", 1], ["a", 2]]);\n'
    'const mixedDottedFromEntries = globalThis.Object["fromEntries"]([["b", 1], ["a", 2]]);\n'
    'const mixedBracketedFromEntries = globalThis["Object"].fromEntries([["b", 1], ["a", 2]]);\n'
    'const mixedBracketedQuotedFromEntries = globalThis["Object"][\'fromEntries\']([["b", 1], ["a", 2]]);\n'
    'const mixedSingleQuotedFromEntries = globalThis[\'Object\']["fromEntries"]([["b", 1], ["a", 2]]);\n'
    'const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2]]);\n'
    'assertFromEntriesShape(directFromEntries);\n'
    'assertFromEntriesShape(wrappedFromEntries);\n'
    'assertFromEntriesShape(frozenFromEntries);\n'
    'assertFromEntriesShape(Object.fromEntries(conditionalEntries));\n'
    'assertFromEntriesShape(dottedFromEntries);\n'
    'assertFromEntriesShape(mixedDottedFromEntries);\n'
    'assertFromEntriesShape(mixedBracketedFromEntries);\n'
    'assertFromEntriesShape(mixedBracketedQuotedFromEntries);\n'
    'assertFromEntriesShape(mixedSingleQuotedFromEntries);\n'
    'assertFromEntriesShape(bracketedFromEntries);\n'
    "console.log('browser object fromEntries ok');\n"
)

CAP_FROM_ENTRIES_HARNESS_TEST_PLAIN = (
    "Kali.test('object fromEntries ordering', () => {\n"
    '  function assertFromEntriesShape(fromEntries) {\n'
    '    const keys = Object.keys(fromEntries);\n'
    '    const entries = Object.entries(fromEntries);\n'
    '    const values = Object.values(fromEntries);\n'
    '    if (\n'
    '      keys.length !== 2 ||\n'
    "      keys[0] !== 'b' ||\n"
    "      keys[1] !== 'a' ||\n"
    '      entries.length !== 2 ||\n'
    "      entries[0][0] !== 'b' ||\n"
    '      entries[0][1] !== 1 ||\n'
    "      entries[1][0] !== 'a' ||\n"
    '      entries[1][1] !== 2 ||\n'
    '      values.length !== 2 ||\n'
    '      values[0] !== 1 ||\n'
    '      values[1] !== 2\n'
    '    ) {\n'
    "      throw new Error('unexpected Object.fromEntries semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const wrappedEntries = ([["b", 1], ["a", 2]]);\n'
    '  const frozenEntries = Object.freeze([["b", 1], ["a", 2]]);\n'
    '  const conditionalEntries = (true ? [["b", 1], ["a", 2]] : [["x", 9]]);\n'
    '  assertFromEntriesShape(Object.fromEntries([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(Object.fromEntries(wrappedEntries));\n'
    '  assertFromEntriesShape(Object.fromEntries(frozenEntries));\n'
    '  assertFromEntriesShape(Object.fromEntries(conditionalEntries));\n'
    '  assertFromEntriesShape(globalThis.Object.fromEntries([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(globalThis.Object["fromEntries"]([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(globalThis["Object"].fromEntries([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(globalThis["Object"][\'fromEntries\']([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(globalThis[\'Object\']["fromEntries"]([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2]]));\n'
    "  console.log('browser object fromEntries ok');\n"
    '});\n'
)

CAP_FROM_ENTRIES_HARNESS_TEST_TS = (
    "Kali.test('object fromEntries ordering', () => {\n"
    '  function assertFromEntriesShape(fromEntries) {\n'
    '    const keys = Object.keys(fromEntries);\n'
    '    const entries = Object.entries(fromEntries);\n'
    '    const values = Object.values(fromEntries);\n'
    '    if (\n'
    '      keys.length !== 2 ||\n'
    "      keys[0] !== 'b' ||\n"
    "      keys[1] !== 'a' ||\n"
    '      entries.length !== 2 ||\n'
    "      entries[0][0] !== 'b' ||\n"
    '      entries[0][1] !== 1 ||\n'
    "      entries[1][0] !== 'a' ||\n"
    '      entries[1][1] !== 2 ||\n'
    '      values.length !== 2 ||\n'
    '      values[0] !== 1 ||\n'
    '      values[1] !== 2\n'
    '    ) {\n'
    "      throw new Error('unexpected Object.fromEntries semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const wrappedEntries = ([["b", 1], ["a", 2]]);\n'
    '  const frozenEntries = Object.freeze([["b", 1], ["a", 2]]);\n'
    '  const conditionalEntries = (true ? [["b", 1], ["a", 2]] : [["x", 9]]);\n'
    '  const wrappedEntriesConst = ([["b", 1], ["a", 2]] as const);\n'
    '  assertFromEntriesShape(Object.fromEntries(wrappedEntriesConst));\n'
    '  const wrappedEntriesSatisfies = ([["b", 1], ["a", 2]] satisfies unknown);\n'
    '  assertFromEntriesShape(Object.fromEntries(wrappedEntriesSatisfies));\n'
    '  assertFromEntriesShape(Object.fromEntries([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(Object.fromEntries(wrappedEntries));\n'
    '  assertFromEntriesShape(Object.fromEntries(frozenEntries));\n'
    '  assertFromEntriesShape(Object.fromEntries(conditionalEntries));\n'
    '  assertFromEntriesShape(globalThis.Object.fromEntries([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(globalThis.Object["fromEntries"]([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(globalThis["Object"].fromEntries([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(globalThis["Object"][\'fromEntries\']([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(globalThis[\'Object\']["fromEntries"]([["b", 1], ["a", 2]]));\n'
    '  assertFromEntriesShape(globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2]]));\n'
    "  console.log('browser object fromEntries ok');\n"
    '});\n'
)

CAP_HAS_OWN_BUNDLE_JS = (
    '// kali-tree-shake: browserObjectHasOwn\n'
    'function browserObjectHasOwn() {\n'
    '  const object = { a: 1, "b": 2 };\n'
    '  const alias = object;\n'
    '  const hasOwn = Object.hasOwn;\n'
    "  const singleQuotedHasOwn = globalThis['Object']['hasOwn'];\n"
    "  const parenthesizedSingleQuotedHasOwn = (globalThis['Object'])['hasOwn'];\n"
    "  const frozenSingleQuotedHasOwn = Object.freeze(globalThis['Object']['hasOwn']);\n"
    "  const frozenParenthesizedSingleQuotedHasOwn = Object.freeze((globalThis['Object'])['hasOwn']);\n"
    '  const hasOwnPropertyCall = Object.prototype.hasOwnProperty.call;\n'
    '  Object.freeze(Object.hasOwn); Object.freeze((Object.hasOwn)); Object.freeze(Object["hasOwn"]); Object.freeze((Object["hasOwn"])); Object.freeze(globalThis.Object.hasOwn); Object.freeze((globalThis.Object.hasOwn)); Object.freeze(globalThis.Object["hasOwn"]); Object.freeze(globalThis.Object[\'hasOwn\']); Object.freeze((globalThis.Object)["hasOwn"]); Object.freeze((globalThis.Object).hasOwn); Object.freeze((globalThis.Object)[\'hasOwn\']); Object.freeze((globalThis.Object["hasOwn"])); Object.freeze(globalThis?.Object.hasOwn); Object.freeze((globalThis?.Object.hasOwn)); Object.freeze((globalThis?.Object).hasOwn); Object.freeze((globalThis?.Object)["hasOwn"]); Object.freeze(globalThis?.Object["hasOwn"]); Object.freeze((globalThis?.Object["hasOwn"])); Object.freeze(globalThis["Object"].hasOwn); Object.freeze((globalThis["Object"].hasOwn)); Object.freeze((globalThis["Object"]).hasOwn); Object.freeze((globalThis["Object"])["hasOwn"]); Object.freeze(globalThis["Object"]["hasOwn"]); Object.freeze((globalThis["Object"]["hasOwn"])); Object.freeze((globalThis["Object"]))["hasOwn"]; Object.freeze((globalThis["Object"]))[\'hasOwn\']; Object.freeze((globalThis[\'Object\']))["hasOwn"]; Object.freeze((globalThis[\'Object\']))[\'hasOwn\']; Object.freeze(globalThis[\'Object\'].hasOwn); Object.freeze((globalThis[\'Object\'].hasOwn)); Object.freeze((globalThis[\'Object\']).hasOwn); Object.freeze((globalThis[\'Object\'])[\'hasOwn\']); Object.freeze(globalThis[\'Object\'][\'hasOwn\']); Object.freeze((globalThis[\'Object\'][\'hasOwn\'])); Object.freeze((null ?? Object.hasOwn)); Object.freeze((true && Object.hasOwn)); Object.freeze((false || Object.hasOwn)); Object.freeze(globalThis.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call)); Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"]); Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"]); Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"])); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call)); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"]); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"])); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call)); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call); Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"])); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call); Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call)); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\']); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)); Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"].hasOwnProperty.call); Object.freeze((globalThis["Object"].hasOwnProperty.call)); Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"])); Object.freeze(Object.prototype.hasOwnProperty.call); Object.freeze((Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype["hasOwnProperty"].call); Object.freeze((Object.prototype["hasOwnProperty"].call)); Object.freeze(Object["prototype"].hasOwnProperty.call); Object.freeze((Object["prototype"].hasOwnProperty.call)); Object.freeze(Object["prototype"]["hasOwnProperty"]["call"]); Object.freeze((Object["prototype"]["hasOwnProperty"]["call"])); Object.freeze((null ?? Object.prototype.hasOwnProperty.call)); Object.freeze((true && Object.prototype.hasOwnProperty.call)); Object.freeze((false || Object.prototype.hasOwnProperty.call)); Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype.hasOwnProperty["call"]); Object.freeze((Object.prototype.hasOwnProperty["call"])); Object.freeze(Object["prototype"].hasOwnProperty["call"]); Object.freeze((Object["prototype"].hasOwnProperty["call"]));\n'
    '  if (!globalThis["Object"]["prototype"].hasOwnProperty["call"](alias, "a") || !hasOwn(alias, "a") || !globalThis["Object"]["hasOwn"](alias, "a") || !globalThis.Object["hasOwn"](alias, "a") || !Object["hasOwn"](alias, "a") || !globalThis["Object"].hasOwn(alias, "a") || !singleQuotedHasOwn(alias, "a") || !parenthesizedSingleQuotedHasOwn(alias, "a") || !frozenSingleQuotedHasOwn(alias, "a") || !frozenParenthesizedSingleQuotedHasOwn(alias, "a") || !Object.freeze(Object.hasOwn)(alias, "a") || !Object.freeze((Object.hasOwn))(alias, "a") || !Object.freeze(Object["hasOwn"])(alias, "a") || !Object.freeze((Object["hasOwn"]))(alias, "a") || !Object.freeze(globalThis.Object.hasOwn)(alias, "a") || !Object.freeze((globalThis.Object.hasOwn))(alias, "a") || !Object.freeze(globalThis.Object["hasOwn"])(alias, "a") || !Object.freeze(globalThis.Object[\'hasOwn\'])(alias, "a") || !Object.freeze((globalThis.Object)["hasOwn"])(alias, "a") || !Object.freeze((globalThis.Object).hasOwn)(alias, "a") || !Object.freeze((globalThis.Object)[\'hasOwn\'])(alias, "a") || !Object.freeze((globalThis.Object["hasOwn"]))(alias, "a") || !Object.freeze(globalThis?.Object.hasOwn)(alias, "a") || !Object.freeze((globalThis?.Object.hasOwn))(alias, "a") || !Object.freeze((globalThis?.Object).hasOwn)(alias, "a") || !Object.freeze((globalThis?.Object)["hasOwn"])(alias, "a") || !Object.freeze(globalThis?.Object["hasOwn"])(alias, "a") || !Object.freeze((globalThis?.Object["hasOwn"]))(alias, "a") || !Object.freeze(globalThis["Object"].hasOwn)(alias, "a") || !Object.freeze((globalThis["Object"].hasOwn))(alias, "a") || !Object.freeze((globalThis["Object"]).hasOwn)(alias, "a") || !Object.freeze((globalThis["Object"])["hasOwn"])(alias, "a") || !Object.freeze(globalThis["Object"]["hasOwn"])(alias, "a") || !Object.freeze((globalThis["Object"]["hasOwn"]))(alias, "a") || !Object.freeze((globalThis["Object"]))["hasOwn"](alias, "a") || !Object.freeze((globalThis["Object"]))[\'hasOwn\'](alias, "a") || !Object.freeze((globalThis[\'Object\']))["hasOwn"](alias, "a") || !Object.freeze((globalThis[\'Object\']))[\'hasOwn\'](alias, "a") || !Object.freeze(globalThis[\'Object\'].hasOwn)(alias, "a") || !Object.freeze((globalThis[\'Object\'].hasOwn))(alias, "a") || !Object.freeze((globalThis[\'Object\']).hasOwn)(alias, "a") || !Object.freeze((globalThis[\'Object\'])[\'hasOwn\'])(alias, "a") || !Object.freeze(globalThis[\'Object\'][\'hasOwn\'])(alias, "a") || !Object.freeze((globalThis[\'Object\'][\'hasOwn\']))(alias, "a") || !Object.freeze((null ?? Object.hasOwn))(alias, "a") || !Object.freeze((true && Object.hasOwn))(alias, "a") || !Object.freeze((false || Object.hasOwn))(alias, "a") || !Object.freeze(globalThis.Object.prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis.Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"])(alias, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])(alias, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))(alias, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call))(alias, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"])(alias, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"]))(alias, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"])(alias, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call)(alias, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call)(alias, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call)(alias, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"])(alias, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"]))(alias, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"])(alias, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"])(alias, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"])(alias, "a") || !Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call)(alias, "a") || !Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call))(alias, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])(alias, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']))(alias, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\'])(alias, "a") || !Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\'])(alias, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)(alias, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call))(alias, "a") || !Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"])(alias, "a") || !Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"]))(alias, "a") || !Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])(alias, "a") || !Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]))(alias, "a") || !Object.freeze(globalThis["Object"].hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis["Object"].hasOwnProperty.call))(alias, "a") || !Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"])(alias, "a") || !Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"]))(alias, "a") || !Object.freeze(Object.prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze(Object.prototype["hasOwnProperty"].call)(alias, "a") || !Object.freeze((Object.prototype["hasOwnProperty"].call))(alias, "a") || !Object.freeze(Object["prototype"].hasOwnProperty.call)(alias, "a") || !Object.freeze((Object["prototype"].hasOwnProperty.call))(alias, "a") || !Object.freeze(Object["prototype"]["hasOwnProperty"]["call"])(alias, "a") || !Object.freeze((Object["prototype"]["hasOwnProperty"]["call"]))(alias, "a") || !Object.freeze((null ?? Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((true && Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((false || Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze(Object.prototype.hasOwnProperty["call"])(alias, "a") || !Object.freeze((Object.prototype.hasOwnProperty["call"]))(alias, "a") || !Object.freeze(Object["prototype"].hasOwnProperty["call"])(alias, "a") || !Object.freeze((Object["prototype"].hasOwnProperty["call"]))(alias, "a") ||\n'
    '    !Object["hasOwnProperty"].call(alias, "a") || !Object["hasOwnProperty"]["call"](alias, "a") || !globalThis.Object.hasOwnProperty.call(alias, "a") || !globalThis["Object"]["hasOwnProperty"].call(alias, "a") || !globalThis["Object"]["hasOwnProperty"]["call"](alias, "a") || !globalThis["Object"].hasOwnProperty.call(alias, "a") || !hasOwnPropertyCall(alias, "a") || !globalThis["Object"].prototype["hasOwnProperty"]["call"](alias, "a") || !globalThis["Object"].prototype.hasOwnProperty.call(alias, "a") || !globalThis.Object.prototype["hasOwnProperty"]["call"](alias, "a") || !globalThis.Object.prototype.hasOwnProperty["call"](alias, "a") || !globalThis.Object["prototype"].hasOwnProperty.call(alias, "a") || !globalThis.Object["prototype"]["hasOwnProperty"]["call"](alias, "a") || !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](alias, "a")) {\n'
    "    throw new Error('unexpected browser Object.hasOwn result');\n"
    '  }\n'
    "  console.log('browser object hasOwn ok');\n"
    '}\n'
)

CAP_HAS_OWN_BUNDLE_TS = (
    '// kali-tree-shake: browserObjectHasOwn\n'
    'function browserObjectHasOwn() {\n'
    '  const object = ({ a: 1, "b": 2 } as const);\n'
    '  const alias = object;\n'
    '  const hasOwn = Object.hasOwn;\n'
    "  const singleQuotedHasOwn = globalThis['Object']['hasOwn'];\n"
    "  const parenthesizedSingleQuotedHasOwn = (globalThis['Object'])['hasOwn'];\n"
    "  const frozenSingleQuotedHasOwn = Object.freeze(globalThis['Object']['hasOwn']);\n"
    "  const frozenParenthesizedSingleQuotedHasOwn = Object.freeze((globalThis['Object'])['hasOwn']);\n"
    '  const hasOwnPropertyCall = Object.prototype.hasOwnProperty.call;\n'
    '  Object.freeze(Object.hasOwn); Object.freeze((Object.hasOwn)); Object.freeze(Object["hasOwn"]); Object.freeze((Object["hasOwn"])); Object.freeze(globalThis.Object.hasOwn); Object.freeze((globalThis.Object.hasOwn)); Object.freeze(globalThis.Object["hasOwn"]); Object.freeze(globalThis.Object[\'hasOwn\']); Object.freeze((globalThis.Object)["hasOwn"]); Object.freeze((globalThis.Object).hasOwn); Object.freeze((globalThis.Object)[\'hasOwn\']); Object.freeze((globalThis.Object["hasOwn"])); Object.freeze(globalThis?.Object.hasOwn); Object.freeze((globalThis?.Object.hasOwn)); Object.freeze((globalThis?.Object).hasOwn); Object.freeze((globalThis?.Object)["hasOwn"]); Object.freeze(globalThis?.Object["hasOwn"]); Object.freeze((globalThis?.Object["hasOwn"])); Object.freeze(globalThis["Object"].hasOwn); Object.freeze((globalThis["Object"].hasOwn)); Object.freeze((globalThis["Object"]).hasOwn); Object.freeze((globalThis["Object"])["hasOwn"]); Object.freeze(globalThis["Object"]["hasOwn"]); Object.freeze((globalThis["Object"]["hasOwn"])); Object.freeze((globalThis["Object"]))["hasOwn"]; Object.freeze((globalThis["Object"]))[\'hasOwn\']; Object.freeze((globalThis[\'Object\']))["hasOwn"]; Object.freeze((globalThis[\'Object\']))[\'hasOwn\']; Object.freeze(globalThis[\'Object\'].hasOwn); Object.freeze((globalThis[\'Object\'].hasOwn)); Object.freeze((globalThis[\'Object\']).hasOwn); Object.freeze((globalThis[\'Object\'])[\'hasOwn\']); Object.freeze(globalThis[\'Object\'][\'hasOwn\']); Object.freeze((globalThis[\'Object\'][\'hasOwn\'])); Object.freeze((null ?? Object.hasOwn)); Object.freeze((true && Object.hasOwn)); Object.freeze((false || Object.hasOwn)); Object.freeze(globalThis.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call)); Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"]); Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"]); Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"])); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call)); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"]); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"])); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call)); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call); Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"])); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call); Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call)); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\']); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)); Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"].hasOwnProperty.call); Object.freeze((globalThis["Object"].hasOwnProperty.call)); Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"])); Object.freeze(Object.prototype.hasOwnProperty.call); Object.freeze((Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype["hasOwnProperty"].call); Object.freeze((Object.prototype["hasOwnProperty"].call)); Object.freeze(Object["prototype"].hasOwnProperty.call); Object.freeze((Object["prototype"].hasOwnProperty.call)); Object.freeze(Object["prototype"]["hasOwnProperty"]["call"]); Object.freeze((Object["prototype"]["hasOwnProperty"]["call"])); Object.freeze((null ?? Object.prototype.hasOwnProperty.call)); Object.freeze((true && Object.prototype.hasOwnProperty.call)); Object.freeze((false || Object.prototype.hasOwnProperty.call)); Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype.hasOwnProperty["call"]); Object.freeze((Object.prototype.hasOwnProperty["call"])); Object.freeze(Object["prototype"].hasOwnProperty["call"]); Object.freeze((Object["prototype"].hasOwnProperty["call"]));\n'
    '  if (!globalThis["Object"]["prototype"].hasOwnProperty["call"](alias, "a") || !hasOwn(alias, "a") || !globalThis["Object"]["hasOwn"](alias, "a") || !globalThis.Object["hasOwn"](alias, "a") || !Object["hasOwn"](alias, "a") || !globalThis["Object"].hasOwn(alias, "a") || !singleQuotedHasOwn(alias, "a") || !parenthesizedSingleQuotedHasOwn(alias, "a") || !frozenSingleQuotedHasOwn(alias, "a") || !frozenParenthesizedSingleQuotedHasOwn(alias, "a") || !Object.freeze(Object.hasOwn)(alias, "a") || !Object.freeze((Object.hasOwn))(alias, "a") || !Object.freeze(Object["hasOwn"])(alias, "a") || !Object.freeze((Object["hasOwn"]))(alias, "a") || !Object.freeze(globalThis.Object.hasOwn)(alias, "a") || !Object.freeze((globalThis.Object.hasOwn))(alias, "a") || !Object.freeze(globalThis.Object["hasOwn"])(alias, "a") || !Object.freeze(globalThis.Object[\'hasOwn\'])(alias, "a") || !Object.freeze((globalThis.Object)["hasOwn"])(alias, "a") || !Object.freeze((globalThis.Object).hasOwn)(alias, "a") || !Object.freeze((globalThis.Object)[\'hasOwn\'])(alias, "a") || !Object.freeze((globalThis.Object["hasOwn"]))(alias, "a") || !Object.freeze(globalThis?.Object.hasOwn)(alias, "a") || !Object.freeze((globalThis?.Object.hasOwn))(alias, "a") || !Object.freeze((globalThis?.Object).hasOwn)(alias, "a") || !Object.freeze((globalThis?.Object)["hasOwn"])(alias, "a") || !Object.freeze(globalThis?.Object["hasOwn"])(alias, "a") || !Object.freeze((globalThis?.Object["hasOwn"]))(alias, "a") || !Object.freeze(globalThis["Object"].hasOwn)(alias, "a") || !Object.freeze((globalThis["Object"].hasOwn))(alias, "a") || !Object.freeze((globalThis["Object"]).hasOwn)(alias, "a") || !Object.freeze((globalThis["Object"])["hasOwn"])(alias, "a") || !Object.freeze(globalThis["Object"]["hasOwn"])(alias, "a") || !Object.freeze((globalThis["Object"]["hasOwn"]))(alias, "a") || !Object.freeze((globalThis["Object"]))["hasOwn"](alias, "a") || !Object.freeze((globalThis["Object"]))[\'hasOwn\'](alias, "a") || !Object.freeze((globalThis[\'Object\']))["hasOwn"](alias, "a") || !Object.freeze((globalThis[\'Object\']))[\'hasOwn\'](alias, "a") || !Object.freeze(globalThis[\'Object\'].hasOwn)(alias, "a") || !Object.freeze((globalThis[\'Object\'].hasOwn))(alias, "a") || !Object.freeze((globalThis[\'Object\']).hasOwn)(alias, "a") || !Object.freeze((globalThis[\'Object\'])[\'hasOwn\'])(alias, "a") || !Object.freeze(globalThis[\'Object\'][\'hasOwn\'])(alias, "a") || !Object.freeze((globalThis[\'Object\'][\'hasOwn\']))(alias, "a") || !Object.freeze((null ?? Object.hasOwn))(alias, "a") || !Object.freeze((true && Object.hasOwn))(alias, "a") || !Object.freeze((false || Object.hasOwn))(alias, "a") || !Object.freeze(globalThis.Object.prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis.Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"])(alias, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])(alias, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))(alias, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call))(alias, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"])(alias, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"]))(alias, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"])(alias, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call)(alias, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call)(alias, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call)(alias, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"])(alias, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"]))(alias, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"])(alias, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"])(alias, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"])(alias, "a") || !Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call)(alias, "a") || !Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call))(alias, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])(alias, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']))(alias, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\'])(alias, "a") || !Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\'])(alias, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)(alias, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call))(alias, "a") || !Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"])(alias, "a") || !Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"]))(alias, "a") || !Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])(alias, "a") || !Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]))(alias, "a") || !Object.freeze(globalThis["Object"].hasOwnProperty.call)(alias, "a") || !Object.freeze((globalThis["Object"].hasOwnProperty.call))(alias, "a") || !Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"])(alias, "a") || !Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"]))(alias, "a") || !Object.freeze(Object.prototype.hasOwnProperty.call)(alias, "a") || !Object.freeze((Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze(Object.prototype["hasOwnProperty"].call)(alias, "a") || !Object.freeze((Object.prototype["hasOwnProperty"].call))(alias, "a") || !Object.freeze(Object["prototype"].hasOwnProperty.call)(alias, "a") || !Object.freeze((Object["prototype"].hasOwnProperty.call))(alias, "a") || !Object.freeze(Object["prototype"]["hasOwnProperty"]["call"])(alias, "a") || !Object.freeze((Object["prototype"]["hasOwnProperty"]["call"]))(alias, "a") || !Object.freeze((null ?? Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((true && Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((false || Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call))(alias, "a") || !Object.freeze(Object.prototype.hasOwnProperty["call"])(alias, "a") || !Object.freeze((Object.prototype.hasOwnProperty["call"]))(alias, "a") || !Object.freeze(Object["prototype"].hasOwnProperty["call"])(alias, "a") || !Object.freeze((Object["prototype"].hasOwnProperty["call"]))(alias, "a") ||\n'
    '    !Object["hasOwnProperty"].call(alias, "a") || !Object["hasOwnProperty"]["call"](alias, "a") || !globalThis.Object.hasOwnProperty.call(alias, "a") || !globalThis["Object"]["hasOwnProperty"].call(alias, "a") || !globalThis["Object"]["hasOwnProperty"]["call"](alias, "a") || !globalThis["Object"].hasOwnProperty.call(alias, "a") || !hasOwnPropertyCall(alias, "a") || !globalThis["Object"].prototype["hasOwnProperty"]["call"](alias, "a") || !globalThis["Object"].prototype.hasOwnProperty.call(alias, "a") || !globalThis.Object.prototype["hasOwnProperty"]["call"](alias, "a") || !globalThis.Object.prototype.hasOwnProperty["call"](alias, "a") || !globalThis.Object["prototype"].hasOwnProperty.call(alias, "a") || !globalThis.Object["prototype"]["hasOwnProperty"]["call"](alias, "a") || !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](alias, "a")) {\n'
    "    throw new Error('unexpected browser Object.hasOwn result');\n"
    '  }\n'
    "  console.log('browser object hasOwn ok');\n"
    '}\n'
)

CAP_HAS_OWN_HARNESS_RUN = (
    'const object = Object.fromEntries([["a", 1], ["b", 2]]);\n'
    'const alias = object;\n'
    'const hasOwn = Object.hasOwn;\n'
    "const singleQuotedHasOwn = globalThis['Object']['hasOwn'];\n"
    "const parenthesizedSingleQuotedHasOwn = (globalThis['Object'])['hasOwn'];\n"
    "const frozenSingleQuotedHasOwn = Object.freeze(globalThis['Object']['hasOwn']);\n"
    "const frozenParenthesizedSingleQuotedHasOwn = Object.freeze((globalThis['Object'])['hasOwn']);\n"
    'const hasOwnPropertyCall = Object.prototype.hasOwnProperty.call;\n'
    'Object.freeze(Object.hasOwn); Object.freeze((Object.hasOwn)); Object.freeze(Object["hasOwn"]); Object.freeze((Object["hasOwn"])); Object.freeze(globalThis.Object.hasOwn); Object.freeze((globalThis.Object.hasOwn)); Object.freeze(globalThis.Object["hasOwn"]); Object.freeze(globalThis.Object[\'hasOwn\']); Object.freeze((globalThis.Object)["hasOwn"]); Object.freeze((globalThis.Object).hasOwn); Object.freeze((globalThis.Object)[\'hasOwn\']); Object.freeze((globalThis.Object["hasOwn"])); Object.freeze(globalThis?.Object.hasOwn); Object.freeze((globalThis?.Object.hasOwn)); Object.freeze((globalThis?.Object).hasOwn); Object.freeze((globalThis?.Object)["hasOwn"]); Object.freeze(globalThis?.Object["hasOwn"]); Object.freeze((globalThis?.Object["hasOwn"])); Object.freeze(globalThis["Object"].hasOwn); Object.freeze((globalThis["Object"].hasOwn)); Object.freeze((globalThis["Object"]).hasOwn); Object.freeze((globalThis["Object"])["hasOwn"]); Object.freeze(globalThis["Object"]["hasOwn"]); Object.freeze((globalThis["Object"]["hasOwn"])); Object.freeze((globalThis["Object"]))["hasOwn"]; Object.freeze((globalThis["Object"]))[\'hasOwn\']; Object.freeze((globalThis[\'Object\']))["hasOwn"]; Object.freeze((globalThis[\'Object\']))[\'hasOwn\']; Object.freeze(globalThis[\'Object\'].hasOwn); Object.freeze((globalThis[\'Object\'].hasOwn)); Object.freeze((globalThis[\'Object\']).hasOwn); Object.freeze((globalThis[\'Object\'])[\'hasOwn\']); Object.freeze(globalThis[\'Object\'][\'hasOwn\']); Object.freeze((globalThis[\'Object\'][\'hasOwn\'])); Object.freeze((null ?? Object.hasOwn)); Object.freeze((true && Object.hasOwn)); Object.freeze((false || Object.hasOwn)); Object.freeze(globalThis.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call)); Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"]); Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"]); Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"])); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call)); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"]); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"])); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call)); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call); Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"])); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call); Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call)); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\']); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)); Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"].hasOwnProperty.call); Object.freeze((globalThis["Object"].hasOwnProperty.call)); Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"])); Object.freeze(Object.prototype.hasOwnProperty.call); Object.freeze((Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype["hasOwnProperty"].call); Object.freeze((Object.prototype["hasOwnProperty"].call)); Object.freeze(Object["prototype"].hasOwnProperty.call); Object.freeze((Object["prototype"].hasOwnProperty.call)); Object.freeze(Object["prototype"]["hasOwnProperty"]["call"]); Object.freeze((Object["prototype"]["hasOwnProperty"]["call"])); Object.freeze((null ?? Object.prototype.hasOwnProperty.call)); Object.freeze((true && Object.prototype.hasOwnProperty.call)); Object.freeze((false || Object.prototype.hasOwnProperty.call)); Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype.hasOwnProperty["call"]); Object.freeze((Object.prototype.hasOwnProperty["call"])); Object.freeze(Object["prototype"].hasOwnProperty["call"]); Object.freeze((Object["prototype"].hasOwnProperty["call"]));\n'
    'const wrapped = (0, alias);\n'
    'if (\n'
    '  !Object.hasOwn(wrapped, "a") ||\n'
    '  !hasOwn(wrapped, "a") ||\n'
    '  !Object["hasOwn"](wrapped, "a") ||\n'
    '  !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '  !globalThis["Object"]["hasOwn"](wrapped, "a") ||\n'
    '  !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '  !globalThis["Object"].hasOwn(wrapped, "a") ||\n'
    '  !singleQuotedHasOwn(wrapped, "a") ||\n'
    '  !parenthesizedSingleQuotedHasOwn(wrapped, "a") ||\n'
    '  !frozenSingleQuotedHasOwn(wrapped, "a") ||\n'
    '  !frozenParenthesizedSingleQuotedHasOwn(wrapped, "a") ||\n'
    '  !Object.freeze(Object.hasOwn)(wrapped, "a") || !Object.freeze((Object.hasOwn))(wrapped, "a") || !Object.freeze(Object["hasOwn"])(wrapped, "a") || !Object.freeze((Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis.Object.hasOwn)(wrapped, "a") || !Object.freeze((globalThis.Object.hasOwn))(wrapped, "a") || !Object.freeze(globalThis.Object["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis.Object[\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis.Object)["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis.Object).hasOwn)(wrapped, "a") || !Object.freeze((globalThis.Object)[\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis.Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis?.Object.hasOwn)(wrapped, "a") || !Object.freeze((globalThis?.Object.hasOwn))(wrapped, "a") || !Object.freeze((globalThis?.Object).hasOwn)(wrapped, "a") || !Object.freeze((globalThis?.Object)["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis?.Object["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis?.Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis["Object"].hasOwn)(wrapped, "a") || !Object.freeze((globalThis["Object"].hasOwn))(wrapped, "a") || !Object.freeze((globalThis["Object"]).hasOwn)(wrapped, "a") || !Object.freeze((globalThis["Object"])["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis["Object"]["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["hasOwn"]))(wrapped, "a") || !Object.freeze((globalThis["Object"]))["hasOwn"](wrapped, "a") || !Object.freeze((globalThis["Object"]))[\'hasOwn\'](wrapped, "a") || !Object.freeze((globalThis[\'Object\']))["hasOwn"](wrapped, "a") || !Object.freeze((globalThis[\'Object\']))[\'hasOwn\'](wrapped, "a") || !Object.freeze(globalThis[\'Object\'].hasOwn)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].hasOwn))(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).hasOwn)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'])[\'hasOwn\'])(wrapped, "a") || !Object.freeze(globalThis[\'Object\'][\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'][\'hasOwn\']))(wrapped, "a") || !Object.freeze((null ?? Object.hasOwn))(wrapped, "a") || !Object.freeze((true && Object.hasOwn))(wrapped, "a") || !Object.freeze((false || Object.hasOwn))(wrapped, "a") || !Object.freeze(globalThis.Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call)(wrapped, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call))(wrapped, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']))(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call))(wrapped, "a") || !Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(globalThis["Object"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object.prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((Object.prototype["hasOwnProperty"].call))(wrapped, "a") || !Object.freeze(Object["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((Object["prototype"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object["prototype"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((Object["prototype"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze((null ?? Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((true && Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((false || Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object.prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((Object.prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze(Object["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((Object["prototype"].hasOwnProperty["call"]))(wrapped, "a") ||\n'
    '  !Object.prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '  !Object["hasOwnProperty"].call(wrapped, "a") ||\n'
    '  !Object["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '  !globalThis.Object.hasOwnProperty.call(wrapped, "a") ||\n'
    '  !globalThis["Object"]["hasOwnProperty"].call(wrapped, "a") ||\n'
    '  !globalThis["Object"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '  !globalThis["Object"].hasOwnProperty.call(wrapped, "a") ||\n'
    '  !hasOwnPropertyCall(wrapped, "a") ||\n'
    '  !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '  !globalThis.Object.prototype.hasOwnProperty["call"](wrapped, "a") ||\n'
    '  !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '  !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '  !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '  !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||\n'
    '  !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '  !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")\n'
    ') {\n'
    "  throw new Error('unexpected browser Object.hasOwn result');\n"
    '}\n'
    "console.log('browser object hasOwn ok');\n"
)

CAP_HAS_OWN_HARNESS_TEST = (
    "Kali.test('object hasOwn primitive literals', () => {\n"
    '  const object = Object.fromEntries([["a", 1], ["b", 2]]);\n'
    '  const alias = object;\n'
    '  const hasOwn = Object.hasOwn;\n'
    '  const hasOwnPropertyCall = Object.prototype.hasOwnProperty.call;\n'
    '  Object.freeze(Object.hasOwn); Object.freeze((Object.hasOwn)); Object.freeze(Object["hasOwn"]); Object.freeze((Object["hasOwn"])); Object.freeze(globalThis.Object.hasOwn); Object.freeze((globalThis.Object.hasOwn)); Object.freeze(globalThis.Object["hasOwn"]); Object.freeze(globalThis.Object[\'hasOwn\']); Object.freeze((globalThis.Object)["hasOwn"]); Object.freeze((globalThis.Object).hasOwn); Object.freeze((globalThis.Object)[\'hasOwn\']); Object.freeze((globalThis.Object["hasOwn"])); Object.freeze(globalThis?.Object.hasOwn); Object.freeze((globalThis?.Object.hasOwn)); Object.freeze((globalThis?.Object).hasOwn); Object.freeze((globalThis?.Object)["hasOwn"]); Object.freeze(globalThis?.Object["hasOwn"]); Object.freeze((globalThis?.Object["hasOwn"])); Object.freeze(globalThis["Object"].hasOwn); Object.freeze((globalThis["Object"].hasOwn)); Object.freeze((globalThis["Object"]).hasOwn); Object.freeze((globalThis["Object"])["hasOwn"]); Object.freeze(globalThis["Object"]["hasOwn"]); Object.freeze((globalThis["Object"]["hasOwn"])); Object.freeze((globalThis["Object"]))["hasOwn"]; Object.freeze((globalThis["Object"]))[\'hasOwn\']; Object.freeze((globalThis[\'Object\']))["hasOwn"]; Object.freeze((globalThis[\'Object\']))[\'hasOwn\']; Object.freeze(globalThis[\'Object\'].hasOwn); Object.freeze((globalThis[\'Object\'].hasOwn)); Object.freeze((globalThis[\'Object\']).hasOwn); Object.freeze((globalThis[\'Object\'])[\'hasOwn\']); Object.freeze(globalThis[\'Object\'][\'hasOwn\']); Object.freeze((globalThis[\'Object\'][\'hasOwn\'])); Object.freeze((null ?? Object.hasOwn)); Object.freeze((true && Object.hasOwn)); Object.freeze((false || Object.hasOwn)); Object.freeze(globalThis.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call)); Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"]); Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"]); Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"])); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call)); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"]); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"])); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call)); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call); Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"])); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call); Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call)); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\']); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)); Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"].hasOwnProperty.call); Object.freeze((globalThis["Object"].hasOwnProperty.call)); Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"])); Object.freeze(Object.prototype.hasOwnProperty.call); Object.freeze((Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype["hasOwnProperty"].call); Object.freeze((Object.prototype["hasOwnProperty"].call)); Object.freeze(Object["prototype"].hasOwnProperty.call); Object.freeze((Object["prototype"].hasOwnProperty.call)); Object.freeze(Object["prototype"]["hasOwnProperty"]["call"]); Object.freeze((Object["prototype"]["hasOwnProperty"]["call"])); Object.freeze((null ?? Object.prototype.hasOwnProperty.call)); Object.freeze((true && Object.prototype.hasOwnProperty.call)); Object.freeze((false || Object.prototype.hasOwnProperty.call)); Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype.hasOwnProperty["call"]); Object.freeze((Object.prototype.hasOwnProperty["call"])); Object.freeze(Object["prototype"].hasOwnProperty["call"]); Object.freeze((Object["prototype"].hasOwnProperty["call"]));\n'
    '  const wrapped = (0, alias);\n'
    "  const singleQuotedHasOwn = globalThis['Object']['hasOwn'];\n"
    "  const parenthesizedSingleQuotedHasOwn = (globalThis['Object'])['hasOwn'];\n"
    "  const frozenSingleQuotedHasOwn = Object.freeze(globalThis['Object']['hasOwn']);\n"
    "  const frozenParenthesizedSingleQuotedHasOwn = Object.freeze((globalThis['Object'])['hasOwn']);\n"
    '  if (\n'
    '    !Object.hasOwn(wrapped, "a") ||\n'
    '    !hasOwn(wrapped, "a") ||\n'
    '    !Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"].hasOwn(wrapped, "a") ||\n'
    '    !singleQuotedHasOwn(wrapped, "a") ||\n'
    '    !parenthesizedSingleQuotedHasOwn(wrapped, "a") ||\n'
    '    !frozenSingleQuotedHasOwn(wrapped, "a") ||\n'
    '    !frozenParenthesizedSingleQuotedHasOwn(wrapped, "a") ||\n'
    '    !Object.freeze(Object.hasOwn)(wrapped, "a") || !Object.freeze((Object.hasOwn))(wrapped, "a") || !Object.freeze(Object["hasOwn"])(wrapped, "a") || !Object.freeze((Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis.Object.hasOwn)(wrapped, "a") || !Object.freeze((globalThis.Object.hasOwn))(wrapped, "a") || !Object.freeze(globalThis.Object["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis.Object[\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis.Object)["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis.Object).hasOwn)(wrapped, "a") || !Object.freeze((globalThis.Object)[\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis.Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis?.Object.hasOwn)(wrapped, "a") || !Object.freeze((globalThis?.Object.hasOwn))(wrapped, "a") || !Object.freeze((globalThis?.Object).hasOwn)(wrapped, "a") || !Object.freeze((globalThis?.Object)["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis?.Object["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis?.Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis["Object"].hasOwn)(wrapped, "a") || !Object.freeze((globalThis["Object"].hasOwn))(wrapped, "a") || !Object.freeze((globalThis["Object"]).hasOwn)(wrapped, "a") || !Object.freeze((globalThis["Object"])["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis["Object"]["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["hasOwn"]))(wrapped, "a") || !Object.freeze((globalThis["Object"]))["hasOwn"](wrapped, "a") || !Object.freeze((globalThis["Object"]))[\'hasOwn\'](wrapped, "a") || !Object.freeze((globalThis[\'Object\']))["hasOwn"](wrapped, "a") || !Object.freeze((globalThis[\'Object\']))[\'hasOwn\'](wrapped, "a") || !Object.freeze(globalThis[\'Object\'].hasOwn)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].hasOwn))(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).hasOwn)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'])[\'hasOwn\'])(wrapped, "a") || !Object.freeze(globalThis[\'Object\'][\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'][\'hasOwn\']))(wrapped, "a") || !Object.freeze((null ?? Object.hasOwn))(wrapped, "a") || !Object.freeze((true && Object.hasOwn))(wrapped, "a") || !Object.freeze((false || Object.hasOwn))(wrapped, "a") || !Object.freeze(globalThis.Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call)(wrapped, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call))(wrapped, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']))(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call))(wrapped, "a") || !Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(globalThis["Object"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object.prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((Object.prototype["hasOwnProperty"].call))(wrapped, "a") || !Object.freeze(Object["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((Object["prototype"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object["prototype"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((Object["prototype"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze((null ?? Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((true && Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((false || Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object.prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((Object.prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze(Object["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((Object["prototype"].hasOwnProperty["call"]))(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !Object["hasOwnProperty"].call(wrapped, "a") ||\n'
    '    !Object["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis.Object.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis["Object"]["hasOwnProperty"].call(wrapped, "a") ||\n'
    '    !globalThis["Object"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].hasOwnProperty.call(wrapped, "a") ||\n'
    '    !hasOwnPropertyCall(wrapped, "a") ||\n'
    '    !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis.Object.prototype.hasOwnProperty["call"](wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")\n'
    '  ) {\n'
    "    throw new Error('unexpected browser Object.hasOwn result');\n"
    '  }\n'
    "  console.log('browser object hasOwn ok');\n"
    '});\n'
)

CAP_HOFE_BUNDLE_PLAIN = (
    '// kali-tree-shake: browserObjectHasOwnFromEntries\n'
    'function browserObjectHasOwnFromEntries() {\n'
    '  const object = Object.fromEntries([["a", 1], ["b", 2]]);\n'
    '  const alias = object;\n'
    '  const wrapped = (0, alias);\n'
    '  const awaited = wrapped;\n'
    '  const conditionalHasOwn = Object.freeze((true ? Object.hasOwn : Object.hasOwn));\n'
    '  const conditionalHasOwnPropertyCall = Object.freeze((true ? Object.prototype.hasOwnProperty.call : Object.prototype.hasOwnProperty.call));\n'
    '  if (\n'
    '    !Object.hasOwn(wrapped, "a") ||\n'
    '    !Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"].hasOwn(wrapped, "a") ||\n'
    '    !Object.hasOwn(awaited, "a") ||\n'
    '    !conditionalHasOwn(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(awaited, "a") ||\n'
    '    !conditionalHasOwnPropertyCall(wrapped, "a") ||\n'
    '    !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")\n'
    '  ) {\n'
    "    throw new Error('unexpected browser Object.hasOwn result');\n"
    '  }\n'
    "  console.log('browser object hasOwn fromEntries ok');\n"
    '}\n'
)

CAP_HOFE_BUNDLE_FROZEN = (
    '// kali-tree-shake: browserObjectHasOwnFromEntries\n'
    'function browserObjectHasOwnFromEntries() {\n'
    '  const object = Object.freeze(Object.fromEntries([["a", 1], ["b", 2]]);\n'
    '  const alias = object;\n'
    '  const wrapped = (0, alias);\n'
    '  const awaited = wrapped;\n'
    '  const conditionalHasOwn = Object.freeze((true ? Object.hasOwn : Object.hasOwn));\n'
    '  const conditionalHasOwnPropertyCall = Object.freeze((true ? Object.prototype.hasOwnProperty.call : Object.prototype.hasOwnProperty.call));\n'
    '  if (\n'
    '    !Object.hasOwn(wrapped, "a") ||\n'
    '    !Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"].hasOwn(wrapped, "a") ||\n'
    '    !Object.hasOwn(awaited, "a") ||\n'
    '    !conditionalHasOwn(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(awaited, "a") ||\n'
    '    !conditionalHasOwnPropertyCall(wrapped, "a") ||\n'
    '    !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")\n'
    '  ) {\n'
    "    throw new Error('unexpected browser Object.hasOwn result');\n"
    '  }\n'
    "  console.log('browser object hasOwn fromEntries ok');\n"
    '}\n'
)

CAP_HOFE_RUN_PLAIN = (
    'function browserObjectHasOwnFromEntries() {\n'
    '  const object = Object.fromEntries([["a", 1], ["b", 2]]);\n'
    '  const alias = object;\n'
    '  const wrapped = (0, alias);\n'
    '  const awaited = wrapped;\n'
    '  const conditionalHasOwn = Object.freeze((true ? Object.hasOwn : Object.hasOwn));\n'
    '  const conditionalHasOwnPropertyCall = Object.freeze((true ? Object.prototype.hasOwnProperty.call : Object.prototype.hasOwnProperty.call));\n'
    '  if (\n'
    '    !Object.hasOwn(wrapped, "a") ||\n'
    '    !Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"].hasOwn(wrapped, "a") ||\n'
    '    !Object.hasOwn(awaited, "a") ||\n'
    '    !conditionalHasOwn(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(awaited, "a") ||\n'
    '    !conditionalHasOwnPropertyCall(wrapped, "a") ||\n'
    '    !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")\n'
    '  ) {\n'
    "    throw new Error('unexpected browser Object.hasOwn result');\n"
    '  }\n'
    "  console.log('browser object hasOwn fromEntries ok');\n"
    '}\n'
    'browserObjectHasOwnFromEntries();\n'
)

CAP_HOFE_RUN_FROZEN = (
    'function browserObjectHasOwnFromEntries() {\n'
    '  const object = Object.freeze(Object.fromEntries([["a", 1], ["b", 2]]);\n'
    '  const alias = object;\n'
    '  const wrapped = (0, alias);\n'
    '  const awaited = wrapped;\n'
    '  const conditionalHasOwn = Object.freeze((true ? Object.hasOwn : Object.hasOwn));\n'
    '  const conditionalHasOwnPropertyCall = Object.freeze((true ? Object.prototype.hasOwnProperty.call : Object.prototype.hasOwnProperty.call));\n'
    '  if (\n'
    '    !Object.hasOwn(wrapped, "a") ||\n'
    '    !Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"].hasOwn(wrapped, "a") ||\n'
    '    !Object.hasOwn(awaited, "a") ||\n'
    '    !conditionalHasOwn(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(awaited, "a") ||\n'
    '    !conditionalHasOwnPropertyCall(wrapped, "a") ||\n'
    '    !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")\n'
    '  ) {\n'
    "    throw new Error('unexpected browser Object.hasOwn result');\n"
    '  }\n'
    "  console.log('browser object hasOwn fromEntries ok');\n"
    '}\n'
    'browserObjectHasOwnFromEntries();\n'
)

CAP_HOFE_TEST_PLAIN = (
    "Kali.test('object hasOwn fromEntries', () => {\n"
    '  const object = Object.fromEntries([["a", 1], ["b", 2]]);\n'
    '  const alias = object;\n'
    '  const wrapped = (0, alias);\n'
    '  const awaited = wrapped;\n'
    '  const conditionalHasOwn = Object.freeze((true ? Object.hasOwn : Object.hasOwn));\n'
    '  const conditionalHasOwnPropertyCall = Object.freeze((true ? Object.prototype.hasOwnProperty.call : Object.prototype.hasOwnProperty.call));\n'
    '  if (\n'
    '    !Object.hasOwn(wrapped, "a") ||\n'
    '    !Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"].hasOwn(wrapped, "a") ||\n'
    '    !Object.hasOwn(awaited, "a") ||\n'
    '    !conditionalHasOwn(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(awaited, "a") ||\n'
    '    !conditionalHasOwnPropertyCall(wrapped, "a") ||\n'
    '    !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")\n'
    '  ) {\n'
    "    throw new Error('unexpected browser Object.hasOwn result');\n"
    '  }\n'
    "  console.log('browser object hasOwn fromEntries ok');\n"
    '});\n'
)

CAP_HOFE_TEST_FROZEN = (
    "Kali.test('object hasOwn fromEntries', () => {\n"
    '  const object = Object.freeze(Object.fromEntries([["a", 1], ["b", 2]]);\n'
    '  const alias = object;\n'
    '  const wrapped = (0, alias);\n'
    '  const awaited = wrapped;\n'
    '  const conditionalHasOwn = Object.freeze((true ? Object.hasOwn : Object.hasOwn));\n'
    '  const conditionalHasOwnPropertyCall = Object.freeze((true ? Object.prototype.hasOwnProperty.call : Object.prototype.hasOwnProperty.call));\n'
    '  if (\n'
    '    !Object.hasOwn(wrapped, "a") ||\n'
    '    !Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis.Object["hasOwn"](wrapped, "a") ||\n'
    '    !globalThis["Object"].hasOwn(wrapped, "a") ||\n'
    '    !Object.hasOwn(awaited, "a") ||\n'
    '    !conditionalHasOwn(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(awaited, "a") ||\n'
    '    !conditionalHasOwnPropertyCall(wrapped, "a") ||\n'
    '    !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||\n'
    '    !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")\n'
    '  ) {\n'
    "    throw new Error('unexpected browser Object.hasOwn result');\n"
    '  }\n'
    "  console.log('browser object hasOwn fromEntries ok');\n"
    '});\n'
)

CAP_IS_ALIAS_CHAIN_RUN = (
    'const object = { a: 1 };\n'
    'const objectAlias = object;\n'
    'const frozenObject = Object.freeze(object);\n'
    'const array = [1, 2];\n'
    'const arrayAlias = array;\n'
    'const frozenArray = Object.freeze(array);\n'
    'if (\n'
    '  Object.is(objectAlias, object) !== true ||\n'
    '  globalThis?.Object?.is(objectAlias, object) !== true ||\n'
    '  globalThis["Object"]["is"](objectAlias, object) !== true ||\n'
    '  globalThis.Object["is"](objectAlias, object) !== true ||\n'
    '  globalThis["Object"].is(objectAlias, object) !== true ||\n'
    '  globalThis.Object.is(objectAlias, object) !== true ||\n'
    '  Object["is"](objectAlias, object) !== true ||\n'
    '  Object.is(frozenObject, object) !== true ||\n'
    '  globalThis["Object"]["is"](frozenObject, object) !== true ||\n'
    '  globalThis.Object["is"](frozenObject, object) !== true ||\n'
    '  globalThis["Object"].is(frozenObject, object) !== true ||\n'
    '  globalThis.Object.is(frozenObject, object) !== true ||\n'
    '  Object["is"](frozenObject, object) !== true ||\n'
    '  Object.is(arrayAlias, array) !== true ||\n'
    '  Object.is(frozenArray, array) !== true ||\n'
    '  Object.is({}, {}) !== false ||\n'
    '  Object.is([], []) !== false\n'
    ') {\n'
    "  throw new Error('unexpected browser Object.is alias chain result');\n"
    '}\n'
    "console.log('browser object is alias chain ok');\n"
)

CAP_IS_ALIAS_CHAIN_TEST = (
    "Kali.test('browser object is alias chain', () => {\n"
    'const object = { a: 1 };\n'
    'const objectAlias = object;\n'
    'const frozenObject = Object.freeze(object);\n'
    'const array = [1, 2];\n'
    'const arrayAlias = array;\n'
    'const frozenArray = Object.freeze(array);\n'
    'if (\n'
    '  Object.is(objectAlias, object) !== true ||\n'
    '  globalThis?.Object?.is(objectAlias, object) !== true ||\n'
    '  globalThis["Object"]["is"](objectAlias, object) !== true ||\n'
    '  globalThis.Object["is"](objectAlias, object) !== true ||\n'
    '  globalThis["Object"].is(objectAlias, object) !== true ||\n'
    '  globalThis.Object.is(objectAlias, object) !== true ||\n'
    '  Object["is"](objectAlias, object) !== true ||\n'
    '  Object.is(frozenObject, object) !== true ||\n'
    '  globalThis["Object"]["is"](frozenObject, object) !== true ||\n'
    '  globalThis.Object["is"](frozenObject, object) !== true ||\n'
    '  globalThis["Object"].is(frozenObject, object) !== true ||\n'
    '  globalThis.Object.is(frozenObject, object) !== true ||\n'
    '  Object["is"](frozenObject, object) !== true ||\n'
    '  Object.is(arrayAlias, array) !== true ||\n'
    '  Object.is(frozenArray, array) !== true ||\n'
    '  Object.is({}, {}) !== false ||\n'
    '  Object.is([], []) !== false\n'
    ') {\n'
    "  throw new Error('unexpected browser Object.is alias chain result');\n"
    '}\n'
    '});\n'
)
