# Runtime `Array.prototype.join` over string-element arrays — design

**Date:** 2026-07-06
**Status:** Approved (design)
**Series:** Runtime strings & dynamic tables for verbatim fasta — **Spec 3 of 6**
**Predecessors:** Spec 1 (`docs/superpowers/specs/2026-07-06-runtime-string-value-flow-design.md`,
PR #9), Spec 2 (`docs/superpowers/specs/2026-07-06-substring-runtime-design.md`, PR #10).

## Series context

Upstream fasta's `fastaRandom` builds each output line as
`line[i] = c; … console.log(line.join(''))` over a `new Array(60)` that is
reassigned to `new Array(n)` for the final partial line. Spec 3 owns the
array side of that surface: **arrays as string containers** (element stores
and reads of runtime string handles), **array binding reassignment**, and a
**runtime `join(sep)`**. The inner element picker is `for (c in table)` —
that is Spec 4; Spec 3's capstone stubs it (below).

## Investigation findings (ground truth, probed on main 745a3ecea)

A runtime array lane already exists and is load-bearing (fannkuch):
`const`/`var` `new Array(n)` with **runtime** size, runtime-index i64
element stores/reads, and `.length` all work. Arrays are i64 slots in
linear memory; the binding is a local holding a base-address handle,
registered in `FunctionEmitter::array_bindings` at *declaration* sites only
(`kali_codegen/src/emit/control_flow.rs` declarator path, plus array
params in `emitter.rs`).

What is broken or missing today:

- **Array binding reassignment** (`let a = new Array(60); a = new Array(n)`)
  — silent-wrong (prints 0): the assignment-expression path never routes an
  array-alloc RHS through `emit_array_allocation`. Needed verbatim by
  fastaRandom's partial-line path.
- **String elements** — correctly rejected by Spec 2's F1 gate
  (E5506 "element and field reads have no string lane yet").
- **Runtime `join`** — exists only as a static fold
  (`resolve_static_array_join_call`); every non-static shape falls through
  `resolve_array_join_member_call`'s `has_static_receiver` early-return with
  NO diagnostic and prints `0` silently. Even a static literal receiver with
  a variable separator prints `0`.
- **Literal-array mutation** — `a[k] = 42; a[k]` prints 0 (and inside a
  function even the static-index form does): literal arrays ride the
  static-fold lane and unfoldable mutation fail-opens.

`string_concat` results are allocated via the guest `__alloc_global`
export — global region, never reclaimed (`kali_runtime/src/host/memory.rs::
alloc_guest_string`). This drives the join-mechanism choice.

## Scope (user-approved)

Bundled into this spec (option "core + fail-closed gates"):

1. **String element store/read lane** — lift Spec 2's F1 reject into real
   support where provable; keep rejecting everything else.
2. **Array binding reassignment** (array-alloc RHS and array-to-array
   handle copy), closing today's silent-wrong.
3. **Runtime `join(sep)`** on proven string-element, ASCII-proven receivers.
4. **Fail-closed gates** for the adjacent silent-wrong holes from the
   Spec 2 follow-up inventory and the probes above: object-literal
   construction stores `{v: s}`, `&&`/`||` store launder, `a.slice(i)`,
   literal-array unfoldable mutation, and every unsupported join shape.

Out of scope: `for..in` / dynamic keys (Spec 4), `process.argv` + coercion
(Spec 5), the verbatim fixture + two-tier validation (Spec 6), runtime join
over *number*-element arrays (no fasta need — YAGNI, rejected), the Spec 2
over-reject idioms (`s.substring(0, s.length)`, ternary bounds — workarounds
exist, no fasta need), UTF-16 semantics for non-ASCII (rejected fail-closed,
as in Spec 2).

## Approach decision (join mechanism)

Three candidates; the types-side axis and gates are identical in all three.

- **A — guest-side synthetic `__join` (CHOSEN).** A pure-wasm function
  emitted on demand like Spec 2's `__substring`:
  `__join(array_handle: i64, sep_handle: i64) -> i64`. Pass 1 loops the
  slots summing `handle & 0xFFFF_FFFF` plus `sep_len × (n−1)`; pass 2 does
  ONE `__alloc` of the total, then `memory.copy`s each element's bytes and
  the separator between them; returns `TAG | ptr<<32 | len`. Fast path:
  empty array → interned `""` handle (immortal pool — alias-safe). A
  single-element array is deliberately NOT returned zero-copy: it copies
  like any other, so the "join result is a fresh allocation" escape
  invariant holds unconditionally (a runtime branch returning an element
  handle would force the static escape model to treat every join result as
  element-aliasing — worse than the one-element copy it saves). No new
  host import — the 4 hand-mirrored browser
  JS import lists stay untouched (known LinkError footgun). One allocation
  per join through the **arena-aware** guest allocator, so fastaRandom's
  per-line join result is reclaimed by the existing per-loop arena
  machinery — the only option whose memory story survives Spec 6's
  N=25,000,000 (~417k lines) without rework. Cost: the most codegen work,
  and escape flow must model the result (below).
- **B — host import `string_join`** — simplest codegen, but a fifth entry
  in all 4 hand-mirrored JS import lists, host coupling to the guest array
  layout, and `__alloc_global` results leak (~25MB dead at N=25M). Rejected.
- **C — desugar to a `string_concat` fold** — cheapest to build; 59 host
  calls and ~1.8KB of leaked dead intermediates per 60-char line
  (~760MB dead at N=25M). Rejected.

## Design

### 1. Element string repr axis (`kali_common` + `kali_types`)

The `ReprTable` element axis (today `I64 | F64`, driving repr-directed
element load/store) gains `String`. Seeds: element-store edges whose RHS is
string-valued per Spec 1's string-seed BFS. The axis unions across every
store to the binding, flows through array params and reassignment merges,
and carries the **non-ASCII** and **concat-taint** provenance bits
element-wise (union — fail-closed by construction). A string/number mix on
one array is a fail-closed conflict diagnostic; never-stored and
never-called shapes keep compiling (Spec 1's monotone precedent —
`kali check`-only benchmark fixtures depend on it).

### 2. Store and read lanes

Spec 2's F1 gate *re-keys* on the proven axis: a store of a string-valued
RHS is accepted iff the target array's element repr is String (otherwise
E5506 as today). Stores accept regardless of the non-ASCII bit — the bit
rides the axis and **rejects at byte-length-sensitive consumers** (join,
`.length` of an element), the same placement as Specs 1–2. Element reads on
proven-String arrays become string-typed expressions feeding every Spec 1/2
consumer: `+`, `console.log`, substring receiver, `.length`, the `==`
concat-taint rules. Codegen is mechanically free (slots and handles are both
i64); the work is oracle arms — `is_string_valued` and its length/concat
siblings gain a "subscript of a proven-String-element array" arm, and the
`kali_types` mirror predicates (`expression_is_string_typed`,
`operand_repr_is_string`, `expression_is_runtime_string_value`,
`expression_is_length_fold_receiver`) gain matching arms **in the same
change** — the Spec 2 hand-mirrored-oracle lesson, restated here as a
standing constraint on every task of the plan.

### 3. Array binding reassignment

The assignment-expression path learns what the declarator path already
does: an array-alloc RHS routes through `emit_array_allocation` +
`LocalSet`, and `a = b` between array bindings copies the handle.
`array_bindings` registration extends beyond declarators (the inference
already knows which bindings are arrays). Repr/shape merges at the binding
like a phi: element axes union, conflicts reject; array-to-scalar or
scalar-to-array reassignment is a conflict reject. `.length` needs no new
work — it already resolves from the handle at runtime.

### 4. Synthetic `__join` (codegen)

Emitted once when any runtime join compiles, `__`-reserved and excluded
from the tag-boxing census like the other five reserved exports. Gate
(types side): receiver is an array binding with proven String element axis
AND ASCII-proven elements; separator is any proven-ASCII string expression —
static or runtime, `''` included. Everything else rejects E5506 (never
silent). The static fold lane (`const a = ["x","y"]; a.join(",")`) is
untouched and must stay compile-time.

### 5. Escape flow (`escape_flow.rs`)

Two new edges, both fail-closed:

- **Join result = fresh allocation.** No aliasing of receiver or elements
  (guaranteed by the always-copy rule in §4; simpler than substring's
  receiver-alias). It is heap: arena-eligible
  when it does not escape; escaping results follow the existing
  escape-flow rules.
- **Store-into-container alias (the dangerous one).** Storing a string
  handle into an array aliases the string to the container: an
  arena-allocated substring stored into an array that outlives the arena
  would be use-after-reset — memory corruption, strictly worse than a
  wrong answer. Element stores of string handles add an alias edge so the
  string's lifetime follows the array's; whenever the fixpoint cannot
  prove containment, the string escapes. This edge gets the adversarial
  review treatment the Spec 2 final review gave `opens_arena`.

### 6. Data flow (fastaRandom shell)

`line = new Array(60)` (function-scope alloc) → loop stores single-char
string handles (String element axis proves) → `line.join('')` → `__join`
arena-allocates the 60-byte line → `console.log` prints it → the loop's
arena reset reclaims it → `n -= line.length` reads the runtime length →
the final iteration reassigns `line = new Array(n)` and the merged binding
keeps its String element repr.

## Error handling — fail-closed matrix

| Case | Behavior |
|---|---|
| Store string into array whose stores are all string-valued | compiles (i64 slot write) |
| Same array also stores numbers (incl. via reassignment merge) | conflict reject |
| `line.join(sep)`, String elements, ASCII-proven, sep proven-ASCII string | compiles → `__join` |
| Join with any non-ASCII element/separator, or unproven receiver/sep | E5506 reject |
| Join over runtime number-element arrays | E5506 reject (static fold lane unchanged) |
| Object-literal construction `{v: s}` with runtime string | E5506 reject (closes silent-wrong) |
| `&&`/`||` laundering a runtime string toward a store or join | reject via new predicate arms (both mirrors) |
| `a.slice(i)` on a string receiver | E5506 reject (closes silent 0) |
| Literal-array mutation the fold lane cannot statically evaluate (incl. `a[k]=42` and the in-function static-index case) | E5506 reject |
| Array binding reassigned to a scalar, or scalar to an array | conflict reject |
| Ternary/logical wrapping of receivers/RHS | predicates recurse or hit the fail-closed default arm — no bypass |
| Every unsupported join shape that prints `0` today | rejects — the fail-open closes wholesale |

## Testing

Five layers (patterns proven in Specs 1–2):

1. **`repr_infer` unit pins** — axis seeding, union, conflict, param flow,
   reassignment merge, non-ASCII bit propagation.
2. **e2e green lane** — store→read roundtrip; join with `''` / `','` /
   runtime separator; empty and single-element arrays; join result
   flowing into `+` / `console.log` / `substring` / `.length`; the
   reassignment shell.
3. **e2e reject pins** — one per matrix row above.
4. **Regression** — shipped CLBG fixtures (fannkuch, spectral-norm, n-body,
   binary-trees, mandelbrot) byte-identical; static join fold stays
   compile-time (pin); `__join` excluded from the tag-boxing census (pin);
   standing 5-crate gate + CI-exact `cargo clippy --workspace -- -D warnings`.
5. **Escape adversarial** — `kali_mir` unit pins for the
   store-into-container edge and join freshness; a use-after-reset probe
   (arena substring stored into an outer-scope array, loop resets, read
   back — must have escaped to global).

**Capstone:** the fastaRandom shell vendored with ONLY the `for..in`
picker swapped for a Spec-2-supported stand-in (substring-based pick from a
seed string), pinned `n` around 200 so the partial-last-line reassignment
path executes, byte-for-byte against a node-run golden of the same adapted
source — the Spec 2 `fastaRepeat` capstone pattern.

## Success criteria

1. Capstone green (byte-for-byte vs node).
2. Every fail-closed matrix row pinned by a test.
3. Zero diffs on shipped CLBG fixtures and the static-fold join lane.
4. The probe family that silently prints `0` today (non-static join shapes,
   literal-array mutation, array reassignment) all reject with diagnostics.
5. Full gate: workspace tests, clippy CI-exact, fmt, browser-glue diff clean.

## Risks

- **Store-into-container escape edge is the safety-critical piece** — a
  fail-open is memory corruption, not just a wrong answer. Mitigation:
  conservative default (escape when unprovable), dedicated adversarial
  probes, and reviewer instruction to attack this edge specifically.
- **Oracle mirroring** — every new expression shape (element read, join
  call) must land arms on the codegen oracle AND the types predicates in
  the same change, or it fails open (Spec 2's two-Critical lesson).
- **Reassignment merge** touches the repr fixpoint; conflicts must stay
  monotone or never-called-function compiles regress.
- **Element axis relaxation** must not perturb the i64/f64 element lanes
  fannkuch/spectral depend on — the fixture goldens are the guardrail.

## Series notes

- Runtime-sized `new Array(n)` turned out to already work (fannkuch lane);
  what Spec 3 adds is reassignment + string elements. No new series slot
  needed for runtime-length arrays.
- Spec 4 (`for..in` + dynamic keys) can replace the capstone's stub picker
  with the verbatim inner loop, upgrading the capstone toward Spec 6.
