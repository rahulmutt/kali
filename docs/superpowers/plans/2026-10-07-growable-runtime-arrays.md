# Growable Runtime Arrays Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** An array a program builds at runtime with `push` can be returned, passed, aliased and used at module scope, and `push`, `pop`, `.length`, index read and write, `indexOf`, `includes`, `slice`, `join` and `for-of` on it behave as node does or fail loudly (E5506 under `check` and `run` alike, or a runtime trap with a `kali:` message).

**Architecture:** One exhaustive syntactic walk (`kali_types/src/growable/facts.rs`) records every array value flow, origin, mutation and occurrence; a union-find solve (`growable/flow.rs`) decides, per connected component, whether the arrays in it are growable, and a position check (`growable/positions.rs`) turns every unsupported occurrence into an E5506. Repr inference runs the walk and the solve before its body walk (Phase A3, replacing Stage 4's allowlist), unions the element nodes of each growable component so the existing element solve gives one element repr (i64, f64 or string) per array, and writes the result into `ReprTable`. The resolver and codegen key every growable lane on that table. Codegen keeps Stage 4's header layout (`[len][cap][data_ptr]`, handle tagged with bit 62) and adds bounds-checked synthetics for index access, `pop`, search, `slice` and float `join`.

**Tech Stack:** Rust (workspace crates `kali_common`, `kali_types`, `kali_codegen`, `kali_cli`), the TOML `cases` runner, node v26.10.0 as the oracle.

**Spec:** `docs/superpowers/specs/2026-10-07-growable-runtime-arrays-design.md`

## Global Constraints

- Every compile-time refusal is `E5506` (`kali_error::_error_codes::e5::FEATURE_UNAVAILABLE`). No new diagnostic code; `specs/15-errors.md` gains no code.
- Message texts live in `crates/kali_common/src/messages.rs` and are used verbatim as written in Task 1. The two runtime traps print `kali: array index out of bounds…` (the existing `runtime_array_index_out_of_bounds_message`) and `kali: pop on empty array…` (Task 1).
- Every refusal this project adds is raised in repr inference (`kali_types`), never only in codegen, so `check` and `run` report it alike. `check` accepts a program that only traps at run time.
- Rust unit tests go in sibling `*_tests.rs` files wired with `#[cfg(test)] #[path = "…_tests.rs"] mod …;`, never inline test bodies.
- Black-box tests are `.toml` files under `crates/kali_cli/tests/cases/`, run by the single `cases` target (see its `README.md`). No new `tests/*.rs` targets.
- Every expected stdout in a case file is node v26.10.0's output for the same program, measured with `env -u FORCE_COLOR node FILE.js`.
- `new Array(n)` keeps its inline `[len][elem…]` layout; the `Repr::GrowableArrayI64` object-field lane is unchanged.
- Escaping growable arrays allocate their header and data blocks with `__alloc_global` and are never reclaimed.
- Resource rules (hard pod limits — breaking them kills the session):
  - build and test with `CARGO_TARGET_DIR` inside the worktree: `CARGO_TARGET_DIR=/workspace/.worktrees/growable-runtime-arrays/target`;
  - always `-j 6` for cargo and `--test-threads=6` for test binaries;
  - never put a target directory under `/tmp`;
  - launch anything longer than a few minutes detached: `setsid nohup bash -c '…; echo $? > EXITFILE' > /dev/null 2>&1 < /dev/null &`, then poll for `EXITFILE`;
  - never `pkill -f`.
- `T` below means `/workspace/.worktrees/growable-runtime-arrays/target`; `K` means `$T/debug/kali`; `$S` means the session scratchpad directory.

## Corrections to the spec found while planning

These are carried out by the tasks below and recorded in the spec's amendments section in Task 13. All measurements are at `edb3a77df` with `/workspace/target/debug/kali` (built at that commit) and node v26.10.0, on 2026-10-07.

- **A-1. A literal passed as an argument is refused today, and that is the lane kept.** Measured: `function sum(a) {…} const a = [1, 2]; console.log(sum(a));` → `run`: `error[E5506]: passing an array literal to function 'sum' is unavailable…` (raised in codegen, `emit/call.rs:3742`); `check` exits 0 (a pre-existing check/run asymmetry, recorded in followups). The same holds for a callee that reads `.length`, iterates with `for-of` (refused by the for-of gate) or joins (`elements of 'a' at module scope are used as both strings and numbers`). So "keeps every lane it has today" means: a literal whose component never becomes growable keeps that refusal.
- **A-2. A source is a component, not a binding.** §3.1 makes a literal binding a source only when *it* is pushed, popped, written or returned. Measured: `const a = [1, 2, 3]; return a;` and `return [4, 5];` already run on the array-return lane (plain layout, `ret1.js` → `3 2`, `ret2.js` → `2 4`), so "returned" cannot be a source without moving working programs. The plan's rule: a connected component of array values is growable when it holds a literal-initialized binding **and** a length/element mutation (`push`, `pop`, an index write) anywhere in it. `function add(a) { a.push(1) } const xs = [0]; add(xs);` is growable through the parameter; a returned literal whose caller pushes the result becomes growable; a returned literal nobody mutates stays on the array-return lane.
- **A-3. The position scan is kept, repurposed.** A growable handle used as a plain value (`typeof xs`, `xs + ""`, `String(xs)`, `{ a: xs }`, `[xs]`, `xs === ys`, a `for-in` over it) would print or compute on the raw tagged handle. Retiring Stage 4's allowlist outright would admit those. The allowlist no longer decides promotion; it becomes a post-solve position check (`growable/positions.rs`) that refuses every growable occurrence outside the supported positions with a new message (`growable_plain_use_message`).
- **A-4. Mixed number/string elements reuse the existing message.** `elements of 'a' in 'f' are used as both strings and numbers` (`repr_infer.rs:7700`, measured on `lit_arg4.js`) already fires for a mixed store; no new text. The new "unsupported element" message covers objects, arrays, functions, booleans (which kali stores as `1`/`0`), `null`, `undefined` and holes.
- **A-5. An array literal written directly as an argument or `return` value in a growable component is refused** (`growable_literal_expression_message`: "bind it to a `const` first"). Codegen has no lane that allocates a literal expression as a growable array outside a declarator.
- **A-6. Index access, `push` and `pop` directly on a call or `slice` result are refused** (`f()[0]`, `f().push(1)`, `a.slice(1)[0]`; `growable_temporary_use_message`). `.length`, `.join`, `.indexOf`, `.includes`, `.slice` and `for-of` on them are admitted — the corpus shape `for (const line of wrap(…))` and `xs.slice(1, 3).join("-")` need exactly those.
- **A-7. Non-array writes are refused.** A binding, parameter or return value in a growable component that is also given a non-array value (`let x = a; x = 5`, `f(xs); f(5)`, a missing argument, a bare `return;`, falling off the end, `let x;`) is refused (`growable_non_array_write_message`). §3.2 covers only the plain-versus-growable mix.
- **A-8. The snapshot rule is component-based.** The baseline rule (`growable::statement_contains_push_on`) is syntactic and name-based. It misses `const b = a; for (const x of a) b.push(x)` and `for (const x of a) add(a, x)` where `add` pushes its parameter — node grows the iteration, kali's snapshot does not. The plan refuses a `push`/`pop` inside a `for-of` body on any binding in the iterated array's growable component, and a call in the body that passes a member of that component to a parameter that may (transitively) be pushed or popped. This is conservative: two arrays that share a component because both were passed to the same function count as one.
- **A-9. The console fix is the removal of a stale guard.** The guard at `emit/call.rs:1067-1088` ("console output of multiple arguments where one reads a growable array") predates the multi-argument lane at `emit/call.rs:1092`, which now joins every argument with `" "` through `emit_as_string`. The guard is kept for growable object *fields* only (that lane is unchanged; `soundness/structured_clone.toml:280` stays pinned). The whole-array print refusal is codegen-only at the baseline (`kali check cons1.js` exits 0 while `run` refuses, measured); this project raises it for growable arrays in inference so `check` agrees. The plain-array asymmetry goes to followups.
- **A-10. Nested `for-of` needs per-depth scratch locals and the removal of a codegen-only refusal.** Measured: `nest.js` (two growable loops nested) → `check` exits 0, `run` refuses with "a for-of over a growable array nested inside another…" (`intrinsics/array.rs:1168`). The plan reserves an index/length/handle local triple per nesting depth.
- **A-11. Large float literals break codegen at the baseline.** Measured: `let x = 0.5; console.log("c" + x * 1e21)` and `1e20 * x` → `error[E4201]: failed to load WASM module`; `Math.pow(10, 21)` and `10 ** 21` silently print `3875820019684212736` (node `1e+21`). §5.2's `1e21` join case therefore builds the value by doubling in a loop (`1.1805916207174113e+21`). Both defects go to followups.
- **A-12. `float_to_string` already renders `-0` as `0`.** `format_js_number` (`kali_common/src/js_number.rs:28`) returns `"0"` for ±0, which is `join`'s rendering; measured `"b" + (-0 * x)` → `b0` under kali and node. `__join_growable_f64` calls it unchanged.
- **A-13. `slice` bounds that are floats are truncated, not refused.** Codegen applies `i64.trunc_sat_f64_s`, which is ToIntegerOrInfinity for every input (NaN → 0, ±Infinity saturates and then clamps).
- **A-14. The search value counts as an element store for the element solve.** A float needle makes the array f64; a number needle into a string array (or the reverse) is the existing mixed-elements refusal (A-4). No search-specific message.
- **A-15. Codegen emitters are pinned by black-box cases.** There are no growable emitter unit tests to extend (`grep -l growable crates/kali_codegen/src/**/*_tests.rs` is empty) and codegen unit tests run without repr inference. The new synthetic bodies are pinned by a wasm-validation unit test (Task 9, Task 10); every emitter by cases.
- **A-16. "Never leaves its creating function"** is computed as: the binding is the only node of its growable component and is not at module scope. Everything else (returned, passed, aliased, sliced, module scope) allocates globally.
- **A-17. The reduced `wrap_paragraph` tracks width in a number.** Measured: `let line = "> "; if (line.length > 3) { line = "> "; }` inside a loop → `error[E5506]: reassigning an array binding to a non-array value…` (`.length` on a `let` string registers it as an array binding; pre-existing, 7 corpus lines). The case keeps a `used` counter instead of `line.length`; the defect goes to followups. Measured at the baseline with a `new Array(9)` word list, the string work of the reduced wrap prints node's three lines.
- **A-18. A function using a module-level growable array needs its own message, raised in inference.** Measured: `const out = [1, 2]; function size() { return out.length; }` (and `const out = []`) → `run`: `error[E5506]: reading module binding 'out' from a function is only available for compile-time-constant `const` initializers`, but `check` exits 0 — the existing refusal lives in codegen only. For a growable array the new `growable_module_read_message` is a shape conflict, so `check` and `run` agree; the plain-array asymmetry goes to followups.
- **A-19. `var` literal declarators are literal origins too**, like `const` and `let`; keys are `(function, name)` either way.
- **A-20. 36 programs, not 37, report an array-family refusal at the baseline.** Measured with the §2.2 families (Task 13 Step 3's command): 36 of 40. The other four stop at a refusal outside the array families (`event_dispatcher.js`: object-literal property form; `path_normalize.js`, `reverse_words.js`: parameter form; `task_queue.js`: object default parameter). The §2.2 line counts reproduce exactly (386; 86, 65, 52, 22, 10, 10, 7, 3, 1).

## Review Focus

1. **A growable array passed to a function that also receives `new Array(n)` elsewhere** (`total(xs); total(p)` with `p = new Array(2)`): a reader expects a refusal naming the parameter, not a miscompile reading one layout as the other. Pinned in Task 3 (`a_parameter_fed_a_growable_and_an_allocation_is_a_mixed_layout_at_the_parameter`) and Task 12 (`mixed_layout_parameter_is_refused`).
2. **An alias pushed through, with `.length` read on the original** (`const b = a; b.push(6); console.log(a.length)` → `2`), and the same alias pushed inside a `for-of` over the original (refused). Pinned in Task 9 (`alias_push_is_seen_through_the_original`) and Task 5 (`a_push_through_an_alias_inside_a_for_of_over_the_original_is_refused`).
3. **`slice` of a slice** (`a.slice(1, 4).slice(1).join(",")` → `20,30`): the inner result is a temporary growable handle that must be sliced again, not re-read from `a`. Pinned in Task 10 (`slice_of_a_slice_matches_node`).
4. **A `for-of` body that calls a function pushing to the iterated array through a parameter** (`for (const x of a) add(a, x)`): node grows the iteration; kali's snapshot would not. Refused. Pinned in Task 5 (`a_call_that_pushes_the_iterated_array_through_a_parameter_is_refused`) and Task 12.
5. **f64 `-0` and NaN in `join`, `indexOf` and `includes`** (`[NaN, -0, 2.5]`: `indexOf(NaN)` → `-1`, `includes(NaN)` → `true`, `indexOf(0)` → `1`, `join` renders `-0` as `0`). Pinned in Task 10 (`f64_search_follows_strict_and_same_value_zero`, `floats_join_like_node`).

---

## File Structure

| file | change | responsibility |
|---|---|---|
| `crates/kali_common/src/messages.rs` | modify | every refusal and trap text (Task 1) |
| `crates/kali_common/src/messages_tests.rs` | modify | texts pinned |
| `crates/kali_common/src/repr.rs` | modify | `growable_returns`, `growable_local_only` (Task 2) |
| `crates/kali_common/src/repr_tests.rs` | modify | the two new tables |
| `crates/kali_types/src/growable.rs` → `crates/kali_types/src/growable/mod.rs` | move, then shrink | keeps only `array_literal_of_scalar_seeds` (object-field lane) and its helpers |
| `crates/kali_types/src/growable_tests.rs` → `crates/kali_types/src/growable/growable_tests.rs` | move, then shrink | tests of what `mod.rs` keeps |
| `crates/kali_types/src/growable/flow.rs` (+ `flow_tests.rs`) | create | fact types, union-find solve, conflicts (Task 3) |
| `crates/kali_types/src/growable/facts.rs` (+ `facts_tests.rs`) | create | the exhaustive fact walk (Task 4) |
| `crates/kali_types/src/growable/positions.rs` (+ `positions_tests.rs`) | create | the post-solve position, capture, module-read and snapshot checks (Task 5) |
| `crates/kali_types/src/repr_infer.rs` | modify | Phase A3, Phase B arms, element unions, table writes, refusals (Tasks 4, 6, 7) |
| `crates/kali_types/src/repr_infer_tests.rs` | modify | edges and refusals end to end through `infer_reprs` |
| `crates/kali_types/src/array_return.rs` (+ `array_return_tests.rs`) | modify | growable-returning functions leave the array-return lane (Task 6) |
| `crates/kali_types/src/scope.rs`, `resolve/expression.rs`, `resolve/mod.rs`, `resolve/function.rs`, `resolve/member.rs`, `static_analysis/array.rs` | modify | resolver mirrors keyed on the table (Task 8) |
| `crates/kali_types/src/static_analysis/array_tests.rs`, `static_analysis/array_tests/growable.rs` | modify, create | resolver tests |
| `crates/kali_codegen/src/lower.rs` | modify | five synthetics, local reservation (Tasks 9-11) |
| `crates/kali_codegen/src/emitter.rs` | modify | param registration, synthetic indices, for-of stack (Tasks 9, 11) |
| `crates/kali_codegen/src/emit/growable.rs` (+ new `emit/growable_tests.rs`) | modify, create | value recognizer, alloc, push, index, pop, search, slice (Tasks 9, 10) |
| `crates/kali_codegen/src/emit/control_flow.rs` | modify | declarator lane (Task 9) |
| `crates/kali_codegen/src/emit/literal.rs` | modify | index write (Task 9) |
| `crates/kali_codegen/src/emit/call.rs` | modify | method dispatch, join, console, literal-argument guard (Tasks 9-11) |
| `crates/kali_codegen/src/emit/operators.rs` | modify | `is_string_valued`/`is_float_valued`/concat-taint arms (Task 10) |
| `crates/kali_codegen/src/intrinsics/array.rs` | modify | the runtime for-of loop (Task 11) |
| `crates/kali_cli/tests/runtime_smoke.rs` | modify | synthetic mirror list |
| `crates/kali_cli/tests/cases/array/growable_layout.toml`, `growable_runtime_arrays_traps.toml` | create | Task 9 |
| `crates/kali_cli/tests/cases/array/growable_methods.toml` | create | Task 10 |
| `crates/kali_cli/tests/cases/array/growable_for_of.toml` | create | Task 11 |
| `crates/kali_cli/tests/cases/array/growable_runtime_arrays.toml`, `growable_runtime_arrays_refused.toml` | create | Task 12 |
| `specs/19-feature-maturity.md` | modify | stale row corrected, new row (Task 13) |
| `docs/superpowers/specs/2026-10-07-growable-runtime-arrays-design.md` | modify | §7 amendments (Task 13) |
| `docs/superpowers/followups/growable-runtime-arrays-discovered-defects.md` | create | residue (Task 13) |

---

### Task 0: Worktree and baseline

- [ ] **Step 1: Confirm the branch**

```bash
cd /workspace/.worktrees/growable-runtime-arrays
git status --short          # must be empty
git log --oneline -3        # this plan, the spec commit, then edb3a77df
```

- [ ] **Step 2: Build the baseline binary, detached**

```bash
cd /workspace/.worktrees/growable-runtime-arrays
rm -f "$S/b0.exit"
setsid nohup bash -c "CARGO_TARGET_DIR=/workspace/.worktrees/growable-runtime-arrays/target cargo build -j 6 -p kali_cli --bin kali > $S/b0.log 2>&1; echo \$? > $S/b0.exit" > /dev/null 2>&1 < /dev/null &
```

Poll until `$S/b0.exit` exists (about 15 minutes cold): `while [ ! -f "$S/b0.exit" ]; do sleep 30; done; cat "$S/b0.exit"`. Expected: `0`.

- [ ] **Step 3: Confirm the baseline refusals**

```bash
cd "$S" && cat > p1.js <<'EOF'
function build(n) { const out = []; for (let i = 0; i < n; i++) out.push(i * i); return out; }
const xs = build(5);
console.log(xs.length, xs[2]);
EOF
/workspace/.worktrees/growable-runtime-arrays/target/debug/kali run p1.js
```

Expected: `error[E5506]: calling `.push()` on a literal array is unavailable in the current phase…`, exit 1.

---
### Task 1: Refusal and trap messages

**Files:**
- Modify: `crates/kali_common/src/messages.rs` (append at the end)
- Test: `crates/kali_common/src/messages_tests.rs` (append)

**Interfaces:**
- Produces (all `pub`, re-exported from `kali_common` by `pub use messages::*`):
  - `growable_scope_phrase(func: &str) -> String` — `at module scope` for `_start`, else ``in `func` ``
  - `growable_binding_subject(name: &str, func: &str) -> String`
  - `growable_return_subject(func: &str) -> String`
  - `growable_call_result_source(callee: &str) -> String`, `growable_slice_result_source() -> &'static str`
  - `growable_mixed_layout_message(subject: &str) -> String` (M1)
  - `growable_unsupported_element_message(subject: &str) -> String` (M2)
  - `growable_module_read_message(name: &str, func: &str) -> String` (M3)
  - `growable_capture_message(subject: &str) -> String` (M4)
  - `growable_for_of_mutation_message(subject: &str) -> String` (M5)
  - `growable_from_index_message(method: &str) -> String` (M6)
  - `growable_length_write_message(subject: &str) -> String` (M7)
  - `growable_unsupported_operation_message(operation: &str, subject: &str) -> String` (M8)
  - `runtime_array_print_unavailable_message() -> &'static str` (M9, the existing codegen text moved here verbatim)
  - `growable_plain_use_message(subject: &str) -> String` (M10)
  - `growable_literal_expression_message(func: &str) -> String` (M11)
  - `growable_temporary_use_message(source: &str) -> String` (M12)
  - `growable_non_array_write_message(subject: &str) -> String` (M13)
  - `growable_pop_empty_message() -> &'static str` (trap)

- [ ] **Step 1: Write the failing test**

Append to `crates/kali_common/src/messages_tests.rs`:

```rust
#[test]
fn growable_runtime_array_messages_name_the_array_and_say_why() {
    assert_eq!(growable_scope_phrase("_start"), "at module scope");
    assert_eq!(growable_scope_phrase("main"), "in `main`");
    assert_eq!(growable_binding_subject("xs", "main"), "`xs` in `main`");
    assert_eq!(growable_binding_subject("xs", "_start"), "`xs` at module scope");
    assert_eq!(growable_return_subject("build"), "the array `build` returns");
    assert_eq!(growable_call_result_source("build"), "the array `build(…)` returns");
    assert_eq!(growable_slice_result_source(), "a `slice()` result");
    assert_eq!(
        growable_mixed_layout_message("`a` in `total`"),
        "`a` in `total` would hold both a growable array (built with `push`, `pop` or an index write) and a fixed-length `new Array(n)` array; mixing the two array layouts is unavailable in the current phase"
    );
    assert_eq!(
        growable_unsupported_element_message("`o` in `main`"),
        "`o` in `main` is a growable array with an element that is an object, an array, a function, a boolean, `null` or `undefined`; a growable array holds only numbers or only strings in the current phase"
    );
    assert_eq!(
        growable_module_read_message("out", "size"),
        "function `size` uses the module-level growable array `out`; a function can reach a module-level growable array only through a parameter in the current phase"
    );
    assert_eq!(
        growable_capture_message("`o` in `main`"),
        "the growable array `o` in `main` is captured by a closure, nested function or class body; capturing a growable array is unavailable in the current phase"
    );
    assert_eq!(
        growable_for_of_mutation_message("`a` in `main`"),
        "`push` or `pop` on the growable array `a` in `main`, directly, through an alias or through a function it is passed to, inside a `for-of` loop over that same array is unavailable in the current phase: kali fixes the iteration count when the loop starts"
    );
    assert_eq!(
        growable_from_index_message("indexOf"),
        "`indexOf` with a `fromIndex` argument on a growable array is unavailable in the current phase"
    );
    assert_eq!(
        growable_length_write_message("`a` in `main`"),
        "assigning to `.length` of the growable array `a` in `main` is unavailable in the current phase"
    );
    assert_eq!(
        growable_unsupported_operation_message("`.reverse()`", "`a` in `main`"),
        "`.reverse()` on the growable array `a` in `main` is unavailable in the current phase; a growable array supports `push`, `pop`, `indexOf`, `includes`, `slice`, `join`, `.length`, index reads and writes, and `for-of`"
    );
    assert_eq!(
        runtime_array_print_unavailable_message(),
        "printing a whole runtime array is unavailable in the current phase: kali would print its handle; print its elements instead"
    );
    assert_eq!(
        growable_plain_use_message("`a` in `main`"),
        "the growable array `a` in `main` is used as a plain value here; a growable array can only be bound, passed to a function, returned, iterated with `for-of`, or used through `.length`, an index or a supported method in the current phase"
    );
    assert_eq!(
        growable_literal_expression_message("f"),
        "an array literal written directly as a call argument or `return` value in `f` would be a growable array; bind it to a `const` first in the current phase"
    );
    assert_eq!(
        growable_temporary_use_message("the array `make(…)` returns"),
        "indexing, `push` or `pop` directly on the array `make(…)` returns is unavailable in the current phase; bind it to a `const` first"
    );
    assert_eq!(
        growable_non_array_write_message("`a` in `f`"),
        "`a` in `f` holds a growable array and is also given a value that is not an array (another value, `undefined`, or a missing argument or `return`); this is unavailable in the current phase"
    );
    assert_eq!(
        growable_pop_empty_message(),
        "kali: pop on empty array: node returns undefined here; kali refuses rather than return a value that is not there"
    );
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_common growable_runtime_array_messages -- --test-threads=6`
Expected: compile error, `cannot find function growable_scope_phrase`.

- [ ] **Step 3: Add the messages**

Append to `crates/kali_common/src/messages.rs`:

```rust
/// Growable-runtime-arrays spec §3.6: where a growable array lives, as a
/// refusal names it. `_start` is the synthetic module-scope function.
pub fn growable_scope_phrase(func: &str) -> String {
    if func == "_start" {
        "at module scope".to_string()
    } else {
        format!("in `{func}`")
    }
}

/// Growable-runtime-arrays spec §3.6: a binding or parameter.
pub fn growable_binding_subject(name: &str, func: &str) -> String {
    format!("`{name}` {}", growable_scope_phrase(func))
}

/// Growable-runtime-arrays spec §3.6: a function's returned array.
pub fn growable_return_subject(func: &str) -> String {
    format!("the array `{func}` returns")
}

/// Growable-runtime-arrays spec A-6: a call result used without a binding.
pub fn growable_call_result_source(callee: &str) -> String {
    format!("the array `{callee}(…)` returns")
}

/// Growable-runtime-arrays spec A-6: a `slice` result used without a binding.
pub const fn growable_slice_result_source() -> &'static str {
    "a `slice()` result"
}

/// Growable-runtime-arrays spec §3.2 (M1).
pub fn growable_mixed_layout_message(subject: &str) -> String {
    format!(
        "{subject} would hold both a growable array (built with `push`, `pop` or an index write) and a fixed-length `new Array(n)` array; mixing the two array layouts is unavailable in the current phase"
    )
}

/// Growable-runtime-arrays spec §3.4, A-4 (M2).
pub fn growable_unsupported_element_message(subject: &str) -> String {
    format!(
        "{subject} is a growable array with an element that is an object, an array, a function, a boolean, `null` or `undefined`; a growable array holds only numbers or only strings in the current phase"
    )
}

/// Growable-runtime-arrays spec §3.3, A-18 (M3).
pub fn growable_module_read_message(name: &str, func: &str) -> String {
    format!(
        "function `{func}` uses the module-level growable array `{name}`; a function can reach a module-level growable array only through a parameter in the current phase"
    )
}

/// Growable-runtime-arrays spec §1.1 (M4).
pub fn growable_capture_message(subject: &str) -> String {
    format!(
        "the growable array {subject} is captured by a closure, nested function or class body; capturing a growable array is unavailable in the current phase"
    )
}

/// Growable-runtime-arrays spec §3.5, A-8 (M5).
pub fn growable_for_of_mutation_message(subject: &str) -> String {
    format!(
        "`push` or `pop` on the growable array {subject}, directly, through an alias or through a function it is passed to, inside a `for-of` loop over that same array is unavailable in the current phase: kali fixes the iteration count when the loop starts"
    )
}

/// Growable-runtime-arrays spec §3.5 (M6).
pub fn growable_from_index_message(method: &str) -> String {
    format!(
        "`{method}` with a `fromIndex` argument on a growable array is unavailable in the current phase"
    )
}

/// Growable-runtime-arrays spec §3.5 (M7).
pub fn growable_length_write_message(subject: &str) -> String {
    format!(
        "assigning to `.length` of the growable array {subject} is unavailable in the current phase"
    )
}

/// Growable-runtime-arrays spec §1.1 (M8). `operation` is already quoted,
/// e.g. "`.reverse()`" or "`[\"push\"]()`".
pub fn growable_unsupported_operation_message(operation: &str, subject: &str) -> String {
    format!(
        "{operation} on the growable array {subject} is unavailable in the current phase; a growable array supports `push`, `pop`, `indexOf`, `includes`, `slice`, `join`, `.length`, index reads and writes, and `for-of`"
    )
}

/// Growable-runtime-arrays spec §3.5 (M9): printing a whole array, growable
/// or plain. Moved verbatim from `kali_codegen/src/emit/call.rs`.
pub const fn runtime_array_print_unavailable_message() -> &'static str {
    "printing a whole runtime array is unavailable in the current phase: kali would print its handle; print its elements instead"
}

/// Growable-runtime-arrays spec A-3 (M10).
pub fn growable_plain_use_message(subject: &str) -> String {
    format!(
        "the growable array {subject} is used as a plain value here; a growable array can only be bound, passed to a function, returned, iterated with `for-of`, or used through `.length`, an index or a supported method in the current phase"
    )
}

/// Growable-runtime-arrays spec A-5 (M11).
pub fn growable_literal_expression_message(func: &str) -> String {
    format!(
        "an array literal written directly as a call argument or `return` value {} would be a growable array; bind it to a `const` first in the current phase",
        growable_scope_phrase(func)
    )
}

/// Growable-runtime-arrays spec A-6 (M12).
pub fn growable_temporary_use_message(source: &str) -> String {
    format!(
        "indexing, `push` or `pop` directly on {source} is unavailable in the current phase; bind it to a `const` first"
    )
}

/// Growable-runtime-arrays spec A-7 (M13).
pub fn growable_non_array_write_message(subject: &str) -> String {
    format!(
        "{subject} holds a growable array and is also given a value that is not an array (another value, `undefined`, or a missing argument or `return`); this is unavailable in the current phase"
    )
}

/// Growable-runtime-arrays spec §3.6: what `__growable_pop` prints on stderr
/// before it traps. Names kali, like the bounds trap, because node raises
/// nothing here.
pub const fn growable_pop_empty_message() -> &'static str {
    "kali: pop on empty array: node returns undefined here; kali refuses rather than return a value that is not there"
}
```

- [ ] **Step 4: Use M9 in codegen**

In `crates/kali_codegen/src/emit/call.rs`, replace both string literals at `:123` and `:170` (inside `emit_console_argument` and `emit_console_argument_as_string`) with `kali_common::runtime_array_print_unavailable_message().to_string()`.

- [ ] **Step 5: Run the tests**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_common growable_runtime_array_messages -- --test-threads=6`, then `CARGO_TARGET_DIR=$T cargo build -j 6 -p kali_codegen`.
Expected: PASS; the build is clean.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_common/src/messages.rs crates/kali_common/src/messages_tests.rs crates/kali_codegen/src/emit/call.rs
git commit -m "feat(growable-runtime-arrays): refusal and trap messages (spec §3.6, A-3..A-7, A-18)"
```

---

### Task 2: `ReprTable` growable returns and local-only bindings

**Files:**
- Modify: `crates/kali_common/src/repr.rs` (struct at `:83-108`, accessors after `is_growable_array_binding` at `:627`)
- Test: `crates/kali_common/src/repr_tests.rs` (append)

**Interfaces:**
- Produces:
  - `ReprTable::set_growable_return(&mut self, func: &str, elem: Repr)` / `ReprTable::growable_return(&self, func: &str) -> Option<Repr>` — `func` returns a growable array whose elements are `elem`.
  - `ReprTable::mark_growable_local_only(&mut self, func: &str, binding: &str)` / `ReprTable::is_growable_local_only(&self, func: &str, binding: &str) -> bool` — the binding's growable array never leaves `func` (spec §3.3, A-16); absent means "allocate globally".

- [ ] **Step 1: Write the failing test**

Append to `crates/kali_common/src/repr_tests.rs`:

```rust
#[test]
fn growable_returns_carry_their_element_repr() {
    let mut table = ReprTable::default();
    assert_eq!(table.growable_return("build"), None);
    table.set_growable_return("build", Repr::String);
    assert_eq!(table.growable_return("build"), Some(Repr::String));
    assert_eq!(table.growable_return("other"), None);
}

#[test]
fn growable_bindings_are_global_unless_marked_local_only() {
    let mut table = ReprTable::default();
    table.set_growable_array_binding("main", "out");
    assert!(!table.is_growable_local_only("main", "out"));
    table.mark_growable_local_only("main", "out");
    assert!(table.is_growable_local_only("main", "out"));
    assert!(!table.is_growable_local_only("main", "other"));
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_common growable_ -- --test-threads=6`
Expected: compile error, `no method named growable_return`.

- [ ] **Step 3: Add the fields and accessors**

In `ReprTable` (after `growable_array_bindings` at `:103`):

```rust
    /// Functions whose every return is a growable array (growable-runtime-
    /// arrays spec §3.1), with the element repr. Disjoint from
    /// `array_returns`: a function is on one array-return lane or neither.
    growable_returns: HashMap<String, Repr>,
    /// Growable bindings whose array never leaves the creating function
    /// (spec §3.3, A-16): codegen may allocate them from the function arena.
    /// Every other growable binding allocates with `__alloc_global`.
    growable_local_only: HashSet<(String, String)>,
```

After `is_growable_array_binding` (`:627-630`):

```rust
    pub fn set_growable_return(&mut self, func: &str, elem: Repr) {
        self.growable_returns.insert(func.to_string(), elem);
    }

    pub fn growable_return(&self, func: &str) -> Option<Repr> {
        self.growable_returns.get(func).copied()
    }

    pub fn mark_growable_local_only(&mut self, func: &str, binding: &str) {
        self.growable_local_only
            .insert((func.to_string(), binding.to_string()));
    }

    pub fn is_growable_local_only(&self, func: &str, binding: &str) -> bool {
        self.growable_local_only
            .contains(&(func.to_string(), binding.to_string()))
    }
```

`ReprTable` derives `Default`, so no constructor changes.

- [ ] **Step 4: Run the tests**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_common -- --test-threads=6`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/repr.rs crates/kali_common/src/repr_tests.rs
git commit -m "feat(growable-runtime-arrays): ReprTable growable returns and local-only bindings"
```

---

### Task 3: The growable solve (`growable/flow.rs`)

**Files:**
- Move: `crates/kali_types/src/growable.rs` → `crates/kali_types/src/growable/mod.rs` and `crates/kali_types/src/growable_tests.rs` → `crates/kali_types/src/growable/growable_tests.rs` (`git mv`; the `#[path = "growable_tests.rs"]` attribute stays as it is, because it resolves relative to `growable/`)
- Create: `crates/kali_types/src/growable/flow.rs`, `crates/kali_types/src/growable/flow_tests.rs`
- Modify: `crates/kali_types/src/growable/mod.rs` (add `pub(crate) mod flow;` after the `use` lines)

**Interfaces:**
- Produces (all in `crate::growable::flow`):
  - `pub(crate) const TOP_LEVEL: &str = "_start";`
  - `enum GrowNode { Binding(String, String), Return(String), Temp(usize) }` (derives `Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash`)
  - `enum TempKind { Slice, Merge, LiteralExpression, Allocation }`
  - `enum UseKind { Flow, Push, Pop, IndexRead, IndexWrite, LengthRead, LengthWrite, ForOf, Join, Slice, Search { method: String, from_index: bool }, Console, Method(String), Plain, Captured, ModuleRead }`
  - `struct Use { node: GrowNode, site: String, kind: UseKind }`
  - `enum ElementValue { Identifier(String), Unsupported, Other }`
  - `struct CallFact { site: String, callee: String, index: usize, node: GrowNode }`
  - `struct LoopFacts { iterable: GrowNode, site: String, mutations: Vec<GrowNode>, calls: Vec<CallFact> }`
  - `struct GrowFacts { edges, literal_origins, plain_origins, demands, non_array_writes, temp_kinds, uses, calls, loops, element_values, opaque_sites, temps }` (field types below)
  - `enum GrowConflict { MixedLayout(GrowNode), NonArrayWrite(GrowNode), LiteralExpression(String) }`
  - `struct GrowSolution` with `component_of`, `is_growable`, `is_growable_binding`, `same_component`, `growable_members`, `members_of`, `is_local_only`, and `pub(crate) conflicts: Vec<GrowConflict>`
  - `fn solve(facts: &GrowFacts) -> GrowSolution`

- [ ] **Step 1: Move the module**

```bash
cd /workspace/.worktrees/growable-runtime-arrays
mkdir -p crates/kali_types/src/growable
git mv crates/kali_types/src/growable.rs crates/kali_types/src/growable/mod.rs
git mv crates/kali_types/src/growable_tests.rs crates/kali_types/src/growable/growable_tests.rs
CARGO_TARGET_DIR=$T cargo build -j 6 -p kali_types
```

Expected: builds (`mod growable;` in `lib.rs:11` now finds `growable/mod.rs`).

- [ ] **Step 2: Write the failing tests**

Create `crates/kali_types/src/growable/flow_tests.rs`:

```rust
//! Growable-runtime-arrays spec §3.1-§3.2, A-2, A-5, A-7, A-16: the solve.

use super::*;

fn b(func: &str, name: &str) -> GrowNode {
    GrowNode::Binding(func.to_string(), name.to_string())
}

fn r(func: &str) -> GrowNode {
    GrowNode::Return(func.to_string())
}

#[test]
fn a_literal_binding_that_is_pushed_is_growable_and_local_only() {
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("main", "out"));
    facts.demands.insert(b("main", "out"));
    let solution = solve(&facts);
    assert!(solution.is_growable_binding("main", "out"));
    assert!(solution.is_local_only("main", "out"));
    assert!(solution.conflicts.is_empty());
}

#[test]
fn a_literal_that_nobody_mutates_is_not_growable() {
    // A-2: `const a = [1, 2, 3]; return a;` stays on the array-return lane.
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("f", "a"));
    facts.edges.push((r("f"), b("f", "a")));
    facts.edges.push((b("_start", "xs"), r("f")));
    let solution = solve(&facts);
    assert!(!solution.is_growable(&r("f")));
    assert!(!solution.is_growable_binding("_start", "xs"));
}

#[test]
fn a_push_on_the_call_result_makes_the_returned_literal_growable() {
    // A-2: the caller's `xs.push(4)` flows back through the return edge.
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("f", "a"));
    facts.edges.push((r("f"), b("f", "a")));
    facts.edges.push((b("_start", "xs"), r("f")));
    facts.demands.insert(b("_start", "xs"));
    let solution = solve(&facts);
    assert!(solution.is_growable(&r("f")));
    assert!(solution.is_growable_binding("f", "a"));
    assert!(solution.is_growable_binding("_start", "xs"));
    assert!(!solution.is_local_only("f", "a"));
    assert!(solution.same_component(&b("f", "a"), &b("_start", "xs")));
}

#[test]
fn a_push_on_a_parameter_makes_the_argument_literal_growable() {
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("_start", "xs"));
    facts.edges.push((b("add", "a"), b("_start", "xs")));
    facts.demands.insert(b("add", "a"));
    let solution = solve(&facts);
    assert!(solution.is_growable_binding("_start", "xs"));
    assert!(solution.is_growable_binding("add", "a"));
}

#[test]
fn a_module_scope_binding_is_never_local_only() {
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("_start", "out"));
    facts.demands.insert(b("_start", "out"));
    let solution = solve(&facts);
    assert!(solution.is_growable_binding("_start", "out"));
    assert!(!solution.is_local_only("_start", "out"));
}

#[test]
fn a_parameter_fed_a_growable_and_an_allocation_is_a_mixed_layout_at_the_parameter() {
    // Review Focus 1: `total(xs); total(p)` with `p = new Array(2)`.
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("_start", "xs"));
    facts.demands.insert(b("_start", "xs"));
    facts.plain_origins.insert(b("_start", "p"));
    facts.edges.push((b("total", "a"), b("_start", "xs")));
    facts.edges.push((b("total", "a"), b("_start", "p")));
    let solution = solve(&facts);
    assert_eq!(solution.conflicts, vec![GrowConflict::MixedLayout(b("total", "a"))]);
}

#[test]
fn an_allocation_without_any_growable_source_is_no_conflict() {
    let mut facts = GrowFacts::default();
    facts.plain_origins.insert(b("_start", "p"));
    facts.demands.insert(b("_start", "p"));
    facts.edges.push((b("total", "a"), b("_start", "p")));
    let solution = solve(&facts);
    assert!(!solution.is_growable_binding("_start", "p"));
    assert!(solution.conflicts.is_empty());
}

#[test]
fn a_literal_expression_in_a_growable_component_is_a_conflict_naming_its_function() {
    // A-5: `function f(c) { if (c) return []; const o = []; o.push(1); return o; }`.
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("f", "o"));
    facts.demands.insert(b("f", "o"));
    facts.edges.push((r("f"), b("f", "o")));
    facts.edges.push((r("f"), GrowNode::Temp(0)));
    facts.temp_kinds.insert(0, (TempKind::LiteralExpression, "f".to_string()));
    let solution = solve(&facts);
    assert_eq!(solution.conflicts, vec![GrowConflict::LiteralExpression("f".to_string())]);
}

#[test]
fn a_non_array_write_in_a_growable_component_is_a_conflict_at_that_node() {
    // A-7: `function f(a) { a.push(1); } f(xs); f(5);`.
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("_start", "xs"));
    facts.edges.push((b("f", "a"), b("_start", "xs")));
    facts.demands.insert(b("f", "a"));
    facts.non_array_writes.insert(b("f", "a"));
    let solution = solve(&facts);
    assert_eq!(solution.conflicts, vec![GrowConflict::NonArrayWrite(b("f", "a"))]);
}

#[test]
fn a_non_array_write_outside_any_growable_component_is_ignored() {
    let mut facts = GrowFacts::default();
    facts.non_array_writes.insert(b("main", "n"));
    facts.edges.push((b("main", "n"), b("main", "m")));
    assert!(solve(&facts).conflicts.is_empty());
}

#[test]
fn growable_members_lists_every_node_of_every_growable_component() {
    let mut facts = GrowFacts::default();
    facts.literal_origins.insert(b("main", "a"));
    facts.demands.insert(b("main", "a"));
    facts.edges.push((b("main", "b"), b("main", "a")));
    facts.literal_origins.insert(b("main", "c"));
    let solution = solve(&facts);
    let members: Vec<GrowNode> = solution.growable_members().cloned().collect();
    assert_eq!(members, vec![b("main", "a"), b("main", "b")]);
    assert_eq!(solution.members_of(&b("main", "b")), &[b("main", "a"), b("main", "b")][..]);
    assert!(solution.members_of(&b("main", "zzz")).is_empty());
}
```

Wire it at the end of the new `flow.rs` (Step 4) with `#[cfg(test)] #[path = "flow_tests.rs"] mod flow_tests;`.

- [ ] **Step 3: Run them to verify they fail**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types growable::flow -- --test-threads=6`
Expected: compile error, `file not found for module flow` / unresolved `solve`.

- [ ] **Step 4: Write `flow.rs`**

Create `crates/kali_types/src/growable/flow.rs`:

```rust
//! Growable-runtime-arrays spec §3.1-§3.2: the growable property, solved.
//!
//! Every array VALUE a program names is a [`GrowNode`]: a binding or
//! parameter, a function's return value, or a temporary (a `slice` result, a
//! `?:`/`||`/`&&` merge, a literal written directly as an argument or
//! `return`, an allocation written directly). [`GrowFacts::edges`] join two
//! nodes that can hold the same array at run time: an alias, an argument and
//! its parameter, a `return` and the call, a `slice` and its receiver. Edges
//! are undirected, so the solve is a union-find and a COMPONENT is the unit
//! every decision is made on.
//!
//! A component is growable when it holds a literal-initialized binding AND a
//! length/element mutation (`push`, `pop`, an index write) somewhere in it
//! (spec A-2). A growable component must not also hold an allocation
//! (`new Array(n)`, §3.2), a literal written directly as an argument or
//! `return` (A-5), or a non-array value (A-7); each is a [`GrowConflict`].
//! A component that is not growable decides nothing: every node in it keeps
//! the lane it has today (A-1).
//!
//! The walk that fills [`GrowFacts`] is `super::facts`; the checks that turn
//! uses into refusals are `super::positions`.

use std::collections::{BTreeMap, BTreeSet};

/// The synthetic module-scope function, as `repr_infer` keys it.
pub(crate) const TOP_LEVEL: &str = "_start";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum GrowNode {
    /// `(func, name)`: a binding or parameter, keyed like `ReprTable`.
    Binding(String, String),
    /// The value function `func` returns.
    Return(String),
    /// A temporary, numbered by the fact walk; its kind and site are in
    /// [`GrowFacts::temp_kinds`].
    Temp(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TempKind {
    Slice,
    Merge,
    LiteralExpression,
    Allocation,
}

/// The position one occurrence of an array value sits in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UseKind {
    /// Declarator init, alias right-hand side, argument to a declared
    /// function, `return` argument: the edge carries it.
    Flow,
    Push,
    Pop,
    IndexRead,
    IndexWrite,
    LengthRead,
    LengthWrite,
    ForOf,
    Join,
    Slice,
    Search { method: String, from_index: bool },
    /// A whole array handed to `console.log` and friends.
    Console,
    /// Any other method call; the text is the quoted operation, e.g.
    /// "`.reverse()`".
    Method(String),
    /// Any other position (operand, property read, object or array element,
    /// argument to an unknown callee, …).
    Plain,
    /// Named from a nested function, closure or class body.
    Captured,
    /// A module-scope binding named inside a function.
    ModuleRead,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Use {
    pub(crate) node: GrowNode,
    /// The function whose body holds the occurrence.
    pub(crate) site: String,
    pub(crate) kind: UseKind,
}

/// A value stored into an array (a `push` argument, an index write, a
/// literal seed), as far as syntax can tell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ElementValue {
    /// A bare identifier; repr inference checks what it names.
    Identifier(String),
    /// An object, array, function or class literal, a boolean-valued
    /// expression, `null`, `undefined`, a BigInt, a spread or a hole.
    Unsupported,
    Other,
}

/// One argument of a call to a declared function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CallFact {
    pub(crate) site: String,
    pub(crate) callee: String,
    pub(crate) index: usize,
    pub(crate) node: GrowNode,
}

/// One `for-of` over an array value, with what its body does (nested
/// functions excluded: a capture is refused on its own).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoopFacts {
    pub(crate) iterable: GrowNode,
    pub(crate) site: String,
    /// Receivers of every `push`/`pop` in the body.
    pub(crate) mutations: Vec<GrowNode>,
    /// Every declared-function call argument in the body.
    pub(crate) calls: Vec<CallFact>,
}

#[derive(Debug, Default)]
pub(crate) struct GrowFacts {
    pub(crate) edges: Vec<(GrowNode, GrowNode)>,
    /// Bindings declared with an array literal.
    pub(crate) literal_origins: BTreeSet<GrowNode>,
    /// Bindings declared with an allocation, and allocation temporaries.
    pub(crate) plain_origins: BTreeSet<GrowNode>,
    /// Receivers of `push`, `pop` and index writes.
    pub(crate) demands: BTreeSet<GrowNode>,
    pub(crate) non_array_writes: BTreeSet<GrowNode>,
    /// `Temp(n)` → (kind, function whose body holds it).
    pub(crate) temp_kinds: BTreeMap<usize, (TempKind, String)>,
    pub(crate) uses: Vec<Use>,
    pub(crate) calls: Vec<CallFact>,
    pub(crate) loops: Vec<LoopFacts>,
    pub(crate) element_values: Vec<(GrowNode, ElementValue)>,
    /// The function-key stack (outermost first) at every class, JSX, `with`,
    /// enum or module-syntax site the walk cannot see through.
    pub(crate) opaque_sites: Vec<Vec<String>>,
    /// Next `Temp` number.
    pub(crate) temps: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GrowConflict {
    /// A growable component that also holds an allocation; the node is the
    /// meeting point (a parameter or alias where possible).
    MixedLayout(GrowNode),
    NonArrayWrite(GrowNode),
    /// The function whose body holds the literal expression.
    LiteralExpression(String),
}

#[derive(Debug, Default)]
pub(crate) struct GrowSolution {
    component: BTreeMap<GrowNode, usize>,
    members: Vec<Vec<GrowNode>>,
    growable: BTreeSet<usize>,
    pub(crate) conflicts: Vec<GrowConflict>,
}

impl GrowSolution {
    pub(crate) fn component_of(&self, node: &GrowNode) -> Option<usize> {
        self.component.get(node).copied()
    }

    pub(crate) fn is_growable(&self, node: &GrowNode) -> bool {
        self.component_of(node)
            .is_some_and(|component| self.growable.contains(&component))
    }

    pub(crate) fn is_growable_binding(&self, func: &str, name: &str) -> bool {
        self.is_growable(&GrowNode::Binding(func.to_string(), name.to_string()))
    }

    pub(crate) fn same_component(&self, a: &GrowNode, b: &GrowNode) -> bool {
        matches!((self.component_of(a), self.component_of(b)), (Some(x), Some(y)) if x == y)
    }

    /// Every node of every growable component, component by component, each
    /// in `GrowNode` order.
    pub(crate) fn growable_members(&self) -> impl Iterator<Item = &GrowNode> {
        self.growable
            .iter()
            .flat_map(|&component| self.members[component].iter())
    }

    /// The nodes of `node`'s component, or nothing for an unknown node.
    pub(crate) fn members_of(&self, node: &GrowNode) -> &[GrowNode] {
        self.component_of(node)
            .map_or(&[], |component| self.members[component].as_slice())
    }

    /// Spec §3.3, A-16: the binding is the only node of its component and is
    /// not at module scope.
    pub(crate) fn is_local_only(&self, func: &str, name: &str) -> bool {
        func != TOP_LEVEL
            && self.members_of(&GrowNode::Binding(func.to_string(), name.to_string())).len() == 1
    }
}

fn find(parent: &mut [usize], x: usize) -> usize {
    let mut root = x;
    while parent[root] != root {
        root = parent[root];
    }
    let mut cursor = x;
    while parent[cursor] != root {
        let next = parent[cursor];
        parent[cursor] = root;
        cursor = next;
    }
    root
}

pub(crate) fn solve(facts: &GrowFacts) -> GrowSolution {
    // 1. Every node any fact names, in `GrowNode` order.
    let mut all: BTreeSet<GrowNode> = BTreeSet::new();
    for (a, b) in &facts.edges {
        all.insert(a.clone());
        all.insert(b.clone());
    }
    all.extend(facts.literal_origins.iter().cloned());
    all.extend(facts.plain_origins.iter().cloned());
    all.extend(facts.demands.iter().cloned());
    all.extend(facts.non_array_writes.iter().cloned());
    all.extend(facts.temp_kinds.keys().map(|&n| GrowNode::Temp(n)));
    let nodes: Vec<GrowNode> = all.into_iter().collect();
    let position: BTreeMap<&GrowNode, usize> =
        nodes.iter().enumerate().map(|(i, node)| (node, i)).collect();

    // 2. Union along every edge; the smaller index is always the root, so a
    //    component's root is its first member.
    let mut parent: Vec<usize> = (0..nodes.len()).collect();
    for (a, b) in &facts.edges {
        let (ra, rb) = (find(&mut parent, position[a]), find(&mut parent, position[b]));
        if ra != rb {
            let (low, high) = if ra < rb { (ra, rb) } else { (rb, ra) };
            parent[high] = low;
        }
    }

    // 3. Dense component ids in order of first member.
    let mut dense: BTreeMap<usize, usize> = BTreeMap::new();
    let mut members: Vec<Vec<GrowNode>> = Vec::new();
    let mut component: BTreeMap<GrowNode, usize> = BTreeMap::new();
    for (i, node) in nodes.iter().enumerate() {
        let root = find(&mut parent, i);
        let next = dense.len();
        let id = *dense.entry(root).or_insert(next);
        if id == members.len() {
            members.push(Vec::new());
        }
        members[id].push(node.clone());
        component.insert(node.clone(), id);
    }

    // 4. Growable: a literal origin and a demand in the same component.
    let growable: BTreeSet<usize> = members
        .iter()
        .enumerate()
        .filter(|(_, group)| {
            group.iter().any(|n| facts.literal_origins.contains(n))
                && group.iter().any(|n| facts.demands.contains(n))
        })
        .map(|(id, _)| id)
        .collect();

    // 5. Conflicts, growable components only.
    let mut conflicts = Vec::new();
    for &id in &growable {
        let group = &members[id];
        if group.iter().any(|n| facts.plain_origins.contains(n)) {
            conflicts.push(GrowConflict::MixedLayout(meeting_point(group, facts)));
        }
        for node in group {
            if let GrowNode::Temp(n) = node {
                if let Some((TempKind::LiteralExpression, site)) = facts.temp_kinds.get(n) {
                    conflicts.push(GrowConflict::LiteralExpression(site.clone()));
                }
            }
        }
        for node in group {
            if facts.non_array_writes.contains(node) {
                conflicts.push(GrowConflict::NonArrayWrite(node.clone()));
            }
        }
    }

    GrowSolution {
        component,
        members,
        growable,
        conflicts,
    }
}

/// Where two layouts meet: the first binding that is neither origin (a
/// parameter or alias), else the first binding, else the first return, else
/// the first node.
fn meeting_point(group: &[GrowNode], facts: &GrowFacts) -> GrowNode {
    let is_binding = |n: &&GrowNode| matches!(n, GrowNode::Binding(..));
    group
        .iter()
        .filter(is_binding)
        .find(|n| !facts.literal_origins.contains(*n) && !facts.plain_origins.contains(*n))
        .or_else(|| group.iter().find(is_binding))
        .or_else(|| group.iter().find(|n| matches!(n, GrowNode::Return(_))))
        .unwrap_or(&group[0])
        .clone()
}

#[cfg(test)]
#[path = "flow_tests.rs"]
mod flow_tests;
```

Add `pub(crate) mod flow;` to `growable/mod.rs` after its `use` block.

- [ ] **Step 5: Run the tests**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types growable::flow -- --test-threads=6`
Expected: PASS, 11 tests. Dead-code warnings for the types Tasks 4-7 consume are expected until Task 6 wires them in (`kali_types` does not deny warnings).

- [ ] **Step 6: Commit**

```bash
git add -A crates/kali_types/src/growable crates/kali_types/src/growable.rs crates/kali_types/src/growable_tests.rs
git commit -m "feat(growable-runtime-arrays): growable component solve (spec §3.1-§3.2, A-2, A-5, A-7, A-16)"
```

---
### Task 4: The fact walk (`growable/facts.rs`)

**Files:**
- Create: `crates/kali_types/src/growable/facts.rs`, `crates/kali_types/src/growable/facts_tests.rs`
- Modify: `crates/kali_types/src/growable/mod.rs` (add `pub(crate) mod facts;`)
- Modify: `crates/kali_types/src/repr_infer.rs` (add `collect_growable_facts` beside `collect_growable_candidates` at `:2832`, and the test hook `growable_facts_for` beside `nested_fn_lockstep_sets` at `:1535`)

**Interfaces:**
- Consumes: every type in `crate::growable::flow` (Task 3).
- Produces:
  - `pub(crate) struct WalkContext<'a> { pub(crate) is_declared: &'a dyn Fn(&str, &str) -> bool, pub(crate) resolve_callee: &'a dyn Fn(&str, &str) -> Option<String>, pub(crate) params: &'a BTreeMap<String, Vec<String>> }`
  - `pub(crate) fn collect_facts(statements: &[Statement], ctx: &WalkContext<'_>) -> GrowFacts`
  - `ReprInfer::collect_growable_facts(&self, statements: &[Statement]) -> GrowFacts` (needs Phase A and A2 done)
  - `#[cfg(test)] pub(crate) fn growable_facts_for(statements: &[Statement]) -> GrowFacts` in `repr_infer.rs`

The walk decides nothing. It is an exhaustive match over `Statement` and `Expression` with no wildcard arm, exactly like the scanner in `growable/mod.rs` (`Scan::stmt`/`Scan::expr`, `:540-920`), so a new AST variant must be classified before it compiles.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_types/src/growable/facts_tests.rs`:

```rust
//! Growable-runtime-arrays spec §3.1: what the walk records.

use super::super::flow::{ElementValue, GrowFacts, GrowNode, TempKind, UseKind};

fn facts(src: &str) -> GrowFacts {
    crate::repr_infer::growable_facts_for(&crate::test_support::parse_statements(src))
}

fn b(func: &str, name: &str) -> GrowNode {
    GrowNode::Binding(func.to_string(), name.to_string())
}

fn has_edge(facts: &GrowFacts, x: &GrowNode, y: &GrowNode) -> bool {
    facts
        .edges
        .iter()
        .any(|(a, c)| (a == x && c == y) || (a == y && c == x))
}

fn kinds_of(facts: &GrowFacts, node: &GrowNode) -> Vec<UseKind> {
    facts
        .uses
        .iter()
        .filter(|u| &u.node == node)
        .map(|u| u.kind.clone())
        .collect()
}

#[test]
fn a_literal_declarator_is_an_origin_and_its_push_a_demand_with_element_values() {
    let f = facts("function main() { const out = [1]; out.push(2, x); out.push({a: 1}); out.push(true); }");
    let out = b("main", "out");
    assert!(f.literal_origins.contains(&out));
    assert!(f.demands.contains(&out));
    assert_eq!(kinds_of(&f, &out), vec![UseKind::Push, UseKind::Push, UseKind::Push]);
    let values: Vec<ElementValue> = f.element_values.iter().filter(|(n, _)| n == &out).map(|(_, v)| v.clone()).collect();
    assert_eq!(
        values,
        vec![
            ElementValue::Other,
            ElementValue::Other,
            ElementValue::Identifier("x".to_string()),
            ElementValue::Unsupported,
            ElementValue::Unsupported,
        ]
    );
}

#[test]
fn a_return_a_call_result_an_argument_and_an_alias_are_edges() {
    let f = facts(
        "function make() { const xs = []; xs.push(1); return xs; }\n\
         function add(a) { a.push(2); }\n\
         const ys = make();\n\
         add(ys);\n\
         const zs = ys;\n",
    );
    assert!(has_edge(&f, &GrowNode::Return("make".to_string()), &b("make", "xs")));
    assert!(has_edge(&f, &b("_start", "ys"), &GrowNode::Return("make".to_string())));
    assert!(has_edge(&f, &b("add", "a"), &b("_start", "ys")));
    assert!(has_edge(&f, &b("_start", "zs"), &b("_start", "ys")));
    assert_eq!(f.calls.len(), 1);
    assert_eq!(f.calls[0].callee, "add");
    assert_eq!(f.calls[0].index, 0);
    assert_eq!(f.calls[0].node, b("_start", "ys"));
}

#[test]
fn a_slice_result_is_a_temporary_joined_to_its_receiver() {
    let f = facts("const a = []; a.push(1); const t = a.slice(1);");
    let slice = f
        .temp_kinds
        .iter()
        .find(|(_, (kind, _))| *kind == TempKind::Slice)
        .map(|(n, _)| GrowNode::Temp(*n))
        .expect("a slice temporary");
    assert!(has_edge(&f, &slice, &b("_start", "a")));
    assert!(has_edge(&f, &b("_start", "t"), &slice));
    assert!(kinds_of(&f, &b("_start", "a")).contains(&UseKind::Slice));
}

#[test]
fn an_allocation_is_a_plain_origin_and_a_literal_argument_a_literal_temporary() {
    let f = facts("function g(a) { return a.length; } const p = new Array(3); g([1, 2]);");
    assert!(f.plain_origins.contains(&b("_start", "p")));
    let literal = f
        .temp_kinds
        .iter()
        .find(|(_, (kind, _))| *kind == TempKind::LiteralExpression)
        .map(|(n, _)| GrowNode::Temp(*n))
        .expect("a literal temporary");
    assert!(has_edge(&f, &b("g", "a"), &literal));
}

#[test]
fn a_function_naming_a_module_binding_records_a_module_read_at_its_own_site() {
    let f = facts("const out = []; out.push(1); function size() { return out.length; }");
    let reads: Vec<_> = f.uses.iter().filter(|u| u.kind == UseKind::ModuleRead).collect();
    assert_eq!(reads.len(), 1);
    assert_eq!(reads[0].node, b("_start", "out"));
    assert_eq!(reads[0].site, "size");
}

#[test]
fn a_closure_naming_an_outer_binding_records_a_capture() {
    let f = facts("function m() { const o = []; o.push(1); const g = () => o.length; }");
    assert!(kinds_of(&f, &b("m", "o")).contains(&UseKind::Captured));
}

#[test]
fn a_for_of_records_its_iterable_body_mutations_and_body_calls() {
    let f = facts(
        "function add(arr, v) { arr.push(v); }\n\
         const a = []; a.push(1); const b = [];\n\
         for (const x of a) { b.push(x); add(a, x); }\n",
    );
    assert_eq!(f.loops.len(), 1);
    assert_eq!(f.loops[0].iterable, b("_start", "a"));
    assert_eq!(f.loops[0].mutations, vec![b("_start", "b")]);
    // `add(a, x)` contributes one fact per argument; the array is index 0.
    assert!(f.loops[0]
        .calls
        .iter()
        .any(|c| c.callee == "add" && c.index == 0 && c.node == b("_start", "a")));
    // The push inside `add` belongs to `add`'s body, not to the loop.
    assert_eq!(f.loops[0].mutations.len(), 1);
}

#[test]
fn console_arguments_plain_operands_and_other_methods_are_recorded_as_such() {
    let f = facts("const a = []; a.push(1); console.log(a, a.length); const s = typeof a; a.reverse(); a[\"push\"](2); a.indexOf(1, 1);");
    let kinds = kinds_of(&f, &b("_start", "a"));
    assert!(kinds.contains(&UseKind::Console));
    assert!(kinds.contains(&UseKind::LengthRead));
    assert!(kinds.contains(&UseKind::Plain));
    assert!(kinds.contains(&UseKind::Method("`.reverse()`".to_string())));
    assert!(kinds.contains(&UseKind::Method("`[\"push\"]()`".to_string())));
    assert!(kinds.contains(&UseKind::Search { method: "indexOf".to_string(), from_index: true }));
}

#[test]
fn a_missing_argument_a_bare_return_and_a_scalar_write_are_non_array_writes() {
    let f = facts("function f(a, b) { if (a) return; return a; } const xs = []; f(xs); let y = xs; y = 5;");
    assert!(f.non_array_writes.contains(&b("f", "b")));
    assert!(f.non_array_writes.contains(&GrowNode::Return("f".to_string())));
    assert!(f.non_array_writes.contains(&b("_start", "y")));
}

#[test]
fn an_index_write_is_a_demand_and_an_element_value_but_not_a_loop_mutation() {
    let f = facts("const a = [0]; for (const x of a) { a[0] = x; } a.length = 0;");
    let a = b("_start", "a");
    assert!(f.demands.contains(&a));
    assert!(f.loops[0].mutations.is_empty());
    let kinds = kinds_of(&f, &a);
    assert!(kinds.contains(&UseKind::IndexWrite));
    assert!(kinds.contains(&UseKind::LengthWrite));
}

#[test]
fn a_class_inside_a_function_is_an_opaque_site_carrying_the_frame_stack() {
    let f = facts("function m() { const o = []; o.push(1); class C {} }");
    assert_eq!(f.opaque_sites, vec![vec!["_start".to_string(), "m".to_string()]]);
}
```

Wire it at the end of `facts.rs`: `#[cfg(test)] #[path = "facts_tests.rs"] mod facts_tests;`.

- [ ] **Step 2: Run them to verify they fail**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types growable::facts -- --test-threads=6`
Expected: compile error (`growable_facts_for` / `facts` module missing).

- [ ] **Step 3: Add the repr-inference entry points**

In `crates/kali_types/src/repr_infer.rs`, beside `collect_growable_candidates` (`:2832`):

```rust
    /// Growable-runtime-arrays spec §3.1: the growable facts, from Phase A's
    /// function table and Phase A2's local names (both must be complete).
    /// A call reaches a declared function exactly as the array-return lane
    /// resolves it (`array_return_callee`: `const` arrow aliases included,
    /// shadowed names excluded).
    fn collect_growable_facts(&self, statements: &[Statement]) -> crate::growable::flow::GrowFacts {
        let is_declared = |func: &str, name: &str| self.is_locally_declared(func, name);
        let resolve_callee = |site: &str, name: &str| {
            self.array_return_callee(site, name)
                .filter(|key| self.functions.contains_key(key))
        };
        let ctx = crate::growable::facts::WalkContext {
            is_declared: &is_declared,
            resolve_callee: &resolve_callee,
            params: &self.functions,
        };
        crate::growable::facts::collect_facts(statements, &ctx)
    }
```

`self.functions` is a `BTreeMap<String, Vec<String>>` (`:818`); if it is another map type, convert with `.iter().map(|(k, v)| (k.clone(), v.clone())).collect()` into a local `BTreeMap` first, as `resolve_array_returns` does at `:5736`.

Beside `nested_fn_lockstep_sets` (`:1535`):

```rust
/// Growable-runtime-arrays test hook: Phase A, A2, then the fact walk.
#[cfg(test)]
pub(crate) fn growable_facts_for(statements: &[Statement]) -> crate::growable::flow::GrowFacts {
    let mut infer = ReprInfer::default();
    infer.collect_functions(statements);
    infer.collect_local_names(TOP_LEVEL, statements);
    infer.collect_growable_facts(statements)
}
```

- [ ] **Step 4: Write `facts.rs`**

Create `crates/kali_types/src/growable/facts.rs` with this skeleton, then fill every method as specified below it:

```rust
//! Growable-runtime-arrays spec §3.1: the syntactic facts the growable
//! property is solved over. One exhaustive walk records, per function, every
//! array value flow (edges), every origin (a literal binding, an allocation,
//! a literal or allocation written directly), every length or element
//! mutation (demands), every non-array write, every occurrence of an array
//! value in a position (uses), every `for-of` frame and every stored element
//! value. It decides nothing: `flow::solve` and `positions` do.

use std::collections::BTreeMap;

use kali_ast::{
    AssignmentExpression, AssignmentOperator, BlockStatement, CallExpression, Expression,
    ExpressionOrSpread, ForInLefthand, ForInit, ForOfLefthand, ForOfStatement, LiteralValue,
    MemberExpression, OptionalChainInner, Statement, VariableDeclaration,
};

use super::flow::{
    CallFact, ElementValue, GrowFacts, GrowNode, LoopFacts, TempKind, Use, UseKind, TOP_LEVEL,
};

/// What the walk needs from repr inference's Phase A and A2.
pub(crate) struct WalkContext<'a> {
    /// `is_declared(func, name)`: `func` declares `name` itself (a parameter
    /// or a local), the `ReprInfer::is_locally_declared` fact.
    pub(crate) is_declared: &'a dyn Fn(&str, &str) -> bool,
    /// `resolve_callee(site, name)`: the declared function a bare call
    /// `name(…)` made in `site` reaches, if any.
    pub(crate) resolve_callee: &'a dyn Fn(&str, &str) -> Option<String>,
    /// Parameter names of every declared function, by key.
    pub(crate) params: &'a BTreeMap<String, Vec<String>>,
}

pub(crate) fn collect_facts(statements: &[Statement], ctx: &WalkContext<'_>) -> GrowFacts {
    let mut walker = Walker {
        ctx,
        facts: GrowFacts::default(),
        frames: vec![TOP_LEVEL.to_string()],
        loops: Vec::new(),
        anonymous: 0,
    };
    walker.statements(statements);
    walker.facts
}

const CONSOLE_METHODS: &[&str] = &["log", "error", "warn", "info", "debug"];

enum Resolved {
    Local(GrowNode),
    Captured(GrowNode),
    Module(GrowNode),
    Unknown,
}

struct Walker<'w, 'c> {
    ctx: &'w WalkContext<'c>,
    facts: GrowFacts,
    /// Function keys, outermost (`_start`) first; never empty.
    frames: Vec<String>,
    /// Indices into `facts.loops` of the active `for-of` frames of the
    /// CURRENT function (saved and cleared on entering a nested function).
    loops: Vec<usize>,
    anonymous: usize,
}

fn unwrap(expr: &Expression) -> &Expression {
    match expr {
        Expression::ParenthesizedExpression(inner) => unwrap(&inner.expression),
        Expression::TypeAssertion(inner) => unwrap(&inner.expression),
        Expression::SatisfiesExpression(inner) => unwrap(&inner.expression),
        other => other,
    }
}

fn is_allocation(expr: &Expression) -> bool {
    crate::resolve::expression::expression_is_array_allocation(expr)
}

/// Mirrors `repr_infer::is_boolean_valued_init` (captured-bindings A-2.1):
/// kali stores such a value as `1`/`0`, so it cannot be an element.
fn is_boolean_valued(expr: &Expression) -> bool {
    match unwrap(expr) {
        Expression::Literal(LiteralValue::Boolean(_)) => true,
        Expression::BinaryExpression(binary) => matches!(
            binary.operator.as_str(),
            "==" | "===" | "!=" | "!==" | "<" | ">" | "<=" | ">="
        ),
        Expression::UnaryExpression(unary) => unary.operator == "!",
        _ => false,
    }
}

fn element_value(expr: &Expression) -> ElementValue {
    match unwrap(expr) {
        Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::BigIntLiteral(_)
        | Expression::SpreadElement(_)
        | Expression::Literal(LiteralValue::Null)
        | Expression::Literal(LiteralValue::Regex { .. }) => ElementValue::Unsupported,
        Expression::Identifier(name) if name == "undefined" => ElementValue::Unsupported,
        Expression::Identifier(name) => ElementValue::Identifier(name.clone()),
        other if is_boolean_valued(other) => ElementValue::Unsupported,
        _ => ElementValue::Other,
    }
}

impl Walker<'_, '_> {
    fn site(&self) -> &str {
        self.frames.last().expect("the module frame is never popped")
    }

    fn temp(&mut self, kind: TempKind) -> GrowNode {
        let n = self.facts.temps;
        self.facts.temps += 1;
        let site = self.site().to_string();
        self.facts.temp_kinds.insert(n, (kind, site));
        GrowNode::Temp(n)
    }

    fn edge(&mut self, a: GrowNode, b: GrowNode) {
        self.facts.edges.push((a, b));
    }

    fn record(&mut self, node: &GrowNode, kind: UseKind) {
        let site = self.site().to_string();
        self.facts.uses.push(Use { node: node.clone(), site, kind });
    }

    fn anonymous_key(&mut self) -> String {
        self.anonymous += 1;
        format!("<anonymous {}>", self.anonymous)
    }

    fn resolve(&self, name: &str) -> Resolved {
        let site = self.site();
        if (self.ctx.is_declared)(site, name) {
            return Resolved::Local(GrowNode::Binding(site.to_string(), name.to_string()));
        }
        for func in self.frames.iter().rev().skip(1) {
            if (self.ctx.is_declared)(func, name) {
                let node = GrowNode::Binding(func.clone(), name.to_string());
                return if func == TOP_LEVEL {
                    Resolved::Module(node)
                } else {
                    Resolved::Captured(node)
                };
            }
        }
        Resolved::Unknown
    }

    /// The node `name` denotes in this function. A captured or module-scope
    /// name records that fact and yields `None`: its flows are not modelled,
    /// because `positions` refuses it whenever it is growable.
    fn name_node(&mut self, name: &str) -> Option<GrowNode> {
        match self.resolve(name) {
            Resolved::Local(node) => Some(node),
            Resolved::Captured(node) => {
                self.record(&node, UseKind::Captured);
                None
            }
            Resolved::Module(node) => {
                self.record(&node, UseKind::ModuleRead);
                None
            }
            Resolved::Unknown => None,
        }
    }

    fn use_name(&mut self, name: &str, kind: UseKind) -> Option<GrowNode> {
        let node = self.name_node(name)?;
        self.record(&node, kind);
        Some(node)
    }

    fn opaque(&mut self) {
        let stack = self.frames.clone();
        self.facts.opaque_sites.push(stack);
    }

    fn mutation(&mut self, receiver: &GrowNode) {
        self.facts.demands.insert(receiver.clone());
        for &frame in &self.loops {
            self.facts.loops[frame].mutations.push(receiver.clone());
        }
    }

    // statements, block, statement, variable_declaration, declarator,
    // loop_variables, for_of, function, arrow, return_value, discarded,
    // expr, elements, member_read, receiver, value, merge, call,
    // declared_call, method_call, assignment — specified below.
}

#[cfg(test)]
#[path = "facts_tests.rs"]
mod facts_tests;
```

Then implement the remaining methods exactly as follows (`self.expr` is the plain-context walk, `self.value` the flow-context walk, `self.receiver` the method/member-base walk):

1. `fn statements(&mut self, statements: &[Statement])` and `fn block(&mut self, block: &BlockStatement)`: visit each statement in order.
2. `fn statement(&mut self, stmt: &Statement)`, exhaustive, no wildcard:
   - `ExpressionStatement(s)` → `self.discarded(&s.expression)`.
   - `BreakStatement | ContinueStatement | DebuggerStatement | TypeAliasDeclaration | InterfaceDeclaration` → nothing.
   - `WithStatement | ClassDeclaration | ImportDeclaration | ExportAll | ExportNamed | ExportDefault | EnumDeclaration` → `self.opaque()` and do not descend.
   - `ReturnStatement(s)` → `self.return_value(s.argument.as_ref())`.
   - `LabeledStatement(s)` → `self.statement(&s.body)`.
   - `IfStatement(s)` → `self.expr(&s.test)`, `self.block(&s.consequent)`, then the alternate block if any.
   - `SwitchStatement(s)` → `self.expr(&s.discriminant)`; for each case: `self.expr(test)` if any, `self.statements(&case.consequent)`.
   - `ThrowStatement(s)` → `self.expr(&s.argument)`.
   - `TryStatement(s)` → the block, the handler's `body`, the finalizer.
   - `BlockStatement(s)` → `self.block(s)`.
   - `ForStatement(s)` → init (`ForInit::VariableDeclaration(d)` → `self.variable_declaration(d)`; `ForInit::Expression(e)` → `self.discarded(e)`), `self.expr(test)`, `self.discarded(update)`, `self.block(&s.body)`.
   - `ForInStatement(s)` → left (`VariableDeclaration(d)` → `self.loop_variables(d)`; `Expression(e)` → `self.expr(e)`), `self.expr(&s.right)` (a growable array iterated with `for-in` is a plain use), `self.statement(&s.body)`.
   - `ForOfStatement(s)` → `self.for_of(s)`.
   - `WhileStatement(s)` → `self.expr(&s.test)`, `self.block(&s.body)`; `DoWhileStatement(s)` → body then test.
   - `FunctionDeclaration(decl)` → `self.function(decl.name.clone(), &decl.body.body)`.
   - `VariableDeclaration(d)` → `self.variable_declaration(d)`.
3. `fn variable_declaration(&mut self, d: &VariableDeclaration)`: for each declarator, `self.declarator(&declarator.id, declarator.init.as_ref())`. The kind is ignored (A-19).
4. `fn declarator(&mut self, id: &str, init: Option<&Expression>)`: let `node = GrowNode::Binding(self.site(), id)`.
   1. No init → insert `node` into `non_array_writes`; return.
   2. `unwrap(init)` is `ArrayExpression(arr)` → insert `node` into `literal_origins`; for each element: `Some(Expression(e))` → push `(node, element_value(e))` onto `element_values`, then `self.expr(e)`; `Some(Spread(s))` → push `(node, ElementValue::Unsupported)`, then `self.expr(&s.argument)`; `Some(Empty)` or `None` (a hole) → push `(node, ElementValue::Unsupported)`. Return.
   3. `is_allocation(unwrap(init))` → insert `node` into `plain_origins`; walk the allocation's arguments: `for arg in crate::array_return::allocation_length_args(init) { self.expr(arg) }` and `if let Some(v) = crate::array_return::fill_value(init) { self.expr(v) }`. Return.
   4. Otherwise `match self.value(init) { Some(n) => self.edge(node, n), None => { non_array_writes.insert(node) } }`.
5. `fn loop_variables(&mut self, d: &VariableDeclaration)`: insert `GrowNode::Binding(site, declarator.id)` into `non_array_writes` for each declarator (a loop variable holds elements or keys, never an array).
6. `fn for_of(&mut self, s: &ForOfStatement)`: left (`VariableDeclaration(d)` → `self.loop_variables(d)`; `Expression(e)` → `self.expr(e)`); `let iterable = self.receiver(&s.right);` if `Some(node)`: `self.record(&node, UseKind::ForOf)`, push `LoopFacts { iterable: node, site, mutations: vec![], calls: vec![] }` onto `facts.loops` and its index onto `self.loops`; walk `self.statement(&s.body)`; pop `self.loops` if pushed.
7. `fn function(&mut self, key: String, body: &[Statement])`: `let saved = std::mem::take(&mut self.loops);` push `key` onto `frames`; `self.statements(body)`; if `crate::array_return::body_falls_off_end(body)` insert `GrowNode::Return(key)` into `non_array_writes`; pop the frame; `self.loops = saved;`.
8. `fn arrow(&mut self, key: String, body: &Expression)`: same frame handling, but the body is one `return`: `self.return_value(Some(body))`, and nothing falls off the end.
9. `fn return_value(&mut self, arg: Option<&Expression>)`: `let site = GrowNode::Return(self.site())`; `Some(a)` → `match self.value(a) { Some(n) => self.edge(site, n), None => non_array_writes.insert(site) }`; `None` → `non_array_writes.insert(site)`.
10. `fn discarded(&mut self, expr: &Expression)` (a value nobody reads): `match unwrap(expr)`: `Identifier(_)` → nothing; `CallExpression(c)` → `let _ = self.call(c);`; `AssignmentExpression(a)` → `let _ = self.assignment(a);`; anything else → `self.expr(expr)`.
11. `fn expr(&mut self, expr: &Expression)`, exhaustive, no wildcard:
    - `Identifier(name)` → `self.use_name(name, UseKind::Plain);`.
    - `Literal | BigIntLiteral | MetaProperty | JsxEmptyExpression | ThisExpression | SuperExpression | PrivateIdentifier` → nothing.
    - `BinaryExpression` → both sides; `UnaryExpression` → the argument; `LogicalExpression` → both; `ConditionalExpression` → test, consequent, alternate; `SequenceExpression` → each; `ParenthesizedExpression`, `TypeAssertion`, `SatisfiesExpression`, `ChainExpression`, `DecoratedExpression` → the inner expression; `AwaitExpression`, `SpreadElement`, `RestElement` → the argument; `YieldExpression` → the argument if any; `ImportExpression` → the source; `UpdateExpression` → the argument (`x++` is a plain use; `a[i]++` an index read, which the resolver refuses on its own).
    - `CallExpression(call)` → `if let Some(node) = self.call(call) { self.record(&node, UseKind::Plain) }` (a call result used as an operand).
    - `MemberExpression(m)` → `self.member_read(m)`.
    - `ArrayExpression(a)` → `self.elements(a)` (an array literal in a plain position; each element is itself a plain use).
    - `ObjectExpression(o)` → `self.expr(&property.value)` for each property.
    - `FunctionExpression(f)` → `let key = f.id.clone().unwrap_or_else(|| self.anonymous_key());` then `if let Some(body) = &f.body { self.function(key, &body.body) }` (a block-bodied arrow is a `FunctionExpression` with `is_arrow`).
    - `ArrowFunctionExpression(a)` → key as above from `a.id`, then `self.arrow(key, &a.body)`.
    - `ClassExpression | JsxElement | JsxFragment` → `self.opaque()`.
    - `NewExpression(e)` → callee and every argument.
    - `TemplateLiteral(t)` → each expression; `TaggedTemplateExpression(e)` → the tag and each template expression.
    - `AssignmentExpression(a)` → `if let Some(node) = self.assignment(a) { self.record(&node, UseKind::Plain) }`.
    - `OptionalChainExpression(e)` → `OptionalChainInner::NonNull { object, .. }` → `if let Some(node) = self.receiver(object) { self.record(&node, UseKind::Method("an optional chain `?.`".to_string())) }`.
12. `fn elements(&mut self, array: &kali_ast::ArrayExpression)`: `Some(Expression(e))` → `self.expr(e)`; `Some(Spread(s))` → `self.expr(&s.argument)`; holes → nothing.
13. `fn member_read(&mut self, member: &MemberExpression)`:
    - computed (`member.computed_index` is `Some(index)`): the use is `IndexRead` unless `member.property` is `Some(text)` with `text.parse::<i64>().is_err()` (a string key like `a["length"]`), which is `Plain`. `if let Some(node) = self.receiver(&member.object) { self.record(&node, kind) }`, then `self.expr(index)`.
    - dot: `LengthRead` when the property is `length`, else `Plain`; recorded on `self.receiver(&member.object)` the same way.
14. `fn receiver(&mut self, expr: &Expression) -> Option<GrowNode>`: `match unwrap(expr)`: `Identifier(name)` → `self.name_node(name)` (the caller records the use); `CallExpression(call)` → `self.call(call)`; anything else → `self.expr(other); None`.
15. `fn value(&mut self, expr: &Expression) -> Option<GrowNode>`: on `let expr = unwrap(expr);`:
    1. `is_allocation(expr)` → walk its arguments (as in 4.3); `let t = self.temp(TempKind::Allocation);` insert `t` into `plain_origins`; `Some(t)`.
    2. `Identifier(name)` → `self.use_name(name, UseKind::Flow)`.
    3. `ArrayExpression(a)` → `self.elements(a); Some(self.temp(TempKind::LiteralExpression))`.
    4. `CallExpression(c)` → `self.call(c)`.
    5. `ConditionalExpression(c)` → `self.expr(&c.test); Some(self.merge(&[&c.consequent, &c.alternate]))`.
    6. `LogicalExpression(l)` → `Some(self.merge(&[&l.left, &l.right]))`.
    7. `SequenceExpression(s)` → `self.discarded(e)` for all but the last, then `self.value(last)` (`None` for an empty sequence).
    8. `AssignmentExpression(a)` → `self.assignment(a)`.
    9. Anything else → `self.expr(expr); None`.
16. `fn merge(&mut self, arms: &[&Expression]) -> GrowNode`: `let m = self.temp(TempKind::Merge);` for each arm: `Some(n) = self.value(arm)` → `self.edge(m.clone(), n)`; `None` → insert `m` into `non_array_writes`. Return `m`.
17. `fn call(&mut self, call: &CallExpression) -> Option<GrowNode>`: `match unwrap(&call.callee)`:
    - `MemberExpression(member)` → `self.method_call(member, &call.args)`.
    - `Identifier(name)` → if `let Some(key) = (self.ctx.resolve_callee)(self.site(), name)` → `self.declared_call(&key, &call.args); Some(GrowNode::Return(key))`; else `self.use_name(name, UseKind::Plain);` every argument through `self.expr` (an argument to an unknown callee is a plain use), `None`.
    - anything else (an IIFE, a call result called) → `self.expr(callee)`, every argument through `self.expr`, `None`.
18. `fn declared_call(&mut self, key: &str, args: &[Expression])`:
    1. `let params = self.ctx.params.get(key).cloned().unwrap_or_default();`
    2. `let spread = args.iter().position(|a| matches!(a, Expression::SpreadElement(_))).unwrap_or(args.len());`
    3. For each `(index, arg)`: if `index >= spread` → `self.expr(arg)` and continue. Else `let value = self.value(arg);` If `params.get(index)` is `None` (an extra argument) continue. Else `let target = GrowNode::Binding(key, param)`: on `Some(node)` → `self.edge(target, node.clone())`, build `CallFact { site: self.site(), callee: key, index, node }`, push a clone onto `self.facts.loops[frame].calls` for every `frame` in `self.loops`, and push it onto `self.facts.calls`; on `None` → insert `target` into `non_array_writes`.
    4. Every parameter at index `>= spread.min(args.len())` that step 3 did not reach (a missing argument, or one a spread hides) → insert `GrowNode::Binding(key, param)` into `non_array_writes`.
19. `fn method_call(&mut self, member: &MemberExpression, args: &[Expression]) -> Option<GrowNode>`:
    1. `console.<log|error|warn|info|debug>(…)` (dot access, object `unwrap`s to `Identifier("console")`, method in `CONSOLE_METHODS`): for each argument, `if let Some(node) = self.receiver(arg) { self.record(&node, UseKind::Console) }`; return `None`.
    2. `let Some(receiver) = self.receiver(&member.object) else { walk the computed index if any, walk every argument through self.expr, return None };`
    3. Computed callee (`member.computed_index.is_some()`): record `UseKind::Method(format!("`[\"{m}\"]()`"))` when `member.property` is `Some(m)`, else `UseKind::Method("a computed method call".to_string())`; walk the index and arguments; `None`.
    4. Dot callee, by `member.property.as_deref().unwrap_or_default()`:
       - `"push"` → record `Push`; `self.mutation(&receiver)`; for each argument push `(receiver.clone(), element_value(arg))` onto `element_values`, then `self.expr(arg)`; `None`.
       - `"pop"` → record `Pop`; `self.mutation(&receiver)`; walk arguments; `None`.
       - `"join"` → record `Join`; walk arguments; `None`.
       - `"slice"` → record `Slice`; walk arguments; `let t = self.temp(TempKind::Slice); self.edge(t.clone(), receiver); Some(t)`.
       - `"indexOf" | "includes"` → record `Search { method, from_index: args.len() > 1 }`; walk arguments; `None`.
       - any other `m` → record `Method(format!("`.{m}()`"))`; walk arguments; `None`.
20. `fn assignment(&mut self, a: &AssignmentExpression) -> Option<GrowNode>` (`plain = matches!(a.operator, AssignmentOperator::Assign)`), on `unwrap(&a.left)`:
    - `Identifier(name)`: not `plain` → `self.use_name(name, UseKind::Plain); self.expr(&a.right); None`. `plain` → `let value = self.value(&a.right); let target = self.name_node(name)?;` then `Some(n)` → `self.edge(target.clone(), n)`, `None` → insert `target` into `non_array_writes`; return `Some(target)`.
    - `MemberExpression(m)` computed with index `index`: `let string_key = m.property.as_deref().is_some_and(|t| t.parse::<i64>().is_err());` `if let Some(receiver) = self.receiver(&m.object)`: when `plain && !string_key` → record `IndexWrite`, insert `receiver` into `demands` (an index write keeps the length, so it is not a loop mutation), push `(receiver, element_value(&a.right))` onto `element_values`; otherwise record `Plain`. Then `self.expr(index)`, `self.expr(&a.right)`, `None`.
    - `MemberExpression(m)` dot `length` → record `LengthWrite` on `self.receiver(&m.object)`; `self.expr(&a.right)`; `None`.
    - `MemberExpression(m)` any other dot property → record `Plain` on the receiver; `self.expr(&a.right)`; `None`.
    - anything else (destructuring) → `self.expr(left)`, `self.expr(&a.right)`, `None`.

- [ ] **Step 5: Run the tests**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types growable:: -- --test-threads=6`
Expected: PASS (11 facts tests, 11 flow tests, and the existing `growable_tests`).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_types/src/growable crates/kali_types/src/repr_infer.rs
git commit -m "feat(growable-runtime-arrays): exhaustive growable fact walk (spec §3.1, A-1..A-8)"
```

---

### Task 5: Position, capture, module-read and snapshot checks (`growable/positions.rs`)

**Files:**
- Create: `crates/kali_types/src/growable/positions.rs`, `crates/kali_types/src/growable/positions_tests.rs`
- Modify: `crates/kali_types/src/growable/mod.rs` (add `pub(crate) mod positions;`)

**Interfaces:**
- Consumes: `GrowFacts`, `GrowSolution`, `solve` (Task 3); `growable_facts_for` (Task 4); the Task 1 messages.
- Produces:
  - `pub(crate) fn growable_refusals(facts: &GrowFacts, solution: &GrowSolution, params: &BTreeMap<String, Vec<String>>) -> Vec<String>` — every refusal text, deduplicated, in a deterministic order (conflicts, then uses in walk order, then opaque sites, then loops).
  - `pub(crate) fn length_mutating_params(facts: &GrowFacts, solution: &GrowSolution, params: &BTreeMap<String, Vec<String>>) -> BTreeSet<(String, usize)>`

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_types/src/growable/positions_tests.rs`:

```rust
//! Growable-runtime-arrays spec §3.3, §3.5-§3.6, A-3, A-6, A-8.

use std::collections::BTreeMap;

use super::growable_refusals;
use crate::growable::flow::solve;

fn params(src: &str) -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::new();
    for stmt in crate::test_support::parse_statements(src) {
        if let kali_ast::Statement::FunctionDeclaration(decl) = stmt {
            map.insert(decl.name.clone(), decl.params.clone());
        }
    }
    map
}

fn refusals(src: &str) -> Vec<String> {
    let facts = crate::repr_infer::growable_facts_for(&crate::test_support::parse_statements(src));
    let solution = solve(&facts);
    growable_refusals(&facts, &solution, &params(src))
}

fn assert_one(src: &str, needle: &str) {
    let messages = refusals(src);
    assert!(
        messages.iter().any(|m| m.contains(needle)),
        "{src}\nexpected a refusal containing {needle:?}, got {messages:?}"
    );
}

#[test]
fn a_program_that_uses_growable_arrays_only_in_supported_positions_is_quiet() {
    let src = "function build(n) { const out = []; for (let i = 0; i < n; i++) out.push(i); return out; }\n\
               function total(a) { let s = 0; for (const x of a) s += x; return s; }\n\
               function main() { const xs = build(3); xs[0] = 5; const ys = xs.slice(1); console.log(xs.length, xs[1], xs.join(\",\"), ys.indexOf(2), total(xs), xs.pop()); }\n\
               main();";
    assert_eq!(refusals(src), Vec::<String>::new());
}

#[test]
fn a_plain_use_is_refused_naming_the_binding() {
    assert_one(
        "function main() { const a = []; a.push(1); console.log(typeof a); }",
        "the growable array `a` in `main` is used as a plain value here",
    );
}

#[test]
fn printing_a_whole_growable_array_is_refused() {
    assert_one(
        "function main() { const a = []; a.push(1); console.log(\"n\", a); }",
        "printing a whole runtime array is unavailable",
    );
}

#[test]
fn a_function_using_a_module_level_growable_array_is_refused() {
    assert_one(
        "const out = []; out.push(1); function size() { return out.length; } console.log(size());",
        "function `size` uses the module-level growable array `out`",
    );
}

#[test]
fn a_captured_growable_array_is_refused() {
    assert_one(
        "function main() { const o = []; o.push(1); const f = () => o.length; console.log(f()); }",
        "the growable array `o` in `main` is captured",
    );
}

#[test]
fn a_class_body_beside_a_growable_array_is_refused_as_a_capture() {
    assert_one(
        "function main() { const o = []; o.push(1); class C {} console.log(o.length); }",
        "the growable array `o` in `main` is captured",
    );
}

#[test]
fn from_index_length_writes_and_unsupported_methods_are_refused() {
    assert_one("const a = []; a.push(1); a.indexOf(1, 1);", "`indexOf` with a `fromIndex` argument");
    assert_one("const a = []; a.push(1); a.includes(1, 1);", "`includes` with a `fromIndex` argument");
    assert_one("const a = []; a.push(1); a.length = 0;", "assigning to `.length` of the growable array `a` at module scope");
    assert_one("const a = []; a.push(1); a.reverse();", "`.reverse()` on the growable array `a` at module scope");
    assert_one("const a = []; a.push(1); a[\"push\"](2);", "`[\"push\"]()` on the growable array `a`");
}

#[test]
fn indexing_or_mutating_a_call_result_directly_is_refused() {
    let src = "function make() { const o = []; o.push(1); return o; } console.log(make()[0]); make().push(2);";
    assert_one(src, "indexing, `push` or `pop` directly on the array `make(…)` returns");
}

#[test]
fn a_mixed_layout_a_literal_expression_and_a_non_array_write_are_rendered() {
    assert_one(
        "function total(a) { return a.length; } const xs = []; xs.push(1); const p = new Array(2); total(xs); total(p);",
        "`a` in `total` would hold both a growable array",
    );
    assert_one(
        "function f(c) { if (c) return []; const o = []; o.push(1); return o; } const r = f(false); console.log(r.length);",
        "an array literal written directly as a call argument or `return` value in `f`",
    );
    assert_one(
        "function f(a) { a.push(1); return a.length; } const xs = [0]; f(xs); f(5);",
        "`a` in `f` holds a growable array and is also given a value that is not an array",
    );
}

#[test]
fn a_push_on_the_iterated_array_inside_its_for_of_is_refused() {
    assert_one(
        "const a = []; a.push(1); for (const x of a) { if (x < 3) a.push(x + 1); }",
        "`push` or `pop` on the growable array `a` at module scope",
    );
}

#[test]
fn a_push_through_an_alias_inside_a_for_of_over_the_original_is_refused() {
    // Review Focus 2.
    assert_one(
        "function main() { const a = []; a.push(1); const b = a; for (const x of a) { if (x < 3) b.push(x + 1); } }",
        "`push` or `pop` on the growable array `a` in `main`",
    );
}

#[test]
fn a_call_that_pushes_the_iterated_array_through_a_parameter_is_refused() {
    // Review Focus 4: node grows the iteration; kali's snapshot would not.
    let src = "function add(arr, v) { arr.push(v); }\n\
               function relay(arr, v) { add(arr, v); }\n\
               function main() { const a = []; a.push(1); for (const x of a) { if (x < 3) relay(a, x + 1); } }";
    assert_one(src, "`push` or `pop` on the growable array `a` in `main`");
}

#[test]
fn a_loop_that_pushes_another_array_or_only_reads_its_own_is_admitted() {
    let src = "function total(a) { let s = 0; for (const x of a) s += x; return s; }\n\
               function main() { const a = []; a.push(1); const b = []; b.push(0); for (const x of a) { b.push(x); console.log(total(a)); } }";
    assert_eq!(refusals(src), Vec::<String>::new());
}

#[test]
fn a_program_without_growable_arrays_is_quiet() {
    assert_eq!(
        refusals("function f() { const a = [1, 2]; return a.length; } const p = new Array(3); p[0] = 1; console.log(f(), typeof p);"),
        Vec::<String>::new()
    );
}
```

Wire it at the end of `positions.rs`: `#[cfg(test)] #[path = "positions_tests.rs"] mod positions_tests;`.

- [ ] **Step 2: Run them to verify they fail**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types growable::positions -- --test-threads=6`
Expected: compile error, `cannot find function growable_refusals`.

- [ ] **Step 3: Write `positions.rs`**

```rust
//! Growable-runtime-arrays spec §3.3, §3.5-§3.6, A-3, A-6, A-8: every
//! refusal the solved growable property implies, as message text. Pure: the
//! facts and the solution in, strings out; repr inference turns each into a
//! shape conflict (E5506), so `check` and `run` report them alike.

use std::collections::{BTreeMap, BTreeSet};

use super::flow::{GrowConflict, GrowFacts, GrowNode, GrowSolution, TempKind, UseKind};

pub(crate) fn growable_refusals(
    facts: &GrowFacts,
    solution: &GrowSolution,
    params: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    for conflict in &solution.conflicts {
        messages.push(match conflict {
            GrowConflict::MixedLayout(node) => {
                kali_common::growable_mixed_layout_message(&subject(node, facts))
            }
            GrowConflict::NonArrayWrite(node) => {
                kali_common::growable_non_array_write_message(&subject(node, facts))
            }
            GrowConflict::LiteralExpression(site) => {
                kali_common::growable_literal_expression_message(site)
            }
        });
    }
    for occurrence in &facts.uses {
        if !solution.is_growable(&occurrence.node) {
            continue;
        }
        let named = matches!(occurrence.node, GrowNode::Binding(..));
        let node = &occurrence.node;
        let message = match &occurrence.kind {
            UseKind::Push | UseKind::Pop | UseKind::IndexRead | UseKind::IndexWrite if !named => {
                Some(kali_common::growable_temporary_use_message(&source(node, facts)))
            }
            UseKind::Flow
            | UseKind::Push
            | UseKind::Pop
            | UseKind::IndexRead
            | UseKind::IndexWrite
            | UseKind::LengthRead
            | UseKind::ForOf
            | UseKind::Join
            | UseKind::Slice
            | UseKind::Search { from_index: false, .. } => None,
            UseKind::Search { method, from_index: true } => {
                Some(kali_common::growable_from_index_message(method))
            }
            UseKind::LengthWrite => {
                Some(kali_common::growable_length_write_message(&subject(node, facts)))
            }
            UseKind::Console => {
                Some(kali_common::runtime_array_print_unavailable_message().to_string())
            }
            UseKind::Method(operation) => Some(kali_common::growable_unsupported_operation_message(
                operation,
                &subject(node, facts),
            )),
            UseKind::Plain => Some(kali_common::growable_plain_use_message(&subject(node, facts))),
            UseKind::Captured => Some(kali_common::growable_capture_message(&subject(node, facts))),
            UseKind::ModuleRead => match node {
                GrowNode::Binding(_, name) => {
                    Some(kali_common::growable_module_read_message(name, &occurrence.site))
                }
                _ => None,
            },
        };
        messages.extend(message);
    }
    for stack in &facts.opaque_sites {
        for node in solution.growable_members() {
            if let GrowNode::Binding(func, _) = node {
                if stack.iter().any(|frame| frame == func) {
                    messages.push(kali_common::growable_capture_message(&subject(node, facts)));
                }
            }
        }
    }
    let mutating = length_mutating_params(facts, solution, params);
    for frame in &facts.loops {
        if !solution.is_growable(&frame.iterable) {
            continue;
        }
        let direct = frame
            .mutations
            .iter()
            .any(|receiver| solution.same_component(receiver, &frame.iterable));
        let through_call = frame.calls.iter().any(|call| {
            solution.same_component(&call.node, &frame.iterable)
                && mutating.contains(&(call.callee.clone(), call.index))
        });
        if direct || through_call {
            messages.push(kali_common::growable_for_of_mutation_message(&subject(
                &frame.iterable,
                facts,
            )));
        }
    }
    let mut seen = BTreeSet::new();
    messages.retain(|message| seen.insert(message.clone()));
    messages
}

/// Spec A-8: `(callee, index)` pairs whose parameter may have `push` or
/// `pop` applied to the array it receives: in the callee's own body, on the
/// parameter or on a local in its growable component, or through a further
/// call that passes such a node on. A least fixed point; it only grows.
pub(crate) fn length_mutating_params(
    facts: &GrowFacts,
    solution: &GrowSolution,
    params: &BTreeMap<String, Vec<String>>,
) -> BTreeSet<(String, usize)> {
    let indices_reaching = |site: &str, node: &GrowNode| -> Vec<usize> {
        params
            .get(site)
            .map(|names| {
                names
                    .iter()
                    .enumerate()
                    .filter(|(_, name)| {
                        solution.same_component(
                            &GrowNode::Binding(site.to_string(), (*name).clone()),
                            node,
                        )
                    })
                    .map(|(index, _)| index)
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut mutating = BTreeSet::new();
    for occurrence in &facts.uses {
        if matches!(occurrence.kind, UseKind::Push | UseKind::Pop) {
            for index in indices_reaching(&occurrence.site, &occurrence.node) {
                mutating.insert((occurrence.site.clone(), index));
            }
        }
    }
    loop {
        let mut changed = false;
        for call in &facts.calls {
            if !mutating.contains(&(call.callee.clone(), call.index)) {
                continue;
            }
            for index in indices_reaching(&call.site, &call.node) {
                changed |= mutating.insert((call.site.clone(), index));
            }
        }
        if !changed {
            return mutating;
        }
    }
}

fn subject(node: &GrowNode, facts: &GrowFacts) -> String {
    match node {
        GrowNode::Binding(func, name) => kali_common::growable_binding_subject(name, func),
        GrowNode::Return(func) => kali_common::growable_return_subject(func),
        GrowNode::Temp(n) => match facts.temp_kinds.get(n) {
            Some((TempKind::Slice, site)) => format!(
                "{} {}",
                kali_common::growable_slice_result_source(),
                kali_common::growable_scope_phrase(site)
            ),
            Some((_, site)) => format!(
                "a `?:`, `||` or `&&` value {}",
                kali_common::growable_scope_phrase(site)
            ),
            None => "a temporary array".to_string(),
        },
    }
}

/// Spec A-6: what a refusal of direct indexing or mutation names.
fn source(node: &GrowNode, facts: &GrowFacts) -> String {
    match node {
        GrowNode::Return(func) => kali_common::growable_call_result_source(func),
        GrowNode::Temp(n) if matches!(facts.temp_kinds.get(n), Some((TempKind::Slice, _))) => {
            kali_common::growable_slice_result_source().to_string()
        }
        other => subject(other, facts),
    }
}

#[cfg(test)]
#[path = "positions_tests.rs"]
mod positions_tests;
```

- [ ] **Step 4: Run the tests**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types growable:: -- --test-threads=6`
Expected: PASS. If `a_program_that_uses_growable_arrays_only_in_supported_positions_is_quiet` reports a `Plain` use, the walk classified a supported position as plain: fix the walk (Task 4's method for that position), never the check.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_types/src/growable
git commit -m "feat(growable-runtime-arrays): position, capture, module-read and snapshot refusals (spec §3.6, A-3, A-6, A-8)"
```

---
### Task 6: Repr inference runs the solve and publishes it (the allowlist retires)

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs`:
  - fields `growable_candidates`/`growable_pushes`/`growable_rejects` (`:1289-1311`);
  - Phase A3 in `infer_reprs` (`:1476-1481`) and in `nested_fn_lockstep_sets` (`:1541`);
  - `NestedFnWalk::Growable` (`:1574-1576`, `:2940-2955`);
  - `collect_growable_candidates(_in_stmt)` (`:2832-2879`);
  - `classify_array_return` (`:1989-1993`);
  - the `push` arm (`:5330-5382`) and the `join` arm (`:5404-5411`) of `visit_call`;
  - `resolve_array_returns` (before `solve` at `:5792`);
  - `emit_table`: after the "Array elements" loop (`:6937`), the shadow publication (`:6982-6990`), and the Stage 4 promotion block (`:7437-7550`);
  - the three Stage 4 message functions (`:7722-7776`).
- Modify: `crates/kali_types/src/array_return.rs` (`ArrayReturnFacts` at `:488-511`, `solve` at `:559-716`)
- Modify: `crates/kali_types/src/growable/mod.rs` and `growable/growable_tests.rs` (delete the Stage 4 candidate code)
- Modify: `crates/kali_common/src/messages.rs:127` (delete `ARRAY_RETURN_GROWABLE`)
- Test: `crates/kali_types/src/repr_infer_tests.rs`, `crates/kali_types/src/array_return_tests.rs`

**Interfaces:**
- Consumes: `flow::{solve, GrowNode, GrowSolution, GrowFacts}` (Task 3), `ReprInfer::collect_growable_facts` (Task 4), `positions::growable_refusals` (Task 5), `ReprTable::{set_growable_return, mark_growable_local_only}` (Task 2).
- Produces:
  - `ReprInfer.growable: crate::growable::flow::GrowSolution` and `ReprInfer.growable_facts: crate::growable::flow::GrowFacts`, filled in Phase A3 (before Phase B); Task 7 reads both.
  - `ReprTable` after inference: `is_growable_array_binding(f, n)` for every binding and parameter of a growable component; `growable_return(f) == Some(elem)` for every function whose return is; `is_growable_local_only(f, n)` per A-16; `array_element(f, n)` one repr per component; every Task 5 refusal as a shape conflict; `is_array_return_callee_shadowed(scope, f)` also for growable-returning `f`.
  - `ArrayReturnFacts.growable_returning: BTreeSet<String>`: functions `solve` neither admits nor taints.

- [ ] **Step 1: Write the failing tests**

In `crates/kali_types/src/repr_infer_tests.rs`, delete the Stage 4 block from `// ---- throw-fallout Stage 4: growable-array promotion gate ----` (`:806`) through the end of `growable_promotion_blocks_escaping_and_module_scope_bindings` (`:910`), and delete `array_return_growable_const_literal_taints_growable` (`:1953-1965`) — both pin the retired allowlist (the escaping and module-scope bindings it pinned as never growable are growable now). Append:

```rust
// ---- growable-runtime-arrays: the solve, published --------------------------

#[test]
fn a_returned_growable_array_reaches_the_call_bound_binding() {
    let t = reprs(
        "function build(n) { const out = []; for (let i = 0; i < n; i++) out.push(i * i); return out; }\n\
         const xs = build(5);\nconsole.log(xs.length, xs[2]);\n",
    );
    assert!(t.is_growable_array_binding("build", "out"));
    assert!(t.is_growable_array_binding("_start", "xs"));
    assert_eq!(t.growable_return("build"), Some(Repr::I64));
    assert_eq!(t.array_return("build"), None);
    assert_eq!(t.array_return_taint("build"), None);
    assert!(!t.is_growable_local_only("build", "out"));
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
}

#[test]
fn an_argument_makes_the_parameter_growable() {
    let t = reprs(
        "function add(a) { a.push(2); }\n\
         function main() { const xs = [1]; add(xs); console.log(xs.length); }\nmain();\n",
    );
    assert!(t.is_growable_array_binding("main", "xs"));
    assert!(t.is_growable_array_binding("add", "a"));
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
}

#[test]
fn an_alias_shares_the_growable_array_and_escapes() {
    let t = reprs(
        "function main() { const a = []; a.push(5); const b = a; b.push(6); console.log(a.length); }\nmain();\n",
    );
    assert!(t.is_growable_array_binding("main", "a"));
    assert!(t.is_growable_array_binding("main", "b"));
    assert!(!t.is_growable_local_only("main", "a"));
}

#[test]
fn a_slice_result_is_growable_when_its_receiver_is() {
    let t = reprs(
        "function main() { const a = []; a.push(1); const t = a.slice(0); console.log(t.length); }\nmain();\n",
    );
    assert!(t.is_growable_array_binding("main", "t"));
}

#[test]
fn a_module_scope_growable_array_is_growable_but_never_local_only() {
    let t = reprs("const o = [];\no.push(1);\nconsole.log(o.length);\n");
    assert!(t.is_growable_array_binding("_start", "o"));
    assert!(!t.is_growable_local_only("_start", "o"));
}

#[test]
fn a_growable_array_that_stays_in_its_function_is_local_only() {
    let t = reprs("function main() { const o = []; o.push(1); console.log(o.length); }\nmain();\n");
    assert!(t.is_growable_array_binding("main", "o"));
    assert!(t.is_growable_local_only("main", "o"));
}

#[test]
fn a_returned_literal_nobody_mutates_stays_on_the_array_return_lane() {
    // Spec A-2, measured on `ret1.js`: this program runs today.
    let t = reprs("function f() { const a = [1, 2, 3]; return a; }\nconst xs = f();\nconsole.log(xs.length);\n");
    assert_eq!(t.array_return("f"), Some(Repr::I64));
    assert_eq!(t.growable_return("f"), None);
    assert!(!t.is_growable_array_binding("_start", "xs"));
}

#[test]
fn string_pushes_give_string_elements_across_the_return() {
    let t = reprs(
        "function build(n) { const out = []; let w = \"a\"; for (let i = 0; i < n; i++) { out.push(w); w = w + \"b\"; } return out; }\n\
         const ws = build(3);\nconsole.log(ws.length);\n",
    );
    assert_eq!(t.array_element("build", "out"), Repr::String);
    assert_eq!(t.array_element("_start", "ws"), Repr::String);
    assert_eq!(t.growable_return("build"), Some(Repr::String));
}

#[test]
fn a_mixed_number_and_string_push_set_is_the_existing_element_conflict() {
    // Spec A-4.
    let t = reprs("function main() { const o = []; o.push(1); o.push(\"a\"); console.log(o.length); }\nmain();\n");
    assert!(
        t.shape_conflicts().iter().any(|m| m.contains("used as both strings and numbers")),
        "{:?}",
        t.shape_conflicts()
    );
}

#[test]
fn the_solved_refusals_become_shape_conflicts() {
    for (src, needle) in [
        (
            "function total(a) { return a.length; }\nconst xs = []; xs.push(1); const p = new Array(2);\nconsole.log(total(xs), total(p));\n",
            "`a` in `total` would hold both a growable array",
        ),
        (
            "const out = []; out.push(1);\nfunction size() { return out.length; }\nconsole.log(size());\n",
            "function `size` uses the module-level growable array `out`",
        ),
        (
            "function main() { const o = []; o.push(1); const f = () => o.length; console.log(f()); }\nmain();\n",
            "the growable array `o` in `main` is captured",
        ),
    ] {
        let t = reprs(src);
        assert!(
            t.shape_conflicts().iter().any(|m| m.contains(needle)),
            "{src}\n{:?}",
            t.shape_conflicts()
        );
    }
}
```

Append to `crates/kali_types/src/array_return_tests.rs`:

```rust
#[test]
fn a_growable_returning_function_is_neither_admitted_nor_tainted() {
    let mut facts = ArrayReturnFacts::default();
    facts.declaration_counts.insert("f".to_string(), 1);
    facts.candidate_forms.insert("f".to_string());
    facts.called.insert("f".to_string());
    facts.returns.insert(
        "f".to_string(),
        vec![ReturnArg::BadArray(kali_common::ARRAY_RETURN_LET_LITERAL)],
    );
    let none: BTreeSet<String> = BTreeSet::new();
    let params: BTreeMap<String, Vec<String>> = BTreeMap::new();
    // Without the growable fact, the `let` literal return taints `f`.
    let before = solve(&facts, &[], &params, &none, &|_, _| false);
    assert!(before.tainted.contains_key("f"));
    facts.growable_returning.insert("f".to_string());
    let after = solve(&facts, &[], &params, &none, &|_, _| false);
    assert!(after.array_returning.is_empty());
    assert!(after.tainted.is_empty());
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types growable -- --test-threads=6`
Expected: compile error (`growable_returning` missing) — then, once it compiles, the new `repr_infer_tests` fail (for example `a_returned_growable_array_reaches_the_call_bound_binding`: `is_growable_array_binding("_start", "xs")` is false).

- [ ] **Step 3: The array-return lane steps aside**

In `crates/kali_types/src/array_return.rs`, add to `ArrayReturnFacts`:

```rust
    /// Growable-runtime-arrays spec §3.1: functions whose return value is in
    /// a growable component. They return a growable handle, so they are
    /// neither admitted to this lane nor tainted by it.
    pub(crate) growable_returning: BTreeSet<String>,
```

In `solve`, add `&& !facts.growable_returning.contains(*f)` to the initial `returning` filter (`:578-583`), and as the first statement of the taint loop body (`for (f, args) in &facts.returns {`, `:660`):

```rust
                if facts.growable_returning.contains(f) {
                    continue;
                }
```

- [ ] **Step 4: Replace Phase A3 and delete the allowlist**

In `ReprInfer`, replace the three fields at `:1289-1311` with:

```rust
    /// Growable-runtime-arrays spec §3.1: the facts and their solution,
    /// computed in Phase A3, before any body walk. Phase B's `push`, `pop`,
    /// search, `join` and `for-of` arms consult `growable`; `emit_table`
    /// publishes it and turns its refusals into shape conflicts.
    growable_facts: crate::growable::flow::GrowFacts,
    growable: crate::growable::flow::GrowSolution,
```

Add the method (beside `collect_growable_facts`):

```rust
    /// Phase A3 (growable-runtime-arrays spec §3.1).
    fn solve_growable(&mut self, statements: &[Statement]) {
        let facts = self.collect_growable_facts(statements);
        self.growable = crate::growable::flow::solve(&facts);
        self.growable_facts = facts;
    }
```

In `infer_reprs`, replace the Phase A3 comment and call (`:1476-1481`) with:

```rust
    // Phase A3 (growable-runtime-arrays spec §3.1): the growable facts and
    // their solution, before any body walk, so Phase B's arms know which
    // arrays are growable. Replaces throw-fallout Stage 4's allowlist.
    infer.solve_growable(statements);
```

and in `nested_fn_lockstep_sets` replace `infer.collect_growable_candidates(statements);` with `infer.solve_growable(statements);`. Delete `collect_growable_candidates`, `collect_growable_candidates_in_stmt`, the `NestedFnWalk::Growable` variant and its `register_nested_fn` arm (the `Functions` and `LocalNames` walks still register every `__kali_fn_N` into `nested_fns_registered`), and update the doc comments that count "three walks" to two. Delete `growable_unsupported_position_message`, `growable_unsupported_push_message` and `growable_unsupported_element_message` (`:7722-7776`); keep `growable_push_identifier_ok` (Task 7 uses it).

In `crates/kali_types/src/growable/mod.rs`, delete the module doc's allowlist description (replace it with a two-line doc naming `flow`, `facts`, `positions` and the object-field helper), `GrowableRejectKind`, `GrowablePushSite`, `DeclInfo`, `Scan` and its `impl`, `growable_array_candidates`, `push_argument_shape_ok`, `push_receiver_base` and `strip_parens_and_optional`. Keep `array_literal_of_scalar_seeds`, `scalar_value_shape_ok`, `strip_parens`, and `statement_contains_push_on` with its `push_scan_*` helpers (the resolver still calls it until Task 8). In `growable/growable_tests.rs`, delete every test that calls the removed functions (all of `:1-235` that reference `growable_array_candidates`, `candidates`, `rejects`, `reject_kinds`); if nothing remains, delete the file and its `#[path]` module declaration.

- [ ] **Step 5: Phase B consults the solution**

In the `"push"` arm of `visit_call` (`:5345-5378`), replace the `if call.args.len() == 1 && self.growable_candidates.contains(…) { … }` block with:

```rust
                        if let Expression::Identifier(name) = receiver {
                            // Growable-runtime-arrays spec §3.4: every pushed
                            // value is a store into the element node, so the
                            // element solve sees i64, f64 and string pushes,
                            // and a mix is the existing mixed-store conflict.
                            if self.growable.is_growable_binding(func, name) {
                                let elem = self.array_elem_node_for(func, name);
                                for (arg, &node) in call.args.iter().zip(&arg_nodes) {
                                    self.add_edge(node, elem);
                                    self.element_store_sources.push((elem, node));
                                    self.elem_number_obligations.push((
                                        elem,
                                        crate::array_return::num_proof(func, arg),
                                    ));
                                }
                            }
                        }
```

In the `"join"` arm (`:5404-5411`), replace `!self.growable_candidates.contains(&(func.to_string(), name.clone()))` with `!self.growable.is_growable_binding(func, name)`.

In `classify_array_return`, delete the growable check (`:1989-1993`, the `if self.growable_candidates.contains(&key) || self.growable_rejects.contains_key(&key) { return ReturnArg::BadArray(kali_common::ARRAY_RETURN_GROWABLE); }` lines and the doc bullet above it at `:1975-1977`). Delete `ARRAY_RETURN_GROWABLE` from `crates/kali_common/src/messages.rs:127`.

In `resolve_array_returns`, immediately before `let solution = crate::array_return::solve(` (`:5792`):

```rust
        self.array_return_facts.growable_returning = self
            .growable
            .growable_members()
            .filter_map(|node| match node {
                crate::growable::flow::GrowNode::Return(f) => Some(f.clone()),
                _ => None,
            })
            .collect();
```

- [ ] **Step 6: One element repr per growable component**

Add, and call in `infer_reprs` right after the Phase B loop (after `infer.assert_nested_fn_lockstep();`, `:1505`) as `infer.union_growable_elements();` with the comment `// Phase B2 (growable-runtime-arrays spec §3.4): one element repr per growable array.`:

```rust
    /// Union the element node of every binding and return in each growable
    /// component, so the existing element solve gives the whole component
    /// one element repr. Temporaries carry no element node of their own.
    fn union_growable_elements(&mut self) {
        use crate::growable::flow::GrowNode;
        let members: Vec<GrowNode> = self.growable.growable_members().cloned().collect();
        let mut first: BTreeMap<usize, usize> = BTreeMap::new();
        for node in members {
            let Some(component) = self.growable.component_of(&node) else {
                continue;
            };
            let elem = match &node {
                GrowNode::Binding(func, name) => self.array_elem_node_for(func, name),
                GrowNode::Return(func) => {
                    self.array_elem_node_for(func, crate::array_return::RETURN_ARRAY_KEY)
                }
                GrowNode::Temp(_) => continue,
            };
            match first.get(&component) {
                Some(&root) => self.uf.union(root, elem),
                None => {
                    first.insert(component, elem);
                }
            }
        }
    }
```

- [ ] **Step 7: `emit_table` publishes the solution**

Immediately after the "Array elements" loop (after `:6937`, before `// Array-return lane (spec 2026-10-02 §3.1)`):

```rust
        // Growable-runtime-arrays spec §3.1-§3.3: publish the solution — after
        // the element pass (a growable return's element repr is read from it),
        // before every later consumer of the growable set (the numeric-binding
        // proof below excludes growable bindings).
        for node in self.growable.growable_members() {
            match node {
                crate::growable::flow::GrowNode::Binding(func, name) => {
                    table.set_growable_array_binding(func, name);
                    if self.growable.is_local_only(func, name) {
                        table.mark_growable_local_only(func, name);
                    }
                }
                crate::growable::flow::GrowNode::Return(func) => {
                    let elem = table.array_element(func, crate::array_return::RETURN_ARRAY_KEY);
                    table.set_growable_return(func, elem);
                }
                crate::growable::flow::GrowNode::Temp(_) => {}
            }
        }
        for message in crate::growable::positions::growable_refusals(
            &self.growable_facts,
            &self.growable,
            &self.functions,
        ) {
            table.add_shape_conflict(message);
        }
```

In the callee-shadow publication (`:6982-6990`), after the existing `for f in table_array_returning_names(…)` loop, add:

```rust
        let growable_returning: Vec<String> = self
            .growable
            .growable_members()
            .filter_map(|node| match node {
                crate::growable::flow::GrowNode::Return(f) => Some(f.clone()),
                _ => None,
            })
            .collect();
        for f in growable_returning {
            for scope in scopes.iter().map(|s| s.as_str()).chain([TOP_LEVEL]) {
                if self.callee_is_shadowed(scope, &f) {
                    table.set_array_return_callee_shadowed(scope, &f);
                }
            }
        }
```

Delete the whole Stage 4 promotion block and the `growable_rejects` reporting loop (`:7437-7550`, from `// Growable-array promotion (throw-fallout Stage 4)` through `table.add_shape_conflict(message);` `}` of the rejects loop). Task 7 puts the element-kind check there.

- [ ] **Step 8: Run the tests**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types -- --test-threads=6`
Expected: PASS. A failure in an unrelated `kali_types` test that names Stage 4 behaviour (an escaping or module-scope push that used to stay plain) is a moved pin: check the program under node, rewrite the assertion to the new solved fact, and note it for Task 13's followups §4. Any other failure: STOP and debug with superpowers:systematic-debugging.

- [ ] **Step 9: Commit**

```bash
git add -A crates/kali_types crates/kali_common/src/messages.rs
git commit -m "feat(growable-runtime-arrays): repr inference solves and publishes the growable property; Stage 4 allowlist retired (spec §3.1, A-2, A-3)"
```

---

### Task 7: Element reprs through `for-of`, `pop` and search; unsupported elements refused

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs` (`visit_stmt`'s `ForOfStatement` arm at `:3649-3715`; `visit_call`'s method `match` at `:5177`; `emit_table` where Task 6 removed the Stage 4 block)
- Test: `crates/kali_types/src/repr_infer_tests.rs`

**Interfaces:**
- Consumes: `ReprInfer.growable`, `ReprInfer.growable_facts` (Task 6); `ElementValue` (Task 3); `growable_unsupported_element_message` (Task 1).
- Produces: `fn growable_elem_node_of(&mut self, func: &str, expr: &Expression) -> Option<usize>` (the element node of a growable value expression: a growable binding, a call to a growable-returning function, or a `slice` of either). After this task: a `for-of` loop variable over a growable value carries the element repr (`scalar(f, loop_var)` is `String`/`F64`); `const v = xs.pop()` carries it; a search value is an element store (A-14); M2 is raised.

- [ ] **Step 1: Write the failing tests**

Append to `crates/kali_types/src/repr_infer_tests.rs`:

```rust
#[test]
fn float_pushes_give_f64_elements_through_parameters_and_returns() {
    let t = reprs(
        "function averages(xs, k) { const out = []; for (let i = 0; i + k <= xs.length; i++) { let s = 0; for (let j = 0; j < k; j++) s += xs[i + j]; out.push(s / k); } return out; }\n\
         const data = [];\nfor (let i = 1; i <= 5; i++) data.push(i * 1.5);\nconst avg = averages(data, 2);\nconsole.log(avg.length);\n",
    );
    assert_eq!(t.array_element("_start", "data"), Repr::F64);
    assert_eq!(t.array_element("averages", "xs"), Repr::F64);
    assert_eq!(t.array_element("_start", "avg"), Repr::F64);
    assert_eq!(t.growable_return("averages"), Some(Repr::F64));
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
}

#[test]
fn a_for_of_loop_variable_takes_the_element_repr() {
    let t = reprs(
        "function main() { const ws = []; ws.push(\"a\"); const fs = []; fs.push(0.5); let n = \"\"; let s = 0;\n\
         for (const w of ws) n = n + w;\nfor (const f of fs) s = s + f;\nfor (const x of fs.slice(0)) s = s + x;\nconsole.log(n, s); }\nmain();\n",
    );
    assert_eq!(t.scalar("main", "w"), Repr::String);
    assert_eq!(t.scalar("main", "f"), Repr::F64);
    assert_eq!(t.scalar("main", "x"), Repr::F64);
}

#[test]
fn a_for_of_over_a_call_takes_the_returned_element_repr() {
    let t = reprs(
        "function build() { const out = []; out.push(\"a\"); return out; }\nfor (const line of build()) console.log(line);\n",
    );
    assert_eq!(t.scalar("_start", "line"), Repr::String);
}

#[test]
fn pop_yields_the_element_repr() {
    let t = reprs(
        "function main() { const s = []; s.push(\"x\"); const top = s.pop(); const f = []; f.push(1.5); const v = f.pop(); console.log(top, v); }\nmain();\n",
    );
    assert_eq!(t.scalar("main", "top"), Repr::String);
    assert_eq!(t.scalar("main", "v"), Repr::F64);
}

#[test]
fn a_float_search_value_makes_the_array_f64() {
    // Spec A-14.
    let t = reprs("function main() { const a = []; a.push(1); let h = 0.5; console.log(a.indexOf(h)); }\nmain();\n");
    assert_eq!(t.array_element("main", "a"), Repr::F64);
}

#[test]
fn a_string_search_value_in_a_number_array_is_the_mixed_element_conflict() {
    let t = reprs("function main() { const a = []; a.push(1); console.log(a.includes(\"1\")); }\nmain();\n");
    assert!(
        t.shape_conflicts().iter().any(|m| m.contains("used as both strings and numbers")),
        "{:?}",
        t.shape_conflicts()
    );
}

#[test]
fn unsupported_elements_are_refused() {
    for src in [
        "function main() { const o = []; o.push({a: 1}); console.log(o.length); }\nmain();\n",
        "function main() { const o = []; o.push(true); console.log(o.length); }\nmain();\n",
        "function main() { const o = []; o.push(undefined); console.log(o.length); }\nmain();\n",
        "function main() { const o = [1, , 2]; o.push(3); console.log(o.length); }\nmain();\n",
        "function g() { return 1; }\nfunction main() { const o = []; o.push(g); console.log(o.length); }\nmain();\n",
        "function main() { const inner = []; inner.push(1); const o = []; o.push(inner); console.log(o.length); }\nmain();\n",
        "function main() { const obj = {a: 1}; const o = []; o.push(obj); console.log(o.length); }\nmain();\n",
    ] {
        let t = reprs(src);
        assert!(
            t.shape_conflicts()
                .iter()
                .any(|m| m.contains("is a growable array with an element that is")),
            "{src}\n{:?}",
            t.shape_conflicts()
        );
    }
}

#[test]
fn number_and_string_elements_are_not_refused_as_unsupported() {
    let t = reprs(
        "function main() { const o = []; const n = 2; o.push(1, n, n * 3); const s = []; const w = \"x\"; s.push(w, \"y\"); console.log(o.length, s.length); }\nmain();\n",
    );
    assert!(t.shape_conflicts().is_empty(), "{:?}", t.shape_conflicts());
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types repr_infer_tests -- --test-threads=6`
Expected: FAIL — e.g. `a_for_of_loop_variable_takes_the_element_repr` (`scalar("main", "w")` is `I64`), `pop_yields_the_element_repr`, `unsupported_elements_are_refused`.

- [ ] **Step 3: The element node of a growable value**

Add to `impl ReprInfer` (beside `binding_has_element_node`, `:5017`):

```rust
    /// Growable-runtime-arrays spec §3.4: the element node of a growable value
    /// expression — a growable binding, a direct call to a growable-returning
    /// function, or a `slice` of either. `None` for anything else.
    fn growable_elem_node_of(&mut self, func: &str, expr: &Expression) -> Option<usize> {
        match crate::array_return::unparen(expr) {
            Expression::Identifier(name) if self.growable.is_growable_binding(func, name) => {
                Some(self.array_elem_node_for(func, name))
            }
            Expression::CallExpression(call) => {
                if let Expression::MemberExpression(member) =
                    crate::array_return::unparen(&call.callee)
                {
                    return if member.dot_name() == Some("slice") {
                        self.growable_elem_node_of(func, &member.object)
                    } else {
                        None
                    };
                }
                let Expression::Identifier(callee) = crate::array_return::unparen(&call.callee)
                else {
                    return None;
                };
                let key = self.array_return_callee(func, callee)?;
                self.growable
                    .is_growable(&crate::growable::flow::GrowNode::Return(key.clone()))
                    .then(|| self.array_elem_node_for(&key, crate::array_return::RETURN_ARRAY_KEY))
            }
            _ => None,
        }
    }
```

- [ ] **Step 4: Wire `for-of`, `pop` and search**

In the `ForOfStatement` arm of `visit_stmt`, immediately before `self.visit_expr(func, &stmt.right);` (`:3713`):

```rust
                // Growable-runtime-arrays spec §3.5: the loop variable holds
                // the elements, so it carries their repr (both axes, like an
                // element read).
                if let Some(elem) = self.growable_elem_node_of(func, &stmt.right) {
                    let loop_var = match &stmt.left {
                        kali_ast::ForOfLefthand::VariableDeclaration(decl) => {
                            decl.declarations.first().map(|d| d.id.clone())
                        }
                        kali_ast::ForOfLefthand::Expression(Expression::Identifier(name)) => {
                            Some(name.clone())
                        }
                        kali_ast::ForOfLefthand::Expression(_) => None,
                    };
                    if let Some(var) = loop_var {
                        let node = self.scalar_node_for(func, &var);
                        self.add_edge(elem, node);
                    }
                }
```

In `visit_call`'s method `match method { … }`, add these arms before `_ =>` (`:5462`):

```rust
                    // Growable-runtime-arrays spec §3.5: `pop` yields an element.
                    "pop" => {
                        let elem = self.growable_elem_node_of(func, &member.object);
                        self.visit_expr(func, &member.object);
                        for arg in &call.args {
                            self.visit_expr(func, arg);
                        }
                        let result = self.new_node();
                        if let Some(elem) = elem {
                            self.add_edge(elem, result);
                        }
                        result
                    }
                    // Spec A-14: the search value counts as an element store,
                    // so a float needle makes the array f64 and a
                    // number/string mismatch is the mixed-store conflict.
                    "indexOf" | "includes" => {
                        let elem = self.growable_elem_node_of(func, &member.object);
                        self.visit_expr(func, &member.object);
                        let mut nodes = Vec::with_capacity(call.args.len());
                        for arg in &call.args {
                            nodes.push(self.visit_expr(func, arg));
                        }
                        if let (Some(elem), Some(&needle)) = (elem, nodes.first()) {
                            self.add_edge(needle, elem);
                            self.element_store_sources.push((elem, needle));
                        }
                        self.new_node()
                    }
```

- [ ] **Step 5: Refuse unsupported elements (M2)**

In `emit_table`, where Task 6 deleted the Stage 4 block (after the numeric-binding finalization, `table.set_boolean_consts(…)`; `materialized` and `fields_of` are the locals the object pass took):

```rust
        // Growable-runtime-arrays spec §3.4, A-4: a growable array holds only
        // numbers or only strings. A mix of the two is the element pass's
        // conflict above; everything else is refused here.
        {
            use crate::growable::flow::{ElementValue, GrowNode};
            let mut unsupported: BTreeSet<GrowNode> = BTreeSet::new();
            for (node, value) in &self.growable_facts.element_values {
                if !self.growable.is_growable(node) {
                    continue;
                }
                let bad = match (value, node) {
                    (ElementValue::Unsupported, _) => true,
                    (ElementValue::Identifier(name), GrowNode::Binding(func, _)) => {
                        !self.growable_push_identifier_ok(func, name, &fields_of, &materialized)
                            || self.fn_alias_target(func, name).is_some()
                    }
                    _ => false,
                };
                if bad {
                    unsupported.insert(node.clone());
                }
            }
            for node in self.growable.growable_members() {
                if let GrowNode::Binding(func, name) = node {
                    let slot = ObjSlot::ArrayElem(func.clone(), name.clone());
                    if materialized.contains(&slot) || fields_of.contains_key(&slot) {
                        unsupported.insert(node.clone());
                    }
                }
            }
            for node in unsupported {
                let subject = match &node {
                    GrowNode::Binding(func, name) => kali_common::growable_binding_subject(name, func),
                    GrowNode::Return(func) => kali_common::growable_return_subject(func),
                    GrowNode::Temp(_) => continue,
                };
                table.add_shape_conflict(kali_common::growable_unsupported_element_message(&subject));
            }
        }
```

If `materialized`/`fields_of` are named differently at that point of `emit_table`, use the names the deleted Stage 4 block used (it passed `&fields_of, &materialized` to `growable_push_identifier_ok` at `:7514`).

- [ ] **Step 6: Run the tests**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types -- --test-threads=6`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_types/src/repr_infer_tests.rs
git commit -m "feat(growable-runtime-arrays): element reprs through for-of, pop and search; unsupported elements refused (spec §3.4, A-4, A-14)"
```

---
### Task 8: Resolver mirrors keyed on the table

The resolver (the `check` front half) must admit exactly the growable programs codegen lowers, and stop routing growable arrays to the literal and static lanes.

**Files:**
- Modify: `crates/kali_types/src/resolve/expression.rs` (`is_growable_array_binding` at `:368-397`; `register_growable_array_binding` at `:855-878`; `reject_array_binding_scalar_reassignment` at `:1775`; new helpers beside `is_growable_array_binding`)
- Modify: `crates/kali_types/src/resolve/mod.rs` (the growable `for-of` arm at `:586-669`; the declarator registration at `:1008-1026`; param registration at `:777-783`)
- Modify: `crates/kali_types/src/resolve/function.rs` (param registration at `:42-49`, `:80-87`, `:145-151`)
- Modify: `crates/kali_types/src/scope.rs` (delete `growable_array_bindings`, `:84-92`, `:129`)
- Modify: `crates/kali_types/src/static_analysis/array.rs` (`resolve_array_slice_member_call` `:675`, `resolve_array_join_member_call` `:838-871`, `resolve_array_search_member_call` `:1033`, `resolve_static_array_binding_name` `:413`, `resolve_array_literal_binding_name` `:433`)
- Modify: `crates/kali_types/src/growable/mod.rs` (delete `statement_contains_push_on` and the `push_scan_*` helpers)
- Create: `crates/kali_types/src/static_analysis/array_tests/growable.rs`; Modify: `crates/kali_types/src/static_analysis/array_tests.rs` (add `#[path = "array_tests/growable.rs"] mod growable;` after `:44`)

**Interfaces:**
- Consumes: `ReprTable::{is_growable_array_binding, growable_return, is_array_return_callee_shadowed, array_element, is_array_element_non_ascii}` (Tasks 2, 6).
- Produces (on `TypeContext`, `pub(crate)`):
  - `is_growable_array_binding(&self, name: &str) -> bool` — now `binding_repr_function_key(name)` + the table (no scope registry).
  - `growable_receiver_elem(&self, expr: &Expression) -> Option<kali_common::Repr>` — `Some(element repr)` for a growable binding, a direct call to a growable-returning function, or a `.slice(…)` of either.
  - `growable_receiver_non_ascii(&self, expr: &Expression) -> bool` — fail-closed: `true` unless proven ASCII.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_types/src/static_analysis/array_tests/growable.rs`:

```rust
//! Growable-runtime-arrays spec §3.5: the resolver admits every growable lane
//! codegen lowers (these programs produce no resolver E5506).

use super::*;

fn e5506(source: &str) -> Vec<String> {
    let statements = crate::test_support::parse_statements(source);
    let mut ctx = TypeContext::new();
    ctx.resolve_statements(&statements)
        .diagnostics
        .into_iter()
        .filter(|d| d.code == Some(e5::FEATURE_UNAVAILABLE as u32))
        .map(|d| d.message)
        .collect()
}

#[test]
fn for_of_over_string_and_float_growable_arrays_is_admitted() {
    assert_eq!(
        e5506("function main() { const ws = []; ws.push(\"a\"); const fs = []; fs.push(0.5); let n = \"\"; let s = 0; for (const w of ws) n = n + w; for (const f of fs) s = s + f; console.log(n, s); }\nmain();"),
        Vec::<String>::new()
    );
}

#[test]
fn for_of_over_a_growable_call_result_and_a_slice_is_admitted() {
    assert_eq!(
        e5506("function build() { const out = []; out.push(1); return out; }\nfor (const x of build()) console.log(x);\nconst a = build();\nfor (const y of a.slice(1)) console.log(y);"),
        Vec::<String>::new()
    );
}

#[test]
fn nested_for_of_over_growable_arrays_is_admitted() {
    assert_eq!(
        e5506("function main() { const a = []; a.push(1); const b = []; b.push(2); let s = 0; for (const x of a) { for (const y of b) { s += x * y; } } console.log(s); }\nmain();"),
        Vec::<String>::new()
    );
}

#[test]
fn join_slice_and_search_on_growable_values_are_admitted() {
    assert_eq!(
        e5506("function main() { const a = []; for (let i = 0; i < 5; i++) a.push(i); let k = 1; console.log(a.join(\",\"), a.slice(k, -1).join(\"-\"), a.slice(1).slice(1).join(), a.indexOf(3), a.includes(k)); }\nmain();"),
        Vec::<String>::new()
    );
}

#[test]
fn a_growable_literal_is_not_a_literal_array_for_the_mutator_and_store_gates() {
    assert_eq!(
        e5506("function main() { const a = [1, 2]; a.push(3); a[0] = 5; let i = 1; a[i] = 6; console.log(a.pop(), a.length); }\nmain();"),
        Vec::<String>::new()
    );
}

#[test]
fn a_growable_parameter_index_and_length_are_admitted() {
    assert_eq!(
        e5506("function total(a) { let s = 0; for (let i = 0; i < a.length; i++) s += a[i]; return s; }\nfunction main() { const xs = []; xs.push(1); console.log(total(xs)); }\nmain();"),
        Vec::<String>::new()
    );
}

#[test]
fn a_literal_nobody_pushes_pops_or_writes_keeps_its_mutator_refusal() {
    // Unchanged lane (spec A-1): `reverse` is no growable demand, so `a` stays
    // a folded literal and the literal-array-mutators refusal still fires.
    // (`a[0] = 5` on the same literal WOULD make it growable: an index write
    // is a source, spec §3.1.)
    assert!(
        e5506("const a = [1, 2]; a.reverse(); console.log(a[0]);")
            .iter()
            .any(|m| m.contains("on a literal array")),
        "the literal-array mutator refusal must stay"
    );
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types array_tests::growable -- --test-threads=6`
Expected: FAIL — e.g. `for_of_over_string_and_float_growable_arrays_is_admitted` reports "for-of iteration over a growable (push-accumulated) array is unavailable…", and the call/slice/parameter tests report the static for-of, join, slice and search refusals.

- [ ] **Step 3: The table is the registry**

Replace the body of `is_growable_array_binding` (`resolve/expression.rs:374-397`) with:

```rust
    pub(crate) fn is_growable_array_binding(&self, name: &str) -> bool {
        // Growable-runtime-arrays spec §3.1: repr inference's solution is the
        // one registry (bindings, parameters, call-bound and aliased names
        // alike), keyed exactly as codegen keys it.
        self.binding_repr_function_key(name)
            .is_some_and(|func| self.repr_table.is_growable_array_binding(&func, name))
    }
```

and update its doc comment to say so. Delete `register_growable_array_binding` (`:855-878`), its only call in `resolve/mod.rs` (`:1008-1026`, the "Growable-array promotion (throw-fallout Stage 4)" block inside the declarator arm), and `growable_array_bindings` from `Scope` (`scope.rs:84-92`, `:129`).

Add beside `is_growable_array_binding`:

```rust
    /// Growable-runtime-arrays spec §3.5, A-6: the element repr of a growable
    /// value expression — a growable binding, a direct call to a growable-
    /// returning function (unshadowed, inference's own fact), or a `.slice(…)`
    /// of either. The resolve-side twin of codegen's `growable_value_elem`.
    pub(crate) fn growable_receiver_elem(&self, expr: &Expression) -> Option<kali_common::Repr> {
        match unwrap_transparent(expr) {
            Expression::Identifier(name) => {
                let func = self.binding_repr_function_key(name)?;
                self.repr_table
                    .is_growable_array_binding(&func, name)
                    .then(|| self.repr_table.array_element(&func, name))
            }
            Expression::CallExpression(call) => match unwrap_transparent(&call.callee) {
                Expression::MemberExpression(member) if member.dot_name() == Some("slice") => {
                    self.growable_receiver_elem(&member.object)
                }
                Expression::Identifier(callee) => {
                    if self
                        .repr_table
                        .is_array_return_callee_shadowed(self.current_function_name(), callee)
                    {
                        return None;
                    }
                    self.repr_table.growable_return(callee)
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Fail-closed: `true` unless the String elements of the growable value
    /// `expr` are proven ASCII (`join` counts bytes).
    pub(crate) fn growable_receiver_non_ascii(&self, expr: &Expression) -> bool {
        match unwrap_transparent(expr) {
            Expression::Identifier(name) => self.array_element_non_ascii(name),
            Expression::CallExpression(call) => match unwrap_transparent(&call.callee) {
                Expression::MemberExpression(member) if member.dot_name() == Some("slice") => {
                    self.growable_receiver_non_ascii(&member.object)
                }
                Expression::Identifier(callee) => self
                    .repr_table
                    .is_array_element_non_ascii(callee, "%return"),
                _ => true,
            },
            _ => true,
        }
    }
```

(`"%return"` is `crate::array_return::RETURN_ARRAY_KEY`; use the constant.)

In `reject_array_binding_scalar_reassignment` (`:1775`), right after `let Expression::Identifier(target) = &assign.left else { return; };`, add `if self.is_growable_array_binding(target) { return; }` — inference owns growable writes (A-7).

In `resolve_static_array_binding_name` (`static_analysis/array.rs:413`) and `resolve_array_literal_binding_name` (`:433`), add as the first statement `if self.is_growable_array_binding(name) { return false; }` — a growable binding is never a folded literal, so the static `for-of`/`slice`/`join`/search lanes and every literal-array gate (`member.rs:442`, `expression.rs:1680`, `:1748`, `array.rs:950`) skip it.

- [ ] **Step 4: Parameters stay off the plain lane**

In each of the four parameter registrations (`resolve/function.rs:43`, `:81`, `:146`; `resolve/mod.rs:778`), change the condition `self.repr_table.is_array_binding(F, P)` to `self.repr_table.is_array_binding(F, P) && !self.repr_table.is_growable_array_binding(F, P)` (with that site's own function and parameter expressions). A growable parameter has an element node, so it is an array binding too, but its layout is the header, not `[len][elem…]`.

- [ ] **Step 5: `for-of`, `join`, `slice` and search admit growable values**

In `resolve/mod.rs`, replace the bare-identifier growable block of the `ForOfStatement` arm (`:602-669`, from `if let Expression::Identifier(name) = rhs {` through its closing `}`) with:

```rust
                    // Growable-runtime-arrays spec §3.5: a growable binding, a
                    // call to a growable-returning function, or a slice of
                    // either runs codegen's counted loop, whatever the element
                    // repr; nesting is admitted. A push/pop on the iterated
                    // array inside the body is refused by inference (A-8).
                    if self.growable_receiver_elem(rhs).is_some() {
                        let left_is_supported = match left {
                            ForOfLefthand::VariableDeclaration(_) => true,
                            ForOfLefthand::Expression(expression) => {
                                self.is_simple_for_of_binding_expression(expression)
                            }
                        };
                        if left_is_supported && !*is_await {
                            self.push_scope(ScopeType::Block);
                            if let ForOfLefthand::VariableDeclaration(decl) = left {
                                self.resolve_variable_declaration(decl)
                            }
                            if matches!(rhs, Expression::Identifier(_)) {
                                self.resolve_static_string_fold_position(right);
                            } else {
                                self.resolve_expression(right);
                            }
                            self.resolve_loop_body(body);
                            self.pop_scope();
                            return;
                        }
                        let loop_kind = if *is_await { "for-await-of" } else { "for-of" };
                        self.diagnostics.push(Diagnostic::error(
                            e5::FEATURE_UNAVAILABLE as u32,
                            format!(
                                "{} iteration over a growable array is unavailable in the current phase unless the loop target is a variable/identifier binding; use an index loop over `.length` or the later compatibility path",
                                loop_kind
                            ),
                        ));
                        return;
                    }
```

In `resolve_array_join_member_call` (`static_analysis/array.rs:838-871`), replace the `if let Expression::Identifier(name) = receiver { if self.is_growable_array_binding(name) { … } }` block with:

```rust
            if let Some(elem) = self.growable_receiver_elem(receiver) {
                self.resolve_expression(&member.object);
                for arg in &expr.args {
                    self.resolve_expression(arg);
                }
                let supported_arg_count = matches!(expr.args.len(), 0 | 1);
                let separator_ok = expr.args.first().is_none_or(|argument| {
                    self.resolve_static_string_expression(argument)
                        .map(|s| s.is_ascii())
                        .unwrap_or_else(|| self.expression_repr_is_ascii_string(argument))
                });
                // i64 and f64 elements render ASCII digits; String elements
                // must be proven ASCII because `__join_growable_*` counts bytes.
                let elements_ok = elem != kali_common::Repr::String
                    || !self.growable_receiver_non_ascii(receiver);
                if supported_arg_count && separator_ok && elements_ok {
                    return;
                }
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Array.prototype.join on a growable array is unavailable unless it has at most one argument that is a proven-ASCII string separator and its String elements are all proven ASCII in the current phase; use an index loop over `.length` or the later compatibility path".to_string(),
                ));
                return;
            }
```

In `resolve_array_slice_member_call` (`:675`), after the `is_runtime_args_slice_member` early return, add:

```rust
        // Growable-runtime-arrays spec §3.5: runtime bounds on a growable value.
        if self.growable_receiver_elem(&member.object).is_some() {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }
```

In `resolve_array_search_member_call` (`:1033`), after the static-string early return, add (inference raises the `fromIndex` refusal, M6):

```rust
        // Growable-runtime-arrays spec §3.5: a runtime search value on a
        // growable value.
        if self.growable_receiver_elem(&member.object).is_some() {
            self.resolve_expression(&member.object);
            for arg in &expr.args {
                self.resolve_expression(arg);
            }
            return;
        }
```

Finally delete `statement_contains_push_on` and every `push_scan_*` helper from `growable/mod.rs` (the only caller was the block just replaced).

- [ ] **Step 6: Run the tests**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_types -- --test-threads=6`
Expected: PASS, including the 7 new tests. A pre-existing resolver test asserting "for-of iteration over a growable (push-accumulated) array is unavailable…" for string elements, or the old self-push resolver message, is a moved pin: rewrite it to the new behaviour (admitted, or inference's M5 under `infer_reprs`) and note it for Task 13.

- [ ] **Step 7: Commit**

```bash
git add -A crates/kali_types
git commit -m "feat(growable-runtime-arrays): resolver mirrors keyed on the repr table (spec §3.5)"
```

---
### Task 9: Codegen — layout, memory and bounds-checked element access

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (`SYNTHETIC_FUNCTIONS` `:54-73`; the `__array_elem_addr` `FunctionPlan` `:889-897`; the signature `match` `:1274`; the body dispatch `:1769`; a new body after `emit_array_elem_addr_body` `:7967`; the growable-scratch trigger `:3734-3744` and a new `body_contains_push_member` beside `body_contains_field_push`)
- Modify: `crates/kali_codegen/src/emitter.rs` (param `array_bindings` `:673-678`; a new accessor after `array_elem_addr_fn_index` `:1346`)
- Modify: `crates/kali_codegen/src/emit/growable.rs` (`emit_growable_alloc` `:85`, `emit_growable_push` `:203`, `emit_growable_index_read` `:470`, `emit_growable_push_call` `:615`, `subtree_mentions_growable` `:791-812`; new `growable_value_elem`, `emit_growable_element_address`, `emit_growable_index_write`; test module wiring at the end)
- Create: `crates/kali_codegen/src/emit/growable_tests.rs`
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (the growable declarator lane `:1738-1801`; the `.length` arm `:2900-2920`)
- Modify: `crates/kali_codegen/src/emit/literal.rs` (the dynamic array element write `:684`)
- Modify: `crates/kali_codegen/src/emit/call.rs` (the literal-argument refusal `:3727-3734`; the console guard `:1060-1088`, `emit_console_argument` `:122`, `emit_console_argument_as_string` `:169`)
- Modify: `crates/kali_codegen/src/emit/object.rs`, `crates/kali_codegen/src/emit/url.rs:217` (callers of `emit_growable_alloc`)
- Modify: `crates/kali_cli/tests/runtime_smoke.rs:807-826` (synthetic mirror list)
- Create: `crates/kali_cli/tests/cases/array/growable_layout.toml`, `crates/kali_cli/tests/cases/array/growable_runtime_arrays_traps.toml`

**Interfaces:**
- Consumes: `ReprTable::{is_growable_array_binding, growable_return, is_growable_local_only, array_element, is_array_return_callee_shadowed}`; `runtime_array_index_out_of_bounds_message`, `growable_length_write_message`, `growable_binding_subject` (Task 1).
- Produces:
  - synthetic `__growable_elem_addr(arr: i64, idx: i64, msg: i64) -> i64`, index via `FunctionEmitter::growable_elem_addr_fn_index(&self) -> u32`;
  - `FunctionEmitter::growable_value_elem(&self, id: LirNodeId) -> Option<kali_common::Repr>` — `Some(element repr)` for a growable binding, a direct call to a growable-returning function, or a `.slice(…)` of either (Tasks 10-11 use it everywhere);
  - `emit_growable_alloc(&mut self, function: &mut Function, seed_len: usize, cap: usize, global: bool) -> EmittedValue`;
  - `emit_growable_push(&mut self, function, handle: GrowableHandle, value: LirNodeId, elem: kali_common::Repr, global: bool) -> EmittedValue`;
  - `emit_growable_element_address(&mut self, function, handle: LirNodeId, index: LirNodeId)` — leaves an i32 slot address (bounds-checked);
  - `emit_growable_index_write(&mut self, function, base: LirNodeId, index: LirNodeId, value: LirNodeId)` — leaves the stored value.

- [ ] **Step 1: Write the failing unit tests**

Create `crates/kali_codegen/src/emit/growable_tests.rs`:

```rust
//! Growable-runtime-arrays spec A-15: the hand-emitted growable synthetics
//! validate, and the growable emitters produce valid wasm for an f64 array.
//! (Codegen unit tests run without repr inference, so the table entries
//! inference would write are set by hand.)

use crate::test_support::{compile_and_measure, parse_and_lower_lir, sample_program};
use crate::{lower_lir_to_wasm, CodegenCtx, TargetConfig};

pub(crate) fn exported_function_names(bytes: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(bytes) {
        if let Ok(wasmparser::Payload::ExportSection(reader)) = payload {
            for export in reader {
                let export = export.expect("export entry");
                if export.kind == wasmparser::ExternalKind::Func {
                    names.push(export.name.to_string());
                }
            }
        }
    }
    names
}

pub(crate) fn ctx_with_growable(func: &str, name: &str, elem: kali_common::Repr) -> CodegenCtx {
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.repr_table.set_growable_array_binding(func, name);
    ctx.repr_table.set_array_binding(func, name);
    if elem != kali_common::Repr::I64 {
        ctx.repr_table.set_array_element(func, name, elem);
    }
    ctx
}

#[test]
fn every_module_carries_the_growable_bounds_guard_and_validates() {
    let (bytes, _) = compile_and_measure(&sample_program());
    let names = exported_function_names(&bytes);
    assert!(
        names.iter().any(|n| n == "__growable_elem_addr"),
        "{names:?}"
    );
}

#[test]
fn an_f64_growable_array_push_read_and_write_lower_to_valid_wasm() {
    let mut ctx = ctx_with_growable("main", "a", kali_common::Repr::F64);
    let program = parse_and_lower_lir(
        "function main() { const a = []; a.push(1.5); a.push(2); let i = 1; a[i] = 0.25; console.log(a[0], a[i], a.length); } main();",
    );
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    wasmparser::Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}
```

At the end of `emit/growable.rs` add `#[cfg(test)] #[path = "growable_tests.rs"] mod growable_tests;`.

- [ ] **Step 2: Run them to verify they fail**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_codegen growable_tests -- --test-threads=6`
Expected: FAIL — `__growable_elem_addr` missing; the f64 test reports "pushing a floating-point value onto a growable array is unavailable".

- [ ] **Step 3: The bounds-guard synthetic**

In `lower.rs`: append `"__growable_elem_addr",` to `SYNTHETIC_FUNCTIONS` after `"__array_elem_addr",`; push a plan right after the `__array_elem_addr` plan:

```rust
    // Growable-runtime-arrays spec §3.5: the growable twin of
    // `__array_elem_addr` — `(arr: i64, idx: i64, msg: i64) -> i64` over the
    // header layout. Present in every module; body hand-emitted by
    // `emit_growable_elem_addr_body`.
    all_functions.push(FunctionPlan {
        name: "__growable_elem_addr".to_string(),
        params: vec!["arr".to_string(), "idx".to_string(), "msg".to_string()],
        locals: Vec::new(),
        body: lir.root,
        result: true,
        is_entry: false,
        flavor: None,
    });
```

In the signature `match` (`:1274`) change `function.name == "__array_elem_addr"` to `matches!(function.name.as_str(), "__array_elem_addr" | "__growable_elem_addr")`. In the local-declaration chain (the `else if` ladder around `:1620-1640`), add before the `__clone_shape_` arm:

```rust
        } else if function.name == "__growable_elem_addr" {
            // `emit_growable_elem_addr_body`: 1 i64 — `hdr` (local 3).
            local_decls.push((1, ValType::I64));
```

In the body dispatch (`:1769`) add `"__growable_elem_addr" => emit_growable_elem_addr_body(&mut body),`. After `emit_array_elem_addr_body`:

```rust
/// `__growable_elem_addr(arr, idx, msg) -> i64` (growable-runtime-arrays spec
/// §3.5): the bounds guard for a growable array. Locals: 0 = arr (tagged
/// handle), 1 = idx, 2 = msg, 3 = hdr. `idx >=u len` — a negative `idx`
/// included — hands `msg` to `console.error` and traps; otherwise returns
/// `data_ptr + idx * 8` (the caller loads/stores at `offset` 0). No
/// `i64.eqz` (see `emit_streq_body`).
fn emit_growable_elem_addr_body(func: &mut Function) {
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(crate::emit::growable::GROWABLE_HANDLE_MASK));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg { offset: 0, align: 3, memory_index: 0 }));
    func.instruction(&Instruction::I64GeU);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::Call(crate::CONSOLE_ERROR_IMPORT_INDEX));
    func.instruction(&Instruction::Unreachable);
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(MemArg { offset: 16, align: 3, memory_index: 0 }));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::I64Const(8));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I64Add);
    // NO trailing End — the dispatch loop appends it (same as every synthetic).
}
```

In `emitter.rs`, after `array_elem_addr_fn_index`:

```rust
    /// Wasm function index of `__growable_elem_addr(arr, idx, msg) -> i64`
    /// (growable-runtime-arrays spec §3.5): every growable element read and
    /// write routes its address through it.
    pub(crate) fn growable_elem_addr_fn_index(&self) -> u32 {
        self.functions["__growable_elem_addr"]
    }
```

In `crates/kali_cli/tests/runtime_smoke.rs`, append `"__growable_elem_addr",` to the test-side `SYNTHETIC_FUNCTIONS` mirror (`:807-826`) and name it in the comment above.

- [ ] **Step 4: Recognize growable values; keep growable params off the plain lane**

In `emitter.rs:673-678`, change the parameter loop's condition to `repr_table.is_array_binding(function_name, name) && !repr_table.is_growable_array_binding(function_name, name)`.

Add to `impl FunctionEmitter` in `emit/growable.rs`:

```rust
    /// Growable-runtime-arrays spec §3.5, A-6: the element repr of a growable
    /// value — a growable binding of this function, a direct call to a
    /// growable-returning function (unshadowed: inference's own fact, plus a
    /// same-named local), or a `.slice(…)` of either. `None` otherwise. The
    /// codegen twin of the resolver's `growable_receiver_elem`.
    pub(crate) fn growable_value_elem(&self, id: LirNodeId) -> Option<kali_common::Repr> {
        let id = self.unwrap_transparent(id);
        if let Some(name) = self.bare_identifier_name(id) {
            return self
                .is_growable_array(&name)
                .then(|| self.array_elem_repr(&name));
        }
        let node = self.node(id);
        if node.kind != LirNodeKind::Call {
            return None;
        }
        let callee = *node.children.first()?;
        if let Some(name) = self.bare_identifier_name(callee) {
            if self.locals.contains_key(&name)
                || self
                    .repr_table
                    .is_array_return_callee_shadowed(&self.function_name, &name)
            {
                return None;
            }
            return self.repr_table.growable_return(&name);
        }
        let member = self.node(self.resolve_transparent_callable_node(callee)?);
        if member.text.as_deref() == Some("slice") && member.children.len() == 1 {
            return self.growable_value_elem(member.children[0]);
        }
        None
    }
```

- [ ] **Step 5: Allocation chooses the arena or the global heap**

Change `emit_growable_alloc` to take `global: bool` and pick `let alloc = if global { self.alloc_global_fn_index() } else { self.alloc_callee_index() };`. Pass `false` from `emit_growable_field_value` (`growable.rs:172`) and `emit/url.rs:217` (both lanes unchanged).

Replace the growable declarator lane in `emit/control_flow.rs` (`:1747-1801`, `if let Some(name) = declarator.text.clone() { if self.is_growable_array(&name) { … } }`) with:

```rust
                        if let Some(name) = declarator.text.clone() {
                            if self.is_growable_array(&name) {
                                let Some(index) = self.locals.get(&name).copied() else {
                                    self.diagnostics.push(Diagnostic::error(
                                        e5::FEATURE_UNAVAILABLE as u32,
                                        format!("growable array `{name}` has no local slot"),
                                    ));
                                    function.instruction(&Instruction::Unreachable);
                                    continue;
                                };
                                // Growable-runtime-arrays spec §3.1: a name, a
                                // call or a `slice` already yields the tagged
                                // handle — it is an alias, a call result or a
                                // copy, never a fresh literal. Checked BEFORE
                                // `resolve_literal_aggregate`, which would see
                                // through a name to its declarator literal and
                                // allocate a second array (breaking the alias).
                                let init_id = self.unwrap_transparent(init);
                                let init_is_value = self.bare_identifier_name(init_id).is_some()
                                    || self.node(init_id).kind == LirNodeKind::Call;
                                let aggregate = (!init_is_value)
                                    .then(|| self.resolve_literal_aggregate(init))
                                    .flatten()
                                    .map(|id| self.node(id).clone())
                                    .filter(|node| self.is_array_literal(node));
                                let Some(aggregate) = aggregate else {
                                    let value = self.emit_node(function, init, true);
                                    if !value.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    function.instruction(&Instruction::LocalSet(index));
                                    continue;
                                };
                                // Spec §3.3, A-16: only an array that never
                                // leaves this function may use the arena.
                                let global = !self
                                    .repr_table
                                    .is_growable_local_only(&self.function_name, &name);
                                let elem = self.array_elem_repr(&name);
                                let seed_len = aggregate.children.len();
                                let cap = seed_len.max(crate::emit::growable::GROWABLE_INITIAL_CAP);
                                let allocated = self.emit_growable_alloc(function, seed_len, cap, global);
                                if !allocated.produced {
                                    function.instruction(&Instruction::I64Const(0));
                                }
                                function.instruction(&Instruction::LocalSet(index));
                                for (i, child) in aggregate.children.iter().copied().enumerate() {
                                    function.instruction(&Instruction::LocalGet(index));
                                    function.instruction(&Instruction::I64Const(
                                        crate::emit::growable::GROWABLE_HANDLE_MASK,
                                    ));
                                    function.instruction(&Instruction::I64And);
                                    function.instruction(&Instruction::I32WrapI64);
                                    function.instruction(&Instruction::I64Load(MemArg {
                                        offset: 16,
                                        align: 3,
                                        memory_index: 0,
                                    }));
                                    function.instruction(&Instruction::I32WrapI64);
                                    let produced = self.emit_node(function, child, true);
                                    if !produced.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    // Spec §3.4: an f64 element is stored
                                    // bit-reinterpreted in its 8-byte slot.
                                    if elem == kali_common::Repr::F64 {
                                        if !produced.produced || !self.is_float_valued(child) {
                                            function.instruction(&Instruction::F64ConvertI64S);
                                        }
                                        function.instruction(&Instruction::I64ReinterpretF64);
                                    }
                                    function.instruction(&Instruction::I64Store(MemArg {
                                        offset: (i * 8) as u64,
                                        align: 3,
                                        memory_index: 0,
                                    }));
                                }
                                continue;
                            }
                        }
```

- [ ] **Step 6: `push` takes several values, f64 values, and grows globally when escaping**

In `emit_growable_push`, add parameters `elem: kali_common::Repr, global: bool`. Replace the leading float refusal with:

```rust
        let value_is_float = self.is_float_valued(value);
        // Belt: inference makes any array holding a float an f64 array, so a
        // float value never reaches an integer or string array.
        if value_is_float && elem != kali_common::Repr::F64 {
            return self.deny_e5506(
                function,
                "pushing a floating-point value onto a growable array of integers or strings is unavailable in the current phase",
            );
        }
```

choose the reallocation allocator with `let alloc = if global || self.arena_frames.iter().any(|frame| frame.loop_frame_index.is_some()) { self.alloc_global_fn_index() } else { self.alloc_callee_index() };`, and after `let produced = self.emit_node(function, value, true); if !produced.produced { function.instruction(&Instruction::I64Const(0)); }` insert:

```rust
        // Spec §3.4: an f64 element is stored bit-reinterpreted.
        if elem == kali_common::Repr::F64 {
            if !produced.produced || !value_is_float {
                function.instruction(&Instruction::F64ConvertI64S);
            }
            function.instruction(&Instruction::I64ReinterpretF64);
        }
```

before `function.instruction(&Instruction::LocalSet(value_scratch));`. Everything after is unchanged.

In `emit_growable_push_call`, replace the arity refusal (`:621-633`) and the two `emit_growable_push` calls: zero arguments yield the length without a store, several append in order and the call's value is the last new length:

```rust
        if args.is_empty() {
            let handle = match &receiver {
                GrowablePushReceiver::Named(name) => self.alloc_scratch_node(
                    LirNodeKind::Value,
                    Some(name.clone()),
                    vec![],
                ),
                GrowablePushReceiver::Field(id) => *id,
            };
            return self.emit_growable_length(function, handle);
        }
        if args.iter().any(|arg| self.is_crypto_random_result_value(*arg)) {
            return self.deny_e5506(function, Self::CRYPTO_RANDOM_RESULT_STORE_DENY);
        }
```

and in the `Named` arm, after `handle_local` is found:

```rust
                let elem = self.array_elem_repr(&base_name);
                let global = !self
                    .repr_table
                    .is_growable_local_only(&self.function_name, &base_name);
                let mut last = EmittedValue { produced: false, shape: ValueShape::Unknown };
                for (i, arg) in args.iter().copied().enumerate() {
                    if i > 0 {
                        function.instruction(&Instruction::Drop);
                    }
                    last = self.emit_growable_push(function, GrowableHandle::Local(handle_local), arg, elem, global);
                }
                last
```

In the `Field` arm, after the field-key guard, replace the single `emit_growable_push` call with (the field lane stays i64 and arena-allocated):

```rust
                let mut last = EmittedValue { produced: false, shape: ValueShape::Unknown };
                for (i, arg) in args.iter().copied().enumerate() {
                    if i > 0 {
                        function.instruction(&Instruction::Drop);
                    }
                    last = self.emit_growable_push(
                        function,
                        GrowableHandle::Field(receiver_id),
                        arg,
                        kali_common::Repr::I64,
                        false,
                    );
                }
                last
```

- [ ] **Step 7: Bounds-checked index read and write**

Add to `emit/growable.rs`:

```rust
    /// Growable-runtime-arrays spec §3.5: push the i32 address of element
    /// `index` of the growable value `handle`, bounds-checked by
    /// `__growable_elem_addr` (`index < 0` or `index >= length` traps with the
    /// kali bounds message). Load/store at `offset` 0.
    pub(crate) fn emit_growable_element_address(
        &mut self,
        function: &mut Function,
        handle: LirNodeId,
        index: LirNodeId,
    ) {
        let base = self.emit_growable_receiver_handle(function, handle);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        let index_value = self.emit_numeric_operand(function, index);
        if !index_value.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        let (offset, len) = self
            .strings
            .intern(kali_common::runtime_array_index_out_of_bounds_message());
        function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
        function.instruction(&Instruction::Call(self.growable_elem_addr_fn_index()));
        function.instruction(&Instruction::I32WrapI64);
    }

    /// `base[index] = value` on a growable binding; leaves the stored value
    /// (the assignment expression's result).
    pub(crate) fn emit_growable_index_write(
        &mut self,
        function: &mut Function,
        base: LirNodeId,
        index: LirNodeId,
        value: LirNodeId,
    ) {
        let elem = self.growable_value_elem(base).unwrap_or(kali_common::Repr::I64);
        let scratch = self.locals.len() as u32;
        self.emit_growable_element_address(function, base, index);
        let rhs = self.emit_node(function, value, true);
        if !rhs.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        if elem == kali_common::Repr::F64 {
            if !rhs.produced || !self.is_float_valued(value) {
                function.instruction(&Instruction::F64ConvertI64S);
            }
            function.instruction(&Instruction::I64ReinterpretF64);
        }
        function.instruction(&Instruction::LocalTee(scratch));
        function.instruction(&Instruction::I64Store(MemArg { offset: 0, align: 3, memory_index: 0 }));
        function.instruction(&Instruction::LocalGet(scratch));
        if elem == kali_common::Repr::F64 {
            function.instruction(&Instruction::F64ReinterpretI64);
        }
    }
```

Rewrite the body of `emit_growable_index_read` after its float-index refusal (and update its doc comment: the read is now bounds-checked):

```rust
        // Field receivers are i64 (the object-field lane); named and call
        // receivers carry their element repr.
        let elem = self.growable_value_elem(handle).unwrap_or(kali_common::Repr::I64);
        self.emit_growable_element_address(function, handle, index);
        function.instruction(&Instruction::I64Load(MemArg { offset: 0, align: 3, memory_index: 0 }));
        if elem == kali_common::Repr::F64 {
            function.instruction(&Instruction::F64ReinterpretI64);
        }
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
```

In `emit/literal.rs`, inside `if let Some(base_name) = self.assignment_target_name(node, base_id) {` (`:684`), insert before `if self.array_bindings.contains(&base_name) {`:

```rust
                        // Growable-runtime-arrays spec §3.5: a bounds-checked
                        // store into the header layout. `.length` writes are
                        // refused by inference (M7); this is the belt.
                        if self.is_growable_array(&base_name) {
                            if matches!(&index, ArrayWriteIndex::Text(text) if text == "length") {
                                let message = kali_common::growable_length_write_message(
                                    &kali_common::growable_binding_subject(&base_name, &self.function_name),
                                );
                                let _ = self.deny_e5506(function, &message);
                                return true;
                            }
                            let index_id = match index {
                                ArrayWriteIndex::Text(text) => {
                                    self.alloc_scratch_node(LirNodeKind::Value, Some(text), vec![])
                                }
                                ArrayWriteIndex::Node(id) => id,
                            };
                            self.emit_growable_index_write(function, base_id, index_id, right);
                            return true;
                        }
```

In `emit/control_flow.rs`'s `.length` member arm, immediately before the array-return `.length` arm (`if self.array_return_call_elem(base_id).is_some() {`, `:2900`), insert:

```rust
                    // Growable-runtime-arrays spec §3.5, A-6: `.length` of
                    // any growable value — a binding, a call to a growable-
                    // returning function, or a `slice` (`a.slice(4, 1).length`).
                    // The call is emitted once as the base.
                    if self.growable_value_elem(base_id).is_some() {
                        return self.emit_growable_length(function, base_id);
                    }
```

The named arm further down (`if self.is_growable_array(&base_name)`) becomes unreachable for growable names; delete it.

- [ ] **Step 8: A growable argument is not a fold-lane literal; reserve the push scratch for parameters**

In `emit/call.rs` (`:3727-3734`), add `let arg_is_growable = self.growable_value_elem(*arg).is_some();` and change `let fold_lane_array = !arg_is_allocation && …` to `let fold_lane_array = !arg_is_allocation && !arg_is_growable && …` — a seeded growable literal (`const xs = [1]; xs.push(2); f(xs)`) is passed as its handle.

In `lower.rs`, add beside `body_contains_field_push`:

```rust
/// True iff any `.push` member (any receiver) is reachable from `body_id`
/// without descending into nested functions. Growable-runtime-arrays: a
/// function whose only growable array is a PARAMETER pushes through
/// `emit_growable_push`, which needs the growable scratch, and parameters
/// are not in the `locals` list the binding trigger scans. Over-reserving
/// one unused i64 local is harmless.
fn body_contains_push_member(nodes: &[LirNode], body_id: LirNodeId) -> bool {
    fn walk(nodes: &[LirNode], id: LirNodeId) -> bool {
        let Some(node) = nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("push")
            && node.children.len() == 1
        {
            return true;
        }
        node.children
            .iter()
            .any(|child| !is_function_like(nodes, *child) && walk(nodes, *child))
    }
    walk(nodes, body_id)
}
```

and replace `|| body_contains_field_push(nodes, body_id)` in the growable-scratch condition (`:3739`) with `|| body_contains_push_member(nodes, body_id)` (a superset). Delete `body_contains_field_push` if nothing else calls it.

- [ ] **Step 8b: Console reads of growable arrays (spec A-9)**

In `emit/call.rs`, the guard at `:1060-1088` keeps firing only for growable FIELD reads (the object-field lane is unchanged; `soundness/structured_clone.toml:280` pins it): rename `subtree_mentions_growable` (`growable.rs:791`) to `subtree_mentions_growable_field`, delete its named-binding arm (the first `if node.text…is_growable_array(text)` block), update its doc and the call site, and rewrite the call-site comment: "Growable-runtime-arrays spec A-9: named growable arrays print through the multi-argument lane below; only a growable object FIELD read keeps this pre-existing refusal." In `emit_console_argument` and `emit_console_argument_as_string`, change `if self.is_runtime_array_value(id) {` to `if self.is_runtime_array_value(id) || self.growable_value_elem(id).is_some() {` (inference refuses this first, M9; this is the belt). The multi-argument cases and unit tests of Tasks 9-12 print growable reads, so this lands here.

- [ ] **Step 9: Run the unit tests**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_codegen -- --test-threads=6`
Expected: PASS, including both new tests.

- [ ] **Step 10: Write the layout and trap cases**

Every program below was run under node v26.10.0 (`env -u FORCE_COLOR node FILE.js`) on 2026-10-07; the expected stdouts are node's. Create `crates/kali_cli/tests/cases/array/growable_layout.toml`:

```toml
# Growable runtime arrays: layout, memory and element access
# (docs/superpowers/specs/2026-10-07-growable-runtime-arrays-design.md §3.1,
# §3.3-§3.5). Every expected stdout is node v26.10.0's for the same file,
# measured 2026-10-07 with `env -u FORCE_COLOR node FILE.js`.

[source]
"alias_module.js" = """
const a = [];
a.push(5);
const b = a;
b.push(6);
console.log(a.length, a[1], b.length);
"""
"alias_function.js" = """
function main() { const a = []; a.push(5); const b = a; b.push(6); console.log(a.length, a[1], b.length); }
main();
"""
"multi_push.js" = """
function main() { const a = []; const n = a.push(1, 2, 3); console.log(n, a.length, a.join("")); }
main();
"""
"seeded.js" = """
function main() { const a = [1, 2]; a.push(3); console.log(a.length, a.join("+")); }
main();
"""
"three_index.js" = """
function make() { const xs = []; xs.push(1); return xs; }
function addTwo(a) { a.push(2); a.push(3); }
function total(a) { let s = 0; for (let i = 0; i < a.length; i++) s += a[i]; return s; }
function main() { const xs = make(); addTwo(xs); console.log(xs.length, total(xs)); }
main();
"""
"two_builds.js" = """
function build(n) { const out = []; let w = "a"; for (let i = 0; i < n; i++) { out.push(w); w = w + "b"; } return out; }
const a = build(3);
const b = build(2);
console.log(a[0], a[2], b[1], a.length + b.length);
"""
"f64_rw.js" = """
function main() { const a = []; a.push(1.5); a.push(2); a[1] = a[1] / 4; console.log(a[0], a[1], a.length); }
main();
"""

[[case]]
name = "alias_push_is_seen_through_the_original_module_scope"
rationale = "Review Focus 2: `b` aliases `a`'s header, so `b.push` grows `a` (spec §3.1 alias edge, A-16 global allocation)."
args = ["run", "alias_module.js"]
exit = "success"
stdout = "2 6 2\n"

[[case]]
name = "alias_push_is_seen_through_the_original"
args = ["run", "alias_function.js"]
exit = "success"
stdout = "2 6 2\n"

[[case]]
name = "push_appends_every_argument_and_returns_the_new_length"
args = ["run", "multi_push.js"]
exit = "success"
stdout = "3 3 123\n"

[[case]]
name = "a_seeded_literal_keeps_its_seeds_when_pushed"
args = ["run", "seeded.js"]
exit = "success"
stdout = "3 1+2+3\n"

[[case]]
name = "an_array_built_in_one_function_grown_in_a_second_and_read_in_a_third"
rationale = "Spec §5.2: the return edge, the argument edge into a parameter that pushes, and a third function reading through its parameter."
args = ["run", "three_index.js"]
exit = "success"
stdout = "3 6\n"

[[case]]
name = "returned_string_arrays_survive_later_allocations"
rationale = "Spec §3.3: an escaping growable array and the strings in it outlive the creating call; the second `build` must not overwrite the first array's data."
args = ["run", "two_builds.js"]
exit = "success"
stdout = "a abb ab 5\n"

[[case]]
name = "f64_elements_are_stored_and_read_back_exactly"
rationale = "Spec §3.4: f64 elements are bit-reinterpreted into the 8-byte slots; an integer pushed into an f64 array is converted."
args = ["run", "f64_rw.js"]
exit = "success"
stdout = "1.5 0.5 2\n"
```

Create `crates/kali_cli/tests/cases/array/growable_runtime_arrays_traps.toml`:

```toml
# Growable runtime arrays: the bounds and empty-pop traps (spec §3.6). node
# v26.10.0 prints `undefined` for the reads and the pop, and `2` for the write
# at `length` (it appends); kali traps instead, printing the kali message and
# exiting 1 (E4000 follows). A trap is a run-time event, so `check` accepts
# each program.

[constants]
OOB = "kali: array index out of bounds"

[source]
"oob_read.js" = """
function main() { const a = []; a.push(1); a.push(2); console.log(a[5]); }
main();
"""
"oob_negative.js" = """
function main() { const a = []; a.push(1); let i = -1; console.log(a[i]); }
main();
"""
"oob_write.js" = """
function main() { const a = []; a.push(1); a[a.length] = 2; console.log(a.length); }
main();
"""

[[case]]
name = "a_read_past_the_end_traps"
rationale = "node v26.10.0 prints `undefined`. At the baseline Stage 4's unchecked read printed a stale slot (`51` measured on a similar program)."
args = ["run", "oob_read.js"]
exit = "failure"
stderr_contains = ["${OOB}", "E4000"]

[[case]]
name = "a_negative_runtime_index_traps"
args = ["run", "oob_negative.js"]
exit = "failure"
stderr_contains = ["${OOB}", "E4000"]

[[case]]
name = "a_write_at_length_traps_where_node_appends"
rationale = "Spec §3.5: `i == length` is out of bounds for a write too."
args = ["run", "oob_write.js"]
exit = "failure"
stderr_contains = ["${OOB}", "E4000"]

[[case]]
name = "check_accepts_the_trapping_programs"
args = ["check", "oob_read.js", "oob_negative.js", "oob_write.js"]
exit = "success"
```

Task 10 adds the empty-`pop` trap to this file.

- [ ] **Step 11: Confirm node agrees, then run the cases**

Create the reusable checker `$S/node_check.py` (Tasks 10-12 run it too):

```python
# Usage: python3 -I node_check.py CASEFILE.toml — runs every `run` case's
# program under node (FORCE_COLOR unset) in the current directory and
# compares stdout with the case's `stdout`.
import os, subprocess, sys, tomllib
doc = tomllib.load(open(sys.argv[1], "rb"))
env = {k: v for k, v in os.environ.items() if k != "FORCE_COLOR"}
for case in doc["case"]:
    if case["args"][0] != "run" or "stdout" not in case:
        continue
    program = case["args"][-1]
    open(program, "w").write(doc["source"][program])
    out = subprocess.run(["node", program], capture_output=True, text=True, env=env).stdout
    print("OK " if out == case["stdout"] else "DIFF", case["name"], repr(out))
```

```bash
mkdir -p "$S/nodecheck" && cd "$S/nodecheck" && python3 -I "$S/node_check.py" /workspace/.worktrees/growable-runtime-arrays/crates/kali_cli/tests/cases/array/growable_layout.toml
```

Expected: every line `OK`. Then (detached if the binary needs rebuilding — the `cases` target builds `kali` first):

```bash
cd /workspace/.worktrees/growable-runtime-arrays
CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_cli --test cases -- array/growable_layout array/growable_runtime_arrays_traps --test-threads=6
```

Expected: PASS. If `alias_*` prints `1 …`, the alias declarator allocated a second array: check `init_is_value` in Step 5. If `two_builds` prints garbage for `a`, `build`'s strings or blocks came from a reset arena: confirm `is_growable_local_only("build", "out")` is false and that the pushed strings are allocated globally (record what you find in Task 13's followups).

- [ ] **Step 12: Commit**

```bash
git add -A crates/kali_codegen crates/kali_cli/tests
git commit -m "feat(growable-runtime-arrays): codegen layout, global allocation for escaping arrays, bounds-checked element access, console reads (spec §3.3-§3.5, A-9)"
```

---
### Task 10: Codegen — `pop`, `indexOf`, `includes`, `slice` and float `join`

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (four synthetics registered exactly like Task 9's; `emit_join_growable_body` `:9189` and `emit_growable_join_element_handle`; the growable join locals branch `:1626-1632`)
- Modify: `crates/kali_codegen/src/emitter.rs` (four accessors after `growable_elem_addr_fn_index`)
- Modify: `crates/kali_codegen/src/emit/growable.rs` (`GrowableMethod`, `growable_method_call_parts`, `emit_growable_pop`, `emit_growable_search`, `emit_growable_slice`)
- Modify: `crates/kali_codegen/src/emit/call.rs` (dispatch after the push dispatch `:1707-1709`; `runtime_join_call_parts` `:6077-6116`; `emit_runtime_join` join selection `:6250-6286`)
- Modify: `crates/kali_codegen/src/intrinsics/array.rs` (`resolve_static_array_join_receiver` `:892`)
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`is_string_valued` `:1102`, `is_float_valued`'s `Call` arm `:1716-1752`, `is_runtime_concat_string` `:1397`)
- Modify: `crates/kali_codegen/src/emit/growable_tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs`
- Create: `crates/kali_cli/tests/cases/array/growable_methods.toml`; Modify: `crates/kali_cli/tests/cases/array/growable_runtime_arrays_traps.toml`

**Interfaces:**
- Consumes: `growable_value_elem`, `emit_growable_receiver_handle` (Task 9); `growable_pop_empty_message` (Task 1).
- Produces:
  - synthetics `__growable_pop(arr, msg) -> i64` (raw slot bits), `__growable_find(arr, needle, mode) -> i64` (index or -1; mode 0 i64, 1 f64 `===`, 2 f64 SameValueZero, 3 string content), `__growable_slice(arr, start, end) -> i64` (a fresh growable handle, global heap), `__join_growable_f64(arr, sep) -> i64`;
  - `enum GrowableMethod { Pop, IndexOf, Includes, Slice }` and `FunctionEmitter::growable_method_call_parts(&self, node: &LirNode) -> Option<(GrowableMethod, LirNodeId, Vec<LirNodeId>)>` (receiver must be a growable value);
  - `emit_growable_pop(&mut self, function, receiver: LirNodeId) -> EmittedValue`, `emit_growable_search(&mut self, function, receiver: LirNodeId, needle: Option<LirNodeId>, includes: bool) -> EmittedValue`, `emit_growable_slice(&mut self, function, receiver: LirNodeId, args: &[LirNodeId]) -> EmittedValue`.

- [ ] **Step 1: Write the failing unit test**

Append to `crates/kali_codegen/src/emit/growable_tests.rs`:

```rust
#[test]
fn every_module_carries_the_growable_method_synthetics_and_validates() {
    let (bytes, _) = compile_and_measure(&sample_program());
    let names = exported_function_names(&bytes);
    for name in ["__growable_pop", "__growable_find", "__growable_slice", "__join_growable_f64"] {
        assert!(names.iter().any(|n| n == name), "{name} missing from {names:?}");
    }
}

#[test]
fn f64_pop_search_slice_and_join_lower_to_valid_wasm() {
    let mut ctx = ctx_with_growable("main", "a", kali_common::Repr::F64);
    ctx.repr_table.set_growable_array_binding("main", "t");
    ctx.repr_table.set_array_binding("main", "t");
    ctx.repr_table.set_array_element("main", "t", kali_common::Repr::F64);
    let program = parse_and_lower_lir(
        "function main() { const a = []; a.push(1.5); a.push(0.25); const t = a.slice(0, 1); console.log(a.indexOf(0.25), a.includes(1.5), a.join(\",\"), t.length, a.pop()); } main();",
    );
    let result = lower_lir_to_wasm(&mut ctx, &program);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    wasmparser::Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_codegen growable_tests -- --test-threads=6`
Expected: FAIL — the four synthetics are missing.

- [ ] **Step 3: The four synthetics**

In `lower.rs`, append to `SYNTHETIC_FUNCTIONS` after `"__growable_elem_addr",`:

```rust
    "__growable_pop",
    "__growable_find",
    "__growable_slice",
    "__join_growable_f64",
```

Push four plans right after the `__growable_elem_addr` plan:

```rust
    // Growable-runtime-arrays spec §3.5: `pop`, `indexOf`/`includes`,
    // `slice` and f64 `join` over the header layout. Present in every
    // module; bodies hand-emitted below.
    for (name, params) in [
        ("__growable_pop", &["arr", "msg"][..]),
        ("__growable_find", &["arr", "needle", "mode"][..]),
        ("__growable_slice", &["arr", "start", "end"][..]),
        ("__join_growable_f64", &["arr", "sep"][..]),
    ] {
        all_functions.push(FunctionPlan {
            name: name.to_string(),
            params: params.iter().map(|p| p.to_string()).collect(),
            locals: Vec::new(),
            body: lir.root,
            result: true,
            is_entry: false,
            flavor: None,
        });
    }
```

In the signature `match`, the join arm becomes `"__join" | "__join_arena" | "__join_growable_i64" | "__join_growable_str" | "__join_growable_f64" | "__growable_pop"` (`(i64, i64) -> i64`) and Task 9's arm becomes `"__array_elem_addr" | "__growable_elem_addr" | "__growable_find" | "__growable_slice"` (`(i64, i64, i64) -> i64`). In the local-declaration ladder, add `"__join_growable_f64"` to the `"__join_growable_i64" | "__join_growable_str"` arm (7 i64) and, beside Task 9's `__growable_elem_addr` arm:

```rust
        } else if function.name == "__growable_pop" {
            // `emit_growable_pop_body`: 2 i64 — `hdr`, `len` (locals 2-3).
            local_decls.push((2, ValType::I64));
        } else if function.name == "__growable_find" {
            // `emit_growable_find_body`: 4 i64 — `len`, `i`, `data`, `elem` (locals 3-6).
            local_decls.push((4, ValType::I64));
        } else if function.name == "__growable_slice" {
            // `emit_growable_slice_body`: 7 i64 — `len`, `s`, `e`, `count`, `hdr`, `cap`, `data` (locals 3-9).
            local_decls.push((7, ValType::I64));
```

Body dispatch:

```rust
                "__growable_pop" => emit_growable_pop_body(&mut body),
                "__growable_find" => {
                    emit_growable_find_body(&mut body, function_name_to_index["__streq"])
                }
                "__growable_slice" => emit_growable_slice_body(&mut body, alloc_global_index),
                "__join_growable_f64" => {
                    emit_join_growable_body(&mut body, alloc_global_index, GrowableJoinRender::Float)
                }
```

and change the two existing join arms to pass `GrowableJoinRender::Int` (`_i64`) and `GrowableJoinRender::Raw` (`_str`).

Change `emit_join_growable_body`'s last parameter from `render_int: bool` to `render: GrowableJoinRender` and `emit_growable_join_element_handle`'s likewise; replace its trailing `if render_int { … }` with:

```rust
    match render {
        GrowableJoinRender::Raw => {}
        GrowableJoinRender::Int => {
            func.instruction(&Instruction::Call(crate::INT_TO_STRING_IMPORT_INDEX));
        }
        // Spec §3.5, A-12: the slot holds f64 bits; `float_to_string` is JS
        // `String(number)` (`-0` renders `0`, as `join` does).
        GrowableJoinRender::Float => {
            func.instruction(&Instruction::F64ReinterpretI64);
            func.instruction(&Instruction::Call(crate::FLOAT_TO_STRING_IMPORT_INDEX));
        }
    }
```

with, beside them:

```rust
/// How `emit_join_growable_body` renders one element slot.
#[derive(Clone, Copy)]
enum GrowableJoinRender {
    /// The slot is already a string handle.
    Raw,
    /// The slot is an i64 number.
    Int,
    /// The slot holds f64 bits.
    Float,
}

fn growable_slot(offset: u64) -> MemArg {
    MemArg { offset, align: 3, memory_index: 0 }
}

/// `__growable_pop(arr, msg) -> i64` (growable-runtime-arrays spec §3.5):
/// removes and returns the last element's raw slot bits; an empty array
/// hands `msg` to `console.error` and traps. Locals: 0 arr, 1 msg, 2 hdr,
/// 3 len.
fn emit_growable_pop_body(func: &mut Function) {
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(crate::emit::growable::GROWABLE_HANDLE_MASK));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::LocalSet(2));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(growable_slot(0)));
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(1));
    func.instruction(&Instruction::Call(crate::CONSOLE_ERROR_IMPORT_INDEX));
    func.instruction(&Instruction::Unreachable);
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Sub);
    func.instruction(&Instruction::LocalSet(3));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Store(growable_slot(0)));
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(growable_slot(16)));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64Const(8));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(growable_slot(0)));
}

/// `__growable_find(arr, needle, mode) -> i64` (spec §3.5): the first index
/// whose element matches `needle`, else -1. Mode 0: i64 `==`. Mode 1: f64
/// `===` (NaN is never found, `-0 === 0`). Mode 2: f64 SameValueZero (also
/// NaN finds NaN). Mode 3: string content equality through `__streq`.
/// Locals: 0 arr, 1 needle, 2 mode, 3 len, 4 i, 5 data, 6 elem.
fn emit_growable_find_body(func: &mut Function, streq_index: u32) {
    let mask = crate::emit::growable::GROWABLE_HANDLE_MASK;
    for (offset, local) in [(0u64, 3u32), (16, 5)] {
        func.instruction(&Instruction::LocalGet(0));
        func.instruction(&Instruction::I64Const(mask));
        func.instruction(&Instruction::I64And);
        func.instruction(&Instruction::I32WrapI64);
        func.instruction(&Instruction::I64Load(growable_slot(offset)));
        func.instruction(&Instruction::LocalSet(local));
    }
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::Block(BlockType::Empty));
    func.instruction(&Instruction::Loop(BlockType::Empty));
    // if i >= len break
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64GeS);
    func.instruction(&Instruction::BrIf(1));
    // elem = data[i]
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(3));
    func.instruction(&Instruction::I64Shl);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(growable_slot(0)));
    func.instruction(&Instruction::LocalSet(6));
    // match: i32
    func.instruction(&Instruction::LocalGet(2));
    func.instruction(&Instruction::I64Const(3));
    func.instruction(&Instruction::I64Eq);
    func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
    {
        func.instruction(&Instruction::LocalGet(6));
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::Call(streq_index));
        func.instruction(&Instruction::I64Const(0));
        func.instruction(&Instruction::I64Ne);
    }
    func.instruction(&Instruction::Else);
    {
        func.instruction(&Instruction::LocalGet(2));
        func.instruction(&Instruction::I64Const(0));
        func.instruction(&Instruction::I64Eq);
        func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        func.instruction(&Instruction::LocalGet(6));
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::I64Eq);
        func.instruction(&Instruction::Else);
        // f64 `===`
        func.instruction(&Instruction::LocalGet(6));
        func.instruction(&Instruction::F64ReinterpretI64);
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::F64ReinterpretI64);
        func.instruction(&Instruction::F64Eq);
        // | (mode == 2 & elem is NaN & needle is NaN)
        func.instruction(&Instruction::LocalGet(2));
        func.instruction(&Instruction::I64Const(2));
        func.instruction(&Instruction::I64Eq);
        for local in [6u32, 1] {
            func.instruction(&Instruction::LocalGet(local));
            func.instruction(&Instruction::F64ReinterpretI64);
            func.instruction(&Instruction::LocalGet(local));
            func.instruction(&Instruction::F64ReinterpretI64);
            func.instruction(&Instruction::F64Ne);
        }
        func.instruction(&Instruction::I32And);
        func.instruction(&Instruction::I32And);
        func.instruction(&Instruction::I32Or);
        func.instruction(&Instruction::End);
    }
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::If(BlockType::Empty));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::Return);
    func.instruction(&Instruction::End);
    // i += 1; continue
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(4));
    func.instruction(&Instruction::Br(0));
    func.instruction(&Instruction::End); // loop
    func.instruction(&Instruction::End); // block
    func.instruction(&Instruction::I64Const(-1));
}

/// `local[out] = bound < 0 ? max(len + bound, 0) : min(bound, len)`, `len` in
/// local 3 (JS `slice` relative-index clamping).
fn emit_growable_slice_bound(func: &mut Function, bound: u32, out: u32) {
    func.instruction(&Instruction::LocalGet(bound));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(bound));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalTee(out));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalGet(out));
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::I64GtS);
    func.instruction(&Instruction::Select);
    func.instruction(&Instruction::Else);
    func.instruction(&Instruction::LocalGet(bound));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::LocalGet(bound));
    func.instruction(&Instruction::LocalGet(3));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::Select);
    func.instruction(&Instruction::End);
    func.instruction(&Instruction::LocalSet(out));
}

/// `__growable_slice(arr, start, end) -> i64` (spec §3.5): a fresh growable
/// array on the global heap holding `arr[s..e]` with JS clamping (a negative
/// bound counts from the end; both clamp to `[0, len]`; `e <= s` is empty).
/// The caller passes 0 and `i64::MAX` for omitted bounds. Locals: 0 arr,
/// 1 start, 2 end, 3 len, 4 s, 5 e, 6 count, 7 hdr, 8 cap, 9 data.
fn emit_growable_slice_body(func: &mut Function, alloc_index: u32) {
    let mask = crate::emit::growable::GROWABLE_HANDLE_MASK;
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(mask));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(growable_slot(0)));
    func.instruction(&Instruction::LocalSet(3));
    emit_growable_slice_bound(func, 1, 4);
    emit_growable_slice_bound(func, 2, 5);
    // count = e > s ? e - s : 0
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Sub);
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalGet(5));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64GtS);
    func.instruction(&Instruction::Select);
    func.instruction(&Instruction::LocalSet(6));
    // cap = max(count, GROWABLE_INITIAL_CAP)
    let initial = crate::emit::growable::GROWABLE_INITIAL_CAP as i64;
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(initial));
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(initial));
    func.instruction(&Instruction::I64GtS);
    func.instruction(&Instruction::Select);
    func.instruction(&Instruction::LocalSet(8));
    // hdr = alloc(24); data = alloc(cap * 8)
    func.instruction(&Instruction::I32Const(24));
    func.instruction(&Instruction::Call(alloc_index));
    func.instruction(&Instruction::I64ExtendI32U);
    func.instruction(&Instruction::LocalSet(7));
    func.instruction(&Instruction::LocalGet(8));
    func.instruction(&Instruction::I64Const(8));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::Call(alloc_index));
    func.instruction(&Instruction::I64ExtendI32U);
    func.instruction(&Instruction::LocalSet(9));
    // hdr = [count, cap, data]
    for (offset, local) in [(0u64, 6u32), (8, 8), (16, 9)] {
        func.instruction(&Instruction::LocalGet(7));
        func.instruction(&Instruction::I32WrapI64);
        func.instruction(&Instruction::LocalGet(local));
        func.instruction(&Instruction::I64Store(growable_slot(offset)));
    }
    // memory.copy(data, old_data + s * 8, count * 8)
    func.instruction(&Instruction::LocalGet(9));
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::I64Const(mask));
    func.instruction(&Instruction::I64And);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::I64Load(growable_slot(16)));
    func.instruction(&Instruction::LocalGet(4));
    func.instruction(&Instruction::I64Const(8));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::LocalGet(6));
    func.instruction(&Instruction::I64Const(8));
    func.instruction(&Instruction::I64Mul);
    func.instruction(&Instruction::I32WrapI64);
    func.instruction(&Instruction::MemoryCopy { src_mem: 0, dst_mem: 0 });
    // result: hdr | ARRAY_HANDLE_TAG
    func.instruction(&Instruction::LocalGet(7));
    func.instruction(&Instruction::I64Const(crate::ARRAY_HANDLE_TAG as i64));
    func.instruction(&Instruction::I64Or);
}
```

`GROWABLE_INITIAL_CAP` is `pub(crate)` already. Add the accessors in `emitter.rs` (`growable_pop_fn_index`, `growable_find_fn_index`, `growable_slice_fn_index`, `join_growable_f64_fn_index`, each `self.functions["<name>"]`) and the four names to `runtime_smoke.rs`'s mirror list.

- [ ] **Step 4: The emitters and their dispatch**

Add to `emit/growable.rs`:

```rust
/// A growable method this lane lowers (growable-runtime-arrays spec §3.5);
/// `push` and `join` have their own recognizers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GrowableMethod {
    Pop,
    IndexOf,
    Includes,
    Slice,
}

impl<'a> FunctionEmitter<'a> {
    /// `(method, receiver, args)` iff `node` is `<growable value>.<method>(…)`.
    pub(crate) fn growable_method_call_parts(
        &self,
        node: &LirNode,
    ) -> Option<(GrowableMethod, LirNodeId, Vec<LirNodeId>)> {
        if node.kind != LirNodeKind::Call || node.children.is_empty() {
            return None;
        }
        let callee = self.resolve_transparent_callable_node(node.children[0])?;
        let callee_node = self.node(callee);
        if callee_node.children.len() != 1 {
            return None;
        }
        let method = match callee_node.text.as_deref()? {
            "pop" => GrowableMethod::Pop,
            "indexOf" => GrowableMethod::IndexOf,
            "includes" => GrowableMethod::Includes,
            "slice" => GrowableMethod::Slice,
            _ => return None,
        };
        let receiver = callee_node.children[0];
        self.growable_value_elem(receiver)?;
        Some((method, receiver, node.children[1..].to_vec()))
    }

    /// `a.pop()`: the last element, removed; an empty array traps with the
    /// kali pop message.
    pub(crate) fn emit_growable_pop(&mut self, function: &mut Function, receiver: LirNodeId) -> EmittedValue {
        let elem = self.growable_value_elem(receiver).unwrap_or(kali_common::Repr::I64);
        // Belt for inference's snapshot refusal (spec A-8).
        if let Some(name) = self.bare_identifier_name(receiver) {
            if self.growable_for_of_active.as_deref() == Some(name.as_str()) {
                let message = kali_common::growable_for_of_mutation_message(
                    &kali_common::growable_binding_subject(&name, &self.function_name),
                );
                return self.deny_e5506(function, &message);
            }
        }
        let base = self.emit_growable_receiver_handle(function, receiver);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        let (offset, len) = self.strings.intern(kali_common::growable_pop_empty_message());
        function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
        function.instruction(&Instruction::Call(self.growable_pop_fn_index()));
        if elem == kali_common::Repr::F64 {
            function.instruction(&Instruction::F64ReinterpretI64);
        }
        EmittedValue {
            produced: true,
            shape: if elem == kali_common::Repr::String { ValueShape::String } else { ValueShape::Scalar },
        }
    }

    /// `a.indexOf(x)` (strict) / `a.includes(x)` (SameValueZero).
    pub(crate) fn emit_growable_search(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
        needle: Option<LirNodeId>,
        includes: bool,
    ) -> EmittedValue {
        let elem = self.growable_value_elem(receiver).unwrap_or(kali_common::Repr::I64);
        let base = self.emit_growable_receiver_handle(function, receiver);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        let Some(needle) = needle else {
            // `indexOf()` searches for `undefined`, which a number or string
            // array never holds.
            function.instruction(&Instruction::Drop);
            function.instruction(&Instruction::I64Const(if includes { 0 } else { -1 }));
            return EmittedValue {
                produced: true,
                shape: if includes { ValueShape::Boolean } else { ValueShape::Scalar },
            };
        };
        let value = self.emit_node(function, needle, true);
        if !value.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        if elem == kali_common::Repr::F64 {
            if !value.produced || !self.is_float_valued(needle) {
                function.instruction(&Instruction::F64ConvertI64S);
            }
            function.instruction(&Instruction::I64ReinterpretF64);
        }
        let mode = match elem {
            kali_common::Repr::F64 if includes => 2,
            kali_common::Repr::F64 => 1,
            kali_common::Repr::String => 3,
            _ => 0,
        };
        function.instruction(&Instruction::I64Const(mode));
        function.instruction(&Instruction::Call(self.growable_find_fn_index()));
        if includes {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GeS);
            function.instruction(&Instruction::I64ExtendI32U);
            return EmittedValue { produced: true, shape: ValueShape::Boolean };
        }
        EmittedValue { produced: true, shape: ValueShape::Scalar }
    }

    /// `a.slice(s?, e?)`: a new growable array (spec §3.5). A float bound is
    /// truncated with `i64.trunc_sat_f64_s` (ToIntegerOrInfinity, A-13).
    pub(crate) fn emit_growable_slice(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
        args: &[LirNodeId],
    ) -> EmittedValue {
        let base = self.emit_growable_receiver_handle(function, receiver);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        for (position, default) in [(0usize, 0i64), (1, i64::MAX)] {
            match args.get(position).copied() {
                None => {
                    function.instruction(&Instruction::I64Const(default));
                }
                Some(bound) => {
                    let value = self.emit_node(function, bound, true);
                    if !value.produced {
                        function.instruction(&Instruction::I64Const(default));
                    } else if self.is_float_valued(bound) {
                        function.instruction(&Instruction::I64TruncSatF64S);
                    }
                }
            }
        }
        function.instruction(&Instruction::Call(self.growable_slice_fn_index()));
        EmittedValue { produced: true, shape: ValueShape::Scalar }
    }
}
```

In `emit/call.rs`, right after the push dispatch (`:1707-1709`):

```rust
        // Growable-runtime-arrays spec §3.5: `pop`, `indexOf`, `includes` and
        // `slice` on a growable value. Before the plain-array mutator refusal
        // and every static array lane, which would read a growable binding's
        // stale declarator literal.
        if let Some((method, receiver, args)) = self.growable_method_call_parts(node) {
            return match method {
                GrowableMethod::Pop => self.emit_growable_pop(function, receiver),
                GrowableMethod::IndexOf => {
                    self.emit_growable_search(function, receiver, args.first().copied(), false)
                }
                GrowableMethod::Includes => {
                    self.emit_growable_search(function, receiver, args.first().copied(), true)
                }
                GrowableMethod::Slice => self.emit_growable_slice(function, receiver, &args),
            };
        }
```

(import `crate::emit::growable::GrowableMethod` at the top of `call.rs` if `use crate::*;` does not reach it).

- [ ] **Step 5: `join` over every growable value, f64 included**

In `runtime_join_call_parts` (`call.rs:6077`), replace `if self.is_growable_array(base) { return Some(…); }` with a check placed before `let receiver_node = …`'s use: `if self.growable_value_elem(receiver).is_some() { return Some((receiver, node.children.get(1).copied())); }` (this admits call and `slice` receivers; keep the field arm after it unchanged). In `emit_runtime_join`, replace the `growable_base`/`join_index` selection (`:6250-6286`) so the growable lane is:

```rust
        let join_index = if let Some(elem) = self.growable_value_elem(receiver) {
            match elem {
                kali_common::Repr::String => self.join_growable_str_fn_index(),
                kali_common::Repr::F64 => self.join_growable_f64_fn_index(),
                _ => self.join_growable_i64_fn_index(),
            }
        } else if self.object_field_is_growable_array(receiver) {
            self.join_growable_i64_fn_index()
        } else {
            /* the existing per-site arena routing for plain `__join`, unchanged */
        };
```

In `resolve_static_array_join_receiver` (`intrinsics/array.rs:892`), add as the first statement after `let source = …children.first().copied()?;`: `if self.growable_value_elem(source).is_some() { return None; }` — a growable receiver is never folded from its declarator literal.

- [ ] **Step 6: The value oracles know the new results**

In `is_string_valued` (`operators.rs:1102`), before the `runtime_join_call_parts` arm (`:1145`):

```rust
        // Growable-runtime-arrays spec §3.5: `pop` on a string-element array.
        if let Some((crate::emit::growable::GrowableMethod::Pop, receiver, _)) =
            self.growable_method_call_parts(self.node(id))
        {
            return self.growable_value_elem(receiver) == Some(kali_common::Repr::String);
        }
```

In `is_float_valued`'s `LirNodeKind::Call` arm, before the final `callee_node.text…return_repr` expression:

```rust
                if let Some((crate::emit::growable::GrowableMethod::Pop, receiver, _)) =
                    self.growable_method_call_parts(node)
                {
                    return self.growable_value_elem(receiver) == Some(kali_common::Repr::F64);
                }
```

In `is_runtime_concat_string` (`:1397`), after the `dynamic_array_read_base` arm:

```rust
        // A growable string element (index read or `pop`) is a runtime handle
        // when any stored string was a runtime concat — identity `==` on it
        // must refuse, exactly as for a plain array element.
        if let Some(base) = self.growable_array_read_base(self.node(id)) {
            return self
                .repr_table
                .is_array_element_concat_tainted(&self.function_name, &base);
        }
        if let Some((crate::emit::growable::GrowableMethod::Pop, receiver, _)) =
            self.growable_method_call_parts(self.node(id))
        {
            return self.bare_identifier_name(receiver).is_none_or(|base| {
                self.repr_table
                    .is_array_element_concat_tainted(&self.function_name, &base)
            });
        }
```

- [ ] **Step 7: Run the unit tests**

Run: `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_codegen -- --test-threads=6`
Expected: PASS.

- [ ] **Step 8: Write the method cases**

Create `crates/kali_cli/tests/cases/array/growable_methods.toml` (stdouts measured under node v26.10.0, 2026-10-07):

```toml
# Growable runtime arrays: pop, indexOf, includes, slice and float join
# (spec §3.5, A-12..A-14). Every expected stdout is node v26.10.0's for the
# same file, measured 2026-10-07 with `env -u FORCE_COLOR node FILE.js`.

[source]
"pop_module.js" = """
const a = [];
a.push(4); a.push(5); a.push(6);
let last = 0;
while (a.length > 1) { last = a.pop(); }
console.log(last, a.length, a[0]);
const s = [];
s.push("x"); s.push("y");
const top = s.pop();
console.log(top, s.length);
a[0] = 9;
a.push(7);
a[a.length - 1] = 8;
console.log(a.join(","));
"""
"pop_function.js" = """
function main() {
  const a = [];
  a.push(4); a.push(5); a.push(6);
  let last = 0;
  while (a.length > 1) { last = a.pop(); }
  console.log(last, a.length, a[0]);
  const s = [];
  s.push("x"); s.push("y");
  const top = s.pop();
  console.log(top, s.length);
  a[0] = 9;
  a.push(7);
  a[a.length - 1] = 8;
  console.log(a.join(","));
}
main();
"""
"search_module.js" = """
const n = [];
n.push(3); n.push(7); n.push(3);
console.log(n.indexOf(3), n.indexOf(9), n.includes(7), n.includes(8));
const f = [];
let big = 0.5;
f.push(0 / (big - 0.5)); f.push(-1 / big * 0); f.push(2.5);
console.log(f.indexOf(0 / (big - 0.5)), f.includes(0 / (big - 0.5)), f.indexOf(0), f.includes(0), f.indexOf(2.5));
const s = [];
let w = "a";
s.push(w + "b"); s.push("cd");
console.log(s.indexOf("ab"), s.includes("cd"), s.indexOf("x"));
"""
"search_function.js" = """
function main() {
  const n = [];
  n.push(3); n.push(7); n.push(3);
  console.log(n.indexOf(3), n.indexOf(9), n.includes(7), n.includes(8));
  const f = [];
  let big = 0.5;
  f.push(0 / (big - 0.5)); f.push(-1 / big * 0); f.push(2.5);
  console.log(f.indexOf(0 / (big - 0.5)), f.includes(0 / (big - 0.5)), f.indexOf(0), f.includes(0), f.indexOf(2.5));
  const s = [];
  let w = "a";
  s.push(w + "b"); s.push("cd");
  console.log(s.indexOf("ab"), s.includes("cd"), s.indexOf("x"));
}
main();
"""
"slice_module.js" = """
const a = [];
for (let i = 0; i < 5; i++) a.push(i * 10);
console.log(a.slice(1, 3).join(","), a.slice(-2).join(","), a.slice(3, 99).join(","), a.slice(4, 1).length, a.slice().length);
const t = a.slice(1);
t.push(99);
console.log(t.length, a.length, t.join("-"));
const u = t.slice(1, -1);
console.log(u.join("-"), a.slice(1, 4).slice(1).join(","));
"""
"slice_function.js" = """
function main() {
  const a = [];
  for (let i = 0; i < 5; i++) a.push(i * 10);
  console.log(a.slice(1, 3).join(","), a.slice(-2).join(","), a.slice(3, 99).join(","), a.slice(4, 1).length, a.slice().length);
  const t = a.slice(1);
  t.push(99);
  console.log(t.length, a.length, t.join("-"));
  const u = t.slice(1, -1);
  console.log(u.join("-"), a.slice(1, 4).slice(1).join(","));
}
main();
"""
"floats_methods.js" = """
function averages(xs, k) { const out = []; for (let i = 0; i + k <= xs.length; i++) { let s = 0; for (let j = 0; j < k; j++) s += xs[i + j]; out.push(s / k); } return out; }
function floats() { const out = []; let big = 0.5; for (let i = 0; i < 71; i++) big = big * 2; out.push(0.5); out.push(big); out.push(-1 / big * 0); out.push(1 / 3); return out; }
function main() {
  const data = [];
  for (let i = 1; i <= 5; i++) data.push(i * 1.5);
  const avg = averages(data, 2);
  console.log(avg.length, avg[0], avg.join(" "));
  console.log(floats().join(","));
}
main();
"""

[[case]]
name = "pop_until_one_element_remains_module_scope"
rationale = "Spec §3.5: `pop` returns and removes the last element (i64 and string), index writes stay in bounds."
args = ["run", "pop_module.js"]
exit = "success"
stdout = "5 1 4\ny 1\n9,8\n"

[[case]]
name = "pop_until_one_element_remains"
args = ["run", "pop_function.js"]
exit = "success"
stdout = "5 1 4\ny 1\n9,8\n"

[[case]]
name = "f64_search_follows_strict_and_same_value_zero_module_scope"
rationale = "Review Focus 5: over [NaN, -0, 2.5], indexOf(NaN) is -1 and includes(NaN) true (SameValueZero); indexOf(0) finds -0. Strings compare by content (a runtime concat finds the literal)."
args = ["run", "search_module.js"]
exit = "success"
stdout = "0 -1 true false\n-1 true 1 true 2\n0 true -1\n"

[[case]]
name = "f64_search_follows_strict_and_same_value_zero"
args = ["run", "search_function.js"]
exit = "success"
stdout = "0 -1 true false\n-1 true 1 true 2\n0 true -1\n"

[[case]]
name = "slice_of_a_slice_matches_node_module_scope"
rationale = "Review Focus 3, spec §3.5: negative and out-of-range bounds clamp; a slice is a new growable array (pushing it leaves the original alone); a slice of a slice re-slices the temporary."
args = ["run", "slice_module.js"]
exit = "success"
stdout = "10,20 30,40 30,40 0 5\n5 5 10-20-30-40-99\n20-30-40 20,30\n"

[[case]]
name = "slice_of_a_slice_matches_node"
args = ["run", "slice_function.js"]
exit = "success"
stdout = "10,20 30,40 30,40 0 5\n5 5 10-20-30-40-99\n20-30-40 20,30\n"

[[case]]
name = "floats_join_like_node"
rationale = "Review Focus 5, spec A-11, A-12: f64 elements across a parameter and a return; `join` renders 0.5, 2^70 (built by doubling: a literal of 1e21 breaks codegen at the baseline), -0 as `0`, and 1/3."
args = ["run", "floats_methods.js"]
exit = "success"
stdout = "4 2.25 2.25 3.75 5.25 6.75\n0.5,1.1805916207174113e+21,0,0.3333333333333333\n"
```

In `growable_runtime_arrays_traps.toml`, add under `[source]`:

```toml
"pop_empty.js" = """
function main() { const a = []; a.push(1); a.pop(); console.log(a.pop()); }
main();
"""
```

and append:

```toml
[[case]]
name = "pop_on_an_empty_array_traps"
rationale = "node v26.10.0 prints `undefined`; kali has no runtime undefined (spec §1.1) and traps."
args = ["run", "pop_empty.js"]
exit = "failure"
stderr_contains = ["kali: pop on empty array", "E4000"]

[[case]]
name = "check_accepts_the_pop_trap_program"
args = ["check", "pop_empty.js"]
exit = "success"
```

- [ ] **Step 9: Confirm node agrees, then run the cases**

```bash
cd "$S/nodecheck" && python3 -I "$S/node_check.py" /workspace/.worktrees/growable-runtime-arrays/crates/kali_cli/tests/cases/array/growable_methods.toml
cd /workspace/.worktrees/growable-runtime-arrays
CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_cli --test cases -- array/growable_ --test-threads=6
```

Expected: every node line `OK`; every trial PASS.

- [ ] **Step 10: Commit**

```bash
git add -A crates/kali_codegen crates/kali_cli/tests
git commit -m "feat(growable-runtime-arrays): pop, indexOf, includes, slice and f64 join on growable arrays (spec §3.5, A-12..A-14)"
```

---

### Task 11: Codegen — `for-of` over every growable value, nested

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/array.rs` (`emit_for_of_growable_runtime_loop` `:1154-1342`, `emit_for_of_array_iteration` `:1369-1404`)
- Modify: `crates/kali_codegen/src/emitter.rs` (`growable_for_of_active` `:392-403`, its initializer `:756`)
- Modify: `crates/kali_codegen/src/emit/growable.rs` (`emit_growable_push_call`'s two guards, `emit_growable_pop`'s guard)
- Modify: `crates/kali_codegen/src/lower.rs` (local-name helpers `:3798-3809`, reservation `:3746-3761`, `for_of_growable_loop_var_names(_walk)` `:3863-3937`)
- Create: `crates/kali_cli/tests/cases/array/growable_for_of.toml`

**Interfaces:**
- Consumes: `growable_value_elem` (Task 9); element reprs of loop variables (Task 7).
- Produces: `growable_foreach_index_local_name(depth: usize) -> String`, `growable_foreach_len_local_name(depth: usize) -> String`, `growable_foreach_handle_local_name(depth: usize) -> String` in `lower.rs`; `FunctionEmitter.growable_for_of_active: Vec<String>` (stack of active iterable keys; empty key for a call or `slice` iterable).

- [ ] **Step 1: Write the failing cases**

Create `crates/kali_cli/tests/cases/array/growable_for_of.toml` (stdouts measured under node v26.10.0, 2026-10-07):

```toml
# Growable runtime arrays: for-of over bindings, calls and slices, nested,
# with i64, f64 and string elements; multi-argument console output
# (spec §3.5, A-9, A-10). Every expected stdout is node v26.10.0's for the
# same file, measured 2026-10-07 with `env -u FORCE_COLOR node FILE.js`.

[source]
"nested_module.js" = """
const a = [];
a.push(1); a.push(2);
const b = [];
b.push(10); b.push(20);
let s = 0;
for (const x of a) { for (const y of b) { s += x * y; } }
let t = 0;
for (const x of a) { for (const y of a) { t += x * y; } }
console.log(s, t);
"""
"nested_function.js" = """
function main() {
  const a = []; a.push(1); a.push(2);
  const b = []; b.push(10); b.push(20);
  let s = 0;
  for (const x of a) { for (const y of b) { s += x * y; } }
  let t = 0;
  for (const x of a) { for (const y of a) { t += x * y; } }
  console.log(s, t);
}
main();
"""
"p2_module.js" = """
function build(n) { const out = []; let w = "a"; for (let i = 0; i < n; i++) { out.push(w); w = w + "b"; } return out; }
const ws = build(3);
for (const w of ws) console.log(w);
console.log(ws.length, ws.join(" "));
"""
"p2_function.js" = """
function build(n) { const out = []; let w = "a"; for (let i = 0; i < n; i++) { out.push(w); w = w + "b"; } return out; }
function main() {
  const ws = build(3);
  for (const w of ws) console.log(w);
  console.log(ws.length, ws.join(" "));
}
main();
"""
"p3_module.js" = """
const out = [];
for (let i = 0; i < 4; i++) out.push(i);
let s = 0;
for (const x of out) s += x;
console.log(s, out.length, out.join("|"));
"""
"p3_function.js" = """
function main() { const out = []; for (let i = 0; i < 4; i++) out.push(i); let s = 0; for (const x of out) s += x; console.log(s, out.length, out.join("|")); }
main();
"""
"float_loop.js" = """
const data = [];
for (let i = 1; i <= 5; i++) data.push(i * 1.5);
let t = 0;
for (const v of data) t += v;
console.log(t);
"""
"slice_loop.js" = """
function main() { const a = []; for (let i = 0; i < 4; i++) a.push(i); let s = ""; for (const x of a.slice(1, 3)) s = s + x; console.log(s); }
main();
"""
"write_in_loop.js" = """
function main() { const a = []; a.push(1); a.push(2); a.push(3); for (const x of a) a[0] = x; console.log(a.join(",")); }
main();
"""
"break_continue.js" = """
function main() { const a = []; for (let i = 0; i < 6; i++) a.push(i); let s = 0; for (const x of a) { if (x === 1) continue; if (x === 4) break; s += x; } console.log(s); }
main();
"""

[[case]]
name = "nested_for_of_over_two_growable_arrays_and_over_one_array_twice_module_scope"
rationale = "Spec §3.5, A-10: per-depth index, length and handle locals. At the baseline `run` refused the nesting and `check` accepted it."
args = ["run", "nested_module.js"]
exit = "success"
stdout = "90 9\n"

[[case]]
name = "nested_for_of_over_two_growable_arrays_and_over_one_array_twice"
args = ["run", "nested_function.js"]
exit = "success"
stdout = "90 9\n"

[[case]]
name = "string_elements_iterate_and_join_module_scope"
rationale = "Spec §2.1 p2. At the baseline: E5506 at `push`, `for-of`, `join`."
args = ["run", "p2_module.js"]
exit = "success"
stdout = "a\nab\nabb\n3 a ab abb\n"

[[case]]
name = "string_elements_iterate_and_join"
args = ["run", "p2_function.js"]
exit = "success"
stdout = "a\nab\nabb\n3 a ab abb\n"

[[case]]
name = "multi_argument_console_reads_a_growable_array_module_scope"
rationale = "Spec §2.1 p3, A-9. At the baseline: \"console output of multiple arguments where one reads a growable array is unavailable\"."
args = ["run", "p3_module.js"]
exit = "success"
stdout = "6 4 0|1|2|3\n"

[[case]]
name = "multi_argument_console_reads_a_growable_array"
args = ["run", "p3_function.js"]
exit = "success"
stdout = "6 4 0|1|2|3\n"

[[case]]
name = "f64_elements_iterate"
args = ["run", "float_loop.js"]
exit = "success"
stdout = "22.5\n"

[[case]]
name = "for_of_over_a_slice_iterates_the_temporary"
args = ["run", "slice_loop.js"]
exit = "success"
stdout = "12\n"

[[case]]
name = "an_index_write_inside_the_loop_is_seen_by_later_iterations"
rationale = "An index write keeps the length (so it is admitted, spec A-8) and each iteration reloads the element: node's `3,2,3`."
args = ["run", "write_in_loop.js"]
exit = "success"
stdout = "3,2,3\n"

[[case]]
name = "break_and_continue_in_a_growable_for_of"
args = ["run", "break_continue.js"]
exit = "success"
stdout = "5\n"
```

Run: `cd "$S/nodecheck" && python3 -I "$S/node_check.py" /workspace/.worktrees/growable-runtime-arrays/crates/kali_cli/tests/cases/array/growable_for_of.toml` — expected all `OK`. Then `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_cli --test cases -- array/growable_for_of --test-threads=6` — expected FAIL: nesting refused, string/float elements refused ("for-of over a growable array of non-integer elements"), the slice iterable unrecognized.

- [ ] **Step 2: Per-depth scratch locals**

In `lower.rs`, replace the two fixed-name helpers (`:3798-3809`) with:

```rust
/// Names of the per-nesting-depth i64 scratch locals of the runtime `for..of`
/// over a growable value (growable-runtime-arrays spec §3.5, A-10): loop
/// index, snapshotted length, and the iterable's handle (evaluated once).
/// Depth 0 is the outermost growable loop of the function.
pub(crate) fn growable_foreach_index_local_name(depth: usize) -> String {
    format!("__growable_foreach_index{depth}")
}

pub(crate) fn growable_foreach_len_local_name(depth: usize) -> String {
    format!("__growable_foreach_len{depth}")
}

pub(crate) fn growable_foreach_handle_local_name(depth: usize) -> String {
    format!("__growable_foreach_handle{depth}")
}

/// True iff `id` is a growable value as codegen's `growable_value_elem`
/// recognizes it (binding, call to a growable-returning function, `slice` of
/// either). Over-approximates the shadow check (a same-named local is not
/// visible here); an over-reserved local is harmless.
fn node_is_growable_value(
    nodes: &[LirNode],
    repr_table: &kali_common::ReprTable,
    function_name: &str,
    id: LirNodeId,
) -> bool {
    let id = unwrap_transparent_value_node_raw(nodes, id);
    if let Some(name) = bare_identifier_name_of(nodes, id) {
        return repr_table.is_growable_array_binding(function_name, &name);
    }
    let Some(node) = nodes.get(id.0 as usize) else {
        return false;
    };
    if node.kind != LirNodeKind::Call {
        return false;
    }
    let Some(&callee) = node.children.first() else {
        return false;
    };
    if let Some(name) = bare_identifier_name_of(nodes, callee) {
        return repr_table.growable_return(&name).is_some();
    }
    let callee = unwrap_transparent_value_node_raw(nodes, callee);
    nodes.get(callee.0 as usize).is_some_and(|member| {
        member.text.as_deref() == Some("slice")
            && member.children.len() == 1
            && node_is_growable_value(nodes, repr_table, function_name, member.children[0])
    })
}
```

Replace `for_of_growable_loop_var_names` and `for_of_growable_loop_var_names_walk` with one walk returning the loop variables and the maximum nesting depth:

```rust
fn for_of_growable_loops(
    nodes: &[LirNode],
    body_id: LirNodeId,
    repr_table: &kali_common::ReprTable,
    function_name: &str,
) -> (Vec<String>, usize) {
    fn walk(
        nodes: &[LirNode],
        id: LirNodeId,
        repr_table: &kali_common::ReprTable,
        function_name: &str,
        depth: usize,
        names: &mut Vec<String>,
        max_depth: &mut usize,
    ) {
        let Some(node) = nodes.get(id.0 as usize) else {
            return;
        };
        let mut inner = depth;
        if node.kind == LirNodeKind::Branch && node.text.as_deref() == Some("for-of") {
            let growable = node.children.get(1).is_some_and(|&iterable| {
                node_is_growable_value(nodes, repr_table, function_name, iterable)
                    || node_is_growable_i64_field(nodes, repr_table, function_name, iterable)
            });
            if growable {
                if let Some(var) = node
                    .children
                    .first()
                    .and_then(|&left| for_of_loop_var_name_of(nodes, left))
                {
                    if !names.contains(&var) {
                        names.push(var);
                    }
                }
                inner = depth + 1;
                *max_depth = (*max_depth).max(inner);
            }
        }
        for child in &node.children {
            if !is_function_like(nodes, *child) {
                walk(nodes, *child, repr_table, function_name, inner, names, max_depth);
            }
        }
    }
    let mut names = Vec::new();
    let mut max_depth = 0;
    walk(nodes, body_id, repr_table, function_name, 0, &mut names, &mut max_depth);
    (names, max_depth)
}
```

and the reservation (`:3746-3761`) becomes:

```rust
    let (for_of_growable_vars, growable_depth) =
        for_of_growable_loops(nodes, body_id, repr_table, function_name);
    if growable_depth > 0 {
        for var in for_of_growable_vars {
            if !locals.contains(&var) {
                locals.push(var);
            }
        }
        for depth in 0..growable_depth {
            locals.push(growable_foreach_index_local_name(depth));
            locals.push(growable_foreach_len_local_name(depth));
            locals.push(growable_foreach_handle_local_name(depth));
        }
    }
```

- [ ] **Step 3: The active-loop stack**

In `emitter.rs`, change `growable_for_of_active: Option<String>` to `growable_for_of_active: Vec<String>` (doc: "the iterated keys of the runtime growable `for..of` loops currently being emitted, outermost first; an empty key for a call or `slice` iterable. Its length is the nesting depth that picks the scratch locals; the push/pop guards are a belt behind inference's snapshot refusal, spec A-8"), initialized `Vec::new()`. In `emit_growable_push_call` change both `self.growable_for_of_active.as_deref() == Some(x)` tests to `self.growable_for_of_active.iter().any(|key| key == x)`, and the same in `emit_growable_pop` (Task 10).

- [ ] **Step 4: Rewrite the runtime loop**

Change the signature to `fn emit_for_of_growable_runtime_loop(&mut self, function: &mut Function, node: &LirNode, iterable_id: LirNodeId, key: String, elem: kali_common::Repr, owner: Option<String>) -> EmittedValue`. Delete the nesting refusal (`:1163-1178`) and the non-integer refusal (`:1180-1198`). Keep the loop-variable slot resolution (`:1200-1232`). Replace the scratch-local lookup and everything from `// i = 0` through the body emission with:

```rust
        let depth = self.growable_for_of_active.len();
        let (Some(index_local), Some(len_local), Some(handle_local)) = (
            self.locals
                .get(&crate::lower::growable_foreach_index_local_name(depth))
                .copied(),
            self.locals
                .get(&crate::lower::growable_foreach_len_local_name(depth))
                .copied(),
            self.locals
                .get(&crate::lower::growable_foreach_handle_local_name(depth))
                .copied(),
        ) else {
            return self.deny_e5506(
                function,
                "growable for-of scratch locals were not reserved; iteration lowering is unavailable",
            );
        };
        let var_is_float = self.scalar_repr(&loop_name) == kali_common::Repr::F64;
        let elem_is_float = elem == kali_common::Repr::F64;
        if elem_is_float && !var_is_float {
            return self.deny_e5506(
                function,
                "a for-of variable over a growable array of floats must be a float in the current phase",
            );
        }
        let body = node.children.get(2).copied();

        // The iterable is evaluated ONCE (a call or `slice` must not re-run
        // per iteration), then its length snapshotted (spec §3.5: a push/pop
        // on it inside the body is refused, so the snapshot is node's length).
        let handle = self.emit_growable_receiver_handle(function, iterable_id);
        if !handle.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::LocalSet(handle_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(handle_local));
        function.instruction(&Instruction::I64Const(crate::emit::growable::GROWABLE_HANDLE_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(MemArg { offset: 0, align: 3, memory_index: 0 }));
        function.instruction(&Instruction::LocalSet(len_local));
        if let Some(label) = &owner {
            self.enter_iteration(function, label);
        }

        let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.loop_frames.push(LoopFrame {
            break_index,
            continue_index: Some(continue_index),
            continue_is_faithful: true,
        });
        if owner.is_some() {
            self.alloc_iteration_record(function);
        }
        // if i >= n break
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeS);
        let break_depth = self.control_frame_depth(break_index);
        function.instruction(&Instruction::BrIf(break_depth));
        // v = data[i] (the data pointer is reloaded: an index write in the
        // body is seen by later iterations)
        function.instruction(&Instruction::LocalGet(handle_local));
        function.instruction(&Instruction::I64Const(crate::emit::growable::GROWABLE_HANDLE_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(MemArg { offset: 16, align: 3, memory_index: 0 }));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32Mul);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(MemArg { offset: 0, align: 3, memory_index: 0 }));
        match loop_slot {
            Ok(loop_local) => {
                if elem_is_float {
                    function.instruction(&Instruction::F64ReinterpretI64);
                } else if var_is_float {
                    function.instruction(&Instruction::F64ConvertI64S);
                }
                function.instruction(&Instruction::LocalSet(loop_local));
            }
            Err(offset) => {
                // A per-iteration record cell (block-scoping §3.3) is an
                // untyped 8-byte slot: `crate::closure::emit_cell_store`
                // stashes an i64 (`closure.rs:220-231`), and an f64 cell
                // holds the double's bits (`closure.rs:71-76`), which is
                // exactly the raw element slot. Iteration records refuse an
                // F64 variable on their own (`Widening::Baseline`), so the raw
                // slot is stored unconverted.
                let scratch = self.locals.len() as u32;
                crate::closure::emit_cell_store(function, self.current_env_global(), 0, offset, scratch);
            }
        }
        // i += 1 (before the body, so `continue` visits the next element)
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        self.growable_for_of_active.push(key);
        if let Some(body) = body {
            let _ = self.emit_node(function, body, false);
        }
        self.growable_for_of_active.pop();
```

Keep the back-edge, `End`s, frame pops and `exit_iteration` that follow (`:1325-1341`) unchanged.

In `emit_for_of_array_iteration`, replace the bare-identifier growable branch (`:1378-1389`) with:

```rust
        // Growable-runtime-arrays spec §3.5: a growable binding, a call to a
        // growable-returning function, or a `slice` of either.
        if let Some(elem) = self.growable_value_elem(array_id) {
            let key = self.bare_identifier_name(array_id).unwrap_or_default();
            return self.emit_for_of_growable_runtime_loop(function, node, array_id, key, elem, owner);
        }
```

and pass `kali_common::Repr::I64` as `elem` in the field branch (`:1402-1403`; drop its `field_receiver` argument).

- [ ] **Step 5: Run the tests**

```bash
cd /workspace/.worktrees/growable-runtime-arrays
CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_codegen -- --test-threads=6
CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_cli --test cases -- array/growable_ misc/growable_ scope/per_iteration --test-threads=6
```

Expected: `kali_codegen` PASS; `array/growable_*` PASS. `misc/growable_array_*` and `scope/per_iteration` failures that pin the retired Stage 4 refusals are Task 13's re-pins; note them, do not change them here. `scope/per_iteration::a_growable_for_of_is_per_iteration` must still PASS (the record-cell branch).

- [ ] **Step 6: Commit**

```bash
git add -A crates/kali_codegen crates/kali_cli/tests
git commit -m "feat(growable-runtime-arrays): for-of over growable values with per-depth locals and every element repr (spec §3.5, A-10)"
```

---
### Task 12: The spec's end-to-end cases and one refusal per message

**Files:**
- Create: `crates/kali_cli/tests/cases/array/growable_runtime_arrays.toml`
- Create: `crates/kali_cli/tests/cases/array/growable_runtime_arrays_refused.toml`

**Interfaces:**
- Consumes: the `kali` binary with Tasks 1-11.

§5.2's list is covered across the five case files: p1, the three-function shape with `for-of`, the reduced `wrap_paragraph` and the full f64 program here; p2, p3, nested `for-of` in `growable_for_of.toml`; aliasing and arrays built, grown and read across functions in `growable_layout.toml`; `indexOf`/`includes`, `slice`, `pop` and float `join` in `growable_methods.toml`; the traps in `growable_runtime_arrays_traps.toml`.

- [ ] **Step 1: Write the end-to-end cases**

Create `crates/kali_cli/tests/cases/array/growable_runtime_arrays.toml` (stdouts measured under node v26.10.0, 2026-10-07):

```toml
# Growable runtime arrays, end to end (spec §2.1, §5.2): the corpus's dominant
# shape — an array built with `push` in one function, returned, and measured,
# indexed, iterated, joined and sliced by the caller — at module scope and
# inside `main()`. Every expected stdout is node v26.10.0's for the same file,
# measured 2026-10-07 with `env -u FORCE_COLOR node FILE.js`.

[source]
"p1_module.js" = """
function build(n) { const out = []; for (let i = 0; i < n; i++) out.push(i * i); return out; }
const xs = build(5);
console.log(xs.length, xs[2]);
let line = "";
for (const x of xs) line = line + x + " ";
console.log(line);
console.log(xs.join(","), xs.slice(1, 3).join("-"));
"""
"p1_function.js" = """
function build(n) { const out = []; for (let i = 0; i < n; i++) out.push(i * i); return out; }
function main() {
  const xs = build(5);
  console.log(xs.length, xs[2]);
  let line = "";
  for (const x of xs) line = line + x + " ";
  console.log(line);
  console.log(xs.join(","), xs.slice(1, 3).join("-"));
}
main();
"""
"three_module.js" = """
function make() { const xs = []; xs.push(1); return xs; }
function addTwo(a) { a.push(2); a.push(3); }
function total(a) { let s = 0; for (const x of a) s += x; return s; }
const xs = make();
addTwo(xs);
console.log(xs.length, total(xs));
"""
"three_function.js" = """
function make() { const xs = []; xs.push(1); return xs; }
function addTwo(a) { a.push(2); a.push(3); }
function total(a) { let s = 0; for (const x of a) s += x; return s; }
function main() { const xs = make(); addTwo(xs); console.log(xs.length, total(xs)); }
main();
"""
"floats_module.js" = """
function averages(xs, k) { const out = []; for (let i = 0; i + k <= xs.length; i++) { let s = 0; for (let j = 0; j < k; j++) s += xs[i + j]; out.push(s / k); } return out; }
function floats() { const out = []; let big = 0.5; for (let i = 0; i < 71; i++) big = big * 2; out.push(0.5); out.push(big); out.push(-1 / big * 0); out.push(1 / 3); return out; }
const data = [];
for (let i = 1; i <= 5; i++) data.push(i * 1.5);
const avg = averages(data, 2);
console.log(avg.length, avg[0], avg.join(" "));
console.log(floats().join(","));
let t = 0;
for (const v of data) t += v;
console.log(t);
"""
"floats_function.js" = """
function averages(xs, k) { const out = []; for (let i = 0; i + k <= xs.length; i++) { let s = 0; for (let j = 0; j < k; j++) s += xs[i + j]; out.push(s / k); } return out; }
function floats() { const out = []; let big = 0.5; for (let i = 0; i < 71; i++) big = big * 2; out.push(0.5); out.push(big); out.push(-1 / big * 0); out.push(1 / 3); return out; }
function main() {
  const data = [];
  for (let i = 1; i <= 5; i++) data.push(i * 1.5);
  const avg = averages(data, 2);
  console.log(avg.length, avg[0], avg.join(" "));
  console.log(floats().join(","));
  let t = 0;
  for (const v of data) t += v;
  console.log(t);
}
main();
"""
"wrap_module.js" = """
function wrap(words, width, indent) {
  const lines = [];
  let line = indent;
  let used = 0;
  let started = false;
  for (const word of words) {
    const n = word.length;
    if (started && used + 1 + n > width) { lines.push(line); line = indent + word; used = n; }
    else if (started) { line = line + " " + word; used = used + 1 + n; }
    else { line = line + word; used = n; }
    started = true;
  }
  if (started) lines.push(line);
  return lines;
}
const words = [];
for (const w of ["the", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"]) words.push(w);
for (const line of wrap(words, 16, "> ")) { console.log(line); }
"""
"wrap_function.js" = """
function wrap(words, width, indent) {
  const lines = [];
  let line = indent;
  let used = 0;
  let started = false;
  for (const word of words) {
    const n = word.length;
    if (started && used + 1 + n > width) { lines.push(line); line = indent + word; used = n; }
    else if (started) { line = line + " " + word; used = used + 1 + n; }
    else { line = line + word; used = n; }
    started = true;
  }
  if (started) lines.push(line);
  return lines;
}
function main() {
  const words = [];
  for (const w of ["the", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"]) words.push(w);
  for (const line of wrap(words, 16, "> ")) { console.log(line); }
}
main();
"""

[[case]]
name = "p1_build_return_measure_index_iterate_join_slice_module_scope"
rationale = "Spec §2.1 p1. At the baseline: E5506 at `push` (literal array), `for-of`, `join`, `slice`."
args = ["run", "p1_module.js"]
exit = "success"
stdout = "5 4\n0 1 4 9 16 \n0,1,4,9,16 1-4\n"

[[case]]
name = "p1_build_return_measure_index_iterate_join_slice"
args = ["run", "p1_function.js"]
exit = "success"
stdout = "5 4\n0 1 4 9 16 \n0,1,4,9,16 1-4\n"

[[case]]
name = "check_accepts_p1"
args = ["check", "p1_module.js", "p1_function.js"]
exit = "success"

[[case]]
name = "built_in_one_function_grown_in_a_second_iterated_in_a_third_module_scope"
args = ["run", "three_module.js"]
exit = "success"
stdout = "3 6\n"

[[case]]
name = "built_in_one_function_grown_in_a_second_iterated_in_a_third"
args = ["run", "three_function.js"]
exit = "success"
stdout = "3 6\n"

[[case]]
name = "f64_arrays_average_join_and_iterate_module_scope"
rationale = "Spec §5.2: a moving-average sum over an f64 parameter, `join` of 0.5, 2^70, -0 and 1/3 (A-11, A-12), and a `for-of` sum."
args = ["run", "floats_module.js"]
exit = "success"
stdout = "4 2.25 2.25 3.75 5.25 6.75\n0.5,1.1805916207174113e+21,0,0.3333333333333333\n22.5\n"

[[case]]
name = "f64_arrays_average_join_and_iterate"
args = ["run", "floats_function.js"]
exit = "success"
stdout = "4 2.25 2.25 3.75 5.25 6.75\n0.5,1.1805916207174113e+21,0,0.3333333333333333\n22.5\n"

[[case]]
name = "reduced_wrap_paragraph_module_scope"
rationale = "Spec §1, §5.2: `wrap_paragraph.js`'s `wrap()` with the words pushed from a literal (no string methods) and the width kept in a number (A-17). `for (const line of wrap(…))` iterates a call result."
args = ["run", "wrap_module.js"]
exit = "success"
stdout = "> the quick brown\n> fox jumps over\n> the lazy dog\n"

[[case]]
name = "reduced_wrap_paragraph"
args = ["run", "wrap_function.js"]
exit = "success"
stdout = "> the quick brown\n> fox jumps over\n> the lazy dog\n"
```

- [ ] **Step 2: Write the refusal cases**

Create `crates/kali_cli/tests/cases/array/growable_runtime_arrays_refused.toml`:

```toml
# Growable runtime arrays: one refusal per message (spec §3.6, A-3..A-8,
# A-18), under both `check` and `run`, so the two are shown to agree. Every
# refusal is raised by repr inference, before codegen. node v26.10.0 runs each
# program (outputs in the rationales).

[matrix]
cmd = ["check", "run"]

[source]
"mixed_param.js" = """
function total(a) { let s = 0; for (let i = 0; i < a.length; i++) s += a[i]; return s; }
const xs = []; xs.push(1);
const p = new Array(2); p[0] = 1; p[1] = 2;
console.log(total(xs), total(p));
"""
"mixed_binding.js" = """
let a = new Array(3);
const b = []; b.push(1);
a = b;
console.log(a.length);
"""
"mixed_conditional.js" = """
function f(c) { const xs = []; xs.push(1); const ys = c ? xs : new Array(2); return ys.length; }
console.log(f(true));
"""
"non_array_write.js" = """
const xs = []; xs.push(1);
let y = xs;
y = 5;
console.log(xs.length, y);
"""
"literal_expression.js" = """
function f(c) { if (c) return []; const o = []; o.push(1); return o; }
const r = f(false);
console.log(r.length);
"""
"element_object.js" = """
function main() { const o = []; o.push({ a: 1 }); console.log(o.length); }
main();
"""
"element_boolean.js" = """
function main() { const o = []; let n = 2; o.push(n > 1); console.log(o[0]); }
main();
"""
"module_read.js" = """
const out = [];
out.push(1);
function size() { return out.length; }
console.log(size());
"""
"capture.js" = """
function main() { const o = []; o.push(1); const f = () => o.length; console.log(f()); }
main();
"""
"for_of_push.js" = """
function main() { const a = []; a.push(1); for (const x of a) { if (x < 3) a.push(x + 1); } console.log(a.length); }
main();
"""
"for_of_alias.js" = """
function main() { const a = []; a.push(1); const b = a; for (const x of a) { if (x < 3) b.push(x + 1); } console.log(a.length); }
main();
"""
"for_of_call.js" = """
function add(arr, v) { arr.push(v); }
function main() { const a = []; a.push(1); for (const x of a) { if (x < 3) add(a, x + 1); } console.log(a.length); }
main();
"""
"from_index.js" = """
function main() { const a = []; a.push(1); a.push(1); console.log(a.indexOf(1, 1)); }
main();
"""
"length_write.js" = """
function main() { const a = []; a.push(1); a.length = 0; console.log(a.length); }
main();
"""
"reverse.js" = """
function main() { const a = []; a.push(1); a.push(2); a.reverse(); console.log(a[0]); }
main();
"""
"computed_push.js" = """
function main() { const a = []; a.push(1); a["push"](2); console.log(a.length); }
main();
"""
"print_whole.js" = """
function main() { const a = []; a.push(1); console.log(a); }
main();
"""
"print_whole_multi.js" = """
function main() { const a = []; a.push(1); console.log("n", a); }
main();
"""
"plain_use.js" = """
function main() { const a = []; a.push(1); console.log(typeof a); }
main();
"""
"temporary.js" = """
function make() { const o = []; o.push(1); return o; }
console.log(make().pop());
"""

[[case]]
name = "mixed_layout_parameter_is_refused"
rationale = "Review Focus 1, spec §3.2. node prints `1 3`."
args = ["${cmd}", "mixed_param.js"]
exit = "failure"
stderr_contains = ["E5506", "`a` in `total` would hold both a growable array"]

[[case]]
name = "mixed_layout_binding_is_refused"
rationale = "Spec §3.2. node prints `1`."
args = ["${cmd}", "mixed_binding.js"]
exit = "failure"
stderr_contains = ["E5506", "`a` at module scope would hold both a growable array"]

[[case]]
name = "mixed_layout_conditional_is_refused"
rationale = "Spec §3.2. node prints `1`."
args = ["${cmd}", "mixed_conditional.js"]
exit = "failure"
stderr_contains = ["E5506", "`ys` in `f` would hold both a growable array"]

[[case]]
name = "a_growable_binding_given_a_non_array_value_is_refused"
rationale = "Spec A-7. node prints `1 5`."
args = ["${cmd}", "non_array_write.js"]
exit = "failure"
stderr_contains = ["E5506", "`y` at module scope holds a growable array and is also given a value that is not an array"]

[[case]]
name = "a_literal_returned_directly_into_a_growable_component_is_refused"
rationale = "Spec A-5. node prints `1`."
args = ["${cmd}", "literal_expression.js"]
exit = "failure"
stderr_contains = ["E5506", "an array literal written directly as a call argument or `return` value in `f`"]

[[case]]
name = "an_object_element_is_refused"
rationale = "Spec §3.4. node prints `1`."
args = ["${cmd}", "element_object.js"]
exit = "failure"
stderr_contains = ["E5506", "`o` in `main` is a growable array with an element that is"]

[[case]]
name = "a_boolean_element_is_refused"
rationale = "Spec A-4: kali would store `1` where node prints `true`."
args = ["${cmd}", "element_boolean.js"]
exit = "failure"
stderr_contains = ["E5506", "`o` in `main` is a growable array with an element that is"]

[[case]]
name = "a_function_using_a_module_level_growable_array_is_refused"
rationale = "Spec §3.3, A-18. node prints `1`. At the baseline the module-binding refusal fired in `run` only."
args = ["${cmd}", "module_read.js"]
exit = "failure"
stderr_contains = ["E5506", "function `size` uses the module-level growable array `out`"]

[[case]]
name = "a_captured_growable_array_is_refused"
rationale = "Spec §1.1. node prints `1`."
args = ["${cmd}", "capture.js"]
exit = "failure"
stderr_contains = ["E5506", "the growable array `o` in `main` is captured"]

[[case]]
name = "a_push_on_the_iterated_array_inside_its_for_of_is_refused"
rationale = "Spec §3.5. node grows the iteration and prints `3`; kali's snapshot would print `2`."
args = ["${cmd}", "for_of_push.js"]
exit = "failure"
stderr_contains = ["E5506", "`push` or `pop` on the growable array `a` in `main`"]

[[case]]
name = "a_push_through_an_alias_inside_the_for_of_is_refused"
rationale = "Spec A-8, Review Focus 2. node prints `3`."
args = ["${cmd}", "for_of_alias.js"]
exit = "failure"
stderr_contains = ["E5506", "`push` or `pop` on the growable array `a` in `main`"]

[[case]]
name = "a_push_through_a_parameter_inside_the_for_of_is_refused"
rationale = "Spec A-8, Review Focus 4. node prints `3`."
args = ["${cmd}", "for_of_call.js"]
exit = "failure"
stderr_contains = ["E5506", "`push` or `pop` on the growable array `a` in `main`"]

[[case]]
name = "from_index_is_refused"
rationale = "Spec §3.5. node prints `1`."
args = ["${cmd}", "from_index.js"]
exit = "failure"
stderr_contains = ["E5506", "`indexOf` with a `fromIndex` argument"]

[[case]]
name = "a_length_write_is_refused"
rationale = "Spec §3.5. node prints `0`."
args = ["${cmd}", "length_write.js"]
exit = "failure"
stderr_contains = ["E5506", "assigning to `.length` of the growable array `a` in `main`"]

[[case]]
name = "an_unsupported_mutator_is_refused"
rationale = "Spec §1.1. node prints `2`."
args = ["${cmd}", "reverse.js"]
exit = "failure"
stderr_contains = ["E5506", "`.reverse()` on the growable array `a` in `main`"]

[[case]]
name = "a_computed_push_is_refused"
rationale = "node prints `2`."
args = ["${cmd}", "computed_push.js"]
exit = "failure"
stderr_contains = ["E5506", "`[\"push\"]()` on the growable array `a` in `main`"]

[[case]]
name = "printing_a_whole_growable_array_is_refused"
rationale = "Spec §3.5, A-9. node prints `[ 1 ]`. At the baseline this refusal was codegen-only for plain arrays."
args = ["${cmd}", "print_whole.js"]
exit = "failure"
stderr_contains = ["E5506", "printing a whole runtime array is unavailable"]

[[case]]
name = "printing_a_whole_growable_array_among_other_arguments_is_refused"
rationale = "node prints `n [ 1 ]`."
args = ["${cmd}", "print_whole_multi.js"]
exit = "failure"
stderr_contains = ["E5506", "printing a whole runtime array is unavailable"]

[[case]]
name = "a_plain_use_of_a_growable_array_is_refused"
rationale = "Spec A-3. node prints `object`."
args = ["${cmd}", "plain_use.js"]
exit = "failure"
stderr_contains = ["E5506", "the growable array `a` in `main` is used as a plain value here"]

[[case]]
name = "popping_a_call_result_directly_is_refused"
rationale = "Spec A-6. node prints `1`."
args = ["${cmd}", "temporary.js"]
exit = "failure"
stderr_contains = ["E5506", "indexing, `push` or `pop` directly on the array `make(…)` returns"]
```

- [ ] **Step 3: Confirm node agrees and run every growable case**

```bash
cd "$S/nodecheck" && python3 -I "$S/node_check.py" /workspace/.worktrees/growable-runtime-arrays/crates/kali_cli/tests/cases/array/growable_runtime_arrays.toml
cd /workspace/.worktrees/growable-runtime-arrays
CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_cli --test cases -- array/growable_ --test-threads=6
```

Expected: every node line `OK`; every trial PASS (the refusal file is 20 cases × 2 commands). A refusal case that shows a different E5506 is a resolver gate firing first (the resolver stops compilation before the shape conflicts): find the gate, make it skip growable values (Task 8's pattern), and add a resolver unit test beside Task 8's.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_cli/tests/cases/array/growable_runtime_arrays.toml crates/kali_cli/tests/cases/array/growable_runtime_arrays_refused.toml
git commit -m "test(growable-runtime-arrays): end-to-end node-checked cases and one refusal per message under check and run"
```

---

### Task 13: Full suite, corpus measurement and bookkeeping

**Files:**
- Modify: re-pinned case files and unit tests the full suite reports (Step 2)
- Modify: `specs/19-feature-maturity.md` (row at `:208`; a new row after the "Default parameters on function declarations" row at `:222`)
- Modify: `docs/superpowers/specs/2026-10-07-growable-runtime-arrays-design.md` (append `## 7. Amendments`)
- Create: `docs/superpowers/followups/growable-runtime-arrays-discovered-defects.md`
- Possibly modify: `tools/blast-radius/accepts.json`, `tools/blast-radius/counts.json`, `docs/superpowers/followups/blast-radius-ranking.md`, register rows and oracle cases (only if Step 4 shows a move)

- [ ] **Step 1: Run the whole workspace, detached**

```bash
cd /workspace/.worktrees/growable-runtime-arrays
rm -f "$S/gra-workspace.exit"
setsid nohup bash -c "CARGO_TARGET_DIR=/workspace/.worktrees/growable-runtime-arrays/target cargo test -j 6 --workspace --no-fail-fast -- --test-threads=6 > $S/gra-workspace.log 2>&1; echo \$? > $S/gra-workspace.exit" > /dev/null 2>&1 < /dev/null &
```

Poll: `while [ ! -f "$S/gra-workspace.exit" ]; do sleep 60; done`. Then:

```bash
grep -E '^test result' "$S/gra-workspace.log" | grep -v ' 0 failed'
grep -E '^test .* FAILED$' "$S/gra-workspace.log"
```

- [ ] **Step 2: Re-pin what moved, debug what broke**

A failure that pins a Stage 4 refusal this project retired, or kali's old output for a program that now runs, is a moved pin: run the program under node, re-pin to node's stdout (or to the new refusal's text) with a dated `RE-PINNED 2026-10-07 by the growable-runtime-arrays project` note in the rationale (the convention in `crates/kali_cli/tests/cases/object/property_key_identity.toml`), and list it in the followups file §4. Measured while planning, these move:

| file | case | node v26.10.0 | now |
|---|---|---|---|
| `misc/growable_array_fail_closed.toml` | `escaping_via_return_fails_closed` | `3` | runs |
| same | `pop_mutator_fails_closed` | `1` | runs |
| same | `alias_binding_fails_closed` | `3` | runs |
| same | `wrong_arity_push_fails_closed` | `2` | runs |
| same | `float_element_push_fails_closed` | `2` | runs |
| same | `multi_arg_console_log_with_growable_read_fails_closed_length_and_index` | `2 1` | runs |
| same | `multi_arg_console_log_with_growable_read_fails_closed_string_and_length` | `len 1` | runs |
| same | `object_identifier_push_fails_closed` | `{ a: 1 }` | still E5506 (M2) |
| same | `computed_push_call_fails_closed`, `optional_chain_push_call_fails_closed`, `closure_capture_fails_closed` | `2`, `2`, `2` | still E5506 (M8, M8, M4) |
| `misc/growable_array_fail_closed_push_diagnostics.toml` | `…_wrong_arity` | `2` | runs |
| same | `…_object_literal_arg` | `1` | E5506, M2 text instead of the literal `.push()` text |

A renamed case keeps its name (the trial id is history); only `exit`, `stdout`/`stderr_contains` and the rationale change. Also expect pins in `misc/growable_array_core.toml` (the string-element for-of refusal and the same-array push refusal now run or carry M5), `runtime/array_bounds.toml`, `array/literal_array_mutators.toml`, `soundness/bitwise_compound.toml`, `soundness/structured_clone.toml`, `object/class_instances.toml`, `misc/set_iteration_runtime.toml` (each pins "calling `.push()` on a literal array" somewhere; a program whose array is now growable runs instead). Any other failure: STOP and debug with superpowers:systematic-debugging. Re-run Step 1 until clean.

- [ ] **Step 3: Re-count the corpus per message family**

```bash
cd /workspace/.worktrees/growable-runtime-arrays/tools/blast-radius/corpus/extension
for bin in /workspace/target/debug/kali /workspace/.worktrees/growable-runtime-arrays/target/debug/kali; do
  for f in *.js; do "$bin" check "$f" 2>&1; done > "$S/corpus-$(basename $(dirname $(dirname $(dirname $bin)))).txt"
done
for file in "$S"/corpus-*.txt; do
  echo "== $file: $(grep -c '^error' "$file") error lines"
  for pat in 'computed member access `' 'for-of array iteration lowering is unavailable unless the iterable is a literal array' 'on a literal array is unavailable in the current phase: kali folds' 'Array.prototype.slice is unavailable unless the receiver is a statically-known' "array search method 'indexOf' is unavailable unless" 'Array.prototype.join is unavailable unless' 'reassigning an array binding to a non-array value' 'Array.prototype.concat is unavailable' 'mutating a literal array is unavailable in the current direct-runtime path' 'growable array'; do
    printf '%5d  %s\n' "$(grep -c -- "$pat" "$file")" "$pat"
  done
  grep '^error' "$file" | sed -E 's/`[^`]*`/`X`/g' | cut -c1-110 | sort | uniq -c | sort -rn > "$file.families"
done
diff "$S"/corpus-workspace.txt.families "$S"/corpus-growable-runtime-arrays.txt.families
```

The baseline (`corpus-workspace.txt`, from the `/workspace` build at `edb3a77df`) must reproduce §2.2: 386 lines; 86, 65, 52, 22, 10, 10, 7, 3, 1. Record both columns in the followups file §2, plus the number of programs with at least one array-family refusal (36 at the baseline with this command; A-20):

```bash
cd /workspace/.worktrees/growable-runtime-arrays/tools/blast-radius/corpus/extension
for bin in /workspace/target/debug/kali /workspace/.worktrees/growable-runtime-arrays/target/debug/kali; do
  for f in *.js; do
    "$bin" check "$f" 2>&1 | grep -qE 'computed member access|for-of array iteration lowering|on a literal array is unavailable|Array\.prototype\.(slice|join|concat) is unavailable|array search method|reassigning an array binding|mutating a literal array|growable array' && echo "$f"
  done | wc -l
done
``` A family that grows, or a non-array family that changes, is investigated before going on: it means this project now refuses something it used to admit.

- [ ] **Step 4: Re-measure the accept set**

```bash
cd /workspace/.worktrees/growable-runtime-arrays/tools/blast-radius
ln -s /workspace/tools/blast-radius/node_modules node_modules
node --test && node accepts.mjs && node count.mjs
cd ../.. && git diff --stat tools/
```

If only `kaliBinary` changed in `accepts.json`, run `git checkout tools/blast-radius/accepts.json tools/blast-radius/counts.json` and record "accept set unchanged" in the followups file (the spec expects 0/40 extension accepts to stay 0/40). If the accept set moved, keep both files, run `CARGO_TARGET_DIR=$T cargo run -j 6 -p kali_blast_radius --example rank`, splice its stdout between the `GENERATED-PROVENANCE` and `GENERATED` markers of `docs/superpowers/followups/blast-radius-ranking.md` (provenance is the part before `## 2. The bands`), add a dated §6 amendment in the form of the existing ones, and run `CARGO_TARGET_DIR=$T cargo test -j 6 -p kali_blast_radius -- --test-threads=6` (expected PASS). If a register lane moved (R-31's array face in `cases/oracle/tier2.toml`, any growable row), re-pin its oracle case and re-derive its §0.2 row. In every case, `rm tools/blast-radius/node_modules` afterwards; it must not be committed.

- [ ] **Step 5: Maturity rows**

Replace row `:208` of `specs/19-feature-maturity.md` (the stale "Computed array subscripts … no `.length`/`push` yet" row) with:

```markdown
| Computed array subscripts: linear-memory arrays via `new Array(n)` with indexed read/write, including arithmetic index expressions (e.g. `a[i+1]`, `a[r-1]`); bump-allocated (no free); fixed length | Phase 1 MVP | Backed by the exported `__heap` bump global; consistent with the no-tracing-GC model. A `new Array(n)` array is a fixed-length `[len][elem…]` buffer: `.length` reads, bounds-checked index reads and writes (an out-of-range index traps), no growth and no free/reclaim; length-changing methods and `.length` writes are refused. Arrays that grow with `push` are a separate layout (see "Growable runtime arrays"); a value that could hold both layouts is refused. Does not imply general objects |
```

and add after the default-parameters row:

```markdown
| Growable runtime arrays (`const out = []; out.push(x); return out;`) | Phase 1 MVP | An array-literal binding (`const`/`let`/`var`) becomes a growable runtime array when a `push`, `pop` or index write reaches it anywhere its value flows: returned, passed to a declared function, aliased, sliced, at module scope or in a function. Elements are all numbers (i64 or f64) or all strings, one repr per array. Supported: `push(…)` (any number of values; returns the new length), `pop()`, `.length`, index read and write with a runtime integer index, `indexOf(x)` (strict; NaN is never found), `includes(x)` (SameValueZero), `slice(s?, e?)` (JS negative-index clamping; float bounds truncated; a new array), `join(sep?)` (default `,`; f64 elements as JS `String(number)`; string elements and separator must be proven ASCII), `for-of` over a binding, a call result or a `slice` (nesting admitted; the length is fixed at loop entry), and multi-argument `console.log` of reads. Memory: an array that never leaves its creating function uses that function's arena; every other is allocated with `__alloc_global` and never reclaimed. Traps (kali message on stderr, exit 1; `check` accepts): an index read or write outside `0 <= i < length` (`kali: array index out of bounds`), `pop()` on an empty array (`kali: pop on empty array`). Refused (E5506, `check` and `run` alike): a value that could hold both a growable array and a `new Array(n)` array (a parameter, a binding, a `?:`/`||` arm); a growable binding, parameter or return also given a non-array value, `undefined` or a missing argument; an array literal written directly as an argument or `return` value where it would be growable; elements that are objects, arrays, functions, booleans, `null`, `undefined` or holes, or a mix of numbers and strings; a function using a module-level growable array; a growable array captured by a closure, nested function or class body; `push`/`pop` on the iterated array, directly, through an alias or through a function it is passed to, inside its own `for-of`; `fromIndex` on `indexOf`/`includes`; a `.length` write; any other method, computed or optional-chain call; printing a whole growable array; any other value use (`typeof`, `+`, comparison, object or array element, argument to an unknown callee); indexing, `push` or `pop` directly on a call or `slice` result. Not claimed: callback methods (`forEach`, `map`, `filter`, `reduce`, `some`, `every`, `find`, `sort`), `shift`/`unshift`/`splice`/`reverse`/`copyWithin`/`fill`/`concat`, printing a whole array, `undefined` results, non-ASCII `join`, reclaiming escaping arrays, closure capture. Evidence: `kali_types/src/growable/{flow,facts,positions}_tests.rs`, `repr_infer_tests.rs`, `array_return_tests.rs`, `static_analysis/array_tests/growable.rs`, `kali_codegen/src/emit/growable_tests.rs`, `cases/array/growable_layout.toml`, `growable_methods.toml`, `growable_for_of.toml`, `growable_runtime_arrays.toml`, `growable_runtime_arrays_refused.toml`, `growable_runtime_arrays_traps.toml` |
```

Check every claim in both rows against the cases before committing; if Step 2 or 3 showed a claim false, correct the row, never the evidence.

- [ ] **Step 6: Spec amendments**

Append to `docs/superpowers/specs/2026-10-07-growable-runtime-arrays-design.md`:

```markdown
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
* **§3.6 additions.** Messages beyond the eight listed: plain use (A-3), literal expression (A-5), temporary use (A-6), non-array write (A-7). The exact texts are in `crates/kali_common/src/messages.rs`.
```

Add any amendment found during execution as A-21 onward, with what was measured.

- [ ] **Step 7: Followups file**

Create `docs/superpowers/followups/growable-runtime-arrays-discovered-defects.md`:

```markdown
# Defects the growable-runtime-arrays project measured and did NOT fix

**Filed** <date> by the **growable-runtime-arrays** project
(`docs/superpowers/specs/2026-10-07-growable-runtime-arrays-design.md`). **Oracle:**
`node v26.10.0`. **Baseline:** `edb3a77df` on `main`. **Branch:**
`growable-runtime-arrays`.

## §1. Pre-existing, found while planning

| program | node | kali at the baseline |
|---|---|---|
| `function sum(a) {…} const a = [1, 2]; console.log(sum(a));` | `3` | `run`: E5506 "passing an array literal to function 'sum'…"; `check` exits 0 (refused in codegen only) |
| `const a = new Array(3); a[0] = 7; console.log(a);` | `[ 7, <2 empty items> ]` | `run`: E5506 "printing a whole runtime array…"; `check` exits 0 |
| `const out = [1, 2]; function size() { return out.length; } console.log(size());` | `2` | `run`: E5506 "reading module binding 'out' from a function…"; `check` exits 0 |
| `function main() { const o = new Array(2); const f = () => o.length; console.log(f()); } main();` | `2` | `run`: E5506 "a closure `__kali_fn_0` that captures `o`…"; `check` exits 0 |
| `let x = 0.5; console.log("c" + x * 1e21);` (any integer-valued float literal ≥ 2^63 in a runtime expression) | `c5e+20` | `run`: `error[E4201]: failed to load WASM module` |
| `let z = Math.pow(10, 21); console.log("c" + z);` (also `10 ** 21`) | `c1e+21` | prints `c3875820019684212736`, exit 0 |
| `function main() { let line = "> "; for (let i = 0; i < 3; i++) { if (line.length > 3) { line = "> "; } else { line = line + "ab"; } } console.log(line); } main();` | `> ab` | E5506 "reassigning an array binding to a non-array value…" (`.length` on a string registers it as an array; 7 corpus lines) |
| `function j(a) { return a.join("-"); } const a = [4, 5]; console.log(j(a));` | `4-5` | E5506 "elements of `a` … are used as both strings and numbers" |
| `function main() { const a = []; a.push(1); let i = 1.5; console.log(a[i]); } main();` (float index) | `undefined` | codegen-only E5506 (Stage 4): `check` exits 0 |

## §2. Corpus

<the §2.2 table, before and after, from Task 13 Step 3; the family diff; programs with at least one array-family refusal, before and after>

## §3. Accept set

<"unchanged at extension 0/40" or the new figures from Step 4>

## §4. Pins moved by this project

<each re-pinned case and unit test from Tasks 6, 8 and 13 Step 2>

## §5. Residue inside the lane

- A pushed identifier bound to a boolean (`let flag = x > 1; out.push(flag)`) is not caught by the element check (it only sees syntax and what the identifier is declared as); it stores and prints `1`/`0`. Inherited from Stage 4's identifier guard.
- The snapshot rule (A-8) treats two arrays as one when they share a component because both reach the same parameter; such a `for-of` that pushes the other array is refused although node would agree with the snapshot.
- Every growable array that leaves its function is allocated globally and never reclaimed, even when the callee does not keep it.
<anything Tasks 9-12 recorded, e.g. what `two_builds` showed about the strings pushed into an escaping array>
```

Replace each `<…>` with the measured content before committing; do not commit angle-bracket text. Remeasure the §1 rows with the new binary where this project could have changed them (the module-read and capture rows now refuse in `check` for growable arrays only; the plain-array rows stay as measured).

- [ ] **Step 8: Format, lint, commit**

```bash
cd /workspace/.worktrees/growable-runtime-arrays
cargo fmt --all --check
CARGO_TARGET_DIR=$T cargo clippy -q -j 6 -p kali_common -p kali_types -p kali_codegen -p kali_cli 2>&1 | grep -E '^(warning|error)' -A4 | head -40
git status --short
git add specs docs tools crates
git commit -m "docs(growable-runtime-arrays): maturity rows, spec amendments A-1..A-20, followups; corpus re-measured"
```

Expected: `fmt` clean, no new clippy warnings in the touched crates, and `git status` clean afterwards (no `node_modules`).

---

## Self-review

**1. Spec coverage.**
- §3.1 growable property, sources, rule, edges (return, call result, argument, alias, slice), "unchanged" lanes, allowlist retired: Tasks 3-6 (A-1, A-2, A-3), Task 8 (resolver), pinned in `repr_infer_tests` (Task 6) and the case files.
- §3.2 layouts never mixed (parameter, binding, conditional): Task 3 solve, Task 5 rendering, Task 12 three refusal cases.
- §3.3 memory (arena for local-only, global otherwise, push in a loop arena, module scope, module read refused): Task 2 table, Task 6 publication, Task 9 allocation, Task 5/12 module-read refusal; `two_builds` case.
- §3.4 element reprs (one per array, f64 reinterpret, string handles, mixed or object refused): Tasks 6-7 (inference), Task 9 (store/load), Task 12.
- §3.5 every operation row: `push` Task 9; `pop` Task 10; `.length` read (existing lane, now for params/call-bound) and write refusal Tasks 5/9; index read/write with traps Task 9; `indexOf`/`includes` Task 10; `slice` Task 10; `join` Task 10; `for-of` with snapshot, nesting and all reprs Tasks 5, 7, 11; console Task 11, M9 Task 5.
- §3.6 every refusal and both traps: Task 1 texts, Task 5 rendering, Task 12 one case each under `check` and `run`; traps in `growable_runtime_arrays_traps.toml` with the `check` outcome.
- §5.1 unit tests per edge and per inference refusal: Tasks 3-7; codegen emitter tests: A-15, Tasks 9-10.
- §5.2 case list: Task 12's coverage note maps every item to a file.
- §5.3 measurement: Task 13 Steps 1, 3, 4. §5.4 bookkeeping: Task 13 Steps 5-7. §5.5 branching: Global Constraints, Task 0.

**2. Placeholder scan.** The only `<…>` text is inside the followups template in Task 13 Step 7, which that step says to replace with measurements before committing. No "TBD", "similar to Task N" or "add error handling" steps.

**3. Type consistency.** `GrowNode`, `GrowFacts` fields, `GrowSolution` methods and `UseKind` variants are defined once in Task 3 and used with the same names in Tasks 4-7; `growable_value_elem` (Task 9) is the name Tasks 10-11 call; `growable_receiver_elem`/`growable_receiver_non_ascii` (Task 8) are resolver-only; `growable_foreach_*_local_name(depth)` is defined and used in Task 11 only; the synthetic names and accessor names match between Tasks 9-10 and the `runtime_smoke.rs` mirror.

**4. Review Focus.** Each of the five lines names the test that pins it in its owning task: Task 3 and Task 12 (mixed layout at a parameter), Task 9 and Task 5 (alias), Task 10 (slice of a slice), Task 5 and Task 12 (push through a parameter inside `for-of`), Task 10 (f64 `-0`/NaN in `join`, `indexOf`, `includes`).
