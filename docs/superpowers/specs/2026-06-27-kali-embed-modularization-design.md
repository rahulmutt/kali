# kali_embed modularization — design (18th in series)

Date: 2026-06-27
Status: approved
Crate: `kali_embed` (18th crate in the kali workspace modularization series; kali_error was 17th)

## Goal & invariant

Pure code-motion. Decompose the single monolith `src/lib.rs` (500 lines) into a thin facade plus four per-concern modules with **zero behavior change** and a **byte-identical public API**. External consumers MUST compile unedited.

Allowed changes only: `mod` declarations, `use` wiring, and `pub(crate)` visibility widening on items that become cross-module. Item bodies are moved **verbatim**. Do **not** run `cargo fmt` (verbatim moves plus the mandated `pub(crate)` prefix push some signatures over 100 columns and leave stray blank lines; the repo's `cargo fmt --all --check` gate is already red on baseline, so these are not regressions).

## Baseline (branch base)

`cargo test -p kali_embed`: record exact branch-base HEAD and test counts in the SDD ledger before starting.

## Current shape

- `src/lib.rs` (500 lines): all public types, the `KaliCompiler` entry point, `EmbeddingCtx` predicate-registration system, artifact types, `CompileError`, and private helper functions — all in one file.
- `src/tests.rs` (570 lines): co-located, declared in `lib.rs` via `#[path = "tests.rs"]`, uses `use super::*`.

## Target layout

### `lib.rs` → thin facade + 4 modules

| Module | Contents |
|---|---|
| `compiler` | `KaliCompiler` (new, default, compile_file, compile_lib, compile_lib_source, normalized_runtime_profiles) + `CompilerConfig` (+ `Default`) + `temporary_source_path` + `sanitize_module_name` (private helpers) |
| `context` | `EmbeddingCtx` (new, with_predicate_registration_enabled, predicate_registration_enabled, register_sandbox_predicate, check_operation_with_policy, build_library) + `PredicateDecision` (allow, deny, From\<bool\>) + `OperationContext` (from_operation) + `RegisteredPredicate` (pub(crate)) + `predicate_violation` (private helper) |
| `artifact` | `CompiledArtifact` (wasm_bytes, metadata) + `LibArtifact` (wasm_bytes, wit, metadata) |
| `error` | `CompileError` (diagnostics, Display, Error, From\<Vec\<Diagnostic\>\>) |

### `lib.rs` facade (~25 lines)

```rust
//! Embedding interfaces for Kali. ... (module doc preserved verbatim)

pub mod artifact;
pub mod compiler;
pub mod context;
mod error;

pub use artifact::{CompiledArtifact, LibArtifact};
pub use compiler::{CompilerConfig, KaliCompiler};
pub use context::{EmbeddingCtx, OperationContext, PredicateDecision};
pub use error::CompileError;

pub use kali_sandbox::{
    HostOperation, HostPredicate, PolicyPredicateContext, PolicyPredicateRegistry, SandboxPolicy,
};
pub use kali_cli::build::{ArtifactMetadata, LibraryExport};

#[cfg(test)]
use std::path::Path;
#[cfg(test)]
use kali_error::_error_codes::{e4, e5};
#[cfg(test)]
use kali_sandbox::{AccessRule, EffectsPolicy, ...};  // test stubs as needed

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
```

## Cross-module dependencies (within crate)

```
compiler → kali_cli::build, kali_error
context  → compiler (owns KaliCompiler), kali_sandbox, kali_error
artifact → kali_cli::build::ArtifactMetadata (type used in field)
error    → kali_error::Diagnostic
```

`context.rs` is the only module that depends on another module within the crate (it owns a `KaliCompiler` field and calls `compile_lib_source` in `build_library`).

## Test import fix

`tests.rs` uses `use super::*` and depends on symbols that are currently in scope via `lib.rs`'s `use` imports. After extraction, `use` statements that only existed to serve moved code will be removed from `lib.rs`. Missing symbols that `tests.rs` needs are added back as `#[cfg(test)] use ...;` stubs in `lib.rs` — import-only, no public-surface change, `tests.rs` stays verbatim.

Known candidates from series precedent: `Path`, `e4`, `e5`, and potential sandbox types used in test policies (`SandboxPolicy`, `AccessRule`, `EffectsPolicy`, `FileSystemPolicy`, `NetworkPolicy`, `ProcessPolicy`, `TimerPolicy`, `ResourceLimits`).

## Build verification

- `cargo build -p kali_embed` — green, 0 warnings
- `cargo test -p kali_embed` — green
- `cargo build` — workspace compiles (no consumer breakage in `kali_cli`, `kali_runtime`, or any other crate)
- `cargo clippy -p kali_embed` — 0 new warnings