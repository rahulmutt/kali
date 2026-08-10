# Task 18 browser/ pilot — verification tooling

Scratch tooling used to migrate the six-file `browser/` pilot batch
(`crates/kali_cli/tests/cases/browser/*.toml`). Committed so its
independence from the generated `.toml` files can be judged directly,
per the task brief's verification section — earlier tasks on this branch
kept equivalent tooling scratchpad-only; this batch commits it instead.

Not wired into `mise.toml`/CI and not imported by anything under
`crates/`. Safe to delete once the pilot is reviewed, or to keep as a
starting point for batches 2-8 of Task 18.

**Fixed after pilot round-1 review (minor):** the six per-file `gen_*.py`/
`emit_*.py` generator scripts used during migration hardcoded an
agent-session scratchpad path (`/tmp/claude-.../scratchpad/t18/...`, 17
occurrences across 8 files) and, for two files, loaded an uncommitted
intermediate (`.pkl`/`.json` dumps from a temporary `#[test] fn dump()`
run) — neither runnable from a clean checkout. Rather than parameterize
eight scripts whose only remaining value is as a historical record (the
`.toml` files they produced are already committed and already re-verified
independently after every review-round fix), they were removed from this
directory. **What's left below is the reusable, path-clean, immediately
runnable core** — no hardcoded paths, no uncommitted inputs:

- `lexer.py` — character-cursor Rust string-literal scanner. The
  generator's fixture-copy mechanism: every JS fixture body embedded in a
  `.toml` was pulled through this (or, for the `format!`/library-call sites
  in files 4 and 6, through a temporary `#[test] fn dump()` that actually
  executed the real Rust code — see the task report).
- `kali_run.py` — spawns the real built `kali` binary (and, transitively,
  `node` for browser-harness steps) against literal in-memory fixtures, so
  every generated case's expected `stdout`/`json` fields were captured live
  rather than hand-computed. Usable standalone: `from kali_run import
  run_kali; run_kali({"a.js": "console.log(1)"}, ["run", "a.js"])`.
- `fidelity.py` — an independent (regex-per-position, not character-cursor)
  bidirectional source-vs-TOML string-literal diff. Prints both `missing`
  and `extra`, per the task brief's explicit requirement. Usage:
  `python3 fidelity.py SOURCE.rs [SOURCE2.rs ...] -- TARGET.toml`.
- `comment_coverage.py` — mechanical, independent: groups `//`/`///`/`//!`
  lines into paragraphs and requires every non-divider paragraph line's
  text to appear verbatim (whitespace-collapsed) in **every individual
  case's own `rationale`** — not just somewhere in the file header or in
  the pooled union of all rationales (that was the round-1 bug: a
  header-only mention read as "covered" for every case). Usage:
  `python3 comment_coverage.py [--allow-empty] SOURCE.rs TARGET.toml`.
  **Exits 1 if any line is missing from any case, 2 if it checked zero
  non-divider comment lines, 0 otherwise** (round-2 review fix: the script
  used to only print and never `sys.exit`, so it read as a pass in any loop
  that checked its exit code even when it printed missing lines; Task 18
  controller ruling 5, added by batch 3: exit 0 on "0 lines checked" was a
  vacuous green, indistinguishable from real coverage, and most of the
  remaining ~133 files carry no Rust comments at all, so that would have
  become the normal unexamined result. `--allow-empty` is the explicit
  acknowledgement, to be passed only after reading the source and confirming
  it genuinely has no Rust comments).
  Deliberate scope limit, stated in the module docstring: it does not
  attempt per-helper attribution for a file with two distinct
  helper-produced comment blocks each covering a disjoint subset of cases
  — none of this pilot's six files has that shape, but batches 2-8 might;
  and it cannot yet gate a *retained* (not fully migrated) `.rs`/`.toml`
  pair like `browser_math_pow_exponent_one.rs` — run against that pair it
  currently reports 57 false-positive missing lines (measured directly:
  the file's own `//!` escalation header prose, which the 4 migrated
  bundle-build cases have no reason to carry into their `rationale`, since
  none of that prose describes what THEY test), so it was not wired into
  this task's verification for that one pair. See the task report's
  pattern notes.
- `toml_emit.py` — single-line vs. triple-quoted TOML string emission.
  `toml_string(value, multiline=None)`, `toml_str_array(values)`.
- `classify_drift.py` — **regenerate-and-diff, as a gate.** Runs every
  `gen_batch*.py` in this directory (the population is the directory, not a
  list), compares each case file it writes against `HEAD`, classifies any
  difference as citation-form-only or content drift by two independent methods
  that must agree, and requires both enumerated sets to equal their
  declarations. `--selftest` runs 13 poisoned probes, including the reflow
  control whose absence made an earlier instrument report 6/20 instead of 25/1.
  Its controls run first: a comparator that has not been shown to fire is not
  evidence. Added in batch 8-inst-1, which is also when regenerating stopped
  being a gate regression — `case_emit.write` now folds the citation reword in,
  so a generator emits the gated form rather than needing a post-pass.

Full narrative (what each file's shape was, why matrix was or wasn't used,
the audit findings, the scaling measurement, and the five review-round-1
fixes) is in the task report, `task-18-pilot-report.md`, alongside the plan
documents for this branch.
