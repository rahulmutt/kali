# Inline Allocation in Value Position Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** An `Array` / `Uint8Array` allocation in value position evaluates to a real array handle, an array kali cannot pass as a handle refuses with `E5506`, and `.fill(v)` evaluates `v` once.

**Architecture:** `kali_types` refuses the one-element array literal that lowers to the same LIR node as an allocation, which is what makes the codegen route sound. `kali_codegen` then routes allocation and `.fill` shapes in `emit_value`'s text-less branch and in `emit_call` to the allocator that already exists, gives allocation and fill their own scratch slots so nesting cannot clobber them, and widens the call-argument guard to refuse every fold-lane array it cannot pass.

**Tech Stack:** Rust (`kali_types`, `kali_codegen`, `kali_cli`, `kali_blast_radius`), the `.toml` case runner, node v26.8.2 as the oracle, `acorn` matchers under `tools/blast-radius`, Python generators under `tools/task-18-browser-pilot`.

**Spec:** `docs/superpowers/specs/2026-09-11-inline-allocation-value-position-design.md` (committed as `46370a3d35`)

## Global Constraints

- **Branch:** `inline-allocation-value-position`. Baseline `733cd26125`, whose suite is **12032 passed, 0 failed, 27 ignored**.
- **Oracle:** `node v26.8.2`. Every expectation records node's output in its rationale.
- **One cargo target directory.** No extra worktrees and no extra target directories: they have exhausted this pod's disk and killed a run.
- **`bash scripts/test-gate.sh` after every task.** From Task 2 on, the gate is red on purpose; the task's gate condition is "the failing set equals the expected-red set named in that task", never "GATE OK".
- **`bash scripts/test-gate.sh --gates-only` after any task that edits a file under `crates/kali_cli/tests/cases/browser/`.**
- **`cargo clippy --workspace -- -D warnings` and `cargo fmt --all -- --check` before pushing.** The gate runs neither.
- **A full workspace suite run takes over an hour and a half on this pod.** Never background one and report before it exits.
- **Fixture files are read, never edited** (`crates/kali_cli/tests/fixtures/`).
- **Rust unit tests live in sibling `*tests.rs` files**, not inline `#[cfg(test)]` modules. Black-box CLI tests are `.toml` case files under `crates/kali_cli/tests/cases/`, run by the single `cases` target.
- **The existing guard message keeps its current text as a prefix.** `array_literal_arguments_benchmark_is_rejected_fail_closed` (`crates/kali_cli/tests/runtime_smoke/misc.rs`) and `crates/kali_cli/tests/inprocess/benchmark_execution.rs:915` pin it.
- **Commit after every task.** End each commit message with:
  `Claude-Session: https://claude.ai/code/session_01481SYVapRePAhfEgNsWrVm`

## File Structure

**Created:**
- `crates/kali_cli/tests/cases/runtime/inline_allocation_value_position.toml` — every pin this project adds (Task 2).
- `docs/superpowers/followups/inline-allocation-value-position-discovered-defects.md` — what this project measured and did not fix (Task 10).

**Modified — production:**
- `crates/kali_types/src/resolve/expression.rs:1862` — the `ArrayExpression` arm gains the allocation refusal (Task 6).
- `crates/kali_codegen/src/emit/call.rs:5567` (`emit_array_allocation_with_len`), `:5991` (`emit_array_fill`) — dedicated scratch slots, value evaluated once (Task 7).
- `crates/kali_codegen/src/lower.rs:1442-1556` — the trailing scratch reservation grows from 2 to 5 (Task 7).
- `crates/kali_codegen/src/emit/control_flow.rs:2181` — allocation and fill shapes route before the aggregate placeholder (Task 8).
- `crates/kali_codegen/src/emit/call.rs:100-102` — a bare `Array(n)` call allocates (Task 8).
- `crates/kali_codegen/src/emit/call.rs:3573-3615` — the argument guard widens (Task 9).

**Modified — tests:**
- `crates/kali_types/src/resolve/expression_tests.rs` — the refusal's unit test (Task 6).
- `crates/kali_codegen/src/emit/call_tests/alloc_helper.rs` — the reservation's unit test (Task 7).
- `crates/kali_cli/tests/cases/browser/{object_entries_iteration,object_keys_entries_spread_bundle,object_keys_entries_spread_harness,object_values_spread_bundle,object_values_spread_harness}.toml` and `crates/kali_cli/tests/cases/soundness/abort.toml:890` — the 81 re-pins (Task 9).

**Modified — ledger:**
- `docs/superpowers/followups/kali-silent-miscompile-register.md` — R-64..R-67 (Tasks 3-5, retired in 6-9).
- `crates/kali_cli/tests/cases/oracle/tier2.toml` — four oracle pairs.
- `tools/blast-radius/{predicates.json,counts.json,matchers.mjs,matchers.test.mjs,count.mjs,clusters.json}`, `crates/kali_blast_radius/src/manifest_tests.rs`.
- `docs/superpowers/followups/{length-fails-closed-discovered-defects.md,codegen-array-literal-predicate-is-still-negative-space.md}` (Task 10).

---

### Task 1: Settle the generator route

Spec §4.1's first stop gate. Four of the five browser case files Task 9 re-pins are generated. Neither generator runs from CI or `--gates-only`, so we must know whether the shipped files are still a fixed point **before** anything edits them. No production code changes in this task.

**Files:**
- Modify: `docs/superpowers/specs/2026-09-11-inline-allocation-value-position-design.md` (§4.1, record the answer)

**Interfaces:**
- Consumes: nothing.
- Produces: the recorded route Task 9 follows — either "regenerate" (edit the generator, re-run it) or "hand-edit, and record the divergence in each file's header".

- [ ] **Step 1: Confirm a clean tree at the baseline**

```bash
cd /workspace
git status --porcelain            # expect: empty
git rev-parse --short HEAD        # expect: the spec commit 46370a3d35
```

- [ ] **Step 2: Run both generators over exactly the five targets**

They write `crates/kali_cli/tests/cases/browser/<target>.toml` unconditionally (`case_emit.write`, called from each generator's `main`).

```bash
cd /workspace/tools/task-18-browser-pilot
python3 gen_batch6a.py object_entries_iteration
python3 gen_batch7b.py object_keys_entries_spread_bundle object_keys_entries_spread_harness \
                       object_values_spread_bundle object_values_spread_harness
```

- [ ] **Step 3: Diff, and record which answer came back**

```bash
cd /workspace
git diff --stat crates/kali_cli/tests/cases/browser/
```

Expected, one of:
- **no diff** → the generators are a fixed point. Task 9 edits the generators' `asserts=` dicts and re-runs them.
- **a diff** → the shipped files already diverge from their generators. Task 9 hand-edits the case files, and each edited file's `#` header gains one line recording that its generator no longer reproduces it.

- [ ] **Step 4: Restore the tree**

```bash
git checkout -- crates/kali_cli/tests/cases/browser/
git status --porcelain            # expect: empty
```

- [ ] **Step 5: Record the answer in the spec**

Append to §4.1, item 1, replacing nothing else:

```markdown
**Route, measured 2026-09-11 at `733cd26125`:** running `gen_batch6a.py
object_entries_iteration` and `gen_batch7b.py object_keys_entries_spread_bundle
object_keys_entries_spread_harness object_values_spread_bundle
object_values_spread_harness` produced <no diff | a diff of N files> against the
shipped case files. Task 9 therefore <edits the generators and re-runs them |
hand-edits the case files and records the divergence in each file's header>.
```

Delete the alternative that did not happen; leave one sentence of fact.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/specs/2026-09-11-inline-allocation-value-position-design.md
git commit -m "docs(specs): record whether the browser case generators still reproduce their files

Claude-Session: https://claude.ai/code/session_01481SYVapRePAhfEgNsWrVm"
```

---

### Task 2: The pins, committed red

Spec §4.2. Every expectation below was measured at `733cd26125` against node v26.8.2. The must-refuse and must-compute pins go in red; the controls go in green.

**Files:**
- Create: `crates/kali_cli/tests/cases/runtime/inline_allocation_value_position.toml`

**Interfaces:**
- Consumes: nothing.
- Produces: the trial ids `runtime/inline_allocation_value_position::<case name>` that Tasks 6-9 turn green, and the expected-red set every task from here on measures its gate against.

- [ ] **Step 1: Measure every program at the baseline, and write the readings down**

Six rationales in Step 2's file quote values measured on the SPIKE build rather
than the baseline — the concat, field-store, compound-assignment, element-store,
size-from-allocation and loop cases. Every rationale must carry a baseline
reading, so take them all first:

```bash
cd /workspace
git status --porcelain                    # expect: empty
cargo build -p kali_cli                   # binary at .cache/cargo-target/debug/kali
mkdir -p /tmp/alloc-probes && cd /tmp/alloc-probes
# write each `[source]` program from Step 2 to its own .js file here, then:
for f in *.js; do
  n=$(node "$f" 2>&1 | tr '\n' ' ')
  /workspace/.cache/cargo-target/debug/kali run "$f" > out.txt 2> err.txt; ec=$?
  printf '%s | node: %s | kali: %s | exit=%s %s\n' "$f" "$n" \
    "$(tr '\n' ' ' < out.txt)" "$ec" "$(grep -o 'error\[E[0-9]*\]' err.txt | head -1)"
done
```

Each rationale then records kali's and node's output from THIS run. Where a
reading disagrees with the value quoted in Step 2, the reading wins and the
rationale says so.

- [ ] **Step 2: Write the case file**

```toml
# Cases for the inline-allocation-value-position project (spec
# docs/superpowers/specs/2026-09-11-inline-allocation-value-position-design.md,
# section 4.2).
#
# An `Array`/`Uint8Array` allocation in value position is a real array handle; an
# array kali cannot pass as a handle refuses; `.fill(v)` evaluates `v` once. Every
# `_computes` and `_is_correct` case must print node's exact output. Every
# `_refuses` case printed a wrong value at exit 0 with no diagnostic at
# `733cd26125`, and its rationale records kali's and node's output there. Measured
# against node v26.8.2.
#
# `[source]` keys are one file per program, named for the shape, because
# `[source]` is file-wide (see this directory's README).

[constants]
ALLOC_IN_LITERAL = "lowers to the same node as the allocation itself"
NOT_PASSABLE = "the callee would read zero placeholders"

[source]
"arg_length.js" = '''
function f(x) { return x.length; } console.log(f(new Array(6)));
'''
"arg_length_fn.js" = '''
function main() { function f(x) { return x.length; } console.log(f(new Array(6))); }
main();
'''
"arg_fill_element.js" = '''
function f(x) { return x[0]; } console.log(f(new Array(3).fill(2)));
'''
"arg_fill_sum.js" = '''
function f(x) { let s = 0; for (let i = 0; i < x.length; i++) { s = s + x[i]; } return s; }
console.log(f(new Array(5).fill(2)));
'''
"arg_second_position.js" = '''
function f(a, x) { return a + x[0]; } console.log(f(1, new Array(3).fill(9)));
'''
"arg_beside_bound.js" = '''
function f(x, y) { return x.length + y.length; }
const a = new Array(2);
console.log(f(a, new Array(3)));
'''
"arg_returned_allocation.js" = '''
function mk() { return new Array(3).fill(4); }
function f(x) { return x[2]; }
console.log(f(mk()));
'''
"arg_through_wrapper.js" = '''
function f(x) { return x.length; } function g(y) { return f(y); } console.log(g(new Array(7)));
'''
"arg_ternary_arm.js" = '''
function f(x) { return x[0]; }
const c = true;
console.log(f(c ? new Array(2).fill(7) : new Array(2).fill(8)));
'''
"arg_dynamic_size.js" = '''
function main() { const n = 4; function f(x) { return x.length; } console.log(f(new Array(n))); }
main();
'''
"arg_bare_array_call.js" = '''
function f(x) { return x.length; } console.log(f(Array(6)));
'''
"arg_size_from_allocation.js" = '''
function f(x) { return x.length; } const a = new Array(f(new Array(3))); console.log(a.length);
'''
"arg_nested_call.js" = '''
function f(x) { return x.length; } function g(n) { return n + 1; } console.log(g(f(new Array(5))));
'''
"arg_in_loop.js" = '''
function f(x) { return x.length; }
let s = 0;
for (let i = 1; i < 4; i++) { s = s + f(new Array(i)); }
console.log(s);
'''
"arg_fill_from_binding.js" = '''
function main() { const v = 7; function f(x) { return x[1]; } console.log(f(new Array(3).fill(v))); }
main();
'''
"arg_in_concat.js" = '''
function f(x) { return x.length; } const s = "ab" + f(new Array(4)); console.log(s);
'''
"arg_in_field_store.js" = '''
function f(x) { return x.length; } const o = {k: 1}; o.k = f(new Array(9)); console.log(o.k);
'''
"arg_in_compound_assign.js" = '''
function f(x) { return x.length; } let t = 0; t += f(new Array(3)); console.log(t);
'''
"arg_in_element_store.js" = '''
function f(x) { return x.length; }
const a = new Array(2).fill(0);
a[1] = f(new Array(6));
console.log(a[0] + "," + a[1]);
'''
"fill_value_allocation.js" = '''
const a = new Array(2).fill(new Array(3)); console.log(a.length);
'''
"fill_value_allocation_fn.js" = '''
function main() { const a = new Array(2).fill(new Array(3)); console.log(a.length); }
main();
'''
"fill_value_call_allocation.js" = '''
function f(x) { return x.length; }
const a = new Array(2).fill(f(new Array(5)));
console.log(a.length + "," + a[0]);
'''
"fill_value_ternary_allocation.js" = '''
function f(x) { return x.length; }
const c = 1;
const a = new Array(2).fill(c ? f(new Array(4)) : 0);
console.log(a[1] + "," + a.length);
'''
"fill_value_once.js" = '''
let n = 0;
function g() { n = n + 1; return 1; }
const a = new Array(3).fill(g());
console.log(n + "," + a[2]);
'''
"fill_value_once_fn.js" = '''
function main() {
  let n = 0;
  function g() { n = n + 1; return 1; }
  const a = new Array(4).fill(g());
  console.log(n + "," + a[0]);
}
main();
'''
"fill_value_once_rebound.js" = '''
let n = 0;
function g() { n = n + 1; return 2; }
const a = new Array(3).fill(0);
a.fill(g());
console.log(n + "," + a[1]);
'''
"literal_of_allocation.js" = '''
const xs = [new Array(3)]; console.log(xs.length);
'''
"literal_of_allocation_fn.js" = '''
function main() { const xs = [new Array(3)]; console.log(xs.length); }
main();
'''
"literal_of_bare_allocation.js" = '''
const xs = [Array(3)]; console.log(xs.length);
'''
"literal_of_fill.js" = '''
const xs = [Array(3).fill(1)]; console.log(xs.length);
'''
"literal_of_allocation_argument.js" = '''
function f(x) { return x.length; } console.log(f([new Array(3)]));
'''
"arg_literal_identifier.js" = '''
function f(x) { return x[0]; } const k = 3; console.log(f([k]));
'''
"arg_literal_expression.js" = '''
function f(x) { return x[0]; } console.log(f([1 + 1]));
'''
"arg_literal_two_elements.js" = '''
function f(x) { return x[0] + x[1]; } const k = 3; console.log(f([k, 1]));
'''
"arg_literal_bound.js" = '''
function f(x) { return x[0]; } const k = 3; const arr = [k]; console.log(f(arr));
'''
"arg_literal_call_element.js" = '''
function g() { return 5; } function f(x) { return x[0]; } console.log(f([g()]));
'''
"arg_values_spread.js" = '''
function show(v) { console.log(v.length + "," + v[0] + "," + v[1]); }
const fe = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
const collected = [...Object.values(fe)];
show(collected);
'''
"arg_entries.js" = '''
function show(e) { console.log(e.length + "," + e[0][0] + "," + e[0][1]); }
function main() { const alias = {b: 1, a: 2}; const entries = Object.entries(alias); show(entries); }
main();
'''
"arg_construction.js" = '''
class C { constructor() { this.v = 4; } }
function f(x) { return x.v; }
console.log(f(new C()));
'''
"control_bound_allocation.js" = '''
function f(x) { return x.length; } const a = new Array(6); console.log(f(a));
'''
"control_let_fill.js" = '''
let a = new Array(3).fill(2); console.log(a[1]);
'''
"control_const_fill_fn.js" = '''
function main() { const a = new Array(3).fill(2); console.log(a[1]); } main();
'''
"control_expression_statement.js" = '''
new Array(3); console.log("ok");
'''
"control_ignored_argument.js" = '''
function f(x) { return 1; } console.log(f(new Array(3)));
'''
"control_two_element_literal.js" = '''
const xs = [new Array(3), new Array(2)]; console.log(xs.length);
'''
"control_bound_returned_allocation.js" = '''
function mk() { const a = new Array(3).fill(4); return a; }
function f(x) { return x[2]; }
console.log(f(mk()));
'''

[[case]]
name = "an_allocation_argument_length_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `6`: silently wrong. An allocation in value position must pass a real handle (spec sections 3.2-3.3)."""
args = ["run", "arg_length.js"]
exit = "success"
stdout = "6\n"

[[case]]
name = "an_allocation_argument_length_computes_in_a_function"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `6`: silently wrong, identically to the module scope (spec sections 3.2-3.3)."""
args = ["run", "arg_length_fn.js"]
exit = "success"
stdout = "6\n"

[[case]]
name = "a_fill_argument_element_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `2`: silently wrong (spec sections 3.2-3.3)."""
args = ["run", "arg_fill_element.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "a_fill_argument_summed_in_the_callee_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `10`: the callee's loop read a length header at address zero (spec sections 3.2-3.3)."""
args = ["run", "arg_fill_sum.js"]
exit = "success"
stdout = "10\n"

[[case]]
name = "an_allocation_in_second_argument_position_computes"
rationale = """At `733cd26125` kali printed `1` at exit 0 with no diagnostic where node v26.8.2 prints `10`: the zero handle reached the second parameter (spec sections 3.2-3.3)."""
args = ["run", "arg_second_position.js"]
exit = "success"
stdout = "10\n"

[[case]]
name = "an_allocation_beside_a_bound_argument_computes"
rationale = """At `733cd26125` kali printed `2` at exit 0 with no diagnostic where node v26.8.2 prints `5`: the bound argument was right and the inline one was zero (spec sections 3.2-3.3)."""
args = ["run", "arg_beside_bound.js"]
exit = "success"
stdout = "5\n"

[[case]]
name = "an_allocation_returned_and_passed_on_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `4` (spec sections 3.2-3.3)."""
args = ["run", "arg_returned_allocation.js"]
exit = "success"
stdout = "4\n"

[[case]]
name = "an_allocation_through_a_wrapper_function_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `7` (spec sections 3.2-3.3)."""
args = ["run", "arg_through_wrapper.js"]
exit = "success"
stdout = "7\n"

[[case]]
name = "an_allocation_in_a_ternary_arm_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `7` (spec sections 3.2-3.3)."""
args = ["run", "arg_ternary_arm.js"]
exit = "success"
stdout = "7\n"

[[case]]
name = "an_allocation_with_a_dynamic_size_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `4` (spec sections 3.2-3.3)."""
args = ["run", "arg_dynamic_size.js"]
exit = "success"
stdout = "4\n"

[[case]]
name = "a_bare_array_call_argument_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `6`. `Array(n)` without `new` reaches `emit_call`, which had no allocation arm (spec section 3.2)."""
args = ["run", "arg_bare_array_call.js"]
exit = "success"
stdout = "6\n"

[[case]]
name = "an_allocation_sized_from_another_allocation_computes"
rationale = """Correct at `733cd26125` (kali and node v26.8.2 both print `3`) because the size argument is evaluated before any slot is written. It must stay correct once allocations nest (spec section 3.3)."""
args = ["run", "arg_size_from_allocation.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "an_allocation_inside_a_nested_call_computes"
rationale = """At `733cd26125` kali printed `1` at exit 0 with no diagnostic where node v26.8.2 prints `6` (spec sections 3.2-3.3)."""
args = ["run", "arg_nested_call.js"]
exit = "success"
stdout = "6\n"

[[case]]
name = "an_allocation_argument_inside_a_loop_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `6` (spec sections 3.2-3.3)."""
args = ["run", "arg_in_loop.js"]
exit = "success"
stdout = "6\n"

[[case]]
name = "a_fill_argument_whose_value_is_a_binding_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `7` (spec sections 3.2-3.3)."""
args = ["run", "arg_fill_from_binding.js"]
exit = "success"
stdout = "7\n"

[[case]]
name = "an_allocation_argument_in_string_concatenation_computes"
rationale = """At `733cd26125` kali printed `ab0` at exit 0 with no diagnostic where node v26.8.2 prints `ab4`. A concat sink holds the general scratch slot while emitting its operand, so this pins that the allocation does not clobber it (spec section 3.3)."""
args = ["run", "arg_in_concat.js"]
exit = "success"
stdout = "ab4\n"

[[case]]
name = "an_allocation_argument_in_a_field_store_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `9`. A field store holds the general scratch slot across its value emission (spec section 3.3)."""
args = ["run", "arg_in_field_store.js"]
exit = "success"
stdout = "9\n"

[[case]]
name = "an_allocation_argument_in_a_compound_assignment_computes"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `3` (spec sections 3.2-3.3)."""
args = ["run", "arg_in_compound_assign.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "an_allocation_argument_in_an_element_store_computes"
rationale = """At `733cd26125` kali printed `0,0` at exit 0 with no diagnostic where node v26.8.2 prints `0,6`. An element store holds an address across its value emission (spec section 3.3)."""
args = ["run", "arg_in_element_store.js"]
exit = "success"
stdout = "0,6\n"

[[case]]
name = "an_allocation_as_a_fill_value_keeps_the_receivers_length"
rationale = """Correct at `733cd26125` (kali and node v26.8.2 both print `2`) only because the fill value was a placeholder zero. Once the value allocates it must not overwrite the receiver's base handle: the spike measured `3` here (spec section 3.3)."""
args = ["run", "fill_value_allocation.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "an_allocation_as_a_fill_value_keeps_the_receivers_length_in_a_function"
rationale = """Node v26.8.2 prints `2`. The spike measured `3`, the same clobber as the module scope (spec section 3.3)."""
args = ["run", "fill_value_allocation_fn.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "a_call_on_an_allocation_as_a_fill_value_computes"
rationale = """At `733cd26125` kali printed `2,0` at exit 0 with no diagnostic where node v26.8.2 prints `2,5`; the spike printed `5,0`, clobbering the receiver's handle with the value's allocation (spec sections 3.2-3.3)."""
args = ["run", "fill_value_call_allocation.js"]
exit = "success"
stdout = "2,5\n"

[[case]]
name = "a_ternary_over_an_allocation_as_a_fill_value_computes"
rationale = """At `733cd26125` kali printed `0,2` at exit 0 with no diagnostic where node v26.8.2 prints `4,2`; the spike printed `0,4` (spec sections 3.2-3.3)."""
args = ["run", "fill_value_ternary_allocation.js"]
exit = "success"
stdout = "4,2\n"

[[case]]
name = "a_fill_value_is_evaluated_once"
rationale = """At `733cd26125` kali printed `3,1` at exit 0 with no diagnostic where node v26.8.2 prints `1,1`: the fill loop re-emitted the value for every element, so `g` ran three times (spec section 3.3)."""
args = ["run", "fill_value_once.js"]
exit = "success"
stdout = "1,1\n"

[[case]]
name = "a_fill_value_is_evaluated_once_in_a_function"
rationale = """At `733cd26125` kali printed `4,1` at exit 0 with no diagnostic where node v26.8.2 prints `1,1` (spec section 3.3)."""
args = ["run", "fill_value_once_fn.js"]
exit = "success"
stdout = "1,1\n"

[[case]]
name = "a_fill_value_on_a_bound_array_is_evaluated_once"
rationale = """At `733cd26125` kali printed `3,2` at exit 0 with no diagnostic where node v26.8.2 prints `1,2` (spec section 3.3)."""
args = ["run", "fill_value_once_rebound.js"]
exit = "success"
stdout = "1,2\n"

[[case]]
name = "a_one_element_literal_of_an_allocation_refuses"
rationale = """At `733cd26125` kali printed `3` at exit 0 with no diagnostic where node v26.8.2 prints `1`: the literal lowers to the same LIR node as `new Array(3)`, and the declarator lane read it as the allocation (spec section 3.1)."""
args = ["run", "literal_of_allocation.js"]
exit = "failure"
stderr_contains = ["E5506", "${ALLOC_IN_LITERAL}"]

[[case]]
name = "a_one_element_literal_of_an_allocation_refuses_in_a_function"
rationale = """At `733cd26125` kali printed `3` at exit 0 with no diagnostic where node v26.8.2 prints `1`, identically to the module scope (spec section 3.1)."""
args = ["run", "literal_of_allocation_fn.js"]
exit = "failure"
stderr_contains = ["E5506", "${ALLOC_IN_LITERAL}"]

[[case]]
name = "a_one_element_literal_of_a_bare_array_call_refuses"
rationale = """At `733cd26125` kali printed `3` at exit 0 with no diagnostic where node v26.8.2 prints `1` (spec section 3.1)."""
args = ["run", "literal_of_bare_allocation.js"]
exit = "failure"
stderr_contains = ["E5506", "${ALLOC_IN_LITERAL}"]

[[case]]
name = "a_one_element_literal_of_a_fill_refuses"
rationale = """At `733cd26125` kali printed `3` at exit 0 with no diagnostic where node v26.8.2 prints `1` (spec section 3.1)."""
args = ["run", "literal_of_fill.js"]
exit = "failure"
stderr_contains = ["E5506", "${ALLOC_IN_LITERAL}"]

[[case]]
name = "a_one_element_literal_of_an_allocation_as_an_argument_refuses"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `1` (spec section 3.1)."""
args = ["run", "literal_of_allocation_argument.js"]
exit = "failure"
stderr_contains = ["E5506", "${ALLOC_IN_LITERAL}"]

[[case]]
name = "an_array_literal_argument_holding_an_identifier_refuses"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `3`: the guard only refused arrays whose elements are all literals (spec section 3.4)."""
args = ["run", "arg_literal_identifier.js"]
exit = "failure"
stderr_contains = ["E5506", "${NOT_PASSABLE}"]

[[case]]
name = "an_array_literal_argument_holding_an_expression_refuses"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `2` (spec section 3.4)."""
args = ["run", "arg_literal_expression.js"]
exit = "failure"
stderr_contains = ["E5506", "${NOT_PASSABLE}"]

[[case]]
name = "an_array_literal_argument_of_two_elements_refuses"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `4` (spec section 3.4)."""
args = ["run", "arg_literal_two_elements.js"]
exit = "failure"
stderr_contains = ["E5506", "${NOT_PASSABLE}"]

[[case]]
name = "a_bound_array_literal_argument_refuses"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `3`: binding the literal first changes nothing, because the guard resolves the fold lane (spec section 3.4)."""
args = ["run", "arg_literal_bound.js"]
exit = "failure"
stderr_contains = ["E5506", "${NOT_PASSABLE}"]

[[case]]
name = "an_array_literal_argument_holding_a_call_refuses"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `5`. This shape lowers identically to `new g()`, which is why the guard refuses both (spec section 3.4)."""
args = ["run", "arg_literal_call_element.js"]
exit = "failure"
stderr_contains = ["E5506", "${NOT_PASSABLE}"]

[[case]]
name = "an_object_values_spread_argument_refuses"
rationale = """At `733cd26125` kali printed `0,0,0` at exit 0 with no diagnostic where node v26.8.2 prints `2,3,2`. Five browser case files program this shape and failed only because their assert helpers throw (spec section 3.4)."""
args = ["run", "arg_values_spread.js"]
exit = "failure"
stderr_contains = ["E5506", "${NOT_PASSABLE}"]

[[case]]
name = "an_object_entries_argument_refuses"
rationale = """At `733cd26125` kali printed `0,0,0` at exit 0 with no diagnostic where node v26.8.2 prints `2,b,1` (spec section 3.4)."""
args = ["run", "arg_entries.js"]
exit = "failure"
stderr_contains = ["E5506", "${NOT_PASSABLE}"]

[[case]]
name = "a_constructed_argument_refuses"
rationale = """At `733cd26125` kali printed `0` at exit 0 with no diagnostic where node v26.8.2 prints `4`. `new C()` lowers to the same LIR node as `[C()]`, so the guard cannot admit one without admitting the other; the human partner ruled the wide guard (spec section 3.4)."""
args = ["run", "arg_construction.js"]
exit = "failure"
stderr_contains = ["E5506", "${NOT_PASSABLE}"]

[[case]]
name = "a_bound_allocation_argument_is_correct"
rationale = """Correct at `733cd26125`: kali and node v26.8.2 both print `6`. The control that locates the fault in the inline allocation rather than in the callee (spec section 2.2)."""
args = ["run", "control_bound_allocation.js"]
exit = "success"
stdout = "6\n"

[[case]]
name = "a_let_fill_declarator_is_correct"
rationale = """Correct at `733cd26125`: kali and node v26.8.2 both print `2`. The declarator lane must not move (spec section 3.2)."""
args = ["run", "control_let_fill.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "a_const_fill_declarator_in_a_function_is_correct"
rationale = """Correct at `733cd26125`: kali and node v26.8.2 both print `2` (spec section 3.2)."""
args = ["run", "control_const_fill_fn.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "an_allocation_expression_statement_is_correct"
rationale = """Correct at `733cd26125`: kali and node v26.8.2 both print `ok`. An allocation whose value is discarded must still compile (spec section 3.2)."""
args = ["run", "control_expression_statement.js"]
exit = "success"
stdout = "ok\n"

[[case]]
name = "an_ignored_allocation_argument_is_correct"
rationale = """Correct at `733cd26125`: kali and node v26.8.2 both print `1`. A callee that ignores its parameter never reads the zero handle (spec section 2.2)."""
args = ["run", "control_ignored_argument.js"]
exit = "success"
stdout = "1\n"

[[case]]
name = "a_two_element_literal_of_allocations_is_correct"
rationale = """Correct at `733cd26125`: kali and node v26.8.2 both print `2`. Only the ONE-element literal collides with an allocation's LIR node, so the refusal must not widen to this (spec section 3.1)."""
args = ["run", "control_two_element_literal.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "a_bound_returned_allocation_argument_is_correct"
rationale = """Correct at `733cd26125`: kali and node v26.8.2 both print `4`. The allocation is bound inside `mk` before being returned, so a real handle crosses the call (spec section 2.2)."""
args = ["run", "control_bound_returned_allocation.js"]
exit = "success"
stdout = "4\n"
```

- [ ] **Step 3: Run the new cases and record the expected-red set**

```bash
cd /workspace
cargo test -p kali_cli --test cases -- runtime/inline_allocation_value_position 2>&1 | tail -30
```

Expected: the 7 `control_` cases pass, and so does every case Step 1 measured as already correct at the baseline (the size-from-allocation case and the two `fill_value_allocation*` clobber cases). **Every other case fails.** Save the failing list — it is the expected-red set for Tasks 3 through 8:

```bash
cargo test -p kali_cli --test cases -- runtime/inline_allocation_value_position 2>&1 \
  | awk '/^failures:$/ {c=1; next} c && /^    [A-Za-z_]/ {print $1; next} c {c=0}' | sort -u \
  > /tmp/expected-red.txt
wc -l /tmp/expected-red.txt          # expect: 36 — but the LIST is what later gates diff against, not the count
```

- [ ] **Step 4: Confirm nothing else moved**

```bash
bash scripts/test-gate.sh 2>&1 | tail -50
```

Expected: `GATE FAILED`, and the failing list is exactly the ids in `/tmp/expected-red.txt`. Any other id means the case file itself is malformed — fix it before committing.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/cases/runtime/inline_allocation_value_position.toml
git commit -m "test(alloc): pin every value-position allocation lane, red

The cases in /tmp/expected-red.txt fail against 733cd26125; the controls and the
three already-correct cases pass. Each rationale records kali's and node
v26.8.2's output at the baseline, measured in this task's first step.

Claude-Session: https://claude.ai/code/session_01481SYVapRePAhfEgNsWrVm"
```

---

### Task 3: File R-64 — an allocation outside the three materializing lanes evaluates to `0`

Spec §6.1. Filed SILENT, in its own commit, so the retirement in Task 8 is a separate step a reader can audit.

**Files:**
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (§0.2 row, §1 count paragraph, a new `### R-64` entry in Tier 2)
- Modify: `tools/blast-radius/matchers.mjs`, `matchers.test.mjs`, `predicates.json`, `count.mjs`, `counts.json`, `clusters.json`
- Modify: `crates/kali_blast_radius/src/manifest_tests.rs`
- Modify: `crates/kali_cli/tests/cases/oracle/tier2.toml`

**Interfaces:**
- Consumes: Task 2's measurements (kali's and node's output per program).
- Produces: register id `R-64`; matcher name `allocationOutsideMaterializingLane`; oracle case names `r64a_allocation_argument_module_scope`, `r64a_allocation_argument_in_function`.

- [ ] **Step 1: Write the matcher's failing test**

Append to `tools/blast-radius/matchers.test.mjs`:

```javascript
test("allocationOutsideMaterializingLane counts allocations no lane materializes", () => {
  // Positives: an allocation as a call argument, as a `.fill` argument's own
  // receiver-free value, in a ternary arm, returned, in an object property, and
  // the bare `Array(n)` spelling. Negatives: the three lanes that DO materialize
  // -- a declarator init, an assignment right-hand side, and a `.fill` receiver --
  // and a non-array constructor.
  const src = `
    function f(x) { return x.length; }
    console.log(f(new Array(6)));        // argument, counts
    console.log(f(Array(6)));            // bare call argument, counts
    console.log(f(new Uint8Array(4)));   // typed-array argument, counts
    const c = true;
    console.log(f(c ? new Array(2) : new Array(3))); // two ternary arms, counts twice
    function mk() { return new Array(3); }           // returned, counts
    const o = { arr: new Array(3) };                 // property value, counts
    const a = new Array(5);              // declarator init, does not count
    let b;
    b = new Array(2);                    // assignment rhs, does not count
    const d = new Array(4).fill(1);      // fill receiver + declarator, does not count
    const e = new Map();                 // not an array allocation, does not count
  `;
  assert.equal(count("allocationOutsideMaterializingLane", src), 7);
});
```

- [ ] **Step 2: Run it to watch it fail**

```bash
cd /workspace/tools/blast-radius
npm ci
node --test 2>&1 | tail -20
```

Expected: FAIL naming `allocationOutsideMaterializingLane` as an unknown matcher.

- [ ] **Step 3: Add the matcher**

In `tools/blast-radius/matchers.mjs`, inside the `MATCHERS` object (before its closing `};` at `:1314`), after the `lengthReadOnUnprovenReceiver` entry:

```javascript
  // R-64: an `Array`/`Uint8Array` allocation whose value is used somewhere none
  // of codegen's three materializing lanes reaches -- the declarator init
  // (`crates/kali_codegen/src/emit/control_flow.rs:1690`, `:1711`), the
  // assignment right-hand side (`emit/literal.rs:1043`) and the `.fill` receiver
  // (`emit/call.rs:6004`). Everywhere else the node is a text-less `Value` and
  // `emit_aggregate_literal` drops it and pushes `0`.
  //
  // Upper bound, disclosed in `count.mjs`'s UPPER_BOUNDS: an acorn AST cannot see
  // that a callee ignores its parameter, and such a call is counted even though
  // it prints correctly.
  allocationOutsideMaterializingLane(ast) {
    const analysis = analysisOf(ast);
    const materialized = new Set();
    for (const node of analysis.of("VariableDeclarator")) {
      if (node.init) materialized.add(node.init);
    }
    for (const node of analysis.of("AssignmentExpression")) {
      materialized.add(node.right);
    }
    for (const node of analysis.of("CallExpression")) {
      if (node.callee.type === "MemberExpression" && node.callee.property.name === "fill") {
        materialized.add(node.callee.object);
      }
    }
    return analysis
      .of("CallExpression")
      .concat(analysis.of("NewExpression"))
      .filter((node) => isArrayAllocation(node) && !materialized.has(node))
      .length;
  },
```

And, beside `isUnprovenLengthReceiver` (`matchers.mjs:463`):

```javascript
/**
 * `Array(n)` / `new Array(n)` / `Uint8Array(n)` / `new Uint8Array(n)`, the callee
 * spellings `FunctionEmitter::is_array_like_constructor`
 * (`crates/kali_codegen/src/emit/call.rs:5510`) accepts, with at most one argument.
 */
function isArrayAllocation(node) {
  const callee = node.callee;
  if (!callee || callee.type !== "Identifier") return false;
  if (callee.name !== "Array" && callee.name !== "Uint8Array") return false;
  return node.arguments.length <= 1;
}
```

- [ ] **Step 4: Run the matcher tests**

```bash
cd /workspace/tools/blast-radius
node --test 2>&1 | tail -20
```

Expected: PASS, all tests.

- [ ] **Step 5: Add the catalogue record and the upper-bound note**

In `tools/blast-radius/predicates.json`, after R-63's record:

```json
    { "id": "R-64", "kind": "countable",
      "matcher": "allocationOutsideMaterializingLane",
      "description": "an `Array`/`Uint8Array` allocation (`new Array(n)`, `Array(n)`, at most one argument) whose value is used outside codegen's three materializing lanes -- a declarator initializer, an assignment right-hand side, or a `.fill` receiver; upper bound, because an acorn AST cannot see that a callee ignores the parameter it is passed to, and such a call prints correctly" }
```

In `tools/blast-radius/count.mjs`, in `UPPER_BOUNDS` beside R-63's entry:

```javascript
  "R-64": {
    disclosedInRecord: true,
    note:
      "Upper bound, per the record: the matcher counts every allocation outside the three " +
      "materializing lanes, and an acorn AST cannot see whether the value is ever read. " +
      "Measured at `733cd26125` against node v26.8.2, `function f(x) { return 1; } " +
      "f(new Array(3))` prints `1` on both engines and is counted.",
  },
```

- [ ] **Step 6: Write the register entry**

In `docs/superpowers/followups/kali-silent-miscompile-register.md`, in Tier 2 after `### R-63`, add `### R-64: An array allocation outside a declarator, assignment or `.fill` receiver evaluates to `0``, following R-63's shape exactly: **Added** (2026-09-11, by the inline-allocation-value-position project, naming this spec, off `733cd26125`), **Verification** (`CONFIRMED-BY-CONTROLLER`, measured at `733cd26125` against node v26.8.2, both scopes), **Root-cause group** (decided under §3's criteria and recorded with the reasoning: the value is dropped and a type-plausible `0` is pushed in its place, with no diagnostic), **Repro** (`function f(x) { return x.length; } console.log(f(new Array(6)));` → node `6`, kali `0`), **Every lane** (the table of §2.2 of the spec, verbatim, all exit 0 with empty stderr), **Severity** (silent-wrong-value), **Blast radius**, **Confidence**.

Add its §0.2 row (`| R-64 an allocation outside the materializing lanes → 0 | **SILENT** (both scopes) | … |`) and update §1's entry-count and case-count paragraphs the way R-63's filing did (two more cases).

- [ ] **Step 7: Add the oracle pair**

In `crates/kali_cli/tests/cases/oracle/tier2.toml`, extend the header index at `:14` and `:122`, add the two programs to `[source]`:

```toml
"r64a_module.js" = """function f(x) { return x.length; }
console.log(f(new Array(6)));
"""
"r64a_function.js" = """function main() {
  function f(x) { return x.length; }
  console.log(f(new Array(6)));
}
main();
"""
```

and the two cases, `verdict = "silent"`, each rationale recording: the register's expected verdict, and `MEASURED 2026-09-11 at 733cd26125 against node v26.8.2: SILENT. kali prints 0 at exit 0 with empty stderr; node prints 6 at exit 0.` plus a `WHAT A REGRESSION HERE WOULD MEAN` paragraph saying the pair flips to FIXED in Task 8's commit.

- [ ] **Step 8: Regenerate the counts and record the re-freeze**

```bash
cd /workspace
cargo build -p kali_cli
cd tools/blast-radius
node --test
node accepts.mjs
node count.mjs
cd ../..
cargo run -p kali_blast_radius --example rank
```

`rank`'s stdout replaces the generated region of `docs/superpowers/followups/blast-radius-ranking.md` between its HTML-comment markers. Then update the frozen hashes and the re-freeze narration in `crates/kali_blast_radius/src/manifest_tests.rs` (`FROZEN_PREDICATES_SHA256`, `FROZEN_MATCHERS_SHA256`), following the 2026-09-11 paragraph R-63 added.

- [ ] **Step 9: Run the gates**

```bash
cargo test -p kali_blast_radius 2>&1 | tail -20
cargo test -p kali_cli --test cases -- oracle/tier2 2>&1 | tail -10
```

Expected: both green. `spliced_document_matches_the_generator` proves the ranking matches the generator; the oracle pair proves R-64 is silent as filed.

- [ ] **Step 10: Commit**

```bash
git add docs/superpowers/followups/kali-silent-miscompile-register.md \
        docs/superpowers/followups/blast-radius-ranking.md \
        tools/blast-radius crates/kali_blast_radius/src/manifest_tests.rs \
        crates/kali_cli/tests/cases/oracle/tier2.toml
git commit -m "docs(register): file R-64 -- an allocation outside the materializing lanes evaluates to 0

Claude-Session: https://claude.ai/code/session_01481SYVapRePAhfEgNsWrVm"
```

---

### Task 4: File R-65 — a fold-lane array passed to a user function reads as zeros

Same shape as Task 3, for the second entry.

**Files:** as Task 3.

**Interfaces:**
- Consumes: Task 3's `isArrayAllocation` helper in `matchers.mjs`.
- Produces: register id `R-65`; matcher `foldLaneArrayArgument`; oracle cases `r65a_fold_lane_argument_module_scope`, `r65a_fold_lane_argument_in_function`.

- [ ] **Step 1: Write the matcher's failing test**

Append to `tools/blast-radius/matchers.test.mjs`:

```javascript
test("foldLaneArrayArgument counts array arguments kali cannot pass as a handle", () => {
  // Positives: an array literal argument (all-literal, identifier, expression,
  // call element), a spread of an Object.values/keys result, a bound literal, and
  // a constructed value -- which lowers to the same node as a one-element literal.
  // Negatives: an allocation argument (a real handle after this project), a bound
  // allocation, a scalar, and an array literal that is NOT an argument.
  const src = `
    function f(x) { return x[0]; }
    const k = 3;
    console.log(f([1, 2]));            // all-literal, counts
    console.log(f([k]));               // identifier element, counts
    console.log(f([1 + 1]));           // expression element, counts
    function g() { return 5; }
    console.log(f([g()]));             // call element, counts
    const arr = [k];
    console.log(f(arr));               // bound literal, counts
    const o = {b: 1};
    console.log(f([...Object.values(o)]));  // spread of a fold-lane result, counts
    class C {}
    console.log(f(new C()));           // constructed value, counts
    console.log(f(new Array(3)));      // allocation, does not count
    const a = new Array(2);
    console.log(f(a));                 // bound allocation, does not count
    console.log(f(7));                 // scalar, does not count
    const standalone = [1, 2];         // not an argument, does not count
  `;
  assert.equal(count("foldLaneArrayArgument", src), 7);
});
```

- [ ] **Step 2: Run it to watch it fail**

```bash
cd /workspace/tools/blast-radius && node --test 2>&1 | tail -20
```

Expected: FAIL naming `foldLaneArrayArgument` as an unknown matcher.

- [ ] **Step 3: Add the matcher**

In `MATCHERS`, after Task 3's entry:

```javascript
  // R-65: an argument to a user function that kali cannot pass as a runtime
  // handle -- an array literal (bound or inline, whatever its elements), a spread
  // of one, or a constructed value, which lowers to the same text-less `Value`
  // node as a one-element literal. The callee reads zero placeholders
  // (`crates/kali_codegen/src/emit/call.rs:3573-3615` at `733cd26125`).
  //
  // Upper bound, disclosed in `count.mjs`'s UPPER_BOUNDS: an acorn AST cannot see
  // whether the callee resolves to a compiled function, and the guard fires only
  // when it does.
  foldLaneArrayArgument(ast) {
    const analysis = analysisOf(ast);
    const arrayNames = new Set();
    for (const node of analysis.of("VariableDeclarator")) {
      if (node.id.type === "Identifier" && node.init && node.init.type === "ArrayExpression") {
        arrayNames.add(node.id.name);
      }
    }
    let total = 0;
    for (const call of analysis.of("CallExpression")) {
      for (const arg of call.arguments) {
        if (arg.type === "ArrayExpression") total += 1;
        else if (arg.type === "NewExpression" && !isArrayAllocation(arg)) total += 1;
        else if (arg.type === "Identifier" && arrayNames.has(arg.name)) total += 1;
      }
    }
    return total;
  },
```

- [ ] **Step 4: Run the matcher tests**

```bash
cd /workspace/tools/blast-radius && node --test 2>&1 | tail -20
```

Expected: PASS.

- [ ] **Step 5: Catalogue record, upper-bound note, register entry, oracle pair**

Four files, each edited the way R-63's filing edited it:

- `tools/blast-radius/predicates.json`: append the catalogue record below after R-64's.
- `tools/blast-radius/count.mjs`: add an `UPPER_BOUNDS` entry for R-65 saying the matcher cannot see whether the callee resolves to a compiled function, which is what the guard requires.
- `docs/superpowers/followups/kali-silent-miscompile-register.md`: a new `### R-65` entry in Tier 2 carrying **Added** (2026-09-11, this project, naming the spec, off `733cd26125`), **Verification** (`CONFIRMED-BY-CONTROLLER`, measured at `733cd26125` against node v26.8.2, both scopes), **Root-cause group** (decided under §3's criteria, with the reasoning recorded), **Repro**, **Every lane** (spec §2.4's table verbatim), **Severity** (silent-wrong-value), **Blast radius**, **Confidence**; plus its §0.2 row and §1's entry- and case-count paragraphs.
- `crates/kali_cli/tests/cases/oracle/tier2.toml`: the header index at `:14` and `:122`, the two programs, and the two cases.

The content:
- `predicates.json`: `{ "id": "R-65", "kind": "countable", "matcher": "foldLaneArrayArgument", "description": "an argument to a call that is an array literal (inline or through a binding), a spread of one, or a constructed value that lowers to the same text-less node; upper bound, because an acorn AST cannot see whether the callee resolves to a compiled function, which is what the guard requires" }`
- register entry `### R-65: A fold-lane array, or a constructed value that lowers like one, passed to a user function reads as zeros in the callee`, **Repro** `function f(x) { return x[0]; } const k = 3; console.log(f([k]));` → node `3`, kali `0`, with the spec §2.4 table as its lanes.
- oracle programs:

```toml
"r65a_module.js" = """function f(x) { return x[0]; }
const k = 3;
console.log(f([k]));
"""
"r65a_function.js" = """function main() {
  function f(x) { return x[0]; }
  const k = 3;
  console.log(f([k]));
}
main();
"""
```

with `verdict = "silent"` and a `WHAT A REGRESSION HERE WOULD MEAN` paragraph saying the pair flips to FAIL_CLOSED in Task 9's commit.

- [ ] **Step 6: Regenerate the counts and record the re-freeze**

```bash
cd /workspace
cargo build -p kali_cli
cd tools/blast-radius
node --test
node accepts.mjs
node count.mjs
cd /workspace
cargo run -p kali_blast_radius --example rank
```

`rank`'s stdout replaces the generated region of
`docs/superpowers/followups/blast-radius-ranking.md` between its HTML-comment
markers. Then update `FROZEN_PREDICATES_SHA256`, `FROZEN_MATCHERS_SHA256` and the
re-freeze narration in `crates/kali_blast_radius/src/manifest_tests.rs`, following
the 2026-09-11 paragraph R-63's filing added. Gate:

```bash
cargo test -p kali_blast_radius 2>&1 | tail -20
cargo test -p kali_cli --test cases -- oracle/tier2 2>&1 | tail -10
```

Expected: both green.

- [ ] **Step 7: Commit**

```bash
git add docs/superpowers/followups/kali-silent-miscompile-register.md \
        docs/superpowers/followups/blast-radius-ranking.md \
        tools/blast-radius crates/kali_blast_radius/src/manifest_tests.rs \
        crates/kali_cli/tests/cases/oracle/tier2.toml
git commit -m "docs(register): file R-65 -- a fold-lane array argument reads as zeros in the callee

Claude-Session: https://claude.ai/code/session_01481SYVapRePAhfEgNsWrVm"
```

---

### Task 5: File R-66 and R-67 — the literal collision and the re-evaluated fill value

Two entries in one cycle: both matchers are a handful of lines, and neither is fixed by the other's task.

**Files:** as Task 3.

**Interfaces:**
- Consumes: Task 3's `isArrayAllocation`.
- Produces: register ids `R-66`, `R-67`; matchers `oneElementLiteralOfAllocation`, `fillValueReevaluated`; oracle cases `r66a_*`, `r67a_*`.

- [ ] **Step 1: Write both matchers' failing tests**

Append to `tools/blast-radius/matchers.test.mjs`:

```javascript
test("oneElementLiteralOfAllocation counts literals that collide with an allocation", () => {
  // Positives: a one-element literal of `new Array(n)`, of a bare `Array(n)`, of a
  // `.fill` on one, and of a `Uint8Array`. Negatives: two elements (no collision),
  // an empty literal, a literal of a non-array constructor, and the allocation
  // itself outside a literal.
  const src = `
    const a = [new Array(3)];        // counts
    const b = [Array(3)];            // counts
    const c = [Array(3).fill(1)];    // counts
    const d = [new Uint8Array(2)];   // counts
    const e = [new Array(3), new Array(2)];  // two elements, does not count
    const f = [];                    // empty, does not count
    const g = [new Map()];           // not an array allocation, does not count
    const h = new Array(3);          // not in a literal, does not count
  `;
  assert.equal(count("oneElementLiteralOfAllocation", src), 4);
});

test("fillValueReevaluated counts fill calls whose value is re-emitted per element", () => {
  // Positives: `.fill(v)` on an allocation and on a bound array, whatever the
  // value. Negatives: the zero-argument `fill()`, and a non-`fill` method.
  const src = `
    function g() { return 1; }
    const a = new Array(3).fill(g());  // counts
    const b = new Array(2).fill(0);    // counts
    a.fill(g());                       // counts
    const c = new Array(2);
    c.fill();                          // zero-argument, does not count
    a.join(",");                       // not fill, does not count
  `;
  assert.equal(count("fillValueReevaluated", src), 3);
});
```

- [ ] **Step 2: Run them to watch them fail**

```bash
cd /workspace/tools/blast-radius && node --test 2>&1 | tail -20
```

Expected: FAIL, both matchers unknown.

- [ ] **Step 3: Add both matchers**

In `MATCHERS`:

```javascript
  // R-66: a one-element array literal whose only element is an array allocation.
  // `[Array(3)]` and `new Array(3)` are the SAME LIR node -- a text-less `Value`
  // with one `Call` child (`crates/kali_mir/src/lower.rs:108`, `:116` erase the
  // HIR distinction) -- so `resolve_array_alloc_call` reads the literal as the
  // allocation and the declarator lane answers the allocation's length.
  oneElementLiteralOfAllocation(ast) {
    const analysis = analysisOf(ast);
    return analysis
      .of("ArrayExpression")
      .filter((node) => {
        if (node.elements.length !== 1) return false;
        let only = node.elements[0];
        if (!only) return false;
        if (
          only.type === "CallExpression" &&
          only.callee.type === "MemberExpression" &&
          only.callee.property.name === "fill"
        ) {
          only = only.callee.object;
        }
        return (
          (only.type === "CallExpression" || only.type === "NewExpression") &&
          isArrayAllocation(only)
        );
      })
      .length;
  },

  // R-67: a `.fill(v)` whose value is re-emitted inside the loop body
  // (`crates/kali_codegen/src/emit/call.rs:6052` at `733cd26125`), so `v`'s side
  // effects run once per element where JavaScript evaluates it once.
  fillValueReevaluated(ast) {
    const analysis = analysisOf(ast);
    return analysis
      .of("CallExpression")
      .filter(
        (node) =>
          node.callee.type === "MemberExpression" &&
          node.callee.property.name === "fill" &&
          node.arguments.length === 1,
      )
      .length;
  },
```

- [ ] **Step 4: Run the matcher tests**

```bash
cd /workspace/tools/blast-radius && node --test 2>&1 | tail -20
```

Expected: PASS.

- [ ] **Step 5: Catalogue records, register entries, oracle pairs**

The same four files as R-64's and R-65's filings, twice over — `predicates.json`,
`kali-silent-miscompile-register.md` (a Tier 2 entry each, with **Added**,
**Verification**, **Root-cause group**, **Repro**, **Every lane**, **Severity**,
**Blast radius**, **Confidence**, a §0.2 row and §1's count paragraphs), and
`crates/kali_cli/tests/cases/oracle/tier2.toml` (header index, programs, cases).
Neither entry needs an `UPPER_BOUNDS` note in `count.mjs`: both matchers count
exactly the construct their entry names.

- `predicates.json`: `{ "id": "R-66", "kind": "countable", "matcher": "oneElementLiteralOfAllocation", "description": "a one-element array literal whose only element is an `Array`/`Uint8Array` allocation, optionally `.fill`ed -- the shape that lowers to the same LIR node as the allocation itself" }` and `{ "id": "R-67", "kind": "countable", "matcher": "fillValueReevaluated", "description": "a `.fill(v)` member call with exactly one argument, whose value codegen re-emits once per element" }`
- register entries `### R-66: A one-element array literal of an allocation IS that allocation` (**Repro** `const xs = [new Array(3)]; console.log(xs.length);` → node `1`, kali `3`) and `### R-67: `.fill(v)` evaluates `v` once per element` (**Repro** `let n = 0; function g() { n = n + 1; return 1; } const a = new Array(3).fill(g()); console.log(n);` → node `1`, kali `3`).
- oracle programs:

```toml
"r66a_module.js" = """const xs = [new Array(3)];
console.log(xs.length);
"""
"r66a_function.js" = """function main() {
  const xs = [new Array(3)];
  console.log(xs.length);
}
main();
"""
"r67a_module.js" = """let n = 0;
function g() { n = n + 1; return 1; }
const a = new Array(3).fill(g());
console.log(n);
"""
"r67a_function.js" = """function main() {
  let n = 0;
  function g() { n = n + 1; return 1; }
  const a = new Array(3).fill(g());
  console.log(n);
}
main();
"""
```

All four `verdict = "silent"`; R-66's pair flips to FAIL_CLOSED in Task 6, R-67's to FIXED in Task 7, and each rationale says so.

- [ ] **Step 6: Regenerate the counts and record the re-freeze**

```bash
cd /workspace
cargo build -p kali_cli
cd tools/blast-radius
node --test
node accepts.mjs
node count.mjs
cd /workspace
cargo run -p kali_blast_radius --example rank
```

`rank`'s stdout replaces the generated region of
`docs/superpowers/followups/blast-radius-ranking.md` between its HTML-comment
markers. Then update `FROZEN_PREDICATES_SHA256`, `FROZEN_MATCHERS_SHA256` and the
re-freeze narration in `crates/kali_blast_radius/src/manifest_tests.rs`, following
the 2026-09-11 paragraph R-63's filing added. Gate:

```bash
cargo test -p kali_blast_radius 2>&1 | tail -20
cargo test -p kali_cli --test cases -- oracle/tier2 2>&1 | tail -10
```

Expected: both green.

- [ ] **Step 7: Commit**

```bash
git add docs/superpowers/followups/kali-silent-miscompile-register.md \
        docs/superpowers/followups/blast-radius-ranking.md \
        tools/blast-radius crates/kali_blast_radius/src/manifest_tests.rs \
        crates/kali_cli/tests/cases/oracle/tier2.toml
git commit -m "docs(register): file R-66 and R-67 -- the literal collision and the re-evaluated fill value

Claude-Session: https://claude.ai/code/session_01481SYVapRePAhfEgNsWrVm"
```

---

### Task 6: `kali_types` refuses a one-element array literal of an allocation

Spec §3.1. This is the change that makes Task 8's routing sound, and it retires R-66.

**Files:**
- Modify: `crates/kali_types/src/resolve/expression.rs` (the `ArrayExpression` arm at `:1862`)
- Modify: `crates/kali_types/src/resolve/expression_tests.rs`
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (R-66 retirement), `crates/kali_cli/tests/cases/oracle/tier2.toml` (R-66 pair → `fail_closed`)

**Interfaces:**
- Consumes: R-66's pins from Task 2 and its oracle pair from Task 5.
- Produces: the refusal message whose needle Task 2's `${ALLOC_IN_LITERAL}` constant matches: `"a one-element array literal holding an array allocation is unavailable in the current direct-runtime path: it lowers to the same node as the allocation itself, so codegen cannot tell them apart (fail-closed)"`.

- [ ] **Step 1: Write the failing unit test**

Append to `crates/kali_types/src/resolve/expression_tests.rs`:

```rust
#[test]
fn a_one_element_array_literal_of_an_allocation_is_refused() {
    // `[new Array(3)]` lowers to the same LIR node as `new Array(3)` itself
    // (`crates/kali_mir/src/lower.rs:108`, `:116` erase the HIR distinction), so
    // codegen reads the literal as the allocation and answers its length.
    let statements = vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "const".to_string(),
        declarations: vec![VariableDeclarator {
            id: "xs".to_string(),
            init: Some(Expression::ArrayExpression(Box::new(
                kali_ast::ArrayExpression {
                    elements: vec![Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::NewExpression(Box::new(kali_ast::NewExpression {
                            callee: Expression::CallExpression(Box::new(CallExpression {
                                callee: Expression::Identifier("Array".to_string()),
                                args: vec![Expression::Literal(LiteralValue::Number(3.0))],
                            })),
                            args: vec![],
                        })),
                    ))],
                },
            ))),
        }],
    })];

    let result = assert_resolution!(statements, diagnostics: 1);
    assert!(
        result.diagnostics[0]
            .message
            .contains("lowers to the same node as the allocation itself"),
        "unexpected message: {}",
        result.diagnostics[0].message
    );
}

#[test]
fn a_two_element_array_literal_of_allocations_is_admitted() {
    // Only the ONE-element literal collides: a two-child text-less `Value` is not
    // a shape `resolve_array_alloc_call` accepts.
    let allocation = || {
        Expression::NewExpression(Box::new(kali_ast::NewExpression {
            callee: Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::Identifier("Array".to_string()),
                args: vec![Expression::Literal(LiteralValue::Number(3.0))],
            })),
            args: vec![],
        }))
    };
    let statements = vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "const".to_string(),
        declarations: vec![VariableDeclarator {
            id: "xs".to_string(),
            init: Some(Expression::ArrayExpression(Box::new(
                kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(allocation())),
                        Some(kali_ast::ExpressionOrSpread::Expression(allocation())),
                    ],
                },
            ))),
        }],
    })];

    assert_resolution!(statements, diagnostics: 0);
}
```

- [ ] **Step 2: Run them to watch the first fail**

```bash
cd /workspace
cargo test -p kali_types a_one_element_array_literal_of_an_allocation_is_refused 2>&1 | tail -20
```

Expected: FAIL — `assertion `left == right` failed: unexpected diagnostics: []`, 0 where 1 was expected.

- [ ] **Step 3: Add the refusal**

In `crates/kali_types/src/resolve/expression.rs`, at the top of the `Expression::ArrayExpression(ArrayExpression { elements })` arm (`:1862`), before the element walk:

```rust
                // `[Array(n)]`, `[new Array(n)]` and `[Array(n).fill(v)]` lower to
                // the SAME LIR node as `new Array(n)` itself: a text-less `Value`
                // with one `Call` child. `kali_mir` erases the HIR `NewExpr` /
                // `ArrayExpr` distinction (`crates/kali_mir/src/lower.rs:108`,
                // `:116`), so no codegen shape check can tell them apart, and
                // `resolve_array_alloc_call` (`emit/call.rs:5464`) reads the
                // literal AS the allocation -- `[new Array(3)].length` answered
                // `3` where node says `1`. Refusing the collision here is what
                // lets codegen route an allocation in value position (spec
                // docs/superpowers/specs/2026-09-11-inline-allocation-value-position-design.md
                // sections 3.1-3.2). Only the one-element form collides; a
                // two-child literal is a shape the recognizer never accepts.
                if let [Some(ExpressionOrSpread::Expression(only))] = elements.as_slice() {
                    if expression_is_array_allocation(only) {
                        self.diagnostics.push(Diagnostic::error(
                            e5::FEATURE_UNAVAILABLE as u32,
                            "a one-element array literal holding an array allocation is \
                             unavailable in the current direct-runtime path: it lowers to the \
                             same node as the allocation itself, so codegen cannot tell them \
                             apart (fail-closed)"
                                .to_string(),
                        ));
                    }
                }
```

And, as a free function at the end of the same file:

```rust
/// `Array(n)` / `new Array(n)` / `Uint8Array(n)` / `new Uint8Array(n)`, optionally
/// `.fill(v)`ed, parenthesized or awaited — the shapes
/// `FunctionEmitter::resolve_array_alloc_call` and `array_fill_call_parts`
/// (`crates/kali_codegen/src/emit/call.rs:5464`, `:5649`) accept after unwrapping
/// transparent value wrappers.
///
/// This MUST stay in lockstep with `FunctionEmitter::is_array_like_constructor`
/// (`emit/call.rs:5510`), the same way `declarator_init_is_array_alloc`
/// (`crates/kali_codegen/src/lower.rs:6535`) records its own lockstep with it: a
/// spelling codegen treats as an allocation but this function does not is a
/// spelling whose one-element literal reaches codegen and is miscompiled.
fn expression_is_array_allocation(expr: &Expression) -> bool {
    match expr {
        Expression::ParenthesizedExpression(paren) => {
            expression_is_array_allocation(&paren.expression)
        }
        Expression::AwaitExpression(await_expr) => {
            expression_is_array_allocation(&await_expr.argument)
        }
        Expression::NewExpression(new_expr) => match &new_expr.callee {
            Expression::Identifier(name) => {
                (name == "Array" || name == "Uint8Array") && new_expr.args.len() <= 1
            }
            callee => new_expr.args.is_empty() && expression_is_array_allocation(callee),
        },
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(name) => {
                (name == "Array" || name == "Uint8Array") && call.args.len() <= 1
            }
            Expression::MemberExpression(member) => {
                member.property.as_deref() == Some("fill")
                    && call.args.len() == 1
                    && expression_is_array_allocation(&member.object)
            }
            _ => false,
        },
        _ => false,
    }
}
```

If `AwaitExpression`'s field is not `argument`, read `crates/kali_ast/src/expression.rs:322` and use the field it declares; do not guess.

- [ ] **Step 4: Run the unit tests**

```bash
cargo test -p kali_types resolve::expression_tests 2>&1 | tail -20
```

Expected: PASS, both new tests.

- [ ] **Step 5: Watch R-66's pins go green**

```bash
cargo test -p kali_cli --test cases -- runtime/inline_allocation_value_position 2>&1 | tail -20
```

Expected: the five `_one_element_literal_*_refuses` cases now pass, `a_two_element_literal_of_allocations_is_correct` still passes, and the rest of the expected-red set is unchanged.

- [ ] **Step 6: Retire R-66**

- Flip both R-66 oracle cases to `verdict = "fail_closed"` and append the `RE-MEASURED 2026-09-11 … with the fix: FAIL_CLOSED (was SILENT)` paragraph each rationale promised, recording the exact stderr.
- In the register, mark `### R-66` **CLOSED 2026-09-11 (FAIL_CLOSED)**, add the retirement bullet, flip its §0.2 row, and update §1's count paragraphs — the footprint R-63's retirement used.
- Then regenerate:

```bash
cd /workspace
cargo build -p kali_cli
cd tools/blast-radius
node --test
node accepts.mjs
node count.mjs
cd /workspace
cargo run -p kali_blast_radius --example rank
```

`rank`'s stdout replaces the generated region of
`docs/superpowers/followups/blast-radius-ranking.md` between its HTML-comment
markers. Then update `FROZEN_PREDICATES_SHA256`, `FROZEN_MATCHERS_SHA256` and the
re-freeze narration in `crates/kali_blast_radius/src/manifest_tests.rs`, following
the 2026-09-11 paragraph R-63's filing added. Gate:

```bash
cargo test -p kali_blast_radius 2>&1 | tail -20
cargo test -p kali_cli --test cases -- oracle/tier2 2>&1 | tail -10
```

Expected: both green.

- [ ] **Step 7: Run the gate**

```bash
bash scripts/test-gate.sh 2>&1 | tail -40
```

Expected: `GATE FAILED`, and the failing set is `/tmp/expected-red.txt` minus the five R-66 pins. **Any other difference stops the task.**

- [ ] **Step 8: Commit**

```bash
git add crates/kali_types crates/kali_cli/tests/cases/oracle/tier2.toml \
        docs/superpowers/followups tools/blast-radius crates/kali_blast_radius
git commit -m "fix(types): a one-element array literal holding an allocation refuses

The literal lowers to the same LIR node as the allocation, so codegen read
[new Array(3)].length as 3 where node says 1. Retires R-66 as FAIL_CLOSED.

Claude-Session: https://claude.ai/code/session_01481SYVapRePAhfEgNsWrVm"
```

---

### Task 7: Allocation and fill own their scratch slots, and fill evaluates its value once

Spec §3.3. Behaviour-neutral for every existing lane except the fill value's re-evaluation, which it fixes. Retires R-67.

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs:1442-1556`
- Modify: `crates/kali_codegen/src/emit/call.rs:5567` (`emit_array_allocation_with_len`), `:5991` (`emit_array_fill`)
- Modify: `crates/kali_codegen/src/emit/call_tests/alloc_helper.rs`
- Modify: the register and `oracle/tier2.toml` (R-67 retirement)

**Interfaces:**
- Consumes: nothing from earlier tasks' code.
- Produces: slot discipline Task 8 relies on — `handle = locals.len() + 2`, `count = locals.len() + 3`, `value = locals.len() + 4`, and the invariant that neither function writes a slot until every child has been emitted.

- [ ] **Step 1: Write the failing unit test for the reservation**

Append to `crates/kali_codegen/src/emit/call_tests/alloc_helper.rs`:

```rust
#[test]
fn every_lowered_function_reserves_five_trailing_i64_scratch_locals() {
    // Two general-purpose slots as before, plus three owned by
    // `emit_array_allocation_with_len` and `emit_array_fill`: the handle, the
    // count and the fill value. The dedicated three exist so that an allocation
    // nested inside any other emitter's scratch-holding window cannot overwrite
    // it (spec section 3.3).
    let program = parse_and_lower_lir("console.log(1);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let text = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    let index = exported_function_index(&text, "_start");
    let decl_needle = format!("(func (;{index};) (type ");
    let start_body = text
        .lines()
        .skip_while(|line| !line.trim_start().starts_with(&decl_needle))
        .take(4)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        start_body.contains("(local i64 i64 i64 i64 i64)"),
        "_start should declare five trailing i64 scratch locals, got:\n{start_body}"
    );
}
```

- [ ] **Step 2: Run it to watch it fail**

```bash
cargo test -p kali_codegen every_lowered_function_reserves_five_trailing 2>&1 | tail -20
```

Expected: FAIL — the body declares `(local i64 i64)`.

- [ ] **Step 3: Grow the reservation**

In `crates/kali_codegen/src/lower.rs`, replace the two-slot append at `:1553-1556`:

```rust
            match local_decls.last_mut() {
                Some((count, ValType::I64)) => *count += 5,
                _ => local_decls.push((5, ValType::I64)),
            }
```

and rewrite the comment at `:1442-1455` so it names all five: `self.locals.len()` general-purpose, `+ 1` the second general-purpose slot, and `+ 2` / `+ 3` / `+ 4` owned exclusively by `emit_array_allocation_with_len` and `emit_array_fill` (handle, count, fill value). State the invariant there too: those two functions emit every child before writing a dedicated slot, so an allocation nested inside another emitter's scratch window cannot clobber it.

- [ ] **Step 4: Move the allocator onto the dedicated slots**

In `crates/kali_codegen/src/emit/call.rs`, in `emit_array_allocation_with_len` (`:5567`), replace the two `let` bindings at its top:

```rust
        // Dedicated slots (see `lower.rs`'s reservation comment): never the two
        // general-purpose scratch slots, so an allocation nested inside another
        // emitter's scratch-holding window cannot overwrite it. Both are written
        // only AFTER the size argument has been emitted.
        let scratch = self.locals.len() as u32 + 2;
        let size_scratch = scratch + 1;
```

The rest of the function is unchanged: it already emits the size before its first `LocalSet`.

- [ ] **Step 5: Rewrite the fill loop to evaluate its value once**

In `emit_array_fill` (`:5991`), replace everything from the two `let` bindings down to the `Block` instruction with:

```rust
        // Dedicated slots (see `lower.rs`'s reservation comment). Both the
        // receiver and the value are emitted BEFORE any of them is written, so a
        // nested allocation or fill in either one completes first and cannot
        // clobber this call's handle, counter or value.
        let base_local = self.locals.len() as u32 + 2;
        let counter_local = base_local + 1;
        let value_local = base_local + 2;

        // Receiver first, then value: JavaScript's evaluation order. A fresh
        // `new Array(n)` receiver allocates (writing its length header); an
        // existing binding just loads its handle. Both leave an i64 on the stack.
        if let Some(size_arg) = self.resolve_array_alloc_call(receiver) {
            let allocated = self.emit_array_allocation(function, size_arg);
            if !allocated.produced {
                function.instruction(&Instruction::I64Const(0));
            }
        } else {
            let base = self.emit_node(function, receiver, true);
            if !base.produced {
                function.instruction(&Instruction::I64Const(0));
            }
        }

        // `.fill(v)` evaluates `v` ONCE in JavaScript. Emitting it inside the loop
        // ran its side effects once per element (register entry R-67:
        // `new Array(3).fill(g())` called `g` three times where node calls it
        // once), so it is evaluated here and held in a slot. An f64 element repr
        // stores the promoted float's BITS, restored at each store below, because
        // the reserved slots are i64.
        let elem_is_float = self.array_elem_repr(binding_name) == kali_common::Repr::F64;
        let value_is_float = self.is_float_valued(value);
        let produced = self.emit_node(function, value, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        if elem_is_float {
            if !produced.produced || !value_is_float {
                function.instruction(&Instruction::F64ConvertI64S);
            }
            function.instruction(&Instruction::I64ReinterpretF64);
        }

        // The stack holds [handle, value]; pop in that order.
        function.instruction(&Instruction::LocalSet(value_local));
        function.instruction(&Instruction::LocalSet(base_local));

        // i = 0
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(counter_local));
```

and replace the value emission inside the loop body (the `let produced = self.emit_node(function, value, true);` block and the promotion that followed it, `:6052-6059`) with a load:

```rust
        function.instruction(&Instruction::LocalGet(value_local));
        if elem_is_float {
            function.instruction(&Instruction::F64ReinterpretI64);
        }
```

The two stores that follow (`F64Store` / `I64Store` at offset 8) are unchanged, as are the counter increment, the `Br`/`End`s and the trailing `LocalGet(base_local)`.

- [ ] **Step 6: Run the unit tests and the R-67 pins**

```bash
cargo test -p kali_codegen 2>&1 | tail -20
cargo test -p kali_cli --test cases -- runtime/inline_allocation_value_position 2>&1 | tail -20
```

Expected: `kali_codegen` green including the new reservation test; the three `_fill_value_*_once*` cases now pass. The clobber cases still fail — they need Task 8.

- [ ] **Step 7: Retire R-67**

Flip R-67's oracle pair to `verdict = "fixed"`, appending to each rationale the
`RE-MEASURED 2026-09-11 … with the fix: FIXED (was SILENT)` paragraph it
promised, with the exact stdout. Mark `### R-67` **CLOSED 2026-09-11 (FIXED)**,
add its retirement bullet, flip its §0.2 row, and update §1's count paragraphs.
Then regenerate:

```bash
cd /workspace
cargo build -p kali_cli
cd tools/blast-radius
node --test
node accepts.mjs
node count.mjs
cd /workspace
cargo run -p kali_blast_radius --example rank
```

`rank`'s stdout replaces the generated region of
`docs/superpowers/followups/blast-radius-ranking.md` between its HTML-comment
markers. Then update `FROZEN_PREDICATES_SHA256`, `FROZEN_MATCHERS_SHA256` and the
re-freeze narration in `crates/kali_blast_radius/src/manifest_tests.rs`, following
the 2026-09-11 paragraph R-63's filing added. Gate:

```bash
cargo test -p kali_blast_radius 2>&1 | tail -20
cargo test -p kali_cli --test cases -- oracle/tier2 2>&1 | tail -10
```

Expected: both green.

- [ ] **Step 8: Run the gate — this is spec §4.1's second stop gate**

```bash
bash scripts/test-gate.sh 2>&1 | tail -60
```

Expected: `GATE FAILED` with exactly `/tmp/expected-red.txt` minus Task 6's five R-66 pins and minus these three R-67 pins. **Any other test moving — including one moved by the reservation alone — stops the plan and goes back to the human partner**, per spec §4.1.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/cases/oracle/tier2.toml \
        docs/superpowers/followups tools/blast-radius crates/kali_blast_radius
git commit -m "fix(codegen): allocation and fill own their scratch slots, and fill evaluates once

Three dedicated i64 slots, written only after every child is emitted, so a
nested allocation cannot clobber an enclosing emitter's scratch. .fill(v) now
evaluates v once, as JavaScript does. Retires R-67 as FIXED.

Claude-Session: https://claude.ai/code/session_01481SYVapRePAhfEgNsWrVm"
```

---

### Task 8: Allocations in value position allocate

Spec §3.2. Retires R-64.

**Files:**
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (before the `emit_aggregate_literal` call at `:2181`)
- Modify: `crates/kali_codegen/src/emit/call.rs:100-102` (after the computed-member deny)
- Modify: the register and `oracle/tier2.toml` (R-64 retirement)

**Interfaces:**
- Consumes: Task 6's refusal (which guarantees a text-less wrapper around an allocation call is a `new`, a parenthesis or an `await`, never a literal) and Task 7's slot discipline.
- Produces: nothing later tasks call; Task 9's guard reads `resolve_array_alloc_call` / `resolve_array_fill_call`, which already exist.

- [ ] **Step 1: Confirm the pins that must move are still red**

```bash
cd /workspace
cargo test -p kali_cli --test cases -- runtime/inline_allocation_value_position 2>&1 | tail -30
```

Expected: the 19 `_computes` cases and the four clobber cases fail; the controls pass.

- [ ] **Step 2: Route allocations in `emit_value`**

In `crates/kali_codegen/src/emit/control_flow.rs`, immediately before `return self.emit_aggregate_literal(function, node, want_value);` (`:2181`):

```rust
            // An `Array`/`Uint8Array` allocation reaching the generic value path
            // is OUTSIDE the three lanes that materialize one -- a declarator
            // init (`:1690`, `:1711`), an assignment right-hand side
            // (`emit/literal.rs:1043`) and a `.fill` receiver
            // (`emit/call.rs:6004`), each of which intercepts before here. The
            // aggregate placeholder below would drop it and push `0`, which the
            // callee then reads as a length header at address zero (register
            // entry R-64). Allocate instead, so every position -- argument,
            // return, property, element, ternary arm -- passes a real handle.
            //
            // Sound only because `kali_types` refuses the one-element array
            // literal of an allocation (`resolve/expression.rs`): `[Array(3)]`
            // is otherwise this exact node.
            if let Some(size_arg) = self.resolve_array_alloc_call(id) {
                return self.emit_array_allocation(function, size_arg);
            }
            // `new Array(n).fill(v)` arrives as the hoisted-`new` wrapper around
            // the `.fill` call (the parser's `new` precedence). Pass through to
            // the fill arm (`emit/call.rs:1037`), which allocates the receiver and
            // runs the init loop, rather than dropping the whole chain.
            if node.children.len() == 1
                && self.resolve_array_fill_call(node.children[0]).is_some()
            {
                return self.emit_node(function, node.children[0], want_value);
            }
```

- [ ] **Step 3: Route the bare `Array(n)` call in `emit_call`**

In `crates/kali_codegen/src/emit/call.rs`, after the computed-member deny (`:100-102`):

```rust
        // A bare `Array(n)` / `Uint8Array(n)` call (no `new`) in value position:
        // the declarator, assignment and `.fill`-receiver lanes intercept their
        // own copies before `emit_call`, so anything arriving here is a value
        // whose allocation nobody has made. Without this arm it falls through to
        // the unresolved-callee placeholder and answers `0` (register entry R-64).
        if let Some(size_arg) = self.resolve_array_alloc_call(id) {
            return self.emit_array_allocation(function, size_arg);
        }
```

- [ ] **Step 4: Run the pins — spec §4.1's third stop gate**

```bash
cargo test -p kali_cli --test cases -- runtime/inline_allocation_value_position 2>&1 | tail -30
```

Expected: every `_computes` case and all four clobber cases pass; every control still passes; only the R-65 refusal cases (Task 9) remain red. **Any silent result — a case that exits 0 with a wrong stdout — stops implementation and goes back to the human partner**, per spec §4.1.

- [ ] **Step 5: Retire R-64**

Flip R-64's oracle pair to `verdict = "fixed"`, appending to each rationale the
`RE-MEASURED 2026-09-11 … with the fix: FIXED (was SILENT)` paragraph it
promised, with the exact stdout. Mark `### R-64` **CLOSED 2026-09-11 (FIXED)**,
add its retirement bullet, flip its §0.2 row, and update §1's count paragraphs.
Then regenerate:

```bash
cd /workspace
cargo build -p kali_cli
cd tools/blast-radius
node --test
node accepts.mjs
node count.mjs
cd /workspace
cargo run -p kali_blast_radius --example rank
```

`rank`'s stdout replaces the generated region of
`docs/superpowers/followups/blast-radius-ranking.md` between its HTML-comment
markers. Then update `FROZEN_PREDICATES_SHA256`, `FROZEN_MATCHERS_SHA256` and the
re-freeze narration in `crates/kali_blast_radius/src/manifest_tests.rs`, following
the 2026-09-11 paragraph R-63's filing added. Gate:

```bash
cargo test -p kali_blast_radius 2>&1 | tail -20
cargo test -p kali_cli --test cases -- oracle/tier2 2>&1 | tail -10
```

Expected: both green.

- [ ] **Step 6: Run the gate**

```bash
bash scripts/test-gate.sh 2>&1 | tail -60
```

Expected: `GATE FAILED` with only the R-65 refusal pins from Task 2 still failing.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/cases/oracle/tier2.toml \
        docs/superpowers/followups tools/blast-radius crates/kali_blast_radius
git commit -m "fix(codegen): an allocation in value position allocates

Argument, return, property, element and ternary-arm positions now pass a real
array handle instead of the aggregate placeholder's zero. Retires R-64 as FIXED.

Claude-Session: https://claude.ai/code/session_01481SYVapRePAhfEgNsWrVm"
```

---

### Task 9: The argument guard refuses every fold-lane array, and the 81 pins are re-pinned

Spec §3.4 and §4.3. Retires R-65.

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs:3573-3615`
- Modify: the five `crates/kali_cli/tests/cases/browser/*.toml` files (or their generators — Task 1's recorded route decides), and `crates/kali_cli/tests/cases/soundness/abort.toml:890`
- Modify: the register and `oracle/tier2.toml` (R-65 retirement)

**Interfaces:**
- Consumes: Task 1's route; Task 8's routing (so an allocation argument is never caught by this guard).
- Produces: the final refusal message, whose prefix every existing needle matches.

- [ ] **Step 1: Confirm the R-65 pins are the only red left**

```bash
cargo test -p kali_cli --test cases -- runtime/inline_allocation_value_position 2>&1 | tail -20
```

Expected: only the eight R-65 refusal cases fail.

- [ ] **Step 2: Widen the guard**

In `crates/kali_codegen/src/emit/call.rs`, replace the `fold_lane_array` binding (`:3591-3601`) and rewrite the stale comment above it (`:3574-3590`):

```rust
            // An array kali cannot pass as a runtime handle: the callee would
            // read zero placeholders where the elements should be, silently
            // (`g([1, 2]) { return items[0] }` → 0, node says 1). That is every
            // fold-lane array literal, whatever its elements -- the earlier
            // all-`Literal` condition let `f([k])`, `f([1 + 1])` and
            // `[...Object.values(o)]` through to the placeholder (register entry
            // R-65) -- and every constructed value, because `new C()` and
            // `[C()]` are the SAME text-less LIR node and no shape check can
            // separate them.
            //
            // An ALLOCATION argument is exempt: `emit_value` and `emit_call` now
            // allocate one in value position, so it arrives as a real handle.
            // REJECT-DON'T-MISCOMPILE.
            if resolved.is_some() {
                let arg_is_allocation = self.resolve_array_alloc_call(*arg).is_some()
                    || self.resolve_array_fill_call(*arg).is_some();
                let fold_lane_array = !arg_is_allocation
                    && self
                        .resolve_literal_aggregate(*arg)
                        .map(|id| self.node(id).clone())
                        .is_some_and(|aggregate| {
                            self.is_array_literal(&aggregate) && !aggregate.children.is_empty()
                        });
```

and extend the message, keeping every existing word as its prefix:

```rust
                        format!(
                            "passing an array literal to function '{callee_name}' is unavailable in the current direct-runtime path (the callee would read zero placeholders, not the elements); allocate with `new Array(n)` and assign elements instead. A constructed value (`new C()`) is refused here too: it lowers to the same node as a one-element array literal"
                        ),
```

- [ ] **Step 3: Run the R-65 pins and the two needle pins**

```bash
cargo test -p kali_cli --test cases -- runtime/inline_allocation_value_position 2>&1 | tail -20
cargo test -p kali_cli --test runtime_smoke array_literal_arguments_benchmark_is_rejected_fail_closed 2>&1 | tail -10
```

Expected: all eight R-65 cases pass; the benchmark needle test still passes on the unchanged prefix.

- [ ] **Step 4: Re-pin the five browser families by Task 1's route**

Each affected case's build step moves from `exit = "success"` to `exit = "failure"` with an `E5506` needle, and every step after a failed build goes; each harness case's `Uncaught Error` needle becomes the E5506 message. In generator form, edit the `asserts=` dicts inside each target's own function — `gen_batch7b.py`'s `@target("object_keys_entries_spread_bundle")` (`:1196`), `@target("object_keys_entries_spread_harness")` (`:1405`), `@target("object_values_spread_bundle")` (`:2943`) and `@target("object_values_spread_harness")` (`:3046`), and `gen_batch6a.py`'s `@target("object_entries_iteration")` (`:2005`) — locating them by searching within the function rather than by line number, then re-run the generators exactly as in Task 1 step 2 and diff.

Each edited case's rationale gains one sentence: at `733cd26125` this program built and then threw in the harness because the callee read zeros (`0,0,0` where node v26.8.2 prints `2,3,2`); it now refuses at build time.

- [ ] **Step 5: Re-pin the abort case**

In `crates/kali_cli/tests/cases/soundness/abort.toml`, the case at `:890`:

```toml
args = ["run", "abort_handle_inline_new_in_arg_position.js"]
exit = "failure"
stderr_contains = ["E5506", "lowers to the same node as a one-element array literal"]
```

with a rationale recording that node prints `ok` for this program and kali now refuses it: `new AbortController()` is the same LIR node as `[AbortController()]`, and the human partner ruled the wide guard over leaving `f([g()])` and `f(new C())` silent (spec §2.5).

- [ ] **Step 6: Retire R-65**

Flip R-65's oracle pair to `verdict = "fail_closed"`, appending to each rationale
the `RE-MEASURED 2026-09-11 … with the fix: FAIL_CLOSED (was SILENT)` paragraph
it promised, with the exact stderr. Mark `### R-65` **CLOSED 2026-09-11
(FAIL_CLOSED)**, add its retirement bullet, flip its §0.2 row, and update §1's
count paragraphs. Then regenerate:

```bash
cd /workspace
cargo build -p kali_cli
cd tools/blast-radius
node --test
node accepts.mjs
node count.mjs
cd /workspace
cargo run -p kali_blast_radius --example rank
```

`rank`'s stdout replaces the generated region of
`docs/superpowers/followups/blast-radius-ranking.md` between its HTML-comment
markers. Then update `FROZEN_PREDICATES_SHA256`, `FROZEN_MATCHERS_SHA256` and the
re-freeze narration in `crates/kali_blast_radius/src/manifest_tests.rs`, following
the 2026-09-11 paragraph R-63's filing added. Gate:

```bash
cargo test -p kali_blast_radius 2>&1 | tail -20
cargo test -p kali_cli --test cases -- oracle/tier2 2>&1 | tail -10
```

Expected: both green.

- [ ] **Step 7: Run both gates**

```bash
bash scripts/test-gate.sh 2>&1 | tail -40
bash scripts/test-gate.sh --gates-only 2>&1 | tail -20
```

Expected: `GATE OK: 0 failing tests`, and `MIGRATION GATES OK` — `citation_sweep.sh`, `batch5_crosscheck.py` and the migration-audit test read the browser case files this task edited.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests tools docs/superpowers/followups \
        crates/kali_blast_radius
git commit -m "fix(codegen): a fold-lane array argument refuses instead of reading zeros

Every array literal argument kali cannot pass as a handle now refuses, not only
the all-literal ones, and so does a constructed value that shares its LIR node.
81 pins move from 'builds, then throws' to a build-time refusal. Retires R-65 as
FAIL_CLOSED.

Claude-Session: https://claude.ai/code/session_01481SYVapRePAhfEgNsWrVm"
```

---

### Task 10: File what this project measured and did not fix, and close the branch out

Spec §6.2 and §6.3.

**Files:**
- Create: `docs/superpowers/followups/inline-allocation-value-position-discovered-defects.md`
- Modify: `docs/superpowers/followups/length-fails-closed-discovered-defects.md`, `docs/superpowers/followups/codegen-array-literal-predicate-is-still-negative-space.md`

**Interfaces:**
- Consumes: every measurement from Tasks 2-9.
- Produces: the document the next project's "next item" pick reads.

- [ ] **Step 1: Write the discovered-defects document**

`docs/superpowers/followups/inline-allocation-value-position-discovered-defects.md`, on the convention `length-fails-closed-discovered-defects.md` uses: a header naming the project, this spec, the oracle (`node v26.8.2`) and the measurement commit, then ranked sections, most consequential first. Every row must be re-measured at the branch's HEAD before it is written down, not copied from this plan:

1. **A returned or passed-through allocation, rebound, reads zeros** — `function mk() { const a = new Array(3).fill(4); return a; } const b = mk(); console.log(b[1]);` → kali `0`, node `4`; `function f(x) { return x; } const a = f(new Array(3).fill(1)); console.log(a[0]);` → kali `0`, node `1`. Cross-reference R-14.
2. **An allocation held by an object property reads zeros** — `const o = { arr: new Array(3).fill(6) }; console.log(o.arr[0]);` → kali `0`, node `6`, both scopes. Cross-reference R-48.
3. **`Array.from({length: 3})` passed to a function reads zeros** — a different recognizer (`is_array_from_call`), untouched by the guard.
4. **The 36 scratch-slot holders are an audit left undone** — this project removed allocation and fill as clobberers; nothing proves the other holders are safe against each other.
5. **A float `.fill` array passed to a function fails to load** — `function f(x) { return x[0]; } console.log(f(new Array(3).fill(1.5)));` and its bound form both fail `E4201`. Loud, not silent.

- [ ] **Step 2: Correct the two followups it supersedes**

- `length-fails-closed-discovered-defects.md`: a dated notice at the top recording that §1 is closed by R-64 and R-65 (naming this spec), that §2 gains the measured row `console.log((new Array(3).fill(2))[1]);` → kali `undefined`, node `2`, and that §3's `(new Array(3)).length` → `1` is unchanged, because it comes from `render_length`'s array-literal arm and never reaches `emit_value`.
- `codegen-array-literal-predicate-is-still-negative-space.md`: a dated notice recording that the LIR collision its correction describes is now broken **for allocation shapes only**, at the AST (`kali_types`), and that `[f()]` and `new f()` remain one node.

- [ ] **Step 3: Run the full gate set**

```bash
bash scripts/test-gate.sh 2>&1 | tail -20
bash scripts/test-gate.sh --gates-only 2>&1 | tail -10
cargo clippy --workspace -- -D warnings 2>&1 | tail -20
cargo fmt --all -- --check
```

Expected: `GATE OK: 0 failing tests`, `MIGRATION GATES OK`, clippy clean on the crates this branch touched (six pre-existing failures in files this branch never touched are recorded in `release-tier-allocation-identity-discovered-defects.md` §10 — if clippy reports only those, say so explicitly rather than claiming clean), and `fmt --check` silent.

- [ ] **Step 4: Diff the whole suite against the baseline**

```bash
cargo test --workspace --no-fail-fast 2>&1 | grep -E "^test result:" \
  | awk '{p+=$4; f+=$6; i+=$8} END {print "passed="p" failed="f" ignored="i}'
```

Expected: `failed=0`, `ignored=27`, and `passed` equal to the baseline's 12032 plus this branch's new cases. **Every test that changed state must be one listed in spec §4.2, §4.3 or §4.4.** Anything else goes back to the human partner under the "measured tiny" rule.

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/followups
git commit -m "docs(followups): file the lanes this project measured and did not fix

Claude-Session: https://claude.ai/code/session_01481SYVapRePAhfEgNsWrVm"
```

- [ ] **Step 6: Hand back**

Report to the human partner: the suite's before and after numbers, the list of tests that changed state with the spec section that licensed each, the four register entries and their retirement classes, and the new followup document. Do not open a pull request without being asked.
