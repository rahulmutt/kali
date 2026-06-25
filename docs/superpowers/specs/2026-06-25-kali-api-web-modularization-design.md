# Kali `kali_api_web` Modularization — Design

**Date:** 2026-06-25
**Status:** Approved (design)
**Scope:** Apply the validated crate-modularization pattern to `kali_api_web` (rollout iteration 11).
**Parent design:** [`2026-06-23-kali-crate-modularization-design.md`](./2026-06-23-kali-crate-modularization-design.md)

## Problem

`kali_api_web/src/lib.rs` is a single 2,682-line file holding the entire Web API
compatibility surface for the Kali runtime — roughly 40 host-API objects (streams,
blob/file, fetch, crypto, events, workers, shared-memory threads, IndexedDB, …)
plus ~25 free functions, statics, and consts. Tests live in a sibling `tests.rs`
(57 tests). It is the largest still-monolithic crate in the workspace and has no
internal structure.

This is the next crate in the parent design's rollout order; 10 crates are already
done and merged (types, codegen, runtime, optimize, common, npm, parser, hir, mir,
ast).

## Goal & Hard Constraints

Inherited verbatim from the parent design — **pure structural refactor, zero
behavior change.**

- The exact same set of tests exists and passes before and after.
- `lib.rs` becomes a thin **facade** (`mod` declarations + `pub use` re-exports).
  Every external path (`kali_api_web::ReadableStream`, `kali_api_web::fetch`,
  `kali_api_web::btoa`, …) keeps resolving. No public API churn.
- Items move **byte-for-byte verbatim** (whitespace, blank-line separators, and
  relative source order included).
- Unit tests live in **sibling `*_tests.rs` files wired via `#[path = "…"] mod`**
  (AGENTS.md convention), not inline `#[cfg(test)]` modules.

### Proof obligation

```
cargo test -p kali_api_web -- --list   # snapshot test names → basename-multiset diff after refactor
cargo test -p kali_api_web             # must stay green at every commit
cargo clippy -p kali_api_web --all-targets -- -D warnings   # warning-free at every commit
```

`--list` includes module-path prefixes, which change as tests relocate into sibling
modules. Prove the identical-set invariant by **stripping prefixes and comparing
BASENAMES** (sort *without* `-u` to preserve any legitimate duplicate names), not by
a raw diff. This is the same basename-multiset proof used by every prior crate.

## Source Decomposition Strategy

### Shape — independent-object pile (a new sub-variant)

Unlike the impl-split crates done so far (one giant `impl` carved by responsibility:
parser, hir, mir-analysis, codegen) and unlike the pure-data `kali_ast` (derives
only, no impls), `kali_api_web` is ~40 **self-contained units**: each Web-API object
is a `struct` + its own `impl` block + (often) an error enum, with no shared mega-
struct. Cross-family references are by public type and are already `pub`.

Consequently **no Task-1 blanket `pub(crate)` receiver-widening is required** — this
crate is closest to `kali_ast`'s "relocate verbatim, no widening," except each object
carries its own `impl` along with it. The ~25 free fns / statics / consts each belong
to exactly one family and move with it.

### Target layout (`crates/kali_api_web/src/`) — ~13 flat modules

```
lib.rs        facade: crate docs + alphabetical `mod` decls + alphabetical `pub use <mod>::*` globs
util.rs       web_api_init, structured_clone, text_encode, text_decode, performance_now (+ TIME_ORIGIN)
streams.rs    DeterministicStreamState, ReadableStream, WritableStream, TransformStream,
              TextEncoderStream, TextDecoderStream
file.rs       Blob, File, FileReader (+ FileReaderState), FormData (+ FormDataValue, FormDataEntry)
storage.rs    Storage, local_storage, session_storage (+ LOCAL_STORAGE, SESSION_STORAGE statics)
navigator.rs  Navigator, navigator (+ NAVIGATOR static)
url.rs        UrlMutationError, URL, URLSearchParams, parse_url, resolve_url
fetch.rs      Headers, Request, Response, fetch, normalize_header_name
base64.rs     Base64Error, btoa, atob, encode_base64, decode_base64, decode_base64_value, BASE64_ALPHABET
crypto.rs     WebCryptoError, Crypto, SubtleCrypto, crypto, canonicalize_digest_algorithm,
              fill_random_values, random_uuid
events.rs     Event, CustomEvent, EventTarget, AbortSignal, AbortController,
              RegisteredEventListener + EventListener* / ListenerMap type aliases
websocket.rs  WebSocket, WebSocketReadyState
worker.rs     Worker, BroadcastChannel, PostedItem, DeterministicPostQueue
threads.rs    SharedArrayBuffer, Atomics, ThreadRuntimeTopology,
              ThreadRuntimeInstanceSnapshot, ThreadRuntimeShutdownReport
indexeddb.rs  IndexedDb, IndexedDB (type alias)
```

Tiny families are merged into their natural neighbor: `performance_now` → `util`,
`formdata` → `file`, `abort` → `events`, `BroadcastChannel` → `worker`. The exact
placement of any individual item is settled during implementation; the structure
above is the target shape, not a frozen file-by-file contract.

### Cross-module references

Families reference each other by public type — each module resolves siblings via
`use crate::{…}` at its head (named-import ethos of the prior crates). Notable edges:

- `fetch.rs` → `Headers`/`Request`/`Response` are mutually in-family; `Request`/
  `Response` reference `Blob`/`FormData` (`file.rs`) and `ReadableStream`
  (`streams.rs`).
- `streams.rs` `TextEncoder/DecoderStream` reference the encoding helpers in
  `util.rs`.
- `worker.rs` references `Event`/`EventTarget` (`events.rs`).
- `events.rs` `AbortSignal` references `Event`.

### Facade re-export

`lib.rs` re-exports each module with a glob:

```rust
pub use base64::*;
pub use crypto::*;
pub use events::*;
pub use fetch::*;
pub use file::*;
pub use indexeddb::*;
pub use navigator::*;
pub use storage::*;
pub use streams::*;
pub use threads::*;
pub use url::*;
pub use util::*;
pub use websocket::*;
pub use worker::*;
```

Glob re-export (rather than enumerating ~40 types + ~25 fns) guarantees every
existing `kali_api_web::Name` path keeps resolving with zero public-API churn.

## Hazards & Rules (carried from prior cycles)

- **Glob rule (clippy `-D warnings`):** a module that exposes nothing re-exportable
  gets `mod X;` *without* `pub use X::*;` — a dead glob hard-fails clippy. All listed
  modules export public types, so all keep the glob; verify at the end and delete any
  dead glob rather than `#[allow]`-ing it. Conversely, a module owning only
  `pub(crate)` items uses `pub(crate) use X::*;` (none expected here).
- **Const / static / private-helper surfacing:** module-private helpers called only
  within their family (`normalize_header_name`, `canonicalize_digest_algorithm`,
  `encode_base64`/`decode_base64`/`decode_base64_value`, `BASE64_ALPHABET`, the four
  `OnceLock` statics) move with that family and stay private. If a later-extracted
  sibling references a crate-root private, `use crate::*;` surfaces it; if a bare
  const does not surface, qualify at the use site as `crate::<NAME>` (mechanical, no
  logic change) — same fallback used by codegen/runtime/npm.
- **Import trimming:** each extraction that empties a name's last use in `lib.rs` may
  leave an unused import (warning) — trim it in the same commit (compiler-driven),
  matching the mir/hir DONE_WITH_CONCERNS pattern.
- **Verbatim incl. whitespace & order:** byte-for-byte means blank-line separators
  and relative source order too; reviewers check both the moved set and the
  left-behind set on each non-contiguous extraction.

## Test Decomposition

The 57 tests in `src/tests.rs` move verbatim into the sibling `*_tests.rs` of the
module each exercises (e.g. stream tests → `streams_tests.rs`, base64 tests →
`base64_tests.rs`, crypto tests → `crypto_tests.rs`). Each destination module is
wired with:

```rust
#[cfg(test)]
#[path = "streams_tests.rs"]
mod streams_tests;
```

`use super::*;` headers become `use crate::*;`; precise per-file imports of
`kali_common`/external names are added as needed. The original `src/tests.rs` is
deleted once empty.

**`kali_test_support` adoption — expected SKIP.** The Web API surface is in-memory
and deterministic (streams, blobs, base64, crypto digests over byte buffers); a scan
is unlikely to find real `tempdir`/filesystem test sites. If so, skip the dev-dep
entirely and hold the suite strictly at 57 (identical-set, no added fixture test) —
the foundational-crate resolution used by `kali_common` and `kali_ast`. The final
decision is made against the file's actual fs usage during implementation and
recorded in the plan.

## Execution & Verification Rhythm

Small, reviewable commits; `cargo test -p kali_api_web` green and clippy
`-D warnings` clean after each:

1. Capture the `cargo test -p kali_api_web -- --list` baseline (57 tests) into a
   durable path under `.superpowers/sdd/`.
2. Extract source modules one family at a time behind the facade, keeping `lib.rs`
   re-exporting throughout (compiles every commit). Hand implementers item **names**
   plus freshly-grepped current line ranges (numbers shift after each removal), an
   explicit "these neighbors STAY" list for non-contiguous extractions, and the
   "preserve original qualification style + whitespace + order" instruction.
3. Relocate the 57 tests into their matching sibling `*_tests.rs`; delete the emptied
   `tests.rs`. Re-run the basename-multiset proof at this milestone.
4. Final check: basename-multiset diff empty, `cargo fmt`, `cargo clippy
   -p kali_api_web --all-targets -- -D warnings`, `cargo build --workspace`,
   `cargo test -p kali_api_web` all green.

Subagent-driven-development; whole-branch opus review before the fast-forward merge
to main. This crate reuses the parent design directly; the only novelty is the
independent-object-pile shape (no receiver-widening), a simpler variant than the
impl-split crates.
