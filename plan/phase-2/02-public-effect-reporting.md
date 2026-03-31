# Stage 2.2 — Public Effect Reporting

**Phase:** 2 — Ownership, Effects & Public Embedding  
**Spec refs:** [`specs/09-sandboxing.md`](../../specs/09-sandboxing.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/18-schemas.md`](../../specs/18-schemas.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)

## Goal

Open the stable **public effect-report surface** in both of its explicit halves:

1. **Reporting half**: `kali effects <file>` (one-root source-graph command) and
   `kali package-effects <package>` (single-package registry-analysis command) emit stable
   schema-v1 effect-report JSON.
2. **Policy-comparison half**: `kali check --sandbox` and `kali build --sandbox` extend their
   Phase-1 static-validation behaviour with compile-time inferred-effect-vs-policy rejection.

## Workable Milestone

- `kali effects <file>` analyses the source graph and emits a structured JSON effect report.
- `kali package-effects <pkg>` analyses a single installed package and emits a per-package
  effect report.
- `kali check --sandbox` / `kali build --sandbox` now reject when inferred effects exceed the
  declared policy (in addition to the existing schema/config validation from Phase 1).
- Phase-1 gating tests for these commands are updated from "assert unavailable" to positive
  coverage.

## Tasks

### 1. Effect model

Formalise the effect vocabulary used internally since Phase 1 into a stable public schema.
An *effect* is a capability used by a program that the sandbox model can reason about:

| Effect kind | Example |
|---|---|
| `fs.read(path_glob)` | `Deno.readTextFile("./data.json")` |
| `fs.write(path_glob)` | `Deno.writeTextFile("./out.txt", data)` |
| `net.connect(host_pattern)` | `fetch("https://api.example.com/...")` |
| `env.read(var_name)` | `Deno.env.get("HOME")` |
| `env.write(var_name)` | `Deno.env.set("PATH", ...)` |
| `proc.exit` | `Deno.exit(0)` |

Effects are inferred conservatively: if Kali cannot determine at compile time that an effect is
absent, it is reported as potentially present.

### 2. `kali effects <file>` — one-root source-graph command

```
kali effects <file>
kali effects --output json <file>
```

Analysis: walk the source graph rooted at `<file>`, collect all inferred effects, and emit a
report. This is a **native-JSON command**: its default success output is structured JSON (no need
for `--output json` to get machine-readable output, though `--output json` still wraps it in the
standard envelope).

Effect report JSON shape (schema `schemas/result/effects/v1.json`):

```json
{
  "$schema": "https://kali-lang.org/schemas/result/effects/v1",
  "schemaVersion": 1,
  "entrypoint": "src/main.ts",
  "effects": [
    { "kind": "net.connect", "pattern": "api.example.com:443", "definite": false },
    { "kind": "fs.read", "pattern": "./data/**", "definite": true }
  ]
}
```

`definite: true` means the effect always occurs on any execution path; `false` means it may occur
conditionally.

`kali effects` is **not** a dry-run variant of `kali run`. It is a static analysis command that
does not execute the program.

### 3. `kali package-effects <package>` — single-package registry-analysis command

```
kali package-effects <pkg>@<version>
kali package-effects --output json <pkg>@<version>
```

Analyses a single installed package (not a project source graph). Reports which effects the
package's published API surface may produce. This command participates in the
**registry-analysis command** workflow — it answers a different question from `kali effects`
(which is a source-graph question).

The dual classification: by command/input shape it is a registry-analysis command; by output
contract it is part of the public effect-report surface. Both classifications are intentional.

Per-package effect report schema: `schemas/result/package-effects/v1.json`.

### 4. Policy-comparison half: inferred-effect-vs-policy rejection

Extend `kali check --sandbox <policy>` and `kali build --sandbox <policy>`:

- **Phase 1** (already shipped): validate the policy schema/config.
- **Phase 2 addition**: run effect inference on the source graph; compare inferred effects
  against the policy's `allow` entries; emit `E9007` for each inferred effect not covered by the
  policy.

```
E9007: inferred effect 'net.connect("api.example.com:443")' is not permitted by the active policy
  --> src/service.ts:42:5
  = note: add "api.example.com:443" to allow.net in your policy file
```

This is not a new `run --dry` or `test --dry` workflow variant. It is the Phase-2 extension of
the same `check/build --sandbox` command paths that existed in Phase 1.

### 5. Tests

- `kali effects fixtures/effects-app.ts` → JSON report matches golden snapshot.
- `kali effects fixtures/pure-compute.ts` → empty effects list.
- `kali package-effects lodash@4.17.21` → reports zero network/FS effects (lodash is pure compute).
- `kali check --sandbox fixtures/deny-net.json fixtures/fetch.ts` → now emits `E9007` for the
  net effect (in addition to the existing schema-validation checks).
- Gating tests from Phase 1 for `kali effects` and `kali package-effects` are updated to positive
  coverage.

## Out of Scope

- `kali package-audit` (Later compatibility).
- Effects on `--api node` surface (Phase 3 target).
- Programmable policy predicates (later compatibility).
