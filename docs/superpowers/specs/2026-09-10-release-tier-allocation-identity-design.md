# The release tiers substitute a binding's initializer for its name, so an allocation loses the identity `--fast` keeps

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `eec408d000` (main, clean tree, the merge of PR #40) |
| kali binary | `kali 0.1.0`, `.cache/cargo-target/debug/kali` (572,090,096 bytes) |
| oracle | `node v26.8.2` — **not** the `v26.8.1` the followup documents pin; every oracle reading below was taken at `v26.8.2` and is marked where it matters |
| measured on | 2026-09-10 |
| the defect this closes | `docs/superpowers/followups/release-mode-optimizer-inlines-an-allocating-initializer.md`, filed 2026-09-09 at `71b5f42f6c` |

This document designs the fix for that followup file. Unlike the incremental-cache
spec that preceded it, **this one does re-measure**, because the investigation
changed the defect's account in three ways the followup does not contain:

1. The followup names the pass wrongly. It says "the optimizer inlines an
   allocating array initializer", pointing at `kali_optimize`'s inliner. The
   substitution is not inlining; it is the **layout-binding specialization** at
   `crates/kali_optimize/src/specialize.rs:120-127`, reached through
   `is_array_literal`'s negative-space definition at
   `crates/kali_optimize/src/layout.rs:177`. §2 establishes this.
2. The followup's §5 records a "second class ... deliberately NOT admitted" —
   specialized-clone array parameters — as a distinct lane that would need the
   recognizer taught a new shape. §2.4 shows it is the **same defect** arriving
   through a parameter. There was never a separate lane to admit.
3. The followup's §6 prescribes a fix ("teach the optimizer that a
   `new Array(n)` initializer is allocating, not pure") that the LIR cannot
   express as written: `LirNode` has no field to carry it and `LirNodeKind` has
   no variant for an array literal. §3 designs what the representation can
   actually support.

**Citation convention.** Every line reference is as of the baseline commit above.

---

## 1. What this project is

Restore correct `--release` / `--release-advanced` compilation of programs that
index an array produced by an allocating initializer, so that `spectral-norm`
and `nbody` return to the *measured* benchmark list and `fannkuch-redux`'s
three-mode build assertion returns to three.

It carries one instrument change with it: the benchmark harness that let this
defect hide becomes a harness that executes what it measures (§4).

### 1.1 The two things that are true together

Said together because either alone misleads:

* `--release` **REFUSES** some programs `--fast` compiles correctly. Loud, and
  what took three fixtures out of the measured list.
* `--release` **SILENTLY MISCOMPILES** others that `--fast` compiles correctly.
  The dangerous one, and the reason this is not merely a capability gap.

### 1.2 What this project does NOT claim to have measured

**The silent half's wrong value is unmeasured, here and in the followup.** The
followup demonstrates it only as "builds, no diagnostic" (its §2 table, last
row), and this investigation could not do better from outside: `kali run` takes
a source file, has no tier flag, and there is no wasm runtime on `PATH` —
`wasmtime` is a workspace dependency reachable only from
`crates/kali_cli/tests/inprocess.rs`. **Task 1 of the implementation plan is to
measure it in-process and record the number**, because this project's priority
rests on a value nobody has yet observed. If it turns out the literal-index half
is *not* wrong, §2.3's account is falsified and the design's scope should shrink
to the loud half before any code is written.

---

## 2. The defect, measured

### 2.1 The mechanism

Four steps, each verified at the baseline:

| # | site | what happens |
|---|---|---|
| 1 | `crates/kali_optimize/src/driver.rs:186-190` | the entire specialization block runs under `Release` / `ReleaseAdvanced` only; `Fast` and `Default` match `{}` and skip it |
| 2 | `crates/kali_optimize/src/specialize.rs:82`, `:109` | `u → init` enters the spec env when `is_specializable_binding(program, init)` holds — on the **raw** `init`, not a resolved one |
| 3 | `crates/kali_optimize/src/layout.rs:177` | `is_array_literal` returns true for *any* text-less `Value` node that is not an object literal, so the wrapper around `new Array(n).fill(v)` qualifies |
| 4 | `crates/kali_optimize/src/specialize.rs:120-127` | each occurrence of the identifier is overwritten: `program.nodes[id] = program.nodes[bound].clone()` |

Step 1 is the tier split. Step 4 is the identity loss: the identifier node
*becomes* a copy of the initializer node, sharing its child ids — which is
verbatim what the controller's LIR dump recorded ("a text-less wrapper around
the same node id as the declarator's initializer", followup §2).

Step 4 is `tracker.allow`-gated, and that is independently checkable from
outside the compiler:

```
$ kali build --release --max-specializations 0 t.js   # Built executable artifact
$ kali build --release --max-specializations 1 t.js   # error[E5506]
```

for `t.js` = `const u = new Array(4).fill(7); let i = 0; console.log(u[i]);`.
A cap of zero makes every `tracker.allow` return false, and the refusal
disappears. No other release-only pass is gated that way and produces this node
shape.

### 2.2 The asymmetry that causes it

`is_array_literal` and `is_object_literal` sit adjacent in `layout.rs` and are
built on opposite principles:

```rust
// layout.rs:150 — POSITIVE. Children must be `init`/`get`/`set` nodes with 2 children.
pub(crate) fn is_object_literal(&self, program: &LirProgram, id: LirNodeId) -> bool { ... }

// layout.rs:177 — NEGATIVE SPACE. Anything text-less that isn't an object literal.
pub(crate) fn is_array_literal(&self, program: &LirProgram, id: LirNodeId) -> bool {
    if node.kind != LirNodeKind::Value || node.text.is_some() { return false; }
    !self.is_object_literal(program, id)
}
```

A predicate defined as "not the other thing" admits every shape nobody thought
about. The allocation is one such shape; it is not distinguished from an array
literal because **the LIR does not represent the difference** (§3.2).

### 2.3 The three consequences

All three follow from step 4, and this is the part the followup treats as
separate defects:

| consequence | why | loudness |
|---|---|---|
| dynamic-index read refuses | base is now the initializer node, so `base_text=None` and the gateway cannot resolve it | **loud** — `E5506` |
| literal-index read miscompiles | a literal index has a static name, takes the ordinary member lane, never reaches the gateway | **silent** — see §1.2 |
| parameter read refuses | the argument identifier resolves to the same substituted node | **loud** — `E5506` |

### 2.4 The "second class" is the same class

The followup's §5 declines to admit specialized-clone array parameters because
"the pre-project binary printed `0` for it too, so admitting it would be
widening the admitted set rather than restoring a lane that used to work."

That reasoning reads a **release-tier** result as evidence about the lane. The
lane works everywhere else. Diagnostic counts, measured at the baseline
(`E5506` occurrences; `node` column is the correct answer):

| program shape | `--fast` | `--release` | default tier vs node |
|---|---|---|---|
| `f(a){let i=0;return a[i];}` over `new Array(2).fill(8)` | 0 | 1 | `8` = `8` ✓ |
| copy loop `b[i]=a[i]` over two allocations | 0 | 1 | `5` = `5` ✓ |
| the spectral-norm `Au(u,v)` shape at `n=3` | 0 | 2 | `3` = `3` ✓ |
| declarator only, dynamic index | 0 | 1 | `7` = `7` ✓ |

`fast=0 / release≥1` on every parameter shape, and every one of them produces
node's answer at the default tier. This is a lane that works at three of four
tiers and is destroyed at the fourth by step 4 — restoration, not widening.

**Ruling R22's exclusion is therefore overturned**, and §6.3 records the
overturn where R22 lives. Nothing new is admitted by doing so: fixing step 4
fixes the parameter shape because it is the same substitution.

### 2.5 What is NOT this defect, and must keep refusing

A plain array literal with a dynamic index refuses at **both** tiers:

```
$ printf 'const u = [7,7,7,7]; let i = 0; console.log(u[i]);' > t.js
$ kali build --fast t.js     # error[E5506]
$ kali build --release t.js  # error[E5506]
```

`fast=1 / release=1`. No tier split, so no part of this is caused by step 4.
It is a pre-existing computed-member gateway gap, it is out of scope, and §4.3
pins it refusing so a later reader does not read it as fallout from this work.

---

## 3. The design

Two stages. Both work by **declining** to classify something as a materializable
literal, so a wrong answer costs an optimization and never a correct program.
That direction is deliberate: this predicate has already produced one silent
miscompile by being permissive.

### 3.1 Stage 1 — close the negative space

`is_array_literal` gains a positive element check. A text-less `Value` node is an
array literal only when **every child is itself materializable**: a `Literal`, a
`Value` whose text parses as a literal, or a nested array/object literal. A
child that is a `Call`, a `ComputedMember`, or `Unknown` disqualifies the node.

The wrapper around `new Array(n).fill(v)` has a `Call` child, so it stops
qualifying; the binding never enters the spec env at `specialize.rs:82`/`:109`; step 4
never fires; and the identifier keeps its name at `--release` exactly as it does
at `--fast`.

**Two expectations that must be verified rather than assumed:**

* The `E3100` warnings (`undefined call target 'Array'`, `undefined identifier
  'n'`, both lowered "through a zero placeholder compatibility fallback") should
  disappear with the `E5506`s, since both arise from the substituted position.
  If any `E3100` survives, that is a **fail-open lowering** in its own right and
  gets filed, not absorbed.
* Stage 1 must not admit §2.5's shape. Pinned by §4.3.

**Deliberately left alone:** `specialize.rs:82`/`:109` passes the raw `init` where
`collect_constant_bindings_into` (`object_fold.rs:457`) resolves first. Making
the two agree is a defensible cleanup and is **not** done here — it changes
which bindings are eligible in a pass this project is already changing, and one
behavioural change per project is the rule that keeps a bisect readable.

### 3.2 Stage 2 — represent it instead of inferring it

Stage 1 fixes the measured defect by exclusion. It does not stop the next
allocating construct — `Array.from(...)`, `new SomeClass()`, a spread — from
entering through the same hole, because the hole is that **array-literal-ness is
inferred from node shape rather than represented**.

`LirNode` carries only `kind`, `text`, `children`, `function_flavor`
(`crates/kali_lir/src/node.rs:34-39`). There is nowhere to hang a flag. But the
repository has already solved this problem once, and says so in a doc comment
(`node.rs:12-15`):

```rust
/// A computed member access `[object, index]` with no static property
/// name. Every recognizer that matches `Value` declines this kind by
/// construction; codegen's `emit_computed_member` is its only consumer.
ComputedMember,
```

Stage 2 follows that precedent: add `LirNodeKind::ArrayLiteral`, tag genuine
array literals at lowering (`crates/kali_hir/src/lowering/expression.rs`), and
reduce `is_array_literal` to `node.kind == LirNodeKind::ArrayLiteral`. Positive
by construction.

**The blast radius is the feature, not the cost.** Adding an enum variant makes
`rustc` enumerate every `match` on `LirNodeKind` that must now decide about
array literals. Negative space cannot be audited; an exhaustive match can. The
implementation plan sizes this by adding the variant first and reading the
compiler's error list.

**This is the honest reading of followup §6.** Its prescription — "teach the
optimizer that the initializer is allocating, not pure" — describes an effects
analysis the LIR has no vocabulary for. The realisable form of the same idea is
to make the *representation* carry what the predicate was guessing.

### 3.3 Why the third option was rejected

A cheaper route exists: keep the substitution and carry the binding's name and
repr metadata onto the substituted node, so `base_text` survives and the gateway
admits the read. It would turn the fixtures green fastest.

**It is rejected, and the reason should outlive this document.** It addresses
`base_text=None`, which is the *symptom*. The array still loses identity under
it, so a literal-index read off a duplicated allocation still reads the wrong
array — the substitution just stops announcing itself. That converts a refusal
into a silent miscompile, which is the exact direction this project exists to
reverse.

---

## 4. Verification

### 4.1 The gate that failed, and why

`assert_optimization_benchmark_fixture` (`crates/kali_cli/tests/runtime_smoke.rs:6061`)
builds each fixture at three tiers, asserts each build succeeds, and counts
instructions, `i64.add`s and tag-boxing ops in the emitted `.wasm`. **It never
executes the module.** A build emitting zeros measured as a passing benchmark;
for spectral-norm, so did a build emitting an unloadable module.

A benchmark harness that never runs the artefact is not a correctness gate, and
this one was being read as one.

### 4.2 The gate that replaces it

A new module `crates/kali_cli/tests/inprocess/benchmark_execution.rs`.

**It must live in that target.** `inprocess.rs` is "the only `kali_cli`
integration test target that links wasmtime ... statically linked once (~450 MB)
instead of separately per binary" (`inprocess.rs:1-12`). A second wasmtime-linking
target costs another ~450 MB, and extra target directories have exhausted this
pod's disk and killed a run before.

The measured list is the 59 fixture stems in the loop at
`crates/kali_cli/tests/runtime_smoke/misc.rs:1374` — the same 59 the followup's
§4 calls "the other 59 fixtures". Per fixture:

1. **Make it observable.** Every fixture ends in a bare call — `entry();`,
   `hot(1, 2);`, `hot(true);` — and **not one of the 59 contains a
   `console.log`** (verified by resolving every stem in the loop to its source).
   The Benchmarks Game fixtures do print, which is why the three being restored
   get a real value oracle without wrapping. Executing the 59 as committed
   proves only that the module loads. So each is copied to a temp
   file with its terminal call wrapped as `console.log(<call>)`. This is the
   existing EXECUTION GUARD pattern (`runtime_smoke/misc.rs`, the comment block
   below the re-pin loop), which already does exactly this for three fixtures and
   records why: the wrapped copy is never passed to
   `assert_optimization_benchmark_fixture` and never hashed against the
   fixture's `sourceSha256`, so the measured workload and pinned metadata are
   untouched.
2. **Oracle.** Run the wrapped copy under `node` and take its stdout. CI installs
   Node (`.github/workflows/ci.yml:25-32`) and five existing test files already
   shell out to `Command::new("node")`, so this is idiomatic here. The version
   floats (`node-version: latest`) while the followup documents pin `v26.8.1`;
   the differential is against whatever CI installs, and that is recorded rather
   than hidden.
3. **Compare.** Compile and execute at `--fast`, `--release` and
   `--release-advanced` through `compile_source_file` + `RuntimeCtx`, following
   `inprocess/release_constant_condition_loop.rs:36`. Assert all three equal the
   oracle.

Where a fixture's entry returns `undefined`, wrapping yields a liveness check
rather than a value oracle. That is accepted and documented; amending fixture
sources to return values would break `sourceSha256` and change the measured
workload.

This catches, for all 59 fixtures at all three tiers, the class that hid this
defect: a build producing zeros, an unloadable module, and any tier divergence.

### 4.3 Regressions this project must pin

| what | assertion |
|---|---|
| the silent half | the §1.2 measurement, pinned as a test at all three tiers |
| the loud half, declarator | `new Array(n).fill(v)` + dynamic index builds **and runs** correctly at both release tiers |
| the loud half, parameter | the §2.4 parameter shapes, same |
| §2.5's pre-existing gap | plain array literal + dynamic index still **refuses** at all three tiers |
| the predicate itself | `is_array_literal` unit tests in `layout_tests.rs`: accepts literal-element arrays, rejects a `Call`-child wrapper |

### 4.4 Fixture restoration

The two `matches!` exclusion arms naming `"spectral-norm"` and `"nbody"` come
out. The dedicated re-pin block (`runtime_smoke/misc.rs:1631`) is replaced
according to the instruction it carries in its own failure message — *"do NOT
delete this block ... verify the emitted module actually RUNS and agrees with
node, then move the fixture back into the list above."* fannkuch's three-mode
build assertion returns to three.

### 4.5 The branch this plan must be ready for

`assert_optimization_benchmark_fixture` asserts that release improves at least
one footprint metric over fast. **Stage 1 works by removing a release-tier
fold.** So for spectral-norm and nbody, release may no longer beat fast on any
metric, and that assertion may fail when they rejoin the measured list.

If it does, the response is to add those names to the exclusion list **with a
comment recording why** — that the release tiers do not optimize this shape once
the unsound fold is gone — and never to weaken the assertion. The exclusion list
already carries one such honest entry (`const-object-property-access`, rewritten
2026-07-19) and this would be a second. This is a plausible outcome, not a
certainty; the plan treats it as a branch, not a surprise.

---

## 5. Non-goals, each with the reason

* **The alias-mutation hole.** `collect_mutated_binding_names`
  (`object_fold.rs:519`) is a program-wide, scope-blind scan for store *base
  identifiers*. A const array mutated only through a differently-named parameter
  is never a store base under its own name — spectral-norm's `const w` is
  written as `v[i] = t` inside `Au`, so the name `w` is never seen. Stage 1
  removes the exposure for allocations; the hole stays open for genuine array
  literals. **Filed separately** (§6.4): it is a distinct root cause with a
  distinct fix, and folding it in would make this project two claims.
* **§2.5's computed-member gateway gap.** Refuses at both tiers, no tier split,
  not caused by this defect.
* **Reconciling `specialize.rs:82`/`:109` with `object_fold.rs:457`.** §3.1.
* **Widening the computed-member gateway generally.** This project restores a
  lane that works at three tiers; it does not admit new shapes.

---

## 6. Ledger obligations

Documentation this project owes, and where:

1. **`release-mode-optimizer-inlines-an-allocating-initializer.md` §6** —
   replace the prescription with §3's mechanism-accurate account, and correct
   the pass attribution in the title and §1 (it is layout-binding
   specialization, not inlining).
2. **The same file, §5** — record the R22 overturn with §2.4's table. The
   "second class deliberately NOT admitted" note is withdrawn: there is no
   second class.
3. **The register**, per that followup's §7 — the silent literal-index half to
   §2 (silent), the refusing half to §7 (fail-loudly-but-wrong). Record with it
   the instrument gap that §7 names: the register's oracle harness measures the
   **default tier only**, so a §0.2 row for this entry cannot be backed by an
   oracle case as the harness stands.
4. **A new followup** for the alias-mutation hole (§5, first bullet), with the
   spectral-norm `w` evidence and a repro.
5. **Regenerate the blast-radius ranking** if any register entry changes —
   `cargo run -p kali_blast_radius --example rank`.
   `kali_blast_radius::ranking::ranking_tests::spliced_document_matches_the_generator`
   goes red otherwise, by design.

---

## 7. Risks, and what would falsify this design

| risk | what it would look like | response |
|---|---|---|
| **The silent half isn't wrong** | Task 1 measures the literal-index case at `--release` and it agrees with node | §2.3's account is falsified. Shrink scope to the loud half and re-file the followup's §3 claim as unproven. This is the design's load-bearing assumption. |
| **Stage 1 doesn't restore the fixtures** | `E5506`s persist after the predicate is tightened | The binding enters the env by a route other than `specialize.rs:82`/`:109`. Re-open §2.1 before writing more code; do not add a second patch on top of a wrong mechanism. |
| **`E3100` survives** | allocation still lowered through a zero placeholder | A fail-open lowering independent of the substitution. File it (§3.1); do not absorb it silently. |
| **Release stops beating fast** | §4.5's assertion fails for the restored fixtures | §4.5. Exclusion list with a recorded reason; never weaken the assertion. |
| **Stage 2's blast radius is large** | adding the enum variant produces a long `match` error list | Size it before committing to it — the variant can be added and reverted cheaply. If it is genuinely large, Stage 2 becomes its own project and Stage 1 ships alone with §3.2's reasoning recorded as the reason a follow-on exists. |
| **Node version drift** | the differential fails on a node upgrade in CI | Expected and acceptable; it is a real divergence signal. The floating version is recorded in §4.2 rather than pinned, matching existing practice in five test files. |

## 8. Process notes carried into the plan

* `scripts/test-gate.sh` does **not** cover clippy or fmt. Both run explicitly
  before pushing; `GATE OK` is not CI-green.
* One cargo target directory. No extra worktrees, no extra target dirs — they
  have exhausted this pod's disk and killed a run.
* The on-disk incremental cache now carries compiler identity (PR #40), so a
  local green is trustworthy in a way it was not when the fannkuch row was
  missed. That fix is what makes this project's own measurements safe to take.
