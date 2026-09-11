# An array allocation in value position is a real array, or a refusal

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `733cd26125` (main, clean tree, the merge of PR #43) |
| kali binary | `kali 0.1.0`, `.cache/cargo-target/debug/kali`, rebuilt at the baseline commit |
| oracle | `node v26.8.2` |
| measured on | 2026-09-11 |
| item picked | `docs/superpowers/followups/length-fails-closed-discovered-defects.md` §1, "An inline allocation passed as an argument reaches the callee as zeros" |
| defects this closes | §1 of that document, widened to every lane §2.2 measures; a fold-lane array argument reading zeros (§2.4); the `[new Array(n)]` collision (§2.3); `.fill(v)` re-evaluating `v` per element (§2.6) |

**The picked item is one lane of a wider class, and the human partner chose the
wider scope.** §1 of the length-fails-closed defects document names the call
argument. The same zero comes back in a return, a ternary arm, a nested call and
a second argument, and a second, unfiled class sits beside it: any fold-lane array
passed to a user function reads as zeros in the callee (§2.4).

**Citation convention.** Every line reference is as of the baseline commit above.

**Spike convention.** One throwaway patch (§2.5) was applied to the working tree,
measured with the full workspace suite (`cargo test --workspace --no-fail-fast`,
the command `scripts/test-gate.sh` runs) and with 78 node-oracle probe programs,
then reverted with `git checkout`. It was not committed. The baseline suite at
`733cd26125` reported **12032 passed, 0 failed, 27 ignored**, so every failure
counted below is new.

---

## 1. What this project is

**The claim:** in `kali_codegen`, an `Array` / `Uint8Array` allocation in value
position evaluates to a real array handle. An array kali cannot pass as a handle,
passed to a user function, refuses with `E5506`. `.fill(v)` evaluates `v` once.

Four changes, two crates (§3). Four register entries, each filed and retired by
this project (§6.1). One followup document filed (§6.3).

### 1.1 How it was chosen

1. The picked item's mechanism was traced to the aggregate placeholder, which
   every lane without a materializing arm reaches (§2.1).
2. Probing the lanes showed the class is wider than the item and has a sibling
   the item's guard misses (§2.2, §2.4).
3. The obvious compute move is unsound on its own, because `new Array(3)` and
   `[Array(3)]` are the same LIR node, and the declarator lane already
   miscompiles the second (§2.3). The human partner chose to break the collision
   at the AST, ahead of codegen.
4. A spike of the compute move plus that refusal turned 36 of the 47 probes that
   were silent at the baseline correct (18) or refusing (18), with one silent
   regression whose mechanism is traced (§2.5, §2.6).

### 1.2 What this project does NOT claim

* It does **not** make an allocation correct once it crosses a return into a
  binding, or once it is stored in an object property. Those lanes read zeros
  after the spike too, and belong to R-14 and R-48 (§5).
* §3.3's scratch discipline is argued from source, not measured. The spike
  measured the regression it removes, not the fix. §4.1 makes that measurement a
  stop gate.
* It does **not** close `(new Array(3)).length` → `1`
  (`length-fails-closed-discovered-defects.md` §3). That value comes from
  `render_length`'s array-literal arm, which never reaches `emit_value`.

---

## 2. The defect, measured

### 2.1 The mechanism: every lane without an arm reaches a zero

An allocation gets linear memory in exactly three places:

* the declarator lane (`emit/control_flow.rs:1690` for `new Array(n)`, `:1711` for
  `.fill`);
* the assignment lane, `a = new Array(n)` (`emit/literal.rs:1043`);
* the receiver of `.fill` (`emit/call.rs:6004`).

Anywhere else the node is a text-less `Value`, and `emit_value`'s text-less branch
ends at `emit_aggregate_literal` (`emit/control_flow.rs:2181`), which drops every
child and pushes `I64Const(0)` (`emit/literal.rs:13-45`). A bare `Array(n)` call
with no `new` reaches `emit_call`, which has no allocation arm, and also produces
`0`.

The callee side is not at fault. A parameter becomes an array binding from how the
callee uses it (`emitter.rs:649-662`), which is why `f(a)` with a bound
`const a = new Array(6)` prints `6`. The handle an inline allocation delivers is
`0`, so the callee reads a length header and elements at address zero.

The one guard on this lane (`emit/call.rs:3573-3615`) refuses an array-literal
argument only when every element is a `Literal`. Its comment at `:3579-3582`
assumes materialized arrays "live in locals … and pass a real handle, so they are
untouched here". That holds for a bound allocation and is false for an inline one.

### 2.2 Silent lanes: an allocation in value position

Measured at `733cd26125`, kali at exit 0 with empty stderr on every row:

| program | kali | node |
|---|---|---|
| `function f(x) { return x.length; } console.log(f(new Array(6)));` | `0` | `6` |
| the same inside `function main() { … } main();` | `0` | `6` |
| `function f(x) { return x[0]; } console.log(f(new Array(3).fill(2)));` | `0` | `2` |
| `function f(x) { let s = 0; for (let i = 0; i < x.length; i++) { s = s + x[i]; } return s; } console.log(f(new Array(5).fill(2)));` | `0` | `10` |
| `function f(a, x) { return a + x[0]; } console.log(f(1, new Array(3).fill(9)));` | `1` | `10` |
| `function f(x, y) { return x.length + y.length; } const a = new Array(2); console.log(f(a, new Array(3)));` | `2` | `5` |
| `function mk() { return new Array(3).fill(4); } function f(x) { return x[2]; } console.log(f(mk()));` | `0` | `4` |
| `function f(x) { return x.length; } function g(y) { return f(y); } console.log(g(new Array(7)));` | `0` | `7` |
| `function f(x) { return x[0]; } const c = true; console.log(f(c ? new Array(2).fill(7) : new Array(2).fill(8)));` | `0` | `7` |
| `function main() { const n = 4; function f(x) { return x.length; } console.log(f(new Array(n))); } main();` | `0` | `4` |
| `function f(x) { return x.length; } console.log(f(Array(6)));` | `0` | `6` |

Controls, correct at the baseline and required to stay correct: a bound argument
(`const a = new Array(6); f(a)` → `6`); `let` and `const` `.fill` declarators in
both scopes; `new Array(3); console.log("ok")`; an inline allocation the callee
ignores (`f(new Array(3))` returning `1`).

### 2.3 The collision: `new Array(3)` and `[Array(3)]` are one node

The parser reads `new Array(3)` as `new (Array(3))`: a `NewExpression` whose callee
is the `Array(3)` call (`crates/kali_parser/src/expression/primary.rs:241`). HIR
keeps `NewExpr` and `ArrayExpr` apart; MIR erases both to `MirNodeKind::Expr`
(`crates/kali_mir/src/lower.rs:108`, `:116`). In LIR both are a text-less `Value`
with one `Call(Array, 3)` child, and `resolve_array_alloc_call`
(`emit/call.rs:5464`) unwraps either.

The declarator lane already uses that recognizer, so the collision is live today:

| program | kali | node |
|---|---|---|
| `const xs = [new Array(3)]; console.log(xs.length);` (both scopes) | `3` | `1` |
| `const xs = [Array(3)]; console.log(xs.length);` | `3` | `1` |
| `const xs = [Array(3).fill(1)]; console.log(xs.length);` | `3` | `1` |
| `function f(x) { return x.length; } console.log(f([new Array(3)]));` | `0` | `1` |

No committed test, benchmark fixture or example contains a one-element array
literal of an allocation (`grep -E "\[\s*(new\s+)?(Array|Uint8Array)\("` over
`crates/kali_cli/tests`, which holds every committed test and benchmark fixture: 0 hits). Nothing has filed it.

Routing allocations in value position without breaking this collision would carry
the declarator's `3` into every lane. §3.1 breaks it.

### 2.4 Silent lanes: a fold-lane array passed to a user function

The guard's all-`Literal` condition lets every other fold-lane array through to
the placeholder:

| program | kali | node |
|---|---|---|
| `function f(x) { return x[0]; } const k = 3; console.log(f([k]));` (both scopes) | `0` | `3` |
| `function f(x) { return x[0]; } console.log(f([1 + 1]));` | `0` | `2` |
| `function f(x) { return x[0] + x[1]; } const k = 3; console.log(f([k, 1]));` | `0` | `4` |
| `function f(x) { return x[0]; } const k = 3; const arr = [k]; console.log(f(arr));` | `0` | `3` |
| `function show(v) { console.log(v.length + "," + v[0] + "," + v[1]); } const fe = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]); const collected = [...Object.values(fe)]; show(collected);` (both scopes) | `0,0,0` | `2,3,2` |
| `function show(e) { console.log(e.length + "," + e[0][0] + "," + e[0][1]); } function main() { const alias = {b: 1, a: 2}; const entries = Object.entries(alias); show(entries); } main();` | `0,0,0` | `2,b,1` |
| `function show(k) { console.log(k.length + "," + k[0] + "," + k[1]); } function main() { const o = {b: 1, a: 2}; const keys = [...Object.keys(o)]; show(keys); } main();` | `0,0,0` | `2,b,a` |
| `function f(x) { return x[0]; } function g() { return 5; } console.log(f([g()]));` | `0` | `5` |
| `class C { constructor() { this.v = 4; } } function f(x) { return x.v; } console.log(f(new C()));` | `0` | `4` |

Five browser case files program exactly the `Object.values` / `keys` / `entries`
rows and have passed only because their assert helpers throw: the bundle builds,
then the harness exits non-zero with `Uncaught Error`. The same program with the
throw replaced by a print is silent (the table above).

### 2.5 The spike

Four edits, `kali_codegen` and `kali_types`:

1. `emit_value`'s text-less branch, before `emit_aggregate_literal`:
   `resolve_array_alloc_call(id)` → `emit_array_allocation`; a single child
   `resolve_array_fill_call` accepts → `emit_node` of that child.
2. `emit_call`, after the computed-member deny: `resolve_array_alloc_call(id)` →
   `emit_array_allocation`.
3. The argument guard: refuse any non-empty fold-lane array literal argument that
   is not an allocation, dropping the all-`Literal` condition.
4. `resolve_expression`'s `ArrayExpression` arm: refuse a one-element literal
   whose element is an `Array` / `Uint8Array` allocation, with or without `new`,
   parentheses or `.fill`.

**Probes.** 78 programs, classified against node:

| | CORRECT | REFUSED | SILENT |
|---|---|---|---|
| baseline | 13 | 18 | 47 |
| spike | 30 | 36 | 12 |

Of the 12 still silent, 11 are lanes §5 excludes (R-14, R-48, the parser's `new`
precedence, `Array.from({length})`). **One is a regression**, §2.6.

**Suite.** 11951 passed, 81 failed, 27 ignored. Every new failure:

| family | failures | what moved |
|---|---|---|
| `browser/object_entries_iteration` | 16 (4 cases × 4 ext) | build `exit = "success"` → E5506 |
| `browser/object_keys_entries_spread_bundle` | 8 (2 × 4) | same |
| `browser/object_keys_entries_spread_harness` | 32 (8 × 4) | `Uncaught Error` needle → E5506 |
| `browser/object_values_spread_bundle` | 8 (2 × 4) | build `exit = "success"` → E5506 |
| `browser/object_values_spread_harness` | 16 (4 × 4) | `Uncaught Error` needle → E5506 |
| `soundness/abort::abort_handle_inline_new_in_arg_position` | 1 | a working program now refuses |

The 80 browser failures are re-pins of programs that never ran correctly (§2.4).
The abort case is the one capability loss:
`function f(x) { return 1; } f(new AbortController()); console.log("ok");`
prints `ok` today, and `new AbortController()` is the same LIR shape as
`[AbortController()]`. **The human partner ruled the wide guard and a re-pin of
this case to a refusal** over narrowing the guard, which would have left
`f([g()])` and `f(new C())` silent.

### 2.6 The regression, and the pre-existing defect under it

| program | baseline | spike | node |
|---|---|---|---|
| `const a = new Array(2).fill(new Array(3)); console.log(a.length);` | `2` | `3` | `2` |
| the same inside `function main() { … } main();` | not measured | `3` | `2` |
| `function f(x) { return x.length; } const a = new Array(2).fill(f(new Array(5))); console.log(a.length + "," + a[0]);` | not measured | `5,0` | `2,5` |

`emit_array_fill` (`emit/call.rs:5991`) keeps the array's base handle in scratch
slot `locals.len()` and the loop counter in `locals.len() + 1`, then emits the
value inside the loop. `emit_array_allocation_with_len` (`:5567`) writes the same
two slots. Before the spike a fill value was never an allocation, so nothing
enclosing held those slots. Now the value's allocation overwrites the base handle.
**36 sites** in `kali_codegen` use slot `locals.len()` (`control_flow.rs` 9,
`call.rs` 7, `operators.rs` 6, `object.rs` 4, `literal.rs` 4, `closure_access.rs`
3, `url.rs` 2, `growable.rs` 1); any that emits a child while holding it can be
clobbered the same way once that child allocates.

Probing that shape found a silent defect already present at the baseline:
**the loop re-emits the value for every element.**

| program | kali | node |
|---|---|---|
| `let n = 0; function g() { n = n + 1; return 1; } const a = new Array(3).fill(g()); console.log(n + "," + a[2]);` | `3,1` | `1,1` |
| the same inside `function main`, length 4 | `4,1` | `1,1` |
| `… const a = new Array(3).fill(0); a.fill(g()); console.log(n + "," + a[1]);` | `3,2` | `1,2` |

The human partner chose to fix both with one change (§3.3).

---

## 3. The design

### 3.1 `kali_types`: a one-element array literal of an allocation refuses

In `resolve_expression`'s `ArrayExpression` arm (`resolve/expression.rs:1862`),
push `E5506` when the literal has exactly one element, that element is an
expression, and the expression is an array allocation. An allocation is, after
removing parentheses and `await`:

* a call to `Array` or `Uint8Array` with at most one argument;
* a `new` of one, in either parser shape (`new (Array(n))` with no arguments of
  its own, or `new Array` with its arguments on the `NewExpression`);
* `<allocation>.fill(v)` with exactly one argument.

The callee spellings must be exactly the ones `is_array_like_constructor`
(`emit/call.rs:5510`) accepts, including `globalThis["Uint8Array"]`, and the doc
comment says so, the way `declarator_init_is_array_alloc` (`lower.rs:6535`) records
its own lockstep with that recognizer. The message names the shape and the reason:
the literal lowers to the same node as the allocation, so kali cannot tell them
apart.

After this, a text-less `Value` whose unwrapped child `resolve_array_alloc_call`
accepts is a `new`, a parenthesis or an `await`, never a literal. §3.2 relies on
that. `kali check` refuses what `kali run` refuses.

### 3.2 `kali_codegen`: allocations in value position allocate

* `emit_value`'s text-less branch (`emit/control_flow.rs`, before the
  `emit_aggregate_literal` call at `:2181`): a node `resolve_array_alloc_call`
  accepts is emitted by `emit_array_allocation`; a node whose single child
  `resolve_array_fill_call` accepts is emitted as that child, reaching
  `emit_call`'s fill arm (`emit/call.rs:1037`).
* `emit_call` (after the computed-member deny at `emit/call.rs:100-102`): a node
  `resolve_array_alloc_call` accepts is emitted by `emit_array_allocation`.

The declarator, assignment and receiver lanes intercept earlier and are unchanged.
An inline `.fill` has no binding name, so its elements use the default `I64` repr.
A float fill passed to a function already fails to load (E4201) when bound, so this
adds no float lane (§5).

### 3.3 Allocation and fill own their scratch slots, and fill evaluates once

Three trailing i64 locals are reserved for every function lowered from LIR (the
`else` arm at `lower.rs:1533`; the hand-emitted synthetics above it declare their
own), after the two that `lower.rs:1442-1455` reserves today. Only `emit_array_allocation_with_len` and
`emit_array_fill` use them:

* **handle** (the new array's base, or the fill receiver's base)
* **count** (the allocation's length, or the fill loop counter)
* **value** (the fill value; for an `F64` element repr, its bits via
  `I64ReinterpretF64`, restored with `F64ReinterpretI64` at the store)

The invariant, stated in both functions' doc comments: **every child is emitted
before the first write to a dedicated slot, and between that write and the
function's last read only loads, stores, arithmetic and constants are emitted.**

* Allocation already emits the size before writing; only its slot indices move.
* Fill emits the receiver (allocating it if `resolve_array_alloc_call` accepts it,
  otherwise loading its handle), then the value, both onto the operand stack, then
  sets **value** and **handle**, zeroes **count**, and runs the loop, loading
  **value** at each store. Receiver before value is JavaScript's order. Integer to
  float promotion happens once, before the value is stored.

Consequences: a nested allocation or fill anywhere in a receiver or value finishes
before the outer one writes a slot, so §2.6's clobber cannot happen; neither
function writes `locals.len()` or `locals.len() + 1` any more, so none of the other
sites holding those slots can be clobbered by an allocation or a fill; `v` is evaluated once (§2.6's second table).

The reservation is unconditional. A conditional one would need a lowering-time
walk kept in lockstep with the emit recognizers, and a miss would be invalid wasm.
Adding local declarations changes no instruction count; §4.1 measures whether it
moves anything else.

### 3.4 The argument guard refuses every non-allocation fold-lane array

At `emit/call.rs:3583-3615`, the guard refuses when the argument is not an
allocation (neither `resolve_array_alloc_call` nor `resolve_array_fill_call`
accepts it) and `resolve_literal_aggregate` yields a non-empty `is_array_literal`
node. The all-`Literal` condition goes. The existing message stays as a prefix, so
every current needle still matches, and a clause is appended saying that a
constructed value (`new C()`) is refused too, because it lowers to the same node.
The stale comment at `:3574-3590` is rewritten to state §2.1's actual facts.

### 3.5 Rejected alternatives

| alternative | why not |
|---|---|
| Narrow `is_array_literal` so `new` shapes are not literals | Fail-open in codegen: the false branch reaches placeholders. Measured by the length-fails-closed spike, 3 suite tests and 5 probes silently wrong (`length-fails-closed-design.md` §2.2) |
| Hoist inline allocations into temporaries at lowering | Moves evaluation out of ternary arms, `&&`/`||` operands and loop tests, and spans several crates |
| Carry a `new` marker from HIR into LIR | Correct by construction, but changes what 16 transparent-wrapper matchers and 23 text-less checks in `kali_codegen` see. Must be sized by recognizer (`release-tier-allocation-identity-discovered-defects.md` §9); its own project |
| Leave the one-`Call`-child shape out of §3.4 | Keeps `f(new AbortController())` working, leaves `f([g()])` and `f(new C())` silent. The human partner ruled the wide guard |
| Refuse inline allocations instead of computing them | Leaves the capability absent. The human partner chose compute |

---

## 4. Verification

### 4.1 Task order and stop gates

Order: the generator question; the §4.2 pins, red; the R-64..R-67 filings; §3.1;
§3.3; §3.2; §3.4 with §4.3's re-pins; the retirements and followups.

1. **Before any production change: the generator question.** Four of the five
   browser case files are generated: `object_entries_iteration.toml` by
   `tools/task-18-browser-pilot/gen_batch6a.py`, the other three by `gen_batch7b.py`,
   with `batch7b_captures.py`, `batch5_crosscheck.py` and
   `scripts/audit-case-migration.py` reading some of them. Neither generator runs
   from `scripts/test-gate.sh --gates-only` or CI. The first task runs each in
   check mode (or, lacking one, into a temporary directory and diffs) to learn
   whether the shipped files are still a fixed point. If they are, §4.3's re-pins
   go through the generators; if not, the re-pins are hand edits and each file's
   header records the divergence from its generator. Either way the plan records
   the route before editing.
2. **After §3.3 lands, before §3.2:** the suite diff against the baseline contains
   only the fill-once pins of §4.2 turning green. Any other test moving, including
   from the reservation alone, stops the plan and goes back to the human partner.
3. **After §3.2 lands:** every §4.2 compute and clobber pin is correct. **Any
   silent result stops implementation.** The design goes back to the human partner
   before another line is written.

### 4.2 New pins: `crates/kali_cli/tests/cases/runtime/inline_allocation_value_position.toml`

Every case records node's value in its rationale. Both scopes wherever the shape
allows.

* **Must compute** (`exit = "success"`, exact stdout): every row of §2.2's table;
  a size taken from an allocation (`new Array(f(new Array(3)))` → `3`); two
  allocating arguments; an allocating argument inside string concatenation, a
  field store, `+=`, an element store and a loop.
* **Must compute, clobber shapes** (§2.6): `new Array(2).fill(new Array(3))` → `2`;
  `.fill(f(new Array(5)))` → `2,5`; a ternary with an allocating arm inside a fill
  value → `4,2`.
* **Must evaluate once:** §2.6's three side-effect programs.
* **Must refuse** (`exit = "failure"`, `stderr_contains` names the message): §2.3's
  rows (§3.1's message); §2.4's rows, including the bound `const arr = [k]`
  argument and `f(new C())` (§3.4's message).
* **Must stay correct:** §2.2's controls, and a two-element literal of allocations
  (`[new Array(3), new Array(2)].length` → `2`).

Order: must-refuse and must-compute pins are committed red against the baseline
before any production change. Controls go in green.

### 4.3 Existing tests that change, and why

| test | change | reason |
|---|---|---|
| the 16 failing cases of `browser/object_entries_iteration.toml` | build step `exit = "success"` → `"failure"` with an E5506 needle; the steps after a failed build go | their programs pass a fold-lane `Object.entries` result to a user function, silent without the throw (§2.4) |
| `browser/object_keys_entries_spread_bundle.toml`, `object_values_spread_bundle.toml` (8 each) | same | same, `[...Object.keys/values(o)]` |
| `browser/object_keys_entries_spread_harness.toml` (32), `object_values_spread_harness.toml` (16) | `Uncaught Error` needle → the E5506 message | same |
| `soundness/abort::abort_handle_inline_new_in_arg_position` (`cases/soundness/abort.toml:890`) | `exit = "success"` / `ok` → `exit = "failure"` with §3.4's needle | the human partner's wide-guard ruling (§2.5); the rationale records that the program is correct in node |
| `array_literal_arguments_benchmark_is_rejected_fail_closed` (`runtime_smoke/misc.rs`), `array-literal-arguments-benchmark-v1` (`inprocess/benchmark_execution.rs:915`) | none | must pass unchanged; §3.4 keeps the message prefix |

Routes per §4.1's first gate.

### 4.4 Oracle cases

Each entry of §6.1 gets a module-scope and in-function pair in
`crates/kali_cli/tests/cases/oracle/tier2.toml`, with the file's header index
(`:14`, `:30`, `:122`) extended the way R-63's pair was. Each pair is filed
asserting `silent` against the baseline, then flipped in the commit that fixes it:
**R-64 and R-67 to `fixed`, R-65 and R-66 to `fail_closed`.** Enforced by
`kali_blast_radius::oracle_tests::every_zero_two_row_is_the_class_set_its_live_cases_assert`.

### 4.5 The gate

* `bash scripts/test-gate.sh` after every task.
* `bash scripts/test-gate.sh --gates-only` after any task that edits a
  `cases/browser/` file: `citation_sweep.sh`, `batch5_crosscheck.py` and the
  migration-audit test read those files.
* `cargo clippy --workspace -- -D warnings` and `cargo fmt --all -- --check`
  before pushing. The gate runs neither.
* **Final check:** diff the failing-test set against the baseline's empty set,
  and the passing set against the baseline's. Every change must be one listed in
  §4.2, §4.3 or §4.4. Any other test moving from pass to refusal goes back to the
  human partner.

---

## 5. Non-goals, each with the reason

* **An allocation that crosses a return into a binding.** `const a = mk(); a[1]`
  reads `0` even when `mk` returns a bound allocation, before and after the spike.
  R-14's lane. Filed (§6.3).
* **An allocation held by an object property** (`{arr: new Array(3).fill(6)}` then
  `o.arr[0]` → `0`, both scopes). R-48's family. Filed.
* **A passed-through allocation rebound** (`const a = f(new Array(3).fill(1)); a[0]`
  → `0` where `f` returns its parameter). R-14's lane. Filed.
* **The parser's `new` precedence.** `console.log(new Array(3).fill(1)[0])` → `0`,
  and `console.log((new Array(3).fill(2))[1])` → `undefined`. A parser change that
  moves every `new X(…).m()`. Already filed
  (`length-fails-closed-discovered-defects.md` §2); the second row is new and is
  added there.
* **`Array.from({length: n})` as an argument** → `0`. A different recognizer
  (`is_array_from_call`), untouched by §3.4. Filed.
* **A float `.fill` passed to a function** fails to load (E4201), bound or inline.
  Loud; filed.
* **`(new Array(3)).length` → `1`.** `render_length`'s arm, not this lane (§1.2).
* **Narrowing `is_array_literal`.** §3.5.

---

## 6. Ledger obligations

### 6.1 The register

Four entries, each filed SILENT in its own commit off the baseline, measured in
both scopes against `node v26.8.2`, and retired in the commit that lands its fix.
The root-cause group of each is decided at filing under the register's §3
criteria and recorded with the reasoning.

| entry | title | lanes | retired as | by |
|---|---|---|---|---|
| **R-64** | An array allocation outside a declarator, assignment or `.fill` receiver evaluates to `0` | §2.2's rows: argument (both scopes, dynamic size, second argument), bare `Array(n)`, ternary arm, nested call, an allocation returned and passed straight on | **FIXED** | §3.2 + §3.3 |
| **R-65** | A fold-lane array, or a constructed value that lowers like one, passed to a user function reads as zeros in the callee | §2.4's rows | **FAIL_CLOSED** | §3.4 |
| **R-66** | A one-element array literal of an allocation is that allocation (`[new Array(3)].length` → `3`) | §2.3's declarator rows | **FAIL_CLOSED** | §3.1 |
| **R-67** | `.fill(v)` evaluates `v` once per element | §2.6's second table | **FIXED** | §3.3 |

Each gets exactly the artifact footprint R-63 carries as a retired entry. At the
baseline, R-63 appears as a keyed record in `tools/blast-radius/predicates.json`
and `tools/blast-radius/counts.json`; in `tools/blast-radius/matchers.mjs`,
`matchers.test.mjs` and `count.mjs`; in
`crates/kali_blast_radius/src/manifest_tests.rs`; as prose in
`tools/blast-radius/clusters.json`; as a §0.2 row; and in §1's entry-count
paragraph. Re-read each of those sites when filing, rather than copying this list.
§0.2's case-count paragraph moves by eight cases.

### 6.2 Followups corrected, each with a dated notice at the top

1. **`length-fails-closed-discovered-defects.md`**: §1 closed by R-64 and R-65,
   with a pointer to this spec; §2 gains the `(new Array(3).fill(2))[1]` →
   `undefined` row; §3 records that `(new Array(3)).length` is unchanged and why.
2. **`codegen-array-literal-predicate-is-still-negative-space.md`**: record that
   the LIR collision it could not see is now broken for allocation shapes at the
   AST (§3.1), and only for those. `[f()]` and `new f()` remain one node.

### 6.3 New: `docs/superpowers/followups/inline-allocation-value-position-discovered-defects.md`

Ranked, most consequential first:

1. **A returned or passed-through allocation, rebound, reads zeros**, with §5's
   rows, cross-referenced to R-14.
2. **An allocation held by an object property reads zeros**, cross-referenced to
   R-48.
3. **`Array.from({length: n})` passed to a function reads zeros.**
4. **The 36 scratch-slot holders**, as an audit left undone: §3.3 removes
   allocation and fill as clobberers, not other nested writers.
5. **A float `.fill` array passed to a function fails to load (E4201).**

### 6.4 Regeneration

If `counts.json` moves, run `tools/blast-radius/README.md`'s order (`accepts.mjs` →
`count.mjs` → `cargo run -p kali_blast_radius --example rank`), then
`cargo test -p kali_blast_radius`. `spliced_document_matches_the_generator` and the
§4.4 class-set gate go red on any drift, by design.

---

## 7. Risks, and what would falsify this design

| risk | what it would look like | response |
|---|---|---|
| **§3.1 misses a spelling** | a must-refuse `[<allocation>]` pin compiles, or a must-compute pin reads a literal's child count | §3.1 and `is_array_like_constructor` have drifted. Bring them into lockstep; never special-case the pin |
| **The scratch invariant is broken somewhere** | §4.1's third gate: a clobber pin is silent | Stop gate. Find the emission inside a slot's live window |
| **The reservation moves a test** | §4.1's second gate shows a pass → fail with no fill involved | Back to the human partner before choosing a conditional reservation |
| **§3.4 refuses more than measured** | §4.5's final diff shows a pass → refusal outside §4.3 | Back to the human partner. The spike measured 81 |
| **A generated case file is re-pinned by hand when its generator still runs** | the generator's next run reverts the re-pin | §4.1's first gate decides the route before any edit |
| **A fill's element repr disagrees with its callee's** | a must-compute pin prints a reinterpreted float | Out of the measured lanes; refuse the shape rather than guess the repr |
| **Ranking churn** | regeneration moves bands | Record it. Never hand-edit a generated region |
| **Node version drift** | an oracle case fails on a node upgrade | Expected; a real divergence signal |

---

## 8. Process notes carried into the plan

* `scripts/test-gate.sh` runs neither clippy nor fmt. Both run explicitly before
  pushing; `GATE OK` is not CI-green.
* One cargo target directory. No extra worktrees and no extra target directories:
  they have exhausted this pod's disk and killed a run.
* "Before" numbers are §2's measurements at `733cd26125`, not rebuilds of old
  commits.
* A full workspace suite run took over an hour and a half on this pod, twice. A subagent that
  starts one in the background has not finished; check `git` and `ps` before
  reviewing its work.
* The spike patch and probe scripts are not committed. The probe programs become
  §4.2's cases.
