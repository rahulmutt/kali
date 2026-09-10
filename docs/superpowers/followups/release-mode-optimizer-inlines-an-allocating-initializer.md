# ~~The optimizer inlines an ALLOCATING array initializer~~ The release tiers substitute a binding's initializer for its name, so `--release` loses the array identity `--fast` keeps

**Filed** 2026-09-09, at `71b5f42f6c`, by the **computed-member-static-name**
project (`docs/superpowers/sdd/2026-09-08-computed-member-static-name/`).
**This is the most consequential thing that project found, it is wider than that
project's surface, and it was not caused by it** — the project made it VISIBLE,
by turning a silent wrong value into a refusal that a test could see.

**Oracle:** `node v26.8.1`. **Baseline:** `dc19c3a040`. Controller ruling
**R22**.

> **CORRECTED 2026-09-10** by the **release-tier-allocation-identity** project
> (`docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md`,
> `docs/superpowers/sdd/2026-09-10-release-tier-allocation-identity/`), Task 8. The
> title and §1 named the wrong pass: there is no inlining involved. The
> substitution is **layout-binding specialization**
> (`crates/kali_optimize/src/specialize.rs:120-127`), reached because
> `is_array_literal` (`crates/kali_optimize/src/layout.rs`, then negative-space)
> let the `new Array(n).fill(v)` wrapper into the spec env at
> `specialize.rs:82`/`:109`. §5 and §6 below carry the rest of the correction —
> the R22 overturn and the mechanism-accurate account of what actually closed
> it. Nothing below this notice in §2-§4 needed correcting: the measured
> tables, the `--max-specializations 0` isolation and the E3100 warnings all
> describe layout-binding specialization correctly, they just didn't name it.

## 1. The claim, in one sentence

~~At `--release` and `--release-advanced`, the optimizer inlines a
`new Array(n).fill(v)` declarator initializer as if it were a pure constant, so
the array a later read indexes is no longer the same array the declarator
allocated. At `--fast` it is. Every consequence below follows from that one
difference.~~ **Corrected 2026-09-10**: at `--release` and
`--release-advanced`, **layout-binding specialization**
(`specialize.rs:120-127`) overwrites every read of a `new Array(n).fill(v)`
declarator's name with a **clone of the initializer node itself**
(`program.nodes[id] = program.nodes[bound].clone()`), because
`is_specializable_binding` — delegating to `is_array_literal`'s then
negative-space definition, "a text-less `Value` that is not an object
literal" — let the allocating initializer into the specialization environment
in the first place (`specialize.rs:82`/`:109`). So the array a later read
indexes is no longer the same array the declarator allocated; the substitution
duplicates an *allocation*, not a constant. At `--fast` the specialization
block never runs at all (`driver.rs:186-190` matches `{}` for `Fast`/`Default`),
so the name is never rewritten. Every consequence below follows from that one
difference; "inlining" was never the right word for step 4 of this chain —
inlining moves a call body to its call site, and nothing here does that.

## 2. What is measured, and by whom

**Established by the controller, by dumping LIR on both sides of the
optimizer** (this is the part that identifies the mechanism, and it is recorded
here because the dump is not reproducible from a committed artefact):

* At `--fast`, the read base of `u[i]` is the **named binding `u`**.
* At `--release`, it is a **text-less wrapper around the same node id as the
  declarator's `new Array(n).fill(v)` initializer**. There is no shared array
  left to name, and every refused read carries `base_text=None,
  target_name=None`.

**Also established by the controller**, by reinstating the pre-project name
fabrication behind a temporary env gate and running the result — i.e. by
emulating what the release tiers had been emitting BEFORE this project:

| program | pre-project kali at `--release` | node |
|---|---|---|
| `const u = new Array(4).fill(7); let i = 0; u[i]` | `0` | `7` |
| an 11-line repro of the same shape | `0` | `1` |
| the array-parameter form | `0` | `8` |
| `spectral-norm-benchmark-v1.ts` | **invalid wasm** | `1.274219991` |
| the same spectral-norm at `--fast` (control) | `1.274219991` | `1.274219991` |

**Re-measured for this document at `71b5f42f6c`**, on
`.cache/cargo-target/debug/kali`, in a scratch directory:

| fixture / program | `--fast` | `--release` | `--release-advanced` |
|---|---|---|---|
| `spectral-norm-benchmark-v1.ts` | builds; `kali run` prints `1.274219991` | **exit 1**, 16 × `E5506` computed-member, 6 × `E3100` | **exit 1**, 12 × `E5506`, 5 × `E3100` |
| `nbody-benchmark-v1.ts` | builds; `kali run` prints `-0.169075164` / `-0.169087605` | **exit 1**, 6 × `E5506` | **exit 1**, 6 × `E5506` |
| `fannkuch-redux-benchmark-v1.ts` | builds; `kali run` prints `228` / `Pfannkuchen(7) = 16` | **exit 1**, 20 × `E5506`, 2 × `E3100` | **exit 1**, 10 × `E5506`, 1 × `E3100` |
| `const u = new Array(4).fill(7); let i = 0; console.log(u[i]);` | builds | **exit 1**, 1 × `E5506` | — |
| `const u = new Array(4).fill(7); console.log(u[0]);` (literal index) | builds | **builds, no diagnostic** | — |

> **Task 1's measured value, added 2026-09-10 (Task 8, ledger obligations).**
> The last row's "no diagnostic" was, until Task 1, an unmeasured silence — the
> followup demonstrated the *absence of a refusal*, not the *value printed*.
> Task 1 (`crates/kali_cli/tests/inprocess/release_allocation_identity.rs`)
> ran exactly this program in-process against `--release`, on a binary built
> from this branch **before** the Stage 1 fix (Task 2), against **`node
> v26.8.2`** (this machine's oracle; see the note at the top of this file —
> earlier measurements in this document were taken against `v26.8.1`, and
> that discrepancy is not repeated here). The result: `--release` printed the
> literal string **`4144\n`** where `node` and `--fast` both print `7\n`. This
> is this project's headline number — a silent, exit-0, no-diagnostic wrong
> value on a shipped release tier. `--release-advanced`'s own printed value was
> **never captured**: Task 1's test loop aborts on the first failing
> `assert_eq!`, which fires on `Release` before `ReleaseAdvanced` is reached
> (see `task-1-report.md`), so no number is claimed for that tier — only that
> its stop condition ("both `Release` and `ReleaseAdvanced` print `7`") was
> unambiguously not triggered, since `Release` already didn't.

**The fannkuch row was added 2026-09-09 at `6b59ddeef9`**, in the fix round that
unblocked PR #36, and it is a THIRD Benchmarks Game fixture in exactly this
shape: its three allocations are `const perm = new Array(n);`,
`const perm1 = new Array(n);` and `const count = new Array(n);` (lines 7-9), and
its release tiers lower them through the same zero placeholder. **It was not
found by the sweep that re-pinned the other two**, and the reason matters more
than the finding: a stale pre-project wasm artifact in the gitignored on-disk
incremental cache (`crates/kali_cli/tests/fixtures/.kali-cache/incremental/`,
dated 2026-07-16) short-circuited `compile_source_file` before codegen, so
fannkuch's three-mode build test read cached bytes and passed in ~0.00s on the
machine doing the sweeping. CI runners are cold and compiled for real, which is
where it surfaced. So **the sweep of this defect's fixtures was not exhaustive,
and a local green on this repository is only as trustworthy as its cache** --
filed as §6 of `computed-member-static-name-discovered-defects.md`.

All three `--fast` outputs are byte-identical to node. The `E5506` counts are an
artefact of how far the optimizer got before refusing and are deliberately not
asserted by any test; only the refusal and its code are.

## 3. The part that is still SILENT, and is the reason this is filed

The six `E3100` warnings spectral-norm emits at `--release` are the important
reading, not the `E5506`s:

```
4 × warning[E3100]: undefined call target 'Array' reached codegen and was lowered through a zero placeholder compatibility fallback
2 × warning[E3100]: undefined identifier 'n' reached codegen and was lowered through a zero placeholder compatibility fallback
```

The fixture's three allocations are `const u = new Array(n).fill(1);`,
`const v = new Array(n);` and `const w = new Array(n);` (lines 31-33). At
`--release` the allocation itself is lowered through a **zero placeholder**. So:

* A **non-literal** index read off such an allocation reaches this project's
  computed-member gateway and is REFUSED. That is loud, and it is what took the
  benchmark fixtures out of the measured list.
* A **literal** index read off the same allocation — `u[0]` — never reaches that
  gateway, because a literal index has a static name and takes the ordinary
  member lane. **It is miscompiled at `--release` in exactly the same way, and it
  stays SILENT.** The last row of the table above is the demonstration: the same
  allocation, the same tier, no diagnostic at all.

**So closing the loud half did not close the defect. It closed the half that
happened to route through a new gate.** Nothing in this repository currently
pins the silent half.

## 4. Why nobody noticed for as long as they didn't

`optimization_benchmark_suite_tracks_compile_time_size_and_speed`
(`crates/kali_cli/tests/runtime_smoke/misc.rs`) is green on a **build**, not on a
run: `assert_optimization_benchmark_fixture` counts instructions, adds and
tag-boxing ops in the emitted `.wasm` and **never executes the module**. A build
that produced zeros — or, for spectral-norm, an unloadable module — measured as
a passing benchmark. That is the observation worth carrying out of this file
even if the mechanism is fixed tomorrow: **a benchmark harness that never runs
the artefact is not a correctness gate, and this one was being read as one.**

Since `820753ee9a` the two fixtures sit in their own block in that test, which
pins the `--fast` build AND RUNS it byte-identically against node, and pins both
release tiers refusing. The other 59 fixtures are untouched, no exclusion list
grew, and the helper is not weakened.

## 5. Ruling R22: the refusal stands, and nothing was restored

The controller ruled that the release-tier refusal is **honest and correct**, and
that the two fixtures are **re-pinned, not restored**. Reasons, recorded so the
decision is arguable:

* The pre-project release tiers were emitting zeros and, for spectral-norm, an
  invalid module. Converting that into a visible refusal is what this project
  exists to do.
* The fixtures are the upstream Benchmarks Game programs; rewriting them to dodge
  the shape would change the workload and make the benchmark meaningless.
* The two `matches!` exclusion arms inside
  `assert_optimization_benchmark_fixture` still name `"spectral-norm"` and
  `"nbody"`. Those arms are now unreached and were **left in place deliberately**,
  so that restoring either fixture to the measured list restores a working
  configuration rather than a differently-broken one.

~~**A second class was identified and deliberately NOT admitted**: specialized-clone
array parameters (`in_array_bindings=false` at the gateway). It is addressable —
the recognizer could be taught the shape — but the pre-project binary printed
`0` for it too, so admitting it would be **widening the admitted set rather than
restoring a lane that used to work**, which is out of scope for a project whose
contract is that a folded bracket access behaves exactly as its dot spelling.~~

> **THE R22 OVERTURN, 2026-09-10, by the release-tier-allocation-identity
> project.** The second-class note above is **withdrawn: there is no second
> class.** The parameter shape is the *same substitution* arriving through an
> inlined argument, not a distinct lane that would need its own recognizer
> taught. The design spec's §2.4 measured it directly, at the baseline
> (`eec408d000`), citing `E5506` diagnostic counts and the value each shape
> produces at the default tier — the tier the "pre-project binary printed `0`"
> reasoning above never actually checked:
>
> | program shape | `--fast` | `--release` | default tier vs node |
> |---|---|---|---|
> | `f(a){let i=0;return a[i];}` over `new Array(2).fill(8)` | 0 | 1 | `8` = `8` ✓ |
> | copy loop `b[i]=a[i]` over two allocations | 0 | 1 | `5` = `5` ✓ |
> | the spectral-norm `Au(u,v)` shape at `n=3` | 0 | 2 | `3` = `3` ✓ |
> | declarator only, dynamic index | 0 | 1 | `7` = `7` ✓ |
>
> (columns: count of `E5506` diagnostics at that tier.) `fast=0 / release≥1`
> on every parameter shape, and every one of them **already produces node's
> answer at the default tier** — this is a lane that works at three of four
> tiers and is destroyed at the fourth by the same layout-binding
> specialization step. That is restoration, not widening. Task 3 then measured
> all four shapes — the dynamic-index declarator, the parameter read, the
> parameter store, and the reduced spectral-norm shape — together in one test
> (`allocation_backed_reads_agree_with_node_at_every_tier`,
> `crates/kali_cli/tests/inprocess/release_allocation_identity.rs`) and got a
> clean pass on the **first run**, at all three tiers (`Fast`, `Release`,
> `ReleaseAdvanced`), against node: no case refused with `E5506`, no case's
> output disagreed (`task-3-4-report.md`). Task 2's Stage 1 fix (§6 below)
> reaches all four shapes because it closes the one hole they all pass
> through, not because it was taught a second shape.

## 6. What would close it, and what actually did — rewritten 2026-09-10

~~Teach the optimizer that a `new Array(n)` / `new Array(n).fill(v)` initializer
is allocating, not pure, so inlining it duplicates an allocation rather than
copying a constant — or, equivalently, refuse to inline a declarator
initializer whose value has identity. That is a change in `kali_optimize`, not
in the computed-member gateway, and it is what would let `"spectral-norm"` and
`"nbody"` go back into the measured benchmark list, and fannkuch's three-mode
build assertion back to three.~~

**This prescribed an effects analysis (an "allocating, not pure" judgement)
that the LIR has no vocabulary for** — `LirNode` carries only `kind`, `text`,
`children`, `function_flavor`; there is nowhere to hang such a flag. The
design spec (§3.3) considered and rejected the nearest cheap alternative too —
carrying the binding's name onto the substituted node so `base_text` survives
and the gateway admits the read — because that fixes the symptom
(`base_text=None`) while leaving the array's lost identity in place: a
literal-index read off a duplicated allocation would still read the wrong
array, just without announcing itself. That converts a refusal into a silent
miscompile, the exact direction this project exists to reverse.

**What actually closed it: a positive element check, not an effects
analysis.** Stage 1 (Task 2) replaced `is_array_literal`'s negative-space
definition with a positive one: a text-less `Value` node is an array literal
only when every child is itself materializable — a `Literal`, a `Value` whose
text parses as a literal, or a nested array/object literal
(`is_materializable_element`, `crates/kali_optimize/src/layout.rs`). The
`new Array(n).fill(v)` wrapper has a `Call` child (`.fill(v)` lowers through a
call node), so it stops qualifying; the binding never enters the spec env at
`specialize.rs:82`/`:109`; the substitution at `specialize.rs:120-127` never
fires; and the identifier keeps its own name at `--release` exactly as it
does at `--fast`. `is_specializable_binding` was reduced to delegate to the
same predicate (`is_materializable_element`) rather than duplicating its
logic — a refactor, not a second fix.

**The plan also mandated something that was measured wrong and removed: do
not repeat it.** The design's Stage 1 (§3.1) called for rejecting an **empty**
child list too, as the fail-closed direction. That shipped in round 0 of
Task 2, and it broke a correct program: `Object.fromEntries([])` — legal,
previously-working JavaScript — started failing to build with `E5506` at both
release tiers, because `fold_object_from_entries_call`
(`crates/kali_optimize/src/object_fold.rs:252`) routes an empty entries array
through `is_array_literal`, and an unconditional empty-rejection declines it
before the per-entry loop is ever reached. It also flipped
`array_literal_length([])` from `Some(0)` to `None` and inverted a negated use
of that call at `specialize.rs:517-521`. The coordinator ruled the guard
reverted: `is_array_literal` accepts an empty child list again (the
`node.children.iter().all(...)` guard is vacuously true on an empty
iterator), restoring the pre-2026-09-10 baseline for that one shape. **The
empty case was never load-bearing for the fix in the first place** — Task 1's
pinned regression test passes identically with or without the empty-rejection
guard, because `is_materializable_element`'s own empty-children arm shadows
arm 3's call back into `is_array_literal` before an empty node would ever
reach it. See `task-2-report.md` (round 1 and round 2 reports) for the full
trace, including two pre-existing, unrelated defects this measurement
surfaced in `Object.fromEntries` (filed separately, §7 below and the silent
miscompile register).

This is what let `fannkuch-redux-benchmark-v1` go back into the measured
benchmark list — genuinely fixed, builds and runs at all three tiers,
byte-identical to node (`task-5-report.md`, `task-6-report.md`). It is **not**
what fixed `spectral-norm-benchmark-v1` or `nbody-benchmark-v1` — see the
status note below.

Until Stage 1, two things were true and had to be said together: `--release`
REFUSED some programs `--fast` compiled correctly, and `--release` SILENTLY
MISCOMPILED others that `--fast` compiled correctly. The second was the
dangerous one, and Task 1 measured it (§2 above, `4144` against node's `7`).
Both are now closed for the shapes Stage 1's positive element check covers —
which is every shape this document measured, all four of §5's table, and the
one Benchmarks Game fixture (fannkuch) whose allocations are exactly this
shape. It is **not** every allocating construct: Stage 1 fixes the measured
defect by exclusion, not by representing array-literal-ness in the LIR, so the
next allocating construct that reaches the same negative-space hole — and
there are two more copies of that hole, unfixed, named in
`docs/superpowers/followups/kali-silent-miscompile-register.md` R-61's
"Root-cause group" bullet — is not covered by this fix. Stage 2 (representing
the property instead of inferring it) was sized and stopped before shipping
any code; see `task-7-report.md`.

### Status as of Task 8 (2026-09-10): which fixtures Stage 1 actually restored

Stage 1 does **not** restore all three of the Benchmarks Game fixtures this
document names. Measured by Task 5's execution gate
(`crates/kali_cli/tests/inprocess/benchmark_execution.rs`), against `node
v26.8.2`:

| fixture | status |
|---|---|
| `fannkuch-redux-benchmark-v1` | **Genuinely fixed.** Builds, loads, runs, and matches node byte-for-byte (`"228\nPfannkuchen(7) = 16\n"`) at all three tiers. |
| `spectral-norm-benchmark-v1` | **Not fixed — a different mechanism.** Now *builds* at both release tiers (Stage 1 removed the `E5506` refusal), but the emitted module is **unloadable** (`E4201`, `wasm[0]::function[46]`/`[41]`). `--fast` still matches node (`1.274219991`). |
| `nbody-benchmark-v1` | **Not fixed — a different mechanism.** Still refuses with `E5506` at both release tiers. Its `bodies` array literal has `Call` nodes as elements (`Sun()`, `Jupiter()`, …), so it fails Stage 1's own positive element check and never enters the spec env this project narrowed — confirmed by direct code trace, not inference (`task-5-report.md`, "Ruling 2 verdict"). The refusal comes from elsewhere; the most plausible candidate is the `kali_codegen` copy of the negative-space predicate named in R-61's root-cause bullet. |

So this project restores **one of the three** target fixtures it named. The
other two's remaining brokenness is a *different* mechanism than the one this
document is about, and is not this document's to close.

## 7. Suggested home — filed 2026-09-10 (Task 8)

**§2 of the register, silent**, for the literal-index half of §3 — it is a
silent wrong value at exit 0 on a shipped build tier. The refusing half belongs
in §7 (*fail-loudly-but-wrong*) or in the phase plan as a release-tier capability
gap, not in §2. Whoever files it should note that the register's oracle harness
measures the DEFAULT tier only, so a §0.2 row for this entry cannot be backed by
an oracle case as the harness stands today — which is itself a gap in the
instrument worth recording.

**Filed as R-61 (§2, Tier 2) and R-62 (§7)** in
`docs/superpowers/followups/kali-silent-miscompile-register.md`, per the above.
R-61 carries Task 1's `4144` measurement and records — rather than papers
over — the instrument gap named in the paragraph above: no §0.2 row exists for
it, because there is no oracle case to back one until the harness can compile
and run a non-default tier. R-62 records the refusing half as fail-closed
(honest, `E5506`), not fail-loudly-but-wrong in FL-01's sense — see R-62's own
severity note.
