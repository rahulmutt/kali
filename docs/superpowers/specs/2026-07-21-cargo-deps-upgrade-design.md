# Cargo Dependency Upgrade + CI Refresh — Design

**Date:** 2026-07-21
**Branch:** `deps-upgrade-2026-07` (from main `df1c919a9`)
**Status:** Approved

## Goal

Bring every external dependency in the workspace `Cargo.toml` to its latest
stable version, refresh `Cargo.lock`, fix all resulting build / clippy / fmt
errors, and bump GitHub Actions in `ci.yml` and `release.yml` to current
majors. No behavior changes to the compiler: byte-for-byte acceptance fixtures
must stay byte-for-byte.

## Current vs target versions (measured 2026-07-21 against crates.io)

| Dep | Pinned | Latest | Kind of jump |
|---|---|---|---|
| wasmtime | 24 | 47.0.1 | 23 majors; kali_runtime only |
| wasm-encoder / wasmparser / wit-component | 0.240 | 0.254 | breaking 0.x set; kali_codegen + kali_cli |
| thiserror | 1.0 | 2.0.19 | mechanical major |
| sha1 / sha2 | 0.10 | 0.11 | RustCrypto digest-ecosystem major |
| hmac | 0.12 | 0.13 | moves with digest 0.11 |
| tungstenite | 0.24 | 0.30 | 6 majors |
| reqwest | 0.12 | 0.13 | 1 major |
| getrandom | 0.3 | 0.4 | 1 major |
| clap, indexmap, serde, log, flate2, … | — | — | semver-compatible; `cargo update` |

Measured usage surface (informs risk):

- wasmtime: direct dep of `kali_runtime` only. Usage is
  `wasmtime::Result` (96×), `wasmtime::Error::msg` (77×), plus
  Engine/Store/Linker/fuel/StoreLimits/Trap/ValType basics. Expected churn is
  Config / error-type / renames, not architecture.
- wasm-encoder: one stable import set in codegen (sections, `Instruction`,
  `Function`, `MemArg`, `BlockType`, `ValType`) + component/custom-section
  types in kali_cli. wasmparser: `Validator`, `ExternalKind`, `TypeRef`.

## Stages (one commit each, risk-ordered)

1. **Lockfile + compatible bumps** — `cargo update`; raise semver-compatible
   requirement strings (clap 4.6, indexmap 2.14, etc.) so `Cargo.toml`
   reflects reality.
2. **Easy majors** — thiserror 1→2, reqwest 0.12→0.13, getrandom 0.3→0.4,
   tungstenite 0.24→0.30.
3. **RustCrypto set** — sha1/sha2 0.10→0.11 + hmac 0.12→0.13 together
   (shared `digest` ecosystem; mixing halves does not compile).
4. **wasm-tools set** — wasm-encoder / wasmparser / wit-component
   0.240→0.254 together.
5. **wasmtime 24→47** — kali_runtime.
6. **CI/actions** — checkout, setup-node, cache, upload/download-artifact,
   action-gh-release, slsa-generator to latest majors in both workflows.
   `dtolnay/rust-toolchain@stable` and the mise `rust = "1.97.1"` pin stay
   (already current stable).

## Verification (after every stage)

- `cargo build --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --all -- --check`
- Full `cargo test --workspace --no-fail-fast` enumeration, diffed against a
  main-worktree baseline run (poisoned-baseline lesson). **0 newly-red
  required**; baseline is currently 0-failed.
- Stage 5 additionally runs the determinism smoke lane
  (`bash scripts/check-determinism.sh`) since wasmtime executes the fixtures.

If any single stage proves disproportionately expensive, stop and report
rather than hack it green; staged commits allow deferring just that dep.

## Failure policy

Deprecation-driven API changes are fixed at the call site following the new
crate's idioms. No `#[allow]` suppressions to silence clippy unless the lint
is a known false positive; no loosening of `-D warnings`.

## Integration

One branch, one PR to main. Push, review via /code-review, merge after green
CI (per standing kali integration convention).
