# Prerequisite project: `throw`-fallout (blocks Soundness Batch 1 PR-A / PR #16)

**Status:** OPEN prerequisite (maintainer decision 2026-07-10). PR #16 (`soundness-batch1-pra`) is held **draft** until this lands. Making `throw` sound (Task 1, print-then-trap) un-masks **922 workspace tests** that were fake-green on `main` (their self-check `throw`s were silent no-ops, so a program with a real bug still exited 0). `cargo test --workspace`: **main = 0 failures, branch = 922**. The other 8 closures (Tasks 2–7, 9, 10) are clean; this project is entirely about the throw-unmask fallout.

**Gate discipline going forward:** verify `cargo test --workspace` (the CI command) diffed against a **`main` worktree**, never a mid-branch red baseline. See memory `ci-gate-vs-poisoned-baseline`.

## The 922 by underlying bug-class (test-name buckets overlap; these are the ROOT causes from the Task 1b triage, re-confirmed)

Each class is fake-green today because a self-check `throw` never fired. "Fix" = make the feature correct vs node; "flip" = the construct is genuinely unsupported and should fail-closed (reject/trap) with the test re-pinned to assert that.

1. **await / Promise value lane → placeholder 0** (~200 tests: async_await_sequencing, promise_all/any/race/all_settled, queueMicrotask). `await Promise.resolve(7)` yields 0, not 7; microtasks run eagerly. Real async machinery — largest single fix, likely its own spec. Candidate for **flip** (fail-closed reject of `await`/Promise in the unsupported lane) if a real impl is out of scope, but that flips many tests and PR-B may implement it — coordinate.
2. **Object enumeration handle-blind string compares** (part of ~518 enumeration tests). `Object.keys(o)[0] !== '1'` is TRUE even though it prints `1` — runtime string equality compares fresh buffer handle vs interned literal handle. Same class as deno env `Deno.env.get('X') !== 'y'`. This is the runtime string-equality gap; fixing it greens a large fraction. **Fix** (real, shared with #3).
3. **Runtime string equality handle-blind generally** (deno env/chdir, web-baseline corpus). Same root as #2. **Fix** together.
4. **delete + reinsert enumeration is stale** (reflect_own_keys, frozen_object_enumeration): `delete o.b; o.b=4` gives stale key order/values. **Fix** (object-model).
5. **performance.now() → 0** (~21 + browser variants): host import not wired in the wasm lane (needs the 4 hand-mirrored import lists — see memory `kali-browser-harness-import-sync`). **Fix** (host wiring).
6. **Web crypto** (getRandomValues, subtle digest, randomUUID): same host-wiring family as #5. **Fix or flip**.
7. **dynamic_import member typeof** (~32): `typeof mod.f !== 'function'` outside the provable lane. **Fix or flip**.
8. **`&&`/`||` don't short-circuit side-effecting RHS** (boolean_logic, ~4+): explicitly **PR-B scope** (short-circuit semantics). Coordinate — likely leave to PR-B and flip here.
9. **Browser-harness `Kali.test` swallows traps** (a big slice of the 611 browser tests): the harness envelope reports `passed:1, success:true, exitCode 0` while stderr shows `RuntimeError: unreachable`. **This is a HARNESS fail-open worth its own fix** — once a callback body traps, the harness must propagate failure. Fixing this may turn MORE tests red (honestly) but is the correct foundation; without it, browser-mode results are untrustworthy.
10. **`array_callback_identity` .map/.filter → for-of push lane yields empty** (~8): pushing into a const array through for-of over `.map`/`.filter` results observes nothing. **Fix** (array/for-of lane).

## Interaction with the deferred item 9 (block arrows)
Block-arrow flatten closure (docs/superpowers/followups/task8-block-arrows-DEFERRED.md) is ALSO blocked on a repr gap (untracked function scopes). Both that repr-tracking prerequisite and this throw-fallout share the theme "un-flattening / un-masking exposes real runtime gaps." Sequence them together in planning.

## Recommended path
This is multi-spec-scale. Brainstorm → write a plan → subagent-driven execution, class by class, each class gated on `cargo test --workspace` vs main. Suggested order: harness-trap-swallow (#9, foundation) → runtime string equality (#2/#3, biggest green delta) → enumeration/delete-reinsert (#4) → host wiring (#5/#6) → array/for-of (#10) → adjudicate async (#1) and short-circuit (#8) vs PR-B / flip. Anything genuinely unsupported gets an honest fail-closed reject + re-pinned test, never a silent pass.

## Preserved artifacts
- All Batch-1 work committed on `soundness-batch1-pra` (PR #16 draft), final commit `100c8fed9`.
- Deferred item 9 patch: docs/superpowers/followups/task8-block-arrows-deferred.patch.
- Failing-set snapshots this session: the 922 are reproducible via `cargo test --workspace` on the branch vs a `main` worktree.
