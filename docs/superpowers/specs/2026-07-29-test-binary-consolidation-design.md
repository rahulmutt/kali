# Test binary consolidation and file-driven case runner

Date: 2026-07-29
Status: approved design, not yet implemented

## 1) Problem

`target/` (redirected to `/workspace/.cache/cargo-target` by `.cargo/config.toml`)
is 79 GB. Measured composition:

| segment | size |
| --- | --- |
| `debug/deps` | 70 GB |
| `debug/incremental` | 8.7 GB |
| `debug/build` | 370 MB |

Within `debug/deps`, test and bin executables account for ~67 GB and `.rlib`
files for 2.7 GB.

`crates/kali_cli/tests/` declares **328 top-level integration test targets**
(401 `.rs` files including submodules), ~204k lines. **324 of 328 drive the CLI
black-box** via `CARGO_BIN_EXE_kali` and `std::process::Command`: write a source
file into a temp dir, run `kali run|check|build|test`, assert on exit status,
stdout, stderr, or the JSON output envelope.

### 1.1 The dominant cost is one import, not the binary count

A test binary is ~400–500 MB **if and only if** it references `kali_runtime::`
or `kali_cli::`. Both pull in wasmtime and cranelift with debuginfo
(`kali_cli`'s library depends on `kali_runtime`). Otherwise the binary is
7–19 MB. Measured:

| test target | references runtime/cli | size |
| --- | --- | --- |
| `switch_fail_closed` | no | 9 MB |
| `switch_runtime` | no | 9 MB |
| `browser_math_round` | no | 11 MB |
| `soundness_closures` | no | 9 MB |
| `runtime_forin` | no | 7 MB |
| `browser_math_exp_log_mixed_root` | `kali_runtime` | 401 MB |
| `browser_cdp_smoke` | `kali_runtime` | 406 MB |
| `runtime_smoke` | `kali_runtime` | 448 MB |
| `schema_validation` | `kali_cli` | 495 MB |

162 executables in `debug/deps` exceed 100 MB. Eight of those are not `kali_cli`
integration tests — the `kali` binary itself (545 MB, and stale build hashes
accumulate copies) and the in-crate unit-test binaries for `kali_cli` (533 MB),
`kali_embed` (503 MB), `kali_runtime` (442 MB), `kali_types` (109 MB), and
`kali_codegen` (104 MB). Those are out of scope here (§2) and constitute a
~2.6 GB floor this design does not move.

That leaves **154 fat `kali_cli` integration test targets** at ~410 MB each ≈
**63 GB**. The other 174 integration test targets total under 2 GB. 162 files
reference `kali_runtime::` textually; the difference is references appearing only
in comments.

Most of those 154 do not need the runtime at all. They import it for three
trivial items:

| symbol | uses | kind |
| --- | --- | --- |
| `BROWSER_HARNESS_COMMAND_ENV` | 334 | `pub const &str` |
| `browser_bundle_harness_script` | 106 | pure string builder |
| `browser_harness_command_parts_for` | 92 | pure `Vec<String>` builder |

Raw reference counts are misleading here and must be read per distinct symbol.
`runtime_smoke` has 200 `kali_runtime::` references but only **five distinct
symbols** — `browser_bundle_harness_script`, `BROWSER_HARNESS_COMMAND_ENV`,
`browser_harness_command_parts_for`, `BrowserRuntimeContract`,
`split_command_spec` — every one of which is in the §4.1 moved set. Likewise
`browser_cdp_smoke` (406 MB) references exactly one symbol,
`browser_bundle_harness_page`, which also moves. Both become thin binaries under
Phase 1 despite being among the largest today.

Exactly **three** test files reference symbols that stay behind in
`kali_runtime`, and they are the genuine floor:

| file | symbol it needs | why it cannot move |
| --- | --- | --- |
| `browser_harness_cdp_in_page_trap_propagates` | `browser_runtime_execute_checked` | lives in `browser/execute.rs`, which touches wasmtime and reqwest |
| `release_constant_condition_loop` | `RuntimeCtx` | executes wasm in-process to assert release-profile codegen |
| `release_mutated_binding_specialization` | `RuntimeCtx` | same |

A fourth, `schema_validation` (495 MB), uses `kali_cli::{build, output}` — the
CLI library, which depends on `kali_runtime`. It asserts over the JSON output
envelope and can invoke the `kali` binary as a subprocess instead, dropping to
~10 MB.

### 1.2 Secondary problem: authoring cost

The 204k lines are overwhelmingly boilerplate — `tempdir()`, `fs::write`,
`Command::new(kali_bin())`, `assert!(output.status.success(), ...)` — repeated
per case. Adding a test means writing Rust and compiling a new binary. The
assertion vocabulary is small and closed; measured across all 328 files:

| assertion | occurrences |
| --- | --- |
| JSON field equality | 2,671 |
| exit status success/failure | 836 |
| `stdout.contains(...)` | 523 |
| `stderr.contains(...)` | 418 |
| env vars set on the child | 217 |
| exit code equality | 182 |
| read + assert a produced file (mostly `app.meta.json`) | 114 |
| exact stdout equality | 30 |
| negated `contains` (absence claims) | 19 |
| file exists | 18 |
| `starts_with` / `lines()` | 33 |

## 2) Goals and non-goals

Goals:

1. Reduce `target/` from 79 GB to roughly 8 GB (`debug/deps` from 70 GB to
   ~6 GB), of which ~5 GB is a floor outside this design's scope (§7).
2. Reduce the number of compiled test targets in `kali_cli` from 328 to ~9.
3. Make a new CLI test a data file, not a Rust file and a new binary.
4. Migrate the repetitive majority of existing cases without weakening a single
   assertion.

Non-goals:

- Migrating tests that genuinely embed the runtime or drive a real browser.
- Touching the sibling `*_tests.rs` unit tests inside `src/` — they compile into
  their crate and are not a binary-count problem.
- Changing `kali_cli`'s `src/**` public behavior in any way.

## 3) Approach: two independent phases

Phase 1 removes wasmtime from the test binaries. It changes no assertion and is
verified by `cargo test -p kali_cli` passing with an unchanged test count. It
delivers ~95% of the achievable disk win.

Phase 2 builds a file-driven case runner and migrates families into it. Its
value is authoring ergonomics and deleting ~200k lines of boilerplate; the
remaining disk win is secondary.

The phases are sequenced this way deliberately: if Phase 2 stalls, Phase 1's
win is already banked, and Phase 2 becomes a quality decision made under no
disk pressure. That matters because migrating 204k lines of assertions is
precisely where meaning gets silently dropped, and this repository's history
records that failure mode twice on the switch work
(`crates/kali_cli/tests/switch_fail_closed.rs` header).

## 4) Phase 1 — unlink wasmtime from the test binaries

### 4.1 New leaf crate `kali_runtime_contract`

Four files move out of `kali_runtime` verbatim:

| moved file | contents | external symbols it needs |
| --- | --- | --- |
| `src/profiles.rs` | `RuntimeHostContract`, `RuntimeBackend`, `normalize_runtime_profiles` | `BTreeSet` |
| `src/browser/contract.rs` | `BROWSER_HARNESS_COMMAND_ENV`, `BrowserRuntimeContract`, `BrowserRuntimeContractDescriptor`, contract/diagnostic helpers | `Diagnostic`, `DiagnosticContext`, `DiagnosticContextOrigin`, `e5`, `serde_json::Value`, `BTreeSet`, `RuntimeBackend`, `RuntimeHostContract` |
| `src/browser/command.rs` | `browser_harness_command_parts*`, `split_command_spec` | `std::path::Path` |
| `src/browser/harness.rs` | `browser_bundle_harness_script`, `_page`, `_prelude`, `browser_runtime_harness_*`, `BROWSER_HARNESS_DONE_BINDING` | `serde_json`, `base64` |

Total ~1,780 lines. The crate's full dependency set is `kali_error` +
`serde_json` + `base64`; `kali_error` depends only on `serde` and `kali_common`.
Nothing reaches wasmtime, cranelift, reqwest, `kali_sandbox`, or `kali_api_*`.
`profiles.rs`'s three `wasmtime` matches are a doc comment and two string
labels.

`browser/execute.rs` and `browser/summary.rs` stay in `kali_runtime` — those
genuinely reference wasmtime and reqwest.

### 4.2 Compatibility

`kali_runtime` takes a dependency on `kali_runtime_contract` and re-exports
every moved symbol at its current path, so `kali_runtime::browser_bundle_harness_script`
keeps resolving and `crates/kali_cli/src/**` needs no edits.

The 162 test files get a mechanical `kali_runtime::` → `kali_runtime_contract::`
rewrite plus a dev-dependency line.

### 4.3 Known cost

All four moved files currently open with `use crate::*;` — a crate-wide glob.
Extraction requires writing explicit `use` lists for the first time. This is the
real work of Phase 1, and the step where a compile error is the desired
outcome rather than a silent behavior change. The sibling `*_tests.rs` files
move with their modules, per the repository's stated convention.

### 4.4 The four tests that link the runtime or the CLI library

This section originally proposed switching `schema_validation` to invoke the
`kali` binary as a subprocess, making it black-box like its 324 siblings and
dropping it to ~10 MB, on the premise that it used `kali_cli::{build, output}`
only to assert over the JSON output envelope. **That premise was wrong and the
conversion was not made.** All 11 of its tests call the validator functions
directly — `output::validate_envelope_value`,
`output::validate_package_audit_payload_value`,
`output::validate_package_effects_payload_value`,
`build::validate_build_result_value` — on hand-constructed values, most of them
malformed on purpose (backwards diagnostic spans, duplicate primary-artifact
roles, whitespace-padded hashes, unexpected keys) to prove the validator rejects
them. No CLI subcommand accepts arbitrary or malformed JSON, and the CLI's own
emitters cannot produce those shapes by construction, so none of the 11
assertions can be driven through a subprocess without weakening what it proves.
Adding a CLI validation entry point for external input would mean editing
`crates/kali_cli/src/**`, which Phase 1 forbids. The file was moved wholesale
into the consolidated in-process target instead, with no assertion, test body or
test name changed; `crates/kali_cli/tests/inprocess/schema_validation.rs` still
opens with `use kali_cli::{build, output};` today (§9, deviation 6).

The other three cannot be made thin — they need `RuntimeCtx` or
`browser/execute.rs` (see §1.1). Instead of three ~450 MB binaries, **consolidate
them into a single `tests/inprocess.rs` target** that `mod`-includes the three
existing files. Linking wasmtime once instead of three times is the entire point;
their contents are unchanged. As built, that target `mod`-includes four files,
`schema_validation` being the fourth.

`release_constant_condition_loop` and `release_mutated_binding_specialization`
additionally use `kali_cli::{build, ApiSurface}`, which is irrelevant to their
size once they already link `kali_runtime`.

`soundness_structured_clone` names `kali_cli` only in a comment and is already
thin.

### 4.5 Phase 1 result

| | before | after |
| --- | --- | --- |
| `kali_cli` integration test targets > 100 MB | 154 | 1 (`inprocess`, ~450 MB) |
| test targets in `kali_cli` | 328 | 326 |
| `debug/deps` | 70 GB | ~9 GB |

No test's assertions change. Verification is `cargo test -p kali_cli` passing
with an unchanged total test count.

## 5) Phase 2 — file-driven case runner

### 5.1 Layout

```
crates/kali_cli/tests/
  cases.rs                                  # the runner, ~5 modules + main
  cases/
    string/repeat_static_ascii.toml
    array/at.toml
    math/inverse_trig_identities.toml
    soundness/closures.toml
    switch/fail_closed.toml
    browser/math_exp_log_mixed_root.toml
```

Families are the existing filename prefixes: `string/`, `array/`, `math/`,
`object/`, `soundness/`, `switch/`, `browser/`, `package/`.

Test id is `<family>/<name>[<matrix>]::<case>`, which is what
`cargo test -- <filter>` matches on.

### 5.2 Case file schema

A case file has four optional top-level sections and one required one:

- `[constants]` — named string constants, referenced as `${NAME}`.
- `[matrix]` — named axes; the cartesian product expands into one trial set per
  cell. Referenced as `${axis}`.
- `[source]` — filename → file body, written into the trial's temp dir.
- `[[case]]` — required, at least one. Each is an **independent test** with a
  fresh temp dir and freshly written `[source]` files.
- `[[case.step]]` — an ordered sequence within one case, sharing that case's
  temp dir. Only needed when a later step depends on an earlier step's
  artifacts. A single-step case puts the step fields inline on `[[case]]`.

### 5.3 Step kinds

| `kind` | behavior |
| --- | --- |
| `cli` (default) | run `kali` with `args` in the trial dir |
| `file_json` | read `path` relative to the trial dir, assert dotted JSON paths (numeric segments index arrays -- see §5.4) in `fields` |
| `browser_bundle_harness` | generate the harness `.mjs` via `kali_runtime_contract::browser_bundle_harness_script(entry, ..., body)`, run it under the command from `browser_harness_command_parts_for`, assert on its output |

### 5.4 Assertion keys

Twelve assertion keys on a step, covering the full measured vocabulary of §1.2:

```toml
exit = "success" | "failure" | 2        # status class or exact code
stdout = "hahaha\n\n"                   # exact equality
stdout_contains = ["1\n", "0\n"]
stdout_absent   = ["E5506"]
stdout_count = [{ needle = "3\n", at_least = 2 }]   # occurrence count in stdout
stderr = ""                             # exact equality, symmetric with stdout
stderr_contains = ["..."]
stderr_absent   = ["..."]
json.payload.artifactKind = "bundle"    # dotted path into the stdout JSON envelope
json.errors.0.code = "E5506"            # a numeric segment indexes a JSON array
json_null = ["stdout", "stderr"]        # dotted paths that must be JSON null
json_count = [{ path = "stdout", needle = "3\n", exact = 6 }]  # the same count, in a JSON leaf
env = { KALI_BROWSER_BUNDLE_HARNESS_COMMAND = "node" }
```

A dotted-path segment that parses as a non-negative integer fitting a
`usize` indexes into a JSON array (`errors.0.code` reads `errors[0].code`);
against anything else -- an object, including one with a numeric-looking key
like `{"0": "x"}`, or a scalar -- a segment is always a plain key lookup,
numeric-looking or not. This is closed dotted-path indexing, not an
expression language: no slices, no wildcards, no negative-from-end indexing,
no filters, and only one segment is consumed per `.`. It exists specifically
so a case can pin *which* diagnostic is first (`errors.0.code = "E5506"`)
without asserting the rest of the diagnostic object, which is unmatchable in
this format -- every diagnostic carries a `"fix"` key that is unconditionally
JSON `null`, and TOML has no null literal capable of matching it, so whole-
array equality can never succeed. An out-of-range index, a non-numeric or
negative-looking segment against an array, and any other unresolvable
segment are all hard failures, never a silent skip.

`json_null` exists for the same reason `errors.0.code` exists: TOML has no
null literal (jsonpath.rs's `values_equal` hard-rejects every TOML type
against a JSON `null` by construction), so a claim like
`json["stderr"].is_null()` has no expressible form inside `json` itself --
not even via the dotted-path indexing above, since there is still no TOML
value to put on the right-hand side of `=`. `json_null` is a list of dotted
paths, checked against the same parsed stdout `json` is checked against,
each of which must resolve to present *and* JSON `null`; a path that does
not resolve at all is a hard failure, not a pass, for the identical reason
an absent `json` path is (§5.10 -- "not found" must never silently mean
"nothing to assert"). It was added during Task 15's fix round when a real,
reachable `check --output json`'s `stdout`/`stderr` null claim (from
`array_callback_find.rs`) had no other way to survive the migration without
either dropping the claim or asserting it on an unrelated case -- both
rejected. Because every diagnostic's `"fix"` key (previous paragraph) is
the same class of gap, `json_null` is expected to see broader use across
the remaining families in Tasks 16-19, not just this one site.

`stderr` was added during Task 16 batch 4's review round for the same
reason `json_null` was added during Task 15's: `stderr_contains` and
`stderr_absent` are both substring claims, and neither can express "stderr
is exactly this string" -- in particular "stderr is exactly empty," which
`soundness_block_arrows.rs`'s `anonymous_export_default_function_compiles_
and_runs` asserts directly (`stderr.is_empty()`, "expected no diagnostics").
A stray unrelated diagnostic on stderr would satisfy every `stderr_absent`
needle the source never wrote and still pass, silently weakening that
assertion during migration. `stderr` is exact equality against the step's
captured stderr, evaluated the same way `stdout` is; `file_json` steps
reject it for the same reason they reject `stdout` (they never run a
process). Symmetric with `stdout`'s existing exact-equality key, not a new
kind of claim.

`stdout_count` / `json_count` were added during Task 18 batch 4's interlude —
the third mid-migration addition, after `json_null` (Task 15) and `stderr`
(Task 16 batch 4), and for the same reason both of those were added: a real,
reachable source claim had no expressible form, and the only alternatives were
dropping it or weakening it. The claim is `haystack.matches(needle).count()`,
which appears 32 times across the unmigrated `browser_*.rs` files: 29 sites as
`count() >= n` and 3 as `count() == n`. (Those figures are a snapshot of the
survey that motivated the key, measured across `browser_*.rs` at `b189575556`;
they are not a live count, and they will not reproduce from a later tree as
batch 4 migrates the sites away. `browser_bundle_toplevel_start.rs`'s
`count() == 1` had already left the corpus by `f0bfb76d79`, migrated to an
exact `stdout` pin — a stronger form, per ruling 3 — which is the expected
direction of drift.) `stdout_contains` cannot carry it —
`contains` is satisfied by a *single* occurrence, so migrating `count() >= 2`
onto it silently weakens the claim to `count() >= 1`, and output that folded
two independently-computed constants into one emission would still pass the
migrated case while failing the original. Of the 18 files / 168 tests measured
as carrying an un-expressible claim across the 111 remaining `browser_*.rs`
files, **13 files are this count shape**; without the key they would all have
been retained hand-written under §5.11, against a §5.11 budget of ~8 retained
targets for the entire crate that the browser family had already overrun at 10
— directly against goal #2 ("328 compiled targets down to ~9"). The other two
un-expressible shapes (`.lines()` and `errors.iter().all/any`) remain retained;
this key does not attempt them.

The two keys are separate because both target surfaces are genuinely required:
a single migrated helper routinely asserts the *same* count on both branches of
its `--output json` split (`browser_math_log2_log10.rs:177-179` against
`json["stdout"].as_str()`, `:186` against the raw stdout), so a key covering
only raw stdout would leave half of every such helper hand-written. `json_count`
takes its count against the JSON string leaf at `path` in the same parsed stdout
`json`/`json_null` read; a path that does not resolve, *or resolves to a
non-string*, is a hard failure, never a silent pass — §5.10 again, the identical
rule `json_null` follows. (`json["stdout"]` is legitimately `null` in this
envelope, which is exactly why counting-zero-in-a-null would be the wrong
answer.)

Three semantics are fixed and load-bearing. **Counting is non-overlapping and
left-to-right**, because that is what Rust's `str::matches` does and every
migrated claim was written against it: `"aaa".matches("aa").count()` is 1, not
2. The evaluator delegates to `str::matches` rather than re-implementing the
scan, and a unit test pins the overlapping case specifically — an implementation
counting overlapping occurrences would silently *strengthen* every claim it
carries, passing on output the source assertion rejected. **The bound is closed
at `at_least` and `exact`**, exactly the two comparisons the corpus contains; no
`<`, `<=`, `>`, or `!=`, none of which any site spells. Exactly one of the two
must be set — a claim table with neither, or with both, is a parse error rather
than a claim compared against nothing — and `at_least = 0` is rejected as
vacuous (every output satisfies it), while `exact = 0` stays legal as a
falsifiable absence claim. **An empty `needle` is rejected at parse time**:
`str::matches("")` matches at every character boundary and yields `len + 1`,
a number no author writing `count() >= 2` ever meant, and special-casing it to
0 instead would diverge from the `str::matches` semantics the migrated claims
depend on. `file_json` steps reject both keys for the same reason they reject
`stdout_contains` and `json` — they never run a process.

Two non-assertion keys live on a `[[case]]`, not a step: `name` and `rationale`,
plus `ignore = true` to run only under `--ignored` (the 9 currently-ignored
tests, including the CDP ones). `file_json` steps take `path` and `fields`
instead of the stdout/stderr keys; `browser_bundle_harness` steps take `entry`
and `body` alongside them.

The `starts_with` / `lines()` outliers (33 sites) are not in the vocabulary.
Cases needing them stay hand-written Rust.

### 5.5 Rationale is a field, not a comment

`switch_fail_closed.rs` carries ~80 lines of commentary explaining why each cell
denies by a specific rule and how the test previously degraded. As comments,
that text is invisible when the test fails. As a `rationale` field the runner
prints it on failure.

`[constants]` replaces the per-file `const RULE_*: &str` declarations at equal
fidelity, and `${...}` substitution keeps the "rule literals are never
hand-rolled" discipline enforceable: a bare string where a constant belongs is
greppable. Scoping stays per-file, matching today. A shared
`cases/_constants.toml`, merged into every case file, is available if
cross-family pinning is later wanted; it is not part of this design.

### 5.6 Worked example

```toml
# crates/kali_cli/tests/cases/browser/math_exp_log_mixed_root.toml
[matrix]
ext = ["js", "ts", "jsx", "tsx"]

[source]
"app.${ext}" = '''
// kali-tree-shake: mixedRootExpLog
export function mixedRootExpLog() {
  console.log(globalThis.Math["exp"](0));
  console.log(globalThis.Math["log"](1));
}
'''

[[case]]
name = "bundle_and_harness"
rationale = """
Pins that a mixed bracketed/dotted `globalThis.Math` root still folds exp/log
identities in a browser bundle, and that the emitted bundle actually executes.
"""

  [[case.step]]
  kind = "cli"
  args = ["build", "--bundle", "--api", "browser", "app.${ext}"]
  exit = "success"

  [[case.step]]
  kind = "file_json"
  path = "app/app.meta.json"
  fields = { apiSurface = "browser", artifactKind = "bundle" }

  [[case.step]]
  kind = "browser_bundle_harness"
  entry = "app"
  body = "await mod.mixedRootExpLog();"
  stdout_contains = ["1\n", "0\n"]

[[case]]
name = "bundle_and_harness_json"
rationale = "Same bundle, asserting the JSON output envelope rather than text."

  [[case.step]]
  kind = "cli"
  args = ["--output", "json", "build", "--bundle", "--api", "browser", "app.${ext}"]
  exit = "success"
  json.schemaVersion = 1
  json.command = "build"
  json.success = true
  json.exitCode = 0
  json.payload.artifactKind = "bundle"
  json.payload.bundleFormat = "esm"

  [[case.step]]
  kind = "browser_bundle_harness"
  entry = "app"
  body = "await mod.mixedRootExpLog();"
  stdout_contains = ["1\n", "0\n"]
```

Expands to 8 trials, e.g.
`browser/math_exp_log_mixed_root[ext=tsx]::bundle_and_harness_json`.

Note what this example does **not** do: text-vs-JSON output is *not* a `[matrix]`
axis. Varying output shape changes both the argv and the assertions, and §5.7
admits no conditionals, so it is expressed as two sibling `[[case]]` blocks. A
matrix axis is only for variation that substitutes uniformly — source extension,
source-file name, a numeric literal. This is a deliberate limit: it keeps
expansion a pure product and keeps every case readable in full.

### 5.7 Substitution

Exactly two forms: `${matrix_axis}` and `${CONSTANT}`. No conditionals, no
expressions, no arithmetic. Substitution applies to source filenames, source
bodies, argv elements, env values, and expected strings. An unresolved `${...}`
remaining after substitution is an error.

Anything needing more expressiveness stays hand-written Rust. That is the
escape hatch; there is no inline-code facility in the format.

### 5.8 Runner

Target definition in `crates/kali_cli/Cargo.toml`:

```toml
[[test]]
name = "cases"
path = "tests/cases.rs"
harness = false
```

Dependencies added, all pure Rust per the no-C/C++ hard invariant:
`libtest-mimic`, `toml`, plus existing `serde`, `serde_json`, `tempfile`, and
the new `kali_runtime_contract`. Nothing reaches wasmtime; the binary is
expected around 10 MB.

`cases.rs` is five modules and a `main`:

1. **discover** — walk `concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cases")` for
   `*.toml`, sorted, so trial order is deterministic.
2. **parse** — serde into a typed model with `deny_unknown_fields` on every
   struct. The unknown-key hard error of §5.10 is therefore a property of the
   deserializer, not a hand-written check.
3. **expand** — cartesian product of `[matrix]` axes with substitution applied.
4. **run** — per trial: temp dir, write `[source]`, execute steps in order.
   `kali` is located via `env!("CARGO_BIN_EXE_kali")`, which Cargo sets for
   `harness = false` test targets as for ordinary ones.
5. **assert** — evaluate the twelve keys; the first failure ends the trial.

`libtest-mimic` provides filtering, `--ignored`, parallel execution across
trials, and `--format terse|pretty`. Each trial owns its temp dir, so
parallelism is safe. `cargo-nextest` continues to work: it re-invokes the
binary per test, and `libtest-mimic` implements the list/filter protocol that
requires.

### 5.9 Failure output

On failure the runner prints, in order: case file path and step index, the
expanded matrix cell, the case's `rationale`, the exact argv and env, then the
full stdout and stderr. A red test in CI should be diagnosable without opening
the `.toml`.

### 5.10 Framework error handling

Every one of these is a hard failure, never a skip:

- malformed TOML, or an unknown key — error naming the file and key
- a `${...}` unresolved after substitution
- a case file with zero `[[case]]` entries — a silently-passing file is worse
  than a missing one
- **zero case files discovered** — otherwise a wrong discovery path reports
  `0 tests, ok` and turns CI green
- a `[matrix]` axis with zero values

### 5.11 What stays hand-written Rust

~8 targets, contents unchanged:

| target | why it stays code | size after Phase 1 |
| --- | --- | --- |
| `inprocess` (§4.4) | needs `RuntimeCtx` / `browser/execute.rs` | ~450 MB |
| `runtime_smoke` | 6,573 lines of imperative multi-step fixtures | ~12 MB |
| `browser_cdp_smoke` | drives a real Chromium via `tests/cdp_driver` | ~12 MB |
| `browser_harness_failing_test_propagates_failure` | asserts harness failure propagation | ~11 MB |
| `package_corpus` | multi-module corpus fixtures | ~19 MB |
| `schema_docs` | asserts over the schema/doc trees, not the CLI | ~14 MB |
| `node_api_surface` | multi-module surface enumeration | ~16 MB |
| outliers | the `starts_with` / `lines()` sites, if they do not fit §5.4 | ~10 MB |

Note that `runtime_smoke` and `browser_cdp_smoke` stay hand-written because they
are *shaped* like code — imperative sequences and a live CDP session — not
because they are large. Phase 1 already made them thin (§1.1), so leaving them
as Rust costs almost nothing. The single ~450 MB `inprocess` target is the floor.

## 6) Migration and verification

### 6.1 Order

By risk, not by size — a property the phase split buys, since disk pressure is
resolved before Phase 2 begins:

1. `string/` (~20 files, most formulaic) — proves the format end to end
2. `array/`, `math/`, `object/`
3. `soundness/`
4. `switch/` — prose-heavy, needs care
5. `browser/` — largest, and the only family exercising all three step kinds

One family per commit.

### 6.2 The audit gate

No `.rs` file is deleted until its family's audit is clean. A one-shot audit
script extracts from each old `.rs` file every:

- literal string argument to `.contains(...)` and `!....contains(...)`
- `assert_eq!(json[...]["..."], <value>)` path/value pair
- `.arg(...)` argv sequence
- `#[test]` function name

and checks each appears in the corresponding case file's expanded trial set.
**Any literal present in the old test but absent from the new case file fails
the audit.**

This catches wholesale drops and quiet weakenings — for instance
`contains("E5506")` surviving while the pinned rule constant vanishes — without
asking a reviewer to diff 6,000 lines by eye.

Both suites are green in the same commit before the old file is deleted, so a
family's migration is revertible as one commit.

### 6.3 CI surface

The test surface itself is unchanged in shape: `cargo test -p kali_cli` runs
everything, `mise run browser-smoke` still points at `--test browser_cdp_smoke`,
and `mise run determinism` is untouched.

**The workflow ships unchanged in shape after all.** A `migration-gates` job was
added to `.github/workflows/ci.yml` during this branch and removed again before
merge; `git diff main -- .github/workflows/ci.yml` is a comment block and nothing
else. The comment stands where the job stood and says why there is no job.

The 14 migration gates behind `bash scripts/test-gate.sh --gates-only` are
therefore a **developer command, not a CI gate** — nothing under `.github/` runs
them. The job could not have passed: it installed no Rust toolchain and ran no
`cargo build`, while two of the 14 gates (`tools/migration/gen_task19_batch4.py`
and `gen_task19_batch5.py`) run the compiled `kali` binary and fail rather than
skip when it is absent. It was removed rather than repaired because the migration
is finished and frozen at merge and the corpus those gates guard does not change
again. See §9, deviation 7. A bare `bash scripts/test-gate.sh` is unchanged from
base either way.

## 7) Expected outcome

| | `kali_cli` integration tests | out-of-scope floor | rlibs | `deps` total |
| --- | --- | --- | --- | --- |
| now | 154 × ~410 MB + 174 × ~10 MB ≈ 65 GB | ~2.6 GB (`kali` bin + 5 unit-test bins) | 2.7 GB | 70 GB |
| after Phase 1 | 1 × ~450 MB + 325 × ~10 MB ≈ 3.7 GB | ~2.6 GB | 2.7 GB | **~9 GB** |
| after Phase 2 | 1 × ~450 MB + 7 × ~13 MB + runner ~10 MB ≈ 0.55 GB | ~2.6 GB | 2.7 GB | **~5.9 GB** |

`debug/incremental` also shrinks, since most of its 8.7 GB is 328 separate test
compilation units; no precise post-migration figure is claimed here, because it
depends on build history rather than on tree shape.

Whole-tree estimate: **79 GB → ~11 GB after Phase 1 → ~8 GB after Phase 2**.

After Phase 2 the largest remaining items are all outside this design's scope:
2.7 GB of wasmtime/cranelift rlibs, ~2.6 GB of `kali` and in-crate unit-test
binaries that legitimately link the runtime, and ~0.45 GB for the one
consolidated in-process target. Shrinking those further is a separate question —
the most promising lever would be `debug = 1` or `split-debuginfo` on the dev
profile, which is orthogonal to test structure and is **not** proposed here.

Test targets in `kali_cli`: **328 → ~9**. Lines of test boilerplate deleted:
~200k.

Phase 1 therefore delivers ~95% of the achievable disk win; Phase 2 recovers
~3 GB more. Phase 2's case is primarily authoring ergonomics and ~200k deleted
lines, not disk.

## 8) Risks

| risk | mitigation |
| --- | --- |
| Extraction changes behavior via the `use crate::*;` rewrite | Phase 1 changes no assertions; `cargo test -p kali_cli` must pass with an unchanged test count |
| A migrated case silently asserts less than its predecessor | §6.2 audit gate; no deletion before a clean audit |
| The format itself becomes a degradation vector (typo'd key asserts nothing) | `deny_unknown_fields`; §5.10 hard failures including the zero-cases and zero-files cases |
| Format pressure to grow a DSL | Substitution is closed at two forms (§5.7); anything more stays hand-written Rust |
| Loss of per-target `cargo test --test X` granularity | Test ids are path-prefixed (§5.1), so `cargo test -- switch/` replaces `--test switch_fail_closed` |
| Misjudging which tests must stay fat, by counting `kali_runtime::` references instead of distinct symbols | §1.1 enumerates the three files by the specific symbol each needs; the moved set is fixed by §4.1. This error was made and corrected once while drafting this spec — `runtime_smoke` looks like the worst offender at 200 references and is in fact fully migratable |
| `[matrix]` pressure to express non-uniform variation | Axes substitute uniformly only; output-shape and other assertion-changing variation uses sibling `[[case]]` blocks (§5.6) |

## 9) Outcome as built

Every figure below was measured in Task 20 from the command shown, on a cold
target dir (`cargo clean`, then `cargo test -p kali_cli --no-run`, 2m37s wall on
24 cores). The raw output of each is in
`.superpowers/sdd/2026-07-29-test-binary-consolidation/task-20-report.md`.

| | predicted | actual | command |
| --- | --- | ---: | --- |
| `debug/deps` | ~5.9 GB (§7) | **5.2 GB** | `du -sh .cache/cargo-target/debug/deps` |
| whole tree | ~8 GB (§7) | **7.2 GB** | `du -sh .cache/cargo-target` |
| `kali_cli` test targets | ~9 (§7) | **68** hand-written `tests/*.rs` (incl. `cases`) + 2 unit-test binaries | `ls crates/kali_cli/tests/*.rs \| wc -l` |
| case files | — | **287** | `find crates/kali_cli/tests/cases -name '*.toml' \| wc -l` |
| expanded trials | — | **5,587** (5,585 run, 2 `ignore = true`) | `cargo test -p kali_cli --test cases -- --list \| grep -c ': test$'` |
| test lines deleted | ~200k | **85,409** deleted / 3,823 added in `.rs`, 112,347 added in `.toml` | `git diff --numstat --no-renames main -- crates/kali_cli/tests` |

Targets over 100 MB in `debug/deps` of those this command builds, all four of
them (the target dir is shared workspace-wide, not scoped to `kali_cli`; a
`cargo test --workspace --no-run` sweep finds four more, `kali_codegen`'s,
`kali_embed`'s, `kali_runtime`'s and `kali_types`' unit-test binaries):

```
545 MB  kali-09e739f15cd9e23d          the `kali` binary itself
539 MB  inprocess-ccaa0ce7a6d76999     the one consolidated in-process target (§4.4)
532 MB  kali_cli-4c03baab1f9515dc      unittests src/lib.rs
504 MB  kali-e863de0af5af1a1b          unittests src/bin/kali.rs
```

`debug/deps` accounts fully as: those four at 2.07 GB, the other 67 integration
test binaries at 0.87 GB (mean 13.3 MB, max 86.6 MB `runtime_smoke`, min 6.7 MB,
`cases` itself 35.0 MB), 11 proc-macro `.so` at 0.10 GB, 297 rlibs at 1.81 GB,
297 rmetas at 0.33 GB. The remaining ~2.0 GB of the tree is `debug/build`,
`debug/incremental` and the top-level artifacts.

### Where the predictions held, and where they did not

**Disk held.** 5.2 GB against §7's ~5.9 GB and 7.2 GB against ~8 GB are both
inside 12%. *(Note for anyone re-reading the plan: the plan's Task 20 quotes the
deps prediction as "~3.9 GB". That is a misquote of this section, which says
~5.9 GB. Measured against the plan's figure the miss would look like +33%;
against the spec's own figure it is −12%.)*

**"~9 test targets" missed by 7×, and the reason is a scoping error in §7, not a
shortfall in the migration.** 68 `tests/*.rs` remain. §5.11 named ~8 targets that
would stay hand-written and §7 turned that into the total; but §5.11's list only
ever covered the targets that stay *because of what they do*. It never accounted
for the browser family (**21** `browser_*` targets, mostly retained by the
batch-8C classification), the six `clbg_*` benchmark-runtime targets, the five
`late_compat_*` targets, or the U4 trim-and-keep sources whose migratable
`#[test]` fns moved out while the rest stayed. The 42 sources Task 19 deleted are
exactly the sources that were *fully* migrated and carried no retention; the
other 51 on-disk sources were never claimed by a case file at all. Both facts are
reproducible: `python3 tools/migration/t19_deletion_classify.py --ref 8ba0b64593`
prints `delete=42 retain=17 not_migrated=51 total=110`.

**"~200k lines deleted" missed by 2.4×, and the ~200k figure was never the
deletable population.** `crates/kali_cli/tests` held 203,638 lines of `.rs`
across 387 files at `main`; it holds 122,052 across 110 files now. So 200k was
the *size of the directory*, not the size of what could be deleted. What actually
happened: 85,409 `.rs` lines deleted, 3,823 added, and 112,347 lines of `.toml`
added — the corpus grew by 30,761 lines net. Case files are more verbose per
assertion than the Rust they replace, because they carry the `Migrated from`
provenance headers, the `rationale` fields §5.5 asked for, and inline `[source]`
fixtures that the Rust sources shared through helpers. **Phase 2's win is
authoring cost and blast radius, not line count**: 5,587 trials now compile
nothing, and `runtime_smoke` alone — a §5.11 retention — is 73,815 of the 122,052
lines that remain.

### Deviations from the design

1. **Families.** §5.1 predicted `string/ array/ math/ object/ soundness/ switch/
   browser/ package/`. Built: `array/ browser/ math/ misc/ nullish/ object/
   runtime/ soundness/ string/ switch/`. No `package/` family exists (the package
   corpus stayed hand-written per §5.11); `misc/`, `nullish/` and `runtime/` were
   added during migration.
2. **`cases` is 35 MB, not ~10 MB** (§5.8's estimate). It links `kali_case_runner`,
   `kali_runtime_contract`, `tungstenite`, `tar`, `flate2` and `base64` through
   `kali_cli`'s dev-dependencies. That makes it the second-largest of the 67
   non-fat test binaries, behind `runtime_smoke` at 86.6 MB; nothing here
   depends on the estimate.
3. **`inprocess` is 539 MB, not ~450 MB** (§5.11). The floor moved up ~20%; §7's
   conclusion that it is the floor is unchanged.
4. **The `Case` / `deny_unknown_fields` rough edge was resolved differently than
   the plan's fallback.** `model.rs` routes the inline-step shorthand through a
   `toml::Table` residual and a hand-written `finalize_step`, because
   `#[serde(flatten)]` silently ignores `deny_unknown_fields` on the *flattened*
   type as well (serde#1600), not merely on the container. `finalize_step` also
   grew a rule the design did not anticipate: `kind` defaults to `cli` only when
   no kind-specific field is set, so a forgotten `kind =
   "browser_bundle_harness"` is an error rather than a silently-ignored
   `entry`/`body`.
5. **Twelve assertion keys, not the eight of an earlier draft.** §5.4 already says
   twelve; `stdout_count`, `stderr`, `json_null` and `json_count` were each added
   during migration because a real source assertion had no expressible form
   without them. Their doc comments in `model.rs` record which one.
6. **`schema_validation` did not become a subprocess test.** §4.4 prescribed
   converting it to invoke the `kali` binary; Task 7 found the premise false and
   folded the file into the `inprocess` target unchanged instead. All 11 tests
   call `output::validate_envelope_value`,
   `output::validate_package_audit_payload_value`,
   `output::validate_package_effects_payload_value` and
   `build::validate_build_result_value` directly on deliberately-malformed
   hand-constructed values; no CLI subcommand accepts arbitrary JSON, the
   emitters cannot produce those shapes, and adding an entry point would have
   required editing `crates/kali_cli/src/**`, which Phase 1 forbids. So the
   fallback was not merely easier, it was the only compliant path.
   `crates/kali_cli/tests/inprocess/schema_validation.rs` still names
   `kali_cli::{build, output}`, and `inprocess` therefore carries four suites,
   not three. The reasoning was recorded in Task 7's amended message, which
   lived in git-ignored scratch and does not ship; it is restated here in full
   rather than cited to a path a clean checkout cannot resolve. §4.4 has been
   corrected.
7. **A `migration-gates` CI job was added and then removed before merge; CI
   ships with no job added.** The job was a `fetch-depth: 0` checkout (needed
   because `citation_sweep.sh` resolves a deleted source's citations against a
   historical blob) followed by `bash scripts/test-gate.sh --gates-only`. It
   **never ran and could not have passed**, for three compounding reasons:
   it installed no Rust toolchain and had no `cargo build` step; two of the 14
   gates, `tools/migration/gen_task19_batch4.py` and `gen_task19_batch5.py`,
   invoke the compiled `kali` binary and fail rather than skip when it is absent
   (batch 4 raised `GenError("the U2 policy control cannot run: no kali
   binary")`; batch 5 had no existence check at all and died with a bare
   `FileNotFoundError`); and the path both resolved,
   `$REPO/.cache/cargo-target/debug/kali`, comes from a machine-local
   `~/.cargo/config.toml` in the dev container — there is no `.cargo/` directory
   in this repository, so a runner's cargo builds to `./target` and that path is
   wrong regardless of any build step. Reproduced against the pre-removal
   generators, both arms: with `CARGO_BIN_EXE_kali` unset and
   `KALI_BIN=/nonexistent`, `gen_task19_batch4.py` exited 1 on
   `GenError: the U2 policy control cannot run: no kali binary`; with the
   container's binary moved aside, `gen_task19_batch5.py` exited 1 on
   `FileNotFoundError: [Errno 2] No such file or directory:
   '/workspace/.cache/cargo-target/debug/kali'`, a raw traceback. `run_gates`
   sets `fail=1` on any non-zero rc (`scripts/test-gate.sh`, the `(( rc )) &&
   fail=1` line), so either one takes the whole job to exit 1.

   The human's decision was to **remove the job**, not to add a toolchain and a
   build: the migration is finished and frozen at merge, so the corpus those
   gates guard does not change again and there is nothing for a per-PR run to
   catch. `--gates-only` stays as a documented developer command; the gate set
   in `scripts/test-gate.sh` is unmodified, and a bare `scripts/test-gate.sh` is
   still exactly what it was at base. Two follow-through changes went with the
   removal, because a developer command that only works on one machine is not a
   command anybody else can run:
   - **The binary is now resolved, not hardcoded.** `tools/migration/kali_bin.py`
     walks `$CARGO_BIN_EXE_kali`, `$KALI_BIN`, `$CARGO_TARGET_DIR/debug/kali`,
     `cargo metadata --format-version 1 --no-deps`'s `.target_directory`, then
     `<repo>/target/debug/kali`, and raises naming **every** candidate it tried
     and why each failed. Both generators use it; `gen_task19_batch5.py`'s
     silent hardcoded path and its missing existence check are gone.
   - **It is documented as not-CI.** `crates/kali_cli/tests/cases/README.md` and
     `tools/task-18-browser-pilot/README.md` say the gates are run by hand, and
     why. `scripts/test-gate.sh`'s own header still says the `migration-gates`
     job invokes `--gates-only`; that sentence is now **stale and wrong**, and it
     was left alone only because that file is under the branch's do-not-modify
     constraint. Whoever owns that constraint should correct it.

   §6.3 has been corrected to describe the shipped state.

### What the audit gate (§6.2) does and does not guarantee

Stated here because it is the control the whole migration leaned on, and it is
weaker than "the migration is faithful". `scripts/audit-case-migration.py`
extracts six claim kinds from the `.rs` source and requires each literal to
appear as a **substring** of the case files' *assertion-bearing strings* — the
whitelisted step keys plus `[constants]` values that expansion actually reaches.
Prose does not satisfy a claim: a literal present only in a `rationale`, a `#`
comment, a case `name` or a `[source]` body fails the audit, and so does an
unreferenced `[constants]` entry.

But the check is one-directional and surface-blind. Measured in Task 20 against
the shipped script, with a source asserting `assert_eq!(stdout, "1\n")`:

| the case file wrote | audit |
| --- | --- |
| `stderr_contains = ["1\n"]` | **OK** — the surface is not checked |
| `stdout_contains = ["1\n"]` | **OK** — the strength is not checked |
| `stdout_contains = ["1\n2\n3\n"]` | **OK** — the literal is *contained*, not *demanded* |
| the literal only in `rationale` | **FAILED** |

The only reverse-direction check is on `stdout_count`/`json_count`, where a case
file's claim must correspond to a real `.matches(...).count()` in the source,
needle and bound. Task 19 batch 5 measured this hole mechanically on its own
family and left it open; a green audit is a floor on fidelity, not a proof of it.

### Verification at the end of Task 20

```
bash scripts/test-gate.sh                                 GATE OK: 0 failing tests
bash scripts/test-gate.sh --gates-only                    14/14, MIGRATION GATES OK
bash scripts/check-determinism.sh                         exit 0 (but see below)
cargo test -p kali_cli --test cases                       5585 passed; 0 failed; 2 ignored
cargo test -p kali_cli --test browser_cdp_smoke -- --ignored   5 passed; 0 failed
```

`scripts/check-determinism.sh` exits 0 while running **zero** tests: each of its
20 `--exact` filters names an unqualified fn (`build_artifacts_are_deterministic_
across_repeated_invocations`), but every one of those fns lives in a
`runtime_smoke/` submodule and libtest names it `build::build_artifacts_...`, so
each invocation reports `0 passed; 1829 filtered out`. This is **pre-existing on
`main`** — `runtime_smoke.rs`'s eight `#[path]` submodules predate this branch
and the script is byte-identical to `main`'s — and is recorded rather than fixed
because `scripts/check-determinism.sh` is under this project's do-not-modify
constraint. It should be fixed by whoever owns that constraint. Tracked at
`docs/superpowers/followups/test-binary-consolidation-determinism-lane.md`.
