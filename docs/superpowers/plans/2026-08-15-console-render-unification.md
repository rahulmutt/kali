# Console Render Unification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the rendering half of the register's G8 cluster by making every console lane render through one ladder, retiring R-30 and R-33 and closing R-08 residual 5.

**Architecture:** Four renderers currently disagree because each sits at a different point relative to one proof boundary — the wasmtime host holds the runtime string-handle tag but no repr, while codegen holds the repr but must prove stringness statically. We add a host import `value_to_string` that does what the host already does for console values, make it the terminal arm of `emit_as_string` **for the console sink only**, and route both console lanes through it. Afterwards the host is a sink rather than a renderer, so the two cannot drift apart again.

**Tech Stack:** Rust (workspace crates `kali_common`, `kali_codegen`, `kali_runtime`, `kali_runtime_contract`, `kali_cli`, `kali_blast_radius`), wasmtime host imports, `wasm-encoder`, `ryu-js`, TOML oracle-case fixtures, node `v26.7.0` as the differential oracle.

**Spec:** `docs/superpowers/specs/2026-08-15-console-render-unification-design.md` (committed `3c9328a4cf`, corrected `c59b2917bf`)

**Branch:** `console-render-unification-design`

## Global Constraints

- **Do-not-modify files.** `scripts/test-gate.sh`, `scripts/check-determinism.sh`, `mise.toml`, `.github/workflows/ci.yml`. Nothing in this plan touches them. Oracle cases are `.toml` under the existing `cases` binary, which the gate already runs.
- **One import means five registration points.** The wasmtime host plus **four** hand-maintained `kali:rt` JS import lists: `crates/kali_runtime_contract/src/browser/harness.rs:398` and `:964`, `crates/kali_cli/src/bin/cmd_build.rs:1722` and `:2229`. Missing any of the four JS ones is a browser-lane `LinkError`, not a wrong answer.
- **Import indices are positional constants** (`crates/kali_codegen/src/lib.rs:44-75`, currently through `22`). New imports are **appended**. Never renumber.
- **`STRING_HANDLE_TAG = 0x8000_0000_0000_0000` is the sign bit** (`crates/kali_runtime/src/host/memory.rs:291`). Every negative i64 carries it. Do not make `value_to_string` the terminal arm for any sink other than console — spec §3.1.
- **The `+` and template-literal path does not change**, terminal arm and taint alike. Spec §5.2.
- **Verdict vocabulary** in oracle cases is lowercase: `verdict = "silent"`, `"fixed"`, `"fail_closed"`. `observe = "stderr"` only where the entry renders on stderr.
- **A lane that runs nothing is a failure, not a pass.** `check-determinism.sh` has been green while executing zero tests since `2448dd8839`. Any test filter added here must be observed actually running.
- **Test commands:** `cargo test --workspace` is the gate. Narrower: `cargo test -p kali_cli --test cases`, `cargo test -p kali_blast_radius`, `cargo test -p kali_runtime`, `cargo test -p kali_codegen`.
- **Commit after every task.** Never squash tasks together.

---

## File Structure

| file | responsibility | task |
|---|---|---|
| `docs/superpowers/followups/kali-silent-miscompile-register.md` | §7 gains the `1e21` entry; §0.2 rows updated at the end | 1, 8 |
| `crates/kali_cli/tests/cases/oracle/tier4.toml` | R-30/R-32/R-33 cases: new programs in `[source]`, new/flipped `[[case]]` entries | 1, 2, 8 |
| `crates/kali_common/src/js_number.rs` | **new** — the single JS `Number::toString` formatter, shared by host and codegen | 3 |
| `crates/kali_runtime/src/host/imports_default.rs` | registers `value_to_string`; loses the `[warn] ` prefix; loses `format_js_number` | 3, 4, 7 |
| `crates/kali_runtime/src/host/io.rs` | `format_console_value` unchanged; becomes the shared core | — |
| `crates/kali_codegen/src/lib.rs` | `VALUE_TO_STRING_IMPORT_INDEX` | 4 |
| `crates/kali_codegen/src/lower.rs` | import declaration | 4 |
| `crates/kali_runtime_contract/src/browser/harness.rs` | two JS mirrors | 4 |
| `crates/kali_cli/src/bin/cmd_build.rs` | two JS mirrors | 4 |
| `crates/kali_codegen/src/emit/operators.rs` | `emit_as_string` gains `StringSink` and the terminal arm | 5 |
| `crates/kali_codegen/src/emit/call.rs` | both console lanes pass `StringSink::Console` | 5 |
| `crates/kali_codegen/src/intrinsics/host.rs` | static fold's numeric Literal arm | 6 |
| `crates/kali_cli/tests/runtime_smoke/{run,test}.rs`, `crates/kali_runtime/src/execute_tests/host_env.rs` | the four `[warn] ` assertions | 7 |
| `docs/superpowers/followups/blast-radius-ranking.md` | regenerated | 8 |

---

## Task 1: Pin the scoping probes and file the `1e21` finding

The spec's §1.3 and §3.1 measurements were taken on a debug binary dated 2026-08-14, one commit behind. Re-take them at this project's HEAD **before any code changes**, so a later "it was always like that" is a reading rather than a memory. This also fires the "measurements were stale" risk immediately if it is going to fire.

**Files:**
- Modify: `crates/kali_cli/tests/cases/oracle/tier4.toml` (`[source]` table, then `[[case]]` entries at end of file)
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (§7, after R-50)

**Interfaces:**
- Consumes: nothing.
- Produces: oracle case names `r32d_negative_integer_direct_log_module_scope` / `_in_function`; register entry `R-55`.

- [ ] **Step 1: Re-run the spec's probes at HEAD**

```bash
cd /workspace
cargo build -p kali_cli 2>&1 | tail -3
mkdir -p /tmp/g8probe && cd /tmp/g8probe
printf 'console.log(1e-7);\n' > a.js
printf 'var y = 1e-7; console.log(y);\n' > b.js
printf 'console.log(1e21);\n' > c.js
printf 'console.log("v=" + 1e21);\n' > d.js
printf 'var b = true; console.log(b);\n' > e.js
printf 'var n = -5; console.log(n); console.log(-1234567);\n' > f.js
for x in a b c d e f; do
  echo "=== $x"; echo "--kali"; /workspace/.cache/cargo-target/debug/kali run $x.js 2>&1 | head -4
  echo "--node"; node $x.js 2>&1 | head -4
done
```

Record the output verbatim into the scratchpad; it is quoted in the rationales below and in Task 8's register amendment.

Expected, from the spec: `a` → kali `0.0000001` / node `1e-7`; `b` → both `1e-7`; `c` → kali `1000000000000000000000` / node `1e+21`; `d` → kali `error[E4201]` / node `v=1e+21`; `e` → kali `1` / node `true`; `f` → both `-5` then `-1234567`.

**If any probe disagrees with the spec, stop and report.** The design's scope depends on these; a disagreement means re-scoping, not proceeding.

- [ ] **Step 2: Add the negative-integer programs to `tier4.toml`'s `[source]` table**

TOML forbids reopening `[source]` after the `[[case]]` array begins, so these keys go in the existing `[source]` table (starts at line 365), **not** at the end of the file. Add after the existing `r32c_function.js` key:

```toml
"r32d_module.js" = """var n = -5; console.log(n)
console.log(-1234567)
"""
"r32d_function.js" = """function main() {
  var n = -5; console.log(n)
  console.log(-1234567)
}
main();
"""
```

- [ ] **Step 3: Append the two cases at the end of `tier4.toml`**

```toml
[[case]]
name = "r32d_negative_integer_direct_log_module_scope"
kind = "oracle"
register_entry = "R-32"
program = "r32d_module.js"
verdict = "fixed"
rationale = """R-32, negative-integer control lane, module scope. SINK: the same single-argument direct `console.log` as `r32a`/`r32b`. This lane is NOT in the register's R-32 body; it is added by the console-render-unification project (`docs/superpowers/specs/2026-08-15-console-render-unification-design.md` §3.1) as a CONTROL, and it is filed under R-32 because it pins a boundary of the same direct-log number sink that entry owns.

WHY IT EXISTS. `STRING_HANDLE_TAG` is `0x8000_0000_0000_0000` (`crates/kali_runtime/src/host/memory.rs:291`) -- the sign bit. Every negative i64 carries it, so `format_console_value` attempts a string-handle decode on every negative integer it renders. It survives only because the decoded offset/length fail `read_guest_bytes`' bounds check and the integer fallback runs. That is correctness by bounds-check failure, and this case is what pins it: if a future representation change makes some negative integer's bit pattern decode to a valid in-bounds range, this case goes red instead of a program silently printing garbage bytes.

Expected verdict FIXED: both engines exit 0 with identical stdout.

MEASURED at this project's HEAD against node v26.7.0: FIXED. kali prints `-5` then `-1234567`; node prints the same.

WHAT IT DOES NOT ESTABLISH. It measures two negative integers, not the property that no negative integer decodes in-bounds -- which is not testable by example and is recorded as an open hazard in the spec's §3.1 and §8 rather than asserted here."""

[[case]]
name = "r32d_negative_integer_direct_log_in_function"
kind = "oracle"
register_entry = "R-32"
program = "r32d_function.js"
verdict = "fixed"
rationale = """R-32, negative-integer control lane, in-function scope. The module-scope program verbatim inside `function main() { ... }` with a trailing `main();` -- the wrapper is the only addition. See the module-scope case for why this lane exists.

Expected verdict FIXED, as for the module scope; the expected class assumes the scopes agree so that a disagreement goes red.

MEASURED at this project's HEAD against node v26.7.0: FIXED. Byte-identical to the module scope."""
```

- [ ] **Step 4: Run the case suite and the §0.2 gate**

Run: `cargo test -p kali_cli --test cases 2>&1 | tail -20`
Expected: PASS, with a case count two higher than before.

Run: `cargo test -p kali_blast_radius 2>&1 | tail -20`
Expected: PASS. `every_zero_two_row_is_the_class_set_its_live_cases_assert` compares *class sets* per entry, and R-32 already has `fixed` in its set from `r32b`/`r32c`, so adding another `fixed` lane does not change the set and §0.2 needs no edit yet.

If that gate goes red, the set-vs-multiset assumption is wrong for this file — stop and report rather than editing §0.2 to match.

- [ ] **Step 5: File the `1e21` finding in the register's §7**

§7 is "Fail-loudly-but-wrong defects (not silent — recorded for completeness)", R-50's home. An entry here carries no §0.2 row, so it needs no oracle case and cannot trip the gate. Insert after R-50's entry:

```markdown
### R-55: A numeric literal at or past `1e21` never reaches the JS number formatter — expanded digits in every sink, invalid wasm in concat

- **Added**: 2026-08-15, by the console-render-unification project
  (`docs/superpowers/specs/2026-08-15-console-render-unification-design.md` §1.3),
  found while scoping R-32. **It is not R-32**, and the two must not be merged:
  R-32 is a per-sink rendering divergence and closes when the sinks agree; this
  one diverges in *every* sink, including the ones R-32 records as correct.
- **Verification**: measured at this project's HEAD against `node v26.7.0`, with
  the just-inside control that pins the boundary.
- **Repro**, three lanes:
  ```js
  console.log(1e21);            // kali 1000000000000000000000, node 1e+21
  var x = 1e21; console.log(x); // kali 1000000000000000000000, node 1e+21
  console.log("v=" + 1e21);     // kali error[E4201], node v=1e+21
  ```
- **Why it is not a rendering defect.** `format_js_number`
  (`crates/kali_runtime/src/host/imports_default.rs`) is `ryu_js`, which
  implements ECMAScript `Number::toString` and therefore has **both** thresholds.
  `1e-7` renders correctly through the binding and concat lanes, which prove the
  formatter works. `1e21` renders wrongly through *every* lane, which proves the
  value never reaches the formatter at all. The defect is upstream, in how the
  literal is classified and emitted.
- **The concat lane is the loud one.** `"v=" + 1e21` fails to produce valid wasm
  (`error[E4201]: failed to load WASM module: failed to compile`), so this entry
  is fail-loudly-but-wrong, not silent — which is why it is filed here and not in
  §2, and why it carries no §0.2 row and no verdict class.
- **Boundary**: `1e20` is correct in every lane. `1e21` is the first wrong one,
  which is exactly the ECMAScript exponential threshold, so the classification is
  keyed on the right constant and applied in the wrong place.
- **Not fixed by the console-render-unification project**, which states so in its
  §8. That project makes the sinks agree; this defect is upstream of all of them.
- **Severity**: not silent. The direct-log lane is a wrong answer at exit 0; the
  concat lane is a hard compile failure on valid JavaScript.
```

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/tests/cases/oracle/tier4.toml docs/superpowers/followups/kali-silent-miscompile-register.md
git commit -m "test(console): pin the negative-integer decode control, and file R-55

The spec's scoping probes were taken on a binary one commit behind HEAD.
Re-taken here at HEAD before any code changes, so the baseline this project
argues from is a reading rather than a memory.

r32d pins the hazard behind the design's central choice: STRING_HANDLE_TAG
is the sign bit, so every negative integer attempts a handle decode and
survives only because the bounds check fails. That is why value_to_string
is the terminal arm for the console sink alone and not unconditionally.

R-55 files the 1e21 half of R-32 as its own entry under §7, where
fail-loudly-but-wrong defects live. It is not a rendering defect: ryu_js
has both thresholds and 1e-7 proves the formatter works, so a literal that
renders wrongly in EVERY lane never reaches the formatter at all. §7 also
means no §0.2 row and no oracle case, so filing it cannot trip the
§0.2/oracle agreement gate."
```

---

## Task 2: The taint-exemption regression pair

Spec §5.1 and §5.1.1. These are written **before** the unification so they can actually catch the regression they exist for. Without Step 1's case, the unification can silently trade a working lane for a fail-closed one and every other test still passes.

**Files:**
- Modify: `crates/kali_cli/tests/cases/oracle/tier4.toml`

**Interfaces:**
- Consumes: Task 1's `[source]` table conventions.
- Produces: case names `r30e_string_result_single_arg_*` (must stay `fixed` forever) and `r30f_string_result_multi_arg_*` (flips `fail_closed` → `fixed` in Task 5).

- [ ] **Step 1: Confirm the current behavior of both lanes**

```bash
cd /tmp/g8probe
printf 'const s = String(42n);\nconsole.log(s);\n' > g.js
printf 'const s = String(42n);\nconsole.log("x", s);\n' > h.js
for x in g h; do echo "=== $x"; echo "--kali"; /workspace/.cache/cargo-target/debug/kali run $x.js 2>&1 | head -4; echo "--node"; node $x.js 2>&1 | head -4; done
```

Expected from the spec: `g` prints correctly on both (the taint exemption). `h` fails closed with `E5506` on kali (the taint firing at the multi-arg console sink).

**If `h` does not fail closed, the widening in §5.1.1 does not exist.** Stop and report — the design's §5.1.1 would then be describing a lane that already works, and Task 5 gets simpler rather than riskier.

- [ ] **Step 2: Add both programs to `tier4.toml`'s `[source]` table**

Again mid-file, in the existing `[source]` table, after the `r32d` keys from Task 1:

```toml
"r30e_module.js" = """const s = String(42n);
console.log(s);
"""
"r30e_function.js" = """function main() {
  const s = String(42n);
  console.log(s);
}
main();
"""
"r30f_module.js" = """const s = String(42n);
console.log("x", s);
"""
"r30f_function.js" = """function main() {
  const s = String(42n);
  console.log("x", s);
}
main();
"""
```

- [ ] **Step 3: Append the four cases**

```toml
[[case]]
name = "r30e_string_result_single_arg_direct_log_module_scope"
kind = "oracle"
register_entry = "R-30"
program = "r30e_module.js"
verdict = "fixed"
rationale = """R-30, `String()`-result single-argument lane, module scope. This lane is NOT in the register's R-30 body; it is added by the console-render-unification project as the REGRESSION GUARD for its central risk (spec §5.1), and filed under R-30 because it constrains the same single-argument direct-log sink that entry owns.

WHAT IT GUARDS. `emit_console_argument` (`crates/kali_codegen/src/emit/call.rs:23`) is deliberately EXEMPT from `string_result_render_taint`, and its own comment says why: the single-argument lane hands the host a raw tagged i64 and lets the host decode the string-handle tag, so a `String()`-result binding prints correctly even though codegen cannot prove it is a string. The unification routes this lane through `emit_as_string`, which IS tainted. Doing that without preserving the exemption converts this working program into a fail-closed `E5506` -- a regression the register would score as a new entry.

Expected verdict FIXED, and it must remain FIXED across the whole project. If this case ever reads `fail_closed`, the unification took the exemption away and the change must be reverted, not accommodated.

MEASURED at this project's HEAD against node v26.7.0: FIXED. Both engines exit 0 with identical stdout.

WHAT IT DOES NOT ESTABLISH. It measures one `String()`-result shape at one sink. The taint's provenance analysis is a whole-program fixpoint over bindings, reassignments and returns; this case samples it at a single point and asserts nothing about the rest."""

[[case]]
name = "r30e_string_result_single_arg_direct_log_in_function"
kind = "oracle"
register_entry = "R-30"
program = "r30e_function.js"
verdict = "fixed"
rationale = """R-30, `String()`-result single-argument lane, in-function scope. The module-scope program verbatim inside `function main() { ... }` with a trailing `main();`. See the module-scope case for what it guards and why it must stay FIXED.

The scope pair matters here beyond convention: `string_result_render_taint` keys partly on binding names and partly on callee return provenance, and those resolve differently at module scope and inside a function. A regression that took the exemption away at one scope only would show as a split pair rather than as silence.

MEASURED at this project's HEAD against node v26.7.0: FIXED."""

[[case]]
name = "r30f_string_result_multi_arg_direct_log_module_scope"
kind = "oracle"
register_entry = "R-30"
program = "r30f_module.js"
verdict = "fail_closed"
rationale = """R-30, `String()`-result MULTI-argument lane, module scope. The pair to `r30e`: the same binding, the same sink family, differing only in argument count.

WHY IT EXISTS. `string_result_render_taint`'s own doc lists its sinks as "`+`, template literal, multi-arg console via `emit_as_string`, or arithmetic operator lowering". So today kali gives two different answers for the same value depending on how many arguments the call has: `console.log(s)` prints, `console.log("x", s)` fails closed `E5506`. That is G8's own pattern -- two sinks, two answers -- one level down inside console itself.

EXPECTED TO MOVE. This case is authored `fail_closed` to pin the CURRENT behavior, and the console-render-unification project deliberately flips it to `fixed` (spec §5.1.1). That flip is the project's single behavior widening, and pinning it from both sides is what makes the widening legible as a decision in the diff rather than as a side effect. A reader should be able to see exactly one fail-closed disappear, on purpose.

The widening is safe on the same grounds as the design's §3.1: `value_to_string` renders the handle that the taint existed to keep away from `int_to_string`, so on the console path the hazard the deny guarded is gone rather than merely tolerated.

MEASURED at this project's HEAD against node v26.7.0: FAIL_CLOSED. kali exits nonzero with `E5506`; node prints `x 42`."""

[[case]]
name = "r30f_string_result_multi_arg_direct_log_in_function"
kind = "oracle"
register_entry = "R-30"
program = "r30f_function.js"
verdict = "fail_closed"
rationale = """R-30, `String()`-result MULTI-argument lane, in-function scope. The module-scope program verbatim inside `function main() { ... }` with a trailing `main();`. See the module-scope case for why the lane exists and why it is expected to move to `fixed` in this project.

MEASURED at this project's HEAD against node v26.7.0: FAIL_CLOSED, matching the module scope."""
```

- [ ] **Step 4: Run the suite and the §0.2 gate**

Run: `cargo test -p kali_cli --test cases 2>&1 | tail -20`
Expected: PASS, four more cases.

Run: `cargo test -p kali_blast_radius 2>&1 | tail -20`
Expected: **FAIL** on `every_zero_two_row_is_the_class_set_its_live_cases_assert`. R-30's class set gains `fail_closed`, which §0.2's R-30 row does not list.

- [ ] **Step 5: Add `fail_closed` to §0.2's R-30 row**

In `docs/superpowers/followups/kali-silent-miscompile-register.md`, the R-30 row currently reads `**SILENT** (…) / **FIXED** (…)`. Extend it to name the third class and say what it is:

```
| R-30 booleans render 1/0 in direct log | **SILENT** (`var` binding `r30a`; `const` object field `r30c`) / **FIXED** (`const` scalar `r30b`; concat and template `r30d`; `String()`-result single-arg `r30e`) / **FAIL_CLOSED** (`String()`-result multi-arg `r30f`), both scopes | narrowed by the R-04 fix and narrowed again by the 2026-07-19 correction: among plain bindings only `var` is still wrong (`console.log(b)`→`1`, node `true`), `const` **object fields** remain wrong, and the concat/template sinks are correct for operands kali can prove. **Added 2026-08-15 by the console-render-unification project**: `r30e`/`r30f` pin the taint boundary — the same `String()`-result binding prints at the single-argument sink and fails closed `E5506` at the multi-argument one, differing only by argument count. Six lanes, three classes. The FIXED and FAIL_CLOSED lanes are declared controls and **do not retire the entry**. |
```

- [ ] **Step 6: Re-run the gate**

Run: `cargo test -p kali_blast_radius 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_cli/tests/cases/oracle/tier4.toml docs/superpowers/followups/kali-silent-miscompile-register.md
git commit -m "test(console): pin the taint boundary from both sides before touching it

r30e is the regression guard for this project's central risk. The
single-argument console lane is deliberately exempt from
string_result_render_taint, and that exemption is the only reason a
String()-result binding prints correctly. Routing the lane through
emit_as_string without preserving it converts working programs into E5506.
Written now, before the unification, so it can catch the regression it
exists for rather than document it afterwards.

r30f pins its pair: the same binding at the multi-argument sink fails
closed today, so kali gives two answers for one value depending on argument
count. It is authored fail_closed deliberately and flips to fixed in the
unification task -- that flip is this project's one behavior widening, and
pinning it from both sides makes it legible as a decision in the diff.

§0.2's R-30 row gains the third class so the oracle agreement gate stays
green."
```

---

## Task 3: Move `format_js_number` into `kali_common`

Spec §4. This is what makes the static fold (codegen) and the host agree by construction rather than by mirroring. Both crates already depend on `kali_common`.

**Files:**
- Create: `crates/kali_common/src/js_number.rs`
- Create: `crates/kali_common/src/js_number_tests.rs`
- Modify: `crates/kali_common/src/lib.rs`, `crates/kali_common/Cargo.toml`
- Modify: `crates/kali_runtime/src/host/imports_default.rs`, `crates/kali_runtime/Cargo.toml`

**Interfaces:**
- Consumes: nothing.
- Produces: `kali_common::js_number::format_js_number(value: f64) -> String`. Task 6 calls it from `kali_codegen`.

- [ ] **Step 1: Write the failing test**

Create `crates/kali_common/src/js_number_tests.rs`:

```rust
//! Threshold tests for the single JS number formatter.
//!
//! These pin the two ECMAScript `Number::toString` exponential thresholds from
//! both sides. The 1e21 pair is the interesting one: the formatter is correct
//! there, which is the evidence that R-55 (a literal at or past 1e21 rendering
//! as expanded digits in every sink) is NOT a formatter defect but an upstream
//! classification one. See the register's §7 R-55.
use super::js_number::format_js_number;

#[test]
fn small_magnitude_threshold_is_exact_from_both_sides() {
    assert_eq!(format_js_number(1e-6), "0.000001");
    assert_eq!(format_js_number(1e-7), "1e-7");
}

#[test]
fn large_magnitude_threshold_is_exact_from_both_sides() {
    assert_eq!(format_js_number(1e20), "100000000000000000000");
    assert_eq!(format_js_number(1e21), "1e+21");
}

#[test]
fn the_formatter_is_not_what_r55_is_about() {
    // R-55 reports `console.log(1e21)` printing 22 literal digits. This asserts
    // the formatter would have rendered it correctly if it had been reached, so
    // the defect is upstream of here and this test is what pins that reasoning.
    assert_ne!(format_js_number(1e21), "1000000000000000000000");
}

#[test]
fn non_finite_and_zero_render_as_javascript_does() {
    assert_eq!(format_js_number(f64::NAN), "NaN");
    assert_eq!(format_js_number(f64::INFINITY), "Infinity");
    assert_eq!(format_js_number(f64::NEG_INFINITY), "-Infinity");
    assert_eq!(format_js_number(0.0), "0");
    assert_eq!(format_js_number(-0.0), "0");
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p kali_common js_number 2>&1 | tail -20`
Expected: FAIL to compile — `js_number` module does not exist.

- [ ] **Step 3: Create the module**

Create `crates/kali_common/src/js_number.rs`:

```rust
//! The single JS `Number::toString` formatter.
//!
//! Lives here rather than in `kali_runtime` because BOTH the wasmtime host and
//! `kali_codegen`'s static-literal fold must render numbers identically. When
//! they were two functions they disagreed: the fold returned a literal's own
//! source text, so `console.log(1e-7)` printed `0.0000001` while every dynamic
//! lane printed `1e-7`. That divergence is register entry R-32's small-magnitude
//! half, and one shared function is what closes it by construction.
//!
//! `ryu_js` implements the ECMAScript `Number::toString` algorithm, including
//! both exponential thresholds; the arms above it are the cases the algorithm
//! does not cover.

/// Renders `value` the way JavaScript's `String(number)` does.
pub fn format_js_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned();
    }
    if value == 0.0 {
        // Covers -0.0 too: JS renders both as "0" in string position, and the
        // `==` comparison is true for both.
        return "0".to_owned();
    }
    ryu_js::Buffer::new().format_finite(value).to_owned()
}

#[cfg(test)]
#[path = "js_number_tests.rs"]
mod js_number_tests;
```

Add to `crates/kali_common/src/lib.rs`, alongside the existing module declarations:

```rust
pub mod js_number;
```

Add to `crates/kali_common/Cargo.toml` under `[dependencies]`:

```toml
ryu-js = { workspace = true }
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_common js_number 2>&1 | tail -20`
Expected: PASS, 4 tests.

- [ ] **Step 5: Point the host at the shared formatter**

In `crates/kali_runtime/src/host/imports_default.rs`, delete the local `fn format_js_number(value: f64) -> String { ... }` definition (it is immediately above the `#[cfg(test)] mod imports_default_tests;` at the end of the file) and import the shared one instead. Add near the top of the file, after `use crate::*;`:

```rust
use kali_common::js_number::format_js_number;
```

Confirm `crates/kali_runtime/Cargo.toml` already lists `kali_common = { workspace = true }` — it does, at line 8 — and remove `ryu-js` from that file's `[dependencies]`, since the host no longer calls it directly.

- [ ] **Step 6: Run the runtime tests**

Run: `cargo test -p kali_runtime 2>&1 | tail -20`
Expected: PASS. `float_to_string`'s behavior is unchanged — the function moved, its body did not.

- [ ] **Step 7: Run the workspace build**

Run: `cargo build --workspace 2>&1 | tail -5`
Expected: success, no unused-dependency warnings for `ryu-js`.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_common/src/js_number.rs crates/kali_common/src/js_number_tests.rs crates/kali_common/src/lib.rs crates/kali_common/Cargo.toml crates/kali_runtime/src/host/imports_default.rs crates/kali_runtime/Cargo.toml
git commit -m "refactor(common): one JS number formatter, shared by host and codegen

The host and codegen's static-literal fold must render numbers identically
or R-32's small-magnitude half cannot close: the fold returns a literal's
source text, so console.log(1e-7) prints 0.0000001 while every dynamic lane
prints 1e-7. Moving the formatter to kali_common -- which both crates
already depend on -- makes them agree by construction instead of by
mirroring, which is the same failure the four console renderers are.

The threshold tests pin both exponential boundaries from both sides. The
1e21 pair does double duty: the formatter is CORRECT there, which is the
evidence that R-55 is an upstream classification defect and not a formatter
one. Body unchanged; only its home moved."
```

---

## Task 4: Add `value_to_string` at all five registration points

Spec §4 and the global constraint. The import is registered and declared but **not yet wired into the ladder** — Task 5 does that. Splitting them means a `LinkError` from a missed mirror surfaces on its own, not tangled with a codegen change.

**Files:**
- Modify: `crates/kali_runtime/src/host/imports_default.rs`
- Modify: `crates/kali_codegen/src/lib.rs`, `crates/kali_codegen/src/lower.rs`
- Modify: `crates/kali_runtime_contract/src/browser/harness.rs` (two sites)
- Modify: `crates/kali_cli/src/bin/cmd_build.rs` (two sites)

**Interfaces:**
- Consumes: nothing.
- Produces: host import `kali:rt.value_to_string` with signature `(i64) -> i64`; `kali_codegen::VALUE_TO_STRING_IMPORT_INDEX: u32 = 23`. Task 5 emits `Instruction::Call(VALUE_TO_STRING_IMPORT_INDEX)`.

- [ ] **Step 1: Write the failing test**

Add to `crates/kali_runtime/src/execute_tests/host_env.rs`, following the hand-written-wat pattern already in that file:

```rust
#[test]
fn value_to_string_renders_a_handle_as_text_and_a_scalar_as_digits() {
    // `value_to_string` is `format_console_value` + an allocation: it decodes a
    // tagged string handle to its text and renders anything else as an integer.
    // It exists so `emit_as_string` has a terminal arm that can consult runtime
    // state, which `int_to_string` cannot -- see the console-render-unification
    // spec §3. This asserts the import links and round-trips through console.
    let wasm = wat_module(
        r#"
            (import "kali:rt" "value_to_string" (func $value_to_string (param i64) (result i64)))
            (import "kali:rt" "console_log" (func $console_log (param i64)))
            (func (export "_start")
                i64.const 42
                call $value_to_string
                call $console_log)
        "#,
    );

    let runtime = RuntimeCtx::default();
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.stdout, "42\n");
}
```

Match the existing helper's name and signature in that file — the module-building helper there is used by `runtime_exposes_console_routing` immediately above; reuse it verbatim rather than writing a new one.

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p kali_runtime value_to_string 2>&1 | tail -20`
Expected: FAIL — unknown import `kali:rt::value_to_string`.

- [ ] **Step 3: Register the host import**

In `crates/kali_runtime/src/host/imports_default.rs`, immediately after the `int_to_string` registration block:

```rust
    // The terminal arm of `emit_as_string` for the CONSOLE sink. Identical in
    // behavior to what the console imports already do to a raw value -- decode a
    // tagged string handle, else render the integer -- but it returns a guest
    // string handle instead of writing to a stream, so codegen can put it in a
    // ladder. That is what lets the single- and multi-argument console lanes
    // share one renderer with the host rather than each having their own.
    //
    // Deliberately NOT the terminal arm for `+`/template literals:
    // STRING_HANDLE_TAG is the sign bit, so every negative integer attempts a
    // decode here and survives only because the bounds check fails. Widening
    // that to the concat population is a hazard this project declines to take
    // (spec §3.1).
    linker
        .func_wrap(
            "kali:rt",
            "value_to_string",
            |mut caller: Caller<'_, KaliHostState>, value: i64| -> i64 {
                let text = format_console_value(&mut caller, value);
                alloc_guest_string(&mut caller, text.as_bytes()).unwrap_or(0)
            },
        )
        .map_err(|error| host_import_error("value_to_string", error))?;
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p kali_runtime value_to_string 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 5: Declare the import in codegen**

In `crates/kali_codegen/src/lib.rs`, append after `COVERAGE_HIT_IMPORT_INDEX`:

```rust
// Appended, never renumbered: these indices are positional and every emitted
// `Call` names one by value. `value_to_string` is `(i64) -> i64`, the same
// signature as `int_to_string`, so it reuses function type 4.
const VALUE_TO_STRING_IMPORT_INDEX: u32 = 23;
```

In `crates/kali_codegen/src/lower.rs`, after the `import_section.import("kali:rt", "string_concat_arena", ...)` line and in index order with the other appended imports:

```rust
    import_section.import("kali:rt", "value_to_string", EntityType::Function(4));
```

**Check the index arithmetic before moving on.** `COVERAGE_HIT_IMPORT_INDEX` is 22 and its comment at `lib.rs:70` notes a neighbouring index shifts by +1 under some configuration. Read that comment and confirm 23 is genuinely free in every configuration; if the coverage import is conditional, the new import must be declared **before** it or the index is wrong when coverage is off.

- [ ] **Step 6: Add the four JS mirrors**

Identical text at all four sites, placed next to each list's existing `int_to_string` entry. Note the doubled braces — these are Rust `format!` templates.

`crates/kali_runtime_contract/src/browser/harness.rs` near `:398` and near `:964`, and `crates/kali_cli/src/bin/cmd_build.rs` near `:1722` and near `:2229`:

```rust
    value_to_string(value) {{
      return allocGuestString(new TextEncoder().encode(formatConsoleValue(value)));
    }},
```

All four mirrors already define both helpers (`formatConsoleValue` at `harness.rs:180`/`:755`, `cmd_build.rs:1895`/`:2402`; `allocGuestString` at `harness.rs:120`/`:640`, `cmd_build.rs:1916`/`:2423`), so no helper needs adding.

- [ ] **Step 7: Verify all five points are covered**

```bash
grep -rn 'value_to_string' crates/kali_runtime/src/host/imports_default.rs crates/kali_runtime_contract/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs crates/kali_codegen/src/lib.rs crates/kali_codegen/src/lower.rs | wc -l
```

Expected: at least 6 (one host registration, four JS mirrors, one index constant, one import declaration — the host block also names it in its error mapping). Confirm by eye that **four** of the hits are in the two JS-mirror files, two per file. A count of three there means a `LinkError` waiting in the browser lane.

- [ ] **Step 8: Run the workspace tests**

Run: `cargo test --workspace 2>&1 | tail -30`
Expected: PASS. Nothing calls the new import yet, so behavior is unchanged; this run is proving the import declaration did not disturb the existing indices.

**If anything fails here it is almost certainly the index.** An off-by-one in a positional import table shows up as unrelated intrinsics calling the wrong function, not as a link error.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_runtime/src/host/imports_default.rs crates/kali_runtime/src/execute_tests/host_env.rs crates/kali_codegen/src/lib.rs crates/kali_codegen/src/lower.rs crates/kali_runtime_contract/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs
git commit -m "feat(runtime): add value_to_string, the ladder's runtime-consulting arm

emit_as_string's terminal arm is int_to_string, which is sound only for a
real integer -- an unproven string handle through it measured
-9223354444668731387 for 'hello'. That is why string_result_render_taint
exists and why the single-argument console lane was left OUT of the ladder
entirely, handing the host a raw tagged i64 to render instead.

value_to_string is what lets those two designs merge: the host's own
decode-tag-else-render logic, returning a guest string handle instead of
writing to a stream, so codegen can put it in a ladder.

Registered but not yet wired -- the ladder change is its own commit, so a
LinkError from a missed mirror surfaces on its own. Five registration
points for one import: the wasmtime host plus FOUR hand-maintained kali:rt
JS lists. emit_boolean_as_string's doc records that cost and chose a Select
over interned constants to avoid paying it, which was right for a boolean's
two possible strings and is not available to a function whose purpose is to
consult runtime state."
```

---

## Task 5: The sink parameter, the terminal arm, and both console lanes

Spec §3, §5.1, §5.1.1. This is the task the whole project exists for, and the one that can regress Task 2's `r30e`.

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs:1826` (`emit_as_string`)
- Modify: `crates/kali_codegen/src/emit/call.rs:23` (`emit_console_argument`), `:75` (`emit_console_argument_as_string`)
- Modify: `crates/kali_cli/tests/cases/oracle/tier4.toml` (flip `r30f`)

**Interfaces:**
- Consumes: `VALUE_TO_STRING_IMPORT_INDEX` from Task 4; cases `r30e`/`r30f` from Task 2.
- Produces: `enum StringSink { Concat, Console }` in `emit/operators.rs`; `emit_as_string(&mut self, function: &mut Function, id: LirNodeId, sink: StringSink)`.

- [ ] **Step 1: Confirm the current state of both guard cases**

Run: `cargo test -p kali_cli --test cases r30e 2>&1 | tail -10`
Expected: PASS (`fixed`).

Run: `cargo test -p kali_cli --test cases r30f 2>&1 | tail -10`
Expected: PASS (`fail_closed`).

Both must be green before the change, or the before/after story is unreadable.

- [ ] **Step 2: Add the sink type and thread it through `emit_as_string`**

In `crates/kali_codegen/src/emit/operators.rs`, above `emit_as_string`:

```rust
/// Which sink is consuming a string coercion.
///
/// The first three arms of `emit_as_string` -- proven string, boolean shape,
/// float -- are where all the repr knowledge lives and are shared by every
/// caller. Only the TERMINAL arm differs, and it differs for a measured reason
/// rather than a stylistic one (spec §3.1):
///
/// - `Concat` keeps `int_to_string`, and keeps the `string_result_render_taint`
///   deny that protects it. Unchanged in every respect.
/// - `Console` uses `value_to_string`, which decodes a tagged string handle at
///   run time, and skips the taint deny -- because the hazard the deny guards
///   (a handle's raw bits through `int_to_string`) cannot occur on this arm.
///
/// `Console` is NOT a better terminal arm that `Concat` should also adopt.
/// `STRING_HANDLE_TAG` is the sign bit, so every negative integer reaching
/// `value_to_string` attempts a decode and survives only because the bounds
/// check fails. Console already pays that cost today -- the host does exactly
/// this to every console value. Extending it to `+` would widen a live hazard to
/// a much larger population for no measured benefit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum StringSink {
    Concat,
    Console,
}
```

Change the signature and the two arms it governs:

```rust
    pub(crate) fn emit_as_string(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        sink: StringSink,
    ) {
        // Stage P5 T-new-E: a `String()`-result bound to a variable or returned
        // from a function carries a real string handle in an `I64` slot
        // (`repr_infer` seeds no `Repr::String` — F-newB-1). Reaching this
        // coercion ladder it would fall through to `int_to_string` and print the
        // raw handle bits — the measured `x-9223354375949254655` silent
        // divergence. Fail CLOSED. Positive provenance only.
        //
        // CONSOLE IS EXEMPT, and was exempt before this ladder was shared:
        // `emit_console_argument` used to bypass `emit_as_string` entirely and
        // hand the host a raw tagged i64, which the host decodes correctly. The
        // exemption is what makes `console.log(s)` print for a `String()`-result
        // binding, and `r30e` is the case that keeps it honest. Applying the
        // deny here would convert working programs into E5506.
        if sink == StringSink::Concat && self.string_result_render_taint(id) {
            self.deny_e5506(function, Self::STRING_RESULT_RENDER_DENY);
            return;
        }
```

and the terminal `else`:

```rust
        if emitted.produced
            && (matches!(emitted.shape, ValueShape::Float) || self.is_float_valued(id))
        {
            function.instruction(&Instruction::Call(FLOAT_TO_STRING_IMPORT_INDEX));
        } else {
            match sink {
                StringSink::Concat => {
                    function.instruction(&Instruction::Call(INT_TO_STRING_IMPORT_INDEX));
                }
                StringSink::Console => {
                    function.instruction(&Instruction::Call(VALUE_TO_STRING_IMPORT_INDEX));
                }
            }
        }
    }
```

- [ ] **Step 3: Update every existing caller to `StringSink::Concat`**

```bash
grep -rn 'emit_as_string(' crates/kali_codegen/src/ --include=*.rs | grep -v 'fn emit_as_string'
```

Every hit that is not in `emit/call.rs`'s two console helpers takes `StringSink::Concat` — that is the no-change default, and it must be applied mechanically without judgment. Only the two console lanes get `Console`, in Step 4.

- [ ] **Step 4: Route both console lanes through the ladder**

In `crates/kali_codegen/src/emit/call.rs`, replace the body of `emit_console_argument` after its object-reference rejection. The whole of the old emit-and-post-process block (the `emit_node` call, the `is_usp_string_call` materialize, and the float check) is replaced, because `emit_as_string` already does all three:

```rust
    fn emit_console_argument(&mut self, function: &mut Function, id: LirNodeId) {
        if self.object_shape_of_node(id).is_some() {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "printing an object reference is unavailable in the current phase; print its fields instead"
                    .to_string(),
            ));
        }
        // Was: emit a raw tagged i64 and let the HOST render it. That is why a
        // boolean printed as `1` -- the host has the runtime tag but no repr, so
        // it cannot tell a boolean from an integer (R-30). The ladder has the
        // repr, and `StringSink::Console` gives it a terminal arm that consults
        // the runtime tag, so this lane now gets both instead of one.
        self.emit_as_string(function, id, StringSink::Console);
    }
```

And in `emit_console_argument_as_string`, change the sink:

```rust
        self.emit_as_string(function, id, StringSink::Console);
```

The object-reference rejection stays in **both** helpers, exactly as today — spec §5.3. Do not deduplicate it.

- [ ] **Step 5: Run the guard cases**

Run: `cargo test -p kali_cli --test cases r30e 2>&1 | tail -10`
Expected: **PASS**, still `fixed`. This is the regression guard; if it fails, the taint exemption was lost and Step 2's `sink == StringSink::Concat` condition is wrong.

Run: `cargo test -p kali_cli --test cases r30a 2>&1 | tail -10`
Expected: **FAIL** — `r30a` is authored `silent` and now renders `true`. That failure is the fix working. Do not edit it yet; Task 8 flips the verdicts together.

Run: `cargo test -p kali_cli --test cases r30f 2>&1 | tail -10`
Expected: **FAIL** — authored `fail_closed`, now renders. Also the fix working.

- [ ] **Step 6: Flip `r30f` to `fixed`**

This is the project's one deliberate behavior widening, so it moves in the commit that causes it rather than with the bulk verdict flip in Task 8. In `tier4.toml`, change both `r30f` cases' `verdict = "fail_closed"` to `verdict = "fixed"` and append to each rationale:

```
MOVED 2026-08-15 by the console-render-unification project, deliberately and as its only behavior widening (spec §5.1.1). Both console lanes now take `StringSink::Console`, whose terminal arm is `value_to_string` and which skips the `string_result_render_taint` deny. kali now prints `x 42`, matching node. The two console lanes agree with each other for the first time; before this commit the same binding printed at the single-argument sink and failed closed at the multi-argument one, differing only by argument count.
```

- [ ] **Step 7: Run the codegen and runtime suites**

Run: `cargo test -p kali_codegen 2>&1 | tail -30`
Run: `cargo test -p kali_runtime 2>&1 | tail -20`

Expected: PASS. `+` and template literals are untouched, so a failure here means the `StringSink::Concat` sweep in Step 3 missed a caller or changed one it should not have.

- [ ] **Step 8: Run the full workspace**

Run: `cargo test --workspace 2>&1 | tail -40`
Expected: failures **only** in the oracle cases authored `silent` for lanes this fix moves — `r30a`, `r30c`, and possibly `r32a` — plus the `kali_blast_radius` §0.2 gate. Any other failure is a real regression.

Record the exact failing list; Task 8 consumes it.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_codegen/src/emit/operators.rs crates/kali_codegen/src/emit/call.rs crates/kali_cli/tests/cases/oracle/tier4.toml
git commit -m "fix(console): one ladder for both console lanes, closing R-30

The single-argument console lane never went through emit_as_string: it
handed the host a raw tagged i64 and let the host render. The host holds the
runtime string-handle tag but no repr, so it cannot tell a boolean from an
integer -- console.log(b) printed 1. The ladder holds the repr but its
terminal arm, int_to_string, corrupts a handle it cannot prove. Neither side
had enough information alone, which is what G8 actually is.

StringSink gives the shared ladder a terminal arm per sink. Console gets
value_to_string, so it has the repr AND the runtime tag; Concat keeps
int_to_string and its taint deny, entirely unchanged. Console is not a
better arm that Concat should adopt -- STRING_HANDLE_TAG is the sign bit, so
every negative integer there attempts a decode and survives on a bounds
check. Console already pays that today; + should not start.

The multi-argument lane joins it, which flips r30f from fail_closed to
fixed. That is this project's only behavior widening and it is made here,
in the commit that causes it, with the case pinned from both sides in the
previous commit so the diff shows exactly one fail-closed disappearing on
purpose. The two console lanes now agree with each other for the first
time.

r30a/r30c go red here and are flipped with the rest of the verdicts once
the static fold and the warn prefix have landed too."
```

---

## Task 6: The static fold's numeric Literal arm

Spec §3 and §4. The fourth renderer. `console.log(1e-7)` is a bare literal, so it never enters a dynamic lane at all — it folds at compile time to the literal's own source text.

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/host.rs:717` (`render_static_value`)

**Interfaces:**
- Consumes: `kali_common::js_number::format_js_number` from Task 3.
- Produces: nothing new.

- [ ] **Step 1: Confirm the lane is still broken**

```bash
cargo build -p kali_cli 2>&1 | tail -3
cd /tmp/g8probe && /workspace/.cache/cargo-target/debug/kali run a.js
```

Expected: `0.0000001`. Task 5 did not touch this — the value never reaches the ladder.

- [ ] **Step 2: Change the numeric Literal arm**

In `render_static_value`'s `LirNodeKind::Literal` match arm, the numeric branch currently reads:

```rust
                Some(text) => {
                    if parse_number_literal(text).is_some() {
                        Some(text.to_string())
                    } else {
                        Some(strip_string_delimiters(text).to_string())
                    }
                }
```

Replace with:

```rust
                Some(text) => {
                    // Render through the SAME formatter the host and the dynamic
                    // lanes use, not the literal's own source text. Returning the
                    // text is why `console.log(1e-7)` printed `0.0000001` while
                    // `var y = 1e-7; console.log(y)` and `"v=" + 1e-7` both
                    // printed `1e-7` -- R-32's small-magnitude half, and the
                    // fourth of the four renderers this project collapses.
                    if let Some(value) = parse_number_literal(text) {
                        Some(format_js_number(value))
                    } else {
                        Some(strip_string_delimiters(text).to_string())
                    }
                }
```

Add the import at the top of `crates/kali_codegen/src/intrinsics/host.rs`:

```rust
use kali_common::js_number::format_js_number;
```

**Check `parse_number_literal`'s return type first.** If it yields something other than `f64` (a wrapper, or an integer/float enum), convert at this call site — do not change `format_js_number`'s signature, which Task 3's tests and the host both depend on. If it cannot yield an `f64` for a BigInt literal, keep those on the text path: a BigInt renders as its digits and must not go through a float formatter.

- [ ] **Step 3: Verify the lane by hand**

```bash
cargo build -p kali_cli 2>&1 | tail -3
cd /tmp/g8probe
for x in a b c; do echo "=== $x"; /workspace/.cache/cargo-target/debug/kali run $x.js; node $x.js; done
```

Expected: `a` now prints `1e-7` on both. `b` unchanged, both `1e-7`. `c` still kali `1000000000000000000000` vs node `1e+21` — that is R-55 and stays open.

- [ ] **Step 4: Run the codegen suite**

Run: `cargo test -p kali_codegen 2>&1 | tail -30`
Expected: PASS, or failures only in tests asserting the old literal-text rendering. Read each such failure and confirm the new value is what node produces before updating it; a literal rendering that changes for a value node renders the same way is a bug in this change, not a stale expectation.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_codegen/src/intrinsics/host.rs
git commit -m "fix(codegen): the static fold renders numbers like JS, not like source

console.log(1e-7) is a bare literal, so it folds at compile time and never
reaches any dynamic lane -- render_static_value returned the literal's own
source text. That is why it printed 0.0000001 while the binding and concat
lanes both printed 1e-7 correctly: three sinks, two answers, and the odd one
out was the one that never ran.

Now it goes through kali_common's format_js_number, the same function the
host uses, so the fourth renderer collapses into the same rule as the other
three. R-32's small-magnitude half closes here.

R-55 (1e21) is untouched and stays open: that value never reaches the
formatter in ANY lane, which is what makes it a classification defect rather
than a rendering one."
```

---

## Task 7: Delete the `[warn] ` prefix

Spec §4 and §11. R-33. One line of behavior, four assertions, and two lookalikes that must be left alone.

**Files:**
- Modify: `crates/kali_runtime/src/host/imports_default.rs:53`
- Modify: `crates/kali_runtime/src/execute_tests/host_env.rs:32`
- Modify: `crates/kali_cli/tests/runtime_smoke/run.rs:11010`, `:12802`
- Modify: `crates/kali_cli/tests/runtime_smoke/test.rs:11180`

**Interfaces:**
- Consumes: nothing.
- Produces: nothing.

- [ ] **Step 1: Delete the prefix**

In `crates/kali_runtime/src/host/imports_default.rs`, the `console_warn` registration:

```rust
                append_stderr(caller.data_mut(), format!("[warn] {}", rendered));
```

becomes:

```rust
                // No prefix: node's `console.warn` writes the message alone, and
                // NONE of the four JS import mirrors prefixed it either -- so
                // this line made kali's own runtimes disagree with each other
                // before it made kali disagree with node. R-33.
                append_stderr(caller.data_mut(), rendered);
```

- [ ] **Step 2: Run the tests to see exactly what pinned it**

Run: `cargo test --workspace 2>&1 | grep -i "warn" | head -20`
Expected: FAIL at four sites — `kali_runtime` `host_env.rs`, and three in `kali_cli`'s runtime smoke.

- [ ] **Step 3: Update the four assertions**

`crates/kali_runtime/src/execute_tests/host_env.rs:32`:

```rust
    assert_eq!(outcome.stderr, "2\n3\n");
```

`crates/kali_cli/tests/runtime_smoke/run.rs:11010`, `run.rs:12802`, `test.rs:11180` — each currently asserts `stderr.contains("[warn] warn")`:

```rust
    assert!(stderr.contains("warn"), "stderr: {stderr}");
```

- [ ] **Step 4: Leave the two lookalikes alone**

`crates/kali_case_runner/src/steps_tests.rs:789` and `crates/kali_blast_radius/src/verdict_tests.rs:99`, `:126`, `:192` contain `[warn] ` strings. These are **simulated kali output feeding classifier fixtures**, not assertions about kali's behavior. `verdict_tests.rs:99`'s own comment explains that the fixture exists to exercise how the classifier treats that damage shape. Changing them would silently weaken the classifier's tests while appearing to be part of this fix.

Confirm you have not touched them:

```bash
git diff --name-only | grep -E 'case_runner|blast_radius'
```

Expected: **no output**.

- [ ] **Step 5: Verify against node**

```bash
cargo build -p kali_cli 2>&1 | tail -3
cd /tmp/g8probe && printf 'console.warn("hi");\nconsole.error("bye");\n' > i.js
echo "--kali"; /workspace/.cache/cargo-target/debug/kali run i.js
echo "--node"; node i.js
```

Expected: both print `hi` then `bye` on stderr, byte-identical.

- [ ] **Step 6: Run the workspace**

Run: `cargo test --workspace 2>&1 | tail -40`
Expected: failures only in the oracle cases awaiting Task 8 (`r30a`, `r30c`, `r33a`, possibly `r32a`) plus the `kali_blast_radius` §0.2 gate.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_runtime/src/host/imports_default.rs crates/kali_runtime/src/execute_tests/host_env.rs crates/kali_cli/tests/runtime_smoke/run.rs crates/kali_cli/tests/runtime_smoke/test.rs
git commit -m "fix(runtime): console.warn writes the message, not '[warn] ' + message

R-33. node's console.warn emits the message alone, and this prefix broke
byte-for-byte comparison of any program using it -- which matters more than
the entry's tier suggests, because byte-for-byte acceptance is this
project's primary correctness method. A rendering divergence corrupts the
instrument every other verdict is measured with.

It was also an internal inconsistency first: none of the four kali:rt JS
import mirrors prefixed it, so kali's wasmtime host and its browser lane
already disagreed with each other and node was the third opinion.

Four assertions pinned the prefix and are updated. The [warn] strings in
kali_case_runner and kali_blast_radius are deliberately untouched -- they
are simulated kali output feeding classifier fixtures, not assertions about
kali, and 'fixing' them would weaken the classifier's own tests while
looking like part of this change."
```

---

## Task 8: Flip the verdicts, update §0.2, regenerate the ranking

Spec §7 and §7.1. The measurement catches up with the code, and the ranking document is re-derived rather than re-read.

**Files:**
- Modify: `crates/kali_cli/tests/cases/oracle/tier4.toml`
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (§0.2 rows, §2 entry bodies)
- Modify: `docs/superpowers/followups/blast-radius-ranking.md`

**Interfaces:**
- Consumes: the failing-case list recorded in Task 5 Step 8 and Task 7 Step 6.
- Produces: a green `cargo test --workspace`.

- [ ] **Step 1: Get the authoritative list of moved lanes**

Run: `cargo test -p kali_cli --test cases 2>&1 | grep -E "^(test .* FAILED|failures:)" -A 40 | head -40`

Write down every failing case name and, for each, what it now measures. **Do not flip a verdict you have not read the new output for** — a case that moved to `fail_closed` rather than `fixed` is a regression wearing the same red.

- [ ] **Step 2: Flip each moved case and append its amendment**

For `r30a` (both scopes), `r30c` (both scopes), `r33a` (both scopes), and `r32a` if it moved: change `verdict = "silent"` to `verdict = "fixed"` and append to each rationale:

```
MOVED 2026-08-15 by the console-render-unification project (`docs/superpowers/specs/2026-08-15-console-render-unification-design.md`). Re-measured against node v26.7.0: FIXED. The single-argument console lane now renders through `emit_as_string` with `StringSink::Console`, so it has both the repr this lane always lacked and the runtime string-handle tag the host has always had. Before this commit the host rendered the value and had no repr to render it with.
```

For `r32a` specifically, if only its `1e-7` half moved, the case must be **split** rather than flipped — its program covers both thresholds and they now diverge. Add `r32a`'s large-magnitude half as a new case pair pointing at R-55 and referencing Task 1's entry, and narrow `r32a`'s program and rationale to the small-magnitude half it now measures. A single case cannot hold two verdicts.

- [ ] **Step 3: Update §0.2's rows**

For R-30, R-32 and R-33 in `docs/superpowers/followups/kali-silent-miscompile-register.md` §0.2, restate the class sets and name the commit they were re-measured at. R-30 and R-33 now have **every** lane `fixed`, which under §3.4's rule retires them — say so explicitly in the row rather than leaving a reader to infer it from a set of classes.

Also amend the §2 entry bodies for R-30 and R-33 to record the close, following the register's convention: state what supersedes what, and **strike** rather than delete.

- [ ] **Step 4: Record R-08 residual 5 as closed**

The register records residual 5 as blocked on R-30's fix ("it is blocked on R-30's own fix (unify the two console formatters)"). Amend it to closed, naming this project and the commit. Verify by hand first:

```bash
cd /tmp/g8probe && printf 'console.log(Number.isInteger(5) ?? 9);\n' > j.js
/workspace/.cache/cargo-target/debug/kali run j.js; node j.js
```

Expected: both print `true`. **If they disagree, residual 5 did not close** — record that instead, and do not claim a close the measurement does not support.

- [ ] **Step 5: Run the §0.2 gate**

Run: `cargo test -p kali_blast_radius 2>&1 | tail -30`
Expected: `every_zero_two_row_is_the_class_set_its_live_cases_assert` PASSES, and `spliced_document_matches_the_generator` **FAILS** — the ranking's inputs moved.

- [ ] **Step 6: Regenerate the ranking document**

```bash
cargo run -p kali_blast_radius --example rank > /tmp/g8probe/rank.out
wc -l /tmp/g8probe/rank.out
```

Splice the output into `docs/superpowers/followups/blast-radius-ranking.md` between its two `<!-- GENERATED -->` marker pairs, replacing everything between them. The provenance table goes in §1.4's marked region and sections 2-5 go in the main one. **Do not hand-edit inside the markers** — the test compares them verbatim.

Run: `cargo test -p kali_blast_radius 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 7: Update the ranking's authored commentary**

§6 is authored, not generated, and now contains statements the regeneration falsified — §6.2's "G8 at 65" and the §1.1/§6.1 framing of what band 1 holds. Add a dated amendment at the top of §6 stating what this project moved and what it did not, in the register's convention: state what supersedes what, retain struck text.

Do **not** predict or hand-write any number that the generator produces — read them out of the regenerated §2.

- [ ] **Step 8: Full workspace green**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: PASS, no failures.

Run: `cargo build --workspace 2>&1 | tail -5`
Expected: success.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_cli/tests/cases/oracle/tier4.toml docs/superpowers/followups/kali-silent-miscompile-register.md docs/superpowers/followups/blast-radius-ranking.md
git commit -m "docs(register): R-30 and R-33 retire, and the ranking is re-derived

Every lane of R-30 and R-33 now measures FIXED, which under §3.4's rule --
an entry retires when every lane moves, not one -- retires both. R-08
residual 5 closes with R-30, as the register predicted it would. §0.2's rows
and the §2 entry bodies are amended in the register's convention: state what
supersedes what, strike rather than delete.

The ranking document is REGENERATED, not edited. Moving R-30's verdicts
moved the generator's inputs, so spliced_document_matches_the_generator went
red until the command was actually re-run -- which is the mechanism the
ranking project built and the first time it has been exercised by a change
from outside that project. §6.6 item 4 asked for exactly this: re-run, do
not re-read.

§6's authored commentary carries a dated amendment for the statements the
regeneration falsified. No number in it was predicted or hand-written; they
are read out of the regenerated §2."
```

---

## Self-Review

**Spec coverage.** §1.3's probes → Task 1. §3's ladder rule → Task 5. §3.1's negative-integer hazard → Task 1 (`r32d` control) and Task 5 (the `StringSink` doc). §3.2's structural claim → Task 5. §4's components: `kali_common` → Task 3, host import → Task 4, index/declaration → Task 4, four JS mirrors → Task 4, ladder → Task 5, static fold → Task 6. §5.1 → Task 2 Step 3 + Task 5 Step 2. §5.1.1 → Task 2 + Task 5 Step 6. §5.2 (the `+` path unchanged) → Task 5 Step 3's mechanical `Concat` sweep. §5.3 (object rejection stays in both helpers) → Task 5 Step 4. §6 (do-not-modify) → Global Constraints, and no task touches them. §7's expectation table → Task 8. §7.1's regeneration → Task 8 Steps 6-7. §8's R-55 filing → Task 1 Step 5. §11's sequencing → Tasks 1-8 in order.

Two spec items are covered only as *non-goals* and correctly have no task: R-31 and R-23.

**Placeholder scan.** No TBD/TODO. Every code step carries the actual code. Three steps carry a conditional branch rather than fixed content — Task 4 Step 5 (the import-index check), Task 6 Step 2 (`parse_number_literal`'s return type), and Task 8 Step 2 (`r32a` may need splitting). Each states the condition, what to check, and what to do either way, because each depends on a fact that is cheaper to read at implementation time than to guess at now.

**Type consistency.** `format_js_number(f64) -> String` is defined in Task 3 and called in Task 6 by that exact name and signature. `VALUE_TO_STRING_IMPORT_INDEX` is defined in Task 4 and used in Task 5. `StringSink::{Concat, Console}` is defined in Task 5 and used only there. Case names `r30e`/`r30f`/`r32d` are introduced in Tasks 1-2 and referenced by those names in Tasks 5 and 8. The oracle case schema (`name`, `kind`, `register_entry`, `program`, `verdict`, `observe`, `rationale`) matches the existing `tier4.toml` entries verbatim.

**One risk this plan cannot remove.** Task 5 is the only task that can regress working programs, and its guard (`r30e`) is written in Task 2 — two tasks earlier, by a different implementer, who will not see Task 5. That separation is deliberate: a guard written by the person making the change tends to be written to pass. Task 5 Step 1 re-runs both guards before touching anything, so a fresh implementer sees the green baseline they must preserve.
