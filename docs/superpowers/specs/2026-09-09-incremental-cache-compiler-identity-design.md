# The incremental cache key must name the compiler that produced the artifact

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `3243e60cf6` (main, clean tree, the merge of PR #36) |
| kali binary | `kali 0.1.0`, `.cache/cargo-target/debug/kali` (571,774,952 bytes, built 2026-09-09) |
| oracle | not applicable — this is a tooling defect, not a divergence from node |
| measured on | 2026-09-09 |
| the defect this closes | §6 of `docs/superpowers/followups/computed-member-static-name-discovered-defects.md`, filed 2026-09-09 at `6b59ddeef9` |

This document designs the fix for one entry in that followup file. It does not
re-measure the entry: §6 established the mechanism and the consequence, both are
quoted below with their line references re-checked at the baseline, and the work
here is a design, not a re-investigation.

**The one thing this document adds to §6's account** is a correction of scope.
§6 says the fix needs "new build machinery — a build script plus, in practice, a
dependency". §3.1 shows it needs neither: the discriminator is available at run
time from `std::env::current_exe()` for the cost of one `stat`, and §6's
rejection of that route was a *scope* objection ("made sideways, in a PR about
something else"), not a correctness one. Scope is now the point of the work, so
the objection does not carry.

---

## 1. What this project is

`incremental_cache_path` builds a cache key entirely out of **inputs** and ends
it in `env!("CARGO_PKG_VERSION")`, frozen at `"0.1.0"` since the repository
began. Nothing in the key identifies the compiler build. An artifact therefore
survives arbitrary compiler-semantics changes and is served to a compiler that
would no longer produce it.

This project adds a compiler-build discriminator to that key, makes the cache
fail closed when the discriminator cannot be computed, gives the project manifest
a way to decline the cache entirely, and bounds the cache directory's growth —
which the discriminator itself makes unbounded, and which is the reason the
reaper is in scope rather than filed.

**Two halves, both in scope**, on the ruling taken during design:

1. **Cache correctness.** No user's artifact may be served to a different
   compiler build. This is the half that ships to end users, and it is the half
   nothing else can fix — see §2.3.
2. **A local gate that cannot be silently warm.** The test suite must not depend
   on a machine-local cache to decide whether codegen ran.

---

## 2. The defect, measured

### 2.1 The mechanism

`compile_source_file_with_cache_state_and_profile_data_and_validation`
(`crates/kali_cli/src/build/compile.rs`) short-circuits on an on-disk wasm cache.
The read is at `compile.rs:237-244`: a hit returns the cached bytes with
`cache_hit: true` (`compile.rs:241`) and never runs resolver, HIR, MIR, LIR,
optimizer or **codegen**.

The key is built at `compile.rs:567-578`:

```
{source_hash}-{build_mode}-{api_surface}-{max_specializations}
  -profiles:{runtime_profiles}-{profile_key}-{compat_eval}-{coverage}
  -{CARGO_PKG_VERSION}
```

Source hash, build mode, API surface, specialization budget, runtime profiles,
profile data, `compat_eval`, `coverage` — every one an **input**. The final
component (`compile.rs:577`) is the only candidate for compiler identity and it
is a constant that has never moved.

The cache root is `{project_root}/.kali-cache/incremental/` (`compile.rs:581`),
where `project_root` is found by walking ancestors for a `kali.json`
(`project_root_for_source`, `compile.rs:587`).

### 2.2 The consequence, measured rather than hypothesised

`crates/kali_cli/tests/fixtures/kali.json` makes the fixtures directory a project
root, so `crates/kali_cli/tests/fixtures/.kali-cache/incremental/` is a live,
gitignored (`.gitignore:33`), machine-local cache that every fixture-driven test
reads. It held two release-tier artifacts for `fannkuch-redux-benchmark-v1.ts`
written **2026-07-16 by the pre-project compiler**.

`E5506` is emitted at **codegen**, so a cache hit cannot produce it. With those
two files present, `fannkuch_redux_builds_in_all_release_modes` passed in
`0.00s` — that is the test's name *at the time of the incident*; `8cade901ee`
has since renamed it to
`fannkuch_redux_builds_and_runs_at_fast_and_is_refused_at_both_release_tiers` — three `fs::read`s, not three release compiles. With them moved aside
it failed immediately, identically to CI. A local `bash scripts/test-gate.sh`
reporting `GATE OK` was therefore meaningless for that branch, and the sweep that
re-pinned `spectral-norm` and `nbody` for the optimizer defect skipped fannkuch
because the machine doing the sweeping was being told fannkuch was fine. CI
runners are cold and went red on both ubuntu and macOS.

**At the baseline the directory holds 15 entries totalling 106,131 bytes** —
about 7 KB each. That number matters for §3.4 and is recorded here because the
reaper's budget is sized against it, not against a guess.

### 2.3 What cannot fix this

A Bazel migration was evaluated during design and rejected. The decisive point is
recorded here because it is the natural wrong answer to reach for: **this cache
belongs to the shipped `kali` binary and lives in an end user's project
directory.** A build system that builds `kali` is not present when someone runs
`kali build app.ts` on their own machine, so it cannot supply compiler identity
to the key. Bazel would have caught §2.2's *test* failure through sandboxing, and
would have left the shipped defect entirely intact.

The full evaluation — polyglot (350,906 LOC of Rust against 9 `.lean` and 7
`.mjs` files), incrementality (rustc's unit is the crate, so no granularity gain
over cargo, and a cold workspace build with dependencies warm measures **30 s**),
remote caching, hermeticity (already owned by `devenv.nix` + `mise` +
`Cargo.lock` + SLSA3) — found zero of the four adoption triggers holding. It is
summarised here rather than filed separately because the question will be asked
again.

**One correction to that evaluation, recorded rather than quietly fixed.** An
earlier draft of this section claimed `crates/kali_cli/tests` was "122,598 LOC in
2 targets" and used it to argue that Bazel's per-target test-result caching had
nothing to work with here. That was wrong: 2 is the number of *explicitly
declared* `[[test]]` entries in `crates/kali_cli/Cargo.toml`, and Cargo
auto-discovers every top-level `tests/*.rs` besides. The real count is **68 test
targets in `kali_cli` and 71 across the workspace**. The inference therefore runs
the other way — the suite is already finely partitioned, so cargo parallelises it
and Bazel would inherit a granularity it did not create. The conclusion is
unchanged, on the other three triggers; the supporting fact is not.

`AGENTS.md` §5's rule that black-box CLI tests go in the single `cases` target
governs *that lane only*, and does not describe the workspace's test-target
strategy generally.

---

## 3. The design

### 3.1 The fingerprint

New module `crates/kali_cli/src/build/fingerprint.rs`:

```rust
pub(crate) fn compiler_build_fingerprint() -> Option<&'static str>
```

Memoized in a `OnceLock`, so it costs **one `stat` per process**. It reads
`std::env::current_exe()`, takes `fs::metadata` for `len()` and `modified()`, and
hashes `path | len | mtime_nanos` with the `sha2` already in the workspace,
truncated to 16 hex characters to keep filenames tractable.

Why those three inputs, each for its own reason:

* **mtime and len** move on every relink — including a relink caused by a change
  in `kali_optimize` or `kali_codegen`, which is the case that actually broke in
  §2.2. A no-op `cargo build` does not relink, so mtime holds steady and a real
  user's cache keeps hitting.
* **path** separates the `kali` binary from each in-process test binary. §6 of
  the followup called this re-namespacing a defect; under a discriminator that
  must never miss it is **correct**, because those are genuinely different
  compiler builds carrying independently-linked copies of the compiler.

**Why not content-hash the binary.** The debug binary is 571,774,952 bytes.
SHA-256 over it is on the order of a second or two per process, paid by every
`kali` invocation and every test binary. Rejected on cost, not on principle.

**Why not a build script.** A `build.rs` in `kali_cli` does not re-run when
`kali_optimize` changes — Cargo reruns a build script on its own package's files,
not its dependencies'. A build-script-emitted constant would therefore have
missed §2.2's exact failure, where the semantics change was in the optimizer and
codegen. This is the load-bearing reason the design does not follow §6's
suggested shape.

**The accepted residual.** On a filesystem with one-second mtime granularity, two
builds within the same second producing byte-identical lengths would collide.
`len` makes this vanishingly unlikely and no observation of it exists. Folding in
the inode (`std::os::unix::fs::MetadataExt`, with a `cfg` fallback for Windows)
would close it completely. **Ruled: not now, tracked as a GitHub issue** — see
§6.

### 3.2 The key, and failing closed

At `compile.rs:577` the trailing `env!("CARGO_PKG_VERSION")` becomes
`{version}-{fingerprint}`.

**If the fingerprint is `None`, `incremental_cache_path` returns `Ok(None)`.**
The cache is disabled for that process rather than falling back to a
version-only key. A cache we cannot prove is ours is a cache we do not read.
This costs nothing structurally: `Ok(None)` is already a supported return, and
the caller's `else` branch already compiles uncached through
`compile_source_file_uncached` (`compile.rs:282`).

To make key composition testable without relinking a binary mid-test, the
formatting moves into a private function taking the fingerprint as a parameter,
with the existing `incremental_cache_path` signature kept as the wrapper that
supplies the real one. Tests then assert composition against synthetic
fingerprints.

**Every existing entry becomes unreachable, by design.** The 15 entries in
`crates/kali_cli/tests/fixtures/.kali-cache/incremental/` are deleted by this
change; they are gitignored, so this is a working-tree cleanup, not a commit.

### 3.3 The manifest opt-out

The fixture-driven tests reach fixtures **in place** through
`CARGO_MANIFEST_DIR` — `clbg_fannkuch_runtime.rs`, `clbg_nbody_runtime.rs`,
`clbg_spectral_norm_runtime.rs`, `clbg_mandelbrot_runtime.rs`,
`clbg_fasta_runtime.rs`, `clbg_binary_trees_runtime.rs`,
`reclamation_bounded_peak.rs`, `inprocess/release_constant_condition_loop.rs`
and others. Rewriting all of them onto `TempDir` roots is a wide mechanical
change with real regression risk and no better guarantee.

Instead: **an `incrementalCache: false` field in the project manifest**, set in
`crates/kali_cli/tests/fixtures/kali.json`. `incremental_cache_path` already
walks up to find that exact file to locate the project root (`compile.rs:587`);
reading one more field off the manifest it has just found is nearly free.

Why this shape rather than an environment variable:

* No `std::env::set_var` race. Tests run as parallel threads in one process, and
  a `set_var` racing a concurrent read is exactly the class of bug this project
  exists to stop introducing.
* No per-test-file edits, so no regression surface across nine files.
* It is a defensible user-facing feature, not a test hack: "this project does not
  want an on-disk artifact cache" is a legitimate thing for a manifest to say.

`crates/kali_cli/tests/fixtures/kali.json` **keeps its `exclude` list** — that is
load-bearing, read by `load_exclude_set` (`crates/kali_cli/src/lib.rs:535`)
during directory descent. The manifest is amended, never removed, and the
fixtures directory stays a project root.

`schemas/manifest/v1.json` is `additionalProperties: true`, so the field is
additive and needs no `schemaVersion` bump. It is nonetheless added to the schema
explicitly, so it is documented rather than merely tolerated.

### 3.4 The reaper

§3.1's fingerprint makes cache growth **unbounded**: under the old key the
directory was bounded by the number of input combinations, and under the new one
every rebuild orphans an entire namespace. This is a cost the design introduces,
and it is closed here rather than filed, on the ruling that this pod has already
crashed once from disk exhaustion.

**The acute resource is file count, not bytes.** Entries are ~7 KB (§2.2). A
machine rebuilding the compiler through a fixture suite of ~100 sources across 3
build modes orphans thousands of small files per day; tens of thousands of
entries in one directory degrade every `readdir` long before the bytes matter.
The budget therefore caps entries first.

New module `crates/kali_cli/src/build/reap.rs`. Policy, applied in order:

1. Delete entries whose mtime is older than **`MAX_AGE` = 7 days**.
2. If the directory still exceeds **`MAX_ENTRIES` = 4096** or **`MAX_BYTES` =
   128 MiB**, delete oldest-mtime-first until both are under budget.

**Trigger:** after a successful cache *write*, on the first write in a process
and every 256th thereafter, counted with an `AtomicUsize`. Sweeping on every
write would pay an O(N) `readdir` for every compile; sweeping once per process
would let a single long test binary write unbounded entries. This is the
compromise, and the constant is arbitrary within an order of magnitude.

**Best-effort, always.** Every IO error in the reaper is ignored. A reaper that
can fail a build is worse than a full disk.

**Deletion races are safe by construction**, and this is why the reaper needs no
locking: a concurrent reader whose entry is deleted between path computation and
`fs::read` gets `ErrorKind::NotFound`, which `compile.rs:245` already treats as a
miss and falls through to a real compile. The worst outcome of a race is a
recompile.

**It is not a true LRU, and the document says so rather than calling it one.**
There is no touch-on-hit: `std::fs` cannot set an mtime, and pulling in
`filetime` for that alone is not worth a dependency. Eviction is therefore by
*insertion* age, not by *use* age — a hot entry older than `MAX_AGE` is evicted
and recompiled once. Given that §3.1 already namespaces entries per compiler
build, the population is naturally short-lived and the distinction is small.

---

## 4. Non-goals, each with the reason it is outside

| not done | why |
|---|---|
| Inode in the fingerprint | Ruled out for now; the mtime+len collision is an accepted residual, tracked as a GitHub issue (§6). |
| Touch-on-hit for a true LRU | Needs a `filetime` dependency to set mtimes; §3.4 takes insertion-age eviction instead and names the consequence. |
| Manifest-tunable reaper budgets | More public surface for no demonstrated need. Constants now; a field if someone hits the cap. |
| `metadata.rs:90`'s frozen `kali_version` | The same constant is stamped into every artifact's metadata, which is a *claims* defect, not a *cache* defect. Separate surface, separate schema question, not this project. |
| §4's batch-3 generator defect | Named by §6 as the same class of instrument defect. Same class is not same fix. |
| Migrating the build to Bazel | §2.3. Zero of four adoption triggers hold, and it cannot fix the shipped half regardless. |
| Rewriting fixture tests onto `TempDir` roots | §3.3. Wide mechanical change, no better guarantee than the manifest field. |

---

## 5. Verification

Rust unit tests in sibling `*tests.rs` files, per `AGENTS.md` §5 — never inline
`#[cfg(test)]` modules.

**`crates/kali_cli/src/build/fingerprint_tests.rs`**

* the fingerprint is `Some` inside a test binary, and two calls in one process
  return the same value (the `OnceLock` holds);
* two distinct synthetic fingerprints produce two distinct cache paths;
* the same synthetic fingerprint reproduces the same path exactly;
* a `None` fingerprint yields `Ok(None)` — **not** a version-only key. This is
  the fail-closed assertion of §3.2 and it must fail if someone reinstates a
  fallback.

**`crates/kali_cli/src/build/reap_tests.rs`**

* an entry older than `MAX_AGE` is removed; one younger is kept;
* over `MAX_ENTRIES`, the oldest are removed first and the newest survive;
* over `MAX_BYTES` with a small entry count, eviction still runs;
* a directory that does not exist, and an unreadable entry, are both no-ops
  rather than errors.

**The regression test that pins the incident.** In a `TempDir` project root,
plant a wasm file under an **old-style key** — frozen version, no fingerprint —
then compile that source and assert `cache_hit == false` and that the returned
bytes are not the planted ones. That is §2.2's fannkuch failure in miniature, and
nothing in the repository currently pins it.

**The manifest opt-out.** In a `TempDir` root whose `kali.json` carries
`incrementalCache: false`, `incremental_cache_path` returns `Ok(None)`; with the
field absent or `true`, it returns a path.

**The durable form of "read the timing".** A fixture compile asserts
`cache_hit == false` using the flag `CompileOutput` already carries
(`compile.rs:89`), rather than trusting that a 0.00s release compile looks
suspicious to whoever is reading the output.

**Gates.** `bash scripts/test-gate.sh` and `mise run determinism`. Neither script
is modified — `scripts/test-gate.sh` is under an explicit prohibition recorded in
its own header, and 71 markdown references across 11 plan and design documents
invoke it as a per-task gate.

---

## 6. Ledger obligations

* `schemas/manifest/v1.json` — add the `incrementalCache` boolean property.
* `specs/18-schemas.md` — the canonical full config schema, which
  `specs/12-cli.md:812` names as the schema's owner. The field is defined here.
* `specs/12-cli.md` — the `Configuration (kali.json)` block at lines 810-856
  repeats the naming and omission rules "so CLI and schema docs do not drift",
  so it needs the omission rule: **an omitted `incrementalCache` means the cache
  is enabled**, which is the pre-existing behaviour. `AGENTS.md` §4 requires both
  updates to land with the behaviour change, not after it.
* `docs/superpowers/followups/computed-member-static-name-discovered-defects.md`
  §6 — a closing note naming this design and the commit that lands it, and
  **withdrawing §6's "needs new build machinery" claim**, which §3.1 falsifies.
* `crates/kali_cli/tests/inprocess/release_constant_condition_loop.rs:195-199` —
  the comment block asserting the key "carries no compiler-build identity at
  all" becomes false when this lands and must be corrected in the same commit,
  not left to rot.
* **A GitHub issue on `rahulmutt/kali`** tracking the mtime+len collision
  residual of §3.1, with the `soundness` label used by the existing open issue
  #18. To be filed when the work lands, with its number recorded back into
  §3.1's residual paragraph.

---

## 7. Risks, and what would falsify this design

**The fingerprint could be too strict to be useful.** If `current_exe()` resolves
differently between invocations of the same binary — a symlinked install, a
wrapper script, a relocated binary — every invocation gets its own namespace and
the cache never hits for anyone. The path component is what buys test-binary
separation and it is also what carries this risk. *Falsified by:* a real user
reporting a cache that never hits with an unchanged compiler. *Mitigation if it
happens:* drop the path component and keep len+mtime, which costs test-binary
separation but is still strictly correct for the shipped binary.

**The reaper could evict something hot.** Insertion-age eviction (§3.4) will
throw away a frequently-used entry that is simply old. The cost is one recompile.
*Accepted.*

**The 256-write sweep interval could be wrong in either direction.** Too rare and
a single long test binary outruns it; too frequent and every compile pays a
`readdir`. Nothing measured picked this number. *Falsified by:* a directory found
over budget after a normal suite run, or a measurable compile-time regression.

**`incrementalCache: false` on the fixtures could mask a real caching bug.** With
the fixtures declining the cache, no fixture-driven test exercises the cache path
at all, so a regression in caching would only be caught by §5's unit tests. This
is the price of §3.3 and it is why §5 pins the cache path directly in `TempDir`
roots rather than relying on the fixture suite to exercise it incidentally.
