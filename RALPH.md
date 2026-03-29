# RALPH

- Iteration: 10 of 10
- Prompt file: /workspace/prompts/iterate-spec.md
- Updated: 2026-03-29T13:00:17.767Z

## Last emitted message

```text
Done.

Updated:
- `/workspace/SPEC.md`
- `/workspace/specs/04-type-system.md`
- `/workspace/specs/07-specialization.md`
- `/workspace/specs/18-schemas.md`
- `/workspace/specs/19-feature-maturity.md`

What I improved

- Added cross-spec terminology to `SPEC.md`
  - defines:
    - `linked artifact`
    - `compatibility mode`
    - `dynamic effect boundary`
  - this reduces repeated ad hoc wording across specs

- Tightened phaseing around effects
  - `specs/04-type-system.md` now makes it explicit that:
    - built-in effect inference / `pure` / explicit effect annotations are Phase 2
    - user-defined / algebraic effects are later and experimental
  - this now matches the rest of the sandboxing story better

- Fixed a specialization inconsistency
  - `specs/07-specialization.md` previously said:
    - Phase 3 = “full specialization, no cap”
    - but elsewhere still described fallback behavior
  - now it consistently says:
    - `--release-advanced` greatly expands specialization
    - but still keeps an emergency fallback to avoid pathological code size

- Reduced schema drift
  - `specs/18-schemas.md` now explicitly maps policy keys to canonical effect names, e.g.
    - `fileSystem.read` ↔ `FileSystem.Read`
    - `process.spawn` ↔ `Process.Spawn`
  - this makes the policy schema, effect report schema, and type/effect system line up more clearly

- Extended the maturity matrix
  - `specs/19-feature-maturity.md` now includes:
    - built-in effect inference / `kali effects`
    - explicit effect annotations / `pure`
    - algebraic effect declarations / handlers
  - this gives a canonical place for their phase status

Why these changes help

- fewer implicit assumptions between specs
- less risk of future contradictions around effects
- clearer MVP vs later-phase boundaries
- simpler implementation guidance for the compiler/runtime split

Main inconsistencies fixed

1. Effect syntax and capability summaries were not fully phase-aligned
2. Specialization “no cap” conflicted with fallback rules
3. Policy JSON keys and effect-kind names were related, but not explicitly normalized
4. Some important shared terms existed informally but not centrally

Good next improvements

1. Add a short compatibility target appendix
   - define what “TypeScript-compatible” means by phase:
     - syntax
     - type-system behavior
     - runtime behavior
     - package compatibility

2. Add a canonical “unsupported / gated features” diagnostic table
   - e.g. for:
     - `Proxy`
     - `eval`
     - dynamic `require()`
     - non-literal `import(expr)`

3. Normalize all phase-sensitive features to reference `specs/19-feature-maturity.md`
   - some files still restate status inline instead of pointing back to the matrix

If you want, I can do that final normalization pass across the remaining spec files too.
```
