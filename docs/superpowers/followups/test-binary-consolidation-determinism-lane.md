# FOLLOW-UP: `scripts/check-determinism.sh` runs zero tests — needs a human decision

**Status:** open, discovered during Task 20 of `2026-07-29-test-binary-consolidation`
(fix round 1, finding M5, 2026-08-14). Not fixed here: the fix touches a file
under this project's do-not-modify constraint, so it needs a human decision
rather than an agent edit.

## What is broken

`bash scripts/check-determinism.sh` exits **0** while running **zero** tests.
Each of its 20 invocations is
`cargo test -p kali_cli --test runtime_smoke <fn-name> -- --exact`, where
`<fn-name>` is an *unqualified* function name such as
`build_artifacts_are_deterministic_across_repeated_invocations`. But
`runtime_smoke.rs` is split into `#[path]` submodules (`build`, `effects`,
`package`, `install`, `test`, …), so libtest resolves that same function as
`build::build_artifacts_are_deterministic_across_repeated_invocations`. The
unqualified `--exact` filter matches nothing, `cargo test` reports
`0 passed; 0 failed; 1829 filtered out`, and `set -euo pipefail` never sees a
failure because "ran nothing" is not an error to Cargo.

Reproduced (Task 20 fix round 1):

```
$ bash scripts/check-determinism.sh
     Running tests/runtime_smoke.rs (.../runtime_smoke-...)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1829 filtered out
EXIT=0
```

## Evidence this is pre-existing, not caused by the test-binary-consolidation branch

```
$ git diff --stat main -- scripts/check-determinism.sh        (empty — byte-identical to main)
$ git show main:crates/kali_cli/tests/runtime_smoke.rs | grep -c '#[path'
8
```

The `#[path]` submodule split in `runtime_smoke.rs` predates this branch
(commit `2448dd8839`, 2026-07-23), and `scripts/check-determinism.sh` is
byte-identical between `main` and this branch. The determinism lane has
therefore been green while testing nothing since at least that commit —
including in CI, on `main`, for weeks before this branch existed.

## Why it is not fixed here

`scripts/check-determinism.sh` is one of this project's do-not-modify files
(alongside `scripts/test-gate.sh`, `mise.toml`, `.github/workflows/ci.yml`).
Whoever owns that constraint needs to decide the fix — most directly,
qualifying each of the 20 filters with its submodule prefix
(`build::build_artifacts_are_deterministic_across_repeated_invocations`, etc.),
but that decision belongs to a human, not to an agent working around the
constraint.

## Where else this is recorded

- `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md` §9,
  "Verification at the end of Task 20" — the design spec's outcome section.
- `.superpowers/sdd/2026-07-29-test-binary-consolidation/task-20-report.md` §6
  and the FIX ROUND 1 addendum.
