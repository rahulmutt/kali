# kali_api_deno Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose the monolithic `crates/kali_api_deno/src/lib.rs` (1014 lines) into a thin facade plus 7 per-family modules + 1 internal `path` module, and split `src/tests.rs` (18 tests) into co-located `*_tests.rs` files — zero behavior change, preserved public API.

**Architecture:** INDEPENDENT-OBJECT-PILE (same shape as kali_api_web/kali_api_node). The facade re-exports each family via a glob (`pub use <mod>::*;`) so every `kali_api_deno::Name` flat path is preserved → zero consumer edits. The facade additionally keeps verbatim: the cross-crate `pub use kali_api_web::{...}` re-export block and `deno_api_init()`. Two private path helpers shared by `fs` and `command` move into an internal `path` module (`pub(crate)`, no glob) — the one predicted widening.

**Tech Stack:** Rust 2021, cargo workspace, `serde_json`, dep on `kali_api_web`; dev-dep `tempfile`.

## Global Constraints

- **Zero behavior change, preserved public API.** The public surface of `kali_api_deno` must be byte-for-byte identical (same names, same flat paths). The only visibility change is `normalize_path`/`resolve_path` going private → `pub(crate)` (internal, not public).
- **No changes to `kali_api_web`** (the re-export source) and no changes to any consumer crate.
- **Facade stays logic-free** except the two items that must live at the crate root: the `pub use kali_api_web::{...}` block and `deno_api_init()`.
- **Test co-location mechanics (match the node twin exactly):** each module file ends with
  ```rust
  #[cfg(test)]
  #[path = "<family>_tests.rs"]
  mod <family>_tests;
  ```
  and each `<family>_tests.rs` begins with `use crate::*;` plus any explicit std/external imports its bodies reference. **Never** re-export test helpers into the facade.
- **Per-task verification:** every task ends with `cargo build -p kali_api_deno` + `cargo test -p kali_api_deno` green, then a commit.
- Mid-plan unused-import warnings are acceptable (crate does not deny warnings); the crate-root `use std{...}` block is removed in the finalize task once empty.
- **Module extraction order constraint:** `path` (Task 1) MUST precede `fs` (Task 5) and `command` (Task 6), which depend on it.

**Source line map of current `lib.rs`** (for cut operations):

| region | lines | items |
|---|---|---|
| crate doc | 1–5 | `//!` header |
| web re-export block | 7–20 | `pub use kali_api_web::{ … };` |
| crate-root `use std{}` + serde | 21–30 | shared imports |
| `deno_api_init` | 31–35 | crate entry |
| env | 37–146 | `DenoEnv` (+impl) |
| args | 147–167 | `DenoArgs` (+impl) |
| permissions | 168–271 | `DenoPermissionKind`, `DenoPermissionStatus`, `DenoPermissionError` (+`Display`/`Error`), `DenoPermissions` |
| fs | 272–483 | `DenoFileInfo`, `DenoFile`, `DenoFs` |
| command | 484–614 | `DenoCommandOutput`, `DenoCommandError` (+`Display`/`Error`), `DenoCommand` |
| net | 615–838 | `DenoTcpConnection`, `DenoTcpListener`, `DenoHttpServer`, `connect`, `listen`, `serve`, privates `read_http_request`/`write_http_response` |
| runtime | 839–976 | `DenoRuntimeProjection` |
| path (privates) | 977–1011 | `normalize_path`, `resolve_path` |
| test wiring | 1012–1014 | `#[cfg(test)] #[path="tests.rs"] mod tests;` |

> Line numbers drift as you cut. After each task, re-locate the next region by item name (e.g. `grep -n 'pub struct DenoEnv' src/lib.rs`), not by absolute line.

---

### Task 1: Internal `path` module (the shared helper)

**Files:**
- Create: `crates/kali_api_deno/src/path.rs`
- Modify: `crates/kali_api_deno/src/lib.rs`

**Interfaces:**
- Produces: `pub(crate) fn normalize_path(path: impl AsRef<Path>) -> PathBuf` and `pub(crate) fn resolve_path(base: impl AsRef<Path>, input: impl AsRef<Path>) -> PathBuf`, importable as `use crate::path::{normalize_path, resolve_path};`. Consumed later by `fs` (Task 5) and `command` (Task 6).

- [ ] **Step 1: Create `path.rs` with the two helpers moved verbatim, marked `pub(crate)`**

Cut the bodies of `normalize_path` (currently ~977) and `resolve_path` (currently ~1003) out of `lib.rs`. New file:

```rust
//! Internal path-normalization helpers shared by the `fs` and `command` families.
//!
//! Not part of the public surface — `pub(crate)` only, intentionally not glob-exported by the facade.

use std::path::{Component, Path, PathBuf};

pub(crate) fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    // ... moved body verbatim from lib.rs, with `fn` → `pub(crate) fn`
}

pub(crate) fn resolve_path(base: impl AsRef<Path>, input: impl AsRef<Path>) -> PathBuf {
    // ... moved body verbatim from lib.rs, with `fn` → `pub(crate) fn`
}
```

- [ ] **Step 2: Wire the module into the facade and keep crate-root call sites compiling**

In `lib.rs`, add near the other module decls (top, after the `use` block):

```rust
mod path;
use crate::path::{normalize_path, resolve_path};
```

Do **not** add `pub use path::*;` — this module is internal. The `use crate::path::{...}` line keeps the still-in-`lib.rs` `fs`/`command` call sites (e.g. `normalize_path(cwd.into())`) resolving until those families extract.

- [ ] **Step 3: Build**

Run: `cargo build -p kali_api_deno`
Expected: PASS (a warning that `Component`/`Path` may now be unused in the crate-root `use std{}` block is acceptable).

- [ ] **Step 4: Test**

Run: `cargo test -p kali_api_deno`
Expected: PASS — all 18 tests green.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_api_deno/src/path.rs crates/kali_api_deno/src/lib.rs
git commit -m "refactor(kali_api_deno): extract internal path module [refactor]"
```

---

### Task 2: `env` module

**Files:**
- Create: `crates/kali_api_deno/src/env.rs`
- Create: `crates/kali_api_deno/src/env_tests.rs`
- Modify: `crates/kali_api_deno/src/lib.rs`
- Modify: `crates/kali_api_deno/src/tests.rs` (remove the 2 moved tests)

**Interfaces:**
- Produces: `pub struct DenoEnv` (+ its impl), flat path `kali_api_deno::DenoEnv` preserved via facade glob.

- [ ] **Step 1: Create `env.rs`** — move the `DenoEnv` struct + `impl DenoEnv` (currently ~37–146) verbatim. Header:

```rust
//! Deterministic environment view for the Deno compatibility layer.

use serde_json::Value;
use std::collections::BTreeMap;

// ... moved DenoEnv struct + impl verbatim ...

#[cfg(test)]
#[path = "env_tests.rs"]
mod env_tests;
```

- [ ] **Step 2: Create `env_tests.rs`** — move tests `env_view_is_deterministic_and_mutable` and `env_view_snapshot_is_sorted_and_detached_from_later_mutations` out of `tests.rs`:

```rust
use crate::*;
use std::collections::BTreeMap;

// ... the two moved #[test] fns verbatim ...
```

- [ ] **Step 3: Update facade** — in `lib.rs` add (alphabetical among module decls):

```rust
mod env;
pub use env::*;
```

- [ ] **Step 4: Build** — `cargo build -p kali_api_deno` → PASS.
- [ ] **Step 5: Test** — `cargo test -p kali_api_deno` → PASS (18 tests, unchanged count).
- [ ] **Step 6: Commit**

```bash
git add crates/kali_api_deno/src/env.rs crates/kali_api_deno/src/env_tests.rs crates/kali_api_deno/src/lib.rs crates/kali_api_deno/src/tests.rs
git commit -m "refactor(kali_api_deno): extract env module [refactor]"
```

---

### Task 3: `args` module

**Files:**
- Create: `crates/kali_api_deno/src/args.rs`
- Create: `crates/kali_api_deno/src/args_tests.rs`
- Modify: `crates/kali_api_deno/src/lib.rs`, `crates/kali_api_deno/src/tests.rs`

**Interfaces:**
- Produces: `pub struct DenoArgs(Vec<String>)` (+ impl), flat path preserved.

- [ ] **Step 1: Create `args.rs`** — move `DenoArgs` + `impl DenoArgs` (currently ~147–167). No imports from the std block are needed. Header:

```rust
//! Host argument view for the Deno compatibility layer.

// ... moved DenoArgs struct + impl verbatim ...

#[cfg(test)]
#[path = "args_tests.rs"]
mod args_tests;
```

- [ ] **Step 2: Create `args_tests.rs`** — move `args_view_round_trips_host_arguments`:

```rust
use crate::*;

// ... moved #[test] fn verbatim ...
```

- [ ] **Step 3: Update facade** — add to `lib.rs`:

```rust
mod args;
pub use args::*;
```

- [ ] **Step 4: Build** — `cargo build -p kali_api_deno` → PASS.
- [ ] **Step 5: Test** — `cargo test -p kali_api_deno` → PASS.
- [ ] **Step 6: Commit**

```bash
git add crates/kali_api_deno/src/args.rs crates/kali_api_deno/src/args_tests.rs crates/kali_api_deno/src/lib.rs crates/kali_api_deno/src/tests.rs
git commit -m "refactor(kali_api_deno): extract args module [refactor]"
```

---

### Task 4: `permissions` module

**Files:**
- Create: `crates/kali_api_deno/src/permissions.rs`
- Create: `crates/kali_api_deno/src/permissions_tests.rs`
- Modify: `crates/kali_api_deno/src/lib.rs`, `crates/kali_api_deno/src/tests.rs`

**Interfaces:**
- Produces: `pub enum DenoPermissionKind`, `pub enum DenoPermissionStatus`, `pub struct DenoPermissionError` (+ `Display`/`Error` impls), `pub struct DenoPermissions` (+ impl). Flat paths preserved.

- [ ] **Step 1: Create `permissions.rs`** — move the whole permissions region (currently ~168–271): both enums, the error struct with its `impl`, the `impl std::fmt::Display`, `impl std::error::Error`, and `DenoPermissions` with its impl — verbatim. The `Display` impl already uses fully-qualified `std::fmt::*`, so no `use` from the std block is required. Header:

```rust
//! Deterministic permission model for the Deno compatibility layer.

// ... moved enums + error type + DenoPermissions verbatim ...

#[cfg(test)]
#[path = "permissions_tests.rs"]
mod permissions_tests;
```

- [ ] **Step 2: Create `permissions_tests.rs`** — move `permissions_query_reports_granted_and_denied`:

```rust
use crate::*;

// ... moved #[test] fn verbatim ...
```

- [ ] **Step 3: Update facade** — add to `lib.rs`:

```rust
mod permissions;
pub use permissions::*;
```

- [ ] **Step 4: Build** — `cargo build -p kali_api_deno` → PASS.
- [ ] **Step 5: Test** — `cargo test -p kali_api_deno` → PASS.
- [ ] **Step 6: Commit**

```bash
git add crates/kali_api_deno/src/permissions.rs crates/kali_api_deno/src/permissions_tests.rs crates/kali_api_deno/src/lib.rs crates/kali_api_deno/src/tests.rs
git commit -m "refactor(kali_api_deno): extract permissions module [refactor]"
```

---

### Task 5: `fs` module (consumes `crate::path`)

**Files:**
- Create: `crates/kali_api_deno/src/fs.rs`
- Create: `crates/kali_api_deno/src/fs_tests.rs`
- Modify: `crates/kali_api_deno/src/lib.rs`, `crates/kali_api_deno/src/tests.rs`

**Interfaces:**
- Consumes: `use crate::path::{normalize_path, resolve_path};` (from Task 1).
- Produces: `pub struct DenoFileInfo`, `pub struct DenoFile`, `pub struct DenoFs` (+ impls). Flat paths preserved.

- [ ] **Step 1: Create `fs.rs`** — move `DenoFileInfo`/`DenoFile`/`DenoFs` (+ impls, currently ~272–483) verbatim. Header (note the io traits, needed for `read_to_string`/`read_to_end`/`write_all`, and the path helper import):

```rust
//! Deterministic filesystem view for the Deno compatibility layer.

use std::fs::{self, File as StdFile, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::path::{normalize_path, resolve_path};

// ... moved DenoFileInfo / DenoFile / DenoFs verbatim ...

#[cfg(test)]
#[path = "fs_tests.rs"]
mod fs_tests;
```

- [ ] **Step 2: Create `fs_tests.rs`** — move `filesystem_round_trips_files_and_metadata` (the **only** `tempfile::tempdir` user). This is the tempdir-trap site flagged in the spec:

```rust
use crate::*;
use tempfile::tempdir;

// ... moved #[test] fn verbatim ...
```

- [ ] **Step 3: Update facade** — add to `lib.rs`:

```rust
mod fs;
pub use fs::*;
```

- [ ] **Step 4: Build** — `cargo build -p kali_api_deno` → PASS. If the build reports a missing/extra io trait, add/remove from the `use std::io::{...}` line accordingly.
- [ ] **Step 5: Test** — `cargo test -p kali_api_deno` → PASS.
- [ ] **Step 6: Verify tempdir landed only here**

Run: `grep -rn "tempdir\|tempfile" crates/kali_api_deno/src`
Expected: matches appear **only** in `fs_tests.rs` (the `use tempfile::tempdir;` and its call). If any other file matches, move that usage into the correct family's `*_tests.rs` before committing — do not trust the line map, trust this grep.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_api_deno/src/fs.rs crates/kali_api_deno/src/fs_tests.rs crates/kali_api_deno/src/lib.rs crates/kali_api_deno/src/tests.rs
git commit -m "refactor(kali_api_deno): extract fs module [refactor]"
```

---

### Task 6: `command` module (consumes `crate::path`)

**Files:**
- Create: `crates/kali_api_deno/src/command.rs`
- Create: `crates/kali_api_deno/src/command_tests.rs`
- Modify: `crates/kali_api_deno/src/lib.rs`, `crates/kali_api_deno/src/tests.rs`

**Interfaces:**
- Consumes: `use crate::path::normalize_path;` (from Task 1).
- Produces: `pub struct DenoCommandOutput`, `pub struct DenoCommandError` (+ `Display`/`Error`), `pub struct DenoCommand` (+ impls). Flat paths preserved.

- [ ] **Step 1: Create `command.rs`** — move the command region (currently ~484–614) verbatim. Header:

```rust
//! Deterministic subprocess command model for the Deno compatibility layer.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::path::normalize_path;

// ... moved DenoCommandOutput / DenoCommandError (+Display/Error) / DenoCommand verbatim ...

#[cfg(test)]
#[path = "command_tests.rs"]
mod command_tests;
```

- [ ] **Step 2: Create `command_tests.rs`** — move `command_helper_runs_child_process_and_captures_output`:

```rust
use crate::*;

// ... moved #[test] fn verbatim ...
```

- [ ] **Step 3: Update facade** — add to `lib.rs`:

```rust
mod command;
pub use command::*;
```

- [ ] **Step 4: Build** — `cargo build -p kali_api_deno` → PASS.
- [ ] **Step 5: Test** — `cargo test -p kali_api_deno` → PASS.
- [ ] **Step 6: Commit**

```bash
git add crates/kali_api_deno/src/command.rs crates/kali_api_deno/src/command_tests.rs crates/kali_api_deno/src/lib.rs crates/kali_api_deno/src/tests.rs
git commit -m "refactor(kali_api_deno): extract command module [refactor]"
```

---

### Task 7: `net` module

**Files:**
- Create: `crates/kali_api_deno/src/net.rs`
- Create: `crates/kali_api_deno/src/net_tests.rs`
- Modify: `crates/kali_api_deno/src/lib.rs`, `crates/kali_api_deno/src/tests.rs`

**Interfaces:**
- Consumes: `Headers`, `Request`, `Response` from `kali_api_web`.
- Produces: `pub struct DenoTcpConnection`, `pub struct DenoTcpListener`, `pub struct DenoHttpServer`, `pub fn connect`, `pub fn listen`, `pub fn serve`. Private `read_http_request`/`write_http_response` stay inside this module (family-local, **no widening**). Flat paths preserved.

- [ ] **Step 1: Create `net.rs`** — move the whole net region (currently ~615–838): the three structs + impls, `connect`, `listen`, the private `read_http_request`/`write_http_response`, and `serve` — verbatim. The two http helpers remain **private** `fn` (do not change visibility). Header:

```rust
//! Deterministic TCP/HTTP server surface for the Deno compatibility layer.

use std::io::{BufReader, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use kali_api_web::{Headers, Request, Response};

// ... moved structs/impls + connect/listen + private read_http_request/write_http_response + serve verbatim ...

#[cfg(test)]
#[path = "net_tests.rs"]
mod net_tests;
```

- [ ] **Step 2: Create `net_tests.rs`** — move `tcp_connect_and_listen_round_trip_bytes` and `serve_emits_a_basic_http_response`:

```rust
use crate::*;

// ... the two moved #[test] fns verbatim ...
```

- [ ] **Step 3: Update facade** — add to `lib.rs`:

```rust
mod net;
pub use net::*;
```

- [ ] **Step 4: Build** — `cargo build -p kali_api_deno` → PASS. Adjust the `use std::io::{...}`/`use std::net::{...}` lines if the compiler reports an unused/missing name.
- [ ] **Step 5: Test** — `cargo test -p kali_api_deno` → PASS.
- [ ] **Step 6: Commit**

```bash
git add crates/kali_api_deno/src/net.rs crates/kali_api_deno/src/net_tests.rs crates/kali_api_deno/src/lib.rs crates/kali_api_deno/src/tests.rs
git commit -m "refactor(kali_api_deno): extract net module [refactor]"
```

---

### Task 8: `runtime` module

**Files:**
- Create: `crates/kali_api_deno/src/runtime.rs`
- Create: `crates/kali_api_deno/src/runtime_tests.rs`
- Modify: `crates/kali_api_deno/src/lib.rs`, `crates/kali_api_deno/src/tests.rs`

**Interfaces:**
- Consumes: sibling families `DenoArgs`, `DenoEnv`, `DenoFs`, `DenoPermissions` (via `use crate::{...}`).
- Produces: `pub struct DenoRuntimeProjection` (+ impl). Flat path preserved.

- [ ] **Step 1: Create `runtime.rs`** — move `DenoRuntimeProjection` + impl (currently ~839–976) verbatim. Header:

```rust
//! Aggregated runtime projection bundling the Deno baseline context.

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::{DenoArgs, DenoEnv, DenoFs, DenoPermissions};

// ... moved DenoRuntimeProjection struct + impl verbatim ...

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;
```

- [ ] **Step 2: Create `runtime_tests.rs`** — move `runtime_projection_bundles_baseline_context` and `runtime_projection_new_defaults_to_open_permissions_and_empty_views`:

```rust
use crate::*;
use std::collections::BTreeMap;

// ... the two moved #[test] fns verbatim ...
```

(If a body references `PathBuf` or other std names, add the explicit `use` — let `cargo build` confirm.)

- [ ] **Step 3: Update facade** — add to `lib.rs`:

```rust
mod runtime;
pub use runtime::*;
```

- [ ] **Step 4: Build** — `cargo build -p kali_api_deno` → PASS.
- [ ] **Step 5: Test** — `cargo test -p kali_api_deno` → PASS.
- [ ] **Step 6: Commit**

```bash
git add crates/kali_api_deno/src/runtime.rs crates/kali_api_deno/src/runtime_tests.rs crates/kali_api_deno/src/lib.rs crates/kali_api_deno/src/tests.rs
git commit -m "refactor(kali_api_deno): extract runtime module [refactor]"
```

---

### Task 9: Facade finalize + re-export tests

At this point `lib.rs` should contain only: crate doc, the `pub use kali_api_web::{...}` block, the now-unused crate-root `use std{...}` + `use serde_json::Value` + `use crate::path::{...}` lines, `deno_api_init()`, the `mod`/`pub use` decls (7 globs + internal `mod path;`), and the old `#[cfg(test)] mod tests;` wiring. `tests.rs` should hold only the 8 web-re-export/init tests.

**Files:**
- Modify: `crates/kali_api_deno/src/lib.rs`
- Rename: `crates/kali_api_deno/src/tests.rs` → `crates/kali_api_deno/src/reexport_tests.rs`

**Interfaces:**
- Produces: a logic-free facade. No new public items.

- [ ] **Step 1: Remove the dead crate-root imports** from `lib.rs` — delete the entire `use std::{ … };` block, the `use serde_json::Value;` line, and the `use crate::path::{normalize_path, resolve_path};` line (all consumers have moved into modules). Keep the `pub use kali_api_web::{...}` block and `deno_api_init()`.

- [ ] **Step 2: Rename the residual test file** to reflect its facade-level scope:

```bash
git mv crates/kali_api_deno/src/tests.rs crates/kali_api_deno/src/reexport_tests.rs
```

It should now contain exactly these 8 tests: `navigator_baseline_is_reexported`, `random_uuid_is_reexported`, `crypto_facade_is_reexported`, `initialization_drags_in_shared_web_baseline`, `web_file_reader_is_reexported_through_the_deno_surface`, `form_data_is_reexported_through_the_deno_surface`, `browser_url_is_reexported_through_the_deno_surface`, `browser_stubs_are_reexported_through_the_deno_surface`. Set its first line to `use crate::*;` (these reference web-re-exported names, all covered by the facade glob).

- [ ] **Step 3: Update the test wiring** at the bottom of `lib.rs`:

```rust
#[cfg(test)]
#[path = "reexport_tests.rs"]
mod reexport_tests;
```

- [ ] **Step 4: Confirm the final facade shape** — `lib.rs` should read approximately:

```rust
//! Deno API compatibility surface for Kali runtime.
//! ...(existing doc retained)...

pub use kali_api_web::{ /* ...unchanged block... */ };

/// Initialize the Deno API compatibility surface.
pub fn deno_api_init() {
    kali_api_web::web_api_init();
}

mod args;
pub use args::*;
mod command;
pub use command::*;
mod env;
pub use env::*;
mod fs;
pub use fs::*;
mod net;
pub use net::*;
mod path; // internal — no glob re-export
mod permissions;
pub use permissions::*;
mod runtime;
pub use runtime::*;

#[cfg(test)]
#[path = "reexport_tests.rs"]
mod reexport_tests;
```

- [ ] **Step 5: Build** — `cargo build -p kali_api_deno` → PASS, with **no warnings** (all dead imports removed).

Run: `cargo build -p kali_api_deno 2>&1 | grep -i warning || echo "no warnings"`
Expected: `no warnings`

- [ ] **Step 6: Test** — `cargo test -p kali_api_deno` → PASS, 18 tests.
- [ ] **Step 7: Commit**

```bash
git add crates/kali_api_deno/src/lib.rs crates/kali_api_deno/src/reexport_tests.rs
git commit -m "refactor(kali_api_deno): finalize facade, co-locate re-export tests [refactor]"
```

---

### Task 10: Final verification + integration

**Files:** none modified (verification + merge only).

- [ ] **Step 1: Basename-multiset proof (zero surface/behavior change)**

Compare the multiset of public-item + test basenames before vs after. Against the pre-refactor commit and the working tree:

```bash
cd /workspace
# public items (structs/enums/pub fns) + test fn names, sorted with counts
collect() {
  git grep -hE '^\s*(pub (fn|struct|enum)|fn [a-z_]+\(\)|impl )' "$1" -- 'crates/kali_api_deno/src/*.rs' \
    | grep -oE '(Deno[A-Za-z]+|deno_api_init|connect|listen|serve|normalize_path|resolve_path|[a-z_]+(?=\(\)))' \
    | sort | uniq -c
}
```

Practical form: extract the set of `pub` item names + `#[test]` fn names from the first refactor commit's `lib.rs`+`tests.rs` and from the current module tree; assert the two **sorted multisets are identical**. The 18 test names and the full public item list (16 Deno* types + `connect`/`listen`/`serve`/`deno_api_init`) must match exactly. `normalize_path`/`resolve_path` move from public-file-private to `pub(crate)` — they were never public, so they are absent from the *public* set on both sides (verify they do NOT appear as `pub`).

- [ ] **Step 2: Full workspace build + test (no downstream breakage)**

Run: `cargo build && cargo test -p kali_api_deno -p kali_api_web`
Expected: PASS. (kali_api_web included to confirm the re-export source is untouched and consumers still resolve.)

Also confirm no consumer crate references broke:

Run: `cargo build -p kali_runtime -p kali_cli 2>&1 | tail -5`
Expected: PASS (these transitively use the api surface).

- [ ] **Step 3: File-size sanity** — confirm the decomposition actually happened:

Run: `wc -l crates/kali_api_deno/src/*.rs | sort -n`
Expected: `lib.rs` is now a thin facade (~40 lines); 8 module files + 8 test files present; no single file dominates.

- [ ] **Step 4: Integration — merge to local main**

This crate's series merges to **local main only** (matching crates 2–12), not pushed to origin this cycle.

```bash
cd /workspace
git checkout main
git merge --ff-only <feature-branch>   # fast-forward only
cargo test -p kali_api_deno            # re-verify on merged main
git branch -d <feature-branch>
```

Expected: fast-forward succeeds, tests green on merged main, branch deleted.

- [ ] **Step 5: Update memory** — append the `kali_api_deno` outcome to the modularization memory (13th crate; INDEPENDENT-OBJECT-PILE; the **one predicted widening** that materialized — `normalize_path`/`resolve_path` → `pub(crate)` in internal `path` module; no url-shadow; tempdir single-site verified by grep, not map; facade keeps cross-crate web re-export block + `deno_api_init`; residual facade tests renamed `reexport_tests.rs`).

---

## Self-Review

**Spec coverage:**
- Facade keeps web re-export block + `deno_api_init` → Task 1 (wiring) / Task 9 (finalize). ✓
- 7 public families (env, args, permissions, fs, command, net, runtime) → Tasks 2–8. ✓
- Internal `path` module, `pub(crate)`, no glob → Task 1 + Task 9 facade shape. ✓
- One predicted widening recorded → Task 10 Step 5 memory. ✓
- No url-shadow hazard → confirmed in spec; nothing to do (no `url` module). ✓
- Test split into co-located `*_tests.rs` + self-sufficiency rule → each family task Step 2; reexport tests → Task 9. ✓
- tempdir single-site trap (grep the final layout) → Task 5 Step 6. ✓
- basename-multiset proof + local-main integration → Task 10. ✓

**Placeholder scan:** Module bodies are *moved verbatim* from exact line ranges (a precise mechanical op, not a placeholder); every new-file header, facade edit, and command is shown concretely. Import lists are derived from actual symbol usage with a build-fix fallback. No TBD/TODO. ✓

**Type consistency:** `normalize_path`/`resolve_path` signatures identical across Task 1 (produce), Task 5/6 (consume). `DenoArgs/DenoEnv/DenoFs/DenoPermissions` names consistent in Task 8 consume block. `Headers/Request/Response` from `kali_api_web` in Task 7. Test fn names match the 18 enumerated in the source. ✓
