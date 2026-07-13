# throw-fallout Stage 4 triage — growable-array push lane (pinning the target set empirically)

Stage 4 of the throw-fallout program (plan:
`docs/superpowers/plans/2026-07-13-throw-fallout-stage4-array-push-lane.md`).
Branch `soundness-batch1-pra`, Stage-4 BASE `939770b51`; main worktree verified at `b48a067d3`
(0 failures).

**Every claim below is backed by a command run on a freshly-built branch binary
(`cargo build -p kali_cli`, `./target/debug/kali`, code identical to `939770b51`).** Per the
program's thrice-learned lesson (Stage-1/2/3 forecasts each falsified in triage), no failure mode
is assumed — each is reproduced. This time the forecast HELD: every Step-4 pin matches the plan's
push-no-op model exactly; one NEW latent divergence surfaced (boolean stringification, below).

## Pre-stage count + drift

- Main worktree (`b48a067d3`, gate baseline): `cargo test --workspace --no-fail-fast` →
  **0 FAILED** (`$SCRATCH/stage4-main.txt`, empty; consumed by the checkpoint diff). Gate is NOT
  poisoned.
- Branch (`939770b51`): `cargo test --workspace --no-fail-fast` → **exactly 834 FAILED names**
  (`$SCRATCH/stage4-pre.txt`, sorted). **Zero drift** vs the Stage-3 exit denominator (834).

## Target set — exactly the 16 `array_callback_identity_browser_harness` names

`grep -E 'array_callback_identity_slices_in_browser_api_surface_with_harness' stage4-pre.txt`
→ **16 names**, the full `{run,test,json_run,json_test} × {js,ts,jsx,tsx}` matrix from
`crates/kali_cli/tests/array_callback_identity_browser_harness.rs`:

```
json_run_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_{js,jsx,ts,tsx}_input
json_test_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_{js,jsx,ts,tsx}_input
run_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_{js,jsx,ts,tsx}_input
test_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_{js,jsx,ts,tsx}_input
```

`grep -E 'array_callback' stage4-pre.txt` → the SAME 16 names and **no others** — the rest of the
`array_callback_*` family (`array_callback_identity_map.rs` / `_filter.rs` / `_flat_map.rs`) is
green on the branch and **must stay green** (Stage 4 regression floor).

**No deviation.** 16/834. Target exit denominator: 834 − 16 = **818**.

---

## Pinned failure mode (branch binary `939770b51`; node = v26.5.0)

### `Array.prototype.push` is a **silent no-op** (fail-open wrong answer, NOT a reject)

All four Step-4 probes, `./target/debug/kali run <file>` (wasmtime lane):

| # | probe (`$SCRATCH/…`) | stdout | stderr | exit | node would print |
|---|---|---|---|---|---|
| 1 | `s4-push-length.js` — `const o=[];o.push(1);o.push(2);console.log(o.length);` | `0` | — | 0 | `2` |
| 2 | `s4-push-index.js` — `const o=[];o.push(1);console.log(o[0]);` | `undefined` | — | 0 | `1` |
| 3 | `s4-push-join.js` — `const o=[];o.push(1);o.push(2);console.log(o.join(","));` | *(empty line)* | — | 0 | `1,2` |
| 4 | `s4-reduced-harness.js` — push in a for-of-over-map body + join guard (below) | *(none)* | `Uncaught exception` + E4000 | 1 | `ok`, exit 0 |

Reduced harness (probe 4, the minimal shape of the failing fixture):

```js
function m(){const o=[];for(const x of [1,2].map(v=>v)){o.push(x);}if(o.join(",")!=="1,2")throw new Error("got:"+o.join(","));console.log("ok");}m();
```

Verbatim probe-4 stderr:

```
Uncaught exception
error[E4000]: runtime trap (unreachable — allocation failure or an unsupported-path guard): error while executing at wasm backtrace:
    0:  0x986 - <unknown>!<wasm function 31>
    1:  0x40b - <unknown>!<wasm function 22>
```

**Root cause pinned:** `o.push(x)` compiles and runs but stores nothing — `o.length` stays `0`,
`o[0]` is `undefined`, `o.join(",")` is `""`. The fixture guard
`observed.join(",") !== "1,2,1,2,1,2,1,2,1,2"` is therefore true → `throw` → honest **E4000
unreachable trap, exit 1** (test asserts exit 0 / `success:true`). It is **not** an E5506/compile
reject and **not** a for-of/source problem. This matches the `kali-repo-verification-env` follow-up
inventory ("throw is a no-op!"-era gap family: push is the no-op here; the throw itself now traps
honestly per Stage 0). **This table is the DELTA Tasks 2–5 must close** (probes 1–3 → `2` /
`1` / `1,2`; probe 4 → `ok`, exit 0).

## Sub-constructs already green (Stage 4 must NOT regress these)

All six probes print exactly `1\n2` (or the some/every line noted below), exit 0, empty stderr,
on the same branch binary:

| probe | source | stdout | exit |
|---|---|---|---|
| `s4-forof-map.js` | `for(const x of [1,2].map(v=>v))console.log(x);` | `1\n2` | 0 |
| `s4-forof-filter.js` | `for(const x of [1,2].filter(v=>v))console.log(x);` | `1\n2` | 0 |
| `s4-forof-arrayfrom.js` | `for(const x of Array.from([1,2].filter(v=>v)))console.log(x);` | `1\n2` | 0 |
| `s4-forof-spread.js` | `for(const x of [...[1,2].filter(v=>v)])console.log(x);` | `1\n2` | 0 |
| `s4-forof-flatmap.js` | `for(const x of [1,2].flatMap(v=>[v]))console.log(x);` | `1\n2` | 0 |
| `s4-some-every.js` | `` console.log(`some:${[0,1].some(v=>v)}`);console.log(`every:${[1,0].every(v=>v)}`); `` | `some:1\nevery:0` | 0 |

**All six already green; Stage 4 must not touch the for-of *source* lane (map / filter /
Array.from / spread / flatMap) nor some/every — only the loop *body*'s `push` + read-back
(`length` / `o[i]` / `join`).**

**Latent divergence (NOT a Stage-4 green-blocker):** some/every booleans stringify as `1`/`0`
inside template literals; node prints `some:true` / `every:false`. The fixture tolerates this — it
asserts only `status.success()` / JSON `success:true` + `exitCode:0`, never stdout content. Record
as a follow-up (boolean-to-string rendering), and do NOT let Stage 4 count on stdout equality with
node for the some/every lines.

## Browser lane + host-import finding

Reduced harness through the browser lane
(`KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node ./target/debug/kali --output json run --api browser
$SCRATCH/s4-reduced-harness.js`; env var = `kali_runtime::BROWSER_HARNESS_COMMAND_ENV`,
`contract.rs:81`) → **exit 1**, JSON payload verbatim-relevant fields:

```
"exitCode":1, "success":false, "stdout":"",
"stderr":"Uncaught exception\n…RuntimeError: unreachable\n    at wasm-function[31]:0x986\n    at wasm-function[22]:0x40b…"
```

Same push-no-op → guard throws → unreachable in both lanes; **the browser lane adds no second
failure mode.**

**Host-import inventory:** all four hand-mirrored `kali:rt` JS import lists
(`crates/kali_runtime/src/browser/harness.rs` ×2 — lines 233 / 649; and
`crates/kali_cli/src/bin/cmd_build.rs` bundle glue ×2 — lines 1554 / 1892) were enumerated:
identical 28-name sets (+ `thread_spawn` in the harness pair only, pre-existing). `grep -nE
'__join|array_push|array_len|array_get|"array'` over all four → **NO match**. `__join` (and its
`__join_arena` twin) are **pure-wasm synthetics** emitted by `kali_codegen/src/emitter.rs`
(`emitter.rs:379`, `:386`), not host imports. **Growable-array ops need no new `kali:rt` host
import; the 4-list sync hazard (`kali-browser-harness-import-sync`) is N/A for Stage 4 —
RE-CONFIRM at the Task-7 gate if any new synthetic ends up needing a host helper.**

## Escape facts — `observed` is arena-eligible

Fixture source (`array_callback_identity_browser_harness.rs:11`, quoted verbatim):

```js
function browserArrayCallbackIdentitySlices() {
  const observed = [];
  for (const item of [1, 2].map((value) => value)) { observed.push(item); }
  // …4 more for-of loops, each body only `observed.push(item);` …
  console.log(`some:${[0, 1].some((value) => value)}`);
  console.log(`every:${[1, 0].every((value) => value)}`);
  if (observed.join(",") !== "1,2,1,2,1,2,1,2,1,2") {
    throw new Error('unexpected array callback identity semantics');
  }
  console.log(observed.join("\n"));
}
browserArrayCallbackIdentitySlices();
```

Reasoning note: `observed` is a `const` **function-local** of
`browserArrayCallbackIdentitySlices`. Its only uses are as the **receiver** of `.push(item)` (×5)
and `.join(…)` (×3). It is **never returned** (the function implicitly returns undefined),
**never stored into an object/array field**, **never assigned to an outer/module binding**, and
**never captured by a closure** (the arrow callbacks reference only their own `value` param). It
is never even passed as a call *argument* — receiver position only.

Empirical probe (ephemeral `kali_mir` unit test via `test_support::analyze` on the verbatim
fixture body, run then reverted — not part of this commit's diff):

```
PROBE: observed.escapes=false ownership=Stack
```

→ `binding_escapes("browserArrayCallbackIdentitySlices","observed")` is **false**
(`kali_mir/src/analysis/escape_flow.rs:314`); ownership class `Stack`. **`observed` is
arena-eligible — Task 2's per-function-arena allocation of the growable backing store is justified
by this fact.** (Caveat for Task 2's review: any NEW flow edge the push lowering introduces —
e.g. modeling `push` as an unknown callee taking `o` as an argument — must not flip this to
tainted; keep the receiver a recognized-synthetic, not an unknown-callee arg.)

---

## Findings / corrections recorded during triage

1. **The Stage-4 forecast HELD** (first stage where triage falsified nothing): push is a silent
   no-op with exactly the predicted read-backs (`length`→0, `o[0]`→undefined, `join`→""), and the
   fixture failure is the honest E4000 trap from the guard throw — Stage-0 trap honesty intact,
   fail-open only in the silent wrong values, which the guard converts to a hard failure.
2. **Boolean template-literal stringification diverges from node** (`some:1`/`every:0` vs
   `some:true`/`every:false`), silently tolerated by the fixture's exit-code-only assertion.
   Follow-up, not a green-blocker; do not assert stdout-equality with node on these lines in
   Stage 4.
3. **No new host import needed** — array push/read-back can be pure wasm like `__join`; the 4-list
   browser-import sync hazard does not bind Stage 4 (re-check at the Task-7 gate).
4. **The for-of source lane is entirely green** (map/filter/Array.from/spread/flatMap), so Stage 4
   work is confined to the loop body's push + read-back; any regression in the six recorded probes
   is a Stage-4 defect.

## Pinned "current behavior" table for Tasks 2–6 to assert the DELTA against

| probe | current (branch `939770b51`) | target (node semantics / fixture asserts) |
|---|---|---|
| `o.push(1);o.push(2); o.length` | `0`, exit 0 | `2`, exit 0 |
| `o.push(1); o[0]` | `undefined`, exit 0 | `1`, exit 0 |
| `o.push(1);o.push(2); o.join(",")` | `""` (empty line), exit 0 | `1,2`, exit 0 |
| reduced harness (push in for-of body + join guard) | guard throws → E4000 unreachable, exit 1 | `ok`, exit 0 |
| reduced harness, browser lane (`--output json run --api browser`) | `success:false`, `exitCode:1`, RuntimeError: unreachable | `success:true`, `exitCode:0` |
| 16 `array_callback_identity_…browser…harness` tests | FAILED | green; denominator 834 → 818 |
| six sub-construct probes (for-of sources + some/every) | green (`1\n2` / `some:1\nevery:0`) | MUST stay green (regression floor) |

## Follow-ups opened this stage

- Boolean stringification in template literals prints `1`/`0` instead of `true`/`false`
  (some/every probe); masked by exit-code-only fixture assertions. Fix outside Stage 4 scope.
- Harness `kali:rt` lists carry `thread_spawn` while the cmd_build bundle-glue lists do not
  (pre-existing asymmetry, presumably intentional — bundle lane has no thread support); noting so
  a future import-sync pass doesn't "fix" it blind.

## Scratch artifacts (consumed by later tasks)

- `$SCRATCH/stage4-main.txt` — main-worktree failing set (0 lines).
- `$SCRATCH/stage4-pre.txt` — branch failing set (834 lines, sorted).
- `$SCRATCH/s4-*.js` + paired `.out`/`.err` — the ten probes above with recorded outputs.
