# Console render unification — closing G8's rendering half

---

## OUTCOME 2026-08-16 (at `3a636f62fb`) — READ THIS BEFORE THE BODY

**The body below is the design as argued on 2026-08-15 and is NOT amended.** The
register's convention is supersede-and-strike, and a spec whose original argument
is still readable is more useful than one silently corrected — the reasoning that
produced a wrong prediction is the part worth keeping. This block lists every
claim the implementation falsified, with the artifact that now holds the truth.
Modelled on `blast-radius-ranking.md` §6's `AMENDMENT` blocks.

**The one-sentence outcome.** The unification landed and R-33 retired. **R-30 did
not retire, and was never going to on the evidence this spec had**; the ranking's
G8 kept R-30's 57 and its band 1 place; and the half of R-32 this spec declared
out of reach was partly closed by a change this spec did not connect to it.

| where | what the body claims | what is true at `3a636f62fb`, and where it is recorded |
|---|---|---|
| §2 Goals | "Retire **R-30** and **R-33**, and with R-30 close **R-08 residual 5**" | **R-33 retired; R-30 did not.** R-30's plain `var`-binding lane (`r30a`) is still SILENT and holds the entry open — §0.2's R-30 row. R-08 **residual 5 did close**, but not as a consequence of R-30 retiring: it closed when the two console formatters were unified, because its `??` operand carries a proven `ValueShape::Boolean`. §2's R-30 entry states the distinction in its own words: *"closes when this entry's FIX lands" was true; "closes when this entry closes" was not.* |
| §7 table | `r30a` (`var` binding): SILENT → **FIXED** | **Still SILENT.** `var b = true; console.log(b)` prints kali `1` / node `true` at `62b11a78c3` and at `3a636f62fb`. §0.2's R-30 row; the `r30a` cases in `tier4.toml`. |
| §7 | "All four R-30 lanes then measure FIXED, so R-30 **retires**" | **False twice.** The lanes do not all measure FIXED (see the row above), and R-30 does not have four lanes — it has **seven**, six FIXED and one SILENT. §0.2's R-30 row enumerates them. |
| §7.1 | "G8 loses R-30's 57, which is the bulk of its 65, so G8's position on the reachable axis moves" and "G8's membership shrinks to R-23, R-31 and R-32" | **G8 keeps the 57 and keeps reachable band 1.** Its frequency fell 65 → 59 and its membership shrank to R-23, R-30, R-31 — R-30 stayed and R-32 left, which is the opposite of the sentence's second half on both names. The ranking's §6 AMENDMENT 2026-08-16 records it and says so explicitly: *"G8 does not lose R-30's 57, because R-30 does not retire."* |
| §8 | R-32's `1e21` half — "the value never reaches `float_to_string` in any lane … **this project cannot close it**" | **The direct-log bare-literal lane WAS closed**, by this project's own static-fold work (Task 6). `console.log(1e21)` now prints `1e+21`. The binding and concat lanes are still open. §7's R-55 entry carries the struck sentence and the per-lane re-measurement; §0.2's R-32 row carries the consequence. |
| §1.1 | the static fold "returns the literal text verbatim" | **It never saw the literal's text.** `kali_hir`'s `lower_literal_value` (`crates/kali_hir/src/helpers.rs:8`) had already rewritten the number with Rust's `Display for f64`, so `1e-7` reached the fold as `"0.0000001"` and `1e21` as `"1000000000000000000000"`. The fix is a re-parse and re-render, which is only intelligible once that is known. Recorded in `render_static_value`'s own comment and in §7's R-55. |
| §4 | `VALUE_TO_STRING_IMPORT_INDEX: u32 = 23` | **22.** `COVERAGE_HIT_IMPORT_INDEX` is 23; the six unconditional runtime helpers occupy 17 through 22. `crates/kali_codegen/src/lib.rs:88`. |
| §4 | the fold uses `format_js_number(parse_number_literal(text))` | It uses **`parse_numeric_literal_value`**, behind an `is_bigint_literal_text` guard that returns a BigInt's text unchanged — the digits of a BigInt too large for an `f64` must not round-trip through one. `crates/kali_codegen/src/intrinsics/host.rs:748-765`. |
| §7 item 1 | the guard is "a `String()`-result binding in single-argument `console.log` position" | It had to be a **REASSIGNED** binding (`let s = 0n; s = String(42n);`). The spec's proposed `const s = String(42n)` never reaches `string_result_render_taint` at all — `is_string_valued` proves a direct `String(<bigint-literal>)` call a string outright — so it could not have guarded the exemption it was written for. That program is kept under its own honest label as `r30g`; the actual guard pair is `r30e`/`r30f`. Both cases' rationales in `tier4.toml` carry the finding. |

**One thing the body got right that is worth naming, because it was a prediction
and not a hedge.** §7.1's *"The new bands are whatever the generator says — this
spec does not predict them"* held: the bands were regenerated, not argued, and the
regeneration is what caught the four claims above. The failure was in the
sentences that did predict, immediately underneath it.

**Two things filed after the body was written, which a reader of this spec should
know exist.** Neither is a correction to the argument below; both are findings the
project produced.

- **R-55** (§7) — the `1e21` family this spec routed out of the SILENT set. Its
  binding lane is still a wrong answer at exit 0 and, because §7 entries carry no
  §0.2 row, it has no oracle case and cannot go red. §0.2's movement paragraph now
  discloses that aggregate; splitting the entry is filed there as a follow-up.
- **R-56** (§2, Tier 2) — a NEW silent miscompile, found in the final whole-branch
  review by probing the double-quote gate this project narrowed. `Object.hasOwn`
  denies a string key the same program has just read. One cell of it regressed on
  this branch. Its 2x2, with a pre-branch column, is in §2 of the register.

---

## 1) Problem

`docs/superpowers/followups/blast-radius-ranking.md` is the first measured
ranking of the silent-miscompile register's frontier. On its reachable axis,
band 1 contains one cluster that is there **by measurement** rather than by tier
alone or by non-comparability: **G8 — per-sink rendering divergence: direct-log
and concat are separate formatters**, tier 2, frequency 65.

The register's §0.2 audit note records what that 65 is made of: it is "R-30's 57
and R-23's tier, not R-31's own count of 2". R-30 is the cluster's mass, and
R-30's own fix-cost read names the fix:

> because concat/template already render correctly, the missing piece is the
> direct-log argument path lacking the boolean repr the concat path already has,
> rather than a missing `Repr::Boolean` axis end to end.

R-32's entry names the same fix from the other side, and says why patching one
side is not enough:

> Two independent number formatters exist and they disagree. […] Any fix should
> unify them rather than patch one, or they will keep drifting.

This project does that unification, and closes the two G8 entries it retires
outright.

### 1.1 There are four renderers, not two

The register says "two formatters". The code has four, and they disagree because
each holds different information:

| renderer | where | what it knows | what it cannot do |
|---|---|---|---|
| static fold | `crates/kali_codegen/src/intrinsics/host.rs:717` `render_static_value` | the literal's own source text | render a number the way JS does — it returns the literal text verbatim |
| codegen ladder | `crates/kali_codegen/src/emit/operators.rs:1826` `emit_as_string` | the repr/shape (`Boolean`, `Float`, proven `String`) | render a value it cannot statically prove — its terminal `int_to_string` corrupts an unproven string handle |
| wasmtime host | `crates/kali_runtime/src/host/io.rs:22` `format_console_value` | the **runtime** string-handle tag | distinguish a boolean from an integer — no repr survives the call |
| browser harness | `crates/kali_runtime_contract/src/browser/harness.rs:180` `formatConsoleValue` | the same runtime tag | the same |

They are not four accidents. They are four positions relative to one proof
boundary, and **no single position holds enough information to render
correctly**:

- The host has the runtime tag, so it renders *any* string handle correctly —
  including ones codegen cannot prove. It has no repr, so `true` is `1`.
- Codegen has the repr, so `emit_as_string` renders booleans and floats. It must
  prove stringness statically, and where it cannot, its terminal `int_to_string`
  produced the measured `-9223354444668731387` for `'hello'` — which is why
  `string_result_render_taint`
  (`crates/kali_codegen/src/emit/call.rs:4754`) fails those closed instead.

That is the root cause of G8, and it is a better statement than "two formatters
drifted": the drift is downstream of an information split that nothing in the
current design closes.

### 1.2 The naive unification regresses

`emit_console_argument` (`crates/kali_codegen/src/emit/call.rs:23`) carries a
standing warning aimed directly at this project:

> Stage P5 T-new-E note: the SINGLE-argument console lane hands the host a raw
> tagged i64 and lets IT render — the host decodes a string-handle tag and prints
> the text, so `console.log(s)` for a `String()`-result binding prints correctly
> (measured `1` on parent, matching node) and **must NOT be tainted here**.

So routing the single-argument lane through `emit_as_string` as it stands would
take values that print correctly today and either fail them closed (the taint) or
render them as 20-digit integers (the terminal arm). **Unification is only safe
once the ladder's terminal arm can do what the host does.** That ordering is the
whole design.

### 1.3 R-32 is not one defect

Measured 2026-08-15 with `/workspace/.cache/cargo-target/debug/kali`, a debug
binary dated 2026-08-14 — one commit behind HEAD `8974cc6b57`, so these are
scoping measurements and are re-taken as oracle cases in Task 1:

| program | kali | node |
|---|---|---|
| `console.log(1e-7)` — literal | `0.0000001` | `1e-7` |
| `var y = 1e-7; console.log(y)` — binding | `1e-7` | `1e-7` |
| `console.log("v=" + 1e-7)` — concat | `v=1e-7` | `v=1e-7` |
| `console.log(1e21)`, and via a binding | `1000000000000000000000` | `1e+21` |
| `console.log("v=" + 1e21)` | `error[E4201]: failed to load WASM module` | `v=1e+21` |
| `var b = true; console.log(b)` | `1` | `true` |

`format_js_number` (`crates/kali_runtime/src/host/imports_default.rs:981`) is
`ryu_js`, which implements ECMAScript `Number::toString` and therefore has both
thresholds. So the two halves of R-32 have different causes:

- **The `1e-7` half is a lane problem.** The binding and concat lanes reach
  `float_to_string` and are correct; the bare literal goes through the static
  fold, which returns the literal's own text. In scope.
- **The `1e21` half is not a rendering problem.** The value never reaches
  `float_to_string` in any lane, and in concat position it emits invalid wasm.
  It is a literal-classification/emit defect, it is **not silent**, and this
  project cannot close it. See §8.

## 2) Goals and non-goals

**Goals**

- One rendering rule, structurally enforced: after this project there is no path
  by which the runtime host renders a value, so the host and the ladder cannot
  drift apart again.
- Retire **R-30** and **R-33**, and with R-30 close **R-08 residual 5**, which
  the register records as blocked on exactly this fix.
- Move R-32's literal lane, and record the half that does not move.
- Make the two console lanes agree with each other, not only with `+` (§5.1.1).
  This is the project's one deliberate behavior widening.
- Regenerate `blast-radius-ranking.md` from the moved verdicts — the first real
  exercise of its own §6.6 item 4, "re-run, do not re-read".

**Non-goals**

- **R-31** (array→length, object→0). Needs a node-compatible inspect formatter;
  a different mechanism and a different project.
- **R-23** (`typeof`→`0`). A value defect, not a rendering one. §6.3 of the
  ranking flags it as the assignment that decides band 1's shape; leaving it
  untouched means the ranking's contested cluster stays contested, which is
  stated rather than hidden.
- **R-34.** The register states in terms that it does not close with R-30.
- **Relaxing `string_result_render_taint` at the `+` sink.** §5.2.
- Any change to what kali *accepts*. This project changes rendering only.

## 3) The rule

**Every value rendered for output goes through one ladder, and the ladder's
terminal arm can always ask the runtime.**

```
emit_as_string(id, sink):
  proven String      -> handle passthrough        (unchanged)
  shape Boolean      -> interned "true"/"false"   (unchanged)
  Float / is_float   -> float_to_string           (unchanged)
  otherwise          -> sink == Concat  ? int_to_string   (unchanged)
                      : sink == Console ? value_to_string (NEW)
```

`value_to_string(i64) -> i64` is a host import running today's
`format_console_value` logic — decode the string-handle tag, else render the
integer — and interning the result with `alloc_guest_string`, exactly the shape
of `int_to_string` and `float_to_string`.

**The terminal arm is selected by sink, not by a proof, and that is deliberate.**
See §3.1: making `value_to_string` unconditional would widen a live hazard.
Selecting by sink makes the change strictly behavior-preserving on both paths —
the `+` ladder is untouched, and the console path's terminal arm becomes exactly
what the host already does for that same value today.

With that arm present, `emit_console_argument` routes through `emit_as_string`
like its multi-argument twin already does, and the runtime-decode guarantee the
single-argument lane provides today **survives the move** instead of being traded
for repr knowledge.

The static fold's numeric Literal arm takes the same rule: `format_js_number`
rather than the literal's source text.

### 3.1 Why the terminal arm is not unconditional

`STRING_HANDLE_TAG` is `0x8000_0000_0000_0000`
(`crates/kali_runtime/src/host/memory.rs:291`) — **the sign bit**. Every negative
i64 carries it, so `format_console_value` attempts a handle decode on every
negative integer it is given.

It survives that today by accident of bounds-checking: for `-5` the decode
computes offset `0x7fff_ffff` and a negative length, `read_guest_bytes` fails,
`.ok()` yields `None`, and the integer fallback renders `-5`. Measured
2026-08-15 — `console.log(-5)`, `console.log(n)` and `console.log(m)` for
`n = -5`, `m = -1234567` all match node.

So the fallback works **by bounds-check failure**, and a negative integer whose
bit pattern decoded to a valid in-bounds guest range would render as garbage
bytes instead. That hazard is pre-existing and live in the single-argument
console lane. This project does not fix it, and — the load-bearing part — **does
not widen it**: `value_to_string` is reachable only where `format_console_value`
is reachable today. An unconditional terminal arm would have extended it to every
non-proven-integer reaching `+` and template literals, which is a much larger
population, so the sink parameter is what keeps the blast radius of this change
at zero on that axis.

Recorded as a follow-up in its own right: the tag scheme has no bit available
that a negative i64 cannot set, so distinguishing a handle from a negative
integer needs a representation change, not a guard.

### 3.2 Why this is structural and not another patch

The multi-argument lane is the existence proof. Its doc comment
(`crates/kali_codegen/src/emit/call.rs:68`) says it was built on `emit_as_string`
so that console rendering "agrees with `+` rendering by construction instead of
via a second hand-mirrored ladder". This project extends that property to the one
lane that was left out, and removes the reason the exception existed.

After the change, `console.log(x)` flows: codegen emits `x` → `emit_as_string`
selects an arm from repr → **a string handle in every case** → `console_log(handle)`
→ host decodes and prints. The host stops being a renderer and becomes a sink.

## 4) Components

**`kali_common`** — `format_js_number` moves here from
`crates/kali_runtime/src/host/imports_default.rs:981`, with the `ryu-js`
dependency. `kali_codegen` and `kali_runtime` both already depend on
`kali_common`, so this is what makes the static fold and the host agree by
construction rather than by mirroring.

**`kali_runtime`**

- `host/imports_default.rs`: register `value_to_string` `(i64) -> i64`; delete
  the `[warn] ` prefix at `:53`.
- `host/io.rs`: `format_console_value` is unchanged and becomes the shared core
  of both the console sinks and `value_to_string`.

**`kali_codegen`**

- `lib.rs`: `VALUE_TO_STRING_IMPORT_INDEX: u32 = 23`, **appended**. Import
  indices are hardcoded positional constants (`lib.rs:44-75`, currently through
  22); appending avoids index churn.
- `lower.rs`: declare the import with the existing **type 4** (`(i64) -> i64`,
  the same signature as `int_to_string`) — no new function type.
- `emit/operators.rs:1826`: `emit_as_string` gains the sink parameter and the
  `value_to_string` terminal arm.
- `emit/call.rs:23`: `emit_console_argument` routes through `emit_as_string` with
  `sink = Console`, taint-exempt per §5.1. Its `emit_console_argument_as_string`
  twin at `:75` passes the same sink, so both console lanes render identically —
  today they differ, because the multi-argument one goes through `int_to_string`.
- `intrinsics/host.rs`: `render_static_value`'s numeric Literal arm uses
  `format_js_number(parse_number_literal(text))` instead of `text.to_string()`.

**The JS import mirrors — there are four of them, not one.**
`emit_boolean_as_string`'s doc (`emit/operators.rs:1797`) states the constraint
that this project must obey, and states why it was avoided last time:

> adding an import would have to be mirrored across the four hand-maintained
> `kali:rt` JS import lists (host + browser bundle glue) or the browser lane
> fails with a `LinkError`.

The four are `crates/kali_runtime_contract/src/browser/harness.rs:398` and
`:964`, and `crates/kali_cli/src/bin/cmd_build.rs:1722` and `:2229`. With the
wasmtime host that is **five registration points for one import**, and missing
any of the four JS ones is a `LinkError` in the browser lane rather than a wrong
answer.

The addition is identical at all four, because every mirror already has both
helpers (`formatConsoleValue` at `harness.rs:180`/`:755` and
`cmd_build.rs:1895`/`:2402`; `allocGuestString` at `harness.rs:120`/`:640` and
`cmd_build.rs:1916`/`:2423`):

```js
value_to_string(value) {
  return allocGuestString(new TextEncoder().encode(formatConsoleValue(value)));
},
```

Note that **no** JS mirror emits the `[warn] ` prefix — it exists only in the
wasmtime host. So R-33's fix makes kali's own runtimes *agree* rather than
diverge further; the prefix is an internal inconsistency before it is a
divergence from node.

This is also the strongest argument for the design: `emit_boolean_as_string`
chose a `Select` over interned constants specifically to avoid paying this
five-point cost. That was right for a boolean, which has two possible strings.
It is not available for `value_to_string`, whose whole purpose is to consult
runtime state. The cost is paid once, here.

## 5) Three things that must not be lost

Ordered by how much damage losing them does.

### 5.1 The taint must not start firing on the single-argument console path

`emit_console_argument` is deliberately exempt from `string_result_render_taint`,
and that exemption is why `console.log(s)` prints correctly for a
`String()`-result binding. Routing through `emit_as_string` naively applies the
taint and converts working programs into `E5506` — a regression the register
would score as a **new fail-closed entry**.

So `sink = Console` selects the `value_to_string` arm *and* skips the taint deny,
preserving today's behavior byte-for-byte. The test for this is written **first**
(§7).

### 5.1.1 The multi-argument lane joins it, and this is the one widening

`string_result_render_taint`'s own doc lists its sinks as "`+`, template literal,
**multi-arg console** via `emit_as_string`, or arithmetic operator lowering". So
today the two console lanes disagree about the same value:

| program, for a `String()`-result binding `s` | today |
|---|---|
| `console.log(s)` — single-arg, taint-exempt | prints correctly |
| `console.log(x, s)` — multi-arg, tainted | fails closed `E5506` |

That is G8's own pattern one level down: two sinks, two answers, differing by
argument count rather than by anything about the value. Both console lanes
therefore take `sink = Console`, which makes "one rendering rule" true for
console rather than for one of its two lanes.

**This is the only behavior widening in the project, and it converts a
fail-closed into rendering.** It is called out separately from §5.1 for that
reason: §5.1 preserves behavior, §5.1.1 changes it. The change is safe on the
same grounds as §3.1 — `value_to_string` renders the handle the taint was
protecting from `int_to_string`, so the hazard the deny existed for is gone on
this path — but it is a widening and gets its own oracle case pinning
`console.log(x, s)` before and after, not a shared one.

### 5.2 The `+` path is untouched, terminal arm included

Because §3 selects the terminal arm by sink, the concat/template path keeps
`int_to_string` and keeps its taint exactly as they are. Nothing about `+`
changes, and no R-08/P5 behavior is widened.

This is worth stating positively rather than as an omission, because the obvious
next thought is wrong: `value_to_string` does **not** make the `+` taint
redundant. The taint guards an unproven handle from reaching `int_to_string`, and
at the `+` sink `int_to_string` is still the terminal arm. Routing `+` through
`value_to_string` too would relax the taint's *reason* — but it would also widen
§3.1's negative-integer hazard to the much larger `+` population, so it is not a
free follow-up and must not be filed as one. If it is ever taken up it is a
project with its own measurement, not a cleanup.

The cost of the deferral is one sink parameter on the ladder. That parameter is a
design feature of this change, not a leftover.

### 5.3 The object-reference `E5506` stays on the console path

The rejection exists twice today, once per console lane, and the duplication is
deliberate — `emit_console_argument_as_string`'s doc says it must apply to every
argument position, not just position 0. After unification exactly one copy must
survive on the console path, or R-31's object lane stops failing closed and
silently starts printing pointers.

## 6) Constraint: the do-not-modify files

`scripts/test-gate.sh`, `scripts/check-determinism.sh`, `mise.toml` and
`.github/workflows/ci.yml` are do-not-modify for agent work.

Nothing here touches them. The oracle cases are `.toml` files under the existing
`cases` binary, which the gate already runs — the same dividend of the
test-binary-consolidation project that the ranking project relied on.

## 7) Testing

**The verification already exists.** Every §2 register entry has live oracle
cases under `crates/kali_cli/tests/cases/oracle/` asserting a derived verdict
class, which fail when an entry moves. The success criterion is therefore a diff
in expectations, not a new harness:

| case | today | after |
|---|---|---|
| `r30a` (`var` binding), `r30c` (`const` object field) | SILENT | FIXED |
| `r30b` (`const` scalar), `r30d` (concat/template) | FIXED | unchanged |
| `r33a` (`console.warn`, observed on **stderr**) | SILENT | FIXED |
| `r33b` (`console.error` control, stderr) | FIXED | unchanged |
| `r32a` (past-threshold direct log) | SILENT | **splits** — see §8 |

All four R-30 lanes then measure FIXED, so R-30 **retires** under §3.4 of the
ranking's rule that an entry retires when every lane moves, not one. R-33 retires
the same way.

Three additions beyond the expectation diff:

1. **The regression test that matters most**, written before the unification: a
   `String()`-result binding in single-argument `console.log` position still
   prints correctly. This is the exact behavior §5.1 protects. Without it, the
   unification can trade a working lane for a fail-closed one and every other
   test still passes.
   Its pair, per §5.1.1: an oracle case for the same binding in **multi-argument**
   position (`console.log(x, s)`), pinning the current `E5506` first and the
   rendered output after. The two cases together are what make the widening
   legible as a decision rather than as a side effect — a reader diffing the
   project should be able to see exactly one fail-closed disappear, on purpose.
2. **`format_js_number` unit tests in `kali_common`** on the threshold pairs
   `1e20`/`1e21` and `1e-6`/`1e-7`, with `1e21` **pinned to its current wrong
   answer** and a comment naming why, so the half this project does not fix is
   recorded as measured rather than missed.
3. **Browser-lane parity** — a boolean log through the browser harness, or the
   wasmtime and browser hosts start disagreeing in the other direction.

**A lane that runs nothing is a failure, not a pass.** `check-determinism.sh` has
been green while executing zero tests since `2448dd8839`. Any filter this project
adds asserts a nonzero expected count.

### 7.1 The ranking document goes red until regenerated

`blast-radius-ranking.md` is pinned by
`kali_blast_radius::ranking::ranking_tests::spliced_document_matches_the_generator`.
Moving R-30's and R-33's verdicts changes the generator's inputs, so that test
fails until the document is regenerated by
`cargo run -p kali_blast_radius --example rank`.

Regenerating it is a **deliverable of this project**, not an afterthought. It is
the first real exercise of the ranking's own §6.6 item 4.

Two consequences to expect in the regenerated tables: G8 loses R-30's 57, which
is the bulk of its 65, so G8's position on the reachable axis moves; and G8's
membership shrinks to R-23, R-31 and R-32, whose tier is still R-23's. The new
bands are whatever the generator says — this spec does not predict them, because
predicting a generated table is the failure mode the ranking project existed to
end.

## 8) What stays open, and why

- **R-31** (array→length, object→0) — needs a node-compatible inspect formatter.
  Not attempted. The object lane keeps failing closed; the array lane keeps
  printing its length.
- **R-23** (`typeof`→`0`) — a value defect. Untouched, so the assignment §6.3 of
  the ranking calls the most consequential judgment call in the document remains
  unsettled.
- **R-32's `1e21` half** — the value never reaches `float_to_string` in any lane,
  and in concat position it emits invalid wasm (`error[E4201]`). **This needs its
  own register entry, filed under the register's §7 "Fail-loudly-but-wrong
  defects (not silent — recorded for completeness)"** — R-50's home, and the
  correct one for two reasons: it is a hard failure on valid JavaScript rather
  than a silent miscompile, and an entry there carries no §0.2 row, so it needs
  no oracle case and cannot trip
  `every_zero_two_row_is_the_class_set_its_live_cases_assert`. Filing it is a
  deliverable of Task 1, so the finding is not lost in this spec's prose. It does
  not belong in the ranking's SILENT set.
- **R-34** — the register states in terms that it does not close with R-30.
- **The `+` and template-literal lanes** — untouched, terminal arm and taint
  alike (§5.2). Note this is *not* filed as an easy follow-up: routing them
  through `value_to_string` would widen §3.1's negative-integer hazard to a much
  larger population, so it needs its own measurement.
- **The negative-integer handle-decode hazard** (§3.1) — measured, bounded, not
  widened, not fixed. The real fix is a representation change, since no bit is
  available that a negative i64 cannot set.

Net effect: G8 goes from five open entries to three, its measured mass retires,
and the cluster stops being a rendering-drift cluster. What is left of it is an
inspect formatter and a `typeof` value bug — two things that should probably not
share a cluster, which is itself evidence for §6.6 item 2's "trace G4, G7 and
G8".

## 9) Failure modes

**Silently trading a working lane for a fail-closed one.** The §5.1 hazard. A
fail-closed is not a neutral outcome here: the register scores it as its own
class, and a project that closes R-30 while opening an `E5506` on
`console.log(s)` has moved damage, not removed it. Mitigated by writing that test
first.

**Allocation failure.** `alloc_guest_string` returns `unwrap_or(0)` in
`int_to_string`, `float_to_string` and `string_concat`. `value_to_string` uses the
same convention rather than inventing a new failure mode for one import.

**Host/browser drift, again.** The whole point of the project is defeated if
`value_to_string` lands in the wasmtime host and not the harness. The browser
parity test in §7 is what prevents shipping half of it.

**Predicting the regenerated ranking.** §7.1. The tables are generated; this spec
states what changes as an input, never what the output will be.

## 10) Risks

**The sink parameter is a seam.** §5.2 leaves one ladder with two terminal arms
selected by the caller. That is a much smaller compromise than two ladders — the
first three arms, which are where all the repr knowledge lives and where all the
drift happened, are shared — but it is a seam, and §3.1 explains why closing it
is not obviously desirable. Recorded so a later reader can see it was chosen, not
overlooked.

**`emit_as_string` is load-bearing for `+`.** Even though the `+` path's
behavior is unchanged, the function itself is edited, and string concatenation is
one of the most heavily exercised paths in the compiler. The check is the full
`cargo test --workspace`, not a targeted filter.

**The negative-integer decode hazard stays live.** §3.1. This project measures
it, bounds it, and declines to widen it, but `console.log` of a negative integer
still reaches a handle decode that is only saved by a bounds check. A later
representation change is the real fix.

**The scoping measurements are one commit stale.** §1.3's table was taken on a
binary dated 2026-08-14 against HEAD `8974cc6b57`. Task 1 re-takes every one of
them as an oracle case at the project's own HEAD before any code changes.

**`value_to_string` retains guest memory permanently, and the body never priced
it. ADDED 2026-08-16, measured at `3a636f62fb`.** This risk is not a prediction:
it is a measurement taken after the fact, recorded here because §10 is where a
reader looks for the costs of the design and this one was missing.

`value_to_string` calls `alloc_guest_string`, which routes to **`__alloc_global`
— the never-reset arena** (`crates/kali_runtime/src/host/memory.rs:143`), not the
resettable `__alloc` that `string_concat_arena` uses. So every non-string,
non-float single-argument `console.log` now permanently retains guest bytes where
before this project it allocated **none** — the single-argument lane handed the
host a raw `i64` and the host formatted it into its own `String`.

Measured with `while (i < N) { console.log(i); i = i + 1; }`, peak child RSS
(`ru_maxrss`), the **same fixture on two binaries** — HEAD and one built from the
merge base `8974cc6b57` — because the host's stdout buffer grows with N on both
and a single-binary figure cannot separate the two:

| N | `8974cc6b57` | `3a636f62fb` | delta |
|---|---|---|---|
| 100 000 | 26 664 KiB | 27 112 KiB | +448 KiB |
| 1 000 000 | 38 440 KiB | 45 900 KiB | +7 460 KiB |

Over the 100k → 1M interval the two binaries grow by 11 776 KiB and 18 788 KiB,
so the retention this project added is **(18 788 − 11 776) KiB / 900 000 = 7.98
bytes per log** — which is exactly `alloc_guest_string`'s `(len + 7) & !7` slot
for a decimal string of eight bytes or fewer, i.e. the whole of the cost and
nothing else. A single-binary reading of the same runs gives ~21 bytes/log and is
wrong: it charges this project for the host's stdout buffer.

At **N = 4 000 000 both binaries abort** on `error[E4003]` (the default CPU-fuel
guard), at 71 476 KiB and 85 648 KiB respectively. **The abort is pre-existing and
is not caused by this change** — the guard is on CPU, not memory, and the
pre-branch binary reached a *higher* RSS because it got further through the loop
per unit of fuel. Do not read the 4M row as a memory ceiling.

**The consequence at exhaustion is a new silent-wrong-answer path on a lane that
previously had none — and this paragraph is REASONED FROM SOURCE, NOT MEASURED.
Read it as an argument, not as a row in the table above.** Every figure in the
table was run; nothing below was. The state it describes was never reached: the
default CPU-fuel guard fires long before guest memory is exhausted (see the 4M
row), so producing it would need a raised `resources.maxCpuTimeMs` and a run
nobody has done. It is written down because it is register-shaped, and it is
labelled because a document whose value is that its claims are measured must not
let an unmeasured one wear the same clothes.

The argument is three lines of source. `value_to_string` ends in
`alloc_guest_string(...).unwrap_or(0)` (`host/imports_default.rs:729`). Handle
`0` has no `STRING_HANDLE_TAG`, so `decode_string_handle_bytes` returns `None`
(`host/memory.rs:50`) and `format_console_value` falls through to
`value.to_string()` (`host/io.rs:29`) — the string **`"0"`**. So a print-heavy
program running long enough to exhaust guest memory would reach a state where
`console.log(i)` prints `0` at exit 0 with no diagnostic. Bounded and extreme.
`int_to_string`, `float_to_string` and `string_concat` share the `unwrap_or(0)`
convention (§9), so the convention is not new — what is new is a console lane
that reaches it.

## 11) Sequencing

1. **Re-measure and pin.** Re-take §1.3's probes as oracle cases at this
   project's HEAD. File the `1e21` finding as its own register entry (§8). This
   fires the "the measurements were stale" risk immediately if it is going to
   fire.
2. **The regression test first.** The §5.1 `String()`-result single-arg console
   test, written and green *before* the unification, so it can actually catch the
   regression it exists for.
3. **`format_js_number` to `kali_common`**, with its threshold unit tests.
4. **`value_to_string`** — the host import, the browser mirror, the import index
   and declaration. Not yet wired into the ladder.
5. **The terminal arm and the unification** — `emit_as_string` gains the sink
   parameter and the arm; both console lanes route through it with
   `sink = Console`, taint-exempt per §5.1 and §5.1.1.
6. **The static fold's Literal arm.**
7. **`[warn] `** — delete the prefix, update the four assertions that pin it:
   `kali_cli/tests/runtime_smoke/run.rs:11010` and `:12802`,
   `kali_cli/tests/runtime_smoke/test.rs:11180`, and
   `kali_runtime/src/execute_tests/host_env.rs:32`. The `[warn] ` strings in
   `kali_case_runner/src/steps_tests.rs` and `kali_blast_radius/src/verdict_tests.rs`
   are **simulated kali output feeding classifier fixtures, not assertions about
   kali**, and must be left alone — changing them would silently weaken the
   classifier's own tests.
8. **Regenerate** `blast-radius-ranking.md`, and amend the register's §0.2 rows
   for R-30, R-32 and R-33 with the commit they were re-measured at.

Steps 3 and 4 are independent. Step 7 is independent of everything and can land
in either order, but it is kept late so the unification's diff stays readable.
