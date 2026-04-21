# Stage 1.12 — Developer Workflow

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/15-errors.md`](../../specs/15-errors.md), [`specs/18-schemas.md`](../../specs/18-schemas.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.5 — Type Checker](05-type-checker.md), [1.8 — Runtime & Execution](08-runtime-execution.md), [1.11 — Build Artifacts](11-build-artifacts.md)

## Goal

Complete the Phase-1 developer workflow surface: `kali init`, `kali fmt`, and `kali lint`.

## Workable Milestone

- `kali init` creates the minimal current-directory project scaffold.
- `kali init --lib` creates the minimal library scaffold.
- `kali fmt [files...]` formats project files, with `--check` for read-only CI usage.
- `kali lint [files...]` reports the stable Phase-1 lint registry, with `--fix` for the supported
  lint-only autofix path.

## Progress

- `kali init` and `kali init --lib` create the minimal current-directory scaffold with
  `kali.json` plus `main.ts` or `lib.ts`.
- `kali fmt` is deterministic, supports `--check`, and is wired through the CLI.
- `kali lint` exposes the Phase-1 lint registry, supports `--fix`, and is wired through the CLI.
- Project discovery honors the current schema-v1 config/discovery rules, including nested project
  boundaries and declaration-file participation where the command allows it.

## Historical stage tasks

### 1. `kali init` scaffold

The schema-v1 contract is intentionally minimal and current-directory-scoped:

```bash
kali init
kali init --lib
```

Phase-1 scaffold rules:

- `kali init` creates `kali.json` plus `main.ts`
- `kali init --lib` creates `kali.json` plus `lib.ts`
- the canonical initial config is the smallest valid schema-v1 config:

```json
{
  "schemaVersion": 1
}
```

- `init` does not install dependencies, write `kali.lock`, or materialize packages
- `init` is current-directory-scoped; it does not retarget itself to an ancestor project root
- if the current working directory already contains `kali.json`, fail with the canonical invalid
  usage/config path from the owning specs instead of overwriting files

### 2. Formatter (`kali fmt`)

Implement the deterministic formatter and its read-only companion mode:

```bash
kali fmt [files...]
kali fmt --check [files...]
```

Requirements:

- deterministic/idempotent formatting
- project discovery when no explicit files are provided
- declaration-only files participate because `fmt` works over the canonical project file set
- `--check` reports drift without rewriting files

### 3. Linter (`kali lint`)

Implement the stable Phase-1 lint registry and the lint-only autofix path:

```bash
kali lint [files...]
kali lint --fix [files...]
```

The lint rule registry follows the canonical `W2xxx` set owned by the error spec. `--fix` applies
only structured, non-speculative lint fixes; it does not widen into checker autofix.

### 4. Project discovery

Keep discovery aligned with the config/schema rules instead of inventing per-command walkers:

- use the effective project root selected by the nearest `kali.json` when one exists
- honor `include` / `exclude`
- stop recursive discovery at nested child directories that contain their own `kali.json`
- include declaration files for commands whose file class allows them
- keep `fmt` and `lint` aligned with the canonical project file set rather than a hidden smaller
  subset

### 5. Workflow composition

After this stage, the Phase-1 developer workflow is explicit and non-magical:

```bash
kali init
kali fmt
kali lint
kali check
kali test
kali build main.ts
kali run main.ts
```

Commands remain independent. None performs hidden dependency installation or hidden command chaining.

### 6. Evidence

- `kali init` / `kali init --lib` scaffold tests
- existing-project-root rejection tests
- formatter idempotence tests
- `fmt --check` tests
- lint diagnostics and `--fix` tests
- discovery tests for explicit files vs discovery-driven runs

## Out of Scope

- checker autofix (`kali check --fix`), which remains later compatibility
- custom lint-rule plugins
- publishing workflows not owned by the current spec set

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
