# A runtime array built by `push` flows through a program as node runs it, or kali refuses

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `edb3a77df` (`main`, the default-parameters merge) |
| kali binary | `target/debug/kali`, built at the baseline |
| oracle | `node v26.10.0`, run as `env -u FORCE_COLOR node` |
| measured on | 2026-10-07 |
| item picked | lane C of the parallel slate agreed on 2026-10-07: growable runtime arrays, the largest blocker family in the blast-radius extension corpus |
| prior art | throw-fallout Stage 4 (`2026-07-13-throw-fallout-stage4-array-push-lane-design.md`), runtime join (`2026-07-06-runtime-join-string-arrays-design.md`), literal-array mutators (`2026-10-03-literal-array-mutators-design.md`), array bounds (`2026-10-02-array-bounds-design.md`) |

**Scope chosen by the human partner (2026-10-07):**

* **Success criterion:** the dominant corpus shape runs end to end. An array is built
  in a function with `push`, returned, and the caller iterates, measures, indexes,
  joins or slices it, with output matching node. The corpus's array-refusal count
  is reported. It is not a target. The accept rate is expected to stay 0/40,
  because every extension program also hits runtime-string refusals.
* **Operations:** the core set (`push`, `.length`, a runtime-index read, `for-of`,
  `join`, `slice`, return and argument passing), plus the cheap ones on the same
  layout: index write, `pop`, `indexOf`, `includes`. Callback methods (`forEach`,
  `map`, `filter`, `reduce`, `sort`) are a later lane.
* **Elements:** i64, f64 and runtime strings, one repr per array. Objects as
  elements are refused.
* **Approach A** (§4): one growable representation, carried by repr inference;
  `new Array(n)` keeps its own layout; escaping arrays are allocated globally
  and never reclaimed.

**Citation convention.** Every line reference is as of the baseline commit.

---

## 1. What this project is

**The claim:** an array that a program builds at runtime with `push` can be
returned from a function, passed to another, aliased, and used at module
scope. `push`, `pop`, `.length`, index read and write, `indexOf`, `includes`,
`slice`, `join` and `for-of` on it either behave as node behaves or fail
loudly, with E5506 at compile time or a runtime trap with a message. Each
loud failure is named in §3.6.

**Why this item.** `kali check` over the 40 extension programs in
`tools/blast-radius/corpus/extension/` reports 386 error lines. Five
array-family messages account for 159 of them (§2.2). 37 programs report at
least one array-family refusal. The corpus's dominant array shape is `wrap()` in
`wrap_paragraph.js`:

```js
function wrap(text, width = 72, indent = "") {
  const words = text.split(" ").filter((word) => word.length > 0);
  const lines = [];
  …
  for (const word of words) { … lines.push(line); … }
  if (started) lines.push(line);
  return lines;
}
for (const line of wrap(PARAGRAPH, 50, "> ")) { console.log(line); }
```

`push` appears in 32 of the 40 programs and `for-of` in 38.

### 1.1 What this project does NOT claim

* **The accept rate.** It is measured and expected to stay 0/40.
* **Callback methods:** `forEach`, `map`, `filter`, `reduce`, `some`,
  `every`, `find`, and `sort` with or without a comparator. They need a closure call
  per element and capture of the array, so they are a separate lane.
* **The other mutators:** `shift`, `unshift`, `splice`, `reverse`,
  `copyWithin`, `fill` on a growable array, and `concat`.
* **Printing a whole array** (`console.log(xs)`). Kali has no array formatter
  (R-31 records that literals print `0`).
* **Objects, arrays or functions as elements.**
* **Reclaiming escaping arrays.** They are allocated in global memory and
  never freed (§3.3).
* **`undefined` results.** `pop()` on an empty array and an out-of-range index
  trap where node yields `undefined` (§3.6). Kali has no runtime `undefined`
  (register cluster G4).
* **Non-ASCII `join`.** The existing ASCII proof is kept.
* **Closure capture of an array**, and a function body reading a module-level
  array directly.

---

## 2. What was measured

### 2.1 The target shape, at the baseline

| program | node | kali `run` at the baseline |
|---|---|---|
| p1: `function build(n) { const out = []; for (…) out.push(i * i); return out; }` then `xs.length`, `xs[2]`, `for-of`, `xs.join(",")`, `xs.slice(1, 3).join("-")` | `5 4` / `0 1 4 9 16` / `0,1,4,9,16 1-4` | E5506 at `push` (literal array), `for-of`, `join`, `slice` |
| p2: the same with runtime strings (`out.push(w); w = w + "b"`) | `a` `ab` `abb` / `3 a ab abb` | E5506 at `push`, `for-of`, `join` |
| p3: inside one function: `out.push(i)`, `for (const x of out) s += x`, `console.log(s, out.length, out.join("\|"))` | `6 4 0\|1\|2\|3` | E5506 "console output of multiple arguments where one reads a growable array is unavailable" |
| p4: inside one function: `out.push(i)`, `console.log(out)` | `[ 0, 1, 2 ]` | E5506 at `push` (the bare console argument disqualifies Stage 4 promotion) |

### 2.2 The corpus

`kali check` over all 40 extension programs; backtick contents elided:

| message family | lines |
|---|---|
| `computed member access X is unavailable … unless the index is a l…` | 86 (includes object-key reads such as `counts[word]`, which this project does not change) |
| `for-of array iteration lowering is unavailable unless the iterable is a literal array …` | 65 |
| `calling X on a literal array is unavailable in the current phase: kali folds …` | 52 |
| `Array.prototype.slice is unavailable unless the receiver is a statically-known …` | 22 |
| `array search method 'indexOf' is unavailable unless …` | 10 |
| `Array.prototype.join is unavailable unless …` | 10 |
| `reassigning an array binding to a non-array value is unavailable …` | 7 |
| `Array.prototype.concat is unavailable …` | 3 |
| `mutating a literal array is unavailable in the current direct-runtime path …` | 1 |

### 2.3 Mechanism at the baseline

* **Literal arrays** fold at compile time and have no runtime layout. Mutators,
  element stores and `.length` writes on them are refused
  (`kali_types/src/resolve/member.rs:480-492`, `resolve/expression.rs:1660-1686`).
* **Plain runtime arrays** (`new Array(n)`) are inline `[len:i64][elem…]` in
  linear memory. Their binding holds the base address
  (`kali_codegen/src/emit/call.rs:5891-5990`). They are fixed-length. Elements are i64,
  f64 or string handles (`ReprTable.array_elements`, `kali_common/src/repr.rs:85`).
  They pass to functions as parameters, and their element nodes are unified
  across call edges (`kali_types/src/repr_infer.rs:6150-6160`).
* **Growable arrays** (Stage 4, `kali_codegen/src/emit/growable.rs:1-30`) use
  a handle `hdr_ptr | ARRAY_HANDLE_TAG` (bit 62) and a header
  `[len][cap][data_ptr]`. The data block is `cap` 8-byte slots and doubles on push
  (initial cap 4). The handle never changes. A binding is promoted only by a
  **syntactic allowlist** (`kali_types/src/growable.rs`): it must be declared
  once in a function body, initialised with a literal of scalar seeds, and
  pushed at least once. Every other occurrence must be a push receiver,
  `.length`, an index read, a `for-of` iterable or a `.join` receiver. A return,
  argument, alias, capture, index write, console argument or reassignment
  disqualifies it. Module scope is never analysed
  (`repr_infer.rs:1476-1481`). `for-of` handles only i64 elements
  (`intrinsics/array.rs:1189`). Nested growable `for-of` is refused (shared
  scratch, `static_analysis/array.rs:1168`). **The index read has no bounds
  check** (`emit_growable_index_read`, `growable.rs:470`).
* **Repr inference** is whole-program (`infer_reprs`, `repr_infer.rs:1462`),
  with top level analysed as the synthetic function `_start`. Array returns are
  recorded in `ReprTable.array_returns` (function → element repr,
  `repr.rs:108`). `ReprTable` keys are `(function, binding)` strings.
* **Memory** comes from `__alloc` (the function arena) or `__alloc_global` (never
  reset), chosen per function by `arena_table.arena_eligible`
  (`emitter.rs:1201-1207`). No per-allocation-site escape bit reaches codegen
  (`MirBinding.escapes` is unread outside `kali_mir`).
* **f64 to string**: the host import `kali:rt.float_to_string`
  (`lower.rs:1084`) has JS `String(number)` semantics.

---

## 3. Design

### 3.1 The growable property

Repr inference gains a **growable** flag on array values. It replaces the
syntactic allowlist as the thing that decides promotion. It is solved over the
same graph that already carries element reprs:

* **Sources.** A binding initialised with an array literal (`[]` or
  `[a, b, …]`) is a **source** when it is pushed, popped or written by index,
  or when it is returned from a function.
* **Rule.** An array value is growable when it is a source, or when it flows
  along an edge below to or from a growable value. A literal array that
  reaches no source keeps every lane it has today, including any existing
  handling when it is passed as an argument. The plan measures those lanes
  before changing anything.
* **Edges.** Growability flows:
  * from a returned value to the function's array-return entry (a growable
    flag beside `array_returns`);
  * from a call result to the binding that receives it;
  * from an argument to the callee's parameter;
  * from an alias's source to the alias (`const b = a`);
  * from `slice`'s receiver to its result (always growable).
* **Unchanged.** A literal array that is only read with literal or
  `const`-folded indices, `.length`, or the existing static lanes still folds
  exactly as today. `new Array(n)` stays a plain runtime array.

`growable.rs`'s allowlist is retired once its cases pass through the new
path, along with its "safe position" scanner and `growable_rejects`. The
`Repr::GrowableArrayI64` object-field lane is unchanged.

### 3.2 Layouts are never mixed

A plain runtime array and a growable array have different layouts. A site
that could see both is refused with E5506, naming the binding or parameter:

* a parameter whose call sites pass a plain array at one site and a growable
  array at another;
* a binding assigned both (`let a = new Array(3); … a = build();`);
* a conditional or logical expression whose arms differ
  (`c ? xs : new Array(2)`).

### 3.3 Memory

* **A growable array that never leaves its creating function** keeps Stage
  4's allocator. "Never leaves" means it is not returned, not passed as an
  argument, not aliased, and not at module scope. That allocator is the
  function arena when the escape gate makes the function arena-eligible.
  `push` inside a loop arena already falls back to global
  (`growable.rs:~236-250`).
* **Every other growable array** allocates its header and its data blocks
  with `__alloc_global`, and is never reclaimed. Growth still doubles the data
  block and abandons the old one.
* Arrays passed to a callee that does not keep them still pay global memory.
  This is conservative, and it needs no per-site escape bit in codegen.
* Module-scope growable arrays are `_start` locals. A function cannot see
  them, so a function body that reads a module-level growable array by name is
  refused (§3.6). A module-level array passed as an argument is fine.

### 3.4 Element reprs

* One element repr per array: i64, f64 or runtime string. It is solved by the
  existing element union-find, which now spans growable edges too.
* f64 elements are stored bit-reinterpreted in the 8-byte slots
  (`i64.reinterpret_f64` / `f64.reinterpret_i64`), as closure cells already do
  (`kali_codegen/src/closure.rs:71-76`).
* A string element is the tagged string handle, stored as-is.
* A mixed array (numbers and strings), or one holding objects, arrays or
  functions, is refused with E5506.

### 3.5 Operations

Each is one emitter on the header layout, branching on the element repr:

| operation | semantics | departure from node |
|---|---|---|
| `a.push(x, …)` | appends each argument in order; the value is the new length | — |
| `a.pop()` | removes and returns the last element | an empty array **traps** with `kali: pop on empty array` (node returns `undefined`) |
| `a.length` | read | writing `.length` is refused (E5506) |
| `a[i]` | read with a runtime integer index | `i < 0` or `i >= length` **traps** with `kali: array index out of bounds` (node yields `undefined`). This also adds the bounds check Stage 4's index read lacks |
| `a[i] = v` | stores for `0 <= i < length` | any other `i` **traps** with the same message, including `i == length`, where node appends |
| `a.indexOf(x)` | first index with `===`, else `-1`. Strings compare by content. NaN is never found | a second (`fromIndex`) argument is refused |
| `a.includes(x)` | SameValueZero: like `indexOf`, but NaN is found | a `fromIndex` argument is refused |
| `a.slice(s?, e?)` | runtime integer bounds, with JS negative-index and clamping rules; returns a new growable array (global) | — |
| `a.join(sep?)` | default separator `","`. i64 elements are formatted as today; f64 elements through `kali:rt.float_to_string`; string elements as today | string elements and a string separator must pass the existing ASCII proof, or the join is refused |
| `for (const x of a)` | i64, f64 and string elements; the length is snapshotted at loop entry | a `push`/`pop` on `a` inside its own loop body is refused (E5506), which makes the snapshot equal to node's live length. Nested loops over growable arrays are admitted, with per-loop scratch locals |

**Console.** A multi-argument `console.log` whose arguments read from
growable arrays (`xs.length`, `xs[i]`, `xs.join()`, `xs.indexOf(v)`) is
admitted, which fixes p3. An argument that is a whole growable array is
refused with the existing "printing a whole runtime array is unavailable"
message, which is widened to cover growable arrays.

### 3.6 Diagnostics and traps

New compile-time refusals are all `E5506` (`FEATURE_UNAVAILABLE`). The texts
live in `crates/kali_common/src/messages.rs`; the exact wording is settled in
the plan. Every refusal names the binding or function and says why:

| refusal | trigger |
|---|---|
| mixed layout | §3.2 |
| mixed or unsupported element repr | §3.4 |
| a function reads a module-level growable array | §3.3 |
| a growable array captured by a closure or arrow | §1.1 |
| push or pop on the array of the enclosing `for-of` | §3.5 |
| `fromIndex` on `indexOf`/`includes` | §3.5 |
| a `.length` write, or an unsupported mutator on a growable array | §1.1 |
| a whole growable array printed by `console.log` | §3.5 |

Runtime traps print their message to stderr and exit 1, as the existing
`kali: array index out of bounds` trap does (`cases/runtime/array_bounds.toml`):

| trap | trigger |
|---|---|
| `kali: array index out of bounds` | a growable index read or write outside `0 <= i < length` |
| `kali: pop on empty array` | `pop()` on an empty growable array |

`check` reports every compile-time refusal that `run` reports. A trap is a
runtime event, so `check` accepts a program that traps under `run`. The case
files pin both outcomes for each trap program.

---

## 4. Approaches not taken

* **B: one growable layout for every runtime array, including
  `new Array(n)`.** There are no mixed-layout refusals. But every access in
  the fixed-size integer kernels (fannkuch and others) gains an indirection,
  and every plain-array lane (bounds, join, return, parameters) must be
  re-validated. It can follow if the mixed-layout refusals bite.
* **C: widen Stage 4's allowlist case by case** (admit `return`, then
  arguments, then module scope). It is the cheapest first step, but the
  allowlist is already the brittle part (p3 fails on a console shape), and
  each new corpus shape would need another rule.
* **Reference counting for escaping arrays.** It would reclaim memory, but it
  is a much larger project (specs/06-memory.md) and is not needed for
  correctness.

---

## 5. Testing and measurement

### 5.1 Unit tests (sibling `*_tests.rs` files)

* **Repr inference:** one test per growable edge (a return, a call result, an
  argument into a parameter, an alias, a `slice` result, module scope), and
  one per refusal in §3.6 that inference raises (mixed layout, mixed element,
  capture, a module-level array read by a function).
* **Codegen emitters,** where the existing growable tests have them: f64
  reinterpret on store and load, the bounds-checked read and write, and `pop`.

### 5.2 Black-box cases

A new `crates/kali_cli/tests/cases/array/growable_runtime_arrays.toml` and a
refusal file beside it. Every expected stdout is node v26.10.0's, in module
scope and inside `function main() { … } main();`:

* p1 to p3 from §2.1;
* f64 arrays (a moving-average style sum, and `join` of floats such as `0.5`, `1e21`
  and `-0`);
* aliasing (`const b = a; b.push(1); console.log(a.length)`);
* an array built in one function, passed to a second that pushes, and read
  in a third;
* nested `for-of` over two growable arrays;
* `indexOf` and `includes`, including NaN and string content equality;
* `slice` with negative and out-of-range bounds;
* `pop` until one element remains;
* a reduced `wrap_paragraph` that splits words from a literal array (no
  string methods);
* each trap (`pop` on empty, an out-of-range read, an out-of-range write)
  under `run`, with the program's `check` outcome alongside;
* one refusal row per §3.6 message, under `check` and `run`.

Existing `new Array(n)`, literal-array and Stage 4 cases keep passing.
A moved pin is compared with node and re-pinned with a dated
`RE-PINNED <date> by the growable-runtime-arrays project` note.

### 5.3 Measurement

* The full `cases` target and `cargo test --workspace`, at `-j 6` and
  `--test-threads=6`, under the resource watchdog, launched with `setsid`.
* `kali check` over the 40 extension programs, with the §2.2 table re-counted
  per message family, before and after. A drop in a family that is not an
  array family is investigated.
* `node accepts.mjs` and `node count.mjs` re-run. If the accept set moves,
  both JSON files are committed and the ranking is re-spliced with a §6
  amendment. If a register lane moves (R-31's array face, any growable row),
  its oracle case is re-pinned and its §0.2 row re-derived.

### 5.4 Bookkeeping

* `specs/19-feature-maturity.md`:
  * the stale row "Computed array subscripts … no `.length`/`push` yet" is
    corrected;
  * a new row names the supported operations, element reprs, layouts and
    memory policy, every refusal and trap, and §1.1.
* `specs/15-errors.md`: no new code. Every refusal is E5506.
* `specs/12-cli.md` and `specs/18-schemas.md` are unchanged.
* `docs/superpowers/followups/growable-runtime-arrays-discovered-defects.md`
  records the residue.

### 5.5 Branching

Worktree `/workspace/.worktrees/growable-runtime-arrays`, branch
`growable-runtime-arrays`, off `main` at `edb3a77df`, with `CARGO_TARGET_DIR`
inside the worktree.
