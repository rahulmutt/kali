# Array Return Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A `function` declaration whose every `return` yields an `I64`-element array hands its caller a real runtime array, readable bound (`const b = f(); b[i]`) or direct (`f()[k]`, `f().length`). Every other array-shaped return refuses with `E5506` at `kali check` and `kali run` alike.

**Architecture:** `kali_types` gains `array_return.rs`, a pure classifier plus an optimistic fixed point. `repr_infer` feeds it facts during its body walk, then unions element nodes for the admitted set before its interprocedural phase, and writes three new facts into `ReprTable`: array-returning functions, call-bound array bindings, and taint refusals. The `kali check` resolver and `kali_codegen` read only those facts. Codegen materializes a returned literal through a helper extracted from the declarator lane, and reads call results through the existing array-read lanes. Three backstops then replace silent `0`s with `E5506`, at the width the spike measures.

**Tech Stack:** Rust (`kali_common`, `kali_types`, `kali_codegen`, `kali_cli`, `kali_blast_radius`), the `.toml` case runner, node v26.8.2 as the oracle, `tools/blast-radius` (node + acorn).

**Spec:** `docs/superpowers/specs/2026-10-02-array-return-design.md` (as amended by `c5cdce286`; amendment A4, nested declarations, is added by Task 1 of this plan).

## Global Constraints

- **Branch:** `array-return`. Baseline `368b5b5ea` (Rust 1.99.0): **12105 passed, 0 failed, 27 ignored**.
- **Oracle:** `node v26.8.2`. Every case rationale records kali's baseline output and node's output.
- **Binary:** `target/debug/kali` (this container has no `.cache/cargo-target`; `cargo build -p kali_cli` writes to `target/`).
- **One cargo target directory.** No extra worktrees, no extra target dirs.
- **`bash scripts/test-gate.sh` after every task that changes Rust.** It prints `GATE OK: 0 failing tests` or the failing list. A run takes 40+ minutes; never report a gate result before it exits.
- **`cargo clippy --workspace -- -D warnings` and `cargo fmt --all -- --check` before the final push.** The gate runs neither.
- **Fixture files are read, never edited** (`crates/kali_cli/tests/fixtures/`).
- **Rust unit tests live in sibling `*tests.rs` files**, wired with `#[cfg(test)] #[path = "x_tests.rs"] mod x_tests;`. Never an inline test module body.
- **Black-box tests are `.toml` case files** under `crates/kali_cli/tests/cases/`, run by `cargo test -p kali_cli --test cases`. No new `tests/*.rs` targets.
- **No new diagnostic code.** Every refusal is `E5506` (`e5::FEATURE_UNAVAILABLE`).
- **Generated blast-radius regions are regenerated, never hand-edited.**
- **Commit after every task.** Message prefix `feat(array-return):`, `fix(array-return):`, `test(array-return):` or `docs(array-return):`.

## Review Focus

1. **A `const` local bound to an array literal, then returned** (`const a=[1,2,3]; return a;`). A reader expects `1,2,3` at the caller. Pinned in Task 6 (`const_literal_binding_return`).
2. **An array-returning function that is also used as a callback value** (`[1].map(mk)`, `setTimeout(mk)`). Its params must never be inferred array-fed from its direct call sites. Pinned in Task 4 (`escaping_function_params_are_never_array_fed`).
3. **An optimizer-rewritten initializer.** If `const b = f()` reaches codegen with a non-call init, `b` must not be treated as an array. Codegen registers `b` only when the init is still a call to an array-returning function. Pinned in Task 7 (`call_bound_registration_requires_a_call_init`).
4. **A function whose last statement is an `if` with returns in both arms.** It must be admitted, not tainted for "falling off the end". Pinned in Task 3 (`if_else_both_returning_does_not_fall_off`).
5. **A returned allocation made inside a loop.** It must survive the loop's arena. Pinned in Task 6 (`allocation_returned_from_inside_a_loop`).

## File Structure

**Created:**
- `tools/array-return-probes/probes/*.js`, `tools/array-return-probes/run.sh`: the probe corpus and its node-vs-kali runner (Task 1).
- `crates/kali_types/src/array_return.rs`: the return classifier, the facts struct and the fixed-point solver (Task 3).
- `crates/kali_types/src/array_return_tests.rs`: its unit tests (Task 3).
- `crates/kali_cli/tests/cases/runtime/array_return.toml`: the working lanes (Tasks 6-7).
- `crates/kali_cli/tests/cases/runtime/array_return_refusals.toml`: the refusal lanes (Tasks 4, 8, 9).
- `docs/superpowers/followups/array-return-discovered-defects.md`: what was measured and not fixed (Task 10).

**Modified, production:**
- `crates/kali_common/src/repr.rs`: three new `ReprTable` fields and their accessors (Task 2).
- `crates/kali_common/src/messages.rs`: the refusal message builder and reason strings (Task 2).
- `crates/kali_types/src/lib.rs`: `mod array_return;` (Task 3).
- `crates/kali_types/src/resolve/expression.rs:3199`: `expression_is_array_allocation` becomes `pub(crate)` (Task 3).
- `crates/kali_types/src/repr_infer.rs`: fact recording in Phase B, a new Phase C0, `CallEdge` arg shapes, and `emit_table` output (Task 4).
- `crates/kali_types/src/resolve/mod.rs:967-982` and `resolve/member.rs:351-366`: call-bound registration and the direct-call receiver (Task 5).
- `crates/kali_codegen/src/lower.rs` (`collect_function_locals`): the `__array_return_scratch` reservation (Task 6).
- `crates/kali_codegen/src/emit/control_flow.rs:177` (`emit_return`), `:1640-1683` (the declarator literal lane), and its member-read arms near `:2719` and `:2847`/`:3077` (Tasks 6-7).
- `crates/kali_codegen/src/emit/call.rs:6318` and `:6414` (the read helpers take an element repr), plus `emit_console_argument{,_as_string}` (Tasks 7-8).
- `crates/kali_codegen/src/emit/computed_member.rs:104` (Task 7).
- `crates/kali_codegen/src/emitter.rs`: the `array_return_call_elem` recognizer (Task 7).
- `crates/kali_codegen/src/emit/operators.rs:563-571` and `emit/literal.rs:13-45`: the backstops (Task 9).

**Modified, ledger:**
- `crates/kali_cli/tests/cases/oracle/tier2.toml`: the R-14 cases are re-pinned (Task 10).
- `docs/superpowers/followups/kali-silent-miscompile-register.md`, `docs/superpowers/followups/blast-radius-ranking.md` (its generated regions), `tools/blast-radius/{counts.json,clusters.json}`, and `docs/superpowers/followups/inline-allocation-value-position-discovered-defects.md` (Task 10).

---

### Task 1: The probe corpus, the baseline, and amendment A4

Before anything changes, record what every probe does at the baseline. That gives each later task a fixed "before". This task also writes amendment A4 into the spec: nested `function` declarations are admitted (see Step 4).

**Files:**
- Create: `tools/array-return-probes/run.sh`
- Create: `tools/array-return-probes/probes/*.js` (one program per file)
- Create: `tools/array-return-probes/baseline.tsv`
- Modify: `docs/superpowers/specs/2026-10-02-array-return-design.md`

**Interfaces:**
- Consumes: `target/debug/kali` built at `HEAD`.
- Produces: `tools/array-return-probes/run.sh <out.tsv>`, which writes one row per probe: `name<TAB>verdict<TAB>node<TAB>kali`. The verdict is `CORRECT` (same stdout, kali exits 0), `REFUSES` (kali exits non-zero and stderr contains `E5506`), `SILENT` (kali exits 0 with different stdout) or `OTHER`. Tasks 2, 9 and 10 call it.

- [ ] **Step 1: Write the runner**

```bash
#!/usr/bin/env bash
# Node-vs-kali probe runner for the array-return project
# (docs/superpowers/specs/2026-10-02-array-return-design.md §4.1).
# Usage: tools/array-return-probes/run.sh OUT.tsv
set -u
here="$(cd "$(dirname "$0")" && pwd)"
kali="${KALI:-$here/../../target/debug/kali}"
out="$1"
: > "$out"
for p in "$here"/probes/*.js; do
  name="$(basename "$p" .js)"
  node_out="$(node "$p" 2>&1)"
  kali_out="$("$kali" run "$p" 2>/tmp/array-return-probe.err)"
  kali_exit=$?
  kali_err="$(cat /tmp/array-return-probe.err)"
  if [ $kali_exit -eq 0 ] && [ "$kali_out" = "$node_out" ]; then
    verdict=CORRECT
  elif [ $kali_exit -ne 0 ] && grep -q E5506 /tmp/array-return-probe.err; then
    verdict=REFUSES
  elif [ $kali_exit -eq 0 ]; then
    verdict=SILENT
  else
    verdict=OTHER
  fi
  shown="$kali_out"
  [ $kali_exit -ne 0 ] && shown="$(printf '%s' "$kali_err" | head -1)"
  printf '%s\t%s\t%s\t%s\n' "$name" "$verdict" \
    "$(printf '%s' "$node_out" | tr '\n' '|')" \
    "$(printf '%s' "$shown" | tr '\n' '|')" >> "$out"
done
```

`chmod +x tools/array-return-probes/run.sh`.

- [ ] **Step 2: Write the probes**

Each file holds exactly the program shown. Names are the file stems.

| file | program |
|---|---|
| `s01_direct_index.js` | `function f() { return [1, 2, 3]; } console.log("r=" + f()[0]);` |
| `s02_bound_module.js` | `function f() { return [1, 2, 3]; } const a = f(); console.log(a[0] + "," + a[2]);` |
| `s03_bound_in_main.js` | `function f() { return [1, 2, 3]; } function main() { const a = f(); console.log(a[0] + "," + a[2]); } main();` |
| `s04_fill_binding_return.js` | `function mk() { const a = new Array(3).fill(4); return a; } const b = mk(); console.log(b[1]);` |
| `s05_param_pass_through.js` | `function f(x) { return x; } const a = f(new Array(3).fill(1)); console.log(a[0]);` |
| `s06_loop_filled.js` | `function f(n) { const a = new Array(n).fill(0); for (let i = 0; i < n; i++) a[i] = i * i; return a; } function main() { const a = f(4); console.log(a[3]); } main();` |
| `s07_store_then_return.js` | `function f() { const a = new Array(3); a[0] = 7; return a; } function main() { const b = f(); console.log(b[0]); } main();` |
| `s08_direct_fill.js` | `function f() { return new Array(3).fill(2); } function main() { console.log(f()[0]); } main();` |
| `s09_print_call_bound.js` | `function f() { return [1, 2, 3]; } function main() { const a = f(); console.log(a); } main();` |
| `s10_print_local.js` | `function main() { const a = new Array(3).fill(4); console.log(a); } main();` |
| `s11_string_elements.js` | `function mk() { const a = new Array(2).fill("x"); return a; } function main() { const c = mk(); console.log(c[0]); } main();` |
| `dyn_index.js` | `function f() { return [1, 2, 3]; } function main() { const a = f(); let i = 2; console.log(a[i]); } main();` |
| `bound_length.js` | `function f() { return [1, 2, 3]; } const a = f(); console.log(a.length);` |
| `direct_length.js` | `function f() { return [4, 5]; } console.log(f().length);` |
| `passed_on.js` | `function f() { return [1, 2, 3]; } function g(x) { return x[1]; } console.log(g(f()));` |
| `bound_passed_on.js` | `function f() { return [1, 2, 3]; } function g(x) { return x[2]; } function main() { const a = f(); console.log(g(a)); } main();` |
| `mutate_call_bound.js` | `function f() { return [1, 2, 3]; } function main() { const a = f(); a[1] = 9; console.log(a[1]); } main();` |
| `recursion.js` | `function f(n) { if (n === 0) { return [7, 8]; } return f(n - 1); } console.log(f(3)[1]);` |
| `mutual_recursion.js` | `function f(n) { if (n === 0) { return [5]; } return g(n - 1); } function g(n) { return f(n); } console.log(f(2)[0]);` |
| `loop_alloc_return.js` | `function f() { for (let i = 0; i < 3; i++) { const a = new Array(2).fill(i); if (i === 2) { return a; } } return new Array(1).fill(0); } console.log(f()[0]);` |
| `empty_literal.js` | `function f() { return []; } function main() { const a = f(); console.log(a.length); } main();` |
| `computed_elements.js` | `function f(x) { return [x, x * 2, x + 1]; } function main() { const a = f(5); console.log(a[0] + "," + a[1] + "," + a[2]); } main();` |
| `const_literal_return.js` | `function f() { const a = [1, 2, 3]; return a; } function main() { const b = f(); console.log(b[1]); } main();` |
| `let_literal_return.js` | `function f() { let a = [1, 2, 3]; return a; } function main() { const b = f(); console.log(b[1]); } main();` |
| `nested_decl.js` | `function main() { function f() { return [1, 2, 3]; } const a = f(); console.log(a[0] + "," + a[2]); } main();` |
| `arrow_return.js` | `const f = () => [1, 2, 3]; function main() { const a = f(); console.log(a[0]); } main();` |
| `mixed_returns.js` | `function f(c) { if (c) { return [1]; } return 0; } function main() { const a = f(true); console.log(a[0]); } main();` |
| `falls_off_end.js` | `function f(c) { if (c) { return [1]; } } function main() { const a = f(true); console.log(a[0]); } main();` |
| `if_else_both_return.js` | `function f(c) { if (c) { return [1]; } else { return [2]; } } function main() { const a = f(false); console.log(a[0]); } main();` |
| `float_elements.js` | `function f() { return [1.5, 2.5]; } function main() { const a = f(); console.log(a[0]); } main();` |
| `string_literal_elements.js` | `function f() { return ["a", "b"]; } function main() { const a = f(); console.log(a[0]); } main();` |
| `boolean_elements.js` | `function f() { return [true, false]; } function main() { const a = f(); console.log(a[0]); } main();` |
| `nested_array_elements.js` | `function f() { return [[1], [2]]; } function main() { const a = f(); console.log(a[0][0]); } main();` |
| `float_fill_return.js` | `function f() { const a = new Array(2).fill(1.5); return a; } function main() { const b = f(); console.log(b[0]); } main();` |
| `call_arg_direct.js` | `function f() { return [3, 4]; } function g(x) { return x.length; } console.log(g(f()));` |
| `object_return_control.js` | `function f() { return {a: 1}; } const r = f(); console.log(r.a);` |
| `r21_oob_control.js` | `function main() { const a = new Array(3).fill(4); console.log(a[5]); } main();` |
| `r48_module_control.js` | `const o = { arr: new Array(3).fill(6) }; console.log(o.arr[0]);` |
| `r48_function_control.js` | `function main() { const o = { arr: new Array(3).fill(6) }; console.log(o.arr[0]); } main();` |
| `growable_escape_control.js` | `function f() { const a = []; a.push(1); a.push(2); return a; } function main() { const b = f(); console.log(b[1]); } main();` |
| `reassign_control.js` | `function f() { const a = new Array(3).fill(4); return a; } function main() { let b; b = f(); console.log(b[1]); } main();` |
| `callback_escape.js` | `function mk(x) { return x; } function main() { const a = mk(new Array(2).fill(3)); const r = [1, 2].map(mk); console.log(a[0] + "," + r[1]); } main();` |

- [ ] **Step 3: Record the baseline**

```bash
cd /workspace
cargo build -p kali_cli
tools/array-return-probes/run.sh tools/array-return-probes/baseline.tsv
cut -f1,2 tools/array-return-probes/baseline.tsv
```

Expected: `s01`-`s08`, `s09`, `s10`, `s11` read `SILENT`; `object_return_control` reads `CORRECT`; `dyn_index` reads `REFUSES`. Any `s0x` row that is not `SILENT` means the baseline moved since the spec was measured. Stop and report.

- [ ] **Step 4: Write amendment A4 into the spec**

In `docs/superpowers/specs/2026-10-02-array-return-design.md`, append to §3.4 (after A3):

```markdown
**A4 — nested `function` declarations are admitted.** The register's own
in-function R-14 repro (`crates/kali_cli/tests/cases/oracle/tier2.toml`,
`r14_function.js`) declares `f` *inside* `main`. Under "top-level only" it would
refuse rather than compute. `repr_infer` and codegen both key a declaration by
its own name, at any depth (`repr_infer.rs:2044`, `visit_stmt`'s
`FunctionDeclaration` arm), and `emit_return` runs the same epilogue for both.
So a `function` declaration at **any** depth is a candidate, provided its name
is declared **exactly once** in the program. `self.functions` is last-wins on a
repeated name, so a repeated name is not one function. Arrow functions,
function expressions, methods and class members stay excluded (§3.1). The
`nested_decl` probe pins it.
```

In the same file, replace the two occurrences of "top-level `function`" in §1 and §3.1 with "`function` declaration (at any depth, A4)". Leave the reason string's wording to Task 2.

- [ ] **Step 5: Commit**

```bash
git add tools/array-return-probes docs/superpowers/specs/2026-10-02-array-return-design.md
git commit -m "test(array-return): the probe corpus, its baseline, and amendment A4"
```

---

### Task 2: The `ReprTable` facts and the refusal message (`kali_common`)

**Files:**
- Modify: `crates/kali_common/src/repr.rs` (struct `ReprTable` at `:83`, accessors next to `set_array_binding` at `:574`)
- Modify: `crates/kali_common/src/messages.rs`
- Test: `crates/kali_common/src/repr_tests.rs`

**Interfaces:**
- Produces:
  - `ReprTable::set_array_return(&mut self, func: &str, elem: Repr)`
  - `ReprTable::array_return(&self, func: &str) -> Option<Repr>`
  - `ReprTable::set_array_return_taint(&mut self, func: &str, reason: &'static str)`
  - `ReprTable::array_return_taint(&self, func: &str) -> Option<&'static str>`
  - `ReprTable::set_call_bound_array_binding(&mut self, func: &str, binding: &str)`
  - `ReprTable::is_call_bound_array_binding(&self, func: &str, binding: &str) -> bool`
  - `kali_common::array_return_refused_message(func: &str, reason: &str) -> String`
  - reason constants `ARRAY_RETURN_MIXED`, `ARRAY_RETURN_ELEMENT`, `ARRAY_RETURN_GROWABLE`, `ARRAY_RETURN_FORM`, `ARRAY_RETURN_LET_LITERAL` (all `&'static str`, exported from `kali_common`)

- [ ] **Step 1: Write the failing tests** (append to `crates/kali_common/src/repr_tests.rs`)

```rust
#[test]
fn array_return_facts_default_to_absent() {
    let t = ReprTable::default();
    assert_eq!(t.array_return("f"), None);
    assert_eq!(t.array_return_taint("f"), None);
    assert!(!t.is_call_bound_array_binding("main", "a"));
}

#[test]
fn array_return_facts_round_trip() {
    let mut t = ReprTable::default();
    t.set_array_return("f", Repr::I64);
    t.set_array_return_taint("g", crate::ARRAY_RETURN_MIXED);
    t.set_call_bound_array_binding("main", "a");
    assert_eq!(t.array_return("f"), Some(Repr::I64));
    assert_eq!(t.array_return_taint("g"), Some(crate::ARRAY_RETURN_MIXED));
    assert!(t.is_call_bound_array_binding("main", "a"));
    assert!(!t.is_call_bound_array_binding("f", "a"));
}

#[test]
fn array_return_refused_message_names_function_and_reason() {
    let m = crate::array_return_refused_message("f", crate::ARRAY_RETURN_MIXED);
    assert_eq!(
        m,
        "returning an array from `f` is unavailable in the current phase: it mixes array and non-array returns"
    );
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p kali_common array_return`
Expected: compile errors, `no method named array_return` and `cannot find function array_return_refused_message`.

- [ ] **Step 3: Implement**

In `ReprTable` (after `growable_array_bindings`):

```rust
    /// Functions that return a runtime `[len][elem…]` array on every path,
    /// with the element repr (array-return project, spec
    /// docs/superpowers/specs/2026-10-02-array-return-design.md §3.1).
    /// `return_repr` stays `I64` for them: the handle IS an i64.
    array_returns: HashMap<String, Repr>,
    /// Functions with at least one array-shaped return that are NOT
    /// array-returning, with the refusal reason. Every entry is also a
    /// shape conflict, so a program with one never reaches codegen.
    array_return_taints: HashMap<String, &'static str>,
    /// `(func, binding)` declared `const`/`let` with a bare-identifier call
    /// to an array-returning function as its initializer. Narrower than
    /// `array_bindings` on purpose: the resolver and codegen register a
    /// runtime array from THIS set, never from `array_bindings`, which
    /// over-proves array-ness for any bracket-indexed binding.
    call_bound_array_bindings: HashSet<(String, String)>,
```

Accessors (after `is_growable_array_binding`):

```rust
    pub fn set_array_return(&mut self, func: &str, elem: Repr) {
        self.array_returns.insert(func.to_string(), elem);
    }

    pub fn array_return(&self, func: &str) -> Option<Repr> {
        self.array_returns.get(func).copied()
    }

    pub fn set_array_return_taint(&mut self, func: &str, reason: &'static str) {
        self.array_return_taints.insert(func.to_string(), reason);
    }

    pub fn array_return_taint(&self, func: &str) -> Option<&'static str> {
        self.array_return_taints.get(func).copied()
    }

    pub fn set_call_bound_array_binding(&mut self, func: &str, binding: &str) {
        self.call_bound_array_bindings
            .insert((func.to_string(), binding.to_string()));
    }

    pub fn is_call_bound_array_binding(&self, func: &str, binding: &str) -> bool {
        self.call_bound_array_bindings
            .contains(&(func.to_string(), binding.to_string()))
    }
```

In `messages.rs` (and re-export from `lib.rs` the same way `computed_member_access_unavailable_message` is exported; check with `grep -n computed_member_access_unavailable_message crates/kali_common/src/lib.rs`):

```rust
/// Refusal reasons for an array-shaped return that is not admitted
/// (docs/superpowers/specs/2026-10-02-array-return-design.md §3.1).
pub const ARRAY_RETURN_MIXED: &str = "it mixes array and non-array returns";
pub const ARRAY_RETURN_ELEMENT: &str = "an element is not an integer";
pub const ARRAY_RETURN_GROWABLE: &str = "it returns a growable array";
pub const ARRAY_RETURN_FORM: &str =
    "only a `function` declaration with a unique name can return an array";
pub const ARRAY_RETURN_LET_LITERAL: &str =
    "it returns a `let`/`var` binding of an array literal, which can be reassigned";

pub fn array_return_refused_message(func: &str, reason: &str) -> String {
    format!("returning an array from `{func}` is unavailable in the current phase: {reason}")
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_common array_return`
Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common
git commit -m "feat(array-return): ReprTable carries array returns, taints and call-bound bindings"
```

---

### Task 3: The classifier and the solver (`kali_types/src/array_return.rs`)

A pure module: no union-find, no `ReprInfer`. Task 4 feeds it facts and applies its answer. It is kept separate so it can be tested on its own and so `repr_infer.rs` (6,862 lines) does not grow by the whole lane.

**Files:**
- Create: `crates/kali_types/src/array_return.rs`
- Create: `crates/kali_types/src/array_return_tests.rs`
- Modify: `crates/kali_types/src/lib.rs` (add `mod array_return;` next to `mod growable;` at `:9`)
- Modify: `crates/kali_types/src/resolve/expression.rs:3199` (`fn expression_is_array_allocation` becomes `pub(crate) fn`; the module path is `crate::resolve::expression`. If `resolve::expression` is a private module, also add `pub(crate)` to its `mod` line in `resolve/mod.rs`.)

**Interfaces:**
- Consumes: `kali_common::{ARRAY_RETURN_*}` (Task 2).
- Produces (all `pub(crate)`):

```rust
pub(crate) const RETURN_ARRAY_KEY: &str = "%return";

pub(crate) enum ReturnArg { Literal(Option<String>), Binding(String), Allocation, Call(String), BadArray(&'static str), NonArray }
pub(crate) enum ArgShape { Identifier(String), Allocation, Call(String), Other }

pub(crate) fn classify_return_arg(arg: Option<&Expression>, is_const_literal: &dyn Fn(&str) -> bool, is_let_literal: &dyn Fn(&str) -> bool) -> ReturnArg
pub(crate) fn literal_elements_are_integer_shaped(arr: &ArrayExpression) -> bool
pub(crate) fn arg_shape(arg: &Expression) -> ArgShape
pub(crate) fn body_falls_off_end(body: &[Statement]) -> bool

pub(crate) struct ArrayReturnFacts {
    pub(crate) declaration_counts: BTreeMap<String, usize>,
    pub(crate) candidate_forms: BTreeSet<String>,
    pub(crate) returns: BTreeMap<String, Vec<ReturnArg>>,
    pub(crate) falls_off_end: BTreeSet<String>,
    pub(crate) call_bound: Vec<(String, String, String)>, // (caller, binding, callee)
}

pub(crate) struct Feed { pub(crate) caller: String, pub(crate) callee: String, pub(crate) index: usize, pub(crate) shape: ArgShape }

pub(crate) struct Solution {
    pub(crate) array_returning: BTreeSet<String>,
    pub(crate) tainted: BTreeMap<String, &'static str>,
    pub(crate) call_bound: BTreeSet<(String, String)>,
    pub(crate) array_fed_params: BTreeSet<(String, String)>,
}

pub(crate) fn solve(
    facts: &ArrayReturnFacts,
    feeds: &[Feed],
    params: &BTreeMap<String, Vec<String>>,
    escaping: &BTreeSet<String>,
    is_base_runtime_array: &dyn Fn(&str, &str) -> bool,
) -> Solution
```

- [ ] **Step 1: Write the failing tests** (`array_return_tests.rs`)

```rust
use super::*;
use kali_ast::{Expression, Statement};
use std::collections::{BTreeMap, BTreeSet};

fn parse(src: &str) -> Vec<Statement> {
    crate::test_support::parse_statements(src)
}

/// The argument of the first `return` in the first function of `src`.
fn first_return_arg(src: &str) -> Option<Expression> {
    for stmt in parse(src) {
        if let Statement::FunctionDeclaration(decl) = stmt {
            for s in &decl.body.body {
                if let Statement::ReturnStatement(r) = s {
                    return r.argument.clone();
                }
            }
        }
    }
    panic!("no return in {src}");
}

fn classify(src: &str) -> ReturnArg {
    let arg = first_return_arg(src);
    classify_return_arg(arg.as_ref(), &|n| n == "lit", &|n| n == "letlit")
}

#[test]
fn integer_literal_is_literal() {
    assert_eq!(classify("function f() { return [1, 2, 3]; }"), ReturnArg::Literal(None));
}

#[test]
fn empty_literal_is_literal() {
    assert_eq!(classify("function f() { return []; }"), ReturnArg::Literal(None));
}

#[test]
fn computed_integer_elements_are_literal() {
    assert_eq!(classify("function f(x) { return [x, x * 2, -x]; }"), ReturnArg::Literal(None));
}

#[test]
fn boolean_string_nested_and_spread_elements_are_bad() {
    for src in [
        "function f() { return [true]; }",
        "function f() { return [\"a\"]; }",
        "function f() { return [[1]]; }",
        "function f(a) { return [...a]; }",
        "function f(x) { return [x > 1]; }",
        "function f() { return [null]; }",
    ] {
        assert_eq!(
            classify(src),
            ReturnArg::BadArray(kali_common::ARRAY_RETURN_ELEMENT),
            "{src}"
        );
    }
}

#[test]
fn const_literal_identifier_is_literal_and_let_literal_is_bad() {
    assert_eq!(classify("function f() { return lit; }"), ReturnArg::Literal(Some("lit".into())));
    assert_eq!(
        classify("function f() { return letlit; }"),
        ReturnArg::BadArray(kali_common::ARRAY_RETURN_LET_LITERAL)
    );
}

#[test]
fn other_identifier_is_binding() {
    assert_eq!(classify("function f(x) { return x; }"), ReturnArg::Binding("x".into()));
}

#[test]
fn allocations_are_allocation() {
    assert_eq!(classify("function f() { return new Array(3); }"), ReturnArg::Allocation);
    assert_eq!(classify("function f() { return new Array(3).fill(2); }"), ReturnArg::Allocation);
    assert_eq!(classify("function f() { return (new Array(3)); }"), ReturnArg::Allocation);
}

#[test]
fn bare_call_is_call() {
    assert_eq!(classify("function f() { return g(); }"), ReturnArg::Call("g".into()));
}

#[test]
fn scalars_and_bare_return_are_non_array() {
    assert_eq!(classify("function f() { return 0; }"), ReturnArg::NonArray);
    assert_eq!(classify("function f() { return {a: 1}; }"), ReturnArg::NonArray);
    assert_eq!(classify("function f() { return; }"), ReturnArg::NonArray);
}

fn falls_off(src: &str) -> bool {
    for stmt in parse(src) {
        if let Statement::FunctionDeclaration(decl) = stmt {
            return body_falls_off_end(&decl.body.body);
        }
    }
    unreachable!()
}

#[test]
fn falls_off_end_detection() {
    assert!(!falls_off("function f() { return [1]; }"));
    assert!(falls_off("function f(c) { if (c) { return [1]; } }"));
    assert!(!falls_off("function f(c) { if (c) { return [1]; } else { return [2]; } }"));
    assert!(!falls_off("function f() { throw 1; }"));
    assert!(falls_off("function f() { }"));
}

#[test]
fn if_else_both_returning_does_not_fall_off() {
    assert!(!falls_off(
        "function f(c) { const x = 1; if (c) { return [x]; } else { if (c) { return [2]; } else { return [3]; } } }"
    ));
}

fn facts_one(name: &str, returns: Vec<ReturnArg>) -> ArrayReturnFacts {
    let mut f = ArrayReturnFacts::default();
    f.declaration_counts.insert(name.into(), 1);
    f.candidate_forms.insert(name.into());
    f.returns.insert(name.into(), returns);
    f
}

fn no_base(_: &str, _: &str) -> bool {
    false
}

#[test]
fn literal_only_function_is_array_returning() {
    let facts = facts_one("f", vec![ReturnArg::Literal(None)]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.contains("f"));
    assert!(s.tainted.is_empty());
}

#[test]
fn mixed_function_is_tainted_mixed() {
    let facts = facts_one("f", vec![ReturnArg::Literal(None), ReturnArg::NonArray]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(!s.array_returning.contains("f"));
    assert_eq!(s.tainted.get("f"), Some(&kali_common::ARRAY_RETURN_MIXED));
}

#[test]
fn falling_off_the_end_taints_mixed() {
    let mut facts = facts_one("f", vec![ReturnArg::Literal(None)]);
    facts.falls_off_end.insert("f".into());
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("f"), Some(&kali_common::ARRAY_RETURN_MIXED));
}

#[test]
fn bad_array_reason_wins() {
    let facts = facts_one(
        "f",
        vec![ReturnArg::Literal(None), ReturnArg::BadArray(kali_common::ARRAY_RETURN_ELEMENT)],
    );
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("f"), Some(&kali_common::ARRAY_RETURN_ELEMENT));
}

#[test]
fn non_candidate_form_with_array_return_is_tainted_form() {
    let mut facts = facts_one("__kali_fn_0", vec![ReturnArg::Literal(None)]);
    facts.candidate_forms.clear();
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("__kali_fn_0"), Some(&kali_common::ARRAY_RETURN_FORM));
}

#[test]
fn repeated_name_is_tainted_form() {
    let mut facts = facts_one("f", vec![ReturnArg::Literal(None)]);
    facts.declaration_counts.insert("f".into(), 2);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("f"), Some(&kali_common::ARRAY_RETURN_FORM));
}

#[test]
fn recursion_is_array_returning() {
    let facts = facts_one("f", vec![ReturnArg::Literal(None), ReturnArg::Call("f".into())]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.contains("f"));
}

#[test]
fn mutual_recursion_is_array_returning() {
    let mut facts = facts_one("f", vec![ReturnArg::Literal(None), ReturnArg::Call("g".into())]);
    facts.declaration_counts.insert("g".into(), 1);
    facts.candidate_forms.insert("g".into());
    facts.returns.insert("g".into(), vec![ReturnArg::Call("f".into())]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.contains("f"));
    assert!(s.array_returning.contains("g"));
}

#[test]
fn call_to_non_array_function_is_not_array_shaped() {
    let mut facts = facts_one("f", vec![ReturnArg::Call("g".into())]);
    facts.declaration_counts.insert("g".into(), 1);
    facts.candidate_forms.insert("g".into());
    facts.returns.insert("g".into(), vec![ReturnArg::NonArray]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.is_empty());
    assert!(s.tainted.is_empty(), "a scalar-returning call chain is untouched");
}

#[test]
fn call_bound_binding_makes_return_admitted() {
    let mut facts = facts_one("f", vec![ReturnArg::Literal(None)]);
    facts.declaration_counts.insert("g".into(), 1);
    facts.candidate_forms.insert("g".into());
    facts.returns.insert("g".into(), vec![ReturnArg::Binding("a".into())]);
    facts.call_bound.push(("g".into(), "a".into(), "f".into()));
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.call_bound.contains(&("g".to_string(), "a".to_string())));
    assert!(s.array_returning.contains("g"));
}

#[test]
fn param_fed_only_arrays_is_array_fed() {
    let facts = facts_one("f", vec![ReturnArg::Binding("x".into())]);
    let mut params = BTreeMap::new();
    params.insert("f".to_string(), vec!["x".to_string()]);
    let feeds = vec![Feed {
        caller: "_start".into(),
        callee: "f".into(),
        index: 0,
        shape: ArgShape::Allocation,
    }];
    let s = solve(&facts, &feeds, &params, &BTreeSet::new(), &no_base);
    assert!(s.array_fed_params.contains(&("f".to_string(), "x".to_string())));
    assert!(s.array_returning.contains("f"));
}

#[test]
fn param_with_any_scalar_feed_is_not_array_fed() {
    let facts = facts_one("f", vec![ReturnArg::Binding("x".into())]);
    let mut params = BTreeMap::new();
    params.insert("f".to_string(), vec!["x".to_string()]);
    let feeds = vec![
        Feed { caller: "_start".into(), callee: "f".into(), index: 0, shape: ArgShape::Allocation },
        Feed { caller: "_start".into(), callee: "f".into(), index: 0, shape: ArgShape::Other },
    ];
    let s = solve(&facts, &feeds, &params, &BTreeSet::new(), &no_base);
    assert!(s.array_fed_params.is_empty());
    assert!(s.array_returning.is_empty());
}

#[test]
fn escaping_function_params_are_never_array_fed() {
    let facts = facts_one("f", vec![ReturnArg::Binding("x".into())]);
    let mut params = BTreeMap::new();
    params.insert("f".to_string(), vec!["x".to_string()]);
    let feeds = vec![Feed {
        caller: "_start".into(),
        callee: "f".into(),
        index: 0,
        shape: ArgShape::Allocation,
    }];
    let escaping: BTreeSet<String> = ["f".to_string()].into();
    let s = solve(&facts, &feeds, &params, &escaping, &no_base);
    assert!(s.array_fed_params.is_empty());
}

#[test]
fn base_runtime_array_binding_is_admitted() {
    let facts = facts_one("f", vec![ReturnArg::Binding("a".into())]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &|f, n| f == "f" && n == "a");
    assert!(s.array_returning.contains("f"));
}

#[test]
fn non_array_binding_return_is_untouched() {
    let facts = facts_one("f", vec![ReturnArg::Binding("n".into())]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.is_empty());
    assert!(s.tainted.is_empty());
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p kali_types array_return`
Expected: compile error, `file not found for module array_return`.

- [ ] **Step 3: Implement `array_return.rs`**

```rust
//! Array-return lane (docs/superpowers/specs/2026-10-02-array-return-design.md
//! §3.1, amendments A2-A4): which functions return a runtime `[len][elem…]`
//! array on every path, which locals are bound to such a call, and which params
//! only ever receive arrays.
//!
//! Pure on purpose. `repr_infer` records the facts during its body walk and
//! applies the [`Solution`] (element-node unions, `ReprTable` writes); this
//! module owns the classification and the fixed point, so both can be tested
//! without the union-find.

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::{ArrayExpression, Expression, ExpressionOrSpread, LiteralValue, Statement};

/// The reserved element-node key for a function's returned array. Not a legal
/// identifier, so it cannot collide with a binding.
pub(crate) const RETURN_ARRAY_KEY: &str = "%return";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ReturnArg {
    /// An array literal of integer-shaped elements (`None`), or a `const`
    /// binding of one (`Some(name)`, amendment A2).
    Literal(Option<String>),
    /// A bare identifier; the fixed point decides whether it is a runtime array.
    Binding(String),
    /// `new Array(n)`, `Array(n)`, `new Array(n).fill(v)`.
    Allocation,
    /// A bare-identifier call.
    Call(String),
    /// Array-shaped but never admitted; carries the refusal reason.
    BadArray(&'static str),
    /// Anything else, including a bare `return;`.
    NonArray,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ArgShape {
    Identifier(String),
    Allocation,
    Call(String),
    Other,
}

fn unparen(expr: &Expression) -> &Expression {
    match expr {
        Expression::ParenthesizedExpression(inner) => unparen(&inner.expression),
        other => other,
    }
}

fn is_allocation(expr: &Expression) -> bool {
    crate::resolve::expression::expression_is_array_allocation(expr)
}

/// True when every element is an expression whose JS value is a number on
/// every evaluation and whose repr the solver can then check for `I64`. A
/// boolean-valued element (`true`, `x > 1`, `!x`, `a && b`) is excluded: kali
/// stores it as `1`/`0`, and a caller printing it would see `1`, not `true`.
pub(crate) fn literal_elements_are_integer_shaped(arr: &ArrayExpression) -> bool {
    arr.elements.iter().all(|element| match element {
        Some(ExpressionOrSpread::Expression(expr)) => element_is_integer_shaped(expr),
        Some(ExpressionOrSpread::Spread(_)) | Some(ExpressionOrSpread::Empty) | None => false,
    })
}

fn element_is_integer_shaped(expr: &Expression) -> bool {
    match unparen(expr) {
        Expression::Literal(LiteralValue::Number(n)) => n.fract() == 0.0,
        Expression::Identifier(_)
        | Expression::CallExpression(_)
        | Expression::MemberExpression(_)
        | Expression::UpdateExpression(_) => true,
        Expression::UnaryExpression(u) => {
            matches!(u.operator.as_str(), "-" | "+" | "~") && element_is_integer_shaped(&u.argument)
        }
        Expression::BinaryExpression(b) => matches!(
            b.operator.as_str(),
            "+" | "-" | "*" | "%" | "|" | "&" | "^" | "<<" | ">>" | ">>>"
        ),
        _ => false,
    }
}

/// Classify one `return` argument. `is_const_literal(name)` is true when `name`
/// is a `const` binding of an array literal in the returning function;
/// `is_let_literal(name)` when it is a `let`/`var` one.
///
/// Exhaustive over `Expression` with no wildcard arm, as `growable.rs`'s
/// scanner is: a new AST variant must be classified here before it compiles.
pub(crate) fn classify_return_arg(
    arg: Option<&Expression>,
    is_const_literal: &dyn Fn(&str) -> bool,
    is_let_literal: &dyn Fn(&str) -> bool,
) -> ReturnArg {
    let Some(arg) = arg else {
        return ReturnArg::NonArray;
    };
    if is_allocation(arg) {
        return ReturnArg::Allocation;
    }
    match unparen(arg) {
        Expression::ArrayExpression(arr) => {
            if literal_elements_are_integer_shaped(arr) {
                ReturnArg::Literal(None)
            } else {
                ReturnArg::BadArray(kali_common::ARRAY_RETURN_ELEMENT)
            }
        }
        Expression::Identifier(name) => {
            if is_const_literal(name) {
                ReturnArg::Literal(Some(name.clone()))
            } else if is_let_literal(name) {
                ReturnArg::BadArray(kali_common::ARRAY_RETURN_LET_LITERAL)
            } else {
                ReturnArg::Binding(name.clone())
            }
        }
        Expression::CallExpression(call) => match unparen(&call.callee) {
            Expression::Identifier(callee) => ReturnArg::Call(callee.clone()),
            _ => ReturnArg::NonArray,
        },
        Expression::Literal(_)
        | Expression::BinaryExpression(_)
        | Expression::UnaryExpression(_)
        | Expression::MemberExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::NewExpression(_)
        | Expression::MetaProperty(_)
        | Expression::TemplateLiteral(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::SequenceExpression(_)
        | Expression::ParenthesizedExpression(_)
        | Expression::YieldExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::OptionalChainExpression(_)
        | Expression::ChainExpression(_)
        | Expression::SpreadElement(_)
        | Expression::RestElement(_)
        | Expression::ImportExpression(_)
        | Expression::DecoratedExpression(_)
        | Expression::JsxElement(_)
        | Expression::JsxFragment(_)
        | Expression::JsxEmptyExpression
        | Expression::BigIntLiteral(_) => ReturnArg::NonArray,
    }
}
```

The variant list above was read from `crates/kali_ast/src/expression.rs` at `5e3d85bd2`. If `cargo build` reports a missing or unknown variant, adjust the list to the enum. **Never add a `_` arm.** A `ConditionalExpression` whose arms are both arrays (`return c ? [1] : [2]`) is deliberately `NonArray`: it keeps its current lane and is filed in Task 10.

```rust
/// Argument shape at a call site, for the array-fed param fact (A3).
pub(crate) fn arg_shape(arg: &Expression) -> ArgShape {
    if is_allocation(arg) {
        return ArgShape::Allocation;
    }
    match unparen(arg) {
        Expression::Identifier(name) => ArgShape::Identifier(name.clone()),
        Expression::CallExpression(call) => match unparen(&call.callee) {
            Expression::Identifier(callee) => ArgShape::Call(callee.clone()),
            _ => ArgShape::Other,
        },
        _ => ArgShape::Other,
    }
}

/// Conservative: true unless the body provably ends in `return`/`throw` on
/// every path (a trailing `return`/`throw`, or a trailing `if`/`else` whose two
/// arms both provably end so). A false "falls off" taints an otherwise-good
/// function, which refuses; it never admits a bad one.
pub(crate) fn body_falls_off_end(body: &[Statement]) -> bool {
    match body.last() {
        Some(Statement::ReturnStatement(_)) | Some(Statement::ThrowStatement(_)) => false,
        Some(Statement::BlockStatement(block)) => body_falls_off_end(&block.body),
        Some(Statement::IfStatement(stmt)) => match &stmt.alternate {
            Some(alt) => body_falls_off_end(&stmt.consequent.body) || body_falls_off_end(&alt.body),
            None => true,
        },
        _ => true,
    }
}

#[derive(Default, Debug)]
pub(crate) struct ArrayReturnFacts {
    /// Every `function` declaration name, with how many times it is declared.
    pub(crate) declaration_counts: BTreeMap<String, usize>,
    /// Functions of a candidate form: a `function` declaration (any depth, A4).
    pub(crate) candidate_forms: BTreeSet<String>,
    /// Every function's classified returns, candidate form or not.
    pub(crate) returns: BTreeMap<String, Vec<ReturnArg>>,
    pub(crate) falls_off_end: BTreeSet<String>,
    /// `(caller, binding, callee)` for `const`/`let binding = callee()`.
    pub(crate) call_bound: Vec<(String, String, String)>,
}

#[derive(Clone, Debug)]
pub(crate) struct Feed {
    pub(crate) caller: String,
    pub(crate) callee: String,
    pub(crate) index: usize,
    pub(crate) shape: ArgShape,
}

#[derive(Default, Debug)]
pub(crate) struct Solution {
    pub(crate) array_returning: BTreeSet<String>,
    pub(crate) tainted: BTreeMap<String, &'static str>,
    pub(crate) call_bound: BTreeSet<(String, String)>,
    pub(crate) array_fed_params: BTreeSet<(String, String)>,
}

impl ArrayReturnFacts {
    fn is_candidate(&self, func: &str) -> bool {
        self.candidate_forms.contains(func) && self.declaration_counts.get(func) == Some(&1)
    }
}

/// The optimistic fixed point (spec §3.1, A3). Start with every candidate
/// whose returns are all array-shaped; repeatedly drop any function whose
/// returns are not all admitted under the current sets; recompute the
/// call-bound and array-fed sets from the survivors; stop when nothing
/// changes. Everything only shrinks, so it terminates.
///
/// `is_base_runtime_array(func, name)` answers for the runtime arrays that
/// exist before this lane: an allocation local and an already-array param.
pub(crate) fn solve(
    facts: &ArrayReturnFacts,
    feeds: &[Feed],
    params: &BTreeMap<String, Vec<String>>,
    escaping: &BTreeSet<String>,
    is_base_runtime_array: &dyn Fn(&str, &str) -> bool,
) -> Solution {
    let syntactically_possible = |arg: &ReturnArg| {
        matches!(
            arg,
            ReturnArg::Literal(_) | ReturnArg::Allocation | ReturnArg::Binding(_) | ReturnArg::Call(_)
        )
    };
    let mut returning: BTreeSet<String> = facts
        .returns
        .iter()
        .filter(|(f, args)| {
            facts.is_candidate(f)
                && !facts.falls_off_end.contains(*f)
                && !args.is_empty()
                && args.iter().all(syntactically_possible)
        })
        .map(|(f, _)| f.clone())
        .collect();

    // Params with at least one feed, of a candidate, non-escaping callee.
    let mut feeds_by_param: BTreeMap<(String, String), Vec<&Feed>> = BTreeMap::new();
    for feed in feeds {
        if escaping.contains(&feed.callee) || !facts.is_candidate(&feed.callee) {
            continue;
        }
        if let Some(name) = params.get(&feed.callee).and_then(|ps| ps.get(feed.index)) {
            feeds_by_param
                .entry((feed.callee.clone(), name.clone()))
                .or_default()
                .push(feed);
        }
    }

    loop {
        let call_bound: BTreeSet<(String, String)> = facts
            .call_bound
            .iter()
            .filter(|(_, _, callee)| returning.contains(callee))
            .map(|(caller, binding, _)| (caller.clone(), binding.clone()))
            .collect();

        // Inner optimistic fixed point for array-fed params.
        let mut fed: BTreeSet<(String, String)> = feeds_by_param.keys().cloned().collect();
        loop {
            let is_array = |func: &str, name: &str, fed: &BTreeSet<(String, String)>| {
                is_base_runtime_array(func, name)
                    || call_bound.contains(&(func.to_string(), name.to_string()))
                    || fed.contains(&(func.to_string(), name.to_string()))
            };
            let next: BTreeSet<(String, String)> = fed
                .iter()
                .filter(|key| {
                    feeds_by_param[*key].iter().all(|feed| match &feed.shape {
                        ArgShape::Allocation => true,
                        ArgShape::Call(g) => returning.contains(g),
                        ArgShape::Identifier(n) => is_array(&feed.caller, n, &fed),
                        ArgShape::Other => false,
                    })
                })
                .cloned()
                .collect();
            if next == fed {
                break;
            }
            fed = next;
        }

        let is_array = |func: &str, name: &str| {
            is_base_runtime_array(func, name)
                || call_bound.contains(&(func.to_string(), name.to_string()))
                || fed.contains(&(func.to_string(), name.to_string()))
        };
        let next: BTreeSet<String> = returning
            .iter()
            .filter(|f| {
                facts.returns[*f].iter().all(|arg| match arg {
                    ReturnArg::Literal(_) | ReturnArg::Allocation => true,
                    ReturnArg::Binding(n) => is_array(f, n),
                    ReturnArg::Call(g) => returning.contains(g),
                    ReturnArg::BadArray(_) | ReturnArg::NonArray => false,
                })
            })
            .cloned()
            .collect();
        if next == returning {
            let mut tainted = BTreeMap::new();
            for (f, args) in &facts.returns {
                if returning.contains(f) {
                    continue;
                }
                let array_shaped = |arg: &ReturnArg| match arg {
                    ReturnArg::Literal(_) | ReturnArg::Allocation | ReturnArg::BadArray(_) => true,
                    ReturnArg::Binding(n) => is_array(f, n),
                    ReturnArg::Call(g) => returning.contains(g),
                    ReturnArg::NonArray => false,
                };
                if !args.iter().any(array_shaped) {
                    continue;
                }
                let reason = args
                    .iter()
                    .find_map(|arg| match arg {
                        ReturnArg::BadArray(reason) => Some(*reason),
                        _ => None,
                    })
                    .unwrap_or(if facts.is_candidate(f) {
                        kali_common::ARRAY_RETURN_MIXED
                    } else {
                        kali_common::ARRAY_RETURN_FORM
                    });
                tainted.insert(f.clone(), reason);
            }
            return Solution {
                array_returning: returning,
                tainted,
                call_bound,
                array_fed_params: fed,
            };
        }
        returning = next;
    }
}

#[cfg(test)]
#[path = "array_return_tests.rs"]
mod array_return_tests;
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_types array_return`
Expected: every test in `array_return_tests.rs` passes.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_types/src/array_return.rs crates/kali_types/src/array_return_tests.rs crates/kali_types/src/lib.rs crates/kali_types/src/resolve
git commit -m "feat(array-return): classify returns and solve the array-return fixed point"
```

---

### Task 4: Wire the lane into `repr_infer`

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs`
- Test: `crates/kali_types/src/repr_infer_tests.rs`
- Create: `crates/kali_cli/tests/cases/runtime/array_return_refusals.toml` (the taint rows only)

**Interfaces:**
- Consumes: `crate::array_return::*` (Task 3) and the `ReprTable` setters (Task 2).
- Produces: after `infer_reprs`:
  - `array_return(f) == Some(Repr::I64)` for each admitted `f`.
  - `is_call_bound_array_binding(caller, b)` for each admitted call-bound binding, which is also `is_array_binding`.
  - an array-fed param is `is_array_binding(f, p)`.
  - every taint is both an `array_return_taint(f)` and a `shape_conflicts()` entry carrying `array_return_refused_message(f, reason)`.

- [ ] **Step 1: Write the failing inference tests** (append to `repr_infer_tests.rs`)

```rust
#[test]
fn array_return_literal_function_and_call_bound_binding() {
    let t = reprs("function f() { return [1, 2, 3]; }\nfunction main() { const a = f(); console.log(a[0]); }\nmain();\n");
    assert_eq!(t.array_return("f"), Some(Repr::I64));
    assert!(t.is_call_bound_array_binding("main", "a"));
    assert!(t.is_array_binding("main", "a"));
    assert!(t.shape_conflicts().is_empty());
}

#[test]
fn array_return_const_literal_binding_is_admitted() {
    let t = reprs("function f() { const a = [1, 2, 3]; return a; }\nconst b = f();\n");
    assert_eq!(t.array_return("f"), Some(Repr::I64));
    assert!(t.is_call_bound_array_binding("_start", "b"));
}

#[test]
fn array_return_param_pass_through_is_array_fed() {
    let t = reprs("function f(x) { return x; }\nconst a = f(new Array(3).fill(1));\n");
    assert_eq!(t.array_return("f"), Some(Repr::I64));
    assert!(t.is_array_binding("f", "x"));
    assert!(t.is_call_bound_array_binding("_start", "a"));
}

#[test]
fn array_return_float_elements_taint_after_solving() {
    let t = reprs("function f() { const a = new Array(2).fill(1.5); return a; }\nconst b = f();\n");
    assert_eq!(t.array_return("f"), None);
    assert_eq!(t.array_return_taint("f"), Some(kali_common::ARRAY_RETURN_ELEMENT));
    assert!(t
        .shape_conflicts()
        .iter()
        .any(|m| m.contains("returning an array from `f`")));
}

#[test]
fn array_return_string_fill_taints() {
    let t = reprs("function mk() { const a = new Array(2).fill(\"x\"); return a; }\nfunction main() { const c = mk(); }\n");
    assert_eq!(t.array_return_taint("mk"), Some(kali_common::ARRAY_RETURN_ELEMENT));
}

#[test]
fn array_return_mixed_taints() {
    let t = reprs("function f(c) { if (c) { return [1]; } return 0; }\n");
    assert_eq!(t.array_return_taint("f"), Some(kali_common::ARRAY_RETURN_MIXED));
}

#[test]
fn array_return_arrow_taints_form() {
    let t = reprs("const f = () => { return [1, 2]; };\n");
    assert!(t
        .shape_conflicts()
        .iter()
        .any(|m| m.contains(kali_common::ARRAY_RETURN_FORM)));
}

#[test]
fn array_return_nested_declaration_is_admitted() {
    let t = reprs("function main() { function f() { return [1, 2, 3]; } const a = f(); }\nmain();\n");
    assert_eq!(t.array_return("f"), Some(Repr::I64));
    assert!(t.is_call_bound_array_binding("main", "a"));
}

#[test]
fn array_return_absent_for_programs_without_array_returns() {
    let t = reprs("function f(x) { return x + 1; }\nconst a = f(2);\nfunction g() { return {a: 1}; }\n");
    assert_eq!(t.array_return("f"), None);
    assert_eq!(t.array_return("g"), None);
    assert!(!t.is_call_bound_array_binding("_start", "a"));
    assert!(t.shape_conflicts().is_empty());
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p kali_types array_return_`
Expected: the `array_return_*` tests fail (`array_return` returns `None`; no taints). `array_return_absent_for_programs_without_array_returns` passes already.

- [ ] **Step 3: Add the facts to `ReprInfer`**

Next to `array_binding_returns` (`:735`), add:

```rust
    /// Array-return lane facts (spec 2026-10-02-array-return-design.md §3.1),
    /// recorded in Phase B and solved in Phase C0.
    array_return_facts: crate::array_return::ArrayReturnFacts,
    /// `(func, binding)` declared `const` with an array-literal initializer
    /// (A2), and `let`/`var` ones, for the return classifier.
    const_literal_array_bindings: BTreeSet<(String, String)>,
    let_literal_array_bindings: BTreeSet<(String, String)>,
    /// `(func, binding)` declared `const`/`let` with an allocation initializer:
    /// the runtime arrays codegen's declarator lanes already allocate.
    allocation_bindings: BTreeSet<(String, String)>,
```

`CallEdge` gains `arg_array_shapes: Vec<crate::array_return::ArgShape>` and a `caller: String`. Fill both where the edge is built (`:4584-4615`): `arg_array_shapes.push(crate::array_return::arg_shape(arg));` inside the existing argument loop, and `caller: func.to_string()` in the struct literal.

- [ ] **Step 4: Record during Phase B**

(a) `visit_stmt`'s `FunctionDeclaration` arm (`:2657`). Before walking the body:

```rust
                *self
                    .array_return_facts
                    .declaration_counts
                    .entry(decl.name.clone())
                    .or_insert(0) += 1;
                self.array_return_facts
                    .candidate_forms
                    .insert(decl.name.clone());
                if crate::array_return::body_falls_off_end(&decl.body.body) {
                    self.array_return_facts
                        .falls_off_end
                        .insert(decl.name.clone());
                }
```

A function expression or arrow is walked under its `__kali_fn_N` name, which is never in `candidate_forms`. Those functions are classified as `ARRAY_RETURN_FORM` by the solver if any return is array-shaped.

(b) `visit_declarator_init` (`:2979`). At the top, before the existing `if kind == "const"` block:

```rust
        match crate::array_return::classify_init(init) {
            crate::array_return::InitKind::ArrayLiteral => {
                let key = (func.to_string(), id.to_string());
                if kind == "const" {
                    self.const_literal_array_bindings.insert(key);
                } else {
                    self.let_literal_array_bindings.insert(key);
                }
            }
            crate::array_return::InitKind::Allocation => {
                if kind != "var" {
                    self.allocation_bindings
                        .insert((func.to_string(), id.to_string()));
                }
            }
            crate::array_return::InitKind::Call(callee) => {
                if kind != "var" {
                    self.array_return_facts.call_bound.push((
                        func.to_string(),
                        id.to_string(),
                        callee,
                    ));
                }
            }
            crate::array_return::InitKind::Other => {}
        }
```

and add to `array_return.rs` (with tests `classify_init_kinds` in `array_return_tests.rs` covering each arm):

```rust
pub(crate) enum InitKind { ArrayLiteral, Allocation, Call(String), Other }

pub(crate) fn classify_init(init: &Expression) -> InitKind {
    if is_allocation(init) {
        return InitKind::Allocation;
    }
    match unparen(init) {
        Expression::ArrayExpression(_) => InitKind::ArrayLiteral,
        Expression::CallExpression(call) => match unparen(&call.callee) {
            Expression::Identifier(callee) => InitKind::Call(callee.clone()),
            _ => InitKind::Other,
        },
        _ => InitKind::Other,
    }
}
```

(c) The `ReturnStatement` arm (`:2711`). Classify first, then route an array-literal argument's elements into the function's return element node, so its repr is solved like any array's:

```rust
                let class = crate::array_return::classify_return_arg(
                    stmt.argument.as_ref(),
                    &|n| self.const_literal_array_bindings.contains(&(func.to_string(), n.to_string())),
                    &|n| self.let_literal_array_bindings.contains(&(func.to_string(), n.to_string())),
                );
                self.array_return_facts
                    .returns
                    .entry(func.to_string())
                    .or_default()
                    .push(class.clone());
```

Then, in the existing `else` branch, replace `let rn = self.visit_expr(func, arg);` with:

```rust
                        let rn = match (&class, arg) {
                            (
                                crate::array_return::ReturnArg::Literal(None),
                                Expression::ArrayExpression(_),
                            ) => {
                                self.note_array_init(
                                    func,
                                    crate::array_return::RETURN_ARRAY_KEY,
                                    arg,
                                );
                                self.new_node()
                            }
                            _ => self.visit_expr(func, arg),
                        };
```

Before relying on this, confirm that `visit_expr` has no `ArrayExpression` arm of its own: run `awk 'NR>=3438 && NR<=4660 && /ArrayExpression/' crates/kali_types/src/repr_infer.rs`. Expected: no line inside `visit_expr`'s top-level match, so an array literal there reached the `_ => self.new_node()` default, and `note_array_init` (which visits each element once) visits no element twice. If an arm does exist, do not call `note_array_init`. Instead read that arm and route its element nodes into `array_elem_node_for(func, RETURN_ARRAY_KEY)` with `add_edge(en, elem)` plus `element_store_sources.push((elem, en))`.

The *Allocation* class records its element repr through the fill value. In the same `match`, add an arm before `_`:

```rust
                            (crate::array_return::ReturnArg::Allocation, _) => {
                                let elem = self.array_elem_node_for(
                                    func,
                                    crate::array_return::RETURN_ARRAY_KEY,
                                );
                                if let Some(value) = crate::array_return::fill_value(arg) {
                                    let vn = self.visit_expr(func, value);
                                    self.add_edge(vn, elem);
                                    self.element_store_sources.push((elem, vn));
                                    self.new_node()
                                } else {
                                    self.visit_expr(func, arg)
                                }
                            }
```

with, in `array_return.rs`:

```rust
/// The `v` of an allocation spelled `new Array(n).fill(v)` / `Array(n).fill(v)`.
pub(crate) fn fill_value(expr: &Expression) -> Option<&Expression> {
    let call = match unparen(expr) {
        Expression::NewExpression(n) => match unparen(&n.callee) {
            Expression::CallExpression(call) => call,
            _ => return None,
        },
        Expression::CallExpression(call) => call,
        _ => return None,
    };
    match unparen(&call.callee) {
        Expression::MemberExpression(m) if m.dot_name() == Some("fill") => call.args.first(),
        _ => None,
    }
}
```

(Test `fill_value_finds_the_value` in `array_return_tests.rs`: `Some` for `new Array(3).fill(4)`, `None` for `new Array(3)`.)

- [ ] **Step 5: Phase C0, solve and union**

Add a method and call it from `infer_reprs` **between** `assert_nested_fn_lockstep()` and `resolve_calls()`:

```rust
    // Phase C0 (array-return lane, spec 2026-10-02 §3.1 + A1-A4): solve which
    // functions return a runtime array, which locals are bound to such a call,
    // and which params only ever receive arrays; then union their element
    // nodes so the existing element-repr solve covers them. Runs BEFORE
    // `resolve_calls`, whose array-param fixpoint seeds from
    // `array_elem_node`'s keys, so the new array bindings propagate too.
    fn resolve_array_returns(&mut self) {
        let feeds: Vec<crate::array_return::Feed> = self
            .calls
            .iter()
            .flat_map(|edge| {
                edge.arg_array_shapes
                    .iter()
                    .enumerate()
                    .map(|(index, shape)| crate::array_return::Feed {
                        caller: edge.caller.clone(),
                        callee: edge.callee.clone(),
                        index,
                        shape: shape.clone(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let params: BTreeMap<String, Vec<String>> = self
            .functions
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let allocation_bindings = self.allocation_bindings.clone();
        let elem_keys: BTreeSet<(String, String)> =
            self.array_elem_node.keys().cloned().collect();
        let functions = self.functions.clone();
        let is_base = |func: &str, name: &str| {
            let key = (func.to_string(), name.to_string());
            allocation_bindings.contains(&key)
                || (functions
                    .get(func)
                    .is_some_and(|ps| ps.iter().any(|p| p == name))
                    && elem_keys.contains(&key))
        };
        let solution = crate::array_return::solve(
            &self.array_return_facts,
            &feeds,
            &params,
            &self.escaping_function_names,
            &is_base,
        );

        let ret = crate::array_return::RETURN_ARRAY_KEY;
        for f in &solution.array_returning {
            let f_elem = self.array_elem_node_for(f, ret);
            for arg in self.array_return_facts.returns[f].clone() {
                match arg {
                    crate::array_return::ReturnArg::Binding(n) => {
                        let n_elem = self.array_elem_node_for(f, &n);
                        self.uf.union(f_elem, n_elem);
                    }
                    crate::array_return::ReturnArg::Call(g) => {
                        let g_elem = self.array_elem_node_for(&g, ret);
                        self.uf.union(f_elem, g_elem);
                    }
                    crate::array_return::ReturnArg::Literal(Some(n)) => {
                        // A `const` literal binding (A2): its elements already
                        // flow into its own element node (`note_array_init`);
                        // join that node to the return node.
                        let n_elem = self.array_elem_node_for(f, &n);
                        self.uf.union(f_elem, n_elem);
                    }
                    _ => {}
                }
            }
        }
        for (caller, binding, callee) in self.array_return_facts.call_bound.clone() {
            if solution
                .call_bound
                .contains(&(caller.clone(), binding.clone()))
            {
                let b_elem = self.array_elem_node_for(&caller, &binding);
                let f_elem = self.array_elem_node_for(&callee, ret);
                self.uf.union(b_elem, f_elem);
            }
        }
        for feed in &feeds {
            let Some(param) = params.get(&feed.callee).and_then(|ps| ps.get(feed.index)) else {
                continue;
            };
            if !solution
                .array_fed_params
                .contains(&(feed.callee.clone(), param.clone()))
            {
                continue;
            }
            let p_elem = self.array_elem_node_for(&feed.callee, param);
            match &feed.shape {
                crate::array_return::ArgShape::Identifier(n) => {
                    let a_elem = self.array_elem_node_for(&feed.caller, n);
                    self.uf.union(p_elem, a_elem);
                }
                crate::array_return::ArgShape::Call(g) => {
                    let g_elem = self.array_elem_node_for(g, ret);
                    self.uf.union(p_elem, g_elem);
                }
                _ => {}
            }
        }
        self.array_return_solution = solution;
    }
```

Add the field `array_return_solution: crate::array_return::Solution` (it derives `Default`).

Before using it, check the union-find method name: `grep -n "fn union" crates/kali_types/src/repr_infer.rs` (`self.uf.union(a, b)` is used at `:4896`).

- [ ] **Step 6: Emit the facts and the taints in `emit_table`**

After the "Array elements" loop and before the I2 block (`:5645`), add:

```rust
        // Array-return lane (spec 2026-10-02 §3.1): an admitted function's
        // returned elements must solve `I64`. A float or string element taints
        // it here, after solving, so the check sees computed elements too.
        let solution = std::mem::take(&mut self.array_return_solution);
        let mut taints = solution.tainted.clone();
        for f in &solution.array_returning {
            let node = self
                .array_elem_node
                .get(&(f.clone(), crate::array_return::RETURN_ARRAY_KEY.to_string()))
                .copied();
            let non_int = node.is_some_and(|n| {
                let rep = self.uf.find(n);
                string[rep] || float[rep]
            });
            if non_int {
                taints.insert(f.clone(), kali_common::ARRAY_RETURN_ELEMENT);
            }
        }
        for f in &solution.array_returning {
            if !taints.contains_key(f) {
                table.set_array_return(f, Repr::I64);
            }
        }
        for (caller, binding) in &solution.call_bound {
            table.set_call_bound_array_binding(caller, binding);
        }
        for (f, reason) in &taints {
            table.set_array_return_taint(f, reason);
            table.add_shape_conflict(kali_common::array_return_refused_message(f, reason));
        }
```

`string` and `float` are the solved per-node vectors already in scope at that point (`:5600-5640` uses them). A `string` vector may be empty when `has_strings` is false: guard with `string.get(rep).copied().unwrap_or(false)` and the same for `float`, if indexing would panic.

- [ ] **Step 7: Run the inference tests**

Run: `cargo test -p kali_types array_return`
Expected: every `array_return*` test passes.

- [ ] **Step 8: Write the check-level refusal cases**

Create `crates/kali_cli/tests/cases/runtime/array_return_refusals.toml`:

```toml
# Cases for the array-return project (spec
# docs/superpowers/specs/2026-10-02-array-return-design.md, sections 3.1, 3.4, 4.2).
#
# A returned array kali cannot make real refuses with E5506, under `kali check`
# and `kali run` alike (amendment A1). Each rationale records kali's output and
# node v26.8.2's at the baseline `368b5b5ea`.
#
# `[source]` keys are one file per program, because `[source]` is file-wide
# (see this directory's README).

[constants]
REFUSED = "is unavailable in the current phase"

[source]
"mixed_returns.js" = '''
function f(c) { if (c) { return [1]; } return 0; } function main() { const a = f(true); console.log(a[0]); } main();
'''
"falls_off_end.js" = '''
function f(c) { if (c) { return [1]; } } function main() { const a = f(true); console.log(a[0]); } main();
'''
"float_elements.js" = '''
function f() { return [1.5, 2.5]; } function main() { const a = f(); console.log(a[0]); } main();
'''
"string_literal_elements.js" = '''
function f() { return ["a", "b"]; } function main() { const a = f(); console.log(a[0]); } main();
'''
"boolean_elements.js" = '''
function f() { return [true, false]; } function main() { const a = f(); console.log(a[0]); } main();
'''
"nested_array_elements.js" = '''
function f() { return [[1], [2]]; } function main() { const a = f(); console.log(a[0][0]); } main();
'''
"float_fill_return.js" = '''
function f() { const a = new Array(2).fill(1.5); return a; } function main() { const b = f(); console.log(b[0]); } main();
'''
"string_fill_return.js" = '''
function mk() { const a = new Array(2).fill("x"); return a; } function main() { const c = mk(); console.log(c[0]); } main();
'''
"let_literal_return.js" = '''
function f() { let a = [1, 2, 3]; return a; } function main() { const b = f(); console.log(b[1]); } main();
'''
"arrow_return.js" = '''
const f = () => [1, 2, 3]; function main() { const a = f(); console.log(a[0]); } main();
'''
```

and one `[[case]]` per program **for each of `check` and `run`**. The pattern, shown for the first program:

```toml
[[case]]
name = "mixed_array_and_scalar_returns_refuse_at_check"
rationale = """At `368b5b5ea` kali printed `0` at exit 0 (check passed) where node v26.8.2 prints `1`. A function returning an array on one path and a scalar on another has no single return repr, so it refuses (spec §3.1, reason "it mixes array and non-array returns")."""
args = ["check", "mixed_returns.js"]
exit = "failure"
stderr_contains = ["E5506", "returning an array from `f`", "it mixes array and non-array returns"]

[[case]]
name = "mixed_array_and_scalar_returns_refuse_at_run"
rationale = """The `run` twin of the case above (amendment A1: check and run refuse the same programs)."""
args = ["run", "mixed_returns.js"]
exit = "failure"
stderr_contains = ["E5506", "returning an array from `f`", "it mixes array and non-array returns"]
```

Before writing each rationale, take each program's baseline kali and node output from `tools/array-return-probes/baseline.tsv` (Task 1). For `arrow_return.js` the function name in the message is the synthetic `__kali_fn_N`, so assert only `["E5506", "${REFUSED}", "only a `function` declaration"]`. Use the reason strings from Task 2 exactly.

- [ ] **Step 9: Run the case file**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_return_refusals`
Expected: all pass. If `arrow_return` passes `check` today without refusing, inspect `t.shape_conflicts()` for the program (add a temporary `dbg!` in a unit test, not in production) and fix the arrow's classification before continuing.

- [ ] **Step 10: Run the gate**

Run: `bash scripts/test-gate.sh`
Expected: `GATE OK: 0 failing tests`. A newly failing case means an existing program now refuses. Classify each one:
- A program that was **silently wrong** and now refuses: re-pin it, with a rationale naming this project and its baseline output.
- A program that was **correct** and now refuses: a capability loss. If the cause is `ARRAY_RETURN_FORM` on an arrow or function expression, apply the pre-decided narrowing: in `solve`'s taint pass, skip (do not taint) a non-candidate whose name starts with `__kali_fn_`. Record it for Task 10's followups document, delete the `arrow_return` cases, and re-run. Any other cause: stop and report it.

- [ ] **Step 11: Commit**

```bash
git add crates/kali_types crates/kali_cli/tests/cases/runtime/array_return_refusals.toml
git commit -m "feat(array-return): infer array-returning functions, call-bound bindings and array-fed params"
```

---

### Task 5: `kali check` learns call-bound arrays (amendment A1)

**Files:**
- Modify: `crates/kali_types/src/resolve/mod.rs:967-982` (declarator registration)
- Modify: `crates/kali_types/src/resolve/member.rs:351-366` (`nameless_computed_member_is_admitted_by_a_runtime_lane`)
- Test: `crates/kali_types/src/resolve/member_tests.rs`

**Interfaces:**
- Consumes: `ReprTable::is_call_bound_array_binding`, `ReprTable::array_return` (Task 2), as populated by Task 4.
- Produces: `TypeContext::call_returns_runtime_array(&self, call: &CallExpression) -> bool` (in `member.rs`), used by the computed-member gate.

- [ ] **Step 1: Write the failing tests** (in `member_tests.rs`, using the same check helper its existing tests use; find it with `sed -n 1,40p crates/kali_types/src/resolve/member_tests.rs`)

```rust
#[test]
fn computed_index_on_call_bound_array_is_admitted() {
    let diags = check_source(
        "function f() { return [1, 2, 3]; } function main() { const a = f(); let i = 2; console.log(a[i]); } main();",
    );
    assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
}

#[test]
fn computed_index_on_direct_array_returning_call_is_admitted() {
    let diags = check_source(
        "function f() { return [1, 2, 3]; } function main() { let i = 1; console.log(f()[i]); } main();",
    );
    assert!(diags.iter().all(|d| !d.is_error()), "{diags:?}");
}

#[test]
fn computed_index_on_scalar_call_still_refuses() {
    let diags = check_source(
        "function f() { return 3; } function main() { let i = 1; console.log(f()[i]); } main();",
    );
    assert!(diags.iter().any(|d| d.is_error()));
}
```

If the test module's helper has a different name, use that name. Do not add a new helper if one exists.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p kali_types computed_index_on_`
Expected: the first two fail with the computed-member `E5506`; the third passes.

- [ ] **Step 3: Implement**

In `resolve/mod.rs`, after the `if self.declarator_registers_runtime_array(init) { … }` block:

```rust
                // Array-return lane (spec 2026-10-02 A1): `const b = f()` where
                // `f` returns a runtime array. Keyed on the NARROW call-bound
                // fact and on the init still being that call, mirroring
                // codegen's declarator arm; never on `is_array_binding`, which
                // over-proves (see `nameless_computed_member_is_admitted_by_a_runtime_lane`).
                if declaration.kind != "var"
                    && matches!(init, Expression::CallExpression(call)
                        if self.call_returns_runtime_array(call))
                    && self
                        .binding_repr_function_key(&declarator.id)
                        .is_some_and(|func| {
                            self.repr_table
                                .is_call_bound_array_binding(&func, &declarator.id)
                        })
                {
                    if let Some(scope) = self.scopes.get_mut(&target_scope) {
                        scope
                            .runtime_array_bindings
                            .insert(declarator.id.clone(), true);
                    } else if self.global_scope.contains(&declarator.id) {
                        self.global_scope
                            .runtime_array_bindings
                            .insert(declarator.id.clone(), true);
                    }
                }
```

In `member.rs`, the `match &member.object` in `nameless_computed_member_is_admitted_by_a_runtime_lane` gains an arm:

```rust
            // Mirrors codegen's direct-call read lane (`array_return_call_elem`,
            // spec 2026-10-02 §3.3): `f()[i]` where `f` returns a runtime array.
            Expression::CallExpression(call) => self.call_returns_runtime_array(call),
```

and add the method:

```rust
    /// `call` is a bare-identifier call to a function the inference admitted
    /// as array-returning, and the name resolves to that function (not a
    /// local binding shadowing it).
    pub(crate) fn call_returns_runtime_array(&self, call: &kali_ast::CallExpression) -> bool {
        let Expression::Identifier(callee) = &call.callee else {
            return false;
        };
        self.repr_table.array_return(callee).is_some()
            && !self.is_locally_bound_non_function(callee)
    }
```

For `is_locally_bound_non_function`, use whatever existing resolver query says "this name resolves to a declared function". Find it with `grep -n "fn .*function.*binding\|fn is_function" crates/kali_types/src/resolve/*.rs`. If there is none, use `self.binding_repr_function_key(callee).is_some()`. That is safe: the function's own declaration makes the name a binding in its declaring scope, so the key resolves. A local `const f = 3` shadow yields a key whose `array_return` is `None`. In that case, check `self.repr_table.array_return(callee)` only through the key's function and drop the helper. Write down which of the two you used in the method's doc comment.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_types computed_index_on_`
Expected: 3 passed.

- [ ] **Step 5: Run the gate and commit**

Run: `bash scripts/test-gate.sh` and expect `GATE OK`.

```bash
git add crates/kali_types/src/resolve
git commit -m "feat(array-return): kali check registers call-bound arrays and admits direct array-returning calls"
```

---

### Task 6: Codegen, the callee: a returned literal is allocated

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (`collect_function_locals`, next to the growable reservation near `:3583`; add `array_return_scratch_local_name()` next to `growable_scratch_local_name()` at `:3663`)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs:177` (`emit_return`) and `:1640-1683` (the declarator's object-element literal lane)
- Create: `crates/kali_cli/tests/cases/runtime/array_return.toml`

**Interfaces:**
- Consumes: `ReprTable::array_return`, `ReprTable::array_return_taint` (Task 2).
- Produces:
  - `crate::lower::array_return_scratch_local_name() -> String` (returns `"__array_return_scratch"`)
  - `FunctionEmitter::emit_static_array_materialize(&mut self, function: &mut Function, aggregate: &LirNode, slot: u32, elem_shape: Option<kali_common::ShapeId>)`. It allocates `aggregate.children.len()` slots, stores the handle in local `slot`, then stores each element at `8 + i*8`. When `elem_shape` is `Some`, an object-literal child goes through `emit_object_allocation`.

- [ ] **Step 1: Write the failing cases** (`array_return.toml`; the callee-side rows that need no caller change)

```toml
# Cases for the array-return project (spec
# docs/superpowers/specs/2026-10-02-array-return-design.md, sections 3.2-3.3, 4.2).
#
# A function that returns an I64-element array on every path hands its caller a
# real runtime array. Every `_computes` case prints node v26.8.2's exact output;
# each rationale records kali's output at the baseline `368b5b5ea`.
#
# `[source]` keys are one file per program (see this directory's README).

[source]
"passed_on.js" = '''
function f() { return [1, 2, 3]; } function g(x) { return x[1]; } console.log(g(f()));
'''
"call_arg_direct.js" = '''
function f() { return [3, 4]; } function g(x) { return x.length; } console.log(g(f()));
'''
"const_literal_binding_return.js" = '''
function f() { const a = [1, 2, 3]; return a; } function g(x) { return x[1]; } console.log(g(f()));
'''
"allocation_returned_from_inside_a_loop.js" = '''
function f() { for (let i = 0; i < 3; i++) { const a = new Array(2).fill(i); if (i === 2) { return a; } } return new Array(1).fill(0); }
function g(x) { return x[0]; }
console.log(g(f()));
'''

[[case]]
name = "a_returned_literal_passed_on_computes"
rationale = """At `368b5b5ea` kali printed `0` at exit 0 where node v26.8.2 prints `2`: the literal was never allocated (emit_aggregate_literal pushed 0). Spec §3.2."""
args = ["run", "passed_on.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "a_returned_literal_length_through_a_param_computes"
rationale = """Measure the baseline with `tools/array-return-probes/baseline.tsv` row `call_arg_direct` and record it here. node v26.8.2 prints `2`. Spec §3.2."""
args = ["run", "call_arg_direct.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "const_literal_binding_return"
rationale = """Amendment A2: a `const` literal binding is fold-lane, so the return materializes it. Baseline: take the output from `kali run` at `368b5b5ea` (measure before implementing). node v26.8.2 prints `2`."""
args = ["run", "const_literal_binding_return.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "allocation_returned_from_inside_a_loop"
rationale = """Pins spec §2.1's arena claim: a returned allocation made inside a loop is not reclaimed by that loop's arena (arena_gate marks returned may-heap sites). node v26.8.2 prints `2`."""
args = ["run", "allocation_returned_from_inside_a_loop.js"]
exit = "success"
stdout = "2\n"
```

Replace each "measure" instruction in a rationale with the measured value before committing. The probe TSV has `passed_on` and `call_arg_direct`; run `target/debug/kali run` on the other two programs once, at this task's starting commit, before Step 3.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_return::`
Expected: the literal cases fail (`0` printed). The loop-allocation case may already pass. If so, it stays as a pin.

- [ ] **Step 3: Reserve the scratch local**

In `lower.rs`:

```rust
/// Name of the dedicated i64 scratch local `emit_return` materializes a
/// returned array literal into (array-return project, spec 2026-10-02 §3.2).
/// Dedicated because element emission may use every generic and allocation
/// scratch slot.
pub(crate) fn array_return_scratch_local_name() -> String {
    "__array_return_scratch".to_string()
}
```

and in `collect_function_locals`, just before `locals` is returned:

```rust
    // Array-return lane: reserve the materialization scratch only in a function
    // the inference admitted as array-returning, so every other function stays
    // byte-identical.
    if repr_table.array_return(function_name).is_some() {
        locals.push(array_return_scratch_local_name());
    }
```

- [ ] **Step 4: Extract the materialize helper**

Move the body of the object-element literal lane (`control_flow.rs`, the block that starts `let allocated = self.emit_array_allocation_static(function, aggregate.children.len());` and ends after the per-child `I64Store`) into a new method on `FunctionEmitter` in `emit/control_flow.rs`:

```rust
    /// Allocate a `[len][elem…]` runtime array for the array literal
    /// `aggregate`, leave its handle in local `slot`, and store each element at
    /// `8 + i*8`. With `elem_shape`, an object-literal child is materialized
    /// through `emit_object_allocation`; every other child is emitted as an i64.
    /// Shared by the declarator's object-element lane and `emit_return`'s array
    /// arm (spec 2026-10-02 §3.2), so the two cannot drift.
    pub(crate) fn emit_static_array_materialize(
        &mut self,
        function: &mut Function,
        aggregate: &LirNode,
        slot: u32,
        elem_shape: Option<kali_common::ShapeId>,
    ) {
        let allocated = self.emit_array_allocation_static(function, aggregate.children.len());
        if !allocated.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::LocalSet(slot));
        for (i, child) in aggregate.children.iter().copied().enumerate() {
            function.instruction(&Instruction::LocalGet(slot));
            function.instruction(&Instruction::I32WrapI64);
            let child_node = self.node(child).clone();
            let produced = match elem_shape {
                Some(shape) if self.is_object_literal(&child_node) => {
                    self.emit_object_allocation(function, &child_node, shape)
                }
                _ => self.emit_node(function, child, true),
            };
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::I64Store(MemArg {
                offset: (8 + i * 8) as u64,
                align: 3,
                memory_index: 0,
            }));
        }
    }
```

The declarator lane becomes:

```rust
                                if let (Some(aggregate), Some(index)) =
                                    (aggregate, self.locals.get(&name).copied())
                                {
                                    self.emit_static_array_materialize(
                                        function,
                                        &aggregate,
                                        index,
                                        Some(elem_shape),
                                    );
                                    self.array_bindings.insert(name.clone());
                                    continue;
                                }
```

Note one ordering change: the original inserted into `array_bindings` *before* the element stores; the helper version inserts after. Confirm that no element emission reads `array_bindings` for `name`. Object-literal children cannot reference the binding being declared, because it is in its TDZ. If unsure, keep the original order by inserting before calling the helper.

The type of `elem_shape` in the declarator is whatever `Repr::Object(_)` carries; use that type in the helper's signature (`grep -n "Object(ShapeId)" crates/kali_common/src/repr.rs` confirms `ShapeId`, and check its path with `grep -n "pub use\|pub struct ShapeId" crates/kali_common/src/*.rs`).

- [ ] **Step 5: The `emit_return` arm**

At the top of `emit_return`'s `if let Some(arg)` block, before the object arm:

```rust
            // Array-return lane (spec 2026-10-02 §3.2): an admitted function
            // materializes a returned literal (or `const` literal binding,
            // A2) into a runtime array. Any other admitted argument already
            // produces its handle through `emit_node` below.
            if self.repr_table.array_return(&self.function_name).is_some() {
                if let Some(aggregate_id) = self.resolve_literal_aggregate(arg) {
                    let aggregate = self.node(aggregate_id).clone();
                    if self.is_array_literal(&aggregate) {
                        let slot = self.locals[&crate::lower::array_return_scratch_local_name()];
                        self.emit_static_array_materialize(function, &aggregate, slot, None);
                        function.instruction(&Instruction::LocalGet(slot));
                        self.emit_arena_unwind_for_return(function);
                        self.emit_env_restore(function);
                        function.instruction(&Instruction::Return);
                        return EmittedValue {
                            produced: false,
                            shape: ValueShape::Unknown,
                        };
                    }
                }
            } else if let Some(reason) = self.repr_table.array_return_taint(&self.function_name) {
                // Unreachable for an admitted program: every taint is also a
                // shape conflict, which stops compilation before codegen.
                // Kept so a future path that skips that check still refuses.
                let message =
                    kali_common::array_return_refused_message(&self.function_name, reason);
                return self.deny_e5506(function, &message);
            }
```

The spec says "a non-array return in a tainted function emits as today". The arm above refuses every return of a tainted function instead. That is unreachable for any program that reaches codegen (see the comment), so it is the simpler of two equivalent behaviours. Record the choice in the commit message.

- [ ] **Step 6: Run the cases**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_return::`
Expected: 4 passed.

- [ ] **Step 7: Run the gate and commit**

Run: `bash scripts/test-gate.sh` and expect `GATE OK`. This task changes the bytes of functions that return arrays. Any determinism or byte-pin test that moves must involve such a function; if one doesn't, stop.

```bash
git add crates/kali_codegen crates/kali_cli/tests/cases/runtime/array_return.toml
git commit -m "feat(array-return): a returned array literal is allocated; the declarator and return share one materializer"
```

---

### Task 7: Codegen, the caller: bound and direct reads

**Files:**
- Modify: `crates/kali_codegen/src/emitter.rs` (add the recognizer next to `array_elem_repr` at `:812`)
- Modify: `crates/kali_codegen/src/emit/call.rs:6318`, `:6414` (read helpers take an element repr)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (the declarator arm before `resolve_array_alloc_call` at `:1686`; the `.length` arm near `:2719`; the read arms at `:2847` and `:3077`)
- Modify: `crates/kali_codegen/src/emit/computed_member.rs:104`
- Modify: `crates/kali_cli/tests/cases/runtime/array_return.toml`

**Interfaces:**
- Consumes: Tasks 2, 4 and 6.
- Produces:
  - `FunctionEmitter::array_return_call_elem(&self, id: LirNodeId) -> Option<kali_common::Repr>`: `Some(elem)` when `id` (after `unwrap_transparent_value_node`) is a `Call` whose callee is a bare identifier that is not a local and is `array_return`-admitted.
  - `emit_dynamic_array_read_elem(&mut self, function, base_id, index_text: &str, elem: Repr)` and `emit_dynamic_array_read_node_elem(&mut self, function, base_id, index_id, elem: Repr)`. The existing two helpers delegate to these with `self.array_elem_repr(base_name)`.

- [ ] **Step 1: Write the failing cases** (append to `array_return.toml`: sources and cases)

Sources to add (one key each, named exactly as the file stems):
- `s01_direct_index.js`
- `s02_bound_module.js`
- `s03_bound_in_main.js`
- `s04_fill_binding_return.js`
- `s05_param_pass_through.js`
- `s06_loop_filled.js`
- `s07_store_then_return.js`
- `s08_direct_fill.js`
- `dyn_index.js`
- `bound_length.js`
- `direct_length.js`
- `bound_passed_on.js`
- `mutate_call_bound.js`
- `recursion.js`
- `mutual_recursion.js`
- `empty_literal.js`
- `computed_elements.js`
- `nested_decl.js`
- `if_else_both_return.js`
- `r14_register_in_function.js`

Each body is copied from `tools/array-return-probes/probes/<stem>.js`. The last one is new:

```js
function main() {
  function f() { return [1, 2, 3]; }
  console.log("r=" + f()[0]);
}
main();
```

Then one `_computes` case per source: `args = ["run", "<stem>.js"]`, `exit = "success"`, `stdout` = node's output plus `\n`, and a rationale quoting the baseline row's kali output. Node's output per stem:

| stem | node stdout |
|---|---|
| `s01_direct_index` | `r=1` |
| `s02_bound_module` | `1,3` |
| `s03_bound_in_main` | `1,3` |
| `s04_fill_binding_return` | `4` |
| `s05_param_pass_through` | `1` |
| `s06_loop_filled` | `9` |
| `s07_store_then_return` | `7` |
| `s08_direct_fill` | `2` |
| `dyn_index` | `3` |
| `bound_length` | `3` |
| `direct_length` | `2` |
| `bound_passed_on` | `3` |
| `mutate_call_bound` | `9` |
| `recursion` | `8` |
| `mutual_recursion` | `5` |
| `empty_literal` | `0` |
| `computed_elements` | `5,10,6` |
| `nested_decl` | `1,3` |
| `if_else_both_return` | `2` |
| `r14_register_in_function` | `r=1` |

Also add the check-level twin for `dyn_index`: `args = ["check", "dyn_index.js"]`, `exit = "success"`. This is the A1 agreement pin.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_return::`
Expected: the new caller-side cases fail with `0`. `empty_literal` may refuse on `.length`.

- [ ] **Step 3: The recognizer**

In `emitter.rs`:

```rust
    /// Element repr of a call to an array-returning function (array-return
    /// project, spec 2026-10-02 §3.3), or `None`. A callee that is a local
    /// binding of this function is not the declared function, so it declines.
    pub(crate) fn array_return_call_elem(&self, id: LirNodeId) -> Option<kali_common::Repr> {
        let target = self.unwrap_transparent_value_node(id);
        let node = self.node(target);
        if node.kind != LirNodeKind::Call {
            return None;
        }
        let callee = self.bare_identifier_name(*node.children.first()?)?;
        if self.locals.contains_key(&callee) {
            return None;
        }
        self.repr_table.array_return(&callee)
    }
```

If `unwrap_transparent_value_node` or `bare_identifier_name` live in another `impl` file, they are still methods on the same type; nothing to import.

- [ ] **Step 4: The declarator arm** (`control_flow.rs`, immediately before `if let Some(size_arg) = self.resolve_array_alloc_call(init) {`)

```rust
                        // Array-return lane (spec 2026-10-02 §3.3): `const b = f()`
                        // where `f` returns a runtime array. Registered only while
                        // the init is still that call, so an optimizer-rewritten
                        // init (a literal) keeps its own lane.
                        if let Some(name) = declarator.text.clone() {
                            if self
                                .repr_table
                                .is_call_bound_array_binding(&self.function_name, &name)
                                && self.array_return_call_elem(init).is_some()
                            {
                                if let Some(index) = self.locals.get(&name).copied() {
                                    let produced = self.emit_node(function, init, true);
                                    if !produced.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    function.instruction(&Instruction::LocalSet(index));
                                    self.array_bindings.insert(name);
                                    continue;
                                }
                            }
                        }
```

The spec (§3.3) says to seed `array_bindings` "at function entry" from the repr table. This plan registers at the declarator instead, gated on the init still being the call. Entry seeding would mark `b` as an array even when an optimizer pass had replaced `f()` with its literal body, and the reads would then go through a handle the declarator never stored. Record this in the commit message.

Add the unit test `call_bound_registration_requires_a_call_init`. Put it in the codegen test file that covers the declarator lanes: find it with `ls crates/kali_codegen/src/emit/control_flow_tests* crates/kali_codegen/src/emit/*tests*`, and use that file's existing compile-and-inspect helper. The test builds `ReprTable` facts by hand: `set_call_bound_array_binding("main","b")`, with `array_return("f")` **absent**. It then asserts that a module containing `const b = f()` compiles with `b` absent from `array_bindings`. If no existing helper exposes `array_bindings`, assert instead that a `b[i]` read in that program emits the computed-member `E5506`.

- [ ] **Step 5: The read helpers take an element repr**

In `call.rs`, rename the bodies of `emit_dynamic_array_read` and `emit_dynamic_array_read_node` to `emit_dynamic_array_read_elem(..., elem: kali_common::Repr)` and `emit_dynamic_array_read_node_elem(..., elem: kali_common::Repr)`. In each, change `match self.array_elem_repr(base_name) {` to `match elem {`. Keep the old names as thin wrappers:

```rust
    pub(crate) fn emit_dynamic_array_read(
        &mut self,
        function: &mut Function,
        base_id: LirNodeId,
        index_text: &str,
        base_name: &str,
    ) -> EmittedValue {
        let elem = self.array_elem_repr(base_name);
        self.emit_dynamic_array_read_elem(function, base_id, index_text, elem)
    }
```

(and the same for `_node`). Every existing caller is unchanged.

- [ ] **Step 6: The three direct-call read arms**

`control_flow.rs` one-child member arm (after `if let Some(base_name) = self.dynamic_array_read_base(node) { … }` at `:2847`):

```rust
                // Array-return lane (spec 2026-10-02 §3.3): `f()[k]`. The call
                // is the base, emitted exactly once by the address helper.
                if let Some(index_text) = node.text.as_deref().filter(|t| {
                    !t.is_empty() && *t != "length" && t.parse::<usize>().is_ok()
                }) {
                    if let Some(elem) = self.array_return_call_elem(node.children[0]) {
                        let index_text = index_text.to_string();
                        return self.emit_dynamic_array_read_elem(
                            function,
                            node.children[0],
                            &index_text,
                            elem,
                        );
                    }
                }
```

`control_flow.rs` two-child arm (after the `dynamic_array_read_base` block at `:3077`), and `computed_member.rs` (after its `dynamic_array_read_base` block at `:104`):

```rust
        if let Some(elem) = self.array_return_call_elem(node.children[0]) {
            return self.emit_dynamic_array_read_node_elem(
                function,
                node.children[0],
                node.children[1],
                elem,
            );
        }
```

`.length` (`control_flow.rs`, next to the `is_usp_getall_call(base_id)` arm near `:2706`):

```rust
                    // Array-return lane: `f().length` reads the returned
                    // array's length header.
                    if self.array_return_call_elem(base_id).is_some() {
                        self.emit_array_base_address(function, base_id);
                        function.instruction(&Instruction::I64Load(MemArg {
                            offset: 0,
                            align: 3,
                            memory_index: 0,
                        }));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Scalar,
                        };
                    }
```

- [ ] **Step 7: Run the cases**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_return`
Expected: every case in both files passes. If `s01_direct_index` still prints `r=0`, the literal-index read is not reaching the one-child arm: the `"r=" + …` concat routes through the string oracles (`operators.rs:1084`, `:1361`, `call.rs:4072`, each of which calls `dynamic_array_read_base`). Add the same `array_return_call_elem` arm after each of those three calls, mirroring that site's binding arm with the element repr. Then re-run.

- [ ] **Step 8: Re-run the probes**

Run: `tools/array-return-probes/run.sh /tmp/array-return-task7.tsv && diff <(cut -f1,2 tools/array-return-probes/baseline.tsv) <(cut -f1,2 /tmp/array-return-task7.tsv)`
Expected: `s01`-`s08`, `dyn_index`, `bound_length`, `direct_length`, `passed_on`, `bound_passed_on`, `mutate_call_bound`, `recursion`, `mutual_recursion`, `empty_literal`, `computed_elements`, `const_literal_return`, `nested_decl` and `if_else_both_return` read `CORRECT`. The taint rows read `REFUSES`. The controls are unchanged. `callback_escape`: record whatever it reads, and add it to Task 10's followups if it is not `CORRECT` or `REFUSES`.

- [ ] **Step 9: Run the gate and commit**

Run: `bash scripts/test-gate.sh` and expect `GATE OK`.

```bash
git add crates/kali_codegen crates/kali_cli/tests/cases/runtime/array_return.toml
git commit -m "feat(array-return): a call-bound array and a direct array-returning call read real memory"
```

---

### Task 8: A whole runtime array never prints as a handle

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs:22` (`emit_console_argument`) and `:62` (`emit_console_argument_as_string`)
- Modify: `crates/kali_cli/tests/cases/runtime/array_return_refusals.toml`

**Interfaces:**
- Consumes: `array_return_call_elem` (Task 7), `array_bindings`, `bare_identifier_name`.
- Produces: `FunctionEmitter::is_runtime_array_value(&self, id: LirNodeId) -> bool`.

- [ ] **Step 1: Write the failing cases** (add to `array_return_refusals.toml`)

Sources: `s09_print_call_bound.js`, `s10_print_local.js`, copied from the probes, plus:

```js
// print_second_argument.js
function main() { const a = new Array(3).fill(4); console.log("a", a); } main();
```

```toml
[[case]]
name = "printing_a_call_bound_array_refuses"
rationale = """At `368b5b5ea` kali printed `0` at exit 0 where node v26.8.2 prints `[ 1, 2, 3 ]`. Once the binding is a real array it would print the raw handle instead, so printing a whole runtime array refuses (spec §2.4, §3.3 backstop 3)."""
args = ["run", "s09_print_call_bound.js"]
exit = "failure"
stderr_contains = ["E5506", "printing a whole runtime array"]

[[case]]
name = "printing_a_local_runtime_array_refuses"
rationale = """At `368b5b5ea` kali printed `4104` (the array's handle) at exit 0 where node v26.8.2 prints `[ 4, 4, 4 ]` (spec §2.2 S10)."""
args = ["run", "s10_print_local.js"]
exit = "failure"
stderr_contains = ["E5506", "printing a whole runtime array"]

[[case]]
name = "printing_a_runtime_array_as_a_later_argument_refuses"
rationale = """The multi-argument lane: measure the baseline before implementing and record it here. node v26.8.2 prints `a [ 4, 4, 4 ]`."""
args = ["run", "print_second_argument.js"]
exit = "failure"
stderr_contains = ["E5506", "printing a whole runtime array"]
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p kali_cli --test cases -- runtime/array_return_refusals::printing`
Expected: 3 failed (exit 0).

- [ ] **Step 3: Implement**

```rust
    /// A value that is a whole runtime `[len][elem…]` array: a bare
    /// identifier in `array_bindings`, or a call to an array-returning
    /// function. Its i64 is a handle, never a printable number.
    pub(crate) fn is_runtime_array_value(&self, id: LirNodeId) -> bool {
        self.bare_identifier_name(id)
            .is_some_and(|name| self.array_bindings.contains(&name))
            || self.array_return_call_elem(id).is_some()
    }
```

In both `emit_console_argument` and `emit_console_argument_as_string`, first thing:

```rust
        if self.is_runtime_array_value(id) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "printing a whole runtime array is unavailable in the current phase: kali would print its handle; print its elements instead"
                    .to_string(),
            ));
        }
```

- [ ] **Step 4: Run the cases**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test cases -- runtime/array_return_refusals`
Expected: all pass.

- [ ] **Step 5: Run the gate**

Run: `bash scripts/test-gate.sh`
Any case that printed a whole runtime array and was pinned to the handle's digits is a silent wrong value pinned. Re-pin it to the refusal, with a rationale naming this project. Any case that printed one *correctly* would mean a lane this guard does not know about. Stop and report.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/cases
git commit -m "fix(array-return): printing a whole runtime array refuses instead of printing its handle"
```

---

### Task 9: The two remaining backstops, at measured width

The numeric-index fallback (`operators.rs:563-571`) and the aggregate placeholder (`literal.rs:13-45`) each push a silent `0`. This task makes them refuse. Each is first measured on its own, because a wide refusal can turn correct programs into refused ones.

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs:563-571`
- Modify: `crates/kali_codegen/src/emit/literal.rs:13-45`
- Modify: case files the gate reports (re-pins only)
- Modify: `docs/superpowers/specs/2026-10-02-array-return-design.md` (§4.1, the measured result)

**Interfaces:**
- Consumes: `deny_e5506` (`crates/kali_codegen/src/intrinsics/host.rs:1666`).
- Produces: nothing new for later tasks.

- [ ] **Step 1: Write the failing cases** (add to `array_return_refusals.toml`)

```js
// index_on_scalar_call.js
function f() { return 3; } console.log("v=" + f()[0]);
```

```toml
[[case]]
name = "a_literal_index_on_a_non_array_receiver_refuses"
rationale = """The numeric-index fallback used to drop the receiver and print `0`. Measure the baseline (expected `v=0` at exit 0; node v26.8.2 prints `v=undefined`). Spec §3.3 backstop 1."""
args = ["run", "index_on_scalar_call.js"]
exit = "failure"
stderr_contains = ["E5506", "no lane proves this receiver is an array"]
```

- [ ] **Step 2: Backstop 1 at full width**

Replace the fallback in `operators.rs`:

```rust
                // The floor (spec 2026-10-02 §3.3 backstop 1). Every lane above
                // declined; this used to drop the receiver and push `0`, the
                // silent value R-14 came out of. Emit the receiver first so a
                // refusal specific to it wins, as the `.length` floor does.
                let errors_before = self.diagnostics.iter().filter(|d| d.is_error()).count();
                let produced = self.emit_node(function, arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                if self.diagnostics.iter().filter(|d| d.is_error()).count() > errors_before {
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                self.deny_e5506(
                    function,
                    "an indexed read is unavailable in the current phase for this receiver: no lane proves this receiver is an array, so kali refuses rather than emit a placeholder 0",
                )
```

- [ ] **Step 3: Measure backstop 1**

```bash
cargo build -p kali_cli
bash scripts/test-gate.sh 2>&1 | tee /tmp/array-return-backstop1.log
tools/array-return-probes/run.sh /tmp/array-return-backstop1.tsv
```

For every failing test, classify it in a table at spec §4.1 under a new heading `#### Backstop 1, measured`:
- **(a) silent before, now refuses:** re-pin.
- **(b) correct before, now refuses:** capability loss.
- **(c) other.**

If there is any (b): narrow the backstop so it refuses only when the receiver (`arg`) is a `Call` node (`self.node(self.unwrap_transparent_value_node(arg)).kind == LirNodeKind::Call`) or a bare identifier, keep the old `I64Const(0)` path otherwise, and record each (b) row in the followups list for Task 10. Re-run until there is no (b).

- [ ] **Step 4: Backstop 2 at full width, then measure**

In `emit_aggregate_literal`'s array branch (the `else` branch), before the final `I64Const(0)`:

```rust
        } else {
            // Spec 2026-10-02 §3.3 backstop 2: an array literal reaching the
            // placeholder in a consumed position has no runtime value; refuse
            // rather than hand `0` to the consumer.
            if _want_value {
                for child in &node.children {
                    let produced = self.emit_node(function, *child, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                }
                return self.deny_e5506(
                    function,
                    "an array literal in this position is unavailable in the current phase: no lane gives it a runtime value, so kali refuses rather than emit a placeholder 0",
                );
            }
            for child in &node.children {
```

(`_want_value` is the existing parameter. Rename it to `want_value` now that it is read.)

Measure exactly as in Step 3 (`/tmp/array-return-backstop2.log`), classify into a `#### Backstop 2, measured` table, and apply the same narrowing rule if there is any (b). Narrowing here means: refuse only when the literal's parent node is a `Return`, a `Call` argument, or a declarator initializer. If the parent is not available at this site, drop backstop 2 entirely and record that in the spec and the followups list.

- [ ] **Step 5: Re-pin, re-run, commit**

Re-pin every (a) case with a rationale naming this project and its old output. Then:

Run: `bash scripts/test-gate.sh` and expect `GATE OK`.

```bash
git add crates/kali_codegen crates/kali_cli/tests/cases docs/superpowers/specs/2026-10-02-array-return-design.md
git commit -m "fix(array-return): the index fallback and the array placeholder refuse instead of emitting 0"
```

---

### Task 10: Close the ledger

**Files:**
- Modify: `crates/kali_cli/tests/cases/oracle/tier2.toml` (cases `r14_returned_array_reads_zeros_module_scope` at `:1536` and `r14_returned_array_reads_zeros_in_function` at `:1550`, and the header tallies at `:92`, `:400-424`)
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (R-14 at `:2467`, the §0.2 row at `:275`, R-06-R1 at `:1459`, the work-order row at `:6074`, and a new entry for the console handle print)
- Modify: `tools/blast-radius/counts.json`, `tools/blast-radius/clusters.json`, `docs/superpowers/followups/blast-radius-ranking.md` (generated regions only)
- Modify: `docs/superpowers/followups/inline-allocation-value-position-discovered-defects.md` §1
- Create: `docs/superpowers/followups/array-return-discovered-defects.md`

**Interfaces:**
- Consumes: everything above.
- Produces: the closed branch.

- [ ] **Step 1: Re-pin the R-14 oracle cases**

Run `cargo test -p kali_cli --test cases -- oracle/tier2::r14`. Both cases now fail, because their verdict was `SILENT` and is now `CORRECT`. Change each case's expected verdict to the one it now measures. Rewrite the rationale's "Expected verdict" paragraph to cite this project, `docs/superpowers/specs/2026-10-02-array-return-design.md`. Update the header tallies at `:92` and `:400-424` (`silent` → `correct` for R-14). Follow the header's own strike-through convention for superseded counts.

- [ ] **Step 2: Retire R-14 in the register, and file the console entry**

At R-14 (`:2467`), add:

```markdown
- **STATUS 2026-10-02 (array-return): CLOSED.** Mechanism traced (spec
  `docs/superpowers/specs/2026-10-02-array-return-design.md` §2.1): no arena or
  escape reclamation is involved. The callee never allocated the literal
  (`emit_aggregate_literal` pushed `0`), and the caller's `a[0]` fell into the
  numeric-index fallback (`emit/operators.rs`), which pushed `0` too. The
  arena hypothesis above is refuted. A `function` declaration returning
  integer arrays now hands back a real array in both scopes, bound or direct;
  every other array-shaped return refuses with `E5506`. Oracle cases
  `r14_returned_array_reads_zeros_{module_scope,in_function}` measure CORRECT.
```

Strike the "Mechanism hypothesis" bullet with `~~…~~`, keeping it readable. Correct R-06-R1's "Real fix = R-14 escape stage" (`:1459`) and the `:6074` work-order row in place, both pointing to the spec. Update §0.2's row (`:275`).

File the next free register ID (find it with `grep -n "^### R-6[0-9]" docs/superpowers/followups/kali-silent-miscompile-register.md | tail -1`) for "printing a whole runtime array prints its handle", with S10 as the repro. Retire it in the same edit (`CLOSED, 2026-10-02, array-return`), following R-64..R-67's format.

- [ ] **Step 3: Regenerate the blast-radius artifacts**

Follow `tools/blast-radius/README.md`'s regeneration command. The ranking spec names `count.mjs` followed by `cargo run -p kali_blast_radius --example rank`. Run `git diff --stat tools/blast-radius docs/superpowers/followups/blast-radius-ranking.md` and confirm that only generated regions and the JSON files moved. If `crates/kali_blast_radius/src/manifest_tests.rs` or `register_tests.rs` pin a register-entry status that moved, update the pin and say why in the commit. Record any band churn in the ranking's authored commentary (§6), one paragraph, dated.

- [ ] **Step 4: The followups document**

Create `docs/superpowers/followups/array-return-discovered-defects.md` on the convention of `inline-allocation-value-position-discovered-defects.md`: provenance header, then sections ranked by consequence. It must contain, each with its measured program and node vs kali output from `tools/array-return-probes` at the branch head:
1. R-48's two rows (cross-reference, not refiled).
2. The R-21 out-of-bounds row (`r21_oob_control`).
3. Ternary array returns (`return c ? [1] : [2]`, classified `NonArray`): measure and record.
4. Arrow and function-expression returns: refused, or, if Task 4's narrowing was applied, left on their old lane, with the measured output.
5. Non-`I64` element arrays (float, string, boolean, nested): refused.
6. Assignment and destructuring from an array-returning call (`b = f()`, `const [x] = f()`): measure and record.
7. Each backstop's final width, and every (b) row Task 9 recorded.
8. `callback_escape`'s verdict from Task 7 Step 8, if it was neither `CORRECT` nor `REFUSES`.
9. Booleans stored into runtime arrays by a computed element (`[t]` where `t = x > 1`) read back as `1`/`0`: measure and record.

- [ ] **Step 5: Point the picked item at its resolution**

In `inline-allocation-value-position-discovered-defects.md` §1, append:

```markdown
**Resolved 2026-10-02** by the array-return project
(`docs/superpowers/specs/2026-10-02-array-return-design.md`). Both rows above
now print node's output. The mechanism was not R-14's arena hypothesis; see
that spec's §2.1.
```

- [ ] **Step 6: Final gates**

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
bash scripts/test-gate.sh
tools/array-return-probes/run.sh /tmp/array-return-final.tsv
cut -f1,2 /tmp/array-return-final.tsv
```

Expected:
- fmt and clippy are clean, and `GATE OK`.
- Probes: every `s0x` row and every Task 7 Step 8 row is `CORRECT`; `s09`, `s10`, `s11` and every taint row is `REFUSES`; each control matches its baseline verdict.
- Record the final passed/ignored counts in the spec's §4.3 next to the baseline.

- [ ] **Step 7: Commit**

```bash
git add -A docs tools crates/kali_cli/tests/cases/oracle crates/kali_blast_radius
git commit -m "docs(array-return): retire R-14, file what was measured and not fixed, regenerate the ranking"
```
