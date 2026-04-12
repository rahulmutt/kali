# Spec Mailbox — Stage 2.2 Public Effect-Reporting Notes

## Observations and Required Spec Updates

### 1. The public effect-report surface is now being implemented

Stage 2.2 work wires the stable `kali effects` and `kali package-effects` commands into the CLI, along with inferred-effect-vs-policy validation on `check/build --sandbox`.

The spec/docs alignment packet now includes:
- `specs/09-sandboxing.md` — compile-time effect-vs-policy rejection now names the dedicated `E9007` comparison diagnostic alongside the existing runtime `E4001` sandbox-policy violation path.
- `specs/15-errors.md` — sandbox diagnostics now distinguish runtime `E4001` enforcement failures from the Phase-2 compile-time `E9007` comparison failure.
- `README.md`, `specs/12-cli.md`, and `specs/19-feature-maturity.md` already reflect the current availability split for the new reporting surface.

### 2. Effect-policy diagnostic code alignment

The policy-comparison path uses the Phase-2 effect-vs-policy diagnostic slot from the sandbox error registry. The implementation maps that path to the canonical `E9007` wording used in the spec set.

**Date:** 2026-04-12  
**Status:** Processed — effect-reporting implementation landed and the spec/docs packet is aligned
