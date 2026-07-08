# fasta output layer + `process.argv`/N (Spec 5) — design

**Date:** 2026-07-08
**Status:** Approved (design)
**Series:** Runtime strings & dynamic tables for verbatim fasta — **Spec 5 of 6**

## Series context

The CLBG target is **fasta**, compiled from the *verbatim upstream* Node.js
program (Ian Osgood's `fasta-node-1`). Specs 1–4a shipped the string/table core:

- Spec 1 (PR #9) — string-typed value flow (fix `E3200`).
- Spec 2 (PR #10) — runtime `substring` / `.length` (relax `E5506`), incl.
  one-arg `substring(i)` and two-substring `+` concat (the wrap-boundary shape).
- Spec 3 (PR #11) — `Array.prototype.join(sep)` over string-element arrays,
  `new Array(n)` + `line[i] = …` element stores, array reassignment.
- Spec 4a (PR #12) — fixed-shape `for..in` + computed string-keyed get/set;
  `makeCumulative` + `selectRandom` byte-for-byte; mutable module-scope scalar
  globals (the LCG `rand` state).

With those, most of fasta's remaining surface is **assembled from shipped
primitives**. This spec closes the last engineering gap: the two **output
functions** (`repeatFasta`, `randomFasta`) and the **`process.argv[2]` N
argument**. After this, Spec 6 is validation-only.

**Original roadmap folded here.** The prior series doc split the tail into
Spec 5 (`process.argv` + string→number) and Spec 6 (verbatim vendoring + N
validation). The user chose to pull argv/N **into this spec** so Spec 5 delivers
an end-to-end fasta shell that reads N from the CLI. Spec 6 collapses to: vendor
the upstream file **verbatim** + the canonical **N=25,000,000 SHA-256**
two-tier validation.

## The target program (fasta-node-1, output layer)

Output is `console.log`-per-60-column-line — upstream `print` → `console.log`,
each call appends the `\n`. There is **no** byte-buffer writer.

```js
function fastaRepeat(n, seq) {                 // ">ONE" section
  var seqi = 0, lenOut = 60;
  while (n > 0) {
    if (n < lenOut) lenOut = n;
    if (seqi + lenOut < seq.length) {
      console.log(seq.substring(seqi, seqi + lenOut));
      seqi += lenOut;
    } else {
      console.log(seq.substring(seqi) +
                  seq.substring(0, lenOut - (seq.length - seqi)));
      seqi = lenOut - (seq.length - seqi);
    }
    n -= lenOut;
  }
}

function fastaRandom(n, table) {               // ">TWO"/">THREE" sections
  var line = new Array(60);
  makeCumulative(table);
  while (n > 0) {
    if (n < line.length) line = new Array(n);  // partial final line
    for (var i = 0; i < line.length; i++) {
      var r = rand(1);
      for (var c in table)
        if (r < table[c]) break;               // <-- break out of for..in
      line[i] = c;                             // <-- key `c` live AFTER the loop
    }
    console.log(line.join(''));
    n -= line.length;
  }
}

var n = +process.argv[2];                       // <-- runtime argv + coercion
console.log(">ONE Homo sapiens alu");
fastaRepeat(2 * n, ALU);
console.log(">TWO IUB ambiguity codes");
fastaRandom(3 * n, IUB);
console.log(">THREE Homo sapiens frequency");
fastaRandom(5 * n, HomoSap);
```

`ALU` is a long multi-line `"…" + "…" + …` static-concat string literal; `IUB`
and `HomoSap` are the fixed-shape float tables already handled by Spec 4a.

## What is already shipped (reused, no new work)

- One-arg `substring(i)` (to end) and two-arg `substring(a, b)` — Spec 2; the
  else-branch `seq.substring(seqi) + seq.substring(0, …)` is proven byte-for-byte
  at `runtime_substring_length.rs:45`.
- `new Array(n)`, `line[i] = <value>` element store, `line[i] = <for-in key>`
  (the `collect` test, `runtime_forin.rs:196`), `.join('')` — Spec 3.
- Per-line `console.log` of runtime strings — Spec 1.
- `for..in` counted loop, computed `table[c]`, `makeCumulative` cumulative sum —
  Spec 4a.
- Mutable module-scope scalar globals for the LCG `rand` (`last/A/C/M`) —
  Spec 4a.
- `.length` on strings and arrays; `process.argv.length` /
  `process.argv.slice(n).length` (lowers to the `args_length` import).
- The `args_get` host import (writes an argument's bytes into a guest buffer)
  already exists on the `--api node` surface.

## The four new primitives

### N1 — `break` out of a `for..in`, key `c` live after the loop *(crux)*

`for (var c in table) if (r < table[c]) break; line[i] = c;`

Spec 4a lowers `for..in` as a counted loop over ordinals `0..N-1` with a
dedicated scratch local. This spec must:

1. **Accept a direct `break` targeting the `for..in` loop.** Today
   `emit_break_or_continue` targets the innermost open loop frame; the `for..in`
   lowering must push a loop frame with a valid `break_index` so an unlabeled
   `break` inside it lands after the loop (existing tests only nest `break` in an
   *inner* while/for, never break the for-in itself — untested today).
2. **Keep the key binding live past loop exit.** After the loop, `c` holds the
   last-visited key — as an i64 ordinal at index/truthiness sites and as a
   **materialized String handle** at `line[i] = c`. The per-shape interned handle
   table (Spec 4a) is allocated in the preheader and persists; the ordinal scratch
   local retains its last value on the break path. `line[i] = c` materializes
   `c`'s handle exactly as `return c` did in `selectRandom`.

**Escape-invariant (Spec 4a headline lesson, extended).** The for-in key's raw
ordinal must not leak as a value. `line[i] = c` is a *post-loop* materialization
site. The allowlist at the `resolve_identifier` choke point must admit "for-in
key read after its loop body, repr==String, at a string-array element store"
**without** fail-opening the ordinal at any other post-loop position. Allowlist
the safe position at the single read site — do **not** denylist sinks.

### N2 — array-var reassignment to `new Array(n)` mid-loop

`if (n < line.length) line = new Array(n);` — reallocates `line` to a shorter
array for the partial final line. Spec 3 shipped string-array reassignment;
confirm it covers realloc-to-shorter inside a `while` and that `line.length`,
`line[i] = …`, and `line.join('')` all track the new binding. Close any gap
(e.g. length re-read after reassignment) in the same task.

### N3 — `process.argv[i]` element read → runtime string handle

`process.argv[2]` must materialize a **runtime String handle** (today it yields
`0`). Lower an element index on a `process.argv` receiver to: fetch the arg's
bytes via the existing `args_get` host buffer, intern them into a persistent
string handle (`StringPool` / `__alloc_global`, never the resettable `__alloc`),
and yield the tagged handle. Reuses Spec 1's string-value flow for everything
downstream. Index must be a provable in-range integer; unprovable → reject.

### N4 — runtime string→number coercion `+process.argv[2]`

Unary `+` on a runtime string → number. `n = +process.argv[2]` is the verbatim
form. Lower the unary-plus of a runtime-string-typed operand to a
string→number parse producing an f64 (fasta multiplies `2*n`/`3*n`/`5*n` and
compares; the value flows numeric). Either a small host parse import or an inline
digit-parse loop. `parseInt` is folded in **only if cheap** (same parse, integer
truncation); `Number()` and radix semantics are out of scope. Coercion of a
runtime string that is not provably numeric → reject (do not miscompile to 0).

## Fail-closed matrix (reject, don't miscompile — `E5506`)

- `break` out of a `for..in` where the key is used at a non-materializable
  position after the loop → reject.
- `line[i] = c` where the element array is not a provable string-element array,
  or `c`'s repr is not String at the store → reject.
- `process.argv[i]` with a non-provably-in-range / non-integer index → reject.
- Unary `+` / coercion of a runtime string not provably numeric → reject.
- Array reassignment to a non-`new Array(n)` producer, or mixing element reprs
  across the reassignment → reject.

**Both-sides oracle mirroring (standing series constraint).** Every new
expression shape — the post-loop for-in-key materialization, the `argv[i]`
string-handle read, the unary-plus coercion — gets arms on **both** the codegen
recognizers *and* the four `kali_types` predicates
(`expression_is_string_typed`, `operand_repr_is_string`,
`expression_is_length_fold_receiver`, `expression_is_runtime_string_value`,
`crates/kali_types/src/resolve/expression.rs`) in the **same change**, or it
fails open.

## Base-behavior invariants (guardrails)

- All 5 CLBG fixtures byte-identical: **nbody, fannkuch-redux, spectral-norm,
  mandelbrot, binary-trees**. binary-trees remains the both-walks arena
  guardrail; the `for..in` break must not perturb any existing loop's arena
  ordinal assignment (kali_mir / kali_codegen walks stay in lockstep, still
  skipping `for..in` for arenas).
- Static object-fold and numeric loops unchanged.
- No new host imports beyond the already-present `args_get` / `args_length` node
  surface: the 4 hand-mirrored `kali:rt` JS import lists
  (`kali_runtime/src/browser/harness.rs`; `kali_cli/src/bin/cmd_build.rs`) stay
  byte-identical (`git diff` clean) ([[kali-browser-harness-import-sync]]). If
  N4 needs a string→number host import, it lands on the node surface with all 4
  lists updated in lockstep and documented.
- Strings never dangle: argv handles and materialized key handles route through
  `StringPool` / `__alloc_global`, never the resettable `__alloc`
  ([[kali-reclaiming-allocator-phase0]]).
- GC-less invariant preserved ([[kali-gc-less-invariant]]): no runtime
  string→value map; the `line` array and argv handles are region/persistent
  allocations, not GC roots.

## Testing & validation

Per-task discipline from Specs 1–4a: a gate relaxation and its codegen lane land
in the **same task**; both-sides oracle arms in the same change; fail-closed
pins alongside each relaxation; deletion-tested provenance arms.

**Unit / crate level:**
- `kali_codegen` / `kali_types`: `break` out of a `for..in` with the key read
  after the loop (ordinal-site and materialized-String-site); `argv[i]`
  string-handle read; unary-plus coercion of a runtime string.
- `kali_mir`: a real loop lexically after a `for..in` containing a `break` keeps
  its arena ordinal (the desync guard, REDing if one walk is taught).

**End-to-end (`kali_cli`, `run_source` vs node v26.4.0 golden):**
- `fastaRepeat` shell byte-for-byte at a fixed N (both branches: mid-string and
  wrap-boundary lines).
- `fastaRandom` shell byte-for-byte (line array + `break`-selected char +
  `join('')` + partial final line).
- **Capstone (success criterion):** the full fasta shell — all three sections
  (`fastaRepeat(2n, ALU)`, `fastaRandom(3n, IUB)`, `fastaRandom(5n, HomoSap)`),
  headers, `n = +process.argv[2]` — reading N from a CLI argument, **byte-for-byte
  vs `node`** at a small N. Golden independently re-derived twice (implementer +
  reviewer), per series convention.
- Fail-closed pins: one live e2e per row of the fail-closed matrix.
- Regression guardrails: the 5 CLBG fixtures byte-identical; the 4 `kali:rt`
  import lists unchanged.

**Gate:** standing 8-crate set (`kali_lexer kali_common kali_types kali_codegen
kali_cli kali_parser kali_mir kali_hir`) per task, plus per-task
`cargo clippy -p <touched> -- -D warnings`. Final task adds
`cargo test --workspace`, `cargo clippy --workspace -- -D warnings`,
`cargo fmt --all -- --check`.

**Conventions:** conventional-commit messages; commit after every task; the
synthetic top-level function name is `_start`.

## Integration

Push a PR and self-merge when CI is green, per the `kali-integration-convention`
memory (`gh` authed as `rahulmutt`; `gh auth setup-git` if git can't read
credentials).

## Out of scope

- **Spec 6** (validation-only): vendor the upstream `fasta-node-1` file
  **verbatim** + the canonical **N=25,000,000 SHA-256** vs a node-computed
  reference (small-N golden already covered here).
- **Spec 4b** (general runtime string-keyed maps) — fasta does not need it.
- `Number()` / `parseInt` radix semantics beyond what unary `+` requires.
- General runtime array element reprs beyond string / uniform-float; the
  Spec-3/4a deferred inventory, left rejecting.
- The throw-is-a-no-op standing bug (out of scope; no new dependence).
