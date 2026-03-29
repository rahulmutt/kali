# RALPH

- Iteration: 10 of 10
- Prompt file: /workspace/prompts/iterate-spec.md
- Updated: 2026-03-29T14:38:29.880Z

## Last emitted message

```text
Updated the specs to tighten a few inconsistencies and simplify duplication.

Changed files:
- `/workspace/SPEC.md`
- `/workspace/specs/11-standard-apis.md`
- `/workspace/specs/12-cli.md`
- `/workspace/specs/19-feature-maturity.md`

What I fixed

1. Test command/API-surface ambiguity
- Added explicit `kali test` host-profile rules to:
  - `SPEC.md`
  - `specs/12-cli.md`
  - `specs/11-standard-apis.md`
  - `specs/19-feature-maturity.md`
- Canonicalized:
  - `kali test` / `--api deno` = Phase 1 MVP
  - `kali test --api node` = Phase 3 target
  - `kali test --api browser` = rejected by default early

This removes drift between `run`, `build`, `check`, and `test`.

2. CLI config inconsistency
- `specs/12-cli.md` had an older config example using `compilerOptions.api`.
- Replaced it with the canonical naming:
  - `compilerOptions.apiSurface`
  - `compilerOptions.buildMode`
  - `compilerOptions.runtimeProfiles`

3. Spec simplification
- Removed the large duplicated full `kali.json` example from `specs/12-cli.md`.
- Replaced it with a minimal canonical shape and a link to `specs/18-schemas.md`, which should remain the schema source of truth.

4. Minor wording cleanup
- Clarified browser build wording in CLI:
  - “browser build path requires `--bundle`”
- Added explicit note that `kali test` follows the same early-phase API gating as `kali run`.

Why these changes help

- Reduces duplication between CLI docs and schema docs
- Makes command/profile behavior more machine-checkable
- Avoids undocumented “maybe supported” interpretations for browser/node test flows
- Keeps `SPEC.md` and the maturity matrix aligned

If you want, I can do a second pass focused on:
- simplifying repeated phase-language across all `specs/*.md`
- tightening package-resolution wording
- auditing memory/effect/runtime chapters for any remaining subtle contradictions
```
