"""Rule-8 / rule-9 captured fixture texts for Task 18 batch 7B.

Every constant here is the BYTE-EXACT OUTPUT OF EXECUTING THE REAL CODE, never a
hand-applied `str::replace` and never a retyped approximation. Rule 9 extends
rule 8's "never hand-simulate" discipline to any fixture built one level removed
from a plain string literal, and this batch has exactly one such shape: a
`str::replace` WHOSE NEEDLE CARRIES TWO LEADING SPACES,

    browser_harness_object_keys_entries_spread_source(test_mode).replace(
        "  const fromEntries = Object.fromEntries(...);",
        "  const fromEntries = Object.freeze(Object.fromEntries(...));")

(`browser_object_keys_entries_spread_harness.rs` and
`browser_object_values_harness.rs` each carry one). Hand-deriving the resulting
indentation, and hand-deciding how many occurrences `str::replace` rewrites, is
precisely the trap rule 8 exists to prevent.

HOW THEY WERE CAPTURED, so they can be re-derived.

A temporary target `crates/kali_cli/tests/zz_b7b_dump.rs`, deleted in the same
session, with one `mod` per source that `include!`d the shipped `.rs` and a
`#[test] fn zz_dump_*` inside that `mod` (the fixture builders are private, so
the dump has to live in the module that includes them). Run as

    ZZ_B7B_OUT=<dir> cargo test -p kali_cli --test zz_b7b_dump -- zz_dump \
        --test-threads=1

`include!` rather than a retyped copy, so the executed `replace` is literally
the one in the shipped source. Every constant below came from that one run;
none was edited afterwards.

WHY THE *_PLAIN CONSTANTS ARE HERE TOO, when the lexer could pull them straight
out of the `.rs`: so the plain/frozen pair is PROVED to differ by exactly the
`Object.freeze(` wrap rather than assumed to. `gen_batch7b.assert_frozen_pair`
diffs each pair and raises if the difference is anything else, and it also
re-checks each *_PLAIN constant against the literal the lexer extracts from the
shipped source -- so a stale capture (taken before a source edit) fails the
generator instead of shipping a program that is no longer the program under
test.

THE TWO HARNESS CAPTURES AT THE BOTTOM are here for the same reason, one level
across. `browser_object_keys_entries_spread_bundle.rs` takes an OR-shaped claim
on the BROWSER-BUNDLE HARNESS's own streams, so resolving it under rule 11 means
running the script the case RUNNER runs -- and that script is built by a
`format!` inside `kali_runtime_contract`, which rule 8 forbids hand-simulating.
`CAP_HARNESS_PRELUDE_APP` is `browser_bundle_harness_prelude("app", false)` and
`CAP_HARNESS_SCRIPT_APP_ENTRIES_SPREAD` is the complete
`browser_bundle_harness_script("app", false, <that file's body>)`, both from the
same dump run. `gen_batch7b.harness_script` asserts the second equals the first
concatenated with the body before composing any other script that way, so the
composition rule it relies on is proved from the real code rather than read off
the helper's source.

They are embedded here rather than read from a dump file so this module runs
from a clean checkout with no uncommitted inputs -- the defect that got the
pilot's per-file generators deleted (see README).
"""


CAP_ENTRIES_SPREAD_RUN_PLAIN = (
    'function browserObjectKeysEntriesSpreadIteration() {\n'
    '  function assertObjectKeysIteration(keys) {\n'
    "    if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {\n"
    "      throw new Error('unexpected Object.keys spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  function assertObjectEntriesIteration(entries) {\n'
    '    if (\n'
    '      entries.length !== 2 ||\n'
    "      entries[0][0] !== 'b' ||\n"
    '      entries[0][1] !== 3 ||\n'
    "      entries[1][0] !== 'a' ||\n"
    '      entries[1][1] !== 2\n'
    '    ) {\n'
    "      throw new Error('unexpected Object.entries spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);\n'
    '  const collectedKeys = [...Object.keys(fromEntries)];\n'
    '  const globalKeys = [...globalThis.Object.keys(fromEntries)];\n'
    '  const mixedKeys = [...globalThis.Object["keys"](fromEntries)];\n'
    '  const mixedBracketedKeys = [...globalThis["Object"].keys(fromEntries)];\n'
    '  const bracketedKeys = [...globalThis["Object"]["keys"](fromEntries)];\n'
    "  const singleBracketedKeys = [...globalThis['Object']['keys'](fromEntries)];\n"
    '  const parenthesizedReceiverBracketedKeys = [...Object.freeze((globalThis["Object"])["keys"])(fromEntries)];\n'
    "  const parenthesizedSingleQuotedReceiverBracketedKeys = [...Object.freeze((globalThis['Object'])['keys'])(fromEntries)];\n"
    "  const parenthesizedSingleQuotedReceiverPropertyKeys = [...Object.freeze((globalThis['Object']).keys)(fromEntries)];\n"
    '  const parenthesizedBracketedKeys = [...Object.freeze((globalThis["Object"]).keys)(fromEntries)];\n'
    '  const collectedEntries = [...Object.entries(fromEntries)];\n'
    '  const globalEntries = [...globalThis.Object.entries(fromEntries)];\n'
    '  const mixedEntries = [...globalThis.Object["entries"](fromEntries)];\n'
    '  const mixedBracketedEntries = [...globalThis["Object"].entries(fromEntries)];\n'
    '  const bracketedEntries = [...globalThis["Object"]["entries"](fromEntries)];\n'
    "  const singleBracketedEntries = [...globalThis['Object']['entries'](fromEntries)];\n"
    '  const parenthesizedReceiverBracketedEntries = [...Object.freeze((globalThis["Object"])["entries"])(fromEntries)];\n'
    "  const parenthesizedSingleQuotedReceiverBracketedEntries = [...Object.freeze((globalThis['Object'])['entries'])(fromEntries)];\n"
    '  const parenthesizedBracketedEntries = [...Object.freeze((globalThis["Object"]).entries)(fromEntries)];\n'
    '\n'
    '  assertObjectKeysIteration(collectedKeys);\n'
    '  assertObjectKeysIteration(globalKeys);\n'
    '  assertObjectKeysIteration(mixedKeys);\n'
    '  assertObjectKeysIteration(mixedBracketedKeys);\n'
    '  assertObjectKeysIteration(bracketedKeys);\n'
    '  assertObjectKeysIteration(singleBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedReceiverBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverPropertyKeys);\n'
    '  assertObjectKeysIteration(parenthesizedBracketedKeys);\n'
    '  assertObjectEntriesIteration(collectedEntries);\n'
    '  assertObjectEntriesIteration(globalEntries);\n'
    '  assertObjectEntriesIteration(mixedEntries);\n'
    '  assertObjectEntriesIteration(mixedBracketedEntries);\n'
    '  assertObjectEntriesIteration(bracketedEntries);\n'
    '  assertObjectEntriesIteration(singleBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedSingleQuotedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedBracketedEntries);\n'
    "  console.log('browser object keys and entries spread iteration ok');\n"
    '}\n'
    '\n'
    'browserObjectKeysEntriesSpreadIteration();\n'
)

CAP_ENTRIES_SPREAD_TEST_PLAIN = (
    "Kali.test('object keys and entries spread iteration', () => {\n"
    '  function assertObjectKeysIteration(keys) {\n'
    "    if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {\n"
    "      throw new Error('unexpected Object.keys spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  function assertObjectEntriesIteration(entries) {\n'
    '    if (\n'
    '      entries.length !== 2 ||\n'
    "      entries[0][0] !== 'b' ||\n"
    '      entries[0][1] !== 3 ||\n'
    "      entries[1][0] !== 'a' ||\n"
    '      entries[1][1] !== 2\n'
    '    ) {\n'
    "      throw new Error('unexpected Object.entries spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);\n'
    '  const collectedKeys = [...Object.keys(fromEntries)];\n'
    '  const globalKeys = [...globalThis.Object.keys(fromEntries)];\n'
    '  const mixedKeys = [...globalThis.Object["keys"](fromEntries)];\n'
    '  const mixedBracketedKeys = [...globalThis["Object"].keys(fromEntries)];\n'
    '  const bracketedKeys = [...globalThis["Object"]["keys"](fromEntries)];\n'
    "  const singleBracketedKeys = [...globalThis['Object']['keys'](fromEntries)];\n"
    '  const parenthesizedReceiverBracketedKeys = [...Object.freeze((globalThis["Object"])["keys"])(fromEntries)];\n'
    "  const parenthesizedSingleQuotedReceiverBracketedKeys = [...Object.freeze((globalThis['Object'])['keys'])(fromEntries)];\n"
    "  const parenthesizedSingleQuotedReceiverPropertyKeys = [...Object.freeze((globalThis['Object']).keys)(fromEntries)];\n"
    '  const parenthesizedBracketedKeys = [...Object.freeze((globalThis["Object"]).keys)(fromEntries)];\n'
    '  const collectedEntries = [...Object.entries(fromEntries)];\n'
    '  const globalEntries = [...globalThis.Object.entries(fromEntries)];\n'
    '  const mixedEntries = [...globalThis.Object["entries"](fromEntries)];\n'
    '  const mixedBracketedEntries = [...globalThis["Object"].entries(fromEntries)];\n'
    '  const bracketedEntries = [...globalThis["Object"]["entries"](fromEntries)];\n'
    "  const singleBracketedEntries = [...globalThis['Object']['entries'](fromEntries)];\n"
    '  const parenthesizedReceiverBracketedEntries = [...Object.freeze((globalThis["Object"])["entries"])(fromEntries)];\n'
    "  const parenthesizedSingleQuotedReceiverBracketedEntries = [...Object.freeze((globalThis['Object'])['entries'])(fromEntries)];\n"
    '  const parenthesizedBracketedEntries = [...Object.freeze((globalThis["Object"]).entries)(fromEntries)];\n'
    '\n'
    '  assertObjectKeysIteration(collectedKeys);\n'
    '  assertObjectKeysIteration(globalKeys);\n'
    '  assertObjectKeysIteration(mixedKeys);\n'
    '  assertObjectKeysIteration(mixedBracketedKeys);\n'
    '  assertObjectKeysIteration(bracketedKeys);\n'
    '  assertObjectKeysIteration(singleBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedReceiverBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverPropertyKeys);\n'
    '  assertObjectKeysIteration(parenthesizedBracketedKeys);\n'
    '  assertObjectEntriesIteration(collectedEntries);\n'
    '  assertObjectEntriesIteration(globalEntries);\n'
    '  assertObjectEntriesIteration(mixedEntries);\n'
    '  assertObjectEntriesIteration(mixedBracketedEntries);\n'
    '  assertObjectEntriesIteration(bracketedEntries);\n'
    '  assertObjectEntriesIteration(singleBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedSingleQuotedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedBracketedEntries);\n'
    "  console.log('browser object keys and entries spread iteration ok');\n"
    '});\n'
)

CAP_ENTRIES_SPREAD_RUN_FROZEN = (
    'function browserObjectKeysEntriesSpreadIteration() {\n'
    '  function assertObjectKeysIteration(keys) {\n'
    "    if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {\n"
    "      throw new Error('unexpected Object.keys spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  function assertObjectEntriesIteration(entries) {\n'
    '    if (\n'
    '      entries.length !== 2 ||\n'
    "      entries[0][0] !== 'b' ||\n"
    '      entries[0][1] !== 3 ||\n'
    "      entries[1][0] !== 'a' ||\n"
    '      entries[1][1] !== 2\n'
    '    ) {\n'
    "      throw new Error('unexpected Object.entries spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const fromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));\n'
    '  const collectedKeys = [...Object.keys(fromEntries)];\n'
    '  const globalKeys = [...globalThis.Object.keys(fromEntries)];\n'
    '  const mixedKeys = [...globalThis.Object["keys"](fromEntries)];\n'
    '  const mixedBracketedKeys = [...globalThis["Object"].keys(fromEntries)];\n'
    '  const bracketedKeys = [...globalThis["Object"]["keys"](fromEntries)];\n'
    "  const singleBracketedKeys = [...globalThis['Object']['keys'](fromEntries)];\n"
    '  const parenthesizedReceiverBracketedKeys = [...Object.freeze((globalThis["Object"])["keys"])(fromEntries)];\n'
    "  const parenthesizedSingleQuotedReceiverBracketedKeys = [...Object.freeze((globalThis['Object'])['keys'])(fromEntries)];\n"
    "  const parenthesizedSingleQuotedReceiverPropertyKeys = [...Object.freeze((globalThis['Object']).keys)(fromEntries)];\n"
    '  const parenthesizedBracketedKeys = [...Object.freeze((globalThis["Object"]).keys)(fromEntries)];\n'
    '  const collectedEntries = [...Object.entries(fromEntries)];\n'
    '  const globalEntries = [...globalThis.Object.entries(fromEntries)];\n'
    '  const mixedEntries = [...globalThis.Object["entries"](fromEntries)];\n'
    '  const mixedBracketedEntries = [...globalThis["Object"].entries(fromEntries)];\n'
    '  const bracketedEntries = [...globalThis["Object"]["entries"](fromEntries)];\n'
    "  const singleBracketedEntries = [...globalThis['Object']['entries'](fromEntries)];\n"
    '  const parenthesizedReceiverBracketedEntries = [...Object.freeze((globalThis["Object"])["entries"])(fromEntries)];\n'
    "  const parenthesizedSingleQuotedReceiverBracketedEntries = [...Object.freeze((globalThis['Object'])['entries'])(fromEntries)];\n"
    '  const parenthesizedBracketedEntries = [...Object.freeze((globalThis["Object"]).entries)(fromEntries)];\n'
    '\n'
    '  assertObjectKeysIteration(collectedKeys);\n'
    '  assertObjectKeysIteration(globalKeys);\n'
    '  assertObjectKeysIteration(mixedKeys);\n'
    '  assertObjectKeysIteration(mixedBracketedKeys);\n'
    '  assertObjectKeysIteration(bracketedKeys);\n'
    '  assertObjectKeysIteration(singleBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedReceiverBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverPropertyKeys);\n'
    '  assertObjectKeysIteration(parenthesizedBracketedKeys);\n'
    '  assertObjectEntriesIteration(collectedEntries);\n'
    '  assertObjectEntriesIteration(globalEntries);\n'
    '  assertObjectEntriesIteration(mixedEntries);\n'
    '  assertObjectEntriesIteration(mixedBracketedEntries);\n'
    '  assertObjectEntriesIteration(bracketedEntries);\n'
    '  assertObjectEntriesIteration(singleBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedSingleQuotedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedBracketedEntries);\n'
    "  console.log('browser object keys and entries spread iteration ok');\n"
    '}\n'
    '\n'
    'browserObjectKeysEntriesSpreadIteration();\n'
)

CAP_ENTRIES_SPREAD_TEST_FROZEN = (
    "Kali.test('object keys and entries spread iteration', () => {\n"
    '  function assertObjectKeysIteration(keys) {\n'
    "    if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {\n"
    "      throw new Error('unexpected Object.keys spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  function assertObjectEntriesIteration(entries) {\n'
    '    if (\n'
    '      entries.length !== 2 ||\n'
    "      entries[0][0] !== 'b' ||\n"
    '      entries[0][1] !== 3 ||\n'
    "      entries[1][0] !== 'a' ||\n"
    '      entries[1][1] !== 2\n'
    '    ) {\n'
    "      throw new Error('unexpected Object.entries spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const fromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));\n'
    '  const collectedKeys = [...Object.keys(fromEntries)];\n'
    '  const globalKeys = [...globalThis.Object.keys(fromEntries)];\n'
    '  const mixedKeys = [...globalThis.Object["keys"](fromEntries)];\n'
    '  const mixedBracketedKeys = [...globalThis["Object"].keys(fromEntries)];\n'
    '  const bracketedKeys = [...globalThis["Object"]["keys"](fromEntries)];\n'
    "  const singleBracketedKeys = [...globalThis['Object']['keys'](fromEntries)];\n"
    '  const parenthesizedReceiverBracketedKeys = [...Object.freeze((globalThis["Object"])["keys"])(fromEntries)];\n'
    "  const parenthesizedSingleQuotedReceiverBracketedKeys = [...Object.freeze((globalThis['Object'])['keys'])(fromEntries)];\n"
    "  const parenthesizedSingleQuotedReceiverPropertyKeys = [...Object.freeze((globalThis['Object']).keys)(fromEntries)];\n"
    '  const parenthesizedBracketedKeys = [...Object.freeze((globalThis["Object"]).keys)(fromEntries)];\n'
    '  const collectedEntries = [...Object.entries(fromEntries)];\n'
    '  const globalEntries = [...globalThis.Object.entries(fromEntries)];\n'
    '  const mixedEntries = [...globalThis.Object["entries"](fromEntries)];\n'
    '  const mixedBracketedEntries = [...globalThis["Object"].entries(fromEntries)];\n'
    '  const bracketedEntries = [...globalThis["Object"]["entries"](fromEntries)];\n'
    "  const singleBracketedEntries = [...globalThis['Object']['entries'](fromEntries)];\n"
    '  const parenthesizedReceiverBracketedEntries = [...Object.freeze((globalThis["Object"])["entries"])(fromEntries)];\n'
    "  const parenthesizedSingleQuotedReceiverBracketedEntries = [...Object.freeze((globalThis['Object'])['entries'])(fromEntries)];\n"
    '  const parenthesizedBracketedEntries = [...Object.freeze((globalThis["Object"]).entries)(fromEntries)];\n'
    '\n'
    '  assertObjectKeysIteration(collectedKeys);\n'
    '  assertObjectKeysIteration(globalKeys);\n'
    '  assertObjectKeysIteration(mixedKeys);\n'
    '  assertObjectKeysIteration(mixedBracketedKeys);\n'
    '  assertObjectKeysIteration(bracketedKeys);\n'
    '  assertObjectKeysIteration(singleBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedReceiverBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverBracketedKeys);\n'
    '  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverPropertyKeys);\n'
    '  assertObjectKeysIteration(parenthesizedBracketedKeys);\n'
    '  assertObjectEntriesIteration(collectedEntries);\n'
    '  assertObjectEntriesIteration(globalEntries);\n'
    '  assertObjectEntriesIteration(mixedEntries);\n'
    '  assertObjectEntriesIteration(mixedBracketedEntries);\n'
    '  assertObjectEntriesIteration(bracketedEntries);\n'
    '  assertObjectEntriesIteration(singleBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedSingleQuotedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedBracketedEntries);\n'
    "  console.log('browser object keys and entries spread iteration ok');\n"
    '});\n'
)

CAP_VALUES_SPREAD_RUN_PLAIN = (
    'function browserObjectValuesSpreadIteration() {\n'
    '  function assertObjectValuesSpreadIteration(values) {\n'
    '    if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {\n'
    "      throw new Error('unexpected Object.values spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);\n'
    '  const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);\n'
    '  const collected = [...Object.values(fromEntries)];\n'
    '  const globalCollected = [...globalThis.Object.values(fromEntries)];\n'
    '  const bracketedCollected = [...Object.values(bracketedFromEntries)];\n'
    '  const mixedCollected = [...globalThis.Object["values"](fromEntries)];\n'
    '  const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];\n'
    "  const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];\n"
    "  const singleBracketedPropertyCollected = [...globalThis['Object'].values(fromEntries)];\n"
    '  const bracketedAliasCollected = [...globalThis["Object"]["values"](fromEntries)];\n'
    '  const bracketedAliasFromEntriesCollected = [...globalThis["Object"]["values"](bracketedFromEntries)];\n'
    '  assertObjectValuesSpreadIteration(collected);\n'
    '  assertObjectValuesSpreadIteration(globalCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(mixedCollected);\n'
    '  assertObjectValuesSpreadIteration(mixedBracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(singleBracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(singleBracketedPropertyCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedAliasCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedAliasFromEntriesCollected);\n'
    "  console.log('browser object values spread iteration ok');\n"
    '}\n'
    '\n'
    'browserObjectValuesSpreadIteration();\n'
)

CAP_VALUES_SPREAD_TEST_PLAIN = (
    "Kali.test('object values spread iteration', () => {\n"
    '  function assertObjectValuesSpreadIteration(values) {\n'
    '    if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {\n'
    "      throw new Error('unexpected Object.values spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);\n'
    '  const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);\n'
    '  const collected = [...Object.values(fromEntries)];\n'
    '  const globalCollected = [...globalThis.Object.values(fromEntries)];\n'
    '  const bracketedCollected = [...Object.values(bracketedFromEntries)];\n'
    '  const mixedCollected = [...globalThis.Object["values"](fromEntries)];\n'
    '  const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];\n'
    "  const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];\n"
    "  const singleBracketedPropertyCollected = [...globalThis['Object'].values(fromEntries)];\n"
    '  const bracketedAliasCollected = [...globalThis["Object"]["values"](fromEntries)];\n'
    '  const bracketedAliasFromEntriesCollected = [...globalThis["Object"]["values"](bracketedFromEntries)];\n'
    '  assertObjectValuesSpreadIteration(collected);\n'
    '  assertObjectValuesSpreadIteration(globalCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(mixedCollected);\n'
    '  assertObjectValuesSpreadIteration(mixedBracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(singleBracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(singleBracketedPropertyCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedAliasCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedAliasFromEntriesCollected);\n'
    "  console.log('browser object values spread iteration ok');\n"
    '});\n'
)

CAP_VALUES_SPREAD_RUN_FROZEN = (
    'function browserObjectValuesSpreadIteration() {\n'
    '  function assertObjectValuesSpreadIteration(values) {\n'
    '    if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {\n'
    "      throw new Error('unexpected Object.values spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const fromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));\n'
    '  const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);\n'
    '  const collected = [...Object.values(fromEntries)];\n'
    '  const globalCollected = [...globalThis.Object.values(fromEntries)];\n'
    '  const bracketedCollected = [...Object.values(bracketedFromEntries)];\n'
    '  const mixedCollected = [...globalThis.Object["values"](fromEntries)];\n'
    '  const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];\n'
    "  const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];\n"
    "  const singleBracketedPropertyCollected = [...globalThis['Object'].values(fromEntries)];\n"
    '  const bracketedAliasCollected = [...globalThis["Object"]["values"](fromEntries)];\n'
    '  const bracketedAliasFromEntriesCollected = [...globalThis["Object"]["values"](bracketedFromEntries)];\n'
    '  assertObjectValuesSpreadIteration(collected);\n'
    '  assertObjectValuesSpreadIteration(globalCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(mixedCollected);\n'
    '  assertObjectValuesSpreadIteration(mixedBracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(singleBracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(singleBracketedPropertyCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedAliasCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedAliasFromEntriesCollected);\n'
    "  console.log('browser object values spread iteration ok');\n"
    '}\n'
    '\n'
    'browserObjectValuesSpreadIteration();\n'
)

CAP_VALUES_SPREAD_TEST_FROZEN = (
    "Kali.test('object values spread iteration', () => {\n"
    '  function assertObjectValuesSpreadIteration(values) {\n'
    '    if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {\n'
    "      throw new Error('unexpected Object.values spread iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const fromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));\n'
    '  const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);\n'
    '  const collected = [...Object.values(fromEntries)];\n'
    '  const globalCollected = [...globalThis.Object.values(fromEntries)];\n'
    '  const bracketedCollected = [...Object.values(bracketedFromEntries)];\n'
    '  const mixedCollected = [...globalThis.Object["values"](fromEntries)];\n'
    '  const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];\n'
    "  const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];\n"
    "  const singleBracketedPropertyCollected = [...globalThis['Object'].values(fromEntries)];\n"
    '  const bracketedAliasCollected = [...globalThis["Object"]["values"](fromEntries)];\n'
    '  const bracketedAliasFromEntriesCollected = [...globalThis["Object"]["values"](bracketedFromEntries)];\n'
    '  assertObjectValuesSpreadIteration(collected);\n'
    '  assertObjectValuesSpreadIteration(globalCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(mixedCollected);\n'
    '  assertObjectValuesSpreadIteration(mixedBracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(singleBracketedCollected);\n'
    '  assertObjectValuesSpreadIteration(singleBracketedPropertyCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedAliasCollected);\n'
    '  assertObjectValuesSpreadIteration(bracketedAliasFromEntriesCollected);\n'
    "  console.log('browser object values spread iteration ok');\n"
    '});\n'
)


# the browser-bundle harness PRELUDE for bundle_dir `app`, dumped from the real `kali_runtime_contract` helper in the
# same run as the fixtures above.
CAP_HARNESS_PRELUDE_APP = (
    "import fs from 'node:fs/promises';\n"
    "import { fileURLToPath } from 'node:url';\n"
    '\n'
    "const bundleJs = new URL('./app/app.js', import.meta.url);\n"
    "const wasmUrl = new URL('./app/app.wasm', import.meta.url);\n"
    '\n'
    'globalThis.fetch = async (input) => {\n'
    '  const url = input instanceof URL ? input : new URL(String(input));\n'
    '  if (url.href === wasmUrl.href) {\n'
    '    const bytes = await fs.readFile(fileURLToPath(url));\n'
    "    return new Response(bytes, { headers: { 'content-type': 'application/wasm' } });\n"
    '  }\n'
    '  throw new Error(`unexpected fetch ${String(input)}`);\n'
    '};\n'
    '\n'
)


# one complete harness script, dumped from the real `kali_runtime_contract` helper in the
# same run as the fixtures above.
CAP_HARNESS_SCRIPT_APP_ENTRIES_SPREAD = (
    "import fs from 'node:fs/promises';\n"
    "import { fileURLToPath } from 'node:url';\n"
    '\n'
    "const bundleJs = new URL('./app/app.js', import.meta.url);\n"
    "const wasmUrl = new URL('./app/app.wasm', import.meta.url);\n"
    '\n'
    'globalThis.fetch = async (input) => {\n'
    '  const url = input instanceof URL ? input : new URL(String(input));\n'
    '  if (url.href === wasmUrl.href) {\n'
    '    const bytes = await fs.readFile(fileURLToPath(url));\n'
    "    return new Response(bytes, { headers: { 'content-type': 'application/wasm' } });\n"
    '  }\n'
    '  throw new Error(`unexpected fetch ${String(input)}`);\n'
    '};\n'
    '\n'
    'const mod = await import(bundleJs.href);\n'
    'await mod.browserObjectKeysEntriesSpread();\n'
)
