# kali_embed Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose the 500-line `crates/kali_embed/src/lib.rs` into a thin facade plus four focused per-concern modules, with zero behavior change and a byte-identical public API.

**Architecture:** Pure code-motion. `lib.rs` splits into 4 sibling modules (`compiler`, `context`, `artifact`, `error`); `lib.rs` ends as declarations + re-exports + test wiring. The existing test suite (`tests.rs`, 570 lines) is the regression oracle and must stay green after every task. No visibility widening pass needed — all cross-module types are already `pub`.

**Tech Stack:** Rust (edition 2021), Cargo workspace. Dependencies: `kali_cli` (`build::{self, BuildMode}`, `ApiSurface`, `ArtifactMetadata`, `LibraryExport`), `kali_error` (`Diagnostic`, `_error_codes::{e4, e5, e8}`), `kali_sandbox` (`SandboxPolicy`, `HostOperation`, etc.).

## Global Constraints

- **Verbatim moves only.** Type/method/fn bodies are moved byte-identical (cut from the source file, paste into the new module). Do NOT retype, reformat, reorder, or "improve" any moved code. The only edits permitted are: `mod`/`use` wiring and re-export lines. No `pub(crate)` widening is needed in this crate.
- **Do NOT run `cargo fmt`.** The repo's `cargo fmt --all --check` gate is already red on baseline (10+ crates). Verbatim moves may push some lines >100 cols or leave stray blank lines — these are accepted cosmetic minors, not regressions. Running fmt would violate the verbatim mandate.
- **Every task ends green:** `cargo build -p kali_embed` with **0 warnings** and `cargo test -p kali_embed` showing all tests passed. Remove any `use` line that goes unused as code leaves the source file (the build will flag it); add any `use` a moved item now needs.
- **Public surface stays byte-identical.** Crate-root `pub`: `KaliCompiler`, `CompilerConfig`, `EmbeddingCtx`, `PredicateDecision`, `OperationContext`, `CompiledArtifact`, `LibArtifact`, `CompileError` plus all their existing `pub` methods; `pub use kali_sandbox::{HostOperation, HostPredicate, PolicyPredicateContext, PolicyPredicateRegistry, SandboxPolicy}`; `pub use kali_cli::build::{ArtifactMetadata, LibraryExport}`.
- **Commit message convention:** `refactor(kali_embed): <description> [refactor]`.
- **Integration:** work on branch `refactor/kali-embed-modularization` off `main`. Local-main ff-merge only — NEVER push to origin. (Branch is created in Task 1 Step 0; the final ff-merge is Task 6.)
- **Recurring gotcha — test import fix:** `tests.rs` uses `use super::*`. When private `use` imports are removed from `lib.rs` (they only served moved code), `tests.rs` loses access to those symbols. After the final extraction, add `#[cfg(test)] use` stubs to `lib.rs` for any symbols `tests.rs` needs. Known candidates: `std::path::PathBuf`, `kali_error::_error_codes::{e4, e5}`.

---

## File Structure (end state)

- `crates/kali_embed/src/lib.rs` — thin facade: crate doc, 4 `mod` decls, `pub use` re-exports, `#[cfg(test)]` stubs, test wiring.
- `crates/kali_embed/src/compiler.rs` — `KaliCompiler` (new, default, compile_file, compile_lib, compile_lib_source, normalized_runtime_profiles) + `CompilerConfig` (+ `Default`) + `temporary_source_path` + `sanitize_module_name`.
- `crates/kali_embed/src/context.rs` — `EmbeddingCtx` (new, with_predicate_registration_enabled, predicate_registration_enabled, register_sandbox_predicate, check_operation_with_policy, build_library) + `PredicateDecision` (allow, deny, From\<bool\>) + `OperationContext` (from_operation) + `RegisteredPredicate` (private) + `predicate_violation` (private).
- `crates/kali_embed/src/artifact.rs` — `CompiledArtifact` (wasm_bytes, metadata) + `LibArtifact` (wasm_bytes, wit, metadata).
- `crates/kali_embed/src/error.rs` — `CompileError` (diagnostics, Display, Error, From\<Vec\<Diagnostic\>\>).
- `crates/kali_embed/src/tests.rs` — untouched; stays declared in the `lib.rs` facade with `#[path = "tests.rs"]` and `use super::*`.

**Source line map — `lib.rs`** (current, for verbatim cut/paste):

| Item | Lines |
|---|---|
| crate doc `//!` | 1–5 |
| `use std::{…}` | 6–14 |
| `use kali_cli::{…}` | 16–19 |
| `use kali_error::{…}` | 20–23 |
| `pub use kali_sandbox::{…}` | 24–26 |
| `CompilerConfig` struct + `Default` | 29–47 |
| `PredicateDecision` + `impl` + `From<bool>` | 50–77 |
| `OperationContext` + `impl` | 81–104 |
| `KaliCompiler` struct | 107–110 |
| `impl Default for KaliCompiler` | 112–116 |
| `impl KaliCompiler { new }` | 118–123 |
| `impl KaliCompiler { compile_file }` | 124–153 |
| `impl KaliCompiler { compile_lib }` | 155–189 |
| `impl KaliCompiler { compile_lib_source }` | 191–255 |
| `impl KaliCompiler { normalized_runtime_profiles }` | 257–273 |
| `CompiledArtifact` struct + `impl` | 276–293 |
| `LibArtifact` struct + `impl` | 295–318 |
| `CompileError` struct + `impl` + `Display` + `Error` | 320–348 |
| `RegisteredPredicate` struct | 350–354 |
| `EmbeddingCtx` struct | 356–361 |
| `impl Default for EmbeddingCtx` | 363–367 |
| `impl EmbeddingCtx { new, with_…, predicate_registration_enabled }` | 369–386 |
| `impl EmbeddingCtx { register_sandbox_predicate }` | 387–409 |
| `impl EmbeddingCtx { check_operation_with_policy }` | 411–434 |
| `impl EmbeddingCtx { build_library }` | 436–443 |
| `pub use build::LibraryExport;` | 445 |
| `pub use kali_cli::build::ArtifactMetadata;` | 446 |
| `predicate_violation` fn | 448–470 |
| `temporary_source_path` fn | 472–479 |
| `sanitize_module_name` fn | 481–496 |
| test wiring | 498–500 |

---

### Task 1: Extract `error.rs`

**Files:**
- Create: `crates/kali_embed/src/error.rs`
- Modify: `crates/kali_embed/src/lib.rs`

**Interfaces:**
- Consumes: nothing (leaf module).
- Produces: `crate::error::CompileError`, re-exported at crate root.

- [ ] **Step 0: Create the work branch**

Confirm baseline green on `main`, then branch:

```bash
cargo test -p kali_embed 2>&1 | tail -3
git checkout -b refactor/kali-embed-modularization
```

Record the baseline test count and HEAD in `.superpowers/sdd/progress.md`.

- [ ] **Step 1: Create `error.rs` with the moved `CompileError`**

```rust
use kali_error::Diagnostic;

/// Compile error wrapper for embedding callers.
#[derive(Debug, Clone)]
pub struct CompileError {
    diagnostics: Vec<Diagnostic>,
}

impl CompileError {
    /// Access the underlying diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

impl From<Vec<Diagnostic>> for CompileError {
    fn from(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.diagnostics.first() {
            Some(diagnostic) => write!(f, "{diagnostic}"),
            None => f.write_str("embedding compile error"),
        }
    }
}

impl std::error::Error for CompileError {}
```

(Cut verbatim from lib.rs lines 320–348. The `use kali_error::Diagnostic;` is new — added because this module now needs it.)

- [ ] **Step 2: Remove `CompileError` from `lib.rs` and wire the module**

Delete from `lib.rs`: the entire `CompileError` block (lines 320–348). Add after the `pub use kali_sandbox::{…}` line:

```rust
mod error;

pub use error::CompileError;
```

- [ ] **Step 3: Verify build + tests**

```bash
cargo build -p kali_embed 2>&1 | tail -5 && cargo test -p kali_embed 2>&1 | tail -3
```

Expected: 0 warnings; all tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_embed/src/error.rs crates/kali_embed/src/lib.rs
git commit -m "refactor(kali_embed): extract error module [refactor]"
```

---

### Task 2: Extract `artifact.rs`

**Files:**
- Create: `crates/kali_embed/src/artifact.rs`
- Modify: `crates/kali_embed/src/lib.rs`

**Interfaces:**
- Consumes: nothing (leaf module; `ArtifactMetadata` is from `kali_cli`).
- Produces: `crate::artifact::{CompiledArtifact, LibArtifact}`, re-exported at crate root.

- [ ] **Step 1: Create `artifact.rs` with the moved artifact types**

```rust
use kali_cli::build::ArtifactMetadata;

/// Compiled standalone artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledArtifact {
    wasm_bytes: Vec<u8>,
    metadata: ArtifactMetadata,
}

impl CompiledArtifact {
    /// Get the compiled WASM bytes.
    pub fn wasm_bytes(&self) -> &[u8] {
        &self.wasm_bytes
    }

    /// Get the associated artifact metadata.
    pub fn metadata(&self) -> &ArtifactMetadata {
        &self.metadata
    }
}

/// Compiled library artifact with a WIT sidecar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibArtifact {
    wasm_bytes: Vec<u8>,
    wit: String,
    metadata: ArtifactMetadata,
}

impl LibArtifact {
    /// Get the compiled WASM bytes.
    pub fn wasm_bytes(&self) -> &[u8] {
        &self.wasm_bytes
    }

    /// Get the generated WIT interface description.
    pub fn wit(&self) -> &str {
        &self.wit
    }

    /// Get the associated artifact metadata.
    pub fn metadata(&self) -> &ArtifactMetadata {
        &self.metadata
    }
}
```

(Cut verbatim from lib.rs lines 276–318. The `use kali_cli::build::ArtifactMetadata;` is new — added because this module now needs it.)

- [ ] **Step 2: Remove artifact types from `lib.rs` and wire the module**

Delete from `lib.rs`: the `CompiledArtifact` block (276–293) and `LibArtifact` block (295–318). Add after the error module lines:

```rust
mod artifact;

pub use artifact::{CompiledArtifact, LibArtifact};
```

- [ ] **Step 3: Verify build + tests**

```bash
cargo build -p kali_embed 2>&1 | tail -5 && cargo test -p kali_embed 2>&1 | tail -3
```

Expected: 0 warnings; all tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_embed/src/artifact.rs crates/kali_embed/src/lib.rs
git commit -m "refactor(kali_embed): extract artifact module [refactor]"
```

---

### Task 3: Extract `compiler.rs`

**Files:**
- Create: `crates/kali_embed/src/compiler.rs`
- Modify: `crates/kali_embed/src/lib.rs`

**Interfaces:**
- Consumes: `crate::error::CompileError` (from Task 1), `kali_cli::build`, `kali_cli::ApiSurface`, `kali_error`.
- Produces: `crate::compiler::{KaliCompiler, CompilerConfig}`, re-exported at crate root.

- [ ] **Step 1: Create `compiler.rs` with the moved compiler items**

```rust
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use kali_cli::{
    build::{self, BuildMode},
    ApiSurface,
};
use kali_error::{
    _error_codes::{e5, e8},
    Diagnostic,
};

use crate::artifact::{CompiledArtifact, LibArtifact};
use crate::error::CompileError;

/// Compiler configuration for the embedding API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerConfig {
    /// Selected build mode.
    pub build_mode: BuildMode,
    /// Effective API surface used for analysis and artifact metadata.
    pub api_surface: ApiSurface,
    /// Requested runtime profiles.
    pub runtime_profiles: Vec<String>,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            build_mode: BuildMode::Fast,
            api_surface: ApiSurface::Deno,
            runtime_profiles: Vec::new(),
        }
    }
}

/// Stable embedding compiler entry point.
#[derive(Debug, Clone)]
pub struct KaliCompiler {
    config: CompilerConfig,
}

impl Default for KaliCompiler {
    fn default() -> Self {
        Self::new(CompilerConfig::default())
    }
}

impl KaliCompiler {
    /// Construct a compiler with the provided configuration.
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// Compile a source file into a standalone executable artifact.
    pub fn compile_file(&self, path: &Path) -> Result<CompiledArtifact, CompileError> {
        // <verbatim from lib.rs 125–153>
    }

    /// Compile a source file into a library artifact plus a deterministic WIT sidecar.
    pub fn compile_lib(&self, path: &Path) -> Result<LibArtifact, CompileError> {
        // <verbatim from lib.rs 156–189>
    }

    /// Compile a raw source string into a library artifact plus a deterministic WIT sidecar.
    pub fn compile_lib_source(
        &self,
        module_name: &str,
        source: &str,
    ) -> Result<LibArtifact, CompileError> {
        // <verbatim from lib.rs 192–255>
    }

    fn normalized_runtime_profiles(&self) -> Result<Vec<String>, CompileError> {
        // <verbatim from lib.rs 257–273>
    }
}

fn temporary_source_path(module_name: &str) -> PathBuf {
    // <verbatim from lib.rs 472–479>
}

fn sanitize_module_name(module_name: &str) -> String {
    // <verbatim from lib.rs 481–496>
}
```

**Critical detail:** The `compile_file`, `compile_lib`, `compile_lib_source`, and `normalized_runtime_profiles` method bodies are moved **verbatim** from lib.rs. The two private helpers `temporary_source_path` and `sanitize_module_name` are also moved verbatim.

The new additions in this module (not from lib.rs): `use crate::error::CompileError;` (was in-scope in lib.rs, now needs explicit import), and `use crate::artifact::{CompiledArtifact, LibArtifact};` — wait, these aren't needed because the methods return `Result<CompiledArtifact, CompileError>` and `Result<LibArtifact, CompileError>`. Since `CompiledArtifact` and `LibArtifact` are re-exported from `crate`, they're accessible as `crate::CompiledArtifact` and `crate::LibArtifact`. Actually, in Rust, items declared in submodules of the crate root can be accessed as `crate::item_name` only if they're re-exported from the crate root. Since we'll re-export them, `crate::CompiledArtifact` works. But actually the more idiomatic path is `crate::artifact::CompiledArtifact`. Let me use `crate::artifact::{CompiledArtifact, LibArtifact}` and `crate::error::CompileError`.

Wait, actually I need to check — in the verbatim code, `CompiledArtifact` is used without qualification (it was in the same file). When moved to `compiler.rs`, it needs to be imported. Same for `LibArtifact` and `CompileError`. But actually, `CompileError` is the only one that matters — `CompiledArtifact` and `LibArtifact` appear as return types in the method signatures, which are behind the `.map_err(CompileError::from)?` calls in the method bodies.

Let me check: does `compile_file` actually use `CompiledArtifact` by name? Let me look:

```rust
pub fn compile_file(&self, path: &Path) -> Result<CompiledArtifact, CompileError> {
    ...
    Ok(CompiledArtifact {
        wasm_bytes,
        metadata,
    })
}
```

Yes! `CompiledArtifact` is constructed by name. So it needs to be imported. Similarly `LibArtifact` is constructed in `compile_lib` and `compile_lib_source`.

So the imports needed in `compiler.rs`:
```rust
use crate::artifact::{CompiledArtifact, LibArtifact};
use crate::error::CompileError;
```

OK, let me fix the plan. I'll keep the task steps focused and correct.

- [ ] **Step 2: Remove compiler items from `lib.rs` and wire the module**

Delete from `lib.rs`:
- `CompilerConfig` struct + `Default` (lines 29–47)
- `KaliCompiler` struct + `impl Default` + `impl KaliCompiler` (lines 107–274)
- `temporary_source_path` (lines 472–479)
- `sanitize_module_name` (lines 481–496)

Remove now-unused imports from `lib.rs`:
- `use std::fs;` (only served compiler.rs)
- `use std::path::{Path, PathBuf};` (only served compiler.rs)
- `use std::sync::atomic::{AtomicU64, Ordering};` (only served compiler.rs)
- `use kali_cli::build::{self, BuildMode};` (only served compiler.rs)
- `use kali_cli::ApiSurface;` (only served compiler.rs)
- `use kali_error::_error_codes::{e4, e5, e8};` — `e8` only served compiler.rs; `e4` and `e5` still needed by context items in lib.rs. Change to `use kali_error::_error_codes::{e4, e5};`.
- `use kali_error::Diagnostic;` — still needed by remaining context items.

Add after the artifact module lines:

```rust
mod compiler;

pub use compiler::{CompilerConfig, KaliCompiler};
```

- [ ] **Step 3: Verify build + tests**

```bash
cargo build -p kali_embed 2>&1 | tail -5 && cargo test -p kali_embed 2>&1 | tail -3
```

Expected: 0 warnings; all tests pass.
If the build flags a missing `use` in compiler.rs (e.g., `CompiledArtifact`/`LibArtifact` not in scope), add the missing import and re-verify.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_embed/src/compiler.rs crates/kali_embed/src/lib.rs
git commit -m "refactor(kali_embed): extract compiler module [refactor]"
```

---

### Task 4: Extract `context.rs`

**Files:**
- Create: `crates/kali_embed/src/context.rs`
- Modify: `crates/kali_embed/src/lib.rs`

**Interfaces:**
- Consumes: `crate::compiler::KaliCompiler` (from Task 3), `crate::error::CompileError` (from Task 1), `kali_sandbox`, `kali_error`.
- Produces: `crate::context::{EmbeddingCtx, OperationContext, PredicateDecision}`, re-exported at crate root. `RegisteredPredicate` and `predicate_violation` are module-private.

- [ ] **Step 1: Create `context.rs` with the moved context items**

```rust
use std::{
    collections::BTreeMap,
    sync::Arc,
};

use kali_error::{
    _error_codes::{e4, e5},
    Diagnostic,
};
use kali_sandbox::{
    HostOperation, PolicyPredicateContext, PolicyPredicateRegistry, SandboxPolicy,
};

use crate::compiler::KaliCompiler;

/// Decision returned by host-registered sandbox predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateDecision {
    /// Allow the guarded operation to proceed.
    Allow,
    /// Reject the guarded operation with a host-specific note.
    Deny(String),
}

impl PredicateDecision {
    // <verbatim from lib.rs 58–68>
}

impl From<bool> for PredicateDecision {
    // <verbatim from lib.rs 70–78>
}

/// Canonical operation context observed by host-registered narrowing predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationContext {
    // <verbatim from lib.rs 82–91>
}

impl OperationContext {
    // <verbatim from lib.rs 93–104>
}

#[derive(Clone)]
struct RegisteredPredicate {
    // <verbatim from lib.rs 350–354>
}

/// Embedding context retained for compatibility with the original stub API.
pub struct EmbeddingCtx {
    // <verbatim from lib.rs 357–361>
}

impl Default for EmbeddingCtx {
    // <verbatim from lib.rs 363–367>
}

impl EmbeddingCtx {
    pub fn new() -> Self {
        // <verbatim from lib.rs 370–372>
    }

    pub fn with_predicate_registration_enabled(enabled: bool) -> Self {
        // <verbatim from lib.rs 374–381>
    }

    pub fn predicate_registration_enabled(&self) -> bool {
        // <verbatim from lib.rs 383–386>
    }

    pub fn register_sandbox_predicate(
        &mut self,
        capability: impl Into<String>,
        name: impl Into<String>,
        predicate: impl Fn(&OperationContext) -> PredicateDecision + Send + Sync + 'static,
    ) -> Result<&mut Self, Diagnostic> {
        // <verbatim from lib.rs 388–409>
    }

    pub fn check_operation_with_policy(
        &self,
        policy: &SandboxPolicy,
        operation: HostOperation,
    ) -> Result<(), Diagnostic> {
        // <verbatim from lib.rs 411–434>
    }

    pub fn build_library(&self, source: &str) -> Vec<u8> {
        // <verbatim from lib.rs 436–443>
    }
}

fn predicate_violation(name: &str, context: &OperationContext, reason: &str) -> Diagnostic {
    // <verbatim from lib.rs 448–470>
}
```

**Critical detail:** Every item body is moved **verbatim** from lib.rs. The new additions to this module (not from lib.rs):
- `use crate::compiler::KaliCompiler;` — `EmbeddingCtx` owns a `KaliCompiler` field
- `use crate::error::CompileError;` — `EmbeddingCtx::build_library` returns `Result<LibArtifact, CompileError>` (though it maps to `.unwrap_or_default()`, the `compile_lib_source` return type references `CompileError`)

Note: `use kali_sandbox::{HostOperation, ...}` imports resolve types that were previously in scope via lib.rs's `pub use kali_sandbox::{...}`. Since `context.rs` is not a child of lib.rs, it can't use `crate::` re-exports for these — it must import them directly from `kali_sandbox`.

- [ ] **Step 2: Remove context items from `lib.rs` and wire the module**

Delete from `lib.rs`:
- `PredicateDecision` + impls (lines 50–77)
- `OperationContext` + impl (lines 81–104)
- `RegisteredPredicate` (lines 350–354)
- `EmbeddingCtx` struct + impls (lines 356–443)
- `predicate_violation` (lines 448–470)

Remove now-unused imports from `lib.rs`:
- `use std::collections::BTreeMap;` (only served context.rs)
- `use std::sync::Arc;` (only served context.rs)
- `use kali_error::_error_codes::{e4, e5};` (only served context.rs — e4+e5 were in predicate_violation and check_operation_with_policy, e8 already removed)
- `use kali_error::Diagnostic;` (only served context.rs and the already-extracted error.rs)

Add after the compiler module lines:

```rust
mod context;

pub use context::{EmbeddingCtx, OperationContext, PredicateDecision};
```

- [ ] **Step 3: Verify build + tests**

```bash
cargo build -p kali_embed 2>&1 | tail -5 && cargo test -p kali_embed 2>&1 | tail -3
```

Expected: 0 warnings; all tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_embed/src/context.rs crates/kali_embed/src/lib.rs
git commit -m "refactor(kali_embed): extract context module [refactor]"
```

---

### Task 5: Finalize lib.rs facade + test import fix

**Files:**
- Modify: `crates/kali_embed/src/lib.rs`

**Interfaces:**
- Consumes: all four modules from Tasks 1–4.
- Produces: thin facade — module doc, `mod` decls, `pub use` re-exports, `#[cfg(test)]` stubs, test wiring.

- [ ] **Step 1: Verify current state compiles and tests pass**

```bash
cargo build -p kali_embed 2>&1 | tail -5 && cargo test -p kali_embed 2>&1 | tail -3
```

Expected: 0 warnings; all tests pass.

If tests fail, the most likely cause is `tests.rs` losing access to symbols via `use super::*` that were previously in scope via lib.rs's private `use` imports. Common candidates:
- `PathBuf` — from `use std::path::PathBuf;` (removed with compiler extraction)
- `e4`, `e5` — from `use kali_error::_error_codes::{e4, e5};` (removed with context extraction)

- [ ] **Step 2: Add `#[cfg(test)]` use stubs for any missing symbols**

After verifying what `tests.rs` needs, add to `lib.rs` (before the test wiring):

```rust
#[cfg(test)]
use std::path::PathBuf;

#[cfg(test)]
use kali_error::_error_codes::{e4, e5};
```

Add ONLY the stubs that tests.rs actually needs. If tests pass without any stubs, skip this step.

- [ ] **Step 3: Clean up remaining unused imports in lib.rs**

At this point, `lib.rs` should have only:
- The `pub use kali_sandbox::{...}` re-export (stays)
- The `pub use kali_cli::build::{...}` re-export (stays)
- Any `#[cfg(test)]` stubs (added in step 2)
- The test wiring

Any remaining private `use` imports that survived from the original file (e.g., `use std::...;` already removed in Tasks 3–4) should be gone. Verify with `cargo build -p kali_embed` — it should show 0 warnings. If any unused import warning remains, remove the offending line.

- [ ] **Step 4: Final verification**

```bash
cargo build -p kali_embed 2>&1 | tail -5
cargo test -p kali_embed 2>&1 | tail -3
cargo build 2>&1 | tail -5
```

Expected: 0 warnings on all three; all tests pass; workspace compiles (no consumer breakage).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_embed/src/lib.rs
git commit -m "refactor(kali_embed): finalize facade, add test import stubs [refactor]"
```

---

### Task 6: FF-merge to main and verify

- [ ] **Step 1: FF-merge to main**

```bash
git checkout main
git merge --ff-only refactor/kali-embed-modularization
```

- [ ] **Step 2: Final verification on main**

```bash
cargo build -p kali_embed 2>&1 | tail -3
cargo test -p kali_embed 2>&1 | tail -3
cargo build 2>&1 | tail -3
```

Expected: 0 warnings; all tests pass; workspace compiles.

- [ ] **Step 3: Delete the feature branch**

```bash
git branch -d refactor/kali-embed-modularization
```

- [ ] **Step 4: Final commit (if any stubs adjusted)**

If no changes were needed, skip. Otherwise:
```bash
git commit -m "chore(kali_embed): finalize modularization [refactor]"
```

**Do NOT push to origin.** The series convention is local-main ff-merge only; origin/main intentionally lags.