# fasta `for..in` + fixed-shape dynamic string-keyed access (Spec 4a) — design

**Date:** 2026-07-07
**Status:** Approved (design)
**Series:** Runtime strings & dynamic tables for verbatim fasta — **Spec 4a of 6**
(Spec 4 split into 4a/4b during brainstorming; see Series context.)

## Series context

The CLBG target is **fasta**, compiled from the *verbatim upstream* Node.js
program. Specs 1–3 shipped the runtime-string foundation:

- Spec 1 (PR #9) — string-typed value flow (fix `E3200`): variables / params /
  returns carry a string handle through the value model.
- Spec 2 (PR #10) — runtime `substring` / `.length` (relax `E5506`).
- Spec 3 (PR #11) — `Array.prototype.join(sep)` over string-element arrays.

The remaining string-native surface fasta needs is its **object tables**
enumerated with `for..in`, where the enumerated key is used as a runtime
property index (`table[c] += table[last]`) and, in `selectRandom`, returned as a
string. That is the original **Spec 4 — `for..in` live-key + dynamic
string-keyed property get/set (fix `E4201`)**, flagged in the Spec 1 series doc
as "hardest / most orthogonal; may itself split."

**It splits here into 4a and 4b:**

- **Spec 4a (this doc)** — fixed-shape `for..in` enumeration + computed
  string-keyed get/set over an object whose **shape (and therefore key set) is
  known at compile time**. This is exactly what fasta's two `for..in` sites
  need. Kali is a fixed-shape, GC-less compiler ([[kali-gc-less-invariant]],
  [[kali-heap-object-lane]]): the keys in `makeCumulative`/`selectRandom` are
  precisely the object's own fields, so no runtime string→value map is required.
- **Spec 4b (deferred, only if a later target needs it)** — general runtime
  string-keyed property maps: arbitrary runtime keys not derived from a
  `for..in` over the same object, real string-compare/hash lookup in linear
  memory. No current CLBG target requires it.

Remaining series after 4a: **Spec 5** (`process.argv` runtime element read +
string→number coercion) and **Spec 6** (vendor fasta verbatim + two-tier
validation capstone).

## The two fasta `for..in` sites (the whole target of 4a)

```js
// table = { a:0.27, c:0.12, g:0.12, t:0.27, B:0.02, D:0.02, ... }  (float values)

function makeCumulative(table) {
  var last = null;
  for (var c in table) {
    if (last) table[c] += table[last];   // key used only to INDEX table's own fields
    last = c;
  }
}

function selectRandom(table) {
  var r = random(1), c;                   // `random` = fasta's deterministic LCG
  for (c in table)
    if (r < table[c]) return c;           // key RETURNED as a string (the nucleotide char)
  return c;
}
```

`makeCumulative` uses the key only as an index into `table`'s own fields.
`selectRandom` additionally returns the key as a real string handle. **Spec 4a
covers both** (decided in brainstorming): the key is usable as an index *and*
materialized as a string value.

fasta's `random` is a deterministic LCG (`last = (last*3877 + 29573) % 139968;
return max*last/139968`) — integer/float arithmetic already supported. The table
values are floats. Keys are single-character valid-identifier field names.

## Problem

`for..in` is currently a no-op-ish miscompile: it falls through `emit_node`'s
`Branch` match to `emit_branch` (mis-lowered as an `if`) and is **deliberately
skipped by both arena-ordinal walks** (`kali_codegen/src/lower.rs:1200`,
`kali_mir::analysis::walk`). Computed string-keyed access over a fixed-shape
object is not lowered. The front-end gates `E5506` / `E4201` reject these shapes
on purpose, to avoid miscompiles on unwired paths.

The standing warning in `lower.rs:1200` is a first-class constraint: any real
`for..in` implementation **must teach both walks to recognize `for..in`
together** — giving one walk an ordinal without the other desyncs every real
loop lexically after it, sending `loop_arena(fn, ordinal)` lookups to the wrong
loop (a use-after-reset miscompile). The `for..in` loop itself allocates no
per-iteration heap, so it takes **no arena** — but both walks must agree on that
in the same change.

## Chosen approach: runtime ordinal key + provenance-checked computed access

`for..in` over a fixed-shape object lowers to a **real counted loop** over an
ordinal, reusing three existing lanes rather than inventing new machinery:

```
for (var c in table)      ==>   i = 0; while (i < N) { c = i; <body>; i += 1 }
                                 // N = table's shape field count (compile-time)

table[c]                  ==>   load(base + i*8)          // == existing array-element read
table[c] += table[last]   ==>   load/add/store at base + i*8 and base + last*8
return c                  ==>   return key_handles[i]      // interned field-name string handle
if (last)                 ==>   last >= 0                  // null sentinel: -1 = null, >=0 = key
```

**Why this over the alternatives** (both considered and rejected in
brainstorming):
- *Compile-time unroll* (emit N body copies, key a constant per copy): simplest
  analysis and sidesteps the both-walks ordinal fix, but code size scales with
  shape (IUB = 15 fields → 15× body), needs a novel unrolling pass, and is a
  dead-end for larger shapes.
- *Runtime string-key dispatch* (bind key to a handle, compare against each
  field name per access): over-general for 4a — it is most of 4b's machinery,
  and blurs the 4a/4b line.

The chosen approach reuses the loop-lowering lane, the array-element-address
lane (a fixed-shape object's field `j` sits at `base + j*8`, **identical to
array layout**), and Spec 1's string-value flow (for key-as-string). It
generalizes cleanly toward 4b later.

## Architecture — four new pieces, each reusing an existing lane

**1. `for..in` recognition in both walks — the desync-safe part.**
`kali_codegen/src/lower.rs` and `kali_mir::analysis::walk` learn to recognize
`ForInStatement` **in the same change**. The `for..in` loop takes no arena; both
walks must agree on that so no real loop after it desyncs its ordinal. A pin
proves a real loop lexically following a `for..in` keeps its arena ordinal, and
REDs if only one walk is taught.

**2. Counted-loop lowering (`kali_codegen`, reuse the control-flow lane).**
`for (var c in obj)` reuses the existing while/for loop machinery, counting an
ordinal `i` from `0` to `N-1` (`N` = the shape's field count, a compile-time
constant from the resolved `Repr::Object(ShapeId)`). The loop variable `c` is
bound to `i`.

**3. Computed key access = array-element access (`kali_codegen`, reuse
`emit_array_element_address`).** `obj[c]` with `c` an ordinal lowers through the
existing `emit_array_element_address` / element-write path. Compound
`obj[c] += obj[last]` = load both addresses (f64 field repr from the shape),
add, store. This is the "`a[i]` mirror" prior reviews praised.

**4. Key-as-string materialization (`kali_codegen` + `kali_types`, reuse Spec 1
string flow + literal interning).** When `c`/`last` is used as a value
(`return c`), emit `key_handles[i]`: a compile-time table of the shape's field
names interned as string handles
(`STRING_HANDLE_TAG | offset<<32 | len`, `kali_codegen/src/lib.rs:66`), indexed
by ordinal. From there it flows through Spec 1's string-value model
(`return`, `console.log`, `+`, `==`).

**Binding these together — a provenance axis in `kali_types` repr_infer.** A
binding is tagged **for..in-key-of-shape-S**, seeded at the `for..in` left-hand
variable and propagated through `last = c`. Computed access `obj[key]` is
admitted **only** when `key` carries key-provenance matching `obj`'s shape;
everything else fails closed. This is the Spec-3 lesson applied: *mirror binding
provenance, not just expression shapes* ([[kali-runtime-join-spec3]]).

## Data flow

**`makeCumulative(table)`:**
1. repr_infer infers `table`'s param shape from the call-site object literal
   (existing nbody param-shape flow). **Field order = literal order = ordinal
   order** — the load-bearing correspondence.
2. `var last = null` seeds a key-or-null binding (null sentinel `-1`).
   `for (var c in table)` seeds `c` with for..in-key-of-`table`-shape
   provenance. `last = c` propagates it.
3. `table[c] += table[last]` — both indices carry matching key-provenance →
   admitted; two `base + i*8` loads, add, store at `base + i*8`.
4. `if (last)` lowers to `last >= 0`; first iteration `last == -1` → false,
   matching JS `if (null)`.

**`selectRandom(table)`:** `table[c]` is an f64 field load; `if (r < table[c])
return c` materializes `key_handles[i]` and returns a string handle. The
post-loop `return c` returns the last key.

## Error handling — fail-closed matrix

Fail-closed, never fail-open: any receiver / key / target the analysis cannot
prove safe rejects with a diagnostic (`E5506 = FEATURE_UNAVAILABLE`, and/or the
`E4201` invalid-module guard). Each row gets a live e2e reject pin:

- `obj[k]` where `k` lacks key-provenance for `obj`'s shape (plain runtime
  string, mismatched-shape key) → reject.
- `for..in` over an array, a non-object, or an object of unknown / polymorphic
  shape → reject.
- Writing a **string** into a field via `obj[c] = someString` (fasta never does;
  values are floats) → reject.
- Key provenance used against a *different* object (`otherObj[c]`) → reject.
- `obj[runtimeStrNotFromForIn]` (general dynamic key) → reject (this is Spec 4b).

**Both-sides oracle mirroring (standing series constraint):** every new
expression shape — computed member read/write with a key-provenance index, the
`for..in` key as a string value — gets arms on **both** the codegen recognizers
*and* the `kali_types` predicates in the same change, or it fails open. This has
bitten every spec in the series.

## Deferred inventory (enumerated, not fixed in 4a)

Carried forward from the Spec-3 roll-up and this spec, left **rejecting**:

- The three deferred `join`-receiver families: member/dot receiver
  (`o.arr.join()` — whole member-read construct broken on main, masked by the
  standing throw-is-a-no-op bug), call-result-bound receiver, cross-scope module
  binding. Orthogonal to `for..in`; fasta does not need them (its `join` targets
  are already covered by Spec 3).
- Object **string-valued** fields (store/read of a string into an object field).
  Not needed — fasta's table values are floats.
- `.fill(string)`, inline `join(..).length` over-reject, ternary-of-static-
  literals separator over-reject (Spec-3 minors).
- General runtime string-keyed maps → **Spec 4b**.

The standing **throw-is-a-no-op** bug is noted (it masks broken member-reads)
but is out of scope; 4a introduces no dependence on it.

## Base-behavior invariants (guardrails)

- All CLBG fixtures byte-identical: **nbody, fannkuch-redux, spectral-norm,
  mandelbrot, binary-trees** (numeric + object + arena lanes).
- Static object-fold and numeric `for` loops unchanged.
- The both-walks ordinal fix leaves **every existing loop's arena assignment
  identical** — verified against binary-trees (the arena guardrail).
- No new host imports: the 4 hand-mirrored `kali:rt` JS import lists
  (`kali_runtime/src/browser/harness.rs`; `kali_cli/src/bin/cmd_build.rs`) stay
  byte-identical (`git diff` clean) ([[kali-browser-harness-import-sync]]).
- Strings never dangle: any runtime string allocation (including materialized
  key handles, which are interned constants in a data segment — no runtime
  allocation) never routes through the resettable `__alloc`.

## Testing & validation

Per-task discipline from Specs 1–3: a gate relaxation and its codegen lane land
in the **same task**; both-sides oracle arms in the same change; fail-closed
pins alongside each relaxation.

**Unit / crate level:**
- `kali_types` repr_infer: key-provenance seeding at the `for..in` var,
  propagation through `last = c`, non-propagation across a different object,
  null-sentinel union. Deletion-tested (each provenance arm REDs a pin when
  removed).
- `kali_mir`: both-walks `for..in` recognition — a pin that a real loop
  lexically *after* a `for..in` keeps its arena ordinal (the desync guard),
  REDing if only one walk is taught.
- `kali_codegen`: computed key read/write through `emit_array_element_address`;
  `key_handles` interning; null-sentinel `if (last)`.

**End-to-end (`kali_cli`, `run_source` vs golden):**
- **Capstone (success criterion):** `makeCumulative` + `selectRandom` over a
  hardcoded IUB-style table driven by the real LCG, **byte-for-byte vs `node`**.
  Golden independently re-derived twice (implementer + reviewer), per series
  convention.
- Seam pins: `table[c] += table[last]` cumulative correctness; `return c`
  prints the right nucleotide chars; `if (last)` first-iteration skip.
- Fail-closed pins: one live e2e per row of the fail-closed matrix.
- Regression guardrails: the 5 CLBG fixtures byte-identical; the 4 `kali:rt`
  import lists unchanged.

**Gate:** standing 8-crate set (`kali_lexer kali_common kali_types kali_codegen
kali_cli kali_parser kali_mir kali_hir`) per task, plus per-task
`cargo clippy -p <touched> -- -D warnings` (the Spec-3 process lesson: final-
task-only clippy hid a lint). Final task adds `cargo test --workspace`,
`cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check`.

**Conventions:** conventional-commit messages; commit after every task; the
synthetic top-level function name is `_start` in repr_infer / resolver / codegen.

## Integration

Push a PR and self-merge when CI is green, per the `kali-integration-convention`
memory (`gh` authed as `rahulmutt`; `gh auth setup-git` if git can't read
credentials).

## Out of scope

- Spec 4b (general runtime string-keyed maps), Spec 5 (`process.argv` +
  string→number), Spec 6 (verbatim fasta vendoring + canonical N validation).
- The deferred inventory above (enumerated, left rejecting).
- The throw-is-a-no-op standing bug.
