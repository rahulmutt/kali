# Defects and instrument gaps the release-tier-allocation-identity project found and did NOT fix

**Filed** 2026-09-10, by the **release-tier-allocation-identity** project
(`docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md`,
`docs/superpowers/sdd/2026-09-10-release-tier-allocation-identity/`), Task 8,
Part B — on the convention this repository already uses for
`console-render-unification-discovered-defects.md` and
`computed-member-static-name-discovered-defects.md`: a project that measures
more than it fixes writes down what it left, so a later reader does not read
the silence as absence.

**Do not read this document as an ordered severity list across its own
sections — read the ranking note below first.** It groups everything Task 8
was told to file that isn't the register entries (§2 of that separate file),
the optimizer followup's own corrections, or the alias-mutation hole — all
three of which have their own documents.

## Ranking note — read this first

The two most consequential findings this project made are **not** in this
document:

1. **The negative-space array-literal predicate is a TRIPLET, not a single
   mistake.** Filed as its own document,
   `codegen-array-literal-predicate-is-still-negative-space.md`, because it is
   the same root cause this whole project fixed, still live, one crate away,
   on the exact code path (declarator initializers) that caused the headline
   defect. **This is the single most important thing to read out of Task 8's
   filing**, and it says so itself, with the reasoning.
2. **Which of the three target Benchmarks Game fixtures Stage 1 actually
   restored** — one of three (`fannkuch-redux`), not all three. Filed as a
   new "Status as of Task 8" section inside
   `release-mode-optimizer-inlines-an-allocating-initializer.md`, because it
   is a direct continuation of that document's own §5/§6 narrative, not a
   separate finding.

Everything below is real, measured, and worth a reader's attention, but none
of it is *this project's own root cause* the way the two items above are.
Ranked within this document, most consequential first: **§1 (a compiler
crash) and §2 (a real, previously-unmeasured refusal)** are defects a
compiler user could hit today and should be prioritized over the rest;
**§3-§6 (silent wrong-value classes)** are large in count but every one of
them is confirmed pre-existing at this branch's baseline and unrelated to
this project's own change, so their priority is "someone should own this,"
not "this project regressed something"; **§7-§8** are unloadable-module
refusals, a build-quality issue rather than a correctness one; **§9 (process
guidance)** and **§10 (pre-existing clippy)** are not defects at all and are
filed last on purpose.

---

## §0. How these were found

`crates/kali_cli/tests/inprocess/benchmark_execution.rs` (new this project,
Task 5) compiles and **executes** all 68 benchmark-fixture-family programs at
all three build tiers, against `node v26.8.2`. The harness it replaces,
`assert_optimization_benchmark_fixture` (`crates/kali_cli/tests/runtime_smoke.rs`),
counted instructions in the emitted `.wasm` and **never ran the module** — a
build emitting zeros, or an unloadable module, measured as a passing
benchmark. Its verdict on the new gate: of 68 fixtures with committed
metadata, **22 `Runs`** (build, run, and agree with node at all three tiers),
**1 `RefusedEverywhere`** (`array-literal-arguments-benchmark-v1`, a genuine
compile-time refusal at every tier, unrelated to this project), and **45
`KnownBroken`** — measured, pre-existing brokenness the old harness could
never see, now pinned so a fix shows up as a red test demanding
reclassification (`task-5-report.md`, final `Expectation` counts). Every
finding in §1-§8 below is data this gate surfaced; none of it was invented,
and none of it was chased down to a fix — that was explicitly out of scope
for a test-writing task (`task-5-report.md`'s own self-review says so).

---

## §1. `binary-trees-benchmark-v1` crashes the compiler — a native stack overflow at `--release`/`--release-advanced`, during COMPILATION

**Measured** (`task-5-report.md`, "Important 2"): `kali build --release` on
the real, unwrapped, committed fixture crashes with **SIGABRT** (`fatal
runtime error: stack overflow, aborting`), exit 134. `kali build
--release-advanced` crashes identically. Isolated to compilation, not
execution: calling `compile_wrapped` alone for `--release`, with no execution
whatsoever, still crashes — this is in `compile_source_file`'s own optimizer,
not `RuntimeCtx`/wasmtime. Confirmed independently via an external
subprocess (a crash in a subprocess cannot bring down the parent test
process, which is how the gate itself stays safe while measuring it).

**Why it went unnoticed.** `kali run`'s own dedicated acceptance test
(`clbg_binary_trees_runtime.rs`) always compiles at `BuildMode::Fast`, which
does not crash, and has therefore never reached this. It was found only once
the fixture's own `.policy.json` was loaded by the new gate, which got
execution past a fuel wall that had been separately masking whether the
release tiers could even be reached (`task-5-report.md`, "Important 2" —
under the DEFAULT, no-policy budget, `--release` still crashed, so the
crash is unconditional and not caused by the fuel policy).

**Not this project's defect.** It reproduces on `kali build --release`
directly, compiling `binary-trees-benchmark-v1`'s deeply recursive
`bottomUpTree`/`itemCheck` shape at N=21 — unrelated to array allocation
identity, and not touched by anything this project's Stage 1 changed.

**Priority.** This is the most severe item in this document: a native crash
compiling a legitimate, committed CLBG fixture is worse than any refusal or
wrong value, because it can bring down whatever process invoked the
compiler in-process (the reason `benchmark_execution.rs` gives this fixture
its own special-cased, subprocess-isolated handling rather than compiling it
in-process like every other fixture). It deserves its own followup document
and priority attention from whoever picks this up next; this project did not
investigate the root cause (out of scope for a test-writing task), only its
blast radius (compile-time, tier-gated to release, independent of
execution) — see `task-5-report.md`, Concern 5.

---

## §2. `fasta-benchmark-v1` refuses at both release tiers over module-level `const` closures — a real, previously-unmeasured defect

**Measured** (`task-5-report.md`, "Important 3"): compiled under
`ApiSurface::Node` (the surface the fixture actually needs — a hardcoded
`ApiSurface::Deno` had been masking this as an unrelated `undefined
identifier 'process'` artifact before this project's harness fix), `--fast`
builds and runs, matching node's header lines; `--release`/`--release-advanced`
refuse with:

```
error[E5506]: reading module binding 'ALU'/'IUB'/'HomoSap' from a function is only available for compile-time-constant const initializers
```

plus an `E3100` fallback warning for `.join`. Reclassified to
`Expectation::KnownBroken` — Fast does not fully agree with node either (a
pre-existing, unrelated `console.log(undefined)` -> `"0"` artifact on the
wrapped tail, §5 below), and Release/ReleaseAdvanced genuinely refuse.

**Not this project's defect.** It is about closing over module-level `const`
bindings from inside a function, unrelated to arrays or to allocation
identity. A real, tier-split, previously-unmeasured defect
(`task-5-report.md`, Concern 6) — the old build-only harness could not see it
because it never ran the fixture at any tier against a correct API surface.

---

## §3. A large silent wrong-value class: 33 of 60 swept fixtures print a small wrong value at Release/ReleaseAdvanced only, no diagnostic

**Pre-existing at baseline `eec408d000`** — independently reconfirmed there
for `math-benchmark-v1` via `git checkout <rev> -- <paths>` in the same
working tree, restored immediately after (`task-5-report.md`). Not caused or
fixed by this project. Reproduced here as data, per the source report's own
framing ("Ruling 3: findings as data, not prose"):

| stem | tier pattern | expected (node) | actual (kali) |
|---|---|---|---|
| `algebraic-simplification-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `24` | `1` |
| `boolean-literal-arguments-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `36` | `1` |
| `branch-specialization-repeat-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `36` | `1` |
| `call-inlining-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `24` | `1` |
| `call-inlining-chain-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `27` | `1` |
| `const-array-element-access-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `23` | `1` |
| `const-object-property-access-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `35` | `1` |
| `dead-inlined-function-pruning-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `39` | `1` |
| `duplicate-pure-expression-elimination-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `140` | `1` |
| `identity-chain-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `38` | `1` |
| `integer-like-object-enumeration-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `21` | `1` |
| `math-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `24` | `1` |
| `math-benchmark-v1-js` | Fast OK; Release=ReleaseAdvanced wrong | `24` | `1` |
| `math-variant-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `24` | `1` |
| `math-variant-benchmark-v1-js` | Fast OK; Release=ReleaseAdvanced wrong | `24` | `1` |
| `multiplication-by-one-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `24` | `1` |
| `nested-call-inlining-chain-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `42` | `2` |
| `numeric-literal-arguments-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `151` | `4` |
| `object-enumeration-alias-chain-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `9` | `1` |
| `object-enumeration-alias-chain-benchmark-v1-js` | Fast OK; Release=ReleaseAdvanced wrong | `9` | `1` |
| `object-enumeration-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `9` | `1` |
| `object-enumeration-const-bound-literal-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `9` | `1` |
| `object-enumeration-delete-reinsert-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `15` | `10` |
| `object-literal-property-order-canonicalization-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `9` | `1` |
| `object-literal-property-order-canonicalization-benchmark-v1-js` | Fast OK; Release=ReleaseAdvanced wrong | `9` | `1` |
| `object-string-enumeration-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `12` | `1` |
| `reflect-own-keys-alias-chain-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `3` | `1` |
| `reflect-own-keys-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `3` | `1` |
| `reflect-own-keys-const-bound-literal-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `3` | `1` |
| `specialization-reuse-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `114` | `3` |
| `string-concatenation-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `start-ahead-of-time-end` | `1` |
| `template-literal-concatenation-benchmark-v1` | Fast OK; Release=ReleaseAdvanced wrong | `start-ahead-of-time-end` | `1` |
| `template-literal-concatenation-benchmark-v1-js` | Fast OK; Release=ReleaseAdvanced wrong | `start-ahead-of-time-end` | `1` |

Every row is exit 0, no diagnostic, at both release tiers. Not a wrapper
artifact — reproduces identically with an intermediate `const r = hot(1, 2);
console.log(r);` binding, not just the direct `console.log(hot(1, 2))` form
(`task-5-report.md`). None of these fixtures involve array allocation; this
project did not investigate the mechanism behind the class (out of scope),
only confirmed it is real, pre-existing, and now pinned as `KnownBroken` so a
future fix surfaces as a red test. **This is the largest single finding by
fixture count in this document** — 33 of 60 swept fixtures — and is exactly
the kind of measurement the old build-only harness structurally could not
make.

---

## §4. Two silent miscompiles and a sibling predicate bug in `Object.fromEntries`

**Both pre-existing, confirmed at the pre-Task-2 baseline `36140ec0fa`** (via
`git checkout --detach`), and unrelated to `is_array_literal`/
`is_materializable_element` (`task-2-report.md`, round 1 and round 2
findings).

1. **A bare `Object.fromEntries(...)` result, read as an ordinary property,
   returns the wrong value at all three tiers, no diagnostic:**

   ```js
   const k = "a";
   const o = Object.fromEntries([[k, 1]]);
   console.log(o.a);
   ```

   kali prints `0` at `--fast`, `--release`, and `--release-advanced`; node
   prints `1`. Likely mechanism (not fully traced — recorded as measured,
   not chased): a bare `Object.fromEntries(...)` call used only as a normal
   expression value (not the operand of a recognized `has_own`/`keys`/
   `values`/`entries`/`ownKeys` call) is never folded into an object literal
   at `--fast` (the fold that does that only runs for Release/
   ReleaseAdvanced), so it reaches codegen unresolved and gets the `E3100`
   zero-placeholder fallback — but no `E3100` diagnostic actually printed for
   this exact shape, which `task-2-report.md` flags as itself worth
   attention.

2. **`is_object_literal` (`crates/kali_optimize/src/layout.rs:150-173`)
   unconditionally declines a zero-property object shape**, via its own
   `children.is_empty()` guard — mechanically the same defect this project
   fixed in the sibling predicate (`is_array_literal`), left unfixed here
   because it is a *different* function with its own ~eight call sites this
   project did not audit. Consequence: `Object.keys(Object.fromEntries([]))`
   fails `error[E5506]` at **all three tiers, including `--fast`** —
   `fold_object_from_entries_call` now successfully materializes `[]` into an
   empty-children `Value` node (this project's round-2 fix restored that),
   but `is_object_literal`'s own empty-children rejection declines the
   materialized result before `Object.keys` can fold it. Confirmed on the
   pre-Task-2 baseline `36140ec0fa` too — not a regression from this
   project.

**Not filed as their own register entries by this task** — both are
pre-existing, both are traced only partway (a plausible mechanism for #1, an
exact call site for #2), and both would need the same TDD discipline Stage 1
used (a failing test first) before a fix, which is a task of its own. Recorded
here so they are not lost.

---

## §5. `console.log(undefined)` prints `"0"`, not `"undefined"`

**Pre-existing.** Surfaced by Task 5's harness because
`fannkuch-redux-benchmark-v1`'s wrapped tail (`console.log(<already-printing-call>)`)
exposed the call's `undefined` return value once the wrapper was applied —
which is also why the harness stopped wrapping the three Benchmarks Game
fixtures at all (they already `console.log` their real output internally;
wrapping them was itself a defect in the harness, fixed this round —
`task-5-report.md`, "One additional fix"). Not a register-worthy new finding
in its own right (the register's existing `R-21` entry — "there is no
`undefined` value distinct from scalar `0`" — already covers this class); noted
here because it is exactly this shape and a reader tracing why fannkuch's
wrapped comparison needed a harness fix should find the explanation.

---

## §6. Three BigInt fixtures are wrong even at `--fast`

**Pre-existing**, missing the `n` suffix in every rendered value:
`bigint-addition-chain-benchmark-v1` (expected `112n`, kali prints `112` at
Fast and `3` at Release/ReleaseAdvanced), `bigint-literal-arguments-benchmark-v1`
(same), `bigint-multiplication-chain-benchmark-v1` (expected `93312n`, kali
prints `93312` at Fast and `1` at Release/ReleaseAdvanced). See
`task-5-report.md`'s compact table for the full tier pattern. Unrelated to
array allocation identity; the `--fast` divergence is a rendering gap (the
`n` suffix), the release-tier divergence is a further, separate wrong value
on top of it.

---

## §7. `math-ceil-benchmark-v1` / `math-trunc-benchmark-v1` (and their `-js` siblings) build but emit UNLOADABLE modules at `--fast` and `--release`

**Measured** (`task-5-report.md`): all four stems fail to load
(`error[E4201]`, `wasm[0]::function[41]`) at `--fast` and `--release`; only
`--release-advanced` loads and agrees with node (`24`). **These are in the
measured 59** the old, build-only harness counted — so it reported all four
as passing benchmarks, because it never tried to load or run the module. Not
caused by this project; unrelated to array allocation.

---

## §8. `mandelbrot-benchmark-v1` has no possible node oracle and independently hits E4201 at both release tiers

**Measured** (`task-5-report.md`, "Important 1"): the fixture uses the
Kali-only `Kali.writeStdoutBytes`, so no `node` run can serve as an oracle for
it at all. `--fast` matches the committed golden `.pbm` exactly
(`outcome.stdout_bytes` compared directly against
`mandelbrot-benchmark-v1.expected.pbm`). `--release`/`--release-advanced` fail
to load (`E4201`, `wasm[0]::function[43]`/`[42]`). The gate's `KnownBroken`
arm was itself strengthened this round so a vacuous pin (accepting "node
failed, nothing was ever compiled" as sufficient) is no longer possible — see
`task-5-report.md`, "Important 1" for the harness fix that made this
measurement meaningful. Unrelated to array allocation identity.

---

## §9. Process guidance: size a migration on behaviour, not on compiler errors

**Not a compiler defect — a lesson from how Stage 2 was sized and correctly
abandoned**, worth recording as guidance for whoever plans the next migration
in this codebase (`task-7-report.md`).

The design plan's sizing gate for Stage 2 (representing array-literal-ness as
a real `LirNodeKind` variant instead of inferring it structurally) was:
"proceed if ≤15 sites named by `cargo build` as non-exhaustive patterns."
Adding the variant and building reported **3** sites — comfortably under the
bar, and the gate would have waved the task through.

**The true behavioural radius is 133 core sites across three crates**
(`kali_codegen`, `kali_optimize`, `kali_lir`), because every `match node.kind
{ ... _ => ... }` ending in a catch-all arm absorbs a new variant silently —
`cargo build`'s exhaustiveness check cannot see it, by construction. Including
sites reachable only by an empty array literal (a real, load-bearing shape —
see `layout.rs`'s own comment on `Object.fromEntries([])`), the number is
193. A further 68 test-construction sites (`alloc(LirNodeKind::Value)`, hand-
built LIR in test helpers) each need their own per-site decision and none of
them fails to compile if made wrong — some fail loudly, others silently stop
constructing the shape their test name claims.

**The plan's own file list was also wrong** in a way that matters for
whoever writes the next one: it named `crates/kali_hir/src/lowering/expression.rs`
as the tagging site, but `kali_hir` contains **zero** references to
`LirNodeKind` — the property is erased one level higher than the plan
believed. `kali_mir` (where array-ness is actually erased into
`MirNodeKind::Expr`, `crates/kali_mir/src/lower.rs:116`) is a required third
crate the plan never scoped at all.

**The guidance**: a "count the compiler errors after adding a variant"
sizing gate measures the wrong quantity for any change to a type used in a
non-exhaustive `match` with catch-all arms. The behavioural radius is a
function of how many recognizers *could* see the new variant, not how many
the compiler forces you to update — and the only way to find that is the
survey `task-7-report.md`'s §1b did: enumerate every occurrence of the old
representation's discriminating condition (here, `LirNodeKind::Value` with a
text-less guard) and classify each by whether the new variant could reach it.
If a future project in this codebase plans a similar representation change
(giving an inferred property its own tag), size it this way, not by compiler
error count. `task-7-report.md`'s own Concern 1 makes the same point and
asks for it to be "promoted out of this task's context and into the
project's standing rules" — this document is that promotion, until it is
picked up somewhere more permanent.

---

## §10. Pre-existing clippy failures, unrelated to this project — filed, not fixed

`cargo clippy --workspace --all-targets -- -D warnings` fails on six files
this project never touched:

- `crates/kali_cli/tests/soundness_console_multiarg.rs:58` (`needless_borrow`)
- `crates/kali_cli/tests/soundness_abort.rs:59` (`doc_lazy_continuation`)
- `crates/kali_cli/tests/browser_promise_any_bundle.rs:26` (`test_attr_in_doctest`)
- `crates/kali_cli/tests/browser_promise_any_harness.rs:26` (`test_attr_in_doctest`)
- `crates/kali_cli/tests/browser_reflect_own_keys.rs` (6 errors, `doc_lazy_continuation`)
- `crates/kali_blast_radius/src/manifest_tests.rs:301-303` (3 errors,
  `doc_lazy_continuation`) — **the sixth file, found by this task**. Every
  earlier task's `cargo clippy --workspace --all-targets` (without
  `--keep-going`) halted on the first compile error it hit inside
  `kali_cli`'s test targets and never reached `kali_blast_radius` at all
  (`task-2-report.md`, `task-3-4-report.md`, `task-6-report.md` each name only
  a subset of the five `kali_cli` files, for exactly this reason — none of
  them ran `--keep-going` far enough to see past `kali_cli`). Task 8 did, to
  get a complete picture rather than trust one halted run, and found this
  sixth pre-existing failure the same way.

**Confirmed pre-existing, at this branch's own baseline `eec408d000`,
directly** (not inferred from an earlier task's report): `git checkout
--detach eec408d000`, then `cargo clippy -p kali_cli --all-targets --keep-going
-- -D warnings` for the first five, and `cargo clippy -p kali_blast_radius
--all-targets -- -D warnings` for the sixth — every one of these six files
fails identically at that commit, with the same lint and the same line.
Checked back out to `release-tier-allocation-identity` immediately after; no
baseline content was left in the tree.

**Not fixed, on instruction** — a doc-lint sweep of six unrelated test files
would obscure this branch's actual diff, and none of the six was touched by
any task in this project (the sixth, `manifest_tests.rs`, was edited by
Task 8 itself — see `task-8-report.md` — but only by inserting a new doc
comment block later in the file, after this pre-existing paragraph; the
failure sits at the same lines 301-303 both before and after that insertion,
and the baseline reproduction above confirms its text is unchanged, not
merely its line numbers). Filed here as the single place a reader can find
all six named together, with the confirmation method recorded, rather than
scattered across task reports each naming a partial subset.
