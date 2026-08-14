# Task 18 browser/ batch 3 — verification tooling

Committed per U12 ("verification tooling is committed, not scratchpad-only"), so
the independence of this batch's checks from the `.toml` files they produced can
be judged directly. Runnable from a clean checkout: no hardcoded scratchpad
paths, no uncommitted inputs. Not wired into `mise.toml`/CI and not imported by
anything under `crates/`.

Reuses `tools/task-18-browser-pilot/`'s `lexer.py`, `toml_emit.py`,
`fidelity.py` and `comment_coverage.py` rather than rebuilding them.

## Generation

- `inventory.py FILE.rs [...]` — prints every `#[test]` fn's body verbatim, so
  the per-file invocation arithmetic (rule 7) is re-derived from the real call
  sites and expanded loops rather than from fn-name patterns.
- `extract.py` — `fixture(rs, fn)` / `fixtures(rs, fn)` return fixture bodies
  pulled through the pilot's character-cursor Rust literal scanner, never
  retyped. `comment_block(rs, first, last)` copies a `//` block verbatim, so an
  em-dash or an ellipsis survives byte-identically into a `rationale` (rule 12).
- `capture.py` — spawns the real built `kali` (with `node` as the browser-harness
  backend) against literal in-memory fixtures. Every exact `stdout`/`json.*`
  value this batch pins was read back from here, never hand-computed (U9).
  Override the binary with `KALI_BIN=...`.
- `emit.py` — TOML emitter. `emit_case_file()` re-parses everything it renders
  with `tomllib` and compares field-by-field against the Python values it was
  built from *before* the file reaches disk, so a quoting bug cannot ship.
- `gen_iteration.py`, `gen_math.py`, `gen_map.py` — the per-file generators.
  Each is runnable standalone and rewrites its own case files in place.

## Verification

- `verify_batch3.py` — the independent re-derivation required by U11. Does NOT
  reuse the generators' machinery: fixture fidelity is re-checked by ENCODING
  each shipped TOML value back into the two spellings a Rust literal can legally
  have and searching the raw `.rs` (the inverse of the generators' decode), and
  trial arithmetic is re-derived from `cargo test --list` against a hand-written
  table of (`#[test]` fns, real invocations, expected trials).
- `fidelity_sweep.py PAIRS_FILE` — runs the pilot's bidirectional source-vs-TOML
  diff for every pair and classifies the raw output so the `extra` side (the
  checkable invariant behind "never invent an assertion") is reviewable entry by
  entry instead of drowning in TOML keywords. `PAIRS_FILE` lines are
  `<rs stem> <toml stem>`.
- `rationale_fn_check.py SOURCE.rs TARGET.toml` — U8's gate: every backticked,
  fn-shaped identifier in a case file's header and rationales must name a real
  `fn` in the source it was migrated from. Non-fn vocabulary is an explicit,
  commented allowlist, not a heuristic.

Full narrative in `task-18-batch3-report.md` alongside this branch's plan
documents.
