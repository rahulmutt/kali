# throw-fallout Stage 2 — static object enumeration semantics

**Date:** 2026-07-11
**Branch:** `soundness-batch1-pra` (PR #16, draft/held per program policy)
**Status:** Design approved — ready for `writing-plans`
**Umbrella:** `docs/superpowers/specs/2026-07-11-throw-fallout-design.md` (Stage 2)
**Follows:** Stage 1 (`2026-07-11-throw-fallout-stage1-string-equality-design.md`, complete at 0a9d470da, drain 977→974)

## Problem

The umbrella's Stage 2 names one root cause — `delete o.b; o.b = 4` yields stale
enumeration — but the #4 name-bucket it points at (46 tests) is **multi-blocked**,
exactly like Stage 1's #2/#3 bucket turned out to be:

- The 40 `browser_reflect_own_keys` tests contain **no `delete` at all**. Their
  fixture exercises quoted keys (`{ "b": 1, "2": 2, "a": 3, "1": 4 }`),
  integer-first key ordering, `keys[0] !== '1'` element reads, and `for await`
  loops.
- Quoted-string object-literal keys have **no repr shape** (F-Stage1-4,
  `record_object_literal`'s Identifier-only let-else,
  `crates/kali_types/src/repr_infer.rs:478-486`): for..in fails closed E5506,
  and `Object.keys` element/length reads over a quoted-key literal are a
  **silent miscompile** (`keys.length` prints 2 for a 1-key object, `keys[0]`
  garbage; pre-existing on main).
- `delete obj.prop` on a plain object is an UNIMPLEMENTED **warning** +
  evaluate-and-drop no-op (`crates/kali_codegen/src/emit/operators.rs:200`;
  only the `process.env` lane is real) — a silent-miscompile factory.
- Enumeration (`Object.keys/values/entries`, `Reflect.ownKeys`) is a
  **compile-time constant fold** (`crates/kali_optimize/src/object_fold.rs`)
  over constant object-literal bindings. `collect_constant_bindings` treats a
  `const` binding to an object literal as an immutable value, so mutations
  through the binding (delete, member re-stores) are invisible — the staleness
  root cause. There is no runtime enumeration of mutated objects at all.
- The delete+reinsert self-check shape lives in the `runtime_smoke`
  object-enumeration fixture (`crates/kali_cli/tests/runtime_smoke.rs:954`).

Maintainer decision (this brainstorm): Stage 2's scope is the **static-
enumeration theme**, not the literal delete-only reading — everything blocking
honest static object enumeration, excluding blockers owned by other stages.

## Scope

**In scope (four lanes, detailed below):**

- Lane A — quoted-key repr shape (F-Stage1-4), end-to-end.
- Lane B — ES integer-first enumeration order as a single shared function.
- Lane C — delete+reinsert via a **static shape timeline** in the fold layer,
  plus replacing the delete warn+no-op with a fail-closed reject outside the
  provable lane (approach decision: static timeline over runtime key tables —
  every target fixture is static top-level code; runtime tombstone tables are
  YAGNI and touch the fixed-shape heap layout).
- Lane D — the silent element/length miscompile over enumeration results
  (diagnose first; fix here if in-theme, attribute if it is a Stage-4 array
  blocker).

**Out of scope, attributed not fixed:** `[]`+`.push` no-op (Stage 4), async /
for-await / Promise machinery (Stage 7), host wiring (Stage 3), mixed-type `==`
coercion (F-Stage1-1), env-vs-env equality (F-Stage1-2), bound-alias env.get
equality (F-Stage1-3).

**Target superset (names, deduped):** the #4 bucket's 46
(`browser_reflect_own_keys` 40, `reflect_own_keys_js_input` 4, `runtime_smoke`
direct-iteration 2) **plus** the 44 `frozen_object`-pattern names the Stage 1
triage tagged "#4-adjacent, Stage 2". The exact per-name attribution is an
output of the opening triage, not an input assumption.

## Opening triage (mandatory first task — Stage 1's headline lesson)

On a fresh branch binary, run each target fixture family and record the **first
observable divergence vs node** (byte-for-byte, same fixture). Attribute every
target name to lane A/B/C/D or to an out-of-stage cause (Stage 3/4/7). Drain
expectations for the stage checkpoint come **only** from this triage — Stage 1's
name-pattern drain forecast (~656) was falsified (actual: 3); no drain claim in
this stage may rest on a name pattern.

## Lane A — quoted-key repr shape (F-Stage1-4)

`record_object_literal` accepts `PropertyName::String` keys and materializes the
same `Repr::Object(shape)` an identifier-keyed literal gets — quoted and
unquoted keys are the same object in JS. End-to-end means:

- `kali_types` repr_infer records the shape; `object_shape_of_expression`
  resolves it; the for..in fixed-shape gate (E5506) admits it.
- The codegen mirror gets matching arms in the same task (the hand-mirrored
  oracle/predicate discipline from Specs 2/6: new key handling needs arms on
  **both** sides or it fails open).
- Interned key-table entries (Spec 4a handle table, Spec 7 module-constant
  for-in key tables) carry the key **text** (quotes are syntax, not content).
- Genuinely numeric literal keys (`{ 1: x }`, unquoted `PropertyName::Number`)
  are implemented only if the triage shows a target fixture needs them;
  otherwise they get an **honest explicit reject** (E-code), never the current
  silent deferred-conflict narrowing.

## Lane B — enumeration order, one source of truth

The ES order (integer-like keys first, ascending numeric; then string keys in
insertion order) currently lives in
`object_fold.rs::object_property_order_key`. Hoist the ordering into one shared
function in `kali_common` and make **all** producers call it: the optimizer
fold, kali_types shape construction, and codegen key-table emission. Ordering
divergence across layers becomes impossible by construction rather than by
vigilance. The audit of existing call sites is part of this lane.

## Lane C — delete+reinsert static shape timeline

Extend the fold's `BindingEnv` walk into a per-binding **property timeline**
through straight-line top-level code:

- `delete o.k` → remove `k` from the timeline.
- `o.k = v` with `k` absent → append `(k, v-node)` (insertion order restarts).
- `o.k = v` with `k` present → update the value node in place (order unchanged).
- Each enumeration call site (`Object.keys/values/entries`, `Reflect.ownKeys`)
  folds against the timeline state **at that program point**. The pinned shape:
  `const r = { a: 1, b: 2, c: 3 }; delete r.b; r.b = 4;` →
  `Object.keys(r)` = `['a','c','b']`, `Object.values(r)` = `[1,3,4]`,
  `Object.entries(r)` matching — node parity by construction.

**Validity gate (conservative, fail-closed):** the timeline applies only when
every mutation between the literal and the enumeration site is provably
straight-line (no loops, no branches around the mutations) and the binding does
not escape (not passed to a call, not aliased) in that span. Any shape outside
the gate does not fold and does not silently fall through — see error handling.

**Codegen:** an in-lane `delete` whose every observable effect is captured by
folded reads emits no runtime instructions. The existing `process.env` delete
lane is untouched.

## Lane D — enumeration-result element/length reads

The recorded silent miscompile: `const keys = Object.keys({ "b": 1 });
keys.length` prints `2` and `keys[0]` prints garbage (exit 0, no diagnostic;
node prints `1` / `b`; pre-existing on main). Root cause is **not yet
diagnosed** — the triage pins it. Expected outcomes, in order of likelihood:
it falls out of Lane A (no shape → fold misfires); it is a separate fold or
element-read gap (fix here, in-theme); it is the Stage-4 array lane (attribute,
do not fix here).

## Error handling / fail-closed posture

- **`delete` on an object outside the provable lane** gets a new error
  diagnostic (E-code allocated in the plan, following the object-lane E55xx
  convention), replacing the UNIMPLEMENTED warning + drop. A member read of a
  deleted-and-never-reinserted key (node: `undefined`) also **fails closed** —
  untested surface, YAGNI. This does not violate the program's zero-flips
  invariant: no failing target test is greened by a reject; rejects only
  replace silent no-ops on surface no target test exercises.
- **No silent narrowing:** every not-yet-supported key or mutation shape gets an
  explicit diagnostic, never a deferred-conflict silence or a warn-and-continue.
- **Both-sides discipline:** every new repr/expression arm lands in kali_types
  predicates and codegen oracles in the same task. The stage checklist includes
  the hand-mirrored-list grep sweep (`SYNTHETIC_FUNCTIONS` in
  `count_tag_boxing_ops`, runtime_smoke.rs — no new synthetics expected, sweep
  mandatory after Stage 1's two-test regression).

## Testing

- **Node-derived pins per lane, fresh binary** (Spec 5 lesson: re-run every
  reproducer on a freshly built binary; fix reports are unreliable).
  Byte-for-byte vs node on the same fixture.
- **Re-mask guards (Invariant 3):** for each greened fixture family, a
  deliberately-wrong variant proves the self-check `throw` still fires,
  anchored on a genuine mismatch (Stage 1 final-review fix pattern).
- **Negative pins:** delete-outside-lane rejects with the new E-code;
  delete-then-read-without-reinsert rejects; quoted-key `for..in` runs (E5506
  lifted) with correct key text and order; ordering parity pins across fold /
  for..in / key-table lanes.
- **Lane B property:** one shared ordering function; a test asserts fold order
  and key-table order agree on a mixed quoted/numeric/string-key literal.

## Stage gate (unchanged program mechanics)

`cargo test --workspace --no-fail-fast` → enumerate failing set → diff against
the persistent `../kali-main` worktree (re-verified 0 failures) → stage closes
only when the failing set **strictly shrank** with **zero** newly-red
green-on-main tests. Drain snapshot appended to the denominator doc
(`docs/superpowers/followups/throw-fallout-stage0-denominator.md`) with honest
per-name attribution, including target names that stay red on Stage 3/4/7
causes.

## Risks

1. **Multi-blocked bucket, again** — the 40 browser tests use `for await`;
   codegen call_tests show for-await over folded `Reflect.ownKeys` arrays
   already lowers, but if the browser lane still blocks on Stage-7 machinery,
   those names are attributed, not chased. The triage settles this first.
2. **Timeline validity gate too loose** — an escape/aliasing hole here is a
   silent staleness miscompile. The gate is conservative by construction and
   its negative pins are part of the definition of done.
3. **Ordering unification touches three layers** — behavior-neutral refactor
   risk; covered by the parity pins in Lane B.
4. **Hand-mirror regressions** — same class as Stage 1's `SYNTHETIC_FUNCTIONS`
   miss; the grep sweep is a mandatory checklist item.

## Definition of done

Opening triage recorded (per-name attribution); Lanes A–D implemented or
explicitly attributed per the triage; all in-lane pins green vs node on a fresh
binary; negative + re-mask pins green; stage gate passed (strict shrink, zero
newly-red vs `../kali-main`); drain snapshot + follow-up inventory appended to
the stage triage doc.
