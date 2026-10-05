# Block Scoping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A block-scoped `let` / `const` / `class` / function declaration is a
binding of its own, separate from any same-named outer binding. A closure
registered for later inside a loop sees that iteration's bindings. Each shape
kali cannot lower that way refuses with `E5506`; none prints a wrong value at
exit 0.

**Architecture:**
- **An AST rename pass, `kali_cli::build::block_scope_rename`.** It runs in
  `analyze_source_file` after module linking and before monomorphize. It
  rebuilds the scope tree the resolver models, then gives every binding that
  shadows, or shares its frame chain with, a same-named binding a unique
  spelling `<name>{b<N>}`. Every later stage keys by name, so every later
  stage becomes correct without changes.
- **Diagnostics map the spelling back** through
  `kali_common::display_names_in`.
- **Per-iteration closure records.** MIR marks a loop as an *iteration owner*
  when it registers a deferred callback and a closure in it captures one of
  its own `let` / `const`. `derive_env_plans` gives such a loop its own
  `EnvPlan`, keyed `<fn>{iter<N>}`, holding those cells. Codegen allocates a
  fresh record per iteration and, for `for`, copies the head bindings into
  the next record before the update.

**Tech Stack:** Rust workspace (`kali_common`, `kali_cli`, `kali_mir`,
`kali_codegen`), TOML case files, the bash probe runner, and node v26.10.0 as
the oracle.

**Spec:** `docs/superpowers/specs/2026-10-04-block-scoping-design.md`, read
together with its amendments A-1 to A-7 in §6. **The amendments override §3
and §1.1:**
- A-2: an owner loop is one that "contains a deferred registration", not one
  decided by `escape_flow`.
- A-3: there is no `await` / `yield` refusal.
- A-4 and A-5: three run-only refusals.
- A-6: owner loops are matched by cell name, with a backstop.
- A-7: `display_name` applies to diagnostics only, and there is no cache work.

## Global Constraints

- **Baseline:** `6345f082b`. **Oracle:** `node v26.10.0`. Every case rationale
  quotes node's output and kali's output at `6345f082b`.
- **Diagnostic code.** Every refusal is `E5506` (`e5::FEATURE_UNAVAILABLE`).
  There is no new code, flag or schema; `specs/12`, `specs/18` and
  `README.md` stay untouched.
- **Message text.** Every refusal string comes from a function in
  `kali_common::messages` (Task 1). No pass formats its own text.
- **Renamed spelling.** `<name>{b<N>}`, with `N` a per-program counter
  assigned in ascending order of the binding's declaration ordinal. `{` and `}`
  are not identifier characters, so it cannot collide with a source name.
- **Never renamed:**
  - a module-scope binding (scope id 0);
  - a second declaration of a name in the *same* scope (the resolver reports
    E3101);
  - property keys, member names, labels and JSX tag names.
- **No-op guarantee.** For a program in which no binding needs renaming, the
  pass leaves the statements unchanged (`renamed == 0`). A program with no
  iteration owner produces byte-identical wasm, because nothing in
  `kali_codegen` changes for it.
- **Iteration-plan key.** `kali_mir::iteration_label(function_key, n)` returns
  `format!("{function_key}{{iter{n}}}")`, where `function_key` is `""` for the
  module root (as in `env_plan::function_key`) and `n` counts iteration
  *candidates* in pre-order per function.
- **Deferred-registration callees.** Bare `queueMicrotask`, `setTimeout` and
  `setInterval`, and a member `addEventListener`. The single source is
  `kali_common::is_deferred_registration_callee(name, is_member)`.
- **Test layout.** Rust unit tests go in sibling `*_tests.rs` files wired with
  `#[cfg(test)] #[path = "…"] mod …;`. Never use an inline `mod tests {}` in a
  file this plan creates. Existing inline test modules (`env_plan.rs`) may be
  extended where they already live.
- **CLI tests.** CLI tests are `.toml` cases under
  `crates/kali_cli/tests/cases/`. Don't add a `tests/*.rs` target.
- **Stop rule.** If more than about 50 existing trials move in Task 9, or any
  capability loss of class 1 appears (spec §5.4), stop and report before
  re-pinning anything.
- **Commit messages.** Use `feat(block-scoping): …`, `test(block-scoping): …`
  and `docs(block-scoping): …`.

## Review Focus

1. **A reference that precedes its declaration in the same block.** In
   `let x=1; { console.log(typeof g); function g(){} let x=2; }`, the hoisted
   `g` and the block's `x` must resolve to the block bindings, from any point
   in the block. Resolution uses the whole scope table (pass A), not the
   bindings seen so far. Pinned in Task 3 (unit
   `a_use_before_the_block_declaration_resolves_to_the_block_binding`).
2. **A nested function that captures an outer name while declaring the same
   name in its own block.** In
   `function m(){ let x=1; function g(){ { let x=2; } return x; } return g(); }`,
   node prints `1`. `g`'s block `x` is renamed, and `g`'s `return x` must stay
   bound to `m`'s `x`. Pinned in Task 3 (unit) and Task 4 (case
   `inner_function_block_shadow_keeps_the_capture`).
3. **`var` inside a block, alongside a block `let` of the same name in a
   sibling block.** In `function f(){ { var v=1; } { let v=2; } return v; }`,
   node prints `1`. The `var` lives in the function scope and keeps its
   spelling; the sibling `let` is renamed. Pinned in Task 3 (unit) and Task 4
   (case `var_hoists_past_a_sibling_let`).
4. **A `return` from inside an iteration-owner loop in a function that owns
   no record of its own.** `g8` must be restored to the value it had at loop
   entry, or the caller's env is corrupted. In
   `function m(){ for(let i=0;i<3;i++){ queueMicrotask(()=>console.log(i)); if(i===1) return 7; } } function k(){ let c=0; const inc=()=>{c+=1;}; inc(); console.log(m(), c); } k();`,
   node prints `7 1`, then `0`, then `1`. Pinned in Task 8 (case
   `return_from_an_owner_loop_restores_the_caller_env`).
5. **Nested owner loops refuse rather than misread.** In
   `for(let i=0;i<2;i++){ for(let j=0;j<2;j++){ setTimeout(()=>console.log(i,j),0); } }`
   (node prints `0 0`, `0 1`, `1 0`, `1 1`), the outer `i` is two records away
   from the closure, through the inner iteration record. That is the A-4
   depth-2 shape, so `run` must refuse it and never print a wrong pair. Pinned
   in Task 6 (unit
   `nested_owners_put_the_outer_cell_at_depth_two_without_crossing_a_foreign_record`)
   and Task 8 (case `nested_owner_loops_are_refused_at_depth_two`).

---

## File Structure

| file | status | responsibility |
|---|---|---|
| `crates/kali_common/src/display_name.rs` | create | `display_names_in(&str) -> Cow<str>`: strip every `{b<N>}` suffix in a text |
| `crates/kali_common/src/display_name_tests.rs` | create | its tests |
| `crates/kali_common/src/registration.rs` | create | `is_deferred_registration_callee` |
| `crates/kali_common/src/registration_tests.rs` | create | its tests |
| `crates/kali_common/src/messages.rs` | modify | five new message functions |
| `crates/kali_common/src/lib.rs` | modify | `mod` + `pub use` for the two new modules |
| `crates/kali_codegen/src/env_safety.rs` | modify | call the shared predicate |
| `crates/kali_cli/src/build/block_scope_rename/mod.rs` | create | entry point `rename_block_scoped_bindings` |
| `crates/kali_cli/src/build/block_scope_rename/walk.rs` | create | the exhaustive AST walk over a `Hooks` trait |
| `crates/kali_cli/src/build/block_scope_rename/table.rs` | create | pass A: `Collector` builds the `ScopeTable` |
| `crates/kali_cli/src/build/block_scope_rename/plan.rs` | create | the keeper rule: `plan_renames` |
| `crates/kali_cli/src/build/block_scope_rename/apply.rs` | create | pass B: `Renamer` rewrites declarations and references |
| `crates/kali_cli/src/build/block_scope_rename/*_tests.rs` | create | unit tests per file |
| `crates/kali_cli/src/build/mod.rs` | modify | `pub mod block_scope_rename;` |
| `crates/kali_cli/src/build/compile.rs` | modify | call the pass; the `eval` refusal; map diagnostics at the boundary |
| `crates/kali_mir/src/analysis/mod.rs`, `walk.rs`, `scope.rs` | modify | loop frames, candidates, owner decision, parent relabel |
| `crates/kali_mir/src/analysis/iteration.rs` + `iteration_tests.rs` | create | the loop-frame bookkeeping and owner rule |
| `crates/kali_mir/src/program.rs`, `lower.rs` | modify | `MirProgram::iteration_scopes` |
| `crates/kali_mir/src/env_plan.rs` | modify | iteration plans, `iteration_of`, `through_iteration`, `repr_owner`, `iteration_label` |
| `crates/kali_codegen/src/iteration.rs` + `iteration_tests.rs` | create | loop identification, depth-2 refusals, the backstop helper |
| `crates/kali_codegen/src/lower.rs` | modify | locals reservation, module-global exclusion, call the depth-2 refusal |
| `crates/kali_codegen/src/emitter.rs` | modify | `active_iterations`, `emitted_iterations`, `iteration_plans` |
| `crates/kali_codegen/src/emit/control_flow.rs`, `emit/closure_access.rs`, `intrinsics/array.rs`, `intrinsics/host.rs` | modify | emission and capture access |
| `crates/kali_cli/tests/cases/scope/block_shadowing.toml` | create | rename cases |
| `crates/kali_cli/tests/cases/scope/per_iteration.toml` | create | iteration cases |
| `crates/kali_cli/tests/cases/scope/eval_refusal.toml` | create | `--compat eval` cases |
| `tools/array-return-probes/probes/bs_*.js`, `baseline-bs.tsv` | create | probes |
| docs (Task 10) | modify | `specs/15`, `specs/19`, register, three followups files, new followups file |

---

### Task 0: Branch, probes and the baseline (before any code change)

**Files:**
- Create: `tools/array-return-probes/probes/bs_*.js` (37 files, listed below)
- Create: `tools/array-return-probes/baseline-bs.tsv`

**Interfaces:**
- Produces: `baseline-bs.tsv` (columns `name verdict node_out kali_out check_exit`), which Tasks 4, 8 and 9 diff against.

- [ ] **Step 1: Branch**

```bash
git switch -c block-scoping
git log --oneline -2   # expect the spec commit on top of 6345f082b
```

- [ ] **Step 2: Build the baseline binary in a separate worktree**

```bash
git worktree add "$TMPDIR/kali-bs-baseline" 6345f082b
(cd "$TMPDIR/kali-bs-baseline" && cargo build -q -p kali_cli)
export KALI_BASE="$TMPDIR/kali-bs-baseline/target/debug/kali"
"$KALI_BASE" --version
```

- [ ] **Step 3: Write the probes** (one program per file, exactly as below)

```bash
P=tools/array-return-probes/probes
w(){ printf '%s\n' "$2" > "$P/bs_$1.js"; }
w top_if 'let x=1; if(true){ let x=2; console.log(x); } console.log(x);'
w top_block_const 'const x=1; { const x=2; console.log(x); } console.log(x);'
w fn_if 'function f(){ let x=1; if(true){ let x=2; console.log(x); } return x; } console.log(f());'
w fn_param 'function f(c){ { const c=7; console.log(c); } return c; } console.log(f(2));'
w for_let 'let i=100; for(let i=0;i<2;i++){ console.log(i); } console.log(i);'
w for_body 'let s=0; const v=9; for(let i=0;i<2;i++){ const v=i*10; s+=v; } console.log(s, v);'
w while_shadow 'let n=3; let k=0; while(k<2){ let n=k; k++; } console.log(n);'
w arr_shadow 'const a=[1,2,3]; { const a=[9]; console.log(a.length); } console.log(a.length);'
w nested_fn 'let x=1; { let x=2; function g(){ return x; } console.log(g()); } console.log(x);'
w closure_outer 'let x=1; function g(){ return x; } { let x=2; console.log(g()); }'
w sibling 'function f(){ { let a=1; console.log(a); } { let a="s"; console.log(a); } } f();'
w samefn 'function a(){ function h(){ return 1; } return h(); } function b(){ function h(){ return 2; } return h(); } console.log(a(), b());'
w blockfn '{ function h(){ return 1; } console.log(h()); } { function h(){ return 2; } console.log(h()); }'
w inner_capture 'function m(){ let x=1; function g(){ { let x=2; } return x; } return g(); } console.log(m());'
w var_sibling 'function f(){ { var v=1; } { let v=2; } return v; } console.log(f());'
w r_catch_shadow 'let e=1; try { throw 5; } catch(e){ console.log(e); } console.log(e);'
w r_switch_case 'let y=1; switch(2){ case 2: { let y=5; console.log(y); } } console.log(y);'
w r_str_shadow 'let s="a"; if(true){ let s=3; console.log(s+1); } console.log(s);'
w ok_tdz_var 'var x=1; { let y=x+1; var z=y; } console.log(x, z);'
w ok_no_shadow 'function f(a){ let b=a+1; { let c=b*2; return c; } } console.log(f(3));'
w defer1 'for(let i=0;i<3;i++){ queueMicrotask(()=>console.log(i)); }'
w defer2 'function m(){ for(let i=0;i<3;i++){ setTimeout(()=>console.log(i), 0); } } m();'
w defer5 'function m(){ for(let i=0;i<2;i++){ const k=i*3; queueMicrotask(()=>console.log(k)); } } m();'
w defer_while 'function m(){ let k=0; while(k<2){ const v=k*5; queueMicrotask(()=>console.log(v)); k++; } } m();'
w nested_loops 'for(let i=0;i<2;i++){ for(let j=0;j<2;j++){ setTimeout(()=>console.log(i,j),0); } }'
w asyncloop 'async function m(){ for(let i=0;i<2;i++){ await null; setTimeout(()=>console.log(i),0); } } m();'
w r_defer4 'function m(){ for(const x of [5,6]){ queueMicrotask(()=>console.log(x)); } } m();'
w r_loopmix 'function m(){ let a=10; for(let i=0;i<2;i++){ setTimeout(()=>console.log(a+i),0); } } m();'
w r_continue 'function m(){ for(let i=0;i<3;i++){ if(i===1) continue; queueMicrotask(()=>console.log(i)); } } m();'
w r_depth2 'function m(){ let a=1; function mid(){ let b=2; queueMicrotask(()=>console.log(a+b)); } mid(); } m();'
w ok_loopc3 'function m(){ let s=0; for(let i=0;i<3;i++){ const g=()=>i; s+=g(); } return s; } console.log(m());'
w r_loopc1 'let f0=null; let f2=null; for(let i=0;i<3;i++){ const g=()=>i; if(i===0) f0=g; if(i===2) f2=g; } console.log(f0(), f2());'
w r_loopc2 'function m(){ let f0; let f2; for(let i=0;i<3;i++){ const g=()=>i; if(i===0) f0=g; if(i===2) f2=g; } return f0()*10+f2(); } console.log(m());'
w r_loopc4 'function m(){ let f0; for(let i=0;i<3;i++){ const k=i*2; const g=()=>k; if(i===0) f0=g; } return f0(); } console.log(m());'
w r_loopc5 'function m(){ let f0; for(const x of [5,6,7]){ const g=()=>x; if(x===5) f0=g; } return f0(); } console.log(m());'
w r_defer3 'for(let i=0;i<2;i++){ Promise.resolve().then(()=>console.log("p", i)); }'
w return_restores 'function m(){ for(let i=0;i<3;i++){ queueMicrotask(()=>console.log(i)); if(i===1) return 7; } } function k(){ let c=0; const inc=()=>{c+=1;}; inc(); console.log(m(), c); } k();'
ls "$P"/bs_*.js | wc -l   # expect 37
```

- [ ] **Step 4: Record the baseline**

```bash
KALI="$KALI_BASE" tools/array-return-probes/run.sh "$TMPDIR/bs-all.tsv"
grep '^bs_' "$TMPDIR/bs-all.tsv" > tools/array-return-probes/baseline-bs.tsv
cut -f1,2,5 tools/array-return-probes/baseline-bs.tsv
```

Expected verdicts:
- **SILENT:** `bs_top_if`, `bs_top_block_const`, `bs_fn_if`, `bs_fn_param`, `bs_for_let`, `bs_for_body`, `bs_while_shadow`, `bs_arr_shadow`, `bs_nested_fn`, `bs_closure_outer`, `bs_defer1`, `bs_defer2`, `bs_defer5`, `bs_asyncloop`, `bs_r_loopmix`.
- **REFUSES:** `bs_sibling`, `bs_r_*` (except `r_loopmix`).
- **OTHER (E4201):** `bs_samefn`, `bs_blockfn`.
- **CORRECT:** `bs_ok_*`.

Rows not yet measured in the spec (`inner_capture`, `var_sibling`, `defer_while`, `nested_loops`, `r_continue`, `return_restores`) are recorded as found. **Do not edit a probe to fit an expectation.** If a row differs from the list above, note it in the commit message.

- [ ] **Step 5: Commit**

```bash
git add tools/array-return-probes/probes/bs_*.js tools/array-return-probes/baseline-bs.tsv
git commit -m "test(block-scoping): add bs_* probes and their baseline at 6345f082b"
```

---

### Task 1: Shared helpers in `kali_common` (display names, registration callees, messages)

**Files:**
- Create: `crates/kali_common/src/display_name.rs`, `display_name_tests.rs`, `registration.rs`, `registration_tests.rs`
- Modify: `crates/kali_common/src/lib.rs`, `crates/kali_common/src/messages.rs`, `crates/kali_codegen/src/env_safety.rs:158-179`

**Interfaces:**
- Produces:
  - `pub fn display_names_in(text: &str) -> std::borrow::Cow<'_, str>`
  - `pub fn is_deferred_registration_callee(name: &str, is_member: bool) -> bool`
  - `pub fn block_scope_eval_refused_message() -> &'static str`
  - `pub fn iteration_capture_through_record_message(name: &str, capturer: &str) -> String`
  - `pub fn iteration_for_continue_message() -> &'static str`
  - `pub fn iteration_unrolled_for_of_message(name: &str) -> String`
  - `pub fn iteration_record_unplaced_message(label: &str) -> String`

- [ ] **Step 1: Write the failing tests**

`crates/kali_common/src/display_name_tests.rs`:
```rust
use super::*;

#[test]
fn a_plain_text_is_returned_borrowed() {
    let text = "binding `x` in `f` is used as both a string and a number";
    assert!(matches!(display_names_in(text), std::borrow::Cow::Borrowed(_)));
}

#[test]
fn every_rename_suffix_is_stripped() {
    assert_eq!(
        display_names_in("binding `x{b3}` in `f{b12}` and `y{b0}`"),
        "binding `x` in `f` and `y`"
    );
}

#[test]
fn a_monomorphized_clone_keeps_its_own_suffix() {
    assert_eq!(display_names_in("`h{b1}${0}`"), "`h${0}`");
}

#[test]
fn braces_that_are_not_a_rename_suffix_are_kept() {
    assert_eq!(display_names_in("{b} {b1} x{bb1} x{b1x}"), "{b} {b1} x{bb1} x{b1x}");
}
```
(A suffix counts only when it directly follows an identifier character (alphanumeric, `_` or `$`) and has the form `{b` + one or more digits + `}`.)

`crates/kali_common/src/registration_tests.rs`:
```rust
use super::*;

#[test]
fn bare_scheduling_callees_register() {
    for name in ["queueMicrotask", "setTimeout", "setInterval"] {
        assert!(is_deferred_registration_callee(name, false), "{name}");
        assert!(!is_deferred_registration_callee(name, true), "member {name}");
    }
}

#[test]
fn only_a_member_add_event_listener_registers() {
    assert!(is_deferred_registration_callee("addEventListener", true));
    assert!(!is_deferred_registration_callee("addEventListener", false));
}

#[test]
fn other_callees_do_not_register() {
    for name in ["test", "then", "forEach", "setImmediate", ""] {
        assert!(!is_deferred_registration_callee(name, false));
        assert!(!is_deferred_registration_callee(name, true));
    }
}
```

- [ ] **Step 2: Wire the modules and run the tests to see them fail**

In `crates/kali_common/src/lib.rs`, next to `mod messages; pub use messages::*;`:
```rust
mod display_name;
pub use display_name::display_names_in;
mod registration;
pub use registration::is_deferred_registration_callee;
```
Create both source files containing only the test wiring:
```rust
#[cfg(test)]
#[path = "display_name_tests.rs"]
mod display_name_tests;
```
(and likewise `registration_tests.rs`).

Run: `cargo test -p kali_common display_name registration`
Expected: compile FAIL, `cannot find function display_names_in`.

- [ ] **Step 3: Implement**

`crates/kali_common/src/display_name.rs` (above the test wiring):
```rust
//! The written spelling of a binding the block-scope rename pass
//! (`kali_cli::build::block_scope_rename`) renamed to `<name>{b<N>}`.
//! `{` and `}` are not identifier characters, so the suffix never occurs in a
//! source name; it is stripped wherever kali shows a name to a person.

use std::borrow::Cow;

/// `text` with every `{b<digits>}` that directly follows an identifier
/// character removed. Borrowed when there is nothing to strip.
pub fn display_names_in(text: &str) -> Cow<'_, str> {
    if !text.contains("{b") {
        return Cow::Borrowed(text);
    }
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut changed = false;
    while i < bytes.len() {
        if let Some(len) = suffix_len_at(bytes, i) {
            changed = true;
            i += len;
            continue;
        }
        let ch = text[i..].chars().next().expect("in bounds");
        out.push(ch);
        i += ch.len_utf8();
    }
    if changed { Cow::Owned(out) } else { Cow::Borrowed(text) }
}

fn suffix_len_at(bytes: &[u8], i: usize) -> Option<usize> {
    if i == 0 || bytes.get(i) != Some(&b'{') || bytes.get(i + 1) != Some(&b'b') {
        return None;
    }
    let prev = bytes[i - 1];
    if !(prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'$') {
        return None;
    }
    let digits = bytes[i + 2..].iter().take_while(|b| b.is_ascii_digit()).count();
    if digits == 0 || bytes.get(i + 2 + digits) != Some(&b'}') {
        return None;
    }
    Some(3 + digits)
}
```

`crates/kali_common/src/registration.rs`:
```rust
//! The callees whose call registers its callback argument for a later
//! host-driven invocation, with the env active at the call. One list, read by
//! MIR (per-iteration env records, block-scoping spec A-2) and by codegen's
//! dynamic-env safety gate (`kali_codegen/src/env_safety.rs`).

/// True when a call to `name` registers a callback for later: a bare
/// `queueMicrotask` / `setTimeout` / `setInterval`, or a member
/// `addEventListener`.
pub fn is_deferred_registration_callee(name: &str, is_member: bool) -> bool {
    if is_member {
        name == "addEventListener"
    } else {
        matches!(name, "queueMicrotask" | "setTimeout" | "setInterval")
    }
}
```

Append to `crates/kali_common/src/messages.rs`:
```rust
/// Block-scoping spec §3.4: `--compat eval` with a renamed binding.
pub const fn block_scope_eval_refused_message() -> &'static str {
    "a block-scoped binding that shadows another binding is unavailable with `--compat eval` in the current phase: `eval` code could name the shadowed binding, and kali gives block-scoped bindings separate storage"
}

/// Block-scoping spec A-4.
pub fn iteration_capture_through_record_message(name: &str, capturer: &str) -> String {
    format!(
        "a closure `{capturer}` in a loop that captures `{name}` through a per-iteration record is unavailable in the current phase: `{name}` belongs to the enclosing function, two records away; move `{name}` into the loop or pass it as an argument"
    )
}

/// Block-scoping spec A-5, first bullet.
pub const fn iteration_for_continue_message() -> &'static str {
    "`continue` in a `for` loop whose bindings a registered callback captures is unavailable in the current phase: `continue` would skip the copy into the next iteration's record"
}

/// Block-scoping spec A-5, second bullet.
pub fn iteration_unrolled_for_of_message(name: &str) -> String {
    format!(
        "a callback registered in a `for…of` over a compile-time iterable that captures `{name}` is unavailable in the current phase: kali unrolls this loop and `{name}` has no per-iteration storage"
    )
}

/// Block-scoping spec A-6: the backstop.
pub fn iteration_record_unplaced_message(label: &str) -> String {
    format!(
        "the per-iteration closure record `{label}` was planned but no loop declared its bindings; kali refuses rather than let callbacks share one record. This is unavailable in the current phase"
    )
}
```

- [ ] **Step 4: Route `env_safety` through the shared predicate**

In `crates/kali_codegen/src/env_safety.rs`, keep both function names and their doc comments, and replace the bodies:
```rust
fn is_scheduling_registration_callee(nodes: &[LirNode], callee: LirNodeId) -> bool {
    nodes.get(callee.0 as usize).is_some_and(|node| {
        node.children.is_empty()
            && node
                .text
                .as_deref()
                .is_some_and(|name| kali_common::is_deferred_registration_callee(name, false))
    })
}

fn is_event_registration_callee(nodes: &[LirNode], callee: LirNodeId) -> bool {
    nodes.get(callee.0 as usize).is_some_and(|node| {
        !node.children.is_empty()
            && node
                .text
                .as_deref()
                .is_some_and(|name| kali_common::is_deferred_registration_callee(name, true))
    })
}
```

- [ ] **Step 5: Run the tests**

Run: `cargo test -p kali_common display_name registration && cargo test -p kali_codegen env_safety`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_common crates/kali_codegen/src/env_safety.rs
git commit -m "feat(block-scoping): display_names_in, the shared deferred-registration predicate, and the refusal messages"
```

---

### Task 2: The rename walk and the scope table (pass A)

**Files:**
- Create: `crates/kali_cli/src/build/block_scope_rename/mod.rs`, `walk.rs`, `table.rs`, `table_tests.rs`, `test_support.rs`
- Modify: `crates/kali_cli/src/build/mod.rs` (add `pub mod block_scope_rename;` after `pub mod name_anon_functions;`)

**Interfaces:**
- Produces, in `walk.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScopeKind { Module, Function, Block }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindKind { Var, Lexical, Param, FunctionDecl, ClassDecl, FunctionExprId, ClassExprId }

impl BindKind {
    /// Function / class names are keyed program-wide downstream (spec §3.2).
    pub(crate) fn is_program_wide(self) -> bool {
        matches!(self, Self::FunctionDecl | Self::ClassDecl | Self::FunctionExprId | Self::ClassExprId)
    }
}

pub(crate) trait Hooks {
    fn enter(&mut self, kind: ScopeKind);
    fn exit(&mut self);
    fn bind(&mut self, name: &mut String, kind: BindKind);
    fn reference(&mut self, name: &mut String);
}

pub(crate) fn walk_program(statements: &mut [Statement], hooks: &mut impl Hooks);
```
- Produces, in `table.rs`:
```rust
pub(crate) type ScopeId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Binding { pub ordinal: u32, pub kind: BindKind }

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Scope {
    pub parent: Option<ScopeId>,
    pub kind: ScopeKind,
    /// The nearest Module/Function scope (itself for those kinds).
    pub frame: ScopeId,
    /// Function nesting level of `frame`: module 0, a top-level function 1, …
    pub frame_level: u32,
    /// Block nesting depth inside `frame` (the frame scope itself is 0).
    pub depth: u32,
    pub bindings: BTreeMap<String, Binding>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ScopeTable { pub scopes: Vec<Scope> }

impl ScopeTable {
    /// The scope that binds `name` as seen from `from`, walking parents.
    pub(crate) fn resolve(&self, from: ScopeId, name: &str) -> Option<ScopeId>;
    /// Where a `bind(kind)` issued in `current` lands: `var` goes to the
    /// frame, everything else to `current`.
    pub(crate) fn target_scope(&self, current: ScopeId, kind: BindKind) -> ScopeId;
}

#[derive(Default)]
pub(crate) struct Collector { /* table, stack, next ordinal */ }
impl Hooks for Collector { … }
impl Collector { pub(crate) fn finish(self) -> ScopeTable; }
```
Scope ids are assigned in the order `enter` is called, so pass A and pass B number scopes identically.

- [ ] **Step 1: Write the test support and the failing table tests**

`crates/kali_cli/src/build/block_scope_rename/test_support.rs`:
```rust
use kali_ast::Statement;
use kali_common::FileId;
use kali_lexer::Lexer;
use kali_parser::Parser;

pub(crate) fn parse(source: &str) -> Vec<Statement> {
    let lexed = Lexer::new(FileId::new(0), source.to_string()).lex_all();
    assert!(lexed.diagnostics.is_empty(), "lex diagnostics: {:?}", lexed.diagnostics);
    let mut parser = Parser::new(FileId::new(0), lexed.tokens);
    let parsed = parser.parse(None);
    assert!(parsed.diagnostics.is_empty(), "parse diagnostics: {:?}", parsed.diagnostics);
    parsed.statements
}

pub(crate) fn table_of(source: &str) -> crate::build::block_scope_rename::table::ScopeTable {
    let mut statements = parse(source);
    let mut collector = crate::build::block_scope_rename::table::Collector::default();
    crate::build::block_scope_rename::walk::walk_program(&mut statements, &mut collector);
    collector.finish()
}
```

`crates/kali_cli/src/build/block_scope_rename/table_tests.rs`:
```rust
use super::*;
use crate::build::block_scope_rename::test_support::table_of;
use crate::build::block_scope_rename::walk::{BindKind, ScopeKind};

fn names(table: &ScopeTable, id: ScopeId) -> Vec<&str> {
    table.scopes[id].bindings.keys().map(String::as_str).collect()
}

#[test]
fn module_block_and_function_scopes_are_numbered_in_entry_order() {
    let t = table_of("let x=1; { let x=2; } function f(a){ { const b=1; } }");
    assert_eq!(t.scopes[0].kind, ScopeKind::Module);
    assert_eq!(names(&t, 0), vec!["f", "x"]);
    assert_eq!(t.scopes[1].kind, ScopeKind::Block);
    assert_eq!(names(&t, 1), vec!["x"]);
    assert_eq!(t.scopes[2].kind, ScopeKind::Function);
    assert_eq!(names(&t, 2), vec!["a"]);
    assert_eq!(t.scopes[3].kind, ScopeKind::Block);
    assert_eq!(t.scopes[3].depth, 1);
    assert_eq!(t.scopes[3].frame, 2);
    assert_eq!(t.scopes[3].frame_level, 1);
}

#[test]
fn var_hoists_to_the_frame_and_let_stays_in_the_block() {
    let t = table_of("function f(){ { var v=1; let w=2; } }");
    assert_eq!(names(&t, 1), vec!["v"]);
    assert_eq!(names(&t, 2), vec!["w"]);
}

#[test]
fn a_for_head_is_its_own_block_and_the_body_nests_in_it() {
    let t = table_of("for(let i=0;i<2;i++){ const v=i; }");
    assert_eq!(names(&t, 1), vec!["i"]);
    assert_eq!(t.scopes[2].parent, Some(1));
    assert_eq!(names(&t, 2), vec!["v"]);
}

#[test]
fn a_function_declaration_binds_in_its_block_and_its_params_inside() {
    let t = table_of("{ function h(p){ return p; } }");
    assert_eq!(names(&t, 1), vec!["h"]);
    assert_eq!(t.scopes[1].bindings["h"].kind, BindKind::FunctionDecl);
    assert_eq!(names(&t, 2), vec!["p"]);
}

#[test]
fn a_named_function_expression_binds_its_id_inside_itself() {
    let t = table_of("const f = function g(){ return 1; };");
    assert_eq!(names(&t, 0), vec!["f"]);
    assert_eq!(t.scopes[1].bindings["g"].kind, BindKind::FunctionExprId);
}

#[test]
fn imports_bind_in_the_module_scope() {
    let t = table_of(r#"import { a as b } from "./m.js"; import * as ns from "./n.js";"#);
    assert_eq!(names(&t, 0), vec!["b", "ns"]);
}

#[test]
fn a_same_scope_redeclaration_keeps_the_first_binding() {
    let t = table_of("var v=1; var v=2;");
    assert_eq!(t.scopes[0].bindings["v"].ordinal, 0);
}

#[test]
fn resolve_walks_parents_and_target_scope_hoists_var() {
    let t = table_of("let x=1; { let y=2; }");
    assert_eq!(t.resolve(1, "x"), Some(0));
    assert_eq!(t.resolve(1, "y"), Some(1));
    assert_eq!(t.resolve(1, "z"), None);
    assert_eq!(t.target_scope(1, BindKind::Var), 0);
    assert_eq!(t.target_scope(1, BindKind::Lexical), 1);
}
```

`mod.rs` (initial):
```rust
//! Block-scope rename (block-scoping spec §3.1–§3.2). See `rename_block_scoped_bindings`.

pub(crate) mod apply;
pub(crate) mod plan;
pub(crate) mod table;
pub(crate) mod walk;

#[cfg(test)]
pub(crate) mod test_support;
```
For this task, create `apply.rs` and `plan.rs` as empty files so the module compiles.

Run: `cargo test -p kali_cli block_scope_rename::table`
Expected: compile FAIL (`walk_program`, `Collector` undefined).

- [ ] **Step 2: Implement `walk.rs`**

Module doc: copy the "Exhaustiveness" paragraph of `name_anon_functions.rs:20-36`. The checklist is `deny_import_positions_statement` / `_expression` (`module_link.rs:2382`, `:2607`). There is no `_ =>` arm anywhere. The walk:

```rust
use kali_ast::*;

pub(crate) fn walk_program(statements: &mut [Statement], hooks: &mut impl Hooks) {
    hooks.enter(ScopeKind::Module);
    for statement in statements.iter_mut() {
        walk_statement(statement, hooks);
    }
    hooks.exit();
}

fn walk_block(block: &mut BlockStatement, hooks: &mut impl Hooks) {
    hooks.enter(ScopeKind::Block);
    walk_statements(&mut block.body, hooks);
    hooks.exit();
}

fn walk_statements(statements: &mut [Statement], hooks: &mut impl Hooks) {
    for statement in statements.iter_mut() {
        walk_statement(statement, hooks);
    }
}

fn walk_var_decl(decl: &mut VariableDeclaration, hooks: &mut impl Hooks) {
    let kind = if decl.kind == "var" { BindKind::Var } else { BindKind::Lexical };
    for declarator in decl.declarations.iter_mut() {
        hooks.bind(&mut declarator.id, kind);
        if let Some(init) = declarator.init.as_mut() {
            walk_expression(init, hooks);
        }
    }
}

/// Params and body share one Function scope (`function f(c){ const c }` is a
/// redeclaration, not a shadow).
fn walk_function_parts<'p>(
    params: impl Iterator<Item = &'p mut String>,
    body: Option<&mut BlockStatement>,
    own_id: Option<&mut String>,
    hooks: &mut impl Hooks,
) {
    hooks.enter(ScopeKind::Function);
    if let Some(id) = own_id {
        hooks.bind(id, BindKind::FunctionExprId);
    }
    for param in params {
        hooks.bind(param, BindKind::Param);
    }
    if let Some(body) = body {
        walk_statements(&mut body.body, hooks);
    }
    hooks.exit();
}

fn walk_class_body(body: &mut ClassBody, hooks: &mut impl Hooks) {
    for method in body.methods.iter_mut() {
        walk_function_parts(method.params.iter_mut(), method.body.as_deref_mut(), None, hooks);
    }
    for field in body.fields.iter_mut() {
        if let Some(value) = field.value.as_mut() {
            walk_expression(value, hooks);
        }
    }
}
```

`walk_statement`, one arm per `Statement` variant:
- `ExpressionStatement(s)`: walk `s.expression`.
- `BreakStatement(_) | ContinueStatement(_) | DebuggerStatement(_)`: nothing.
- `WithStatement(s)`: walk `object`, then `walk_statement(body)`.
- `ReturnStatement(s)`: walk `argument` if present.
- `LabeledStatement(s)`: walk `body`.
- `IfStatement(s)`: walk `test`, `walk_block(consequent)`, and `walk_block(alternate)` if present.
- `SwitchStatement(s)`: walk `discriminant`; then `enter(Block)`; for each case, walk `test` if present and `walk_statements(consequent)`; then `exit`.
- `ThrowStatement(s)`: walk `argument`.
- `TryStatement(s)`: `walk_block(block)`. If a handler is present: `enter(Block)`, `bind(&mut handler.param, Lexical)`, `walk_block(&mut handler.body)`, `exit`. Then `walk_block(finalizer)` if present.
- `BlockStatement(b)`: `walk_block(b)`.
- `ForStatement(s)`: `enter(Block)`. For `init`: a `VariableDeclaration` goes to `walk_var_decl`, an `Expression` goes to `walk_expression`. Walk `test`, then `update`, then `walk_block(body)`, then `exit`.
- `ForInStatement(s)` and `ForOfStatement(s)`: `enter(Block)`. For `left`: a `VariableDeclaration` goes to `walk_var_decl`, an `Expression` goes to `walk_expression`. Walk `right`, then `walk_statement(body)`, then `exit`.
- `WhileStatement(s)`: walk `test`, then `walk_block(body)`.
- `DoWhileStatement(s)`: `walk_block(body)`, then walk `test`.
- `FunctionDeclaration(f)`: `bind(&mut f.name, FunctionDecl)`, then `walk_function_parts(f.params.iter_mut(), Some(&mut f.body), None)`.
- `ClassDeclaration(c)`: `bind(&mut c.name, ClassDecl)`; if `super_class` is present, `reference(super_class)`; then `walk_class_body`.
- `VariableDeclaration(d)`: `walk_var_decl(d)`.
- `ImportDeclaration(i)`: for each specifier:
  - `Default(n)` and `Namespace(n)`: `bind(n, Lexical)`.
  - `Named(v)` and `Type(v)`: `bind(&mut s.local, Lexical)` for each entry.
  - `SideEffect`: nothing.
- `ExportAll(_)`: nothing.
- `ExportNamed(e)`: if `source` is `None`, `reference(&mut s.local)` for each specifier.
- `ExportDefault(d)`:
  - `Expression(e)`: walk it.
  - `FunctionDeclaration(f)`: if `!f.name.is_empty()`, `bind(name, FunctionDecl)`; then `walk_function_parts`.
  - `ClassDeclaration(c)`: if the name is non-empty, `bind(ClassDecl)`; then the super reference and the body.
- `EnumDeclaration(e)`: `bind(&mut e.name, Lexical)`, then walk each member's `value`.
- `TypeAliasDeclaration(_) | InterfaceDeclaration(_)`: nothing.

`walk_expression`, one arm per `Expression` variant:
- `Identifier(n)`: `reference(n)`.
- No children: `Literal(_)`, `MetaProperty(_)`, `ThisExpression`, `SuperExpression`, `PrivateIdentifier(_)`, `BigIntLiteral(_)`, `JsxEmptyExpression`.
- `BinaryExpression`: `left`, `right`. `UnaryExpression`: `argument`.
- `CallExpression` and `NewExpression`: `callee`, then `args`.
- `MemberExpression`: `object`, and `computed_index` if present. `property` is never touched.
- `ArrayExpression`: each element through `walk_expression_or_spread`, which covers the `Expression`, `Spread` (its `argument`) and `Empty` arms.
- `ObjectExpression`: each property's `value` only. Keys are never touched, so shorthand `{x}` becomes `{x: x{b1}}` (A-1).
- `FunctionExpression(f)`: `walk_function_parts(f.params.iter_mut().map(|p| &mut p.name), f.body.as_deref_mut(), f.id.as_mut())`.
- `ArrowFunctionExpression(a)`: `enter(Function)`; `bind(Param)` for each param; if `a.id` is `Some`, `bind(id, FunctionExprId)`; walk `a.body`; `exit`.
- `ClassExpression(c)`: `enter(Block)`; if `c.id` is `Some`, `bind(id, ClassExprId)`; the super reference; `walk_class_body`; `exit`.
- `TemplateLiteral(t)`: `expressions`. `TaggedTemplateExpression(t)`: `tag`, then `template.expressions`.
- One child each: `UpdateExpression` (`argument`), `ParenthesizedExpression` (`expression`), `AwaitExpression` (`argument`), `YieldExpression` (`argument` if present), `ChainExpression` (`expression`), `SpreadElement` and `RestElement` (`argument`), `ImportExpression` (`source`), `DecoratedExpression` (`expression`), `TypeAssertion` and `SatisfiesExpression` (`expression`).
- Two or more children: `AssignmentExpression` (`left`, `right`), `LogicalExpression` (`left`, `right`), `ConditionalExpression` (`test`, `consequent`, `alternate`), `SequenceExpression` (each).
- `OptionalChainExpression(o)`: the `NonNull { object, .. }` arm walks `object`.
- `JsxElement` and `JsxFragment`: through the jsx walkers, which mirror `module_link.rs:2770-2823`. They walk attribute values, spread arguments, container expressions and children. Tag names are never touched.

- [ ] **Step 3: Implement `table.rs`**

```rust
use std::collections::BTreeMap;

use super::walk::{BindKind, Hooks, ScopeKind};

pub(crate) type ScopeId = usize;
// (Binding, Scope and ScopeTable exactly as in Interfaces.)

impl ScopeTable {
    pub(crate) fn resolve(&self, from: ScopeId, name: &str) -> Option<ScopeId> {
        let mut cursor = Some(from);
        while let Some(id) = cursor {
            if self.scopes[id].bindings.contains_key(name) {
                return Some(id);
            }
            cursor = self.scopes[id].parent;
        }
        None
    }

    pub(crate) fn target_scope(&self, current: ScopeId, kind: BindKind) -> ScopeId {
        if kind == BindKind::Var { self.scopes[current].frame } else { current }
    }
}

#[derive(Default)]
pub(crate) struct Collector {
    table: ScopeTable,
    stack: Vec<ScopeId>,
    next_ordinal: u32,
}

impl Collector {
    pub(crate) fn finish(self) -> ScopeTable {
        debug_assert!(self.stack.is_empty(), "unbalanced enter/exit");
        self.table
    }
}

impl Hooks for Collector {
    fn enter(&mut self, kind: ScopeKind) {
        let id = self.table.scopes.len();
        let parent = self.stack.last().copied();
        let (frame, frame_level, depth) = match (kind, parent) {
            (ScopeKind::Module, _) => (id, 0, 0),
            (ScopeKind::Function, Some(p)) => (id, self.table.scopes[p].frame_level + 1, 0),
            (ScopeKind::Function, None) => (id, 1, 0),
            (ScopeKind::Block, Some(p)) => {
                let ps = &self.table.scopes[p];
                (ps.frame, ps.frame_level, ps.depth + 1)
            }
            (ScopeKind::Block, None) => (id, 0, 0),
        };
        self.table.scopes.push(Scope { parent, kind, frame, frame_level, depth, bindings: BTreeMap::new() });
        self.stack.push(id);
    }

    fn exit(&mut self) {
        self.stack.pop();
    }

    fn bind(&mut self, name: &mut String, kind: BindKind) {
        let current = *self.stack.last().expect("bind inside a scope");
        let target = self.table.target_scope(current, kind);
        let ordinal = self.next_ordinal;
        self.next_ordinal += 1;
        self.table.scopes[target]
            .bindings
            .entry(name.clone())
            .or_insert(Binding { ordinal, kind });
    }

    fn reference(&mut self, _name: &mut String) {}
}

#[cfg(test)]
#[path = "table_tests.rs"]
mod table_tests;
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_cli block_scope_rename::table`
Expected: PASS (8 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/src/build/mod.rs crates/kali_cli/src/build/block_scope_rename
git commit -m "feat(block-scoping): the exhaustive scope walk and the scope table (pass A)"
```

---

### Task 3: The keeper rule and the rename (pass B)

**Files:**
- Create: `crates/kali_cli/src/build/block_scope_rename/plan_tests.rs`, `apply_tests.rs`
- Modify: `plan.rs`, `apply.rs` and `mod.rs` in the same directory

**Interfaces:**
- Consumes: `ScopeTable`, `Scope`, `Binding`, `ScopeId`, `Hooks`, `walk_program`, `BindKind::is_program_wide` (Task 2)
- Produces:
```rust
// plan.rs
pub(crate) type RenamePlan = BTreeMap<(ScopeId, String), String>;
pub(crate) fn plan_renames(table: &ScopeTable) -> RenamePlan;
// apply.rs
pub(crate) struct Renamer<'a> { /* table, plan, stack, next scope id */ }
impl<'a> Renamer<'a> { pub(crate) fn new(table: &'a ScopeTable, plan: &'a RenamePlan) -> Self; }
// mod.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenameOutcome { pub renamed: usize }
pub fn rename_block_scoped_bindings(statements: &mut [Statement]) -> RenameOutcome;
```

**The rule (spec §3.2, A-1).** Write it as the doc comment on `plan_renames`.
- A binding `b = (s, name)` with `s != 0` (not module scope) is renamed if either rule below says so. Module-scope bindings are never renamed.
- **Variable rule.** `chain(s)` = `s`'s frame and every enclosing frame. The rivals are every other binding of `name` whose scope's frame is in `chain(s)`, excluding a binding in scope `s` itself. `b` is renamed if any rival ranks lower, where rank is `(frame_level, depth, ordinal)` and lower wins. Enclosing frames have a lower `frame_level`, so they always win.
- **Program-wide rule** (only when `b.kind.is_program_wide()`). The rivals are every other program-wide binding of `name` anywhere. `b` is renamed if any rival ranks lower, where rank is `(scope != 0, ordinal)`: a top-level one first, then the earliest.
- **Name.** Renamed bindings sorted by `ordinal` get `format!("{name}{{b{n}}}")`, with `n` = 0, 1, 2, ….

- [ ] **Step 1: Write the failing tests**

`plan_tests.rs`:
```rust
use super::*;
use crate::build::block_scope_rename::test_support::table_of;

fn renamed(source: &str) -> Vec<(ScopeId, String, String)> {
    plan_renames(&table_of(source))
        .into_iter()
        .map(|((s, n), new)| (s, n, new))
        .collect()
}

#[test]
fn nothing_shadows_nothing_is_renamed() {
    assert!(renamed("function f(a){ let b=a+1; { let c=b*2; return c; } }").is_empty());
    assert!(renamed("function a(){ let i=0; } function b(){ let i=1; }").is_empty());
}

#[test]
fn a_block_shadow_of_a_module_binding_is_renamed() {
    assert_eq!(renamed("let x=1; { let x=2; }"), vec![(1, "x".into(), "x{b0}".into())]);
}

#[test]
fn a_block_shadow_of_a_parameter_is_renamed() {
    assert_eq!(renamed("function f(c){ { const c=7; } }"), vec![(2, "c".into(), "c{b0}".into())]);
}

#[test]
fn among_siblings_the_first_keeps_its_name() {
    assert_eq!(
        renamed("function f(){ { let a=1; } { let a=\"s\"; } }"),
        vec![(3, "a".into(), "a{b0}".into())]
    );
}

#[test]
fn a_shallower_sibling_branch_keeps_its_name() {
    assert_eq!(
        renamed("function f(){ { { let a=1; } } { let a=2; } }"),
        vec![(3, "a".into(), "a{b0}".into())]
    );
}

#[test]
fn var_in_a_block_keeps_its_name_against_a_sibling_let() {
    assert_eq!(
        renamed("function f(){ { var v=1; } { let v=2; } return v; }"),
        vec![(3, "v".into(), "v{b0}".into())]
    );
}

#[test]
fn a_nested_function_parameter_sharing_a_module_name_is_renamed() {
    assert_eq!(renamed("const n=1; function f(n){ return n; }"), vec![(1, "n".into(), "n{b0}".into())]);
}

#[test]
fn same_named_nested_functions_are_renamed_program_wide() {
    let r = renamed("function a(){ function h(){ return 1; } return h(); } function b(){ function h(){ return 2; } return h(); }");
    assert_eq!(r, vec![(3, "h".into(), "h{b0}".into())]);
}

#[test]
fn a_top_level_function_keeps_its_name_against_a_nested_one() {
    let r = renamed("function g(){ function h(){} } function h(){}");
    assert_eq!(r, vec![(1, "h".into(), "h{b0}".into())]);
}

#[test]
fn a_same_scope_redeclaration_is_left_for_the_resolver() {
    assert!(renamed("function f(){ let a=1; let a=2; }").is_empty());
}

#[test]
fn numbering_follows_declaration_order() {
    let r = renamed("let x=1; let y=1; { let y=2; } { let x=2; }");
    assert_eq!(r, vec![(1, "y".into(), "y{b0}".into()), (2, "x".into(), "x{b1}".into())]);
}
```

`apply_tests.rs` (asserts on the rewritten AST through `serde_json`, because there is no AST printer):
```rust
use crate::build::block_scope_rename::rename_block_scoped_bindings;
use crate::build::block_scope_rename::test_support::parse;

fn renamed_json(source: &str) -> (usize, String) {
    let mut statements = parse(source);
    let outcome = rename_block_scoped_bindings(&mut statements);
    (outcome.renamed, serde_json::to_string(&statements).expect("serialize"))
}

#[test]
fn a_program_that_shadows_nothing_is_unchanged() {
    let source = "function f(a){ let b=a+1; { let c=b*2; return c; } } console.log(f(3));";
    let before = parse(source);
    let mut after = parse(source);
    assert_eq!(rename_block_scoped_bindings(&mut after).renamed, 0);
    assert_eq!(after, before);
}

#[test]
fn declaration_and_every_reference_in_the_block_are_rewritten() {
    let (n, json) = renamed_json("let x=1; { let x=2; console.log(x); } console.log(x);");
    assert_eq!(n, 1);
    assert_eq!(json.matches("\"x{b0}\"").count(), 2, "{json}");
    assert_eq!(json.matches("\"x\"").count(), 2, "{json}");
}

#[test]
fn a_use_before_the_block_declaration_resolves_to_the_block_binding() {
    let (_, json) = renamed_json("let x=1; function g(){} { console.log(x); g(); function g(){} let x=2; }");
    assert_eq!(json.matches("\"x{b").count(), 2, "{json}");
    assert_eq!(json.matches("\"g{b").count(), 2, "{json}");
}

#[test]
fn an_inner_function_keeps_its_capture_when_its_block_shadows() {
    let (_, json) = renamed_json("function m(){ let x=1; function g(){ { let x=2; } return x; } return g(); }");
    assert_eq!(json.matches("\"x{b0}\"").count(), 1, "{json}");
}

#[test]
fn a_shorthand_property_keeps_its_key() {
    let (_, json) = renamed_json("let x=1; { const x=2; const o={x}; }");
    assert!(json.contains("\"Identifier\":\"x\""), "{json}");
    assert!(json.contains("\"x{b0}\""), "{json}");
}

#[test]
fn a_member_name_and_a_label_are_never_renamed() {
    let (_, json) = renamed_json("let x=1; { const x={x:1}; console.log(x.x); }");
    assert!(json.contains("\"property\":\"x\""), "{json}");
}

#[test]
fn running_twice_is_a_no_op_the_second_time() {
    let mut statements = parse("let x=1; { let x=2; } function a(){ function h(){} } function b(){ function h(){} }");
    rename_block_scoped_bindings(&mut statements);
    let first = statements.clone();
    assert_eq!(rename_block_scoped_bindings(&mut statements).renamed, 0);
    assert_eq!(statements, first);
}
```
(`running_twice_is_a_no_op` holds because the renamed spellings are distinct, so the second pass finds no rivals.)

Wire both test files with `#[cfg(test)] #[path = "…"] mod …;` at the ends of `plan.rs` and `apply.rs`.

Run: `cargo test -p kali_cli block_scope_rename`
Expected: compile FAIL.

- [ ] **Step 2: Implement `plan.rs`**

```rust
use std::collections::BTreeMap;

use super::table::{ScopeId, ScopeTable};

pub(crate) type RenamePlan = BTreeMap<(ScopeId, String), String>;

/// (The rule, verbatim from this task's "The rule" block.)
pub(crate) fn plan_renames(table: &ScopeTable) -> RenamePlan {
    let all: Vec<(ScopeId, &str, &super::table::Binding)> = table
        .scopes
        .iter()
        .enumerate()
        .flat_map(|(id, scope)| scope.bindings.iter().map(move |(n, b)| (id, n.as_str(), b)))
        .collect();

    let mut chosen: Vec<(u32, ScopeId, &str)> = Vec::new();
    for &(s, name, b) in &all {
        if s == 0 {
            continue;
        }
        let chain = frame_chain(table, s);
        let rank = |id: ScopeId, ord: u32| {
            let sc = &table.scopes[id];
            (sc.frame_level, sc.depth, ord)
        };
        let mine = rank(s, b.ordinal);
        let variable_rival = all.iter().any(|&(o, n, ob)| {
            n == name && o != s && chain.contains(&table.scopes[o].frame) && rank(o, ob.ordinal) < mine
        });
        let program_rival = b.kind.is_program_wide()
            && all.iter().any(|&(o, n, ob)| {
                n == name
                    && o != s
                    && ob.kind.is_program_wide()
                    && ((o != 0) as u8, ob.ordinal) < (1u8, b.ordinal)
            });
        if variable_rival || program_rival {
            chosen.push((b.ordinal, s, name));
        }
    }
    chosen.sort();
    chosen
        .into_iter()
        .enumerate()
        .map(|(n, (_, s, name))| ((s, name.to_string()), format!("{name}{{b{n}}}")))
        .collect()
}

fn frame_chain(table: &ScopeTable, s: ScopeId) -> Vec<ScopeId> {
    let mut chain = Vec::new();
    let mut frame = Some(table.scopes[s].frame);
    while let Some(f) = frame {
        chain.push(f);
        frame = table.scopes[f].parent.map(|p| table.scopes[p].frame);
    }
    chain
}
```
The quadratic scan is fine for current program sizes. If `all.len()` grows past a few thousand, group by name first. That's a performance change, not a behaviour change.

- [ ] **Step 3: Implement `apply.rs` and the entry point**

```rust
use super::plan::RenamePlan;
use super::table::{ScopeId, ScopeTable};
use super::walk::{BindKind, Hooks, ScopeKind};

pub(crate) struct Renamer<'a> {
    table: &'a ScopeTable,
    plan: &'a RenamePlan,
    stack: Vec<ScopeId>,
    next_scope: ScopeId,
}

impl<'a> Renamer<'a> {
    pub(crate) fn new(table: &'a ScopeTable, plan: &'a RenamePlan) -> Self {
        Self { table, plan, stack: Vec::new(), next_scope: 0 }
    }

    fn rewrite(&self, scope: ScopeId, name: &mut String) {
        if let Some(new) = self.plan.get(&(scope, name.clone())) {
            *name = new.clone();
        }
    }
}

impl Hooks for Renamer<'_> {
    fn enter(&mut self, _kind: ScopeKind) {
        self.stack.push(self.next_scope);
        self.next_scope += 1;
    }
    fn exit(&mut self) {
        self.stack.pop();
    }
    fn bind(&mut self, name: &mut String, kind: BindKind) {
        let current = *self.stack.last().expect("bind inside a scope");
        let target = self.table.target_scope(current, kind);
        self.rewrite(target, name);
    }
    fn reference(&mut self, name: &mut String) {
        let current = *self.stack.last().expect("reference inside a scope");
        if let Some(scope) = self.table.resolve(current, name) {
            self.rewrite(scope, name);
        }
    }
}
```

In `mod.rs`:
```rust
use kali_ast::Statement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenameOutcome {
    pub renamed: usize,
}

/// Give every binding that shadows, or shares its frame chain with, a
/// same-named binding the spelling `<name>{b<N>}` (block-scoping spec §3.2,
/// A-1). Runs before monomorphize so every later stage, all of which key by
/// name, sees one binding per spelling. A program that needs no rename is
/// left untouched.
pub fn rename_block_scoped_bindings(statements: &mut [Statement]) -> RenameOutcome {
    let mut collector = table::Collector::default();
    walk::walk_program(statements, &mut collector);
    let table = collector.finish();
    let plan = plan::plan_renames(&table);
    if plan.is_empty() {
        return RenameOutcome { renamed: 0 };
    }
    let mut renamer = apply::Renamer::new(&table, &plan);
    walk::walk_program(statements, &mut renamer);
    RenameOutcome { renamed: plan.len() }
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_cli block_scope_rename`
Expected: PASS (all table, plan and apply tests).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/src/build/block_scope_rename
git commit -m "feat(block-scoping): the keeper rule and the rename pass (pass B)"
```

---

### Task 4: Wire the pass into `analyze_source_file`, map diagnostics, refuse under `--compat eval`

**Files:**
- Modify: `crates/kali_cli/src/build/compile.rs` (around `:668-806`, `:431-517`, `:53`)
- Create: `crates/kali_cli/tests/cases/scope/block_shadowing.toml`, `crates/kali_cli/tests/cases/scope/eval_refusal.toml`
- Modify: `crates/kali_cli/tests/runtime_smoke/build.rs` (the fixture of `build_artifacts_are_deterministic_across_repeated_invocations`, `:10188`)

**Interfaces:**
- Consumes: `rename_block_scoped_bindings`, `RenameOutcome` (Task 3); `display_names_in`, `block_scope_eval_refused_message` (Task 1)
- Produces: `fn display_diagnostics(diagnostics: &mut [Diagnostic])` in `compile.rs`

- [ ] **Step 1: Write the failing cases**

`crates/kali_cli/tests/cases/scope/block_shadowing.toml`:
- `[source]` holds one file per §2.1 probe, with the same program as `tools/array-return-probes/probes/bs_<name>.js`, named `<name>.js`. That covers `top_if`, `top_block_const`, `fn_if`, `fn_param`, `for_let`, `for_body`, `while_shadow`, `arr_shadow`, `nested_fn`, `closure_outer`, `sibling`, `samefn`, `blockfn`, `inner_capture` and `var_sibling`.
- Add these extra files:
```toml
"top_if.ts" = """let x: number = 1; if (true) { let x: number = 2; console.log(x); } console.log(x);
"""
"dup_in_block.js" = """let x = 1; { let x = 2; let x = 3; }
"""
"shadow_refusal.js" = """let s = 1; { let s = "a"; console.log(s + 1); }
"""
```
- One `[[case]]` per program. Node outputs:
  - `top_if` and `top_block_const`: `2\n1\n`
  - `fn_if`: `2\n1\n`
  - `fn_param`: `7\n2\n`
  - `for_let`: `0\n1\n100\n`
  - `for_body`: `10 9\n`
  - `while_shadow`: `3\n`
  - `arr_shadow`: `1\n3\n`
  - `nested_fn`: `2\n1\n`
  - `closure_outer`: `1\n`
  - `sibling`: `1\ns\n`
  - `samefn` and `blockfn`: `1 2\n` and `1\n2\n`
  - `inner_capture`: `1\n`
  - `var_sibling`: `1\n`
  - `top_if.ts`: `2\n1\n`

Example:
```toml
[[case]]
name = "a_block_let_does_not_change_the_module_binding"
rationale = """R-10. node v26.10.0 prints `2` then `1`. At `6345f082b` kali printed `2` then `2` at exit 0 (probe `bs_top_if`, SILENT): the block's `x` and the module's `x` were one module global."""
args = ["run", "top_if.js"]
exit = "success"
stdout = "2\n1\n"
```
Also add a `check` case for `top_if.js` and for `samefn.js`: `args = ["check", …]`, `exit = "success"`.

Name the inner-capture and `var` cases `inner_function_block_shadow_keeps_the_capture` and `var_hoists_past_a_sibling_let`.

Two diagnostic cases show the written name, never `{b`:
```toml
[[case]]
name = "a_redeclaration_in_a_block_names_the_binding_as_written"
rationale = """The rename leaves same-scope duplicates for the resolver; E3101 names `x`, never `x{b0}`."""
args = ["check", "dup_in_block.js"]
exit = "failure"
stderr_contains = ["E3101", "'x'"]
stderr_absent = ["{b"]

[[case]]
name = "a_refusal_on_a_renamed_binding_names_it_as_written"
rationale = """The block's `s` is renamed `s{b0}`; the string-`+` refusal must name `s`."""
args = ["run", "shadow_refusal.js"]
exit = "failure"
stderr_absent = ["{b"]
```
(If `shadow_refusal.js` happens to run successfully after the rename, replace this case with the refusal kali does raise. The assertion that matters is `stderr_absent = ["{b"]` on a failing run.)

`crates/kali_cli/tests/cases/scope/eval_refusal.toml`:
```toml
[source]
"shadow.js" = """let x = 1; { let x = 2; console.log(x); } console.log(eval("x"));
"""
"plain.js" = """let x = 1; console.log(eval("x + 1"));
"""

[[case]]
name = "a_renamed_binding_under_compat_eval_is_refused_by_run"
rationale = """Spec §3.4: direct eval could name the shadowed spelling. node v26.10.0 prints `2` then `1`."""
args = ["run", "--compat", "eval", "shadow.js"]
exit = "failure"
stderr_contains = ["E5506", "with `--compat eval`"]

[[case]]
name = "a_renamed_binding_under_compat_eval_is_refused_by_check"
rationale = """The same refusal, raised in analyze_source_file, so check mirrors it."""
args = ["check", "--compat", "eval", "shadow.js"]
exit = "failure"
stderr_contains = ["E5506", "with `--compat eval`"]

[[case]]
name = "a_program_that_shadows_nothing_under_compat_eval_is_unchanged"
rationale = """Control: no rename, so no refusal. Its output at `6345f082b` is pinned verbatim (measure it in Step 2)."""
args = ["run", "--compat", "eval", "plain.js"]
exit = "success"
stderr_absent = ["with `--compat eval`"]
```

Run: `cargo test -p kali_cli --test cases -- scope/`
Expected: FAIL. `block_shadowing` cases print the wrong values, and `eval_refusal` runs without the refusal.

- [ ] **Step 2: Measure the `plain.js` control at the baseline and pin its stdout**

```bash
printf 'let x = 1; console.log(eval("x + 1"));\n' > "$TMPDIR/plain.js"
"$KALI_BASE" run --compat eval "$TMPDIR/plain.js"; echo "exit=$?"
```
Pin exactly what this prints in the control case, as `stdout` plus `exit`.

- [ ] **Step 3: Call the pass**

In `analyze_source_file`, after the `link_provable_module_namespaces` error check (`compile.rs:~737`) and before `monomorphize_statements`:
```rust
    // Block scoping (block-scoping spec §3.1): give every shadowing binding a
    // unique spelling BEFORE monomorphize, so every later stage, all of which
    // key by name, sees one binding per spelling.
    let rename = crate::build::block_scope_rename::rename_block_scoped_bindings(
        &mut parsed.statements,
    );
    if compat_eval && rename.renamed > 0 {
        return Err(vec![Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            kali_common::block_scope_eval_refused_message(),
        )]);
    }
```

- [ ] **Step 4: Map diagnostics at the boundary**

Add to `compile.rs`:
```rust
/// Show every name the block-scope rename changed as the program wrote it
/// (block-scoping spec §3.5, A-7). Applied once per returned diagnostic list.
fn display_diagnostics(diagnostics: &mut [Diagnostic]) {
    for diagnostic in diagnostics.iter_mut() {
        if let std::borrow::Cow::Owned(text) = kali_common::display_names_in(&diagnostic.message) {
            diagnostic.message = text;
        }
        if let Some(suggestion) = diagnostic.suggestion.as_mut() {
            if let std::borrow::Cow::Owned(text) = kali_common::display_names_in(suggestion) {
                *suggestion = text;
            }
        }
        for note in diagnostic.notes.iter_mut() {
            if let std::borrow::Cow::Owned(text) = kali_common::display_names_in(note) {
                *note = text;
            }
        }
    }
}
```
Rename the current `analyze_source_file` body to `analyze_source_file_inner` (same signature), and add:
```rust
fn analyze_source_file(
    source_path: &Path,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    compat_eval: bool,
    sandbox_policy_attached: bool,
) -> Result<AnalyzedSource, Vec<Diagnostic>> {
    analyze_source_file_inner(source_path, api_surface, runtime_profiles, compat_eval, sandbox_policy_attached)
        .map(|mut analyzed| {
            display_diagnostics(&mut analyzed.diagnostics);
            analyzed
        })
        .map_err(|mut diagnostics| {
            display_diagnostics(&mut diagnostics);
            diagnostics
        })
}
```
Do the same for `compile_source_file_uncached`: rename its body to `compile_source_file_uncached_inner`, and wrap only the `Err` arm (its `Ok` is wasm bytes). Mapping twice is harmless, because the function is idempotent.

- [ ] **Step 5: Run the cases, then the rename unit tests**

Run: `cargo test -p kali_cli --test cases -- scope/ && cargo test -p kali_cli block_scope_rename`
Expected: PASS. If a `block_shadowing` case still fails, debug it with `kali run` on that file and fix the pass. Do not re-pin a case to kali's output.

- [ ] **Step 6: Give the determinism test a shadowed binding**

In `crates/kali_cli/tests/runtime_smoke/build.rs`, find the source string that `build_artifacts_are_deterministic_across_repeated_invocations` (`:10188`) builds. Append a function that shadows a binding, and call it:
```ts
function shadowed(n: number): number { let r = n; { let r = n * 2; n = r; } return r + n; }
console.log(shadowed(3));
```
Run: `cargo test -p kali_cli --test runtime_smoke -- build_artifacts_are_deterministic`
Expected: PASS (the bytes are identical across the repeated builds).

- [ ] **Step 7: Diff the probes**

```bash
tools/array-return-probes/run.sh "$TMPDIR/bs-head.tsv"
diff <(cut -f1,2 tools/array-return-probes/baseline-bs.tsv) <(grep '^bs_' "$TMPDIR/bs-head.tsv" | cut -f1,2)
```
Expected: every §2.1 row (and `inner_capture`, `var_sibling`) is CORRECT. The `defer*` rows are unchanged; Task 8 fixes them.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_cli/src/build/compile.rs crates/kali_cli/tests/cases/scope crates/kali_cli/tests/runtime_smoke/build.rs
git commit -m "feat(block-scoping): rename before monomorphize, show written names in diagnostics, refuse under --compat eval"
```

---

### Task 5: MIR iteration candidates and owners

**Files:**
- Create: `crates/kali_mir/src/analysis/iteration.rs`, `crates/kali_mir/src/analysis/iteration_tests.rs`
- Modify: `crates/kali_mir/src/analysis/mod.rs` (`OwnershipAnalyzer` fields, `new`, `analyze_program_with_arena` return), `crates/kali_mir/src/analysis/walk.rs` (loop arms `:291-340`, the `VarDecl` handling, the `CallExpr` handling, function scope push `:82-165`), `crates/kali_mir/src/program.rs` (`MirProgram`), `crates/kali_mir/src/lower.rs:26`, `crates/kali_mir/src/env_plan.rs` (only `iteration_label`)

**Interfaces:**
- Produces:
```rust
// kali_mir::env_plan (pub, re-exported at the crate root like derive_env_plans)
pub fn iteration_label(function_key: &str, n: usize) -> String { format!("{function_key}{{iter{n}}}") }

// kali_mir::program
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IterationScope {
    /// `iteration_label(function_key, n)`.
    pub label: String,
    /// The owning function's plan key (`""` for the module root).
    pub function: String,
    /// The loop-declared `let` / `const` a loop closure captures, sorted.
    pub cells: Vec<String>,
}
// MirProgram gains: pub iteration_scopes: Vec<IterationScope>,
// and parent_labels gains: each owner's label → Some(enclosing function label)
// (None at the module root), and each closure inside an owner → Some(innermost
// owner label).
```

**The owner rule (A-2).** A loop frame is opened in the `ForStmt`, `WhileStmt`, `DoWhileStmt`, `ForOfStmt` and `ForInStmt` arms of `walk_scope_node`, for every loop kind, `ForInStmt` included.
- **Its label** is `iteration_label(function_key(current), n)`, where `n` is the per-function count of loop frames opened so far. The count is kept in a `BTreeMap<String, usize>` keyed by function key.
- **Declared names.** A `VarDecl` whose text is `"let"` or `"const"`, walked while this frame is the innermost loop frame *of the current function*, adds its declarator names.
- **Closures.** A function scope pushed while the innermost loop frame belongs to the current function records the new label as a closure of that frame (and of every enclosing loop frame of the same function).
- **Registration.** A `CallExpr` whose callee node `c` satisfies `kali_common::is_deferred_registration_callee(c.text, !c.children.is_empty())` sets `has_registration` on every open loop frame of the current function. That is the same callee shape `env_safety` reads on LIR, which HIR lowers to node for node. The unit test pins it on real source.
- **Closing a frame.** The frame becomes an `IterationScope` iff `has_registration` and its `cells` (declared names whose binding in the current function scope has a `captured_by` entry `c` such that `c`, or an ancestor of `c` in `parent_labels`, is one of the frame's closures) is non-empty.
- **After the walk.** Each owner's `parent_labels` entry is set to the enclosing function's label (`None` for the module). Each closure is re-parented to the innermost owner among the loop frames it was recorded in. A closure in no owner keeps its parent.

- [ ] **Step 1: Write the failing tests**

`crates/kali_mir/src/analysis/iteration_tests.rs`:
```rust
use crate::test_support::analyze;

#[test]
fn a_registered_closure_over_a_for_let_makes_the_loop_an_owner() {
    let p = analyze("function m(){ for(let i=0;i<3;i++){ setTimeout(()=>console.log(i), 0); } } m();");
    assert_eq!(p.iteration_scopes.len(), 1);
    let s = &p.iteration_scopes[0];
    assert_eq!(s.label, "m{iter0}");
    assert_eq!(s.function, "m");
    assert_eq!(s.cells, vec!["i".to_string()]);
    assert_eq!(p.parent_labels.get("m{iter0}"), Some(&Some("m".to_string())));
    let closure = p.parent_labels.iter().find(|(_, parent)| parent.as_deref() == Some("m{iter0}"));
    assert!(closure.is_some(), "{:?}", p.parent_labels);
}

#[test]
fn a_module_loop_owner_has_no_parent() {
    let p = analyze("for(let i=0;i<3;i++){ queueMicrotask(()=>console.log(i)); }");
    assert_eq!(p.iteration_scopes[0].label, "{iter0}");
    assert_eq!(p.iteration_scopes[0].function, "");
    assert_eq!(p.parent_labels.get("{iter0}"), Some(&None));
}

#[test]
fn a_body_const_is_a_cell_and_the_loop_counter_is_not_captured() {
    let p = analyze("function m(){ for(let i=0;i<2;i++){ const k=i*3; queueMicrotask(()=>console.log(k)); } } m();");
    assert_eq!(p.iteration_scopes[0].cells, vec!["k".to_string()]);
}

#[test]
fn a_synchronous_closure_does_not_make_an_owner() {
    let p = analyze("function m(){ let s=0; for(let i=0;i<3;i++){ const g=()=>i; s+=g(); } return s; } m();");
    assert!(p.iteration_scopes.is_empty());
}

#[test]
fn a_registration_without_a_loop_capture_does_not_make_an_owner() {
    let p = analyze("function m(){ let a=1; for(let i=0;i<2;i++){ setTimeout(()=>console.log(a), 0); } } m();");
    assert!(p.iteration_scopes.is_empty());
}

#[test]
fn an_add_event_listener_member_call_registers() {
    let p = analyze("function m(t){ for(let i=0;i<2;i++){ t.addEventListener(\"x\", ()=>console.log(i)); } } m(new EventTarget());");
    assert_eq!(p.iteration_scopes.len(), 1);
}

#[test]
fn nested_owner_loops_are_two_owners_and_the_inner_parents_to_the_outer() {
    let p = analyze("for(let i=0;i<2;i++){ for(let j=0;j<2;j++){ setTimeout(()=>console.log(i,j),0); } }");
    let labels: Vec<&str> = p.iteration_scopes.iter().map(|s| s.label.as_str()).collect();
    assert_eq!(labels, vec!["{iter1}", "{iter0}"]); // closed inner-first
    // An owner's parent is the innermost enclosing owner loop of the same
    // function, else the function (None at the module root).
    assert_eq!(p.parent_labels.get("{iter1}"), Some(&Some("{iter0}".to_string())));
    assert_eq!(p.parent_labels.get("{iter0}"), Some(&None));
    assert!(p.parent_labels.values().any(|parent| parent.as_deref() == Some("{iter1}")));
}

#[test]
fn a_loop_in_a_nested_function_belongs_to_that_function() {
    let p = analyze("function m(){ function g(){ for(let i=0;i<2;i++){ setTimeout(()=>console.log(i),0); } } g(); } m();");
    assert_eq!(p.iteration_scopes[0].function, "g");
    assert_eq!(p.iteration_scopes[0].label, "g{iter0}");
}
```
An owner's parent is the innermost enclosing owner loop of the same function if there is one, otherwise the function's label (`None` at the module root). `relabel` in Step 2 implements exactly that.

Wire `#[cfg(test)] #[path = "iteration_tests.rs"] mod iteration_tests;` at the end of `iteration.rs`, and add `mod iteration;` in `analysis/mod.rs`.

Run: `cargo test -p kali_mir iteration`
Expected: compile FAIL (`iteration_scopes` undefined).

- [ ] **Step 2: Implement `iteration.rs`**

```rust
//! Per-iteration env records (block-scoping spec §3.3, A-2): which loops own a
//! record of their own. Bookkeeping only; `env_plan` turns the result into
//! plans.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub(crate) struct LoopFrame {
    pub(crate) label: String,
    /// Scope label of the function the loop is in (`"<module>"` at the root).
    pub(crate) scope_label: String,
    pub(crate) declared: BTreeSet<String>,
    pub(crate) closures: BTreeSet<String>,
    pub(crate) has_registration: bool,
}

#[derive(Debug, Default)]
pub(crate) struct IterationCollector {
    pub(crate) stack: Vec<LoopFrame>,
    pub(crate) counters: BTreeMap<String, usize>,
    /// Closure label → the loop labels it was created in, outermost first.
    pub(crate) closure_loops: BTreeMap<String, Vec<String>>,
    /// Owner label → (scope label of its function, enclosing owner candidates outermost first).
    pub(crate) owners: BTreeMap<String, (String, Vec<String>)>,
    pub(crate) scopes: Vec<crate::IterationScope>,
}

impl IterationCollector {
    pub(crate) fn open(&mut self, scope_label: &str, function_key: &str) {
        let n = self.counters.entry(function_key.to_string()).or_insert(0);
        let label = crate::env_plan::iteration_label(function_key, *n);
        *n += 1;
        self.stack.push(LoopFrame {
            label,
            scope_label: scope_label.to_string(),
            declared: BTreeSet::new(),
            closures: BTreeSet::new(),
            has_registration: false,
        });
    }

    fn frames_of<'s>(&'s mut self, scope_label: &'s str) -> impl Iterator<Item = &'s mut LoopFrame> + 's {
        self.stack.iter_mut().filter(move |f| f.scope_label == scope_label)
    }

    pub(crate) fn note_declared(&mut self, scope_label: &str, names: impl IntoIterator<Item = String>) {
        if let Some(frame) = self.stack.iter_mut().rev().find(|f| f.scope_label == scope_label) {
            frame.declared.extend(names);
        }
    }

    pub(crate) fn note_closure(&mut self, enclosing_scope_label: &str, closure_label: &str) {
        let loops: Vec<String> = self
            .stack
            .iter()
            .filter(|f| f.scope_label == enclosing_scope_label)
            .map(|f| f.label.clone())
            .collect();
        for frame in self.frames_of(enclosing_scope_label) {
            frame.closures.insert(closure_label.to_string());
        }
        if !loops.is_empty() {
            self.closure_loops.insert(closure_label.to_string(), loops);
        }
    }

    pub(crate) fn note_registration(&mut self, scope_label: &str) {
        for frame in self.frames_of(scope_label) {
            frame.has_registration = true;
        }
    }

    /// Close the innermost frame. `captured_by[name]` holds the capturer
    /// labels of `name`'s binding in the frame's function scope, precomputed
    /// by the caller. `parents` is the analyzer's `parent_labels`, used to tell
    /// whether a capturer is one of the frame's closures or nested in one.
    pub(crate) fn close(
        &mut self,
        function_key: &str,
        captured_by: &BTreeMap<String, Vec<String>>,
        parents: &BTreeMap<String, Option<String>>,
    ) {
        let frame = self.stack.pop().expect("balanced loop frames");
        if !frame.has_registration {
            return;
        }
        let is_within = |capturer: &str, closure: &str| {
            let mut cursor = Some(capturer.to_string());
            while let Some(label) = cursor {
                if label == closure {
                    return true;
                }
                cursor = parents.get(&label).cloned().flatten();
            }
            false
        };
        let cells: Vec<String> = frame
            .declared
            .iter()
            .filter(|name| {
                captured_by.get(*name).is_some_and(|capturers| {
                    capturers
                        .iter()
                        .any(|c| frame.closures.iter().any(|closure| is_within(c, closure)))
                })
            })
            .cloned()
            .collect();
        if cells.is_empty() {
            return;
        }
        let enclosing: Vec<String> = self
            .stack
            .iter()
            .filter(|f| f.scope_label == frame.scope_label)
            .map(|f| f.label.clone())
            .collect();
        self.owners.insert(frame.label.clone(), (frame.scope_label.clone(), enclosing));
        self.scopes.push(crate::IterationScope {
            label: frame.label,
            function: function_key.to_string(),
            cells,
        });
    }

    /// Re-parent owners and closures (call after the whole walk).
    pub(crate) fn relabel(&self, parent_labels: &mut BTreeMap<String, Option<String>>) {
        for (owner, (scope_label, enclosing)) in &self.owners {
            let parent = enclosing
                .iter()
                .rev()
                .find(|l| self.owners.contains_key(*l))
                .cloned()
                .or_else(|| (scope_label != "<module>").then(|| scope_label.clone()));
            parent_labels.insert(owner.clone(), parent);
        }
        for (closure, loops) in &self.closure_loops {
            if let Some(owner) = loops.iter().rev().find(|l| self.owners.contains_key(*l)) {
                parent_labels.insert(closure.clone(), Some(owner.clone()));
            }
        }
    }
}
```

- [ ] **Step 3: Hook it into the walk**

1. `OwnershipAnalyzer` gains `pub(crate) iteration: iteration::IterationCollector` (initialised with `Default::default()` in `new`).
2. **Loop arms in `walk_scope_node`.** Before walking the children, call `self.iteration.open(&self.current_scope_label(), &plan_key(&self.current_scope_label()))`, where `plan_key` maps `"<module>"` to `""` and leaves other labels unchanged. After the children, call:
   ```rust
   let declared = self.iteration.stack.last().map(|f| f.declared.clone()).unwrap_or_default();
   let captured: BTreeMap<String, Vec<String>> = declared
       .iter()
       .filter_map(|name| {
           let scope = self.scope_stack.last()?;
           let binding = scope.bindings.get(scope.get_binding_index(name)?)?;
           Some((name.clone(), binding.captured_by.iter().cloned().collect()))
       })
       .collect();
   let parents = self.parent_labels.clone();
   self.iteration.close(&key, &captured, &parents);
   ```
   This goes alongside the existing `arena_enter_loop` / `arena_exit_loop` calls, and also in the `ForInStmt` arm, which has no arena call. The maps are precomputed so that no borrow of `self` is held while `self.iteration` is borrowed mutably.
3. **The `VarDecl` arm** (or the node `precollect_scope_bindings` reads: HIR `VarDecl` with text `"let"`/`"const"` and `VarDeclarator` children carrying names). During the walk, call `self.iteration.note_declared(&self.current_scope_label(), names)`.
4. **The `FunctionDecl` / `FunctionExpr` arms**, right after `push_scope(label, …)`: call `self.iteration.note_closure(&enclosing_label, &label)`, where `enclosing_label` is the scope label *before* the push.
5. **The `CallExpr` arm:** if `kali_common::is_deferred_registration_callee(callee.text, !callee.children.is_empty())`, call `self.iteration.note_registration(&self.current_scope_label())`.
6. **In `analyze_program_with_arena`,** after `pop_scope_and_record` of the module: `self.iteration.relabel(&mut self.parent_labels)`. Return `self.iteration.scopes` as a 4th tuple element, and thread it into `MirProgram { iteration_scopes, .. }` at `kali_mir/src/lower.rs:26`. Every other `MirProgram` literal in the crate gets `iteration_scopes: Vec::new()`.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_mir`
Expected: PASS. That covers the new `iteration` tests and every existing MIR test. `parent_labels` is unchanged for programs with no owner.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_mir
git commit -m "feat(block-scoping): MIR iteration owners — loops that register a callback capturing their own bindings"
```

---

### Task 6: Iteration plans in `derive_env_plans`

**Files:**
- Modify: `crates/kali_mir/src/env_plan.rs` (`EnvPlan`, `CapturedRef`, `derive_env_plans`, its inline `mod tests`), `crates/kali_mir/src/lib.rs` (re-export `repr_owner`, `iteration_label`)

**Interfaces:**
- Consumes: `MirProgram::iteration_scopes`, the relabelled `parent_labels` (Task 5)
- Produces:
```rust
pub struct EnvPlan { pub owns_env: bool, pub cells: Vec<EnvCell>, pub captured: Vec<CapturedRef>,
    /// `Some(function plan key)` for a per-iteration plan.
    pub iteration_of: Option<String> }
pub struct CapturedRef { pub name: String, pub depth: u32, pub offset: u32, pub is_scalar: bool,
    pub owner: String,
    /// The hop path from the capturer to `owner` passes through a per-iteration
    /// record that is not `owner` (spec A-4).
    pub through_iteration: bool }
/// The repr namespace of an env owner: an iteration plan's function, else the owner.
pub fn repr_owner<'a>(plans: &'a BTreeMap<String, EnvPlan>, owner: &'a str) -> &'a str;
```

**The algorithm change:**
- **Pass 1** is unchanged for functions.
- **Then,** for each `IterationScope`, the function plan's cells named in `scope.cells` move to a new plan keyed `scope.label`. Both plans' offsets are renumbered `idx * 8` in name order, and `owns_env` is recomputed for both. The new plan has `iteration_of: Some(scope.function)`.
- **`env_owners`** includes iteration labels.
- **Pass 2:** for a binding `name` of function `f`, the owner is the iteration label whose cells contain `name` (scoped to function `f`), else `f`.
  - `through_iteration` is true when the `parent_labels` walk from the capturer to the owner steps through an iteration label other than the owner.
  - `depth` is `env_owning_hops` as before.

- [ ] **Step 1: Write the failing tests** (extend the existing inline `mod tests` in `env_plan.rs`)

```rust
#[test]
fn an_iteration_owner_holds_the_loop_cells_and_the_closure_reads_them_at_depth_one() {
    let p = crate::test_support::analyze(
        "function m(){ for(let i=0;i<3;i++){ setTimeout(()=>console.log(i), 0); } } m();",
    );
    let plans = derive_env_plans(&p);
    let iter = plans.get("m{iter0}").expect("iteration plan");
    assert_eq!(iter.iteration_of.as_deref(), Some("m"));
    assert_eq!(iter.cells, vec![EnvCell { name: "i".into(), offset: 0, is_scalar: true }]);
    assert!(!plans.get("m").map(|f| f.owns_env).unwrap_or(false));
    let capture = plans
        .values()
        .flat_map(|plan| plan.captured.iter())
        .find(|r| r.name == "i")
        .expect("closure captures i");
    assert_eq!((capture.depth, capture.owner.as_str(), capture.through_iteration), (1, "m{iter0}", false));
    assert_eq!(repr_owner(&plans, "m{iter0}"), "m");
}

#[test]
fn a_function_binding_captured_from_inside_an_owner_is_through_iteration_at_depth_two() {
    let p = crate::test_support::analyze(
        "function m(){ let a=10; for(let i=0;i<2;i++){ setTimeout(()=>console.log(a+i),0); } } m();",
    );
    let plans = derive_env_plans(&p);
    let a = plans
        .values()
        .flat_map(|plan| plan.captured.iter())
        .find(|r| r.name == "a")
        .expect("closure captures a");
    assert_eq!((a.depth, a.owner.as_str(), a.through_iteration), (2, "m", true));
}

#[test]
fn nested_owners_put_the_outer_cell_at_depth_two_without_crossing_a_foreign_record() {
    let p = crate::test_support::analyze(
        "for(let i=0;i<2;i++){ for(let j=0;j<2;j++){ setTimeout(()=>console.log(i,j),0); } }",
    );
    let plans = derive_env_plans(&p);
    let refs: Vec<_> = plans.values().flat_map(|plan| plan.captured.iter()).collect();
    let j = refs.iter().find(|r| r.name == "j").unwrap();
    let i = refs.iter().find(|r| r.name == "i").unwrap();
    assert_eq!((j.depth, j.owner.as_str(), j.through_iteration), (1, "{iter1}", false));
    assert_eq!((i.depth, i.owner.as_str(), i.through_iteration), (2, "{iter0}", true));
}

#[test]
fn a_program_without_owners_gets_the_same_plans_as_before() {
    let p = crate::test_support::analyze("function outer(){ let c = 0; function inc(){ c += 1; } inc(); return c; }");
    let plans = derive_env_plans(&p);
    assert!(plans.values().all(|plan| plan.iteration_of.is_none()));
    assert!(plans.values().flat_map(|plan| plan.captured.iter()).all(|r| !r.through_iteration));
}
```
**The nested-owner test pins a deliberate limit.** `i` is reached through the inner iteration record, so `through_iteration` is true and depth is 2. That is refused in Task 7 (spec A-4; Review Focus 5), and Task 8 pins it as `nested_owner_loops_are_refused_at_depth_two`.

Run: `cargo test -p kali_mir env_plan`
Expected: compile FAIL (`iteration_of`, `through_iteration`, `repr_owner` undefined).

- [ ] **Step 2: Implement**

- Add the two fields. `iteration_of` is `None` in every existing constructor and `through_iteration` is `false`. Update every struct literal in the workspace: `cargo build --workspace` lists them.
- Add `iteration_label` (from Task 5, already present) and:
```rust
pub fn repr_owner<'a>(plans: &'a BTreeMap<String, EnvPlan>, owner: &'a str) -> &'a str {
    plans
        .get(owner)
        .and_then(|plan| plan.iteration_of.as_deref())
        .unwrap_or(owner)
}

fn path_crosses_iteration(
    from: &str,
    to: &str,
    parents: &BTreeMap<String, Option<String>>,
    iteration_labels: &BTreeSet<String>,
) -> bool {
    let mut cursor = parents.get(from).cloned().flatten();
    while let Some(label) = cursor {
        if label == to {
            return false;
        }
        if iteration_labels.contains(&label) {
            return true;
        }
        cursor = parents.get(&label).cloned().flatten();
    }
    false
}
```
- In `derive_env_plans`, after Pass 1:
```rust
let iteration_labels: BTreeSet<String> =
    program.iteration_scopes.iter().map(|s| s.label.clone()).collect();
// owner_of[(function_key, name)] = iteration label
let mut owner_of: BTreeMap<(String, String), String> = BTreeMap::new();
for scope in &program.iteration_scopes {
    let moved: Vec<EnvCell> = plans
        .get_mut(&scope.function)
        .map(|f| {
            let (moved, kept): (Vec<_>, Vec<_>) =
                f.cells.drain(..).partition(|c| scope.cells.contains(&c.name));
            f.cells = renumber(kept);
            f.owns_env = !f.cells.is_empty();
            moved
        })
        .unwrap_or_default();
    for cell in &moved {
        owner_of.insert((scope.function.clone(), cell.name.clone()), scope.label.clone());
    }
    plans.insert(
        scope.label.clone(),
        EnvPlan {
            owns_env: !moved.is_empty(),
            cells: renumber(moved),
            captured: Vec::new(),
            iteration_of: Some(scope.function.clone()),
        },
    );
}
```
  - `renumber` sorts by name and sets `offset = idx * 8`.
  - The module plan (`""`) never has cells. For a module-root iteration scope, Pass 1 never gave `""` the cells. So for `scope.function == ""`, build the cells from the module function's bindings that are named in `scope.cells` and have a non-empty `captured_by`, using `is_scalar_cell(&binding.layout)`. Do not take them from the `""` plan.
- In Pass 2, compute the owner key with `owner_of.get(&(function_key, name)).cloned().unwrap_or(function_key)`. Include module-root bindings that are in `owner_of`; others at the root stay skipped as before. Set `through_iteration: path_crosses_iteration(capturer, &owner, &program.parent_labels, &iteration_labels)`. `env_owners` is computed after the moves, so it includes iteration labels.

- [ ] **Step 3: Run the tests**

Run: `cargo test -p kali_mir && cargo build --workspace`
Expected: PASS and builds.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir crates/kali_codegen crates/kali_cli
git commit -m "feat(block-scoping): per-iteration env plans, through_iteration, repr_owner"
```

---

### Task 7: Codegen plumbing — loop identification, locals, module globals, owner namespaces, refusals

**Files:**
- Create: `crates/kali_codegen/src/iteration.rs`, `crates/kali_codegen/src/iteration_tests.rs`
- Modify: `crates/kali_codegen/src/lib.rs` (`mod iteration;`), `lower.rs` (`:612-626` start locals, `:1359-1392` module bindings, `:1411-1438` locals reservation, `:1449` diagnostics, `:4138` module scalar globals), `emitter.rs` (fields), `env_safety.rs` (owner namespace and loop context), `intrinsics/host.rs:1733-1780` (owner namespace), `emit/closure_access.rs` (owner namespace)

**Interfaces:**
- Consumes: `EnvPlan::iteration_of`, `CapturedRef::through_iteration`, `kali_mir::repr_owner` (Task 6); messages (Task 1)
- Produces:
```rust
// kali_codegen::iteration
/// Iteration plans whose `iteration_of` is `function_key` (`""` for `_start`).
pub(crate) fn iteration_plans_of<'p>(plans: &'p BTreeMap<String, EnvPlan>, function_key: &str)
    -> Vec<(&'p str, &'p EnvPlan)>;
/// `let` / `const` names declared directly in loop `loop_id` (not in a nested
/// loop or function): its head and body, matching MIR's rule.
pub(crate) fn names_declared_in_loop(nodes: &[LirNode], loop_id: LirNodeId) -> BTreeSet<String>;
/// The iteration plan whose cells are all declared directly in `loop_id`.
pub(crate) fn iteration_label_for_loop<'p>(nodes: &[LirNode], loop_id: LirNodeId,
    plans: &[(&'p str, &'p EnvPlan)]) -> Option<&'p str>;
/// `"_start"` → `""`, else the name.
pub(crate) fn plan_key(function_name: &str) -> &str;
/// E5506 per captured ref with `through_iteration && depth >= 2` (spec A-4).
pub(crate) fn iteration_capture_diagnostics(plans: &BTreeMap<String, EnvPlan>) -> Vec<Diagnostic>;
/// Save-local names: `__iter_save{label}#env` and the copy scratch `__iter_prev#env`.
pub(crate) fn iteration_save_local_name(label: &str) -> String;
pub(crate) fn iteration_prev_local_name() -> String;
```
Loop kinds are the LIR `Branch` texts `for`, `while`, `do-while`, `for-of`, `for-await-of` and `for-in`. A declaration is a `LirNodeKind::Instruction` with text `let` or `const` (`lower.rs:6378`), and its declarator children's `text` holds the names. Nested functions are skipped with `is_function_like` (`lower.rs:6440`), and nested loop `Branch` nodes are not descended into.

- [ ] **Step 1: Write the failing tests**

`crates/kali_codegen/src/iteration_tests.rs`, using `crate::test_support::parse_and_lower_lir_with_env_plans` (`test_support.rs:52`):
```rust
use super::*;
use crate::test_support::parse_and_lower_lir_with_env_plans;

fn loops(nodes: &[LirNode]) -> Vec<LirNodeId> {
    nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.kind == LirNodeKind::Branch
            && matches!(n.text.as_deref(), Some("for" | "while" | "do-while" | "for-of" | "for-await-of" | "for-in")))
        .map(|(i, _)| LirNodeId(i as u32))
        .collect()
}

#[test]
fn the_owner_loop_is_found_by_its_declared_cells() {
    let (lir, plans) = parse_and_lower_lir_with_env_plans(
        "function m(){ for(let i=0;i<3;i++){ setTimeout(()=>console.log(i), 0); } } m();",
    );
    let mine = iteration_plans_of(&plans, "m");
    assert_eq!(mine.len(), 1);
    let found: Vec<_> = loops(&lir.nodes)
        .into_iter()
        .filter_map(|l| iteration_label_for_loop(&lir.nodes, l, &mine))
        .collect();
    assert_eq!(found, vec!["m{iter0}"]);
}

#[test]
fn a_nested_loop_declaration_belongs_to_the_inner_loop_only() {
    let (lir, _) = parse_and_lower_lir_with_env_plans(
        "function m(){ for(let i=0;i<2;i++){ for(let j=0;j<2;j++){ const k=j; } } } m();",
    );
    let ls = loops(&lir.nodes);
    let sets: Vec<_> = ls.iter().map(|l| names_declared_in_loop(&lir.nodes, *l)).collect();
    assert!(sets.contains(&["i".to_string()].into_iter().collect()));
    assert!(sets.contains(&["j".to_string(), "k".to_string()].into_iter().collect()));
}

#[test]
fn a_depth_two_capture_through_a_record_is_refused() {
    let (_, plans) = parse_and_lower_lir_with_env_plans(
        "function m(){ let a=10; for(let i=0;i<2;i++){ setTimeout(()=>console.log(a+i),0); } } m();",
    );
    let d = iteration_capture_diagnostics(&plans);
    assert_eq!(d.len(), 1);
    assert!(d[0].message.contains("captures `a` through a per-iteration record"), "{}", d[0].message);
}

#[test]
fn the_module_root_plan_key_is_empty() {
    assert_eq!(plan_key("_start"), "");
    assert_eq!(plan_key("m"), "m");
}
```

Run: `cargo test -p kali_codegen iteration`
Expected: compile FAIL.

- [ ] **Step 2: Implement `iteration.rs`** (the functions in Interfaces; the walk follows `collect_function_locals_from_node`'s shape)

```rust
pub(crate) fn names_declared_in_loop(nodes: &[LirNode], loop_id: LirNodeId) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some(node) = nodes.get(loop_id.0 as usize) {
        for child in &node.children {
            collect_declared(nodes, *child, &mut out);
        }
    }
    out
}

fn collect_declared(nodes: &[LirNode], id: LirNodeId, out: &mut BTreeSet<String>) {
    let Some(node) = nodes.get(id.0 as usize) else { return };
    if node.kind == LirNodeKind::Branch && is_loop_text(node.text.as_deref()) {
        return; // a nested loop's declarations are its own
    }
    if node.kind == LirNodeKind::Instruction && matches!(node.text.as_deref(), Some("let" | "const")) {
        for declarator in &node.children {
            if let Some(name) = nodes.get(declarator.0 as usize).and_then(|d| d.text.clone()) {
                out.insert(name);
            }
        }
    }
    for child in &node.children {
        if crate::lower::is_function_like(nodes, *child) {
            continue;
        }
        collect_declared(nodes, *child, out);
    }
}

pub(crate) fn is_loop_text(text: Option<&str>) -> bool {
    matches!(text, Some("for" | "while" | "do-while" | "for-of" | "for-await-of" | "for-in"))
}

pub(crate) fn iteration_label_for_loop<'p>(
    nodes: &[LirNode],
    loop_id: LirNodeId,
    plans: &[(&'p str, &'p EnvPlan)],
) -> Option<&'p str> {
    let declared = names_declared_in_loop(nodes, loop_id);
    plans
        .iter()
        .find(|(_, plan)| !plan.cells.is_empty() && plan.cells.iter().all(|c| declared.contains(&c.name)))
        .map(|(label, _)| *label)
}

pub(crate) fn iteration_capture_diagnostics(plans: &BTreeMap<String, EnvPlan>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (capturer, plan) in plans {
        for reference in &plan.captured {
            if reference.through_iteration && reference.depth >= 2 {
                out.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    kali_common::iteration_capture_through_record_message(&reference.name, capturer),
                ));
            }
        }
    }
    out
}
```
(Make `is_function_like` `pub(crate)` if it isn't already. `iteration_plans_of`, `plan_key` and the two name functions are one-liners as specified. The save-local names use the `#env` suffix, which a source identifier cannot contain, following `closure::env_save_local_name`.)

- [ ] **Step 3: Plumbing in `lower.rs`**

1. **Locals reservation (`:1411-1438`).** After the existing per-function block, for each function (and separately for the `_start` local list at `:612-626`), take `iteration_plans_of(&ctx.env_plans, plan_key(&function.name))`.
   - Remove every promotable iteration cell name from that function's locals, using the predicate `cell_is_promotable(&ctx.repr_table, repr_owner(&ctx.env_plans, label), &cell.name, cell.is_scalar)`.
   - For each iteration plan with at least one promotable cell, push `iteration_save_local_name(label)`.
   - If any such plan exists, push `iteration_prev_local_name()` once.
2. **Module globals (`:4138` `collect_module_scalar_globals`, and `:1359-1392` `module_const_inits` / `module_binding_names`).** Exclude every cell name of `iteration_plans_of(&ctx.env_plans, "")`. Pass the excluded set in as a parameter. These are per-iteration bindings, not module storage.
3. **Diagnostics (`:1449`).** Next to `env_capture_safety_diagnostics(...)`, extend the diagnostics with `crate::iteration::iteration_capture_diagnostics(&ctx.env_plans)`.

- [ ] **Step 4: Owner namespaces**

Every call that passes a `CapturedRef::owner`, or an iteration label, as the repr namespace to `crate::closure::cell_is_promotable` or `FunctionEmitter::promotable_scalar_cell_in` must pass `kali_mir::repr_owner(self.env_plans, owner)` instead. Find the calls with:
```bash
grep -rn "cell_is_promotable\|promotable_scalar_cell_in" crates/kali_codegen/src
```
The known sites are `env_safety.rs:208-215`, `emit/closure_access.rs:187-194` and `intrinsics/host.rs:1741-1780`. `repr_owner` maps an iteration label to its function key. If that key is `""` (module root), map it to the repr namespace that module bindings use. Find that namespace by reading `binding_repr_function_key` (`kali_types/src/resolve/expression.rs:297`) and `ReprTable` usage for `_start`, and pin it with the Task 8 case `a_module_loop_owner_prints_each_iteration`.

- [ ] **Step 5: `env_safety` loop context**

In `env_capture_safety_diagnostics`, when the body walk enters a LIR `Branch` loop node for which `iteration_label_for_loop(nodes, id, &iteration_plans_of(plans, plan_key(current_function)))` is `Some(label)`, the active context inside that loop's children is `EnvCtx::Record(label)`. Restore the previous context after the loop. Without this, a registration inside an owner loop would be judged against the function's record and refused.

The test for this change is Task 8 Step 5: `a_function_loop_owner_prints_each_iteration` and `a_module_loop_owner_prints_each_iteration` both fail with `env_safety`'s E5506 ("cannot be proven to be … record") without it. It cannot be observed before emission exists, so Task 7 adds no separate test.

- [ ] **Step 6: Emitter fields**

`FunctionEmitter` gains:
```rust
/// This function's per-iteration plans (spec §3.3), from `iteration_plans_of`.
pub(crate) iteration_plans: Vec<(String, kali_mir::EnvPlan)>,
/// Owner loops currently being emitted, outermost first.
pub(crate) active_iterations: Vec<ActiveIteration>,
/// Iteration labels whose loop was emitted (the A-6 backstop).
pub(crate) emitted_iterations: BTreeSet<String>,
```
with
```rust
pub(crate) struct ActiveIteration { pub(crate) label: String, pub(crate) save_local: u32, pub(crate) plan: kali_mir::EnvPlan }
```
Initialize them in `FunctionEmitter::new` from `ctx.env_plans` (owned clones), keyed by `plan_key(&function.name)`.

- [ ] **Step 7: Run**

Run: `cargo test -p kali_codegen && cargo test -p kali_cli --test cases -- scope/block_shadowing`
Expected: PASS. No emission change yet, so programs without owners are unaffected.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_codegen
git commit -m "feat(block-scoping): codegen plumbing for per-iteration records — loop identification, locals, module globals, owner namespaces, depth-2 refusal"
```

---

### Task 8: Emit per-iteration records

**Files:**
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (`emit_loop` `:319-480`, `emit_for_in` `:572`, `emit_env_restore` `:880`, `emit_function_body` `:889`), `crates/kali_codegen/src/emit/closure_access.rs` (`resolve_capture_access_inner` `:186`), `crates/kali_codegen/src/intrinsics/array.rs` (`emit_for_of_growable_runtime_loop` `:1153`, the unrolled path `:~1405-1505`)
- Create: `crates/kali_cli/tests/cases/scope/per_iteration.toml`

**Interfaces:**
- Consumes: everything from Task 7
- Produces:
  - `fn enter_iteration(&mut self, function: &mut Function, label: &str)`: saves `g8` and pushes `active_iterations`.
  - `fn alloc_iteration_record(&mut self, function: &mut Function)`: allocates a record with parent = the innermost save local, and sets `g8`.
  - `fn copy_head_cells_into_new_record(&mut self, function: &mut Function, head: &BTreeSet<String>)`
  - `fn exit_iteration(&mut self, function: &mut Function)`: restores `g8` from the innermost save local and pops.

**Emission (spec §3.3, A-5).** In `emit_loop`, after clause resolution, compute `owner = iteration_label_for_loop(nodes, id, &plans)` over `self.iteration_plans`. If it is `Some(label)`:

| point | `for` | `while` / `do-while` / `for-in` / growable `for-of` |
|---|---|---|
| before `init` | `enter_iteration`; `alloc_iteration_record` | `enter_iteration` |
| top of each iteration (after the arena reset) | — | `alloc_iteration_record` |
| after the body, before `update` | `copy_head_cells_into_new_record(head)`, where `head` = `names_declared_in` the init clause | — |
| after the loop's outer `End` | `exit_iteration` | `exit_iteration` |

- **`copy_head_cells_into_new_record`:**
  1. `global.get g8; local.set __iter_prev#env`.
  2. `alloc_iteration_record` (parent = the loop's save local, not `prev`, so records never chain to each other).
  3. For each head cell `c` that is in the plan, in offset order: `local.get g8-base; local.get prev-base; i64.load off=8+c.offset; i64.store off=8+c.offset`, using `closure::env_memarg`.
- **A `for` owner whose body contains a `continue`** (direct, not in a nested loop or function) pushes `iteration_for_continue_message()` as an E5506 diagnostic onto the emitter's diagnostics (the same list the label refusal at `control_flow.rs:36-48` uses), and is emitted without iteration records.
- **An unrolled `for-of` owner** pushes `iteration_unrolled_for_of_message(first cell name)`, likewise.
- **`emitted_iterations.insert(label)`** happens once the records are emitted.

**Capture access (`resolve_capture_access_inner`).** First search `active_iterations` innermost-first. A cell found in the `k`-th from the innermost resolves to `(k, offset)`, gated on promotability under `repr_owner`. Otherwise, let `extra = active_iterations.len() as u32`: an own function cell resolves to `(extra, offset)`, and a captured ref to `(walk + extra, offset)`.

**Exits.** `emit_env_restore`: if `owns_promotable_env()`, restore as today. Otherwise, if `active_iterations` is non-empty, `local.get <outermost active save>; global.set g8`. That restores the caller's env on a `return` from inside an owner loop. `break` leaves through the outer `End`, so `exit_iteration` runs.

**Backstop (A-6).** At the end of `emit_function_body` (and of the `_start` sequence emission), for each label in `iteration_plans` that is not in `emitted_iterations`, push `iteration_record_unplaced_message(label)` as E5506.

- [ ] **Step 1: Write the failing cases**

`crates/kali_cli/tests/cases/scope/per_iteration.toml`:
- `[source]` holds the probe programs `defer1`, `defer2`, `defer5`, `defer_while`, `asyncloop`, `nested_loops`, `r_defer4`, `r_loopmix`, `r_continue`, `ok_loopc3` and `return_restores` (same text as `bs_*.js`), plus:
```toml
"for_of_growable.js" = """function m(){ const xs=[]; xs.push(5); xs.push(6); for(const x of xs){ queueMicrotask(()=>console.log(x)); } } m();
"""
"for_in.js" = """function m(){ const o={a:1,b:2}; for(const k in o){ queueMicrotask(()=>console.log(k)); } } m();
"""
"closure_writes.js" = """function m(){ for(let i=0;i<3;i++){ setTimeout(()=>{ i+=10; console.log(i); },0); } } m();
"""
```
- Cases, each with node's output and the `6345f082b` output in the rationale:
  - `a_module_loop_owner_prints_each_iteration` (`defer1`, `0\n1\n2\n`)
  - `a_function_loop_owner_prints_each_iteration` (`defer2`, `0\n1\n2\n`)
  - `a_body_const_is_per_iteration` (`defer5`, `0\n3\n`)
  - `a_while_body_const_is_per_iteration` (`defer_while`: measure node in Step 2)
  - `an_awaiting_loop_is_per_iteration` (`asyncloop`, `0\n1\n`)
  - `a_growable_for_of_is_per_iteration` (`for_of_growable.js`, `5\n6\n`)
  - `a_for_in_is_per_iteration` (`for_in.js`, `a\nb\n`)
  - `a_closure_write_stays_in_its_iteration` (`closure_writes.js`, `10\n11\n12\n`)
  - `return_from_an_owner_loop_restores_the_caller_env` (`return_restores`: measure node in Step 2)
  - `a_synchronous_closure_is_unchanged` (`ok_loopc3`, `3\n`)
- Refusals:
  - `a_capture_of_a_function_binding_through_a_record_is_refused_under_run` (`r_loopmix`; `stderr_contains = ["E5506", "through a per-iteration record"]`)
  - its `check` twin `…_is_not_seen_by_check` (`exit = "success"`, pinning the gap in spec §3.4)
  - `nested_owner_loops_are_refused_at_depth_two` (`nested_loops`, run; the Task 6 note)
  - `continue_in_an_owner_for_is_refused` (`r_continue`; `stderr_contains = ["E5506", "`continue` in a `for` loop"]`)
  - `an_unrolled_for_of_owner_is_refused` (`r_defer4`; `stderr_contains = ["E5506", "compile-time iterable"]`)

Run: `cargo test -p kali_cli --test cases -- scope/per_iteration`
Expected: FAIL (wrong values and no refusals).

- [ ] **Step 2: Measure the two unmeasured node outputs**

```bash
node tools/array-return-probes/probes/bs_defer_while.js
node tools/array-return-probes/probes/bs_return_restores.js
```
Pin these exact outputs in the two cases.

- [ ] **Step 3: Implement the emission** as specified above, in `emit_loop`, `emit_for_in`, `emit_for_of_growable_runtime_loop` (allocation goes before the per-iteration store of the loop variable) and the unrolled `for-of` path (refusal only).

- [ ] **Step 4: Implement capture access and exits** as specified above.

- [ ] **Step 5: Run**

Run: `cargo test -p kali_cli --test cases -- scope/ && cargo test -p kali_codegen && cargo test -p kali_mir`
Expected: PASS.

- [ ] **Step 6: Diff the probes**

```bash
tools/array-return-probes/run.sh "$TMPDIR/bs-head.tsv"
diff <(cut -f1,2 tools/array-return-probes/baseline-bs.tsv) <(grep '^bs_' "$TMPDIR/bs-head.tsv" | cut -f1,2)
```
Expected:
- `defer1`, `defer2`, `defer5`, `defer_while`, `asyncloop` and `return_restores` are CORRECT.
- `r_loopmix`, `r_defer4`, `r_continue` and `nested_loops` are REFUSES.
- `ok_*` stay CORRECT.
- `r_loopc*` and `r_defer3` are unchanged.


- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/cases/scope/per_iteration.toml
git commit -m "feat(block-scoping): per-iteration env records for loops that register callbacks over their own bindings"
```

---

### Task 9: Blast radius, triage and re-pins

**Files:**
- Create: `docs/superpowers/followups/block-scoping-discovered-defects.md` (the triage section; Task 10 completes the file)
- Modify: `crates/kali_cli/tests/cases/oracle/tier2.toml:1377-1400` (and the header index line 87), `crates/kali_cli/tests/cases/oracle/classifier_ground_truth.toml:125,188-202`, plus whatever the sweep moves

- [ ] **Step 1: Run everything**

```bash
cargo test --workspace --no-fail-fast 2>&1 | tee "$TMPDIR/bs-workspace.txt" | tail -40
cargo test -p kali_cli --test cases -- --ignored 2>&1 | tail -20
```
List every failing trial.

- [ ] **Step 2: Apply the stop rule**

If more than 50 trials moved, or any trial is a capability loss of class 1 (spec §5.4), stop. Write the list into the followups file and report to the human partner without re-pinning.

- [ ] **Step 3: The known movers**

- **`oracle/tier2.toml`:** `r10_block_scope_shadowing_module_scope` and `r10_block_scope_shadowing_in_function` change to `verdict = "fixed"`. Each rationale gets a dated line: "FIXED 2026-10-04 by the block-scoping project (`docs/superpowers/specs/2026-10-04-block-scoping-design.md`): kali prints `r=1`, node v26.10.0 prints `r=1`." Update the header index line to match.
- **`oracle/classifier_ground_truth.toml`:** **stop and ask.** Measure three candidates from the register's entries still SILENT at HEAD:
  ```bash
  grep -n "Status\|silent" docs/superpowers/followups/kali-silent-miscompile-register.md | head -80
  ```
  Pick three with a one-line repro. Run each under node and `target/debug/kali run`. Present the three (repro, node output, kali output, register id) to the human partner, and re-pin `silent.js` plus the case's `register_entry` to the one they pick. Do not choose alone (spec §5.2).

- [ ] **Step 4: The sweep**

For every `cli` or `oracle` trial whose HEAD stderr contains `through a per-iteration record`, `` `continue` in a `for` loop ``, `compile-time iterable`, `was planned but no loop declared` or `with \`--compat eval\``, re-run the same program with `$KALI_BASE`. Classify each as wanted, capability loss (class 1, 2 or 3), rationale only, or stderr only.

- [ ] **Step 5: Re-pin and record**

Re-pin each moved trial according to its class. Write the triage table in the followups file in the format of `class-instances-discovered-defects.md` §4: name, before (`6345f082b`), after (HEAD), class.

- [ ] **Step 6: Green**

Run: `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` and `cargo fmt --all -- --check`
Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add -A crates docs/superpowers/followups/block-scoping-discovered-defects.md
git commit -m "test(block-scoping): triage and re-pin the trials the change moved; R-10 oracle cases fixed"
```

---

### Task 10: Docs

**Files:**
- Modify: `specs/15-errors.md`, `specs/19-feature-maturity.md`, `docs/superpowers/followups/kali-silent-miscompile-register.md`, `docs/superpowers/followups/class-instances-discovered-defects.md`, `docs/superpowers/followups/literal-array-mutators-discovered-defects.md`, `docs/superpowers/followups/unresolved-member-call-discovered-defects.md`, `docs/superpowers/followups/block-scoping-discovered-defects.md`

- [ ] **Step 1: `specs/15-errors.md`**

Under E5506's registered texts, add the five messages from Task 1, each with one line naming the spec section that raises it (§3.4, A-4, A-5 ×2, A-6). `iteration_capture_through_record_message`, `iteration_for_continue_message`, `iteration_unrolled_for_of_message` and `iteration_record_unplaced_message` are run-only. `block_scope_eval_refused_message` is raised by `run` and `check`.

- [ ] **Step 2: `specs/19-feature-maturity.md`**

Add one row after row 203 (mutable locals). Leave row 203 untouched.
```markdown
| Block-scoped bindings and per-iteration loop bindings for registered callbacks | Phase 1 MVP | A `let`/`const`/`class`/function declaration in a block, `if` arm, loop body or `for` head is a binding of its own: it does not change a same-named module binding, local, `var` or parameter, at top level or in functions; sibling blocks may reuse a name with another type; same-named nested functions are separate. A closure registered with `queueMicrotask`/`setTimeout`/`setInterval`/`addEventListener` inside a `for`, `while`, `do`, `for…in` or growable-array `for…of` loop sees that iteration's loop bindings. Refused (E5506, `run` only): such a closure that also captures a binding of the enclosing function or of an outer loop, a `continue` in such a `for`, and such a `for…of` over a compile-time iterable. Refused under `run` and `check`: any shadowing binding with `--compat eval`. Not claimed: `Kali.test` callbacks, `catch`/`switch`-fallthrough shapes (refused for other reasons), sloppy-mode block functions. One record per iteration is allocated and never freed |
```

- [ ] **Step 3: The register**

In R-10 (`kali-silent-miscompile-register.md:2166`), add a status line: `**Status (2026-10-04): FIXED** by the block-scoping project at <commit of Task 4>; cases scope/block_shadowing::*, oracle/tier2::r10_*.` Update the status table row (line ~271) to match.

Add a new entry after the highest R-id (find it with `grep -n "^### R-" … | tail -1`): "Per-iteration loop bindings are shared by registered callbacks". It gets:
- repros `defer1` / `defer2` / `defer5`, with node and baseline output from `baseline-bs.tsv`;
- severity silent-wrong-value;
- status FIXED at the Task 8 commit, with cases `scope/per_iteration::*`.

- [ ] **Step 4: "Fixed by" notes**

In the established style (see `literal-array-mutators-discovered-defects.md` §3's first paragraph):
- `class-instances-discovered-defects.md` §6 items 11 and 15;
- `literal-array-mutators-discovered-defects.md` §14, third bullet;
- `unresolved-member-call-discovered-defects.md` §6 item 9.

Each note names the commit and a case.

- [ ] **Step 5: Complete `block-scoping-discovered-defects.md`**

Its header follows the class-instances file: filed by, oracle, measured at, what ships. Then these sections:
- §1 the triage table (Task 9);
- §2 measured capability loss;
- §3 the run-only refusals and the `check` / `run` gap (`r_loopmix` passes `check`);
- §4 nested owner loops refuse at depth 2 (a future item: lower depth-2 walks);
- §5 never-freed per-iteration records;
- §6 shapes not reached: `catch`, switch fallthrough, string `+`, unrolled `for…of`, `Kali.test` registrations, `loopc*` / `defer3` (first-class calls);
- §7 anything else found during Tasks 4–9.

- [ ] **Step 6: Consistency re-read**

Re-read `specs/12-cli.md`, `specs/18-schemas.md` and `README.md`, and confirm none needs a change: no command, flag, schema or usage moved. Re-read the maturity row against the per-iteration and block-shadowing cases: every claim has a case, and every refusal has a case.

- [ ] **Step 7: Commit**

```bash
git add specs docs
git commit -m "docs(block-scoping): errors, maturity row, register R-10 fixed and the per-iteration entry, followups"
```
