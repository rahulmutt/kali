# Spec Mailbox — Stage 1.12 Notes

## Observations and Required Spec Updates

### 1. Lint warning/error code registry is still missing

Stage 1.12 needs a canonical `W2xxx` registry for the initial built-in lint rules so `kali lint` can emit stable machine-friendly diagnostics.

Proposed first-pass registry for `specs/15-errors.md`:
- `W2000` — `no-unused-vars`
- `W2001` — `no-unused-imports`
- `W2002` — `no-explicit-any`
- `W2003` — `prefer-const`
- `W2004` — `no-var`
- `W2005` — `eqeqeq`
- `W2006` — `no-debugger`
- `W2007` — `no-console`
- `W2008` — `no-empty`
- `W2009` — `no-unreachable`
- `W2010` — `no-undef`

### 2. Severity normalization for hard lint rules

The stage-1 linter needs a consistent spec note for which lint rules are warnings vs errors. The current implementation plan treats `no-debugger` and `no-unreachable` as hard failures while keeping the rest as warnings.

**Date:** 2026-04-12
**Status:** Pending spec registry update for lint diagnostics
