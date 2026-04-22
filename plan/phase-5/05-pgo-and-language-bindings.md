# Stage 5.5 — PGO & Language Bindings

**Phase:** 5 — Later Compatibility & Platform Expansion  
**Spec refs:** [`specs/07-specialization.md`](../../specs/07-specialization.md), [`specs/13-embedding.md`](../../specs/13-embedding.md), [`specs/16-testing.md`](../../specs/16-testing.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [3.1 — Optimization & Specialization](../phase-3/01-optimization-and-specialization.md), [2.3 — Public Embedding Surface](../phase-2/03-public-embedding-surface.md), and whichever Phase-5 runtime/host stages the target binding or profile data depends on

## Goal

Finish the spec surfaces that are intentionally beyond the early optimization and embedding core:

- profile-guided optimization as an explicit later toolchain mode
- expansion from the stable public C ABI / WIT contract into higher-level language bindings and
  distribution workflows

## Workable Milestone

- Kali can collect profile data and feed it back into later optimization decisions without changing
  the stable build-mode vocabulary.
- At least one non-Rust language binding is generated or maintained over the stable public ABI
  contract.
- PGO artifacts, reports, and binding-generation outputs are deterministic and schema-backed where
  they become machine-visible.

## Progress

- Added the first deterministic profile-data format to `kali_optimize`, including stable versioning,
  normalization, aggregation, and JSON round-trip coverage so the later PGO lane has one canonical
  collection shape to build on.
- Wired the optimizer to carry normalized profile data as an explicit optional input, keeping the
  future profile-guided decision points isolated from the existing release/release-advanced
  vocabulary.
- Added the first profile-guided optimization hook by widening the inlining budget for hot
  functions recorded in profile data, so the later PGO lane now influences a concrete optimization
  decision instead of remaining collection-only plumbing.
- Added a maintained Python ctypes binding helper over the generated Kali C ABI header, giving the
  later language-binding lane one concrete non-Rust wrapper surface to preserve alongside the
  stable ABI metadata and export layout.
- Added deterministic host ABI metadata parsing and compatibility checks to the Python ctypes
  helper, so the binding path now validates the generated `cabi-metadata` version window before
  exposing exports.
- Added deterministic optimization-report helpers in `kali_optimize` so callers can distinguish
  attached profile data from hot-function inlining usage, and added regression coverage for the
  no-profile, cold-profile, and hot-profile cases.
- Wired `--profile` through the CLI build path with version-checked loading and a
  profile-fingerprint cache key, so profile-guided builds now stay deterministic across repeated
  invocations as well as inside the optimizer.

## Tasks

### 1. Profile data collection

Design and implement the PGO data path:

- `--profile` or equivalent profiling workflow
- stable profile-data format and versioning rules
- capture of hot paths relevant to inlining, specialization, and branch/layout decisions
- deterministic merging/normalization rules for repeated runs

### 2. PGO-guided optimization pipeline

Integrate profile data into the optimizer without changing the user-facing build-mode vocabulary:

- `fast`, `release`, and `release-advanced` remain the stable mode names
- PGO becomes an additive workflow, not a fourth hidden replacement mode
- optimization reports and diagnostics stay explicit about when profile data was or was not used

### 3. Binding-generation and packaging workflow

Build on the stable public embedding surface to support higher-level language bindings:

- bindings over `kali_capi` and/or WIT for languages such as Python, Go, C#, Java, Ruby, Zig
- version and compatibility checks against the host ABI metadata
- packaging/distribution rules for headers, metadata, and generated glue

### 4. Tooling and documentation alignment

Ensure binding and PGO tooling do not fork the core vocabulary:

- reuse the canonical build mode / API surface / runtime profile / compat feature names
- preserve deterministic JSON/schema contracts where outputs become machine-readable
- keep README/examples/support wording aligned with the maturity matrix

### 5. Tests and benchmarks

- benchmark suite proving PGO gains on representative workloads
- reproducibility tests for profile-data ingestion and optimized outputs
- ABI/version compatibility tests for generated language bindings
- smoke tests for at least one maintained non-Rust binding

## Out of Scope

- replacing the stable build-mode vocabulary with a second optimization naming scheme
- ad hoc language bindings that bypass the public ABI/WIT contract
- widening public maturity claims without the corresponding evidence lane

## Status

In progress.
