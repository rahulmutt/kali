# CLI test cases

Each `.toml` here is one black-box test of the `kali` binary. Adding a case
compiles nothing — the single `cases` target discovers this tree at runtime.

```bash
cargo test -p kali_cli --test cases                     # everything
cargo test -p kali_cli --test cases -- switch/          # one family
cargo test -p kali_cli --test cases -- --ignored        # the 2 gated cases — see below
cargo test -p kali_cli --test cases -- --exact 'switch/runtime::a_single_case_switch_with_no_default_is_admitted'
cargo test -p kali_cli --test cases -- --list           # every trial id, no execution
```

As of Task 20 this tree is **287 case files expanding to 5,587 trials** (5,585
run, 2 `ignore = true`), against **68** hand-written `tests/*.rs` targets
including the runner itself.

This file is derived from the runner's source, not from the design prose:
`crates/kali_cli/tests/cases.rs` and `crates/kali_case_runner/src/{discover,
model,expand,steps,assertions,jsonpath}.rs`. The design rationale lives in
`docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md` §5; where
the two disagree, the code is what runs.

---

## How discovery works

`discover.rs` walks this directory recursively, sorted by file name, and takes
every file whose extension is `toml` (case-insensitively). The trial id is

```
<dir>/<file stem>[<axis=value,...>]::<case name>
```

so `cases/switch/runtime.toml`'s case `x` under no matrix is `switch/runtime::x`,
and the same case under `[matrix] ext = ["js","ts"]` becomes
`switch/runtime[ext=js]::x` and `switch/runtime[ext=ts]::x`. Directory nesting is
unbounded; the family is just the directory prefix, which is what makes
`-- switch/` work as a filter.

Today's families: `array/`, `browser/`, `math/`, `misc/`, `nullish/`, `object/`,
`runtime/`, `soundness/`, `string/`, `switch/`.

`ignore = true` marks a case that is registered and listed but not run — it
mirrors a `#[ignore]` the source carried, i.e. a known-broken behaviour the case
pins for the day it is fixed. There are exactly two:
`soundness/block_arrows.toml` and `soundness/r06_object_init.toml`. **Both fail
today**, on purpose, so `-- --ignored` reports `0 passed; 2 failed`; that is the
expected state, not a regression. `-- --include-ignored` therefore also goes red.

Discovery refuses rather than degrades. Each of these fails the whole target:

- the `cases` directory is missing, or is a file;
- **zero** case files found — "0 tests, ok" is a green CI run that tested
  nothing, so it is an error instead;
- two case files with the same stem (`pad.toml` and `pad.TOML`), which would
  collide as trial ids;
- any `.toml` in the tree that does not parse as a case file. (Non-`.toml`
  files — this README, for instance — are simply not collected.)

## File format

Four top-level sections. Only `[[case]]` is required.

```toml
[constants]                 # ${NAME} -> value, file-scoped
GREETING = "hello"

[matrix]                    # cartesian product; each cell is a separate trial set
ext = ["js", "ts"]

[source]                    # filename -> body, written into the trial's temp dir
"main.${ext}" = "console.log('${GREETING}')\n"

[[case]]                    # at least one; each is an independent test
name = "it_runs"            # required, unique within the file
rationale = """             # optional; printed on failure
Why this test exists, and what regression it pins.
"""
ignore = false              # optional; when true the trial is listed but skipped
args = ["run", "main.${ext}"]   # a single inline step
exit = "success"
stdout = "${GREETING}\n"
```

That example is not pseudocode: dropped into `cases/misc/` it expands to exactly
`misc/<stem>[ext=js]::it_runs` and `misc/<stem>[ext=ts]::it_runs` and both pass.

Every trial gets a **fresh temp dir** with `[source]` written into it, and runs
its steps in order; the first failing step ends the trial. `[source]` keys are
paths relative to the trial dir — an absolute path, or any `..` component, is
rejected *after* substitution, so `"${dir}/main.js"` cannot expand its way out.

### Steps

A case is either a single **inline** step (fields written directly on `[[case]]`,
as above) or an ordered `[[case.step]]` list sharing one temp dir. Mixing the two
in one case is an error, and so is a case with no step at all.

```toml
[[case]]
name = "build_then_read_the_manifest"
[[case.step]]
args = ["build", "main.js"]
exit = "success"
[[case.step]]
kind = "file_json"
path = "dist/manifest.json"
fields = { target = "browser", entries = 1 }
```

Three step kinds (`model.rs::StepKind`):

| `kind` | what it does |
| --- | --- |
| `cli` (default) | runs the `kali` binary with `args`, in the trial dir |
| `file_json` | reads `path` (relative to the trial dir) as JSON and checks `fields`; runs no process |
| `browser_bundle_harness` | writes `browser-bundle-smoke.mjs` from `entry` + `body` via `kali_runtime_contract::browser_bundle_harness_script`, runs it under the resolved harness command, and asserts on its output |

`kind` defaults to `cli` **only when no kind-specific field is set**. A step that
writes `path`/`fields` (`file_json`-only) or `entry`/`body`
(`browser_bundle_harness`-only) without spelling `kind` is a hard error, not a
silently-misread `cli` step. Fields that do not apply to a kind are rejected by
name, so a `file_json` step cannot carry `stdout_contains` and quietly assert
nothing.

### The twelve assertion keys

Closed vocabulary, `deny_unknown_fields`; a typo'd key fails the run rather than
asserting nothing.

| key | on | meaning |
| --- | --- | --- |
| `exit` | `cli`, `browser_bundle_harness` | `"success"`, `"failure"`, or an exact code (`exit = 2`) |
| `stdout` | " | exact equality |
| `stdout_contains` | " | every needle is a substring of stdout |
| `stdout_absent` | " | no needle occurs in stdout |
| `stdout_count` | " | `[{ needle = "3\n", at_least = 2 }]` — non-overlapping occurrences, exactly `str::matches` semantics |
| `stderr` | " | exact equality (this is the only way to pin "stderr is exactly empty") |
| `stderr_contains` | " | substring |
| `stderr_absent` | " | substring must not occur |
| `json` | " | stdout parsed as JSON; a nested table flattens to dotted paths, each compared to the leaf |
| `json_null` | " | dotted paths that must resolve to JSON `null` (TOML has no null literal, so `json` cannot express this) |
| `json_count` | " | `stdout_count` taken against a JSON *string* leaf: `[{ path = "stdout", needle = "ok", exact = 3 }]` |
| `fields` | `file_json` | the same dotted-path comparison, against the file at `path` |

Plus `args`, `env`, `path`, `entry`, `body` — inputs, not assertions.

Dotted paths are closed: `errors.0.code` walks objects by key and arrays by a
non-negative integer segment. No wildcards, slices, filters, or negative
indexing. A path that does not resolve is a **failure**, never a skip — the
message names which segment broke and why.

Count claims must spell exactly one of `at_least` / `exact`; both, neither,
`at_least = 0`, or an empty `needle` are parse errors, because each is a claim
nothing can fail. `exact = 0` is allowed — "this needle never appears" is
falsifiable.

### `[matrix]` expansion

Each axis is a named list; `expand.rs` takes the cartesian product of the axes
**sorted by axis name**, so trial ids are stable across runs. Every case in the
file is expanded once per cell, so trials = cases × ∏ axis lengths — a file with
2 cases and `ext = ["js","ts"]` yields 4 trials, not 2.

Inside a cell, `${axis}` and `${CONSTANT}` are substituted into source
filenames, source bodies, argv, env keys and values, every needle, every JSON
path, and every string leaf of `json`/`fields`. Those are the **only two forms**:
no conditionals, no expressions, no defaults. An unresolved `${...}` surviving
substitution is a hard failure, precisely because a placeholder left inside a
needle would match nothing and let the case pass having asserted nothing.

A matrix cannot vary the *shape* of an assertion — text output vs JSON output is
two sibling `[[case]]` blocks, not an axis.

---

## Two rules worth stating up front

- **Put the reason in `rationale`, not a `#` comment.** The runner prints
  `rationale` (indented under `  | `) when the case fails, alongside the step
  index, the argv, the env, and the full captured stdout/stderr. A comment
  explaining why a test exists is invisible exactly when someone needs it.
- **Pin diagnostic text through `[constants]`.** A hand-copied message prefix
  goes insensitive the moment the diagnostic widens. That has happened here
  before. A `[constants]` entry no `${NAME}` reaches fails
  `scripts/audit-case-migration.py` by name, so a hoisted constant cannot rot
  into free text silently either.

A case the format cannot express belongs in its own hand-written target, not in
a weakened `.toml`. See spec §5.11 for what stays Rust and why; today that is
`inprocess`, `runtime_smoke`, `package_corpus`, `schema_docs`,
`node_api_surface`, `browser_cdp_smoke`,
`browser_harness_failing_test_propagates_failure`, and the browser targets batch
8C retained.

---

## Where the gates live

- `bash scripts/test-gate.sh` — the full-workspace test gate; it enumerates every
  failing test rather than stopping at the first red binary. The runner's failure
  text is indented with **two** spaces and `  | ` on purpose: this script parses
  `^    [A-Za-z_]` as a failed-test name, so a four-space-indented detail line
  would be reported as a test that does not exist.
- `bash scripts/test-gate.sh --gates-only` — the 14 migration gates (case-corpus
  generators, fixture and citation sweeps, the audit's own regression suite). A
  bare invocation does not run them; CI's `migration-gates` job passes the flag.
- `python3 scripts/audit-case-migration.py <old.rs> <new.toml>...` — the
  migration audit, run when a hand-written target is replaced by case files.

### What the audit actually guarantees — and what it does not

It is a **literal-coverage** check, in one direction, over
*assertion-bearing strings only*. It extracts six claim kinds from the `.rs`
source (`.contains` literals, `const NAME: &str` rule constants, `assert_eq!`
string literals, `.matches(...).count()` needles, bracketed JSON keys, `.arg()`
tokens) and requires each to appear as a substring of the concatenated
assertion-bearing strings of the new case files — the whitelisted step keys plus
`[constants]` values that expansion actually reaches. Text in a `rationale`, a
`#` comment, a case `name`, a `[source]` body, or an unreferenced constant does
**not** satisfy a claim.

It does not check that the surviving claim means the same thing. Measured
directly against the shipped script:

| the source asserted | the case file wrote | audit |
| --- | --- | --- |
| `assert_eq!(stdout, "1\n")` | `stderr_contains = ["1\n"]` | **OK** — surface is not checked |
| `assert_eq!(stdout, "1\n")` | `stdout_contains = ["1\n"]` | **OK** — strength is not checked |
| `assert_eq!(stdout, "1\n")` | `stdout_contains = ["1\n2\n3\n"]` | **OK** — the literal is *contained*, not *demanded* |
| `assert_eq!(stdout, "1\n")` | the literal only in `rationale` | **FAILED** |

The only reverse-direction check is on `stdout_count` / `json_count`: a count
claim in a case file must correspond to a real `.matches(...).count()` assertion
in the source, needle *and* bound.

So: a clean audit means **no literal was dropped**. It does not mean the
migration is faithful. The hole was measured mechanically during the migration
and left open; spec §9 records it. Read a green audit as a floor on fidelity,
not a proof of it — and when you change a case, read its `rationale` rather than
trusting that the gate would catch a weakening.
