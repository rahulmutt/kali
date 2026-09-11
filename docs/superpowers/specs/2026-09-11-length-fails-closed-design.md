# `.length` never invents a number: its floor refuses, and the fallbacks above it decline

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `152fdd5364` (main, clean tree, the merge of PR #42) |
| kali binary | `kali 0.1.0`, `.cache/cargo-target/debug/kali`, rebuilt at the baseline commit |
| oracle | `node v26.8.2` |
| measured on | 2026-09-10 and 2026-09-11 |
| item picked | `docs/superpowers/followups/codegen-array-literal-predicate-is-still-negative-space.md` |
| defect this closes | the `.length` fabricated-count class: `docs/superpowers/followups/member-length-renders-the-child-count.md`, plus two lanes nothing has filed, the in-function `let` array (§2.5) and the awaited `Promise.all` result (§2.5.1) |

**This spec does not fix the item that was picked, and §2.2 is why.** The picked
followup prescribes "the Stage-1-shaped move": narrow `kali_codegen`'s two
negative-space `is_array_literal` copies the way Stage 1 narrowed
`kali_optimize`'s. A throwaway spike of exactly that move turned **three suite
tests and five probe programs silently wrong** and fixed no measured wrong value.
In codegen the predicate's false branch falls to fallbacks that fabricate a
number. The silence lives in those fallbacks, so this project targets them. The
human partner ruled the re-scope after seeing the spike.

**Citation convention.** Every line reference is as of the baseline commit above.

**Spike convention.** Three throwaway patches were measured (§2.2, §2.4, §2.5).
Each was applied to the working tree, measured with the full workspace suite
(`cargo test --workspace --no-fail-fast`, the same command
`scripts/test-gate.sh` runs) and with node-oracle probe programs, then reverted
with `git checkout`. None was committed. The baseline suite at `152fdd5364`
reported **`GATE OK: 0 failing tests`**, so every failure counted below is new.

---

## 1. What this project is

**The claim:** a `.length` read in `kali_codegen` either computes the real value
or refuses with `E5506`. It never renders a node's child count, and it never bakes
in `0`.

Three changes, one crate (§3). One new register entry, filed and retired by this
project (§6). One register row changes class (R-15, §4.4).

### 1.1 How it was chosen

1. The picked item's prescription was measured and falsified (§2.2).
2. The mechanism behind the one wrong value that item's predicate does produce
   was traced to a three-rung ladder whose bottom rung is silent (§2.3).
3. Removing only the upper rungs moved each wrong value onto a different wrong
   value (§2.4). Making the bottom rung refuse as well turned every probed silent
   wrong value in that ladder into a refusal, with no new silent regressions
   (§2.5).

### 1.2 What this project does NOT claim

* It does **not** close `(new Array(3)).length` → `1`. That value comes from the
  predicate rung, which is still negative space (§2.6).
* It does **not** prove every consumer of `render_static_value` is fail-closed
  once `render_length` declines. The spikes never measured that matrix, so
  §4.1 makes it the plan's first task and a stop gate.
* `crypto.randomUUID().length` staying correct under the change is argued from
  source (§2.5.2), not measured under the patch. §4.2 pins it.

---

## 2. The defect, measured

### 2.1 The fact the picked item missed: `new C(x)` and `[C(x)]` are the same LIR node

A throwaway unit test printed `crate::test_support::parse_and_lower_lir`'s output
for eight programs. The relevant rows:

| source | LIR |
|---|---|
| `g([f()]);` | `Value(None, [Call(Value "f")])` |
| `g(new Error("m"));` | `Value(None, [Call(Value "Error", Literal "\"m\"")])` |
| `g(new Array(3));` | `Value(None, [Call(Value "Array", Literal "3")])` |
| `g([Array, 3]);` | `Value(None, [Value "Array", Literal "3"])` |
| `g(new Array(3).length);` | `Value(None, [Value "length" [Call(Array, 3)]])`, which is `new (Array(3).length)` |
| `g((new Array(3)).length);` | `Value "length" [Value(None, [Call(Array, 3)])]` |

HIR distinguishes the two (`HirNodeKind::NewExpr` / `ArrayExpr`), and MIR erases
both to `MirNodeKind::Expr` (`crates/kali_mir/src/lower.rs:108-116`). No shape
check in codegen can tell a one-element array literal from a `new` wrapper. The
repository already knew this. `emit/operators.rs:943-953` says so of
`unwrap_transparent`, and `emit/call.rs:3584-3590` says so of its
literal-argument guard.

The fifth row is a separate parser defect. `new` parses its callee with
`parse_call_expression()` (`crates/kali_parser/src/expression/primary.rs:241`), so
the member chain after the arguments becomes part of the callee. It is filed, not
fixed (§6.3).

### 2.2 Spike 1: narrowing the codegen predicate is fail-OPEN

**Patch:** `intrinsics/array.rs:5` and `lower.rs:7189` accept a one-child
text-less `Value` only when that child is a `Literal`, a spread marker, an object
literal or an unambiguous array literal. Everything else is unchanged.

**Full suite:** 0 → **32 failing** (11,897 passed).

* 29 loud: 20 `build_tests` and 9 `runtime_smoke` `run::` tests over
  `for (const item of [alias])` / `for await`, now `E5506`.
* **3 silent**, which is disqualifying on its own:
  * `misc/arena_reclamation_runtime::module_global_store_fails_closed`:
    `const arr = [mk(0)]` printed `0` where the pin expects `201`.
  * `object/call_result_args_runtime::array_literal_element_field_value_read_is_rejected_not_miscompiled`:
    an `E5506` pin exited 0 printing `0`.
  * `kali_codegen` `crypto_get_random_values_result_stored_into_an_aggregate_fails_closed`:
    the fail-closed diagnostic for `const a = [fb]` vanished.

**Probes:** of 12 one-element controls, 5 went from correct to silently wrong
(`[f()][0]` → `0`, `[x][0]` → `0`, `[x + 1]` → `0 0`, in-function `[f()]` →
`0 0`, `[p].length` → `0`). None improved. **The target,
`(new Array(3)).length` → `1`, did not move.**

### 2.3 The `.length` ladder

A `.length` read passes through three rungs. Each one can fabricate.

1. **Static render.** `render_length` (`intrinsics/host.rs:1194`), reached from
   `render_static_value`'s `"length"` arm (`:916-921`).
   * Per-hazard bails (`:1195-1250`).
   * The array-literal arm (`:1267-1285`), which uses the negative-space
     predicate and returns `children.len()`.
   * The static string fold (`:1287-1291`) and the runtime-string bail
     (`:1293-1301`).
   * **Fallbacks:**
     * `:1336-1338`: a text-less node → `children.len()`
     * `:1369`: an unbound identifier → `Some("0")`
     * `:1373-1377`: the tail → recurse into one child, else `children.len()`
2. **Runtime member lane** (`emit/control_flow.rs:2490`). Per-hazard `E5506`
   denials, the string-handle lane (`:2617`), growable arrays (`:2638`, `:2654`),
   the named-array header read (`:2657`).
3. **`emit_unary`'s `"length"` arm** (`emit/operators.rs:368`). `process.argv`,
   the array-literal arm (`:389-413`, again the predicate, emitting
   `I64Const(children.len())`), the static string fold (`:416-425`), and the
   **floor** (`:427-435`): emit the receiver, drop it, push `I64Const(0)`. No
   diagnostic.

One outside consumer reads rung 1 as evidence. `emit/call.rs:4096-4103` admits
`String(x.length)` when `render_length(&member).is_some()`, which the baked
`Some("0")` satisfies.

Each per-hazard bail in rung 1 and denial in rung 2 records a shape that was
found "the expensive way" falling through to a fabricated count
(`member-length-renders-the-child-count.md` §2).

### 2.4 Spike 2: declining the fallbacks alone is fail-open too

**Patch:** `render_length`'s three fallbacks return `None`.

**Full suite:** 2 new failures of 11,929. One was the WRONG-ON-PURPOSE pin
`object/computed_member_static_name::a_dot_member_length_still_renders_the_child_count`
(`1` → `0`, still silent). The other was
`soundness/textcodec::string_call_proof_admits_the_scalar_shapes_the_parent_build_rendered_static_length`
(success → exit 1, the `call.rs:4100` proof).

**Probes:** each silent wrong value moved to a *different* silent wrong value on
the floor (`o.a.length` `1` → `0`). Two programs that were right by coincidence
became silently `0`. `(new Array(3)).length` stayed `1`, now from rung 3's
predicate arm.

### 2.5 Spike 3: spike 2 plus a floor that refuses

**Patch:** spike 2, plus `operators.rs:427-435` replaced by `deny_e5506`.

**Probes** (`kali run`, node in parentheses):

| program | baseline | spike 2 | spike 3 |
|---|---|---|---|
| `const o = {a: "xyz"}; o.a.length` | `1` (3) | `0` | **`E5506`** |
| `const o = {a: "xyzwv"}; o.a.length` | `1` (5) | `0` | **`E5506`** |
| `const o = {a: "xyz", z: 1}; o.a.length` | `2` (3) | `0` | **`E5506`** |
| `const o = {a: "xyz", z: 1, y: 2}; o.a.length` | `3` (3), coincidence | `0` | `E5506` |
| `const o = {a: "xyz"}; o["a"].length` | `2` (3) | `0` | **`E5506`** |
| `const o = {a: [1, 2, 3]}; o["a"].length` | `2` (3) | `E5506` | **`E5506`** |
| `let o = {a: "xyz"}; o.a.length` | `0` (3) | `0` | **`E5506`** |
| `["abc"][0].length` | `2` (3) | `0` | **`E5506`** |
| `const o = {ab: 1, c: 2}; Object.keys(o)[0].length` | `2` (2), coincidence | `0` | `E5506` |
| `function main() { const o = {a: "xyz"}; … o.a.length }` | `1` (3) | `0` | **`E5506`** |
| `function main() { let a = [1, 2]; … a.length }` | `0` (2) | `0` | **`E5506`** |
| `const a = [1, 2]; String(a.length)` | `2` (2) | `E5506` | `E5506`, fixed by §3.3 |
| `(new Array(3)).length` | `1` (3) | `1` | `1`, §2.6 |
| `function f(x) { return x.length } f(new Array(6))` | `0` (6) | `0` | `0`, §2.6 |

Controls that stayed correct in all three columns: `s.length`,
`[1, 2, 3].length`, `Object.keys(o).length`, `new Array(5)` bound,
`new Array(4).fill(7)` bound, `"a,b,c".split(",").length`, `[f()]` length and
element, `"n=" + a.length`, a string parameter's length, `m[1].length`,
`[].length`, `{a: [1, 2, 3]}` `.a.length`. The 12 one-element probes from spike 1
were unchanged from baseline.

**Full suite:** 26 new failures of 11,929.

* **Silent → loud, intended (3):** the WRONG-ON-PURPOSE pin, and both R-15
  oracle cases (`oracle/tier2::r15_split_returns_empty_array_{module_scope,in_function}`),
  whose measured verdict moved `silent` → `fail_closed`.
* **Loud → loud, message only (1):** `object/computed_member_static_name::run_refuses_a_computed_callee`.
  The spike's floor did not emit the receiver first, so the computed-member
  diagnostic the needle looks for never fired. §3.1 emits the receiver first.
* **Build-only tests now refusing (21):** 16 `browser/promise_all_bundle` /
  `promise_all_settled_bundle` cases, 4 `runtime_smoke` `build::*promise_all_sequencing`
  tests, and `kali_codegen` `crypto_random_uuid_lowers_to_kalirt_import`. §2.5.1
  and §2.5.2 show none of these is a real capability loss.
* **Real capability loss (1):** the textcodec `String(a.length)` case, fixed by §3.3.

#### 2.5.1 The Promise.all failures were hiding a silent wrong value

Measured at the baseline binary:

| program | kali | node |
|---|---|---|
| `(await Promise.all([p1, p2])).length` | `2` | `2` |
| `(await Promise.all([p1, p2, p3])).length` | **`2`** | `3` |
| `(await Promise.all([p1])).length` | **`2`** | `1` |
| `(await Promise.allSettled([p1, p2, p3])).length` | **`2`** | `3` |
| `if (d.length !== 3) "WRONG" else "ok"` over three promises | **`WRONG`** | `ok` |

The value is the awaited `Call` node's arity (callee plus one array argument),
through `render_length`'s tail fallback. The bundle fixtures pass only because
each uses exactly two promises.

#### 2.5.2 `crypto.randomUUID().length`

`kali run` prints `36` at baseline, matching node. In the real pipeline the
receiver takes the string-handle lane (`control_flow.rs:2617`), and
`render_length` bails at `:1293` before any fallback. The unit test fails because
`parse_and_lower_lir` runs no type inference, so nothing proves the handle is a
string. This is argued from source, not measured under the patch; §4.2 pins the
real pipeline.

### 2.6 What is NOT this defect, and stays as it is

| program | kali | node | why it is out of scope |
|---|---|---|---|
| `(new Array(3)).length` | `1` | `3` | rung 3's and rung 1's array-literal arms (`operators.rs:391`, `host.rs:1269`), i.e. the picked item's predicate; narrowing it is now fail-closed for `.length` but unmeasured, and would refuse `[f()].length` |
| `f(new Array(6))` reading `x.length` / `x[0]` | `0` | `6` | an inline allocation argument reaches the callee as zeros; `emit/call.rs:3579-3582` assumes allocations "pass a real handle", true only for a bound allocation (`const a = new Array(6); f(a)` → `6`) |
| `new Array(3).length` | `2` | `3` | the parser precedence defect (§2.1) |
| `console.log([1, 2])` | the length, per the register's R-31 (not re-measured here) | `[ 1, 2 ]` | R-31, `render_static_value`'s text-less arm (`host.rs:928-942`) |
| `p[1]` of `"a,b,c".split(",")` | a leaked handle, per R-15's oracle rationale (`cases/oracle/tier2.toml:1495`, not re-measured here) | `b` | R-15's element half |

---

## 3. The design

### 3.1 The floor refuses

`emit_unary`'s `"length"` terminal arm (`emit/operators.rs:427-435`) keeps emitting
the receiver, so any receiver-specific refusal fires first. Then it pushes
`E5506` through `deny_e5506` (`intrinsics/host.rs:1656`) instead of
`I64Const(0)`. If emitting the receiver already pushed an error, the floor adds
none of its own (§7, third row). The message names the gap: no proven length
lane for this receiver.

### 3.2 `render_length`'s three fallbacks decline

`intrinsics/host.rs:1336-1338`, `:1369` and `:1373-1377` return `None`. A declined
static render reaches rungs 2 and 3, which compute the real value or reach the
refusing floor.

### 3.3 `call.rs:4100` gets a real proof

`String(x.length)` is admitted on the member shape alone: text `"length"`, one
child. With §3.1 in place, a `.length` read either yields an integer or stops
compilation, so no rendered value is needed as evidence. `String(o.a.length)`
still refuses, because its member read refuses (§4.2 pins it).

### 3.4 Left alone, deliberately

**The per-hazard bails and denials** in rungs 1 and 2 become redundant: each now
declines onto a floor that refuses anyway. Removing them is a second kind of
change and is filed (§6.3), not done. The human partner confirmed this scope.

### 3.5 Rejected alternatives, each measured or reasoned

| alternative | why rejected |
|---|---|
| narrow the codegen predicate (the picked item) | spike 1: fail-open, 3 suite and 5 probe silent regressions, target unmoved |
| decline the fallbacks only | spike 2: every wrong value moves to another wrong value on the silent floor |
| refuse at the floor only | rung 1's fallbacks still render child counts; `o.a.length` → `1` and `Promise.all(...).length` → `2` stay silent |
| represent `new` vs array literal in LIR | fixes no measured wrong value on its own, because the fallbacks produce the same numbers; Stage 2-sized |
| also narrow the `.length` predicate arms | unmeasured, and likely refuses correct `[f()].length`; filed (§6.3) |

---

## 4. Verification

### 4.1 First task: the consumer matrix, as a stop gate

`render_static_value` has 56 call sites across 9 files. When `render_length`
declines, each caller either routes to emission, which is fail-closed after §3.1,
or folds a number of its own. The spikes never measured which. Before any
production code changes, the plan measures `.length` in these positions:

* `===` / `!==` against a literal
* arithmetic (`+ 1`)
* a template literal and string concatenation
* a ternary test and an `if` condition
* `Math.max(…)` and `String(…)`
* as an array index

Each position is measured on a receiver that must refuse (`o.a.length`) and one
that must stay correct (`[1, 2, 3].length`), at module scope and in a function,
against node. **Any silent result in a must-refuse receiver stops
implementation**, and the design goes back to the human partner before code is
written.

### 4.2 New pins: `crates/kali_cli/tests/cases/runtime/length_fails_closed.toml`

Every case records node's value in its rationale.

* **Must refuse** (`exit = "failure"`, `stderr_contains` names the floor's
  message): every bold row of §2.5's table, and §2.5.1's three-promise and
  one-promise `Promise.all` and `allSettled` programs. Both scopes wherever the
  shape allows.
* **Must refuse, right today by coincidence** (the coincidence is recorded in the
  rationale): three-property `o.a.length`, `Object.keys(o)[0].length`, two-promise
  `Promise.all`.
* **Must refuse through a consumer:** `String(o.a.length)`.
* **Must stay correct** (`exit = "success"`, exact stdout): every control listed
  under §2.5's table, plus `crypto.randomUUID().length` → `36`, and
  `String(a.length)` for `[1n, 2n]` → `2` and `[1n, 2n, 3n]` → `3`.

Order: the must-refuse pins are committed red against the baseline before any
production change. The controls go in green.

### 4.3 Existing tests that change, and why

| test | change | reason |
|---|---|---|
| `object/computed_member_static_name::a_dot_member_length_still_renders_the_child_count` (`cases/object/computed_member_static_name.toml:629`) | renamed to a refusal pin | it pinned the wrong value on purpose, until something owned closing it |
| `browser/promise_all_bundle.toml` (guards from `:129`), `browser/promise_all_settled_bundle.toml` (from `:134`) | delete each `X.length !== 2 ||` clause | the fixtures cover Promise.all/allSettled sequencing; the element checks beside every guard stay, and §4.2's pin covers the `.length` |
| `browser_bundle_promise_all_sequencing_source()` (`crates/kali_cli/tests/runtime_smoke.rs:147`), used by `runtime_smoke/build.rs:4214`, `:4224` | same clause deleted | same reason |
| `crypto_random_uuid_lowers_to_kalirt_import` (`intrinsics/host_tests/crypto.rs:24`) | stop reading `u.length` | the test asserts the import and lowers without type inference; §4.2 pins the real `36` |
| `object/computed_member_static_name::run_refuses_a_computed_callee` | none | must pass unchanged; §3.1 emits the receiver first |
| the textcodec `String(a.length)` case (`cases/soundness/textcodec.toml:261`) | none | must pass unchanged; §3.3 |

### 4.4 Oracle cases

* **R-15.** The two verbatim-repro cases (`cases/oracle/tier2.toml:1486`, `:1500`;
  programs `:715-720`) keep their programs and flip `verdict` to `fail_closed`.
  Two new element-only cases (`p[1]` of the same split, module scope and in a
  function) assert `silent`, so the leaking half stays measured. §0.2's row
  becomes `FAIL_CLOSED + SILENT`.
* **R-63** (§6.1). An oracle pair, module scope and in a function, for
  `o.a.length`. Filed asserting `silent` against the baseline, then flipped to
  `fail_closed` in the commit that fixes it.

Both are enforced by
`kali_blast_radius::oracle_tests::every_zero_two_row_is_the_class_set_its_live_cases_assert`
(`crates/kali_blast_radius/src/oracle_tests.rs:132`). A §0.2 row's class set must
equal the verdicts its live cases assert.

### 4.5 The gate

* `bash scripts/test-gate.sh` after every task.
* `cargo clippy --workspace -- -D warnings` and `cargo fmt --all -- --check`
  before pushing. The gate runs neither.
* **Final check:** diff the failing-test set against the baseline. Every change
  must be one listed in §4.3 or §4.4. Any other test moving from pass to refusal
  goes back to the human partner, whose standing rule is "turn a working program
  into a refusal only if measured tiny".

---

## 5. Non-goals, each with the reason

* **`(new Array(3)).length`.** The picked item's predicate rung. With a loud floor,
  narrowing it is fail-closed for `.length`, but unmeasured, and it would refuse
  `[f()].length`. Filed (§6.3).
* **Other consumers of the negative-space predicate.** Spike 1 shows their floors
  are silent. Each needs its own floor before its predicate can be narrowed.
* **Inline allocation arguments reading zeros.** A different rung, the call-site
  argument lane. Filed.
* **The parser's `new` precedence.** A parser change that alters the LIR shape of
  every `new X(…).m()`. Filed.
* **R-31** (`render_static_value`'s child-count value arm). A separate register
  entry with its own sinks.
* **R-15's element half.** Still silent. Kept measured by §4.4.
* **Removing the redundant per-hazard bails.** A second kind of change. Filed.

---

## 6. Ledger obligations

### 6.1 The register

1. **File R-63**: "`.length` renders a node's child count, or `0`, for a receiver
   with no length lane". Silent, both scopes, measured against the baseline (§2.5,
   §2.5.1). Its lanes are the member receiver, the `let` receiver, the element
   receiver, the in-function `let` array, and the awaited `Promise.all` /
   `allSettled` result. Root-cause group: decided under the register's §3
   criteria and recorded with the reasoning. G3, which the member-length followup
   named, no longer exists.
2. **Retire R-63** in the commit that lands §3, with its oracle pair flipped
   (§4.4). Give it exactly the artifact footprint R-59 carries as a retired entry.
   At the baseline, R-59 appears as a keyed record in
   `tools/blast-radius/predicates.json` and `tools/blast-radius/counts.json`; in
   `tools/blast-radius/matchers.mjs`, `matchers.test.mjs` and `count.mjs`; and in
   `crates/kali_blast_radius/src/manifest_tests.rs`. `clusters.json` names it only
   in prose, with no keyed row, because a retired entry leaves the SILENT filter.
   Re-read each of those sites when filing, rather than copying this list.
3. **R-15's §0.2 row**: `SILENT` → `FAIL_CLOSED + SILENT`, recording that the
   `.length` half now refuses without the entry being fixed (the R-12/R-32
   precedent: a row can leave a class without the behaviour being fixed).

### 6.2 Followups corrected, each with a dated notice at the top

1. **`member-length-renders-the-child-count.md`**: closed by R-63's retirement.
   Its §4 pin list points at the renamed case.
2. **`codegen-array-literal-predicate-is-still-negative-space.md`**: §4's
   "highest-priority" ranking is withdrawn and its §3/§4 prescription is marked
   falsified, citing §2.2. Record that the predicate does produce one measured
   wrong value (`(new Array(3)).length` → `1`, rung 1 and rung 3), still open. State
   the prerequisite: a consumer's floor must refuse before its predicate is
   narrowed.
3. **`release-tier-allocation-identity-discovered-defects.md`**: ranking-note item 1
   points at that correction.
4. **Dead citations.** Both 2026-09-10 documents cite
   `docs/superpowers/sdd/2026-09-10-release-tier-allocation-identity/task-{5,7,8}-report.md`,
   and none of those files exists in the repository. The notices say so rather
   than leave the citations looking resolvable.

### 6.3 New: `docs/superpowers/followups/length-fails-closed-discovered-defects.md`

Ranked, most consequential first:

1. **Inline allocation arguments read zeros** (§2.6), with the repro table and the
   `call.rs:3579-3582` assumption it falsifies.
2. **The parser's `new` precedence** (§2.1): `new Array(3).length` → `2`, and every
   `new Array(n).fill(v)` in the benchmark fixtures parses as
   `new (Array(n).fill(v))`.
3. **`(new Array(3)).length` at the predicate rung**, with §3.5's reasoning.
4. **R-31's value arm** (`host.rs:928-942`), the sibling of §3.2's fallbacks,
   cross-referenced to R-31 and not re-filed.
5. **The redundant per-hazard bails** (§3.4).

### 6.4 Regeneration

If `counts.json` moves, run `tools/blast-radius/README.md`'s order (`accepts.mjs` →
`count.mjs` → `cargo run -p kali_blast_radius --example rank`), then
`cargo test -p kali_blast_radius`. `spliced_document_matches_the_generator` and the
§4.4 class-set gate go red on any drift, by design.

---

## 7. Risks, and what would falsify this design

| risk | what it would look like | response |
|---|---|---|
| **A consumer folds its own number** | §4.1's matrix: a must-refuse receiver prints a value | Stop gate. Extend the design with the human partner before production code |
| **A rung nobody mapped** | a §4.2 must-refuse pin still prints a number after §3 lands | §2.3's ladder is incomplete. Re-open it; do not stack a second patch on a wrong map |
| **Receiver-first emission loses a specific message** | `run_refuses_a_computed_callee` fails on its needle despite §3.1's ordering | Some path reaches the floor without emitting the receiver, or pushes the floor's diagnostic first. Find that path and fix the ordering. Never loosen the needle |
| **`randomUUID().length` refuses for real** | §4.2's `36` control fails | §2.5.2's reading is wrong. It is a capability loss: fix the lane order, never re-pin |
| **§3.3 admits too much** | `String(o.a.length)` compiles | The proof is keyed wrong; it must refuse (§4.2) |
| **More losses than spike 3 measured** | §4.5's final diff shows another pass → refusal | Back to the human partner under the "measured tiny" rule |
| **Ranking churn** | regeneration moves bands unexpectedly | Record it in the ledger. Never hand-edit a generated region |
| **Node version drift** | an oracle case fails on a node upgrade | Expected; a real divergence signal. The floating version is recorded, not pinned |

---

## 8. Process notes carried into the plan

* `scripts/test-gate.sh` runs neither clippy nor fmt. Both run explicitly before
  pushing; `GATE OK` is not CI-green.
* One cargo target directory. No extra worktrees and no extra target
  directories: they have exhausted this pod's disk and killed a run.
* "Before" numbers come from the baseline measurements recorded here at
  `152fdd5364`, not from rebuilding old commits.
* The incremental cache key carries compiler identity (`e69869356f`), so a local
  green run is trustworthy on a compiler-semantics branch.
* The spikes' probe scripts are not committed; their programs become §4.2's cases.
