# Plan Set Guide

This directory is the implementation playbook for Kali.

Use it in this order:

1. [`../PLAN.md`](../PLAN.md) — top-level sequencing, phase map, critical path, and completion gates.
2. [`00-planning-conventions.md`](./00-planning-conventions.md) — shared rules for stage design, workable-state discipline, and update packets.
3. [`01-repository-layout.md`](./01-repository-layout.md) — recommended long-lived repository structure and when each area should appear.
4. [`02-workstreams-and-handoffs.md`](./02-workstreams-and-handoffs.md) — cross-phase streams, ownership boundaries, and safe parallelism.
5. [`03-spec-to-stage-traceability.md`](./03-spec-to-stage-traceability.md) — spec-chapter-to-stage mapping and evidence ownership.
6. [`04-stage-dependency-matrix.md`](./04-stage-dependency-matrix.md) — compact per-stage prerequisites, demos, ownership areas, and evidence lanes.
7. [`05-delivery-increments.md`](./05-delivery-increments.md) — milestone-sized slices that keep the repository usable between stage closures.
8. [`06-current-workspace-rollout.md`](./06-current-workspace-rollout.md) — concrete crate/directory growth order for this repository.
9. The relevant phase index under `phase-*/README.md`.
10. The exact stage document you are implementing.

## Directory map

```text
plan/
├── README.md                         # this guide
├── 00-planning-conventions.md        # shared planning rules
├── 01-repository-layout.md           # target repository structure + rollout
├── 02-workstreams-and-handoffs.md    # cross-phase stream coordination
├── 03-spec-to-stage-traceability.md  # spec chapter -> stage/evidence mapping
├── 04-stage-dependency-matrix.md     # per-stage dependency and demo matrix
├── 05-delivery-increments.md         # milestone-sized workable repository states
├── 06-current-workspace-rollout.md   # concrete growth order for the current workspace
├── phase-1/                          # MVP compiler/toolchain stages
├── phase-2/                          # ownership/effects/embedding/verification foundation
├── phase-3/                          # optimization and compatibility breadth
├── phase-4/                          # hard dynamic features and proof depth
└── phase-5/                          # explicitly deferred later-compatibility work
```

## Current repository reading note

The plan uses a **logical ownership** vocabulary (`crates/core`, `crates/runtime`, `crates/packages`, and so on) so the roadmap stays readable over time.
In this repository, those buckets currently map onto finer-grained crates such as `kali_lexer`, `kali_parser`, `kali_codegen`, `kali_runtime`, `kali_npm`, and the host API crates.
Use [`01-repository-layout.md`](./01-repository-layout.md) whenever you need the exact current-repo mapping.

## Reading shortcuts

- **What should I build next?** → `../PLAN.md` → phase README → stage file.
- **What files/areas should this work touch?** → [`01-repository-layout.md`](./01-repository-layout.md).
- **Can two streams proceed in parallel?** → `../PLAN.md` plus [`02-workstreams-and-handoffs.md`](./02-workstreams-and-handoffs.md).
- **Which stage owns a given spec chapter or maturity row?** → [`03-spec-to-stage-traceability.md`](./03-spec-to-stage-traceability.md).
- **What are the exact prerequisites and demo for one stage?** → [`04-stage-dependency-matrix.md`](./04-stage-dependency-matrix.md).
- **What usable milestone should the repo reach next?** → [`05-delivery-increments.md`](./05-delivery-increments.md).
- **Which current crates/directories should grow next?** → [`06-current-workspace-rollout.md`](./06-current-workspace-rollout.md).
- **What must be true before closing a stage?** → [`00-planning-conventions.md`](./00-planning-conventions.md).
- **Is the feature publicly shipped yet?** → [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md), not this directory.

## Maintenance rule

The plan set owns implementation order and milestones only.

If a change affects:
- product behavior or support claims → update `SPEC.md` / `specs/`
- public availability wording → update `specs/19-feature-maturity.md`
- current proof-backed status → update `proofs/BOUNDARY.md`
- implementation sequencing or stage boundaries → update this `plan/` tree
