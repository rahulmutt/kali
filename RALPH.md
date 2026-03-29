# RALPH

- Iteration: 10 of 10
- Prompt file: /workspace/prompts/iterate-spec.md
- Updated: 2026-03-29T13:41:57.630Z

## Last emitted message

```text
Updated the specs to tighten phase boundaries and remove drift.

Changed:
- `SPEC.md`
- `specs/07-specialization.md`
- `specs/09-sandboxing.md`
- `specs/12-cli.md`
- `specs/14-packages.md`
- `specs/19-feature-maturity.md`

What I fixed
- Removed the ad hoc `--compat dynamic-require` reference from `specs/14-packages.md`.
  - It was not defined anywhere canonically.
  - Now future compatibility paths must be defined in `specs/19-feature-maturity.md` first.
- Fixed an IR-phase inconsistency in `specs/07-specialization.md`.
  - It previously assumed specialization always lowers through MIR.
  - Now it matches the architecture spec: Phase 1 may lower directly to LIR.
- Added `Sandbox validator functions` to the canonical maturity matrix in `specs/19-feature-maturity.md`.
  - This was described in sandboxing, but not centrally tracked.
- Tightened `specs/09-sandboxing.md` to make the “declarative policy first” rule explicit and aligned with the maturity matrix.
- Gave `kali package-audit` an explicit “later tooling feature” status in:
  - `specs/12-cli.md`
  - `specs/14-packages.md`
  This avoids implying it is part of the early core delivery.
- Strengthened `SPEC.md` with two simplifying rules:
  - declarative sandbox policies first
  - no undocumented compatibility flags

Net effect
- fewer undocumented feature switches
- clearer Phase 1 vs Phase 2 vs later behavior
- better single-source-of-truth around feature gating
- slightly simpler package/runtime/sandbox story

Main inconsistencies found
1. `dynamic require` referenced a flag that didn’t exist canonically.
2. specialization flow assumed MIR even though Phase 1 permits HIR → LIR.
3. sandbox validator functions existed conceptually but had no maturity entry.
4. `package-audit` appeared as a command without a clear delivery status.

If you want, I can do one more pass and normalize the remaining specs around a few more axes:
- ensure every CLI command with phased behavior is listed in `19-feature-maturity.md`
- add a small “non-goals for Phase 1” section to `SPEC.md`
- add a config/schema section for `kali.json` similar to the sandbox policy schema so CLI/config docs don’t drift later
```
