# A runtime array is read in bounds, or kali refuses

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `016557d60` (`main`, the anon-array-return merge) |
| kali binary | `kali 0.1.0`, `target/debug/kali`, built at the baseline (`cargo build -p kali_cli`, `dev` profile) |
| oracle | `node v26.10.0` |
| measured on | 2026-10-02 |
| item picked | `docs/superpowers/followups/anon-array-return-discovered-defects.md` §1, "Runtime-array defects now reachable through an anonymous return" |
| defects this closes | that §1 in both lanes, which also closes `array-return-discovered-defects.md` §3's `[-1]`, `[5]` and `push` rows; every row of §2.1 below |

**Scope was chosen by the human partner:** fail closed, in both the
declaration and anonymous lanes (option A of four; the others were "real
semantics" (an `undefined` and growable `push`), "split", and "a different
item"). Mechanism: a runtime bounds guard, plus compile-time refusals that
`kali check` mirrors (approach 1 of three; §4 records the other two).

**Citation convention.** Every line reference is as of the baseline commit.

---

## 1. What this project is

**The claim:** for every plain fixed-length `[len][elem…]` runtime array,
kali either reads or writes the element node would, or refuses. The receivers
covered are array-return bindings and direct calls (declaration and anonymous
lanes), `new Array(n)` / `.fill` bindings, array params, and nameless computed
reads. Concretely:

1. An indexed read or write whose index is not below the array's length
   (unsigned, so a negative index is caught too) prints a kali message on
   stderr and traps. `kali run` exits 1 with `E4000`.
2. A negative integer-literal index (`a[-1]`, `f()[-1]`, `a[-1] = v`) refuses
   with `E5506` under both `kali check` and `kali run`.
3. `push`, `pop`, `shift`, `unshift` and `splice` called on a plain runtime
   array, and an assignment to its `.length`, refuse with `E5506` under both
   `kali check` and `kali run`.
4. An in-bounds program behaves as it does at the baseline.

### 1.1 What this project does NOT claim

* **No `undefined`.** An out-of-range read refuses at run time; it does not
  print node's `undefined`. R-21 stays open. Its out-of-bounds lane `r21o` is
  on a literal array, which takes the static-fold lane (`call.rs:6514-6573`),
  so this project does not touch it (§6).
* **No growable lane.** `emit_growable_index_read` (`emit/growable.rs:470`)
  has the same unguarded out-of-range read, by its own doc comment. Its header
  layout differs. It is measured as a control (§5.2) and filed (§6), not
  fixed.
* **No R-68.** A whole runtime array still prints its handle through a
  template literal, a concat or an alias.
* **No float index.** The plain lane's float-index shape gap is unchanged.
* **One disclosed `check` / `run` gap.** An out-of-range non-negative index
  is known only at run time, so `kali check` exits 0 on it while `kali run`
  refuses. Every compile-time refusal of §1 items 2 and 3 is mirrored, except
  on the anonymous lane (amendment A-3).

---

## 2. What was measured

### 2.1 The silent surface at the baseline

Every row: `kali check` exits 0, and `kali run` exits 0 with the value shown.
Program text is in full. `a…` rows are the anonymous lane, `d…` the
declaration twins, `n…`/`w…` a `new Array(3).fill(4)` binding inside
`function main(){…} main();`, `p1` an array param.

| id | program | node | kali `run` |
|---|---|---|---|
| a1 | `const f = () => [1,2,3]; console.log(f()[-1]);` | `undefined` | `3` (the length header) |
| a2 | `const f = () => [1,2,3]; console.log(f()[5]);` | `undefined` | `0` |
| a3 | `const f = () => [1,2,3]; const a = f(); a.push(4); console.log(a.length);` | `4` | `3` |
| a4 | `const f = () => [1,2,3]; const a = f(); a.push(4); console.log(a[3]);` | `4` | `0` |
| a5 | `const f = () => [1,2,3]; const a = f(); a.length = 1; console.log(a.length);` | `1` | `3` |
| d1-d5 | a1-a5 with `function f(){ return [1,2,3]; }` | as a1-a5 | as a1-a5 |
| n1 | `const a = new Array(3).fill(4); console.log(a[-1]);` | `undefined` | `3` |
| n2 | `… console.log(a[5]);` | `undefined` | `0` |
| n3 | `… a.length = 1; console.log(a[0], a.length);` | `4 1` | `1 3` (the store went to `a[0]`) |
| n4 | `… a.pop(); console.log(a.length);` | `2` | `3` |
| n5 | `… let i = 3; console.log(a[i]);` | `undefined` | `0` |
| w1 | `… a[3] = 9; console.log(a.length, a[3]);` | `4 9` | `3 51` (written past the block, read back changed) |
| w2 | `… a[-1] = 9; console.log(a[0], a.length);` | `4 3` | `4 9` (**the length header was overwritten**) |
| p1 | `function g(a){ return a[7]; } function f(){ return [1,2,3]; } console.log(g(f()));` | `undefined` | `0` |

w1 and w2 are heap writes outside the element slots. w2 corrupts the header,
so every later bounds-relevant read of that array is wrong.

### 2.2 Controls at the baseline

| id | program | node | kali `run` |
|---|---|---|---|
| g1 | `function main(){ const a = [1,2,3]; a.push(4); console.log(a[9]); } main();` (growable lane) | `undefined` | `0`, exit 0. Stays silent (§1.1). |

The in-bounds cases in `runtime/array_return.toml`, `runtime/anon_array_return.toml`
and `runtime/inline_allocation_value_position.toml` are the in-bounds controls.
Their outputs must not move.

### 2.3 Mechanism

* **Address.** Each read and write forms `wrap(base) + wrap(idx) * 8` and
  loads or stores at `offset: 8` (`emit/call.rs:6407`, `:6430-6447`; reads at
  `:6341`, `:6355`, `:6451`, `:6464`; writes at `emit/literal.rs:633-754`).
  Nothing compares `idx` with the header. `-1` addresses the header and `5`
  addresses memory past the block.
* **Mutators.** `growable_push_call_parts` (`emit/growable.rs:575-608`)
  declines a plain receiver. `push` is not function-valued, so
  `call_target_keeps_placeholder_lowering` (`emit/call.rs:4570`) admits it to
  the terminal fallback (`emit/call.rs:3878-3893`). That fallback warns,
  drops the arguments, pushes `i64.const 0` and never emits the receiver. The
  call is a silent no-op. `pop`, `shift`, `unshift` and `splice` take the same
  path.
* **`.length =`.** The one-child write arm (`emit/literal.rs:640-647`) takes
  `length` as `ArrayWriteIndex::Text("length")`, the index lowers to `0`, and
  the value is stored into element 0 (row n3).
* **`check`.** `kali check` never runs codegen (`kali_cli/src/build/compile.rs:46-61`),
  and `kali_types` has no gate for any of these.

---

## 3. Design

### 3.1 The bounds guard: one synthetic helper

A new synthetic wasm function, emitted in the same way as `__substring` and
`__streq`:

```
__array_elem_addr(base: i32, idx: i64) -> i32
  if idx >=u i64.load(base) { console.error(MESSAGE); unreachable }
  return base + i32.wrap(idx) * 8
```

* Its index is exposed as `array_elem_addr_fn_index()` beside its siblings in
  `emitter.rs`.
* Every plain-lane address helper in §2.3 emits `base`, then `idx`, then the
  call. The load or store at `offset: 8` that follows is unchanged. Reads and
  writes share the one helper.
* The helper is declared only when the program has a plain runtime-array
  indexed access, gated as the other conditional helpers are. A program with
  none gets byte-identical wasm.
* **Why a helper rather than inline:** an inline guard needs `base` and `idx`
  twice, so it needs scratch locals. The function's trailing scratch slots
  are shared, and a nested access (`a[b[i]]`, `f(a[i])[j]`) would clobber
  them. A call's params are fresh per call, so there is no reentrancy to get
  right.
* `MESSAGE` names kali, not a JavaScript error, because node raises nothing
  here: `kali: array index out of bounds: node reads undefined here (or grows
  the array on a write); kali refuses rather than read or write past the
  allocation`.
* `kali run` reports the trap through the existing path
  (`kali_runtime/src/execute.rs:202-248`, `kali_cli/src/bin/cmd_run.rs:280-287`):
  the message on stderr, then `error[E4000]: runtime trap …`, exit 1.

### 3.2 Compile-time refusals in codegen

These are the floors `kali run` reaches. Each uses `deny_e5506`
(`intrinsics/host.rs:1666`).

* **Negative literal index.** This is refused in the read arms of
  `emit/control_flow.rs` (named binding `:2940-2951` and `:3185-3198`, and
  `f()[k]` `:3200-3208`), in `emit/computed_member.rs:105-121`, and in the
  write arm of `emit/literal.rs:633-754`. The parser folds the minus into the
  index text (`kali_parser/src/literal.rs:83-88`), so the test is on that
  text. The `f()[k]` one-child arm (`:2954-2969`) parses `usize` and already
  declines `-1`. The plan verifies that it reaches a refusal and does not fall
  to another lane.
* **Mutators.** A new arm ahead of the terminal fallback in
  `emit/call.rs:3878` refuses `push`, `pop`, `shift`, `unshift` and `splice`
  on a receiver that `is_runtime_array_value` (`emit/call.rs:17`) admits and
  `growable_push_call_parts` declines.
* **`.length =`.** The one-child write arm in `emit/literal.rs:640` refuses
  `text == "length"` on a plain runtime-array receiver before treating it as
  an index.

### 3.3 The `kali_types` mirror

`kali_types` already decides "this receiver is a plain runtime array" for the
resolver's computed-read admission (`resolve/member.rs:345-389`). Beside the
existing `reject_*` gates in `resolve/expression.rs` (`:1651`, `:1775`), three
refusals use that same predicate and go through the existing `E5506` path:
negative literal index, mutator call, and `.length` assignment. Their message
text is identical to codegen's. Codegen keeps its own refusals as the
backstop.

**A growable-shape receiver must not be caught.** `const a=[1,2,3]; a.push(4)`
inside a function is promoted to the growable lane
(`kali_types/src/growable.rs:668-674`) and must keep working. The mirror runs
only where growable promotion does not apply.

### 3.4 Diagnostics

There are no new codes. The compile-time refusals are `E5506` and the run-time
refusal is `E4000`. `specs/15-errors.md` gains one sentence under `E5506`
naming these refusals and one under `E4000` naming the bounds trap. The CLI
surface, schemas and maturity are unchanged, so the AGENTS.md §6 CLI change
packet does not apply beyond `15-errors.md`.

---

## 4. Approaches not taken

* **Compile-time only.** Refusing every index kali cannot prove in bounds
  needs no runtime cost, but it would refuse nearly every `a[i]` loop over an
  array of unknown length. That capability loss is far larger than the defect.
* **Guard reads only.** In node a write past the end grows the array. Left
  unguarded, it is a silent heap write (rows w1, w2). That is the class this
  project exists to close.
* **Inline guard with scratch locals.** Rejected in §3.1 on reentrancy.

---

## 5. Testing and measurement

### 5.1 The capability-loss spike (gate)

First land §3.1 and §3.2 only, without the §3.3 mirror. Run
`cargo test --workspace` and `tools/array-return-probes/run.sh`, and list
every case or probe whose verdict moves.

* A silent row becoming a refusal is the intent.
* An in-bounds program that now traps or refuses is a bug, or it is brought
  back to the human partner before proceeding. This includes any `push` on a
  plain array whose result was never read.

The same diff is recomputed at the end of the branch and recorded in the
followups file (§6).

### 5.2 Probes and cases

* **New probes** in `tools/array-return-probes/probes/bounds_*.js` cover:
  * every row of §2.1 and the g1 control
  * nested in-bounds `a[b[i]]` and `f(a[i])[j]`, whose output must not change
  * an in-bounds loop over each receiver kind, whose output must not change

  `baseline.tsv` gains their baseline column, measured at `016557d60`.
* **New case file** `crates/kali_cli/tests/cases/runtime/array_bounds.toml`:
  * a2, d2, n2, n5, w1, p1: `kali check` exits 0, and `kali run` exits 1 with
    `MESSAGE` and `E4000` on stderr.
  * a1, d1, n1, w2, a3-a5, d3-d5, n3, n4: `E5506` under both `check` and
    `run`.
  * In-bounds loops, nested accesses and a growable `push` match node.
* Any existing case or probe that pins a §2.1 form at a silent value is
  re-pinned.

There is no new `tests/*.rs` integration target.

### 5.3 Unit tests (sibling `*_tests.rs` files)

* Codegen:
  * `__array_elem_addr` is declared iff the program has a plain runtime-array
    indexed access.
  * Its body traps at `idx == len` and at `idx == -1`, and returns the
    element address at `idx == len - 1`.
* `kali_types`:
  * Each of the three refusals fires on a plain runtime-array receiver.
  * None fires on a growable-shape receiver.
* **Agreement test:** over the `bounds_*` probe sources, every codegen §3.2
  refusal has a matching §3.3 refusal, and the reverse.

### 5.4 Gates

`cargo test --workspace` and `cargo clippy --workspace` pass, including
`kali_blast_radius::ranking::ranking_tests::spliced_document_matches_the_generator`.

---

## 6. Bookkeeping

* `anon-array-return-discovered-defects.md` §1 and
  `array-return-discovered-defects.md` §3 are marked FIXED (fail-closed) with
  the closing commit. The out-of-range reads now refuse rather than print
  `undefined`, and the note says so. §3's closure-capture row is not this
  project's work and stays open.
* `array-return-discovered-defects.md` §6 (R-21) gets a cross-reference: the
  runtime-array out-of-range read now refuses, and `undefined` stays open.
* A new `docs/superpowers/followups/array-bounds-discovered-defects.md`
  records what was measured and not fixed. At minimum that is:
  * the growable-lane out-of-range read (g1)
  * the §1.1 `check` / `run` gap
  * the §5.1 capability-loss diff
* `kali-silent-miscompile-register.md` is amended only if a §0.2 lane moves.
  R-21's `r21o` is on the static-fold lane and is not expected to move. This
  is re-measured with its oracle cases rather than assumed. If a lane does
  move, the ranking is regenerated with
  `cargo run -p kali_blast_radius --example rank`.
* `specs/15-errors.md` gets the two sentences of §3.4.

---

## 7. Amendments

Added while writing the implementation plan, before any code.

* **A-1. The helper is in every module.** §3.1 said `__array_elem_addr` is
  declared only when the program needs it, giving byte-identical wasm
  otherwise. Every fixed synthetic (`__streq`, `__join`, the `__usp_*`
  family) is instead present in every module, listed in
  `SYNTHETIC_FUNCTIONS` (`kali_codegen/src/lower.rs:52`). A precise "needs
  it" probe would have to predict codegen's `array_bindings` before codegen
  runs. The helper follows that precedent. It shifts every source-defined
  function's wasm index by one, and the §5.1 spike surfaces any test pinning
  an index or a module-wide instruction count. Its signature is
  `(base: i64, idx: i64, msg: i64) -> i64`. The call site passes the interned
  message handle as `msg`, because a hand-emitted synthetic body has no
  string interner. The caller wraps the result to i32.
* **A-2. The helper's body is tested through CLI cases.** §5.3 asked for a
  codegen unit test of the body. The crate has no harness that executes a
  synthetic body. Its three boundary rows (`idx == len - 1` reads, while
  `idx == len` and a run-time `idx == -1` trap) are `array_bounds.toml`
  cases instead.
* **A-3. A second `check` / `run` gap, on the anonymous lane.** The §3.3
  mirror proves a receiver through the resolver's runtime-array registry,
  which learns `const a = f()` from `call_returns_runtime_array`. That
  function keys by the bare callee name, and an anonymous function's fact is
  keyed by its `__kali_fn_N` id (`anon-array-return-discovered-defects.md`
  §9). So for rows a1 and a3-a5, `kali check` exits 0 while `kali run`
  refuses with `E5506`. Rows d1 and d3-d5, n1, n3, n4 and w2 refuse under
  both. The human partner chose to disclose this rather than fold §9's fix
  in. It is recorded in the new followups file, pointing at §9.
* **A-4. The agreement test is the paired cases.** §5.3 asked for a test
  that every codegen §3.2 refusal has a matching §3.3 refusal. Every refusal
  row in `array_bounds.toml` is pinned twice, once under `kali check` and
  once under `kali run`. A mismatch fails a case, so the pairs are that
  test. The anonymous rows of A-3 are pinned at their disclosed split.
* **A-5. The §5.1 capability loss is accepted.** Added after implementation.
  The §5.1 capability-loss rows (a `push` whose result is never read, in an
  `if` that never runs, or in a never-called function) were measured; see
  `array-bounds-discovered-defects.md` §5. The human partner accepted them as
  fail-closed on 2026-10-02: a fixed-length array cannot perform a length
  change, and narrowing to "refuse only when later observed" needs
  whole-program read analysis whose misses would reintroduce a silent
  miscompile. The real fix is a feature, filed as followups §12 (growable
  arrays from array-returning functions).
