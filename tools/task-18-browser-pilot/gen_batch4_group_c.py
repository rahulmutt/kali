#!/usr/bin/env python3
"""Generate the batch 4 "group C" case files (5 browser math root-shape targets).

A separate module from gen_batch4.py on purpose: four implementers ran
concurrently in this batch and a shared generator file is a write race, in
which one whole-file write silently drops another agent's function. This
module only READS the shared helpers (case_emit.py, math_shapes.py); it edits
nothing outside itself.

Targets:
  browser_math_exp_log_mixed_root.rs            -> math_exp_log_mixed_root.toml
  browser_math_expm1_log1p_bracketed_root.rs    -> math_expm1_log1p_bracketed_root.toml
  browser_math_expm1_log1p_mixed_root.rs        -> math_expm1_log1p_mixed_root.toml
  browser_math_expm1_log1p_fully_bracketed_root.rs
                                     -> math_expm1_log1p_fully_bracketed_root.toml
  browser_math_log2_log10_fully_bracketed_root.rs
                                     -> math_log2_log10_fully_bracketed_root.toml

All five share ONE shape: a `build --bundle --api browser` group (cli ->
file_json meta -> browser_bundle_harness) and a `run`/`test --api browser`
harness group, each crossed with `--output json`. `_six_cases` renders that
shape, but it defaults NOTHING: every assertion set, every argv flag and every
axis value is passed explicitly by the per-target function below, because these
five files differ from each other in exactly the places a default would paper
over (rule 2). One file asserts `errors = []` on the build envelope and four do
not; one carries a `.matches().count()` claim and four do not; two cover four
extensions and two cover only js/ts.

One function per source file. Each returns the full spec -- the mapping
decision, the matrix arithmetic and the assertion set -- so a reviewer reads it
in one place. Every fixture is pulled from the .rs by CONTENT anchor (fn name /
literal prefix), never by line range and never retyped (rule 9).

Run: python3 gen_batch4_group_c.py [name ...]   (no args = all)
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

from case_emit import emit, write, fixture_in_fn, fixture_starting  # noqa: E402
from lexer import find_string_literals  # noqa: E402
from math_shapes import (  # noqa: E402
    bundle_steps, harness_step, envelope_build, envelope_harness, META,
)

REGISTRY = {}


def target(name):
    def deco(fn):
        REGISTRY[name] = fn
        return fn
    return deco


def rs(name):
    return open(os.path.join(TESTS, f"browser_{name}.rs")).read()


def repeated_fixture_in_fn(rs_text, fn_name, prefix):
    """The one DISTINCT string literal starting with `prefix` inside `fn_name`.

    `case_emit.fixture_starting` requires exactly one matching literal and is
    the right tool when a fixture is declared once. Two files in this group
    (browser_math_expm1_log1p_bracketed_root.rs, and nothing else here) inline
    their harness fixtures directly into the `#[test]` fn's tuple list, once
    per extension -- four byte-identical copies of the run source and four of
    the test source. That is not ambiguity, it is repetition, so this asserts
    the copies are byte-identical and returns the single value. Written here
    rather than in the shared helper per the brief: a shared helper that
    tolerated multiple hits would also tolerate a genuinely ambiguous match
    elsewhere, which is a silent wrong-fixture bug.
    """
    marker = f"fn {fn_name}"
    at = rs_text.find(marker)
    if at == -1:
        raise AssertionError(f"no `fn {fn_name}` in source")
    brace = rs_text.find("{", at)
    depth, i, n = 0, brace, len(rs_text)
    while i < n:
        if rs_text[i] == "{":
            depth += 1
        elif rs_text[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    hits = [x["value"] for x in find_string_literals(rs_text[brace:i + 1])
            if x["value"].startswith(prefix)]
    if not hits:
        raise AssertionError(f"`fn {fn_name}`: no literal starts with {prefix!r}")
    distinct = set(hits)
    if len(distinct) != 1:
        raise AssertionError(
            f"`fn {fn_name}`: {len(distinct)} DISTINCT literals start with "
            f"{prefix!r} -- ambiguous, refuse to guess")
    return hits[0]


def harness_envelope(command, stdout_pin):
    """`run|test --api browser --output json` envelope + an exact `json.stdout`.

    All five sources assert `json["stdout"].as_str().contains(<literal>)` and
    `assert_eq!(json["stderr"], "")` on this branch. A nested `json` leaf has
    no substring-assertion form in the case format (only exact equality), so
    per controller ruling 3 the `.contains` resolves to an exact pin -- and per
    U9 the pinned value is live-captured from the real `kali` binary
    (.cache/cargo-target/debug/kali, KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node)
    and checked to CONTAIN the source's own literal before being written, never
    hand-computed. None of the five asserts `errors` on this envelope.
    """
    j = envelope_harness(command, stderr=False, errors=False)
    j["stdout"] = stdout_pin
    j["stderr"] = ""
    return j


NO_RUST_COMMENTS = """\
RULE 12 (carry every source comment verbatim): `browser_{stem}.rs` contains no
Rust comments at all -- `grep -nE '^\\s*//'` finds nothing, and the only `//`
anywhere in the file is the `// kali-tree-shake:` marker on line 11, inside a
JS fixture body, which is program text and is carried verbatim into [source].
There is therefore no prose to move into any `rationale`, and no `///` doc
comment on any helper in either call chain (checked: `grep -n '///'` finds
nothing). comment_coverage.py is run with --allow-empty for this pair."""

RULE13_NOTE = """\
RULE 13 (transitive helper docs): every fn in both call chains was checked for
a `///` doc comment and none of this file's own helpers carries one. The bundle
chain reaches `kali_runtime_contract::browser_bundle_harness_script` and
`::browser_harness_command_parts_for`, which do carry one-line `///` docs, but
the migrated form never calls them -- the `browser_bundle_harness` step kind
means the case RUNNER does (design spec 5.3), so those docs describe shared
runner infrastructure, not what this case claims. Consistent with every case
file shipped before batch 4."""

U2_NOTE = """\
U2 ([source] is file-wide): safe here. All three fixtures are written
unconditionally into a fresh temp dir, none is behind an `if`, and no case's
point is a file's presence or absence -- every command names its entry
explicitly on argv. The `${ext}` suffix keeps each matrix cell's three files
distinct, so no [source] key needs U5 disambiguation."""


def _six_cases(*, entry, run_file, test_file, harness_body,
               bundle_harness_asserts, build_errors,
               harness_text_asserts, harness_stdout_pin,
               thread_flags, bundle_prose, harness_prose_text,
               harness_prose_json, name_build, name_harness):
    """The 6 [[case]] entries all five files in this group expand to.

    2 bundle cases (text / json envelope) + 4 harness cases (run|test x
    text/json). `command` and `json_output` are NOT matrix axes in any of these
    files: each changes the assertion SHAPE (json envelope vs raw stdout;
    `exitCode` for run vs `total`/`passed`/`failed` for test), which design
    spec 5.6's closing note excludes from a matrix. Only `ext` substitutes
    uniformly, so only `ext` is an axis.
    """
    cases = [
        {"name": name_build,
         "rationale": bundle_prose.format(mode="`kali build --bundle --api browser`"),
         "steps": bundle_steps(entry, harness_body, bundle_harness_asserts,
                               json_output=False, meta_fields=META)},
        {"name": f"json_{name_build}",
         "rationale": bundle_prose.format(
             mode="`kali build --bundle --api browser --output json`")
         + " This sibling additionally asserts the JSON build envelope: "
           "schemaVersion/command/success/exitCode and payload "
           "artifactKind/bundleFormat"
         + (", and that `errors` is the empty array."
            if build_errors else
            " -- the source makes no `errors` claim on this envelope, so none "
            "is written."),
         "steps": bundle_steps(entry, harness_body, bundle_harness_asserts,
                               json_output=True,
                               json_claims=envelope_build(errors=build_errors),
                               meta_fields=META)},
    ]
    for command, fname in (("run", run_file), ("test", test_file)):
        cases.append({
            "name": f"{command}_{name_harness}",
            "rationale": harness_prose_text.format(cmd=command),
            "steps": [harness_step(command, fname, json_output=False,
                                   thread_flags=thread_flags,
                                   asserts=harness_text_asserts)],
        })
    branches = {
        "run": "`exitCode` and `payload.exitCode` (the source's `run` branch)",
        "test": "`payload.total`/`passed`/`failed` (the source's `test` branch)",
    }
    for command, fname in (("run", run_file), ("test", test_file)):
        cases.append({
            "name": f"json_{command}_{name_harness}",
            "rationale": harness_prose_json.format(cmd=command,
                                                   branch=branches[command]),
            "steps": [harness_step(command, fname, json_output=True,
                                   thread_flags=thread_flags,
                                   json_claims=harness_envelope(
                                       command, harness_stdout_pin),
                                   asserts={})],
        })
    return cases


# --------------------------------------------------------------------------
# 1. browser_math_exp_log_mixed_root.rs -- 9 #[test] fns, 24 invocations.
#
# The design spec's own 5.6 worked example is titled with this file's path.
# That example is ILLUSTRATIVE AND INCOMPLETE -- it shows 2 cases x ext(4) = 8
# trials and does not account for the 9th #[test] fn (the inline-loop harness
# fn) or its 16 invocations at all. It is followed for house style only; the
# arithmetic below is derived from the real source.
# --------------------------------------------------------------------------
@target("math_exp_log_mixed_root")
def exp_log_mixed_root():
    stem = "math_exp_log_mixed_root"
    text = rs(stem)
    bundle_src = fixture_in_fn(
        text, "browser_bundle_global_this_math_bracketed_exp_log_source")
    run_src = fixture_in_fn(
        text, "browser_harness_global_this_math_bracketed_exp_log_run_source")
    test_src = fixture_in_fn(
        text, "browser_harness_global_this_math_bracketed_exp_log_test_source")
    harness_body = fixture_starting(
        text, "assert_browser_bundle_global_this_math_bracketed_exp_log",
        "const mod = await import(")

    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '1\\n0\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
Migrated from tests/browser_math_exp_log_mixed_root.rs.

NOTE ON THE DESIGN SPEC. Section 5.6's worked example is titled
`crates/kali_cli/tests/cases/browser/math_exp_log_mixed_root.toml` -- this
file's path. That example is illustrative and incomplete: it shows two
[[case]] entries fanned over ext(4) = 8 trials and does not account for the
source's 9th #[test] fn, the inline-loop harness fn, or any of its 16
invocations. Its house style (matrix axis for ext, sibling cases for the
text/json split) is followed; its case count is not, because the real file
has 24 invocations to cover.

{NO_RUST_COMMENTS.format(stem=stem)}

RULE 7 / U1 -- MATRIX ARITHMETIC, closes exactly. 9 #[test] fns, 24 real
invocations:
  * `assert_browser_bundle_global_this_math_bracketed_exp_log`(filename,
    json_output) -- 8 invocations from 8 one-line #[test] fns =
    ext(js/ts/jsx/tsx) x json_output(false/true), a full cross product
    (enumerated with tools/task-18-browser-pilot/enumerate_invocations.py).
  * `run_and_test_supports_global_this_math_bracketed_exp_log_identities_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input`
    -- ONE #[test] fn with an INLINE loop body (no assert_* helper, which is
    why enumerate_invocations.py reports it as UNPARSED rather than guessing).
    Read by hand: it loops over EIGHT `(command, source_name, source,
    expected_stdout)` tuples -- (run, main.js), (test, smoke.test.js),
    (run, main.ts), (test, smoke.test.ts), (run, main.jsx),
    (test, smoke.test.jsx), (run, main.tsx), (test, smoke.test.tsx) -- and,
    inside that, over `for output_json in [false, true]`: 8 x 2 = 16
    invocations. Unlike its siblings in this batch this loop DOES cover all
    four extensions, which its fn name states correctly.
Both groups therefore vary over ext(js/ts/jsx/tsx) uniformly and completely:
6 [[case]] x ext(4) = 24 trials = 24 invocations. The arithmetic closes, so
the axis is kept.

RULE 5 / RULE 6. The 16 harness invocations are NOT folded into one case: the
inline loop runs 8 independently-written programs, and each becomes its own
descriptively-named sibling [[case]] (4 of them, x 4 ext cells = 16 trials).
Per rule 6 the matrix fold is stated here -- each of the 2 bundle [[case]]
entries corresponds to 4 source #[test] fns (one per ext cell) and each of the
4 harness [[case]] entries corresponds to 4 iterations of the single looping
fn; the assertion mapping stays 1:1 per trial.

{U2_NOTE}

{RULE13_NOTE}

ASSERTION SHAPE, mirrored and nothing more. Bundle: `exit = "success"`; the
json sibling adds schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat -- the source asserts NO `errors` array on this
envelope (:64-73), so no `errors = []` is written; then the emitted
`app/app.meta.json` apiSurface/artifactKind; then the bundle-harness
`stdout_contains = ["1\\n", "0\\n"]` (:116-117, two plain `.contains` calls,
mirrored as two needles, not strengthened to an exact pin) and the harness
process's own `exit = "success"` (:109). Harness: argv DOES carry
`--max-threads 0 --max-spawned-processes 0` (:229-232); env
KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node; text mode asserts
`stdout.contains(expected_stdout)` where the loop's own literal is `1\\n0`
(:268), mirrored as `stdout_contains = ["1\\n0"]`; json mode asserts
schemaVersion/command/success/payload(hostContract, runtimeBackend),
`exitCode` + `payload.exitCode` for run or `payload.total/passed/failed` for
test, an exact `json.stdout` pin (see below) and `json.stderr = ""` (:265).
No `errors` claim on the harness envelope either.

U9 -- the exact `json.stdout` pin. The source's claim is
`json["stdout"].as_str().contains(expected_stdout)` (:258-264) with
`expected_stdout` = `1\\n0`; a json leaf has no substring form, so ruling 3
resolves it to an exact pin. Captured from the real binary for all 8
(ext x command) cells: every one returns `1\\n0\\n`, so the single pin is
correct in every matrix cell and it contains the source's literal."""

    bundle_prose = (
        "Migrated from browser_math_exp_log_mixed_root.rs. "
        "`assert_browser_bundle_global_this_math_bracketed_exp_log` builds a browser "
        "bundle ({mode}), asserts the emitted `app/app.meta.json` metadata, then runs "
        "the bundle glue under the browser-bundle-harness contract. The bundled program "
        "reaches Math through a mixed dotted/bracketed root -- `globalThis.Math[\"exp\"]` "
        "and `globalThis.Math[\"log\"]` -- and prints exp(0) = 1 then log(1) = 0. The "
        "source's two stdout claims are plain `.contains` against a field that has a "
        "substring form, so they are mirrored as `stdout_contains` and NOT strengthened "
        "to an exact pin (controller ruling 3)."
    )
    harness_common = (
        "Migrated from browser_math_exp_log_mixed_root.rs. This case is 4 of the 16 "
        "invocations made by the single #[test] fn `run_and_test_supports_global_this_"
        "math_bracketed_exp_log_identities_when_browser_harness_is_configured_in_js_ts_"
        "jsx_and_tsx_input`, whose body loops over eight (command, source_name, source, "
        "expected_stdout) tuples and then over `for output_json in [false, true]`; per "
        "the split-don't-fold rule each independent program becomes its own named "
        "sibling [[case]] rather than one folded case, and the four extensions the loop "
        "covers are the file's `ext` matrix axis. It runs `kali {cmd} --api browser "
        "--max-threads 0 --max-spawned-processes 0` with the browser harness backed by "
        "`node`. "
    )
    harness_text = harness_common + (
        "Text mode: a clean exit and `stdout.contains(\"1\\n0\")` -- the loop's own "
        "`expected_stdout` literal -- mirrored as `stdout_contains`."
    )
    harness_json = harness_common.replace("{cmd} --api", "{cmd} --api") + (
        "JSON mode (`--output json`): the envelope's schemaVersion/command/success and "
        "payload hostContract/runtimeBackend, plus {branch}, `stderr` exactly empty, "
        "and an exact `json.stdout` pin. The source claims "
        "`json[\"stdout\"].as_str().contains(\"1\\n0\")`; a json leaf has no substring "
        "form in this format, so ruling 3 resolves it to an exact pin, live-captured "
        "from the real `kali` binary and checked to contain the source's literal before "
        "being written. The source asserts no `errors` array on this envelope, so none "
        "is written."
    )

    cases = _six_cases(
        entry="app.${ext}", run_file="main.${ext}", test_file="smoke.test.${ext}",
        harness_body=harness_body,
        bundle_harness_asserts={"stdout_contains": ["1\n", "0\n"]},
        build_errors=False,
        harness_text_asserts={"stdout_contains": ["1\n0"]},
        harness_stdout_pin="1\n0\n",
        thread_flags=True,
        bundle_prose=bundle_prose,
        harness_prose_text=harness_text,
        harness_prose_json=harness_json,
        name_build="build_emits_global_this_math_bracketed_exp_log_identity_literals",
        name_harness="supports_global_this_math_bracketed_exp_log_identities_"
                     "when_browser_harness_is_configured",
    )
    return (f"{stem}.toml", header, {"ext": ["js", "ts", "jsx", "tsx"]},
            {"app.${ext}": bundle_src, "main.${ext}": run_src,
             "smoke.test.${ext}": test_src},
            cases)


# --------------------------------------------------------------------------
# 2. browser_math_expm1_log1p_bracketed_root.rs -- 6 #[test] fns, 24 invocations.
# --------------------------------------------------------------------------
@target("math_expm1_log1p_bracketed_root")
def expm1_log1p_bracketed_root():
    stem = "math_expm1_log1p_bracketed_root"
    text = rs(stem)
    loop_fn = ("run_and_test_supports_bracketed_global_this_math_expm1_log1p_"
               "identities_when_browser_harness_is_configured_in_js_and_ts_input")
    bundle_src = fixture_in_fn(
        text, "browser_bundle_bracketed_global_this_math_expm1_log1p_source")
    harness_body = fixture_starting(
        text, "assert_browser_bundle_bracketed_global_this_math_expm1_log1p",
        "const mod = await import(")
    run_src = repeated_fixture_in_fn(text, loop_fn, "const zero = 0;")
    test_src = repeated_fixture_in_fn(text, loop_fn, "Kali.test(")

    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '0\\n0\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
Migrated from tests/browser_math_expm1_log1p_bracketed_root.rs.

{NO_RUST_COMMENTS.format(stem=stem)}

RULE 7 / U1 -- MATRIX ARITHMETIC, closes exactly. 6 #[test] fns, 24 real
invocations. (HISTORICAL NOTE, fix round 1 / I1: this paragraph used to say
enumerate_invocations.py reported only 6 here and called that a permanent tool
limitation. That WAS true when this file was written and it is no longer: the
undercount -- the parser taking only the FIRST `assert_*` call in a fn body, so
the second call inside the jsx/tsx loop was dropped -- was found during this
same batch and FIXED in the same commit that shipped this file. The repaired
tool now reports TOTAL INVOCATIONS: 8, agreeing with the hand count below.
Re-run it rather than trusting either number from prose.) Counted by hand from
the source, and now also confirmed by the tool:
  * `assert_browser_bundle_bracketed_global_this_math_expm1_log1p`(filename,
    json_output) -- 8 invocations. Four one-line #[test] fns give
    (app.js,false), (app.ts,false), (app.js,true), (app.ts,true); the fifth,
    `build_emits_bracketed_global_this_math_expm1_log1p_identity_literals_in_jsx_and_tsx_input`,
    loops `for filename in ["app.jsx", "app.tsx"]` and calls the helper TWICE
    per iteration (false then true), giving the other four. Together: a full
    ext(4) x json_output(2) cross product.
  * `{loop_fn}`
    -- ONE #[test] fn with an INLINE loop body and no assert_* helper (which
    is why the tool reports it as UNPARSED rather than guessing). Read by
    hand: eight `(command, source_name, source)` tuples -- run/test x
    js/ts/jsx/tsx -- crossed with `for output_json in [false, true]` = 16
    invocations.
Both groups vary over ext(js/ts/jsx/tsx) uniformly and completely:
6 [[case]] x ext(4) = 24 trials = 24 invocations. The arithmetic closes, so
the axis is kept.

MIGRATION NOTE (source fn name, not a comment -- carried here because a
reader will otherwise mistrust the arithmetic above): the looping #[test]
fn's name ends `_in_js_and_ts_input`, but its tuple list actually covers
js, ts, jsx AND tsx. The name is stale; the loop body is authoritative and is
what the `ext` axis is derived from. Nothing is silently corrected -- the fn
keeps its name in the .rs.

RULE 5 / RULE 6. The 16 harness invocations run 8 independently-written
programs, so they become 4 descriptively-named sibling [[case]] entries
(x 4 ext cells), never one folded case. Per rule 6 the matrix fold is stated
here: each bundle [[case]] corresponds to 4 source invocations spread across
five #[test] fns, and each harness [[case]] to 4 iterations of the single
looping fn; the assertion mapping stays 1:1 per trial.

{U2_NOTE}
Both harness fixtures are written inline in the looping fn's tuple list, once
per extension -- four byte-identical copies of the run source and four of the
test source. This file's generator
(tools/task-18-browser-pilot/gen_batch4_group_c.py) asserts that byte-identity
mechanically before collapsing them to one [source] entry -- U13's "assert the
identity, don't eyeball it" -- rather than assuming it.

{RULE13_NOTE}

ASSERTION SHAPE, mirrored and nothing more. Bundle: `exit = "success"`; the
json sibling adds schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat -- the source asserts NO `errors` array on this
envelope (:49-58), so no `errors = []` is written; then the emitted
`app/app.meta.json` apiSurface/artifactKind; then the bundle-harness
`stdout_contains = ["0\\n"]` (:101, a SINGLE plain `.contains`, unlike the
mixed-root sibling file's two) and the harness process's own
`exit = "success"` (:94). Harness: argv DOES carry `--max-threads 0
--max-spawned-processes 0` (:193-196); env
KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node; text mode asserts
`stdout.contains("0\\n")` (:232); json mode asserts
schemaVersion/command/success/payload(hostContract, runtimeBackend),
`exitCode` + `payload.exitCode` for run or `payload.total/passed/failed` for
test, an exact `json.stdout` pin and `json.stderr = ""` (:229). No `errors`
claim on the harness envelope either.

U9 -- the exact `json.stdout` pin. The source's claim is
`json["stdout"].as_str().contains("0\\n")` (:222-228); a json leaf has no
substring form, so ruling 3 resolves it to an exact pin. Captured from the
real binary for all 8 (ext x command) cells: every one returns `0\\n0\\n`
(expm1(0) = 0 and log1p(0) = 0), so the single pin holds in every matrix cell
and it contains the source's literal."""

    bundle_prose = (
        "Migrated from browser_math_expm1_log1p_bracketed_root.rs. "
        "`assert_browser_bundle_bracketed_global_this_math_expm1_log1p` builds a browser "
        "bundle ({mode}), asserts the emitted `app/app.meta.json` metadata, then runs the "
        "bundle glue under the browser-bundle-harness contract. The bundled program "
        "reaches Math through a bracketed root with dotted members -- "
        "`globalThis[\"Math\"].expm1` and `globalThis[\"Math\"].log1p` -- and prints "
        "expm1(0) = 0 then log1p(0) = 0. The source makes a single plain "
        "`.contains(\"0\\n\")` claim here, mirrored as `stdout_contains` and NOT "
        "strengthened to an exact pin (controller ruling 3)."
    )
    harness_common = (
        "Migrated from browser_math_expm1_log1p_bracketed_root.rs. This case is 4 of the "
        "16 invocations made by the single #[test] fn `run_and_test_supports_bracketed_"
        "global_this_math_expm1_log1p_identities_when_browser_harness_is_configured_in_"
        "js_and_ts_input`, whose body loops over eight (command, source_name, source) "
        "tuples and then over `for output_json in [false, true]`; per the split-don't-fold "
        "rule each independent program becomes its own named sibling [[case]] rather than "
        "one folded case, and the four extensions the loop covers -- despite the fn name "
        "saying js and ts, the tuple list also covers jsx and tsx -- are the file's `ext` "
        "matrix axis. It runs `kali {cmd} --api browser --max-threads 0 "
        "--max-spawned-processes 0` with the browser harness backed by `node`. "
    )
    harness_text = harness_common + (
        "Text mode: a clean exit and `stdout.contains(\"0\\n\")`, mirrored as "
        "`stdout_contains`."
    )
    harness_json = harness_common + (
        "JSON mode (`--output json`): the envelope's schemaVersion/command/success and "
        "payload hostContract/runtimeBackend, plus {branch}, `stderr` exactly empty, "
        "and an exact `json.stdout` pin. The source claims "
        "`json[\"stdout\"].as_str().contains(\"0\\n\")`; a json leaf has no substring "
        "form in this format, so ruling 3 resolves it to an exact pin, live-captured from "
        "the real `kali` binary and checked to contain the source's literal before being "
        "written. The source asserts no `errors` array on this envelope, so none is "
        "written."
    )

    cases = _six_cases(
        entry="app.${ext}", run_file="main.${ext}", test_file="smoke.test.${ext}",
        harness_body=harness_body,
        bundle_harness_asserts={"stdout_contains": ["0\n"]},
        build_errors=False,
        harness_text_asserts={"stdout_contains": ["0\n"]},
        harness_stdout_pin="0\n0\n",
        thread_flags=True,
        bundle_prose=bundle_prose,
        harness_prose_text=harness_text,
        harness_prose_json=harness_json,
        name_build="build_emits_bracketed_global_this_math_expm1_log1p_identity_literals",
        name_harness="supports_bracketed_global_this_math_expm1_log1p_identities_"
                     "when_browser_harness_is_configured",
    )
    return (f"{stem}.toml", header, {"ext": ["js", "ts", "jsx", "tsx"]},
            {"app.${ext}": bundle_src, "main.${ext}": run_src,
             "smoke.test.${ext}": test_src},
            cases)


# --------------------------------------------------------------------------
# 3. browser_math_expm1_log1p_mixed_root.rs -- 6 #[test] fns, 24 invocations.
# --------------------------------------------------------------------------
@target("math_expm1_log1p_mixed_root")
def expm1_log1p_mixed_root():
    stem = "math_expm1_log1p_mixed_root"
    text = rs(stem)
    bundle_src = fixture_in_fn(
        text, "browser_bundle_global_this_math_bracketed_expm1_log1p_source")
    run_src = fixture_in_fn(
        text, "browser_harness_global_this_math_bracketed_expm1_log1p_run_source")
    test_src = fixture_in_fn(
        text, "browser_harness_global_this_math_bracketed_expm1_log1p_test_source")
    harness_body = fixture_starting(
        text, "assert_browser_bundle_global_this_math_bracketed_expm1_log1p",
        "const mod = await import(")

    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '0\\n0\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
Migrated from tests/browser_math_expm1_log1p_mixed_root.rs.

{NO_RUST_COMMENTS.format(stem=stem)}

RULE 7 / U1 -- MATRIX ARITHMETIC, closes exactly. 6 #[test] fns, 24 real
invocations. (HISTORICAL NOTE, fix round 1 / I1: this paragraph used to say
enumerate_invocations.py reported only 6 here and called that a permanent tool
limitation. That WAS true when this file was written and it is no longer: the
undercount -- the parser taking only the FIRST `assert_*` call in a fn body, so
the second call inside the jsx/tsx loop was dropped -- was found during this
same batch and FIXED in the same commit that shipped this file. The repaired
tool now reports TOTAL INVOCATIONS: 8, agreeing with the hand count below.
Re-run it rather than trusting either number from prose.) Counted by hand from
the source, and now also confirmed by the tool:
  * `assert_browser_bundle_global_this_math_bracketed_expm1_log1p`(filename,
    json_output) -- 8 invocations. Four one-line #[test] fns give
    (app.js,false), (app.ts,false), (app.js,true), (app.ts,true); the fifth,
    `build_emits_global_this_math_bracketed_expm1_log1p_identity_literals_in_jsx_and_tsx_input`,
    loops `for filename in ["app.jsx", "app.tsx"]` and calls the helper TWICE
    per iteration (false then true), giving the other four. Together: a full
    ext(4) x json_output(2) cross product.
  * `run_and_test_supports_global_this_math_bracketed_expm1_log1p_identities_when_browser_harness_is_configured_in_js_and_ts_input`
    -- ONE #[test] fn with an INLINE loop body and no assert_* helper (which
    is why the tool reports it as UNPARSED rather than guessing). Read by
    hand: eight
    `(command, source_name, source, expected_stdout)` tuples -- run/test x
    js/ts/jsx/tsx -- crossed with `for output_json in [false, true]` = 16
    invocations.
Both groups vary over ext(js/ts/jsx/tsx) uniformly and completely:
6 [[case]] x ext(4) = 24 trials = 24 invocations. The arithmetic closes, so
the axis is kept.

MIGRATION NOTE (source fn name, not a comment -- carried here because a
reader will otherwise mistrust the arithmetic above): the looping #[test]
fn's name ends `_in_js_and_ts_input`, but its tuple list actually covers
js, ts, jsx AND tsx. The name is stale; the loop body is authoritative and is
what the `ext` axis is derived from. Nothing is silently corrected -- the fn
keeps its name in the .rs.

RULE 5 / RULE 6. The 16 harness invocations run 8 independently-written
programs, so they become 4 descriptively-named sibling [[case]] entries
(x 4 ext cells), never one folded case. Per rule 6 the matrix fold is stated
here: each bundle [[case]] corresponds to 4 source invocations spread across
five #[test] fns, and each harness [[case]] to 4 iterations of the single
looping fn; the assertion mapping stays 1:1 per trial.

{U2_NOTE}

{RULE13_NOTE}

ASSERTION SHAPE, mirrored and nothing more. Bundle: `exit = "success"`; the
json sibling adds schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat -- the source asserts NO `errors` array on this
envelope (:62-71), so no `errors = []` is written; then the emitted
`app/app.meta.json` apiSurface/artifactKind; then the bundle-harness
`stdout_contains = ["0\\n"]` and the harness process's own `exit = "success"`
(:107). MIGRATION NOTE on that needle: the source writes the SAME assertion
twice, `assert!(stdout.contains("0\\n"))` on :114 and again verbatim on :115.
Two identical claims are one claim -- `stdout_contains` is a set of needles
and listing `"0\\n"` twice would assert nothing further -- so it appears once.
No claim is dropped: the duplicated line is satisfied by exactly the same
check. Harness: argv DOES carry `--max-threads 0 --max-spawned-processes 0`
(:215-218); env KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node; text mode asserts
`stdout.contains(expected_stdout)` where the loop's own literal is `0\\n0`
(:254); json mode asserts schemaVersion/command/success/payload(hostContract,
runtimeBackend), `exitCode` + `payload.exitCode` for run or
`payload.total/passed/failed` for test, an exact `json.stdout` pin and
`json.stderr = ""` (:251). No `errors` claim on the harness envelope either.

U9 -- the exact `json.stdout` pin. The source's claim is
`json["stdout"].as_str().contains(expected_stdout)` (:244-250) with
`expected_stdout` = `0\\n0`; a json leaf has no substring form, so ruling 3
resolves it to an exact pin. Captured from the real binary for all 8
(ext x command) cells: every one returns `0\\n0\\n`, so the single pin holds
in every matrix cell and it contains the source's literal."""

    bundle_prose = (
        "Migrated from browser_math_expm1_log1p_mixed_root.rs. "
        "`assert_browser_bundle_global_this_math_bracketed_expm1_log1p` builds a browser "
        "bundle ({mode}), asserts the emitted `app/app.meta.json` metadata, then runs the "
        "bundle glue under the browser-bundle-harness contract. The bundled program "
        "reaches Math through a mixed dotted/bracketed root -- "
        "`globalThis.Math[\"expm1\"]` and `globalThis.Math[\"log1p\"]` -- and prints "
        "expm1(0) = 0 then log1p(0) = 0. The source asserts "
        "`stdout.contains(\"0\\n\")` twice, verbatim, on two consecutive lines; two "
        "identical plain `.contains` claims are one claim, so the needle appears once in "
        "`stdout_contains`, mirrored and not strengthened to an exact pin (controller "
        "ruling 3)."
    )
    harness_common = (
        "Migrated from browser_math_expm1_log1p_mixed_root.rs. This case is 4 of the 16 "
        "invocations made by the single #[test] fn `run_and_test_supports_global_this_"
        "math_bracketed_expm1_log1p_identities_when_browser_harness_is_configured_in_js_"
        "and_ts_input`, whose body loops over eight (command, source_name, source, "
        "expected_stdout) tuples and then over `for output_json in [false, true]`; per the "
        "split-don't-fold rule each independent program becomes its own named sibling "
        "[[case]] rather than one folded case, and the four extensions the loop covers -- "
        "despite the fn name saying js and ts, the tuple list also covers jsx and tsx -- "
        "are the file's `ext` matrix axis. It runs `kali {cmd} --api browser "
        "--max-threads 0 --max-spawned-processes 0` with the browser harness backed by "
        "`node`. "
    )
    harness_text = harness_common + (
        "Text mode: a clean exit and `stdout.contains(\"0\\n0\")` -- the loop's own "
        "`expected_stdout` literal -- mirrored as `stdout_contains`."
    )
    harness_json = harness_common + (
        "JSON mode (`--output json`): the envelope's schemaVersion/command/success and "
        "payload hostContract/runtimeBackend, plus {branch}, `stderr` exactly empty, "
        "and an exact `json.stdout` pin. The source claims "
        "`json[\"stdout\"].as_str().contains(\"0\\n0\")`; a json leaf has no substring "
        "form in this format, so ruling 3 resolves it to an exact pin, live-captured from "
        "the real `kali` binary and checked to contain the source's literal before being "
        "written. The source asserts no `errors` array on this envelope, so none is "
        "written."
    )

    cases = _six_cases(
        entry="app.${ext}", run_file="main.${ext}", test_file="smoke.test.${ext}",
        harness_body=harness_body,
        bundle_harness_asserts={"stdout_contains": ["0\n"]},
        build_errors=False,
        harness_text_asserts={"stdout_contains": ["0\n0"]},
        harness_stdout_pin="0\n0\n",
        thread_flags=True,
        bundle_prose=bundle_prose,
        harness_prose_text=harness_text,
        harness_prose_json=harness_json,
        name_build="build_emits_global_this_math_bracketed_expm1_log1p_identity_literals",
        name_harness="supports_global_this_math_bracketed_expm1_log1p_identities_"
                     "when_browser_harness_is_configured",
    )
    return (f"{stem}.toml", header, {"ext": ["js", "ts", "jsx", "tsx"]},
            {"app.${ext}": bundle_src, "main.${ext}": run_src,
             "smoke.test.${ext}": test_src},
            cases)


# --------------------------------------------------------------------------
# 4. browser_math_expm1_log1p_fully_bracketed_root.rs -- 5 fns, 12 invocations.
# --------------------------------------------------------------------------
@target("math_expm1_log1p_fully_bracketed_root")
def expm1_log1p_fully_bracketed_root():
    stem = "math_expm1_log1p_fully_bracketed_root"
    text = rs(stem)
    bundle_src = fixture_in_fn(
        text, "browser_bundle_fully_bracketed_global_this_math_expm1_log1p_source")
    run_src = fixture_in_fn(
        text, "browser_harness_fully_bracketed_global_this_math_expm1_log1p_run_source")
    test_src = fixture_in_fn(
        text, "browser_harness_fully_bracketed_global_this_math_expm1_log1p_test_source")
    harness_body = fixture_starting(
        text, "assert_browser_bundle_fully_bracketed_global_this_math_expm1_log1p",
        "const mod = await import(")

    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '0\\n0\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
Migrated from tests/browser_math_expm1_log1p_fully_bracketed_root.rs.

{NO_RUST_COMMENTS.format(stem=stem)}

RULE 7 / U1 -- MATRIX ARITHMETIC, closes exactly, over a TWO-VALUE axis.
5 #[test] fns, 12 real invocations (enumerated with
tools/task-18-browser-pilot/enumerate_invocations.py, which parses this file
completely because both groups call a named assert_* helper):
  * `assert_browser_bundle_fully_bracketed_global_this_math_expm1_log1p`(
    filename, json_output) -- 4 invocations from 4 one-line #[test] fns =
    ext(js/ts) x json_output(false/true). NOTE what is absent: unlike its
    bracketed-root and mixed-root siblings in this batch, this file has NO
    jsx/tsx bundle tests at all.
  * `assert_browser_harness_fully_bracketed_global_this_math_expm1_log1p`(
    command, filename, source, json_output) -- 8 invocations from the single
    #[test] fn
    `run_and_test_supports_fully_bracketed_global_this_math_expm1_log1p_identities_when_browser_harness_is_configured_in_js_and_ts_input`,
    which loops over four `(command, source_name, source)` tuples --
    (run, main.js), (test, smoke.test.js), (run, main.ts),
    (test, smoke.test.ts) -- and then over `for output_json in [false, true]`.
    Here the fn name is accurate: js and ts only.
Both groups vary over ext(js/ts) uniformly and completely, and NEITHER covers
jsx/tsx, so the axis is `["js", "ts"]` and not the four-value axis used by
this batch's other root-shape files: 6 [[case]] x ext(2) = 12 trials = 12
invocations. Adding jsx/tsx to the axis would fan every case over four cells
the source never runs -- inventing untested combinations (rule 2) and breaking
the arithmetic (rule 7) -- and `[matrix]` is file-wide with no per-case opt-out
(U1), so the axis values are exactly what the source exercises.

RULE 5 / RULE 6. The 8 harness invocations run 4 independently-written
programs, so they become 4 descriptively-named sibling [[case]] entries
(x 2 ext cells), never one folded case. Per rule 6 the matrix fold is stated
here: each bundle [[case]] corresponds to 2 source #[test] fns (one per ext
cell) and each harness [[case]] to 2 iterations of the single looping fn; the
assertion mapping stays 1:1 per trial.

{U2_NOTE}

{RULE13_NOTE}

ASSERTION SHAPE, mirrored and nothing more. Bundle: `exit = "success"`; the
json sibling adds schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat -- the source asserts NO `errors` array on this
envelope (:65-74), so no `errors = []` is written; then the emitted
`app/app.meta.json` apiSurface/artifactKind; then the bundle-harness
`stdout_contains = ["0\\n"]` (:117, a single plain `.contains`) and the
harness process's own `exit = "success"` (:110). Harness: argv DOES carry
`--max-threads 0 --max-spawned-processes 0` (:141-144); env
KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node; text mode asserts
`stdout.contains("0\\n")` (:181); json mode asserts
schemaVersion/command/success/payload(hostContract, runtimeBackend),
`exitCode` + `payload.exitCode` for run or `payload.total/passed/failed` for
test, an exact `json.stdout` pin and `json.stderr = ""` (:178). No `errors`
claim on the harness envelope either.

U9 -- the exact `json.stdout` pin. The source's claim is
`json["stdout"].as_str().contains("0\\n")` (:171-177); a json leaf has no
substring form, so ruling 3 resolves it to an exact pin. Captured from the
real binary for all 4 (ext x command) cells: every one returns `0\\n0\\n`
(expm1(0) = 0 and log1p(0) = 0), so the single pin holds in both matrix cells
and it contains the source's literal."""

    bundle_prose = (
        "Migrated from browser_math_expm1_log1p_fully_bracketed_root.rs. "
        "`assert_browser_bundle_fully_bracketed_global_this_math_expm1_log1p` builds a "
        "browser bundle ({mode}), asserts the emitted `app/app.meta.json` metadata, then "
        "runs the bundle glue under the browser-bundle-harness contract. The bundled "
        "program reaches Math through a fully bracketed root -- "
        "`globalThis[\"Math\"][\"expm1\"]` and `globalThis[\"Math\"][\"log1p\"]` -- and "
        "prints expm1(0) = 0 then log1p(0) = 0. The source makes a single plain "
        "`.contains(\"0\\n\")` claim here, mirrored as `stdout_contains` and NOT "
        "strengthened to an exact pin (controller ruling 3)."
    )
    harness_common = (
        "Migrated from browser_math_expm1_log1p_fully_bracketed_root.rs. "
        "`assert_browser_harness_fully_bracketed_global_this_math_expm1_log1p` runs "
        "`kali {cmd} --api browser --max-threads 0 --max-spawned-processes 0` with the "
        "browser harness backed by `node`. This case is 2 of the 8 invocations the single "
        "#[test] fn `run_and_test_supports_fully_bracketed_global_this_math_expm1_log1p_"
        "identities_when_browser_harness_is_configured_in_js_and_ts_input` makes by "
        "looping over four (command, source_name, source) tuples and then over "
        "`for output_json in [false, true]`; per the split-don't-fold rule each "
        "independent program becomes its own named sibling [[case]], and the two "
        "extensions the loop covers are the file's `ext` matrix axis -- js and ts only, "
        "since this file has no jsx/tsx tests in either group. "
    )
    harness_text = harness_common + (
        "Text mode: a clean exit and `stdout.contains(\"0\\n\")`, mirrored as "
        "`stdout_contains`."
    )
    harness_json = harness_common + (
        "JSON mode (`--output json`): the envelope's schemaVersion/command/success and "
        "payload hostContract/runtimeBackend, plus {branch}, `stderr` exactly empty, "
        "and an exact `json.stdout` pin. The source claims "
        "`json[\"stdout\"].as_str().contains(\"0\\n\")`; a json leaf has no substring "
        "form in this format, so ruling 3 resolves it to an exact pin, live-captured from "
        "the real `kali` binary and checked to contain the source's literal before being "
        "written. The source asserts no `errors` array on this envelope, so none is "
        "written."
    )

    cases = _six_cases(
        entry="app.${ext}", run_file="main.${ext}", test_file="smoke.test.${ext}",
        harness_body=harness_body,
        bundle_harness_asserts={"stdout_contains": ["0\n"]},
        build_errors=False,
        harness_text_asserts={"stdout_contains": ["0\n"]},
        harness_stdout_pin="0\n0\n",
        thread_flags=True,
        bundle_prose=bundle_prose,
        harness_prose_text=harness_text,
        harness_prose_json=harness_json,
        name_build="build_emits_fully_bracketed_global_this_math_expm1_log1p_"
                   "identity_literals",
        name_harness="supports_fully_bracketed_global_this_math_expm1_log1p_identities_"
                     "when_browser_harness_is_configured",
    )
    return (f"{stem}.toml", header, {"ext": ["js", "ts"]},
            {"app.${ext}": bundle_src, "main.${ext}": run_src,
             "smoke.test.${ext}": test_src},
            cases)


# --------------------------------------------------------------------------
# 5. browser_math_log2_log10_fully_bracketed_root.rs -- 5 fns, 12 invocations.
#    The only file in this group carrying a `.matches().count()` claim.
# --------------------------------------------------------------------------
@target("math_log2_log10_fully_bracketed_root")
def log2_log10_fully_bracketed_root():
    stem = "math_log2_log10_fully_bracketed_root"
    text = rs(stem)
    bundle_src = fixture_in_fn(
        text, "browser_bundle_fully_bracketed_global_this_math_log2_log10_source")
    run_src = fixture_in_fn(
        text, "browser_harness_fully_bracketed_global_this_math_log2_log10_run_source")
    test_src = fixture_in_fn(
        text, "browser_harness_fully_bracketed_global_this_math_log2_log10_test_source")
    harness_body = fixture_starting(
        text, "assert_browser_bundle_fully_bracketed_global_this_math_log2_log10",
        "const mod = await import(")

    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '3\\n3\\n3\\n3\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
Migrated from tests/browser_math_log2_log10_fully_bracketed_root.rs.

{NO_RUST_COMMENTS.format(stem=stem)}

RULE 7 / U1 -- MATRIX ARITHMETIC, closes exactly, over a TWO-VALUE axis.
5 #[test] fns, 12 real invocations (enumerated with
tools/task-18-browser-pilot/enumerate_invocations.py, which parses this file
completely because both groups call a named assert_* helper):
  * `assert_browser_bundle_fully_bracketed_global_this_math_log2_log10`(
    filename, json_output) -- 4 invocations from 4 one-line #[test] fns =
    ext(js/ts) x json_output(false/true). This file has NO jsx/tsx tests.
  * `assert_browser_harness_fully_bracketed_global_this_math_log2_log10`(
    command, filename, source, json_output) -- 8 invocations from the single
    #[test] fn
    `run_and_test_supports_fully_bracketed_global_this_math_log2_log10_identities_when_browser_harness_is_configured_in_js_and_ts_input`,
    which loops over four `(command, source_name, source)` tuples --
    (run, main.js), (test, smoke.test.js), (run, main.ts),
    (test, smoke.test.ts) -- and then over `for output_json in [false, true]`.
    The fn name is accurate here: js and ts only.
Both groups vary over ext(js/ts) uniformly and completely, and NEITHER covers
jsx/tsx, so the axis is `["js", "ts"]`: 6 [[case]] x ext(2) = 12 trials = 12
invocations. Adding jsx/tsx would fan every case over cells the source never
runs (rule 2) and break the arithmetic (rule 7); `[matrix]` is file-wide with
no per-case opt-out (U1), so the axis values are exactly what is exercised.

RULE 5 / RULE 6. The 8 harness invocations run 4 independently-written
programs, so they become 4 descriptively-named sibling [[case]] entries
(x 2 ext cells), never one folded case. Per rule 6 the matrix fold is stated
here: each bundle [[case]] corresponds to 2 source #[test] fns (one per ext
cell) and each harness [[case]] to 2 iterations of the single looping fn; the
assertion mapping stays 1:1 per trial.

{U2_NOTE}

{RULE13_NOTE}

THE COUNT CLAIM (ruling 3 -- mirror the source). ONE site: `:132`,
`assert!(stdout.matches("3\\n").count() >= 2)`, on the RAW stdout of the
bundle harness process, carried as
`stdout_count = [{{ needle = "3\\n", at_least = 2 }}]`. It is not weakened to
a `*_contains` (one occurrence would satisfy that, and this program's whole
point is that four independent evaluations each print 3) and it is not
strengthened to `exact` even though the live output is observably `3\\n` four
times: the source states a lower bound, and pinning equality would assert
something it never did (rule 2). The immediately preceding line `:131`,
`assert!(stdout.contains("3\\n"))`, is a SEPARATE source claim about the same
needle and is carried as well, as `stdout_contains = ["3\\n"]` -- both are
kept, per the rule that a `.contains` and a `.count()` about one needle are
two claims. There is NO `json_count` anywhere in this file: the harness
helper's `if json_output` branch (:186-192) makes only a plain
`.contains("3\\n")` against `json["stdout"].as_str()`, never a count, so
inventing a `json_count` there would be a rule 2 violation.

ASSERTION SHAPE, mirrored and nothing more. Bundle: `exit = "success"`; the
json sibling adds schemaVersion/command/success/exitCode, payload
artifactKind/bundleFormat, AND `errors = []` -- this file DOES assert the
build envelope's errors array is empty (:84-87), unlike the four other
root-shape files migrated alongside it, which do not; then the emitted
`app/app.meta.json` apiSurface/artifactKind; then the bundle-harness
`stdout_contains` + `stdout_count` above and the harness process's own
`exit = "success"` (:124). Harness: argv DOES carry `--max-threads 0
--max-spawned-processes 0` (:156-159); env
KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node; text mode asserts
`stdout.contains("3\\n")` (:196); json mode asserts
schemaVersion/command/success/payload(hostContract, runtimeBackend),
`exitCode` + `payload.exitCode` for run or `payload.total/passed/failed` for
test, an exact `json.stdout` pin and `json.stderr = ""` (:193). The harness
envelope gets NO `errors` claim -- the source makes none there, only on the
build envelope.

U9 -- the exact `json.stdout` pin. The source's claim is
`json["stdout"].as_str().contains("3\\n")` (:186-192); a json leaf has no
substring form, so ruling 3 resolves it to an exact pin. Captured from the
real binary for all 4 (ext x command) cells: every one returns
`3\\n3\\n3\\n3\\n` -- log2(8), the frozen alias of log2 applied to 8,
log10(1000) and the frozen alias of log10 applied to 1000 all print 3 -- so
the single pin holds in both matrix cells and it contains the source's
literal."""

    bundle_prose = (
        "Migrated from browser_math_log2_log10_fully_bracketed_root.rs. "
        "`assert_browser_bundle_fully_bracketed_global_this_math_log2_log10` builds a "
        "browser bundle ({mode}), asserts the emitted `app/app.meta.json` metadata, then "
        "runs the bundle glue under the browser-bundle-harness contract. The bundled "
        "program reaches Math through a fully bracketed root -- "
        "`globalThis[\"Math\"][\"log2\"]` and `globalThis[\"Math\"][\"log10\"]` -- and "
        "calls each both directly and through an `Object.freeze`d reference, printing 3 "
        "four times. The source makes TWO separate claims about the needle `3\\n` on this "
        "output and both are carried: a plain `.contains` (:131), mirrored as "
        "`stdout_contains`, and `stdout.matches(\"3\\n\").count() >= 2` (:132), mirrored "
        "as `stdout_count` with `at_least = 2`. The count is not weakened to a "
        "`*_contains` (one occurrence would satisfy that) and not strengthened to "
        "`exact` (the source states a lower bound), per controller ruling 3 -- mirror the "
        "source."
    )
    harness_common = (
        "Migrated from browser_math_log2_log10_fully_bracketed_root.rs. "
        "`assert_browser_harness_fully_bracketed_global_this_math_log2_log10` runs "
        "`kali {cmd} --api browser --max-threads 0 --max-spawned-processes 0` with the "
        "browser harness backed by `node`, against a program that computes log2(8) and "
        "log10(1000) through a fully bracketed `globalThis[\"Math\"][...]` root, both "
        "directly and through `Object.freeze`d references, printing 3 four times. This "
        "case is 2 of the 8 invocations the single #[test] fn `run_and_test_supports_"
        "fully_bracketed_global_this_math_log2_log10_identities_when_browser_harness_is_"
        "configured_in_js_and_ts_input` makes by looping over four (command, source_name, "
        "source) tuples and then over `for output_json in [false, true]`; per the "
        "split-don't-fold rule each independent program becomes its own named sibling "
        "[[case]], and the two extensions the loop covers are the file's `ext` matrix "
        "axis -- js and ts only, since this file has no jsx/tsx tests in either group. "
    )
    harness_text = harness_common + (
        "Text mode: a clean exit and `stdout.contains(\"3\\n\")`, mirrored as "
        "`stdout_contains`. NOTE that the harness helper makes no count claim -- the "
        "`.matches(\"3\\n\").count() >= 2` assertion lives only on the bundle-harness "
        "path, so no `stdout_count` is written here (inventing one would be a rule 2 "
        "violation)."
    )
    harness_json = harness_common + (
        "JSON mode (`--output json`): the envelope's schemaVersion/command/success and "
        "payload hostContract/runtimeBackend, plus {branch}, `stderr` exactly empty, "
        "and an exact `json.stdout` pin. The source claims "
        "`json[\"stdout\"].as_str().contains(\"3\\n\")` -- a plain `.contains`, NOT a "
        "count, so no `json_count` is written; a json leaf has no substring form in this "
        "format, so ruling 3 resolves the claim to an exact pin, live-captured from the "
        "real `kali` binary and checked to contain the source's literal before being "
        "written. The source asserts `errors` empty only on the build envelope, never on "
        "this one, so no `errors` claim is written here."
    )

    cases = _six_cases(
        entry="app.${ext}", run_file="main.${ext}", test_file="smoke.test.${ext}",
        harness_body=harness_body,
        bundle_harness_asserts={
            "stdout_contains": ["3\n"],
            "stdout_count": [{"needle": "3\n", "at_least": 2}],
        },
        build_errors=True,
        harness_text_asserts={"stdout_contains": ["3\n"]},
        harness_stdout_pin="3\n3\n3\n3\n",
        thread_flags=True,
        bundle_prose=bundle_prose,
        harness_prose_text=harness_text,
        harness_prose_json=harness_json,
        name_build="build_emits_fully_bracketed_global_this_math_log2_log10_"
                   "identity_literals",
        name_harness="supports_fully_bracketed_global_this_math_log2_log10_identities_"
                     "when_browser_harness_is_configured",
    )
    return (f"{stem}.toml", header, {"ext": ["js", "ts"]},
            {"app.${ext}": bundle_src, "main.${ext}": run_src,
             "smoke.test.${ext}": test_src},
            cases)


def main(argv):
    names = argv or sorted(REGISTRY)
    for name in names:
        if name not in REGISTRY:
            raise SystemExit(f"unknown target {name!r}; known: {sorted(REGISTRY)}")
        out, header, matrix, source, cases = REGISTRY[name]()
        write(os.path.join(CASES, out), emit(header.split("\n"), matrix, source, cases))


if __name__ == "__main__":
    main(sys.argv[1:])
