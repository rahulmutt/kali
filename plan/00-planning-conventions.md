# Planning Conventions

This file defines the shared rules for writing, updating, and completing Kali plan stages.

It complements [`../PLAN.md`](../PLAN.md):
- `PLAN.md` owns the roadmap and ordering,
- this file owns the common stage-writing and stage-closing discipline.

## Core planning rule

A stage is well-formed only if it preserves a workable repository state and makes one new capability demonstrably better, broader, or more stable.

That means a stage should never be only:
- internal refactoring with no preserved demo path,
- speculative documentation with no implementation target,
- or an evidence claim that is not tied to a concrete command, artifact, or invariant.

## Required sections for stage documents

Every stage document should include these sections, in this order where practical:

1. **Title / phase / spec refs**
2. **Depends on**
3. **Goal**
4. **Workable milestone**
5. **Tasks** or **historical stage tasks**
6. **Out of scope**
7. **Status**

Optional sections that are encouraged when useful:
- ordering note
- current repository state
- evidence
- follow-up work / remaining work
- coordination notes

## Definition of a workable stage

A stage is workable when all of the following are true:

1. `cargo build` succeeds.
2. `cargo test --workspace` passes.
3. The repository still has at least one user-visible demonstration path.
4. The stage adds or stabilizes one concrete capability.
5. Hard invariants remain intact:
   - AOT-only compilation
   - pure Rust implementation
   - no tracing/background GC
   - sandbox-first honesty
   - deterministic machine contracts

## Stage-completion packet

Before calling a stage complete, ship this minimum packet:

### 1. Implementation slice
The relevant code, config, docs, and project wiring for the stage land together.

### 2. Normative coordination slice
If public behavior changed, update the owning documents:
- [`../specs/12-cli.md`](../specs/12-cli.md)
- [`../specs/15-errors.md`](../specs/15-errors.md)
- [`../specs/18-schemas.md`](../specs/18-schemas.md)
- [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md)
- [`../README.md`](../README.md) when summary wording changed

Not every stage touches all of them, but a public-surface change should not skip the ones it does affect.

### 3. Evidence slice
Add or extend the evidence lane that proves the milestone:
- unit/integration tests
- checker baselines
- package corpus updates
- browser smoke tests
- determinism checks
- Lean proof jobs when the verification boundary changed

### 4. Operator proof
Record the command or repeatable workflow that demonstrates the milestone.

Examples:
- `cargo run -p kali -- check fixtures/...`
- `cargo run -p kali -- build --bundle ...`
- `mise run lean-proofs`

### 5. Regression proof
Rerun the baseline verification for the repository state:
- `cargo test --workspace`
- any stage-specific canonical task (`mise` task, browser smoke lane, proof lane, etc.)

## Dependency-writing rule

A `Depends on` line should list only the stages that are true prerequisites for the milestone.

Use these distinctions consistently:
- **hard dependency** — the stage cannot start meaningfully before this prerequisite lands
- **ordering note** — recommended sequence for workability, but not a semantic prerequisite
- **coordination requirement** — may proceed in parallel if shared surfaces stay aligned

Do not collapse all three into one vague dependency list.

## Scope-writing rule

Every stage should explicitly name what it does **not** do.

This prevents phase bleed such as:
- Phase-1 docs accidentally claiming Phase-2 effect reporting,
- package-install progress being mistaken for Node compatibility,
- browser-targeted build support being mistaken for a standalone browser runtime.

## Historical-stage rule

Some stage files in this repository are already closed and now describe historical milestones.
That is fine.

When a stage is historical:
- keep the original milestone visible,
- state clearly that the stage is complete,
- point readers to `specs/19-feature-maturity.md` for public availability,
- point readers to the owning spec chapter for current behavior.

Historical stage docs should remain useful as implementation archaeology, not turn into a second source of truth for current support claims.

## Parallel-development rule

Parallel work is allowed only when both of these are true:
1. `../PLAN.md` or the phase README says the streams may proceed in parallel.
2. The streams coordinate on shared surfaces: CLI, diagnostics, schemas, maturity rows, and tests.

If either condition is missing, default to sequential work.

## Commit and review guidance

A good stage-sized commit should:
- preserve workability,
- have an explainable milestone,
- include the required docs/spec/test packet,
- and avoid bundling unrelated later-phase breadth.

Recommended commit-message style:
- `feat: implement lexer [stage 1.2]`
- `feat: open browser bundle artifacts [stage 1.11]`
- `docs: refine implementation roadmap and workstream guides`

## Anti-overclaim checklist

Before merging plan changes, check:
- Does the plan accidentally imply earlier shipping than `specs/19-feature-maturity.md` allows?
- Does a build-only/browser-targeted path get mistaken for runtime support?
- Does install support get mistaken for executable package compatibility?
- Does proof-ready language get mistaken for proof-backed language?
- Does a stage mention a command shape without saying whether it is only defined early or actually shipped?

If yes, tighten the wording.
