# Task 18 browser/ pilot — verification tooling

Scratch tooling used to migrate the six-file `browser/` pilot batch
(`crates/kali_cli/tests/cases/browser/*.toml`). Committed so its
independence from the generated `.toml` files can be judged directly,
per the task brief's verification section — earlier tasks on this branch
kept equivalent tooling scratchpad-only; this batch commits it instead.

Not wired into `mise.toml`/CI and not imported by anything under
`crates/`. Safe to delete once the pilot is reviewed, or to keep as a
starting point for batches 2-8 of Task 18.

- `lexer.py` — character-cursor Rust string-literal scanner. The
  generator's fixture-copy mechanism: every JS fixture body embedded in a
  `.toml` was pulled through this (or, for the two `format!`/library-call
  sites in files 4 and 6, through a temporary `#[test] fn dump()` that
  actually executed the real Rust code — see the task report).
- `kali_run.py` — spawns the real built `kali` binary (and, transitively,
  `node` for browser-harness steps) against literal in-memory fixtures, so
  every generated case's expected `stdout`/`json` fields were captured live
  rather than hand-computed.
- `fidelity.py` — an independent (regex-per-position, not character-cursor)
  bidirectional source-vs-TOML string-literal diff. Prints both `missing`
  and `extra`, per the task brief's explicit requirement.
- `comment_coverage.py` — mechanical, independent: groups `//`/`///`/`//!`
  lines into paragraphs and requires every non-divider line's text to
  appear verbatim (whitespace-collapsed) in the target `.toml`'s `#` header
  plus every case's `rationale`.
- `toml_emit.py` — single-line vs. triple-quoted TOML string emission.
- `gen_*.py` / `emit_*.py` — one generator+emitter pair per source file,
  each: builds the case list from data read directly out of the `.rs`
  source (by lexer index or by executing the real `format!`/library-helper
  calls), live-verifies every case against the real binary, then writes the
  final `.toml`.

Full narrative (what each file's shape was, why matrix was or wasn't used,
the two audit findings, the scaling measurement) is in the task report,
`task-18-pilot-report.md`, alongside the plan documents for this branch.
