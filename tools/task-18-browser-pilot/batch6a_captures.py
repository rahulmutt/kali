"""Rule-8 / rule-9 captured fixture texts for Task 18 batch 6A.

Every constant here is the BYTE-EXACT OUTPUT OF EXECUTING THE REAL CODE, never
a hand-applied `format!` substitution or a hand-applied `str::replace`. Rule 8
forbids hand-simulating a `format!`; rule 9 extends the same discipline to a
fixture built one level removed inside a library crate (`kali_common::`).

HOW THEY WERE CAPTURED, so they can be re-derived.

1. A temporary target `crates/kali_cli/tests/zz_b6a_dump.rs`, deleted in the
   same session, with one `mod` per source that `include!`d the shipped `.rs`
   and a `pub fn dump()` inside that `mod` (the fixture builders are private, so
   the dump has to live in the module that includes them). Run as

       ZZ_B6A_OUT=<dir> cargo test -p kali_cli --test zz_b6a_dump -- zz_dump \
           --test-threads=1

   `include!` rather than a retyped copy, so the executed `format!` /
   `kali_common` call is literally the one in the shipped source. This produced
   every constant below except the two harness bodies.

2. THE TWO HARNESS BODIES were not reachable as a value: they are built by a
   `format!` written INLINE inside
   `browser_object_computed_numeric_keys_bundle.rs`'s assert helper, with a
   `{harness_function}` placeholder. Captured the way batch 5 captured the same
   shape -- by running that target's real tests with
   `KALI_BROWSER_BUNDLE_HARNESS_COMMAND` pointed at a wrapper that copies the
   harness script it is handed and then `exec node "$@"` (so the target's own
   assertions still hold), then subtracting
   `kali_runtime_contract::browser_bundle_harness_prelude("app", false)` from
   each captured script. `browser_bundle_harness_script` is defined as
   prelude + body, so the remainder IS the resolved body, still never
   hand-substituted.

They are embedded here rather than read from a dump file so this module runs
from a clean checkout with no uncommitted inputs -- the defect that got the
pilot's per-file generators deleted (see README). `gen_batch6a.py` re-checks
each one against its own `.rs` before emitting it (`check_captured`), so a
stale capture taken before a source edit fails the generator rather than
shipping a program that is no longer the program under test.
"""

CAP_NUMBER_BUNDLE_JS = (
    '// kali-tree-shake: browserNumberPredicates\n'
    'async function browserNumberPredicates() {\n'
    '  const alias = 1;\n'
    '  const finite = Number.isFinite;\n'
    '  const integer = Number.isInteger;\n'
    '  const safeInteger = Number.isSafeInteger;\n'
    '  const frozenFinite = Object.freeze(Number.isFinite);\n'
    '  const frozenNaN = Object.freeze(Number.isNaN);\n'
    '  const frozenInteger = Object.freeze(Number.isInteger);\n'
    '  const frozenSafeInteger = Object.freeze(Number.isSafeInteger);\n'
    '  const frozenBracketedFinite = Object.freeze(Number["isFinite"]);\n'
    '  const frozenBracketedNaN = Object.freeze(Number["isNaN"]);\n'
    '  const frozenBracketedInteger = Object.freeze(Number["isInteger"]);\n'
    '  const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]);\n'
    '  const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]);\n'
    '  const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]);\n'
    '  const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]);\n'
    '  const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]);\n'
    '  const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite);\n'
    '  const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN);\n'
    '  const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger);\n'
    '  const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger);\n'
    '  if (\n'
    '    Number.isFinite(alias) !== true ||\n'
    '    Number.isSafeInteger(await alias) !== true ||\n'
    '    integer(alias) !== true ||\n'
    '    Number.isSafeInteger(alias) !== true ||\n'
    '    integer(1.5) !== false ||\n'
    '    Number.isFinite("hello") !== false ||\n'
    '    Number.isSafeInteger(1.5) !== false ||\n'
    '    globalThis["Number"]["isNaN"](NaN) !== true ||\n'
    '    globalThis.Number.isNaN(1) !== false ||\n'
    '    globalThis["Number"].isNaN(1) !== false ||\n'
    '    globalThis["Number"]["isFinite"](alias) !== true ||\n'
    '    globalThis["Number"]["isInteger"](alias) !== true ||\n'
    '    globalThis["Number"]["isSafeInteger"](alias) !== true ||\n'
    '    globalThis.Number["isNaN"](1) !== false ||\n'
    '    globalThis["Number"].isFinite(alias) !== true ||\n'
    '    globalThis.Number["isInteger"](alias) !== true ||\n'
    '    globalThis["Number"].isSafeInteger(alias) !== true ||\n'
    '    Number["isFinite"](alias) !== true ||\n'
    '    Number["isInteger"](alias) !== true ||\n'
    '    Number["isSafeInteger"](alias) !== true ||\n'
    '    Number["isNaN"](1) !== false ||\n'
    '    frozenFinite(alias) !== true ||\n'
    '    frozenNaN(NaN) !== true ||\n'
    '    frozenNaN(1) !== false ||\n'
    '    frozenInteger(alias) !== true ||\n'
    '    frozenSafeInteger(alias) !== true ||\n'
    '    frozenBracketedFinite(alias) !== true ||\n'
    '    frozenBracketedNaN(NaN) !== true ||\n'
    '    frozenBracketedNaN(1) !== false ||\n'
    '    frozenBracketedInteger(alias) !== true ||\n'
    '    frozenBracketedSafeInteger(alias) !== true ||\n'
    '    frozenParenthesizedBracketedFinite(alias) !== true ||\n'
    '    frozenParenthesizedBracketedNaN(NaN) !== true ||\n'
    '    frozenParenthesizedBracketedNaN(1) !== false ||\n'
    '    frozenParenthesizedBracketedInteger(alias) !== true ||\n'
    '    frozenParenthesizedBracketedSafeInteger(alias) !== true ||\n'
    '    frozenParenthesizedPropertyFinite(alias) !== true ||\n'
    '    frozenParenthesizedPropertyNaN(NaN) !== true ||\n'
    '    frozenParenthesizedPropertyNaN(1) !== false ||\n'
    '    frozenParenthesizedPropertyInteger(alias) !== true ||\n'
    '    frozenParenthesizedPropertySafeInteger(alias) !== true ||\n'
    '    safeInteger(alias) !== true ||\n'
    '    finite(alias) !== true\n'
    '  ) {\n'
    "    throw new Error('unexpected browser Number predicate result');\n"
    '  }\n'
    "  console.log('browser number predicates ok');\n"
    '}\n'
)

CAP_NUMBER_BUNDLE_TS = (
    '// kali-tree-shake: browserNumberPredicates\n'
    'async function browserNumberPredicates() {\n'
    '  const alias = 1 as const;\n'
    '  const finite = Number.isFinite;\n'
    '  const integer = Number.isInteger;\n'
    '  const safeInteger = Number.isSafeInteger;\n'
    '  const frozenFinite = Object.freeze(Number.isFinite);\n'
    '  const frozenNaN = Object.freeze(Number.isNaN);\n'
    '  const frozenInteger = Object.freeze(Number.isInteger);\n'
    '  const frozenSafeInteger = Object.freeze(Number.isSafeInteger);\n'
    '  const frozenBracketedFinite = Object.freeze(Number["isFinite"]);\n'
    '  const frozenBracketedNaN = Object.freeze(Number["isNaN"]);\n'
    '  const frozenBracketedInteger = Object.freeze(Number["isInteger"]);\n'
    '  const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]);\n'
    '  const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]);\n'
    '  const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]);\n'
    '  const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]);\n'
    '  const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]);\n'
    '  const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite);\n'
    '  const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN);\n'
    '  const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger);\n'
    '  const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger);\n'
    '  if (\n'
    '    Number.isFinite(alias) !== true ||\n'
    '    Number.isSafeInteger(await alias) !== true ||\n'
    '    integer(alias) !== true ||\n'
    '    Number.isSafeInteger(alias) !== true ||\n'
    '    integer(1.5) !== false ||\n'
    '    Number.isFinite("hello") !== false ||\n'
    '    Number.isSafeInteger(1.5) !== false ||\n'
    '    globalThis["Number"]["isNaN"](NaN) !== true ||\n'
    '    globalThis.Number.isNaN(1) !== false ||\n'
    '    globalThis["Number"].isNaN(1) !== false ||\n'
    '    globalThis["Number"]["isFinite"](alias) !== true ||\n'
    '    globalThis["Number"]["isInteger"](alias) !== true ||\n'
    '    globalThis["Number"]["isSafeInteger"](alias) !== true ||\n'
    '    globalThis.Number["isNaN"](1) !== false ||\n'
    '    globalThis["Number"].isFinite(alias) !== true ||\n'
    '    globalThis.Number["isInteger"](alias) !== true ||\n'
    '    globalThis["Number"].isSafeInteger(alias) !== true ||\n'
    '    Number["isFinite"](alias) !== true ||\n'
    '    Number["isInteger"](alias) !== true ||\n'
    '    Number["isSafeInteger"](alias) !== true ||\n'
    '    Number["isNaN"](1) !== false ||\n'
    '    frozenFinite(alias) !== true ||\n'
    '    frozenNaN(NaN) !== true ||\n'
    '    frozenNaN(1) !== false ||\n'
    '    frozenInteger(alias) !== true ||\n'
    '    frozenSafeInteger(alias) !== true ||\n'
    '    frozenBracketedFinite(alias) !== true ||\n'
    '    frozenBracketedNaN(NaN) !== true ||\n'
    '    frozenBracketedNaN(1) !== false ||\n'
    '    frozenBracketedInteger(alias) !== true ||\n'
    '    frozenBracketedSafeInteger(alias) !== true ||\n'
    '    frozenParenthesizedBracketedFinite(alias) !== true ||\n'
    '    frozenParenthesizedBracketedNaN(NaN) !== true ||\n'
    '    frozenParenthesizedBracketedNaN(1) !== false ||\n'
    '    frozenParenthesizedBracketedInteger(alias) !== true ||\n'
    '    frozenParenthesizedBracketedSafeInteger(alias) !== true ||\n'
    '    frozenParenthesizedPropertyFinite(alias) !== true ||\n'
    '    frozenParenthesizedPropertyNaN(NaN) !== true ||\n'
    '    frozenParenthesizedPropertyNaN(1) !== false ||\n'
    '    frozenParenthesizedPropertyInteger(alias) !== true ||\n'
    '    frozenParenthesizedPropertySafeInteger(alias) !== true ||\n'
    '    safeInteger(alias) !== true ||\n'
    '    finite(alias) !== true\n'
    '  ) {\n'
    "    throw new Error('unexpected browser Number predicate result');\n"
    '  }\n'
    "  console.log('browser number predicates ok');\n"
    '}\n'
)

CAP_NUMBER_HARNESS_RUN = (
    'const alias = 1; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number["isFinite"]); const frozenBracketedNaN = Object.freeze(Number["isNaN"]); const frozenBracketedInteger = Object.freeze(Number["isInteger"]); const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]); const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite); const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN); const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger); const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger); console.log(Number.isFinite(alias)); console.log(integer(alias)); console.log(Number.isSafeInteger(alias)); console.log(integer(1.5)); console.log(Number.isFinite("hello")); console.log(Number.isSafeInteger(1.5)); console.log(globalThis["Number"]["isNaN"](NaN)); console.log(globalThis.Number.isNaN(1)); console.log(globalThis["Number"].isNaN(1)); console.log(globalThis["Number"]["isFinite"](alias)); console.log(globalThis["Number"]["isInteger"](alias)); console.log(globalThis["Number"]["isSafeInteger"](alias)); console.log(globalThis.Number["isNaN"](1)); console.log(globalThis["Number"].isFinite(alias)); console.log(globalThis.Number["isInteger"](alias)); console.log(globalThis["Number"].isSafeInteger(alias)); console.log(Number["isFinite"](alias)); console.log(Number["isInteger"](alias)); console.log(Number["isSafeInteger"](alias)); console.log(Number["isNaN"](1)); console.log(frozenFinite(alias)); console.log(frozenNaN(NaN)); console.log(frozenNaN(1)); console.log(frozenInteger(alias)); console.log(frozenSafeInteger(alias)); console.log(frozenBracketedFinite(alias)); console.log(frozenBracketedNaN(NaN)); console.log(frozenBracketedNaN(1)); console.log(frozenBracketedInteger(alias)); console.log(frozenBracketedSafeInteger(alias)); console.log(frozenParenthesizedBracketedFinite(alias)); console.log(frozenParenthesizedBracketedNaN(NaN)); console.log(frozenParenthesizedBracketedNaN(1)); console.log(frozenParenthesizedBracketedInteger(alias)); console.log(frozenParenthesizedBracketedSafeInteger(alias)); console.log(frozenParenthesizedPropertyFinite(alias)); console.log(frozenParenthesizedPropertyNaN(NaN)); console.log(frozenParenthesizedPropertyNaN(1)); console.log(frozenParenthesizedPropertyInteger(alias)); console.log(frozenParenthesizedPropertySafeInteger(alias)); console.log(finite(alias)); console.log(integer(alias)); console.log(safeInteger(alias));'
)

CAP_NUMBER_HARNESS_TEST = (
    'Kali.test(\'number predicates\', () => { const alias = 1; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number["isFinite"]); const frozenBracketedNaN = Object.freeze(Number["isNaN"]); const frozenBracketedInteger = Object.freeze(Number["isInteger"]); const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]); const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite); const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN); const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger); const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger); console.log(Number.isFinite(alias)); console.log(integer(alias)); console.log(Number.isSafeInteger(alias)); console.log(integer(1.5)); console.log(Number.isFinite("hello")); console.log(Number.isSafeInteger(1.5)); console.log(globalThis["Number"]["isNaN"](NaN)); console.log(globalThis.Number.isNaN(1)); console.log(globalThis["Number"].isNaN(1)); console.log(globalThis["Number"]["isFinite"](alias)); console.log(globalThis["Number"]["isInteger"](alias)); console.log(globalThis["Number"]["isSafeInteger"](alias)); console.log(globalThis.Number["isNaN"](1)); console.log(globalThis["Number"].isFinite(alias)); console.log(globalThis.Number["isInteger"](alias)); console.log(globalThis["Number"].isSafeInteger(alias)); console.log(Number["isFinite"](alias)); console.log(Number["isInteger"](alias)); console.log(Number["isSafeInteger"](alias)); console.log(Number["isNaN"](1)); console.log(frozenFinite(alias)); console.log(frozenNaN(NaN)); console.log(frozenNaN(1)); console.log(frozenInteger(alias)); console.log(frozenSafeInteger(alias)); console.log(frozenBracketedFinite(alias)); console.log(frozenBracketedNaN(NaN)); console.log(frozenBracketedNaN(1)); console.log(frozenBracketedInteger(alias)); console.log(frozenBracketedSafeInteger(alias)); console.log(frozenParenthesizedBracketedFinite(alias)); console.log(frozenParenthesizedBracketedNaN(NaN)); console.log(frozenParenthesizedBracketedNaN(1)); console.log(frozenParenthesizedBracketedInteger(alias)); console.log(frozenParenthesizedBracketedSafeInteger(alias)); console.log(frozenParenthesizedPropertyFinite(alias)); console.log(frozenParenthesizedPropertyNaN(NaN)); console.log(frozenParenthesizedPropertyNaN(1)); console.log(frozenParenthesizedPropertyInteger(alias)); console.log(frozenParenthesizedPropertySafeInteger(alias)); console.log(finite(alias)); console.log(integer(alias)); console.log(safeInteger(alias)); });'
)

CAP_COMPUTED_KEYS_RUN = (
    "const obj = { [-1]: 'neg', [+2]: 'pos', [(-0)]: 'zero' };\n"
    'console.log(obj[-1]);\n'
    'console.log(obj[2]);\n'
    'console.log(obj[0]);\n'
)

CAP_COMPUTED_KEYS_TEST = (
    "Kali.test('computed numeric object keys', () => {\n"
    "const obj = { [-1]: 'neg', [+2]: 'pos', [(-0)]: 'zero' };\n"
    'console.log(obj[-1]);\n'
    'console.log(obj[2]);\n'
    'console.log(obj[0]);\n'
    '});\n'
)

CAP_COMPUTED_AWAIT_RUN = (
    'async function computedNumericObjectKeysWithAwaitWrappers() {\n'
    '  const obj = {\n'
    "    [await 1]: 'neg',\n"
    "    [+(await 2)]: 'pos',\n"
    "    [(0, await 0)]: 'zero',\n"
    '  };\n'
    '  console.log(obj[1]);\n'
    '  console.log(obj[2]);\n'
    '  console.log(obj[0]);\n'
    '}\n'
    'computedNumericObjectKeysWithAwaitWrappers();\n'
)

CAP_COMPUTED_AWAIT_TEST = (
    "Kali.test('computed numeric object keys with await wrappers', () => {\n"
    'async function computedNumericObjectKeysWithAwaitWrappers() {\n'
    '  const obj = {\n'
    "    [await 1]: 'neg',\n"
    "    [+(await 2)]: 'pos',\n"
    "    [(0, await 0)]: 'zero',\n"
    '  };\n'
    '  console.log(obj[1]);\n'
    '  console.log(obj[2]);\n'
    '  console.log(obj[0]);\n'
    '}\n'
    'computedNumericObjectKeysWithAwaitWrappers();\n'
    '  return computedNumericObjectKeysWithAwaitWrappers();\n'
    '});\n'
)

CAP_ENTRIES_FROZEN_RUN = (
    'function assertObjectEntriesIteration(entries) {\n'
    '  if (\n'
    '    entries.length !== 2 ||\n'
    "    entries[0][0] !== 'b' ||\n"
    '    entries[0][1] !== 1 ||\n'
    "    entries[1][0] !== 'a' ||\n"
    '    entries[1][1] !== 2\n'
    '  ) {\n'
    "    throw new Error('unexpected Object.entries iteration semantics');\n"
    '  }\n'
    '}\n'
    '\n'
    'function browserObjectEntriesIteration() {\n'
    '  const values = Object.freeze({ "b": 1, "a": 2 });\n'
    '  const alias = values;\n'
    '  const entries = Object.entries(alias);\n'
    '  const globalEntries = globalThis.Object.entries(alias);\n'
    '  const mixedEntries = globalThis.Object["entries"](alias);\n'
    '  const mixedBracketedEntries = globalThis["Object"].entries(alias);\n'
    '  const bracketedEntries = globalThis["Object"]["entries"](alias);\n'
    '  const parenthesizedReceiverBracketedEntries = (globalThis["Object"])["entries"](alias);\n'
    '  const parenthesizedSingleQuotedReceiverBracketedEntries = (globalThis[\'Object\'])["entries"](alias);\n'
    '  const frozenEntries = Object.freeze(Object.entries)(alias);\n'
    '  const frozenGlobalEntries = Object.freeze(globalThis.Object.entries)(alias);\n'
    '  const frozenBracketedEntries = Object.freeze(globalThis["Object"]["entries"])(alias);\n'
    '  const frozenParenthesizedBracketedEntries = Object.freeze((globalThis["Object"]).entries)(alias);\n'
    '  const frozenParenthesizedDotRootEntries = Object.freeze((globalThis.Object).entries)(alias);\n'
    '  const frozenParenthesizedReceiverBracketedEntries = Object.freeze((globalThis["Object"])["entries"])(alias);\n'
    '  const frozenParenthesizedSingleQuotedReceiverBracketedEntries = Object.freeze((globalThis[\'Object\'])["entries"])(alias);\n'
    "  const frozenParenthesizedSingleQuotedReceiverBracketedPropertyEntries = Object.freeze((globalThis['Object']).entries)(alias);\n"
    "  const frozenSingleQuotedBracketedEntries = Object.freeze((globalThis['Object']['entries']))(alias);\n"
    '  assertObjectEntriesIteration(entries);\n'
    '  assertObjectEntriesIteration(globalEntries);\n'
    '  assertObjectEntriesIteration(mixedEntries);\n'
    '  assertObjectEntriesIteration(mixedBracketedEntries);\n'
    '  assertObjectEntriesIteration(bracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedSingleQuotedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(frozenEntries);\n'
    '  assertObjectEntriesIteration(frozenGlobalEntries);\n'
    '  assertObjectEntriesIteration(frozenBracketedEntries);\n'
    '  assertObjectEntriesIteration(frozenParenthesizedBracketedEntries);\n'
    '  assertObjectEntriesIteration(frozenParenthesizedDotRootEntries);\n'
    '  assertObjectEntriesIteration(frozenParenthesizedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(frozenParenthesizedSingleQuotedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(frozenParenthesizedSingleQuotedReceiverBracketedPropertyEntries);\n'
    '  assertObjectEntriesIteration(frozenSingleQuotedBracketedEntries);\n'
    "  console.log('browser object entries iteration ok');\n"
    '}\n'
    '\n'
    'browserObjectEntriesIteration();\n'
)

CAP_ENTRIES_FROZEN_TEST = (
    "Kali.test('object entries iteration', () => {\n"
    '  function assertObjectEntriesIteration(entries) {\n'
    '    if (\n'
    '      entries.length !== 2 ||\n'
    "      entries[0][0] !== 'b' ||\n"
    '      entries[0][1] !== 1 ||\n'
    "      entries[1][0] !== 'a' ||\n"
    '      entries[1][1] !== 2\n'
    '    ) {\n'
    "      throw new Error('unexpected Object.entries iteration semantics');\n"
    '    }\n'
    '  }\n'
    '\n'
    '  const values = Object.freeze({ "b": 1, "a": 2 });\n'
    '  const alias = values;\n'
    '  const entries = Object.entries(alias);\n'
    '  const globalEntries = globalThis.Object.entries(alias);\n'
    '  const mixedEntries = globalThis.Object["entries"](alias);\n'
    '  const mixedBracketedEntries = globalThis["Object"].entries(alias);\n'
    '  const bracketedEntries = globalThis["Object"]["entries"](alias);\n'
    '  const parenthesizedReceiverBracketedEntries = (globalThis["Object"])["entries"](alias);\n'
    '  const parenthesizedSingleQuotedReceiverBracketedEntries = (globalThis[\'Object\'])["entries"](alias);\n'
    '  const frozenEntries = Object.freeze(Object.entries)(alias);\n'
    '  const frozenGlobalEntries = Object.freeze(globalThis.Object.entries)(alias);\n'
    '  const frozenBracketedEntries = Object.freeze(globalThis["Object"]["entries"])(alias);\n'
    '  const frozenParenthesizedBracketedEntries = Object.freeze((globalThis["Object"]).entries)(alias);\n'
    '  const frozenParenthesizedDotRootEntries = Object.freeze((globalThis.Object).entries)(alias);\n'
    '  const frozenParenthesizedReceiverBracketedEntries = Object.freeze((globalThis["Object"])["entries"])(alias);\n'
    '  const frozenParenthesizedSingleQuotedReceiverBracketedEntries = Object.freeze((globalThis[\'Object\'])["entries"])(alias);\n'
    "  const frozenParenthesizedSingleQuotedReceiverBracketedPropertyEntries = Object.freeze((globalThis['Object']).entries)(alias);\n"
    "  const frozenSingleQuotedBracketedEntries = Object.freeze((globalThis['Object']['entries']))(alias);\n"
    '  assertObjectEntriesIteration(entries);\n'
    '  assertObjectEntriesIteration(globalEntries);\n'
    '  assertObjectEntriesIteration(mixedEntries);\n'
    '  assertObjectEntriesIteration(mixedBracketedEntries);\n'
    '  assertObjectEntriesIteration(bracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(parenthesizedSingleQuotedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(frozenEntries);\n'
    '  assertObjectEntriesIteration(frozenGlobalEntries);\n'
    '  assertObjectEntriesIteration(frozenBracketedEntries);\n'
    '  assertObjectEntriesIteration(frozenParenthesizedBracketedEntries);\n'
    '  assertObjectEntriesIteration(frozenParenthesizedDotRootEntries);\n'
    '  assertObjectEntriesIteration(frozenParenthesizedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(frozenParenthesizedSingleQuotedReceiverBracketedEntries);\n'
    '  assertObjectEntriesIteration(frozenParenthesizedSingleQuotedReceiverBracketedPropertyEntries);\n'
    '  assertObjectEntriesIteration(frozenSingleQuotedBracketedEntries);\n'
    "  console.log('browser object entries iteration ok');\n"
    '});\n'
)

# The two `format!`-built browser-bundle harness bodies (capture procedure 2
# above). They differ only in the exported function name they call.
CAP_COMPUTED_BUNDLE_BODY_PLAIN = (
    "const mod = await import(bundleJs.href);\n"
    "await mod.computedNumericObjectKeys();\n"
)

CAP_COMPUTED_BUNDLE_BODY_AWAIT = (
    "const mod = await import(bundleJs.href);\n"
    "await mod.computedNumericObjectKeysWithAwaitWrappers();\n"
)
