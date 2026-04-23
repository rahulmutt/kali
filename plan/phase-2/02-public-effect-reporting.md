# Stage 2.2 — Public Effect Reporting

**Phase:** 2 — Ownership, Effects & Public Embedding  
**Spec refs:** [`specs/09-sandboxing.md`](../../specs/09-sandboxing.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/18-schemas.md`](../../specs/18-schemas.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [2.1 — MIR & Ownership Analysis](01-mir-and-ownership.md) (effect inference runs over the canonical mid-pipeline analysis graph)

## Goal

Open the stable **public effect-report surface** in both of its explicit halves:

1. **Reporting half**
   - `kali effects <file>` as the one-root source-graph report command
   - `kali package-effects <package>` as the single-package registry-analysis report command
2. **Policy-comparison half**
   - `kali check --sandbox ...`
   - `kali build --sandbox ...`

The stage must follow the current schema/CLI split exactly: `kali effects` and
`kali package-effects` are the Phase-2 **native-JSON commands**, while policy comparison extends
existing `check/build --sandbox` flows instead of inventing a second sandbox command family.

## Workable Milestone

- `kali effects <file>` emits the schema-owned reusable **EffectReport** payload.
- `kali package-effects <package>` emits the schema-owned outer package payload plus the nested
  reusable **EffectReport**.
- `kali check --sandbox` / `kali build --sandbox` reject inferred built-in effects that the active
  policy does not permit, using the canonical diagnostic path.
- Phase-1 unavailability tests for these surfaces are replaced by positive evidence.

## Progress

**Status:** Complete. The public reporting surface is implemented and aligned with the current
schema-v1 command and payload rules.

Progress note:
- `kali effects` now accepts both explicit `--api browser` and inherited browser API-surface contexts, and the CLI smoke suite pins the same stable effect payload shape under those browser-analysis paths.
- `kali package-effects` now follows the same browser-configured materialized-package resolution path as the other browser-aware analysis commands, so browser-targeted package reports analyze the browser entrypoint instead of the default standalone main entry when the project manifest selects browser analysis.
- `kali package-effects` now has explicit smoke coverage for its native JSON payload, pretty-printed native output, and `--output json` envelope form, keeping the registry-analysis report contract exercised across both presentation modes.

## Historical stage tasks

### 1. Stabilize the built-in effect vocabulary

Promote the internal sandbox-oriented bookkeeping from Phase 1 into the public built-in vocabulary
owned by [`specs/18-schemas.md`](../../specs/18-schemas.md):

- `FileSystem.Read`
- `FileSystem.Write`
- `Network.Fetch`
- `Network.Connect`
- `Network.Listen`
- `Process.Spawn`
- `Process.EnvRead`
- `Process.EnvWrite`
- `Timer.Schedule`
- `Random.GetBytes`
- `Console.Write`
- `Eval`

The public surface reports a conservative upper bound. Dynamic/incomplete cases are surfaced by the
shared `dynamicEffects` and `dynamicReasons` fields rather than by an ad hoc second effect format.

### 2. `kali effects <file>` — source-graph effect reporting

The CLI contract is the schema-owned one:

```bash
kali effects <file>
kali effects --pretty <file>
kali effects --output json <file>
kali effects --pretty --output json <file>
```

Key rules preserved by the stage:

- default success mode emits the native **EffectReport** JSON payload
- `--output json` wraps that payload in the standard command envelope
- `--pretty` reformats the active JSON document only; it does not create a second availability path
- the report records `analysisContext`, `entryPoints`, `effects`, `dynamicEffects`, and
  `dynamicReasons` using the exact schema-owned field names

### 3. `kali package-effects <package>` — registry-analysis effect reporting

This command follows the schema-v1 registry-analysis split:

```bash
kali package-effects lodash
kali package-effects --pretty lodash
kali package-effects --output json lodash
```

The package selector is the canonical **identity-only registry target** (`lodash`,
`jsr:@std/path`, etc.), not an inline `pkg@version` selector. The resolved concrete version is
recorded in the emitted payload's `package` coordinate after the command applies the shared
stable-release selection rule.

The command remains registry-analysis by input shape and effect-reporting by output contract.
Those two classifications are both intentional.

### 4. Policy-comparison half on `check/build --sandbox`

Extend the existing Phase-1 static policy-validation surface so that once effect inference is
publicly stable:

- `kali check --sandbox ...` compares inferred built-in effects against the active policy
- `kali build --sandbox ...` does the same for build lanes that accept sandbox attachment
- denials use the canonical compile-time policy-comparison diagnostic path (`E9007`)

This remains the pass/fail half of the same public effect-report surface; it is not a hidden
`run --dry`, `test --dry`, or `effects --sandbox` command family.

### 5. Evidence

- positive golden coverage for `kali effects`
- positive golden coverage for `kali package-effects`
- policy-comparison positives and negatives on `check/build --sandbox`
- deterministic JSON ordering tests for effect kinds, locations, and dynamic-reason arrays
- inherited-context tests proving browser/Node/threaded/compatibility contexts follow the same
  maturity gates as their explicit source-graph counterparts

## Out of Scope

- `kali package-audit` (Phase 4 compatibility)
- user-defined/custom effect kinds in the stable public machine contracts
- executable project-local sandbox policy code

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
