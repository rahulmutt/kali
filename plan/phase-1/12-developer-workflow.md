# Stage 1.12 — Developer Workflow

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/15-errors.md`](../../specs/15-errors.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.5 — Type Checker](05-type-checker.md) (for `kali lint`), [1.8 — Runtime & Execution](08-runtime-execution.md) (for `kali init` project validation), [1.11 — Build Artifacts](11-build-artifacts.md) (all prior pipeline stages complete)

## Goal

Complete the Deno-inspired developer workflow surface: `kali init`, `kali fmt`, and `kali lint`.
After this stage the full set of Phase-1 CLI subcommands is available, giving developers a
coherent end-to-end workflow from project creation to formatting, linting, type-checking, running,
testing, and building.

## Workable Milestone

- `kali init` scaffolds a new project with a valid `kali.json` and starter source files.
- `kali init --lib` scaffolds a library project.
- `kali fmt [files...]` formats TypeScript/JavaScript source files in-place.
- `kali fmt --check [files...]` checks formatting without writing; exits non-zero if any file
  would change.
- `kali lint [files...]` reports lint warnings/errors.
- `kali lint --fix [files...]` auto-fixes fixable lint issues.

## Progress

- `kali init` and `kali init --lib` now create the current-directory scaffold when the target
  directory is empty, writing a minimal `kali.json` plus starter `main.ts` / `lib.ts` files.
- `kali fmt` now formats source files in-place, supports `--check`, and is wired through the CLI.
- `kali lint` now reports the initial Phase-1 built-in lint set, supports `--fix`, and is wired
  through the CLI.
- Project discovery now excludes hidden directories, nested project roots, and test files from the
  source-file walk while still including declaration files for source-oriented commands.

## Tasks

### 1. `kali init` scaffold

```
kali init [name]
kali init --lib [name]
```

`kali init` creates a minimal project scaffold in the current directory (or a new subdirectory
named `<name>` if provided):

**Executable project scaffold (`kali init`):**

```
<name>/
├── kali.json          — project manifest
├── main.ts            — "Hello, World!" entrypoint
└── main.test.ts       — minimal test stub
```

`kali.json` for an executable:

```json
{
  "$schema": "https://kali-lang.org/schemas/manifest/v1",
  "name": "<name>",
  "version": "0.1.0",
  "compilerOptions": {
    "apiSurface": "deno",
    "strict": true
  }
}
```

**Library project scaffold (`kali init --lib`):**

```
<name>/
├── kali.json          — project manifest with lib markers
└── src/
    ├── lib.ts         — exported library entry point
    └── lib.test.ts    — minimal test stub
```

`kali.json` for a library adds `"lib": true` under `compilerOptions` and sets the
`"entrypoint"` field for `kali build --lib`.

**Rules:**

- `kali init` must not invoke `kali install` or mutate any dependency state. It only creates files.
- `kali init` must not blur into a materialisation step; the user runs `kali install` separately.
- If the target directory already exists and is non-empty, emit `E5010` (init target not empty)
  rather than overwriting files silently.

Error codes:

| Code | Meaning |
|---|---|
| `E5010` | Init target directory already exists and is non-empty |
| `E5011` | Invalid project name (must be a valid npm package name) |

### 2. Formatter (`kali fmt`, `kali_fmt`)

Implement `kali_fmt` — an opinionated, deterministic TypeScript/JavaScript formatter. The formatter
operates on the AST produced by `kali_parser`, so it never re-parses source that has already been
parsed by the pipeline.

Formatting rules (Prettier-compatible defaults where possible):

- Indent: 2 spaces.
- Max line length: 80 characters (soft); 100 (hard wrap for strings/template literals).
- Trailing commas: `"all"` (ES5+ mode).
- Semicolons: always.
- Quotes: double quotes for strings; single quotes for template expressions.
- Arrow functions: always parenthesise parameters.
- Bracket spacing: `{ key: value }` not `{key: value}`.
- End-of-file newline: always.

The formatter must be **idempotent**: `fmt(fmt(x)) == fmt(x)`.

```
kali fmt [files...]          # format files in-place
kali fmt --check [files...]  # check only; exit non-zero if any would change
```

When no files are given, `fmt` discovers all source files in the project tree.

The formatter preserves comments and their attachment to the surrounding AST nodes (as threaded
in the lexer's trivia in Stage 1.2).

### 3. Linter (`kali lint`, `kali_lint`)

Implement `kali_lint` with a set of Phase-1 built-in lint rules. The linter walks the typed AST
(post name-resolution and type-checking) so rules can use type information.

Phase-1 built-in lint rules (the canonical `W2xxx` registry):

| Code | Rule | Severity | Auto-fixable |
|---|---|---|---|
| `W2000` | `no-unused-vars` | warning | no |
| `W2001` | `no-unused-imports` | warning | yes |
| `W2002` | `no-explicit-any` | warning | no |
| `W2003` | `prefer-const` | warning | yes |
| `W2004` | `no-var` | warning | yes |
| `W2005` | `eqeqeq` | warning | yes |
| `W2006` | `no-debugger` | error | yes |
| `W2007` | `no-console` | warning (off by default) | no |
| `W2008` | `no-empty` | warning | no |
| `W2009` | `no-unreachable` | error | no |
| `W2010` | `no-undef` (redundant with E3003 but useful for `--fix`) | warning | no |

Lint rules are configured in `kali.json` under `"lint": { "rules": { ... } }`.

```
kali lint [files...]          # check files; exit 1 if any errors
kali lint --fix [files...]    # auto-fix fixable issues, then check
```

When no files are given, `lint` discovers all source files in the project tree.

### 4. Project discovery

Several commands (`check`, `fmt`, `lint`, `test`) use **project discovery** when no explicit files
are provided. Implement the canonical project-discovery algorithm (shared across all
discovery-driven commands):

1. Walk the directory tree from the current working directory.
2. Include files matching the **executable/analyzable source-file class**: `.ts`, `.tsx`,
   `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`.
3. Include declaration files for `check`, `fmt`, and `lint`.
4. Exclude: `node_modules/`, `.kali-cache/`, hidden directories, and any paths listed in
   `kali.json`'s `"exclude"` array.
5. Respect `.gitignore` patterns.

### 5. Integration between workflow commands

After this stage, the typical developer workflow is:

```bash
kali init my-app          # scaffold
cd my-app
kali install              # install deps
kali fmt                  # format
kali lint                 # lint
kali check                # type-check
kali test                 # run tests
kali build main.ts        # build artifact
kali run main.ts          # run directly
```

Each command is independent; none triggers another automatically. This keeps the workflow explicit
and predictable.

### 6. Tests

- `kali init my-proj` creates the expected file tree with valid `kali.json`.
- `kali init --lib my-lib` creates the library scaffold.
- `kali init` in a non-empty directory → `E5010`.
- `kali fmt fixtures/unformatted.ts` → file content matches the expected formatted output.
- `kali fmt --check fixtures/unformatted.ts` → exits 1.
- `kali fmt --check fixtures/formatted.ts` → exits 0.
- Formatter idempotence: `fmt(fmt(x)) == fmt(x)` for all fixture files.
- `kali lint fixtures/with-issues.ts` → exits 1, reports expected lint codes.
- `kali lint --fix fixtures/with-fixes.ts` → auto-fixes `prefer-const`, `no-var`, `eqeqeq`;
  remaining non-fixable issues still reported.
- Project-discovery test: running `fmt`/`lint`/`check` without explicit files discovers the
  correct file set.

## Out of Scope

- Custom lint rule plugins (later compatibility).
- Incremental formatting (format only changed files) — Phase 3 target.
- `kali publish` or package publishing workflow (not in spec).

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
