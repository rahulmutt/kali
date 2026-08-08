#!/usr/bin/env python3
"""Generate the batch 4 GROUP D case files (5 targets).

A separate module from `gen_batch4.py` deliberately: four batch-4 implementers
run concurrently and a shared module is a write race in which one whole-file
write silently drops another's function. Scaffolding (registry, `target`,
`main`) is copied from `gen_batch4.py`; the shared *helpers*
(`case_emit`, `math_shapes`, `lexer`) are imported, never edited.

Targets:
  math_floor_trunc_ceil_bracketed_root   5 fns -> 20 cases, NO matrix
  math_log2_log10_bracketed_root         5 fns -> 20 cases, NO matrix
  math_fully_bracketed_root_core_suite   9 fns ->  6 cases x ext(4) = 24
  math_global_this_root_core_suite       9 fns ->  6 cases x ext(4) = 24
  math_imul_clz32_aliases                4 fns ->  2 cases x ext(2) =  4

Run: python3 gen_batch4_group_d.py [name ...]   (no args = all)
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

from case_emit import fixture_in_fn, fixture_starting, emit, write  # noqa: E402
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


def _fn_body(text, fn_name):
    """Source text of `fn <fn_name>`'s body, brace-matched. Content-anchored so
    it survives line shifts (the failure mode `fixture_in_fn`'s docstring
    describes)."""
    import re
    marker = re.search(r"\bfn\s+" + re.escape(fn_name) + r"\s*[(<]", text)
    if not marker:
        raise AssertionError(f"no `fn {fn_name}` in source")
    brace = text.find("{", marker.end() - 1)
    depth, i, n = 0, brace, len(text)
    while i < n:
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    return text[brace:i + 1]


def loop_tuples(text, fn_name, per_tuple):
    """The literal tuples of an INLINE `for (...) in [ ... ] {` loop.

    `case_emit.fixture_starting` cannot reach these: four of this batch's files
    put their harness programs in a literal tuple list inside the `#[test]` fn
    itself, and the four `main.<ext>` bodies are byte-identical, so any content
    prefix matches 4 times and `fixture_starting` (correctly) refuses an
    ambiguous match. Anchoring on the loop's own `for (` ... `] {` delimiters
    keeps the extraction content-anchored while allowing duplicates.

    `per_tuple` is the number of STRING LITERALS per tuple, which is not always
    the tuple's arity: in the `*_core_suite` files the program slot is a
    `*_source()` CALL, not a literal, so a 4-arity tuple yields 3 literals.

    Returns list[tuple[str, ...]] of length `per_tuple`.
    """
    body = _fn_body(text, fn_name)
    start = body.index("for (")
    end = body.index("] {", start)
    lits = [x["value"] for x in find_string_literals(body[start:end])]
    if len(lits) % per_tuple:
        raise AssertionError(
            f"`fn {fn_name}`: {len(lits)} literals is not a multiple of {per_tuple}")
    out = [tuple(lits[i:i + per_tuple]) for i in range(0, len(lits), per_tuple)]
    for t in out:
        if t[0] not in ("run", "test"):
            raise AssertionError(f"tuple {t[0]!r} is not a command -- wrong anchor")
        if not t[1].startswith(("main.", "smoke.test.")):
            raise AssertionError(f"tuple {t[1]!r} is not a source filename -- wrong anchor")
    return out


# --------------------------------------------------------------------------
# Prose shared by this group's five headers. Every backticked identifier below
# is grepped against the real source's fn list before shipping (U8).
# --------------------------------------------------------------------------

def no_rust_comments(name):
    return f"""\
RULE 12 (carry every source comment verbatim): `browser_{name}.rs` has NO Rust
comments. Verified with `grep -nE '//' browser_{name}.rs`, whose single hit is
line 11's `// kali-tree-shake:` marker -- that sits INSIDE the `r##"..."##` JS
bundle fixture, so it is program text carried verbatim into [source] below,
not Rust prose. There is therefore nothing to move into any `rationale`, and
comment_coverage.py is run with --allow-empty for this pair."""


RULE13_NOTE = """\
RULE 13 (transitive helper docs): every fn in each call chain was checked for a
`///` doc comment and none carries one -- the file's only `//` is the JS
tree-shake marker inside a fixture (see the rule 12 note above). The chain does
reach `kali_runtime_contract::browser_bundle_harness_script` and
`::browser_harness_command_parts_for`, which carry one-line `///` docs, but in
the migrated form this case file never calls them: the `browser_bundle_harness`
step kind means the case RUNNER does (crates/kali_case_runner/src/steps.rs), so
those docs describe shared runner infrastructure (design spec 5.3), not what
this case claims. Every browser/ case file shipped before batch 4 does the
same."""


NO_MATRIX_NOTE = """\
NO [matrix] (rule 7 / U1). [matrix] is FILE-WIDE -- `expand()` fans every
[[case]] by the full cross product of every axis with no per-case opt-out
(crates/kali_case_runner/src/expand.rs) -- so an `ext` axis here would fan the
four bundle cases over jsx/tsx, manufacturing four build invocations the source
never runs. That is both a rule 7 arithmetic failure (cases x axis product
would not equal the real invocation count) and a rule 2 invention. The axis is
therefore dropped for the WHOLE file and every real invocation is written as
its own named sibling [[case]]. Same disposition, same reason, as the
already-shipped `math_exp_log_bracketed_root.toml`."""


MIRROR_COUNT = (
    "The source spells this as `.matches({needle})` `.count() >= {n}`, carried "
    "directly as `{key}` with `at_least = {n}`. It is NOT weakened to a "
    "`*_contains` (a single occurrence would satisfy that) and NOT strengthened "
    "to `exact` (the source states a lower bound, not an equality), per "
    "controller ruling 3 -- mirror the source."
)


# ==========================================================================
# 1. browser_math_floor_trunc_ceil_bracketed_root.rs -- 5 fns, 20 invocations
# ==========================================================================
@target("math_floor_trunc_ceil_bracketed_root")
def floor_trunc_ceil():
    name = "math_floor_trunc_ceil_bracketed_root"
    text = rs(name)
    bundle_src = fixture_in_fn(
        text, "browser_bundle_bracketed_global_this_math_floor_trunc_ceil_source")
    harness_body = fixture_starting(
        text, "assert_browser_bundle_bracketed_global_this_math_floor_trunc_ceil",
        "const mod = await import(")
    tuples = loop_tuples(
        text,
        "run_and_test_supports_bracketed_global_this_math_floor_trunc_ceil_identity_"
        "when_browser_harness_is_configured_in_js_and_ts_input",
        3)
    assert len(tuples) == 8, len(tuples)

    ONES, TWOS = "1\n", "2\n"
    count_stdout = [{"needle": ONES, "at_least": 6}, {"needle": TWOS, "at_least": 3}]
    count_json = [{"path": "stdout", "needle": ONES, "at_least": 6},
                  {"path": "stdout", "needle": TWOS, "at_least": 3}]

    header = f"""\
Migrated from tests/browser_math_floor_trunc_ceil_bracketed_root.rs.

{no_rust_comments(name)}

RULE 7 / U1 -- INVOCATION ARITHMETIC, and why the matrix is DECLINED.
5 #[test] fns, 20 real invocations, in two groups that do NOT cover the same
extensions:
  * `assert_browser_bundle_bracketed_global_this_math_floor_trunc_ceil(
    filename, json_output)` -- 4 unlooped #[test] fns = ext(js/ts) x
    json_output(false/true). js and ts ONLY; there is no jsx or tsx build fn.
  * `run_and_test_supports_bracketed_global_this_math_floor_trunc_ceil_identity_
    when_browser_harness_is_configured_in_js_and_ts_input` -- ONE #[test] fn
    whose body is an INLINE loop over eight literal
    `(command, source_name, source)` tuples -- ("run","main.js"),
    ("test","smoke.test.js"), then the same pair for ts, jsx and tsx -- nested
    inside `for output_json in [false, true]`: 8 x 2 = 16 invocations.
    (enumerate_invocations.py reports this fn as `UNPARSED (no helper call
    found)`: it only expands loops that call a named helper, and this loop
    builds its Command inline. It is enumerated by hand above and mechanically
    by the loop-tuple extractor in tools/task-18-browser-pilot/
    gen_batch4_group_d.py, which asserts the 8-tuple count.)
Total 4 + 16 = 20 invocations from 5 fns.

{NO_MATRIX_NOTE}

RULE 5 (split, don't fold) / rule 6 (1:1). The single looped #[test] fn writes
16 independently-created programs into 16 fresh temp dirs and asserts each
separately, so it becomes 16 sibling [[case]] entries named descriptively (by
command, extension and output mode), not one case and not numbered ones. The
four bundle fns keep their own case each. 20 [[case]] entries, 20 invocations,
1:1.

U2 -- [source] is file-wide, and that is safe here: all ten fixtures are
written unconditionally into a fresh temp dir by the source, none sits behind
an `if`, and no case's point is a file's presence or absence. Every command
names its own entry on argv, so the siblings are inert. U5 needs no renaming:
`app.js`/`app.ts` (bundle) and `main.<ext>`/`smoke.test.<ext>` (harness) are
already distinct keys.

THE COUNT CLAIMS -- this file is one of batch 4's two count-carrying targets,
and it makes TWO separate count claims, not one. Six sites in the source, all
the same pair of bounds:
  :121  `stdout.matches("1\\n").count() >= 6`   raw bundle-harness stdout
  :122  `stdout.matches("2\\n").count() >= 3`   raw bundle-harness stdout
  :236  `stdout.matches("1\\n").count() >= 6`   `json["stdout"].as_str()`, :235
  :237  `stdout.matches("2\\n").count() >= 3`   same JSON leaf
  :241  `stdout.matches("1\\n").count() >= 6`   raw stdout, the `else` branch
  :242  `stdout.matches("2\\n").count() >= 3`   raw stdout, the `else` branch
The raw-stdout sites become `stdout_count`; the two sites taken against
`json["stdout"].as_str()` become `json_count` with `path = "stdout"`, which is
exactly why the two keys are separate (design spec 5.4). BOTH needles are
carried at every site -- `1\\n` at `at_least = 6` and `2\\n` at `at_least = 3`
-- because they are two distinct source claims; carrying one would drop the
other. Neither is strengthened to `exact`: the live output is
`1\\n1\\n2\\n1\\n1\\n2\\n1\\n1\\n2\\n` (floor(1.6)=1, trunc(1.6)=1,
ceil(1.6)=2, through three bracketing spellings), so both claims sit exactly on
their `>=` boundary at 6 and 3 -- but the source states lower bounds and pinning
equality would assert something it never did (rule 2). The source makes no
`.contains` claim alongside these counts, so no `stdout_contains` is written.

{RULE13_NOTE}

ASSERTION SHAPE, mirrored from the source and nothing more.
Bundle group: `exit = "success"` on the build (:62) and on the harness process
(:114); in json mode the envelope's schemaVersion/command/success/exitCode and
payload artifactKind/bundleFormat (:71-77) -- the source asserts NO `errors`
array on this envelope, so none is written; the emitted `app/app.meta.json`
metadata (:85-86), asserted in BOTH modes because the source reads it outside
the `if json_output`; then the harness step's two `stdout_count` claims.
Harness group: `exit = "success"` (:214); json mode carries schemaVersion/
command/success/payload(hostContract, runtimeBackend) (:222-226), plus
`exitCode` and `payload.exitCode` for `run` (:228-229) or payload
total/passed/failed for `test` (:231-233), then the two `json_count` claims and
`stderr = ""` (:238); again no `errors` claim, so none is written. No
`json.stdout` equality pin is written either -- the source asserts only how
many times each needle occurs in that leaf, never what it equals.
ARGV: unlike the bundle group, this file's harness invocations DO pass
`--max-threads 0 --max-spawned-processes 0` (:206-209), so both flags appear on
every harness argv below, in source order, after `--api browser`."""

    src = {"app.js": bundle_src, "app.ts": bundle_src}
    for _command, fname, body in tuples:
        src[fname] = body

    bundle_prose = (
        "Migrated from browser_math_floor_trunc_ceil_bracketed_root.rs. "
        "`assert_browser_bundle_bracketed_global_this_math_floor_trunc_ceil` builds a browser "
        "bundle (`kali build --bundle --api browser`), asserts the emitted `app/app.meta.json` "
        "metadata, then runs the bundle glue under the browser-bundle-harness contract. The "
        "bundled program takes value = 1.6 through nine bracketed-root spellings of "
        "globalThis.Math floor/trunc/ceil, printing 1 six times and 2 three times. The source's "
        "only stdout claims here are the two counts at :121-122. " +
        MIRROR_COUNT.format(needle='"1\\n"', n=6, key="stdout_count") +
        " The same holds for the second needle at `at_least = 3`. "
        "No [matrix] in this file: the bundle group covers only js/ts while the harness group "
        "covers all four extensions, and [matrix] is file-wide with no per-case opt-out (the "
        "header carries the arithmetic)."
    )
    harness_prose = (
        "Migrated from browser_math_floor_trunc_ceil_bracketed_root.rs, from the inline loop in "
        "`run_and_test_supports_bracketed_global_this_math_floor_trunc_ceil_identity_when_"
        "browser_harness_is_configured_in_js_and_ts_input`. That one #[test] fn loops over eight "
        "literal (command, source_name, source) tuples and over "
        "`for output_json in [false, true]`, giving 16 independent invocations; per rule 5 each "
        "becomes its own named sibling rather than being folded. This case is the "
        "`{cmd}` / `{ext}` / {mode} cell. `kali {cmd} --api browser --max-threads 0 "
        "--max-spawned-processes 0` runs the program with the browser harness backed by `node`; "
        "the program prints 1 six times and 2 three times from nine bracketed-root spellings of "
        "globalThis.Math floor/trunc/ceil on 1.6. "
    )

    cases = []
    for ext in ("js", "ts"):
        cases.append({
            "name": f"build_emits_bracketed_global_this_math_floor_trunc_ceil_identity_"
                    f"literals_in_{ext}_input",
            "rationale": bundle_prose,
            "steps": bundle_steps(f"app.{ext}", harness_body,
                                  {"stdout_count": count_stdout},
                                  json_output=False, meta_fields=META),
        })
    for ext in ("js", "ts"):
        cases.append({
            "name": f"json_build_emits_bracketed_global_this_math_floor_trunc_ceil_identity_"
                    f"literals_in_{ext}_input",
            "rationale": bundle_prose + " This sibling asserts the JSON build envelope "
                         "(schemaVersion/command/success/exitCode and payload "
                         "artifactKind/bundleFormat) instead of plain text; the source makes no "
                         "`errors` claim on that envelope, so none is written. Output shape is "
                         "not a matrix axis because it changes the assertion shape.",
            "steps": bundle_steps(f"app.{ext}", harness_body,
                                  {"stdout_count": count_stdout},
                                  json_output=True,
                                  json_claims=envelope_build(errors=False),
                                  meta_fields=META),
        })

    for command, fname, _body in tuples:
        ext = fname.split(".")[-1]
        cases.append({
            "name": f"{command}_supports_bracketed_global_this_math_floor_trunc_ceil_identity_"
                    f"when_browser_harness_is_configured_in_{ext}_input",
            "rationale": harness_prose.format(cmd=command, ext=ext, mode="plain-text-output") +
                         "The source's only stdout claims on this branch are the two counts at "
                         ":241-242. " +
                         MIRROR_COUNT.format(needle='"1\\n"', n=6, key="stdout_count") +
                         " The second needle `\"2\\n\"` is carried the same way at "
                         "`at_least = 3`; both are separate source claims and both are kept.",
            "steps": [harness_step(command, fname, json_output=False, thread_flags=True,
                                   asserts={"stdout_count": count_stdout})],
        })
    for command, fname, _body in tuples:
        ext = fname.split(".")[-1]
        cases.append({
            "name": f"json_{command}_supports_bracketed_global_this_math_floor_trunc_ceil_"
                    f"identity_when_browser_harness_is_configured_in_{ext}_input",
            "rationale": harness_prose.format(cmd=command, ext=ext, mode="--output json") +
                         "On this branch the same two counts are taken against the JSON string "
                         "leaf `json[\"stdout\"]` (:235-237) rather than raw stdout, which is why "
                         "`json_count` exists alongside `stdout_count`. " +
                         MIRROR_COUNT.format(needle='"1\\n"', n=6, key="json_count") +
                         " The second needle `\"2\\n\"` is carried the same way at "
                         "`at_least = 3`. No equality pin is written for `json.stdout`: the "
                         "source asserts only how many times each needle occurs in it. `stderr` "
                         "is asserted exactly empty (:238); the source makes no `errors` claim on "
                         "this envelope, so none is written.",
            "steps": [harness_step(command, fname, json_output=True, thread_flags=True,
                                   json_claims=envelope_harness(command, stderr=True,
                                                                errors=False),
                                   asserts={"json_count": count_json})],
        })

    return ("math_floor_trunc_ceil_bracketed_root.toml", header, None, src, cases)


# ==========================================================================
# 2. browser_math_log2_log10_bracketed_root.rs -- 5 fns, 20 invocations
# ==========================================================================
@target("math_log2_log10_bracketed_root")
def log2_log10_bracketed():
    name = "math_log2_log10_bracketed_root"
    text = rs(name)
    bundle_src = fixture_in_fn(
        text, "browser_bundle_bracketed_global_this_math_log2_log10_source")
    harness_body = fixture_starting(
        text, "assert_browser_bundle_bracketed_global_this_math_log2_log10",
        "const mod = await import(")
    tuples = loop_tuples(
        text,
        "run_and_test_supports_bracketed_global_this_math_log2_log10_identities_"
        "when_browser_harness_is_configured_in_js_and_ts_input",
        4)
    assert len(tuples) == 8, len(tuples)

    NEEDLE = "3\n"
    count_stdout = [{"needle": NEEDLE, "at_least": 2}]
    # Live-captured from .cache/cargo-target/debug/kali with node as the harness
    # backend, for all eight (command, ext) cells -- identical in every one.
    JSON_STDOUT = "3\n3\n3\n3\n"

    header = f"""\
Migrated from tests/browser_math_log2_log10_bracketed_root.rs.

{no_rust_comments(name)}

RULE 7 / U1 -- INVOCATION ARITHMETIC, and why the matrix is DECLINED.
5 #[test] fns, 20 real invocations, in two groups that do NOT cover the same
extensions:
  * `assert_browser_bundle_bracketed_global_this_math_log2_log10(filename,
    json_output)` -- 4 unlooped #[test] fns = ext(js/ts) x
    json_output(false/true). js and ts ONLY; there is no jsx or tsx build fn.
  * `run_and_test_supports_bracketed_global_this_math_log2_log10_identities_
    when_browser_harness_is_configured_in_js_and_ts_input` -- ONE #[test] fn
    whose body is an INLINE loop over eight literal
    `(command, source_name, source, expected_stdout)` tuples -- ("run",
    "main.js"), ("test","smoke.test.js"), then the same pair for ts, jsx and
    tsx -- nested inside `for output_json in [false, true]`: 8 x 2 = 16.
    (enumerate_invocations.py reports this fn as `UNPARSED (no helper call
    found)` because it builds its Command inline rather than calling a named
    helper; it is enumerated by hand above and mechanically by the loop-tuple
    extractor in tools/task-18-browser-pilot/gen_batch4_group_d.py, which
    asserts the 8-tuple count.)
Total 4 + 16 = 20 invocations from 5 fns.

{NO_MATRIX_NOTE}

RULE 5 (split, don't fold) / rule 6 (1:1): the looped #[test] fn becomes 16
sibling [[case]] entries named by command, extension and output mode; the four
bundle fns keep one case each. 20 [[case]] entries, 20 invocations, 1:1.

U2 -- [source] is file-wide and that is safe here: all ten fixtures are written
unconditionally into a fresh temp dir, none sits behind an `if`, and no case's
point is a file's presence or absence. Every command names its entry on argv.
U5 needs no renaming: `app.js`/`app.ts` and `main.<ext>`/`smoke.test.<ext>` are
already distinct keys.

THE COUNT CLAIM -- this is batch 4's second count-carrying target. Sites:
  :111  `stdout.contains("3\\n")`                     raw bundle-harness stdout
  :112  `stdout.matches("3\\n").count() >= 2`         raw bundle-harness stdout
:111 and :112 are TWO SEPARATE source claims about the same needle, so BOTH are
carried on the bundle-harness step -- `stdout_contains = ["3\\n"]` and
`stdout_count = [{{ needle = "3\\n", at_least = 2 }}]`. Collapsing them into
either one alone would drop a claim. The count is mirrored exactly: not
weakened to `contains` (one occurrence satisfies that), not strengthened to
`exact` even though the live output is `3\\n3\\n3\\n3\\n` -- four occurrences,
since log2(8), frozenLog2(8), log10(1000) and frozenLog10(1000) all print 3 --
because the source states a lower bound (rule 2 / ruling 3).
The harness group makes NO count claim: its plain-text branch asserts
`stdout.contains(expected_stdout)` (:240) against the loop's own
expected_stdout literal, which is `"3\\n3"` for `run` and `"3\\nok 1"` for
`test`, carried as `stdout_contains` (both literals are pulled from the .rs,
not retyped). Its json branch asserts
`json["stdout"].as_str().contains("3")` (:233-236).

THE ONE STRENGTHENING, and it is live-verified. `json["stdout"]` is a JSON leaf
and the format has no substring form for a `json` path, so per controller
ruling 3 that `.contains("3")` becomes an exact `json.stdout` pin -- captured
from the real `.cache/cargo-target/debug/kali` with `node` as the harness
backend for all EIGHT (command, ext) cells, every one of which produced
`"3\\n3\\n3\\n3\\n"`, never hand-computed (U9). Every run satisfying the pin
satisfies the source's `.contains("3")`, so this is a strengthening, not a
change of claim. Same disposition as the shipped sibling
`math_exp_log_bracketed_root.toml`. (`json_count` with `at_least = 1` was
considered as a closer mirror now that the count keys exist; the exact pin was
kept because ruling 3 names it for json leaves and because the sibling file in
this same family already ships that shape.)

{RULE13_NOTE}

ASSERTION SHAPE, mirrored and nothing more.
Bundle group: `exit = "success"` on the build (:52) and on the harness process
(:104); json mode carries schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat (:61-67) -- no `errors` claim in the source, so none
is written; `app/app.meta.json` metadata (:75-76) in BOTH modes, since the
source reads it outside the `if json_output`; then the harness step's
`stdout_contains` + `stdout_count`.
Harness group: `exit = "success"` (:212); json mode carries schemaVersion/
command/success/payload(hostContract, runtimeBackend) (:220-224), `exitCode`
and `payload.exitCode` for `run` (:226-227) or payload total/passed/failed for
`test` (:229-231), the exact `stdout` pin above, and `stderr = ""` (:237); no
`errors` claim, so none is written.
ARGV: the harness invocations pass `--max-threads 0 --max-spawned-processes 0`
(:204-207); the bundle build does not. Mirrored per group."""

    src = {"app.js": bundle_src, "app.ts": bundle_src}
    for _command, fname, body, _expected in tuples:
        src[fname] = body

    bundle_prose = (
        "Migrated from browser_math_log2_log10_bracketed_root.rs. "
        "`assert_browser_bundle_bracketed_global_this_math_log2_log10` builds a browser bundle "
        "(`kali build --bundle --api browser`), asserts the emitted `app/app.meta.json` "
        "metadata, then runs the bundle glue under the browser-bundle-harness contract. The "
        "bundled program's four calls -- globalThis.Math.log2(8), a frozen alias of it, "
        "globalThis[\"Math\"].log10(1000) and a frozen alias of that -- each print 3. The source "
        "makes TWO separate claims about that output, `stdout.contains(\"3\\n\")` (:111) and "
        "`stdout.matches(\"3\\n\").count() >= 2` (:112), so both are carried, as "
        "`stdout_contains` and `stdout_count` respectively; carrying either alone would drop a "
        "claim. " + MIRROR_COUNT.format(needle='"3\\n"', n=2, key="stdout_count") +
        " The real output holds four occurrences, but pinning `exact = 4` would assert "
        "something the source never claimed. No [matrix] in this file: the bundle group covers "
        "only js/ts while the harness group covers all four extensions, and [matrix] is "
        "file-wide with no per-case opt-out (the header carries the arithmetic)."
    )
    harness_prose = (
        "Migrated from browser_math_log2_log10_bracketed_root.rs, from the inline loop in "
        "`run_and_test_supports_bracketed_global_this_math_log2_log10_identities_when_browser_"
        "harness_is_configured_in_js_and_ts_input`. That one #[test] fn loops over eight literal "
        "(command, source_name, source, expected_stdout) tuples and over "
        "`for output_json in [false, true]`, giving 16 independent invocations; per rule 5 each "
        "becomes its own named sibling. This case is the `{cmd}` / `{ext}` / {mode} cell. "
        "`kali {cmd} --api browser --max-threads 0 --max-spawned-processes 0` runs the program "
        "with the browser harness backed by `node`; log2(8), a frozen alias of log2, "
        "log10(1000) and a frozen alias of log10 each print 3. "
    )

    cases = []
    for ext in ("js", "ts"):
        cases.append({
            "name": f"build_emits_bracketed_global_this_math_log2_log10_identity_literals_"
                    f"in_{ext}_input",
            "rationale": bundle_prose,
            "steps": bundle_steps(f"app.{ext}", harness_body,
                                  {"stdout_contains": [NEEDLE],
                                   "stdout_count": count_stdout},
                                  json_output=False, meta_fields=META),
        })
    for ext in ("js", "ts"):
        cases.append({
            "name": f"json_build_emits_bracketed_global_this_math_log2_log10_identity_literals_"
                    f"in_{ext}_input",
            "rationale": bundle_prose + " This sibling asserts the JSON build envelope "
                         "(schemaVersion/command/success/exitCode and payload "
                         "artifactKind/bundleFormat) instead of plain text; the source makes no "
                         "`errors` claim on that envelope, so none is written.",
            "steps": bundle_steps(f"app.{ext}", harness_body,
                                  {"stdout_contains": [NEEDLE],
                                   "stdout_count": count_stdout},
                                  json_output=True,
                                  json_claims=envelope_build(errors=False),
                                  meta_fields=META),
        })

    for command, fname, _body, expected in tuples:
        ext = fname.split(".")[-1]
        cases.append({
            "name": f"{command}_supports_bracketed_global_this_math_log2_log10_identities_"
                    f"when_browser_harness_is_configured_in_{ext}_input",
            "rationale": harness_prose.format(cmd=command, ext=ext, mode="plain-text-output") +
                         "The source's only stdout claim on this branch is "
                         "`stdout.contains(expected_stdout)` (:240) against this tuple's own "
                         f"expected_stdout literal, carried verbatim as `stdout_contains`. It "
                         "is a plain `.contains` against a field that has a substring form, so "
                         "per controller ruling 3 it stays a substring claim and is NOT "
                         "strengthened to an exact `stdout` pin.",
            "steps": [harness_step(command, fname, json_output=False, thread_flags=True,
                                   asserts={"stdout_contains": [expected]})],
        })
    for command, fname, _body, _expected in tuples:
        ext = fname.split(".")[-1]
        env = envelope_harness(command, stderr=False, errors=False)
        env["stdout"] = JSON_STDOUT
        env["stderr"] = ""
        cases.append({
            "name": f"json_{command}_supports_bracketed_global_this_math_log2_log10_identities_"
                    f"when_browser_harness_is_configured_in_{ext}_input",
            "rationale": harness_prose.format(cmd=command, ext=ext, mode="--output json") +
                         "On this branch the source asserts "
                         "`json[\"stdout\"].as_str().contains(\"3\")` (:233-236). `json` paths "
                         "have no substring form, so per controller ruling 3 that becomes an "
                         "exact `json.stdout` pin -- live-captured from the real kali binary for "
                         "this exact cell (U9), never hand-computed, and checked to contain the "
                         "source's own `\"3\"` literal before being written. Every run "
                         "satisfying the pin satisfies the source's `.contains`, so this is a "
                         "verified strengthening. `stderr` is asserted exactly empty (:237); the "
                         "source makes no `errors` claim on this envelope, so none is written.",
            "steps": [harness_step(command, fname, json_output=True, thread_flags=True,
                                   json_claims=env, asserts={})],
        })

    return ("math_log2_log10_bracketed_root.toml", header, None, src, cases)


# ==========================================================================
# 3 & 4. The two *_core_suite files -- identical shape, 9 fns / 24 invocations,
#        matrix arithmetic closes exactly.
# ==========================================================================

def _core_suite(name, *, toml, bundle_src_fn, assert_fn, loop_fn, spelling,
                run_src, test_src, per_tuple):
    text = rs(name)
    bundle_src = fixture_in_fn(text, bundle_src_fn)
    harness_body = fixture_starting(text, assert_fn, "const mod = await import(")
    tuples = loop_tuples(text, loop_fn, per_tuple)
    assert len(tuples) == 8, len(tuples)
    exts = ["js", "ts", "jsx", "tsx"]
    assert [t[1].split(".")[-1] for t in tuples] == [e for e in exts for _ in (0, 1)], tuples
    expected = {t[3 if per_tuple == 4 else 2] for t in tuples}
    assert len(expected) == 1, expected
    expected = expected.pop()
    # Live-captured from .cache/cargo-target/debug/kali with node as the harness
    # backend, for all eight (command, ext) cells -- identical in every one.
    JSON_STDOUT = "3\n1\n3\n-1\n-2\n31\n"
    bundle_contains = ["3\n", "1\n", "-2\n", "31\n", "-1\n"]

    header = f"""\
Migrated from tests/browser_{name}.rs.

{no_rust_comments(name)}

RULE 7 / U1 -- MATRIX ARITHMETIC, and it CLOSES EXACTLY.
9 #[test] fns, 24 real invocations:
  * `{assert_fn}(filename, json_output)`
    -- 8 unlooped #[test] fns = ext(js/ts/jsx/tsx) x json_output(false/true),
    a full cross product.
  * `{loop_fn}`
    -- ONE #[test] fn whose body is an INLINE loop over eight literal
    `(command, source_name, source, expected_stdout)` tuples -- ("run",
    "main.js"), ("test","smoke.test.js"), then the same pair for ts, jsx and
    tsx -- nested inside `for output_json in [false, true]`: 8 x 2 = 16
    invocations. (enumerate_invocations.py reports this fn as `UNPARSED (no
    helper call found)` because it builds its Command inline rather than
    calling a named helper; it is enumerated by hand above and mechanically by
    the loop-tuple extractor in tools/task-18-browser-pilot/
    gen_batch4_group_d.py, which asserts both the 8-tuple count and that the
    extensions run js, js, ts, ts, jsx, jsx, tsx, tsx.)
FACTUAL NOTE ON THE LOOPED FN'S NAME: it ends `_in_js_and_ts_input`, but its
body loops over jsx and tsx as well. The name is stale, the body is what runs,
and the body is what is migrated. This matters because it is exactly what makes
the matrix legal here -- unlike the sibling files
`math_floor_trunc_ceil_bracketed_root.toml` and
`math_log2_log10_bracketed_root.toml`,
whose bundle groups really do cover js/ts only and which therefore decline the
matrix. (Not a rule 12 / U7 item: a fn NAME is not a comment, and this file has
no Rust comments at all. Recorded here so the difference between these files is
not read as an inconsistency.)
`ext` is the one axis BOTH groups vary over uniformly and completely, so:
6 [[case]] x ext(4) = 24 trials = 24 invocations = 9 #[test] fns' worth of
work. Exact, with nothing invented and nothing duplicated.
Per rule 6 the fold is stated here: each [[case]] below corresponds to 4 real
invocations, one per `ext` cell, and the assertion mapping stays 1:1 per trial.
`command` and `json_output` are NOT axes -- each changes the assertion SHAPE
rather than a substituted string (JSON envelope vs text stdout; `exitCode` for
`run` vs `total`/`passed`/`failed` for `test`), which design spec 5.6 excludes
from a matrix -- so they are sibling [[case]] entries instead.

U2 -- [source] is file-wide and that is safe here: `app.${{ext}}`,
`main.${{ext}}` and `smoke.test.${{ext}}` are all written unconditionally into
a fresh temp dir, none sits behind an `if`, and no case's point is a file's
presence or absence. Every command names its entry on argv, so the two unused
siblings in a trial dir are inert. U5 needs no renaming: the three keys are
already distinct.

ASSERTION SHAPE, mirrored from the source and nothing more. Every stdout claim
in this file is a PLAIN `.contains` -- there is no `.matches().count()`
anywhere, so no `stdout_count`/`json_count` appears, and no `*_contains` is
strengthened to an exact pin (ruling 3: a `.contains` against a field that has
a substring form stays a substring claim).
Bundle group: `exit = "success"` on the build and on the harness process; json
mode carries schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat -- the source asserts NO `errors` array, so none is
written; the emitted `app/app.meta.json` metadata in BOTH modes, since the
source reads it outside the `if json_output`; then the bundle-harness
`stdout_contains` of the source's five separate needles, in source order:
"3\\n", "1\\n", "-2\\n", "31\\n", "-1\\n" (max=3, clz32(1)=31, min=1,
abs(-3)=3, sign(-3)=-1, imul(2147483647,2)=-2).
Harness group: `exit = "success"`; the plain-text branch asserts
`stdout.contains(expected_stdout)` against the loop's own literal
`"{expected}"` (pulled from the .rs, not retyped), carried as
`stdout_contains`; the json branch carries schemaVersion/command/success/
payload(hostContract, runtimeBackend), `exitCode` and `payload.exitCode` for
`run` or payload total/passed/failed for `test`, `stderr = ""`, and an exact
`json.stdout` pin.
THE ONE STRENGTHENING, live-verified: the json branch's claim is
`json["stdout"].as_str().contains(expected_stdout)`. A `json` path has no
substring form, so per controller ruling 3 it becomes an exact `json.stdout`
pin -- captured from the real `.cache/cargo-target/debug/kali` with `node` as
the harness backend for all EIGHT (command, ext) cells, every one of which
produced the identical `"3\\n1\\n3\\n-1\\n-2\\n31\\n"` (U9, never
hand-computed). That the value is identical across all four extensions is also
what lets the pin sit inside a matrix-fanned case at all. Every run satisfying
the pin satisfies the source's `.contains`, so this is a strengthening, not a
change of claim; the same disposition as the shipped
`math_exp_log_bracketed_root.toml`.
ARGV: the harness invocations pass `--max-threads 0 --max-spawned-processes 0`;
the bundle build does not. Mirrored per group.

{RULE13_NOTE}"""

    bundle_prose = (
        f"Migrated from browser_{name}.rs. `{assert_fn}` builds a browser bundle "
        "(`kali build --bundle --api browser`), asserts the emitted `app/app.meta.json` "
        "metadata, then runs the bundle glue under the browser-bundle-harness contract and "
        f"checks the {spelling} Math core-suite output: max(1,2,3)=3, min(3,2,1)=1, "
        "abs(3-6)=3, sign(3-6)=-1, imul(2147483647,2)=-2, clz32(1)=31. The source makes five "
        "separate plain `.contains` claims about that output, carried in source order as "
        "`stdout_contains`; they are substring claims against a field that has a substring "
        "form, so per controller ruling 3 they are not strengthened to an exact `stdout` pin."
    )
    harness_prose = (
        f"Migrated from browser_{name}.rs, from the inline loop in `{loop_fn}`. That one "
        "#[test] fn loops over eight literal (command, source_name, source, expected_stdout) "
        "tuples and over `for output_json in [false, true]`, giving 16 independent invocations; "
        "the eight extension cells fold into this file's `ext` matrix axis (see the header's "
        "arithmetic) and the command/output-mode cells become sibling cases per rule 7. This "
        f"case is the `{{cmd}}` / {{mode}} cell. `kali {{cmd}} --api browser --max-threads 0 "
        "--max-spawned-processes 0` runs the program with the browser harness backed by `node`; "
        f"its six {spelling} Math calls print 3, 1, 3, -1, -2 and 31. "
    )

    cases = [
        {"name": f"build_emits_{toml}",
         "rationale": bundle_prose,
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": bundle_contains},
                               json_output=False, meta_fields=META)},
        {"name": f"json_build_emits_{toml}",
         "rationale": bundle_prose + " This sibling asserts the JSON build envelope "
                      "(schemaVersion/command/success/exitCode and payload "
                      "artifactKind/bundleFormat) instead of plain text; the source makes no "
                      "`errors` claim on that envelope, so none is written.",
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": bundle_contains},
                               json_output=True,
                               json_claims=envelope_build(errors=False),
                               meta_fields=META)},
    ]
    for command, fname in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        cases.append({
            "name": f"{command}_supports_{toml}_when_browser_harness_is_configured",
            "rationale": harness_prose.format(cmd=command, mode="plain-text-output") +
                         "The source's only stdout claim on this branch is "
                         "`stdout.contains(expected_stdout)` against the loop's own literal, "
                         "carried verbatim as `stdout_contains` and deliberately not "
                         "strengthened to an exact `stdout` pin (ruling 3).",
            "steps": [harness_step(command, fname, json_output=False, thread_flags=True,
                                   asserts={"stdout_contains": [expected]})],
        })
    for command, fname in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        env = envelope_harness(command, stderr=False, errors=False)
        env["stdout"] = JSON_STDOUT
        env["stderr"] = ""
        cases.append({
            "name": f"json_{command}_supports_{toml}_when_browser_harness_is_configured",
            "rationale": harness_prose.format(cmd=command, mode="--output json") +
                         "On this branch the source asserts "
                         "`json[\"stdout\"].as_str().contains(expected_stdout)`. A `json` path "
                         "has no substring form, so per controller ruling 3 that becomes an "
                         "exact `json.stdout` pin -- live-captured from the real kali binary for "
                         "every extension cell (U9), never hand-computed, and checked to contain "
                         "the source's own expected literal before being written. `stderr` is "
                         "asserted exactly empty; the source makes no `errors` claim on this "
                         "envelope, so none is written.",
            "steps": [harness_step(command, fname, json_output=True, thread_flags=True,
                                   json_claims=env, asserts={})],
        })

    source = {"app.${ext}": bundle_src,
              "main.${ext}": run_src(text),
              "smoke.test.${ext}": test_src(text, tuples)}
    return (f"{name}.toml", header, {"ext": exts}, source, cases)


@target("math_fully_bracketed_root_core_suite")
def fully_bracketed_core_suite():
    return _core_suite(
        "math_fully_bracketed_root_core_suite",
        toml="fully_bracketed_global_this_math_core_suite",
        bundle_src_fn="browser_bundle_fully_bracketed_global_this_math_core_suite_source",
        assert_fn="assert_browser_bundle_fully_bracketed_global_this_math_core_suite",
        loop_fn="run_and_test_supports_fully_bracketed_global_this_math_core_suite_"
                "when_browser_harness_is_configured_in_js_and_ts_input",
        spelling="fully bracketed globalThis[\"Math\"][...]",
        # This file's harness programs live in named *_source() fns which the
        # loop CALLS, so the tuples carry 3 literals, not 4.
        per_tuple=3,
        run_src=lambda t: fixture_in_fn(
            t, "browser_harness_fully_bracketed_global_this_math_core_suite_run_source"),
        test_src=lambda t, _tuples: fixture_in_fn(
            t, "browser_harness_fully_bracketed_global_this_math_core_suite_test_source"),
    )


@target("math_global_this_root_core_suite")
def global_this_core_suite():
    return _core_suite(
        "math_global_this_root_core_suite",
        toml="global_this_math_core_suite",
        bundle_src_fn="browser_bundle_global_this_math_core_suite_source",
        assert_fn="assert_browser_bundle_global_this_math_core_suite",
        loop_fn="run_and_test_supports_global_this_math_core_suite_"
                "when_browser_harness_is_configured_in_js_and_ts_input",
        spelling="dotted globalThis.Math",
        # This file inlines its harness programs as literals in the tuples, so
        # each tuple carries 4 literals and the fixtures come from the tuples.
        per_tuple=4,
        run_src=lambda t: _uniq(t, "run"),
        test_src=lambda t, _tuples: _uniq(t, "test"),
    )


def _uniq(text, command):
    """The single distinct program literal the global-this loop writes for
    `command`, asserted identical across all four extensions (which is what
    lets one `main.${ext}` / `smoke.test.${ext}` [source] entry stand for four
    invocations under the matrix)."""
    tuples = loop_tuples(
        text,
        "run_and_test_supports_global_this_math_core_suite_"
        "when_browser_harness_is_configured_in_js_and_ts_input",
        4)
    bodies = {t[2] for t in tuples if t[0] == command}
    if len(bodies) != 1:
        raise AssertionError(f"{command}: {len(bodies)} distinct program bodies, wanted 1")
    return bodies.pop()


# ==========================================================================
# 5. browser_math_imul_clz32_aliases.rs -- 4 fns, 4 invocations
# ==========================================================================
@target("math_imul_clz32_aliases")
def imul_clz32_aliases():
    name = "math_imul_clz32_aliases"
    text = rs(name)
    bundle_src = fixture_in_fn(text, "browser_bundle_math_imul_clz32_aliases_source")
    harness_body = fixture_starting(
        text, "assert_browser_bundle_math_imul_clz32_aliases", "const mod = await import(")

    header = f"""\
Migrated from tests/browser_{name}.rs.

{no_rust_comments(name)}

RULE 7 / U1 -- MATRIX ARITHMETIC, and it CLOSES EXACTLY.
4 #[test] fns, 4 real invocations, ONE helper, NO loops anywhere in the file
(verified with enumerate_invocations.py, which parses every fn in this file and
reports TOTAL INVOCATIONS: 4 with no UNPARSED entries):
  * `assert_browser_bundle_math_imul_clz32_aliases(filename, json_output)`
    -- 4 unlooped #[test] fns = ext(js/ts) x json_output(false/true), a full
    cross product.
This file has NO harness group at all -- there is no `run`/`test` #[test] fn
and no second helper -- so the js/ts coverage is uniform across the whole file
by construction and the axis fans nothing untested:
2 [[case]] x ext(js, ts) = 4 trials = 4 #[test] fns. Exact.
The axis is `ext = ["js", "ts"]` and NOT the four-value axis several siblings
in this batch use: this source has no `app.jsx` or `app.tsx` #[test] fn, and
adding those cells would invent two untested build invocations (rule 2).
Per rule 6 the fold is stated here: each [[case]] corresponds to 2 source
#[test] fns, one per `ext` cell, and the assertion mapping stays 1:1 per trial.
`json_output` is NOT an axis -- it changes the assertion SHAPE (a JSON envelope
claim instead of no stdout claim on the build step), which design spec 5.6
excludes from a matrix -- so it is a sibling [[case]].

U2 -- [source] is file-wide and that is safe here: `app.${{ext}}` is the file's
only fixture, written unconditionally into a fresh temp dir by every one of the
four #[test] fns, never behind an `if`, and no case's point is a file's
presence or absence.

ASSERTION SHAPE, mirrored from the source and nothing more.
`exit = "success"` on the build (:83) and on the harness process (:135); json
mode carries the envelope's schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat (:92-98) -- the source asserts NO `errors` array, so
none is written; the emitted `app/app.meta.json` metadata (:106-107) is
asserted in BOTH modes, because the source reads it outside the
`if json_output`.
THE TWO HARNESS CLAIMS ARE PLAIN `.contains`, NOT COUNTS: `stdout.contains
("-2\\n")` (:142) and `stdout.contains("31\\n")` (:143). There is no
`.matches().count()` anywhere in this file, so no `stdout_count`/`json_count`
appears -- and neither claim is strengthened to an exact `stdout` pin, because
`stdout` has a substring form and ruling 3 keeps a plain `.contains` as a
substring claim. The bundled program prints imul(2147483647, 2) = -2 nine times
through nine alias spellings and clz32(1) = 31 nine times through nine more, so
an exact pin would also have asserted a repetition count the source never
mentions.
ARGV: the source passes no `--max-threads` / `--max-spawned-processes` (it has
no `run`/`test` invocation to pass them to), so neither appears below.

{RULE13_NOTE}"""

    prose = (
        f"Migrated from browser_{name}.rs. `assert_browser_bundle_math_imul_clz32_aliases` "
        "builds a browser bundle (`kali build --bundle --api browser`), asserts the emitted "
        "`app/app.meta.json` metadata, then runs the bundle glue under the "
        "browser-bundle-harness contract. The bundled program reaches Math.imul and Math.clz32 "
        "through eighteen alias spellings -- dotted, bracketed, mixed, bare `Math[...]`, and "
        "`Object.freeze`d captures of each -- printing -2 for imul(2147483647, 2) and 31 for "
        "clz32(1). The source's two stdout claims are plain `.contains` (:142-143), mirrored as "
        "`stdout_contains` and deliberately neither turned into a count (the source makes no "
        "`.matches().count()` claim) nor strengthened to an exact `stdout` pin (ruling 3)."
    )

    cases = [
        {"name": "build_emits_math_imul_clz32_aliases",
         "rationale": prose,
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": ["-2\n", "31\n"]},
                               json_output=False, meta_fields=META)},
        {"name": "json_build_emits_math_imul_clz32_aliases",
         "rationale": prose + " This sibling asserts the JSON build envelope "
                      "(schemaVersion/command/success/exitCode and payload "
                      "artifactKind/bundleFormat) instead of plain text; the source makes no "
                      "`errors` claim on that envelope, so none is written. Output shape is not "
                      "a matrix axis because it changes the assertion shape.",
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": ["-2\n", "31\n"]},
                               json_output=True,
                               json_claims=envelope_build(errors=False),
                               meta_fields=META)},
    ]

    return (f"{name}.toml", header, {"ext": ["js", "ts"]},
            {"app.${ext}": bundle_src}, cases)


def main(argv):
    names = argv or sorted(REGISTRY)
    for name in names:
        if name not in REGISTRY:
            raise SystemExit(f"unknown target {name!r}; known: {sorted(REGISTRY)}")
        out, header, matrix, source, cases = REGISTRY[name]()
        write(os.path.join(CASES, out), emit(header.split("\n"), matrix, source, cases))


if __name__ == "__main__":
    main(sys.argv[1:])
