# Stage 2.3 — Public Embedding Surface

**Phase:** 2 — Ownership, Effects & Public Embedding  
**Spec refs:** [`specs/13-embedding.md`](../../specs/13-embedding.md), [`specs/08-wasm-codegen.md`](../../specs/08-wasm-codegen.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [2.1 — MIR & Ownership Analysis](01-mir-and-ownership.md) (stable MIR-backed export surfaces needed for WIT generation); can proceed in parallel with 2.2

## Goal

Open the stable **public embedding surface**: promote `kali build --lib` from the Phase-1 unstable
base library artifact to the stable **WIT-first** `--lib` contract; add `--capi` (C ABI) and
`--component` (Component Model) as explicit projections over that same export surface; and
stabilise the public Rust embedding API in `kali_embed`.

## Workable Milestone

- `kali build --lib <file>` now emits a WIT sidecar alongside the WASM artifact; the contract
  is stable across Kali versions (not just exact-version consumers).
- `kali build --capi <file>` emits a C-callable shared library artifact + C header.
- `kali build --component <file>` emits a WebAssembly Component Model component.
- The public Rust embedding API in `kali_embed` is stable and documented.
- Phase-1 gating tests for these surfaces are updated to positive coverage.

## Progress

- 2026-04-23: `kali build --capi` now threads the same deterministic provenance tuple (`runtimeProfiles`, `maxSpecializations`, `hostContract`, and `runtimeBackend`) into the emitted `cabi-metadata` sidecar that the later binding workflows already consume, so the C-ABI metadata now mirrors the broader embedding-side provenance contract alongside the existing binding-package manifest coverage.
- 2026-04-23: the shared `kali_capi` binding-package manifest generator now accepts explicit host/runtime provenance labels instead of hard-coding them internally, and the CLI build paths now pass through the same `kali-hosted` / `wasmtime` contract labels that the metadata sidecars already record. That keeps the higher-level binding manifest aligned with the build-context provenance tuple without relying on a second ad hoc helper path.
- 2026-04-23: binding-package manifests emitted by `kali build --capi` and `kali build --component` now carry deterministic `maxSpecializations` provenance, and the new binding-package schema document keeps that sidecar shape explicit for downstream tooling.
- 2026-04-18: `kali_embed` now also accepts raw source text through a stable
  `compile_lib_source` convenience entry point, so the embedding surface can
  produce a library artifact + WIT sidecar without requiring callers to manage
  their own temporary materialization path.
- 2026-04-12: `kali_embed` now exposes a stable `KaliCompiler` API with
  `compile_file` / `compile_lib` entry points, deterministic artifact metadata,
  and a library-side WIT sidecar generated from the statically known export surface.
- 2026-04-12: `kali build --capi` now emits a deterministic C ABI artifact and
  C header, and `kali build --component` now emits a valid component artifact;
  both flows are covered by positive CLI smoke tests.

## Tasks

### 1. Stable WIT-first `--lib` contract

Extend `kali build --lib` to emit a WIT interface description sidecar:

```
<basename>.lib.wasm          — WASM core module (unchanged from Phase 1)
<basename>.lib.wit            — WIT interface description
<basename>.lib.meta.json     — schema-v1 artifact metadata (updated with WIT path)
```

The WIT file is generated from the **statically known export surface** of the entrypoint. Each
exported TypeScript function is mapped to a WIT function with the appropriate value types. The WIT
types supported in Phase 2:

- Primitives: `bool`, `u8`–`u64`, `s8`–`s64`, `f32`, `f64`, `char`, `string`.
- Structured: `record`, `tuple`, `list`, `option`, `result`.
- TypeScript `number` maps to `f64` by default; integer narrowing annotations (`@wit u32`, etc.)
  can refine this.

Cross-version ABI stability: with the WIT sidecar, host programs that consume the library no
longer need to pin to the exact Kali version used to compile it. The WIT contract governs
compatibility.

### 2. `kali build --capi <file>`

Emit a C-callable artifact: a WASM module whose exports follow a C ABI convention, plus a
generated C header file.

```
<basename>.capi.wasm         — WASM module with C-ABI exports
<basename>.h                  — C header (extern declarations)
<basename>.capi.meta.json    — schema-v1 artifact metadata
```

The C ABI uses the standard WASM-compatible C calling conventions:
- Integers pass as `i32` / `i64`.
- Floats pass as `f32` / `f64`.
- Strings pass as `(ptr: i32, len: i32)` pairs.
- Structs and arrays are passed via linear memory pointers.

### 3. `kali build --component <file>`

Emit a [WebAssembly Component Model](https://component-model.bytecodealliance.org/) component,
composed from the core WASM module and the WIT interface:

```
<basename>.component.wasm    — WIT component bundle
<basename>.component.meta.json
```

The component is produced by packaging the core WASM module as a valid component with
pure-Rust tooling and keeping the WIT sidecar aligned with the same exported surface.
`--component` is an explicit packaging flow over that same WIT export surface, not a separate
embedding semantic.

`kali build --component` requires that the entrypoint has a statically known export surface
(same precondition as `--lib`); emit `E5509` otherwise.

### 4. Stable public Rust embedding API (`kali_embed`)

Stabilise the `kali_embed` crate's public API surface:

```rust
pub struct KaliCompiler { ... }
impl KaliCompiler {
    pub fn new(config: CompilerConfig) -> Self;
    pub fn compile_file(&self, path: &Path) -> Result<CompiledArtifact, CompileError>;
    pub fn compile_lib(&self, path: &Path) -> Result<LibArtifact, CompileError>;
}

pub struct CompiledArtifact { ... }
impl CompiledArtifact {
    pub fn wasm_bytes(&self) -> &[u8];
    pub fn metadata(&self) -> &ArtifactMetadata;
}

pub struct LibArtifact { ... }
impl LibArtifact {
    pub fn wasm_bytes(&self) -> &[u8];
    pub fn wit(&self) -> &str;
    pub fn metadata(&self) -> &ArtifactMetadata;
}
```

Publish `kali_embed` as a public crate on crates.io with a stable semver version.

### 5. Tests

- `kali build --lib fixtures/math.ts` → emits `math.lib.wasm` + `math.lib.wit`; WIT content
  matches golden snapshot.
- `kali build --capi fixtures/math.ts` → emits `math.capi.wasm` + `math.h`; C header compiles
  with `clang`.
- `kali build --component fixtures/math.ts` → emits `math.component.wasm`; validate with
  `wasm-tools component validate`.
- Rust embedding API integration test: use `kali_embed` in a test binary to compile and execute
  a fixture program.
- Phase-1 gating tests for `--capi` and `--component` are updated to positive coverage.

## Out of Scope

- `kali package-audit` (Later compatibility).
- Cross-language component composition beyond the WIT sidecar (later compatibility).
- Node.js compatibility (Phase 3 target).

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
