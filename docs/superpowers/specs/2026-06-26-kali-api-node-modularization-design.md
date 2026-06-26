# kali_api_node modularization — design

**Date:** 2026-06-26
**Crate:** `kali_api_node` (12th in the kali crate-modularization series)
**Predecessor:** `kali_api_web` (11th) — structural twin; this spec reuses its INDEPENDENT-OBJECT-PILE playbook.
**Execution:** subagent-driven-development, one module per task.
**Integration:** fast-forward merge to **local main only** (match crates 2–10), re-verify on merged main, delete branch. Not pushed to origin in this cycle.

## Goal

Decompose the monolithic `crates/kali_api_node/src/lib.rs` (1511 lines) into a thin facade plus per-family modules, and split `src/tests.rs` (572 lines, 16 tests) into co-located sibling `*_tests.rs` files — with **zero behavior change** and a **preserved public API**.

## Shape: INDEPENDENT-OBJECT-PILE

`kali_api_node` is ~40 self-contained Node API objects, each a `struct` + its own `impl`(s) + sometimes an error enum, plus family-local free fns and private helper types. There is **no shared mega-struct**, so there is **no Task-1 blanket `pub(crate)` receiver-widening** (cross-family references go through already-`pub` types). Same shape as kali_api_web.

Current surface: 39 `pub` top-level items, 30 `impl` blocks, 6 private top-level items (all family-local, see Widening).

## Architecture

`lib.rs` 1511 → **pure facade** (~30 lines): crate doc + 14 alphabetical `mod` decls + 14 alphabetical `pub use <mod>::*;` globs. No logic, no private re-exports.

The glob facade preserves every `kali_api_node::Name` flat path → zero consumer edits.

### Module decomposition (14 families)

| module | contents |
|---|---|
| `assert` | `NodeAssert`, `assert_true` |
| `buffer` | `NodeBuffer`, `hex_digit` (priv, family-local) |
| `child_process` | `NodeChildProcess`, `NodeChildProcessOutput`, `NodeChildProcessError` |
| `crypto` | `sha256_hex`, `NodeCryptoError`, `NodeDigestAlgorithm` (priv), `NodeCrypto`, `random_bytes`, `random_uuid_v4` |
| `events` | `NodeEvent`, `Listener`/`ListenerMap` (priv type aliases), `EventEmitter` |
| `fs` | `NodeFs`, `NodeFsPromises`, `NodeFsMetadata` |
| `http` | `NodeHttpError`, `NodeHttp`, `NodeHttpResponse` |
| `os` | `NodeOs` |
| `path` | `normalize_path`/`join_path`/`resolve_path`/`relative_path`/`dirname`/`basename`/`extname`, `resolve_node_path` + `path_root_key` (priv), `NodePath` |
| `process` | `NodeProcess` |
| `runtime` | `NodeRuntimeProjection` |
| `stream` | `NodeStream` |
| `url` | `parse_url`, `resolve_url`, `NodeUrl` |
| `util` | `util_format`, `util_inspect`, `NodeUtil`, `util_promisify` |

`node_api_init()` (empty crate-entry fn): folded into the `process` module (or kept in the facade if cleaner) — decided in the plan; behavior-neutral either way.

### `url` module — no external-crate shadow hazard

Unlike kali_api_web (where `mod url` shadowed the extern `url` crate at crate root and forced a `::url::` alias for code that stayed in lib.rs), here the only `url::`-referencing code (`parse_url`, `resolve_url`, `NodeUrl`) **moves into** `url.rs`, where `use url::Url` / `url::ParseError` still resolve to the external crate (a child module's bare `use url::` is not self-shadowed). The facade glob `pub use url::*;` resolves to the local module. **Guard:** if any mid-split intermediate state leaves `url::`-referencing code in lib.rs while `mod url;` is declared, apply the `::url::` absolute path at those sites until they move (expected: not needed, since the whole `url` family extracts in one task).

## Widening — predicted NONE

All 6 private top-level items are used **only within their own family** (verified by call-site grep):

- `resolve_node_path`, `path_root_key` → used only by `relative_path` (path family)
- `NodeDigestAlgorithm` → used only by `NodeCrypto` (crypto family)
- `hex_digit` → used only by `NodeBuffer` (buffer family)
- `Listener` / `ListenerMap` → used only by `EventEmitter` (events family)

They stay **private** inside their family module. No `pub(crate)` widening anticipated.

**Escape hatch (kali_api_web Worker↔threads precedent):** each extraction task verifies its moved private items have no surviving cross-family caller. If one does, widen *that one item* private→`pub(crate)` minimally (never `pub`; a `pub(crate)` item under a `pub use *` glob stays crate-internal) and record the exact coupling in the commit. Expected to fire zero times.

## Fixture-adoption — CONVERT, hold count at 16

The crate has real fs/tempdir tests (3 `tempdir()` sites in the fs / fs-promises tests). Matching the kali_runtime and kali_npm precedent:

- Adopt the `kali_test_support` dev-dep; convert `tempdir()` → `fixtures::tempdir()` and UTF-8 `fs::write` → `fixtures::write_file` / `write_manifest` where the `&str` content signature matches.
- Leave non-matching sites as-is (binary `Vec<u8>` writes, `NamedTempFile`, etc.).
- **No added fixture test** — the suite stays exactly **16** (identical-set invariant, not the codegen/optimize "add one" variant).
- If total adoption leaves the `tempfile` dev-dep unused crate-wide, **remove it** and commit the resulting `Cargo.lock` delta together with the `Cargo.toml` change (the kali_npm lesson: a dep change must include the root `Cargo.lock`).

## Test split

`tests.rs` (572 lines, 16 tests) → up to 14 sibling `*_tests.rs` files by family. Most families carry 1 test; `runtime` carries 3. **Two tests span two families** (`os_and_url_helpers_*`, `buffer_and_util_helpers_*`) — each such test is assigned verbatim to **one** family file (the plan picks which, e.g. the first-named family; the verbatim body is unchanged regardless), so the file count is ≤14 and a family without its own test gets no `*_tests.rs`. Each test file is wired at the tail of its source module:

```rust
#[cfg(test)]
#[path = "<family>_tests.rs"]
mod <family>_tests;
```

Test file headers use `use crate::*;` and are made **self-sufficient from the start**: add explicit `use` for any bare external name (`serde_json::Value`, `std::sync::Arc`, etc.) rather than freeloading on lib.rs's private imports through the glob. **Never** re-export into the facade to feed a test glob (the kali_api_web Task-15 facade-purity defect). `tests.rs` is deleted at the final test-split task (git renders the last one as a rename).

## Invariants (replicate the pilot)

- **Zero behavior change**; moves are **byte-for-byte verbatim**, including whitespace and relative item order (blank-line drops count as violations).
- Green at **every commit**; `cargo clippy -p kali_api_node --all-targets -- -D warnings` clean at every commit (run clippy, not just `cargo test`, in each task's verification — the kali_runtime lesson).
- **Public API preserved**: exactly the 39 `pub` items at flat `kali_api_node::Name` paths (verify with a probe compile or `--list`-style item inventory before/after).
- **Test-set proof = basename-multiset:** `cargo test -p kali_api_node -- --list` includes module-path prefixes after co-location, so prove the identical set by stripping prefixes, `sort` **without** `-u` (preserve any legit duplicate basenames), and `diff` → empty. Re-run at the test-split milestone and at final.
- `cargo fmt` as the final task; commit any behavior-neutral reflow as `style(...): cargo fmt`.

## Process flow per task

One module per task (subagent-driven-development). Each task: extract the family's items verbatim into `<family>.rs`, add `mod <family>; pub use <family>::*;` to lib.rs, trim any now-unused lib.rs imports, verify build + clippy + tests green. Non-contiguous extractions (items interleaved in the source) get item **names** + freshly-grepped current line ranges + an explicit "neighbors that STAY" list in the dispatch (the kali_mir / kali_api_web hazard). Controller pre-scouts current line numbers right before dispatching non-contiguous tasks.

## Out of scope

- Cracking any large method/fn into helpers (none here approach the codegen/runtime mega-fn size; all move intact).
- Touching any other crate.
- Pushing to origin/main (deferred; local main only this cycle).
