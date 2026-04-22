# Delivery Increments

This document groups stages into reviewable implementation increments.

The phase plan is the authoritative sequencing model, but day-to-day implementation often needs a slightly higher-level answer to a practical question:

> What is the next **usable** state of the repository we are trying to reach?

These increments answer that question.

They are intentionally larger than a single stage and smaller than a full phase. Each increment should leave the repository in a state that is understandable, demoable, and safe to build on.

## Reading rule

Use increments for milestone planning and branch sizing.
Use stage files for exact task execution.
If an increment and a stage ever feel inconsistent, the stage keeps the implementation detail and this file should be updated.

## Increment map

| Increment | Stages | Outcome |
|---|---|---|
| I0 — Spec-first bootstrap | planning baseline before 1.1 | normative specs and the roadmap exist before implementation starts |
| I1 — Workspace boot | 1.1 | buildable workspace, CLI entrypoint, proof-ready baseline |
| I2 — Frontend syntax acceptance | 1.2-1.3 | deterministic lexing, parsing, and AST production |
| I3 — Static semantic checking | 1.4-1.5 | `kali check` works on local TS/JS inputs |
| I4 — Local compile pipeline | 1.6-1.7 | typed programs lower and compile to deterministic WASM |
| I5 — Local execution baseline | 1.8 | `kali run` and `kali test` work in the default standalone context |
| I6 — Phase-1 product surface | 1.9-1.13 | sandboxing, install/build/workflow, and machine-readable outputs form one coherent toolchain |
| I7 — Phase-1 evidence closure | 1.14 | the shipped Phase-1 surface is backed by explicit evidence lanes |
| I8 — Semantic depth & external surfaces | 2.1-2.5 | MIR/ownership, effect reports, embedding, Lean foundation, and coverage reporting stabilize |
| I9 — Performance & compatibility broadening | 3.1-3.4 | optimization, Node path, ecosystem breadth, and host-capability growth are evidence-backed |
| I10 — Hard late-core features | 4.1-4.2 | dynamic compatibility and proof-backed published boundary |
| I11 — Deferred platform expansion | 5.1-5.5 | later compatibility and platform/runtime breadth open one surface at a time |

## Increment details

### I0 — Spec-first bootstrap

**Why it exists**
- Keeps implementation from racing ahead of normalized product claims.
- Establishes the split between `SPEC.md`, `specs/`, `PLAN.md`, and `plan/`.

**Repository should contain**
- the normative spec set,
- the maturity matrix,
- the roadmap and phase/stage skeleton,
- proof-boundary discipline language.

**Do not claim yet**
- implementation progress,
- public feature availability,
- proof-backed status beyond what `proofs/BOUNDARY.md` actually says.

### I1 — Workspace boot

**Stages**: 1.1

**Usable outcome**
- the workspace builds,
- the CLI binary exists,
- the repo has canonical developer entrypoints,
- proof-ready hygiene starts immediately.

**Demo**
- `cargo build`
- `cargo test --workspace`
- `cargo run -p kali -- --version`

### I2 — Frontend syntax acceptance

**Stages**: 1.2-1.3

**Usable outcome**
- Kali can deterministically tokenize and parse supported TS/JS inputs,
- frontend fixtures and snapshots become meaningful review artifacts.

**Why this increment matters**
It gives the project a real language front door before semantic interpretation and keeps grammar work reviewable on its own.

**Demo**
- parse fixture files and inspect deterministic AST/token output

### I3 — Static semantic checking

**Stages**: 1.4-1.5

**Usable outcome**
- imports resolve,
- names bind correctly,
- the type system enforces the bounded inference contract,
- `kali check` becomes the first real end-user command.

**Guardrail**
Parsing alone is not support. This increment is where syntax becomes semantic behavior.

**Demo**
- `cargo run -p kali -- check fixtures/compiler/...`

### I4 — Local compile pipeline

**Stages**: 1.6-1.7

**Usable outcome**
- checked programs lower through internal IR,
- deterministic WASM artifacts are produced,
- the compiler is end-to-end for local programs even before runtime execution is wired.

**Guardrail**
Do not widen runtime or package claims yet. This is compile pipeline closure, not host compatibility.

**Demo**
- `cargo run -p kali -- build fixtures/compiler/...` against a local fixture in the executable path

### I5 — Local execution baseline

**Stages**: 1.8

**Usable outcome**
- Kali can execute compiled local programs in its default standalone context,
- the test runner exists,
- runtime semantics are concrete enough for later sandbox work.

**Why this is the hinge increment**
Once this lands, the repository stops being “just a compiler” and becomes a usable toolchain.

**Demo**
- `cargo run -p kali -- run fixtures/runtime/...`
- `cargo run -p kali -- test fixtures/runtime/...`

### I6 — Phase-1 product surface

**Stages**: 1.9-1.13

**Usable outcome**
The main Phase-1 product slices now fit together:
- runtime sandbox enforcement and static policy validation,
- deterministic package installation and lock state,
- executable, bundle, and base-library builds,
- `init`, `fmt`, and `lint`,
- stable diagnostics and schema-v1 JSON outputs.

**Why this increment is grouped**
These streams can proceed in parallel after 1.8, but users experience them as one coherent product surface.

**Required shared surfaces**
- `specs/12-cli.md`
- `specs/15-errors.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

**Demo set**
- `kali run --sandbox ...`
- `kali install`
- `kali build --bundle ...`
- `kali init`
- `kali check --output json ...`

### I7 — Phase-1 evidence closure

**Stages**: 1.14

**Usable outcome**
- the shipped Phase-1 claims are evidence-backed rather than only implemented,
- browser-targeted smoke, determinism checks, conformance, and package-corpus lanes are in place,
- the repo remains proof-ready.

**Guardrail**
A feature is not publicly “done” just because code exists. This increment is where the proof burden catches up to the implementation burden.

**Demo**
- the canonical CI/mise tasks for Phase-1 evidence all pass

### I8 — Semantic depth & external surfaces

**Stages**: 2.1-2.5

**Usable outcome**
- MIR and ownership become canonical,
- the public effect-report surface opens,
- embedding becomes a stable external contract,
- Lean work moves beyond placeholder discipline,
- coverage reporting becomes stable for documented contexts.

**Why this increment matters**
This is where Kali starts acting like a platform, not only a CLI compiler.

### I9 — Performance & compatibility broadening

**Stages**: 3.1-3.4

**Usable outcome**
- release modes deliver evidence-backed gains,
- the Node path opens for its documented subset,
- host capabilities widen under explicit sandbox rules,
- package and ecosystem claims climb the support ladder with corpus proof.

**Guardrail**
Breadth claims must always name the exact rung: installable, checkable, buildable, executable, or deployable.

### I10 — Hard late-core features

**Stages**: 4.1-4.2

**Usable outcome**
- late dynamic features open only through explicit gates,
- `package-audit` exists as its own command family,
- the project can make proof-backed claims for a non-empty published boundary.

**Guardrail**
This increment must not silently weaken AOT-only semantics or blur proof-ready with proof-backed.

### I11 — Deferred platform expansion

**Stages**: 5.1-5.5

**Usable outcome**
- thread-aware runtime profile,
- standalone browser runtime/test support,
- programmable policy/effect extensions,
- late host/object-model breadth,
- additive PGO and broader bindings.

**Planning rule**
This increment is intentionally decomposed feature-by-feature. It should never be shipped or described as one blanket “future compatibility” bucket.

## How to use increments in practice

When choosing what to implement next:
1. pick the next incomplete increment,
2. identify the stage boundary that unlocks the next user-visible demo,
3. keep the branch small enough that the increment still feels reviewable,
4. ship the evidence lane with the implementation,
5. update the maturity matrix only when the evidence supports promotion.

## Maintenance rule

When stage sequencing changes, update this file if the practical user-visible increments change.
That usually means updating this file together with:
- [`../PLAN.md`](../PLAN.md),
- the affected phase README,
- and any stage docs whose milestone wording moved.
