# throw-fallout Stage 1 — triage (runtime string equality)

**Date:** 2026-07-11 · **Branch:** `soundness-batch1-pra` at 8c8edbcc2 (Stage 1 base) · **Plan:** `docs/superpowers/plans/2026-07-11-throw-fallout-stage1-string-equality.md`

## Pre-stage failing set

Enumerated with `cargo test --workspace --no-fail-fast` (the plain gate command fail-fasts at the first failing binary; see the Stage 0 denominator doc's gate-mechanics caveat). Snapshot: `stage1-pre.txt` in the stage scratchpad (machine-local, one `test name` per line, sorted, duplicates kept for same-named tests across binaries).

- **Count: 977** — exactly the Stage 0 denominator.
- **Drift vs the denominator listing: none.** `diff` of the pre-stage set against the denominator doc's full 977-name listing is empty (byte-identical). No flaky/env-dependent entries to note; `check_discovers_fixture_tree_from_cwd` (the known cwd/parallel flake from the Task-11 gate) is not in the set.
- Main worktree re-verified at `b48a067d3` (0 failures) before enumeration.

## Expected to drain: the #2/#3 bucket (656)

Per the denominator doc's bucket table, `#2/#3 enumeration + runtime string equality = 656`, the biggest bucket. Root cause drained by this stage: enumeration keys (Object.keys/entries/for-in) and other fresh runtime string handles (concat, substring, join, argv, env) compare by **handle identity** under `==`/`!==`, so self-check shapes like `keys[0] !== "b"` are true even when the text matches → the fixture's guard `throw` fires → honest failure since Stage 0. `__streq` content equality flips these self-checks back to their node-derived truth.

The full 656-name listing lives in the denominator doc (§"#2/#3 enumeration + runtime string equality"); it is not duplicated here.

## Expected to REMAIN red (overlap entries inside and outside #2/#3)

Buckets are name-pattern primary assignments and overlap (the denominator doc's own warning). Pattern counts below are across the whole 977 (a name can match several patterns):

| pattern | matches | stays red because | drains in |
|---|---|---|---|
| `for_await` | 40 | needs async-iteration machinery, not just key equality | Stage 7 |
| `promise` | 128 | Promise value lane unimplemented (#1 bucket) | Stage 7 |
| `async` | 19 | async/await sequencing lane (#1 bucket) | Stage 7 |
| `queue_microtask` | 22 | microtask queue host machinery (#1 bucket) | Stage 7 |
| `reflect_own_keys` | 46 | delete+reinsert / own-keys staleness (#4 bucket) | Stage 2 |
| `frozen_object` | 44 | frozen-object delete-reinsert shapes (#4-adjacent; some listed under #2/#3, their fixtures also exercise staleness) | Stage 2 |
| `performance` | 21 | `performance.now` host wiring (#5 bucket) | Stage 3 |
| `crypto` | 18 | web-crypto host wiring (#6 bucket) | Stage 3 |
| `coverage` | 2 | browser harness import list missing `coverage_hit` → LinkError (bucket H) | Stage 3 |

Also expected to remain: `process.kill(0)` probe subfamily (bucket K, 4), dynamic-import member typeof (#7, 32), short-circuit family (#8, 13), array/for-of push lane (#10, 16).

Any #2/#3-listed name matching one of these patterns (e.g. the `frozen_object_entries` harness tests, the `for_await_object_string_enumeration` family) may stay red after Stage 1 with these root causes; Task 7 attributes the exact remainder.

## Follow-ups opened this stage

- **F-Stage1-1 — mixed-type `==` coercion (spec §Scope).** `"5" == 5` style coercing equality is out of scope; a proven string vs non-string operand keeps today's fail-closed E3200 reject (tainted) or accidental-correct strict compare (untainted, Task 6 pin). Recorded in the spec; candidate for a later numeric-coercion lane.
- **F-Stage1-2 — env-vs-env equality is unsound (pre-existing).** `Deno.env.get(a) == Deno.env.get(b)` compares two handles aliasing the SAME reserved buffer (call.rs env lane, buffer offset 0): on `main` the identity compare is wrong for equal-length differing values, and Stage 1 deliberately does NOT route env-vs-env through `__streq` (the second call overwrites the first's bytes pre-compare). Fix requires per-call buffers or copy-out — host-wiring family, candidate for Stage 3.
- **F-Stage1-3 — bound-alias `Deno.env.get` equality is a silent raw handle compare (always-false `===` for matching text; node: true) (pre-existing).** `const g = Deno.env.get; g("K") === "y"` — the call itself emits via `resolve_bound_member_callable_node` (call.rs), but the equality recognizers (`is_string_valued`, `is_env_get_string_call`, `is_runtime_concat_string`) all miss the alias shape (none of them resolve bound-alias identifiers the way the call emitter does), so the comparison falls through to the pre-existing unsound raw `i64.eq` handle compare — no E3200 or any other diagnostic, just a silently wrong boolean. Predates Stage 1 (direct calls had the same gap before Task 4; Task 4 only widened the direct-call shape and did not regress the alias shape). Fix direction: teach `is_env_get_string_call` to resolve the callee via the same `resolve_bound_member_callable_node` path the call emitter uses. Empirically confirmed on the branch binary: fixture `const g = Deno.env.get; if (g("KALI_F3_PROBE") === "y") { console.log("eq"); } else { console.log("ne"); }` run with `KALI_F3_PROBE=y` via `./target/debug/kali run` prints `ne` (exit 0), vs. `eq` for the equivalent aliased-getter node script — i.e. observed behavior is a silent wrong result, not a compile-time reject.

## Stage checkpoint (filled by Task 7)

- Post-stage count / drained / remaining by bucket: see the "Stage 1 drain" section of the denominator doc.
