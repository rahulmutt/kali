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

---

## 7. Amendments

Found while planning (`docs/superpowers/plans/2026-10-07-growable-runtime-arrays.md`), before any code was written, measured at `edb3a77df` with node v26.10.0:

* **A-1.** A literal passed as an argument is refused at the baseline (codegen-only: `passing an array literal to function 'sum'…`; `check` exits 0). That refusal is the lane kept for a literal whose component never becomes growable.
* **A-2.** A source is a component, not a binding: a connected component of array values is growable when it holds a literal-initialized binding and a `push`, `pop` or index write anywhere in it. "Returned" is not a source: a returned literal nobody mutates stays on the array-return lane (`const a = [1, 2, 3]; return a;` runs today).
* **A-3.** Stage 4's position scan is kept as a post-solve check rather than retired outright: a growable array in any position other than the supported ones (bound, passed to a declared function, returned, iterated, `.length`, index, supported method) is refused (`growable_plain_use_message`), because the raw tagged handle would otherwise be printed or computed on.
* **A-4.** A mix of numbers and strings reuses the existing "used as both strings and numbers" refusal. The new element refusal covers objects, arrays, functions, booleans, `null`, `undefined` and holes.
* **A-5.** An array literal written directly as an argument or `return` value in a growable component is refused: bind it to a `const` first.
* **A-6.** Indexing, `push` or `pop` directly on a call or `slice` result is refused; `.length`, `join`, `indexOf`, `includes`, `slice` and `for-of` on them are admitted.
* **A-7.** A growable binding, parameter or return also given a non-array value (another value, `undefined`, a missing argument, a bare `return`, falling off the end) is refused.
* **A-8.** The `for-of` snapshot rule is component-based: a `push`/`pop` in the body on any binding in the iterated array's growable component (aliases included), or a call passing such a binding to a parameter that may be pushed or popped (transitively), is refused. The syntactic same-name rule missed both.
* **A-9.** The console fix is the removal of Stage 4's stale guard (the multi-argument console lane already joins arguments). The guard stays for growable object fields. The whole-array print refusal, codegen-only at the baseline, is raised in inference for growable arrays.
* **A-10.** Nested `for-of` needed per-depth scratch locals and the removal of a codegen-only refusal (`check` accepted it at the baseline).
* **A-11.** Float literals of 2^63 or more inside a runtime expression fail to compile at the baseline (E4201), and `Math.pow(10, 21)` / `10 ** 21` print `3875820019684212736`. The §5.2 `1e21` join case builds 2^70 by doubling instead.
* **A-12.** `float_to_string` already renders `-0` as `0`, which is `join`'s rendering.
* **A-13.** Float `slice` bounds are truncated (`i64.trunc_sat_f64_s`, ToIntegerOrInfinity), not refused.
* **A-14.** The search value counts as an element store: a float needle makes the array f64; a number/string mismatch is the existing mixed-elements refusal.
* **A-15.** Codegen emitters are pinned by black-box cases; the new synthetic bodies by a wasm-validation unit test (§5.1's emitter unit tests had nothing to extend).
* **A-16.** "Never leaves its creating function" is: the binding is the only node of its growable component and is not at module scope.
* **A-17.** The reduced `wrap_paragraph` keeps the width in a number: `line.length` on a reassigned `let` string trips the pre-existing "reassigning an array binding to a non-array value" refusal.
* **A-18.** The existing "reading module binding … from a function" refusal fires for an array literal too, but only in codegen; the new module-read message is raised in inference so `check` agrees.
* **A-19.** `var` array-literal declarators are literal origins like `const` and `let`.
* **A-20.** §1's "37 programs report at least one array-family refusal" measures 36 with the §2.2 families; the other four stop at a non-array refusal first.
* **§3.6 additions.** Messages beyond the eight listed: plain use (A-3), literal expression (A-5), temporary use (A-6), non-array write (A-7); later, an unproven index, bound or search value (A-36), a literal reassignment (A-38), a floating-point index not proven whole (A-39) and an `includes` result kali cannot keep as a boolean (A-40). The exact texts are in `crates/kali_common/src/messages.rs`.

Found during execution (the controller's rulings in the SDD ledger, `.superpowers/sdd/2026-10-07-growable-runtime-arrays/progress.md`, and Task 13's measurement at `417b58fb8` with node v26.10.0):

* **A-21. The element proof is fail-closed (narrows §3.4).** §3.4 refuses objects, arrays and functions as elements; the plan classified a pushed value by its syntax and by what an identifier was declared as, which admitted call, member and conditional results, function aliases, and `let` bindings holding booleans, `null` or `undefined` as i64 (silent miscompiles: Task 7 review I1-I3). The ruling: an element is admitted only when it is provably a number or a string — numeric and string literals, arithmetic, string concatenation and templates, identifiers that pass the identifier check (by their solved scalar repr, W1, not by whether an element node exists), `.length`, index reads and `pop` of a number or string array, and calls whose callee's solved return repr is i64/f64/string and is not boolean-producing. Everything else gets the element refusal, including string-method results (the ruling listed them as admitted; Task 13 measured `out.push("b".toUpperCase())` refused at `417b58fb8`). Why: §1 promises node's answer or a loud failure, and a guess is neither. Cost, measured in Task 13: 16 `array/callback_identity_browser_harness` trials and 8 `misc/for_of_object_keys_iteration` `Object.values(alias)` trials that passed at the baseline are now refused, and a pushed `NaN` identifier or `"b".toUpperCase()` is refused.
* **A-22. The array-return callee key is published, not re-derived (W2).** Inference keys a function's array return through const-arrow aliases (`__kali_fn_N`), while the resolver, codegen and lowering looked callees up by source name. The ruling: one published key, `ReprTable::array_return_callee_key`, which every later stage reads, so `const mk = () => { const o = []; o.push(1); return o; }; for (const x of mk()) …` keeps its capability rather than being refused or miscompiled. What it does not cover: a module-level arrow called from another function is still refused (inference resolves the alias only in the scope that declares it; followups).
* **A-23. An index write evaluates the value before the bounds check (overrides the plan's order).** The plan computed the slot address and then evaluated the right-hand side, so `a[0] = grow(a)` with a right-hand side that grows the array wrote into the abandoned data block (kali `1`, node `99`; Task 9 review C1). The ruling: evaluate the array handle and the index into locals, then the value, then bounds-check, address and store, through a synthetic `__growable_store(arr, idx, val, msg)`. Why: JS evaluation order, and §1's "behave as node or fail loudly", outrank a plan step.
* **A-24. A floating-point index is refused on writes as well as reads, but only by `run`.** A float index on a write produced invalid wasm (E4201; Task 9 review I1); it now gets the Stage 4 read refusal, `indexing a growable array with a floating-point value is unavailable`. Both refusals are raised in codegen, so `check` accepts a program `run` refuses — a known exception to §3.6's "`check` reports every compile-time refusal that `run` reports", recorded in the followups. **Superseded by A-39** (2026-10-08): inference now refuses a float index on a growable binding unless it proves it whole, so `check` agrees.
* **A-25. `pop` with an argument and `slice` with more than two are refused (§3.5 narrowed).** Node evaluates and ignores extra arguments; the plan's emitters dropped them unevaluated, losing their side effects (`a.pop(bump())`; Task 10 review I1). The ruling: refuse them in inference with the unsupported-operation message (`` `.pop()` with an argument ``, `` `.slice()` with more than two arguments ``), so `check` and `run` agree. Why: evaluating ignored arguments in node's order is not worth codegen complexity for a shape no program in the corpus uses.
* **A-26. An f64 element write used as a value is f64.** `const r = (f[0] = f[1] * 2)` failed to load (E4201) because the assignment node was not float-valued; Task 10 added the `is_float_valued` arm for an assignment whose target is an f64 growable element.
* **A-27. A module-scope closure over a growable `for-of` binding is refused (Task 11b; adds a §3.6 row).** A closure or nested function at module scope that captures a binding declared by or inside a loop reads `0` — a pre-existing bug for every loop form, which Task 11 newly exposed for f64, string, nested, call-result and `slice` growable loops. The ruling: refuse, in inference (so `check` agrees), a closure or nested function at module scope that captures a binding declared by or inside a `for-of` over a growable value. The general bug for non-growable loops is left as it was at the baseline and recorded in the followups (`for (let i = 0; i < 3; i++) { const g = () => i; console.log(g()); }` prints `0 0 0`, node `0 1 2`). Why: this lane must not newly admit silent miscompiles, and a general refusal would move many unrelated pins.
* **A-28. `push(-0)` keeps `-0` (Task 11b).** The slot encoder converted an integer-valued literal with `f64.convert_i64_s`, so `a.push(-0)` stored `+0` (`1/a[0]` gave `Infinity`, node `-Infinity`). A negated zero literal, and a unary minus of a float zero, now encodes the f64 `-0` bits. Not covered: `-z` where `z` is an i64 runtime zero still stores `+0` (followups).
* **A-29. A growable string-element return is exempt from the I2 refusal.** The pre-existing "returning X whose elements are strings" refusal blocked §3.5's string returns (Task 6 review); a growable string-element return is now exempt, and the refusal stays for every other string-array return.
* **A-30. A `for-of` loop variable over a plain array or a string carries its item repr.** `const words = ["a", "bb"]; for (const w of words) out.push(w); out.join(",")` passed `check` and printed raw string handles, because the element solve saw the loop variable as i64 (Task 7 re-review). The loop variable's item repr now reaches the growable element repr.
* **A-31. The pre-existing E3200 gate stays; the p1 cases are spelled around it.** `reject_unsupported_string_variable_addition` refuses `line = line + x + " "` (a `var + var` left operand) on `main` too. The plan's p1 cases were rewritten to a node-identical spelling that passes the gate; fixing the gate (resolver plus codegen `+` arm) is outside this lane and is recorded in the followups.
* **A-32. Capability losses this spec's rules cause, measured in Task 13.** All are refusals, never silent: (1) `misc/arena_reclamation_runtime::module_global_store_fails_closed` stored objects into an array literal by index and printed node's `201`; the index write makes the literal growable (A-2) and objects are refused as elements (§3.4) — the controller accepted the loss (Task 6 concern 2); (2) 16 `browser/for_await_object_string_enumeration_browser_smoke` trials push `[key, value]` arrays from `Object.entries` into an array literal and are now refused (§3.4); a reduced program reading only `.length` printed node's `2` at the baseline; (3) the 24 element-proof over-refusals of A-21. Every one is re-pinned to its refusal with a dated note.
* **A-33. Pins the project moved beyond §5.2's list.** 270 trials of the `cases` target changed outcome, all explained by §3.1's sources and edges or by A-21/A-32: 216 now run (each checked against node, or against the fixture's own self-check where it prints nothing), 7 refusals changed message, 41 are refused by A-21/A-32, 4 ran a malformed fixture (below; the fixture was corrected and the cases now agree with node), and 2 are the R-12 oracle cases. **R-12 is FIXED**: `const a=[1,2]; const b=a; b[0]=7;` prints node's `b0=7` in both scopes, because the index write makes the literal growable and the alias joins its component; the register's §0.2 row and the ranking's §3 cell were re-derived (no band moved).
* **A-34. A pre-existing parser leniency surfaced as a moved pin.** The four `misc/set_iteration_runtime::run_supports_set_constructor_iteration_*` fixtures are not valid JavaScript (`'frozen globalThis['Set'] …'` closes the string at `[`; node exits 1 with a SyntaxError). They were refused at the baseline by the literal `.push()` refusal; now the push is growable and kali runs them to exit 0, printing `frozen globalThis[` for the malformed lines. kali accepting `console.log('a['Set'] b')` (printing `a[`) is reproducible at `edb3a77df`. In Task 13's fix round 1 the fixtures' inner quotes were escaped (`[\'Set\']`), which is what the test meant; kali's stdout then equals node's byte for byte and the four cases pin it. The parser leniency itself is in the followups. Likewise kali accepts `for await` inside a non-async function (node: SyntaxError), which two browser smoke-test fixtures rely on.
* **A-35. Corpus measurement (§5.3).** Over the 40 extension programs, `check` error lines fell from 386 to 302. Array families: computed member 86 → 68, for-of 65 → 50, literal-array call 52 → 0, `slice` 22 → 18, `indexOf` 10 → 10, `join` 10 → 7, reassigning an array binding 7 → 7, `concat` 3 → 3, literal-array mutation 1 → 0; one new growable refusal (an object element). Programs with at least one array-family refusal: 36 → 34. Five non-array families grew (`String.prototype.slice` 6 → 7, computed key access 3 → 6, compound assignment 3 → 5, `String.prototype.repeat` 2 → 3, `String.fromCharCode` 2 → 3); each new line sits inside a loop body or function whose `for-of`/`push` refusal used to stop resolution before it, so they are pre-existing refusals now reached, not new ones (e.g. `counts[word] += 1` inside `for (const word of words)` in `word_frequency.js`). The accept set did not move: anchor 125/137, extension 0/40.

Found by the final whole-branch review (`.superpowers/sdd/2026-10-07-growable-runtime-arrays/final-fix-brief.md`), fixed 2026-10-08 and measured against node v26.10.0:

* **A-36. An index, a `slice` bound and a search value must be proven (final review C1; narrows §3.5).** These operands reached codegen unproven, where only a float index or bound was handled and the search value was encoded as an element slot. With `a` a growable `[1, 2, 3]`, `check` exited 0 and `run` gave silent wrong answers: `a.includes(true)` printed `true` (node `false`), `a[true]` printed `2` (node `undefined`), `let i; a[i] = 7` wrote slot 0, `a.slice(1, undefined)` printed nothing (node `2,3`) and `a.slice("1")` all three elements (node `2,3`). The ruling: refuse, do not coerce. The fact walk records every index (read or write), both `slice` bounds and the `indexOf`/`includes` search value with its element proof (A-21), and repr inference discharges it: an index or a bound must be proven a number, a search value a number or a string. Everything else (a boolean, `null`, `undefined`, a string, an array, an object, an unproven value) is refused with E5506, `growable_unproven_operand_message`, under `check` and `run` alike. A search value of the wrong kind for the array (a string searched in a number array) keeps the mixed-elements refusal (A-14). A float index or bound is a number and stays admitted by inference: a bound is truncated (A-13) and an index is refused by `run` (A-24). The element proof now proves `x += v` as `x + v` (it took any compound assignment as a number, which a string `x` is not). Costs: a string-literal index such as `a["1"]` is refused by `check` (node reads index 1; the write form was refused only by `run` before); a `NaN` or `Infinity` identifier as an index or search value is refused with this message, which wrongly says it is not proven a number (it used to fail at load with E4201); and an identifier index is refused in a program where the proof's program-wide `unwalked` flag is set (A-21's callback-in-a-spread case). Why: §1.
* **A-37. A binding initialised by a call is an array only when its callee returns one (final review I2).** The element proof counted every binding initialised by a bare call as an array, so `const v = sq(i); out.push(v)`, with `sq` returning a number, was refused with the element message (node `0 1 4 9`), while `let v = 0; v = sq(i)` and `out.push(sq(i))` were admitted. A call origin now counts as an array only when its callee, resolved like the published callee key (A-22), is array-returning or returns a growable array. A shadowed callee (a parameter or a local) still counts as one, which refuses. Any other callee leaves the binding to the scalar proof, which refuses an unknown callee. A `const` bound to a call that returns a growable array, pushed as an element, stays refused.
* **A-38. A growable binding reassigned from an array literal has its own message (final review minor 6; a §3.6 row).** `let a = []; a.push(1); a = [];` was refused with A-5's "an array literal written directly as a call argument or `return` value" text, which does not describe it. It is still refused, because the literal joins the growable component, but now with `growable_literal_assignment_message` ("assigning an array literal to a binding that holds a growable array … declare a new `const` for the new array instead"; node `1 2`).

Residual fixes R1 and R2, authorized by the user on 2026-10-08 (`.superpowers/sdd/2026-10-07-growable-runtime-arrays/residual-fix-brief.md`), measured against node v26.10.0:

* **A-39. A floating-point index from `Math.floor`, `Math.ceil`, `Math.trunc` or `Math.round` is truncated; any other float index is refused by inference (residual R2; supersedes A-24).** `xs[Math.floor(x / 2)]`, `xs[Math.floor(xs.length / 2)]` and `const m = Math.floor(xs.length / 2); xs[m]` passed `check` and failed to load under `run` (E4201). The cause was not the index lane: `Math.floor`/`ceil`/`trunc`/`round`/`abs`/`min`/`max` had only integer lanes, so with a float operand the call left an f64 where an i64 was promised, anywhere (`console.log(Math.floor(x / 2))` failed the same way at the baseline). The ruling, in two parts. (1) These seven calls get an f64 lane: with a float first argument (for `min`/`max`, any float argument) the result is an f64, through `f64.floor`/`ceil`/`trunc`/`abs`/`min`/`max` and JS's `Math.round` (`floor(x)`, plus one when `x - floor(x) >= 0.5`, with the sign of `x`), so NaN, the infinities and `-0` survive as in node; repr inference gives the result a float edge from the same arguments (`repr_infer`'s `Math` arm, codegen's `math_float_lane`), except a signed numeric literal argument, which keeps the constant-fold lane. Only the bare `Math.<method>` spelling takes the lane. (2) A float index on a growable binding must be proven a whole number (`growable::values::integral_proof`): a rounding call, an integer literal, `.length`, `+ - * %` and unary sign over whole values, `Math.min`/`Math.max`/`Math.abs` of whole values, a non-f64 binding, element or return, or an f64 binding every write of which is whole (a binding cycle such as a binary search's `lo = mid + 1` holds by induction; a parameter, a loop or `catch` variable or a declarator without an initializer does not). Inference refuses any other float index with E5506 (`growable_fractional_index_message`), under `check` and `run` alike; codegen converts an admitted one with `i64.trunc_f64_s`, which traps on NaN and the infinities (node reads `undefined`; kali fails loudly, as for any out-of-range index). A float index on a growable object field keeps the codegen refusal. `slice` bounds needed no change beyond (1): A-13's truncation is correct for any float. Costs: `xs[x / 2]` (node `undefined`) is now refused by `check` too; `Math.sign`, `Math.imul`, `Math.clz32`, `Math.pow`, `**` and `%` with a float operand still fail at load with E4201 (pre-existing, outside the index lane; followups).
* **A-40. A growable `includes` result keeps its boolean value, or is refused (residual R1).** `const r = xs.includes(3); console.log(r)` printed `1` (node `true`), `typeof r` gave `0` (node `boolean`), and `function has(xs, v) { return xs.includes(v); }` returned `1`. kali has no boolean repr: a boolean is an i64 `0`/`1` whose shape codegen knows only while it holds the expression, so a binding or a return lost it. (The same holds outside this lane, e.g. `let c = n < 2; console.log(c)` prints `1`; the brief's premise that comparisons and string `includes` results keep the shape when stored does not hold at the baseline; followups.) The ruling: a syntactic walk (`growable::values`) records every write of every binding and every `return` of every declared function, and every occurrence of a value that may be a search result — an `includes` call on a growable receiver, a read of a binding, a call of a declared function — with its position. A binding is a *search boolean* when every write is a growable `includes` result, a boolean literal, a comparison, `!x`, another search-boolean binding or a call of a search-boolean function, and at least one write is not merely syntactic; a declared function likewise for its `return`s (it must not fall off its end). These are published in `ReprTable::{binding_is_search_boolean, return_is_search_boolean}`; codegen gives such a read (local or module global) or call the boolean shape, and `typeof` of it, of a direct `includes` call or (for `indexOf`) a number gives `"boolean"` (`"number"`), and `===`/`==` classify it as a boolean (`r === 1` is `false`). A search value is kept in a condition, under `!`, in a discarded statement, as a `console` argument, an operand of `+` (concatenation or arithmetic), a template, `typeof`, a comparison, an arithmetic, bitwise or unary-sign operand, or written to a search-boolean binding or returned from a search-boolean function. Anywhere else — a call argument (`String(r)` included), an array or object element, a member or index write, a `&&`/`||`/`??`/`?:` value outside a condition, an assignment used as a value, a capture by a closure or nested function, a binding or `return` that also holds something else (`let r = 0; r = xs.includes(3)`), a `switch`, a `throw`, a `return` from an arrow or function expression — inference refuses it with E5506 (`growable_search_result_use_message`), under `check` and `run` alike. A binding with a compound assignment, an update, a parameter, a loop or `catch` variable, or a scope a class, `with` or module syntax sits in, is never a search boolean. `indexOf` is unchanged and stays a number. Not covered: a module-scope search boolean read from a function hits the pre-existing codegen-only "reading module binding … from a function" refusal (`check` exits 0).

Residual round 1 (review of A-39/A-40, 2026-10-08):

* **A-41. A rounding call over a compile-time number keeps the integer fold (amends A-39).** A-39 exempted only a literal argument, so `const t = 7.9; Math.floor(t) % 3` (and `| 0`, `<< 1`, `** 2`, `new Array(Math.floor(t))`, a `Math.ceil` loop bound, `Math.round(t)` passed to a `%` function) became f64 and failed at load or was refused while `check` passed, where `main` printed node's value. Now a literal, a unary sign or `Object.freeze` of one, or a `const` alias chain ending in one (`repr_infer::is_static_numeric`, codegen `is_static_numeric_arg`) gets no float edge and takes the i64 fold, on both sides. A module `const` of a literal rounded inside a function now folds too (`math_round_like_static_literal_value` reads its initializer); it failed to load at the baseline. A-39's whole-number proof no longer admits `%` (a float `%` does not lower), so `xs[i % 2]` with a float `i` is refused by inference.
* **A-42. `!` and comparisons over an `includes` result are search booleans (amends A-40).** `const r = !xs.includes(2)`, `let r = !xs.includes(2); r = !r`, `xs.includes(2) === false`, `xs.includes(2) == xs.includes(3)` and `return !xs.includes(v)` printed `0`/`1` once stored or returned: `!` and comparisons counted as syntactic booleans, and the `includes` beneath them sat in a kept position, so nothing was refused and nothing got the boolean shape. A `!` or a comparison whose operands include a possible search value is now `BoolValue::Over` those operands: always a boolean, and derived (a search boolean) when an operand is. Its occurrences follow A-40's positions, so `show(!xs.includes(2))` is refused. The same shapes outside the growable lane (`const r = ![1, 2].includes(2)` prints `0`) are unchanged (followups).

Residual round 2 (2026-10-08):

* **A-43. One reach for "compile-time number"; `!` over logical forms; `typeof` of a boolean expression (amends A-41, A-42).** (1) Codegen followed one module-`const` hop where inference (`is_static_numeric`) follows the whole chain, so `const t = 7.9; const u = t; function f() { return Math.floor(u); }`, `const b = -a` and a local `const t2 = t` of a module const passed `check` and failed at load. Codegen's `static_numeric_chain` (`intrinsics/math.rs`) now follows local fold aliases and module `const` chains (module initializers resolve at module scope only), through unary signs, transparent wrappers and `Object.freeze`, depth-bounded, and is the single codegen definition: the float-lane test and the `floor`/`ceil`/`trunc`/`round`/`abs` folds read it. A shared AST/LIR definition was not feasible (inference walks the AST before lowering); the two are kept to the same forms. (2) `!` over a `||`, `&&`, `??`, `?:` or `,` expression holding a search value is `Over` the search values inside it (looked through recursively), so it is a search boolean when stored or returned. A comparison over such an expression stays refused (its operand is a value position). (3) `typeof` of `!x` or of a comparison is `"boolean"` (it printed the placeholder `0`; general, not growable-only). Not changed: a float `%` in a `slice` bound still fails at load (followups).

Residual round 3 (2026-10-08):

* **A-44. Inference proves, codegen follows; commas and unary signs over a search value are refused (amends A-41-A-43).** (1) Codegen's fold also went through `for-of` variables the static-unroll lane binds to literal elements, which inference did not treat as compile-time numbers, so `for (const x of [1.5, 2.5]) { const k = Math.floor(x); … k % 2 }` passed `check` and failed at load. Inference now treats a `const` loop variable of a `for-of` over a literal array (or a non-growable `const` binding of one) whose every element is a compile-time number as one, and publishes every binding it proves (`ReprTable::binding_is_static_numeric`); codegen's `static_numeric_chain` follows an identifier only when inference published it, so the two agree by construction. A name declared twice in one scope (sibling blocks or loops), or a `let` loop variable, is never one. Both sides bound the chain by one constant, `kali_common::STATIC_NUMERIC_CHAIN_DEPTH` (1024). (2) Any `,` expression holding a growable search value is refused: kali's comma value is not JS's at the baseline (`console.log((1, 5))` prints `2`, `!(0, 5)` prints `true`, `!(xs.includes(1), xs.includes(5))` prints `true`; node `5`, `false`, `false`), so the coordinator's "admit when every operand is a search value" form is refused too. (3) Unary `+`/`-` over a search value is refused (`+r` printed `false`, node `0`). Binary `+` is unchanged (concatenation and arithmetic print node's values).
