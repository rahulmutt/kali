# Stage P2 — `structuredClone` deep-clone lane (design)

**Status:** design, approved for planning 2026-07-17
**Branch:** `soundness-batch1-pra`
**Predecessors:** Stage D close-out ([[kali-block-arrows-stageD]]); parity roadmap
P2→P3→P4→P5→webBaselineSmoke (stageD-triage §8.6).
**Follow-up inventory this stage draws from:** `docs/superpowers/followups/stageD-triage.md`
§8.6 ("Stage P2 — `structuredClone`") and the object-`===` fail-open observed
during this brainstorm.

---

## 0. Goal and the acceptance target

`structuredClone` is the first roadmap parity primitive. Today it is a bare
builtin name (`kali_types/src/builtins.rs:111`) with **no lowering**: wherever it
is the first genuinely-unsupported call in a fixture it traps `E4000` at runtime
(stageD-triage §8.4's deliberate flip). This stage makes it evaluate a real deep
clone over the smallest sound value envelope, and closes the two soundness gaps
that block its own acceptance fixture from going green.

The acceptance fixture is the `structuredClone` prefix of `webBaselineSmoke`
(`runtime_smoke.rs:424` `structured_clone_and_event_primitives_source`, and
`runtime_smoke.rs:4462` `browser_bundle_web_baseline_source`):

```js
const original = { count: 1, values: [1, 2, 3] };
const cloned = structuredClone(original);
if (cloned === original || cloned.values === original.values) {
  throw new Error('structuredClone should deep-clone object graphs');
}
original.values.push(4);
if (cloned.count !== 1 || cloned.values.join(',') !== '1,2,3') {
  throw new Error(`unexpected structuredClone result ${JSON.stringify(cloned)}`);
}
```

For this to pass byte-for-byte against node, four things must be true at once:

1. `structuredClone(original)` returns a **distinct** heap object whose `count`
   scalar and `values` array are **independently allocated** (mutating
   `original.values` must not touch `cloned.values`).
2. `cloned === original` and `cloned.values === original.values` must evaluate to
   a **real `false`** — object/array reference identity.
3. An **array-valued object field** (`original.values`) must actually work:
   `o.values.push(...)`, `o.values.join(...)`, `o.values.length` on a field must
   round-trip. **Probes show this does not work today** — see §1.
4. Out-of-envelope `structuredClone` arguments elsewhere in the corpus (notably
   `structuredClone(new Blob([...]))` in the browser package corpus) must still
   **build** (`check` / `build --bundle` succeed) — see §2.3.

### Probes run during brainstorm (clean debug binary)

| probe | source | node | kali today | consequence |
|---|---|---|---|---|
| p2a | `structuredClone(o); cloned === original` | `false` | prints `1` | `===` **fails OPEN** on unknown-repr operands — §3 |
| p2b | `const b=a; a===b` (object identity) | `true` / `false` | `E5506` ×2 | object `===` currently compile-denied when a shape is known |
| p2c | `JSON.stringify(original)` | `{...}` | `0` | `JSON.stringify` on objects unsupported — out of scope, see §4 |
| p2d | `o.values.push(4); o.values.join(',')` | `1,2,3,4` | prints `0` | **array-valued field is a silent miscompile** — §1 |
| p2e | `o.values.join(',')` (read-only) | `1,2,3` | prints `0` | same class, even without mutation |

p2d/p2e are the load-bearing discovery: **array fields do not round-trip at all
today**, so Lane 1 (array-valued object fields) is a hard prerequisite, not an
optional widening.

### Non-goals (fail closed, not miscompile)

- String fields / nested objects / aliasing / cycles inside a cloned value.
- `JSON.stringify` over objects (the fixture's error branch calls it, but only on
  the failure path, which must not be taken; if it *were* taken kali would print
  the wrong string — acceptable because it is unreachable in a passing run, and
  §5 pins that the happy path is taken).
- `AbortController` / `Event` / URL / TextEncoder (P3–P5).

---

## 1. Lane 1 — array-valued object fields (prerequisite)

**Problem.** `object_shape_of_node` assigns a field the repr of its initializer.
For `{ count: 1, values: [1,2,3] }`, `count` is `I64` and `values` should be an
array. Probes p2d/p2e prove that reading `o.values` and then calling
`.push`/`.join`/`.length` on the result yields `0` — the field load produces a
value the array-method lowering does not recognise as an array handle, so it
silently no-ops. This is a **pre-existing silent miscompile** independent of
`structuredClone`; P2 cannot land on top of it.

**Fix, stated as an allowlist at the field-access choke point (not a denylist).**
Per the standing lesson (denylists leak forever; only an allowlist at the single
read site closes a class — [[kali-throw-fallout-stage5]], Stage D C-1), the object
shape model must carry, for each field, whether the field is a **growable-i64
array** (the only array field shape P2 supports). At the point where `o.field` is
read into a value that then feeds an array method:

- **Allow** exactly when the field's declared repr is a growable-i64 array
  (element repr `I64`, same lane as `growable_array_bindings`), producing a real
  `ARRAY_HANDLE_TAG` handle so `push`/`join`/`length`/index/`for..of` lower
  through the existing growable synthetics.
- **Deny (E5506)** every other array-shaped field position: string-element
  arrays, nested-object fields, inline `[len][elem…]` (non-growable) fields, or
  any field whose element repr is unproven. These are P3+ or never; they must not
  reach a method lowering that treats the loaded slot as a scalar.

**Field storage.** A growable-array field stores the array **handle** (an i64) in
the object's 8-byte slot, exactly as a growable-array binding stores its handle in
a local. No new memory layout: the object slot already holds an i64; we are only
teaching the shape model + the field-read path that this particular i64 is an
array handle, not a number.

**Why this is the minimal envelope.** `webBaselineSmoke`'s `original.values` is a
growable-i64 array field and nothing else in the acceptance fixture needs more. A
wider envelope (string/nested fields) has no fixture demanding it and would carry
an aliasing story P2 has no reason to open.

**Tripwire.** A field whose element repr is not provably `I64` must red a test,
not silently widen. Add `structured_clone_string_array_field_fails_closed`
(a `{ vals: ['a','b'] }` field method → assert E5506).

---

## 2. Lane 2 — the clone itself (per-shape synthetic + arg dispatch)

### 2.1 Per-shape clone synthetic

Following the `__join` / `__substring` synthetic precedent
(`SYNTHETIC_FUNCTIONS`, `lower.rs:38`), P2 synthesises **one clone function per
distinct cloned shape**: `__clone_shape_<ShapeId>() `. The synthetic:

1. Allocates a fresh object of the same `ShapeId` (same field count, `n*8` bytes,
   headerless — same allocation path object literals use).
2. For each field, by the field's repr:
   - **`I64` / `F64` scalar** → copy the 8-byte slot verbatim.
   - **growable-i64 array field** (Lane 1) → allocate a fresh array handle and
     deep-copy the elements (a per-element `i64` copy loop over the source
     handle's `[len][cap][data_ptr]`), so `cloned.values !== original.values`
     and later `original.values.push(4)` cannot reach the clone.
   - **anything else** → unreachable: the clone synthetic is only emitted for a
     shape all of whose fields are in the allowlist (§2.2 gates this before
     emission), so no field arm can silently fall through.
3. Returns the new object pointer.

Registered in `SYNTHETIC_FUNCTIONS` **and** the test-side census
(`runtime_smoke.rs:806` `SYNTHETIC_FUNCTIONS` + `count_tag_boxing_ops`) — the
known census-sync requirement ([[kali-throw-fallout-stage4]]): a WAT census of the
hot path must prove **zero** tag-boxing ops introduced, and the two lists must
match or the census test reds.

### 2.2 Call-site dispatch — a 3-way allowlist

At a `structuredClone(arg)` call, resolve `arg`'s provenance and route:

1. **In-envelope object** — `arg` proves `Repr::Object(shape)` and every field of
   `shape` is in the Lane-1 allowlist (scalar or growable-i64 array) → emit/call
   `__clone_shape_<shape>`. This is the only lane that produces a real clone.
2. **Provable zero-placeholder construct** — `arg` is a `new X()` whose lowering
   is the drop-and-push-`0` aggregate placeholder
   (`declarator_init_is_placeholder_construct`, `lower.rs:2145`, excluding the
   real-value constructs `Array`/`Uint8Array`/`EventTarget`). This is the
   `structuredClone(new Blob([...]))` corpus case. Keep **today's** behavior:
   the argument lowers to placeholder `0`, `structuredClone` returns `0`, and a
   warn (not error) is emitted. `check` / `build --bundle` succeed — the corpus
   build pins (`package_corpus.rs`, `browser_corpus.rs`) stay green under the
   same "unsupported constructs must still build, same-0-both-sides" rationale
   ratified in Stage D C-1. This lane produces **no** usable clone (nothing reads
   the result in those fixtures; they call `structuredClone(new Blob(...))` for
   its build-surface only).
3. **Everything else** — params, strings, growable arrays passed directly,
   unknown-repr values, in-envelope-but-string/nested-field objects → **E5506**
   at the call site (default-deny). The clone synthetic is never emitted for
   these; nothing reaches a field-copy loop it cannot prove.

This mirrors the Stage D C-1 `unlowered_capture_denied` allowlist shape exactly:
default-deny, two narrow provable-safe allowlist entries, everything else closed.

### 2.3 Hand-mirror tripwire (standing wrong-allow risk)

Lane-2 entry 2 admits a captured/argument `new X()` **only because** its lowering
is drop-and-push-0 **today**. The exclusion list inside
`declarator_init_is_placeholder_construct` (`Array`/`Uint8Array`/`EventTarget`) is
hand-mirrored; if `Blob`/`File`/`Set`/`Map` ever gain a real-value lowering
without being added to the exclusion list, entry 2 flips into a value-losing
wrong-allow (structuredClone would return `0` for a now-real value). Reuse the
existing Stage D tripwire pattern: add
`structured_clone_of_placeholder_construct_tripwire` pinning kali's current
`structuredClone(new Blob([...]))` → returns-`0` / build-succeeds behavior against
node's real Blob, as a **deliberate tripwire** (not a correctness claim) that
reds the day Blob gains a real lowering, forcing the exclusion-list decision.

---

## 3. Lane 3 — object/array reference `===` / `!==`

**Problem.** The fixture needs `cloned === original` → `false` and
`cloned.values === original.values` → `false`. Probe p2a shows the current path
**fails OPEN**: with `structuredClone` returning an unknown-repr value, neither
operand proves an object shape, the object-misuse gate at `operators.rs:1500`
does not fire, and the `===` arm falls through to a scalar/bigint compare that
prints `1` (node prints `false`). Probe p2b shows the *other* direction: when a
shape **is** known, the same gate currently emits `E5506` (compile-deny), so
plain object identity is unavailable even where it is sound.

**Fix — same-shape-only allowlist, default-deny.** Replace the blanket object-gate
behavior for `===`/`!==` with:

- **Allow (real pointer compare)** exactly when **both** operands are **proven
  heap references of the same shape** — both `Repr::Object(same ShapeId)`, or both
  growable-i64 arrays. Lower to an i64 pointer equality (`I64Eq` / `I64Ne`).
  Distinct allocations compare unequal at runtime; an alias (`q = p`) compares
  equal — matching node.
- **Deny (E5506)** everything else: one operand object + one not, cross-shape,
  unknown-repr (the p2a case), string-vs-object, etc. This **closes the p2a
  fail-open at the same choke point** the allow-lane lives in — no separate
  denylist to leak.

Once P2 gives `structuredClone(original)` a real `Repr::Object(shape)` result
(§2.2 lane 1 returns the same `ShapeId` as `original`), both fixture compares land
in the allow lane: same shape, distinct pointers → real `false`. The
`cloned.values === original.values` compare lands in the growable-array allow lane
(both fields are growable-i64 arrays, distinct handles → `false`).

**Note on `!==` scalar-field compares.** `cloned.count !== 1` and
`cloned.values.join(',') !== '1,2,3'` are scalar/string compares that already
work; they are not touched by this lane (they never reach the object gate).

**Tripwire.** `structured_clone_cross_shape_identity_fails_closed`: `a === b`
where `a` and `b` are different shapes → assert E5506 (proves cross-shape did not
silently pointer-compare two incompatible layouts).

---

## 4. Edge cases, diagnostics, build-compatibility

- **`JSON.stringify(cloned)`** appears only on the fixture's *error* path
  (`unexpected structuredClone result ${JSON.stringify(cloned)}`). A passing run
  never evaluates it. P2 does **not** implement `JSON.stringify` over objects
  (probe p2c: kali prints `0`). §5 pins that the happy path is taken, so this
  string is never materialized. If a future fixture takes that branch, it is a
  separate follow-up (inventory it, do not silently miscompile — the
  scalar-`0` output is loud-wrong, not silently-plausible).
- **`structuredClone` with zero or >1 args** → E5506 (arity mismatch at the call
  site; node's `structuredClone` is single-arg with an options bag we do not
  model).
- **Diagnostic text.** Reuse the canonical `E5506` "feature unavailable" family
  with a specific message per deny site: field-shape deny (Lane 1), call-arg deny
  (Lane 2 entry 3), identity-operand deny (Lane 3). Each names *what* was
  unproven so the message is actionable ("structuredClone argument of unproven
  shape", "'===' between values of different shapes", "array-valued field with
  non-integer elements").
- **Build-surface preservation.** The corpus build pins
  (`package_corpus.rs:205/218/649`, `browser_corpus.rs:2306`) call
  `structuredClone(new Blob([...]))` / `(new File([...]))` and assert
  `check` / `build --bundle` **success only** (they do not execute). Lane 2 entry
  2 keeps those green with **zero re-pins**. This is verified in §5 before any
  gate claim.

---

## 5. Testing & acceptance protocol

Aligned with the project's full-workspace gate discipline
([[ci-gate-vs-poisoned-baseline]]): gate `cargo test --workspace` diffed against a
**main worktree**, never a mid-branch baseline; enumerate with `--no-fail-fast`.

**New pins (all in `soundness_events.rs` / a new `soundness_structured_clone.rs`,
each asserting the program RUNS, not just AST shape — [[kali-throw-fallout-stage5]]):**

1. `structured_clone_deep_clones_scalar_and_array_object` — the acceptance
   fixture body, run via `kali run`, byte-for-byte vs node: distinct object,
   distinct array, `original.values.push(4)` does not touch `cloned.values`,
   `.join(',')` → `1,2,3`.
2. `structured_clone_result_identity_is_false` — `cloned === original` and
   `cloned.values === original.values` both → `false` (Lane 3 allow lane).
3. `object_array_field_push_join_length_round_trip` — Lane 1 standalone (p2d/p2e
   as a green test).
4. `structured_clone_string_array_field_fails_closed` — Lane 1 tripwire (E5506).
5. `structured_clone_of_unproven_argument_fails_closed` — Lane 2 entry 3 (param /
   unknown-repr arg → E5506).
6. `structured_clone_of_placeholder_construct_tripwire` — Lane 2 entry 2
   (Blob/File returns-0 + build-succeeds; deliberate tripwire vs node).
7. `structured_clone_cross_shape_identity_fails_closed` — Lane 3 tripwire (E5506).
8. `same_shape_object_identity_alias_is_true` — `q = p; p === q` → `true` (proves
   the allow lane is not vacuously always-false).

**Census:** WAT-census the acceptance hot path proves **0** tag-boxing ops; the
`runtime_smoke.rs:806` `SYNTHETIC_FUNCTIONS` list and `count_tag_boxing_ops`
allowlist include `__clone_shape_*` (or the census test reds).

**Re-masking checks (prove the lanes are real, not coincidence-green):**
- Break the array-field deep-copy (share the handle) → test 1 must go red
  (`cloned.values` would see the `push(4)`).
- Force `structuredClone` to return the same pointer → test 2 must go red.
- These mirror the Stage D "coincidence-green" lesson (a test that passes only
  because two values happen to coincide is not a real pin).

**Gate acceptance:**
- Full `cargo test --workspace --no-fail-fast` enumerated twice, zero drift.
- **0 newly-red** vs the main worktree (the only gate that counts).
- The 4 corpus build pins + the `structured_clone_and_event_primitives` /
  `browser_bundle_web_baseline` tests re-run on a freshly-built binary
  ([[kali-fasta-output-argv-spec5]]: fix reports are unreliable — re-run the
  reproducer on a fresh build).
- Whole-stage adversarial review (most-capable model) before certification —
  every prior stage's whole-stage review caught a silent miscompile no per-task
  review saw; budget for it.

**What this stage does NOT claim.** `webBaselineSmoke` end-to-end stays red after
P2 — it still hits `AbortController`/URL/TextEncoder (P3–P5). P2's acceptance is
the `structuredClone`-prefix behavior above plus 0-newly-red; the full
byte-for-byte `webBaselineSmoke` flip is the P5 close-out item (§8.6).

---

## 6. Soundness posture summary

Every new capability is an **allowlist at a single choke point**, default-deny:

| Lane | Choke point | Allow | Deny (E5506 / warn-build) |
|---|---|---|---|
| 1 array fields | field-read → array method | growable-i64 field | every other field array shape |
| 2 clone dispatch | `structuredClone(arg)` call | in-envelope object; zero-placeholder construct (warn) | all else |
| 3 identity | `===`/`!==` object gate | same-shape heap ref pair | mixed / cross-shape / unknown |

No denylists. Three tripwire pins guard the two hand-mirrored surfaces (placeholder
exclusion list, cross-shape identity). The fail-open probes (p2a `===`, p2d/p2e
array field) are each closed **by construction at the allow site**, not patched at
a sink.
