"""Rule-8 / rule-9 captured fixture texts for Task 19 batch 2.

Every constant here is the BYTE-EXACT OUTPUT OF EXECUTING THE REAL CODE. Rule 8
forbids hand-simulating a `format!`; rule 9 extends the same discipline to a
fixture built one level removed inside a library crate (`kali_common::`). The
sources in this batch whose fixtures are plain string literals are NOT here --
`gen_task19_batch2.py` pulls those straight out of the `.rs` through
`lexer.find_string_literals`, which is this project's copy-never-retype
mechanism. Only the fixtures that do not exist as a literal anywhere need a
capture.

HOW THEY WERE CAPTURED, so they can be re-derived:

  1. A temporary target `crates/kali_cli/tests/zz_t19b2_dump.rs`, deleted in the
     same session, with one `mod` per source that `include!`d the shipped `.rs`
     and a `pub fn dump()` inside that `mod` (the fixture builders are private,
     so the dump has to live in the module that includes them). Same mechanism
     as batch 6A's `zz_b6a_dump.rs`.

  2. Run as

         ZZ_OUT=<dir> cargo test -p kali_cli --test zz_t19b2_dump \\
             -- zz_dump --test-threads=1

     `include!` rather than a retyped copy, so the executed `format!` /
     `kali_common` call is literally the one in the shipped source.

  3. This module was WRITTEN BY A SCRIPT from those dump files and its
     round-trip was asserted before it was committed. No byte of it passed
     through a transcription step.

They are embedded here rather than read from a dump file so the generator runs
from a clean checkout with no uncommitted inputs (U12) -- the defect that got
the pilot's per-file generators deleted. `gen_task19_batch2.py` re-checks every
capture against its own `.rs` before emitting it (`check_captured`), so a stale
capture taken before a source edit fails the generator rather than shipping a
program that is no longer the program under test.
"""

CAP_FASTA_CAPSTONE__SHELL = (
    'var last = 42;\n'
    'function rand(max) { last = (last * 3877 + 29573) % 139968; return max * last / 139968; }\n'
    'function makeCumulative(table) {\n'
    '  var prev = null;\n'
    '  for (var c in table) {\n'
    '    if (prev) table[c] = table[c] + table[prev];\n'
    '    prev = c;\n'
    '  }\n'
    '}\n'
    'function fastaRepeat(n, seq) {\n'
    '  var seqi = 0;\n'
    '  var lenOut = 60;\n'
    '  while (n > 0) {\n'
    '    if (n < lenOut) lenOut = n;\n'
    '    if (seqi + lenOut < seq.length) {\n'
    '      console.log(seq.substring(seqi, seqi + lenOut));\n'
    '      seqi = seqi + lenOut;\n'
    '    } else {\n'
    '      console.log(seq.substring(seqi) + seq.substring(0, lenOut - (seq.length - seqi)));\n'
    '      seqi = lenOut - (seq.length - seqi);\n'
    '    }\n'
    '    n = n - lenOut;\n'
    '  }\n'
    '}\n'
    'function fastaRandom(n, table) {\n'
    '  var line = new Array(60);\n'
    '  makeCumulative(table);\n'
    '  while (n > 0) {\n'
    '    if (n < line.length) line = new Array(n);\n'
    '    for (var i = 0; i < line.length; i = i + 1) {\n'
    '      var r = rand(1);\n'
    '      for (var c in table) { if (r < table[c]) break; }\n'
    '      line[i] = c;\n'
    '    }\n'
    '    console.log(line.join(""));\n'
    '    n = n - line.length;\n'
    '  }\n'
    '}\n'
    'var ALU = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG" +\n'
    '"GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA" +\n'
    '"CCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAAT" +\n'
    '"ACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCA" +\n'
    '"GCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGG" +\n'
    '"AGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCC" +\n'
    '"AGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAAA";\n'
    'var IUB = { a: 0.27, c: 0.12, g: 0.12, t: 0.27, B: 0.02, D: 0.02, H: 0.02, K: 0.02, M: 0.02, N: 0.02, R: 0.02, S: 0.02, V: 0.02, W: 0.02, Y: 0.02 };\n'
    'var HomoSap = { a: 0.3029549426680, c: 0.1979883004921, g: 0.1975473066391, t: 0.3015094502008 };\n'
    'var n = +process.argv[2];\n'
    'console.log(">ONE Homo sapiens alu");\n'
    'fastaRepeat(2 * n, ALU);\n'
    'console.log(">TWO IUB ambiguity codes");\n'
    'fastaRandom(3 * n, IUB);\n'
    'console.log(">THREE Homo sapiens frequency");\n'
    'fastaRandom(5 * n, HomoSap);\n'
    ''
)

CAP_FASTA_OUTPUT__RANDOM = (
    'var last = 42;\n'
    'function rand(max) { last = (last * 3877 + 29573) % 139968; return max * last / 139968; }\n'
    'function makeCumulative(table) {\n'
    '  var prev = null;\n'
    '  for (var c in table) {\n'
    '    if (prev) table[c] = table[c] + table[prev];\n'
    '    prev = c;\n'
    '  }\n'
    '}\n'
    'function fastaRandom(n, table) {\n'
    '  var line = new Array(60);\n'
    '  makeCumulative(table);\n'
    '  while (n > 0) {\n'
    '    if (n < line.length) line = new Array(n);\n'
    '    for (var i = 0; i < line.length; i = i + 1) {\n'
    '      var r = rand(1);\n'
    '      for (var c in table) { if (r < table[c]) break; }\n'
    '      line[i] = c;\n'
    '    }\n'
    '    console.log(line.join(""));\n'
    '    n = n - line.length;\n'
    '  }\n'
    '}\n'
    'var IUB = { a: 0.27, c: 0.12, g: 0.12, t: 0.27, B: 0.02, D: 0.02, H: 0.02, K: 0.02, M: 0.02, N: 0.02, R: 0.02, S: 0.02, V: 0.02, W: 0.02, Y: 0.02 };\n'
    'fastaRandom(70, IUB);\n'
    ''
)

CAP_FASTA_OUTPUT__REPEAT = (
    'function fastaRepeat(n, seq) {\n'
    '  var seqi = 0;\n'
    '  var lenOut = 60;\n'
    '  while (n > 0) {\n'
    '    if (n < lenOut) lenOut = n;\n'
    '    if (seqi + lenOut < seq.length) {\n'
    '      console.log(seq.substring(seqi, seqi + lenOut));\n'
    '      seqi = seqi + lenOut;\n'
    '    } else {\n'
    '      console.log(seq.substring(seqi) + seq.substring(0, lenOut - (seq.length - seqi)));\n'
    '      seqi = lenOut - (seq.length - seqi);\n'
    '    }\n'
    '    n = n - lenOut;\n'
    '  }\n'
    '}\n'
    'var ALU = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG" + "GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA";\n'
    'fastaRepeat(120, ALU);\n'
    ''
)

CAP_NUMBER_PREDICATES__RUN = (
    'const alias = 1; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number["isFinite"]); const frozenBracketedNaN = Object.freeze(Number["isNaN"]); const frozenBracketedInteger = Object.freeze(Number["isInteger"]); const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]); const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite); const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN); const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger); const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger); console.log(Number.isFinite(alias)); console.log(integer(alias)); console.log(Number.isSafeInteger(alias)); console.log(integer(1.5)); console.log(Number.isFinite("hello")); console.log(Number.isSafeInteger(1.5)); console.log(globalThis["Number"]["isNaN"](NaN)); console.log(globalThis.Number.isNaN(1)); console.log(globalThis["Number"].isNaN(1)); console.log(globalThis["Number"]["isFinite"](alias)); console.log(globalThis["Number"]["isInteger"](alias)); console.log(globalThis["Number"]["isSafeInteger"](alias)); console.log(globalThis.Number["isNaN"](1)); console.log(globalThis["Number"].isFinite(alias)); console.log(globalThis.Number["isInteger"](alias)); console.log(globalThis["Number"].isSafeInteger(alias)); console.log(Number["isFinite"](alias)); console.log(Number["isInteger"](alias)); console.log(Number["isSafeInteger"](alias)); console.log(Number["isNaN"](1)); console.log(frozenFinite(alias)); console.log(frozenNaN(NaN)); console.log(frozenNaN(1)); console.log(frozenInteger(alias)); console.log(frozenSafeInteger(alias)); console.log(frozenBracketedFinite(alias)); console.log(frozenBracketedNaN(NaN)); console.log(frozenBracketedNaN(1)); console.log(frozenBracketedInteger(alias)); console.log(frozenBracketedSafeInteger(alias)); console.log(frozenParenthesizedBracketedFinite(alias)); console.log(frozenParenthesizedBracketedNaN(NaN)); console.log(frozenParenthesizedBracketedNaN(1)); console.log(frozenParenthesizedBracketedInteger(alias)); console.log(frozenParenthesizedBracketedSafeInteger(alias)); console.log(frozenParenthesizedPropertyFinite(alias)); console.log(frozenParenthesizedPropertyNaN(NaN)); console.log(frozenParenthesizedPropertyNaN(1)); console.log(frozenParenthesizedPropertyInteger(alias)); console.log(frozenParenthesizedPropertySafeInteger(alias)); console.log(finite(alias)); console.log(integer(alias)); console.log(safeInteger(alias));'
)

CAP_NUMBER_PREDICATES__TEST = (
    'Kali.test(\'number predicates\', () => { const alias = 1; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number["isFinite"]); const frozenBracketedNaN = Object.freeze(Number["isNaN"]); const frozenBracketedInteger = Object.freeze(Number["isInteger"]); const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]); const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite); const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN); const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger); const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger); console.log(Number.isFinite(alias)); console.log(integer(alias)); console.log(Number.isSafeInteger(alias)); console.log(integer(1.5)); console.log(Number.isFinite("hello")); console.log(Number.isSafeInteger(1.5)); console.log(globalThis["Number"]["isNaN"](NaN)); console.log(globalThis.Number.isNaN(1)); console.log(globalThis["Number"].isNaN(1)); console.log(globalThis["Number"]["isFinite"](alias)); console.log(globalThis["Number"]["isInteger"](alias)); console.log(globalThis["Number"]["isSafeInteger"](alias)); console.log(globalThis.Number["isNaN"](1)); console.log(globalThis["Number"].isFinite(alias)); console.log(globalThis.Number["isInteger"](alias)); console.log(globalThis["Number"].isSafeInteger(alias)); console.log(Number["isFinite"](alias)); console.log(Number["isInteger"](alias)); console.log(Number["isSafeInteger"](alias)); console.log(Number["isNaN"](1)); console.log(frozenFinite(alias)); console.log(frozenNaN(NaN)); console.log(frozenNaN(1)); console.log(frozenInteger(alias)); console.log(frozenSafeInteger(alias)); console.log(frozenBracketedFinite(alias)); console.log(frozenBracketedNaN(NaN)); console.log(frozenBracketedNaN(1)); console.log(frozenBracketedInteger(alias)); console.log(frozenBracketedSafeInteger(alias)); console.log(frozenParenthesizedBracketedFinite(alias)); console.log(frozenParenthesizedBracketedNaN(NaN)); console.log(frozenParenthesizedBracketedNaN(1)); console.log(frozenParenthesizedBracketedInteger(alias)); console.log(frozenParenthesizedBracketedSafeInteger(alias)); console.log(frozenParenthesizedPropertyFinite(alias)); console.log(frozenParenthesizedPropertyNaN(NaN)); console.log(frozenParenthesizedPropertyNaN(1)); console.log(frozenParenthesizedPropertyInteger(alias)); console.log(frozenParenthesizedPropertySafeInteger(alias)); console.log(finite(alias)); console.log(integer(alias)); console.log(safeInteger(alias)); });'
)

CAP_NUMBER_PREDICATES_FREEZE__EXPECTED_STDOUT = (
    '1\n'
    '1\n'
    '1\n'
    '0\n'
    '0\n'
    '0\n'
    '1\n'
    '0\n'
    '0\n'
    '1\n'
    '1\n'
    '1\n'
    '0\n'
    '1\n'
    '1\n'
    '1\n'
    '1\n'
    '1\n'
    '1\n'
    '0\n'
    '1\n'
    '1\n'
    '0\n'
    '1\n'
    '1\n'
    '1\n'
    '1\n'
    '0\n'
    '1\n'
    '1\n'
    '1\n'
    '1\n'
    '0\n'
    '1\n'
    '1\n'
    '1\n'
    '1\n'
    '0\n'
    '1\n'
    '1\n'
    '1\n'
    '1\n'
    '1'
)

CAP_NUMBER_PREDICATES_FREEZE__RUN = (
    'const alias = 1; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number["isFinite"]); const frozenBracketedNaN = Object.freeze(Number["isNaN"]); const frozenBracketedInteger = Object.freeze(Number["isInteger"]); const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]); const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite); const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN); const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger); const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger); console.log(Number.isFinite(alias)); console.log(integer(alias)); console.log(Number.isSafeInteger(alias)); console.log(integer(1.5)); console.log(Number.isFinite("hello")); console.log(Number.isSafeInteger(1.5)); console.log(globalThis["Number"]["isNaN"](NaN)); console.log(globalThis.Number.isNaN(1)); console.log(globalThis["Number"].isNaN(1)); console.log(globalThis["Number"]["isFinite"](alias)); console.log(globalThis["Number"]["isInteger"](alias)); console.log(globalThis["Number"]["isSafeInteger"](alias)); console.log(globalThis.Number["isNaN"](1)); console.log(globalThis["Number"].isFinite(alias)); console.log(globalThis.Number["isInteger"](alias)); console.log(globalThis["Number"].isSafeInteger(alias)); console.log(Number["isFinite"](alias)); console.log(Number["isInteger"](alias)); console.log(Number["isSafeInteger"](alias)); console.log(Number["isNaN"](1)); console.log(frozenFinite(alias)); console.log(frozenNaN(NaN)); console.log(frozenNaN(1)); console.log(frozenInteger(alias)); console.log(frozenSafeInteger(alias)); console.log(frozenBracketedFinite(alias)); console.log(frozenBracketedNaN(NaN)); console.log(frozenBracketedNaN(1)); console.log(frozenBracketedInteger(alias)); console.log(frozenBracketedSafeInteger(alias)); console.log(frozenParenthesizedBracketedFinite(alias)); console.log(frozenParenthesizedBracketedNaN(NaN)); console.log(frozenParenthesizedBracketedNaN(1)); console.log(frozenParenthesizedBracketedInteger(alias)); console.log(frozenParenthesizedBracketedSafeInteger(alias)); console.log(frozenParenthesizedPropertyFinite(alias)); console.log(frozenParenthesizedPropertyNaN(NaN)); console.log(frozenParenthesizedPropertyNaN(1)); console.log(frozenParenthesizedPropertyInteger(alias)); console.log(frozenParenthesizedPropertySafeInteger(alias)); console.log(finite(alias)); console.log(integer(alias)); console.log(safeInteger(alias));'
)

CAP_NUMBER_PREDICATES_FREEZE__TEST = (
    'Kali.test(\'number predicates\', () => { const alias = 1; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger; const frozenFinite = Object.freeze(Number.isFinite); const frozenNaN = Object.freeze(Number.isNaN); const frozenInteger = Object.freeze(Number.isInteger); const frozenSafeInteger = Object.freeze(Number.isSafeInteger); const frozenBracketedFinite = Object.freeze(Number["isFinite"]); const frozenBracketedNaN = Object.freeze(Number["isNaN"]); const frozenBracketedInteger = Object.freeze(Number["isInteger"]); const frozenBracketedSafeInteger = Object.freeze(Number["isSafeInteger"]); const frozenParenthesizedBracketedFinite = Object.freeze((globalThis["Number"])["isFinite"]); const frozenParenthesizedBracketedNaN = Object.freeze((globalThis["Number"])["isNaN"]); const frozenParenthesizedBracketedInteger = Object.freeze((globalThis["Number"])["isInteger"]); const frozenParenthesizedBracketedSafeInteger = Object.freeze((globalThis["Number"])["isSafeInteger"]); const frozenParenthesizedPropertyFinite = Object.freeze((globalThis["Number"]).isFinite); const frozenParenthesizedPropertyNaN = Object.freeze((globalThis["Number"]).isNaN); const frozenParenthesizedPropertyInteger = Object.freeze((globalThis["Number"]).isInteger); const frozenParenthesizedPropertySafeInteger = Object.freeze((globalThis["Number"]).isSafeInteger); console.log(Number.isFinite(alias)); console.log(integer(alias)); console.log(Number.isSafeInteger(alias)); console.log(integer(1.5)); console.log(Number.isFinite("hello")); console.log(Number.isSafeInteger(1.5)); console.log(globalThis["Number"]["isNaN"](NaN)); console.log(globalThis.Number.isNaN(1)); console.log(globalThis["Number"].isNaN(1)); console.log(globalThis["Number"]["isFinite"](alias)); console.log(globalThis["Number"]["isInteger"](alias)); console.log(globalThis["Number"]["isSafeInteger"](alias)); console.log(globalThis.Number["isNaN"](1)); console.log(globalThis["Number"].isFinite(alias)); console.log(globalThis.Number["isInteger"](alias)); console.log(globalThis["Number"].isSafeInteger(alias)); console.log(Number["isFinite"](alias)); console.log(Number["isInteger"](alias)); console.log(Number["isSafeInteger"](alias)); console.log(Number["isNaN"](1)); console.log(frozenFinite(alias)); console.log(frozenNaN(NaN)); console.log(frozenNaN(1)); console.log(frozenInteger(alias)); console.log(frozenSafeInteger(alias)); console.log(frozenBracketedFinite(alias)); console.log(frozenBracketedNaN(NaN)); console.log(frozenBracketedNaN(1)); console.log(frozenBracketedInteger(alias)); console.log(frozenBracketedSafeInteger(alias)); console.log(frozenParenthesizedBracketedFinite(alias)); console.log(frozenParenthesizedBracketedNaN(NaN)); console.log(frozenParenthesizedBracketedNaN(1)); console.log(frozenParenthesizedBracketedInteger(alias)); console.log(frozenParenthesizedBracketedSafeInteger(alias)); console.log(frozenParenthesizedPropertyFinite(alias)); console.log(frozenParenthesizedPropertyNaN(NaN)); console.log(frozenParenthesizedPropertyNaN(1)); console.log(frozenParenthesizedPropertyInteger(alias)); console.log(frozenParenthesizedPropertySafeInteger(alias)); console.log(finite(alias)); console.log(integer(alias)); console.log(safeInteger(alias)); });'
)

CAP_OBJECT_HAS_OWN__RUN = (
    'const object = Object.freeze(Object.fromEntries([["a", 1], ["b", 2]]));\n'
    'const alias = object;\n'
    'const wrapped = (0, alias);\n'
    'const hasOwn = Object.hasOwn;\n'
    "const singleQuotedHasOwn = globalThis['Object']['hasOwn'];\n"
    "const parenthesizedSingleQuotedHasOwn = (globalThis['Object'])['hasOwn'];\n"
    "const frozenSingleQuotedHasOwn = Object.freeze(globalThis['Object']['hasOwn']);\n"
    "const frozenParenthesizedSingleQuotedHasOwn = Object.freeze((globalThis['Object'])['hasOwn']);\n"
    'Object.freeze(Object.hasOwn); Object.freeze((Object.hasOwn)); Object.freeze(Object["hasOwn"]); Object.freeze((Object["hasOwn"])); Object.freeze(globalThis.Object.hasOwn); Object.freeze((globalThis.Object.hasOwn)); Object.freeze(globalThis.Object["hasOwn"]); Object.freeze(globalThis.Object[\'hasOwn\']); Object.freeze((globalThis.Object)["hasOwn"]); Object.freeze((globalThis.Object).hasOwn); Object.freeze((globalThis.Object)[\'hasOwn\']); Object.freeze((globalThis.Object["hasOwn"])); Object.freeze(globalThis?.Object.hasOwn); Object.freeze((globalThis?.Object.hasOwn)); Object.freeze((globalThis?.Object).hasOwn); Object.freeze((globalThis?.Object)["hasOwn"]); Object.freeze(globalThis?.Object["hasOwn"]); Object.freeze((globalThis?.Object["hasOwn"])); Object.freeze(globalThis["Object"].hasOwn); Object.freeze((globalThis["Object"].hasOwn)); Object.freeze((globalThis["Object"]).hasOwn); Object.freeze((globalThis["Object"])["hasOwn"]); Object.freeze(globalThis["Object"]["hasOwn"]); Object.freeze((globalThis["Object"]["hasOwn"])); Object.freeze((globalThis["Object"]))["hasOwn"]; Object.freeze((globalThis["Object"]))[\'hasOwn\']; Object.freeze((globalThis[\'Object\']))["hasOwn"]; Object.freeze((globalThis[\'Object\']))[\'hasOwn\']; Object.freeze(globalThis[\'Object\'].hasOwn); Object.freeze((globalThis[\'Object\'].hasOwn)); Object.freeze((globalThis[\'Object\']).hasOwn); Object.freeze((globalThis[\'Object\'])[\'hasOwn\']); Object.freeze(globalThis[\'Object\'][\'hasOwn\']); Object.freeze((globalThis[\'Object\'][\'hasOwn\'])); Object.freeze((null ?? Object.hasOwn)); Object.freeze((true && Object.hasOwn)); Object.freeze((false || Object.hasOwn)); Object.freeze(globalThis.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call)); Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"]); Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"]); Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"])); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call)); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"]); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"])); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call)); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call); Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"])); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call); Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call)); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\']); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)); Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"].hasOwnProperty.call); Object.freeze((globalThis["Object"].hasOwnProperty.call)); Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"])); Object.freeze(Object.prototype.hasOwnProperty.call); Object.freeze((Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype["hasOwnProperty"].call); Object.freeze((Object.prototype["hasOwnProperty"].call)); Object.freeze(Object["prototype"].hasOwnProperty.call); Object.freeze((Object["prototype"].hasOwnProperty.call)); Object.freeze(Object["prototype"]["hasOwnProperty"]["call"]); Object.freeze((Object["prototype"]["hasOwnProperty"]["call"])); Object.freeze((null ?? Object.prototype.hasOwnProperty.call)); Object.freeze((true && Object.prototype.hasOwnProperty.call)); Object.freeze((false || Object.prototype.hasOwnProperty.call)); Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype.hasOwnProperty["call"]); Object.freeze((Object.prototype.hasOwnProperty["call"])); Object.freeze(Object["prototype"].hasOwnProperty["call"]); Object.freeze((Object["prototype"].hasOwnProperty["call"]));\n'
    'if (!Object.hasOwn(wrapped, "a") || !Object["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"]["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"].hasOwn(wrapped, "a") || !singleQuotedHasOwn(wrapped, "a") || !parenthesizedSingleQuotedHasOwn(wrapped, "a") || !frozenSingleQuotedHasOwn(wrapped, "a") || !frozenParenthesizedSingleQuotedHasOwn(wrapped, "a") || !Object.freeze(Object.hasOwn)(wrapped, "a") || !Object.freeze((Object.hasOwn))(wrapped, "a") || !Object.freeze(Object["hasOwn"])(wrapped, "a") || !Object.freeze((Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis.Object.hasOwn)(wrapped, "a") || !Object.freeze((globalThis.Object.hasOwn))(wrapped, "a") || !Object.freeze(globalThis.Object["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis.Object[\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis.Object)["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis.Object).hasOwn)(wrapped, "a") || !Object.freeze((globalThis.Object)[\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis.Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis?.Object.hasOwn)(wrapped, "a") || !Object.freeze((globalThis?.Object.hasOwn))(wrapped, "a") || !Object.freeze((globalThis?.Object).hasOwn)(wrapped, "a") || !Object.freeze((globalThis?.Object)["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis?.Object["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis?.Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis["Object"].hasOwn)(wrapped, "a") || !Object.freeze((globalThis["Object"].hasOwn))(wrapped, "a") || !Object.freeze((globalThis["Object"]).hasOwn)(wrapped, "a") || !Object.freeze((globalThis["Object"])["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis["Object"]["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["hasOwn"]))(wrapped, "a") || !Object.freeze((globalThis["Object"]))["hasOwn"](wrapped, "a") || !Object.freeze((globalThis["Object"]))[\'hasOwn\'](wrapped, "a") || !Object.freeze((globalThis[\'Object\']))["hasOwn"](wrapped, "a") || !Object.freeze((globalThis[\'Object\']))[\'hasOwn\'](wrapped, "a") || !Object.freeze(globalThis[\'Object\'].hasOwn)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].hasOwn))(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).hasOwn)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'])[\'hasOwn\'])(wrapped, "a") || !Object.freeze(globalThis[\'Object\'][\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'][\'hasOwn\']))(wrapped, "a") || !Object.freeze((null ?? Object.hasOwn))(wrapped, "a") || !Object.freeze((true && Object.hasOwn))(wrapped, "a") || !Object.freeze((false || Object.hasOwn))(wrapped, "a") || !Object.freeze(globalThis.Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call)(wrapped, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call))(wrapped, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']))(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call))(wrapped, "a") || !Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(globalThis["Object"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object.prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((Object.prototype["hasOwnProperty"].call))(wrapped, "a") || !Object.freeze(Object["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((Object["prototype"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object["prototype"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((Object["prototype"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze((null ?? Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((true && Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((false || Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object.prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((Object.prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze(Object["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((Object["prototype"].hasOwnProperty["call"]))(wrapped, "a") ||\n'
    '  !Object.prototype.hasOwnProperty.call(wrapped, "a")) {\n'
    "  throw new Error('unexpected frozen Object.hasOwn result');\n"
    '}\n'
    "console.log('frozen object hasOwn ok');\n"
    ''
)

CAP_OBJECT_HAS_OWN__TEST = (
    "Kali.test('frozen object hasOwn', () => {\n"
    '  const object = Object.freeze(Object.fromEntries([["a", 1], ["b", 2]]));\n'
    '  const alias = object;\n'
    '  const wrapped = (0, alias);\n'
    '  const hasOwn = Object.hasOwn;\n'
    "  const singleQuotedHasOwn = globalThis['Object']['hasOwn'];\n"
    "  const parenthesizedSingleQuotedHasOwn = (globalThis['Object'])['hasOwn'];\n"
    "  const frozenSingleQuotedHasOwn = Object.freeze(globalThis['Object']['hasOwn']);\n"
    "  const frozenParenthesizedSingleQuotedHasOwn = Object.freeze((globalThis['Object'])['hasOwn']);\n"
    '  Object.freeze(Object.hasOwn); Object.freeze((Object.hasOwn)); Object.freeze(Object["hasOwn"]); Object.freeze((Object["hasOwn"])); Object.freeze(globalThis.Object.hasOwn); Object.freeze((globalThis.Object.hasOwn)); Object.freeze(globalThis.Object["hasOwn"]); Object.freeze(globalThis.Object[\'hasOwn\']); Object.freeze((globalThis.Object)["hasOwn"]); Object.freeze((globalThis.Object).hasOwn); Object.freeze((globalThis.Object)[\'hasOwn\']); Object.freeze((globalThis.Object["hasOwn"])); Object.freeze(globalThis?.Object.hasOwn); Object.freeze((globalThis?.Object.hasOwn)); Object.freeze((globalThis?.Object).hasOwn); Object.freeze((globalThis?.Object)["hasOwn"]); Object.freeze(globalThis?.Object["hasOwn"]); Object.freeze((globalThis?.Object["hasOwn"])); Object.freeze(globalThis["Object"].hasOwn); Object.freeze((globalThis["Object"].hasOwn)); Object.freeze((globalThis["Object"]).hasOwn); Object.freeze((globalThis["Object"])["hasOwn"]); Object.freeze(globalThis["Object"]["hasOwn"]); Object.freeze((globalThis["Object"]["hasOwn"])); Object.freeze((globalThis["Object"]))["hasOwn"]; Object.freeze((globalThis["Object"]))[\'hasOwn\']; Object.freeze((globalThis[\'Object\']))["hasOwn"]; Object.freeze((globalThis[\'Object\']))[\'hasOwn\']; Object.freeze(globalThis[\'Object\'].hasOwn); Object.freeze((globalThis[\'Object\'].hasOwn)); Object.freeze((globalThis[\'Object\']).hasOwn); Object.freeze((globalThis[\'Object\'])[\'hasOwn\']); Object.freeze(globalThis[\'Object\'][\'hasOwn\']); Object.freeze((globalThis[\'Object\'][\'hasOwn\'])); Object.freeze((null ?? Object.hasOwn)); Object.freeze((true && Object.hasOwn)); Object.freeze((false || Object.hasOwn)); Object.freeze(globalThis.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call)); Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call); Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"]); Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"]); Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"])); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call)); Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"]); Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"])); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call); Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call)); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call); Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"])); Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"]); Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"]); Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call); Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call)); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])); Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\']); Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\']); Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call); Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)); Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"]); Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])); Object.freeze(globalThis["Object"].hasOwnProperty.call); Object.freeze((globalThis["Object"].hasOwnProperty.call)); Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"]); Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"])); Object.freeze(Object.prototype.hasOwnProperty.call); Object.freeze((Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype["hasOwnProperty"].call); Object.freeze((Object.prototype["hasOwnProperty"].call)); Object.freeze(Object["prototype"].hasOwnProperty.call); Object.freeze((Object["prototype"].hasOwnProperty.call)); Object.freeze(Object["prototype"]["hasOwnProperty"]["call"]); Object.freeze((Object["prototype"]["hasOwnProperty"]["call"])); Object.freeze((null ?? Object.prototype.hasOwnProperty.call)); Object.freeze((true && Object.prototype.hasOwnProperty.call)); Object.freeze((false || Object.prototype.hasOwnProperty.call)); Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call)); Object.freeze(Object.prototype.hasOwnProperty["call"]); Object.freeze((Object.prototype.hasOwnProperty["call"])); Object.freeze(Object["prototype"].hasOwnProperty["call"]); Object.freeze((Object["prototype"].hasOwnProperty["call"]));\n'
    '  if (!Object.hasOwn(wrapped, "a") || !Object["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"]["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"].hasOwn(wrapped, "a") || !singleQuotedHasOwn(wrapped, "a") || !parenthesizedSingleQuotedHasOwn(wrapped, "a") || !frozenSingleQuotedHasOwn(wrapped, "a") || !frozenParenthesizedSingleQuotedHasOwn(wrapped, "a") || !Object.freeze(Object.hasOwn)(wrapped, "a") || !Object.freeze((Object.hasOwn))(wrapped, "a") || !Object.freeze(Object["hasOwn"])(wrapped, "a") || !Object.freeze((Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis.Object.hasOwn)(wrapped, "a") || !Object.freeze((globalThis.Object.hasOwn))(wrapped, "a") || !Object.freeze(globalThis.Object["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis.Object[\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis.Object)["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis.Object).hasOwn)(wrapped, "a") || !Object.freeze((globalThis.Object)[\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis.Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis?.Object.hasOwn)(wrapped, "a") || !Object.freeze((globalThis?.Object.hasOwn))(wrapped, "a") || !Object.freeze((globalThis?.Object).hasOwn)(wrapped, "a") || !Object.freeze((globalThis?.Object)["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis?.Object["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis?.Object["hasOwn"]))(wrapped, "a") || !Object.freeze(globalThis["Object"].hasOwn)(wrapped, "a") || !Object.freeze((globalThis["Object"].hasOwn))(wrapped, "a") || !Object.freeze((globalThis["Object"]).hasOwn)(wrapped, "a") || !Object.freeze((globalThis["Object"])["hasOwn"])(wrapped, "a") || !Object.freeze(globalThis["Object"]["hasOwn"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["hasOwn"]))(wrapped, "a") || !Object.freeze((globalThis["Object"]))["hasOwn"](wrapped, "a") || !Object.freeze((globalThis["Object"]))[\'hasOwn\'](wrapped, "a") || !Object.freeze((globalThis[\'Object\']))["hasOwn"](wrapped, "a") || !Object.freeze((globalThis[\'Object\']))[\'hasOwn\'](wrapped, "a") || !Object.freeze(globalThis[\'Object\'].hasOwn)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].hasOwn))(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).hasOwn)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'])[\'hasOwn\'])(wrapped, "a") || !Object.freeze(globalThis[\'Object\'][\'hasOwn\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'][\'hasOwn\']))(wrapped, "a") || !Object.freeze((null ?? Object.hasOwn))(wrapped, "a") || !Object.freeze((true && Object.hasOwn))(wrapped, "a") || !Object.freeze((false || Object.hasOwn))(wrapped, "a") || !Object.freeze(globalThis.Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object).prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis?.Object["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis?.Object["prototype"].hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis?.Object)["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'].call)(wrapped, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]).prototype["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"])["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze(globalThis["Object"].prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((globalThis["Object"].prototype["hasOwnProperty"].call))(wrapped, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'][\'call\']))(wrapped, "a") || !Object.freeze((globalThis[\'Object\']).prototype[\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze((globalThis[\'Object\'])[\'prototype\'][\'hasOwnProperty\'][\'call\'])(wrapped, "a") || !Object.freeze(globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call)(wrapped, "a") || !Object.freeze((globalThis[\'Object\'].prototype[\'hasOwnProperty\'].call))(wrapped, "a") || !Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(globalThis["Object"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((globalThis["Object"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(globalThis["Object"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((globalThis["Object"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze(Object.prototype.hasOwnProperty.call)(wrapped, "a") || !Object.freeze((Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object.prototype["hasOwnProperty"].call)(wrapped, "a") || !Object.freeze((Object.prototype["hasOwnProperty"].call))(wrapped, "a") || !Object.freeze(Object["prototype"].hasOwnProperty.call)(wrapped, "a") || !Object.freeze((Object["prototype"].hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object["prototype"]["hasOwnProperty"]["call"])(wrapped, "a") || !Object.freeze((Object["prototype"]["hasOwnProperty"]["call"]))(wrapped, "a") || !Object.freeze((null ?? Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((true && Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((false || Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((null ?? globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((true && globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze((false || globalThis.Object.prototype.hasOwnProperty.call))(wrapped, "a") || !Object.freeze(Object.prototype.hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((Object.prototype.hasOwnProperty["call"]))(wrapped, "a") || !Object.freeze(Object["prototype"].hasOwnProperty["call"])(wrapped, "a") || !Object.freeze((Object["prototype"].hasOwnProperty["call"]))(wrapped, "a") ||\n'
    '    !Object.prototype.hasOwnProperty.call(wrapped, "a")) {\n'
    "    throw new Error('unexpected frozen Object.hasOwn result');\n"
    '  }\n'
    '});\n'
    ''
)

CAP_PARSE_INT__SUPPORTED = (
    "console.log(parseInt('42'));\n"
    "console.log(globalThis.parseInt('-0x10'));\n"
    "console.log(Number.parseInt('ff', 16));\n"
    'console.log(globalThis["Number"]["parseInt"](\'101\', 2));\n'
    'const frozenParseInt = Object.freeze(parseInt);\n'
    'const frozenNumberParseInt = Object.freeze(globalThis["Number"]["parseInt"]);\n'
    "console.log(frozenParseInt(Object.freeze('77'), 8));\n"
    "console.log(frozenNumberParseInt('10', Object.freeze(2)));\n"
    ''
)

CAP_PROMISE_ANY__RUN = (
    'async function promiseAnySmoke() {\n'
    "  const winner = await Promise.any([Promise.reject('boom'), Promise.resolve(1)]);\n"
    '  if (winner !== 1) {\n'
    "    throw new Error('unexpected Promise.any sequencing');\n"
    '  }\n'
    '}\n'
    '\n'
    'async function main() {\n'
    '  await promiseAnySmoke();\n'
    "  console.log('promise any ok');\n"
    '}\n'
    '\n'
    'main();\n'
    ''
)

CAP_PROMISE_ANY__TEST = (
    'async function promiseAnySmoke() {\n'
    "  const winner = await Promise.any([Promise.reject('boom'), Promise.resolve(1)]);\n"
    '  if (winner !== 1) {\n'
    "    throw new Error('unexpected Promise.any sequencing');\n"
    '  }\n'
    '}\n'
    '\n'
    "Kali.test('promise any', () => promiseAnySmoke());\n"
    ''
)

CAP_PROMISE_RACE__RUN = (
    'async function promiseRaceSmoke() {\n'
    '  const winner = await Promise.race([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  if (winner !== 1) {\n'
    "    throw new Error('unexpected Promise.race sequencing');\n"
    '  }\n'
    '}\n'
    '\n'
    'async function main() {\n'
    '  await promiseRaceSmoke();\n'
    "  console.log('promise race ok');\n"
    '}\n'
    '\n'
    'main();\n'
    ''
)

CAP_PROMISE_RACE__TEST = (
    'async function promiseRaceSmoke() {\n'
    '  const winner = await Promise.race([Promise.resolve(1), Promise.resolve(2)]);\n'
    '  if (winner !== 1) {\n'
    "    throw new Error('unexpected Promise.race sequencing');\n"
    '  }\n'
    '}\n'
    '\n'
    "Kali.test('promise race', () => promiseRaceSmoke());\n"
    ''
)

CAP_REFLECT_OWN_KEYS__RUN = (
    'const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };\n'
    'const keys = globalThis.Reflect.ownKeys(obj);\n'
    'const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);\n'
    'const mixedBracketedDirectKeys = globalThis["Reflect"][\'ownKeys\'](obj);\n'
    'const mixedSingleQuotedDirectKeys = globalThis[\'Reflect\']["ownKeys"](obj);\n'
    'const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);\n'
    'const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);\n'
    'const fullyBracketedKeys = globalThis["Reflect"]["ownKeys"](obj);\n'
    'const parenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj);\n'
    'const frozenBracketRootKeys = Object.freeze((globalThis["Reflect"]))["ownKeys"](obj);\n'
    "const singleQuotedKeys = globalThis['Reflect']['ownKeys'](obj);\n"
    "const singleQuotedMixedBracketedKeys = globalThis.Reflect['ownKeys'](obj);\n"
    'const frozenBareCallableKeys = Object.freeze(Reflect.ownKeys)(obj); const parenthesizedFrozenBareCallableKeys = Object.freeze((Reflect.ownKeys))(obj); const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)(obj); const mixedBracketedRootKeys = Object.freeze(globalThis["Reflect"][\'ownKeys\'])(obj); const parenthesizedMixedBracketedRootKeys = Object.freeze((globalThis["Reflect"][\'ownKeys\']))(obj); const mixedSingleQuotedRootKeys = Object.freeze(globalThis[\'Reflect\']["ownKeys"])(obj); const parenthesizedMixedSingleQuotedRootKeys = Object.freeze((globalThis[\'Reflect\']["ownKeys"]))(obj); const frozenMixedBracketedKeys = Object.freeze(globalThis.Reflect["ownKeys"])(obj); const frozenSingleQuotedMixedBracketedKeys = Object.freeze(globalThis.Reflect[\'ownKeys\'])(obj); const parenthesizedFrozenSingleQuotedMixedBracketedKeys = Object.freeze((globalThis.Reflect[\'ownKeys\']))(obj); const nullishFrozenCallableKeys = Object.freeze((null ?? globalThis.Reflect.ownKeys))(obj); const logicalAndFrozenCallableKeys = Object.freeze((true && globalThis.Reflect.ownKeys))(obj); const logicalOrFrozenCallableKeys = Object.freeze((false || globalThis.Reflect.ownKeys))(obj); const frozenMixedRootKeys = Object.freeze(globalThis["Reflect"].ownKeys)(obj); const parenthesizedFrozenDotRootBracketedKeys = Object.freeze((globalThis.Reflect)["ownKeys"])(obj); const frozenParenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"]).ownKeys)(obj); const frozenParenthesizedSingleQuotedBracketRootKeys = Object.freeze((globalThis[\'Reflect\']).ownKeys)(obj); const frozenParenthesizedBracketedRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj); const frozenParenthesizedMixedRootKeys = Object.freeze((globalThis["Reflect"].ownKeys))(obj); const frozenBracketedKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])(obj); const parenthesizedFrozenMixedBracketedKeys = Object.freeze((globalThis.Reflect["ownKeys"]))(obj); const parenthesizedFrozenBracketedKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))(obj); const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))(obj); const frozenSingleQuotedRootKeys = Object.freeze(globalThis[\'Reflect\'].ownKeys)(obj); const nullishFrozenBracketedKeys = Object.freeze((null ?? globalThis["Reflect"]["ownKeys"]))(obj); const logicalAndFrozenBracketedKeys = Object.freeze((true && globalThis["Reflect"]["ownKeys"]))(obj); const logicalOrFrozenBracketedKeys = Object.freeze((false || globalThis["Reflect"]["ownKeys"]))(obj); const frozenParenthesizedSingleQuotedRootKeys = Object.freeze((globalThis[\'Reflect\']).ownKeys)(obj); const frozenParenthesizedSingleQuotedBracketedKeys = Object.freeze((globalThis[\'Reflect\'])[\'ownKeys\'])(obj); const frozenSingleQuotedBracketedKeys = Object.freeze(globalThis[\'Reflect\'][\'ownKeys\'])(obj); const parenthesizedFrozenSingleQuotedRootKeys = Object.freeze((globalThis[\'Reflect\'].ownKeys))(obj); const parenthesizedFrozenSingleQuotedBracketedKeys = Object.freeze((globalThis[\'Reflect\'][\'ownKeys\']))(obj); const frozenNullishCallableKeys = Object.freeze((null ?? Reflect.ownKeys))(obj); const frozenLogicalAndCallableKeys = Object.freeze((true && Reflect.ownKeys))(obj); const frozenLogicalOrCallableKeys = Object.freeze((false || Reflect.ownKeys))(obj); const conditionalFrozenCallableKeys = Object.freeze((true ? Reflect.ownKeys : Reflect.ownKeys))(obj); const conditionalFrozenGlobalCallableKeys = Object.freeze((true ? globalThis.Reflect.ownKeys : globalThis.Reflect.ownKeys))(obj);\n'
    'let syncCount = 0;\n'
    'for (const key of Reflect.ownKeys(obj)) {\n'
    '  syncCount += 1;\n'
    '}\n'
    'let sequenceCount = 0;\n'
    'for (const key of (0, Reflect.ownKeys(obj))) {\n'
    '  sequenceCount += 1;\n'
    '}\n'
    'let mixedSequenceCount = 0;\n'
    'for (const key of (0, globalThis["Reflect"]["ownKeys"](obj))) {\n'
    '  mixedSequenceCount += 1;\n'
    '}\n'
    'let breakContinueCount = 0;\n'
    'for (const key of Reflect.ownKeys(obj)) {\n'
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
    '  parenthesizedBracketRootKeys.length !== 4 ||\n'
    "  parenthesizedBracketRootKeys[0] !== '1' ||\n"
    "  parenthesizedBracketRootKeys[1] !== '2' ||\n"
    "  parenthesizedBracketRootKeys[2] !== 'b' ||\n"
    "  parenthesizedBracketRootKeys[3] !== 'a' ||\n"
    '  frozenBracketRootKeys.length !== 4 ||\n'
    "  frozenBracketRootKeys[0] !== '1' ||\n"
    "  frozenBracketRootKeys[1] !== '2' ||\n"
    "  frozenBracketRootKeys[2] !== 'b' ||\n"
    "  frozenBracketRootKeys[3] !== 'a' ||\n"
    '  singleQuotedKeys.length !== 4 ||\n'
    "  singleQuotedKeys[0] !== '1' ||\n"
    "  singleQuotedKeys[1] !== '2' ||\n"
    "  singleQuotedKeys[2] !== 'b' ||\n"
    "  singleQuotedKeys[3] !== 'a' ||\n"
    '  singleQuotedMixedBracketedKeys.length !== 4 ||\n'
    "  singleQuotedMixedBracketedKeys[0] !== '1' ||\n"
    "  singleQuotedMixedBracketedKeys[1] !== '2' ||\n"
    "  singleQuotedMixedBracketedKeys[2] !== 'b' ||\n"
    "  singleQuotedMixedBracketedKeys[3] !== 'a' ||\n"
    '  syncCount !== 4 ||\n'
    '  sequenceCount !== 4 ||\n'
    '  mixedSequenceCount !== 4 ||\n'
    '  breakContinueCount !== 1\n'
    ') {\n'
    "  throw new Error('unexpected Reflect.ownKeys ordering');\n"
    '}\n'
    "console.log('reflect ownKeys ok');\n"
    ''
)

CAP_REFLECT_OWN_KEYS__TEST = (
    "Kali.test('reflect ownKeys', () => {\n"
    '  const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };\n'
    '  const keys = globalThis.Reflect.ownKeys(obj);\n'
    '  const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);\n'
    '  const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);\n'
    '  const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);\n'
    '  const fullyBracketedKeys = globalThis["Reflect"]["ownKeys"](obj);\n'
    '  const parenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj);\n'
    "  const singleQuotedKeys = globalThis['Reflect']['ownKeys'](obj);\n"
    "const singleQuotedMixedBracketedKeys = globalThis.Reflect['ownKeys'](obj);\n"
    '  const frozenBareCallableKeys = Object.freeze(Reflect.ownKeys)(obj);\n'
    'const parenthesizedFrozenBareCallableKeys = Object.freeze((Reflect.ownKeys))(obj);\n'
    'const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)(obj);\n'
    '  const frozenMixedBracketedKeys = Object.freeze(globalThis.Reflect["ownKeys"])(obj);\n'
    '  const frozenBracketedKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])(obj);\n'
    '  const parenthesizedFrozenBracketedKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))(obj);\n'
    '  const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))(obj);\n'
    '  let syncCount = 0;\n'
    '  for (const key of Reflect.ownKeys(obj)) {\n'
    '    syncCount += 1;\n'
    '  }\n'
    '  let sequenceCount = 0;\n'
    '  for (const key of (0, Reflect.ownKeys(obj))) {\n'
    '    sequenceCount += 1;\n'
    '  }\n'
    '  let mixedSequenceCount = 0;\n'
    '  for (const key of (0, globalThis["Reflect"]["ownKeys"](obj))) {\n'
    '    mixedSequenceCount += 1;\n'
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
    '    fullyBracketedKeys.length !== 4 ||\n'
    "    fullyBracketedKeys[0] !== '1' ||\n"
    "    fullyBracketedKeys[1] !== '2' ||\n"
    "    fullyBracketedKeys[2] !== 'b' ||\n"
    "    fullyBracketedKeys[3] !== 'a' ||\n"
    '    parenthesizedBracketRootKeys.length !== 4 ||\n'
    "    parenthesizedBracketRootKeys[0] !== '1' ||\n"
    "    parenthesizedBracketRootKeys[1] !== '2' ||\n"
    "    parenthesizedBracketRootKeys[2] !== 'b' ||\n"
    "    parenthesizedBracketRootKeys[3] !== 'a' ||\n"
    '    singleQuotedKeys.length !== 4 ||\n'
    '    frozenBareCallableKeys.length !== 4 ||\n'
    '    parenthesizedFrozenBareCallableKeys.length !== 4 ||\n'
    '    frozenCallableKeys.length !== 4 ||\n'
    '    frozenMixedBracketedKeys.length !== 4 ||\n'
    '    frozenBracketedKeys.length !== 4 ||\n'
    '    parenthesizedFrozenBracketedKeys.length !== 4 ||\n'
    '    parenthesizedFrozenCallableKeys.length !== 4 ||\n'
    "    singleQuotedKeys[0] !== '1' ||\n"
    "    singleQuotedKeys[1] !== '2' ||\n"
    "    singleQuotedKeys[2] !== 'b' ||\n"
    "    singleQuotedKeys[3] !== 'a' ||\n"
    "    frozenBareCallableKeys[0] !== '1' ||\n"
    "    frozenBareCallableKeys[1] !== '2' ||\n"
    "    frozenBareCallableKeys[2] !== 'b' ||\n"
    "    frozenBareCallableKeys[3] !== 'a' ||\n"
    "    parenthesizedFrozenBareCallableKeys[0] !== '1' ||\n"
    "    parenthesizedFrozenBareCallableKeys[1] !== '2' ||\n"
    "    parenthesizedFrozenBareCallableKeys[2] !== 'b' ||\n"
    "    parenthesizedFrozenBareCallableKeys[3] !== 'a' ||\n"
    '    syncCount !== 4 ||\n'
    '    sequenceCount !== 4 ||\n'
    '    mixedSequenceCount !== 4 ||\n'
    '    breakContinueCount !== 1\n'
    '  ) {\n'
    "    throw new Error('unexpected Reflect.ownKeys ordering');\n"
    '  }\n'
    '});\n'
    ''
)

CAP_STANDALONE_ITER__MAP = (
    'function main() {\n'
    '  let values = [[1, 2], [3, 4]];\n'
    '  values = values;\n'
    '  for (const entry of new Map(values.filter(Boolean))) {\n'
    '    console.log(entry[0]);\n'
    '    console.log(entry[1]);\n'
    '  }\n'
    '}\n'
    'main();\n'
    ''
)

CAP_STANDALONE_ITER__SET = (
    'function main() {\n'
    '  let values = [1, 2];\n'
    '  values = values;\n'
    '  for (const value of new Set(values.filter(Boolean))) {\n'
    '    console.log(value);\n'
    '  }\n'
    '}\n'
    'main();\n'
    ''
)
