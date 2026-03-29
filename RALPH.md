# RALPH

- Iteration: 10 of 10
- Prompt file: /workspace/prompts/iterate-spec.md
- Updated: 2026-03-29T14:12:56.943Z

## Last emitted message

```text
I reviewed `BOOTSTRAP.md` against the current spec set and applied another cleanup pass.

Updated files:
- `SPEC.md`
- `specs/04-type-system.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

What I changed

- `SPEC.md`
  - Added a canonical **Phase 1 non-goals** section.
  - Tightened wording around:
    - no full Node parity in Phase 1
    - no browser/DOM runtime in Phase 1
    - no dynamic-code compatibility release in Phase 1
    - no stable public embedding ABI in Phase 1
    - no full static effect proof system in Phase 1
  - This reduces drift across other specs.

- `specs/04-type-system.md`
  - Added a canonical **JavaScript fallback contract**.
  - Clarified the ordered fallback ladder:
    - precise static type + stable layout
    - small unions
    - `unknown` at boundaries
    - dynamic/tagged values
    - dynamic object layout
    - reject unsupported semantics rather than guessing
  - Also made explicit that Kali should not invent fresh `any` just to avoid analysis.

- `specs/18-schemas.md`
  - Added a canonical **built-in effect naming table**.
  - Centralized mappings like:
    - `FileSystem.Read`
    - `Network.Fetch`
    - `Process.EnvRead`
    - `Timer.Schedule`
    - `Random.GetBytes`
    - `Console.Write`
    - `Eval`
  - Simplified sandbox-policy wording so it points to this table instead of redefining names inline.

- `specs/19-feature-maturity.md`
  - Added **phase exit criteria** for Phases 1–4.
  - This makes the maturity doc actionable instead of only descriptive.

Main improvements from this pass

1. Better simplification
   - Phase 1 is now more clearly bounded.
   - Fewer places need to repeat caveats.

2. Better consistency
   - Effect naming is now centralized.
   - JS fallback behavior is now explicit instead of implied.

3. Better execution guidance
   - Phase labels now have exit checklists.
   - Easier to use the spec set as a delivery roadmap.

Remaining suggestion

One additional improvement I’d still consider:
- add a short “canonical terminology” glossary somewhere near `SPEC.md` or `specs/01-architecture.md`
  - e.g. define:
    - “dynamic/tagged value”
    - “dynamic object layout”
    - “linked artifact”
    - “profile”
    - “compatibility path”
  - That would reduce subtle wording drift later.

If you want, I can do one more pass focused purely on cross-file terminology cleanup and deduplicating repeated phase language.
```
