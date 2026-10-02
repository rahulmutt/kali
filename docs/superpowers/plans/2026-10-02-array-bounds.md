# Array Bounds Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every plain fixed-length `[len][elem…]` runtime array is read and
written in bounds, or kali refuses. An out-of-range index traps at run time
(`E4000`). A negative literal index, the mutators `push`/`pop`/`shift`/
`unshift`/`splice`, and a `.length =` write refuse at compile time (`E5506`).

**Architecture:** One synthetic wasm helper, `__array_elem_addr(base, idx,
msg) -> i64`, replaces the unguarded `base + idx*8` in the single address
choke point (`emit_array_element_address_node`). Codegen refuses the static
shapes at their floors. `kali_types` mirrors those static refusals so
`kali check` agrees, except on the anonymous lane (amendment A-3).

**Tech Stack:** Rust (`kali_codegen`, `kali_types`, `kali_common`),
`wasm_encoder`, the `cases` TOML runner, node v26.10.0 as oracle.

**Spec:** `docs/superpowers/specs/2026-10-02-array-bounds-design.md`. Read
it, especially §2.1 (the rows), §3 and §7 (amendments A-1..A-4), before
starting any task.

## Global Constraints

- Baseline commit: `016557d60`. Oracle: `node v26.10.0`. Branch: `array-bounds`.
- No new diagnostic codes: compile-time refusals are `E5506`
  (`e5::FEATURE_UNAVAILABLE`), and the run-time refusal is the existing trap,
  `E4000`.
- No new `tests/*.rs` integration target. Black-box tests are `.toml` cases
  under `crates/kali_cli/tests/cases/`.
- Rust unit tests go in sibling `*_tests.rs` files, never inline `#[cfg(test)]`.
- The growable lane (`emit/growable.rs`) is out of scope. Its `push` must
  keep working.
- No `i64.eqz` in any synthetic body (`boolean_branches_use_the_layout_fast_path`).
- Message text is defined once in `kali_common/src/messages.rs`, and codegen
  and `kali_types` both use it.
- Commits reference the project: `feat(array-bounds): …`, `test(array-bounds): …`,
  `docs(array-bounds): …`. End each with
  `Co-Authored-By: Claude <noreply@anthropic.com>`.
- `cargo test --workspace` and `cargo clippy --workspace --all-targets` pass
  at the end of every task that touches Rust.

## Review Focus

- **A nested access** (`a[b[i]]`, `f(a[i])[j]`) must compute exactly as before.
  The helper takes params, not shared scratch locals, for this reason. Task 3
  pins both.
- **A negative index from a variable** (`let i = -1; a[i]`) must trap at run
  time, not read the length header. The unsigned compare is what catches it.
  Task 3 pins it.
- **A write at `idx == len`** (`a[3] = 9` on length 3) must trap before the
  store. Writes share the helper. Task 3 pins it.
- **A growable-shape `push`** (`const a = [1,2,3]; a.push(4)` inside a
  function) must keep working under both `check` and `run`. Tasks 4 and 6
  pin it.
- **A compound `.length` write** (`a.length -= 1`) must not stay silent.
  The types gate catches every assignment operator, and `run` runs the
  checker first. Task 6 pins it.

## File Structure

| file | change |
|---|---|
| `crates/kali_common/src/messages.rs` (+ `messages_tests.rs`) | four message functions and the `RUNTIME_ARRAY_MUTATORS` list |
| `crates/kali_codegen/src/lower.rs` | the `__array_elem_addr` synthetic: list entry, plan, signature, body |
| `crates/kali_codegen/src/emitter.rs` | `array_elem_addr_fn_index()` |
| `crates/kali_codegen/src/emit/call.rs` | the guarded address, the negative-literal refusal, the mutator refusal |
| `crates/kali_codegen/src/emit/literal.rs` | the `.length =` refusal |
| `crates/kali_cli/tests/runtime_smoke.rs` | the test-side `SYNTHETIC_FUNCTIONS` mirror |
| `crates/kali_types/src/resolve/member.rs` (+ `member_tests.rs`) | the receiver predicate and three gates |
| `crates/kali_types/src/resolve/call.rs`, `resolve/expression.rs` | the gate call sites |
| `crates/kali_cli/tests/cases/runtime/array_bounds.toml` | new case file |
| `tools/array-return-probes/run.sh`, `probes/bounds_*.js`, `baseline-bounds.tsv` | the `TRAPS` verdict, the probes, the baseline |
| `specs/15-errors.md`, followups, new `array-bounds-discovered-defects.md` | bookkeeping |

---

### Task 1: Probes and the baseline column (before any code change)

**Files:**
- Modify: `tools/array-return-probes/run.sh`
- Create: `tools/array-return-probes/probes/bounds_*.js` (the files below)
- Create: `tools/array-return-probes/baseline-bounds.tsv`

**Interfaces:**
- Produces: the probe names used by Task 5's diff, and the new `TRAPS`
  verdict string.

- [ ] **Step 1: Add the `TRAPS` verdict to `run.sh`.** Replace the verdict block:

```bash
  if [ $kali_exit -eq 0 ] && [ "$kali_out" = "$node_out" ]; then
    verdict=CORRECT
  elif [ $kali_exit -ne 0 ] && grep -q E5506 /tmp/array-return-probe.err; then
    verdict=REFUSES
  elif [ $kali_exit -ne 0 ] && grep -q 'kali: array index out of bounds' /tmp/array-return-probe.err; then
    verdict=TRAPS
  elif [ $kali_exit -eq 0 ]; then
    verdict=SILENT
  else
    verdict=OTHER
  fi
```

- [ ] **Step 2: Write the probes.** Each line is `filename: content`. Create
  every file with exactly that content plus a trailing newline.

```
bounds_a1.js: const f = () => [1,2,3]; console.log(f()[-1]);
bounds_a2.js: const f = () => [1,2,3]; console.log(f()[5]);
bounds_a3.js: const f = () => [1,2,3]; const a = f(); a.push(4); console.log(a.length);
bounds_a4.js: const f = () => [1,2,3]; const a = f(); a.push(4); console.log(a[3]);
bounds_a5.js: const f = () => [1,2,3]; const a = f(); a.length = 1; console.log(a.length);
bounds_d1.js: function f(){ return [1,2,3]; } console.log(f()[-1]);
bounds_d2.js: function f(){ return [1,2,3]; } console.log(f()[5]);
bounds_d3.js: function f(){ return [1,2,3]; } const a = f(); a.push(4); console.log(a.length);
bounds_d4.js: function f(){ return [1,2,3]; } const a = f(); a.push(4); console.log(a[3]);
bounds_d5.js: function f(){ return [1,2,3]; } const a = f(); a.length = 1; console.log(a.length);
bounds_n1.js: function main(){ const a = new Array(3).fill(4); console.log(a[-1]); } main();
bounds_n2.js: function main(){ const a = new Array(3).fill(4); console.log(a[5]); } main();
bounds_n3.js: function main(){ const a = new Array(3).fill(4); a.length = 1; console.log(a[0], a.length); } main();
bounds_n4.js: function main(){ const a = new Array(3).fill(4); a.pop(); console.log(a.length); } main();
bounds_n5.js: function main(){ const a = new Array(3).fill(4); let i = 3; console.log(a[i]); } main();
bounds_n6.js: function main(){ const a = new Array(3).fill(4); let i = -1; console.log(a[i]); } main();
bounds_n7.js: function main(){ const a = new Array(3).fill(4); a.shift(); console.log(a.length); } main();
bounds_n8.js: function main(){ const a = new Array(3).fill(4); a.unshift(1); console.log(a.length); } main();
bounds_n9.js: function main(){ const a = new Array(3).fill(4); a.splice(0, 1); console.log(a.length); } main();
bounds_w1.js: function main(){ const a = new Array(3).fill(4); a[3] = 9; console.log(a.length, a[3]); } main();
bounds_w2.js: function main(){ const a = new Array(3).fill(4); a[-1] = 9; console.log(a[0], a.length); } main();
bounds_p1.js: function g(a){ return a[7]; } function f(){ return [1,2,3]; } console.log(g(f()));
bounds_g1.js: function main(){ const a = [1,2,3]; a.push(4); console.log(a[9]); } main();
bounds_ok_last.js: function main(){ const a = new Array(3).fill(4); a[2] = 9; console.log(a[2], a.length); } main();
bounds_ok_loop.js: function main(){ const a = new Array(3).fill(4); let s = 0; for (let i = 0; i < a.length; i++) { s += a[i]; } console.log(s); } main();
bounds_ok_nested.js: function main(){ const a = new Array(3).fill(2); const b = new Array(3).fill(1); let i = 0; console.log(a[b[i]]); } main();
bounds_ok_call_nested.js: function f(){ return [7,8,9]; } function main(){ const a = new Array(3).fill(1); let j = 2; console.log(f()[a[0]], f()[j]); } main();
bounds_ok_growable_push.js: function main(){ const a = [1,2,3]; a.push(4); console.log(a.length, a[3]); } main();
```

- [ ] **Step 3: Build the baseline binary and record the baseline.** HEAD
  must still be the spec commit, with no compiler change since `016557d60`.

Run:
```bash
git diff --stat 016557d60 -- crates/   # expect: empty
cargo build -p kali_cli
tools/array-return-probes/run.sh tools/array-return-probes/baseline-bounds.tsv
grep '^bounds_' tools/array-return-probes/baseline-bounds.tsv
```
`baseline-bounds.tsv` holds EVERY probe (not only `bounds_*`) at
`016557d60`, so Tasks 5 and 7 diff against one file measured at one commit.
Expected: every `bounds_a*`, `_d*`, `_n*`, `_w*`, `_p1` and `_g1` row reads
`SILENT`. Every `bounds_ok_*` row reads `CORRECT`, or records whatever it
measures (the spec's §2.1 cells are the reference). If any `bounds_ok_*` row
is not `CORRECT`, stop and report it. That control is not a valid in-bounds
pin, and the human partner decides on a replacement.

- [ ] **Step 4: Commit**

```bash
git add tools/array-return-probes/run.sh tools/array-return-probes/probes/bounds_*.js tools/array-return-probes/baseline-bounds.tsv
git commit -m "test(array-bounds): bounds probes, a TRAPS verdict, and their baseline at 016557d60"
```

---

### Task 2: Shared message text

**Files:**
- Modify: `crates/kali_common/src/messages.rs` (append)
- Test: `crates/kali_common/src/messages_tests.rs` (append)

**Interfaces:**
- Produces (all `pub`, re-exported through `kali_common::*`):
  - `RUNTIME_ARRAY_MUTATORS: &[&str]`
  - `runtime_array_negative_index_unavailable_message() -> &'static str`
  - `runtime_array_mutator_unavailable_message(method: &str) -> String`
  - `runtime_array_length_write_unavailable_message() -> &'static str`
  - `runtime_array_index_out_of_bounds_message() -> &'static str`

- [ ] **Step 1: Write the failing tests** (append to `messages_tests.rs`):

```rust
#[test]
fn runtime_array_refusal_messages_are_stable() {
    assert_eq!(
        runtime_array_negative_index_unavailable_message(),
        "a negative index on a runtime array is unavailable in the current phase: node reads `undefined` there and kali has no `undefined` value, so kali refuses rather than read the array's length header"
    );
    assert_eq!(
        runtime_array_mutator_unavailable_message("push"),
        "calling `.push()` on a runtime array is unavailable in the current phase: the array has a fixed length, so kali refuses rather than silently skip the call"
    );
    assert_eq!(
        runtime_array_length_write_unavailable_message(),
        "assigning to `.length` of a runtime array is unavailable in the current phase: the array has a fixed length, so kali refuses rather than store into an element"
    );
    assert_eq!(
        runtime_array_index_out_of_bounds_message(),
        "kali: array index out of bounds: node reads undefined here (or grows the array on a write); kali refuses rather than read or write past the allocation"
    );
}

#[test]
fn runtime_array_mutators_are_the_five_fixed_length_breakers() {
    assert_eq!(RUNTIME_ARRAY_MUTATORS, &["push", "pop", "shift", "unshift", "splice"]);
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p kali_common runtime_array`
Expected: a compile error, because `runtime_array_negative_index_unavailable_message` etc. are not defined.

- [ ] **Step 3: Implement** (append to `messages.rs`):

```rust
/// Methods that change a runtime array's length. A plain `[len][elem…]`
/// array has a fixed length, so each refuses on one (array-bounds spec §3.2).
pub const RUNTIME_ARRAY_MUTATORS: &[&str] = &["push", "pop", "shift", "unshift", "splice"];

/// Canonical wording for a negative integer-literal index on a plain runtime array.
pub const fn runtime_array_negative_index_unavailable_message() -> &'static str {
    "a negative index on a runtime array is unavailable in the current phase: node reads `undefined` there and kali has no `undefined` value, so kali refuses rather than read the array's length header"
}

/// Canonical wording for a [`RUNTIME_ARRAY_MUTATORS`] call on a plain runtime array.
pub fn runtime_array_mutator_unavailable_message(method: &str) -> String {
    format!(
        "calling `.{method}()` on a runtime array is unavailable in the current phase: the array has a fixed length, so kali refuses rather than silently skip the call"
    )
}

/// Canonical wording for an assignment to a plain runtime array's `.length`.
pub const fn runtime_array_length_write_unavailable_message() -> &'static str {
    "assigning to `.length` of a runtime array is unavailable in the current phase: the array has a fixed length, so kali refuses rather than store into an element"
}

/// What `__array_elem_addr` prints on stderr before it traps (array-bounds spec §3.1).
/// It names kali, not a JavaScript error, because node raises nothing here.
pub const fn runtime_array_index_out_of_bounds_message() -> &'static str {
    "kali: array index out of bounds: node reads undefined here (or grows the array on a write); kali refuses rather than read or write past the allocation"
}
```

- [ ] **Step 4: Run them and watch them pass**

Run: `cargo test -p kali_common runtime_array`
Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/messages.rs crates/kali_common/src/messages_tests.rs
git commit -m "feat(array-bounds): shared refusal and trap wording for plain runtime arrays"
```

---

### Task 3: The bounds guard helper

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (`SYNTHETIC_FUNCTIONS` at :52; plan push after the `__usp_tostring` plan, ~:869; signature branch ~:1235; body dispatch ~:1700; new body fn next to `emit_streq_body`)
- Modify: `crates/kali_codegen/src/emitter.rs` (next to `streq_fn_index`, ~:1289)
- Modify: `crates/kali_codegen/src/emit/call.rs` (`emit_array_element_address_node`, ~:6430)
- Modify: `crates/kali_cli/tests/runtime_smoke.rs` (~:806, the test-side mirror)
- Create: `crates/kali_cli/tests/cases/runtime/array_bounds.toml`

**Interfaces:**
- Consumes: `kali_common::runtime_array_index_out_of_bounds_message()` (Task 2).
- Produces: `FunctionEmitter::array_elem_addr_fn_index(&self) -> u32`. After
  this task, `emit_array_element_address_node(function, base_id, index_id)`
  pushes a bounds-checked i32 address. It keeps its signature, and the
  caller's load or store at `offset: 8` is unchanged.

- [ ] **Step 1: Write the failing cases.** Create
  `crates/kali_cli/tests/cases/runtime/array_bounds.toml`:

````toml
# Cases for the array-bounds project (spec
# docs/superpowers/specs/2026-10-02-array-bounds-design.md).
#
# A plain fixed-length runtime array is read and written in bounds, or kali
# refuses: an out-of-range index traps at run time (E4000, after the kali
# message on stderr), and a negative literal index, a length-changing method
# or a `.length` write refuses with E5506 under `check` and `run` alike
# (amendment A-4: each refusal is pinned under both). The anonymous lane's
# static refusals are run-only (amendment A-3). Each rationale gives kali's
# `run` output at the baseline `016557d60`
# (tools/array-return-probes/baseline-bounds.tsv) and node v26.10.0's.
#
# `[source]` keys are one file per program, because `[source]` is file-wide.

[constants]
OOB = "kali: array index out of bounds"
NEG = "a negative index on a runtime array is unavailable"
MUT = "on a runtime array is unavailable in the current phase"
LEN = "assigning to `.length` of a runtime array is unavailable"

[source]
"a2.js" = '''
const f = () => [1,2,3]; console.log(f()[5]);
'''
"d2.js" = '''
function f(){ return [1,2,3]; } console.log(f()[5]);
'''
"n2.js" = '''
function main(){ const a = new Array(3).fill(4); console.log(a[5]); } main();
'''
"n5.js" = '''
function main(){ const a = new Array(3).fill(4); let i = 3; console.log(a[i]); } main();
'''
"n6.js" = '''
function main(){ const a = new Array(3).fill(4); let i = -1; console.log(a[i]); } main();
'''
"w1.js" = '''
function main(){ const a = new Array(3).fill(4); a[3] = 9; console.log(a.length, a[3]); } main();
'''
"p1.js" = '''
function g(a){ return a[7]; } function f(){ return [1,2,3]; } console.log(g(f()));
'''
"ok_last.js" = '''
function main(){ const a = new Array(3).fill(4); a[2] = 9; console.log(a[2], a.length); } main();
'''
"ok_loop.js" = '''
function main(){ const a = new Array(3).fill(4); let s = 0; for (let i = 0; i < a.length; i++) { s += a[i]; } console.log(s); } main();
'''
"ok_nested.js" = '''
function main(){ const a = new Array(3).fill(2); const b = new Array(3).fill(1); let i = 0; console.log(a[b[i]]); } main();
'''
"ok_call_nested.js" = '''
function f(){ return [7,8,9]; } function main(){ const a = new Array(3).fill(1); let j = 2; console.log(f()[a[0]], f()[j]); } main();
'''

[[case]]
name = "an_out_of_range_direct_call_read_traps_anonymous"
rationale = """At `016557d60` kali printed `0` at exit 0 where node v26.10.0 prints `undefined`. The bounds guard traps (spec §3.1)."""
args = ["run", "a2.js"]
exit = "failure"
stderr_contains = ["${OOB}", "E4000"]

[[case]]
name = "an_out_of_range_direct_call_read_passes_check_anonymous"
rationale = """The disclosed run-time-only gap (spec §1.1): an out-of-range non-negative index is unknown until run time."""
args = ["check", "a2.js"]
exit = "success"

[[case]]
name = "an_out_of_range_direct_call_read_traps_declaration"
rationale = """At `016557d60` kali printed `0` at exit 0 where node v26.10.0 prints `undefined`."""
args = ["run", "d2.js"]
exit = "failure"
stderr_contains = ["${OOB}", "E4000"]

[[case]]
name = "an_out_of_range_literal_read_of_a_filled_array_traps"
rationale = """At `016557d60` kali printed `0` at exit 0 where node v26.10.0 prints `undefined`."""
args = ["run", "n2.js"]
exit = "failure"
stderr_contains = ["${OOB}", "E4000"]

[[case]]
name = "an_index_equal_to_the_length_traps"
rationale = """At `016557d60` kali printed `0` at exit 0 where node v26.10.0 prints `undefined`. `idx == len` is the first rejected index."""
args = ["run", "n5.js"]
exit = "failure"
stderr_contains = ["${OOB}", "E4000"]

[[case]]
name = "a_negative_index_held_in_a_variable_traps"
rationale = """Not in the followup: `let i = -1; a[i]`. node v26.10.0 prints `undefined`. The unsigned compare catches a negative run-time index, which would otherwise read the length header."""
args = ["run", "n6.js"]
exit = "failure"
stderr_contains = ["${OOB}", "E4000"]

[[case]]
name = "a_write_at_the_length_traps_before_the_store"
rationale = """At `016557d60` kali printed `3 51` at exit 0 (written past the block) where node v26.10.0 prints `4 9`."""
args = ["run", "w1.js"]
exit = "failure"
stderr_contains = ["${OOB}", "E4000"]

[[case]]
name = "an_out_of_range_read_through_an_array_param_traps"
rationale = """At `016557d60` kali printed `0` at exit 0 where node v26.10.0 prints `undefined`."""
args = ["run", "p1.js"]
exit = "failure"
stderr_contains = ["${OOB}", "E4000"]

[[case]]
name = "the_last_index_reads_and_writes_in_bounds"
rationale = """In-bounds control at `idx == len - 1`: kali and node v26.10.0 both print `9 3`."""
args = ["run", "ok_last.js"]
exit = "success"
stdout = "9 3\n"

[[case]]
name = "a_length_bounded_loop_is_unchanged"
rationale = """In-bounds control: kali and node v26.10.0 both print `12`."""
args = ["run", "ok_loop.js"]
exit = "success"
stdout = "12\n"

[[case]]
name = "a_nested_index_read_is_unchanged"
rationale = """In-bounds control for helper reentrancy (spec §3.1): `a[b[i]]`. node v26.10.0 prints `2`."""
args = ["run", "ok_nested.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "a_nested_index_into_a_direct_call_is_unchanged"
rationale = """In-bounds control for helper reentrancy: `f()[a[0]]` and `f()[j]`. node v26.10.0 prints `8 9`."""
args = ["run", "ok_call_nested.js"]
exit = "success"
stdout = "8 9\n"
````

- [ ] **Step 2: Run them and watch the trap cases fail**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_bounds`
Expected: the 7 `…_traps…` cases FAIL (kali exits 0). The `check` case and
the 4 in-bounds controls pass.

- [ ] **Step 3: Register the synthetic in `lower.rs`.**

(a) Append `"__array_elem_addr",` as the last entry of `SYNTHETIC_FUNCTIONS`.

(b) Directly after the `__usp_tostring` `all_functions.push(...)`, and before
the `collect_requested_clone_shapes` loop, add:

```rust
    // Synthetic runtime-array bounds guard
    // `__array_elem_addr(base: i64, idx: i64, msg: i64) -> i64`
    // (array-bounds spec §3.1, amendment A-1): every plain `[len][elem…]`
    // element read and write routes its address through it, so an index at or
    // past the length header — a negative one too, by the unsigned compare —
    // prints `msg` and traps instead of addressing outside the element slots.
    // Present in every module, like `__streq`. Same inert-placeholder pattern
    // as the synthetics above; body hand-emitted by `emit_array_elem_addr_body`.
    all_functions.push(FunctionPlan {
        name: "__array_elem_addr".to_string(),
        params: vec!["base".to_string(), "idx".to_string(), "msg".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
```

(c) In the signature `if / else if` chain (the one containing
`} else if function.name == "__substring" {`), add before
`} else if function.name == "__arena_reset" {`:

```rust
        } else if function.name == "__array_elem_addr" {
            (
                vec![ValType::I64, ValType::I64, ValType::I64],
                vec![ValType::I64],
            )
```

(d) In the synthetic body `match`, after `"__streq" => emit_streq_body(&mut body),` add:

```rust
                "__array_elem_addr" => emit_array_elem_addr_body(&mut body),
```

(e) Add the body function directly after `emit_streq_body`:

```rust
/// `__array_elem_addr(base, idx, msg) -> i64`: the bounds guard for a plain
/// `[len][elem…]` runtime array (array-bounds spec §3.1). Locals: 0 = base,
/// 1 = idx, 2 = msg (params, no further locals). `idx >=u len` — a negative
/// `idx` included — hands `msg` to `console.error` and traps; otherwise
/// returns `base + idx * 8`, the `+8` header skip staying on the caller's
/// load/store `offset`. No `i64.eqz` (see `emit_streq_body`).
fn emit_array_elem_addr_body(func: &mut Function) {
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    func.instruction(&Instruction::I64GeU);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::Call(crate::CONSOLE_ERROR_IMPORT_INDEX));
    func.instruction(&Instruction::Unreachable);
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(8));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I64Add);
    // NO trailing End — the dispatch loop appends it (same as every synthetic).
}
```

The local-declaration chain needs no branch: its final `else` declares
`function.locals`, which is empty here.

- [ ] **Step 4: Expose the index in `emitter.rs`** (after `streq_fn_index`):

```rust
    /// Wasm function index of the synthetic runtime-array bounds guard
    /// (`__array_elem_addr(base, idx, msg) -> i64`, array-bounds spec §3.1).
    /// Called by `emit_array_element_address_node` for every plain-array
    /// element read and write.
    pub(crate) fn array_elem_addr_fn_index(&self) -> u32 {
        self.functions["__array_elem_addr"]
    }
```

- [ ] **Step 5: Route the address through the helper.** In `emit/call.rs`,
  replace the body of `emit_array_element_address_node` (keep its doc
  comment, and append one sentence to it saying the address is
  bounds-checked by `__array_elem_addr`):

```rust
        // The base stays i64: the guard reads the length header through it.
        let _ = self.emit_node(function, base_id, true);
        // Stage P5 T-new-E: the index operand is a numeric-consumption sink.
        // A `String()`-result index (`a[s]`, and its store twin `a[s] = v`,
        // both routed here) fails closed rather than indexing on the raw
        // handle bits (`a[String(1n)]` → placeholder `0`).
        let _ = self.emit_numeric_operand(function, index_id);
        // Array-bounds spec §3.1: `idx >=u len` traps with the kali message,
        // so no index addresses outside the element slots.
        let (offset, len) = self
            .strings
            .intern(kali_common::runtime_array_index_out_of_bounds_message());
        function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
        function.instruction(&Instruction::Call(self.array_elem_addr_fn_index()));
        function.instruction(&Instruction::I32WrapI64);
```

`emit_array_base_address` stays as it is. The `.length` header read still
uses it.

- [ ] **Step 6: Sync the test-side mirror.** In
  `crates/kali_cli/tests/runtime_smoke.rs`, append `"__array_elem_addr",` to
  the local `SYNTHETIC_FUNCTIONS` array (~:806). Then extend the doc comment's
  list of synthetics with "and the runtime-array bounds guard
  `__array_elem_addr` (array-bounds)".

- [ ] **Step 7: Run the cases and watch them pass**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_bounds`
Expected: all 12 pass. If an in-bounds control fails, the helper's arithmetic
or argument order is wrong. Fix that; do not weaken the case.

- [ ] **Step 8: Run the codegen and smoke suites**

Run: `cargo test -p kali_codegen && cargo test -p kali_cli --test runtime_smoke`
Expected: PASS. A failure that pins a module-wide instruction count, or a
function index, is A-1's index shift. Update that pin's count, and add one
sentence to its comment naming `__array_elem_addr`, the way the
`__usp_tostring` entries in `intrinsics/string_tests/lookup.rs` are
annotated. Any other failure is a real regression: stop and debug.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/runtime_smoke.rs crates/kali_cli/tests/cases/runtime/array_bounds.toml
git commit -m "feat(array-bounds): a plain runtime-array element address is bounds-checked by __array_elem_addr"
```

---

### Task 4: Codegen refusals: negative literal index, mutators, `.length =`

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs` (`emit_array_element_address_node`; a new recognizer near `is_runtime_array_value` at :17; a new arm after the growable-`push` arm at ~:1619)
- Modify: `crates/kali_codegen/src/emit/literal.rs` (the `array_bindings` write arm, ~:661)
- Modify: `crates/kali_cli/tests/cases/runtime/array_bounds.toml` (append)

**Interfaces:**
- Consumes: Task 2's messages and `RUNTIME_ARRAY_MUTATORS`, and Task 3's helper.
- Produces: `FunctionEmitter::plain_runtime_array_mutator(&self, node: &LirNode) -> Option<String>`
  and `FunctionEmitter::is_negative_literal_index(&self, index_id: LirNodeId) -> bool`.

- [ ] **Step 1: Write the failing `run` cases.** Append these sources to
  `[source]` (they must sit above the first `[[case]]`, so insert them at the
  end of the `[source]` table):

```toml
"a1.js" = '''
const f = () => [1,2,3]; console.log(f()[-1]);
'''
"a3.js" = '''
const f = () => [1,2,3]; const a = f(); a.push(4); console.log(a.length);
'''
"a4.js" = '''
const f = () => [1,2,3]; const a = f(); a.push(4); console.log(a[3]);
'''
"a5.js" = '''
const f = () => [1,2,3]; const a = f(); a.length = 1; console.log(a.length);
'''
"d1.js" = '''
function f(){ return [1,2,3]; } console.log(f()[-1]);
'''
"d3.js" = '''
function f(){ return [1,2,3]; } const a = f(); a.push(4); console.log(a.length);
'''
"d4.js" = '''
function f(){ return [1,2,3]; } const a = f(); a.push(4); console.log(a[3]);
'''
"d5.js" = '''
function f(){ return [1,2,3]; } const a = f(); a.length = 1; console.log(a.length);
'''
"n1.js" = '''
function main(){ const a = new Array(3).fill(4); console.log(a[-1]); } main();
'''
"n3.js" = '''
function main(){ const a = new Array(3).fill(4); a.length = 1; console.log(a[0], a.length); } main();
'''
"n4.js" = '''
function main(){ const a = new Array(3).fill(4); a.pop(); console.log(a.length); } main();
'''
"n7.js" = '''
function main(){ const a = new Array(3).fill(4); a.shift(); console.log(a.length); } main();
'''
"n8.js" = '''
function main(){ const a = new Array(3).fill(4); a.unshift(1); console.log(a.length); } main();
'''
"n9.js" = '''
function main(){ const a = new Array(3).fill(4); a.splice(0, 1); console.log(a.length); } main();
'''
"w2.js" = '''
function main(){ const a = new Array(3).fill(4); a[-1] = 9; console.log(a[0], a.length); } main();
'''
"ok_growable_push.js" = '''
function main(){ const a = [1,2,3]; a.push(4); console.log(a.length, a[3]); } main();
'''
```

Append these cases:

```toml
[[case]]
name = "a_negative_literal_index_into_a_direct_call_refuses_at_run_anonymous"
rationale = """At `016557d60` kali printed `3` (the length header) at exit 0 where node v26.10.0 prints `undefined`. Spec §3.2."""
args = ["run", "a1.js"]
exit = "failure"
stderr_contains = ["E5506", "${NEG}"]

[[case]]
name = "a_negative_literal_index_into_a_direct_call_refuses_at_run_declaration"
rationale = """At `016557d60` kali printed `3` (the length header) at exit 0 where node v26.10.0 prints `undefined`."""
args = ["run", "d1.js"]
exit = "failure"
stderr_contains = ["E5506", "${NEG}"]

[[case]]
name = "a_negative_literal_index_into_a_filled_array_refuses_at_run"
rationale = """At `016557d60` kali printed `3` at exit 0 where node v26.10.0 prints `undefined`."""
args = ["run", "n1.js"]
exit = "failure"
stderr_contains = ["E5506", "${NEG}"]

[[case]]
name = "a_negative_literal_index_store_refuses_at_run"
rationale = """At `016557d60` kali printed `4 9` at exit 0, having overwritten the length header, where node v26.10.0 prints `4 3`."""
args = ["run", "w2.js"]
exit = "failure"
stderr_contains = ["E5506", "${NEG}"]

[[case]]
name = "push_on_a_call_bound_array_refuses_at_run_anonymous"
rationale = """At `016557d60` kali printed `3` at exit 0 (the push was a silent no-op) where node v26.10.0 prints `4`."""
args = ["run", "a3.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${MUT}"]

[[case]]
name = "push_then_index_refuses_at_run_anonymous"
rationale = """At `016557d60` kali printed `0` at exit 0 where node v26.10.0 prints `4`."""
args = ["run", "a4.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${MUT}"]

[[case]]
name = "push_on_a_call_bound_array_refuses_at_run_declaration"
rationale = """At `016557d60` kali printed `3` at exit 0 where node v26.10.0 prints `4`."""
args = ["run", "d3.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${MUT}"]

[[case]]
name = "push_then_index_refuses_at_run_declaration"
rationale = """At `016557d60` kali printed `0` at exit 0 where node v26.10.0 prints `4`."""
args = ["run", "d4.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${MUT}"]

[[case]]
name = "pop_on_a_filled_array_refuses_at_run"
rationale = """At `016557d60` kali printed `3` at exit 0 where node v26.10.0 prints `2`."""
args = ["run", "n4.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.pop()`", "${MUT}"]

[[case]]
name = "shift_on_a_filled_array_refuses_at_run"
rationale = """node v26.10.0 prints `2`. See baseline-bounds.tsv `bounds_n7` for kali at `016557d60`."""
args = ["run", "n7.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.shift()`", "${MUT}"]

[[case]]
name = "unshift_on_a_filled_array_refuses_at_run"
rationale = """node v26.10.0 prints `4`. See baseline-bounds.tsv `bounds_n8` for kali at `016557d60`."""
args = ["run", "n8.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.unshift()`", "${MUT}"]

[[case]]
name = "splice_on_a_filled_array_refuses_at_run"
rationale = """node v26.10.0 prints `2`. See baseline-bounds.tsv `bounds_n9` for kali at `016557d60`."""
args = ["run", "n9.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.splice()`", "${MUT}"]

[[case]]
name = "a_length_write_refuses_at_run_anonymous"
rationale = """At `016557d60` kali printed `3` at exit 0 where node v26.10.0 prints `1`."""
args = ["run", "a5.js"]
exit = "failure"
stderr_contains = ["E5506", "${LEN}"]

[[case]]
name = "a_length_write_refuses_at_run_declaration"
rationale = """At `016557d60` kali printed `3` at exit 0 where node v26.10.0 prints `1`."""
args = ["run", "d5.js"]
exit = "failure"
stderr_contains = ["E5506", "${LEN}"]

[[case]]
name = "a_length_write_on_a_filled_array_refuses_at_run"
rationale = """At `016557d60` kali printed `1 3` at exit 0 (the value was stored into element 0) where node v26.10.0 prints `4 1`."""
args = ["run", "n3.js"]
exit = "failure"
stderr_contains = ["E5506", "${LEN}"]

[[case]]
name = "a_growable_push_still_appends_at_run"
rationale = """Control (spec §3.3): a growable-shape literal keeps its working `push`. kali and node v26.10.0 both print `4 4`."""
args = ["run", "ok_growable_push.js"]
exit = "success"
stdout = "4 4\n"
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_bounds`
Expected: the 15 new refusal cases FAIL. The negative-index rows probably
trap (`E4000`) under Task 3's guard rather than print `E5506`; the others
exit 0. The growable control passes.

- [ ] **Step 3: Refuse a negative literal index.** In `emit/call.rs`, add
  next to `is_runtime_array_value`:

```rust
    /// The index node is an integer literal below zero. `a[-1]` reaches here
    /// with the sign folded into the index text
    /// (`kali_parser/src/literal.rs:83-88`), on both the one-child (text) and
    /// two-child (node) member forms. Array-bounds spec §3.2.
    pub(crate) fn is_negative_literal_index(&self, index_id: LirNodeId) -> bool {
        let node = self.node(index_id);
        node.children.is_empty()
            && node
                .text
                .as_deref()
                .and_then(|text| text.parse::<i64>().ok())
                .is_some_and(|value| value < 0)
    }
```

Then make it the first statement of `emit_array_element_address_node`:

```rust
        // Array-bounds spec §3.2: node never has an element at a negative
        // index, so a literal one refuses at compile time rather than trap.
        if self.is_negative_literal_index(index_id) {
            let _ = self.deny_e5506(
                function,
                kali_common::runtime_array_negative_index_unavailable_message(),
            );
            return;
        }
```

`deny_e5506` emits `unreachable`, so the caller's following load or store
type-checks on the polymorphic stack.

- [ ] **Step 4: Run the four negative-index cases**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_bounds::a_negative`
Expected: 4 pass. If `d1`/`a1` (`f()[-1]`) still report only `E4000`, the
two-child index node is not a childless literal. Print it with a temporary
`eprintln!("{:?}", self.node(index_id))` in `is_negative_literal_index`, then
extend the predicate to the shape it shows (for example a unary `-` over a
literal) and remove the print. Do not move the check out of the choke point.

- [ ] **Step 5: Refuse the mutators.** In `emit/call.rs`, add next to `is_runtime_array_value`:

```rust
    /// `Some(method)` iff `node` is `<receiver>.<method>(…)` with `method` in
    /// `kali_common::RUNTIME_ARRAY_MUTATORS` and the receiver a plain runtime
    /// array (`is_runtime_array_value`). A plain array has a fixed length, so
    /// each of these refuses (array-bounds spec §3.2). A growable receiver's
    /// `push` is taken earlier, by `growable_push_call_parts`.
    pub(crate) fn plain_runtime_array_mutator(&self, node: &LirNode) -> Option<String> {
        if node.kind != LirNodeKind::Call || node.children.is_empty() {
            return None;
        }
        let callee = self.resolve_transparent_callable_node(node.children[0])?;
        let callee_node = self.node(callee);
        let method = callee_node.text.as_deref()?;
        if !kali_common::RUNTIME_ARRAY_MUTATORS.contains(&method) {
            return None;
        }
        let receiver = self.unwrap_transparent(*callee_node.children.first()?);
        self.is_runtime_array_value(receiver)
            .then(|| method.to_string())
    }
```

Directly after the growable-`push` arm
(`if let Some((receiver, args)) = self.growable_push_call_parts(node) { … }`, ~:1619), add:

```rust
        // A fixed-length runtime array cannot grow or shrink (array-bounds
        // spec §3.2). These calls used to reach the terminal warn + `0`
        // fallback, which never emits the receiver: a silent no-op.
        if let Some(method) = self.plain_runtime_array_mutator(node) {
            let message = kali_common::runtime_array_mutator_unavailable_message(&method);
            return self.deny_e5506(function, &message);
        }
```

- [ ] **Step 6: Refuse `.length =`.** In `emit/literal.rs`, inside
  `if self.array_bindings.contains(&base_name) {` (~:661) and before
  `let scratch = …`, add:

```rust
                            // Array-bounds spec §3.2: `a.length = v` reaches
                            // this arm as the text index `length`, which used
                            // to lower to element 0 and store there.
                            if matches!(&index, ArrayWriteIndex::Text(text) if text == "length") {
                                self.diagnostics.push(Diagnostic::error(
                                    e5::FEATURE_UNAVAILABLE as u32,
                                    kali_common::runtime_array_length_write_unavailable_message()
                                        .to_string(),
                                ));
                                function.instruction(&Instruction::I64Const(0));
                                return true;
                            }
```

This mirrors the growable-field `.length` refusal earlier in the same function.

- [ ] **Step 7: Run the cases and watch them pass**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_bounds`
Expected: all 28 pass. If `a5`/`d5` (module-scope `a.length = 1`) still exit
0, `assignment_target_name` did not resolve the module binding. Trace with
`kali run` and report it before widening anything.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/cases/runtime/array_bounds.toml
git commit -m "feat(array-bounds): a negative literal index, a length-changing method and a .length write refuse on a plain runtime array"
```

---

### Task 5: The capability-loss spike (gate)

**Files:**
- Create: `/tmp/claude-…/scratchpad/spike.md` (scratch, not committed). Its content goes into Task 7's followups file.

- [ ] **Step 1: Run the whole workspace**

Run: `cargo test --workspace 2>&1 | tee "$SCRATCH/spike-tests.txt" | grep -E "^test .* FAILED|test result:"`
(`SCRATCH` is the session scratchpad directory.)
Expected: every failure listed.

- [ ] **Step 2: Run the probes**

Run:
```bash
tools/array-return-probes/run.sh "$SCRATCH/probes-after.tsv"
diff <(cut -f1,2 tools/array-return-probes/baseline-bounds.tsv | sort) <(cut -f1,2 "$SCRATCH/probes-after.tsv" | sort)
```
Expected: the `bounds_*` rows move `SILENT → REFUSES` or `SILENT → TRAPS`,
and `bounds_g1` stays `SILENT`. Every row not named `bounds_*` keeps its
verdict, and any that moves is triaged in Step 3.

- [ ] **Step 3: Triage every moved test and probe.** For each one, write a
  row in `spike.md`: `name | before | after | class`. `class` is one of:
  - **wanted**: it pinned a silent value, so re-pin it to the refusal or
    trap, with a rationale citing `016557d60`.
  - **index shift**: it pins a function index or a module-wide instruction
    count (amendment A-1), so update the count.
  - **capability loss**: an in-bounds program, or a mutator call whose
    result was never observed, now refuses or traps.

  Re-pin the **wanted** and **index shift** rows.

- [ ] **Step 4: Gate.** If any row is **capability loss**, STOP. Report the
  rows to the human partner with the program text and node's output, and wait
  for a decision. Do not narrow a refusal on your own.

- [ ] **Step 5: Measure the in-place mutators outside the spec's list**
  (`reverse`, `sort`, `copyWithin`, `fill` on an existing binding). For each:

```bash
printf 'function main(){ const a = new Array(3).fill(4); a[0] = 1; a.reverse(); console.log(a[0]); } main();\n' > "$SCRATCH/rev.js"
node "$SCRATCH/rev.js"; ./target/debug/kali run "$SCRATCH/rev.js"; echo "exit $?"
```
Do the same with `a.sort()` (and `a[0] = 9` first, printing `a[0]`), with
`a.copyWithin(0, 2)` (printing `a[0]`, after `a[2] = 7`), and with
`a.fill(5)` (printing `a[1]`). Record the node output, kali's output and
the exit code in `spike.md`. These are filed in Task 7, not fixed.

- [ ] **Step 6: Commit the re-pins**

```bash
cargo test --workspace   # expected: PASS
git add -A crates tools
git commit -m "test(array-bounds): re-pin the silent rows the bounds guard and refusals moved"
```

---

### Task 6: The `kali_types` mirror

**Files:**
- Modify: `crates/kali_types/src/resolve/member.rs` (new fns near `call_returns_runtime_array`, ~:381; one call in `resolve_member_expression`, :6)
- Modify: `crates/kali_types/src/resolve/call.rs` (first statement of `resolve_call_expression`, :5)
- Modify: `crates/kali_types/src/resolve/expression.rs` (the `AssignmentExpression` arm, next to `reject_array_binding_scalar_reassignment(expr)`, ~:1945)
- Test: `crates/kali_types/src/resolve/member_tests.rs` (append)
- Modify: `crates/kali_cli/tests/cases/runtime/array_bounds.toml` (append the `check` twins)

**Interfaces:**
- Consumes: Task 2's messages, plus the existing `is_structural_runtime_array`,
  `is_growable_array_binding` and `call_returns_runtime_array`.
- Produces: `TypeContext::is_plain_runtime_array_receiver(&self, object: &Expression) -> bool`
  and the three `reject_runtime_array_*` gates.

- [ ] **Step 1: Write the failing unit tests** (append to `member_tests.rs`;
  `e5506_messages` is already defined there):

```rust
const NEG: &str = "a negative index on a runtime array is unavailable";
const MUT: &str = "on a runtime array is unavailable in the current phase";
const LEN: &str = "assigning to `.length` of a runtime array is unavailable";

fn any_contains(messages: &[String], needle: &str) -> bool {
    messages.iter().any(|message| message.contains(needle))
}

#[test]
fn a_negative_literal_index_on_a_filled_array_refuses() {
    let messages = e5506_messages(
        "function main(){ const a = new Array(3).fill(4); console.log(a[-1]); } main();",
    );
    assert!(any_contains(&messages, NEG), "{messages:?}");
}

#[test]
fn a_negative_literal_index_store_on_a_filled_array_refuses() {
    let messages =
        e5506_messages("function main(){ const a = new Array(3).fill(4); a[-1] = 9; } main();");
    assert!(any_contains(&messages, NEG), "{messages:?}");
}

#[test]
fn each_length_changing_method_on_a_filled_array_refuses() {
    for call in ["a.push(4)", "a.pop()", "a.shift()", "a.unshift(1)", "a.splice(0, 1)"] {
        let source =
            format!("function main(){{ const a = new Array(3).fill(4); {call}; }} main();");
        let messages = e5506_messages(&source);
        assert!(any_contains(&messages, MUT), "{call}: {messages:?}");
    }
}

#[test]
fn a_length_write_on_a_filled_array_refuses_for_every_operator() {
    for write in ["a.length = 1", "a.length -= 1"] {
        let source =
            format!("function main(){{ const a = new Array(3).fill(4); {write}; }} main();");
        let messages = e5506_messages(&source);
        assert!(any_contains(&messages, LEN), "{write}: {messages:?}");
    }
}

#[test]
fn in_bounds_access_and_a_growable_push_do_not_refuse() {
    for source in [
        "function main(){ const a = new Array(3).fill(4); a[2] = 1; console.log(a[2], a.length); } main();",
        "function main(){ const a = [1,2,3]; a.push(4); console.log(a.length); } main();",
    ] {
        let messages = e5506_messages(source);
        assert!(
            !any_contains(&messages, NEG) && !any_contains(&messages, MUT) && !any_contains(&messages, LEN),
            "{source}: {messages:?}"
        );
    }
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p kali_types resolve::member_tests`
Expected: the four refusal tests FAIL (no message), and the control passes.

- [ ] **Step 3: Implement the predicate and gates** (in `resolve/member.rs`,
  after `call_returns_runtime_array`; add the needed imports at the top of
  the file if they are missing: `kali_ast::{AssignmentExpression, CallExpression}`,
  `kali_error::Diagnostic`, `kali_error::_error_codes::e5`):

```rust
    /// The receiver is a plain fixed-length runtime array: a registered
    /// structural binding that is not growable, or a direct call to an
    /// array-returning declaration. The resolver's twin of codegen's
    /// `is_runtime_array_value` (array-bounds spec §3.3). An anonymous callee
    /// is not recognised, because `call_returns_runtime_array` keys by the bare
    /// name (amendment A-3).
    pub(crate) fn is_plain_runtime_array_receiver(&self, object: &Expression) -> bool {
        match object {
            Expression::Identifier(base) => {
                self.is_structural_runtime_array(base) && !self.is_growable_array_binding(base)
            }
            Expression::CallExpression(call) => self.call_returns_runtime_array(call),
            _ => false,
        }
    }

    /// `a[-1]`, read or store, on a plain runtime array (array-bounds spec §3.3).
    pub(crate) fn reject_runtime_array_negative_index(&mut self, expr: &MemberExpression) {
        let negative = expr
            .property
            .as_deref()
            .and_then(|property| property.parse::<i64>().ok())
            .is_some_and(|value| value < 0);
        if negative && self.is_plain_runtime_array_receiver(&expr.object) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::runtime_array_negative_index_unavailable_message().to_string(),
            ));
        }
    }

    /// `a.push(…)` and the other `RUNTIME_ARRAY_MUTATORS` on a plain runtime
    /// array (array-bounds spec §3.3).
    pub(crate) fn reject_runtime_array_mutator_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) = &expr.callee else {
            return;
        };
        let Some(method) = member.property.as_deref() else {
            return;
        };
        if member.computed_index.is_none()
            && kali_common::RUNTIME_ARRAY_MUTATORS.contains(&method)
            && self.is_plain_runtime_array_receiver(&member.object)
        {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::runtime_array_mutator_unavailable_message(method),
            ));
        }
    }

    /// `a.length = v`, with any assignment operator, on a plain runtime array
    /// (array-bounds spec §3.3).
    pub(crate) fn reject_runtime_array_length_write(&mut self, assign: &AssignmentExpression) {
        let Expression::MemberExpression(member) = &assign.left else {
            return;
        };
        if member.property.as_deref() == Some("length")
            && self.is_plain_runtime_array_receiver(&member.object)
        {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::runtime_array_length_write_unavailable_message().to_string(),
            ));
        }
    }
```

Wire them in:
- `resolve/member.rs` `resolve_member_expression`: add
  `self.reject_runtime_array_negative_index(expr);` after
  `self.reject_nonuniform_forin_key_object_access(expr);`.
- `resolve/call.rs` `resolve_call_expression`: add
  `self.reject_runtime_array_mutator_call(expr);` as the first statement.
  The function has several early `return`s, and the gate must run before all
  of them.
- `resolve/expression.rs`, the `AssignmentExpression` arm: add
  `self.reject_runtime_array_length_write(expr);` after
  `self.reject_literal_array_unfoldable_mutation(expr);`.

- [ ] **Step 4: Run the unit tests and watch them pass**

Run: `cargo test -p kali_types resolve::member_tests`
Expected: PASS. If the store test fails, the assignment target does not reach
`resolve_member_expression`. Call `reject_runtime_array_negative_index` on
`assign.left` from the `AssignmentExpression` arm as well, guarded so a
target already resolved through `resolve_member_expression` is not refused
twice. Then re-run.

- [ ] **Step 5: Write the `check` twins.** Append to `array_bounds.toml`:

```toml
[[case]]
name = "a_negative_literal_index_into_a_direct_call_refuses_at_check_declaration"
rationale = """The `check` twin of the `run` case (amendment A-4)."""
args = ["check", "d1.js"]
exit = "failure"
stderr_contains = ["E5506", "${NEG}"]

[[case]]
name = "a_negative_literal_index_into_a_filled_array_refuses_at_check"
rationale = """The `check` twin (amendment A-4)."""
args = ["check", "n1.js"]
exit = "failure"
stderr_contains = ["E5506", "${NEG}"]

[[case]]
name = "a_negative_literal_index_store_refuses_at_check"
rationale = """The `check` twin (amendment A-4)."""
args = ["check", "w2.js"]
exit = "failure"
stderr_contains = ["E5506", "${NEG}"]

[[case]]
name = "push_on_a_call_bound_array_refuses_at_check_declaration"
rationale = """The `check` twin (amendment A-4)."""
args = ["check", "d3.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${MUT}"]

[[case]]
name = "push_then_index_refuses_at_check_declaration"
rationale = """The `check` twin (amendment A-4)."""
args = ["check", "d4.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${MUT}"]

[[case]]
name = "a_length_write_refuses_at_check_declaration"
rationale = """The `check` twin (amendment A-4)."""
args = ["check", "d5.js"]
exit = "failure"
stderr_contains = ["E5506", "${LEN}"]

[[case]]
name = "a_length_write_on_a_filled_array_refuses_at_check"
rationale = """The `check` twin (amendment A-4)."""
args = ["check", "n3.js"]
exit = "failure"
stderr_contains = ["E5506", "${LEN}"]

[[case]]
name = "pop_on_a_filled_array_refuses_at_check"
rationale = """The `check` twin (amendment A-4)."""
args = ["check", "n4.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.pop()`", "${MUT}"]

[[case]]
name = "shift_on_a_filled_array_refuses_at_check"
rationale = """The `check` twin (amendment A-4)."""
args = ["check", "n7.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.shift()`", "${MUT}"]

[[case]]
name = "unshift_on_a_filled_array_refuses_at_check"
rationale = """The `check` twin (amendment A-4)."""
args = ["check", "n8.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.unshift()`", "${MUT}"]

[[case]]
name = "splice_on_a_filled_array_refuses_at_check"
rationale = """The `check` twin (amendment A-4)."""
args = ["check", "n9.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.splice()`", "${MUT}"]

[[case]]
name = "a_growable_push_still_passes_check"
rationale = """Control (spec §3.3): the mirror must not catch a growable-shape receiver."""
args = ["check", "ok_growable_push.js"]
exit = "success"

[[case]]
name = "the_anonymous_lane_static_refusals_are_run_only_negative_index"
rationale = """Amendment A-3: the resolver keys the array-return fact by the bare callee name, so it never proves an anonymous call's array (anon-array-return-discovered-defects.md §9). `run` refuses this program; `check` exits 0. Pinned so that closing §9 moves this case on purpose."""
args = ["check", "a1.js"]
exit = "success"

[[case]]
name = "the_anonymous_lane_static_refusals_are_run_only_push"
rationale = """Amendment A-3, as above."""
args = ["check", "a3.js"]
exit = "success"

[[case]]
name = "the_anonymous_lane_static_refusals_are_run_only_push_then_index"
rationale = """Amendment A-3, as above."""
args = ["check", "a4.js"]
exit = "success"

[[case]]
name = "the_anonymous_lane_static_refusals_are_run_only_length_write"
rationale = """Amendment A-3, as above."""
args = ["check", "a5.js"]
exit = "success"
```

- [ ] **Step 6: Run the cases**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_bounds`
Expected: all 44 pass. If an anonymous-lane `check` case now FAILS, meaning
`check` refuses, the resolver does recognise that receiver. Report it: A-3
would then be narrower than written, and the case should become a refusal
twin, not be deleted.

- [ ] **Step 7: Full gates**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS. New failures here belong to the mirror. Triage them as in
Task 5 Step 3, and stop on any capability loss.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_types crates/kali_cli/tests/cases/runtime/array_bounds.toml
git commit -m "feat(array-bounds): kali check mirrors the static runtime-array refusals"
```

---

### Task 7: Bookkeeping and the final measurement

**Files:**
- Modify: `specs/15-errors.md`
- Modify: `docs/superpowers/followups/anon-array-return-discovered-defects.md` (§1)
- Modify: `docs/superpowers/followups/array-return-discovered-defects.md` (§3, §6)
- Create: `docs/superpowers/followups/array-bounds-discovered-defects.md`
- Modify (only if a lane moved): `docs/superpowers/followups/kali-silent-miscompile-register.md`, `docs/superpowers/followups/blast-radius-ranking.md`

- [ ] **Step 1: `specs/15-errors.md`.**
  - In the `Use E5506 for cases such as:` list, before the final
    "any parse-supported construct…" bullet, add:
    `- on a plain fixed-length runtime array, a negative integer-literal index, a call to `push` / `pop` / `shift` / `unshift` / `splice`, or an assignment to `.length` — the array cannot represent the result, so kali refuses under both `check` and `run` (array-bounds spec §3.2-§3.3)`
  - In the `E4xxx` range clarification, after the `E4001` bullet, add:
    `- `E4000` also reports kali's own run-time refusal of an out-of-range runtime-array index: the trap follows a stderr line beginning `kali: array index out of bounds`. That is an honest refusal of a value kali cannot produce, not an internal failure (array-bounds spec §3.1).`

- [ ] **Step 2: Re-measure R-21 and the register.**

Run: `cargo test -p kali_cli --test cases -- oracle/` and
`grep -n "r21o" crates/kali_cli/tests/cases/oracle/tier2.toml`.
Expected: PASS, with `r21o` still `silent`, because it is the static-fold
lane. If any oracle case moved verdict, amend that register entry's §0.2 row
in the style of the existing amendments. Then regenerate the ranking with
`cargo run -p kali_blast_radius --example rank`, splice it, and run
`cargo test -p kali_blast_radius`. If nothing moved, edit neither file.

- [ ] **Step 3: Mark the closed followups.**
  - At the head of `anon-array-return-discovered-defects.md` §1, add
    **FIXED (fail-closed)** by array-bounds `<sha of the Task 6 commit>`. Say
    that every row now refuses or traps rather than printing node's value,
    that `undefined` stays open (R-21), and that a1/a3-a5 refuse at `run` only
    (spec amendment A-3).
  - In `array-return-discovered-defects.md` §3, add the same FIXED note for
    the `[-1]`, `[5]` and `push` rows. The closure-capture row stays open.
  - In §6, add one sentence: a runtime-array out-of-range read now traps,
    and `undefined` stays open.

- [ ] **Step 4: Write `array-bounds-discovered-defects.md`.** Follow the
  header convention of `anon-array-return-discovered-defects.md`: Filed
  by, Oracle, Measured at, then a Register paragraph. Include at minimum:
  - §1 the growable lane's out-of-range read (`bounds_g1`: node `undefined`,
    kali `0` at exit 0), still silent, with `emit_growable_index_read`
    (`emit/growable.rs:470`) as the site.
  - §2 the in-place mutators measured in Task 5 Step 5, with their rows.
  - §3 the anonymous-lane `check` gap (amendment A-3), pointing at
    `anon-array-return-discovered-defects.md` §9.
  - §4 the run-time-only out-of-range gap (spec §1.1).
  - §5 the capability-loss diff from Task 5, recomputed now:

```bash
tools/array-return-probes/run.sh "$SCRATCH/probes-final.tsv"
diff <(cut -f1,2 tools/array-return-probes/baseline-bounds.tsv | sort) <(cut -f1,2 "$SCRATCH/probes-final.tsv" | sort)
```

  Paste the diff, and the Task 5 triage table, verbatim.

- [ ] **Step 5: Final gates**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add specs/15-errors.md docs/superpowers/followups
git commit -m "docs(array-bounds): E5506/E4000 scope, closed followups, and what was measured and not fixed"
```
