# kali_api_web co-located src test-monolith modularization — design spec

**Series entry:** #34 (kali_api_web) of the crate-by-crate co-located **src** unit-test-monolith
modularization series. Predecessors: kali_mir, kali_npm, kali_capi, kali_parser, kali_common,
kali_codegen, kali_runtime, kali_types, kali_optimize, kali_cli.

**Date:** 2026-06-30
**Branch:** `refactor/kali_api_web-modularization` (off local `main`)
**Integration:** local-main ff-merge **only — never push to origin**.

## Goal

Decompose kali_api_web's four multi-concern co-located `src/*_tests.rs` unit-test monoliths into a
thin facade + per-concern `#[path] mod` submodules, with **zero behavior change** and a
**byte-identical** set of `#[test]` bodies. Pure verbatim code-motion + `mod`/`use` wiring; no
production-source edits, no public-surface changes, no `cargo fmt`.

This is the established series pattern; this entry is mechanical.

## Scope

In-scope: the four multi-concern monoliths only — **31 of kali_api_web's 57 lib tests**.

| File | Tests | Lines | Module path |
|------|-------|-------|-------------|
| `src/threads_tests.rs` | 11 | 414 | `threads::threads_tests` |
| `src/worker_tests.rs`  | 9  | 147 | `worker::worker_tests` |
| `src/events_tests.rs`  | 6  | 104 | `events::events_tests` |
| `src/crypto_tests.rs`  | 5  | 102 | `crypto::crypto_tests` |

**Out of scope (kept whole):** the remaining 26 lib tests live in 10 small, already single-concern
co-located `*_tests.rs` files (`file` 5, `streams` 3, `url` 3, `util` 3, `base64` 3, `indexeddb` 2,
`navigator` 2, `storage` 2, `websocket` 2, `fetch` 1). Splitting them would not improve clarity
(series convention: small/single-concern files stay whole).

All four in-scope files:
- have **0** non-`#[test]` module-level fns (facades drain fully to 0). Any nested helper fns
  inside a test body travel verbatim with their parent test.
- have **0** `include_str!` / `include_bytes!` / `include!` macros (no facade pins needed).
- are declared from their **product sibling** via an existing `#[cfg(test)] #[path = "…"] mod …;`
  trio that is left **untouched** (it continues to point at the facade):
  - `src/threads.rs:527-529` → `threads_tests`
  - `src/worker.rs:195-197` → `worker_tests`
  - `src/events.rs:241-243` → `events_tests`
  - `src/crypto.rs:111-113` → `crypto_tests`

## Target structure

All four files partition along **mutually-exclusive leading prefixes** of the `#[test]` fn name, so
the mover's native `startswith` grouping applies directly (no exact-name lists). Submodules are
written to `src/<stem>_tests/<mod>.rs`, each headed by exactly `use super::*;`. Final namespace:
`<stem>::<stem>_tests::<mod>::<test>`.

### File 1 — `src/threads_tests.rs` → facade + 3 submodules

Groups-spec: `topology=thread_runtime_topology_;atomics=atomics_;shared_array_buffer=shared_array_buffer_`

| Submodule | N | Concern | Tests |
|-----------|---|---------|-------|
| `topology` | 7 | per-worker thread-runtime instance topology / live-instance snapshots & shutdown | `thread_runtime_topology_assigns_one_instance_per_worker`, `thread_runtime_topology_keeps_monotonic_instance_ids_after_termination`, `thread_runtime_topology_snapshot_reports_live_instances_deterministically`, `thread_runtime_topology_shutdown_reports_live_instances_deterministically`, `thread_runtime_topology_shutdown_keeps_live_instances_sorted_by_id`, `thread_runtime_topology_shutdown_keeps_live_instances_sorted_after_first_termination`, `thread_runtime_topology_counts_multiple_terminated_instances_deterministically` |
| `atomics` | 1 | `Atomics` lock-free status | `atomics_reports_lock_free_status_deterministically` |
| `shared_array_buffer` | 3 | `SharedArrayBuffer` cloning / compare-exchange / zero-length | `shared_array_buffer_clones_share_mutations`, `shared_array_buffer_compare_exchange_failure_leaves_bytes_unchanged`, `shared_array_buffer_supports_zero_length_buffers` |

Facade retains, verbatim, its 3 `use` lines and nothing else (0 `#[test]`):
```
use crate::*;
use kali_common::bytewise_shared_memory_is_lock_free;
use serde_json::Value;
```
Children reach these symbols via `use super::*;` (Rust descendant-visibility re-exports the
facade's private `use` items through the child glob — verified clean at 0 warnings across prior
series entries).

### File 2 — `src/worker_tests.rs` → facade + 2 submodules

Groups-spec: `worker_stub=worker_stub_;broadcast_channel=broadcast_channel_`

| Submodule | N | Concern | Tests |
|-----------|---|---------|-------|
| `worker_stub` | 5 | `Worker` stub message/shared-buffer recording & lifecycle | `worker_stub_records_posted_messages`, `worker_stub_records_shared_buffers_with_shared_backing`, `worker_stub_trims_surrounding_whitespace_from_script_urls`, `worker_stub_ignores_shared_buffer_posts_after_termination`, `worker_stub_preserves_interleaved_post_order` |
| `broadcast_channel` | 4 | `BroadcastChannel` stub message/shared-buffer recording & close | `broadcast_channel_stub_records_posted_messages`, `broadcast_channel_stub_records_shared_buffers_with_shared_backing`, `broadcast_channel_stub_ignores_shared_buffer_posts_after_close`, `broadcast_channel_stub_preserves_interleaved_post_order` |

Facade retains, verbatim, its 2 `use` lines and nothing else (0 `#[test]`):
```
use crate::*;
use serde_json::Value;
```

### File 3 — `src/events_tests.rs` → facade + 3 submodules

Groups-spec: `abort=abort_;event_target=event_target_;custom_event=custom_event_`

| Submodule | N | Concern | Tests |
|-----------|---|---------|-------|
| `abort` | 2 | `AbortController` / `AbortSignal` | `abort_controller_flips_the_signal`, `abort_signal_dispatches_abort_events_once` |
| `event_target` | 3 | `EventTarget` listener dispatch / removal / re-entrancy | `event_target_dispatches_registered_listeners`, `event_target_can_remove_registered_listeners`, `event_target_can_remove_listeners_during_dispatch_without_deadlocking` |
| `custom_event` | 1 | `CustomEvent` detail payload | `custom_event_carries_detail_payload` |

Facade retains, verbatim, its 4 `use` lines and nothing else (0 `#[test]`):
```
use crate::*;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
```
(The `std::sync::atomic` / `Arc` symbols are consumed by the `event_target` re-entrancy test; they
remain in the facade and reach that child via `use super::*;`.)

### File 4 — `src/crypto_tests.rs` → facade + 2 submodules

Groups-spec: `randomness=random_,crypto_facade_;subtle=crypto_subtle_`
(the `randomness` group uses two disjoint leading prefixes; `crypto_facade_` does **not** match the
`subtle` prefix `crypto_subtle_`, so the partition is clean and order-independent).

| Submodule | N | Concern | Tests |
|-----------|---|---------|-------|
| `randomness` | 3 | `crypto.getRandomValues` / `randomUUID` / shared-randomness facade | `random_fill_populates_the_requested_buffer`, `random_uuid_has_the_expected_shape`, `crypto_facade_reuses_the_shared_randomness_helpers` |
| `subtle` | 2 | `crypto.subtle.digest` algorithms / rejection | `crypto_subtle_digest_supports_sha1_sha256_sha384_and_sha512`, `crypto_subtle_digest_rejects_unknown_algorithms` |

Facade retains, verbatim, its 1 `use` line and nothing else (0 `#[test]`):
```
use crate::*;
```

## Mechanics

- **Mover:** `.superpowers/sdd/move_fns.py` (run from `crates/kali_api_web`), **native leading-prefix
  `startswith` grouping** — a `#[test]` fn joins the first group whose prefix-tuple its name starts
  with; comma-separates multiple prefixes per group (used for crypto's `randomness`). No `*`
  catch-all needed (all groups have explicit prefixes; the per-file partition is exhaustive). The
  tool is git-ignored scratch and recreated for this entry from the documented spec; **keep
  `FN_RE` / `IDENT_CHARS` / `find_close_line` byte-identical** — only ROOT/GROUPS/main() vary.
- **Verifier:** `.superpowers/sdd/verify.py` proves `{name: body}` of `#[test]` fns is byte-identical
  between the pre-move snapshot and the submodule glob, per file.
- **Pre-move snapshots:** copy all four files to a fixed out-of-repo scratch dir
  (`/tmp/claude-1000/-workspace/kali_api_web_split_scratch/orig/`) before any move, for verify.py.
- Product siblings (`threads.rs`, `worker.rs`, `events.rs`, `crypto.rs`) and all out-of-scope
  `*_tests.rs` files are **untouched** (diff must be empty for them).

## Gates (literal — no env carve-outs; kali_api_web baseline is clean)

Baseline captured 2026-06-30:
- `cargo build -p kali_api_web --tests` → **0** warnings
- `cargo test -p kali_api_web --lib` → **57 passed; 0 failed**
- per-file `--list` counts: `threads::threads_tests` **11**, `worker::worker_tests` **9**,
  `events::events_tests` **6**, `crypto::crypto_tests` **5**

Post-split, all must hold:
1. `cargo build -p kali_api_web --tests` → **0** warnings (unchanged).
2. `cargo test -p kali_api_web --lib` → **57 pass / 0 fail** (unchanged).
3. Per-file `--list` count preserved (11 / 9 / 6 / 5), comparing name-sets with the new module
   prefix stripped (`sed -E 's/^.*:://'`) → empty diff vs baseline. Anchor any `--list` filter with
   `^` as a precaution (none of `threads`/`worker`/`events`/`crypto` is a suffix-substring of
   another, so the kali_runtime over-count hazard does not apply, but anchor anyway).
4. `verify.py` byte-identity **PROOF OK** for each of the 4 files (orig snapshot vs submodule glob).
5. Each facade `#[test]` count == **0**.
6. Changed paths = exactly **4 facades + 10 submodules** (14 files); no production/`pub`-widen/
   `include`/fmt changes.
7. Dependent crates (any kali_api_web consumer) compile **unedited**.

**fmt:** do **not** run `cargo fmt`. The repo's `cargo fmt --all --check` gate is already red on
baseline across many crates; verbatim moves may leave minor nits in the moved/facade lines — these
are accepted per series convention and are not regressions.

## Process

- SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch), overwritten for this entry.
- 4 implementation tasks (one per file): sonnet implementer → review-package → sonnet task reviewer.
- Final: opus whole-branch review (line-conservation + byte-identity reproof) → ff-merge to local
  `main` → re-verify on merged main → delete branch. **No origin push.**

## Non-goals

- No splitting of the 10 out-of-scope small/single-concern lib-test files.
- No production-source refactoring, no public-API changes, no path rewrites, no `cargo fmt`.
- No origin push (origin/main intentionally lags).
