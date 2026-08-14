#!/usr/bin/env python3
"""Generate cases/browser/math_log2_log10.toml from browser_math_log2_log10.rs.

Batch 4, file 1 -- the first real adopter of the `stdout_count`/`json_count`
keys, kept in its own commit so the keys' first use is isolated in history.

This source is the reason the two keys are separate: ONE helper
(`assert_browser_harness_math_log2_log10`) asserts the SAME count claim on both
surfaces, `json["stdout"].as_str()` in its `--output json` branch (:173-181)
and raw stdout in its else branch (:186). A key covering only raw stdout would
leave half of this helper hand-written.

Run from anywhere: `python3 gen_batch4_log2_log10.py`.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))

from case_emit import fixture, emit, write, source_text_at  # noqa: E402

RS = os.path.join(REPO, "crates/kali_cli/tests/browser_math_log2_log10.rs")
OUT = os.path.join(REPO, "crates/kali_cli/tests/cases/browser/math_log2_log10.toml")

text = source_text_at(RS, quiet=True)   # 8C: RS may be a deleted source

# Every fixture below is pulled from the .rs by line range through lexer.py
# (rule 9 -- never retype the program under test).
BUNDLE_SRC = fixture(text, 11, 23)      # browser_bundle_math_log2_log10_source
HARNESS_BODY = fixture(text, 80, 82)    # the import/call body passed to the harness
RUN_SRC = fixture(text, 111, 111)       # browser_harness_math_log2_log10_run_source
TEST_SRC = fixture(text, 115, 123)      # browser_harness_math_log2_log10_test_source

HEADER = """\
Migrated from tests/browser_math_log2_log10.rs.

RULE 12 (carry every source comment): the source has NO Rust comments. Checked
with `grep -n '//'`, whose single hit is line 11's `// kali-tree-shake:
mathLog2Log10Identities` -- that is inside the `r##"..."##` bundle fixture, so
it is program text carried verbatim into [source] below, not Rust prose. There
is therefore nothing to move into `rationale` from comments, and
comment_coverage.py is run with --allow-empty for this pair.

RULE 7 / U1 -- MATRIX ARITHMETIC, and it closes exactly.
Two helpers, 24 #[test] fns, 24 real invocations, no loops anywhere in the
file (every #[test] fn is a single unlooped helper call):
  * `assert_browser_bundle_math_log2_log10(filename, json_output)` -- 8 fns
    = ext(js/ts/jsx/tsx) x json_output(false/true), a full cross product.
  * `assert_browser_harness_math_log2_log10(command, filename, source,
    json_output)` -- 16 fns = command(run/test) x ext(js/ts/jsx/tsx) x
    json_output(false/true), also a full cross product.
`ext` is the ONE axis every case varies over uniformly, and both helpers cover
all four extensions, so a file-wide axis fans nothing the source never ran:
6 [[case]] entries x ext(4) = 24 trials = 24 #[test] fns. Exact.

`command` and `json_output` are NOT matrix axes, per rule 7 and design spec
5.6's own note: both change the ASSERTION SHAPE, not just a substituted
string. `json_output` switches between a text-stdout claim and a JSON-envelope
claim; `command` switches the envelope's payload between `exitCode` (run) and
`total`/`passed`/`failed` (test). Each is expressed as sibling [[case]]
entries instead. Per rule 6 the matrix fold is stated here: each [[case]]
below corresponds to 4 source #[test] fns, one per `ext` cell, and the
assertion mapping stays 1:1 per trial.

U2 -- [source] is file-wide, and that is safe here. All three fixtures
(`app.${ext}`, `main.${ext}`, `smoke.test.${ext}`) are written unconditionally
by the source in a fresh temp dir, no fixture is written behind an `if`, and
no case's point is the presence or absence of a file. Every command below
names its entry explicitly on argv, so the two unused siblings in a trial dir
are inert. Same shape as the already-shipped
`math_exp_log_bracketed_root.toml`.

THE COUNT KEYS (first use in the tree). The source makes the same
`.matches("3\\n").count() >= 2` claim at four sites: :107 (bundle harness
stdout), :177-179 (`json["stdout"].as_str()` in the harness helper's json
branch), :186 (raw stdout in that helper's else branch), and via :106's
separate `.contains("3\\n")`. Per ruling 3 (mirror the source) every one is
transcribed directly -- `count() >= 2` becomes `at_least = 2`, on the raw
surface as `stdout_count` and on the JSON leaf as `json_count`. The bound is
NOT strengthened to `exact`: the live output is `3\\n3\\n3\\n3\\n` (four
occurrences, since Math.log2(8), frozenLog2(8), Math.log10(1000) and
frozenLog10(1000) all print 3), but the source claims only `>= 2` and pinning
4 would assert something it never did (rule 2). Line 106's `.contains("3\\n")`
is a SEPARATE source claim from line 107's count, so both are carried on the
bundle-harness step; collapsing them into one would drop a claim.

RULE 13 -- transitive helper docs. Checked every fn in each call chain:
`kali_bin`, `browser_bundle_math_log2_log10_source`,
`assert_browser_bundle_math_log2_log10`,
`browser_harness_math_log2_log10_run_source`,
`browser_harness_math_log2_log10_test_source` and
`assert_browser_harness_math_log2_log10` -- none carries a `///` doc comment.
The chain also reaches `kali_runtime_contract::browser_bundle_harness_script`
and `::browser_harness_command_parts_for`, which DO carry one-line `///` docs,
but those are not carried here: in the migrated form this case file does not
call them at all -- the `browser_bundle_harness` step kind means the case
RUNNER calls them (crates/kali_case_runner/src/steps.rs:70), so their docs
describe shared runner infrastructure documented in design spec 5.3, not what
this case claims. All 45 previously-shipped browser/ case files do the same
(measured: 0 of 45 carry either doc string). Flagged in the batch 4 report as
a standing question rather than silently decided.

ASSERTION SHAPE, mirrored from the source and nothing more.
Bundle helper: `exit = "success"` on the build (:43) and on the harness
process (:99); in json mode the envelope's schemaVersion/command/success/
exitCode/payload(artifactKind, bundleFormat) and `errors = []` (:52-62); the
emitted `app/app.meta.json` metadata (:70-71), asserted in BOTH modes because
the source reads it outside the `if json_output`; then the harness step's
`stdout_contains` + `stdout_count`.
Harness helper: `exit = "success"` (:151); json mode carries schemaVersion/
command/success/payload(hostContract, runtimeBackend) (:160-164), plus
`exitCode`/`payload.exitCode` for `run` (:166-167) or payload total/passed/
failed for `test` (:169-171), then `json_count` for :173-181, `stderr = ""`
(:182) and `errors = []` (:183). NO `json.stdout` pin is written: the source
makes only a count claim about that leaf, never an equality one.
The source adds no `--max-threads`/`--max-spawned-processes` arguments (unlike
several siblings in this batch), so neither is added here."""

MATRIX = {"ext": ["js", "ts", "jsx", "tsx"]}

SOURCE = {
    "app.${ext}": BUNDLE_SRC,
    "main.${ext}": RUN_SRC,
    "smoke.test.${ext}": TEST_SRC,
}

META_STEP = {
    "kind": "file_json",
    "path": "app/app.meta.json",
    "fields": {"apiSurface": "browser", "artifactKind": "bundle"},
}

HARNESS_STEP = {
    "kind": "browser_bundle_harness",
    "entry": "app",
    "body": HARNESS_BODY,
    "exit": "success",
    "stdout_contains": ["3\n"],
    "stdout_count": [{"needle": "3\n", "at_least": 2}],
}

BUNDLE_PROSE = (
    "Migrated from browser_math_log2_log10.rs. `assert_browser_bundle_math_log2_log10` "
    "builds a browser bundle (`kali build --bundle --api browser`), asserts the emitted "
    "`app/app.meta.json` metadata, then runs the bundle glue under the browser-bundle-harness "
    "contract and checks that the log2/log10 identity output appears: Math.log2(8), "
    "frozenLog2(8), Math.log10(1000) and frozenLog10(1000) each print `3`. The source makes "
    "TWO separate claims about that output -- `stdout.contains(\"3\\n\")` (:106) and "
    "`stdout.matches(\"3\\n\").count() >= 2` (:107) -- so both are carried, as `stdout_contains` "
    "and `stdout_count` respectively. The count is mirrored at `at_least = 2` exactly as the "
    "source spells it; the real output contains four occurrences, but pinning `exact = 4` "
    "would assert something the source never claimed."
)

HARNESS_PROSE_COMMON = (
    "Migrated from browser_math_log2_log10.rs. `assert_browser_harness_math_log2_log10` runs "
    "`kali {cmd} --api browser` with the browser harness backed by `node` "
    "(KALI_BROWSER_BUNDLE_HARNESS_COMMAND), against a program whose four calls -- "
    "Math.log2(8), frozenLog2(8), Math.log10(1000), frozenLog10(1000) -- each print `3`. "
)

COUNT_NOTE_STDOUT = (
    "The source's only stdout claim on this branch is "
    "`stdout.matches(\"3\\n\").count() >= 2` (:186), carried as `stdout_count` with "
    "`at_least = 2`. It is deliberately NOT weakened to `stdout_contains`, which a single "
    "occurrence would satisfy, and not strengthened to `exact`, which the source never says."
)

COUNT_NOTE_JSON = (
    "The source's stdout claim on this branch is "
    "`json[\"stdout\"].as_str().matches(\"3\\n\").count() >= 2` (:173-181) -- the same count "
    "taken against the JSON string leaf rather than raw stdout, which is why `json_count` "
    "exists alongside `stdout_count`. Carried with `at_least = 2`, mirroring the source. No "
    "equality pin is written for `json.stdout`: the source asserts only how many times `3\\n` "
    "occurs in it, never what it equals."
)

CASES = [
    {
        "name": "build_emits_math_log2_and_log10_identity_literals",
        "rationale": BUNDLE_PROSE,
        "steps": [
            {"args": ["build", "--bundle", "--api", "browser", "app.${ext}"],
             "exit": "success"},
            META_STEP,
            HARNESS_STEP,
        ],
    },
    {
        "name": "json_build_emits_math_log2_and_log10_identity_literals",
        "rationale": BUNDLE_PROSE + " This sibling asserts the JSON output envelope "
                     "(schemaVersion/command/success/exitCode, payload artifactKind and "
                     "bundleFormat, and an empty `errors` array) rather than plain text; "
                     "output shape is not a matrix axis because it changes the assertion "
                     "shape, so it is a separate case.",
        "steps": [
            {"args": ["build", "--bundle", "--api", "browser", "--output", "json", "app.${ext}"],
             "exit": "success",
             "json": {"schemaVersion": 1, "command": "build", "success": True,
                      "exitCode": 0,
                      "payload": {"artifactKind": "bundle", "bundleFormat": "esm"},
                      "errors": []}},
            META_STEP,
            HARNESS_STEP,
        ],
    },
    {
        "name": "run_supports_math_log2_and_log10_when_browser_harness_is_configured",
        "rationale": HARNESS_PROSE_COMMON.format(cmd="run") + COUNT_NOTE_STDOUT,
        "steps": [
            {"args": ["run", "--api", "browser", "main.${ext}"],
             "env": {"KALI_BROWSER_BUNDLE_HARNESS_COMMAND": "node"},
             "exit": "success",
             "stdout_count": [{"needle": "3\n", "at_least": 2}]},
        ],
    },
    {
        "name": "test_supports_math_log2_and_log10_when_browser_harness_is_configured",
        "rationale": HARNESS_PROSE_COMMON.format(cmd="test") + COUNT_NOTE_STDOUT,
        "steps": [
            {"args": ["test", "--api", "browser", "smoke.test.${ext}"],
             "env": {"KALI_BROWSER_BUNDLE_HARNESS_COMMAND": "node"},
             "exit": "success",
             "stdout_count": [{"needle": "3\n", "at_least": 2}]},
        ],
    },
    {
        "name": "json_run_supports_math_log2_and_log10_when_browser_harness_is_configured",
        "rationale": HARNESS_PROSE_COMMON.format(cmd="run --output json") + COUNT_NOTE_JSON,
        "steps": [
            {"args": ["--output", "json", "run", "--api", "browser", "main.${ext}"],
             "env": {"KALI_BROWSER_BUNDLE_HARNESS_COMMAND": "node"},
             "exit": "success",
             "json": {"schemaVersion": 1, "command": "run", "success": True,
                      "payload": {"hostContract": "browser-requested",
                                  "runtimeBackend": "browser-harness",
                                  "exitCode": 0},
                      "exitCode": 0, "stderr": "", "errors": []},
             "json_count": [{"path": "stdout", "needle": "3\n", "at_least": 2}]},
        ],
    },
    {
        "name": "json_test_supports_math_log2_and_log10_when_browser_harness_is_configured",
        "rationale": HARNESS_PROSE_COMMON.format(cmd="test --output json") + COUNT_NOTE_JSON,
        "steps": [
            {"args": ["--output", "json", "test", "--api", "browser", "smoke.test.${ext}"],
             "env": {"KALI_BROWSER_BUNDLE_HARNESS_COMMAND": "node"},
             "exit": "success",
             "json": {"schemaVersion": 1, "command": "test", "success": True,
                      "payload": {"hostContract": "browser-requested",
                                  "runtimeBackend": "browser-harness",
                                  "total": 1, "passed": 1, "failed": 0},
                      "stderr": "", "errors": []},
             "json_count": [{"path": "stdout", "needle": "3\n", "at_least": 2}]},
        ],
    },
]

if __name__ == "__main__":
    write(OUT, emit(HEADER.split("\n"), MATRIX, SOURCE, CASES))
