# Stage D triage — entry baseline + parser-only blast-radius measurement

> Task 1 (D0) of the Stage D plan: no product code lands in this task. This
> doc records the frozen failure baseline every later Stage D gate diffs
> against, and the blast radius of applying ONLY the parser + kali_types
> hunks of the un-flatten WIP patch (the codegen half is stale — pre
> env_ptr-ABI — and is excluded).
> Plan: `docs/superpowers/plans/2026-07-16-stageD-unflatten.md`.
> Design: `docs/superpowers/specs/2026-07-16-stageD-unflatten-design.md`.

## 1. Branch / baseline commit

- Branch: `soundness-batch1-pra`.
- HEAD at triage time: `1a3a1ae80` ("docs(soundness): stageD implementation
  plan — 9 tasks, capability-first, parser flip last [stageD]").
- Working tree clean before and after this task; no product code changed.

## 2. Frozen entry baseline — 731, zero drift

Built `kali_cli` fresh (`cargo build -p kali_cli`), then ran two independent
full-workspace enumerations, each detached and polled to completion (cache
cleared before each):

```
rm -rf .kali-cache
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > stageD-pre-runN.txt
```

- `$SCRATCH/stageD-pre-run1.txt`: **731** failing test names.
- `$SCRATCH/stageD-pre-run2.txt`: **731** failing test names.
- `diff stageD-pre-run1.txt stageD-pre-run2.txt` → **empty** (zero drift).
- `sort -u stageD-pre-run1.txt stageD-pre-run2.txt > $SCRATCH/stageD-pre.txt`
  → **731** lines (union equals either run — confirms identical sets).

**Canonical entry baseline: `$SCRATCH/stageD-pre.txt`, 731 entries.** This is
the honest-red baseline this branch deliberately carries (never merge to
main); every later Stage D gate diffs against it via `comm -13` (newly-red)
and `comm -23` (drain).

## 3. Parser-only scratch measurement

Applied ONLY the parser + kali_types portions of
`docs/superpowers/followups/task5-block-arrows-WIP.patch`:

```
git apply --include='crates/kali_parser/*' --include='crates/kali_types/*' \
  docs/superpowers/followups/task5-block-arrows-WIP.patch
```

`git status` confirmed exactly the 2 expected modified files:
`crates/kali_parser/src/expression/primary.rs`,
`crates/kali_types/src/resolve/call.rs`. `cargo build -p kali_cli` succeeded
clean. Ran the same two-step enumeration (cache cleared) on the patched
binary:

- `$SCRATCH/stageD-parser-post.txt`: **700** failing test names (31 fewer
  than the 731 baseline gross count — the un-flatten parser/types change
  fixes more than it breaks, net; see newly-green note below).
- `comm -13 stageD-pre.txt stageD-parser-post.txt` →
  `$SCRATCH/stageD-parser-newly-red.txt`: **6** entries (candidate re-pin
  list for Tasks 7-8).
- For context (not part of the required interface, informational only):
  `comm -23 stageD-pre.txt stageD-parser-post.txt` → **37** entries newly
  GREEN under the patch (tests that fail on baseline and pass once the
  parser stops flattening block-arrows). Net: 731 − 37 + 6 = 700, consistent
  with the raw counts above.

Per the brief's note: the applied `kali_types` gate still treats
`setTimeout`/`setInterval` as `deferred_but_unwired` (rejects anonymous
callbacks to them) — Tasks 4-5 wire them, and Task 7 lands the gate WITHOUT
that carve-out, so this measurement slightly OVER-counts the final red set.
That direction is safe for planning purposes.

### Revert and clean-tree verification (Step 3)

```
git checkout -- crates/kali_parser/src/expression/primary.rs crates/kali_types/src/resolve/call.rs
git status                                                            # clean
git apply --check docs/superpowers/followups/task5-block-arrows-WIP.patch && echo STILL-APPLIES
```

- `git status` after revert: **clean** (nothing to commit).
- `git apply --check` on the full WIP patch: **STILL-APPLIES** (revert did
  not leave the tree in a state that desyncs the patch's other hunks).
- `kali_cli` rebuilt clean on the reverted (baseline) tree, confirming no
  residual state.

## 4. Classification of the newly-red set (Step 4)

All 6 entries were investigated by rebuilding the patched binary, re-running
each failing test with `--nocapture`, and reading the actual JSON
error/stderr payload — not inferred from the name alone.

| # | test name | family | bucket | evidence |
|---|---|---|---|---|
| 1 | `build::build_emits_browser_bundle_web_baseline_primitives` | `addEventListener` capturing callback in the browser web-baseline fixture | (b) re-pin candidate | `E5506`: *"a callback passed to 'addEventListener' is unavailable unless it is provably non-capturing: codegen emits no call to this scheduling surface... an argument with unresolvable provenance fails closed"* |
| 2 | `build::build_emits_browser_bundle_web_baseline_primitives_in_js_input` | same fixture, `.js` input variant | (b) | same `E5506` addEventListener rejection (JS-input variant of #1) |
| 3 | `build::json_build_emits_browser_bundle_web_baseline_primitives` | same fixture, `--output json` variant | (b) | same `E5506` addEventListener rejection, JSON envelope: `errors[0].code == "E5506"`, message identical to #1 |
| 4 | `build::json_build_emits_browser_bundle_web_baseline_primitives_in_js_input` | same fixture, JSON + JS-input variant | (b) | same `E5506` addEventListener rejection |
| 5 | `test::json_test_supports_object_type_and_constructor_semantics` | `instanceof`/constructor-semantics diagnostic-shape pin | (b) re-pin candidate | Before patch: top-level `errors[0].code` is `E4000`/`E5506`. After patch (un-flattened `Kali.test` callback body): `errors` is `[]`, `payload.failed == 1`, `payload.passed == 0`, trap attributes to `stderr` as `"runtime trap in callback '__kali_callback_34'"` — a strictly better, more specific shape (per-callback attribution vs bare top-level reject), still fails closed (exit != 0, `success: false`) |
| 6 | `test::json_test_supports_object_type_and_constructor_semantics_in_js_input` | same pin, `.js` input variant | (b) | identical shape change to #5 |

**Bucket sizes: (a) = 0, (b) = 6, (c) = 0.**

- **Bucket (a) — deferred-surface families (queueMicrotask/setTimeout/
  setInterval capturing callbacks) closed by Tasks 4-6: 0 entries.** No
  currently-green (baseline-passing) test exercises an anonymous/capturing
  callback to those three surfaces, so applying the parser+types-only patch
  does not newly-red anything in this family. (The `deferred_but_unwired`
  carve-out noted in §3 means any such fixtures that DO exist are either
  already red on baseline or pinned as fail-closed `E5506` expectations that
  hold under both trees — not new fallout from this measurement.)
- **Bucket (b) — re-pin candidates: 6 of 6 entries**, exactly the two
  families the brief predicted: 4 `addEventListener` browser-bundle
  variants (`build`/`json_build` × `.ts`/`.js` input) and the 2
  `json_test_supports_object_type_and_constructor_semantics` diagnostic-shape
  pins. Both families are documented in the full WIP patch itself: the
  `addEventListener` fixture's capturing callback is exactly the shape the
  un-flatten patch's `kali_types` gate is designed to reject fail-closed
  (main is currently only "green" because the flattened-arrow lane silently
  drops the callback — a miscompile, not a real pass); the
  `object_type_and_constructor_semantics` pins assert the OLD top-level
  error-envelope shape, and the WIP patch's own `runtime_smoke.rs` hunk
  (not applied in this measurement — it's outside `kali_parser`/
  `kali_types`) already contains a `test_mode` branch documenting and
  re-pinning the NEW per-callback-attributed shape verbatim. No new
  investigation was required to explain either family; both are expected,
  named re-pins for whichever later task lands the parser flip.
- **Bucket (c) — anything else: 0 entries.** Nothing unexplained; not
  BLOCKED.

## 5. Interfaces produced

- `$SCRATCH/stageD-pre.txt` — canonical 731-entry baseline (consumed by
  every later Stage D gate).
- `$SCRATCH/stageD-parser-newly-red.txt` — 6-entry candidate re-pin list
  (consumed by Tasks 7-8).
- This doc.

## 6. Concerns / follow-ups for later tasks

- Bucket (a) being empty means Tasks 4-6's "closes deferred-surface
  families" framing has no positive baseline evidence from THIS
  measurement — it is a forward-looking claim about capability work not yet
  landed, not something this parser-only slice could exercise (no
  currently-green fixture uses a capturing callback to
  `queueMicrotask`/`setTimeout`/`setInterval`, so there was nothing for the
  parser-only patch to newly-red in that family). Later tasks should
  re-measure once those surfaces are wired.
- The 37 newly-green tests (informational, §3) are not required by the
  brief's interface but are worth Task 7-8 awareness: the parser+types-only
  patch fixes real baseline failures as a side effect, which is expected
  (un-flattening block-arrows closes silent-miscompile pins that were
  previously red for the RIGHT reason).

---

# Task 7 (D3) — measured full-foundation blast radius (parser flip LANDED)

> Task 7 lands the block-arrow un-flatten on the FULL Stage D foundation
> (Tasks 2–6 live). Unlike the Task 1 measurement (parser+types hunks over a
> bare branch), this is the real gate: all codegen deferred-callback lanes,
> virtual-clock drain, env_safety registration edges, and browser glue are
> present. Gate command per the Task 7 brief (fresh `.kali-cache` +
> `cargo build -p kali_cli`, then `cargo test --workspace --no-fail-fast`).

## 7.1 Totals

- Baseline (`stageD-pre.txt`): **731** honest-red.
- Post-Task-7 (`stageD-post-task7.txt`): **698** red.
- **Newly-red: 4** (`stageD-task7-newly-red.txt`).
- **Drain (newly-green): 37.** Net −33.

Every one of the 4 newly-red was already predicted by Task 1's parser-only
measurement (`stageD-parser-newly-red.txt`) — `comm -23` of the Task 7
newly-red against the Task 1 list is EMPTY. The 2
`json_test_supports_object_type_and_constructor_semantics{,_in_js_input}`
entries Task 1 also predicted are now GREEN, pre-handled by the WIP patch's
`runtime_smoke.rs` `test_mode` re-pin hunk applied in Step 4 (verified: the
`json_test_supports_object_type_*` pair passes post-patch).

## 7.2 Classified newly-red (all bucket-b — Task 8 re-pin batch)

| Test (all in `runtime_smoke/build.rs`) | Bucket | Evidence |
|---|---|---|
| `build::build_emits_browser_bundle_web_baseline_primitives` | b | see below |
| `build::build_emits_browser_bundle_web_baseline_primitives_in_js_input` | b | see below |
| `build::json_build_emits_browser_bundle_web_baseline_primitives` | b | see below |
| `build::json_build_emits_browser_bundle_web_baseline_primitives_in_js_input` | b | see below |

**Root cause (shared by all 4):** `browser_bundle_web_baseline_source()`
(`runtime_smoke.rs:4462`) contains
`target.addEventListener('tick', () => { count += 1; controller.abort(); })`
— an anonymous BLOCK-ARROW callback on the `addEventListener` surface, the
ONE remaining undrained scheduling surface (`is_undrained_scheduling_surface`,
Task 5). Pre-D3 the arrow flattened to a `Value("unknown")` placeholder and
`scheduling_call_args_provably_safe` returned `true` for it (the resolve-to-
nothing tail this task flips to `false`), so the bundle BUILD succeeded by
SILENTLY DROPPING the callback (+ its captured `count`/`controller`). Post-D3
the un-flatten compiles the arrow as a real CAPTURING function, and the
flipped guard fails closed:

```
error[E5506]: a callback passed to 'addEventListener' is unavailable unless
it is provably non-capturing: codegen emits no call to this scheduling
surface, so the callback — and any captured environment — would be silently
dropped; an argument with unresolvable provenance fails closed
```

These 4 pins assert `output.status.success()` (the OLD masking shape: build
succeeds via dropped callback). The new behavior — a fail-closed build reject
of a callback that would otherwise vanish — is STRICTLY BETTER (a silent
miscompile becomes a clean diagnostic). They are legitimate re-pins, NOT
defects, and are deferred to **Task 8's user-approval re-pin batch** (this
task does NOT re-pin them beyond the one pre-approved `runtime_smoke.rs`
`assert_json_object_type_and_constructor_semantics` hunk landed in Step 4).

## 7.3 Bucket (a) — deferred-surface families that should now be GREEN: 0 still red

The 37 drained tests ARE the bucket-a family, now green as intended — all
`test_supports_*object_keys/values*` / `*for_of_break_continue*` /
`*set_constructor*` / `*frozen_object*` iteration fixtures (browser-harness
and direct variants). None remained red, so there is NO Task 4–6 defect.
(These went red pre-D3 for the RIGHT reason — the flattened-arrow lane — and
the un-flatten is their real fix.) Full list: `comm -23 stageD-pre.txt
stageD-post-task7.txt`.

## 7.4 Bucket (c) — unexplained: 0

Nothing unexplained; not BLOCKED.

## 7.5 Probe adaptation note (feature-rich deferred-ordering probe)

The Step 5 probe `a_feature_rich_block_arrow_callback_defers_with_correct_
ordering` was adapted from the brief's verbatim snippet: the brief wrote the
callback's module write as `acc = value + b.n` (a module-scope `let` `=` an
expression CONTAINING a `.field` member read). That specific shape
pre-existingly mis-parses (`E8001 unsupported unary operator 'n'` + `E8001
binary operator '='`, then misclassifies the WRITE as a module-binding READ →
`E5506`) from inside ANY function body — CONFIRMED to reproduce identically
with a plain SYNCHRONOUS NAMED function, i.e. wholly orthogonal to arrows /
the un-flatten. The member access was moved into an unobserved local
(`let probe = b.n + value;`) and the module write became `acc = value` (no
member in RHS → the working lane); the body stays feature-rich (`+=`, `*=`,
`new`, `.field` read, module write) and the load-bearing DEFERRED-ORDERING
property is unchanged (node + kali agree byte-for-byte:
`MODULE-END-acc\n0\nINSIDE-CALLBACK\n15\n`). New pre-existing follow-up
inventoried: `<module-let> = <expr with member access>` mis-parse, and a
sibling `.field` read inside a function lowering to `0`.
