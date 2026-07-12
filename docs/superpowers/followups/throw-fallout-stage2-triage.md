# throw-fallout Stage 2 triage — static object enumeration (pinning the target set empirically)

Stage 2 of the throw-fallout program (plan: `docs/superpowers/plans/2026-07-12-throw-fallout-stage2-static-enumeration.md`).
Branch `soundness-batch1-pra`, Stage-2 BASE `f0d4bcf14`; main worktree verified at `b48a067d3` (0 failures).

## Pre-stage count + drift

`cargo test --workspace --no-fail-fast` on the branch enumerates **exactly 974 FAILED names**
(`$SCRATCH/stage2-pre.txt`, sorted, duplicates kept) — the post-Stage-1 count recorded in the
denominator doc. **Zero drift.**

Cross-check note: a diff against the Stage-1 session's scratch snapshot (`stage1-post.txt`, 976
lines) shows exactly 2 extra names there (`misc::optimization_benchmark_suite_tracks_compile_time_size_and_speed`,
`misc::release_hot_paths_stay_unboxed_without_tag_checks`) — that snapshot predates the Stage-1
census-sync fix `b98365992` which re-greened those two. Consistent, not drift.

Target-superset composition inside the 974 (all counts verified by name against `stage2-pre.txt`):

| family | red names | notes |
|---|---|---|
| `browser_reflect_own_keys` (#4 bucket) | 40 | 16 `run::` + 16 `test::` + 8 `build::`; the 4 `check::` variants are GREEN (check doesn't execute) |
| `reflect_own_keys_js_input` (#4 bucket) | 4 | file has 5 tests; `check_accepts_…` is green |
| `runtime_smoke` direct-iteration (#4 bucket) | 2 | `run::json_run_supports_reflect_own_keys_direct_iteration_…`, `test::json_test_supports_reflect_own_keys_direct_iteration_…` |
| frozen_object-pattern (#4-adjacent, Stage-1 triage) | 44 | composition: has_own 4, enumeration_spread js_input 8, enumeration_spread_semantics 12, entries_iteration 18, values_iteration 2, values_spread 2. **Zero** `math_*frozen*` and **zero** `frozen_set_map` names are red — see correction (6) below |

## Probe transcripts (kali = branch debug binary at f0d4bcf14-identical code; node = v26.x)

### Probe 1 — Lane D, quoted-key enumeration element/length
```js
const keys = Object.keys({ "b": 1 });
console.log(keys.length);
console.log(keys[0]);
```
node: `1`, `b`, exit 0. kali: `2`, `0`, **exit 0 — silent miscompile** (F-Stage1-4 addendum confirmed).

### Probe 2 — Lane D, UNQUOTED identifier keys, same reads
```js
const keys = Object.keys({ b: 1 });
console.log(keys.length);
console.log(keys[0]);
```
node: `1`, `b`, exit 0. kali: `2`, `0`, **exit 0 — identical garbage.**
**Sharpest datum for Task 4:** the folded-enumeration element/length breakage is GENERAL
(unquoted identifier keys too), not quoted-key-specific — the unquoted-element hypothesis stands;
one fix covers both probes.

### Probe 3 — Lane C, delete+reinsert (runtime_smoke.rs:954 core)
```js
const r = { "a": 1, "b": 2, "c": 3 };
delete r.b;
r.b = 4;
const ks = Object.keys(r);
const vs = Object.values(r);
if (ks.length !== 3 || ks[0] !== 'a' || ks[1] !== 'c' || ks[2] !== 'b') throw new Error('keys stale');
if (vs[0] !== 1 || vs[1] !== 3 || vs[2] !== 4) throw new Error('values stale');
console.log('ok');
```
node: `ok`, exit 0. kali: stdout empty; stderr `Uncaught Error: keys stale` +
`error[E4000]: runtime trap (unreachable …)`, **exit 1** — the stale fold fires the self-check
(trap honest since Stage 0).

### Probe 4 — Lane A, quoted-key for..in
```js
const o = { "b": 1, "2": 2, "a": 3, "1": 4 };
for (var k in o) { console.log(k); }
```
node: `1`, `2`, `b`, `a`, exit 0. kali: `error[E5506]: for..in is only supported over an object
with a compile-time-known fixed shape …`, **exit 1** (F-Stage1-4 as documented).

## Attribution table (fixture-source-derived, probe-informed; NOT name-pattern-derived)

Lanes: A = quoted-key repr shapes, B = shared ES ordering, C = delete timeline, D = folded-array
element/length reads.

| family (red count) | constructs | lanes | out-of-stage blockers | verdict |
|---|---|---|---|---|
| `browser_reflect_own_keys` (40) | quoted+int-like keys, freeze/bracket/quote alias fan-out, `??`/`&&`/`\|\|`/ternary callable selection, sequence `(0,…)`, for-of, **for-await**, break/continue, `.length`/`[i]` reads | A, B, D | `for await` — see co-drain finding (2) below; likely NOT Stage-7-blocked | expected to drain with A/B/D; if for-await residue blocks the browser lane, attribute, don't chase |
| `reflect_own_keys_js_input` (4) | same fan-out, **no for-await at all** (grep-verified) | A, B, D | none | **cleanest full-drain candidate** |
| runtime_smoke direct-iteration (2) | quoted keys, aliasing, freeze/bracket variants, for-of AND for-await over aliased keys binding; no delete/push | A, B, D | for-await co-drain caveat | expected to drain |
| frozen has_own / from_entries subset (4) | `Object.hasOwn`, `Object.fromEntries`, freeze, sequence | possibly none of A–D (keys come via fromEntries, not literal syntax) | none apparent | verify empirically at Task 7; may drain for a different reason or need none of this stage |
| frozen enumeration_spread (8 js_input + 12 semantics) | `[...Object.keys(x)]` spread into NEW array literal, fromEntries incl. duplicate-key overwrite | D (partially); A likely N/A (no quoted-literal keys) | spread-into-array-literal is a distinct path — flagged, not assumed | partial drain at best; flag for Task 7 family run |
| frozen entries_iteration (18) | `Object.entries(alias)` passed as FUNCTION ARGUMENT, indexed in callee (`entries[0][0]`) | A, D | **array-as-fn-argument element read** — same unnamed gap as the `consumeArray` preamble, outside Lanes A–D | likely partial; flag before assuming drain |
| frozen values_iteration/spread (4) | for-of + `[]`/`.push` collect pattern | A, D | **push → Stage 4** | multi-blocked, stays red |
| `for_of_object_keys_iteration` (54, in #2/#3 + #10 buckets, NOT this stage's superset) | quoted keys + `[]`+push collect | A, D | **push → Stage 4** | confirmed multi-blocked (Stage-1 finding re-confirmed); Lane A is ALSO required — pure-Stage-4 tag was incomplete |
| `browser_object_values_harness` (36 file tests) | same push pattern in named functions | A, D | **push → Stage 4 (newly identified — was not in the Stage-1 push-blocked list)** | multi-blocked, stays red |
| `browser_math_*frozen*` (9 files, 128 tests) | `Object.freeze(Math.abs)` etc. — frozen callable refs only | **none** | none | **NOT Stage-2 surface at all** — zero enumeration constructs (grep-verified); remove from any "frozen_object-pattern" grouping |
| runtime_smoke "Flipped pin" enumeration fixtures (currently GREEN, assert E5506/failure) | quoted keys + top-level `delete reinsertion.b; reinsertion.b = 4` + fromEntries + **`crypto.getRandomValues` (Stage 3)** + **`consumeArray([1n,2n],1n)` array-literal-as-call-argument preamble (outside A–D)** | A, B, C, D | Stage 3 crypto; array-arg gap | **pin maintenance, not drain**: fixing A–D may change WHICH diagnostic fires; these tests assert rejection and must be re-verified/re-pinned honestly at Tasks 5–7, never silently flipped |
| runtime_smoke `object_property_deletion_semantics` (currently GREEN, assert rejection) | **expression-position** `delete obj.a !== true`, `'a' in obj` after delete, one variant inside an async function | none — deliberately fail-closed | n/a | **distinct construct from Lane C's statement-position target**; must STAY fail-closed. Task 6's default-deny is the intended landing spot for this shape. Do not conflate; do not expect a flip |

**Honest expected drain (a RANGE, per the Stage-1 forecast-falsified lesson):**
- Floor ≈ **4** (`reflect_own_keys_js_input` — no out-of-lane constructs at all).
- Core expectation: **46** (#4 bucket: 40 + 4 + 2) if the for-await co-drain finding (2) holds on
  the browser lane.
- Ceiling ≈ **58–70** if the frozen has_own/spread/entries subsets turn out to need only A/B/D
  (the fn-argument-indexing and spread-into-literal flags are the deciders — empirical at Task 7).
- The frozen values (4) and all push-pattern families stay red (Stage 4).

## Findings / corrections recorded during triage

1. `browser_reflect_own_keys` is 44 tests on disk (40 red — check variants green);
   `reflect_own_keys_js_input` is 5 tests (4 red). The denominator doc's "40/4" bucket counts are
   the red counts, consistent.
2. **for-await is very likely NOT an independent Stage-7 blocker here**: kali_types gates
   `ForOfStatement` (await and plain alike) purely via `is_static_array_iteration_target`
   (crates/kali_types/src/resolve/mod.rs:577-608 — `is_await` only changes diagnostic wording);
   codegen dispatches `"for-of"` and `"for-await-of"` to the same
   `emit_for_of_array_iteration` (crates/kali_codegen/src/emit/control_flow.rs:958-960); and a
   passing kali_codegen unit test already lowers `for await` over frozen
   `{ "b": 1, "2": 2, "a": 3, "1": 4 }` (emit/call_tests/reflect_own_keys.rs). Matches design-doc
   risk #1. Attribution: expect co-drain; verify at Task 7, attribute honestly if the browser
   lane still blocks.
3. **New gap flagged (outside Lanes A–D): array-literal-as-call-argument element reads**
   (`consumeArray([1n, 2n], 1n)` preamble; also the entries-as-fn-argument indexing shape in
   `browser_object_entries_harness`). Currently E5506. Not addressed this stage; recorded as a
   follow-up candidate below.
4. `for_of_object_keys_iteration` (54) and `browser_object_values_harness` (36) are BOTH
   push-blocked (Stage 4) AND Lane-A-dependent — the Stage-4 tag alone was incomplete, and the
   values-harness file was missing from the push-blocked list entirely.
5. Expression-position `delete` (`delete obj.a !== true`, runtime_smoke.rs:3436/3643) is a
   deliberately fail-closed family distinct from Lane C's statement-position lane; it must stay
   rejected after Task 6's default-deny (its current pins assert rejection).
6. All 9 `browser_math_*frozen*` files (128 tests) have zero enumeration surface — they are
   frozen-Math-callable tests, unrelated to Stage 2 despite the "frozen" name. None are red.
7. `kali_common::reflect_own_keys_frozen_callable_source` and siblings (kali_common/src/object.rs)
   are pure syntactic fan-out generators (freeze/bracket/quote/short-circuit callable selection) —
   no for-await, no delete, no push, no host wiring; lane-neutral. The `??`/`&&`/`\|\|`/ternary
   forms resolve through the pre-existing `resolve_static_callable_name` both-branches-same-target
   oracle (kali_types/src/resolve/call.rs) — NOT Stage-6-blocked in this shape.

## Delete-usage sweep classification (`$SCRATCH/stage2-delete-sweep.txt`, 70 hits)

**(a) env-delete unary forms:**
- `crates/kali_cli/tests/node_api_surface/core.rs:618-627` — 10 spellings
  (`process.env.X` / `process["env"].X` / `globalThis…` mixes). **Run tests, currently GREEN**
  (no node_api_surface env-mutation name in stage2-pre.txt). Task 1 Step 5 / Task 6 Step 4
  must re-verify green.
- `crates/kali_cli/tests/runtime_smoke/misc.rs:180-182`, `late_compat_js_input.rs:537-539`,
  `late_compat_browser_js_input/misc.rs:727-731`, `crates/kali_common/src/late.rs` +
  `late_tests/process_control.rs`, `crates/kali_cli/src/build_tests/rejects.rs:1407,1450` —
  **source-string assertion tests / recognizer input lists** (assert on emitted bundle text or
  reject behavior, not unary-delete runtime semantics). Currently green.
- `crates/kali_cli/src/build/compile.rs:762-771` — the recognizer's own source-pattern list
  (production code, not a test).

**(b) object-delete forms (all in runtime_smoke fixtures):**
- Statement-position delete+reinsert core: `runtime_smoke.rs:955` (browser enumeration source),
  `:1056`, `:1424` (overwrite-ordering source, straight-line top-level), `:1508` (indented —
  inside a function wrapper, OUT of the straight-line top-level lane); consumed via
  `test.rs:3959/4070/4182/4363` and `run.rs:2190/2304/2418/2595`. Mixed red/flipped-pin — see
  attribution rows above.
- Expression-position: `runtime_smoke.rs:3436` (`delete obj.a !== true`) and `:3643` (same inside
  an async function) — deliberately fail-closed pins, see correction (5).

**(c) irrelevant:** none — every hit classified under (a) or (b).

## Follow-ups opened this stage

(filled by later tasks)

- Candidate (from triage finding 3): array-literal-as-call-argument element reads
  (`f([1n,2n])` then `items[0]` in callee; also enumeration-result-as-argument indexing) —
  outside Lanes A–D, currently E5506 fail-closed.
