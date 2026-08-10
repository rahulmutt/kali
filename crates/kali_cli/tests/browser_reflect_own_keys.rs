//! Task 18 batch 8A audit escalation: kept 100% hand-written, not migrated.
//!
//! GROUND: CLASS A -- A CLAIM FROM UNREACHABLE CODE (ledger ruling R1).
//! This target is a `#[path]` submodule carrier (U10): `grep -c '#[test]'` on
//! THIS file returns 0, and all 44 of its `#[test]` fns live in the sibling
//! directory `browser_reflect_own_keys/` (run.rs 16, test.rs 16, build.rs 8,
//! check.rs 4). Inventory, audit and delete the two together.
//!
//! Three of this file's shared assert helpers --
//! `assert_browser_requested_reflect_own_keys` (`:577`),
//! `assert_json_browser_requested_reflect_own_keys` (`:612`) and
//! `assert_inherited_browser_api_surface_reflect_own_keys` (`:667`) --
//! each carry an `if command == "run"` branch asserting
//! `stdout.contains("reflect ownKeys ok")` (`:606`), and again at
//! `stdout.contains("reflect ownKeys ok")` (`:740`); two of them assert it on
//! the JSON stdout leaf instead, at
//! `.contains("reflect ownKeys ok"),` (`:654`) and again at
//! `.contains("reflect ownKeys ok"),` (`:725`).
//! NO LIVE `#[test]` FN REACHES ANY OF THEM. PR #16 rev2's honest
//! re-pin moved every `run` caller into `run.rs`'s own local `_fails_closed`
//! variants -- that file's module comment says so in as many words -- leaving
//! these three helpers with 16 call sites that all pass `"test"`. Derived
//! rather than asserted:
//!
//!     $ cd crates/kali_cli/tests && grep -rn \
//!       'assert_browser_requested_reflect_own_keys(\|assert_json_browser_requested_reflect_own_keys(\|assert_inherited_browser_api_surface_reflect_own_keys(' \
//!       browser_reflect_own_keys/ | grep -v _fails_closed | grep -c '"test"'
//!     16          # of 16 call sites; none passes "run"
//!
//! `audit-case-migration.py` extracts `.contains` literals with a regex and has
//! no reachability analysis, so it reports `[contains literals] 'reflect
//! ownKeys ok'` absent from any migration of this target. That cannot be
//! discharged honestly: no live test asserts it, so rule 2 forbids inventing
//! it; the `run` cases fail closed and emit no such stdout, so the claim would
//! also be FALSE; and `[source]` is excluded from the audit's case-side search
//! by design (U8), so the fixture's own trailing `console.log('reflect ownKeys
//! ok')` cannot discharge it either. Ledger ruling R1 fixes the disposition:
//! "Unreachable-code claims -> the target stays HAND-WRITTEN per the plan's
//! existing spec 5.11 escape hatch, and its .toml is deleted. Explicitly
//! REJECTED: adding a per-file audit exception mechanism ..., and teaching the
//! script Rust reachability analysis." Five files in this family already sit on
//! that Class A keep list for the same shape.
//!
//! A FULL MIGRATION WAS BUILT AND IS KEPT, BECAUSE THE DISPOSITION IS ESCALATED,
//! NOT CLOSED. `tools/task-18-browser-pilot/gen_batch8a.py` renders both halves
//! of this target under `--reflect-preview`, which writes nothing. The split it
//! encodes is a U2 two-file split on MANIFEST PRESENCE: 28 explicit-`--api
//! browser` fns and 16 manifest-inheriting fns cannot share one `[source]`
//! table, because `expand()` clones that table into every trial and the
//! manifest's mere presence moves `payload.hostContract` from `kali-hosted` to
//! `browser-requested`, which is exactly what these cases assert. That
//! derivation is independent of the Class A question and survives either
//! ruling. Note that the split is on the manifest, NOT on the submodule
//! boundary: run.rs and test.rs each straddle it 8/8, so one case file per
//! submodule would reproduce the disarmament twice.
//!
//! THE OPEN QUESTION, for the controller. U4 says whole-file retention is
//! legitimate only when EVERY test reaches the un-expressible construct. Here
//! ZERO tests reach it -- the construct is in dead code, not in a test -- so U4
//! and R1 point different ways, and U4's trim-and-keep has a live candidate:
//! retain test.rs's 16 fns, whose helpers hold the dead branch, and migrate
//! run.rs/build.rs/check.rs's 28. R1 is applied here because rule 4 calls 5.11
//! "the mechanical default for this situation, not a judgment call", and rule 3
//! forbids shipping a red audit absolutely. Reported rather than decided:
//! .superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch8a-report.md. No case file exists for this target.
//!
//! CONSEQUENCE FOR THE GATES (ruling 9). THIS FILE HAS NO RED-LIST, and that is
//! the finding, not an omission. Ruling 9 addresses a U4 TRIM-and-keep
//! retention, where the on-disk `.rs` is shorter than the source its case file
//! was migrated from and every literal-comparison gate therefore runs against
//! the wrong left-hand side. This is a WHOLE-FILE retention: nothing was
//! trimmed, so there is no pre-trim/post-trim divergence, no `PRE-TRIM REF:`
//! line, and ruling 12's third (migrated-complement) column does not apply --
//! there is no migrated complement. There is also no right-hand side:
//! `verify_pair.sh reflect_own_keys` exits 2 with `missing
//! .../cases/browser/reflect_own_keys.toml` before running any gate, so the
//! five gates that take a `.rs`/`.toml` pair cannot run here at all. The
//! exception is `batch5_crosscheck.py`, which needs no case file -- it resolves
//! THIS header's own `:N` citations against this very file. Run it directly:
//! `batch5_crosscheck.py --citations-only reflect_own_keys`.
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

fn browser_bundle_reflect_own_keys_source() -> &'static str {
    r##"// kali-tree-shake: reflectOwnKeysSmoke
async function reflectOwnKeysSmoke(left, right) {
  const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
  const frozenObj = Object.freeze(obj);
  const keys = Reflect.ownKeys(obj);
  const frozenKeys = Reflect.ownKeys(frozenObj);
  const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);
  const mixedBracketedDirectKeys = globalThis["Reflect"]['ownKeys'](obj);
  const mixedSingleQuotedDirectKeys = globalThis['Reflect']["ownKeys"](obj);
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
  for (const key of Reflect.ownKeys(obj)) {
    syncCount += 1;
  }
  let frozenSyncCount = 0;
  for (const key of Reflect.ownKeys(frozenObj)) {
    frozenSyncCount += 1;
  }
  let sequenceCount = 0;
  for (const key of (0, Reflect.ownKeys(obj))) {
    sequenceCount += 1;
  }
  let frozenSequenceCount = 0;
  for (const key of (0, Reflect.ownKeys(frozenObj))) {
    frozenSequenceCount += 1;
  }
  let asyncCount = 0;
  for await (const key of Reflect.ownKeys(obj)) {
    asyncCount += 1;
  }
  let frozenAsyncCount = 0;
  for await (const key of Reflect.ownKeys(frozenObj)) {
    frozenAsyncCount += 1;
  }
  let asyncSequenceCount = 0;
  for await (const key of (0, Reflect.ownKeys(obj))) {
    asyncSequenceCount += 1;
  }
  let frozenAsyncSequenceCount = 0;
  for await (const key of (0, Reflect.ownKeys(frozenObj))) {
    frozenAsyncSequenceCount += 1;
  }
  let breakContinueCount = 0;
  for (const key of Reflect.ownKeys(obj)) {
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
    parenthesizedFrozenMixedBracketedKeys.length !== 4 ||
    parenthesizedFrozenMixedBracketedKeys[0] !== '1' ||
    parenthesizedFrozenMixedBracketedKeys[1] !== '2' ||
    parenthesizedFrozenMixedBracketedKeys[2] !== 'b' ||
    parenthesizedFrozenMixedBracketedKeys[3] !== 'a' ||
    frozenNullishCallableKeys.length !== 4 ||
    frozenNullishCallableKeys[0] !== '1' ||
    frozenNullishCallableKeys[1] !== '2' ||
    frozenNullishCallableKeys[2] !== 'b' ||
    frozenNullishCallableKeys[3] !== 'a' ||
    frozenLogicalAndCallableKeys.length !== 4 ||
    frozenLogicalAndCallableKeys[0] !== '1' ||
    frozenLogicalAndCallableKeys[1] !== '2' ||
    frozenLogicalAndCallableKeys[2] !== 'b' ||
    frozenLogicalAndCallableKeys[3] !== 'a' ||
    frozenLogicalOrCallableKeys.length !== 4 ||
    frozenLogicalOrCallableKeys[0] !== '1' ||
    frozenLogicalOrCallableKeys[1] !== '2' ||
    frozenLogicalOrCallableKeys[2] !== 'b' ||
    frozenLogicalOrCallableKeys[3] !== 'a' ||
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
  return left - left + right - right;
}
"##
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

fn assert_browser_bundle_reflect_own_keys(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_reflect_own_keys_source()).expect("write source");

    let mut command = Command::new(kali_bin());
    command
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser");
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let envelope: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        assert_eq!(envelope["exitCode"], 0);
        let payload = envelope["payload"].as_object().expect("payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
    }

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");
    assert_eq!(metadata["artifactKind"], "bundle");

    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime_contract::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
const result = await mod.reflectOwnKeysSmoke(1n, 2n);
if (result !== 0n) {
  throw new Error(`unexpected result ${result}`);
}
console.log(String(result));
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime_contract::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('0'), "stdout: {stdout}");
}

fn assert_browser_checked_reflect_own_keys(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, reflect_own_keys_source()).expect("write source");

    let mut command = Command::new(kali_bin());
    command
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser");
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "check");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["filesChecked"], 1);
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    }
}

#[path = "browser_reflect_own_keys/run.rs"]
mod run;

#[path = "browser_reflect_own_keys/build.rs"]
mod build;

#[path = "browser_reflect_own_keys/check.rs"]
mod check;

#[path = "browser_reflect_own_keys/test.rs"]
mod test;
