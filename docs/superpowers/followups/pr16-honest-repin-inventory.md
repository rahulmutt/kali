# PR #16 honest re-pin inventory (canonical)

This is the single canonical adjudication map for the 694 honest-red workspace tests
carried on `soundness-batch1-pra` (baseline: `pr16-honest-red-baseline.txt`, Task 2).
Each later "wave" task (Task 5 template) is instantiated one-per-row from the table below.
`stageD-triage.md` §8.6 points here rather than duplicating this content.

## Method & count-discrepancy notes

- **N = 694**, set-identical to the Task-2 canonical baseline (`git show 407c81002` head commit).
- **The prior stage's "712" baseline artifact was a duplicate-line miscount.** Deduplicated it is
  exactly 694 unique names, set-identical to the Task-2 baseline. There was never a real 18-test
  delta; do not chase a phantom regression. (Stage P3's "712" note is the honest re-measure of the
  same 694 with duplicated rows.)
- Classification key (stricter than raw exit code, because every fixture in this corpus self-checks
  its own result with an in-program `throw`, which turns any wrong value into an E4000 trap):
  - **A — fail-closed already:** kali fails closed *independent of the fixture's self-check* — a
    compile-time E-code rejection (e.g. `E5506`) or a guaranteed trap on the construct itself. The
    test observes exit≠0 for a reason that is not "the fixture caught a wrong value." Action: **pin-reject**.
  - **B — silent miscompile:** with the fixture's `throw` removed, kali **exits 0 with a wrong value**
    (garbage i64 handle, `0`, wrong ordering). The fixture's self-check is the *only* thing making the
    red test exit≠0. Pinning this as "fail-closed" would bless a wrong value on main. Action:
    **deny-lane-then-pin** (extend the fail-closed lane to cover the construct, *then* pin).
  - **C — harness artifact:** product behavior is correct; the test's predicate/envelope is wrong.
    Action: fix the test predicate. **No family classified C** — every red probed here is a real
    product gap, not a harness bug.
- Every class-B claim below is backed by a *strip-the-self-check* probe: the exact construct was run
  through `target/debug/kali run` and `node` differentially, observing exit 0 + wrong value.

### Root cause (shared by all class-B families)

kali lowers `for-of` / spread / iteration-protocol **only over a provably-literal array with literal
elements**. The fail-closed lane for this is `E5506` ("for-of array iteration lowering is unavailable
unless the iterable is a literal array with literal elements"). **That deny lane has holes.** For many
non-literal iterables — `Object.keys/values/entries(...)` results, `new Set`/`new Map`, `Array.from(...)`,
`Promise.all/race/any/allSettled(...)` results, string→array, `for await`, Deno host returns — the
lowering does **not** reach `E5506`; it emits a **zero/placeholder** and returns garbage at exit 0.
Some *syntactic* forms of the same capability *do* correctly hit `E5506` (e.g. `for-of` over a template
literal) or `E3100` (undefined-identifier zero-placeholder fallback). This A/B split *within* one
capability is exactly the soundness defect: the wave action is to close the `E5506` lane's holes so the
whole construct fails closed, then pin.

Primary candidate choke points (named per row):
- for-of / spread iterable classification: `crates/kali_types/src/static_analysis/array.rs`
- for-of / for-await lowering + `E5506` emission: `crates/kali_codegen/src/emit/control_flow.rs`
- host/builtin call returning placeholder `0`: `crates/kali_codegen/src/emit/call.rs`, `crates/kali_types/src/late_host.rs`
- undefined call/identifier zero-placeholder (`E3100`): `crates/kali_types/src/resolve/mod.rs`

## Adjudication table

| family | pattern (grep against baseline) | count | representative | evidence (observed) | class | action | flip-back |
|---|---|---|---|---|---|---|---|
| object-enum | `object_keys\|object_values\|object_entries\|object_from_entries\|object_enumeration\|wrapped_object_enumeration\|frozen_object` | 319 | `run_supports_object_keys_iteration_in_js_input` | `Object.keys({b,a})` → exit 0, elems `-9223354444668731391`,`-9223354440373764095` (garbage i64) vs node `b`,`a`; `for-of Object.values` → exit 0 `0/0` vs `1/2` | **B** | deny-lane-then-pin: for-of/spread over `Object.*` result — `static_analysis/array.rs` iterable class + `emit/control_flow.rs` E5506 | Stage: runtime materialization of enumeration-result arrays (string keys + dynamic elements) |
| promise | `promise_all\|promise_race\|promise_any\|promise_all_settled\|promise_all_sequencing\|_promise_all\|requested_promise` | 128 | `run_supports_promise_all_in_js_input_when_browser_harness_is_configured` | `await Promise.all([1,2])`→exit0 `0/0`; race→`0`; any→`0`; allSettled→`0/0`; all vs correct node | **B** | deny-lane-then-pin: Promise combinator lowering returns placeholder `0` — `late_host.rs`/`emit/call.rs` | Stage: real Promise combinator runtime (beyond admitted `await Promise.resolve(v)`) |
| string-iter | `string_primitive\|string_concatenation\|template_literal\|object_string_enumeration\|string_enumeration` | 94 | `run_supports_object_string_enumeration_iteration_in_js_ts_jsx_tsx_input` | `for-of 'ab'` push→array → exit0 `0/0/0` vs `a/b`; template `for-of` → clean `E5506` (A-subform); string+=concat works (near-miss) | **B** | deny-lane-then-pin: for-of over string/template into array — `emit/control_flow.rs` (close E5506 holes) | Stage: dynamic string-char materialization into heap arrays |
| mapset | `map_constructor\|set_constructor\|array_from\|_set_map_\|set_map_break` | 33 | `run_supports_map_constructor_iteration_in_js_input` | `for-of new Set([3,4])`→exit0 `0/0`; `Array.from(new Set)`→`0/0`; Map destructuring form → `E3100 undefined identifier 'k'` (A-subform) | **B** | deny-lane-then-pin: Set/Map/Array.from iterable classification — `static_analysis/array.rs` | Stage: Set/Map runtime + iterable protocol |
| object-hasown | `object_has_own\|has_own` | 28 | `run_accepts_frozen_object_has_own_in_js_ts_jsx_tsx_input` | plain `Object.hasOwn` WORKS (exit0 `true`); frozen variant uses `Object.fromEntries` whose string keys are garbage → `hasOwn(k)` false → self-check throws | **B** | deny-lane-then-pin: `Object.fromEntries` string-key materialization (same object-enum choke) | Stage: enumeration-result string-key materialization |
| for-await | `for_await` | 24 | `build_emits_for_await_string_primitive_object_enumeration_semantics_in_js_input` | `for await(c of 'ab')`→exit0 `-9223354444668731391,-9223354440373764095` (garbage) vs `a,b` | **B** | deny-lane-then-pin: for-await lowering — `emit/control_flow.rs` | Stage: async iteration runtime |
| microtask | `queue_microtask\|microtask` | 22 | `run::run_supports_queue_microtask_ordering_in_js_input` | `queueMicrotask(cb); log.push(1); assert len==1` → kali self-check "microtask did not run before the next turn" (E4000); probe: microtask runs sync / push mis-orders → wrong ordering exit-0-without-check | **B** | deny-lane-then-pin: microtask scheduling in codegen event-loop runtime | Stage: proper microtask queue ordering |
| reflect | `reflect_own_keys` | 16 | `run::run_supports_reflect_own_keys_in_js_input_when_browser_harness_is_configured` | `Reflect.ownKeys(Object.freeze({b,a}))`→exit0 `-9223354444668731391,-9223354440373764095` vs `b,a` | **B** | deny-lane-then-pin: Reflect.ownKeys string-key materialization (object-enum choke) | Stage: enumeration string-key materialization |
| corpus | `corpus\|web_baseline_packages\|_deno_surface\|jsr_` | 15 | `utility::utility_corpus_packages_with_web_baseline_primitives_remain_executable_on_the_default_standalone_surface` | ramda/react build → **`error[E5506]: only abort() is supported on an AbortController handle … fail closed (Stage P3 scope)`** (exit≠0, fail-closed); deno_host `fresh-env` → E4000 runtime trap | **A** | pin-reject to the observed terminating diagnostic (E5506 / E4000 trap) | Per-underlying-feature: AbortController event surface (P3+), TextEncoder (P5), etc. |
| deno | `deno_env\|deno_chdir\|bracketed_deno` | 4 | `test::test_supports_bracketed_deno_env_get_in_js_input` | `Deno.env.get('PATH')`→exit0 `typeof v` prints `0` (returns `0`, not a string) vs node string; chdir cwd-alias mismatch | **B** | deny-lane-then-pin: Deno host builtin lowering returns placeholder `0` — `late_host.rs` | Stage: Deno host env/cwd runtime |
| bool-bundle | `boolean_logic` | 4 | `build::build_emits_browser_bundle_boolean_logic_semantics_in_js_input` | simple `a&&b`/`a\|\|b` WORKS standalone; bundled `logicSmoke` (async/bundle context) → self-check "unexpected boolean logic" WASM `unreachable` | **B** | deny-lane-then-pin: boolean lowering in async/bundle codegen path | Stage: audit bundle-context boolean/async lowering |
| crypto-bundle | `crypto_web_apis` | 4 | `build::build_emits_browser_bundle_crypto_web_apis_in_js_input` | bundle uses `TextEncoder`/`Uint8Array`/`crypto.getRandomValues`/`crypto.subtle.digest`/`randomUUID`; `Uint8Array`→`E3100` zero-placeholder; bundle traps "Uncaught exception" | **B** | deny-lane-then-pin: `Uint8Array`/WebCrypto/TextEncoder host builtins (E3100 zero-placeholder → fail-closed) | Stage P5 (TextEncoder) + WebCrypto host lane |
| await-wrapped | `await_wrapped` | 3 | `run_supports_await_wrapped_static_helper_inputs_in_js_ts_jsx_tsx_input` | `Object.keys(await {a:1})` / `Reflect.ownKeys((0, Object.freeze(...)))` → same garbage string-key handles as object-enum/reflect | **B** | deny-lane-then-pin: await/sequence-wrapped enumeration (object-enum choke) | Stage: enumeration string-key materialization |

**Class split:** A = 1 family (15 tests) · B = 12 families (679 tests) · C = 0 families.

## Per-family notes

Each note quotes the actual transcript observed. `KALI` = `target/debug/kali run <fixture>`; `node` = Node 26.5.0.

### object-enum (319, class B)
- **Evidence transcript.** `cargo test --test for_of_object_keys_iteration run_supports_object_keys_iteration_in_js_input` panics with
  `stderr: Uncaught Error: unexpected Object.keys iteration semantics` + `error[E4000]: runtime trap`.
  Strip-the-self-check probe (print instead of throw):
  - `const keys=Object.keys({b:1,a:2}); console.log(keys.length, keys[0], keys[1])`
    → node `2 b a`; **KALI exit 0**: `len=2 k0=-9223354444668731391 k1=-9223354440373764095`.
  - `for (const v of Object.values({b:1,a:2})) s.push(v)` → node `1/2`; **KALI exit 0** `0/0`.
  - `Object.entries({b,a})[0][0]` → node `b`; **KALI exit 0** `-9223354444668731391`.
  Note: direct indexing of a *numeric* `Object.values` result happens to be correct, but the `for-of`
  form (which every fixture uses) yields `0`/garbage — so the family is uniformly B.
- **Aspiration:** node materializes `Object.keys/values/entries` results as real arrays of strings/values,
  iterable via for-of/spread with insertion order.
- **Flip-back:** a stage that materializes enumeration-result arrays (string keys + dynamic element values)
  in the runtime, at which point the for-of/spread deny lane can admit them.

### promise (128, class B)
- **Evidence transcript.** `await Promise.all([Promise.resolve(1),Promise.resolve(2)])` → node `1/2`;
  **KALI exit 0** `0/0`. `Promise.race([...1])`→`0`; `Promise.any([...7])`→`0`;
  `Promise.allSettled([...1])[0].status/value`→`0/0`. `run::run_supports_promise_all_sequencing_in_js_input`
  panics `stderr: Uncaught exception` + `E4000`. Bundle form
  (`build_emits_browser_promise_any`) builds then traps in-bundle: `Uncaught Error: unexpected Promise.any semantics`.
- **Aspiration:** Promise combinators return settled aggregate values/arrays.
- **Flip-back:** real Promise combinator runtime. NB: `await Promise.resolve(v)` is already an admitted
  lane (`kali_types/src/static_analysis/promise.rs`); the *combinators* are the gap.

### string-iter (94, class B)
- **Evidence transcript.** `for (const c of 'ab') s.push(c)` → node `a/b`; **KALI exit 0** `0/0/0`.
  Contrast (near-miss, do not over-generalize): `for (const c of 'xy') r=r+c` (concat into a *string*, not
  an array) → **KALI correct** `xy`. And `for (const c of \`ab${n}\`)` (template literal) → **KALI
  `error[E5506]: for-of array iteration lowering is unavailable …`** (an already-fail-closed A-subform).
  Members' terminating text therefore varies, but the class is set by the exit-0-wrong member (string→array).
- **Aspiration:** string is iterable char-by-char; chars materialize as string elements in arrays.
- **Flip-back:** dynamic string-char materialization into heap arrays + closing the E5506 template holes.

### mapset (33, class B)
- **Evidence transcript.** `for (const v of new Set([3,4])) s.push(v)` → node `3/4`; **KALI exit 0** `0/0`.
  `Array.from(new Set([5,6]))` → node `5/6`; **KALI exit 0** `0/0`. `map_iteration_runtime` fixture prints
  `map constructor iteration ok` ×8 then throws `unexpected nullish Map constructor iteration semantics`.
  Map destructuring `for (const [k,v] of m)` → **KALI `error[E3100]: undefined identifier 'k'`** (A-subform).
- **Aspiration:** Set/Map iteration + `Array.from` over any iterable.
- **Flip-back:** Set/Map runtime + iterable protocol.

### object-hasown (28, class B)
- **Evidence transcript.** Plain `Object.hasOwn({a:1},'a')` → **KALI works, exit 0** (`REACHED_END`). The
  RED variants are *frozen*/browser-harness: `Object.freeze(Object.fromEntries([["a",1],["b",2]]))` then
  `Object.hasOwn(wrapped,"a")` — the `fromEntries` object's string keys are garbage (`e0k=-9223354406014025727`),
  so `hasOwn` returns false and the fixture throws. `console.log(Object.hasOwn(...))` also prints `1` not
  `true` (boolean→i64 formatting), a secondary divergence.
- **Aspiration:** `Object.fromEntries` builds a real string-keyed object.
- **Flip-back:** enumeration-result string-key materialization (shared with object-enum).
- **Scope-exception candidate:** plain `Object.hasOwn` already works — only the `fromEntries`-fed and
  browser-harness variants fail. Maintainer may judge a narrow real fix cheaper than a deny lane here.

### for-await (24, class B)
- **Evidence transcript.** `for await (const c of 'ab') s.push(c)` → node `a,b`; **KALI exit 0**
  `-9223354444668731391,-9223354440373764095` (garbage handles).
- **Aspiration:** async iteration over strings/objects.
- **Flip-back:** async iteration runtime.

### microtask (22, class B)
- **Evidence transcript.** `run::run_supports_queue_microtask_ordering_in_js_input` panics
  `stderr: Uncaught Error: microtask did not run before the next turn` + `E4000`. Probe:
  `queueMicrotask(fn); log.push(1); if(log.length!==1) throw 'early'` → node no-throw (deferred);
  **KALI throws 'early'** — the microtask ran synchronously / mis-ordered. Without the self-check the
  program exits 0 with the wrong interleaving.
- **Aspiration:** `queueMicrotask` defers callbacks to the microtask checkpoint after the current job.
- **Flip-back:** proper microtask queue ordering in the runtime.

### reflect (16, class B)
- **Evidence transcript.** `Reflect.ownKeys(Object.freeze({b:1,a:2}))` → node `2/b/a`;
  **KALI exit 0** `2/-9223354444668731391/-9223354440373764095`.
- **Aspiration:** `Reflect.ownKeys` yields own string keys in order.
- **Flip-back:** enumeration string-key materialization (shared with object-enum).

### corpus (15, class A)
- **Evidence transcript.** `utility::…ramda should be buildable` panics with kali stderr:
  `error[E5506]: only abort() is supported on an AbortController handle; addEventListener/onabort/reason/throwIfAborted fail closed (Stage P3 scope)` (plus `E3100`/`E8001` warnings) — **exit≠0, fail-closed at build**, independent of any self-check. `browser_runtime::…react…` fails the same way.
  `misc::deno_host_corpus…fresh-env` → `Uncaught exception` + `E4000 runtime trap in callback`.
- **Why A, not B:** the corpus tests assert real packages "remain buildable/executable/testable"; kali
  *refuses* (E5506) or *traps* (E4000) rather than emitting a blessed wrong value. Pin to the observed
  terminating diagnostic.
- **Aspiration:** real npm/jsr/web-baseline packages build+run+test.
- **Flip-back:** per-underlying-feature (AbortController events P3+, TextEncoder P5, …); a corpus row
  re-greens only when *all* features its packages exercise land.
- **Scope-exception / audit caveat:** the corpus build also emits `E3100` **zero-placeholder** warnings
  (`describe`, `CustomEvent`, `dispatchEvent`, `Event`) *before* the terminating `E5506`. `E3100` is a
  latent silent-miscompile vector. A corpus row must be pinned to a **terminating error/trap** — if any
  corpus package ever reaches exit 0 on `E3100`-only warnings, that package is class B and must be split
  out with its own deny lane. Wave task must re-verify each corpus row's terminating diagnostic at pin time.

### deno (4, class B)
- **Evidence transcript.** `test::test_supports_bracketed_deno_env_get_in_js_input` → `stdout: FAILED 1`,
  `stderr: Uncaught Error: expected env get`. Probe: `typeof Deno.env.get('PATH')` → **KALI exit 0**
  prints `type=0` (returns `0`, not a string). `deno_chdir` → `Uncaught Error: expected cwd aliases to
  agree after chdir`.
- **Aspiration:** `Deno.env.get` returns a string; `Deno.chdir` updates cwd consistently.
- **Flip-back:** Deno host env/cwd runtime.

### bool-bundle (4, class B)
- **Evidence transcript.** `a&&b` / `a||b` standalone → **KALI correct** (`F/T`). Bundle test
  `build::build_emits_browser_bundle_boolean_logic_semantics_in_js_input` builds the bundle then runs it
  under node: `Uncaught Error: unexpected boolean logic` + `RuntimeError: unreachable` at
  `Module.logicSmoke`. The bundled (async / tree-shaken browser-context) boolean path miscompiles.
- **Aspiration:** boolean logic identical in bundle/async context.
- **Flip-back:** audit bundle-context boolean/async lowering.
- **Scope-exception candidate:** simple boolean logic works; only the bundle/async form fails — a targeted
  codegen fix may be cheaper than a deny lane. Maintainer's call.

### crypto-bundle (4, class B)
- **Evidence transcript.** Bundle body uses `new TextEncoder().encode(...)`, `new
  globalThis["Uint8Array"](8)`, `crypto.getRandomValues`, `crypto.subtle.digest('SHA-512', …)`,
  `crypto.randomUUID()`. Probe: `new Uint8Array(4)` → **KALI `error[E3100]: undefined identifier
  'Uint8Array'`** (zero-placeholder path). Bundle test → builds then traps: `Uncaught exception` +
  `RuntimeError: unreachable` at `Module.digestSmoke`.
- **Aspiration:** WebCrypto + TextEncoder + typed arrays.
- **Flip-back:** Stage P5 (`TextEncoder`/`TextDecoder`) + a WebCrypto host lane; `Uint8Array` typed-array runtime.

### await-wrapped (3, class B)
- **Evidence transcript.** Fixture does `Object.keys(await {a:1})`, `Reflect.ownKeys(await
  Object.freeze({b,a}))`, `Object.keys((0, {a:1}))` — the `await`/sequence wrappers are transparent, and
  the underlying `Object.keys`/`Reflect.ownKeys` produce the same garbage string-key handles proven in
  object-enum/reflect. `await Promise.resolve(v)` admittance does not cover these enumeration helpers.
- **Aspiration:** await/sequence-wrapped enumeration helpers behave as their unwrapped forms.
- **Flip-back:** enumeration string-key materialization (shared with object-enum).

## Coverage ledger

Partition is a **priority-ordered, mutually-exclusive** extraction over the 694-line baseline (each
pattern is greppe­d against the residue after all higher-priority patterns are removed, so no line is
counted twice). Order: microtask → promise → reflect → for-await → corpus → deno → mapset → await-wrapped
→ string-iter → object-hasown → object-enum → bool-bundle → crypto-bundle.

| # | family | grep pattern (against residue) | count |
|---|---|---|---|
| 1 | microtask | `queue_microtask\|microtask` | 22 |
| 2 | promise | `promise_all\|promise_race\|promise_any\|promise_all_settled\|promise_all_sequencing\|_promise_all\|requested_promise` | 128 |
| 3 | reflect | `reflect_own_keys` | 16 |
| 4 | for-await | `for_await` | 24 |
| 5 | corpus | `corpus\|web_baseline_packages\|_deno_surface\|jsr_` | 15 |
| 6 | deno | `deno_env\|deno_chdir\|bracketed_deno` | 4 |
| 7 | mapset | `map_constructor\|set_constructor\|array_from\|_set_map_\|set_map_break` | 33 |
| 8 | await-wrapped | `await_wrapped` | 3 |
| 9 | string-iter | `string_primitive\|string_concatenation\|template_literal\|object_string_enumeration\|string_enumeration` | 94 |
| 10 | object-hasown | `object_has_own\|has_own` | 28 |
| 11 | object-enum | `object_keys\|object_values\|object_entries\|object_from_entries\|object_enumeration\|wrapped_object_enumeration\|frozen_object` | 319 |
| 12 | bool-bundle | `boolean_logic` (verified `grep -c` = 4) | 4 |
| 13 | crypto-bundle | `crypto_web_apis` (verified `grep -c` = 4) | 4 |
| | **TOTAL** | | **694** |

`22+128+16+24+15+4+33+3+94+28+319+4+4 = 694` ✅ (== baseline N). Residue after all 13 extractions: **0
lines** (verified). No singletons remain unaccounted.

### Scope-exception candidates (maintainer decides at Task 4 — recommending, not deciding)

- **object-hasown**: plain `Object.hasOwn` already works; only `fromEntries`-fed/browser-harness variants fail.
- **bool-bundle**: simple boolean logic works; only the async/bundle-context form miscompiles.
- **string-iter**: string`+=`concat works; only string→*array* materialization fails.
- For these, a narrow real product fix may be cheaper than a deny lane. Per Task-3 rules the action is
  still recorded as `deny-lane-then-pin`; the maintainer may downgrade to a real-fix wave.
- **corpus (audit caveat)**: pin each row to its *terminating* diagnostic (E5506/E4000); re-verify no
  package reaches exit-0 on `E3100`-only zero-placeholder warnings (that would be a hidden class-B split).
