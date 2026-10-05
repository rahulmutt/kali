# A closure reads the binding it captured, or kali refuses

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `2ddf18c66` (`main`, the block-scoping merge) |
| kali binary | `kali 0.1.0`, `target/debug/kali` (`dev` profile), built 2026-10-05 00:28 UTC from the `block-scoping` branch. The two commits after the build (`0b6ebd6b8`, `871876302`) change tests and docs only. Task 0 rebuilds at the baseline and re-measures every row of §2 before anything else lands. |
| oracle | `node v26.10.0` |
| measured on | 2026-10-05 |
| item picked | `docs/superpowers/followups/block-scoping-discovered-defects.md` §7.11 item 1 ("A captured parameter, or a `let` initialized from one, reads `0` in a closure") |
| defects this closes | §7.11 item 1 (real for the §1 slice, refused outside it); §7.11 item 2, the depth-2 synchronous capture (refused, §3.1); the unpromoted-capture lanes measured by this project (§2.2, §2.3), none filed before |

**Scope was chosen by the human partner:**

* **Item:** §7.11 item 1, over the depth-2 lowering (§7.11 item 2 with
  block-scoping §4), cross-module named imports (§7.11 item 3), and
  boolean / `undefined` rendering (§7.1, §7.2).
* **Depth:** fail closed, then real, in one project (option 1 of three; the
  others were "real" and "fail closed only").
* **Slice:** `i64` parameters including writes from either side, and F64 and
  boolean captures for parameters and locals. Strings and Object parameters
  stay refused (the human partner took the recommended set and added F64 and
  boolean).
* **`check`:** mirrors the refusals where cheap (option 1 of two; the other was
  "run-only"). Any residue is recorded as a `check` / `run` gap.
* **Mechanism:** approach A of three, an AST rewrite of a captured parameter
  into a `let` initialized from a renamed parameter (§4 records B and C).
* **Depth-2:** the phase-1 deny cannot tell an unpromotable repr from an
  unprovable depth, so it refuses §7.11 item 2 as well. The human partner
  approved that explicitly over carving depth 2 out and leaving it silent.

**Citation convention.** Every line reference is as of the baseline commit.

---

## 1. What this project is

**The claim:** a synchronous closure that captures a binding one env record
away reads and writes that binding, or kali refuses the program. At the
baseline a captured binding that Stage C did not lower falls through to the
identifier's zero placeholder (`emit/control_flow.rs:2623-2640`), so the
closure reads `0` at exit 0. Stage C chose that on purpose, to keep
`nested-wrapper-pruning` green (`emit/closure_access.rs:11-19`). A captured
parameter is in the same class (corrected by A-1, §6): MIR gives every
parameter the layout `TaggedVal`, so its cell is a heap cell, and
`cell_is_promotable` promotes a heap cell only with an `Object` repr. The cell
is never promoted, and the closure's read reaches the placeholder while the
owner reads its real WASM parameter (§2.1, `v4`).

After this project:

1. **Phase 1, fail closed.** Every reference to a captured binding is lowered
   through its env cell or refused with E5506. None falls back to a
   placeholder, a stale local, a dropped store or invalid wasm (§3.1).
   `kali check` refuses the same programs, except for a residue it records
   (§3.4).
2. **Phase 2, real.** For a capture one env record away (MIR depth 1), from a
   synchronous closure, `kali run` prints node's output when the binding is:
   * an `i64` parameter, read or written by the closure or by its owner,
     before or after the closure is created;
   * a `let` initialized from such a parameter (`let n = k`);
   * an F64 parameter or local, read or written;
   * a boolean parameter or local whose boolean proof holds, read (rendered
     `true` / `false`) or written.
3. A program with no captured parameter and no captured F64 or boolean binding
   compiles byte-identically to phase 1's output (§5.4).

### 1.1 What this project does NOT claim

* **No depth-2 lowering.** A closure that captures a binding two or more env
  records away is refused (§7.11 item 2, `d02`, `w6`). Block-scoping §4 and
  §3's `r_loopmix` keep their own refusals and messages.
* **No string captures.** A captured `String`-repr binding, parameter or local,
  is refused. Handles point into the arena, which crosses G5 and the N1
  arena-escape family.
* **No Object-repr parameters.** C2 promotes Object-repr *locals* already; a
  captured Object parameter becomes an Object local after the rewrite and is
  refused, because the rewrite's repr inheritance is pinned only for I64, F64
  and boolean (§3.2).
* **No other parameter forms.** Default, rest and destructuring parameters,
  and parameters of a function that references `arguments`, are not rewritten.
  When captured they are refused (§3.1).
* **No deferred lane change.** A deferred callback (`setTimeout`,
  `queueMicrotask`, …) keeps the allowlist at `intrinsics/host.rs:1700-1790`.
  A deferred callback over an F64 binding stays refused (`v2`'s shape).
* **No iteration-record change.** Cells of block-scoping's per-iteration
  records keep their own lowering and refusals.
* **No boolean rendering fix outside captures.** §7.2 / register R-34 (a
  boolean returned from a function renders `1`) is not touched.
* **No proof-boundary change.** `proofs/BOUNDARY.md` does not cover this
  lowering.

---

## 2. What was measured

Every program below becomes a probe `tools/array-return-probes/probes/cb_*.js`
in Task 0 (§5.1). Exit codes were measured separately from the output. Every
`check` column below is exit 0: `check` refuses none of these at the baseline.

### 2.1 The reported lane: captured parameters

| probe | program | node | kali `run` |
|---|---|---|---|
| `p1` | `function f(k){ const g=()=>k; return g(); } console.log(f(5));` | `5` | `0`, exit 0 |
| `p2` | `function f(k){ let n=k; const g=()=>n; return g(); } console.log(f(5));` | `5` | `0`, exit 0 |
| `v6` | `function f(k){ let n=k+0; const g=()=>n; return g(); } console.log(f(5));` | `5` | `5`, exit 0 (control) |
| `v4` | `function f(k){ const g=()=>k+1; console.log(g(), k); } f(5);` | `6 5` | `1 5`, exit 0 |
| `v8` | `function f(k){ function g(){ return k; } return g(); } console.log(f(5));` | `5` | `0`, exit 0 |
| `v9` | `const f=(k)=>{ const g=()=>k; return g(); }; console.log(f(5));` | `5` | `0`, exit 0 |
| `a3` | `function f(k){ const g=()=>k; k=k+1; return g(); } console.log(f(5));` | `6` | `0`, exit 0 |
| `a4` | `function f(k){ const g=()=>{ k=k+1; }; g(); return k; } console.log(f(5));` | `6` | `5`, exit 0 |
| `w4` | `function f(k){ const g=()=>{ k++; }; g(); return k; } console.log(f(5));` | `6` | E5506 (`update expression lowering is unavailable`) |
| `v7` | `function f(s){ const g=()=>s; return g(); } console.log(f("hi"));` | `hi` | `0`, exit 0 |
| `a6` | `function f(x){ const g=()=>x; return g(); } console.log(f(1.5));` | `1.5` | `0`, exit 0 |
| `a7` | `function f(b){ const g=()=>b; return g(); } console.log(f(true));` | `true` | `0`, exit 0 |
| `v2` | `function f(k){ setTimeout(()=>console.log(k),0); } f(5);` | `5` | E5506 (`a captured param binding without closure lowering …`), the deferred lane |

`v4` locates the hole: the closure's `k` reaches the zero placeholder (`0`,
plus one), while the owner's own `k` resolves to its WASM parameter (`5`).

`p2` against `v6`: a `let` initialized from a bare parameter copies the
parameter's MIR layout, `TaggedVal`, while one initialized from `k+0` gets
`Scalar("number")` (A-1). Only the scalar cell promotes.

### 2.2 The wider class: captured locals that Stage C does not promote

| probe | program | node | kali `run` |
|---|---|---|---|
| `a1` | `function f(){ let s="hi"; const g=()=>s; return g(); } console.log(f());` | `hi` | `0`, exit 0 |
| `a2` | `function f(){ let x=1.5; const g=()=>x; return g(); } console.log(f());` | `1.5` | `0`, exit 0 |
| `w5` | `function f(){ let b=true; const g=()=>b; return g(); } console.log(f());` | `true` | `1`, exit 0 |
| `w1` | `function f(){ let s="a"; const g=()=>{ s="b"; }; g(); return s; } console.log(f());` | `b` | `a`, exit 0 (the store is dropped) |
| `w2` | `function f(){ let x=1.5; const g=()=>{ x=2.5; }; g(); return x; } console.log(f());` | `2.5` | E4201, invalid wasm |
| `w3` | `function f(){ let x=1.5; const g=()=>{ x+=1; }; g(); return x; } console.log(f());` | `2.5` | E5506 (`compound assignment lowering is unavailable`) |

`w5` differs from the rest: there is no `Bool` repr (`kali_common/src/repr.rs:18`),
so a boolean is `I64`, its cell is promoted, and the value is right. The read
returns `ValueShape::Scalar` (`closure_access.rs:245-256`) and renders `1`.

### 2.3 Depth 2

| probe | program | node | kali `run` |
|---|---|---|---|
| `d02` | `function m() { let a = 5; setTimeout(() => { let z = 10; const h = () => z + a; console.log(h()); }, 0); } m();` | `15` | `10`, exit 0 |
| `w6` | `function m() { let a = 5; const o = () => { let z = 10; const h = () => z + a; return h(); }; return o(); } console.log(m());` | `15` | `10`, exit 0 |

`env_walk_depth_for` returns `None` for `mir_depth != 1`
(`closure_access.rs:184-189`), and the read reaches the placeholder.
`stageC-closures-triage.md` item 6 says this shape "fails closed"; it does
not, as §7.11 item 2 already records.

---

## 3. Design

### 3.1 Phase 1: the refusal rule

**Rule.** A reference to a binding that the current function's env plan, or
an active iteration plan, lists as `cell_for(name)` or `captured_for(name)` is
lowered through that cell or refused with E5506. "Lowered through that cell"
means `resolve_capture_access` (or `resolve_scalar_capture_access` on an
arithmetic write path) returned `Some`. Every other outcome is a refusal.

| site | baseline | phase 1 |
|---|---|---|
| read of a plan-listed name whose repr is not promotable (String, F64, `TaggedVal`, …) | zero placeholder | refuse, reason *value type* |
| read of a plan-listed name at MIR depth ≥ 2 | zero placeholder | refuse, reason *depth* |
| read or write of a captured **parameter**, any repr, by the closure or by the owner | closure: zero placeholder (A-1); owner: its WASM parameter | refuse, reason *parameter* |
| plain `=` to a plan-listed name that is not promoted, from the closure or the owner | dropped store, or invalid wasm | refuse, reason *value type* or *depth* |
| a read of a promoted `I64` cell whose owner binding is proven boolean | renders `1` / `0` | refuse, reason *value type* (phase 2 lifts it, §3.3) |
| compound assignment, update expression | E5506 | unchanged |

**Placement.** The read deny goes at `IdentifierResolution::CapturedCellOrPlaceholder`
(`emit/control_flow.rs:2623`), between `try_emit_captured_read` and
`push_placeholder_fallback_diagnostic`. A name that is not plan-listed keeps the
placeholder path unchanged: it is an undefined identifier, not a capture. The
write deny goes at the plain-assignment choke that today emits the dropped
store or the invalid wasm; Task 2 locates it and names it in the plan. A
parameter is identified by the owner's MIR binding kind
(`MirBindingKind::Parameter`, `kali_mir/src/binding.rs:8`). To keep codegen off
MIR bindings, `EnvCell` and `CapturedRef` gain one field, `is_parameter`, set
in `derive_env_plans` (`kali_mir/src/env_plan.rs:205`).

**Unchanged sites.** The deferred-registration lane (`host.rs:1700-1790`)
keeps its own allowlist and messages. Module-owned captures are module globals
and never plan-listed. A `const` that folds resolves before the choke. The
URL / USP / abort captured denies keep their messages, because they fire
before the choke.

**Message.** One family in `kali_common::messages`:

```text
a closure `g` that captures `k` is unavailable in the current phase: <reason>
```

with exactly three reasons:

* `its value type has no closure cell`
* `` `k` is two or more closures away``
* `` `k` is a parameter of `f` ``

`g` is the capturing function's display name as written. For an arrow bound to
a `const` that is the binding's name; for an anonymous function it is the
display name block-scoping §3.5 gives it. A refusal at the owner (a write to a
captured parameter in its owner) names the first capturer in source order.

**Ordering.** Phase 1 lands with its own sweep and triage (§5.4) before any
phase-2 code. Phase 2's moves are measured against phase 1, not the baseline.

### 3.2 Phase 2: the parameter-to-local rewrite

A new AST pass, `crates/kali_cli/src/build/capture_param_rewrite/`, runs in
`analyze_source_file` directly after `rename_block_scoped_bindings`
(`compile.rs:819`) and before `monomorphize_statements` (`compile.rs:834`). It
therefore runs under `check`, `run` and `build`, and every monomorphized clone
carries the rewritten body.

**When it applies.** To a simple identifier parameter of a function
declaration, function expression, arrow function or class method, when a
function nested in it references that parameter. "References" is decided by
the block-scope rename's scope tables (`block_scope_rename/table.rs`,
`walk.rs`), which run immediately before and are already scope-correct, so
"captured" has one AST-level definition. The pass does not re-derive scoping.

**The rewrite.**

```js
function f(k) { BODY }        // before
function f(k$p) { let k = k$p; BODY }   // after
```

Only the parameter's spelling changes. `BODY`, call sites and arity are
untouched. The spelling uses the rename pass's convention for names a source
program cannot write, and display names map back to `k` as block-scoping §3.5
does. An arrow with an expression body is given a block body
(`k => e` becomes `k$p => { let k = k$p; return e; }`) first.

**When it does not apply.** Default, rest and destructuring parameters; any
parameter of a function that references `arguments`; a parameter that no
nested function references. Each is left alone, so a captured one reaches
§3.1's *parameter* refusal. After phase 2 that refusal is reachable only
through these forms.

**Repr inheritance.** repr_infer aliases a parameter to its binding's scalar
node (`kali_types/src/repr_infer.rs:7068-7090`). The new `let k = k$p` must
take `k$p`'s repr in each specialization. One case per repr pins it (I64, F64,
boolean, String; §5.2). The String and Object cases pin a *refusal*: those
locals reach §3.1's *value type* reason.

**`let n = k`.** At the baseline `p2` reads `0` and `v6` reads `5`. The cause is
not traced. Task 1 traces it before the rewrite lands. If it is the parameter
alias in repr_infer, the rewrite removes it, because `k` is then a local. If it
is anything else, `p2` either becomes node-equal within this project or is
refused under §3.1. It is never left silent.

### 3.3 Phase 2: typed cells for F64 and boolean

**F64.** `closure::cell_is_promotable` (`closure.rs:67`) admits a scalar cell
whose owner repr is `Repr::F64`. The cell keeps its 8-byte slot. `emit_cell_load`
and the matching store take the owner repr and emit `f64.load` / `f64.store`
for F64, `i64.load` / `i64.store` otherwise. Every caller passes the repr
through: the read, the promoted declaration, `=`, compound assignment and the
update expression. The read returns the float shape the non-captured F64 read
returns. Promotion (`lower.rs:1426-1450`) and access (`resolve_capture_access_inner`)
still call the one predicate, so the lockstep guarantee in `closure.rs:42-49`
holds unchanged. `promotable_scalar_cell_in` (`emitter.rs:1253`) widens with it,
so compound assignment and update on a captured F64 lower too.

**Boolean.** The cell is unchanged (`I64`, 0 / 1). A capture read returns
`ValueShape::Boolean` when the owner's binding is proven boolean by the same
proof a non-captured local of the owner uses to render `true` / `false`. Task 5
names that proof in the plan. A boolean binding without the proof keeps
`ValueShape::Scalar`, which is not a boolean claim, so it is not refused. A
proven-boolean binding whose read cannot carry the shape stays refused (§3.1).

**What phase 2 lifts.** The §3.1 refusals for: a captured `i64`, F64 or
proven-boolean binding at MIR depth 1, local or rewritten parameter, read or
written by the closure or the owner. String, Object parameters, `TaggedVal`,
depth ≥ 2 and the unrewritten parameter forms stay refused.

### 3.4 `check`

A `kali_types` pass, `captured_bindings::capture_refusals`, runs in
`analyze_source_file` after repr_infer and after the class-instance rewrite's
re-inference (`compile.rs:860-880`), so `check` and `run` both see it. It
walks the post-rewrite AST and computes, per captured reference, the capturing
function, the owner, and MIR's **structural** depth: the number of functions
between capturer and owner, inclusive of the owner, that own at least one
captured binding. It refuses with §3.1's messages:

* a captured binding whose repr is not `I64`, `F64` or `Object(_)`, and a
  proven-boolean binding until phase 2 lifts it;
* a structural depth ≥ 2;
* a captured parameter that the rewrite left in place.

**The residue.** Codegen's walk depth depends on which cells promote. MIR's
structural depth counts an intermediate function that owns only unpromotable
cells; codegen's chain skips it. Where they disagree, `check` is the stricter
of the two, because it refuses at structural depth ≥ 2 and `run` refuses the
same program for depth or for the unpromotable cell. The property is: every
program `run` refuses with a §3.1 reason, `check` refuses too. A differential
case asserts it over every `cb_*` probe (§5.2). Anything that breaks it is
recorded in the followups' `check` / `run` gap table, as block-scoping §3 did,
and is not claimed.

Iteration-record captures are out of this pass. Block-scoping's own `check`
coverage and its gap (block-scoping followups §3) are unchanged.

### 3.5 Docs

* `specs/15-errors.md`: the E5506 row gains the three §3.1 reasons.
* `specs/19-feature-maturity.md`: a new row, "Closures over parameters and
  number / boolean bindings": depth 1, synchronous, `i64` / F64 / proven
  boolean, parameters (simple identifiers) and locals, reads and writes.
  The row names what it does not claim (§1.1).
* `kali-silent-miscompile-register.md`: new entries for §7.11 item 1 (FIXED in
  the slice; FAIL_CLOSED outside it) and §7.11 item 2 (FAIL_CLOSED), with the
  §2.2 lanes as lanes of item 1's entry.
* `block-scoping-discovered-defects.md` §7.11: items 1 and 2 point to the new
  entries.
* A new `docs/superpowers/followups/captured-bindings-discovered-defects.md`:
  string captures, Object parameters, the depth-2 lowering, deferred F64, the
  unrewritten parameter forms, the `check` / `run` residue, and anything the
  triage finds.
* `specs/12-cli.md`, `specs/18-schemas.md` and `README.md` do not change. There
  is no new command, flag or schema field.

---

## 4. Approaches not taken

* **B. Prologue copy-in in codegen.** After `emit_env_alloc`, store each
  promoted parameter into its cell, and make every owner access to the name
  resolve to the cell before the WASM parameter. It leaves the AST alone, but
  it changes the resolution order in every access helper (read, `=`, compound,
  update, declaration). That is the one-sided-widening class the C1 review
  closed (`closure.rs:42-49`). It also does nothing for `check`.
* **C. MIR `Local` with a synthetic initializer.** Mark a captured parameter as
  a `Local` in MIR. MIR is repr-blind, so codegen still needs B's resolution
  changes, and it costs B plus a MIR change.

A reuses the C1 local path that already works end to end (`let k=5` captured
reads `5`), and both commands see it, because it runs in the shared front end.

---

## 5. Testing and measurement

### 5.1 Probes

Task 0 adds every program in §2 as `tools/array-return-probes/probes/cb_<id>.js`
and records a baseline column `tools/array-return-probes/baseline-cb.tsv`, from a
binary rebuilt at `2ddf18c66`. It re-measures every row of §2 on that binary.
A row that differs is corrected in this spec as an amendment (§6) before
phase 1 starts.

### 5.2 Cases

A new `crates/kali_cli/tests/cases/closure/captured_bindings.toml`, run by the
single `cases` target (`crates/kali_cli/tests/cases/README.md`):

* every §2 probe, pinned node-equal or refused with its §3.1 reason under
  `run`, and refused or exit 0 under `check` as §3.4 says;
* repr inheritance through the rewrite: an I64, F64, boolean and String
  parameter, each captured;
* the parameter forms left alone: default, rest, destructuring, `arguments`,
  each captured and refused;
* a method parameter and a function-expression parameter, each captured;
* the differential property of §3.4, over every probe: `run` refuses with a
  §3.1 reason ⇒ `check` refuses;
* byte-identity (§1 item 3): two programs with no captured parameter and no F64
  or boolean capture compile to the same wasm at phase 1 and phase 2.

### 5.3 Unit tests

In sibling `*_tests.rs` files, never inline modules:

* `capture_param_rewrite`: applies to a captured simple parameter; skips an
  uncaptured one; skips the default / rest / destructuring / `arguments`
  forms; gives an arrow expression body a block; the rewrite is idempotent.
* `kali_mir::env_plan`: `is_parameter` set on a cell and a ref for a captured
  parameter, and not for a local.
* `kali_codegen::closure`: `cell_is_promotable` admits F64 scalars and still
  refuses String and `TaggedVal`; the typed load and store emit the right
  instruction per repr.
* `kali_types::captured_bindings`: structural depth over nested functions; an
  intermediate function with only unpromotable cells counts.

### 5.4 Blast radius and capability loss

Each phase runs the whole trial sweep, the corpus and `cargo test --workspace`
before and after, and triages every moved trial into one of:

* **wanted:** silent → refused (phase 1) or silent / refused → node-equal
  (phase 2);
* **capability loss:** a program that exited 0 with node's output and is now
  refused. Phase 1 expects none, because every refused lane read a placeholder
  or a dropped store. Any found is reported to the human partner
  before the phase continues;
* **other:** anything else, explained one by one.

The triage tables go in the followups file (§3.5), one per phase.

---

## 6. Amendments

### A-1 (2026-10-05, before the plan): a captured parameter's cell is not promoted

**Measured.** A throwaway `kali_mir` unit test (deleted afterwards) printed the
MIR layouts and `derive_env_plans` output:

| program | binding | layout | cell |
|---|---|---|---|
| `function f(k){ const g=()=>k; return g(); }` | `k` (Parameter) | `TaggedVal` | `is_scalar: false` |
| `function f(k){ let n=k; const g=()=>n; return g(); }` | `n` (Local) | `TaggedVal` | `is_scalar: false` |
| `function f(k){ let n=k+0; const g=()=>n; return g(); }` | `n` (Local) | `Scalar("number")` | `is_scalar: true` |
| `function f(x){ const g=()=>x; return g(); } f(1.5);` | `x` (Parameter) | `TaggedVal` | `is_scalar: false` |

Probes measured by hand at the same binary: `function f(){ let k=5; let n=k; … }`
reads `5` (a local copied from a local stays `Scalar`); `let n=0; n=k;` reads
`5`; `let n=k;` with the owner also logging `n` prints `5` then `0`.

**What it overrides.**

1. **§1 and §2.1.** The spec said a captured `i64` parameter's cell is
   promoted but never initialized. It is not promoted: its layout is
   `TaggedVal`, its cell is a heap cell, and `cell_is_promotable`
   (`closure.rs:67`) admits a heap cell only with an `Object` repr. The
   closure's read reaches the placeholder. §1 and §2.1 are corrected in place.
2. **§3.2 alone fixes nothing.** `let k = k{p}` is the `let n = k` shape and is
   also `TaggedVal`. The rewrite stays, because it gives `k` a declaration
   store and makes the owner's reads go through the cell (once the cell
   promotes, that is the uninitialized-cell bug the spec described). But it
   only takes effect together with point 3.
3. **§3.3 gains a gated widening.** `cell_is_promotable` also admits a
   non-scalar cell when the owner's MIR layout for it is `TaggedVal` and:
   * the owner repr is `I64` and `ReprTable::binding_is_proven_numeric(owner, name)`
     holds, or
   * the owner repr is `F64` and the same numeric proof holds, or
   * the owner repr is `I64` and the binding is proven boolean (§3.3's proof).

   Without the proof the cell stays unpromoted and §3.1 refuses it. Codegen
   cannot see the MIR layout, so `EnvCell` and `CapturedRef` gain
   `is_tagged: bool` beside `is_scalar` and `is_parameter`. `Struct`, `Array`
   and `Closure` cells are unchanged. The promotion site
   (`lower.rs:1426-1450`) and the access gate still call the one predicate.
4. **Open question, resolved by plan Task 1.** `write_value_is_numeric`
   (`repr_infer.rs:2061-2072`) treats a parameter on the right of a write as
   numeric on condition that every call site passes a number. Task 1 measures
   whether `binding_is_proven_numeric` then holds for `let k = k{p}` with
   numeric call sites, and fails without them. If it does not hold, extending
   the proof is in phase 2. If it cannot be extended soundly, the lane stays
   refused and the followups record why.
5. **§3.4** uses the same gate: a captured binding with no proof is refused by
   `check` too.
6. **Phase 1 (§3.1) is unchanged.** It refuses every unpromoted capture,
   whatever the reason.

### A-2 (2026-10-05, while writing the plan): what the code turned out to be

Each point was measured or read at `2ddf18c66`, and the human partner chose
point 1.

1. **Boolean is narrowed to a captured `const`.** kali has no boolean proof for
   any binding. `let b=true; console.log(b)` prints `1` uncaptured (the register
   records it at `kali-silent-miscompile-register.md:5250`). Only a fold-lane
   `const` renders `true`, because its reads re-emit the literal. Measured:
   `const b=true` logged directly prints `true`; logged from a closure it
   prints `1`. That is the one capture-specific boolean defect. So:
   * A new positive proof, `ReprTable::boolean_consts: HashSet<(scope, name)>`,
     is written by `repr_infer` for a `const` whose initializer is a boolean
     literal, a comparison (`== === != !== < > <= >=`) or a `!` expression,
     parenthesized or not. Its hook is `note_fn_alias` (`repr_infer.rs:3207`),
     and `emit_table` installs it next to `numeric_bindings` (`:7427`).
   * Phase 1 refuses a capture read of such a `const`, with reason
     *value type*. Phase 2 lets the capture read return `ValueShape::Boolean`
     for it.
   * A `let` boolean, captured or not, keeps rendering `1`. That is the
     register's lane, not this project's. §1 item 2's boolean bullet, §3.3's
     boolean paragraph and A-1 point 3's third bullet are replaced by this
     point.
2. **There are no unrewritten parameter forms.** Default, rest and
   destructuring parameters are refused by the parser ("kali functions take a
   fixed list of plain named parameters"), and `arguments` is E3100. Every
   captured parameter is a simple identifier and is rewritten. §1.1's
   "other parameter forms" bullet, §3.2's "when it does not apply" list and the
   §5.2 cases for those forms are dropped.
3. **The owner side is never refused.** An unpromoted captured binding keeps a
   WASM local in its owner, so the owner's own reads and writes are correct.
   §3.1 refuses only in the capturing function. The table row "read or write
   of a captured parameter … by the closure or by the owner" becomes "by the
   closure".
4. **The capturer is named by its plan key** with `display_names_in` applied:
   a named function shows its name, and an anonymous closure shows
   `__kali_fn_N`, as block-scoping's A-4 message already does. §3.1's display
   name sentence is replaced.
5. **The refusal skips refs owned by block-scoping.** A ref with
   `through_iteration`, or whose owner plan is an iteration plan, keeps
   `iteration::iteration_capture_diagnostics`'s refusal and is not refused
   again.
6. **`check` lives in `kali_cli`, not `kali_types`.** repr_infer's `parents`
   records only `FunctionDeclaration` nesting (`repr_infer.rs:924-936`), so it
   cannot see an arrow's owner. The `check` pass reuses the block-scope rename
   walk instead: `walk::Hooks::enter` gains the function's plan key, and a new
   `build/capture_refusals` pass runs after repr_infer. MIR's scopes are
   function-level (`kali_mir/src/analysis/scope.rs:8-33`), and its depth counts
   ancestors that own at least one captured binding, so `check` computes the
   same structural depth exactly. The residue is the `TaggedVal` local that
   codegen does not promote (`let n = k` in phase 1, an unproven number in
   phase 2): `check` cannot see MIR layouts and admits it, while `run` refuses
   it. The §5.2 differential case lists those programs by name as the known
   residue rather than asserting the property over them. A method's plan key
   is `<Class>__<method>`. Where `check` cannot form a key it admits, and the
   followups record it.
7. **Probes run with `FORCE_COLOR` unset.** It is `3` in the measuring
   environment, and node then colours numbers, which breaks every probe
   comparison.
