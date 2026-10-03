# Literal Array Mutators Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** An in-place mutator on a literal array never silently does nothing.
`push`/`pop`/`shift`/`unshift`/`splice`/`reverse`/`sort`/`fill`/`copyWithin`
and a `.length =` write on a literal array refuse with `E5506` under `kali check`
and `kali run`. The working growable `push` lane is untouched. A `run` backstop
refuses any of those method names that reaches the placeholder fallback.
`reverse`/`sort`/`copyWithin` and an optional call (`a.push?.()`) on a plain
runtime array refuse too.

**Architecture:** The shared method lists and message wording live in
`kali_common/src/messages.rs`. The type layer (`kali_types/src/resolve/member.rs`)
gets one dispatcher covering the plain lane and the new literal lane, and that
dispatcher is what `check` reports. `kali run` runs the same type layer before
codegen (`kali_cli/src/build/compile.rs:768-785`). Codegen gets a matching
literal gate as defence in depth, plus a backstop at the terminal placeholder
fallback (`emit/call.rs`), which is the only route any of these calls is dropped
through (spec amendment A-1).

**Tech Stack:** Rust (`kali_common`, `kali_types`, `kali_codegen`), the `cases`
TOML runner, `tools/array-return-probes/run.sh`, node v26.10.0 as oracle.

**Spec:** `docs/superpowers/specs/2026-10-03-literal-array-mutators-design.md`.
Read it before starting any task, especially §1 (the claim), §2.1 (the rows),
§3 (the design) and §7 (amendments A-1..A-3).

## Global Constraints

- Baseline commit: `9dc751cf8`. Oracle: `node v26.10.0`. Branch: `literal-array-mutators`.
- No new diagnostic codes. Every refusal is `E5506` (`e5::FEATURE_UNAVAILABLE`).
- No new `tests/*.rs` integration target. Black-box tests are `.toml` cases
  under `crates/kali_cli/tests/cases/`.
- Rust unit tests go in sibling `*_tests.rs` files, never inline `#[cfg(test)]`.
- Growable `push` must keep working: `push`, `.length`, index read, `for…of`
  and `.join` on an in-function binding that `growable_array_candidates` promotes.
- `fill` on a plain runtime array (`new Array(3).fill(4); a.fill(5)`) must keep working.
- Message text is defined once, in `kali_common/src/messages.rs`. Both codegen
  and `kali_types` use it from there.
- No real mutator semantics, no top-level growable promotion (spec §1.1).
- Commits use the project prefix: `feat(literal-array-mutators): …`,
  `test(literal-array-mutators): …`, `docs(literal-array-mutators): …`. End each
  commit with `Co-Authored-By: Claude <noreply@anthropic.com>`.
- `cargo test --workspace` and `cargo clippy --workspace --all-targets` pass
  at the end of every task that touches Rust.

## Review Focus

- **A user object with its own `push` or `sort` method**
  (`const s = { n: 0, push(v){ this.n += v; } }; s.push(2);`) must not refuse
  if it compiles at the baseline. The type gate keys on literal-*array*
  bindings only. The backstop only sees calls that already failed to resolve.
  The `litmut_ok_user_push` probe and the Task 5 unit test pin it.
- **A browser or host API named like a mutator.** Canvas `ctx.fill()` is the
  likely one. If it reached the placeholder fallback at the baseline, the
  backstop now refuses it. The Task 6 spike surfaces any pinned case that
  moves, and a move of this kind is capability loss that goes to the human
  partner, not a re-pin.
- **An object-element literal** (`[{v:1},{v:2}]`) in a function may be a
  structural runtime array, not only a literal binding. It must get exactly
  one refusal, from the plain lane. `is_literal_array_receiver` excludes
  structural and growable bindings, and a Task 3 unit test pins it.
- **`a?.pop()` (optional member) and `a.pop?.()` (optional call)** are
  different parse shapes. Both must refuse, under both commands. Task 3
  pins both.
- **A compound `.length` write** (`a.length -= 1`) on a literal must refuse
  like `a.length = 1`. The type gate sees every assignment operator. Task 3
  pins it.

## File Structure

| file | change |
|---|---|
| `crates/kali_common/src/messages.rs` (+ `messages_tests.rs`) | widened `RUNTIME_ARRAY_MUTATORS`, new `LITERAL_ARRAY_MUTATORS`, reworded runtime message, four new message functions |
| `crates/kali_types/src/resolve/member.rs` (+ `member_tests.rs`) | `is_literal_array_receiver`, the two-lane mutator dispatcher, literal `.length` arm |
| `crates/kali_types/src/resolve/expression.rs` | `reject_literal_array_unfoldable_mutation` uses the shared message |
| `crates/kali_codegen/src/emit/call.rs` (+ `call_tests/literal_array_mutators.rs`) | `literal_array_mutator`, callee unwrapping for optional calls, the backstop |
| `crates/kali_codegen/src/emit/literal.rs` | literal `.length =` refusal |
| `crates/kali_cli/tests/cases/array/literal_array_mutators.toml` | new: literal lane, plain-lane additions, backstop, controls |
| `tools/array-return-probes/run.sh`, `probes/litmut_*.js`, `baseline-litmut.tsv` | the check column, the probes, the baseline |
| `specs/15-errors.md`, followups, new `literal-array-mutators-discovered-defects.md` | bookkeeping |

---

### Task 1: Probes, the check column and the baseline (before any code change)

**Files:**
- Modify: `tools/array-return-probes/run.sh`
- Create: `tools/array-return-probes/probes/litmut_*.js` (one file per probe below)
- Create: `tools/array-return-probes/baseline-litmut.tsv`

**Interfaces:**
- Produces: a TSV with columns `name verdict node_out kali_shown check_exit`.
  Tasks 6 and 7 diff columns 1, 2 and 5 against it.

- [ ] **Step 1: Add the check column to `run.sh`.** After the line
  `kali_err="$(cat /tmp/array-return-probe.err)"`, add:

```bash
  "$kali" check "$p" >/dev/null 2>&1
  check_exit=$?
```

  and change the `printf` at the end of the loop to emit a fifth column:

```bash
  printf '%s\t%s\t%s\t%s\t%s\n' "$name" "$verdict" \
    "$(printf '%s' "$node_out" | tr '\n' '|')" \
    "$(printf '%s' "$shown" | tr '\n' '|')" "$check_exit" >> "$out"
```

  Existing baselines have four columns, and every existing diff uses
  `cut -f1,2`, so they are unaffected.

- [ ] **Step 2: Write the probes.** Each line below is `filename: content`.
  Create each file under `tools/array-return-probes/probes/` holding exactly
  that content plus a trailing newline.

```
litmut_t_push.js: const a=[1,2,3]; a.push(4); console.log(a.length);
litmut_t_pop.js: const a=[1,2,3]; const x=a.pop(); console.log(a.length); console.log(x);
litmut_t_shift.js: const a=[1,2,3]; a.shift(); console.log(a[0]);
litmut_t_unshift.js: const a=[1,2,3]; a.unshift(0); console.log(a[0]);
litmut_t_splice.js: const a=[1,2,3]; a.splice(0,1); console.log(a[0]);
litmut_t_reverse.js: const a=[1,2,3]; a.reverse(); console.log(a[0]);
litmut_t_sort.js: const a=[3,1,2]; a.sort(); console.log(a[0]);
litmut_t_fill.js: const a=[1,2,3]; a.fill(9); console.log(a[0]);
litmut_t_copywithin.js: const a=[1,2,3]; a.copyWithin(0,2); console.log(a[0]);
litmut_t_length.js: const a=[1,2,3]; a.length = 1; console.log(a.length);
litmut_t_empty_push.js: const a=[]; a.push(5); console.log(a.length);
litmut_t_let_push.js: let a=[1,2,3]; a.push(4); console.log(a[0]);
litmut_t_var_pop.js: var a=[1,2,3]; a.pop(); console.log(a[0]);
litmut_f_pop.js: function main(){ const a=[1,2,3]; const x=a.pop(); console.log(a.length); console.log(x); } main();
litmut_f_shift.js: function main(){ const a=[1,2,3]; a.shift(); console.log(a[0]); } main();
litmut_f_unshift.js: function main(){ const a=[1,2,3]; a.unshift(0); console.log(a[0]); } main();
litmut_f_splice.js: function main(){ const a=[1,2,3]; a.splice(0,1); console.log(a[0]); } main();
litmut_f_reverse.js: function main(){ const a=[1,2,3]; a.reverse(); console.log(a[0]); } main();
litmut_f_sort.js: function main(){ const a=[3,1,2]; a.sort(); console.log(a[0]); } main();
litmut_f_fill.js: function main(){ const a=[1,2,3]; a.fill(9); console.log(a[0]); } main();
litmut_f_copywithin.js: function main(){ const a=[1,2,3]; a.copyWithin(0,2); console.log(a[0]); } main();
litmut_f_length.js: function main(){ const a=[1,2,3]; a.length = 1; console.log(a.length); } main();
litmut_f_length_compound.js: function main(){ const a=[1,2,3]; a.length -= 1; console.log(a.length); } main();
litmut_f_str_reverse.js: function main(){ const a=["x","y"]; a.reverse(); console.log(a[0]); } main();
litmut_f_obj_pop.js: function main(){ const a=[{v:1},{v:2}]; a.pop(); console.log(a.length); } main();
litmut_w_paren.js: const a=[1,2,3]; (a).pop(); console.log(a[0]);
litmut_w_as.js: const a=[1,2,3]; (a as number[]).pop(); console.log(a[0]);
litmut_w_strkey.js: const a=[1,2,3]; a["pop"](); console.log(a[0]);
litmut_w_optcall.js: const a=[1,2,3]; a.push?.(4); console.log(a[0]);
litmut_w_optmember.js: const a=[1,2,3]; a?.pop(); console.log(a[0]);
litmut_nameless.js: console.log([1,2].push(3));
litmut_closure.js: function main(){ const a=[1,2,3]; const f=()=>a.pop(); f(); console.log(a[0]); } main();
litmut_alias.js: function main(){ const a=[1,2,3]; const b=a; b.pop(); console.log(a[0]); } main();
litmut_param.js: function g(x){ x.pop(); } function main(){ const a=[1,2,3]; g(a); console.log(a[0]); } main();
litmut_push_pop.js: function main(){ const a=[1,2]; a.push(3); a.pop(); console.log(a[0]); } main();
litmut_plain_reverse.js: function main(){ const a=new Array(3).fill(4); a[0]=1; a.reverse(); console.log(a[0]); } main();
litmut_plain_sort.js: function main(){ const a=new Array(3).fill(4); a[0]=9; a.sort(); console.log(a[0]); } main();
litmut_plain_copywithin.js: function main(){ const a=new Array(3).fill(4); a[2]=7; a.copyWithin(0,2); console.log(a[0]); } main();
litmut_plain_optcall.js: function main(){ const a=new Array(3).fill(4); a.push?.(1); console.log(a[0]); } main();
litmut_ok_growable_push.js: function main(){ const a=[1,2,3]; a.push(4); console.log(a.length); console.log(a[3]); } main();
litmut_ok_empty_push.js: function main(){ const a=[]; a.push(5); console.log(a.length); console.log(a[0]); } main();
litmut_ok_push_loop.js: function main(){ const a=[]; for (let i=0;i<3;i++) a.push(i); let s=0; for (const x of a) s+=x; console.log(s); } main();
litmut_ok_growable_join.js: function main(){ const a=[1,2]; a.push(3); console.log(a.join("-")); } main();
litmut_ok_read.js: const a=[1,2,3]; console.log(a[1]); console.log(a.length);
litmut_ok_slice.js: const a=[3,1,2]; const b=a.slice(1); console.log(b[0]);
litmut_ok_indexof.js: const a=[3,1,2]; console.log(a.indexOf(2));
litmut_ok_plain_fill.js: function main(){ const a=new Array(3).fill(4); a.fill(5); console.log(a[1]); } main();
litmut_ok_user_push.js: const s={ n: 0, push(v){ this.n += v; } }; s.push(2); console.log(s.n);
litmut_ok_class_sort.js: class Box { sort(){ return 7; } } const b = new Box(); console.log(b.sort());
```

  The `litmut_t_*` and wrapped rows log `a[0]` rather than `a.length` where
  possible. A `.length` read on a `let`/`var` literal is itself an unrelated
  pre-existing `E5506` (spec §2.1, `var_push`), and that would hide the row.

- [ ] **Step 3: Record the baseline.** HEAD is still `9dc751cf8` plus docs,
  so the current debug build is the baseline binary.

```bash
cargo build -p kali_cli
SCRATCH=$(mktemp -d)
tools/array-return-probes/run.sh "$SCRATCH/all.tsv"
grep '^litmut_' "$SCRATCH/all.tsv" > tools/array-return-probes/baseline-litmut.tsv
cut -f1,2,5 tools/array-return-probes/baseline-litmut.tsv
```

  Expected:
  * every `litmut_*` row except `litmut_push_pop` is `SILENT` or `OTHER`
  * `litmut_push_pop` is `REFUSES`, from the growable scan
  * every `litmut_ok_*` row is `CORRECT`

  **A `litmut_ok_*` row that is not `CORRECT` at the baseline is not a
  control.** Delete its probe file, delete its row from
  `baseline-litmut.tsv`, and note the deleted name and its baseline verdict in
  the commit message. Do not try to fix it.

  **A non-control row that is `CORRECT` at the baseline** contradicts the
  spec. Stop and report it.

- [ ] **Step 4: Commit**

```bash
git add tools/array-return-probes/run.sh tools/array-return-probes/probes/litmut_*.js tools/array-return-probes/baseline-litmut.tsv
git commit -m "test(literal-array-mutators): probes, a check-exit column, and their baseline at 9dc751cf8

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2: Shared vocabulary

**Files:**
- Modify: `crates/kali_common/src/messages.rs:146-166`
- Test: `crates/kali_common/src/messages_tests.rs`

**Interfaces:**
- Produces:
  * `pub const RUNTIME_ARRAY_MUTATORS: &[&str]`, which is now `["push","pop","shift","unshift","splice","reverse","sort","copyWithin"]`
  * `pub const LITERAL_ARRAY_MUTATORS: &[&str]`, the same eight plus `"fill"`
  * `pub fn runtime_array_mutator_unavailable_message(method: &str) -> String`, reworded
  * `pub fn literal_array_mutator_unavailable_message(method: &str) -> String`
  * `pub const fn literal_array_length_write_unavailable_message() -> &'static str`
  * `pub const fn literal_array_store_unavailable_message() -> &'static str`
  * `pub fn array_mutator_unresolved_receiver_message(method: &str) -> String`

  Check how `messages.rs` items are re-exported (the existing
  `kali_common::runtime_array_mutator_unavailable_message` path works). New
  items must be reachable the same way.

- [ ] **Step 1: Write the failing tests.** In `messages_tests.rs`, replace
  `runtime_array_mutators_are_the_five_fixed_length_breakers` and the
  `runtime_array_mutator_unavailable_message("push")` assertion inside
  `runtime_array_refusal_messages_are_stable`, then append the new tests:

```rust
// in runtime_array_refusal_messages_are_stable, replacing the push assertion:
    assert_eq!(
        runtime_array_mutator_unavailable_message("push"),
        "calling `.push()` on a runtime array is unavailable in the current phase: kali has no lowering of it on this array, so kali refuses rather than silently skip the call"
    );

#[test]
fn runtime_array_mutators_are_the_plain_lane_methods_without_a_lowering() {
    assert_eq!(
        RUNTIME_ARRAY_MUTATORS,
        &["push", "pop", "shift", "unshift", "splice", "reverse", "sort", "copyWithin"]
    );
}

#[test]
fn literal_array_mutators_are_the_runtime_list_plus_fill() {
    let mut expected: Vec<&str> = RUNTIME_ARRAY_MUTATORS.to_vec();
    expected.push("fill");
    assert_eq!(LITERAL_ARRAY_MUTATORS, expected.as_slice());
}

#[test]
fn literal_array_refusal_messages_are_stable() {
    assert_eq!(
        literal_array_mutator_unavailable_message("pop"),
        "calling `.pop()` on a literal array is unavailable in the current phase: kali folds a literal array to its initial elements, so kali refuses rather than silently skip the call"
    );
    assert_eq!(
        literal_array_length_write_unavailable_message(),
        "assigning to `.length` of a literal array is unavailable in the current phase: kali folds a literal array to its initial elements, so kali refuses rather than silently skip the write"
    );
    assert_eq!(
        literal_array_store_unavailable_message(),
        "mutating a literal array is unavailable in the current direct-runtime path; use new Array(n) for runtime mutation"
    );
    assert_eq!(
        array_mutator_unresolved_receiver_message("sort"),
        "calling `.sort()` is unavailable in the current phase: it mutates an array in place and kali could not prove which array the receiver is, so kali refuses rather than silently skip the call"
    );
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p kali_common messages`
Expected: FAIL. The new functions and `LITERAL_ARRAY_MUTATORS` do not exist,
so this is a compile error.

- [ ] **Step 3: Implement.** In `messages.rs`, replace the `RUNTIME_ARRAY_MUTATORS`
  block and `runtime_array_mutator_unavailable_message` (`:146-161`) with:

```rust
/// Methods with no lowering on a plain `[len][elem…]` runtime array: the
/// length changers (the array has a fixed length) and the in-place reorderers.
/// Each refuses on one (array-bounds spec §3.2, literal-array-mutators spec §3.1).
/// `fill` is absent: the plain lane lowers it.
pub const RUNTIME_ARRAY_MUTATORS: &[&str] = &[
    "push", "pop", "shift", "unshift", "splice", "reverse", "sort", "copyWithin",
];

/// In-place mutators refused on a literal array, which kali folds to its
/// initial elements (literal-array-mutators spec §3.1). The runtime list plus
/// `fill`, which the literal lane does not lower either.
pub const LITERAL_ARRAY_MUTATORS: &[&str] = &[
    "push", "pop", "shift", "unshift", "splice", "reverse", "sort", "copyWithin", "fill",
];
```

  Keep `runtime_array_negative_index_unavailable_message` where it is, then:

```rust
/// Canonical wording for a [`RUNTIME_ARRAY_MUTATORS`] call on a plain runtime array.
pub fn runtime_array_mutator_unavailable_message(method: &str) -> String {
    format!(
        "calling `.{method}()` on a runtime array is unavailable in the current phase: kali has no lowering of it on this array, so kali refuses rather than silently skip the call"
    )
}

/// Canonical wording for a [`LITERAL_ARRAY_MUTATORS`] call on a literal array.
pub fn literal_array_mutator_unavailable_message(method: &str) -> String {
    format!(
        "calling `.{method}()` on a literal array is unavailable in the current phase: kali folds a literal array to its initial elements, so kali refuses rather than silently skip the call"
    )
}

/// Canonical wording for an assignment to a literal array's `.length`.
pub const fn literal_array_length_write_unavailable_message() -> &'static str {
    "assigning to `.length` of a literal array is unavailable in the current phase: kali folds a literal array to its initial elements, so kali refuses rather than silently skip the write"
}

/// Canonical wording for an element store into a literal array (`a[0] = 7`).
pub const fn literal_array_store_unavailable_message() -> &'static str {
    "mutating a literal array is unavailable in the current direct-runtime path; use new Array(n) for runtime mutation"
}

/// The `run` backstop's wording: a [`LITERAL_ARRAY_MUTATORS`] name reached the
/// placeholder fallback, so kali cannot tell which array, if any, it mutates.
pub fn array_mutator_unresolved_receiver_message(method: &str) -> String {
    format!(
        "calling `.{method}()` is unavailable in the current phase: it mutates an array in place and kali could not prove which array the receiver is, so kali refuses rather than silently skip the call"
    )
}
```

  The literal list is written out in full, not built from the runtime list,
  because a `const` slice cannot be concatenated. The Step 1 test is what
  keeps the two lists in sync.

- [ ] **Step 4: Run them and watch them pass**

Run: `cargo test -p kali_common messages`
Expected: PASS.

- [ ] **Step 5: Run the dependants**

Run: `cargo test -p kali_types && cargo test -p kali_cli --test cases -- runtime/array_bounds`
Expected: PASS. The array-bounds cases pin the substring
`on a runtime array is unavailable in the current phase`, which the new
wording keeps. If any test pinned the old tail ("the array has a fixed
length"), update it to the new tail and list it in the commit message.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_common/src/messages.rs crates/kali_common/src/messages_tests.rs
git commit -m "feat(literal-array-mutators): shared mutator lists and refusal wording

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3: The `kali_types` gate (what `check` and `run` report)

**Files:**
- Modify: `crates/kali_types/src/resolve/member.rs:395-461`
- Modify: `crates/kali_types/src/resolve/expression.rs:1681-1686`
- Test: `crates/kali_types/src/resolve/member_tests.rs`
- Create: `crates/kali_cli/tests/cases/array/literal_array_mutators.toml`

**Interfaces:**
- Consumes: Task 2's lists and messages.
- Produces:
  * `pub(crate) fn is_literal_array_receiver(&self, object: &Expression) -> bool` on `TypeContext`
  * `reject_runtime_array_mutator_call` and `reject_runtime_array_length_write`
    keep their names and signatures. They now cover both lanes.

- [ ] **Step 1: Write the failing unit tests.** Append to `member_tests.rs`.
  `e5506_messages`, `any_contains`, `MUT` and `LEN` already exist in that file.

```rust
const LIT: &str = "on a literal array is unavailable in the current phase";
const LIT_LEN: &str = "assigning to `.length` of a literal array is unavailable";

#[test]
fn each_in_place_mutator_on_a_literal_array_refuses_at_top_level_and_in_a_function() {
    for call in [
        "a.push(4)", "a.pop()", "a.shift()", "a.unshift(0)", "a.splice(0, 1)",
        "a.reverse()", "a.sort()", "a.fill(9)", "a.copyWithin(0, 2)",
    ] {
        for source in [
            format!("const a = [1,2,3]; {call}; console.log(a[0]);"),
            format!("let a = [1,2,3]; {call}; console.log(a[0]);"),
            format!("var a = [1,2,3]; {call}; console.log(a[0]);"),
            format!("function main(){{ const a = [1,2,3]; {call}; console.log(a[0]); }} main();"),
        ] {
            let messages = e5506_messages(&source);
            assert!(any_contains(&messages, LIT), "{source}: {messages:?}");
        }
    }
}

#[test]
fn a_length_write_on_a_literal_array_refuses_for_every_operator() {
    for write in ["a.length = 1", "a.length -= 1"] {
        for source in [
            format!("const a = [1,2,3]; {write};"),
            format!("function main(){{ const a = [1,2,3]; {write}; }} main();"),
        ] {
            let messages = e5506_messages(&source);
            assert!(any_contains(&messages, LIT_LEN), "{source}: {messages:?}");
        }
    }
}

#[test]
fn wrapped_nameless_and_captured_literal_receivers_refuse() {
    for source in [
        "const a = [1,2,3]; (a).pop();",
        "const a = [1,2,3]; (a as number[]).pop();",
        "const a = [1,2,3]; a[\"pop\"]();",
        "const a = [1,2,3]; a?.pop();",
        "const a = [1,2,3]; a.push?.(4);",
        "console.log([1,2].push(3));",
        "function main(){ const a = [\"x\",\"y\"]; a.reverse(); } main();",
        "function main(){ const a = [1,2,3]; const f = () => a.pop(); f(); } main();",
    ] {
        let messages = e5506_messages(source);
        assert!(any_contains(&messages, LIT), "{source}: {messages:?}");
    }
}

#[test]
fn the_plain_lane_refuses_the_reorderers_and_an_optional_call_but_not_fill() {
    for call in ["a.reverse()", "a.sort()", "a.copyWithin(0, 2)", "a.push?.(1)"] {
        let source =
            format!("function main(){{ const a = new Array(3).fill(4); {call}; }} main();");
        let messages = e5506_messages(&source);
        assert!(any_contains(&messages, MUT), "{call}: {messages:?}");
        assert!(!any_contains(&messages, LIT), "{call}: {messages:?}");
    }
    let messages =
        e5506_messages("function main(){ const a = new Array(3).fill(4); a.fill(5); } main();");
    assert!(!any_contains(&messages, MUT) && !any_contains(&messages, LIT), "{messages:?}");
}

#[test]
fn an_object_element_literal_gets_exactly_one_mutator_refusal() {
    let messages =
        e5506_messages("function main(){ const a = [{v:1},{v:2}]; a.pop(); } main();");
    let count = messages
        .iter()
        .filter(|m| m.contains(MUT) || m.contains(LIT))
        .count();
    assert_eq!(count, 1, "{messages:?}");
}

#[test]
fn growable_push_reads_and_non_mutators_on_a_literal_do_not_refuse() {
    for source in [
        "function main(){ const a = [1,2,3]; a.push(4); console.log(a.length); } main();",
        "function main(){ const a = []; for (let i = 0; i < 3; i++) a.push(i); } main();",
        "function main(){ const a = [1,2]; a.push(3); console.log(a.join(\"-\")); } main();",
        "function main(){ const a = [1,2,3]; (a).push(4); } main();",
        "const a = [3,1,2]; const b = a.slice(1); console.log(b[0]);",
        "const a = [3,1,2]; console.log(a.indexOf(2));",
        "const a = [1,2,3]; console.log(a[1]);",
        "const s = { n: 0, push(v){ this.n += v; } }; s.push(2);",
    ] {
        let messages = e5506_messages(source);
        assert!(
            !any_contains(&messages, LIT) && !any_contains(&messages, LIT_LEN),
            "{source}: {messages:?}"
        );
    }
}

#[test]
fn a_push_mixed_with_pop_in_a_function_refuses_the_pop_in_the_resolve_pass() {
    // Spec A-3: the resolve pass reports this before repr_infer's growable reject.
    let messages =
        e5506_messages("function main(){ const a = [1,2]; a.push(3); a.pop(); } main();");
    assert!(
        messages.iter().any(|m| m.contains(LIT) && m.contains("`.pop()`")),
        "{messages:?}"
    );
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p kali_types member_tests`
Expected: the literal-lane tests and the plain-lane reorderer test FAIL.
`growable_push_reads_and_non_mutators_on_a_literal_do_not_refuse` passes
already. It stays as the guard.

- [ ] **Step 3: Implement the predicate and the dispatcher.** In `member.rs`,
  next to `is_plain_runtime_array_receiver`, add:

```rust
    /// A literal-array receiver (literal-array-mutators spec §3.2): an array
    /// literal expression, or a name bound to one in any enclosing scope,
    /// under any `unwrap_transparent` wrapper. A name a runtime lane owns
    /// (growable, or a structural `[len][elem…]` array) is that lane's, so
    /// each call gets exactly one refusal.
    pub(crate) fn is_literal_array_receiver(&self, object: &Expression) -> bool {
        match super::expression::unwrap_transparent(object) {
            Expression::ArrayExpression(_) => true,
            Expression::Identifier(base) => {
                self.resolve_array_literal_binding_name(base)
                    && !self.is_growable_array_binding(base)
                    && !self.is_structural_runtime_array(base)
            }
            _ => false,
        }
    }
```

  `Identifier`'s payload is whatever `is_plain_runtime_array_receiver`
  matches as `base` and passes to `is_structural_runtime_array(base)`. Use it
  the same way. If `ArrayExpression` is spelled differently in `kali_ast`
  (check with `grep -n "ArrayExpression" crates/kali_ast/src/expression.rs`),
  use that spelling.

  Then replace `reject_runtime_array_mutator_call` with:

```rust
    /// An in-place array mutator on a plain runtime array or a literal array
    /// (array-bounds spec §3.3, literal-array-mutators spec §3.2). The callee
    /// is unwrapped first, so an optional call `a.push?.()` is seen as
    /// `a.push`; a string-literal key `a["push"]` carries its name in
    /// `property`.
    pub(crate) fn reject_runtime_array_mutator_call(&mut self, expr: &CallExpression) {
        let Expression::MemberExpression(member) =
            super::expression::unwrap_transparent(&expr.callee)
        else {
            return;
        };
        let Some(method) = member.property.as_deref() else {
            return;
        };
        let message = if kali_common::RUNTIME_ARRAY_MUTATORS.contains(&method)
            && self.is_plain_runtime_array_receiver(&member.object)
        {
            kali_common::runtime_array_mutator_unavailable_message(method)
        } else if kali_common::LITERAL_ARRAY_MUTATORS.contains(&method)
            && self.is_literal_array_receiver(&member.object)
        {
            kali_common::literal_array_mutator_unavailable_message(method)
        } else {
            return;
        };
        self.diagnostics
            .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
    }
```

  and replace the body of `reject_runtime_array_length_write` with:

```rust
        let Expression::MemberExpression(member) = &assign.left else {
            return;
        };
        if member.property.as_deref() != Some("length") {
            return;
        }
        let message = if self.is_plain_runtime_array_receiver(&member.object) {
            kali_common::runtime_array_length_write_unavailable_message()
        } else if self.is_literal_array_receiver(&member.object) {
            kali_common::literal_array_length_write_unavailable_message()
        } else {
            return;
        };
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            message.to_string(),
        ));
```

  Update both doc comments to name the literal lane.

- [ ] **Step 4: Use the shared store message.** In `expression.rs:1683-1686`,
  replace the inline string with
  `kali_common::literal_array_store_unavailable_message().to_string()`.

- [ ] **Step 5: Run the unit tests and watch them pass**

Run: `cargo test -p kali_types member_tests`
Expected: PASS. If `a.push?.(4)` still fails, print the callee's shape
(`eprintln!("{:?}", expr.callee)`) and make the unwrap in Step 3 reach the
`MemberExpression`. The parser wraps the `a.push` member in an
`OptionalChainExpression` (`kali_parser/src/expression/call.rs:228-245`). Remove
the print before committing.

- [ ] **Step 6: Write the case file.** Create
  `crates/kali_cli/tests/cases/array/literal_array_mutators.toml`. `kali run`
  runs the type layer first, so this task's gate is what both commands report.
  The backstop cases are added in Task 5.

```toml
# Cases for the literal-array-mutators project (spec
# docs/superpowers/specs/2026-10-03-literal-array-mutators-design.md).
#
# An in-place mutator on a literal array refuses with E5506 under `check` and
# `run` alike, instead of being silently skipped at exit 0; the plain runtime-
# array lane refuses `reverse`/`sort`/`copyWithin` and an optional call too.
# Growable `push` and every non-mutating use keep node's output. Each
# rationale gives kali's `run` output at the baseline `9dc751cf8`
# (tools/array-return-probes/baseline-litmut.tsv) and node v26.10.0's.
#
# `[source]` keys are one file per program, because `[source]` is file-wide.

[constants]
LIT = "on a literal array is unavailable in the current phase"
LIT_LEN = "assigning to `.length` of a literal array is unavailable"
MUT = "on a runtime array is unavailable in the current phase"

[source]
"t_push.js" = '''
const a=[1,2,3]; a.push(4); console.log(a.length);
'''
"t_pop.js" = '''
const a=[1,2,3]; const x=a.pop(); console.log(a.length); console.log(x);
'''
"t_sort.js" = '''
const a=[3,1,2]; a.sort(); console.log(a[0]);
'''
"t_fill.js" = '''
const a=[1,2,3]; a.fill(9); console.log(a[0]);
'''
"t_length.js" = '''
const a=[1,2,3]; a.length = 1; console.log(a.length);
'''
"t_let_push.js" = '''
let a=[1,2,3]; a.push(4); console.log(a[0]);
'''
"f_shift.js" = '''
function main(){ const a=[1,2,3]; a.shift(); console.log(a[0]); } main();
'''
"f_reverse.js" = '''
function main(){ const a=[1,2,3]; a.reverse(); console.log(a[0]); } main();
'''
"f_length_compound.js" = '''
function main(){ const a=[1,2,3]; a.length -= 1; console.log(a.length); } main();
'''
"w_optcall.js" = '''
const a=[1,2,3]; a.push?.(4); console.log(a[0]);
'''
"w_strkey.js" = '''
const a=[1,2,3]; a["pop"](); console.log(a[0]);
'''
"nameless.js" = '''
console.log([1,2].push(3));
'''
"closure.js" = '''
function main(){ const a=[1,2,3]; const f=()=>a.pop(); f(); console.log(a[0]); } main();
'''
"push_pop.js" = '''
function main(){ const a=[1,2]; a.push(3); a.pop(); console.log(a[0]); } main();
'''
"plain_reverse.js" = '''
function main(){ const a=new Array(3).fill(4); a[0]=1; a.reverse(); console.log(a[0]); } main();
'''
"plain_optcall.js" = '''
function main(){ const a=new Array(3).fill(4); a.push?.(1); console.log(a[0]); } main();
'''
"ok_growable_push.js" = '''
function main(){ const a=[1,2,3]; a.push(4); console.log(a.length); console.log(a[3]); } main();
'''
"ok_push_loop.js" = '''
function main(){ const a=[]; for (let i=0;i<3;i++) a.push(i); let s=0; for (const x of a) s+=x; console.log(s); } main();
'''
"ok_plain_fill.js" = '''
function main(){ const a=new Array(3).fill(4); a.fill(5); console.log(a[1]); } main();
'''
"ok_read.js" = '''
const a=[1,2,3]; console.log(a[1]); console.log(a.length);
'''

[[case]]
name = "a_top_level_literal_push_refuses_under_check"
rationale = """At `9dc751cf8` kali printed `3` at exit 0 where node v26.10.0 prints `4`."""
args = ["check", "t_push.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${LIT}"]

[[case]]
name = "a_top_level_literal_push_refuses_under_run"
rationale = """At `9dc751cf8` kali printed `3` at exit 0 where node v26.10.0 prints `4`."""
args = ["run", "t_push.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${LIT}"]

[[case]]
name = "a_top_level_literal_pop_refuses_under_run"
rationale = """At `9dc751cf8` kali printed `3` then `0` at exit 0 where node v26.10.0 prints `2` then `3`."""
args = ["run", "t_pop.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.pop()`", "${LIT}"]

[[case]]
name = "a_top_level_literal_sort_refuses_under_run"
rationale = """At `9dc751cf8` kali printed `3` at exit 0 where node v26.10.0 prints `1`."""
args = ["run", "t_sort.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.sort()`", "${LIT}"]

[[case]]
name = "a_top_level_literal_fill_refuses_under_run"
rationale = """At `9dc751cf8` kali printed `1` at exit 0 where node v26.10.0 prints `9`. `fill` is refused on the literal lane only."""
args = ["run", "t_fill.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.fill()`", "${LIT}"]

[[case]]
name = "a_top_level_literal_length_write_refuses_under_check"
rationale = """At `9dc751cf8` kali printed `3` at exit 0 where node v26.10.0 prints `1`."""
args = ["check", "t_length.js"]
exit = "failure"
stderr_contains = ["E5506", "${LIT_LEN}"]

[[case]]
name = "a_top_level_literal_length_write_refuses_under_run"
rationale = """At `9dc751cf8` kali printed `3` at exit 0 where node v26.10.0 prints `1`."""
args = ["run", "t_length.js"]
exit = "failure"
stderr_contains = ["E5506", "${LIT_LEN}"]

[[case]]
name = "a_let_bound_literal_push_refuses_under_check"
rationale = """At `9dc751cf8` kali printed `1` at exit 0 (the push dropped) and `check` exited 0; node v26.10.0 prints `1` too, but the array is `[1,2,3,4]`. Refused because the call is dropped."""
args = ["check", "t_let_push.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${LIT}"]

[[case]]
name = "an_in_function_literal_shift_refuses_under_run"
rationale = """At `9dc751cf8` kali printed `1` at exit 0 where node v26.10.0 prints `2`. A literal with no `push` is never promoted to growable."""
args = ["run", "f_shift.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.shift()`", "${LIT}"]

[[case]]
name = "an_in_function_literal_reverse_refuses_under_check"
rationale = """At `9dc751cf8` kali printed `1` at exit 0 where node v26.10.0 prints `3`."""
args = ["check", "f_reverse.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.reverse()`", "${LIT}"]

[[case]]
name = "a_compound_length_write_on_a_literal_refuses_under_run"
rationale = """At `9dc751cf8` kali printed `3` at exit 0 where node v26.10.0 prints `2`."""
args = ["run", "f_length_compound.js"]
exit = "failure"
stderr_contains = ["E5506", "${LIT_LEN}"]

[[case]]
name = "an_optional_call_on_a_literal_refuses_under_check"
rationale = """At `9dc751cf8` kali printed `1` at exit 0 with the push dropped; node v26.10.0 prints `1` with the array grown. Neither gate saw an optional call."""
args = ["check", "w_optcall.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${LIT}"]

[[case]]
name = "a_string_key_mutator_on_a_literal_refuses_under_run"
rationale = """At `9dc751cf8` kali printed `1` at exit 0 (pop dropped); node v26.10.0 prints `1` with the array shortened."""
args = ["run", "w_strkey.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.pop()`", "${LIT}"]

[[case]]
name = "a_mutator_on_a_nameless_literal_refuses_under_run"
rationale = """At `9dc751cf8` kali printed `0` at exit 0 where node v26.10.0 prints `3`."""
args = ["run", "nameless.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${LIT}"]

[[case]]
name = "a_mutator_on_a_captured_literal_refuses_under_check"
rationale = """At `9dc751cf8` kali printed `1` at exit 0 (pop dropped); node v26.10.0 prints `1` with the array shortened. Scope resolution reaches the outer binding."""
args = ["check", "closure.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.pop()`", "${LIT}"]

[[case]]
name = "a_push_mixed_with_pop_refuses_the_pop"
rationale = """At `9dc751cf8` the growable scan refused this with its own E5506. The resolve pass now reports first (spec A-3); still a refusal."""
args = ["run", "push_pop.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.pop()`", "${LIT}"]

[[case]]
name = "reverse_on_a_plain_runtime_array_refuses_under_run"
rationale = """array-bounds followups §2: at `9dc751cf8` kali printed `1` at exit 0 where node v26.10.0 prints `4`."""
args = ["run", "plain_reverse.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.reverse()`", "${MUT}"]

[[case]]
name = "an_optional_push_on_a_plain_runtime_array_refuses_under_check"
rationale = """array-bounds followups §13: at `9dc751cf8` `check` exited 0 and `run` skipped the push at exit 0."""
args = ["check", "plain_optcall.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.push()`", "${MUT}"]

[[case]]
name = "a_growable_push_still_matches_node"
rationale = """Control: kali and node v26.10.0 both print `4` then `4` at `9dc751cf8`."""
args = ["run", "ok_growable_push.js"]
exit = "success"
stdout = "4\n4\n"

[[case]]
name = "a_growable_push_loop_still_matches_node"
rationale = """Control: kali and node v26.10.0 both print `3` at `9dc751cf8`."""
args = ["run", "ok_push_loop.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "fill_on_a_plain_runtime_array_still_matches_node"
rationale = """Control: `fill` is lowered on the plain lane and stays allowed; kali and node v26.10.0 both print `5`."""
args = ["run", "ok_plain_fill.js"]
exit = "success"
stdout = "5\n"

[[case]]
name = "reading_a_literal_still_matches_node"
rationale = """Control: kali and node v26.10.0 both print `2` then `3`."""
args = ["run", "ok_read.js"]
exit = "success"
stdout = "2\n3\n"
```

- [ ] **Step 7: Run the cases**

Run: `cargo test -p kali_cli --test cases -- array/literal_array_mutators`
Expected: PASS. If a control's `stdout` differs from what Task 1's baseline
recorded for its `litmut_ok_*` twin, the baseline is right. Correct the
`stdout` and say so in the commit message.

- [ ] **Step 8: Run the workspace and clippy**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets`
Expected: PASS. If an existing test now fails, **do not re-pin it in this
task.** Record its name and the new output. Task 6 triages every moved test.
Commit this task's work regardless, with the failing names in the commit
message.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_types/src/resolve/member.rs crates/kali_types/src/resolve/member_tests.rs crates/kali_types/src/resolve/expression.rs crates/kali_cli/tests/cases/array/literal_array_mutators.toml
git commit -m "feat(literal-array-mutators): kali check and run refuse an in-place mutator on a literal array

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4: The codegen literal gate (defence in depth)

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs:38-63` (next to `plain_runtime_array_mutator`) and `:1666-1672` (the call site)
- Modify: `crates/kali_codegen/src/emit/literal.rs:659-672`
- Create: `crates/kali_codegen/src/emit/call_tests/literal_array_mutators.rs`
- Modify: `crates/kali_codegen/src/emit/call_tests.rs` (register the new module the way the existing `call_tests/*.rs` modules are registered)

**Interfaces:**
- Consumes: Task 2's lists and messages. Existing helpers:
  `resolve_transparent_callable_node`, `unwrap_transparent`,
  `bare_identifier_name`, `is_growable_array(&str)`, `array_bindings`,
  `resolve_literal_aggregate(LirNodeId) -> Option<LirNodeId>` (`emit/literal.rs:47`),
  `is_array_literal(&LirNode)` (`intrinsics/array.rs:5`), and `deny_e5506`.
- Produces:
  * `pub(crate) fn literal_array_mutator(&self, node: &LirNode) -> Option<String>`
  * `pub(crate) fn is_literal_array_value(&self, id: LirNodeId) -> bool`

Codegen unit tests compile without the type layer
(`emit/computed_member_tests.rs::diagnostics_for`), so they observe this gate
directly. Under `kali run` the type layer refuses first.

- [ ] **Step 1: Write the failing tests.** Create
  `call_tests/literal_array_mutators.rs`:

```rust
use crate::emit::computed_member_tests::{assert_e5506, diagnostics_for};

const LIT: &str = "on a literal array is unavailable in the current phase";
const LIT_LEN: &str = "assigning to `.length` of a literal array is unavailable";
const MUT: &str = "on a runtime array is unavailable in the current phase";

#[test]
fn codegen_refuses_each_mutator_on_a_literal_binding_and_a_nameless_literal() {
    for call in [
        "a.push(4)", "a.pop()", "a.shift()", "a.unshift(0)", "a.splice(0, 1)",
        "a.reverse()", "a.sort()", "a.fill(9)", "a.copyWithin(0, 2)", "(a).pop()",
    ] {
        for source in [
            format!("const a = [1,2,3]; {call}; console.log(a[0]);"),
            format!("function main(){{ const a = [1,2,3]; {call}; console.log(a[0]); }} main();"),
        ] {
            assert_e5506(&diagnostics_for(&source), LIT, &source);
        }
    }
    assert_e5506(&diagnostics_for("console.log([1,2].push(3));"), LIT, "nameless");
}

#[test]
fn codegen_refuses_a_length_write_on_a_literal_binding() {
    let source = "const a = [1,2,3]; a.length = 1; console.log(a[0]);";
    assert_e5506(&diagnostics_for(source), LIT_LEN, source);
}

#[test]
fn codegen_refuses_the_reorderers_and_an_optional_push_on_a_plain_runtime_array() {
    for call in ["a.reverse()", "a.sort()", "a.copyWithin(0, 2)", "a.push?.(1)"] {
        let source =
            format!("function main(){{ const a = new Array(3).fill(4); {call}; }} main();");
        assert_e5506(&diagnostics_for(&source), MUT, &source);
    }
}

#[test]
fn codegen_leaves_growable_push_and_plain_fill_alone() {
    for source in [
        "function main(){ const a = [1,2,3]; a.push(4); console.log(a.length); } main();",
        "function main(){ const a = new Array(3).fill(4); a.fill(5); console.log(a[1]); } main();",
    ] {
        let errors: Vec<_> = diagnostics_for(source)
            .into_iter()
            .filter(|d| d.is_error())
            .collect();
        assert!(errors.is_empty(), "{source}: {errors:?}");
    }
}
```

  Register it next to the other `call_tests` submodules. Check
  `call_tests.rs` for the `mod` lines, e.g. `mod array_iteration;`, and add
  `mod literal_array_mutators;`. If `diagnostics_for`/`assert_e5506` are not
  visible from there, reach them by the path the other `call_tests` modules
  use, or make them `pub(crate)` if they are not already.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p kali_codegen literal_array_mutators`
Expected: FAIL. Without the type layer, the literal rows reach the
placeholder fallback.

- [ ] **Step 3: Implement the literal predicate.** In `call.rs`, after
  `is_runtime_array_value`, add:

```rust
    /// A literal array value (literal-array-mutators spec §3.3): a bare name
    /// bound to an array literal, or an array literal node, under transparent
    /// wrappers. A name a runtime lane owns (`array_bindings`, growable) is
    /// that lane's.
    pub(crate) fn is_literal_array_value(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        if let Some(name) = self.bare_identifier_name(id) {
            if self.array_bindings.contains(&name) || self.is_growable_array(&name) {
                return false;
            }
        }
        self.resolve_literal_aggregate(id)
            .is_some_and(|aggregate| self.is_array_literal(self.node(aggregate)))
    }

    /// A [`kali_common::LITERAL_ARRAY_MUTATORS`] call on a literal array value.
    /// It is checked after the plain gate, and after `growable_push_call_parts`,
    /// which takes a growable `push` first.
    pub(crate) fn literal_array_mutator(&self, node: &LirNode) -> Option<String> {
        let (method, receiver) = self.array_mutator_call_parts(node)?;
        (kali_common::LITERAL_ARRAY_MUTATORS.contains(&method.as_str())
            && self.is_literal_array_value(receiver))
        .then_some(method)
    }

    /// `(method, receiver)` of a member call, with the callee unwrapped so an
    /// optional call `a.push?.()` is seen as `a.push`.
    fn array_mutator_call_parts(&self, node: &LirNode) -> Option<(String, LirNodeId)> {
        if node.kind != LirNodeKind::Call || node.children.is_empty() {
            return None;
        }
        let callee = self.unwrap_transparent(node.children[0]);
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        let method = callee_node.text.clone()?;
        let receiver = self.unwrap_transparent(*callee_node.children.first()?);
        Some((method, receiver))
    }
```

  Then rewrite `plain_runtime_array_mutator` to share the parts helper:

```rust
    pub(crate) fn plain_runtime_array_mutator(&self, node: &LirNode) -> Option<String> {
        let (method, receiver) = self.array_mutator_call_parts(node)?;
        (kali_common::RUNTIME_ARRAY_MUTATORS.contains(&method.as_str())
            && self.is_runtime_array_value(receiver))
        .then_some(method)
    }
```

  If `unwrap_transparent` does not take a `LirNodeId` and return one,
  follow the signature `plain_runtime_array_mutator` already used for its
  receiver and adapt the callee line to match.

- [ ] **Step 4: Call it.** In `emit_call`, right after the plain-mutator
  refusal (`call.rs:1669-1672`), add:

```rust
        if let Some(method) = self.literal_array_mutator(node) {
            let message = kali_common::literal_array_mutator_unavailable_message(&method);
            return self.deny_e5506(function, &message);
        }
```

- [ ] **Step 5: Refuse the literal `.length` write.** In `emit/literal.rs`,
  inside `if let Some(base_name) = self.assignment_target_name(node, base_id) {`
  and before `if self.array_bindings.contains(&base_name) {`, add:

```rust
                        // Literal-array-mutators spec §3.3: a literal's
                        // `.length` write was silently skipped.
                        if matches!(&index, ArrayWriteIndex::Text(text) if text == "length")
                            && self.is_literal_array_value(base_id)
                        {
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                kali_common::literal_array_length_write_unavailable_message()
                                    .to_string(),
                            ));
                            function.instruction(&Instruction::I64Const(0));
                            return true;
                        }
```

- [ ] **Step 6: Run the tests and watch them pass**

Run: `cargo test -p kali_codegen literal_array_mutators`
Expected: PASS.

If `codegen_refuses_a_length_write_on_a_literal_binding` still fails, the
literal's `.length` write does not reach this arm. Find where the assignment
does go: add `eprintln!` at the top of the `"="` arm in `emit/literal.rs` and
print `node.kind` and the target's text. Then put the same check at that
site. Remove the prints.

If `a.push?.(1)` still fails, print the callee chain's `kind`/`text` and make
`array_mutator_call_parts` unwrap down to the member node.

- [ ] **Step 7: Run the workspace, clippy and the cases**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets`
Expected: PASS, apart from any test Task 3 recorded as moved. Record any
newly moved test the same way, without re-pinning it.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_codegen/src/emit/call.rs crates/kali_codegen/src/emit/literal.rs crates/kali_codegen/src/emit/call_tests.rs crates/kali_codegen/src/emit/call_tests/literal_array_mutators.rs
git commit -m "feat(literal-array-mutators): codegen refuses literal-array mutators and an optional mutator call

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5: The `run` backstop

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs`, immediately before `if self.deny_placeholder_lowering(&callee_node, callee_name) {` (about `:3910`)
- Test: `crates/kali_codegen/src/emit/call_tests/literal_array_mutators.rs`
- Modify: `crates/kali_cli/tests/cases/array/literal_array_mutators.toml`

**Interfaces:**
- Consumes: `kali_common::LITERAL_ARRAY_MUTATORS` and
  `kali_common::array_mutator_unresolved_receiver_message`. `callee_node`
  (the resolved callee `LirNode`, whose receiver is in `children[0]`) and
  `callee_name: &str` are already in scope at that line.
- Produces: no new names. Any mutator-named member call that reaches the
  placeholder fallback refuses.

- [ ] **Step 1: Write the failing tests.** Append to
  `call_tests/literal_array_mutators.rs`:

```rust
const UNRESOLVED: &str = "kali could not prove which array the receiver is";

#[test]
fn the_backstop_refuses_a_mutator_on_an_alias_or_a_parameter() {
    for source in [
        "function main(){ const a = [1,2,3]; const b = a; b.pop(); console.log(a[0]); } main();",
        "function g(x){ x.pop(); } function main(){ const a = [1,2,3]; g(a); console.log(a[0]); } main();",
        "var a = [1,2,3]; a.push(4); console.log(a[0]);",
    ] {
        let diagnostics = diagnostics_for(source);
        assert!(
            diagnostics.iter().any(|d| d.is_error()
                && d.code == Some(5506)
                && (d.message.contains(UNRESOLVED) || d.message.contains(LIT))),
            "{source}: {diagnostics:?}"
        );
    }
}

#[test]
fn the_backstop_leaves_a_non_mutator_placeholder_and_a_user_method_alone() {
    for source in [
        "const s = { n: 0, push(v){ this.n += v; } }; s.push(2); console.log(s.n);",
        "class Box { sort(){ return 7; } } const b = new Box(); console.log(b.sort());",
    ] {
        let diagnostics = diagnostics_for(source);
        assert!(
            !diagnostics.iter().any(|d| d.message.contains(UNRESOLVED)),
            "{source}: {diagnostics:?}"
        );
    }
}
```

- [ ] **Step 2: Run them and watch the first fail**

Run: `cargo test -p kali_codegen literal_array_mutators`
Expected: `the_backstop_refuses_a_mutator_on_an_alias_or_a_parameter` FAILS
on the alias and parameter rows. They reach the warn-plus-0 fallback. The
`var` row may already pass through Task 4's literal gate, and that is
accepted by the `LIT` alternative.

- [ ] **Step 3: Implement.** Immediately before the
  `if self.deny_placeholder_lowering(&callee_node, callee_name) {` block, add:

```rust
        // Literal-array-mutators spec §3.4 (amendment A-2): an in-place array
        // mutator that reaches the placeholder fallback was dropped at exit 0
        // (the receiver is never emitted). Whatever the receiver is — an
        // alias, a parameter — refuse rather than skip the call. A user
        // object's own `push`/`sort` resolved far above and never gets here.
        if !callee_node.children.is_empty()
            && kali_common::LITERAL_ARRAY_MUTATORS.contains(&callee_name)
        {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::array_mutator_unresolved_receiver_message(callee_name),
            ));
            for _ in node.children.iter().skip(1) {
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }
```

- [ ] **Step 4: Run the tests and watch them pass**

Run: `cargo test -p kali_codegen literal_array_mutators`
Expected: PASS. If the user-method row fails, the object's method did not
resolve at the baseline either. Check Task 1's `litmut_ok_user_push` row. If
it was deleted as not-a-control, delete this source row from the test and say
so in the commit message. Do not narrow the backstop.

- [ ] **Step 5: Add the backstop cases.** Append to
  `crates/kali_cli/tests/cases/array/literal_array_mutators.toml`. Add the
  constant under `[constants]` and the sources under `[source]`:

```toml
# under [constants]
UNRESOLVED = "kali could not prove which array the receiver is"

# under [source]
"alias.js" = '''
function main(){ const a=[1,2,3]; const b=a; b.pop(); console.log(a[0]); } main();
'''
"param.js" = '''
function g(x){ x.pop(); } function main(){ const a=[1,2,3]; g(a); console.log(a[0]); } main();
'''
```

```toml
[[case]]
name = "a_mutator_through_an_alias_passes_check"
rationale = """The disclosed gap (spec §1.1): the type layer cannot tell which calls codegen will fail to resolve. node v26.10.0 prints `1` with the array shortened."""
args = ["check", "alias.js"]
exit = "success"

[[case]]
name = "a_mutator_through_an_alias_refuses_under_run"
rationale = """At `9dc751cf8` kali printed `1` at exit 0 with the pop dropped. The backstop refuses (spec §3.4)."""
args = ["run", "alias.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.pop()`", "${UNRESOLVED}"]

[[case]]
name = "a_mutator_through_a_parameter_passes_check"
rationale = """The disclosed gap (spec §1.1)."""
args = ["check", "param.js"]
exit = "success"

[[case]]
name = "a_mutator_through_a_parameter_refuses_under_run"
rationale = """At `9dc751cf8` kali printed `1` at exit 0 with the pop dropped. The backstop refuses (spec §3.4)."""
args = ["run", "param.js"]
exit = "failure"
stderr_contains = ["E5506", "calling `.pop()`", "${UNRESOLVED}"]
```

- [ ] **Step 6: Run the cases**

Run: `cargo test -p kali_cli --test cases -- array/literal_array_mutators`
Expected: PASS.

- [ ] **Step 7: Run the workspace and clippy**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets`
Expected: PASS apart from moved tests already recorded. Record any newly moved
test (the backstop is the likeliest source of browser-surface moves, see
Review Focus) without re-pinning it.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_codegen/src/emit/call.rs crates/kali_codegen/src/emit/call_tests/literal_array_mutators.rs crates/kali_cli/tests/cases/array/literal_array_mutators.toml
git commit -m "feat(literal-array-mutators): kali run refuses a mutator call that reaches the placeholder fallback

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 6: The capability-loss spike (gate)

**Files:**
- Modify: whichever tests Step 3 classifies as wanted re-pins
- Create: `$SCRATCH/triage.md` (scratch, not committed; its table goes into Task 7's followups file)

- [ ] **Step 1: Run the whole workspace**

Run: `cargo test --workspace 2>&1 | tee "$SCRATCH/workspace.txt" | grep -E "^test .* FAILED|^failures:|test result"`
List every failing test.

- [ ] **Step 2: Run the probes and diff all four baselines**

```bash
cargo build -p kali_cli
tools/array-return-probes/run.sh "$SCRATCH/final.tsv"
for b in baseline baseline-anon baseline-bounds baseline-litmut; do
  echo "== $b"
  diff <(cut -f1,2 tools/array-return-probes/$b.tsv | sort) \
       <(grep -F -f <(cut -f1 tools/array-return-probes/$b.tsv) "$SCRATCH/final.tsv" | cut -f1,2 | sort)
done
echo "== check column, litmut rows"
grep '^litmut_' "$SCRATCH/final.tsv" | awk -F'\t' '{print $1, $2, "check=" $5}'
```

Expected:
* every non-`ok` `litmut_*` row is `REFUSES`
* `check` is non-zero on all of them except `litmut_alias` and `litmut_param`
* no `litmut_ok_*` row moved
* among the older baselines, nothing moved

  `bounds_*` rows are recorded at `016557d60` and do not include §2/§13 shapes.
  Their `CORRECT` and `REFUSES` rows must be unchanged.

- [ ] **Step 3: Triage every moved test and probe.** For each one, write a
  line `name | before | after | class` into `$SCRATCH/triage.md`. The class is
  one of:
  * **wanted**: it pinned a silent wrong value, or a dropped mutator, at the
    baseline. Re-pin it to the refusal, and quote node's output in the
    rationale.
  * **capability loss**: the program printed node's output at the baseline
    and now refuses.
  * **bug**: anything else.

- [ ] **Step 4: Gate.** Fix every **bug** row in the task that owns the code
  (re-open it). Then sort the **capability loss** rows:
  * A row in one of the classes spec §5.4 already names is accepted under
    array-bounds decision A-5. Those classes are a mutator on a literal whose
    effect is never observed, and a mutator in dead code.
  * **Any other capability-loss row is a STOP.** Examples: a browser or host
    API such as canvas `ctx.fill()`, or a user method that stopped resolving.
    Report those rows with their programs and node/baseline/HEAD output to the
    human partner, and wait for a decision before going on.

- [ ] **Step 5: Measure the expected capability loss.** Run each of these as
  `node P.js`, as `kali run P.js` / `kali check P.js` on HEAD, and on a `kali`
  built from `9dc751cf8` (`git worktree add "$SCRATCH/base" 9dc751cf8 && cargo build -p kali_cli --manifest-path "$SCRATCH/base/Cargo.toml"`).
  Record a table of node / `9dc751cf8` / HEAD for the followups file:

```
const a=[1,2,3]; a.fill(9); console.log("done");
const a=[3,1,2]; a.sort(); console.log("done");
function main(){ const a=[1,2,3]; if (a.length > 5) { a.pop(); } console.log(a[0]); } main();
function f(){ const a=[1,2,3]; a.reverse(); } console.log(1);
```

- [ ] **Step 6: Commit the re-pins**

```bash
git add -A crates/ tools/
git commit -m "test(literal-array-mutators): re-pin cases that pinned a silently dropped mutator

Co-Authored-By: Claude <noreply@anthropic.com>"
```

  Skip the commit if nothing was re-pinned.

---

### Task 7: Bookkeeping

**Files:**
- Modify: `docs/superpowers/followups/array-bounds-discovered-defects.md` (§2, §9, §13)
- Create: `docs/superpowers/followups/literal-array-mutators-discovered-defects.md`
- Modify: `specs/15-errors.md:223`
- Possibly modify: `docs/superpowers/followups/kali-silent-miscompile-register.md`, `blast-radius-ranking.md`

- [ ] **Step 1: Close the three array-bounds followups.** Under each of §2,
  §9 and §13, add a line in this form:

```markdown
**FIXED (fail-closed)** at `<sha of Task 5's commit>` by the literal-array-mutators
project (`docs/superpowers/specs/2026-10-03-literal-array-mutators-design.md`):
the call now refuses with `E5506` under `check` and `run`. It does not run;
node's output is still not produced.
```

- [ ] **Step 2: Write the new followups file**, in the shape of
  `array-bounds-discovered-defects.md`: a header with the oracle, the
  measured-at commit and the register note. Its sections:
  1. The alias/param `check` gap (spec §1.1), with the `litmut_alias` and
     `litmut_param` programs.
  2. Warnings are discarded on a successful build:
     `compile_source_file_uncached` (`kali_cli/src/build/compile.rs:431-517`)
     returns diagnostics only on `Err`. Program:
     `var o={k:1}; console.log(o.zork(4));` makes `kali build --output json`
     report `"warnings":[]` and prints `0`. Not fixed.
  3. Any unresolved member call evaluates to `0` at exit 0 (`o.zork(4)` above,
     `const o={k:1}; console.log(o.zork(4));`). The backstop covers only the
     nine mutator names. Not fixed.
  4. The probe diff from Task 6 Step 2, verbatim, and the triage table.
  5. The measured capability loss from Task 6 Step 5, with the A-5 decision
     restated (or the human partner's new decision, if Step 4 stopped).
  6. Follow-up feature: real mutators on the growable lane and top-level
     promotion. Not started.

- [ ] **Step 3: Widen `specs/15-errors.md:223`.** Replace that bullet with:

```markdown
- on a plain fixed-length runtime array, a negative integer-literal index, a call to `push` / `pop` / `shift` / `unshift` / `splice` / `reverse` / `sort` / `copyWithin` (including an optional call `a.push?.()`), or an assignment to `.length`; and on a literal array (`const a = [1,2,3]`, any binding kind, or a bare array literal), a call to any of those methods or `fill`, or an assignment to `.length` — kali has no lowering that mutates the array, so it refuses under `check` and `run`. On the anonymous-callee lane (`const f = () => [1,2,3]; const a = f();`) only `run` refuses. A mutator called through an alias or a parameter (`const b = a; b.pop()`) is refused by `run` only, and `check` exits 0 (array-bounds spec §3.2-§3.3, amendment A-3; literal-array-mutators spec §3, §1.1; `docs/superpowers/followups/literal-array-mutators-discovered-defects.md` §1)
```

- [ ] **Step 4: Check the register.** Run the oracle cases:

Run: `cargo test -p kali_cli --test cases -- oracle/`
Expected: PASS, with R-21's `r21o` still `silent`. Then search the register
for any §0.2 lane whose repro calls one of the nine mutators on a literal:

```bash
grep -n -E "\.(push|pop|shift|unshift|splice|reverse|sort|fill|copyWithin)\(" docs/superpowers/followups/kali-silent-miscompile-register.md | head -40
```

For each hit that is a §0.2 lane, run its repro on HEAD. If a lane moved
from SILENT, amend the register as earlier projects did, and regenerate the
ranking:

```bash
cargo run -p kali_blast_radius --example rank
cargo test -p kali_blast_radius
```

If nothing moved, state in the new followups file's header that no register
entry moved lane, and leave the register and ranking alone.

- [ ] **Step 5: Re-read for consistency.** Check that the spec, the plan, the
  followups files and `15-errors.md` agree on:
  * the nine method names
  * which lane refuses `fill`
  * the alias/param gap

  If anything changed during implementation, add a spec §7 amendment for it.

- [ ] **Step 6: Final gates**

Run: `cargo test --workspace && cargo clippy --workspace --all-targets`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add docs/ specs/15-errors.md
git commit -m "docs(literal-array-mutators): close array-bounds §2/§9/§13, file what was measured and not fixed, widen E5506 scope

Co-Authored-By: Claude <noreply@anthropic.com>"
```
