# throw-fallout Stage 4 — Growable runtime array (array/for-of push lane, #10)

**Date:** 2026-07-13
**Branch:** `soundness-batch1-pra` (PR #16, draft/held)
**Status:** Design approved — ready for `writing-plans`
**Umbrella:** `docs/superpowers/specs/2026-07-11-throw-fallout-design.md` (Stage 4)
**Stage entry denominator:** 834 (`cargo test --workspace --no-fail-fast`, branch HEAD `04d34b9f2`;
`.worktrees/kali-main` = 0 failures)

## Problem

Stage 0 made `throw` sound (print-then-trap), un-masking a self-check `throw` in the
`array_callback_identity_browser_harness` fixture. The fixture accumulates into a mutable array:

```js
function browserArrayCallbackIdentitySlices() {
  const observed = [];
  for (const item of [1, 2].map((value) => value))            { observed.push(item); }
  for (const item of [1, 2].filter((value) => value))         { observed.push(item); }
  for (const item of Array.from([1, 2].filter((v) => v)))     { observed.push(item); }
  for (const item of [...[1, 2].filter((value) => value)])    { observed.push(item); }
  for (const item of [1, 2].flatMap((value) => [value]))      { observed.push(item); }
  console.log(`some:${[0, 1].some((value) => value)}`);
  console.log(`every:${[1, 0].every((value) => value)}`);
  if (observed.join(",") !== "1,2,1,2,1,2,1,2,1,2") {
    throw new Error('unexpected array callback identity semantics');
  }
  console.log(observed.join("\n"));
}
```

**Empirically pinned on a fresh branch binary (`04d34b9f2`), node v26.x parity:**

- All five for-of **sources** already fused-iterate correctly (`[1,2].map(id)`, `.filter`,
  `Array.from(.filter)`, `[...filter]`, `.flatMap`) — verified standalone, each prints `1\n2`.
- `.some`/`.every` on literals already work.
- **`observed.push(item)` is a silent no-op**: `const o=[]; o.push(1); o.length` → `0`, `o[0]` →
  `undefined`, exit 0. `o.join(",")` over a pushed array → empty. **Silent miscompile.**
- Consequently `observed.join(",")` is `""`, the `!==` guard is true, the un-masked `throw` fires,
  and all 16 tests fail honestly.

**Root cause:** there is **no growable runtime array value**. Arrays today are static-literal
layouts — `Repr` is `I64 | F64 | Object(ShapeId) | String` with no array value kind; array bindings
are tracked as `array_bindings`/`array_elements` over a *statically-known* length and fixed
element offsets, and `__join` walks that static length. A binding that is `push`-mutated has no
backing store to grow.

## The 16 red tests (only array-family reds in the 834)

`crates/kali_cli/tests/array_callback_identity_browser_harness.rs` — `{run,test} ×
{plain,json} × {js,ts,jsx,tsx}` = 16, all running `--api browser` with
`BROWSER_HARNESS_COMMAND=node`. All fail on the same push-no-op. Denominator **834 → 818**.

The rest of the `array_callback_*` family (filter_predicate, find, identity_filter,
identity_flat_map, identity_map, iteration_gates, number_predicates_{runtime,browser_harness},
reduce) is already green and must stay green.

## Scope

**In:** a real growable runtime array supporting exactly the surface the fixture (and the general
silent-miscompile class) needs — `const x = []` / `const x = [seed…]`, `x.push(v)`, `x.length`,
`x[i]` read, `for (const v of x)`, `x.join(sep)` — matching node byte-for-byte. Fixing push
by construction closes the direct-lane silent push-no-op miscompile (`const o=[]; o.push(v)`),
even though no other current test exercises it.

**Out (honest fail-closed E5506, never a silent miscompile; no target test needs these):**

1. Binding a `.map()`/`.filter()`/`.flatMap()` **result** to a variable and reading its
   `.length`/index (materialization — repro D: `const out = src.map(...)`). The sources stay
   *fused* into for-of; they do not materialize.
2. A growable array that **escapes** its defining function (returned, stored into an object/array
   field, or assigned to an outer-scope binding). No cross-arena growable array in this stage.
3. Mutators other than `push` (`pop`/`shift`/`unshift`/`splice`/…) and re-assignment of the
   binding to a different array.

Anything Out rejects with a real E5506 diagnostic. **No-flip invariant (umbrella):** the fix makes
`push` *work*; it never rejects-to-pass a construct that the fixture needs.

## Representation (Approach A — header'd heap array, geometric growth)

A growable array is a tagged handle into an arena-allocated header + data buffer:

```
handle = ARRAY_TAG | (hdr_ptr << 32)          ; tagged i64 value, like the string/object handles
hdr:  [ len | cap | data_ptr ]                 ; 3 i64 words
data: [ v0 | v1 | ... | v(cap-1) ]             ; cap i64 tagged slots at data_ptr
```

- Element slots are **i64 tagged values** — numbers as i64, strings as string handles — reusing the
  existing tagged-value model and the `array_elements` element-repr table. This stage supports
  **i64 and String** element reprs (the fixture pushes integers; string push+join is the same lane
  and is supported). F64 / Object elements are out (fail-closed until a test needs them).
- The header carries `data_ptr` separately so the handle stays **stable across a realloc**: growth
  allocates a new `2×cap` data buffer, copies `len` slots, and rewrites `hdr.data_ptr` — the handle
  (pointing at `hdr`) never changes, so aliases through a local remain valid.
- Initial `cap` for an empty `[]` is a small constant (e.g. 4); a seeded `[a,b]` allocates
  `cap = max(seed_len, 4)` and copies the seed.

## Recognizing a growable binding (the crux)

Today `const x = []` is a static-literal array binding. The inference must **promote** an array
binding to *growable* when it is a `push` receiver (the mutation site), seeding length/elements
from the literal initializer:

- New `kali_types` distinction, e.g. `growable_array_bindings: HashSet<(func, binding)>`, populated
  when a binding both (a) is an array binding and (b) is the receiver of `x.push(...)` somewhere in
  its function, and (c) does **not** escape (per `escape_flow`) and is not reassigned. If it is a
  push receiver but escapes/reassigns → **shape conflict → E5506** (fail-closed), not growable.
- Element repr = the join of seed-element reprs and all pushed-value reprs (via the existing
  `array_elements` / repr-inference union machinery). A mixed/unsupported element repr fails closed.
- **Both-sides mirror:** codegen's growable-array oracle and the `kali_types` predicate are
  hand-mirrored. Every construct below needs an arm on **both** sides or it fails open
  (repeated program lesson — `kali-substring-runtime-spec2`, `kali-forin-spec4a`).

## Operation lowering

Each row needs a codegen emit arm **and** its mirrored `kali_types` recognizer/repr arm.

| construct | lowering |
|---|---|
| `const x = []` / `[a,b]` (growable) | alloc `hdr`+`data` in the enclosing arena; copy seed slots; `len = seed_len` |
| `x.push(v)` | `if len==cap { grow 2×: alloc, copy len, set data_ptr, cap*=2 }`; `data[len]=v`; `len++` |
| `x.length` | load `hdr.len` |
| `x[i]` read | load `data[i]` for `0 ≤ i < len`; **OOB → `undefined`** (node parity). Decide in the plan whether undefined is cheap to represent here or a bounded fail-closed reject is warranted; the fixture never indexes, so either satisfies the gate — undefined is the node-faithful choice. |
| `for (const v of x)` | counted loop `i in 0..len`, body reads `data[i]` (reuses the existing for-of counted-loop lowering, retargeted from static length to `hdr.len`) |
| `x.join(sep)` | generalize `__join` from a static length to a **runtime** `hdr.len`, rendering each slot per its element repr (int → decimal, string → bytes) with `sep` between — reuse the Spec 3 bulk-memory `__join` renderer |
| for-of **sources** (`.map`/`.filter`/`Array.from`/spread/`.flatMap`) | **unchanged** — already fused-iterate; the loop **body's** push is the only new behavior |

## Reclamation & escape (GC-less invariant)

The header + data buffer live in the enclosing **function/loop arena** (the existing reclaiming
allocator from binary-trees Phase 1 / `kali-reclaiming-allocator-phase0`). `escape_flow` gates the
binding:

- Function-local, non-escaping `observed` → the **function arena**; the array and every
  realloc-abandoned old buffer are reclaimed en masse on function return.
- An **escaping** growable array fails closed (§Scope Out #2) — there is no cross-arena growable
  array in this stage.
- **No tracing/copying GC** — reclamation is region/escape only, per `kali-gc-less-invariant`.

## Error handling

Every unsupported growable-array construct emits a **fail-closed E5506** with a real diagnostic;
nothing silently no-ops. This is the direct inverse of the current defect: today `push` silently
does nothing (exit 0, wrong answer); after this stage it either works or rejects, never lies.

## Gate & review mechanics

- **Gate:** `cargo test --workspace --no-fail-fast` on the branch, failing set diffed against the
  persistent `.worktrees/kali-main` worktree (0 failing). A stage passes only when the 16 targets
  are green **and** the global failing set strictly shrinks (834 → 818) with **zero** main-green
  tests newly red. Per `ci-gate-vs-poisoned-baseline`, this whole-workspace enumeration is the only
  sufficient gate — per-task "0 regressions" is necessary but not sufficient (Stage 2/3 lesson).
- **Adversarial whole-stage review** (the review that caught Stage 3's silent regressions the gate
  missed): fresh-binary-vs-node probing of push accumulation, `join` output (both `,` and `\n`
  separators), **growth across a realloc boundary** (push count > initial cap, verifying no
  slot corruption on copy), **int and string** element joins, `x.length`/`x[i]` after pushes, and a
  re-masking check — the push must *actually accumulate*, not silently pass a re-hidden guard
  (Invariant 3).
- **Browser-lane check:** the 16 targets run `--api browser`. Array ops are pure-wasm (no new host
  import), so the 4-list browser import-sync hazard (`kali-browser-harness-import-sync`) is expected
  N/A — **confirm** in the plan (grep the four `kali:rt` import lists; if `__join`/growth needs a
  runtime helper import, all four must carry it).

## Invariants carried from the umbrella

1. **Fix, never flip** — push is implemented, not rejected-to-pass.
2. **No silent miscompiles** — unsupported shapes fail closed with a diagnostic.
3. **No re-masking** — a fix that re-silences the self-check `throw` is a defect even if green.
4. **Parity is node**, same fixture, byte-for-byte.
5. **Both-sides mirror** — codegen oracle ⇔ `kali_types` predicate on every new construct.
6. **GC-less** — arena/escape reclamation only.

## Definition of done (this stage)

The 16 `array_callback_identity_browser_harness` tests green; `cargo test --workspace` failing set
834 → 818 vs `.worktrees/kali-main` with zero newly-red; all fixes real (zero flips); no
re-masking; silent push-no-op closed by construction; follow-ups (materialization, escaping
growable array, extra mutators, OOB-index decision) filed. Branch stays UNMERGED (PR #16 draft).
