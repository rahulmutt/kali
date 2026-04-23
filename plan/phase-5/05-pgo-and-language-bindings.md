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

- The public C-ABI helper now also emits a deterministic stem-specific binding-package manifest
  sidecar for higher-level language bindings, and the `--component` embedding flow now emits the
  same bundle index so the generated layout stays aligned across the exported-library wrappers.
  The manifest generator now also normalizes glue-path order and removes duplicate entries so the
  generated sidecar stays stable even when callers pass repeated helper paths. The maintained
  Python wrapper can auto-discover that generated bundle layout directly while preserving the same
  host-ABI compatibility check, and both the Python and Node helpers now expose public
  bundle-root manifest loaders plus companion `cabi-metadata` root-discovery helpers so higher-level
  binding code can reuse the same discovery rules without duplicating them. The maintained Rust helper now also exposes `load_binding_package_manifest_summary*`
  convenience loaders so callers can load and project the manifest in one deterministic step.
  The manifest-discovery regressions now also pin the deterministic ambiguity error when multiple
  stem-specific manifests are present, while still allowing explicit manifest-name selection for
  callers that want to target one bundle directly. The maintained Node ESM helper now mirrors the
  same manifest/metadata discovery rules over the stable C ABI, and now also exposes a small
  `KaliCAPI` wrapper that binds generated exports onto an existing library object, so the later
  binding lane has more than one non-Rust smoke path to prove the packaging contract. The Python
  and Node manifest parsers now also surface the optional `maxSpecializations` provenance field on
  generated binding-package manifests, and the higher-level binding wrappers now carry that
  provenance through to their public instances so callers can inspect the same deterministic
  specialization budget that the CLI emits. The binding-package manifest sidecar now also carries
  the normalized runtime provenance tuple (`runtimeProfiles`, `hostContract`, and
  `runtimeBackend`) alongside the specialization cap, keeping the later binding workflow aligned
  with the same build context that produced the exported artifacts. The build-metadata regression
  suite now also pins serialized runtime provenance on the artifact sidecars (`runtimeProfiles`,
  `maxSpecializations`, `hostContract`, and `runtimeBackend`), keeping the emitted JSON contract
  explicit across the build kinds that feed the later binding and PGO lanes. The C-ABI JSON-envelope
  regression now also exercises the `build --capi --output json` path and checks the generated
  binding-package manifest through that machine-readable contract, keeping the later language-
  binding workflow covered in both human and JSON output modes. The root README now calls out the
  generated sidecar flow alongside the maintained Python helper docs, so the package/distribution
  shape stays explicit for the later binding workflow instead of limiting the stage to header-only
  glue. The Python binding now also has explicit packaging metadata (`bindings/python/pyproject.toml`
  plus `README.md`) and regression coverage for both the generic package scaffold and the
  stem-specific generated bundle, and the C-ABI smoke coverage now pins the manifest file alongside
  the generated header and metadata outputs.
- The maintained Python and Node C-ABI metadata helpers now also project the optional provenance tuple into deterministic summary loaders, so callers can inspect the same `runtimeProfiles` / `maxSpecializations` / `hostContract` / `runtimeBackend` contract without reimplementing the JSON normalization rules.
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
- Added a dedicated Python `unittest` smoke harness under `bindings/python/tests/` and wired it
  into the Rust workspace test suite, so the maintained non-Rust binding now has a first-class
  end-to-end smoke check instead of only ad hoc inline scripts.
- Added deterministic optimization-report helpers in `kali_optimize` so callers can distinguish
  attached profile data from hot-function inlining usage, and added regression coverage for the
  no-profile, cold-profile, and hot-profile cases.
- Extended those optimization reports to surface hot branch and hot layout profile keys alongside
  hot function keys, keeping the later PGO lane explicit about which profile families are present
  before deeper branch/layout decisioning lands.
- Wired `--profile` through the CLI build path with version-checked loading and a
  profile-fingerprint cache key, so profile-guided builds now stay deterministic across repeated
  invocations as well as inside the optimizer. The CLI now exposes that PGO input as an explicit
  build-only opt-in rather than a hidden implementation detail.
- Added CLI smoke coverage for `kali build --profile` so repeated profile-guided builds stay
  byte-stable across invocations and unsupported profile-data versions fail through the command
  path as well as the lower-level loader.
- Added JSON-envelope regression coverage for successful `build --profile` runs and unsupported
  version mismatches so the machine-readable PGO path now stays deterministic across repeated
  invocations and still rejects bad profile data through the same command contract.
- Added a representative PGO benchmark regression that compares the release baseline against the
  profile-guided build on a hot-function workload while still asserting the hot-call-site reduction,
  so the stage now records a concrete gain-oriented test in addition to the existing determinism
  and version-gate coverage.
- Hot branch and layout profile hints now also unlock the release-mode algebraic-identity
  simplifier, so profile data influences a second optimization decision instead of only widening
  the hot-function inlining budget and report metadata.
- Synced the owning CLI and maturity docs so the build-only `--profile` PGO input is now
  documented as an explicit opt-in rather than a hidden implementation detail.
- Added CLI smoke coverage for `kali build --component --output json --out-dir ...` so the
  component flow now pins the stem-specific binding-package sidecar in an explicit distribution
  directory as well as the source-adjacent default layout.
- Split the maintained Node binding into shared core logic plus explicit ESM and CommonJS entrypoints,
  so the stable C ABI helper is now consumable through either module system while preserving the same
  deterministic manifest and metadata contract. Added a direct CommonJS entrypoint regression as
  well, so the package-root require path and the explicit `kali_capi.cjs` file both stay covered
  by the same maintained binding smoke lane.
- Added Rust-side binding-package manifest discovery/load helpers in `kali_capi`, mirroring the
  deterministic root-manifest and stem-specific discovery rules already exercised by the Python and
  Node helpers so future embedding and packaging code can reuse one shared contract from Rust as well.
- The Rust helper now also exposes the same explicit manifest-name load path as the Python and
  Node helpers, so callers can opt into stem-specific bundle discovery without reimplementing the
  root/manifest split.
- Added Rust-side C ABI metadata load/summary helpers in `kali_capi`, and the helper surface now
  also discovers `*.capi.meta.json` sidecars from a bundle root before loading or summarizing them,
  so the maintained Rust binding workflow can reuse the same deterministic discovery rules instead
  of hard-coding file paths at each call site.
- The binding-package summary helpers in Rust, Python, and Node now normalize their runtime
  provenance tuple and generated glue list on projection as well as on load, so callers get the
  same deterministic summary even when they hand the helper an already-materialized but
  unnormalized manifest object.
- The maintained Python and Node binding helpers now also surface the normalized runtime
  provenance tuple (`runtime_profiles`, `host_contract`, and `runtime_backend`) alongside
  `max_specializations`, so higher-level callers can inspect the exact build context that produced
  a binding package without reading the manifest JSON by hand.
- The Rust `kali_capi` manifest loader now canonicalizes `runtimeProfiles` and `artifacts.glue` on
  load as well, so the maintained helper surfaces stay aligned on the same deterministic string-list
  normalization instead of relying solely on the generator side to produce sorted unique arrays.
- The Rust binding-package manifest parser now also validates the optional `maxSpecializations`
  provenance field, matching the Python and Node helper contracts so higher-level callers get the
  same integer validation regardless of which maintained binding path reads the manifest.
- Added deterministic binding-package summary helpers across the Rust, Node, and Python binding
  shims so higher-level tooling can project the normalized provenance tuple and artifact layout
  through one stable convenience shape instead of reassembling it ad hoc at each call site.
- CLI smoke coverage now also proves that the `--max-specializations` override propagates through
  the `build --capi` and `build --component` artifact sidecars, keeping the later binding layout
  provenance aligned with the same specialization budget that the other build modes already
  expose.

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

Stage 5.5 is complete.

Any further widening of PGO or language-binding coverage belongs in the owning spec chapters and maturity matrix, not by reopening this closed stage checklist.
