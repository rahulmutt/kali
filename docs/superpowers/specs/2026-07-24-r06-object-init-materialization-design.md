# R-06 — read-only `var`/`let` object-literal materialization (design)

**Date:** 2026-07-24
**Branch:** `r06-object-init-materialization`
**Oracle:** `node v26.5.0` · **Binary:** `./target/debug/kali`
**Class:** silent miscompile (exit 0, no diagnostic, wrong value)
**Register entry:** R-06 (objects half only — arrays split to a later stage)

## 1. Problem

A read-only `var`/`let` object literal silently reads back `0`:

```js
var o = { f: 7 }; console.log(o.f);   // node: 7   kali: 0   (exit 0, no diagnostic)
```

The bug fires for numeric, string, boolean, and multi-field objects, in **both**
scopes (module and function), and for `let` identically to `var`. `const` is
correct. It is the highest-blast-radius live silent miscompile remaining in the
register: `var o = {…}` is one of the single most common shapes in JS.

Confirmed on a fresh build of `main` (`f4f73fa9b`) — the register was written on
`soundness-batch1-pra` and is stale (most of its Tier-1 entries are now fixed or
fail-closed; R-06 survives). Reproductions this stage targets:

| probe | source | node | kali (today) |
|---|---|---|---|
| `RO_num` | `var o={f:7}; console.log(o.f);` | `7` | `0` |
| `RO_str` | `var o={f:"hi"}; console.log(o.f);` | `hi` | `0` |
| `RO_bool` | `var o={f:true}; console.log(o.f);` | `true` | `0` |
| `RO_mixed` | `var o={n:7,s:"hi"}; console.log(o.n,o.s);` | `7 hi` | `0 0` |
| `RO_multi` | `var o={a:1,b:2,c:3}; console.log(o.a+o.b+o.c);` | `6` | `0` |
| `RO_fn` | `function h(){var o={f:7};return o.f;} console.log(h());` | `7` | `0` |
| `let_num` | `let o={f:7}; console.log(o.f);` | `7` | `0` |

### Already correct today (bounds the fix — do not regress)

- `const o={f:7}` (fold lane) → `7` ✓
- `var o={f:1}; o.f=5` (write materializes) → `5` ✓
- `var o={f:"x"}; o.f="y"` (write, **string** field) → `y` ✓
- `function get(p){return p.f;} var o={f:7}; get(o)` (arg-flow materializes) → `7` ✓
- `function h(){var o={f:7}; o.f=8; function g(){return o.f;} return g();} h()` (closure capture) → `8` ✓

Because the fold lane is **const-only**, a read-only `var`/`let` object literal
**never** works today unless a write or an object-flow (argument pass, closure
capture) already materialized it. Every in-scope case is currently a bug, so this
change can only **improve** a case or convert it to a **fail-closed** error — it
can never regress a currently-correct read-only path. (The one genuine risk is the
escape boundary; see §6.)

## 2. Root cause (verified in source)

An object-literal binding has two correct lowerings, and read-only mutable
bindings fall into the gap between them:

1. **Fold lane** (compile-time). `crates/kali_codegen/src/emit/operators.rs:634`
   resolves the read's base to its literal aggregate via
   `resolve_literal_aggregate` (`crates/kali_codegen/src/emit/literal.rs:45`),
   then folds `o.f` to the field's literal value. `resolve_literal_aggregate`
   resolves identifiers through `self.bindings` (`literal.rs:66`), which holds
   **const-only** initializer aliases (the comments at
   `crates/kali_types/src/resolve/expression.rs:1283` and `:1373` state the
   fold-alias table "only ever aliases `const` bindings"). Sound because `const`
   is immutable — folding a `var`/`let` binding would be the R-07
   "const-is-not-a-binding" miscompile.

2. **Materialized lane** (runtime allocation). `crates/kali_types/src/repr_infer.rs`
   interns a `Repr::Object(shape)` for a binding, and codegen at
   `crates/kali_codegen/src/emit/control_flow.rs:1421` sees that repr and emits a
   real fixed-layout struct via `emit_object_allocation`
   (`crates/kali_codegen/src/emit/object.rs:86`) with the initializer's field
   stores. This lane already supports **int, string, and bool** fields (proven by
   the write case above).

The gap: in `repr_infer.rs` a binding is inserted into `obj_materialized` only by
a **write** access (`repr_infer.rs:4334` and `:4363`) or by the cross-flow
`promote_via_read` conflict path. A plain field **read** does not materialize
(`:4364` else-branch only wires field storage). So a read-only `var`/`let` object:

- is **not foldable** (not `const`, absent from `self.bindings`) — the fold lane
  at `operators.rs:634` finds no aggregate; and
- is **not materialized** (no write) — no `Repr::Object(shape)` is interned,

so the binding keeps its default `Repr::I64` and the field read falls through to
the pre-existing warning-and-`0` fallback → **silent `0`**.

## 3. The fix (one types-side change, zero new codegen)

1. **Record mutable object-literal bindings.** In `repr_infer.rs`
   `visit_declarator_init(func, kind, id, init)` (which already receives `kind`
   and already records every object-literal binding into
   `object_initialized_bindings` at `:2624`), add a new field
   `mutable_object_literal_bindings: BTreeSet<(String, String)>` and insert
   `(func, id)` when `kind != "const"` **and** `init` is an `ObjectExpression`.

2. **Materialize such bindings on read.** In the materialization pass
   (`repr_infer.rs` ~lines 4324–4371, alongside the existing write-materialization
   at `:4330`), for each object-literal **read** access whose base is in
   `mutable_object_literal_bindings`, insert the base into `obj_materialized` —
   exactly what a write already does at `:4363`. `const` bindings are absent from
   the set, so they stay fold-first and their generated code is byte-identical.

Everything downstream is existing, proven machinery, unchanged:

- shape interning gated on `obj_materialized` (`repr_infer.rs:4972`);
- the codegen declarator path (`control_flow.rs:1421`) allocates the struct and
  stores the initializer fields;
- materialized field reads emit a real load instead of the fold-lane fallback.

No new codegen, no new `Repr`, no new deny lists.

## 4. Fail-closed fallback is free

Routing read-only mutable objects through the materialized lane makes them inherit
the lane's **existing** conflict checks, which promote to `E5506` for materialized
slots. Verified against objects already materialized via arg-flow / write:

| shape | probe | result once materialized |
|---|---|---|
| nested-object field | `var o={inner:{x:1}}; …o.inner…` | `E5506` (`repr_infer.rs:4419`) |
| unknown-field read | `var o={f:7}; …o.zzz` | `E5506` (`repr_infer.rs:4352`) |

So a read-only `var` object with a field shape the materialized lane cannot store
goes from **silent `0`** to an **honest `E5506`** with no new deny code. (For the
unknown-field case node yields `undefined`; kali has no `undefined` value — R-21 —
so an honest `E5506` over-deny is the correct outcome, matching the already-shipped
`mat_unknown` behavior.)

## 5. Scope boundary and residuals

In scope (→ materialize, or fail-closed where unsupported):

- read-only `var`/`let` object-literal **declarator** bindings, field-read within
  their own scope, with int / string / bool / multi fields, both scopes.

Out of scope this stage — pre-existing, distinct mechanisms, must be left **no
worse** and documented as residuals:

- **R-06-R1 — returned/escaping objects.** `function h(){ var o={f:7}; return o; }
  h().f` → silent `0` today, **even for `const` and write-materialized objects**
  (the member-on-call placeholder hole, R-14 territory). This stage must not turn
  it into a crash or a new nonzero-wrong value; escape stays
  silent-`0`-or-fail-closed. Its real fix is the R-14 escape stage.
- **R-06-R2 — whole-object reassignment.** `var o={f:1}; o={f:2}; o.f` → `0` today;
  the object-literal-RHS **assignment** store is a distinct mechanism from the
  declarator init. Unchanged this stage.
- **R-06-R3 — arrays.** `var a=[7,9]` reads back `0`, and even
  `var a=[1,2]; a[0]=9` reads back `0 0` — var-array runtime storage is largely
  unimplemented (only the const-fold and growable-`[]`/`.push` lanes exist).
  Entangled with R-12/R-13 and the arena lanes; its own later stage.

## 6. Key risk — escape interaction

The one way materialization could make things *worse*: a newly-materialized object
that escapes (returned, or stored into an escaping structure) must behave
**identically** to today's escaping materialized objects. Those already exist
(`const`/write objects that are returned) and today produce a silent `0` via the
member-on-call hole — **not** a crash and **not** a use-after-arena-reset. The
escape analysis (`escape_flow.rs`, the binary-trees arena lane) keys on a binding
being object-shaped/materialized, which this fix makes uniformly true, so the
newly-admitted bindings take the same path.

Mitigation, baked into the plan:

- an explicit escape-verification task that runs returned / stored-into-escaping /
  captured-and-returned shapes of newly-materialized read-only objects and asserts
  **no new crash and no new nonzero-wrong value** vs `main` (silent-`0` or
  fail-closed only);
- if any escape shape produces a *new* crash or nonzero-miscompile, that shape is
  routed to a fail-closed `E5506` rather than admitted.

## 7. Testing and gate

- **New** `crates/kali_cli/tests/soundness_r06_object_init.rs`:
  - green pins for read-only `var`/`let` objects — int, string, bool, multi-field,
    both scopes (each asserts the program **runs** and prints the node-identical
    value);
  - `E5506` pins — nested-object field, unknown-field read on a read-only mutable
    object;
  - residual pins — escape (`return o` then member-on-call) and whole-object
    reassignment assert **no crash / no new nonzero-miscompile** (they may remain
    silent-`0` or fail closed; they must not regress).
- **Gate:** `cargo test --workspace` (the CI command) diffed against a `main`
  worktree, **0 newly-red**. Budget fixture/census re-pins for read-only `var`/`let`
  objects that switch from silent-`0` to correct or to `E5506`. `cargo fmt --check`
  + `clippy -D warnings` clean. 6/6 CLBG goldens + web-baseline byte-for-byte
  unchanged. Any tag-boxing/synthetic census (`count_tag_boxing_ops` allowlists)
  re-checked additively per the established procedure.
- **Standing discipline:** re-run every reproducer on a freshly built binary
  (fix reports are unreliable); full enumeration is the only gate; verify the whole
  change with an adversarial whole-stage review, which has caught a store-site /
  escape / value-sink fail-open per-task reviews missed on every prior stage.

## 8. Interfaces produced

- `repr_infer.rs`: new `mutable_object_literal_bindings` set + its one insertion
  site in `visit_declarator_init` + one read-materialization site in the
  materialization pass. No `ReprTable` surface change (materialization feeds the
  existing `Repr::Object` interning).
- `soundness_r06_object_init.rs`; any re-pinned census/soundness fixtures.
- Register update: R-06 objects-half CLOSED (materialize + inherited fail-closed);
  R-06-R1/R2/R3 residuals recorded.

## References

- Register: `docs/superpowers/followups/kali-silent-miscompile-register.md`
  (R-06, §3 cluster G7, §6 Group-1/Group-4 ordering — this stage falsifies G7's
  "R-06 falls out of the R-07 fix" inference: R-07 is fixed, R-06 still reproduces,
  so R-06 is independent).
- Fold-vs-binding soundness law: R-07 (`const` is not a binding) — why the fold
  lane must stay const-only.
- Allowlist-at-choke discipline (materialize the provably-safe case, fail closed on
  the rest): `[[kali-forin-spec4a]]`, `[[kali-throw-fallout-stage5]]`,
  `[[kali-g6-unimplemented-builtin-failclosed]]`.
- Escape/arena lane the risk section leans on: `[[kali-binary-trees-phase1]]`,
  `[[kali-interprocedural-escape-flow]]`.
