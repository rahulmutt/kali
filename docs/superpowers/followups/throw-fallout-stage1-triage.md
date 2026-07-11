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

## Stage 1 discoveries (Task 5)

Task 5's four enumeration-key reproducers surfaced two pre-existing, non-equality gaps (the `__streq` lane itself proved sound for the for-of Object.keys() binding shape — `object_keys_loop_variable_equality`, the direct for-of-binding compare, passed unmodified):

- **for-of `.push` into a `[]` literal is a silent no-op (bucket #10, Stage 4) — and #10's cataloged blast radius is undercounted.** `.push` has NO intrinsic recognizer anywhere in kali_codegen (unlike `fill`/`join`/`toFixed`/`concat`); the call falls through to the generic unresolved-callee placeholder fallback (`crates/kali_codegen/src/emit/call.rs:2356`, `push_placeholder_fallback_diagnostic`) and lowers to a dropped zero. Independently, a `[]` literal never registers as a structural runtime array (`declarator_registers_runtime_array`, `crates/kali_types/src/resolve/expression.rs:602-646` — only `new Array(n)`/`Array(n)`/`.fill()` results register), so there is no backing memory to push into. Repro: `const keys = []; for (const key of ["b","a"]) { keys.push(key); } console.log(keys.length);` prints `0` (node: `2`), exit 0, no diagnostic. Consequence for the bucket accounting: #10's denominator listing (16, all `array_callback_identity_browser_harness`) misses #2/#3-listed families whose fixtures use the same `[]`+push shape with no `.map`/`.filter` — 0/54 tests in `for_of_object_keys_iteration.rs` fail with the same symptom (push-into-`[]` no-op), and the `browser_object_keys_harness` / `browser_object_entries_iteration` object-enumeration harness fixtures use the same `const keys = []; ... keys.push(key)` shape. The root cause was minimally reproduced once (the repro above), not per-fixture-verified across all 54 — attributed to the push no-op, not confirmed as its cause in each case. The Task 7 spot-check of those families may therefore NOT fully pass after Stage 1 even though they are #2/#3-listed — they drain in Stage 4, not Stage 1; Task 7 attributes exactly.
- **Quoted-string object-literal keys have no repr shape (F-Stage1-4 below).** Blocks the brief's `for_in_key_equality` shape as written; test reshaped to unquoted keys (equality semantics unaffected). Additionally, a `const`-bound for-in key still hits the pre-existing "for..in key binding has no reserved local" reject (only `var`/`let` keys are in the admitted surface; already noted in the Batch-1 Task 5 nullish report) — the reshaped test uses `let`.
- **Task 5 test outcome per the record-don't-pin-wrong contingency:** `object_keys_loop_variable_equality` kept verbatim (passes); `for_in_key_equality` reshaped to unquoted keys + `let` binding (passes); `object_keys_element_equality` reshaped to the supported `new Array(2)` + indexed-store lane, quoted keys kept (passes); `object_entries_key_equality` DELETED — its `pair[0]`-via-push shape is expected-to-remain until Stage 4 (push no-op, root cause above, unrelated to equality).
- **`for_in_key_equality` revert-sensitivity, empirically settled (post-Task-5 review probe).** Built the pre-`__streq` binary at `031fcda37` (last commit before the equality arm, Task 2's `76165a395`) in an isolated worktree and ran the test's exact fixture (`const o = { b: 1, a: 2 }; let matched = 0; for (let k in o) { if (k === "b") ...; if (k === "a") ...; } if (matched !== 2) throw; console.log("ok");`). Pre-arm: `ok`, exit 0, empty stderr — identical to the current-branch binary (`ok`, exit 0). **Verdict: vacuous, not revert-sensitive.** The for-in key comes from Spec 4a's interned handle table (`kali-forin-spec4a`), and the handle materialized for the key text `"b"` coincides with the handle the `"b"` literal resolves to, so `===` passes by pre-existing handle identity — `__streq` content equality is not exercised by this shape. The test remains a valid node-derived behavior pin; it is not load-bearing coverage for the Stage 1 lane. No Stage-1-load-bearing for-in-key equality shape currently exists in the suite (see the test's updated comment for the candidate: a key whose handle is NOT table-interned, e.g. a runtime-computed property name — not built here, just recorded).

## Follow-ups opened this stage

- **F-Stage1-1 — mixed-type `==` coercion (spec §Scope).** `"5" == 5` style coercing equality is out of scope; a proven string vs non-string operand keeps today's fail-closed E3200 reject (tainted) or accidental-correct strict compare (untainted, Task 6 pin). Recorded in the spec; candidate for a later numeric-coercion lane.
- **F-Stage1-2 — env-vs-env equality is unsound (pre-existing).** `Deno.env.get(a) == Deno.env.get(b)` compares two handles aliasing the SAME reserved buffer (call.rs env lane, buffer offset 0): on `main` the identity compare is wrong for equal-length differing values, and Stage 1 deliberately does NOT route env-vs-env through `__streq` (the second call overwrites the first's bytes pre-compare). Fix requires per-call buffers or copy-out — host-wiring family, candidate for Stage 3.
- **F-Stage1-3 — bound-alias `Deno.env.get` equality is a silent raw handle compare (always-false `===` for matching text; node: true) (pre-existing).** `const g = Deno.env.get; g("K") === "y"` — the call itself emits via `resolve_bound_member_callable_node` (call.rs), but the equality recognizers (`is_string_valued`, `is_env_get_string_call`, `is_runtime_concat_string`) all miss the alias shape (none of them resolve bound-alias identifiers the way the call emitter does), so the comparison falls through to the pre-existing unsound raw `i64.eq` handle compare — no E3200 or any other diagnostic, just a silently wrong boolean. Predates Stage 1 (direct calls had the same gap before Task 4; Task 4 only widened the direct-call shape and did not regress the alias shape). Fix direction: teach `is_env_get_string_call` to resolve the callee via the same `resolve_bound_member_callable_node` path the call emitter uses. Empirically confirmed on the branch binary: fixture `const g = Deno.env.get; if (g("KALI_F3_PROBE") === "y") { console.log("eq"); } else { console.log("ne"); }` run with `KALI_F3_PROBE=y` via `./target/debug/kali run` prints `ne` (exit 0), vs. `eq` for the equivalent aliased-getter node script — i.e. observed behavior is a silent wrong result, not a compile-time reject.
- **F-Stage1-4 — quoted-string object-literal keys have no repr shape (pre-existing, fail-closed).** `record_object_literal`'s Identifier-only let-else (`crates/kali_types/src/repr_infer.rs:478-486`) records a deferred conflict for any `PropertyName::String`/`Number` key (`kali_ast::PropertyName`, `crates/kali_ast/src/literal.rs:37-41`), so `{ "b": 1, "a": 2 }` never materializes a `Repr::Object(shape)`; `object_shape_of_expression` (`crates/kali_types/src/resolve/expression.rs:587-600`) then returns `None` and the for..in fixed-shape gate (`crates/kali_types/src/resolve/mod.rs:567-572`) rejects E5506. Repro: `const o = { "b": 1, "a": 2 }; for (var c in o) {} console.log("ok");` → E5506 "for..in is only supported over an object with a compile-time-known fixed shape"; the byte-identical program with unquoted keys (`{ b: 1, a: 2 }`) compiles and runs correctly. Fail-closed (no miscompile), but it silently narrows the for..in surface Spec 4a advertised — quoted and unquoted keys are the same object in JS. Fix needs `PropertyName::String` support end-to-end (repr_infer + codegen mirror, both-sides discipline).

## Stage checkpoint (filled by Task 7)

- Post-stage count / drained / remaining by bucket: see the "Stage 1 drain" section of the denominator doc.
