# RALPH

- Iteration: 63 of 1000
- Prompt file: /workspace/prompts/iterate-spec.md
- Updated: 2026-03-29T22:35:00Z

## Last emitted message

```text
Done.

Changed:
- `SPEC.md`
- `specs/12-cli.md`
- `specs/14-packages.md`
- `specs/15-errors.md`
- `specs/19-feature-maturity.md`
- `RALPH.md`

What I improved:
- clarified the exact boundary for early `package-audit`:
  - it still uses normal project/config discovery for generic CLI behavior
  - but it intentionally ignores `apiSurface`, `runtimeProfiles`, `compat.features`, and top-level `sandbox`
- simplified the machine-readable story for `package-audit` so it does not accidentally grow a second native bare-JSON format before a dedicated audit schema exists
- fixed a small install/docs inconsistency by making `kali install --dev https://...` explicitly invalid with `E5008` across CLI, package, error, and maturity docs
- added the missing `package-audit https://...` invalid-usage example to the error catalog

Commit:
- pending

Push:
- pending
```
