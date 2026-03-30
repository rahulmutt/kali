# RALPH

- Iteration: 402 of 1000
- Prompt file: /workspace/prompts/iterate-spec.md
- Updated: 2026-03-30T17:51:28+00:00

## Last emitted message

```text
Done.

Updated:
- `specs/11-standard-apis.md`
- `specs/14-packages.md`
- `specs/15-errors.md`
- `RALPH.md`

What I changed:
- aligned the API-surface snapshot wording so Phase-1 browser support consistently includes supported `--sandbox` variants and equivalent inherited-config forms
- simplified the package-resolution browser rewrite note to point at the exact **Phase-1 browser-targeted command set** instead of re-listing examples with slightly different wording
- filled an error-spec gap by adding the inherited-browser `--sandbox` contradiction examples for `build --capi` and `build --component`

Main inconsistency/drift risk fixed:
- browser-targeted support and its inherited-config equivalents were described consistently in the CLI/maturity specs, but the API-surface snapshot and package-resolution prose were still a bit narrower/looser
- the error chapter also omitted two inherited-browser `E5008` examples that the CLI chapter already treated as invalid command shape

Git:
- commit: pending
- message: pending

Push:
- pending
```
