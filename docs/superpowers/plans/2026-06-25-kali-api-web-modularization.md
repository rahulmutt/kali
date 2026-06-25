# kali_api_web Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose `kali_api_web/src/lib.rs` (2,682 lines) into ~14 flat per-Web-API-family modules behind a glob-re-export facade, with co-located sibling tests, changing zero behavior.

**Architecture:** `kali_api_web` is an *independent-object pile* — ~40 self-contained Web-API objects, each a `struct` + its own `impl` block(s) + (often) an error enum, plus ~25 free fns / statics / consts that each belong to one family. There is **no shared mega-struct**, so **no Task-1 `pub(crate)` receiver-widening is required** (cross-family references are by already-`pub` public types). Every item moves **byte-for-byte verbatim** into its family module; `lib.rs` becomes a thin facade of `mod` decls + alphabetical `pub use <mod>::*;` globs that preserve every `kali_api_web::Name` path.

**Tech Stack:** Rust (workspace crate), `cargo test` / `cargo clippy`, subagent-driven-development.

**Design:** `docs/superpowers/specs/2026-06-25-kali-api-web-modularization-design.md`

## Global Constraints

- **Zero behavior change.** Items relocate verbatim — including whitespace, blank-line separators between items, and relative source order. Never rewrite, reorder within a kept block, or "tidy" code.
- **This is a relocation refactor, not TDD.** No new product code and no new tests are written. The 57 existing tests are the safety net; they must stay green and keep their exact names. "Show the code" for a step here means *name the items and give their current line range*, not paste 2,682 lines — the operation is a mechanical cut-paste.
- **Test set is invariant at 57.** Proven by **basename-multiset**: strip module-path prefixes from `cargo test -p kali_api_web -- --list`, `sort` *without* `-u`, diff against the baseline → must be empty. (`--list` prefixes change as tests relocate; raw diff is non-empty by design.)
- **Green + clippy-clean at every commit:** `cargo test -p kali_api_web` passes and `cargo clippy -p kali_api_web --all-targets -- -D warnings` is clean after every task.
- **`kali_test_support` is NOT adopted.** Confirmed: `tests.rs` has zero fs/tempdir usage (`grep -cE 'tempdir|TempDir|NamedTempFile|tempfile|fs::write|fs::read|std::fs' = 0`). Adding the dev-dep + a token round-trip test would be dead weight (foundational-crate resolution, same as kali_common / kali_ast). Hold the suite strictly at 57 — no added fixture test.
- **Facade-only `lib.rs`.** When complete, `lib.rs` contains only: crate doc comment, alphabetical `mod` decls, alphabetical `pub use <mod>::*;` globs. No type/fn/static/const definitions remain.
- **Glob rule (clippy `-D warnings`):** every family module here exports public types, so each gets `pub use <mod>::*;`. If clippy ever flags a glob as re-exporting nothing, the module owns only `pub(crate)` items → switch that one to `pub(crate) use <mod>::*;`; if it genuinely re-exports nothing used, delete the glob (never `#[allow]` it). Not expected for any module in this plan.

---

## Standard Extraction Procedure (every Task 2–15 follows these 7 steps)

Each family task substitutes `<family>` (e.g. `streams`), its **item list + current line ranges**, its **"neighbors that STAY" list**, and its **test names**. Line ranges in this plan are from the pre-refactor file and **shift after every prior extraction** — always re-grep the current item boundaries before cutting (`grep -nE '^(pub )?(struct|enum|fn|pub fn|const|static|type) <Name>' crates/kali_api_web/src/lib.rs`). Move each struct/enum **together with all of its `impl` blocks** (including trait impls like `Display`/`Default`/`PartialEq` that follow it).

- [ ] **Step A — Create `crates/kali_api_web/src/<family>.rs`.** Start with a module doc line and the precise `use` imports the moved code needs — a subset of `lib.rs`'s header (`serde_json::Value`, `std::{...}`, `kali_common::...`, `sha1`/`sha2`, `url::...`, etc.) plus `use crate::{SiblingType, ...};` for any types owned by other family modules. Let the compiler tell you exactly which (`cargo build -p kali_api_web`, add what's missing, remove what's unused).
- [ ] **Step B — Move the listed items verbatim** from `lib.rs` into `<family>.rs` (cut, not copy), preserving order and blank-line separators. Leave every "neighbor that STAYS" untouched in place.
- [ ] **Step C — Wire the facade.** Add `mod <family>;` and `pub use <family>::*;` to `lib.rs`, both in alphabetical position. Remove any `use` import in `lib.rs`'s header that the moved items took with them and that nothing remaining uses (compiler-driven — fix the resulting `unused_imports` warning in this same commit).
- [ ] **Step D — Relocate this family's tests.** Cut the listed test fns verbatim from `src/tests.rs` into a new `crates/kali_api_web/src/<family>_tests.rs`; change its header from `use super::*;` to `use crate::*;` (add any specific extra import the moved tests used from `tests.rs`'s header — e.g. `use kali_common::bytewise_shared_memory_is_lock_free;` or `use std::sync::atomic::AtomicUsize;` — only if those tests reference it). Add to the bottom of `<family>.rs`:
  ```rust
  #[cfg(test)]
  #[path = "<family>_tests.rs"]
  mod <family>_tests;
  ```
- [ ] **Step E — Verify green + clippy-clean.**
  Run: `cargo test -p kali_api_web` → Expected: PASS (still 57 tests).
  Run: `cargo clippy -p kali_api_web --all-targets -- -D warnings` → Expected: no warnings.
- [ ] **Step F — Verify the basename-multiset proof holds.**
  Run:
  ```bash
  cargo test -p kali_api_web -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
    | diff - <(sed -E 's/^.*:://' .superpowers/sdd/kali_api_web_list_before.txt | sort)
  ```
  Expected: empty output (identical 57-name multiset).
- [ ] **Step G — Commit.**
  ```bash
  git add crates/kali_api_web/src/
  git commit -m "refactor(kali_api_web): extract <family> module [refactor]"
  ```

---

## Task 1: Baseline snapshot (controller-run, no commit)

**Files:** none modified. Produces `.superpowers/sdd/kali_api_web_list_before.txt`.

- [ ] **Step 1:** Capture the test baseline:
  ```bash
  mkdir -p .superpowers/sdd
  cargo test -p kali_api_web -- --list 2>/dev/null | grep ': test$' | sed 's/: test$//' | sed -E 's/^.*:://' | sort > .superpowers/sdd/kali_api_web_list_before.txt
  wc -l .superpowers/sdd/kali_api_web_list_before.txt   # Expected: 57
  ```
- [ ] **Step 2:** Confirm the workspace is green at HEAD: `cargo test -p kali_api_web` → 57 passed.
- [ ] **Step 3:** Confirm `kali_test_support` skip: `grep -cE 'tempdir|TempDir|NamedTempFile|tempfile|fs::write|fs::read|std::fs' crates/kali_api_web/src/tests.rs` → `0`. No dev-dep, no added test.

This task is run directly by the controller (trivial, no commit).

---

## Task 2: Extract `util`

**Files:** Create `crates/kali_api_web/src/util.rs`, `crates/kali_api_web/src/util_tests.rs`; Modify `crates/kali_api_web/src/lib.rs`, `crates/kali_api_web/src/tests.rs`.

**Items to move** (re-grep current ranges; these are interleaved across the file):
- `web_api_init` (≈46), `text_encode` (≈49), `text_decode` (≈54), `structured_clone` (≈59) — the top free-fn block.
- `performance_now` (≈1465) and the `static TIME_ORIGIN: OnceLock<Instant>` (≈19) it reads.

**Neighbors that STAY:** the other three statics (`LOCAL_STORAGE`/`SESSION_STORAGE` → storage, `NAVIGATOR` → navigator) stay at crate root for now; `UrlMutationError` (≈26) stays (→ url, Task 9).

**Tests to move (3):** `performance_now_is_monotonic_and_non_negative`, `text_codec_round_trips_unicode`, `structured_clone_copies_values`.

Follow the Standard Extraction Procedure (Steps A–G) with `<family>` = `util`.

---

## Task 3: Extract `base64`

**Files:** Create `base64.rs`, `base64_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (contiguous block ≈1202–1360, before `URLSearchParams`): `const BASE64_ALPHABET` (≈1202), `Base64Error` (≈1207, + its impls), `btoa` (≈1228), `atob` (≈1244), `encode_base64` (≈1266), `decode_base64` (≈1293), `decode_base64_value` (≈1348).

**Neighbors that STAY:** `URLSearchParams` (≈1361) immediately below STAYS (→ url).

**Tests to move (3):** `base64_helpers_round_trip_binary_strings`, `base64_helpers_reject_out_of_range_input`, `base64_helpers_reject_malformed_input_lengths`.

Follow Steps A–G with `<family>` = `base64`. The base64 helpers are self-contained (no sibling-type imports needed).

---

## Task 4: Extract `crypto`

**Files:** Create `crypto.rs`, `crypto_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (mostly contiguous ≈1474–1577): `fill_random_values` (≈1474), `random_uuid` (≈1479), `WebCryptoError` (≈1498, + impls), `canonicalize_digest_algorithm` (≈1514), `Crypto` (≈1525, + impl), `SubtleCrypto` (≈1546, + impl), `crypto` fn (≈1572).

**Neighbors that STAY:** `performance_now` (≈1465, → util, already moved in Task 2 if done first — re-grep) immediately above; `AbortSignal` (≈1578) immediately below STAYS (→ events).

**Imports:** needs `sha1::Sha1`, `sha2::{Digest, Sha224, Sha256, Sha384, Sha512}`, `getrandom`.

**Tests to move (5):** `random_fill_populates_the_requested_buffer`, `random_uuid_has_the_expected_shape`, `crypto_facade_reuses_the_shared_randomness_helpers`, `crypto_subtle_digest_supports_sha1_sha256_sha384_and_sha512`, `crypto_subtle_digest_rejects_unknown_algorithms`.

Follow Steps A–G with `<family>` = `crypto`.

---

## Task 5: Extract `streams`

**Files:** Create `streams.rs`, `streams_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (contiguous block ≈178–456): `DeterministicStreamState` (≈178, private struct + impl), `ReadableStream` (≈213, + impl + PartialEq/Eq), `WritableStream` (≈277, + impl + PartialEq/Eq), `TransformStream` (≈333, + impl + Default), `TextEncoderStream` (≈369, + impl + Default + PartialEq/Eq), `TextDecoderStream` (≈413, + impl + Default + PartialEq/Eq).

**Neighbors that STAY:** `File` (≈116) above and `FormDataValue` (≈457) below both STAY (→ file, Task 6).

**Imports:** `TextEncoder/DecoderStream` reference the encoding helpers `text_encode`/`text_decode` now in `util` → `use crate::{text_encode, text_decode};` (verify which it actually calls; compiler-driven).

**Tests to move (3):** `readable_stream_shares_state_and_closing_is_deterministic`, `writable_and_transform_streams_share_the_same_backing_state`, `text_encoder_and_decoder_streams_share_the_shared_baseline`.

Follow Steps A–G with `<family>` = `streams`.

---

## Task 6: Extract `file`

**Files:** Create `file.rs`, `file_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (NON-CONTIGUOUS — two regions split by the stream structs, now removed by Task 5): `Blob` (≈65, + impl), `File` (≈116, + impl); then `FormDataValue` (≈457, enum), `FormDataEntry` (≈489), `FormData` (≈516, + impl), `FileReaderState` (≈597, enum), `FileReader` (≈605, + impl).

**Neighbors that STAY:** `Storage` (≈672) below STAYS (→ storage).

**Imports:** `FileReader`/`FormData` may reference `Blob`/`File` (same module, no import). If `File`/`FormData` reference stream types, `use crate::ReadableStream;` etc. (compiler-driven).

**Tests to move (5):** `blob_collects_bytes_and_text`, `file_wraps_blob_metadata`, `file_reader_reads_blob_and_file_payloads`, `blob_and_file_stream_baselines_preserve_bytes`, `form_data_records_entries_and_preserves_order`.

Follow Steps A–G with `<family>` = `file`.

---

## Task 7: Extract `storage`

**Files:** Create `storage.rs`, `storage_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move:** `Storage` (≈672, + impl), `local_storage` fn (≈754), `session_storage` fn (≈759), and the `static LOCAL_STORAGE` + `static SESSION_STORAGE` (≈20–21).

**Neighbors that STAY:** the `navigator` fn (≈764), `parse_url`/`resolve_url` (≈769/774), `Navigator` (≈780) all STAY (→ navigator / url).

**Tests to move (2):** `storage_round_trips_values_and_stays_ordered`, `shared_browser_storage_buckets_remain_isolated`.

Follow Steps A–G with `<family>` = `storage`.

---

## Task 8: Extract `navigator`

**Files:** Create `navigator.rs`, `navigator_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move:** `Navigator` (≈780, + impl), `navigator` fn (≈764), and the `static NAVIGATOR` (≈22).

**Neighbors that STAY:** `parse_url`/`resolve_url` (≈769/774) STAY (→ url); `URL` (≈854) STAYS (→ url).

**Tests to move (2):** `navigator_baseline_exposes_stable_metadata`, `navigator_snapshot_helpers_expose_deterministic_object_and_json_views`.

Follow Steps A–G with `<family>` = `navigator`.

---

## Task 9: Extract `url`

**Files:** Create `url.rs`, `url_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (NON-CONTIGUOUS — four regions): `UrlMutationError` (≈26, enum + Display/Error impls), `parse_url` (≈769), `resolve_url` (≈774), `URL` (≈854, + impl), `URLSearchParams` (≈1361, + impl).

**Neighbors that STAY:** `Headers`/`Request`/`Response`/`fetch`/`normalize_header_name` (≈991–1198) all STAY (→ fetch); the base64 block (already moved Task 3).

**Imports:** `use url::{form_urlencoded, Url};` (and `ParseError`).

**Tests to move (3):** `url_search_params_round_trips_values_and_serializes_deterministically`, `url_parser_can_parse_and_resolve`, `url_object_round_trips_components`.

Follow Steps A–G with `<family>` = `url`. **Module name caution:** the file is `url.rs` and it also imports the external `url` crate — inside `url.rs`, `use url::Url;` still refers to the external crate (Rust 2021 paths), but verify the build; if a collision arises, qualify as `::url::Url` (mechanical, no logic change).

---

## Task 10: Extract `fetch`

**Files:** Create `fetch.rs`, `fetch_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (contiguous ≈991–1201): `normalize_header_name` fn (≈991, private), `Headers` (≈997, + impl), `Request` (≈1063, + impl), `Response` (≈1119, + impl), `fetch` fn (≈1198).

**Neighbors that STAY:** `BASE64_ALPHABET`/base64 block (≈1202+, already moved Task 3).

**Imports:** `Request`/`Response` reference `Blob`/`FormData` (`use crate::{Blob, FormData};`) and possibly `ReadableStream` (`use crate::ReadableStream;`) — compiler-driven.

**Tests to move (1):** `headers_request_and_response_round_trip_deterministically`.

Follow Steps A–G with `<family>` = `fetch`.

---

## Task 11: Extract `events`

**Files:** Create `events.rs`, `events_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (contiguous ≈1578–1716): `AbortSignal` (≈1578, + impl), `AbortController` (≈1607, + impl), `Event` (≈1632, + impl), `CustomEvent` (≈1674, + impl), the three type aliases `EventListenerId`/`EventListener`/`SharedEventListener` (≈1696–1698), `RegisteredEventListener` (≈1700, private struct), the `ListenerMap` type alias (≈1706), `EventTarget` (≈1710, + impl — NOTE its `impl EventTarget` block lives far down near the end of the file ≈2590; re-grep and move ALL `impl EventTarget` blocks with the struct).

**Neighbors that STAY:** `WebSocket` (≈1717) STAYS (→ websocket).

**Tests to move (6):** `abort_controller_flips_the_signal`, `abort_signal_dispatches_abort_events_once`, `event_target_dispatches_registered_listeners`, `event_target_can_remove_registered_listeners`, `event_target_can_remove_listeners_during_dispatch_without_deadlocking`, `custom_event_carries_detail_payload`.

Follow Steps A–G with `<family>` = `events`. **Caution:** `EventTarget`'s `impl` is non-adjacent (near the file tail) — grep `^impl EventTarget` and move every match.

---

## Task 12: Extract `websocket`

**Files:** Create `websocket.rs`, `websocket_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (≈1717–1792): `WebSocket` (≈1717, + impl), `WebSocketReadyState` (≈1726, enum + impls).

**Neighbors that STAY:** `PostedItem` (≈1793) below STAYS (→ worker).

**Tests to move (2):** `websocket_stub_tracks_sent_messages`, `websocket_stub_clones_binary_payloads_deterministically`.

Follow Steps A–G with `<family>` = `websocket`.

---

## Task 13: Extract `worker`

**Files:** Create `worker.rs`, `worker_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (NON-CONTIGUOUS — split by the `SharedArrayBuffer`/`Atomics` structs): `PostedItem` (≈1793, private enum), `DeterministicPostQueue` (≈1799, private struct + impl), `Worker` (≈1848, + impl); then `BroadcastChannel` (≈2092, + impl).

**Neighbors that STAY:** `SharedArrayBuffer` (≈1912) and `Atomics` (≈2023) between them STAY (→ threads); `ThreadRuntimeTopology` (≈2158) below STAYS (→ threads).

**Imports:** `Worker`/`BroadcastChannel` reference `SharedArrayBuffer` (`use crate::SharedArrayBuffer;`) and `Event`/`EventTarget` if used (`use crate::{...};`) — compiler-driven.

**Tests to move (9):** `worker_stub_records_posted_messages`, `worker_stub_records_shared_buffers_with_shared_backing`, `worker_stub_trims_surrounding_whitespace_from_script_urls`, `worker_stub_ignores_shared_buffer_posts_after_termination`, `worker_stub_preserves_interleaved_post_order`, `broadcast_channel_stub_records_posted_messages`, `broadcast_channel_stub_records_shared_buffers_with_shared_backing`, `broadcast_channel_stub_ignores_shared_buffer_posts_after_close`, `broadcast_channel_stub_preserves_interleaved_post_order`.

Follow Steps A–G with `<family>` = `worker`.

---

## Task 14: Extract `threads`

**Files:** Create `threads.rs`, `threads_tests.rs`; Modify `lib.rs`, `tests.rs`.

**Items to move** (NON-CONTIGUOUS — two regions split by `BroadcastChannel`, now removed by Task 13): `SharedArrayBuffer` (≈1912, + impl), `Atomics` (≈2023, + impl); then `ThreadRuntimeTopology` (≈2158, + impl), `ThreadRuntimeInstanceSnapshot` (≈2165, + impl), `ThreadRuntimeShutdownReport` (≈2254, + impl).

**Neighbors that STAY:** `IndexedDb` (≈2487) below STAYS (→ indexeddb).

**Imports:** `Atomics` uses `kali_common::bytewise_shared_memory_is_lock_free` → `use kali_common::bytewise_shared_memory_is_lock_free;`; shared-memory atomics use `std::sync::atomic::{...}`.

**Tests to move (11):** `thread_runtime_topology_assigns_one_instance_per_worker`, `thread_runtime_topology_keeps_monotonic_instance_ids_after_termination`, `thread_runtime_topology_snapshot_reports_live_instances_deterministically`, `thread_runtime_topology_shutdown_reports_live_instances_deterministically`, `thread_runtime_topology_shutdown_keeps_live_instances_sorted_by_id`, `thread_runtime_topology_shutdown_keeps_live_instances_sorted_after_first_termination`, `thread_runtime_topology_counts_multiple_terminated_instances_deterministically`, `atomics_reports_lock_free_status_deterministically`, `shared_array_buffer_clones_share_mutations`, `shared_array_buffer_compare_exchange_failure_leaves_bytes_unchanged`, `shared_array_buffer_supports_zero_length_buffers`.

The `threads_tests.rs` header likely needs `use kali_common::bytewise_shared_memory_is_lock_free;` and `use std::sync::atomic::AtomicUsize;` (both present in the original `tests.rs` header — move whichever these 11 tests reference).

Follow Steps A–G with `<family>` = `threads`.

---

## Task 15: Extract `indexeddb` (empties and deletes `tests.rs`)

**Files:** Create `indexeddb.rs`, `indexeddb_tests.rs`; Modify `lib.rs`; **Delete** `crates/kali_api_web/src/tests.rs`.

**Items to move** (the file tail ≈2487–end): `IndexedDb` (≈2487, struct + impl), `IndexedDB` type alias (≈2493). This is the last remaining definition block; after the move, `lib.rs` holds only the facade.

**Tests to move (2):** `indexed_db_stub_persists_values`, `indexed_db_stub_exposes_deterministic_snapshots`. After moving them, `src/tests.rs` is empty → `git rm crates/kali_api_web/src/tests.rs`.

Follow Steps A–G with `<family>` = `indexeddb`, with these additions:
- [ ] **Step C+ — Confirm `lib.rs` is a pure facade:** it now contains only the crate doc, ~14 alphabetical `mod` decls, and ~14 alphabetical `pub use <mod>::*;` globs. No `struct`/`enum`/`fn`/`static`/`const`/`type` definition remains: `grep -nE '^(pub )?(struct|enum|fn|pub fn|const|static|type) ' crates/kali_api_web/src/lib.rs` → empty.
- [ ] **Step D+ — Delete the empty tests file:** `git rm crates/kali_api_web/src/tests.rs`.
- [ ] Commit message: `refactor(kali_api_web): extract indexeddb module, reduce lib.rs to facade [refactor]`.

---

## Task 16: Final verification & fmt

**Files:** possibly a small `cargo fmt` diff across `crates/kali_api_web/src/`.

- [ ] **Step 1 — Format:** `cargo fmt -p kali_api_web`. If it produces a diff, it is style-only (e.g. a re-export list or a signature reflow); review it is behavior-neutral.
- [ ] **Step 2 — Clippy (whole crate, all targets):** `cargo clippy -p kali_api_web --all-targets -- -D warnings` → no warnings.
- [ ] **Step 3 — Tests:** `cargo test -p kali_api_web` → 57 passed.
- [ ] **Step 4 — Basename-multiset proof (final):**
  ```bash
  cargo test -p kali_api_web -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
    | diff - <(sed -E 's/^.*:://' .superpowers/sdd/kali_api_web_list_before.txt | sort)
  ```
  Expected: empty.
- [ ] **Step 5 — Workspace build:** `cargo build --workspace` → succeeds (proves no downstream consumer broke; `kali_api_web::*` paths preserved).
- [ ] **Step 6 — Facade size sanity:** `wc -l crates/kali_api_web/src/lib.rs` → ~30 lines (doc + 14 mods + 14 globs).
- [ ] **Step 7 — Commit** (only if fmt produced a diff):
  ```bash
  git add crates/kali_api_web/src/
  git commit -m "style(kali_api_web): cargo fmt [refactor]"
  ```

---

## Self-Review Notes (carried from prior cycles — apply during execution)

- **Non-contiguous extractions are the main hazard** (Tasks 6, 9, 11, 13, 14): each moves items from 2+ regions while leaving interleaved non-members behind. Always hand the implementer the item **names** + a fresh re-grep of current ranges + the explicit "neighbors that STAY" list; reviewers verify both the moved set AND the left-behind set.
- **`impl` blocks can be non-adjacent to their struct** (notably `EventTarget` in Task 11 and the `IndexedDb`/`EventTarget` impls near the file tail). Grep `^impl <Type>` and move every match with the struct.
- **Import trimming is part of each task** (Step C): every extraction that empties a name's last use in `lib.rs`'s header leaves an `unused_imports` warning — trim it in the same commit (clippy `-D warnings` enforces this).
- **Verbatim includes whitespace and order:** do not drop blank-line separators or reorder kept items (a dropped blank line was a real Minor finding in kali_common).
- **Const/private-helper surfacing:** family-private helpers (`normalize_header_name`, `canonicalize_digest_algorithm`, base64 helpers, `BASE64_ALPHABET`, the 4 statics) move *with* their family and stay private; no cross-module reference to them is expected. If one arises, `use crate::*;` surfaces crate-root privates to child modules, or qualify a bare const as `crate::<NAME>` (mechanical).
- **Final review:** whole-branch opus review (subagent-driven-development two-stage gate) before the fast-forward merge to main, matching all 10 prior crates.
