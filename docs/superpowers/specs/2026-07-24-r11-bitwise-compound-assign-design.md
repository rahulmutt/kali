# R-11 design — bitwise compound assignment (`&= |= ^= <<= >>= >>>=`)

**Date:** 2026-07-24
**Branch:** `r11-bitwise-compound-assign` (off `main` `62d786e74`)
**Register:** `docs/superpowers/followups/kali-silent-miscompile-register.md` — R-11
(Group-2 #5), re-derived this same day (§0): still **48/48 silent**.

## 1. Goal

Make the six bitwise compound-assignment operators — `&=`, `|=`, `^=`, `<<=`,
`>>=`, `>>>=` — either compute the correct value (integer targets) or fail closed
with an honest `E5506` (every other target), on **every** assignment target kind.
Today all six are **silent no-ops** on all target kinds: the assignment expression
evaluates to the unmodified current value of the target and no write-back happens,
exit 0, no diagnostic (`let n=6; n<<=2` → `6`, node `24`).

This retires R-11 from the silent-miscompile register.

## 2. Non-goals

- **No new numeric semantics.** The plain binary bitwise operators
  (`& | ^ << >> >>>`) already lower with full JS 32-bit semantics and are verified
  byte-for-byte against node, including every int32 edge case (`1<<31` → sign flip,
  `1<<32` → identity via shift-count mod 32, `4294967296<<1` → 0 via int32
  truncation, `-1>>>0` → `4294967295` unsigned). This stage reuses that lowering
  verbatim; it does not touch or re-derive it.
- **No new target-kind capability.** Where the arithmetic sibling `+=` fails closed
  (e.g. an aliased array element, a computed member with no proven shape, a growable
  field), the bitwise form fails closed with the *same* `E5506`. R-11 does not add
  storage lanes; it only ensures the bitwise form matches its arithmetic sibling.
- **No bitwise on floats.** `emit_binary` already rejects bitwise operators on a
  float operand (`operators.rs:2094`). A bitwise compound on an f64-repr target or
  RHS fails closed the same way — not `ToInt32`-coerced. (JS would coerce; kali's
  standing policy is to refuse rather than silently narrow a float.)
- **No logical/nullish compounds.** `&&= ||= ??=` are out of scope (already handled
  or fail-closed) and are untouched.
- **No `-0`/`NaN`/BigInt work.** Out of scope (R-28 / R-45 / their own lanes).

## 3. Mechanism (traced, on `62d786e74`)

### 3.1 The single upstream choke

`emit_assignment` (`crates/kali_codegen/src/emit/literal.rs:213`) opens with an
**allowlist gate** (`literal.rs:222-227`):

```rust
if !matches!(op, "=" | "??=" | "&&=" | "||=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**=") {
    return false;
}
```

The six bitwise compound ops are **absent from this allowlist**, so `emit_assignment`
returns `false` for them **before any per-target arm runs** — for every target kind
(bare local, module global, env cell, object field, array element, computed member).
That single gate is why the register's re-derived matrix is uniform 48/48: none of
the downstream arms are ever reached.

The caller (`crates/kali_codegen/src/emit/operators.rs:1757`):

```rust
if self.emit_assignment(function, id, node, op, left, right) { /* handled */ }
```

On `false` the assignment node falls through to generic node emission, which emits a
bare **read** of the target — the observed "returns the unmodified operand, no
write-back". **That is the fail-open** this stage closes.

### 3.2 The reuse target

`emit_bitwise` (`crates/kali_codegen/src/emit/operators.rs:1610`) is the sole plain
bitwise lowering: it `emit_float_operand`s each operand (rejecting floats),
`I32WrapI64`s each (`ToInt32`), runs the i32 op (wasm masks shift counts mod 32,
matching JS `& 31`), then extends back to i64 — `I64ExtendI32S` for every op except
`>>>`, which `I64ExtendI32U` (uint32). The plain-op dispatch routes
`& | ^ << >> >>>` here (`operators.rs:2433`).

### 3.3 The downstream target sub-paths a compound op reaches

Once admitted at the gate, a compound op flows through `emit_assignment`'s existing
target dispatch. The arms an integer bitwise compound can structurally reach, each
of which must gain a bitwise case that mirrors its `+=` sibling (lower for integer
targets, `E5506` otherwise):

| target | dispatch site | `+=` behavior to mirror |
|---|---|---|
| fixed-shape object field (`o.f op= v`) | `emit_object_field_compound_assign_dynamic` (`object.rs:340`, called at `literal.rs:704`) | lowers for int field; f64/growable → E5506 |
| for-in-key / computed object field (`o[k] op= v`) | `emit_object_field_write_dynamic` path + `object.rs:377/418` | lowers for int field; f64 → E5506 |
| module global (`g op= v`) | `emit_module_global_assignment` (`literal.rs:1049`) | lowers for i64 global; f64 → E5506 |
| captured env cell | `try_emit_captured_assign` (`literal.rs:782`) | read-modify-write for int cell |
| local / param scalar (`n op= v`) | the `match op` at `literal.rs:808`, currently `_ => false` at `:1035` | lowers for i64 local; f64/String → E5506 |
| array element / computed index | array-write path (below the object-field arms) | mirror `+=` exactly: lower on the admitted element lane (e.g. `new Array(n)`), E5506 on the aliased (R-12) / unproven-shape (R-13) lanes |

## 4. Design (approved: reuse `emit_bitwise` + parity + default-deny)

### 4.1 Shared int32 combiner (single oracle)

Refactor `emit_bitwise` so its op-select + extend **tail** becomes a private helper:

```rust
/// Consumes two i32 on the value stack (left, then right already pushed),
/// applies the bitwise op, and extends the i32 result back to i64 (signed,
/// or unsigned for `>>>`). The ONLY place JS bitwise result semantics live.
fn emit_bitwise_i32_op_extend(&mut self, function: &mut Function, op: &str) { … }
```

`emit_bitwise` keeps its current signature and behavior — it pushes+wraps both
operands, then calls the helper — so the plain-operator lowering is **behavior-
neutral** (pinned before/after). The compound arms call the same helper after
pushing their own operands, so int32 coercion and the signed/unsigned extend can
never desync between the plain and compound forms (the hand-mirrored-oracle hazard
the register flags for R-16).

The compound read-modify-write shape at every integer target is:

```
<push current target value : i64>   ; LocalGet / GlobalGet / field load / env load
I32WrapI64                          ; ToInt32(target)
<emit RHS via emit_float_operand>   ; rejects a float RHS (fail-closed)
I32WrapI64                          ; ToInt32(rhs)
emit_bitwise_i32_op_extend(op)      ; shared tail → i64 result
<store back + leave value on stack> ; LocalTee / GlobalSet+Get / field store+reload
```

### 4.2 Admit at the gate, then handle-or-deny at every arm

1. Add the six bitwise ops to the `literal.rs:222` allowlist so `emit_assignment`
   stops short-circuiting them to `false`.
2. Add a bitwise case to each downstream target arm in the table above. Integer
   target → the RMW sequence in §4.1. Non-integer target (f64/String) or a target
   whose `+=` sibling fails closed → the **same `E5506`** that sibling emits.
3. Convert the local-scalar `_ => false` default (`literal.rs:1035`) to a
   fail-closed `E5506`. It is currently reachable only by these six ops (every other
   compound/`=`/`??=`/`&&=`/`||=` has an explicit arm), so this is a pure
   default-deny hardening: after this stage no compound op can return `false` from a
   local target and re-trigger the bare-read fail-open.

### 4.3 Why this is correct by construction

- Result semantics are single-sourced in `emit_bitwise_i32_op_extend`, shared with
  the plain operators that are already proven against node.
- Every target arm either lowers (integer) or emits the arithmetic sibling's own
  `E5506` — an **allowlist of admitted targets**, never a denylist of shapes.
- The gate at `literal.rs:222` stays an allowlist; the only fail-open in the whole
  path (the `_ => false` bare-read fall-through) is removed.

## 5. Error handling / fail-closed surface

Fail closed with `E5506` (never silent, never a new internal `E4201`) for:

- a bitwise compound on an **f64-repr** target or f64 RHS (parity with
  `emit_binary`'s float rejection and the arithmetic f64 arm's `%=`/`**=` refusals);
- a bitwise compound on a **String-repr** target (parity with the `+=` string arm's
  refusal of non-`+=` ops);
- a bitwise compound on any target whose `+=` sibling fails closed — aliased array
  element (R-12 lane), computed member with no proven shape (R-13 lane),
  growable-array field/element, unknown fixed-shape field (parity: wherever `+=`
  lowers on an admitted element/field lane, the bitwise form lowers too);
- any residual op/target combination the arms do not explicitly admit (the new
  default-deny arm).

## 6. Testing

New pin file `crates/kali_cli/tests/soundness_bitwise_compound.rs` (mirrors the
existing `soundness_*` pattern). All reproducers re-run on a **freshly built**
binary (fix reports are unreliable — standing discipline).

**Correctness (MATCH node), both module and function scope:**
- All six ops on a `let`/`var` local and a `let` re-read, from a known start, against
  the node-computed value (the §0 matrix row: `&=3`→2, `|=8`→14, `^=1`→7, `<<=2`→24,
  `>>=1`→3, `>>>=1`→3 from 6).
- int32 edges: `x=1; x<<=31` → `-2147483648`; `x=1; x<<=32` → `1`; `x=-8; x>>=1` →
  `-4`; `x=-1; x>>>=0` → `4294967295` (uint32 **round-trips** through the slot and
  reads back correctly — the I64ExtendI32U path); chained `x<<=2; x|=1`.
- module-global integer target (a promoted mutable module scalar written across
  functions) for at least `<<=`, `&=`, `>>>=`.
- integer object field (`o.f op= v` on a fixed-shape int field) — at least `<<=`,
  `|=`.

**Fail-closed (E5506, exit 1) — honest refusal, pinned as such:**
- f64 target (`let x=1.5; x<<=1`), f64 RHS (`let n=1; n<<=1.5`);
- String target (`let s="a"; s<<=1`);
- aliased array element, computed member with no shape, growable field —
  parity with the `+=` sibling's existing `E5506` at each.

**Reference (unchanged):** plain binary bitwise ops (`6&3`, `6<<2`, `-1>>>0`, …)
still MATCH — proves the `emit_bitwise` refactor is behavior-neutral.

**Gate:** `cargo test --workspace` (the CI command) diffed against a `main`
worktree, **0 newly-red**. `cargo fmt --check` + `clippy -D warnings` clean. 6/6
CLBG goldens + web-baseline byte-for-byte unchanged. Any tag-boxing/synthetic
census (`count_tag_boxing_ops` allowlists) re-checked additively per the
established procedure (this stage adds no synthetic functions or imports, so the
census should be untouched — verified, not assumed).

## 7. Risks

- **Multiple hand-mirrored target arms (the recurring lesson).** The gate admits all
  six ops uniformly, but each downstream target arm is a separate site that must
  gain a bitwise case or a fail-closed default. Missing one re-opens a silent
  no-op *or* (worse) an `I64Store` of a wrong-width value. Mitigation: the §3.3
  table is the enumeration checklist; the whole-stage adversarial review (which has
  caught a store-site/value-sink/escape fail-open on **every** prior stage) must
  walk every arm and every RHS shape, and the fail-closed pins in §6 assert the
  refusals actually fire.
- **`emit_bitwise` refactor must stay behavior-neutral.** Pin plain-operator outputs
  before and after the extract; the reference tests in §6 guard this.
- **`>>>=` uint32 > i32-positive range.** A value like `4294967295` stored in the
  target slot and re-read must survive. It does via `I64ExtendI32U` (zero-extend into
  the i64 slot); explicitly pinned.
- **Env-cell / captured target.** `try_emit_captured_assign` is a distinct RMW path;
  if it does not cleanly admit a bitwise case this stage, it must fail closed rather
  than fall through — verify it returns `Some(handled)` for the bitwise ops, never
  `None`-into-fall-through.

## 8. Interfaces produced

- `operators.rs`: `emit_bitwise` split into a pusher + the new
  `emit_bitwise_i32_op_extend` helper (behavior-neutral for plain ops).
- `literal.rs`: six ops added to the `emit_assignment` allowlist gate; bitwise arms
  in the local-scalar match (+ `_ => false` → `E5506` default-deny) and the
  module-global arm; env-cell path admits-or-denies.
- `object.rs`: bitwise arms in `emit_object_field_compound_assign_dynamic` and the
  computed/for-in field write path.
- `crates/kali_cli/tests/soundness_bitwise_compound.rs`: correctness + fail-closed
  pins.
- Register update: R-11 CLOSED (lower-or-fail-closed at every target site).

## References

- Register `§0` re-derivation (2026-07-24) — R-11 48/48 matrix and the
  "fix is purely the write-back" finding.
- Plain bitwise lowering / JS 32-bit semantics: `emit_bitwise`
  (`operators.rs:1610`), verified against node in the mandelbrot lane
  (`[[kali-bitwise-and-binary-stdout-lane]]`).
- Allowlist-at-choke, fail-closed-default, single-oracle discipline (the
  hand-mirrored-oracle hazard): `[[kali-forin-spec4a]]`,
  `[[kali-g6-unimplemented-builtin-failclosed]]`, R-16's own warning in the
  register.
- Fold-vs-binding / parity-with-arithmetic-sibling framing: R-06 objects-half
  (`[[kali-r06-object-init]]`), R-07.
