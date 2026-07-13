# throw-fallout Stage 3 triage — host-wiring drain (pinning the target set empirically)

Stage 3 of the throw-fallout program (plan: `docs/superpowers/plans/2026-07-12-throw-fallout-stage3-host-wiring.md`).
Branch `soundness-batch1-pra`, Stage-3 BASE `73c1ef3b3`; main worktree verified at `b48a067d3` (0 failures).

**Every claim below is backed by a command run on a freshly-built branch binary
(`cargo build -p kali_cli`, `./target/debug/kali`, code identical to `73c1ef3b3`).** Per the
program's twice-learned lesson (Stage-1 and Stage-2 forecasts both falsified), no failure mode is
assumed — each is reproduced. This triage falsified a THIRD forecast: see the process.kill finding.

## Pre-stage count + drift

- Main worktree (`b48a067d3`, gate baseline): `cargo test --workspace --no-fail-fast` →
  **0 FAILED** (`$SCRATCH/stage3-main.txt`, empty; consumed by the checkpoint diff). Gate is NOT
  poisoned.
- Branch (`73c1ef3b3`): `cargo test --workspace --no-fail-fast` → **exactly 923 FAILED names**
  (`$SCRATCH/stage3-pre.txt`, sorted). **Zero drift** vs the Stage-2 exit denominator.

## Target set — 45 names across four buckets (all counts name-verified against `stage3-pre.txt`)

| bucket | grep | count | expected |
|---|---|---|---|
| #5 performance.now | `performance_now` | 21 | 21 ✓ |
| #6 web crypto | `crypto\|random_uuid\|subtle\|get_random_values` | 18 | 18 ✓ |
| H coverage_hit | `reports_function_coverage.*browser_api_surface` | 2 | 2 ✓ |
| K process.kill | `process_kill_zero_probe\|optional_chain_wrapped_process_kill` | 4 | 4 ✓ |
| **total** | | **45** | **45 ✓** |

**No deviation.** 45/923.

### Bucket #5 — performance.now (21): `build::` 7, `run::` 6, `test::` 8
All `*_performance_now_monotonic_ordering_*` (js/ts × harness-configured/api-inherited × text/json).
Full list in `$SCRATCH/bucket5.txt`.

### Bucket #6 — web crypto (18): `build::` 4, `run::` 7, `test::` 7
`*_crypto_web_apis*` (build bundle), `*_crypto_get_random_values*`,
`*_crypto_subtle_digest_and_random_uuid*`. Full list in `$SCRATCH/bucket6.txt`.

### Bucket H — coverage_hit (2)
`test::test_reports_function_coverage_in_json_output_when_browser_api_surface_is_{configured,inherited}`
(`runtime_smoke/test.rs:6960`, `:6994`).

### Bucket K — process.kill (4), all `node_api_surface::core`
- `..._process_kill_zero_probe_in_js_ts_jsx_and_tsx...` (core.rs:1269)
- `..._optional_chain_wrapped_process_kill_zero_probe...` (core.rs:1381)
- `..._process_kill_zero_probe_through_static_zero_aliases...` (core.rs:1501)
- `..._process_kill_zero_probe_object_freeze_wrappers...` (core.rs:1615)

---

## Pinned failure modes (branch binary; node = v26.5.0)

### Bucket #5 — performance.now → **placeholder-0 with broken `typeof`; NOT E5506, NOT an async-wrapper trap**

Fixture (`run.rs:4820`, my `$SCRATCH/perf_main.js` reproduces it verbatim):
```js
async function main() {
  const first = performance.now();
  await Promise.resolve();
  const second = performance.now();
  if (typeof first !== 'number' || typeof second !== 'number' || second < first) {
    throw new Error('performance.now moved backwards');
  }
  console.log('performance.now ok');
}
main();
```

| probe | wasmtime lane | browser lane (`KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node`) |
|---|---|---|
| `perf_main.js` | `Uncaught Error: performance...`/`x` → **E4000 unreachable, exit 1** | same trap, exit 1 (`RuntimeError: unreachable`) |
| `console.log(performance.now())` | prints **`0`**, exit 0 | prints `0`, exit 0 |
| `console.log(typeof performance.now())` | prints **`0`** (NOT `"number"`), exit 0 | — |
| `async function m(){ await Promise.resolve(); console.log('ok'); } m();` | prints **`ok`**, exit 0 | prints `ok`, exit 0 |

**Root cause pinned:** `performance.now()` lowers to a **placeholder-0 constant** whose `typeof`
yields `0` (not the string `"number"`). The fixture guard `typeof first !== 'number'` is therefore
true → `throw` → E4000 trap. It is **not** an E5506 reject (compiles + runs) and **not** an
async-wrapper problem.

The host side already exists but is unwired: `kali_api_web::util::performance_now() -> f64`
(monotonic ms from an `Instant` origin, `util.rs:26`) has no call-site lowering.

**>>> ASYNC-WRAPPER DECISION (Step 4 deliverable):**
**perf.now currently → placeholder-0 (constant `0`, `typeof` yields `0` not `'number'`, so the
guard throws → E4000 trap); the async wrapper DOES already flatten+run; `await Promise.resolve()`
WORKS (no-op continue, prints `ok`, exit 0).**

**Consequence for the plan:** Task 4 (the synchronously-settled `await` lane) is **NOT a
prerequisite** for Tasks 5–7 and **shrinks to a regression-pin** — the async wrapper and
`await Promise.resolve()` already work end-to-end. The only remaining blocker for bucket #5 is
wiring a real numeric monotonic `performance.now()` recognizer to the existing host fn (must return
a value that (a) `typeof` reports as `number` and (b) is monotonic non-decreasing across calls).

### Bucket #6 — web crypto → **E4000 unreachable trap (codegen-recognizer gap); digest ALSO lacks a host import**

| fixture | lane | result |
|---|---|---|
| `crypto.subtle.digest('SHA-256', …)` + `crypto.randomUUID()` (`run.rs:8850`) | browser | `Uncaught exception` → **RuntimeError: unreachable, exit 1** |
| `crypto.getRandomValues(bytes)` (`run.rs:8622`) | browser | `Uncaught exception` → **unreachable, exit 1** |
| `crypto.getRandomValues(bytes)` | wasmtime | `Uncaught exception` → **E4000 unreachable, exit 1** |

Both are **`unreachable` traps, NOT `LinkError`s** — codegen never emits a call to a crypto host
import; the call site hits an unsupported-path guard.

**Host-import inventory (`imports_default.rs` = the `kali:rt` browser/default namespace):**
- `crypto_get_random_values` / `cryptoGetRandomValues` — **PRESENT** (line 98/125)
- `crypto_random_uuid` / `cryptoRandomUUID` — **PRESENT** (line 151/168)
- `subtle` / `digest` / `crypto_subtle_digest` — **ABSENT** (grep: no match)

`imports_node.rs:202` has `crypto_create_hash` (Node lane SHA-256), but there is **no `kali:rt`
`crypto_subtle_digest` host import.** **CONFIRMED: `crypto_subtle_digest` has no `kali:rt` host
import — Task 7 must add one.**

Nuance for Tasks 5/7: because `getRandomValues`/`randomUUID` imports already exist yet the fixtures
still `unreachable`, the getRandomValues/randomUUID blocker is a **codegen recognizer gap** (call
site not lowered to the existing import), whereas `subtle.digest` needs **both** a recognizer
**and** a new host import.

### Bucket H — coverage_hit → **browser LinkError, missing `kali:rt` import**

`test_reports_function_coverage_in_json_output_when_browser_api_surface_is_configured`
(`runtime_smoke/test.rs:6960`), `--nocapture`, verbatim stderr:
```
[LinkError: WebAssembly.instantiate(): Import #22 "kali:rt" "coverage_hit": function import requires a callable]
```
The JSON payload shows coverage instrumentation ran to the point of emitting a `coverage_hit`
import that the browser harness's `kali:rt` import object does not provide → instantiation
`LinkError`. **Confirmed: it is a missing-`coverage_hit`-import LinkError, not something else.**
Fix must add `coverage_hit` to the four hand-mirrored `kali:rt` JS import lists (memory
`kali-browser-harness-import-sync`: harness.rs ×2 + cmd_build.rs bundle glue ×2).

### Bucket K — process.kill → **guard trips → `Uncaught Error: expected zero probe` → E4000, exit 1 (expected 0)**

All 4 tests fail on the `test`-command Kali.test guard throwing "expected zero probe" (E4000
unreachable, exit code Some(1) vs expected Some(0)).

**FORECAST FALSIFIED (the reason this triage exists).** The plan framed bucket K around
sequence-receiver / static-zero-alias-arg / optional-chain-receiver shapes. Per-shape probes
(`--api node run`) show the dominant blocker is actually a **bare value-position function
reference**:

| shape probe | prints | verdict |
|---|---|---|
| `process.kill(0)` (baseline call) | `1` | WORKS |
| **`process.kill`** (bare value-position ref) | **`0`** (falsy placeholder; node → `[Function]`, truthy) | **BROKEN** |
| `!process.kill` | `1` (i.e. bare ref is falsy) | **BROKEN** |
| **`process?.kill(0)`** (optional-chain receiver) | **`0`** | **BROKEN** |
| **`(process.kill, process.kill)(0)`** (sequence receiver) | **`0`** | **BROKEN** |
| `process.kill(zeroAlias)` where `zeroAlias=zero=0` (static-zero-alias arg) | `1` | **WORKS** |
| `Object.freeze(process.kill)(0)`, `Object.freeze((process)).kill(0)` (freeze wrappers) | `1` | WORKS (matches the 25-case codegen unit test) |
| `((process.kill))(0)`, `process.kill((0))`, `((globalThis["process"]["kill"]))(+0)` | `1` | WORK |

**Missing-shape list (the Step-6 deliverable):**
- **sequence-expression receiver — FAILS** (returns 0). *Genuine soundness gap, but see below: it
  is NOT the green-blocker for any of the 4 tests.*
- **static-zero-alias arg — WORKS** (returns 1). Already handled; NOT a blocker.
- **optional-chain receiver — FAILS** (returns 0). Direct blocker for test #1381.
- **(NEW, not in the plan) bare value-position function reference — FAILS** (returns 0). Direct
  blocker for tests #1269, #1501, #1615.

**Per-test attribution (why each fails):**
- **#1269 / #1615**: shared guard `!process.kill || …` (aliases = direct+wrapped) short-circuits
  **true at its FIRST term** `!process.kill` (bare ref → falsy 0). The freeze/call forms further
  down the guard are never evaluated. → **bare-reference blocker.**
- **#1381** (optional_chain): guard `!process?.kill(0) || …` → first term `!process?.kill(0)` is
  true (optional-chain receiver → 0). → **optional-chain-receiver blocker.**
- **#1501** (static_zero_aliases): TEST guard is *also* `process_kill_zero_probe_guard_source()`
  (bare-ref first) → trips at `!process.kill`. Its RUN file (`node_api_surface_run_source`,
  process_kill.rs:348) *does* contain `console.log(sequenceKill(0))` (→ prints `0`), but the run
  assertion only checks `exit==0 && stdout.contains("1")` — which `console.log(process.kill(zeroAlias))`
  (→ `1`) satisfies — so the wrong `0` from the sequence receiver is **tolerated, not asserted**.
  → **bare-reference blocker** (sequence shape is a latent soundness bug, not this test's cause).

**Scoping consequence for Task 3:** greening bucket K requires (a) making a bare value-position
`process.kill` reference a **truthy** value, and (b) recognizing the **optional-chain receiver**
`process?.kill(0)`. The **sequence-expression receiver** returns a silently-wrong `0` and should be
fixed for soundness, but is not strictly required to turn any bucket-K test green (it is
defined-but-unused in the guards and merely console.log'd where the assertion tolerates it). Do not
let a sequence-only fix falsely "close" bucket K, and do not leave the sequence 0 unaddressed as a
silent miscompile.

---

## Findings / corrections recorded during triage

1. **perf.now is placeholder-0, not E5506** — resolves the two-exploration conflict decisively in
   favor of placeholder-0. `typeof` of the placeholder is `0`, which is the actual guard-tripping
   defect, not value monotonicity (`second < first` is `0 < 0` = false).
2. **The async wrapper + `await Promise.resolve()` already work end-to-end** (both lanes). Task 4
   shrinks to a regression-pin; it is not a prerequisite for #5/#6/#7.
3. **crypto getRandomValues/randomUUID host imports already exist** in `imports_default.rs`, yet
   the fixtures still `unreachable` → the blocker is a codegen recognizer gap, not (for those two)
   a missing import. Only `crypto.subtle.digest` additionally needs a new `kali:rt` host import.
4. **process.kill forecast falsified**: 3 of 4 bucket-K tests fail on a **bare value-position
   function reference** (`process.kill` → falsy 0), a shape not named in the plan; only 1 fails on
   optional-chain; the plan-named sequence-receiver gap, while real (silent 0), blocks none of the
   4 tests; the plan-named static-zero-alias-arg already works. Task 3 must be re-scoped
   accordingly (bare-ref truthiness + optional-chain receiver are the green-blockers).
5. All crypto/perf/process traps are honest **E4000 unreachable** (throw-fallout Stage-0 trap
   honesty intact) — no silent success masking a wrong answer at these sites; the wrong answers
   (`typeof===0`, bare-ref===0) surface only when the fixture guard reads them.

## Pinned "current behavior" table for Tasks 3–9 to assert the DELTA against

| bucket | current (branch `73c1ef3b3`) | target (fixtures assert) |
|---|---|---|
| #5 perf.now | `performance.now()` → const `0`; `typeof` → `0`; guard throws → E4000 exit 1 | numeric monotonic value; `typeof===number`; `performance.now ok`, exit 0 |
| #6 getRandomValues/randomUUID | call site `unreachable` (recognizer gap; imports exist) | fills buffer / returns string uuid; `ok`, exit 0 |
| #6 subtle.digest | call site `unreachable` (recognizer gap + NO host import) | 32-byte SHA-256 digest; `ok`, exit 0 |
| H coverage_hit | browser `LinkError` (Import #22 `kali:rt` `coverage_hit` not callable) | coverage JSON emitted; test passes |
| K process.kill bare ref / optional-chain | bare `process.kill` → `0`; `process?.kill(0)` → `0`; guard throws → E4000 exit 1 | bare ref truthy; optional-chain probe → `1`; exit 0 |
| K process.kill sequence receiver | `(process.kill,process.kill)(0)` → `0` (silent, unasserted) | should be `1` (soundness fix; not a green-blocker) |

## Follow-ups opened this stage

- (Task 3 review) Sequence-expression receiver `(process.kill, process.kill)(0)` silently returns
  `0` — a real miscompile masked by the test's tolerant run assertion. Fix for soundness even
  though it blocks no bucket-K test; guard against a sequence-only "fix" falsely closing bucket K.
- (Task 5/7 note) getRandomValues/randomUUID need only a codegen recognizer (imports present);
  subtle.digest needs recognizer + a new `kali:rt` `crypto_subtle_digest` host import.
- (Task 4 note) `await Promise.resolve()` already flattens+runs; keep the regression-pin lane
  minimal — do not build a full await machine where a pin suffices.
- (Task 4 controller refinement, resolved as REAL not pin) The triage note above under-tested
  value-CARRYING await. Only the *bare* `await Promise.resolve()` (unused value) worked; every
  value-consuming await was silently broken: `await Promise.resolve(7)` → `0`, `await (3+4)` → `0`,
  `await f()` → `0`. MECHANISM: HIR `AwaitExpr` was a text-less 1-child `Value`, so codegen's
  text-less aggregate path DROPPED the operand and pushed `I64Const(0)`. This makes await
  value-passthrough a hard PREREQUISITE for Task 7 (`const digest = await crypto.subtle.digest(...)`
  consumes the value; a `0` digest fails the `byteLength !== 32` guard). Task 4 shipped as a real
  implementation: `"await"` HIR marker + value-passthrough emit arm + `Promise.resolve(v)`
  recognizer + kali_types mirror. Fixed 15 pre-existing red runtime_smoke tests, 0 regressions.

## Task 7 (crypto.subtle.digest + TextEncoder) — scope expansion + mechanism notes

- **User-approved scope expansion beyond the plan's Task 7.** The plan assumed only
  `crypto.subtle.digest` + a new host import were missing. Pinning the fixture on a fresh binary
  (Tasks 4-6 in) showed `new TextEncoder().encode("browser crypto").byteLength` → **0** (node: 14):
  `TextEncoder().encode` was *completely unrecognized* in codegen. Task 7 therefore delivered BOTH
  (A) `TextEncoder().encode(str)` → a contiguous byte buffer whose `.byteLength` == the UTF-8 byte
  count, AND (B) the `crypto.subtle.digest` recognizer + new `kali:rt crypto_subtle_digest` host
  import.

- **Digest input is CONTIGUOUS string bytes (sidesteps m-T6-2).** A kali string is already a tagged
  contiguous byte-buffer handle (`STRING_HANDLE_TAG | (buf<<32) | len`, `len` UTF-8 bytes at `buf`).
  `TextEncoder().encode(<string>)` is a THIN REINTERPRET: the encoded buffer IS the argument's
  string handle, so the digest arm reads `(in_ptr,in_len)` off contiguous bytes. This deliberately
  does NOT route through `new Uint8Array(n)` (Task 6's m-T6-2: i64-stride elements, not contiguous),
  which the digest host could not read as raw bytes. Verified end-to-end: the host receives the real
  14 bytes `"browser crypto"`; SHA-256 hex matches node byte-for-byte.

- **Parser quirk: `new TextEncoder().encode(x)` parses as `new (TextEncoder().encode(x))`.** The
  `new` is hoisted to wrap the whole member-call chain (AST: `NewExpression{callee: CallExpr{callee:
  MemberExpr{object: CallExpr(TextEncoder), property:"encode"}}}`; LIR: a text-less 1-child `Value`
  wrapper — the `new` — around the `.encode` `Call`). The text-less-aggregate emit path DROPS its
  operand and pushes `0` (the "unsupported `new` → empty object" placeholder), so the encoded buffer
  was silently discarded and the binding stored `0` — the digest then hashed EMPTY input, yet
  `byteLength===32` still passed (SHA-256 is always 32 bytes) — a green-for-the-wrong-reason trap.
  Fixed with an encode passthrough in `emit_value` (scoped to a child whose callee is
  `is_text_encoder_encode`) + a `NewExpression`-arm in `repr_infer` + `is_text_encoder_ctor`
  accepting both `new TextEncoder()` and the bare `TextEncoder()` object form.

- **`typeof <runtime-string>` was `0`.** Latent pre-existing gap (also hit `typeof ("a"+s)`):
  `typeof` only classified STATIC operands, so a runtime string (`crypto.randomUUID()`, digest/encode
  buffers, concat) fell to the placeholder `0`, failing the fixture's `typeof uuid !== 'string'`
  guard. Fixed with an `is_string_valued` arm in the `typeof` emitter (interned "string" handle).
  KNOWN latent divergence: the String-repr digest/encode BYTE BUFFERS also match and would report
  "string" where node reports "object"; no fixture applies `typeof` to them. KNOWN residual: `typeof
  X` is not yet `is_string_valued`, so `"x" + typeof y` int-coerces the "string" handle (misprints a
  raw number); no target fixture uses `+ typeof` — follow-up.

## Follow-ups opened by Task 7 (not green-blockers for the subtle_digest targets)

- **`const x = crypto.getRandomValues(buf)` reads `x.length`/`x.byteLength` as `0`** (getRandomValues
  result flowing into a `const` is not registered as an array binding). PRE-EXISTING (confirmed via
  git-stash: red before Task 7, unchanged by it). This is the sole remaining blocker for the 4
  `build_emits_browser_bundle_crypto_web_apis*` bundle tests, whose `digestSmoke` uses SHA-512 (my
  digest returns 64 correctly) but trips the `filledBytes.length !== 8` guard. Task-6 domain
  (getRandomValues), out of Task 7 scope.
- `typeof X` should be `is_string_valued` (see above) so `+`-concat of a typeof result stays a
  string; needs the symmetric kali_types `expression_is_string_typed`/`operand_repr_is_string` arm to
  avoid an E3200 desync.

## Follow-ups opened by Task 9 (CDP/HTML crash-lane reproducer, review pass)

Task 9 (`crates/kali_cli/tests/browser_harness_cdp_in_page_trap_propagates.rs`) confirmed the
production HTML-harness *page content* surfaces an in-page guest trap as an uncaught
`Runtime.exceptionThrown` (kind "exception") rather than swallowing it — but it drove that page
through the test-only `cdp_driver`, not the production launcher. Review of that gap surfaced two
tracked production-CDP-driver items, neither of which is a swallow (both are non-functional-lane
gaps):

- **(a) The production HTML/`--api browser` entrypoint has NO CDP driver.**
  `BrowserHarnessInvocation::launch_with_env` (`crates/kali_runtime/src/browser/execute.rs:66-115`)
  is a bare `Command::new(chromium).arg(<file-url>).output()` spawn — no `--headless`/CDP/dump-dom
  wiring. A real run (`KALI_BROWSER_BUNDLE_HARNESS_COMMAND=chromium --headless --no-sandbox
  --disable-gpu`) launches Chromium, loads the page, and then **HANGS forever** (observed
  `EXIT=124` under a 90s timeout in the Task-9 investigation) — it never captures console, never
  reads a summary, never returns. This is a non-functional lane, not a swallow (a hang cannot be
  read as a silent pass by any consumer), but it means `kali test --api browser` with a
  Chromium-named harness command is currently unusable in production.
- **(b) Productionize a real CDP driver for the browser HTML lane.** Wire a real CDP driver (the
  test-only `crates/kali_cli/tests/cdp_driver/` module is the reference implementation — see also
  `browser_cdp_smoke.rs`) into `BrowserHarnessInvocation::launch_with_env` so the
  `kali test --api browser` CLI path with `KALI_BROWSER_BUNDLE_HARNESS_COMMAND=chromium` actually
  completes and reports success/exitCode (currently it either fails for the wrong reason —
  "No usable sandbox!" — with a bare `chromium <url>` command, or hangs with flags that let
  Chromium start). Until this lands, the CDP/HTML lane's Stage-0-swallow-freedom is proven only at
  the page-content level (Task 9's reproducer), not end-to-end through the CLI.

---

## Checkpoint (Task 10) — full gate + drain snapshot — **STAGE BLOCKED: 19 regressions**

**Measured on a fresh branch binary at HEAD `766f4ae32` (all 9 impl tasks landed).**
`cargo build -p kali_cli` then `cargo test --workspace --no-fail-fast` enumeration
(`stage3-post.txt`, sorted). Main worktree (`b48a067d3`) = 0 FAILED (`stage3-main.txt`, empty —
gate NOT poisoned; the throw-unmask project-wide fallout is expected and is NOT this stage's gate).

- **Pre (stage entry, `73c1ef3b3`): 923 FAILED** (`stage3-pre.txt`).
- **Post (`766f4ae32`): 853 FAILED** (`stage3-post.txt`).
- Arithmetic: `923 − 89 drained + 19 newly-red = 853` (exact).

### ⛔ PRIMARY GATE — FAILED (newly-red vs the 923 stage entry MUST be empty; it is NOT)

`comm -13 <(sort stage3-pre.txt) <(sort stage3-post.txt)` = **19 names** that PASSED at stage
entry and are RED at HEAD = **Stage-3 regressions**. Two independent root causes, both bisected on
freshly-built worktree binaries (parent green → commit red):

**Regression A — Task 4 await lane (`e8922229c`) — 16 tests.** The `AwaitExpr` HIR lowering gained
a `text="await"` marker (was a text-less transparent 1-child `Value`). The **compile-time
static-argument recognizers** that require a statically-known literal —
`Math.atan2` (needs static zero numerator + non-negative denominator) and `Number.isSafeInteger`
(needs a static primitive) — previously **tunneled through the text-less await wrapper** to reach
the literal; the marker made `await <literal>` **opaque**, so they now reject with **E5506**. The
fixtures deliberately probe await-wrapped static args: `Math.atan2(await 0, await 1)` and
`Number.isSafeInteger(await alias)` (`kali_common/src/number.rs:96`). Bisect: green at
`726b4f351` (parent), red at `e8922229c`.
  - `browser_math_atan2_global_this_root`: 6 (`{build,json_build,run,test}_*_global_this_math_atan2_await_wrapped_zero_slice_*`)
  - `kali_cli --lib build::tests::supports_math::...atan2...await_wrappers_{in_browser_api_surface_in_ts,in_js}_input`: 2
  - `browser_number_predicates_bundle`: 8 (`{build,json_build}_emits_browser_number_predicates_in_{js,jsx,ts,tsx}_input`)
  - **This is the exact Task-4 tension:** the await marker *fixes* runtime value-passthrough (+48
    bonus drain below) but *severs* the static-literal tunneling path. The fix is to make the
    static-argument recognizers (and the kali_types transparent-wrapper unwrap) treat a
    `text=="await"` single-child node as transparent **for compile-time static-literal resolution**,
    mirroring the runtime passthrough.

**Regression B — Task 7 `crypto.subtle.digest` (`94e6d22e2`) — 3 tests.** Task 7 added a
**compile-time E5506** rejecting `crypto.subtle.digest` calls that don't match "string algorithm
name + `TextEncoder().encode(<string>)` buffer". A `web-baseline` corpus package uses
`subtle.digest` in a non-conforming form; **before** Task 7 it compiled + deployed (the gap was a
*runtime* `unreachable` trap, not a compile error), so the corpus "checkable and deployable" test
passed. The new hard reject makes it red. Bisect: green at `23586b2da` (parent), red at
`94e6d22e2`.
  - `package_corpus::browser_corpus::browser_corpus_packages_with_web_baseline_primitives_remain_checkable_and_deployable_through_host{,_on_js_input,_on_js_input_when_the_browser_api_surface_is_inherited}` (3)
  - Resolution options (either closes the gate): widen the recognizer to accept the corpus
    package's `subtle.digest` shape, OR update the corpus fixture to the supported shape. Note the
    reject is arguably *more* sound (reject-don't-miscompile), but it broke a stage-entry-green test,
    so per the gate it is a regression that must be resolved, not silently accepted.

**Per the twice-learned lesson (memory `ci-gate-vs-poisoned-baseline`, Stage-2 checkpoint): the
per-task gates that reported "0 regressions" were necessary but NOT sufficient — this full
enumeration is the only real gate, and it caught both regressions the per-task subsets missed.**
**The stage is NOT closed.** Both regressions need a fix task (re-open Task 4 static-literal
tunneling + Task 7 corpus-shape) before Stage 3 can certify.

### DRAINED (in PRE not POST) — 89 names, fully reconciled

`comm -23 <(sort stage3-pre.txt) <(sort stage3-post.txt)` = 89. Disjoint partition (each name to
exactly one bucket), reconciled against the Task-10 brief's expectation:

| bucket | drained | expected | note |
|---|---|---|---|
| #5 performance.now | 21 | 21 | full drain ✓ |
| #6 web crypto | 14 | 18 − 4 | **carve-out holds:** the 4 `build::*_crypto_web_apis*` bundle tests **stay red** (pre-existing `const x = crypto.getRandomValues(buf)` array-binding gap, m-T6-3/Task-7 follow-up; confirmed red before this stage). 14 of 18 drain. ✓ |
| H coverage_hit | 2 | 2 | ✓ |
| K process.kill | 4 | 4 | ✓ |
| **Task-4 await bonus** | **48** | ~15 | **DRIFT +33** — see below |
| **total** | **89** | | |

**Task-4 await bonus = 48 (brief estimated ~15) — honest drift, +33.** Task 4's await
value-passthrough fixed every pre-existing failure whose fixture passed an **await-wrapped
argument**, far beyond the 15 `async`/`await`-named runtime_smoke tests:
  - `object_is` family: **24** (`Object.is(await alias, -0)`, `Object.is(await globalThis.Object, …)` — `browser_object_is_bundle.rs:18`, alias-chain harness, bundle build/json variants)
  - `math_round` family: **9** (`Math.round(await …)` identity literals + global-this root)
  - `async`/`await`-named runtime_smoke: **15** (the originally-estimated set)
  These were red at stage entry because pre-Task-4 `await` dropped its operand → `0` → wrong
  predicate result → guard threw. Value-passthrough makes them correct. **No unattributed drain;
  no expected-drain that failed to drain** (every named target that was supposed to drain did,
  modulo the documented 4 crypto_web_apis carve-out).

### Step 1 — flipped-pin crypto re-verification (honest, no silent flip)

The runtime_smoke enumeration "flipped-pin" fixtures embedding `crypto.getRandomValues` (Stage-2
triage §"Flipped pin"; e.g. `test.rs:4025`, `run.rs:2264`, …) assert `!success` + `E5506`. Verified
empirically on the HEAD binary:
  - **Full fixture still rejects fail-closed (E5506), exit 1** — now via TWO surviving constructs:
    the object-enumeration reassignment `obj["a"]=3` (E5506 fixed-shape) **and** the array-literal
    call arg `consumeArray([1n,2n],1n)` (E5506 "passing an array literal … reads zero placeholders").
  - **`crypto.getRandomValues(bytes)` ALONE now PASSES** (exit 0, `ok 1`) — Task 6 works.
  - **array-literal-arg ALONE still rejects (E5506).**
  Conclusion: **no silent flip.** The pins were already honestly re-anchored (during Task 6) on the
  genuine remaining rejection cause (array-literal-arg), which the comment already states; crypto is
  no longer the reject cause but the file still rejects on the array-arg gap, so the E5506 assertion
  remains genuine. No assertion change was warranted; the `object_enumeration_semantics_when_browser`
  pins are green in `stage3-post.txt` (0 present), confirming they hold.

### Step 4 — exact CI command (fail-fast) verdict

`cargo test --workspace` → **exit 101**, first-failing binary **`kali_cli --lib`** at
`build::tests::supports_math::...atan2...await_wrappers_...` (2 failed of 973). **This first-fail
moved EARLIER than prior checkpoints** (Stage-1/2 first-fail was
`array_callback_identity_browser_harness`) — a direct symptom of Regression A landing in the `--lib`
unit tests, further evidence the gate is genuinely red rather than mid-program bucket residue.

### Follow-ups surfaced this stage (beyond the two BLOCKING regressions)

- **Production-CDP-driver items (a)/(b)** — filed by Task 9 above (non-functional lane, not swallows).
- Deferred per-task Minors (not gate-blocking): `?.` short-circuit not modeled; paren-Promise-receiver
  asymmetry; `globalThis.performance.now` alias; multi-level env aliases; `typeof` String-repr
  divergence + `+ typeof` int-coercion; sequence-expression `process.kill` receiver silent-0
  (soundness, blocks no test); `const x = crypto.getRandomValues(buf)` array-binding (the 4
  crypto_web_apis carve-out).
