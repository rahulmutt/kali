# Stage 1.11 — Build Artifacts

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/08-wasm-codegen.md`](../../specs/08-wasm-codegen.md), [`specs/13-embedding.md`](../../specs/13-embedding.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.9 — Sandbox & Policy](09-sandbox-and-policy.md), [1.10 — Package Management](10-package-management.md)

## Goal

Complete the Phase-1 artifact surface: the default executable build (`kali build <file>`), the
browser bundle (`kali build --bundle <file>`), and the base library artifact
(`kali build --lib <file>`) for exact-version consumers. Together with `--sandbox` variants these
complete the **Phase-1 browser-targeted command set** and the **Phase-1 static policy-validation surface**.

## Workable Milestone

- `kali build <file>` produces a self-contained executable `.wasm` artifact.
- `kali build --bundle <file>` produces a browser-loadable JS+WASM bundle.
- `kali build --lib <file>` produces a base library WASM artifact with deterministic export
  metadata for exact-version consumers (when the export surface is statically known).
- All three artifact modes accept `--sandbox <policy>` for static policy validation.

## Progress

- The executable build path now writes deterministic `kali:metadata` custom sections in the emitted `.wasm` artifact.
- `kali build --lib` now emits `.lib.wasm` plus a sidecar `.lib.meta.json` with deterministic export inventory data for statically known library surfaces.
- `kali build --bundle` now emits a browser bundle directory containing `.wasm`, `.js`, and `.meta.json` outputs, with deterministic metadata and JS glue that uses browser-native instantiation APIs.
- CLI smoke coverage now exercises both `--lib` and `--bundle` output paths in addition to the existing executable/sandbox path.
- The CLI now reconciles the effective API surface for build artifacts: `--api browser` and inherited browser config enable the bundle path, while contradictory browser/non-browser combinations are rejected with the canonical `E5008` shape error and `--api node` remains phase-gated with `E5006`.
- `kali check` now honors the same browser API surface selection, including inherited browser config, while still rejecting node-targeted checking with `E5006`.
- Browser bundle smoke coverage now includes the inherited-config browser path, and the command surface rejects non-browser bundle requests before artifact generation starts.

## Tasks

### 1. Executable artifact (`kali build <file>`)

This was partially wired in Stage 1.7. Complete it:

- The output is a `.wasm` file that, when loaded by a wasmtime-compatible host with the
  **Default standalone context** import table, runs to completion.
- The artifact includes a `kali:metadata` WASM custom section with the schema-v1 artifact
  metadata JSON (see `specs/18-schemas.md`):

```json
{
  "schemaVersion": 1,
  "artifactKind": "executable",
  "entrypoint": "<file>",
  "buildMode": "fast",
  "apiSurface": "deno",
  "kaliVersion": "0.1.0",
  "sourceHash": "sha256-..."
}
```

- When `--sandbox <policy>` is passed: validate the policy file and embed it as `kali:policy`.
- When `--release` or `--release-advanced` is passed: use the appropriate build mode (optimisation
  content behind these modes grows in Phase 3; in Phase 1 they are equivalent to `fast` plus the
  stub optimisation pass).

```
kali build <file>
kali build --fast <file>
kali build --release <file>
kali build --release-advanced <file>
kali build --sandbox <policy> <file>
kali build --out-dir <dir> <file>
```

### 2. Browser bundle (`kali build --bundle <file>`)

The browser bundle is the **Phase-1 browser-targeted command set**'s build half.

Output structure:

```
<basename>/
├── <basename>.wasm       — compiled WASM module
├── <basename>.js         — JS loader / glue script (ESM)
└── <basename>.meta.json  — schema-v1 artifact metadata
```

The JS glue script:

- Uses `WebAssembly.instantiateStreaming` (with a `fetch` fallback for older browsers).
- Provides browser-compatible shims for the host imports the WASM module expects (`console`,
  `fetch`, `TextEncoder`, `URL`, etc.) using the browser's native implementations where available.
- Exports the WASM module's public functions as named ESM exports so the bundle can be consumed
  as an ES module.
- Does **not** include a runtime event loop (the browser provides one).

`--api browser` is the canonical API surface for bundle builds. The combination
`kali build --bundle --api node` is an `E5008` command-shape contradiction and must be rejected.

```
kali build --bundle <file>
kali build --bundle --sandbox <policy> <file>
kali build --bundle --api browser <file>        # explicit; browser is implied by --bundle
kali build --bundle --out-dir <dir> <file>
```

**Inherited-config equivalence:** when `kali.json` sets `compilerOptions.apiSurface = "browser"`,
the same build/check commands behave as if `--api browser` / `--bundle` were passed.

### 3. Base library artifact (`kali build --lib <file>`)

The Phase-1 **base library artifact** for **exact-version consumers**.

Preconditions (emit `E5009` if not satisfied):

- The entrypoint must have a **statically known export surface** — all exported names and their
  types must be determinable at compile time without dynamic re-export patterns.
- The entrypoint may not use `export * from` with a variable specifier.

Output:

```
<basename>.lib.wasm     — WASM module with exported functions
<basename>.lib.meta.json — schema-v1 artifact metadata with export inventory
```

The `.lib.meta.json` contains:

```json
{
  "schemaVersion": 1,
  "artifactKind": "lib",
  "exports": [
    { "name": "add", "signature": "(a: number, b: number) => number" },
    { "name": "greet", "signature": "(name: string) => string" }
  ],
  "buildMode": "fast",
  "kaliVersion": "0.1.0",
  "sourceHash": "sha256-..."
}
```

**Phase-1 limitations for `--lib`:**

- No stable public Rust embedding API yet (Phase 2 target; `kali_embed` is internal/pre-stable).
- No WIT sidecar (Phase 2 target; plain `--lib` with WIT is the Phase-2 public embedding surface).
- No C ABI or Component Model artifact (Phase 2 target).
- The "exact-version consumers" qualifier means the host loading this artifact must use the same
  Kali version that produced it; no cross-version ABI stability is promised in Phase 1.

```
kali build --lib <file>
kali build --lib --sandbox <policy> <file>
kali build --lib --out-dir <dir> <file>
```

`kali build --lib --api browser` is an `E5008` command-shape contradiction.

### 4. Error codes for command-shape violations

| Code | Meaning |
|---|---|
| `E5001` | Missing required primary source input |
| `E5002` | Too many primary source inputs |
| `E5003` | Unknown flag or flag combination |
| `E5004` | Output directory does not exist |
| `E5005` | Policy file not found (when passed with `--sandbox`) |
| `E5006` | `--sandbox` on an invalid command/context combination |
| `E5007` | Declaration-only file as build entrypoint |
| `E5008` | Contradictory flag combination |
| `E5009` | Export surface not statically known (for `--lib`) |

### 5. Artifact metadata schema

Implement the schema-v1 artifact metadata shape in `kali_codegen` and `specs/18-schemas.md`.
The metadata is embedded as a WASM custom section named `kali:metadata` for `.wasm` artifacts
and as a sidecar `.meta.json` for multi-file bundles.

Ensure the metadata is deterministic: given the same source + flags + package lock, the metadata
(and the artifact content) must be byte-for-byte identical across builds.

### 6. Integration tests

- `kali build fixtures/hello.ts` → produces `hello.wasm`; validate binary; run with wasmtime
  directly to confirm execution.
- `kali build --bundle fixtures/app.ts` → produces `app/` directory; validate JS and WASM;
  run the bundle in a headless browser harness (e.g. playwright/deno `browser` API) and assert
  expected output.
- `kali build --lib fixtures/math.ts` → produces `math.lib.wasm` and `math.lib.meta.json`;
  validate export inventory.
- `kali build --bundle --api node fixtures/app.ts` → exits 1 with `E5008`.
- `kali build --lib --api browser fixtures/app.ts` → exits 1 with `E5008`.
- Repeated builds of identical inputs produce byte-identical artifacts.

## Out of Scope

- Stable public WIT sidecar on `--lib` (Phase 2 target).
- `--capi` artifact mode (Phase 2 target).
- `--component` artifact mode (Phase 2 target).
- `--api node` build path (Phase 3 target).
- Incremental builds (Phase 3 target).

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
