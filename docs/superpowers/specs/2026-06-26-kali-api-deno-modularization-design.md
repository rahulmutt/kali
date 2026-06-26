# kali_api_deno modularization — design

**Date:** 2026-06-26
**Crate:** `kali_api_deno` (13th in the kali crate-modularization series)
**Predecessors:** `kali_api_web` (11th) and `kali_api_node` (12th) — structural twins; this spec reuses their INDEPENDENT-OBJECT-PILE playbook and records where deno diverges.
**Execution:** subagent-driven-development, one module per task.
**Integration:** fast-forward merge to **local main only** (match crates 2–12), re-verify on merged main, delete branch. Not pushed to origin in this cycle.

## Goal

Decompose the monolithic `crates/kali_api_deno/src/lib.rs` (1014 lines) into a thin facade plus per-family modules, and split `src/tests.rs` (566 lines, 18 tests) into co-located sibling `*_tests.rs` files — with **zero behavior change** and a **preserved public API**.

## Shape: INDEPENDENT-OBJECT-PILE (with one twist)

`kali_api_deno` is ~18 deno-native self-contained API objects, each a `struct` + its own `impl`(s) + sometimes an error enum, plus family-local free fns and private helper types. There is **no shared mega-struct**, so there is **no Task-1 blanket `pub(crate)` receiver-widening**. Same shape as web/node.

The twist vs the node twin: two **private** path helpers are shared across two families, forcing a real (small) widening. See [Widening](#widening--one-real-site-prediction-breaker).

Current surface (deno-native, excluding the web re-export block): ~18 `pub` top-level items, 18 `impl` blocks, 4 private free fns (`read_http_request`, `write_http_response`, `normalize_path`, `resolve_path`).

## Architecture

`lib.rs` 1014 → **thin facade**. Unlike the node/web facades (which were pure `mod` + `glob`), deno's facade carries two things that must survive verbatim:

1. **The cross-crate web re-export block** (current lines 7–30):
   `pub use kali_api_web::{ atob, btoa, crypto, fetch, fill_random_values, local_storage, navigator, parse_url, performance_now, random_uuid, resolve_url, session_storage, structured_clone, text_decode, text_encode, AbortController, AbortSignal, Base64Error, Blob, BroadcastChannel, Crypto, CustomEvent, Event, EventTarget, File, FileReader, FileReaderState, FormData, FormDataEntry, FormDataValue, Headers, IndexedDB, IndexedDb, Navigator, ReadableStream, Request, Response, Storage, TransformStream, URLSearchParams, WebSocket, WebSocketReadyState, Worker, WritableStream, URL };`
   Deno's public surface is *web's surface plus deno-native objects* — this block stays in the facade unchanged.
2. **`deno_api_init()`** — a 2-line crate entry that delegates to `kali_api_web::web_api_init()`. Kept in the facade (it is the crate's lifecycle entry point; precedent: node allowed its `*_api_init` to stay at the root).

Then: **7** `mod` decls + **7** `pub use <mod>::*;` globs for the deno-native families, **plus one non-glob internal module** (`path`).

The glob facade preserves every `kali_api_deno::Name` flat path → zero consumer edits.

### Module decomposition (7 public families)

| module | contents | impls |
|---|---|---|
| `env` | `DenoEnv` | 1 |
| `args` | `DenoArgs` | 1 |
| `permissions` | `DenoPermissionKind`, `DenoPermissionStatus`, `DenoPermissionError` (+ `Display`/`Error` impls), `DenoPermissions` | 3 |
| `fs` | `DenoFileInfo`, `DenoFile`, `DenoFs` | 3 |
| `command` | `DenoCommandOutput`, `DenoCommandError` (+ `Display`/`Error` impls), `DenoCommand` | 3 |
| `net` | `DenoTcpConnection`, `DenoTcpListener`, `DenoHttpServer`, free fns `connect`/`listen`/`serve`, + family-local privates `read_http_request`/`write_http_response` | 4 |
| `runtime` | `DenoRuntimeProjection` | 1 |

`net`'s two private http helpers (`read_http_request`, `write_http_response`) are used only inside `serve` — they stay private to `net`. **No widening.**

### The internal `path` module (Option A)

`normalize_path` and `resolve_path` are **private** free fns (not in the public surface) but used by **two** families:

- `fs` — `DenoFs::current_dir`, `cwd`, `resolve`
- `command` — `DenoCommand::current_dir`

Resolution: a **dedicated internal `path` module**. `mod path;` holds both fns as `pub(crate)`. The facade declares `mod path;` with **no** `pub use` glob — they stay internal, nothing leaks to the public surface. `fs` and `command` both `use crate::path::{normalize_path, resolve_path}`.

Rationale: neither family owns a helper the other reaches into; the coupling is named honestly. This mirrors node's `path` module *name* but with a key difference: node's `path` was a **public** family (glob-exported); deno's `path` is **internal** (`pub(crate)`, no glob).

## Widening — one real site (prediction-breaker)

The api-twin trilogy record:

- **web (11th):** one coupled-pair `pub(crate)` widening (`Worker` ↔ `ThreadRuntimeTopology`), despite a "no-widening" prediction.
- **node (12th):** **zero** widening — prediction held exactly.
- **deno (13th):** **one real widening** — `normalize_path` and `resolve_path` promoted private → `pub(crate)` into a dedicated internal `path` module, because they are shared by `fs` and `command`.

This is predicted up front (not a surprise mid-execution) and is recorded so the twin-pattern history stays accurate. No other widening is expected: every cross-family reference among the deno-native objects goes through already-`pub` types, and `net`'s http helpers are family-local.

## No url-shadow hazard

Unlike kali_api_web (where `mod url` shadowed the extern `url` crate at the crate root), deno has **no** local `url` module and **no** `use url` / `url::` references in its own code — the URL surface is re-exported from `kali_api_web` (`parse_url`, `resolve_url`, `URL`). Nothing to guard.

## Test split

`tests.rs` (566 lines, 18 tests, single `use super::*;`) → co-located sibling `<family>_tests.rs` files, matching the established pattern:

`env_tests.rs`, `args_tests.rs`, `permissions_tests.rs`, `fs_tests.rs`, `command_tests.rs`, `net_tests.rs`, `runtime_tests.rs`.

- Each test moves next to the family it exercises; its `use super::*;` resolves against the host module.
- **Self-sufficiency rule (learned from web):** each `*_tests.rs` declares its own `use` lines. **Never** re-export test helpers into the facade to satisfy a glob.
- **tempdir placement (the node trap):** `tempfile::tempdir` is used at exactly **one** site (current `tests.rs:206`). The plan pins which test that is and routes it to the correct family's `*_tests.rs`, so the `tempfile` dev-dep stays referenced by exactly the file that needs it. **Verify by grepping the final layout**, not by trusting the map — node's plan said "all tempdir sites in fs" and one had escaped into a different family's test, forcing a cross-task fixup.

## Verification

Per-family, after each extraction task:

- `cargo build -p kali_api_deno`
- `cargo test -p kali_api_deno`

Final proof: **basename-multiset** — the multiset of item basenames (pub items + tests) before == after, proving zero behavior/surface change.

Integration: fast-forward merge to **local main only** (matching crates 2–12), re-verify on merged main, delete branch. Not pushed to origin this cycle.

## Out of scope

- No behavior changes, no public-API changes (beyond the internal `path` widening, which does not alter the public surface).
- No unrelated refactoring of the deno-native objects' internals.
- No changes to `kali_api_web` (the re-export source).
