# Incremental Cache Compiler Identity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the incremental cache key name the compiler build that produced the artifact, so a cached wasm file can never be served to a compiler that would no longer produce it.

**Architecture:** A `OnceLock`-memoized fingerprint of the running executable (`current_exe` path, length, mtime) is folded into the cache key. When it cannot be computed the cache is disabled outright rather than falling back to a non-discriminating value. A new `incrementalCache: false` manifest field lets a project decline the cache, which is how the test fixtures stop depending on a machine-local one. An age/size reaper bounds the directory, because per-build namespacing makes its growth unbounded.

**Tech Stack:** Rust 1.98.1, `sha2` (already a `kali_cli` dependency), `tempfile` (already a `kali_cli` dev-dependency), `serde_json` (already a dependency). **No new dependencies.**

**Spec:** `docs/superpowers/specs/2026-09-09-incremental-cache-compiler-identity-design.md`

**Branch:** `incremental-cache-compiler-identity` (already created; spec committed at `9a61b91b50` and `8432133fd3`)

## Global Constraints

- **Rust unit tests go in sibling `*tests.rs` files, never inline `#[cfg(test)]` modules** (`AGENTS.md` §5). Wire them from the module file with `#[cfg(test)] #[path = "x_tests.rs"] mod x_tests;`, following `crates/kali_cli/src/build/name_anon_functions.rs:830-831`.
- **Do not add a new crate dependency.** Everything needed is already present.
- **Do not modify `scripts/test-gate.sh`, `scripts/check-determinism.sh`, or `mise.toml`.** `scripts/test-gate.sh` is one of four files under an explicit prohibition recorded in its own header, and 71 markdown references across 11 documents invoke it.
- **Do not add a `tests/*.rs` integration target.** `AGENTS.md` §5: black-box CLI tests are `.toml` cases under the single `cases` target.
- **Fail closed.** Anywhere compiler identity cannot be established, decline the cache. Never fall back to a value that does not discriminate.
- **Gate after every task:** `bash scripts/test-gate.sh`. Run `mise run determinism` before the final commit of Task 5.
- **Every commit message ends with:** `Claude-Session: https://claude.ai/code/session_017voXBxMLeCP6b4ySREkmEZ`
- **Beware the very defect being fixed while fixing it.** Until Task 2 lands, a green local run proves nothing about a compiler-semantics change. A release-mode compile finishing in ~0.00s is a cache hit, not a pass.

---

### Task 1: The compiler-build fingerprint

**Files:**
- Create: `crates/kali_cli/src/build/fingerprint.rs`
- Test: `crates/kali_cli/src/build/fingerprint_tests.rs`
- Modify: `crates/kali_cli/src/build/mod.rs` (add `mod fingerprint;` to the module list at lines 3-13)

**Interfaces:**
- Consumes: `super::helpers::hex_encode` (`pub(crate) fn hex_encode(bytes: impl AsRef<[u8]>) -> String`, `crates/kali_cli/src/build/helpers.rs:17`)
- Produces: `pub(crate) fn compiler_build_fingerprint() -> Option<&'static str>` and `pub(crate) const FINGERPRINT_LEN: usize` — Task 2 consumes both.

- [ ] **Step 1: Create the module file with the implementation**

Create `crates/kali_cli/src/build/fingerprint.rs`:

```rust
//! Compiler-build identity for the incremental cache key.
//!
//! The cache key in `compile.rs` is built entirely from *inputs* and ends in a
//! frozen `CARGO_PKG_VERSION`. Nothing in it identifies the compiler that
//! produced the artifact, so a cached wasm file survives arbitrary
//! compiler-semantics changes and is served to a compiler that would no longer
//! produce it. This module supplies the missing discriminator.
//!
//! Design: `docs/superpowers/specs/2026-09-09-incremental-cache-compiler-identity-design.md` §3.1.

use super::helpers::hex_encode;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use std::time::UNIX_EPOCH;

/// Hex characters kept from the digest: long enough that a collision is not a
/// practical concern, short enough to keep cache filenames tractable.
pub(crate) const FINGERPRINT_LEN: usize = 16;

static FINGERPRINT: OnceLock<Option<String>> = OnceLock::new();

/// A discriminator for the *running compiler build*.
///
/// `None` means the compiler could not identify itself. Callers MUST fail
/// closed by declining the cache entirely — never by falling back to a value
/// that does not discriminate, which is the defect this module exists to fix.
pub(crate) fn compiler_build_fingerprint() -> Option<&'static str> {
    FINGERPRINT.get_or_init(compute_fingerprint).as_deref()
}

/// Hashes the running executable's identity.
///
/// `len` and `mtime` move on every relink — including a relink caused by a
/// change in a *dependency* crate such as `kali_optimize`, which is the case
/// that actually broke and the reason a `build.rs`-emitted constant would not
/// work: Cargo reruns a build script on its own package's files, not its
/// dependencies'.
///
/// `path` separates the `kali` binary from each in-process test binary. Those
/// are genuinely different compiler builds, each carrying an independently
/// linked copy of the compiler, so giving them separate namespaces is correct.
///
/// The binary's *contents* are deliberately not hashed: the debug binary is
/// over 500 MB, which costs a second or two per process.
fn compute_fingerprint() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let metadata = std::fs::metadata(&exe).ok()?;
    let mtime_nanos = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();

    let mut hasher = Sha256::new();
    hasher.update(exe.to_string_lossy().as_bytes());
    hasher.update(b"\0");
    hasher.update(metadata.len().to_le_bytes());
    hasher.update(mtime_nanos.to_le_bytes());

    let mut hex = hex_encode(hasher.finalize());
    hex.truncate(FINGERPRINT_LEN);
    Some(hex)
}

#[cfg(test)]
#[path = "fingerprint_tests.rs"]
mod fingerprint_tests;
```

- [ ] **Step 2: Write the failing test**

Create `crates/kali_cli/src/build/fingerprint_tests.rs`:

```rust
//! Tests for the compiler-build fingerprint.
//!
//! These run inside a test binary, which is itself a compiler build, so
//! `compiler_build_fingerprint()` must succeed here for the same reason it must
//! succeed for the `kali` binary.

use super::*;

#[test]
fn fingerprint_is_available_and_stable_within_one_process() {
    let first =
        compiler_build_fingerprint().expect("a test binary must be able to identify itself");
    let second = compiler_build_fingerprint().expect("the second call must also succeed");

    assert_eq!(
        first, second,
        "the OnceLock must hand back exactly one value per process"
    );
}

#[test]
fn fingerprint_is_a_fixed_width_hex_string() {
    let fingerprint = compiler_build_fingerprint().expect("fingerprint must be available");

    assert_eq!(
        fingerprint.len(),
        FINGERPRINT_LEN,
        "the fingerprint is truncated to a fixed width so cache filenames stay tractable"
    );
    assert!(
        fingerprint.chars().all(|c| c.is_ascii_hexdigit()),
        "the fingerprint must be hex so it is safe in a filename, got {fingerprint:?}"
    );
}
```

- [ ] **Step 3: Wire the module in**

In `crates/kali_cli/src/build/mod.rs`, add `mod fingerprint;` to the module list so it reads (alphabetical, after `exports`):

```rust
mod compile;
mod entrypoint;
mod eval;
mod exports;
mod fingerprint;
mod helpers;
mod metadata;
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_cli --lib fingerprint -- --nocapture`
Expected: PASS, 2 tests.

If `fingerprint_is_available_and_stable_within_one_process` fails, the platform cannot report `modified()` for the test binary. Do not add a fallback — that would reintroduce the defect. Report it instead.

- [ ] **Step 5: Run the gate**

Run: `bash scripts/test-gate.sh`
Expected: `GATE OK`.

- [ ] **Step 6: File the residual as a GitHub issue**

The mtime+len collision is an accepted residual per spec §3.1, and §6 requires it tracked.

```bash
gh issue create \
  --repo rahulmutt/kali \
  --label soundness \
  --title "Incremental cache fingerprint: mtime+len can collide on coarse-granularity filesystems" \
  --body "$(cat <<'BODY'
The incremental cache key's compiler-build discriminator hashes the running
executable's path, length and mtime (`crates/kali_cli/src/build/fingerprint.rs`).

On a filesystem with one-second mtime granularity, two compiler builds produced
within the same second whose binaries have identical byte lengths would produce
the same fingerprint, and the second build could read the first's cached
artifacts. `len` makes this vanishingly unlikely and no occurrence has been
observed.

Folding in the inode (`std::os::unix::fs::MetadataExt::ino`, with a `cfg`
fallback for Windows) would close it completely.

Accepted as a residual in
`docs/superpowers/specs/2026-09-09-incremental-cache-compiler-identity-design.md`
§3.1, which also records why content-hashing the binary (over 500 MB in debug)
and a `build.rs`-emitted constant (does not rerun on dependency changes) were
both rejected.
BODY
)"
```

Record the issue number returned, and add it to the residual paragraph in spec §3.1, replacing the sentence `**Ruled: not now, tracked as a GitHub issue** — see §6.` with `**Ruled: not now, tracked as rahulmutt/kali#<N>.**`

- [ ] **Step 7: Commit**

```bash
git add crates/kali_cli/src/build/fingerprint.rs \
        crates/kali_cli/src/build/fingerprint_tests.rs \
        crates/kali_cli/src/build/mod.rs \
        docs/superpowers/specs/2026-09-09-incremental-cache-compiler-identity-design.md
git commit -m "$(cat <<'MSG'
feat(build): a fingerprint that names the running compiler build

`compiler_build_fingerprint()` hashes the running executable's path, length and
mtime, memoized in a `OnceLock` so it costs one `stat` per process. len and
mtime move on every relink, including one caused by a change in a dependency
crate -- the case a `build.rs`-emitted constant would have missed, because Cargo
reruns a build script on its own package's files and not its dependencies'.

Nothing consumes it yet; the cache key picks it up in the next commit.

Claude-Session: https://claude.ai/code/session_017voXBxMLeCP6b4ySREkmEZ
MSG
)"
```

---

### Task 2: The cache key uses the fingerprint, and fails closed without it

**Files:**
- Modify: `crates/kali_cli/src/build/compile.rs:535-585` (`incremental_cache_path`)
- Modify: `crates/kali_cli/src/build/compile.rs` (bottom — wire the new sibling test file)
- Test: `crates/kali_cli/src/build/compile_cache_tests.rs` (create)
- Modify: `crates/kali_cli/tests/inprocess/release_constant_condition_loop.rs:195-199` (a comment this task falsifies)
- Modify: `docs/superpowers/followups/computed-member-static-name-discovered-defects.md` (§6 closing note)

**Interfaces:**
- Consumes: `compiler_build_fingerprint()` and `FINGERPRINT_LEN` from Task 1.
- Produces: `pub(crate) fn incremental_cache_path_with_fingerprint(source_path: &Path, mode: BuildMode, max_specializations: usize, api_surface: ApiSurface, runtime_profiles: &[String], profile_data: Option<&ProfileData>, compat_eval: bool, coverage: bool, fingerprint: Option<&str>) -> Result<Option<PathBuf>, Vec<Diagnostic>>` — Tasks 3 and 4 consume it. `incremental_cache_path` keeps its existing signature as the wrapper.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/src/build/compile_cache_tests.rs`:

```rust
//! Tests for incremental cache key composition.
//!
//! The key must name the compiler build that produced the artifact. These
//! tests pass synthetic fingerprints rather than relinking a binary mid-test,
//! which is why `incremental_cache_path_with_fingerprint` takes one.

use super::*;
use tempfile::TempDir;

/// A project root with a manifest, plus one compilable source file in it.
fn project_with_source() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp project root");
    std::fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#)
        .expect("write manifest");
    let source = dir.path().join("main.ts");
    std::fs::write(&source, "console.log(1);\n").expect("write source");
    (dir, source)
}

fn path_for(source: &Path, fingerprint: Option<&str>) -> Option<PathBuf> {
    incremental_cache_path_with_fingerprint(
        source,
        BuildMode::Fast,
        16,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
        fingerprint,
    )
    .expect("cache path computation must not error")
}

#[test]
fn a_missing_fingerprint_disables_the_cache_rather_than_falling_back() {
    let (_dir, source) = project_with_source();

    assert_eq!(
        path_for(&source, None),
        None,
        "a compiler that cannot identify itself must decline the cache entirely; \
         falling back to a version-only key is the defect this change removes"
    );
}

#[test]
fn distinct_fingerprints_yield_distinct_cache_paths() {
    let (_dir, source) = project_with_source();

    let first = path_for(&source, Some("aaaaaaaaaaaaaaaa")).expect("path for first build");
    let second = path_for(&source, Some("bbbbbbbbbbbbbbbb")).expect("path for second build");

    assert_ne!(
        first, second,
        "two compiler builds must not share a cache entry for the same source"
    );
}

#[test]
fn one_fingerprint_reproduces_one_path() {
    let (_dir, source) = project_with_source();

    let first = path_for(&source, Some("aaaaaaaaaaaaaaaa")).expect("first call");
    let second = path_for(&source, Some("aaaaaaaaaaaaaaaa")).expect("second call");

    assert_eq!(
        first, second,
        "the same compiler and the same inputs must hit the same entry, or the \
         cache never hits at all"
    );
}

#[test]
fn the_cache_filename_carries_the_fingerprint() {
    let (_dir, source) = project_with_source();

    let path = path_for(&source, Some("abcdef0123456789")).expect("cache path");
    let name = path
        .file_name()
        .expect("cache path has a filename")
        .to_string_lossy()
        .into_owned();

    assert!(
        name.contains("abcdef0123456789"),
        "the fingerprint must be visible in the key so a stale entry is \
         identifiable by name, got {name:?}"
    );
}
```

- [ ] **Step 2: Wire the test file and run it to verify it fails**

At the bottom of `crates/kali_cli/src/build/compile.rs`, add:

```rust
#[cfg(test)]
#[path = "compile_cache_tests.rs"]
mod compile_cache_tests;
```

Run: `cargo test -p kali_cli --lib compile_cache_tests`
Expected: FAIL to compile, with `cannot find function 'incremental_cache_path_with_fingerprint' in this scope`.

- [ ] **Step 3: Split the function and fold the fingerprint into the key**

In `crates/kali_cli/src/build/compile.rs`, add the import near the other `use super::` lines at the top of the file:

```rust
use super::fingerprint::compiler_build_fingerprint;
```

Replace the `incremental_cache_path` definition at line 535 with a wrapper plus the real function. The body below is the existing one with two changes: an early fail-closed return, and one extra component on the key.

```rust
#[allow(clippy::too_many_arguments)]
pub(crate) fn incremental_cache_path(
    source_path: &Path,
    mode: BuildMode,
    max_specializations: usize,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    profile_data: Option<&ProfileData>,
    compat_eval: bool,
    coverage: bool,
) -> Result<Option<PathBuf>, Vec<Diagnostic>> {
    incremental_cache_path_with_fingerprint(
        source_path,
        mode,
        max_specializations,
        api_surface,
        runtime_profiles,
        profile_data,
        compat_eval,
        coverage,
        compiler_build_fingerprint(),
    )
}

/// The key builder, taking compiler identity as a parameter so tests can assert
/// composition without relinking a binary.
#[allow(clippy::too_many_arguments)]
pub(crate) fn incremental_cache_path_with_fingerprint(
    source_path: &Path,
    mode: BuildMode,
    max_specializations: usize,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    profile_data: Option<&ProfileData>,
    compat_eval: bool,
    coverage: bool,
    fingerprint: Option<&str>,
) -> Result<Option<PathBuf>, Vec<Diagnostic>> {
    // Fail closed. A cache we cannot prove is ours is a cache we do not read,
    // and a version-only fallback is exactly the defect this replaces. Checked
    // before hashing the source, which is the expensive part.
    let Some(fingerprint) = fingerprint else {
        return Ok(None);
    };

    let source_hash = source_hash_for_file(source_path).map_err(|error| {
        vec![Diagnostic::error(
            e8::INTERNAL_ERROR as u32,
            format!(
                "failed to hash source file '{}': {}",
                source_path.display(),
                error
            ),
        )]
    })?;
    let Some(project_root) = project_root_for_source(source_path) else {
        return Ok(None);
    };
    let normalized_runtime_profiles = normalize_runtime_profiles(runtime_profiles.to_vec());
    let profile_key = profile_data
        .map(|profile| {
            let profile = profile.clone().normalized();
            let profile_json = serde_json::to_string(&profile).expect("serialize profile data");
            let profile_hash = Sha256::digest(profile_json.as_bytes());
            format!("profile:{}", hex_encode(profile_hash))
        })
        .unwrap_or_else(|| "profile:none".to_string());
    let cache_key = format!(
        "{}-{}-{}-{}-profiles:{}-{}-{}-{}-{}-{}",
        source_hash,
        build_mode_name(mode),
        api_surface,
        max_specializations,
        normalized_runtime_profiles.join(","),
        profile_key,
        compat_eval,
        coverage,
        env!("CARGO_PKG_VERSION"),
        fingerprint
    );
    Ok(Some(
        project_root
            .join(".kali-cache")
            .join("incremental")
            .join(format!("{}.wasm", cache_key)),
    ))
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --lib compile_cache_tests`
Expected: PASS, 4 tests.

- [ ] **Step 5: Correct the comment this change falsifies**

`crates/kali_cli/tests/inprocess/release_constant_condition_loop.rs:195-199` currently asserts the key "carries no compiler-build identity at all". That is now false. Replace those five comment lines:

```rust
// and compile for real, which is why PR #36 went red on both. The cache key
// (`crates/kali_cli/src/build/compile.rs:567-578`) carries no compiler-build
// identity at all -- it ends in a frozen `CARGO_PKG_VERSION` of `"0.1.0"` -- so
// an artifact survives arbitrary compiler-semantics changes. That is filed as its
// own entry in the discovered-defects document.
```

with:

```rust
// and compile for real, which is why PR #36 went red on both. The cache key
// USED TO carry no compiler-build identity at all -- it ended in a frozen
// `CARGO_PKG_VERSION` of `"0.1.0"` -- so an artifact survived arbitrary
// compiler-semantics changes. Fixed: the key now folds in a fingerprint of the
// running compiler build (`crates/kali_cli/src/build/fingerprint.rs`), and the
// fixtures decline the on-disk cache outright. Design:
// `docs/superpowers/specs/2026-09-09-incremental-cache-compiler-identity-design.md`.
```

- [ ] **Step 6: Add the closing note to the followup entry**

In `docs/superpowers/followups/computed-member-static-name-discovered-defects.md`, at the end of §6 (after the `**Suggested home:**` paragraph), append:

```markdown
**CLOSED**, 2026-09-09, by the design at
`docs/superpowers/specs/2026-09-09-incremental-cache-compiler-identity-design.md`.
The key now folds in a fingerprint of the running compiler build and fails
closed when it cannot be computed, and the fixtures decline the on-disk cache
through a new `incrementalCache: false` manifest field.

**One claim of this entry is withdrawn.** It said the fix "means **new build
machinery** -- a build script plus, in practice, a dependency". It needs
neither, and a build script would have *missed this exact failure*: Cargo reruns
a build script on its own package's files, not its dependencies', and the
semantics change that broke fannkuch was in `kali_optimize` and `kali_codegen`.
The discriminator is a memoized `stat` of `current_exe()`. This entry's
rejection of that route was a *scope* objection -- "made sideways, in a PR about
something else" -- and scope was the point of the follow-up project.
```

- [ ] **Step 7: Run the gate**

Run: `bash scripts/test-gate.sh`
Expected: `GATE OK`.

This gate is the first one in this branch's history that is trustworthy for a compiler-semantics change, because it is the first run in which a stale artifact cannot be read.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_cli/src/build/compile.rs \
        crates/kali_cli/src/build/compile_cache_tests.rs \
        crates/kali_cli/tests/inprocess/release_constant_condition_loop.rs \
        docs/superpowers/followups/computed-member-static-name-discovered-defects.md
git commit -m "$(cat <<'MSG'
fix(build): the cache key names the compiler, and fails closed without it

`incremental_cache_path` splits into a wrapper and
`incremental_cache_path_with_fingerprint`, which takes compiler identity as a
parameter so key composition is testable without relinking a binary. The key
gains the fingerprint as its final component.

When the fingerprint is `None` the function returns `Ok(None)` and the cache is
declined entirely, rather than falling back to the version-only key. That
fallback IS the defect; there is no safe version of it.

Every existing cache entry becomes unreachable, by design.

Corrects the comment block in `release_constant_condition_loop.rs`, which
asserted the key carries no compiler-build identity and would otherwise have rotted
into a lie in the same commit that falsified it. Closes §6 of the discovered-defects
document and withdraws its "needs new build machinery" claim.

Claude-Session: https://claude.ai/code/session_017voXBxMLeCP6b4ySREkmEZ
MSG
)"
```

---

### Task 3: Pin the incident

**Files:**
- Modify: `crates/kali_cli/src/build/compile_cache_tests.rs` (append)

**Interfaces:**
- Consumes: `incremental_cache_path_with_fingerprint` and the `project_with_source` helper from Task 2; `compile_source_file_with_cache_state` (`crates/kali_cli/src/build/compile.rs:155`).
- Produces: nothing consumed by later tasks.

Nothing in the repository currently pins the failure that started this. This is §2.2's fannkuch incident in miniature: an artifact written by an older compiler under the old key must not be read.

- [ ] **Step 1: Write the failing test**

Append to `crates/kali_cli/src/build/compile_cache_tests.rs`:

```rust
/// The old key, exactly as it was built before this change: every component an
/// input, ending in the frozen `CARGO_PKG_VERSION`. Real examples of this shape
/// were found in `crates/kali_cli/tests/fixtures/.kali-cache/incremental/`,
/// e.g. `sha256-18aab710...-release-deno-16-profiles:-profile:none-false-false-0.1.0.wasm`.
fn old_style_cache_filename(source: &Path) -> String {
    let source_hash = source_hash_for_file(source).expect("hash source");
    format!(
        "{}-fast-deno-16-profiles:-profile:none-false-false-{}.wasm",
        source_hash,
        env!("CARGO_PKG_VERSION")
    )
}

#[test]
fn an_artifact_written_under_the_old_key_is_never_served() {
    let (dir, source) = project_with_source();

    // Plant an artifact where the pre-fix compiler would have written one, with
    // contents that are obviously not a compile of this source. Before the fix,
    // `compile_source_file` returned exactly these bytes with `cache_hit: true`
    // and never reached codegen -- which is how a codegen diagnostic could not
    // be produced, and how a suite passed in ~0.00s.
    let planted = b"NOT-WASM-PLANTED-BY-AN-OLDER-COMPILER".to_vec();
    let cache_dir = dir.path().join(".kali-cache").join("incremental");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    std::fs::write(cache_dir.join(old_style_cache_filename(&source)), &planted)
        .expect("plant stale artifact");

    let output = compile_source_file_with_cache_state(
        &source,
        BuildMode::Fast,
        16,
        ApiSurface::Deno,
        &[],
        false,
        false,
    )
    .expect("compiling a trivial source must succeed");

    assert!(
        !output.cache_hit,
        "an artifact keyed without compiler identity must not be read"
    );
    assert_ne!(
        output.wasm_bytes, planted,
        "the planted bytes must never reach the caller"
    );
}
```

- [ ] **Step 2: Run the test to verify it passes, and prove it would have failed**

Run: `cargo test -p kali_cli --lib an_artifact_written_under_the_old_key_is_never_served`
Expected: PASS.

Now prove the test has teeth. Temporarily change the `format!` in `incremental_cache_path_with_fingerprint` to drop the trailing `-{}` and its `fingerprint` argument, restoring the old key shape.

Run: `cargo test -p kali_cli --lib an_artifact_written_under_the_old_key_is_never_served`
Expected: FAIL, on `an artifact keyed without compiler identity must not be read`.

**Revert that temporary change before continuing.** A test that cannot fail is not a pin.

- [ ] **Step 3: Clear the stale entries from the fixtures cache**

These are gitignored (`.gitignore:33`), so this is a working-tree cleanup with nothing to commit. They are unreachable after Task 2 and would otherwise sit on disk forever.

```bash
ls crates/kali_cli/tests/fixtures/.kali-cache/incremental/ | wc -l
rm -rf crates/kali_cli/tests/fixtures/.kali-cache/incremental/
```

- [ ] **Step 4: Run the gate**

Run: `bash scripts/test-gate.sh`
Expected: `GATE OK`. This run compiles the fixtures for real — a cold fixtures cache is the point.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/src/build/compile_cache_tests.rs
git commit -m "$(cat <<'MSG'
test(build): pin the stale-artifact incident that started this

An artifact planted under the OLD key shape -- every component an input, ending
in the frozen `CARGO_PKG_VERSION` -- must not be served. Before the fix,
`compile_source_file` returned exactly those bytes with `cache_hit: true` and
never reached codegen, which is why a codegen diagnostic could not be produced
and why a release-tier suite passed in ~0.00s.

Verified to have teeth: with the fingerprint component removed from the key, the
test fails on the `cache_hit` assertion.

Claude-Session: https://claude.ai/code/session_017voXBxMLeCP6b4ySREkmEZ
MSG
)"
```

---

### Task 4: The `incrementalCache` manifest opt-out

**Files:**
- Modify: `crates/kali_cli/src/build/compile.rs` (`incremental_cache_path_with_fingerprint`, plus a new private helper)
- Modify: `crates/kali_cli/src/build/compile_cache_tests.rs` (append)
- Modify: `crates/kali_cli/tests/fixtures/kali.json`
- Modify: `schemas/manifest/v1.json`
- Modify: `specs/18-schemas.md`
- Modify: `specs/12-cli.md`

**Interfaces:**
- Consumes: `incremental_cache_path_with_fingerprint` from Task 2.
- Produces: nothing consumed by later tasks.

- [ ] **Step 1: Write the failing tests**

Append to `crates/kali_cli/src/build/compile_cache_tests.rs`:

```rust
/// A project root whose manifest is exactly `manifest`, plus one source file.
fn project_with_manifest(manifest: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp project root");
    std::fs::write(dir.path().join("kali.json"), manifest).expect("write manifest");
    let source = dir.path().join("main.ts");
    std::fs::write(&source, "console.log(1);\n").expect("write source");
    (dir, source)
}

#[test]
fn a_project_can_decline_the_incremental_cache() {
    let (_dir, source) =
        project_with_manifest(r#"{"schemaVersion":1,"incrementalCache":false}"#);

    assert_eq!(
        path_for(&source, Some("aaaaaaaaaaaaaaaa")),
        None,
        "`incrementalCache: false` must decline the on-disk cache"
    );
}

#[test]
fn an_omitted_incremental_cache_field_leaves_the_cache_enabled() {
    let (_dir, source) = project_with_manifest(r#"{"schemaVersion":1}"#);

    assert!(
        path_for(&source, Some("aaaaaaaaaaaaaaaa")).is_some(),
        "omission must preserve the behaviour every existing project already has"
    );
}

#[test]
fn an_explicit_true_leaves_the_cache_enabled() {
    let (_dir, source) =
        project_with_manifest(r#"{"schemaVersion":1,"incrementalCache":true}"#);

    assert!(
        path_for(&source, Some("aaaaaaaaaaaaaaaa")).is_some(),
        "an explicit true must mean the same as omission"
    );
}

#[test]
fn an_unparseable_manifest_leaves_the_cache_enabled() {
    let (_dir, source) = project_with_manifest("{ this is not json");

    assert!(
        path_for(&source, Some("aaaaaaaaaaaaaaaa")).is_some(),
        "a broken manifest must not silently change caching behaviour; \
         `load_exclude_set` is tolerant the same way"
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_cli --lib compile_cache_tests`
Expected: `a_project_can_decline_the_incremental_cache` FAILS with a left/right mismatch (it gets `Some(path)`, expects `None`). The other three pass already.

- [ ] **Step 3: Implement the manifest check**

In `crates/kali_cli/src/build/compile.rs`, inside `incremental_cache_path_with_fingerprint`, immediately after the `project_root` binding:

```rust
    let Some(project_root) = project_root_for_source(source_path) else {
        return Ok(None);
    };
    if !project_incremental_cache_enabled(&project_root) {
        return Ok(None);
    }
```

And add this private helper next to `project_root_for_source`:

```rust
/// `incrementalCache: false` in the project manifest declines the on-disk
/// artifact cache. Omitted, `true`, or anything unreadable means enabled, which
/// is the behaviour every existing project already has.
///
/// Deliberately tolerant of a missing or malformed manifest, matching
/// `load_exclude_set` (`crates/kali_cli/src/lib.rs:524`): a broken manifest must
/// not silently change caching behaviour.
fn project_incremental_cache_enabled(project_root: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(project_root.join("kali.json")) else {
        return true;
    };
    let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return true;
    };
    manifest
        .get("incrementalCache")
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --lib compile_cache_tests`
Expected: PASS, 9 tests.

- [ ] **Step 5: Make the fixtures decline the cache**

Replace `crates/kali_cli/tests/fixtures/kali.json` with:

```json
{
  "schemaVersion": 1,
  "incrementalCache": false,
  "exclude": [
    "benchmarks/fasta-benchmark-v1.ts",
    "benchmarks/join-loop-peak.ts",
    "benchmarks/string-loop-peak.ts"
  ]
}
```

The `exclude` list is load-bearing — `load_exclude_set` (`crates/kali_cli/src/lib.rs:524`) reads it during directory descent — so the manifest is amended, never removed, and the fixtures directory stays a project root.

- [ ] **Step 6: Add the field to the JSON schema**

In `schemas/manifest/v1.json`, add to `properties`, after the `sandbox` entry:

```json
    "incrementalCache": {
      "type": "boolean"
    },
```

- [ ] **Step 7: Document the field in the owning spec chapter**

In `specs/18-schemas.md`, in the `## Project Configuration Schema` section (line 748), add to the `### Rules` bullet list, after the `$schema: string` bullet:

```markdown
- `incrementalCache: boolean` is an optional top-level field; `false` declines the on-disk incremental artifact cache for the project, and omission means enabled
```

And in the `### Schema-v1 defaulting and omission rules` section, add to its omission list:

```markdown
- In schema v1, omitted `incrementalCache` means the on-disk incremental artifact cache is enabled, which is the behavior every existing project already has; `false` declines it and is not a request to delete existing entries
```

- [ ] **Step 8: Mirror the naming rule in the CLI chapter**

`specs/12-cli.md:812` states that chapter "only repeats the naming rules so CLI and schema docs do not drift", so the omission rule must appear there too. In the `Omission/default rule for minimal configs` list (around line 824), add:

```markdown
- In schema v1, omitted `incrementalCache` means the on-disk incremental artifact cache is enabled; setting it to `false` declines the cache for that project.
```

- [ ] **Step 9: Run the gate**

Run: `bash scripts/test-gate.sh`
Expected: `GATE OK`.

Confirm the fixtures cache stays empty, which is the whole point of this task:

```bash
ls crates/kali_cli/tests/fixtures/.kali-cache/ 2>&1
```
Expected: `No such file or directory`.

- [ ] **Step 10: Commit**

```bash
git add crates/kali_cli/src/build/compile.rs \
        crates/kali_cli/src/build/compile_cache_tests.rs \
        crates/kali_cli/tests/fixtures/kali.json \
        schemas/manifest/v1.json \
        specs/18-schemas.md \
        specs/12-cli.md
git commit -m "$(cat <<'MSG'
feat(manifest): a project can decline the on-disk incremental cache

`incrementalCache: false` in `kali.json` declines the cache; omission and `true`
both mean enabled, which is what every existing project already has. A missing or
malformed manifest also means enabled, matching `load_exclude_set`'s tolerance --
a broken manifest must not silently change caching behaviour.

Set on `crates/kali_cli/tests/fixtures/kali.json`, so the fixture-driven tests
stop reading a live machine-local cache. The alternative -- rewriting nine test
files onto `TempDir` roots -- is a wide mechanical change with no better
guarantee, and an env-var switch would race `set_var` against parallel test
threads. The fixtures manifest keeps its `exclude` list, which is load-bearing.

Documented in `specs/18-schemas.md` (the canonical schema) and mirrored into
`specs/12-cli.md`, which exists to keep the two from drifting.

Claude-Session: https://claude.ai/code/session_017voXBxMLeCP6b4ySREkmEZ
MSG
)"
```

---

### Task 5: The cache reaper

**Files:**
- Create: `crates/kali_cli/src/build/reap.rs`
- Test: `crates/kali_cli/src/build/reap_tests.rs`
- Modify: `crates/kali_cli/src/build/mod.rs` (add `mod reap;`)
- Modify: `crates/kali_cli/src/build/compile.rs:271-274` (the cache write site)

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: `pub(crate) fn maybe_reap(cache_dir: &Path)` and `pub(crate) fn reap(cache_dir: &Path)`.

The fingerprint makes growth unbounded: every rebuild orphans an entire namespace. Entries are ~7 KB, so **file count is the acute resource, not bytes** — the budget caps entries first.

- [ ] **Step 1: Create the module**

Create `crates/kali_cli/src/build/reap.rs`:

```rust
//! Bounding the incremental cache directory.
//!
//! The compiler-build fingerprint in the cache key namespaces entries per
//! build, so every rebuild orphans a whole namespace and the directory grows
//! without bound. Entries are small (~7 KB), which makes FILE COUNT the acute
//! resource rather than bytes.
//!
//! This is eviction by *insertion* age, not by use: `std::fs` cannot set an
//! mtime, so there is no touch-on-hit and no dependency is added to get one. A
//! hot entry older than `MAX_AGE` is evicted and recompiled once. Calling it an
//! LRU would be wrong.
//!
//! Design: `docs/superpowers/specs/2026-09-09-incremental-cache-compiler-identity-design.md` §3.4.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

const MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const MAX_ENTRIES: usize = 4096;
const MAX_BYTES: u64 = 128 * 1024 * 1024;

/// Sweeping on every write costs an O(N) `readdir` per compile; sweeping once
/// per process lets a single long test binary outrun the budget. This is the
/// compromise, and the number is arbitrary within an order of magnitude.
const SWEEP_INTERVAL: usize = 256;

static WRITES_SINCE_SWEEP: AtomicUsize = AtomicUsize::new(0);

/// Sweep on the first cache write in this process, and every
/// `SWEEP_INTERVAL` writes thereafter.
pub(crate) fn maybe_reap(cache_dir: &Path) {
    let previous = WRITES_SINCE_SWEEP.fetch_add(1, Ordering::Relaxed);
    if previous % SWEEP_INTERVAL == 0 {
        reap(cache_dir);
    }
}

/// Best-effort. Every IO error is ignored: a reaper that can fail a build is
/// worse than a full disk.
///
/// Needs no locking. A concurrent reader whose entry is deleted between path
/// computation and `fs::read` gets `ErrorKind::NotFound`, which the read path
/// already treats as a miss and falls through to a real compile
/// (`compile.rs:245`). The worst outcome of a race is a recompile.
pub(crate) fn reap(cache_dir: &Path) {
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return;
    };
    let now = SystemTime::now();
    let mut kept: Vec<(SystemTime, u64, PathBuf)> = Vec::new();

    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }
        let path = entry.path();
        // A file with no readable mtime is treated as brand new rather than
        // ancient, so an unreadable clock cannot cause mass deletion.
        let modified = metadata.modified().unwrap_or(now);
        let too_old = now
            .duration_since(modified)
            .map(|age| age > MAX_AGE)
            .unwrap_or(false);
        if too_old {
            let _ = fs::remove_file(&path);
            continue;
        }
        kept.push((modified, metadata.len(), path));
    }

    // Oldest first: those are evicted first when over budget.
    kept.sort_by(|left, right| left.0.cmp(&right.0));

    let mut count = kept.len();
    let mut bytes: u64 = kept.iter().map(|(_, len, _)| *len).sum();

    for (_, len, path) in &kept {
        if count <= MAX_ENTRIES && bytes <= MAX_BYTES {
            break;
        }
        if fs::remove_file(path).is_ok() {
            count -= 1;
            bytes = bytes.saturating_sub(*len);
        }
    }
}

#[cfg(test)]
#[path = "reap_tests.rs"]
mod reap_tests;
```

- [ ] **Step 2: Write the tests**

Create `crates/kali_cli/src/build/reap_tests.rs`:

```rust
//! Tests for the incremental-cache reaper.

use super::*;
use tempfile::TempDir;

/// Writes `name` with `size` bytes, back-dated by `age`.
///
/// `filetime` is not a dependency, so age is simulated by writing the file and
/// asserting against a `MAX_AGE` boundary the test controls, rather than by
/// setting mtimes. For the age test we instead drive `reap` against a directory
/// whose entries are all fresh, and check the count/size paths directly.
fn write_entry(dir: &Path, name: &str, size: usize) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, vec![0u8; size]).expect("write cache entry");
    path
}

#[test]
fn a_missing_directory_is_a_no_op() {
    let dir = TempDir::new().expect("temp dir");
    let missing = dir.path().join("does-not-exist");

    reap(&missing); // must not panic
    assert!(!missing.exists());
}

#[test]
fn an_empty_directory_is_a_no_op() {
    let dir = TempDir::new().expect("temp dir");

    reap(dir.path());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn entries_within_budget_are_all_kept() {
    let dir = TempDir::new().expect("temp dir");
    for index in 0..16 {
        write_entry(dir.path(), &format!("entry-{index}.wasm"), 64);
    }

    reap(dir.path());

    assert_eq!(
        fs::read_dir(dir.path()).unwrap().count(),
        16,
        "nothing may be evicted while under both budgets"
    );
}

#[test]
fn exceeding_the_entry_cap_evicts_down_to_it() {
    let dir = TempDir::new().expect("temp dir");
    for index in 0..(MAX_ENTRIES + 32) {
        write_entry(dir.path(), &format!("entry-{index:05}.wasm"), 1);
    }

    reap(dir.path());

    let remaining = fs::read_dir(dir.path()).unwrap().count();
    assert!(
        remaining <= MAX_ENTRIES,
        "the entry cap is the acute resource; got {remaining} entries"
    );
}

#[test]
fn a_directory_containing_a_subdirectory_ignores_it() {
    let dir = TempDir::new().expect("temp dir");
    fs::create_dir(dir.path().join("nested")).expect("create nested dir");
    write_entry(dir.path(), "entry.wasm", 64);

    reap(dir.path());

    assert!(
        dir.path().join("nested").exists(),
        "the reaper removes files, never directories"
    );
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p kali_cli --lib reap_tests`
Expected: FAIL to compile — `mod reap;` is not yet declared in `build/mod.rs`.

- [ ] **Step 4: Wire the module and the call site**

In `crates/kali_cli/src/build/mod.rs`, add to the module list (after `paths`):

```rust
mod paths;
mod reap;
mod wit;
```

In `crates/kali_cli/src/build/compile.rs`, add to the `use super::` block at the top:

```rust
use super::reap::maybe_reap;
```

Then change the cache write site at `compile.rs:271-274` from:

```rust
        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
            let _ = fs::write(&cache_path, &wasm_bytes);
        }
```

to:

```rust
        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
            let _ = fs::write(&cache_path, &wasm_bytes);
            // After the write, so a fresh entry is never the one evicted for
            // being over budget on the sweep its own write triggered.
            maybe_reap(parent);
        }
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --lib reap_tests`
Expected: PASS, 5 tests.

- [ ] **Step 6: Run the full gate and the determinism lane**

Run: `bash scripts/test-gate.sh`
Expected: `GATE OK`.

Run: `mise run determinism`
Expected: all 20 exact tests pass. The reaper touches a directory the determinism tests' builds write into, so this lane must be clean before the branch is done.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_cli/src/build/reap.rs \
        crates/kali_cli/src/build/reap_tests.rs \
        crates/kali_cli/src/build/mod.rs \
        crates/kali_cli/src/build/compile.rs
git commit -m "$(cat <<'MSG'
feat(build): bound the incremental cache directory

The compiler-build fingerprint namespaces cache entries per build, so every
rebuild orphans a whole namespace and the directory grows without bound. Entries
are ~7 KB, which makes FILE COUNT the acute resource rather than bytes, so the
budget caps entries (4096) before bytes (128 MiB), with a 7-day age cut first.

Swept after a cache write, on the first write in a process and every 256th
after -- sweeping every write costs an O(N) readdir per compile, sweeping once
per process lets a long test binary outrun the budget.

Best-effort throughout: every IO error is ignored, because a reaper that can
fail a build is worse than a full disk. No locking is needed -- a reader whose
entry is deleted mid-flight gets NotFound, which the read path already treats as
a miss, so the worst outcome of a race is a recompile.

This is eviction by INSERTION age, not use. `std::fs` cannot set an mtime, so
there is no touch-on-hit and no dependency was added to get one. It is not an
LRU and the module says so.

Claude-Session: https://claude.ai/code/session_017voXBxMLeCP6b4ySREkmEZ
MSG
)"
```

---

## Self-Review

**Spec coverage.** §3.1 fingerprint → Task 1. §3.2 key and fail-closed → Task 2. §3.3 manifest opt-out → Task 4. §3.4 reaper → Task 5. §5's verification list → Tasks 1-5 (fingerprint stability and width, distinct/identical fingerprint paths, `None` fails closed, manifest opt-out, the four reaper cases, the old-key regression pin). §6's ledger → schema and both spec chapters in Task 4, the followup closing note and the stale comment in Task 2, the GitHub issue in Task 1.

**One §5 item deliberately not implemented as written.** §5 asks for "a fixture compile asserts `cache_hit == false`". After Task 4 the fixtures decline the cache entirely, so *every* fixture compile returns `cache_hit: false` and such an assertion would pass vacuously — it would pin nothing. The `cache_hit` claim is instead pinned where it has teeth, in Task 3's `an_artifact_written_under_the_old_key_is_never_served`, which asserts it against a planted stale artifact in a cache-enabled root. Spec §7 already names this trade ("no fixture-driven test exercises the cache path at all"); this records that the substitution was deliberate.

**One §5 item narrowed.** §5 asks the reaper tests to cover "an entry older than MAX_AGE is removed; one younger is kept". `std::fs` cannot set an mtime and the Global Constraints forbid a new dependency (`filetime`), so back-dating a file is not available. Task 5 covers the entry-cap, budget, no-op, and directory-safety paths directly, and the age path is exercised only through its `unwrap_or(false)` guard. **This is a real coverage gap and should be stated in the PR**, not papered over: the age cut is the least-tested branch of the reaper.

**Placeholder scan.** No TBDs. Every code step carries the actual code. Task 2's spec-doc edits give exact replacement text; Task 4's give exact bullets with their anchoring sections.

**Type consistency.** `compiler_build_fingerprint() -> Option<&'static str>` (Task 1) feeds `fingerprint: Option<&str>` (Task 2) — `&'static str` coerces. `FINGERPRINT_LEN` is `pub(crate)` so `fingerprint_tests.rs` can assert against it. `path_for` (Task 2) is reused by Tasks 3 and 4; `project_with_source` (Task 2) by Task 3; `project_with_manifest` (Task 4) is separate because it varies the manifest. `maybe_reap(&Path)` matches the `parent` binding at the call site.
