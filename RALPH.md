# RALPH

- Iteration: 10 of 10
- Prompt file: /workspace/prompts/iterate-spec.md
- Updated: 2026-03-29T15:08:45.962Z

## Last emitted message

```text
Made a spec cleanup pass and fixed a few inconsistencies.

Updated:
- `specs/05-ir.md`
- `specs/06-memory.md`
- `specs/07-specialization.md`
- `specs/14-packages.md`
- `specs/16-testing.md`
- `RALPH.md`

What I improved

1. Ownership vs layout terminology
- In `specs/05-ir.md`, I made it explicit that:
  - layout (`static`/`dynamic`) and
  - ownership (`stack`/`owned heap`/`shared heap`/`borrowed`)
  are separate axes.
- This removes ambiguity where “heap allocated” could be misread as “dynamic object layout”.

2. Canonical ownership naming
- Standardized spec language around `shared heap` instead of `Rc`.
- Renamed MIR ownership wording to:
  - `Stack`
  - `OwnedHeap`
  - `SharedHeap`
  - `Borrowed`
- Updated related wording in memory/optimization/testing docs.

3. Cycle handling simplified
- In `specs/06-memory.md`, I simplified cycle handling:
  - prefer acyclic ownership when provable
  - use shared-heap fallback for cyclic graphs
  - allow targeted deterministic cycle reclamation
  - allow region teardown for short-lived sandbox runs
- Also clarified this does not imply early support for JS weak-ref APIs like `WeakMap` or `FinalizationRegistry`.

4. `--fast` behavior clarified
- In `specs/07-specialization.md`, I tightened the wording so `--fast` means:
  - skip most user-authored generic specialization
  - but still keep monomorphic/static optimizations where already justified
- This avoids the misleading reading that `--fast` makes everything boxed/dynamic.

5. Browser package resolution consistency
- In `specs/14-packages.md`, I aligned browser-targeted resolution rules for both:
  - `kali check --api browser`
  - `kali build --bundle --api browser`
- This reduces the risk of `check` and `build` resolving different package branches.

Notable inconsistency fixed
- `specs/06-memory.md` previously mixed internal cycle-management ideas with weak-reference semantics.
- I separated those cleanly so ordinary object cycles are handled internally, while weak-ref APIs remain later-phase features.

If you want, I can also do one last pass over:
- `04` / `09` / `18` for effect-system naming consistency, or
- `10` / `11` / `12` / `19` for runtime/API/profile cross-checking.
```
