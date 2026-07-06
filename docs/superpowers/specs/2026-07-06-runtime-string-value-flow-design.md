# Runtime string value flow (fix E3200) — design

**Date:** 2026-07-06
**Status:** Approved (design)
**Series:** Runtime strings & dynamic tables for verbatim fasta — **Spec 1 of 6**

## Series context

The next CLBG target is **fasta**. The user chose to compile the *verbatim
upstream solution* (Ian Osgood's `fasta-node-1` Node.js program) rather than a
byte-buffer normalization. That program is string-native — it relies on
`substring`, string `+` on runtime strings, `Array.prototype.join`, `console.log`
of runtime strings, and object tables enumerated with `for..in` where the
enumerated key is used as a runtime property index (`table[c] += table[last]`).

An investigation of Kali's compiled path found that a **runtime-string value
model already exists**: tagged linear-memory string handles
(`STRING_HANDLE_TAG | offset << 32 | len`, `kali_codegen`), host-side
`alloc_guest_string` / `string_concat` / `int_to_string` / `float_to_fixed`
(`kali_runtime`), and `console.log` of runtime handles (that is how `toFixed`
prints). The `+` concatenation path is already lowered in codegen. The blockers
`E3200` / `E5506` / `E4201` are **front-end gates in `kali_types`** that refuse
to let runtime strings flow from variables, substrings, joins, and dynamic keys
— deliberately, to avoid miscompiles on paths that were never fully wired.

So "verbatim upstream" is **extend the existing handle model to cover four more
sources and relax the matching gates**, decomposed into a dependency-ordered
series (each its own spec → plan → implementation cycle):

1. **Spec 1 (this doc) — String-typed value flow (fix `E3200`).** The linchpin:
   let string-typed variables / params / returns carry their handle through the
   value model so `+` and `console.log` accept them.
2. **Spec 2 — `substring` runtime (relax `E5506` for substring).** Emit a
   runtime slice handle (a contiguous slice is a re-tagged `offset+len`, no
   copy). Replaces the ALU-`charCodeAt` idea from the original single-spec
   design; upstream uses `substring`.
3. **Spec 3 — `Array.prototype.join(sep)` over string elements.** Host import
   concatenating element handles with a separator.
4. **Spec 4 — `for..in` live-key + dynamic string-keyed property get/set (fix
   `E4201`).** Object-model lane for `makeCumulative`. Hardest / most
   orthogonal; may itself split.
5. **Spec 5 — `process.argv` runtime element read + string→number coercion.**
   `process.argv.length` / `.slice(n).length` and the `process_args_get` host
   buffer already exist (on the `--api node` surface), but reading an *element*
   `process.argv[i]` as a runtime **string handle** yields 0 today, and there is
   no runtime `+str` / `Number(str)` / `parseInt` parse (only static const-fold).
   Both build on Spec 1's string-value-flow foundation. Lets the capstone read
   `+process.argv[2]` verbatim instead of pinning `n`.
6. **Spec 6 — fasta fixture + two-tier validation (capstone).** Vendor the
   upstream program **verbatim** (`n = +process.argv[2]`, enabled by Spec 5).
   Small-N golden (byte-for-byte) + canonical N=25,000,000 SHA-256 vs a
   `node`-computed reference.

`console.log` of runtime strings needs no new work — it falls out of Spec 1.

## Problem (Spec 1)

`is_float_valued` (codegen) consults a per-binding **repr** — the `ReprTable` in
`kali_common/src/repr.rs`, whose axis is `I64 | F64 | Object(ShapeId)` — so
codegen knows a local / param / return holds an `f64`. `is_string_valued` has no
such backing; it only pattern-matches literal-rooted `+` expressions:

```rust
// kali_codegen/src/emit/operators.rs — is_string_valued (today)
Literal => looks like a quoted string,
Value("+") with 2 children => either child is_string_valued,
_ => false,   // <-- a string-typed *variable* is invisible
```

Because codegen cannot see that a variable holds a string handle, a
string-typed variable operand of `+` would be miscompiled (integer-add two
handles, or coerce a handle through `int_to_string`). `kali_types` therefore
rejects it up front:

```
error[E3200]: '+' with a string-typed variable operand is unavailable in the
current direct-runtime path ...
```

emitted by `reject_unsupported_string_variable_addition`
(`kali_types/src/resolve/expression.rs`), which fires when an operand is
*string-typed* but not *codegen-recognized structural string* (literal-rooted).

The runtime plumbing to handle a string variable correctly already exists
(`STRING_CONCAT_IMPORT_INDEX`, handle-printing `console.log`). Only the
*shape/repr tracking* is missing.

## Approach

Add a **`String` axis to the repr model**, mirroring the existing `F64` axis end
to end. A string handle already fits the i64 local slot it occupies — the only
missing piece is telling codegen which bindings hold one.

### 1. `kali_common` — repr axis

- Add `Repr::String` to the `Repr` enum.
- `ReprTable`'s existing `scalars` / `params` / `returns` maps carry it with no
  structural change.
- Add an `any_string` flag mirroring `any_float`, so the empty-table
  "no strings anywhere" fast path is preserved and non-string programs are
  untouched.

### 2. `kali_types` — repr inference

- **Seed** `String` for string-typed bindings: string literal, template
  literal, `+` rooted in a string, and the results of intrinsics that already
  produce string handles (`toFixed`, `int_to_string`).
- **Unify** across assignment, params, and returns using the same seed-and-unify
  machinery the float axis uses.
- A binding unified with both `String` and a numeric repr is a **conflict** —
  reject cleanly (a diagnostic), never silently pick one. This mirrors how the
  float axis treats representation conflicts.

### 3. `kali_codegen` — consult the axis

- `is_string_valued(id)` consults `repr.scalar / param / return(...) ==
  Repr::String` for identifiers and calls, mirroring `is_float_valued`.
- With that, string-typed locals, params, and call-returns flow their handle
  through the already-lowered `+` (`string_concat`) and the already-lowered
  `console.log` handle-print path. No new codegen instructions.

### 4. Relax `E3200`

- `reject_unsupported_string_variable_addition` fires only when an operand is
  string-typed **and** its repr is *not* `String` — i.e. genuinely-unsupported
  sources whose repr axis isn't wired yet (substring/join/dynamic-key results,
  reached in later specs). The rejection stays as a soundness backstop; it is
  narrowed, not deleted.

## Data flow

```
string literal / template / toFixed / int_to_string
        │  (seed String)
        ▼
kali_types repr inference ── unify across assign / param / return ──► ReprTable{String}
        │                                                                   │
        │ (conflict with numeric ⇒ reject)                                  │ threaded to codegen
        ▼                                                                   ▼
E3200 fires only if string-typed AND repr≠String            is_string_valued consults repr
                                                                            │
                                                                            ▼
                                              already-lowered string_concat (+) / handle-print (console.log)
```

## Testing (TDD)

Write tests first; each must fail before the change and pass after.

- **String variable concat + print:** `let x="GG"; x=x+"CC"; console.log(x)`
  → `GGCC`.
- **String param round-trip:** `function f(s){ return s + "!"; } console.log(f("hi"))`
  → `hi!`.
- **String return consumed by caller:** callee returns a string handle, caller
  concatenates it.
- **Accumulation loop (the fasta line-building shape):**
  `let a=""; for (let i=0;i<3;i=i+1){ a = a + "y"; } console.log(a)` → `yyy`.
- **Conflict guard (soundness):** a binding used as both string and number is
  rejected with a diagnostic, not miscompiled.
- **No regressions:** the full existing suite stays green — literal-rooted
  concatenation (`"x" + 3`), the float axis, and the `any_string` fast path for
  non-string programs are all unaffected.

## Scope / non-goals

- **In scope:** scalar string bindings, params, and returns flowing through `+`
  and `console.log`.
- **Out of scope (later specs, still gated):** `substring` results (Spec 2),
  `Array.prototype.join` (Spec 3), `for..in` dynamic string keys (Spec 4). Their
  repr sources are not wired here, so `E5506` / `E4201` and the narrowed `E3200`
  still correctly fire for them.

## Risks

- **Union-typed bindings:** must reject on a string/number conflict rather than
  silently choosing a repr — same discipline as the float axis.
- **Gate narrowing must not fail-open:** the `E3200` relaxation is keyed on a
  *proven* `Repr::String`, so anything the inference cannot prove still rejects.
- **Regression surface:** the float axis and literal-rooted concat share the
  inference/codegen paths being touched; the existing suite is the guardrail.
