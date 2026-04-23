# Plan Set Guide

This directory is Kali's implementation playbook.

Use it in this order:

1. [`../PLAN.md`](../PLAN.md) — top-level sequencing, phase map, critical path, and completion gates
2. [`00-planning-conventions.md`](./00-planning-conventions.md) — shared stage rules and workable-state discipline
3. [`01-repository-layout.md`](./01-repository-layout.md) — long-lived repository structure and ownership boundaries
4. [`02-workstreams-and-handoffs.md`](./02-workstreams-and-handoffs.md) — cross-phase streams and safe parallelism
5. [`03-spec-to-stage-traceability.md`](./03-spec-to-stage-traceability.md) — spec chapter to stage/evidence mapping
6. [`04-stage-dependency-matrix.md`](./04-stage-dependency-matrix.md) — compact per-stage prerequisites, demos, and evidence lanes
7. [`05-delivery-increments.md`](./05-delivery-increments.md) — milestone-sized workable repository states
8. [`06-current-workspace-rollout.md`](./06-current-workspace-rollout.md) — concrete crate/directory growth order for this repository
9. [`07-roadmap-status-and-next-steps.md`](./07-roadmap-status-and-next-steps.md) — recommended next execution lanes and prioritization guidance
10. [`08-fresh-implementation-roadmap.md`](./08-fresh-implementation-roadmap.md) — the shortest fresh-start route through the stage graph
11. [`09-stage-acceptance-checklists.md`](./09-stage-acceptance-checklists.md) — close-out checklist by milestone family
12. [`10-risk-register.md`](./10-risk-register.md) — cross-spec risks and mandatory mitigations
13. The relevant phase index under `phase-*/README.md`
14. The exact stage document you are implementing

## Directory map

```text
plan/
├── README.md                           # this guide
├── 00-planning-conventions.md          # shared planning rules
├── 01-repository-layout.md             # target repository structure + ownership
├── 02-workstreams-and-handoffs.md      # cross-phase stream coordination
├── 03-spec-to-stage-traceability.md    # spec chapter -> stage/evidence mapping
├── 04-stage-dependency-matrix.md       # per-stage dependency and demo matrix
├── 05-delivery-increments.md           # milestone-sized workable repository states
├── 06-current-workspace-rollout.md     # concrete growth order for the current workspace
├── 07-roadmap-status-and-next-steps.md # near-term roadmap guidance
├── 08-fresh-implementation-roadmap.md  # fresh-start execution overlay
├── 09-stage-acceptance-checklists.md   # close-out criteria by stage family
├── 10-risk-register.md                 # cross-cutting implementation risks
├── phase-1/                            # MVP compiler/toolchain stages
├── phase-2/                            # ownership/effects/embedding/verification foundation
├── phase-3/                            # optimization and compatibility breadth
├── phase-4/                            # hard dynamic features and proof depth
└── phase-5/                            # explicitly deferred later-compatibility work
```

## Workspace reading note

The plan uses a **logical ownership** vocabulary so the roadmap stays readable over time.
In this repository, those logical buckets map to the current fine-grained `kali_*` crates.
Use [`01-repository-layout.md`](./01-repository-layout.md) and [`06-current-workspace-rollout.md`](./06-current-workspace-rollout.md) whenever you need the exact current-repo mapping.

## Reading shortcuts

- **What should be built next?** → `../PLAN.md` → phase README → stage file
- **What files/areas should this work touch?** → [`01-repository-layout.md`](./01-repository-layout.md)
- **Can two streams proceed in parallel?** → `../PLAN.md` + [`02-workstreams-and-handoffs.md`](./02-workstreams-and-handoffs.md)
- **Which stage owns a given spec chapter or maturity row?** → [`03-spec-to-stage-traceability.md`](./03-spec-to-stage-traceability.md)
- **What are the exact prerequisites and demo for one stage?** → [`04-stage-dependency-matrix.md`](./04-stage-dependency-matrix.md)
- **What usable milestone should the repo reach next?** → [`05-delivery-increments.md`](./05-delivery-increments.md)
- **Which current crates/directories should grow next?** → [`06-current-workspace-rollout.md`](./06-current-workspace-rollout.md)
- **What are the recommended next execution lanes?** → [`07-roadmap-status-and-next-steps.md`](./07-roadmap-status-and-next-steps.md)
- **What is the shortest path for a fresh implementation push from the current specs?** → [`08-fresh-implementation-roadmap.md`](./08-fresh-implementation-roadmap.md)
- **What must be true before closing a stage?** → [`00-planning-conventions.md`](./00-planning-conventions.md) + [`09-stage-acceptance-checklists.md`](./09-stage-acceptance-checklists.md)
- **Which risks should shape the implementation packet and extra hardening work?** → [`10-risk-register.md`](./10-risk-register.md)
- **Is the feature publicly shipped yet?** → [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md), not this directory

## Maintenance rule

The plan set owns implementation order and milestones only.

If a change affects:
- product behavior or support claims → update `SPEC.md` / `specs/`
- public availability wording → update `specs/19-feature-maturity.md`
- current proof-backed status → update `proofs/BOUNDARY.md`
- implementation sequencing or stage boundaries → update this `plan/` tree
