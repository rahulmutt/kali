//! Task 18 batch 8A: U4 TRIM-AND-KEEP. 16 of this target's 44 `#[test]` fns are
//! retained here; the other 28 are migrated.
//!
//! PRE-TRIM REF: b57714094b12bf429b0ccb7985f82704f3a37803
//!
//! WHAT IS RETAINED, AND WHAT WENT. This is a `#[path]` submodule carrier
//! (U10): `grep -c '#[test]'` on THIS file returns 0, and every test lives in
//! the sibling directory. Before the trim that directory held four files --
//! run.rs (16 fns), test.rs (16), build.rs (8), check.rs (4). ONE SUBMODULE
//! STAYS AND THREE GO: `test.rs` is retained with all 16 of its fns; run.rs,
//! build.rs and check.rs are migrated and deleted. The carrier keeps the two
//! fixture builders and the three shared assert helpers `test.rs` reaches, and
//! loses the bundle fixture builder and the build/check helpers, which nothing
//! retained calls.
//!
//! FOR BATCH 8C, WHICH PERFORMS THE FAMILY-WIDE DELETION -- WHAT IT MAY AND MAY
//! NOT REMOVE. This target is NOT fully migrated and must NOT be deleted:
//!   * KEEP  tests/browser_reflect_own_keys.rs (this file)
//!   * KEEP  tests/browser_reflect_own_keys/test.rs
//!   * already gone: tests/browser_reflect_own_keys/{run,build,check}.rs
//! Deleting the carrier would delete the retained 16 tests with it, because the
//! carrier holds every helper they call. The sibling DIRECTORY survives with a
//! single file in it; U10's "delete the directory too" applies to a fully
//! migrated carrier, which this is not.
//!
//! GROUND FOR THE RETENTION: A CLAIM FROM UNREACHABLE CODE.
//! Three of this file's shared assert helpers --
//! `assert_browser_requested_reflect_own_keys` (`:509`),
//! `assert_json_browser_requested_reflect_own_keys` (`:544`) and
//! `assert_inherited_browser_api_surface_reflect_own_keys` (`:599`) --
//! each carry an `if command == "run"` branch asserting
//! `stdout.contains("reflect ownKeys ok")` (`:538`), and again at
//! `stdout.contains("reflect ownKeys ok")` (`:672`); two of them assert it on
//! the JSON stdout leaf instead, at
//! `.contains("reflect ownKeys ok"),` (`:586`) and again at
//! `.contains("reflect ownKeys ok"),` (`:657`).
//! NO `#[test]` FN REACHES ANY OF THEM, before the trim or after it. PR #16
//! rev2's honest re-pin moved every `run` caller into run.rs's own local
//! `_fails_closed` variants, leaving these three helpers with 16 call sites --
//! all in the retained `test.rs`, all passing `"test"`.
//!
//! HOW TO CHECK THAT, AND THE WAY IT GOES WRONG. A crude `grep '"run"'` over the
//! submodules returns 16 literals and reads like a direct contradiction. It is
//! not: those 16 are calls to the `_fails_closed` VARIANTS -- different
//! functions, local to run.rs, which do not carry the claim at all. The
//! predicate has to be the ENCLOSING FUNCTION of each `.contains("reflect
//! ownKeys ok")` site, not the presence of the string `"run"` somewhere nearby.
//! Checked the right way, over the pre-trim tree:
//!
//!     $ cd crates/kali_cli/tests && grep -rn \
//!       'assert_browser_requested_reflect_own_keys(\|assert_json_browser_requested_reflect_own_keys(\|assert_inherited_browser_api_surface_reflect_own_keys(' \
//!       browser_reflect_own_keys/ | grep -v _fails_closed | grep -c '"test"'
//!     16          # of 16 call sites; none passes "run"
//!
//! `audit-case-migration.py` extracts `.contains` literals with a regex and has
//! no reachability analysis, so it reports `[contains literals] 'reflect
//! ownKeys ok'` absent from any migration of this target. That cannot be
//! discharged honestly: no test asserts it, so rule 2 forbids inventing it; the
//! `run` cases fail closed and emit no such stdout, so the claim would also be
//! FALSE; and `[source]` is excluded from the audit's case-side search by
//! design (U8), so the fixture's own trailing `console.log('reflect ownKeys
//! ok')` cannot discharge it either.
//!
//! WHY A TRIM AND NOT A WHOLE-FILE RETENTION -- AND R1 IS BOUNDED, NOT SET
//! ASIDE. Ledger ruling R1 says an unreachable-code claim keeps its target
//! hand-written per design spec 5.11. Read alone it would retain all 44 fns
//! here. The human partner ruled otherwise, and the ruling is narrow: R1's
//! purpose was keeping the audit GATE clean -- no per-file bypass, no
//! reachability analysis in the script -- not maximising retention. U4 was
//! added later precisely because "keep this file hand-written" was being used
//! as a starting hypothesis rather than an answer, and retaining 44 tests to
//! preserve a claim that NO TEST MAKES is the over-retention U4 exists to stop.
//!
//! SO: R1 STILL GOVERNS wherever the retained set is non-empty by reachability
//! -- that is the ordinary case, and the five Class A files already on the
//! family's keep list are unaffected. Where ZERO tests reach the construct, as
//! here, U4's trim decides the split instead. A later reader should not take
//! this file as R1 being abandoned.
//!
//! The retained half is the set whose helpers hold the dead branch: `test.rs`.
//! The migrated half is everything that does not -- run.rs, build.rs, check.rs
//! -- and it went to TWO case files, split on manifest presence per U2:
//! cases/browser/reflect_own_keys_explicit_api.toml (20 fns) and
//! cases/browser/reflect_own_keys_inherited_manifest.toml (8 fns). The split is
//! on the manifest and NOT on the submodule boundary: run.rs straddles it 8/8,
//! so one file per submodule would leak `kali.json` into the explicit cases.
//! Both case-file headers carry the measurement.
//!
//! CONSEQUENCE FOR THE GATES (rulings 9 and 12). This is a trim, so the on-disk
//! `.rs` is shorter than the source the case files were migrated from, and the
//! literal-comparison gates need the right left-hand side. THREE COLUMNS:
//!
//!   gate                                 post-trim  pre-trim  complement
//!   audit-case-migration.py                 RED        RED       green
//!   check_extra_claims.py                   RED        green     green
//!   check_fixtures.py        (explicit)     RED        RED       green
//!   check_fixtures.py        (inherited)    RED        RED       RED    <- below
//!   comment_coverage.py      (explicit)     RED        RED       RED    <- below
//!   comment_coverage.py      (inherited)    RED        green     green
//!   check_rationale_fn_names.py (both)      RED        RED       RED    <- below
//!
//! EVERY CELL ABOVE WAS RUN, not reasoned about; three of them came back the
//! opposite of a first draft of this paragraph, which is why the table is
//! measured rather than derived from the shape of the trim.
//!
//! TO REPRODUCE THE PRE-TRIM COLUMN, MATERIALISE THE SUBMODULES BESIDE THE
//! BLOB. `git show <ref>:...browser_reflect_own_keys.rs > /tmp/x.rs` alone
//! gives a carrier whose `#[path]` declarations resolve to nothing, and the
//! gates then read a source with no `#[test]` fns and no submodule literals in
//! it -- which reports `check_extra_claims.py` RED on filenames that are
//! plainly there. Reproduce the whole directory, exactly as
//! `citation_sweep.sh` now does for a trimmed carrier:
//!
//!     $ ref=$(grep -oP '(?<=PRE-TRIM REF: )[0-9a-f]{40}' \
//!         crates/kali_cli/tests/browser_reflect_own_keys.rs)
//!     $ mkdir -p /tmp/pt/browser_reflect_own_keys
//!     $ git show $ref:crates/kali_cli/tests/browser_reflect_own_keys.rs \
//!         > /tmp/pt/browser_reflect_own_keys.rs
//!     $ for m in run build check test; do git show \
//!         $ref:crates/kali_cli/tests/browser_reflect_own_keys/$m.rs \
//!         > /tmp/pt/browser_reflect_own_keys/$m.rs; done
//!
//! This target NEEDS the third column (ruling 12): the retained `test.rs` half
//! carries literal claims of its own -- `ok 1`, and the envelope's
//! total/passed/failed, hostContract and runtimeBackend -- so the audit is red
//! against the pre-trim blob (those claims are in it and in no case file) AND
//! red against the post-trim file (the migrated claims are no longer in it).
//! Build the correct left-hand side mechanically, which also runs the audit:
//!
//!     $ python3 tools/task-18-browser-pilot/migrated_complement.py --carrier \
//!         crates/kali_cli/tests/browser_reflect_own_keys.rs --audit \
//!         crates/kali_cli/tests/cases/browser/reflect_own_keys_explicit_api.toml \
//!         crates/kali_cli/tests/cases/browser/reflect_own_keys_inherited_manifest.toml
//!     AUDIT OK -- every literal claim is present in the case files.
//!
//! THE THREE CELLS THAT STAY RED ON THE COMPLEMENT ARE U2-SPLIT, U6 AND U8
//! ARTIFACTS, NOT DEFECTS, and none is peculiar to this target:
//!
//!   * `check_fixtures.py` on the INHERITED half reports the bundle fixture and
//!     the bundle-harness body "not present verbatim". They are not supposed to
//!     be: they belong to the `build` cases, which live in the EXPLICIT half.
//!     The gate takes one case file and the migrated half is legitimately two,
//!     so each half is missing the other's fixtures by construction. Batch 6B's
//!     already-shipped two-file U2 split behaves identically -- measured:
//!     `non_literal_iterator_sources_explicit_api` green,
//!     `..._inherited_manifest` RED.
//!
//!   * `comment_coverage.py` on the EXPLICIT half reports run.rs's comment
//!     block missing from that file's 12 build/check cases. It IS missing, and
//!     U6 requires it to be: the block sits in `run.rs` and reaches only the 8
//!     run-derived cases. Copying it into the other 12 would turn the checker
//!     green and violate U6. Post-trim it is red for a duller reason: the
//!     trimmed source carries no Rust comment at all, so the ruling-5 floor
//!     fires on "0 comment lines checked".
//!
//!   * `check_rationale_fn_names.py` reports FOUR residual names per half, and
//!     all four are benign. Named rather than left as a count:
//!       - `reflect_own_keys_source`, `reflect_own_keys_test_source`: real
//!         carrier fns, but RETAINED, so absent from the complement by
//!         construction. Present in the post-trim file; the gate is red there
//!         for the mirror-image reason.
//!       - `reflect_own_keys_frozen_callable_source`: a real `kali_common` fn,
//!         the owner of the rule-13 doc this file carries. Never in any `.rs`
//!         under tests/.
//!       - `check_accepts_reflect_own_keys_in_js_input` (explicit half only): a
//!         NEGATED mention -- the header says this fn does not exist, which is
//!         exactly why `check.rs` covers only jsx/tsx. Substring matching has no
//!         sentence boundary (ruling 18's own note), so the gate reads the
//!         denial as a citation.
//!       - `wasmtime` (inherited half only): a runtime name in the U2 paragraph
//!         recording which measurement was retired, not an identifier at all.
//!     None is fixable reader-side without deleting true prose, and none was
//!     resolved by widening the gate's ALLOW list.
//!
//! EVERY `:N` CITATION IN THE TWO CASE FILES IS A PRE-TRIM LINE NUMBER, against
//! the ref declared above. The citations in THIS header are post-trim numbers,
//! against this file as shipped.
//!
//! Full reasoning: .superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch8a-report.md.
//!
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_common::reflect_own_keys_frozen_callable_source;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn reflect_own_keys_source() -> String {
    let frozen_callable_lines = reflect_own_keys_frozen_callable_source("obj");
    format!(
        r#"const obj = {{ "b": 1, "2": 2, "a": 3, "1": 4 }};
const frozenObj = Object.freeze(obj);
const keys = globalThis.Reflect.ownKeys(obj);
const frozenKeys = globalThis.Reflect.ownKeys(frozenObj);
const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);
const mixedBracketedDirectKeys = globalThis["Reflect"]['ownKeys'](obj);
const mixedSingleQuotedDirectKeys = globalThis['Reflect']["ownKeys"](obj);
const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);
const singleQuotedPropertyKeys = globalThis['Reflect'].ownKeys(obj);
const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
const fullyBracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
const parenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj);
const frozenBracketRootKeys = Object.freeze((globalThis["Reflect"]))["ownKeys"](obj);
const singleQuotedKeys = globalThis['Reflect']['ownKeys'](obj);
const singleQuotedMixedBracketedKeys = globalThis.Reflect['ownKeys'](obj);
const frozenSingleQuotedKeys = globalThis['Reflect']['ownKeys'](frozenObj);
const parenthesizedFrozenSingleQuotedKeys = Object.freeze((globalThis['Reflect']['ownKeys']))(obj);
const parenthesizedFrozenSingleQuotedFrozenKeys = Object.freeze((globalThis['Reflect']['ownKeys']))(frozenObj);
{frozen_callable_lines}
let syncCount = 0;
for (const key of globalThis.Reflect.ownKeys(obj)) {{
  syncCount += 1;
}}
let frozenSyncCount = 0;
for (const key of globalThis.Reflect.ownKeys(frozenObj)) {{
  frozenSyncCount += 1;
}}
let sequenceCount = 0;
for (const key of (0, globalThis.Reflect.ownKeys(obj))) {{
  sequenceCount += 1;
}}
let frozenSequenceCount = 0;
for (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {{
  frozenSequenceCount += 1;
}}
let asyncCount = 0;
for await (const key of globalThis.Reflect.ownKeys(obj)) {{
  asyncCount += 1;
}}
let frozenAsyncCount = 0;
for await (const key of globalThis.Reflect.ownKeys(frozenObj)) {{
  frozenAsyncCount += 1;
}}
let asyncSequenceCount = 0;
for await (const key of (0, globalThis.Reflect.ownKeys(obj))) {{
  asyncSequenceCount += 1;
}}
let frozenAsyncSequenceCount = 0;
for await (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {{
  frozenAsyncSequenceCount += 1;
}}
let breakContinueCount = 0;
for (const key of globalThis.Reflect.ownKeys(obj)) {{
  if (key === '1') {{
    continue;
  }}
  breakContinueCount += 1;
  break;
}}
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  frozenKeys.length !== 4 ||
  frozenKeys[0] !== '1' ||
  frozenKeys[1] !== '2' ||
  frozenKeys[2] !== 'b' ||
  frozenKeys[3] !== 'a' ||
  mixedRootKeys.length !== 4 ||
  mixedRootKeys[0] !== '1' ||
  mixedRootKeys[1] !== '2' ||
  mixedRootKeys[2] !== 'b' ||
  mixedRootKeys[3] !== 'a' ||
  mixedBracketedDirectKeys.length !== 4 ||
  mixedBracketedDirectKeys[0] !== '1' ||
  mixedBracketedDirectKeys[1] !== '2' ||
  mixedBracketedDirectKeys[2] !== 'b' ||
  mixedBracketedDirectKeys[3] !== 'a' ||
  mixedSingleQuotedDirectKeys.length !== 4 ||
  mixedSingleQuotedDirectKeys[0] !== '1' ||
  mixedSingleQuotedDirectKeys[1] !== '2' ||
  mixedSingleQuotedDirectKeys[2] !== 'b' ||
  mixedSingleQuotedDirectKeys[3] !== 'a' ||
  mixedBracketedKeys.length !== 4 ||
  mixedBracketedKeys[0] !== '1' ||
  mixedBracketedKeys[1] !== '2' ||
  mixedBracketedKeys[2] !== 'b' ||
  mixedBracketedKeys[3] !== 'a' ||
  singleQuotedPropertyKeys.length !== 4 ||
  singleQuotedPropertyKeys[0] !== '1' ||
  singleQuotedPropertyKeys[1] !== '2' ||
  singleQuotedPropertyKeys[2] !== 'b' ||
  singleQuotedPropertyKeys[3] !== 'a' ||
  bracketedKeys.length !== 4 ||
  bracketedKeys[0] !== '1' ||
  bracketedKeys[1] !== '2' ||
  bracketedKeys[2] !== 'b' ||
  bracketedKeys[3] !== 'a' ||
  fullyBracketedKeys.length !== 4 ||
  fullyBracketedKeys[0] !== '1' ||
  fullyBracketedKeys[1] !== '2' ||
  fullyBracketedKeys[2] !== 'b' ||
  fullyBracketedKeys[3] !== 'a' ||
    parenthesizedBracketRootKeys.length !== 4 ||
    parenthesizedBracketRootKeys[0] !== '1' ||
    parenthesizedBracketRootKeys[1] !== '2' ||
    parenthesizedBracketRootKeys[2] !== 'b' ||
    parenthesizedBracketRootKeys[3] !== 'a' ||
    frozenBracketRootKeys.length !== 4 ||
    frozenBracketRootKeys[0] !== '1' ||
    frozenBracketRootKeys[1] !== '2' ||
    frozenBracketRootKeys[2] !== 'b' ||
    frozenBracketRootKeys[3] !== 'a' ||
    singleQuotedKeys.length !== 4 ||
    singleQuotedKeys[0] !== '1' ||
    singleQuotedKeys[1] !== '2' ||
    singleQuotedKeys[2] !== 'b' ||
    singleQuotedKeys[3] !== 'a' ||
    frozenSingleQuotedKeys.length !== 4 ||
    frozenSingleQuotedKeys[0] !== '1' ||
    frozenSingleQuotedKeys[1] !== '2' ||
    frozenSingleQuotedKeys[2] !== 'b' ||
    frozenSingleQuotedKeys[3] !== 'a' ||
  singleQuotedMixedBracketedKeys.length !== 4 ||
  singleQuotedMixedBracketedKeys[0] !== '1' ||
  singleQuotedMixedBracketedKeys[1] !== '2' ||
  singleQuotedMixedBracketedKeys[2] !== 'b' ||
  singleQuotedMixedBracketedKeys[3] !== 'a' ||
  parenthesizedFrozenSingleQuotedKeys.length !== 4 ||
  parenthesizedFrozenSingleQuotedKeys[0] !== '1' ||
  parenthesizedFrozenSingleQuotedKeys[1] !== '2' ||
  parenthesizedFrozenSingleQuotedKeys[2] !== 'b' ||
  parenthesizedFrozenSingleQuotedKeys[3] !== 'a' ||
  parenthesizedFrozenSingleQuotedFrozenKeys.length !== 4 ||
  parenthesizedFrozenSingleQuotedFrozenKeys[0] !== '1' ||
  parenthesizedFrozenSingleQuotedFrozenKeys[1] !== '2' ||
  parenthesizedFrozenSingleQuotedFrozenKeys[2] !== 'b' ||
  parenthesizedFrozenSingleQuotedFrozenKeys[3] !== 'a' ||
  parenthesizedFrozenMixedBracketedKeys.length !== 4 ||
  parenthesizedFrozenMixedBracketedKeys[0] !== '1' ||
  parenthesizedFrozenMixedBracketedKeys[1] !== '2' ||
  parenthesizedFrozenMixedBracketedKeys[2] !== 'b' ||
  parenthesizedFrozenMixedBracketedKeys[3] !== 'a' ||
  frozenKeys.length !== 4 ||
  frozenBareCallableKeys.length !== 4 ||
  parenthesizedFrozenBareCallableKeys.length !== 4 ||
  frozenCallableKeys.length !== 4 ||
  frozenMixedBracketedKeys.length !== 4 ||
  frozenBracketedKeys.length !== 4 ||
  parenthesizedFrozenBracketedKeys.length !== 4 ||
  frozenKeys[0] !== '1' ||
  frozenKeys[1] !== '2' ||
  frozenKeys[2] !== 'b' ||
  frozenKeys[3] !== 'a' ||
  frozenBareCallableKeys[0] !== '1' ||
  frozenBareCallableKeys[1] !== '2' ||
  frozenBareCallableKeys[2] !== 'b' ||
  frozenBareCallableKeys[3] !== 'a' ||
  parenthesizedFrozenBareCallableKeys[0] !== '1' ||
  parenthesizedFrozenBareCallableKeys[1] !== '2' ||
  parenthesizedFrozenBareCallableKeys[2] !== 'b' ||
  parenthesizedFrozenBareCallableKeys[3] !== 'a' ||
  frozenCallableKeys[0] !== '1' ||
  frozenCallableKeys[1] !== '2' ||
  frozenCallableKeys[2] !== 'b' ||
  frozenCallableKeys[3] !== 'a' ||
  syncCount !== 4 ||
  frozenSyncCount !== 4 ||
  sequenceCount !== 4 ||
  frozenSequenceCount !== 4 ||
  asyncCount !== 4 ||
  frozenAsyncCount !== 4 ||
  asyncSequenceCount !== 4 ||
  frozenAsyncSequenceCount !== 4 ||
  breakContinueCount !== 1
) {{
  throw new Error('unexpected Reflect.ownKeys ordering');
}}
console.log('reflect ownKeys ok');
"#
    )
}

fn reflect_own_keys_test_source() -> &'static str {
    r#"Kali.test('reflect ownKeys', () => {
  const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
  const frozenObj = Object.freeze(obj);
  const keys = globalThis.Reflect.ownKeys(obj);
  const frozenKeys = globalThis.Reflect.ownKeys(frozenObj);
  const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);
  const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);
  const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
  const fullyBracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
  const parenthesizedBracketRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj);
  const frozenBracketRootKeys = Object.freeze((globalThis["Reflect"]))["ownKeys"](obj);
  const singleQuotedKeys = globalThis['Reflect']['ownKeys'](obj);
const singleQuotedMixedBracketedKeys = globalThis.Reflect['ownKeys'](obj);
  const frozenSingleQuotedKeys = globalThis['Reflect']['ownKeys'](frozenObj);
  const parenthesizedFrozenMixedBracketedKeys = Object.freeze((globalThis.Reflect["ownKeys"]))(obj);
const frozenBareCallableKeys = Object.freeze(Reflect.ownKeys)(obj);
const parenthesizedFrozenBareCallableKeys = Object.freeze((Reflect.ownKeys))(obj);
const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)(obj);
const frozenMixedBracketedKeys = Object.freeze(globalThis.Reflect["ownKeys"])(obj);
const frozenBracketedKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])(obj);
const parenthesizedFrozenBracketedKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))(obj);
const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))(obj);
const frozenNullishCallableKeys = Object.freeze((null ?? Reflect.ownKeys))(obj);
const frozenLogicalAndCallableKeys = Object.freeze((true && Reflect.ownKeys))(obj);
const frozenLogicalOrCallableKeys = Object.freeze((false || Reflect.ownKeys))(obj);
  let syncCount = 0;
  for (const key of globalThis.Reflect.ownKeys(obj)) {
    syncCount += 1;
  }
  let frozenSyncCount = 0;
  for (const key of globalThis.Reflect.ownKeys(frozenObj)) {
    frozenSyncCount += 1;
  }
  let sequenceCount = 0;
  for (const key of (0, globalThis.Reflect.ownKeys(obj))) {
    sequenceCount += 1;
  }
  let frozenSequenceCount = 0;
  for (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {
    frozenSequenceCount += 1;
  }
  let asyncCount = 0;
  for await (const key of globalThis.Reflect.ownKeys(obj)) {
    asyncCount += 1;
  }
  let frozenAsyncCount = 0;
  for await (const key of globalThis.Reflect.ownKeys(frozenObj)) {
    frozenAsyncCount += 1;
  }
  let asyncSequenceCount = 0;
  for await (const key of (0, globalThis.Reflect.ownKeys(obj))) {
    asyncSequenceCount += 1;
  }
  let frozenAsyncSequenceCount = 0;
  for await (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {
    frozenAsyncSequenceCount += 1;
  }
  let breakContinueCount = 0;
  for (const key of globalThis.Reflect.ownKeys(obj)) {
    if (key === '1') {
      continue;
    }
    breakContinueCount += 1;
    break;
  }
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    frozenKeys.length !== 4 ||
    frozenKeys[0] !== '1' ||
    frozenKeys[1] !== '2' ||
    frozenKeys[2] !== 'b' ||
    frozenKeys[3] !== 'a' ||
    mixedRootKeys.length !== 4 ||
    mixedRootKeys[0] !== '1' ||
    mixedRootKeys[1] !== '2' ||
    mixedRootKeys[2] !== 'b' ||
    mixedRootKeys[3] !== 'a' ||
    mixedBracketedKeys.length !== 4 ||
    mixedBracketedKeys[0] !== '1' ||
    mixedBracketedKeys[1] !== '2' ||
    mixedBracketedKeys[2] !== 'b' ||
    mixedBracketedKeys[3] !== 'a' ||
    bracketedKeys.length !== 4 ||
    bracketedKeys[0] !== '1' ||
    bracketedKeys[1] !== '2' ||
    bracketedKeys[2] !== 'b' ||
    bracketedKeys[3] !== 'a' ||
  fullyBracketedKeys.length !== 4 ||
  fullyBracketedKeys[0] !== '1' ||
  fullyBracketedKeys[1] !== '2' ||
  fullyBracketedKeys[2] !== 'b' ||
  fullyBracketedKeys[3] !== 'a' ||
    singleQuotedKeys.length !== 4 ||
    singleQuotedKeys[0] !== '1' ||
    singleQuotedKeys[1] !== '2' ||
    singleQuotedKeys[2] !== 'b' ||
    singleQuotedKeys[3] !== 'a' ||
    frozenSingleQuotedKeys.length !== 4 ||
    frozenSingleQuotedKeys[0] !== '1' ||
    frozenSingleQuotedKeys[1] !== '2' ||
    frozenSingleQuotedKeys[2] !== 'b' ||
    frozenSingleQuotedKeys[3] !== 'a' ||
    parenthesizedFrozenMixedBracketedKeys.length !== 4 ||
    parenthesizedFrozenMixedBracketedKeys[0] !== '1' ||
    parenthesizedFrozenMixedBracketedKeys[1] !== '2' ||
    parenthesizedFrozenMixedBracketedKeys[2] !== 'b' ||
    parenthesizedFrozenMixedBracketedKeys[3] !== 'a' ||
    syncCount !== 4 ||
    frozenSyncCount !== 4 ||
    sequenceCount !== 4 ||
    frozenSequenceCount !== 4 ||
    asyncCount !== 4 ||
    frozenAsyncCount !== 4 ||
    asyncSequenceCount !== 4 ||
    frozenAsyncSequenceCount !== 4 ||
    breakContinueCount !== 1
  ) {
    throw new Error('unexpected Reflect.ownKeys ordering');
  }
});
"#
}

fn assert_browser_requested_reflect_own_keys(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        reflect_own_keys_test_source()
    } else {
        &reflect_own_keys_source()
    };
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    if command == "run" {
        assert!(stdout.contains("reflect ownKeys ok"), "stdout: {stdout}");
    } else {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

fn assert_json_browser_requested_reflect_own_keys(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        reflect_own_keys_test_source()
    } else {
        &reflect_own_keys_source()
    };
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout")
                .contains("reflect ownKeys ok"),
            "json: {json}"
        );
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["stdout"], "");
    }
    assert_eq!(json["stderr"], "");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

fn assert_inherited_browser_api_surface_reflect_own_keys(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        reflect_own_keys_test_source()
    } else {
        &reflect_own_keys_source()
    };
    fs::write(&source_path, source).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let mut command_line = Command::new(kali_bin());
    command_line
        .current_dir(dir.path())
        .env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        command_line.arg("--output").arg("json");
    }
    let output = command_line
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        if command == "run" {
            assert_eq!(json["payload"]["exitCode"], 0);
            assert!(
                json["stdout"]
                    .as_str()
                    .expect("stdout")
                    .contains("reflect ownKeys ok"),
                "json: {json}"
            );
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
            assert_eq!(json["stdout"], "");
        }
        assert_eq!(json["stderr"], "");
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if command == "run" {
        assert!(stdout.contains("reflect ownKeys ok"), "stdout: {stdout}");
    } else {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[path = "browser_reflect_own_keys/test.rs"]
mod test;
