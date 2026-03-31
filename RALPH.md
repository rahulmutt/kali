# RALPH

- Iteration: 2 of 10
- Prompt file: /workspace/prompts/iterate-plan.md
- Updated: 2026-03-31T09:12:25.450Z

## Last emitted message

```text
Here's a summary of every improvement made and why:

---

### Improvements made

**`PLAN.md` — 4 additions**

| Addition | Rationale |
|---|---|
| **Phase 1 parallelism note** | Phases 2 and 3 already had explicit parallelism notes; Phase 1 was missing one. Stage 1.9's static-validation work (policy parsing, `kali check --sandbox`) only needs stage 1.5 — not 1.8 — so it can overlap with the execution stages. 1.12 and 1.13 are also independent of each other. |
| **Phase 1 completion gate** | No phase had an explicit "done means X" criterion. Now each phase names the concrete evidence (passing DoDs + maturity matrix updates) required to formally close it. |
| **Spec chapter mapping tables for Phases 2–4** | Phase 1 had a mapping table showing how plan stages relate to spec steps. Phases 2–4 had none, making it harder to cross-reference with the owning chapters in `specs/`. |
| **Phase completion gates for Phases 2–4** | Same rationale as Phase 1; each phase now has a concise closing criterion. |

**`plan/phase-1/09-sandbox-and-policy.md`**

The `Depends on` field incorrectly implied the entire stage needed stage 1.8 (runtime). In reality only the *runtime enforcement* portion needs 1.8; the *static policy validation* portion (`kali check --sandbox`, policy schema parsing) depends only on stage 1.5. Corrected and cross-referenced with the new parallelism note.

**`plan/phase-2/01-03`, `plan/phase-3/01-03`, `plan/phase-4/01` — 7 stage files**

All Phase 1 stages (14 files) have a `## Definition of Done` checklist; none of the Phase 2–4 stages (7 files) did. Added concrete, checkable DoD items to every missing stage so the plan's structure is now uniform across all 22 stage documents.
```
