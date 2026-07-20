# throw-fallout Stage 3 — Host wiring (perf.now + web crypto + coverage_hit + process.kill) — design

**Date:** 2026-07-12
**Branch:** `soundness-batch1-pra` (PR #16, held draft until the last stage is green)
**Status:** Design approved — ready for `writing-plans`
**Parent program:** `docs/superpowers/specs/2026-07-11-throw-fallout-design.md` (umbrella), Stage 3
**Stage base:** post-Stage-2 branch tip `5815fef08`; denominator entering this stage = **923**
**Main gate worktree:** `/workspace/.worktrees/kali-main` (built at merge-base, 0 failures)

## Problem

Stage 3 of the throw-fallout program drains the **host-wiring** buckets of the
974→923 residual: `performance.now`, web crypto, the `coverage_hit` browser
LinkError, and the `process.kill(0)` probe. Two tagged soundness adjacencies in
the same host lane are carried here because Stage 7 (async) will build on this
surface and they should be closed first: the F-Stage1-2/3 `Deno.env.get`
equality holes, and the Stage-0 residual CDP crash-lane risk.

The umbrella's framing ("host imports not wired into the wasm lane") is only half
the story. Exploration established the sharper root cause:

- The host-side Rust implementations **already exist and are reachable from the
  wasmtime/native lane** (`kali_runtime/src/host/imports_default.rs`:
  `performance_now` at 86-92, `crypto_get_random_values` at 98+, `crypto_random_uuid`
  at 151+, subtle-digest byte writer, `coverage_hit` at 208-219), backed by
  `kali_api_web` (`util.rs:26` `performance_now`, `crypto.rs:9/14`
  `fill_random_values`/`random_uuid`).
- **kali_codegen has no recognizer for `performance.now` or any crypto call.**
  They fall through to the generic unknown-member placeholder (`I64Const(0)`), so
  the host funcs are never invoked from generated code. This is a codegen +
  type-mirror gap, not merely an import-list sync.
- The **four hand-mirrored browser `kali:rt` JS import lists** are separately out
  of sync with the wasmtime linker lane: they carry ~22 basic imports and are
  missing `coverage_hit`, `performance_now`, `crypto_*`, and `env_*`. This is the
  direct cause of bucket H (a `--coverage` browser run imports `coverage_hit`, the
  browser `importObject` doesn't provide it → LinkError), and it will block the
  browser variants of #5/#6 even after the codegen recognizers land.

## Ambition & invariants (inherited, non-negotiable)

From the umbrella program — repeated here so this stage cannot quietly downgrade:

1. **Fix, never flip.** Every target test gets a real implementation matching
   node's observable behavior. No construct is rejected or trapped to pass.
2. **Honest-red mid-stage is fine; the checkpoint must be net-green** vs the
   stage's start, zero main-green tests turned red.
3. **No re-masking.** A fix that re-silences a self-check `throw` is a defect even
   if the test goes green.
4. **Parity is defined by node**, same fixture, byte-for-byte.
5. **Both-sides hand-mirror discipline.** Every new recognizer needs an arm in
   **both** kali_codegen (the emit oracle) and kali_types (the resolve predicate),
   or it fails open (umbrella Risk 3, memory `kali-substring-runtime-spec2`).
6. **GC-less** stays true (memory `kali-gc-less-invariant`) — nothing here
   introduces tracing GC.

## Scope — target set

| Bucket | Tests | Root cause |
|---|---|---|
| #5 `performance.now` | 21 | No codegen recognizer → placeholder-0; absent from the 4 browser import lists |
| #6 web crypto (`getRandomValues` / `subtle.digest` / `randomUUID`) | 18 | No codegen recognizer → placeholder-0; absent from the import lists |
| H `coverage_hit` LinkError | 2 | Import emitted guest-side, missing from all 4 browser JS lists |
| K `process.kill(0)` probe | 4 | Recognizer misses optional-chain / `Object.freeze` / static-zero-alias receiver+arg shapes |
| F-Stage1-2/3 env equality | soundness (~0 net drain) | env-vs-env aliases one shared buffer; bound-alias `Deno.env.get` → raw handle compare |
| CDP crash-lane | harness integrity (0 drain) | Stage-0 residual: HTML/CDP driver could catch an in-page trap and still exit 0 |

**Core drain target: 45** (#5 21 + #6 18 + H 2 + K 4). The env and CDP items are
soundness/integrity work — near-zero count movement, carried here because this is
the host lane and Stage 7 builds on it.

**Explicitly OUT of Stage 3:**
- **Browser tail-replay defect** (post-loop tail replays 4×, `breakContinueCount 0`;
  holds the 16 `browser_reflect_own_keys run::` lanes red). Exploration confirmed
  there is **no replay machinery in the browser driver** — the harness instantiates
  once and runs linearly; the defect is **codegen loop lowering** (break/continue
  post-loop tail emission, `kali_codegen/src/emit/control_flow.rs`), surfaced by a
  browser fixture. The "browser-runtime" framing is a symptom locus, not the code
  locus. Gets its own follow-up lane; not this stage's host wiring.

## Architecture

### A. Per-intrinsic codegen recognizers (both-sides mirrored)

Model on the existing `coverage_hit` intrinsic and the `is_process_kill`
recognizer, both in `kali_codegen/src/intrinsics/host.rs`. For each of
`performance.now`, `crypto.getRandomValues`, `crypto.subtle.digest`,
`crypto.randomUUID`:

- **kali_codegen:** add a recognizer (sibling to `is_process_kill`) that matches
  the member-call shape and emits a `kali:rt` import + call to the host func.
  Host-side is already done — this makes it reachable.
- **kali_types:** mirror the recognizer so resolve admits the call instead of
  leaving it an unproven placeholder or rejecting it (Invariant 5).

### B. Import-index allocation

Imports use a positional index scheme anchored on `COVERAGE_HIT_IMPORT_INDEX`
with conditional offsets (`kali_codegen/src/lower.rs:95-125`; env imports stack
after coverage). Each new host import (`performance_now`, `crypto_*`) slots into
the same conditional-offset chain, gated on a `uses_*` flag so a program that
never calls it emits no import. **This index arithmetic is the highest
mechanical-bug risk in the stage** (an off-by-one desyncs the guest import
declaration from its consumers) and gets dedicated unit coverage.

### C. Data shapes (crypto fixtures)

The crypto fixtures build `new TextEncoder().encode(...)` → `Uint8Array` and read
`digest.byteLength`. `TextEncoder` is already a known builtin
(`kali_types/src/builtins.rs:114`). The recognizers operate on the byte buffer the
fixtures already construct:
- `getRandomValues(buf)` fills `buf` in place and returns it (host
  `fill_random_values` writes guest memory directly); fixture asserts the return
  is the same buffer and `length`/`byteLength` are 8.
- `subtle.digest(algo, bytes)` writes digest bytes into a guest buffer whose
  `byteLength` the fixture checks (32 for SHA-256, 64 for SHA-512).
- `randomUUID()` returns a non-empty string (host `random_uuid`).

### D. The synchronously-settled `await` lane

Both #5 and #6 fixtures wrap work in `async function main(){…}; main();` and use
`await`: perf.now uses `await Promise.resolve()`; crypto uses
`await crypto.subtle.digest(...)`. Stage 3 lowers `await <operand that settles
synchronously>` — a host-intrinsic call **or** `Promise.resolve(v)` — as direct
synchronous evaluation. No interleaving exists in these fixtures, so the
observable result is node-identical.

- This **widens** the umbrella's "await-of-host-intrinsic" note to also cover
  `await Promise.resolve()`, because the perf.now fixtures require it (they are
  not pure host-wiring).
- The **triage task pins the async-wrapper behavior empirically first**: the
  Stage-0 flatten lane already runs async bodies inline during `_start`, so the
  wrapper may already execute and only the placeholder-0 / `await Promise.resolve()`
  shape blocks. No lowering is written against an assumed failure mode.
- Stage 7 later replaces this with the real microtask queue. Zero-flips compliant
  (Invariant 1): the value and behavior are genuinely correct now, via a narrower
  mechanism, not a trap/reject.
- **Risk:** if the async wrapper needs more than the narrow await widening, part
  of #5/#6 attributes forward to Stage 7 rather than draining here. The triage
  task decides and the checkpoint attributes honestly (per the Stage-1/Stage-2
  falsified-forecast lesson).

### E. process.kill(0) receiver/arg widening

`is_process_kill` (`kali_codegen/src/intrinsics/host.rs:446-462`) accepts `kill(0)`
on a plain `process`-exit receiver → boolean `true`, no `process_exit` import. The
4 failing `node_api_surface::core` tests exercise shapes it misses:
optional-chain-wrapped (`process?.kill`), `Object.freeze(process).kill`, and
static-zero-alias args (`const z = 0; process.kill(z)`). The alias inventory
already exists (`kali_common/src/late.rs:93-165`
`LATE_PROCESS_CONTROL_PREFIX_SEGMENTS`; `kali_common/src/process_kill.rs`). Fix:
widen the recognizer's receiver-unwrap and arg-resolution to consult the same
transparent-unwrap / static-zero resolution, so all four shapes reach the accepted
zero-probe path. Non-zero / non-static / extra-arg cases keep their
`FEATURE_UNAVAILABLE` reject (only zero-probe shapes are admitted).

### F. env-equality soundness (F-Stage1-2/3)

- **F-Stage1-2 (env-vs-env, `kali_codegen/src/emit/operators.rs:1421-1428`):** both
  `Deno.env.get` results materialize into the single reserved buffer `[0, 4096)`
  (`ENV_GET_BUFFER_RESERVED`, `lib.rs:75`), so an `__streq` over two env.gets reads
  the second call's bytes twice. Today it is deliberately excluded
  (`!(left_env && right_env)`) — a fail-closed wrong answer. Fix: give the second
  env.get its own scratch region (copy-out to a distinct offset or a small two-slot
  rotation) so content comparison reads the right bytes.
- **F-Stage1-3 (bound-alias, `operators.rs:912-923`):** `is_env_get_string_call`
  unwraps only transparent wrappers, not bound aliases (`const g = Deno.env.get;
  g("K") === "y"`), so the compare silently falls to raw handle `i64.eq`. Fix:
  resolve the callee through `resolve_bound_member_callable_node` (the path the call
  emitter already uses; mirrors how `is_runtime_concat_string` at 935 resolves
  bound nodes).

### G. Import-list sync (hand-sync + guard)

Add the missing entries (`performance_now`, `crypto_*`, `coverage_hit`, `env_*`)
to all four browser `kali:rt` JS import lists **by hand**:
- List A — `kali_runtime/src/browser/harness.rs:220-327` (bundle-emit lane)
- List B — `kali_runtime/src/browser/harness.rs:592-703` (direct-wasm lane; note it
  uniquely also declares `thread_spawn`)
- List C — `kali_cli/src/bin/cmd_build.rs:1552-1649` (ESM bundle emit)
- List D — `kali_cli/src/bin/cmd_build.rs:1850-1948` (CJS bundle emit)

Then add a **mirror-sync guard test** that parses the four `format!` sites and
fails if their `kali:rt` member sets diverge. This is the durable fix for the
`kali-browser-harness-import-sync` footgun without refactoring the emit glue
(single-sourcing the lists was considered and rejected for this stage — it churns
bundle-format output other tests pin, beyond the drain goal).

### H. CDP crash-lane closure (Stage-0 residual)

The crash lane trips on "harness exit ≠ 0 with zero reported failures"
(`kali_runtime/src/execute.rs` `browser_tests_failed`), but Stage 0's reproducer
only covered the node `.mjs` lane. Add a reproducer for the Chromium/HTML CDP
entrypoint (`browser_harness_uses_html_entrypoint`,
`kali_runtime/src/browser/command.rs:285`) confirming an in-page guest trap
surfaces as non-zero exit / `success:false`. If the CDP driver currently swallows
it, wire trap→non-zero propagation; if it already propagates, the reproducer is a
regression pin. Harness integrity, no drain.

## Testing & gate mechanics

**The one hard gate (unchanged program-wide):** `cargo test --workspace
--no-fail-fast` on the branch → capture the FAILED set → diff against the persistent
`main` worktree at `/workspace/.worktrees/kali-main`. Stage done only when target
tests green **and** the global failing set strictly shrank **and** zero main-green
tests turned red. Plain `cargo test --workspace` fail-fasts at the first failing
binary, so enumeration always uses `--no-fail-fast`; exit-code verdicts use the
exact CI command. (Memory `ci-gate-vs-poisoned-baseline`.)

**Expected drain: 45**, with the honest caveat the program has learned twice
(Stage 1 and Stage 2 forecasts both falsified): the crypto/perf fixtures are
multi-construct (async wrapper, typed arrays, `await`), so if the async wrapper
needs more than the narrow await widening, some of #5/#6 attribute forward to
Stage 7. **Triage (Task 1) enumerates the exact target set and pins the
async-wrapper behavior empirically before any fix.**

**New tests this stage:**
- Codegen unit tests per recognizer (perf.now, each crypto call), **including
  explicit import-index arithmetic coverage** (the off-by-one risk from B).
- The **import-list mirror-sync guard** (G).
- CDP crash-lane reproducer (H).
- Node parity byte-for-byte on the same fixtures (perf.now monotonic; digest
  byteLength 32/64; uuid non-empty string).

**Pin maintenance — re-verify honestly, never silently flip:**
- The runtime_smoke "flipped-pin" enumeration fixture embedding
  `crypto.getRandomValues` currently asserts E5506/failure (Stage-2 triage). Once
  crypto is recognized, *which* diagnostic fires may change — re-pin to the honest
  outcome, do not let it flip green by accident.
- process.kill non-zero / non-static-arg cases keep their `FEATURE_UNAVAILABLE`
  reject.

**Checkpoint deliverable:** snapshot the failing-set delta into a Stage-3
triage/progress doc (`docs/superpowers/followups/throw-fallout-stage3-triage.md`)
so the 923→~878 drain is visible, consistent with prior stages.

## Risks

1. **Import-index arithmetic off-by-one** (B) — highest mechanical risk; dedicated
   unit coverage required.
2. **Async wrapper deeper than the narrow await lane** (D) — triage pins it first;
   checkpoint attributes forward honestly if so.
3. **Both-sides mirror miss** (Invariant 5) — every recognizer needs a kali_codegen
   arm and a kali_types arm or it fails open; reviewed per recognizer.
4. **Import-list divergence recurring** (G) — closed by construction via the guard
   test, not just this stage's hand-sync.
5. **Flipped-pin silent flip** — the crypto flipped-pin fixture must be re-pinned
   to its honest post-fix diagnostic, not allowed to go accidentally green.

## Definition of done (stage)

`cargo test --workspace --no-fail-fast` on `soundness-batch1-pra`: the #5/#6/H/K
target set green, global failing set strictly shrank (923 → ≈878), zero main-green
tests turned red; F-Stage1-2/3 env holes closed; CDP crash-lane reproducer green;
import-list guard in place; all fixes real (zero flips); no re-masking. Then the
next stage per umbrella sequencing (Stage 4 — array/for-of push lane).
