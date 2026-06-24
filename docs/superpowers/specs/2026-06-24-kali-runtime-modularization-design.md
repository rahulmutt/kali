# Kali Runtime Modularization — Design

**Date:** 2026-06-24
**Status:** Approved (design)
**Scope:** Apply the established crate-modularization pattern (facade + co-located
tests + shared/local test support) to `kali_runtime`. Pure text-movement, zero
behavior change.

## Problem

`kali_runtime` is the largest remaining monolith in the workspace:

- `crates/kali_runtime/src/lib.rs` — **4,599 lines / 165 KB**. One central
  `impl RuntimeCtx` block (lines ~365–3644, dominated by a ~640-line
  `execute_inner`), two ~770/~800-line free functions
  (`register_default_host_imports`, `register_node_host_imports`), a ~1,500-line
  cluster of `browser_*` free functions + types, and 64 free functions overall.
- `crates/kali_runtime/src/tests.rs` — **7,149 lines, 158 `#[test]`s, flat (no
  submodules)**, with heavy filesystem/tempdir usage (~249 sites).

This is the same shape `kali_types` (pilot) and `kali_codegen` had before their
(now merged) refactors. Large single files are hard to navigate, review, and
hold in context. The goal: break the crate into small, single-purpose modules
and co-locate tests with the code they exercise, factoring shared setup into
helpers and adopting the cross-crate fixture crate.

## Goal & Hard Constraints

This is a **pure structural refactor — zero behavior change.**

- The exact same set of tests exists and passes before and after.
- `lib.rs` becomes a thin **facade** (module declarations, the crate import
  surface, import-index `const`s, and `pub use` re-exports) so every external
  path (e.g. `kali_runtime::RuntimeCtx`) keeps resolving. No public API churn —
  downstream crates (`kali_cli`, …) compile untouched.
- Unit tests live in **sibling `*_tests.rs` files wired via
  `#[cfg(test)] #[path = "…"] mod …`**, not inline `#[cfg(test)]` modules.
- Extraction is **text-movement only**: cross-referenced
  items/fields/methods/functions are widened to `pub(crate)` first, then bodies
  are moved verbatim. The giant functions (`execute_inner`,
  `register_default_host_imports`, `register_node_host_imports`) move
  **byte-for-byte intact** — cracking them into helpers is explicitly deferred
  (see Out of scope), mirroring how `kali_codegen` deferred cracking `emit_call`.

### Proof obligation

Capture a baseline before touching code and compare after:

- `cargo test -p kali_runtime -- --list` yields the **same set of test
  basenames** before and after. Note: `--list` *includes* module-path prefixes,
  so co-location makes a raw diff non-empty by design — prove the invariant by
  stripping prefixes and comparing **basenames**. Baselines recorded under
  `docs/superpowers/baselines/` (`kali_runtime-tests-before.txt`,
  `kali_runtime-tests-after.txt`, and a `…-renames.md` mapping).
- Green at every commit: `cargo build -p kali_runtime`, `cargo test -p
  kali_runtime`, and `cargo clippy -p kali_runtime -- -D warnings` all pass.
- Final gate: full-workspace `cargo build` + `cargo test` confirm no downstream
  breakage.

## Architecture

`RuntimeCtx`, `KaliHostState`, `RuntimeOutcome`, and all current `pub` items keep
their definitions, fields, and public API unchanged. The `impl RuntimeCtx` block
splits into separate `impl RuntimeCtx` blocks across modules (legal within one
crate); free functions move as plain `pub(crate) fn`.

### Target layout

```
crates/kali_runtime/src/
  lib.rs            # facade: mod decls, imports, import-index consts,
                    #   pub use re-exports, #[cfg(test)] wiring. No logic.
  ctx.rs            # RuntimeCtx struct + Default + builders/accessors
                    #   (new, with_*, env_*, snapshot_*, host_contract,
                    #   runtime_backend, process_id, effective_thread_budget,
                    #   canonical_runtime_profiles, …)
  outcome.rs        # RuntimeOutcome struct (+ impls)
  execute.rs        # impl RuntimeCtx: execute / execute_tests /
                    #   execute_inner (intact) / reject_unavailable_threaded_*;
                    #   execute_browser_runtime
  state.rs          # KaliHostState (+ Default/impl), ScheduledTimer
  profiles.rs       # RuntimeHostContract, RuntimeBackend,
                    #   normalize_runtime_profiles, parse_*_label helpers
  host/             # directory module: host-import registration + guest plumbing
    mod.rs          #   facade
    imports_default.rs  # register_default_host_imports (intact, ~770)
    imports_node.rs     # register_node_host_imports   (intact, ~800)
    memory.rs       # guest_memory, checked_offset, read/write guest string/bytes,
                    #   decode_spawn_args
    io.rs           # append_stdout/stderr(_raw), format_console_value
    enforce.rs      # enforce_operation, drain_event_loop, invoke_callback
    diagnostics.rs  # host_import_error, runtime_error_diagnostic,
                    #   resolve_host_path, normalize_path
  browser/          # directory module: the ~1,500-line browser_* cluster
    mod.rs          #   facade
    contract.rs     # BrowserRuntimeContract (+descriptor),
                    #   browser_runtime_contract_value, is_canonical
    command.rs      # split_command_spec, browser_harness_*command_parts*,
                    #   executable-name helpers
    harness.rs      # browser_bundle_harness_*, browser_runtime_harness_*
                    #   (scripts / pages / prelude / module scripts)
    execute.rs      # browser_runtime_execute_checked,
                    #   browser_bundle_runtime_execute_checked,
                    #   browser_harness_invocation_checked, *_run_checked(_with_env);
                    #   BrowserRuntimeExecutionOutcome, BrowserHarnessInvocation,
                    #   BrowserHarnessOutcome, BrowserHarnessError
    summary.rs      # parse_browser_runtime_summary*, parse_thread_runtime_*
```

Exact function-to-module assignment is finalized in the implementation-plan
phase; this design fixes the **module boundaries and names**. A function whose
domain is ambiguous goes to the module its primary caller/sibling lives in; when
still unclear, defer to plan-phase review rather than guessing.

### Components & boundaries

- **`ctx.rs`** — owns the runtime context vocabulary and its builder/accessor
  surface. Independent of the execute/host layers.
- **`execute.rs`** — drives WASM instantiation and execution; wires the host
  linker (`host/`) and dispatches to `browser/` for the browser backend.
- **`state.rs`** — the host-side state the linker closures mutate.
- **`host/`** — registers host imports into the `wasmtime::Linker` and provides
  guest-memory read/write plumbing, IO buffering, sandbox enforcement, and the
  event-loop drain. `imports_default.rs` / `imports_node.rs` hold the two large
  registration functions intact.
- **`browser/`** — everything for the browser runtime backend: contract
  description, command/executable resolution, harness script/page generation,
  checked execution, and summary parsing.
- **`profiles.rs`** — runtime-profile normalization and host-contract/backend
  label parsing.

Directory-module submodules use `use crate::*;` (not `super`).

## Data flow

Unchanged by this refactor. For reference: a caller builds a `RuntimeCtx`,
`execute`/`execute_tests` instantiate the WASM module under `wasmtime` with a
`Linker` populated by `host/` (default + node imports), run it (or hand off to
`browser/` for the browser backend), and collect a `RuntimeOutcome`. The refactor
only relocates the code that already implements this flow.

## Testing

### Co-location

Split the flat `tests.rs` (158 tests) into sibling `*_tests.rs`, one per source
module, each wired at the bottom of its module:

```rust
#[cfg(test)]
#[path = "execute_tests.rs"]
mod execute_tests;
```

Tests are grouped by the module they exercise (`ctx_tests.rs`, `execute_tests.rs`,
`profiles_tests.rs`, `host/*_tests.rs`, `browser/*_tests.rs`, …). Directory-module
test files use `use crate::*;`. The net `cargo test -- --list` **basename** set is
identical to the baseline.

### Shared & local support

- **`kali_test_support`** (existing dev-crate): added to `kali_runtime`'s
  `[dev-dependencies]`; adopt its cross-crate fixtures
  (`fixtures::tempdir/write_file/write_manifest`) at the ~249 filesystem/tempdir
  sites in the current tests. Unlike `kali_codegen` (which had **no** fs tests
  and so needed a one-off added fixture test to exercise the dep), `kali_runtime`
  has abundant fs/tempdir tests to convert — so the **test count stays at 158**
  and the identical-basename-set invariant holds with no deviation.
- **Runtime-local `test_support` module**: collect the repetitive
  WAT/wasm-building and `RuntimeCtx`-building helpers that recur across tests so
  every `*_tests.rs` shares them. Opt-in and behavior-preserving — migrate a test
  only where it reduces noise, and the bytes/assertions stay identical.

## Error handling

No change. Diagnostics (`kali_error`) and fallback paths move with their
functions; behavior is preserved and asserted by the unchanged tests.

## Sequencing & commit strategy

Incremental, green-at-every-commit, mirroring the prior refactors. Work lands on
branch `refactor/kali-runtime-modularization` off `main`. Commit messages follow
the existing style (`refactor(kali_runtime): …`, `test(kali_runtime): … [refactor]`,
`style(kali_runtime): …`).

1. **Baseline** — record `cargo test -- --list` + counts under
   `docs/superpowers/baselines/`. (no code change)
2. **Widen visibility** — flip cross-referenced items/fields/functions to
   `pub(crate)` while still a single `lib.rs`.
3. **Extract scaffolding** — `ctx.rs`, `outcome.rs`, `state.rs`, `profiles.rs`
   (text-movement); facade `lib.rs` grows its `mod`/`pub use` lines.
4. **Extract `host/`** — `memory` → `io` → `diagnostics` → `enforce` →
   `imports_default` → `imports_node` (giant fns moved intact), one commit per
   file or small group.
5. **Extract `browser/`** — `contract` → `command` → `harness` → `summary` →
   `execute`, one commit per file.
6. **Extract `execute.rs`** — the `impl RuntimeCtx` execute methods +
   `execute_browser_runtime`.
7. **`cargo fmt`** normalization — its own commit.
8. **Test support** — introduce runtime-local `test_support`; wire
   `kali_test_support` dev-dep.
9. **Co-locate tests** — split `tests.rs` into `*_tests.rs` per module, one
   commit per module group; delete the monolith last.
10. **Adopt fixtures** in migrated tests (the ~249 fs/tempdir sites) where they
    cut noise.
11. **Post-refactor baseline** + final workspace-wide build/test/clippy.

**Per-step verification:** `cargo build -p kali_runtime` → `cargo test -p
kali_runtime` → `cargo clippy -p kali_runtime -- -D warnings`. Final step also
runs the full-workspace build + test. A whole-branch opus review precedes merge.

## Out of scope

- Any behavior, output, or public-API change.
- **Cracking the giant functions** (`execute_inner`,
  `register_default_host_imports`, `register_node_host_imports`) into smaller
  helpers — that is a separate logic refactor with its own spec → plan cycle,
  deferred exactly as `kali_codegen` deferred cracking `emit_call`.
- Refactoring other crates (`kali_optimize`, `kali_common`, `kali_cli`, …) — each
  is its own spec → plan → implementation cycle.
- Renaming public items or restructuring the WASM/host-import ABI.

## References

- Pilot design: `docs/superpowers/specs/2026-06-23-kali-crate-modularization-design.md`
- Codegen design: `docs/superpowers/specs/2026-06-24-kali-codegen-modularization-design.md`
- Codegen emit sub-split: `docs/superpowers/specs/2026-06-24-kali-codegen-emit-subsplit-design.md`
- Prior baselines: `docs/superpowers/baselines/kali_{types,codegen}-tests-*`
- Memory: `kali-crate-modularization`
