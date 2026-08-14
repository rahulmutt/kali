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

**A filter that matches nothing is a hard error, not a green run.** `-- switch/`
with the family misspelled used to print `0 passed; 0 failed; 5587 filtered out`
and exit 0; `refuse_empty_selection` (`discover.rs:140`) now fails the run
instead:

```
filter `swtich/` matched 0 of 5587 trials -- refusing to report a green run over
zero tests. Check the spelling (trial ids are `<family>/<file>::<case>`;
`--list` prints them all).
```

Only a *selector* triggers it — a filter or a `--skip`. Running everything is
never refused, `--list` is not a test run, and `--ignored` alone finding nothing
is an answer rather than a mistake. See *Four ways a case file is refused*
below for the three refusals that fire earlier still, while the file is parsed.

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
- two case files with the same directory-prefixed stem (`switch/pad.toml` and
  `switch/pad.TOML`), which would collide as trial ids — a bare stem repeating
  across directories, like `array/join.toml` and `runtime/join.toml`, does not;
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

**`[source]` is file-wide, not per-case.** `expand()` builds the source map once
per matrix cell and `source.clone()`s it into *every* trial of that cell, so a
`[[case]]` cannot opt out of a fixture, cannot get a different body for a
filename a sibling case also writes, and cannot depend on a file being *absent*
that a sibling needs present. Merging two tests that both write `main.js` with
different programs into one file does not fail — the last body written wins for
every trial, and the cases quietly stop discriminating. Nothing catches it: no
literal is dropped, so the migration audit stays green, and the trials still
pass. Two ways out, and the corpus uses both:

- **Separate case files.** Only a separate file has its own `[source]` table.
  This is the only option when a case's whole point is that some file is missing.
- **Distinct filenames.** Give each variant its own key and track the rename
  everywhere the name is *read* — argv, a `file_json` `path`, a
  `browser_bundle_harness` `entry` (which names the bundle output directory after
  the input stem). `cases/nullish/assignment_wrapped_local_binding.toml` does
  this the way the migration rules prescribe (`U5`): every key is the source
  `#[test]` fn's own name plus the original suffix chain, so `main.test.js`
  becomes `json_test_supports_..._js_input.test.js`. Keep the *whole* suffix
  chain — `kali test` dispatches on the `.test.` infix, and a filename whose
  shape the tool reads is not free to rename. Nor is one the program itself
  references by string: renaming an `import()` or `require()` specifier changes
  the program under test.

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

That hard failure is also why a fixture containing a **real JS template
literal** needs an escape: `substitute()` (`crates/kali_case_runner/src/expand.rs`)
finds every `${`, demands a `}`, and errors on any name it cannot bind, so
`${7 / 2}` in a `[source]` body would abort the trial. There is no special form
for this. The escape is an ordinary `[constants]` entry — `dollar = "$"` — put
through the generic substituter, so `${dollar}{` expands to a literal `${` and
the program written to disk is byte-for-byte what the test means to run.
`cases/browser/bundle_template_literal_interpolation.toml` is the live example:
line 8 defines it under `[constants]`,

```toml
[constants]
dollar = "$"
```

and line 11 spends it in the fixture,

```toml
[source]
"app.ts" = "console.log(`v: ${dollar}{7 / 2}`);\n"
```

The program that lands in the trial dir is the interpolating one the test means
to compile:

```js
console.log(`v: ${7 / 2}`);
```

**15** case files carry a `dollar` constant — the number of files whose parsed
`[constants]` table actually has the key, which is the only count that means
anything here:

```bash
$ cd "$(git rev-parse --show-toplevel)"
$ python3 -c 'import tomllib,glob; print(sum("dollar" in (tomllib.load(open(f,"rb")).get("constants") or {}) for f in glob.glob("crates/kali_cli/tests/cases/*/*.toml")))'
15
$ grep -rlF '${dollar}' --include='*.toml' crates/kali_cli/tests/cases/ | wc -l   # 39
```

The second figure is **39**, and this paragraph used to quote it as if it were
the first. It is not: it counts every file that *mentions* `${dollar}`
anywhere, and the extra 24 mention it only in `#` header prose — almost all of
them saying they need no such entry. Grep the header and you count the
discussion; parse the file and you count the constant.

Escape a genuine `${`; never delete or reword one to get past the substituter,
because that ships a different program than the one the test claims to cover.

A matrix cannot vary the *shape* of an assertion — text output vs JSON output is
two sibling `[[case]]` blocks, not an axis.

---

## Four ways a case file is refused

Three of these fire while the file is parsed, before a single trial runs; the
fourth fires at discovery. All four exist for one reason: each shape used to
produce a **green run that checked nothing**, which is the failure this whole
format was built to make unrepresentable. None of them is a lint you can
override.

**1. A step that asserts nothing.** A `cli` or `browser_bundle_harness` step
with none of the eleven assertion keys set would spawn `kali`, discard the
result, and pass unconditionally — including when the binary crashed:

```
case `c`: step (kind = "cli") declares no assertion -- it would run the command,
discard the result, and pass unconditionally. Set at least one of `exit`,
`stdout`, `stdout_contains`, `stdout_absent`, `stdout_count`, `stderr`,
`stderr_contains`, `stderr_absent`, `json`, `json_null`, `json_count`.
```

You reach it by deleting the last assertion while editing a case, or by writing
`args = [...]` and forgetting `exit = "success"`. `file_json` is exempt: its one
assertion key is `fields`, and a `file_json` step without it is refused where it
runs instead. `finalize_step`, `model.rs:409`.

**2. A dead `[matrix]` axis.** An axis nothing substitutes multiplies every case
by its length into byte-identical trials — the suite reports more tests than it
has:

```
matrix axis `ext` is never referenced as `${ext}` -- it multiplies every case by
its length into byte-identical trials, so the suite would report more tests than
it has
```

Renaming `${ext}` to `${extension}` in the cases and leaving the old axis behind
is the way this happens.

**3. A dead `[constants]` entry** — see *Pin diagnostic text through
`[constants]`* below for why this one is not merely tidiness:

```
constant `UNUSED_NOTE` is never referenced as `${UNUSED_NOTE}` -- an unread
constant is still read by the migration audit's assertion haystack, so it is a
channel for satisfying a claim the runner never checks. Delete it, or reference
it.
```

A constant with the same name as a matrix axis gets its own message (`shadowed
by the [matrix] axis of the same name`), because `expand` inserts axis values
over the constants and the constant can never be read — a different mistake with
a different fix.

Both 2 and 3 live in `check_bindings_are_referenced`, `model.rs:744`, and both
count references the same narrow way: only `[source]` keys and bodies and
substituted step fields. A `${NAME}` in a `rationale`, inside another constant's
*value*, or in a matrix axis *value* is **not** a reference — `expand.rs` never
substitutes those, so counting them would reopen the channel one sentence of
prose away.

**4. A file named exactly `.toml`.** `Path::extension()` reports `None` for a
leading-dot name with no other dot, so such a file would be discovered by
nothing and reported by nothing:

```
cases/x/.toml: a file named exactly `.toml` has no stem to name its trials with
-- `Path::extension()` treats it as a dotfile, so it would be skipped in
silence. Give it a name.
```

`collect`, `discover.rs:28`. Ordinary hidden files (`.gitkeep`, editor
swapfiles) stay quietly ignored; only the exact name `.toml` is refused.

Alongside these, a count claim that spells both or neither of
`at_least`/`exact`, one that spells `at_least = 0`, an empty `needle`, an axis
with an empty value list, and an unresolved `${...}` surviving substitution are
all parse errors too, each for the same reason: a claim nothing can fail.

---

## Two rules worth stating up front

- **Put the reason in `rationale`, not a `#` comment.** The runner prints
  `rationale` (indented under `  | `) when the case fails, alongside the step
  index. For a `cli` or `browser_bundle_harness` step that also means the argv,
  the env, and the full captured stdout/stderr; a `file_json` step runs no
  process, so its failure detail is `check_json`'s own message instead — there
  is no captured output to print. A comment explaining why a test exists is
  invisible exactly when someone needs it.
- **Pin diagnostic text through `[constants]`.** A hand-copied message prefix
  goes insensitive the moment the diagnostic widens. That has happened here
  before. A `[constants]` entry no `${NAME}` reaches is now a **parse error** —
  the file does not load at all, so a hoisted constant cannot rot into free text
  silently. That is where you will meet it; see *Four ways a case file is
  refused* above. `scripts/audit-case-migration.py` refuses the same shape by
  name, and that second refusal is not redundant: it is the arm that explains
  *why* a dead constant matters — its value still lands in the audit's
  substring haystack, so a dead constant can satisfy a claim the runner never
  checks. Two independent refusals of one shape is the point; a gate whose
  correctness rests on another gate running first is not a gate.

A case the format cannot express belongs in its own hand-written target, not in
a weakened `.toml`. See spec §5.11 for what stays Rust and why; today that is
`inprocess`, `runtime_smoke`, `package_corpus`, `schema_docs`,
`node_api_surface`, `browser_cdp_smoke`,
`browser_harness_failing_test_propagates_failure`, and the browser targets batch
8C retained.

---

## The `#` header: vocabulary and provenance

### The rule numbers headers cite

Case-file headers argue from a numbered vocabulary — `rule 6`, `ruling 12`,
`U5`, and sometimes `batch N`. Those numbers are defined in
[`docs/superpowers/2026-07-29-test-binary-consolidation-migration-rules.md`](../../../../docs/superpowers/2026-07-29-test-binary-consolidation-migration-rules.md),
and nowhere else. They are cited widely enough that reading a header without it
is guesswork:

```bash
$ cd "$(git rev-parse --show-toplevel)"
$ grep -rlEi '\brule [0-9]+'   --include='*.toml' crates/kali_cli/tests/cases/ | wc -l   # 221
$ grep -rlEi '\bruling [0-9]+' --include='*.toml' crates/kali_cli/tests/cases/ | wc -l   # 167
$ grep -rlE  '\bU[0-9]+\b'     --include='*.toml' crates/kali_cli/tests/cases/ | wc -l   # 179
```

`rule N` is one of the thirteen migration rules, `ruling N` one of the nineteen
controller rulings that amend them, `U<N>` one of the unnumbered governing
rules. `batch N` is the one term that is *not* a single namespace — it counts
whichever task's batches the surrounding sentence is about — so read it against
the family the file sits in; the glossary explains how.

### `Migrated from` and `SOURCE REF:`

Two header lines are machine-read, by
`tools/task-18-browser-pilot/citation_sweep.sh`, which works out which Rust
source each case file's `:N` citations should resolve against and hands the pair
to `batch5_crosscheck.py`. They are conventions with a gate behind them, not
decoration — though note how far that gate reaches: the sweep covers one family
per invocation, and the bare call `--gates-only` makes sweeps `browser/`. The
other families need `--family <name>`, which nothing in `test-gate.sh` or CI
passes today.

```toml
# Migrated from tests/browser_bundle_template_literal_interpolation.rs.
#   SOURCE REF: 3e083edc5d8ba24dd69a79cc75c60889d8258cb5
```

- **`Migrated from tests/<X>.rs`** names the source. It is **mandatory whenever
  no same-named `.rs` exists** beside the case file — after a U2 split, whose
  case-file stem differs from its source's, and after the source is deleted. The
  sweep reads the name out of the file rather than deriving it from the stem, so
  nothing is hardcoded; with no `.rs` and no `Migrated from` line it fails,
  because the source the citations resolve against cannot even be named. All 287
  case files carry the line today.
- **`SOURCE REF: <sha>`** names the commit whose copy of that source the
  citations were written against. Rules the sweep enforces:
  - **At most one per file.** Two make it ambiguous which blob the citations
    were written against, so the sweep fails rather than taking the first.
  - **A full 40-character lowercase sha.** A branch name or an abbreviation
    names a different commit as the branch moves or the repository grows.
  - **It must name a commit where the source still EXISTS** — the deletion
    commit's *parent*, not the deletion commit. Derive it with
    `git log --diff-filter=D -1 --format=%H -- crates/kali_cli/tests/<X>.rs`
    and then `git rev-parse <that>^`.
  - **While the source is still in the tree, the declared blob is compared byte
    for byte against it.** Existence alone would let a ref naming an older
    revision of the same file pass every check and then shift every `:N` on the
    day the source went away. The workflow is declare-first-delete-later
    precisely so this window exists; once the source is gone, only existence can
    be checked.

195 case files carry a `SOURCE REF:` line — 149 in `browser/`, 33 in `misc/`,
11 in `runtime/`, 2 in `nullish/`:

```bash
$ grep -rl 'SOURCE REF:' --include='*.toml' crates/kali_cli/tests/cases/ | wc -l   # 195
```

The sweep needs **full git history**. It materialises
`crates/kali_cli/tests` at each declared ref, which a shallow clone cannot do —
so a shallow clone needs `git fetch --unshallow` first, and any CI job that ran
the sweep would need `fetch-depth: 0` on its checkout step. No CI job runs it
today; see *The migration gates are a developer command, not a CI gate* below.
The script says so in its own preconditions.

### Those 195 refs name branch-only commits

**A squash merge or a rebase merge destroys this provenance mechanism.** Both
replace the branch's commits with new objects carrying new shas, so a clone of
the merged history does not contain the commits those 195 refs name.
`citation_sweep.sh` fails by name on exactly that — "`SOURCE REF: <sha>` is not
reachable in this repository" — for the 149 `browser/` files it gates today, and
for the other 46 the moment it is pointed at their families. The declared blobs
are then unrecoverable by any route, so the citations in those files can never be
verified again; the header lines survive as prose naming shas that resolve to
nothing. That is the mechanical consequence. Which merge strategy to use is a
decision for a human, and this README does not make it.

### Some case files are generated and byte-pinned

Four generators under `tools/migration/` own **40 case files** between them —
`gen_task19_batch2.py` (17), `gen_task19_batch3.py` (7), `gen_task19_batch4.py`
(9), `gen_task19_batch5.py` (7). Each generated file carries a
`# GENERATED by tools/migration/gen_task19_batch<N>.py` banner on its first
line.

Their default mode is the *check* direction: re-render every file they own and
fail on any byte difference. All four are wired into
`scripts/test-gate.sh --gates-only`, so **hand-editing a generated case file
fails that command** — the file would silently diverge from the spec a reviewer
actually reads. It does **not** fail CI: nothing in `.github/` runs
`--gates-only` (see *Where the gates live* below). To change one, edit the
generator and re-run it with `--write`.

Two of them re-run the compiled `kali` binary — `gen_task19_batch4.py` for its
cross-stream resolutions and the U2 policy control, `gen_task19_batch5.py` for
its eighteen rule-11 disjunctions — and **fail rather than skip** when it is
absent. They locate it through `tools/migration/kali_bin.py`:
`$CARGO_BIN_EXE_kali`, then `$KALI_BIN`, then `$CARGO_TARGET_DIR/debug/kali`,
then `cargo metadata --format-version 1 --no-deps`'s own `.target_directory`,
then `<repo>/target/debug/kali`; the failure names every candidate it tried.
Build with `cargo build -p kali_cli --bin kali` or set `KALI_BIN`.

---

## Where the gates live

- `bash scripts/test-gate.sh` — the full-workspace test gate; it enumerates every
  failing test rather than stopping at the first red binary. The runner's failure
  text is indented with **two** spaces and `  | ` on purpose: this script parses
  `^    [A-Za-z_]` as a failed-test name, so a four-space-indented detail line
  would be reported as a test that does not exist.
- `bash scripts/test-gate.sh --gates-only` — the 14 migration gates (case-corpus
  generators, fixture and citation sweeps, the audit's own regression suite). A
  bare invocation does not run them.

### The migration gates are a developer command, not a CI gate

**Nothing in `.github/` runs `--gates-only`.** A `migration-gates` job was added
to `.github/workflows/ci.yml` during this branch and removed again before merge,
because it could not pass on a runner: it installed no Rust toolchain and ran no
`cargo build`, while two of the 14 gates run the compiled `kali` binary and fail
rather than skip when it is absent — and the target directory they searched,
`.cache/cargo-target`, comes from a machine-local `~/.cargo/config.toml` in one
dev container and is in no checkout of this repository, so a runner's cargo would
have built to `./target` and the path would have been wrong even with a build
step. (That second defect is fixed: the two generators now derive the target
directory, see *Some case files are generated and byte-pinned* above.)

It was removed rather than repaired because **the migration is finished and
frozen at merge**: the corpus these gates guard — the case files, their
`SOURCE REF:` lines, the generators' fixed points — does not change again, so
there is no drift for a per-PR run to catch. `scripts/test-gate.sh`'s own header
argues that "a check nobody re-runs is indistinguishable from a check that was
deleted", and that argument is now load-bearing *against* these gates rather than
for them: this section exists so nobody reads that header and concludes CI is
re-running anything. Anyone who does reopen the corpus — a new batch, an edit to
a generated case file, a re-derived citation — must run `--gates-only` by hand,
with a built `kali` binary, and is on the hook for its result. The
`fetch-depth: 0` requirement above applies to whoever does: full history, or a
`git fetch --unshallow` first.
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
