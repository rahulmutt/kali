#!/usr/bin/env python3
"""Generate the batch 4 GROUP A case files (5 targets).

Kept in its own module rather than appended to `gen_batch4.py`: four batch-4
implementers run concurrently and a shared file is a write race in which one
whole-file write silently drops another agent's function. The scaffolding
(registry, `main`, the shared `case_emit`/`math_shapes` imports) is copied
from `gen_batch4.py`; nothing here edits a shared module.

Targets, with the invocation arithmetic each one closes on:

  math_hypot_empty_identity            24 fns / 24 invocations -> [matrix] ext
  math_imul_omitted_operands           24 fns / 24 invocations -> [matrix] ext
  math_expm1_log1p_global_this_root    22 fns / 22 invocations -> NO matrix
  math_expm1_log1p_frozen_aliases      17 fns / 20 invocations -> NO matrix
  math_hypot_frozen_aliases            14 fns / 32 invocations -> NO matrix

Every fixture is pulled from the .rs by fn name through `case_emit`
(`fixture_in_fn` / `fixture_starting`), never by line range and never retyped
(rule 9). Every exact `json.stdout` pin below was live-captured from the real
`kali` binary at .cache/cargo-target/debug/kali with `node` as the browser
harness backend (U9); the capture command is recorded next to each pin.

Run: python3 gen_batch4_group_a.py [name ...]   (no args = all)
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

from case_emit import fixture_in_fn, fixture_starting, emit, write  # noqa: E402
from math_shapes import (  # noqa: E402
    rule12_no_comments_prose,  # noqa: E402
    bundle_steps, harness_step, envelope_build, envelope_harness, META,
)

REGISTRY = {}
EXTS = ["js", "ts", "jsx", "tsx"]
HARNESS_ENV = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"
# ^ the value of `kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV`, read from
# crates/kali_runtime_contract/src/browser/contract.rs:84 rather than assumed:
# three of these five sources pass the constant and two spell the literal, and
# they must resolve to the same env var for the migrated `env` to be faithful.


def target(name):
    def deco(fn):
        REGISTRY[name] = fn
        return fn
    return deco


def rs(name):
    return open(os.path.join(TESTS, f"browser_{name}.rs")).read()


def check_program(label, body, *, must_contain="console.log"):
    """Guard the wrong-literal-extraction class of bug at generation time.

    A fixture pulled from the wrong place still produces a parseable case file
    (batch 4 shipped `"app.${ext}" = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"`
    once). Anything this module writes into `[source]` or a harness `body` must
    look like the program it claims to be before it is emitted.
    """
    if must_contain not in body:
        raise AssertionError(f"fixture {label!r} does not look like a program: {body[:80]!r}")
    return body


def harness_json(command, *, stdout_pin=None, stderr=False, errors=False):
    """`envelope_harness` plus an exact `json.stdout` pin, in envelope order.

    `math_shapes.envelope_harness` takes explicit `stderr=`/`errors=` flags
    because these files genuinely differ there; it has no `stdout` parameter
    because most of the batch asserts a COUNT on that leaf rather than an
    equality. All four json-mode files in this group instead make a plain
    `.contains` claim against `json["stdout"]`, which per controller ruling 3
    becomes an exact pin (the leaf has no substring form) -- so the pin is
    spliced in here rather than by changing the shared builder.
    """
    base = envelope_harness(command, stderr=stderr, errors=errors)
    if stdout_pin is None:
        return base
    out = {}
    for key, value in base.items():
        if key in ("stderr", "errors") and "stdout" not in out:
            out["stdout"] = stdout_pin
        out[key] = value
    out.setdefault("stdout", stdout_pin)
    return out


# --------------------------------------------------------------------------
# Shared prose. Every backticked fn-shaped token written into a header or a
# rationale below is checked against the real source fn list by
# check_rationale_fn_names.py (U8) before shipping; nothing here is asserted
# from memory.
# --------------------------------------------------------------------------

def no_rust_comments(name):
    return f"""\
RULE 12 (carry every source comment): `browser_{name}.rs` contains NO Rust
comments. Checked with `grep -nE '//'`: the single hit is the
`// kali-tree-shake:` marker on the first line of the bundle fixture's
`r##"..."##` body, which is program text under test and is carried verbatim
into [source] below, not Rust prose. There is nothing to move into a
`rationale`, and `comment_coverage.py` is therefore run with --allow-empty for
this pair (its batch-3 zero-line floor would otherwise report a vacuous
green)."""


def rule13(name):
    return f"""\
RULE 13 (transitive helper docs): `grep -c '///' browser_{name}.rs` returns 0 --
no fn in any call chain of this file carries a doc comment, so there is none to
carry. The chain does reach `kali_runtime_contract`'s
`browser_bundle_harness_script` and `browser_harness_command_parts_for`, which
carry one-line `///` docs, but the migrated case file never calls them: the
`browser_bundle_harness` step kind means the case RUNNER does, so those docs
describe shared runner infrastructure (design spec 5.3) rather than what this
case claims. Same disposition as the already-shipped `math_log2_log10.toml`
header, flagged there as a standing question rather than decided per file."""


NO_MATRIX = """\
[matrix] DECLINED for the WHOLE file (rule 7 / U1). The enumeration above is
not a uniform cross product, and `[matrix]` is file-wide: `expand()` fans
EVERY [[case]] by the full axis product with no per-case opt-out
(crates/kali_case_runner/src/expand.rs). An `ext` axis would therefore fan the
cases the source never ran at that extension, inventing untested combinations
-- a rule 7 arithmetic failure and a rule 2 violation at the same time. Every
invocation below is its own named sibling [[case]] instead."""


NO_HOIST = """\
U13 -- SHARED FIXTURE BODIES ARE NOT HOISTED INTO [constants] HERE, and that is
deliberate rather than an oversight. Without a [matrix] the same bundle/run/test
program has to be written once per entry filename, so several [source] values
below are byte-identical. U13 would normally hoist them. Two gates say
otherwise for program text specifically:
  * `check_fixtures.py` (the rule 9 gate) searches only [source] values and
    step `body` fields. A hoisted body would live in [constants] and the
    reference would be a `${...}` placeholder, so the gate that exists to catch
    a corrupted fixture would go red on a correct file -- and, worse, could be
    silenced by relaxing it.
  * U13 records the counter-hazard itself: [constants] IS a surface
    `audit-case-migration.py` searches for assertion strings, so hoisting
    program text moves a whole JS program onto the surface a claim can be
    satisfied from, weakening the rule 3 gate for the entire file.
The identity of the duplicated bodies is asserted mechanically rather than
eyeballed: this file is generated, every duplicate is the SAME Python string
object returned by ONE extraction call in
tools/task-18-browser-pilot/gen_batch4_group_a.py, and the generator asserts
that object identity for every [source] value before emitting. Matches the shipped browser/ precedent for a declined matrix
(`math_atan2_global_this_root.toml`, `for_of_array_iteration_alias_chain.toml`)."""


U2_NOTE = """\
U2 -- [source] is file-wide, and that is safe here. Every fixture is written
unconditionally into a fresh temp dir by the source, none sits behind an `if`,
and no case's point is a file's presence or absence. Each command names its
entry explicitly on argv, so the sibling fixtures in a trial dir are inert;
verified against the real binary that a `test` run in a directory containing
all four `smoke.test.<ext>` files still reports total/passed/failed = 1/1/0 for
the one it was given, and that a `build --bundle` of `app.<ext>` in a directory
containing all four `app.*` still emits `app/app.meta.json`."""


PIN_NOTE = """\
The `json` branch's stdout claim is `json["stdout"].as_str().contains(...)`.
That leaf has no substring form: design spec 5.4's twelve keys include no
JSON-substring key at all (only exact `json` paths, `json_null` and
`json_count`). So per controller ruling 3 -- a plain `.contains` against a
field with NO substring form becomes an exact pin -- it is written as an exact
`json.stdout` pin, and per U9 the pinned value was captured by running the real
`kali` binary with `node` as the browser harness backend, never hand-computed."""


# ==========================================================================
# 1. browser_math_hypot_empty_identity.rs -- 24 fns, 24 invocations, matrix.
# ==========================================================================
@target("math_hypot_empty_identity")
def hypot_empty_identity():
    name = "math_hypot_empty_identity"
    text = rs(name)
    bundle_src = check_program(
        "bundle", fixture_in_fn(text, "browser_bundle_global_this_math_hypot_empty_identity_source"))
    run_src = check_program(
        "run", fixture_in_fn(text, "browser_harness_global_this_math_hypot_empty_identity_run_source"))
    test_src = check_program(
        "test", fixture_in_fn(text, "browser_harness_global_this_math_hypot_empty_identity_test_source"))
    harness_body = check_program(
        "harness body",
        fixture_starting(text, "assert_browser_bundle_global_this_math_hypot_empty_identity",
                         "const mod = await import("),
        must_contain="await import(")

    needle = "0\n0\n0\n0\n0\n"
    # Live-captured, all four extensions and both commands, with:
    #   kali --output json <run|test> --api browser --max-threads 0
    #        --max-spawned-processes 0 <entry>
    #   KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node
    # -> json["stdout"] == "0\n0\n0\n0\n0\n" for all 8 combinations, so one pin
    # is valid across the whole `ext` axis.
    pin = "0\n0\n0\n0\n0\n"

    header = f"""\
Migrated from tests/browser_math_hypot_empty_identity.rs.

{no_rust_comments(name)}

RULE 7 / U1 -- MATRIX ARITHMETIC, and it closes exactly. Enumerated with
tools/task-18-browser-pilot/enumerate_invocations.py: 24 #[test] fns, 24 real
invocations, no loops anywhere in the file (every #[test] fn is a single
unlooped helper call, verified by reading both helper bodies -- neither
contains a `for`).
  * `assert_browser_bundle_global_this_math_hypot_empty_identity(filename,
    json_output)` -- 8 = ext(js/ts/jsx/tsx) x json_output(false/true), a full
    cross product.
  * `assert_browser_harness_global_this_math_hypot_empty_identity(command,
    filename, source, json_output)` -- 16 = command(run/test) x ext(4) x
    json_output(false/true), also a full cross product.
`ext` is the one axis both helpers vary over uniformly and completely:
6 [[case]] x ext(4) = 24 trials = 24 #[test] fns. Exact. Per rule 6 the fold is
stated here -- each [[case]] corresponds to 4 source fns, one per cell, and the
assertion mapping stays 1:1 per trial. `command` and `json_output` are NOT
axes: each changes the assertion SHAPE (JSON envelope vs text stdout;
`exitCode` for run vs `total`/`passed`/`failed` for test), which design spec
5.6's closing note excludes from a matrix.

{U2_NOTE}

{rule13(name)}

ASSERTION SHAPE, mirrored from the source and nothing more.
Bundle helper: `exit = "success"` on the build (:65) and on the harness process
(:121); in json mode schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat (:74-80) PLUS `errors = []` (:81-84) -- this file DOES
assert the empty errors array on its build envelope, unlike
`math_imul_omitted_operands`, so it is written here and not there; the emitted
`app/app.meta.json` metadata (:92-93), read outside the `if json_output` and so
asserted in both modes; then the bundle-harness `stdout_contains` (:128).
Harness helper: `exit = "success"` (:160); non-json `stdout_contains` (:192);
json mode schemaVersion/command/success/payload(hostContract, runtimeBackend)
(:168-172), `exitCode` plus `payload.exitCode` for run (:174-175) or
payload total/passed/failed for test (:177-179), the exact `json.stdout` pin
resolving :181-187, `stderr = ""` (:188) and `errors = []` (:189) -- this file
asserts BOTH, which several siblings in this batch do not.
The source DOES pass `--max-threads 0 --max-spawned-processes 0` on the harness
argv (:151-155), so both flags appear; the build argv has neither.
The stdout claim at every site is a plain contiguous
`.contains("0\\n0\\n0\\n0\\n0\\n")` -- it is NOT a `.matches(...).count()`, so no
`stdout_count`/`json_count` key appears in this file, and the contiguous
five-line needle is carried exactly as written rather than split into five
`"0\\n"` needles (that would be a different, weaker claim about adjacency).

{PIN_NOTE} Captured value for both `run` and `test` and all four extensions:
"0\\n0\\n0\\n0\\n0\\n" -- identical in all 8 combinations, which is what makes a
single pin valid under the `ext` axis."""

    bundle_prose = (
        "Migrated from browser_math_hypot_empty_identity.rs. "
        "`assert_browser_bundle_global_this_math_hypot_empty_identity` builds a browser "
        "bundle (`kali build --bundle --api browser`), asserts the emitted "
        "`app/app.meta.json` metadata, then runs the bundle glue under the "
        "browser-bundle-harness contract. The bundled program calls `Math.hypot()` with no "
        "arguments through five spellings of the Math root (bare, dotted globalThis, and "
        "three bracketed forms); the empty-argument identity makes each print `0`, so the "
        "harness stdout carries five consecutive zero lines. The source asserts that as one "
        "contiguous `stdout.contains(\"0\\n0\\n0\\n0\\n0\\n\")` (:128), carried verbatim as a "
        "single `stdout_contains` needle -- not weakened to five separate `\"0\\n\"` needles, "
        "which would drop the adjacency the source claims, and not strengthened to an exact "
        "`stdout` pin, which the source never writes (controller ruling 3: mirror the source)."
    )
    harness_prose = (
        "Migrated from browser_math_hypot_empty_identity.rs. "
        "`assert_browser_harness_global_this_math_hypot_empty_identity` runs "
        "`kali {cmd} --api browser --max-threads 0 --max-spawned-processes 0` with the "
        "browser harness backed by `node`, against a program calling `Math.hypot()` with no "
        "arguments through five spellings of the Math root, each printing `0`. "
    )

    cases = [
        {"name": "build_emits_global_this_math_hypot_empty_identity",
         "rationale": bundle_prose,
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": [needle]},
                               json_output=False, meta_fields=META)},
        {"name": "json_build_emits_global_this_math_hypot_empty_identity",
         "rationale": bundle_prose + " This sibling asserts the JSON build envelope "
                      "(schemaVersion/command/success/exitCode, payload artifactKind and "
                      "bundleFormat, and an empty `errors` array, :74-84) instead of plain "
                      "text. Output shape is a sibling case rather than a matrix axis "
                      "because it changes the assertion shape (design spec 5.6).",
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": [needle]},
                               json_output=True,
                               json_claims=envelope_build(errors=True),
                               meta_fields=META)},
    ]
    for command, fname in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        cases.append({
            "name": f"{command}_supports_global_this_math_hypot_empty_identity_"
                    "when_browser_harness_is_configured",
            "rationale": harness_prose.format(cmd=command) +
                         "This is the non-json branch (:190-193): the only output claim is "
                         "`stdout.contains(\"0\\n0\\n0\\n0\\n0\\n\")`, carried as "
                         "`stdout_contains` at the same strength.",
            "steps": [harness_step(command, fname, json_output=False, thread_flags=True,
                                   env_var=HARNESS_ENV,
                                   asserts={"stdout_contains": [needle]})],
        })
    for command, fname in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        cases.append({
            "name": f"{command}_supports_global_this_math_hypot_empty_identity_"
                    "when_browser_harness_is_configured_in_json",
            "rationale": harness_prose.format(cmd="--output json " + command) +
                         "This is the json branch (:166-189). " + PIN_NOTE.replace("\n", " ") +
                         " The captured value is \"0\\n0\\n0\\n0\\n0\\n\", identical for both "
                         "commands and all four extensions. `stderr` is asserted exactly "
                         "empty (:188) and `errors` exactly empty (:189); both are real "
                         "source assertions in this file, unlike its expm1/log1p siblings "
                         "which assert `stderr` only.",
            "steps": [harness_step(command, fname, json_output=True, thread_flags=True,
                                   env_var=HARNESS_ENV,
                                   json_claims=harness_json(command, stdout_pin=pin,
                                                            stderr=True, errors=True),
                                   asserts={})],
        })

    assert len(cases) * len(EXTS) == 24, "rule 7: 6 cases x ext(4) must equal 24 invocations"
    return ("math_hypot_empty_identity.toml", header,
            {"ext": EXTS},
            {"app.${ext}": bundle_src, "main.${ext}": run_src,
             "smoke.test.${ext}": test_src},
            cases)


# ==========================================================================
# 2. browser_math_imul_omitted_operands.rs -- 24 fns, 24 invocations, matrix.
# ==========================================================================
@target("math_imul_omitted_operands")
def imul_omitted_operands():
    name = "math_imul_omitted_operands"
    text = rs(name)
    bundle_src = check_program(
        "bundle", fixture_in_fn(text, "browser_bundle_math_imul_omitted_operands_source"))
    run_src = check_program(
        "run", fixture_in_fn(text, "browser_harness_math_imul_omitted_operands_run_source"))
    test_src = check_program(
        "test", fixture_in_fn(text, "browser_harness_math_imul_omitted_operands_test_source"))
    harness_body = check_program(
        "harness body",
        fixture_starting(text, "assert_browser_bundle_math_imul_omitted_operands",
                         "const mod = await import("),
        must_contain="await import(")

    bundle_needle = "0\n0\n0\n0\n0\n0\n0\n"
    run_needle = "0\n0\n0\n0\n0"
    test_needle = "0\n0\n0\n0\n0\nok 1"

    # THE ARTIFACT IS RIGHT in the `contrast ... in
    # browser_math_hypot_empty_identity.rs` clause below, and this generator is
    # brought up to it -- one of only two places in the family where the
    # reconciliation runs artifact -> generator. The block used to write
    # `contrast :81-84 of ...`, a line number belonging to a DIFFERENT source.
    # `batch5_crosscheck.CITE` binds a number to the nearest backticked construct
    # within 40 characters, which here is this file's own `errors`, so the pair's
    # gate resolved a cross-file pointer against the wrong source. Batch 7 fix
    # round 1 (M8, commit 32fb3e3fab) dropped the cross-file number and stated the
    # contrast by name -- ruling 15's third answer, delete the figure. That review
    # fix never reached this generator, so regenerating reintroduced the
    # mis-binding. Verified against the `.rs` arbiter, which is what decides
    # anything that is not citation form: browser_math_imul_omitted_operands.rs
    # contains no `errors` at all (`grep -c errors` -> 0) and
    # browser_math_hypot_empty_identity.rs:81-84 does assert
    # `envelope["errors"] ... .is_empty()`, so the surviving sentence is true
    # without the integer.
    header = f"""\
Migrated from tests/browser_math_imul_omitted_operands.rs.

{no_rust_comments(name)}

RULE 7 / U1 -- MATRIX ARITHMETIC, and it closes exactly. 24 #[test] fns, 24
real invocations, no loops (both helper bodies read directly: neither contains
a `for`).
  * `assert_browser_bundle_math_imul_omitted_operands(filename, json_output)`
    -- 8 = ext(js/ts/jsx/tsx) x json_output(false/true), a full cross product.
  * `assert_browser_harness_math_imul_omitted_operands(command, filename,
    source, expected_stdout)` -- 16 fns, i.e. 16 invocations over only 8
    DISTINCT argument tuples (see the note below), still a uniform
    4-cases-per-ext fan.
6 [[case]] x ext(4) = 24 trials = 24 #[test] fns. Exact. Per rule 6 the fold is
stated here: each [[case]] corresponds to 4 source fns, one per `ext` cell, and
the assertion mapping stays 1:1 per trial.

MIGRATION NOTE -- the `json_`-prefixed harness fns do NOT request JSON output.
`assert_browser_harness_math_imul_omitted_operands` (:133-168) takes
`expected_stdout` where its siblings in this batch take `json_output`, and it
never adds `--output json`; it has no json branch at all. So
`json_run_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_js_input`
(:291) issues byte-identically the same command, with byte-identically the same
assertions, as
`run_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_js_input`
(:211) -- the eight `json_`-named fns are duplicates of the eight unprefixed
ones. They are kept as four separate [[case]] entries (fanned to 8 trials)
rather than folded away, per rule 6: the case is the only remaining trace of
the fn, and two distinct #[test] fns are not merged even when their invocations
are literally identical. No JSON envelope claim is written for them -- the
source makes none, and inventing one to justify the name would be a rule 2
violation. The name discrepancy is carried, not corrected (U7).

{U2_NOTE}

{rule13(name)}

ASSERTION SHAPE, mirrored from the source and nothing more.
Bundle helper: `exit = "success"` on the build (:71) and on the harness process
(:123); in json mode schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat (:80-86) and NOTHING ELSE -- this file makes NO
`errors` claim on the build envelope (contrast the same envelope in
`browser_math_hypot_empty_identity.rs`, which does), so `errors = []` is
deliberately absent here; the emitted `app/app.meta.json` metadata (:94-95),
read outside the `if json_output`; then the bundle-harness `stdout_contains`
(:130), a contiguous SEVEN-line needle because the bundled program routes
`Math.imul()` through seven spellings including two `Object.freeze` aliases.
Harness helper: `exit = "success"` (:159) and `stdout.contains(expected_stdout)`
(:167). `expected_stdout` is a real parameter that IS read, not a dead literal:
its value comes from the call site, `"0\\n0\\n0\\n0\\n0"` for `run` (:216) and
`"0\\n0\\n0\\n0\\n0\\nok 1"` for `test` (:236), and each is carried as that case's
`stdout_contains` needle. Note the run needle has no trailing newline and
covers only five of the seven printed zeros -- transcribed exactly as the
source spells it, neither extended to seven nor newline-terminated.
The source DOES pass `--max-threads 0 --max-spawned-processes 0` on the harness
argv (:151-154); the build argv has neither. No `.matches(...).count()` claim
exists anywhere in this file, so no `stdout_count`/`json_count` key appears."""

    bundle_prose = (
        "Migrated from browser_math_imul_omitted_operands.rs. "
        "`assert_browser_bundle_math_imul_omitted_operands` builds a browser bundle "
        "(`kali build --bundle --api browser`), asserts the emitted `app/app.meta.json` "
        "metadata, then runs the bundle glue under the browser-bundle-harness contract. The "
        "bundled program calls `Math.imul()` with both operands omitted through seven "
        "spellings -- bare, dotted globalThis, three bracketed roots, and two "
        "`Object.freeze`d callables -- and the omitted-operand identity makes each print "
        "`0`. The source asserts that as one contiguous "
        "`stdout.contains(\"0\\n0\\n0\\n0\\n0\\n0\\n0\\n\")` (:130), carried verbatim as a single "
        "`stdout_contains` needle at the same strength (controller ruling 3: mirror the "
        "source; a plain `.contains` against a field that has a substring form stays a "
        "`*_contains`)."
    )
    harness_prose = (
        "Migrated from browser_math_imul_omitted_operands.rs. "
        "`assert_browser_harness_math_imul_omitted_operands` runs "
        "`kali {cmd} --api browser --max-threads 0 --max-spawned-processes 0` with the "
        "browser harness backed by `node`, against a program calling `Math.imul()` with both "
        "operands omitted through seven spellings, each printing `0`. Its only output claim "
        "is `stdout.contains(expected_stdout)` (:167), and `expected_stdout` is supplied by "
        "the call site as {needle} -- carried as this case's `stdout_contains` needle exactly "
        "as spelled. "
    )
    dup_note = (
        "This case comes from the `json_`-prefixed source fns, which despite the name request "
        "no JSON output: the helper has no `json_output` parameter and never adds "
        "`--output json`, so these fns issue the identical command and make the identical "
        "assertion as their unprefixed counterparts. Kept as a separate case per rule 6 "
        "(a case is the only remaining trace of its fn) rather than folded; no JSON envelope "
        "claim is written, because the source makes none (rule 2)."
    )

    cases = [
        {"name": "build_emits_math_imul_omitted_operands",
         "rationale": bundle_prose,
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": [bundle_needle]},
                               json_output=False, meta_fields=META)},
        {"name": "json_build_emits_math_imul_omitted_operands",
         "rationale": bundle_prose + " This sibling asserts the JSON build envelope "
                      "(schemaVersion/command/success/exitCode and payload "
                      "artifactKind/bundleFormat, :80-86) instead of plain text. The source "
                      "makes NO `errors` claim on this envelope, so none is written.",
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": [bundle_needle]},
                               json_output=True,
                               json_claims=envelope_build(errors=False),
                               meta_fields=META)},
        {"name": "run_supports_math_imul_omitted_operands_when_browser_harness_is_configured",
         "rationale": harness_prose.format(cmd="run", needle="\"0\\n0\\n0\\n0\\n0\" (:216)") +
                      "The needle deliberately has no trailing newline and covers five of the "
                      "seven printed zeros; it is transcribed as written, not tidied.",
         "steps": [harness_step("run", "main.${ext}", json_output=False, thread_flags=True,
                                env_var=HARNESS_ENV,
                                asserts={"stdout_contains": [run_needle]})]},
        {"name": "test_supports_math_imul_omitted_operands_when_browser_harness_is_configured",
         "rationale": harness_prose.format(cmd="test", needle="\"0\\n0\\n0\\n0\\n0\\nok 1\" (:236)") +
                      "The `ok 1` tail is the harness's own TAP-style line for the single "
                      "passing `Kali.test` case, so this needle additionally pins that the "
                      "test body ran to completion.",
         "steps": [harness_step("test", "smoke.test.${ext}", json_output=False,
                                thread_flags=True, env_var=HARNESS_ENV,
                                asserts={"stdout_contains": [test_needle]})]},
        {"name": "json_run_supports_math_imul_omitted_operands_when_browser_harness_is_configured",
         "rationale": harness_prose.format(cmd="run", needle="\"0\\n0\\n0\\n0\\n0\" (:296)") +
                      dup_note,
         "steps": [harness_step("run", "main.${ext}", json_output=False, thread_flags=True,
                                env_var=HARNESS_ENV,
                                asserts={"stdout_contains": [run_needle]})]},
        {"name": "json_test_supports_math_imul_omitted_operands_when_browser_harness_is_configured",
         "rationale": harness_prose.format(cmd="test", needle="\"0\\n0\\n0\\n0\\n0\\nok 1\" (:316)") +
                      dup_note,
         "steps": [harness_step("test", "smoke.test.${ext}", json_output=False,
                                thread_flags=True, env_var=HARNESS_ENV,
                                asserts={"stdout_contains": [test_needle]})]},
    ]

    assert len(cases) * len(EXTS) == 24, "rule 7: 6 cases x ext(4) must equal 24 invocations"
    return ("math_imul_omitted_operands.toml", header,
            {"ext": EXTS},
            {"app.${ext}": bundle_src, "main.${ext}": run_src,
             "smoke.test.${ext}": test_src},
            cases)


# ==========================================================================
# 3. browser_math_expm1_log1p_global_this_root.rs -- 22 fns, 22 invocations,
#    NO matrix (the harness group is missing two cells).
# ==========================================================================
@target("math_expm1_log1p_global_this_root")
def expm1_log1p_global_this_root():
    name = "math_expm1_log1p_global_this_root"
    text = rs(name)
    bundle_src = check_program(
        "bundle", fixture_in_fn(text, "browser_bundle_global_this_math_expm1_log1p_source"))
    run_src = check_program(
        "run", fixture_in_fn(text, "browser_harness_global_this_math_expm1_log1p_run_source"))
    test_src = check_program(
        "test", fixture_in_fn(text, "browser_harness_global_this_math_expm1_log1p_test_source"))
    harness_body = check_program(
        "harness body",
        fixture_starting(text, "assert_browser_bundle_global_this_math_expm1_log1p",
                         "const mod = await import("),
        must_contain="await import(")

    needle = "0\n"
    pin = "0\n0\n"   # live-captured, run and test, all four extensions

    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '0\\n0\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
Migrated from tests/browser_math_expm1_log1p_global_this_root.rs.

{no_rust_comments(name)}

RULE 7 / U1 -- INVOCATION ARITHMETIC, and it does NOT close on any axis.
22 #[test] fns, 22 real invocations, no loops (both helper bodies read
directly; neither contains a `for`).
  * `assert_browser_bundle_global_this_math_expm1_log1p(filename, json_output)`
    -- 8 invocations = ext(js/ts/jsx/tsx) x json_output(false/true). Complete.
  * `assert_browser_harness_global_this_math_expm1_log1p(command, filename,
    source, json_output)` -- only 14, not the 16 a full
    command(2) x ext(4) x json_output(2) product would give. The two absent
    cells are ("run", "main.ts", true) and ("test", "smoke.test.ts", true):
    the source has `run_supports_..._in_json_js_input` (:311),
    `..._in_json_jsx_input` (:322) and `..._in_json_tsx_input` (:333) but no
    `_in_json_ts_input`, and likewise `test_supports_..._in_json_js_input`
    (:344), `..._in_json_jsx_input` (:355), `..._in_json_tsx_input` (:366) but
    no `_in_json_ts_input`. Verified against the real fn list, not assumed
    from the naming pattern.
A file-wide `ext` axis would fan the two json harness cases to `ts` as well,
manufacturing exactly those two untested combinations.

{NO_MATRIX}
So: 22 named sibling [[case]] entries below, one per real invocation, in source
fn order, each named after the #[test] fn it replaces (rule 6, 1:1).

{NO_HOIST}

{U2_NOTE}

{rule13(name)}

ASSERTION SHAPE, mirrored from the source and nothing more.
Bundle helper: `exit = "success"` on the build (:55) and on the harness process
(:107); in json mode schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat (:64-70) and NO `errors` claim -- this file does not
assert the errors array on either envelope, so `errors = []` appears nowhere
below; the emitted `app/app.meta.json` metadata (:78-79), read outside the
`if json_output`; then the bundle-harness `stdout_contains = ["0\\n"]` (:114).
Harness helper: `exit = "success"` (:146); non-json `stdout_contains = ["0\\n"]`
(:178); json mode schemaVersion/command/success/payload(hostContract,
runtimeBackend) (:155-159), `exitCode` plus `payload.exitCode` for run
(:161-162) or payload total/passed/failed for test (:164-166), the exact
`json.stdout` pin resolving :168-174, and `stderr = ""` (:175). There is NO
`errors` assertion on the harness envelope either (contrast
`browser_math_hypot_empty_identity.rs:189`), so none is written.
The source DOES pass `--max-threads 0 --max-spawned-processes 0` on the harness
argv (:138-141); the build argv has neither. The harness env var is spelled as
a string literal here (:130) rather than through
`kali_runtime_contract`'s constant, but resolves to the same
KALI_BROWSER_BUNDLE_HARNESS_COMMAND. No `.matches(...).count()` claim exists in
this file, so no `stdout_count`/`json_count` key appears.

{PIN_NOTE} Captured value for both commands and all four extensions: "0\\n0\\n"
(globalThis.Math.expm1(0) and globalThis.Math.log1p(0) each print 0)."""

    bundle_prose = (
        "Migrated from browser_math_expm1_log1p_global_this_root.rs. "
        "`assert_browser_bundle_global_this_math_expm1_log1p` builds a browser bundle "
        "(`kali build --bundle --api browser`), asserts the emitted `app/app.meta.json` "
        "metadata, then runs the bundle glue under the browser-bundle-harness contract. The "
        "bundled program reaches expm1/log1p through a dotted `globalThis.Math` root and "
        "both identities at zero print `0`, which the source asserts as "
        "`stdout.contains(\"0\\n\")` (:114) -- carried as `stdout_contains` at the same "
        "strength, neither weakened nor pinned exactly (controller ruling 3). "
        "[matrix] is declined file-wide, so this is one case per (ext, output shape) "
        "invocation; see the file header for the arithmetic."
    )
    harness_prose = (
        "Migrated from browser_math_expm1_log1p_global_this_root.rs. "
        "`assert_browser_harness_global_this_math_expm1_log1p` runs "
        "`kali {cmd} --api browser --max-threads 0 --max-spawned-processes 0` with the "
        "browser harness backed by `node`, against a program reaching expm1/log1p through a "
        "dotted `globalThis.Math` root; both identities at zero print `0`. "
    )
    json_note = (
        "This is the json branch (:153-175). " + PIN_NOTE.replace("\n", " ") +
        " The captured value is \"0\\n0\\n\", identical for both commands and every extension "
        "the source exercises. `stderr` is asserted exactly empty (:175); the source makes "
        "NO `errors` claim on this envelope, so none is written (rule 2). Note the source "
        "has no `_in_json_ts_input` fn for either command -- that gap is preserved by "
        "declining the matrix rather than filled in."
    )
    text_note = (
        "This is the non-json branch (:176-179): the only output claim is "
        "`stdout.contains(\"0\\n\")`, carried as `stdout_contains` at the same strength."
    )

    cases = []
    for ext in EXTS:
        cases.append({
            "name": f"build_emits_global_this_math_expm1_log1p_identity_literals_in_{ext}_input",
            "rationale": bundle_prose,
            "steps": bundle_steps(f"app.{ext}", harness_body,
                                  {"stdout_contains": [needle]},
                                  json_output=False, meta_fields=META)})
    for ext in EXTS:
        cases.append({
            "name": f"json_build_emits_global_this_math_expm1_log1p_identity_literals_in_{ext}_input",
            "rationale": bundle_prose + " This sibling asserts the JSON build envelope "
                         "(schemaVersion/command/success/exitCode and payload "
                         "artifactKind/bundleFormat, :64-70) instead of plain text; the "
                         "source makes no `errors` claim on it, so none is written.",
            "steps": bundle_steps(f"app.{ext}", harness_body,
                                  {"stdout_contains": [needle]},
                                  json_output=True,
                                  json_claims=envelope_build(errors=False),
                                  meta_fields=META)})
    # Source fn order: run non-json ts/js/jsx/tsx, then test non-json ts/js/jsx/tsx.
    for command, stem in (("run", "main"), ("test", "smoke.test")):
        for ext in ("ts", "js", "jsx", "tsx"):
            cases.append({
                "name": f"{command}_supports_global_this_math_expm1_log1p_identity_literals_"
                        f"when_browser_harness_is_configured_in_{ext}_input",
                "rationale": harness_prose.format(cmd=command) + text_note,
                "steps": [harness_step(command, f"{stem}.{ext}", json_output=False,
                                       thread_flags=True, env_var=HARNESS_ENV,
                                       asserts={"stdout_contains": [needle]})]})
    # json branch: js/jsx/tsx only -- `ts` is absent from the source for both commands.
    for command, stem in (("run", "main"), ("test", "smoke.test")):
        for ext in ("js", "jsx", "tsx"):
            cases.append({
                "name": f"{command}_supports_global_this_math_expm1_log1p_identity_literals_"
                        f"when_browser_harness_is_configured_in_json_{ext}_input",
                "rationale": harness_prose.format(cmd="--output json " + command) + json_note,
                "steps": [harness_step(command, f"{stem}.{ext}", json_output=True,
                                       thread_flags=True, env_var=HARNESS_ENV,
                                       json_claims=harness_json(command, stdout_pin=pin,
                                                                stderr=True, errors=False),
                                       asserts={})]})

    assert len(cases) == 22, f"rule 7: expected 22 named siblings, built {len(cases)}"
    source = {}
    for ext in EXTS:
        source[f"app.{ext}"] = bundle_src
    for ext in EXTS:
        source[f"main.{ext}"] = run_src
    for ext in EXTS:
        source[f"smoke.test.{ext}"] = test_src
    assert_shared_identity(source, [bundle_src, run_src, test_src])
    return ("math_expm1_log1p_global_this_root.toml", header, None, source, cases)


# ==========================================================================
# 4. browser_math_expm1_log1p_frozen_aliases.rs -- 17 fns, 20 invocations,
#    NO matrix.
# ==========================================================================
@target("math_expm1_log1p_frozen_aliases")
def expm1_log1p_frozen_aliases():
    name = "math_expm1_log1p_frozen_aliases"
    text = rs(name)
    bundle_src = check_program(
        "bundle", fixture_in_fn(text, "browser_bundle_frozen_math_expm1_log1p_source"))
    run_src = check_program(
        "run", fixture_in_fn(text, "browser_harness_frozen_math_expm1_log1p_run_source"))
    test_src = check_program(
        "test", fixture_in_fn(text, "browser_harness_frozen_math_expm1_log1p_test_source"))
    harness_body = check_program(
        "harness body",
        fixture_starting(text, "assert_browser_bundle_frozen_math_expm1_log1p",
                         "const mod = await import("),
        must_contain="await import(")

    needle = "0\n"
    pin = "0\n0\n0\n0\n0\n0\n"   # live-captured, run and test, all four extensions

    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '0\\n0\\n0\\n0\\n0\\n0\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
Migrated from tests/browser_math_expm1_log1p_frozen_aliases.rs.

{no_rust_comments(name)}

RULE 7 / U1 -- INVOCATION ARITHMETIC. 17 #[test] fns, 20 real invocations.
The gap is one looping fn, and the count needs care:
`build_emits_frozen_math_expm1_log1p_identity_literals_in_jsx_and_tsx_input`
(:224-229) loops `for filename in ["app.jsx", "app.tsx"]` and makes TWO helper
calls inside the loop body -- `json_output` false at :226 and true at :227 --
so it is 4 invocations, not 2. (HISTORICAL NOTE, fix round 1 / I1: this used
to say enumerate_invocations.py undercounts this file, printing 6 bundle
invocations where there are 8 and TOTAL 18 where there are 20. That was true
when written; the parser bug -- taking only the first `assert_*` call in a loop
body -- was found during this batch and fixed in the same commit. The repaired
tool now reports TOTAL INVOCATIONS: 20, agreeing with the hand count.) The
numbers below were derived by reading the loop body and are now also confirmed
by the tool.
  * `assert_browser_bundle_frozen_math_expm1_log1p(filename, json_output)` --
    4 unlooped fns (:204, :209, :214, :219, covering app.js/app.ts x
    false/true) + 4 from the loop = 8, a full ext(4) x json_output(2) product.
  * `assert_browser_harness_frozen_math_expm1_log1p(command, filename, source,
    json_output)` -- 12 invocations from 12 unlooped fns, NOT the 16 a full
    product would give. The four absent cells are the non-json jsx and tsx
    runs of both commands: the source has `run_supports_..._in_js_input`
    (:232) and `..._in_ts_input` (:243) at `json_output = false`, but its jsx
    and tsx harness fns (`json_run_supports_..._in_jsx_input` :320,
    `..._in_tsx_input` :331, `json_test_supports_..._in_jsx_input` :342,
    `..._in_tsx_input` :353) exist only at `json_output = true`.
17 fns -> 20 invocations -> 20 [[case]] entries. 4 + 4 + 12 = 20.
The bundle group alone would close on an `ext` axis; the harness group does
not, and an axis is file-wide.

{NO_MATRIX}
The four siblings split out of the looping fn are named for the cell each one
runs (rule 5: name descriptively, never numerically); the other 16 cases keep
their source #[test] fn's name verbatim (rule 6, 1:1).

{NO_HOIST}

{U2_NOTE}

{rule13(name)}

ASSERTION SHAPE, mirrored from the source and nothing more.
Bundle helper: `exit = "success"` on the build (:76) and on the harness process
(:128); in json mode schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat (:85-91) and NO `errors` claim; the emitted
`app/app.meta.json` metadata (:99-100), read outside the `if json_output`;
then the bundle-harness `stdout_contains = ["0\\n"]` (:135).
Harness helper: `exit = "success"` (:167); non-json `stdout_contains = ["0\\n"]`
(:199); json mode schemaVersion/command/success/payload(hostContract,
runtimeBackend) (:176-180), `exitCode` plus `payload.exitCode` for run
(:182-183) or payload total/passed/failed for test (:185-187), the exact
`json.stdout` pin resolving :189-195, and `stderr = ""` (:196). No `errors`
assertion on either envelope anywhere in this file, so `errors = []` appears
nowhere below.
The source DOES pass `--max-threads 0 --max-spawned-processes 0` on the harness
argv (:159-162); the build argv has neither. The env var is spelled as a
literal (:151) and resolves to KALI_BROWSER_BUNDLE_HARNESS_COMMAND. No
`.matches(...).count()` claim exists in this file.

{PIN_NOTE} Captured value for both commands and all four extensions:
"0\\n0\\n0\\n0\\n0\\n0\\n" -- six zeros, because the program takes expm1 and log1p
through three separately frozen Math roots."""

    bundle_prose = (
        "Migrated from browser_math_expm1_log1p_frozen_aliases.rs. "
        "`assert_browser_bundle_frozen_math_expm1_log1p` builds a browser bundle "
        "(`kali build --bundle --api browser`), asserts the emitted `app/app.meta.json` "
        "metadata, then runs the bundle glue under the browser-bundle-harness contract. The "
        "bundled program freezes three spellings of the Math root with `Object.freeze` -- a "
        "dotted `globalThis.Math`, a bracketed `globalThis[\"Math\"]` and a bare `Math` -- and "
        "calls expm1/log1p at zero through each, so six zero lines are printed. The source "
        "asserts only `stdout.contains(\"0\\n\")` (:135), carried as `stdout_contains` at that "
        "strength; it is NOT strengthened to the six-line output that actually appears, "
        "because the source never claims it (controller ruling 3)."
    )
    loop_note = (
        " This case is one of the four rule-5 siblings split out of the single #[test] fn "
        "`build_emits_frozen_math_expm1_log1p_identity_literals_in_jsx_and_tsx_input` "
        "(:224-229), which loops over [\"app.jsx\", \"app.tsx\"] and calls the helper twice per "
        "iteration (json_output false then true) -- four independent invocations, four "
        "siblings, no folding."
    )
    harness_prose = (
        "Migrated from browser_math_expm1_log1p_frozen_aliases.rs. "
        "`assert_browser_harness_frozen_math_expm1_log1p` runs "
        "`kali {cmd} --api browser --max-threads 0 --max-spawned-processes 0` with the "
        "browser harness backed by `node`, against a program that freezes three spellings of "
        "the Math root and calls expm1/log1p at zero through each. "
    )
    json_note = (
        "This is the json branch (:174-196). " + PIN_NOTE.replace("\n", " ") +
        " The captured value is \"0\\n0\\n0\\n0\\n0\\n0\\n\". `stderr` is asserted exactly empty "
        "(:196); the source makes NO `errors` claim on this envelope, so none is written "
        "(rule 2)."
    )
    text_note = (
        "This is the non-json branch (:197-200): the only output claim is "
        "`stdout.contains(\"0\\n\")`, carried as `stdout_contains` at the same strength."
    )

    def bundle_case(case_name, ext, json_output, extra=""):
        return {
            "name": case_name,
            "rationale": bundle_prose + extra +
                         (" This sibling asserts the JSON build envelope "
                          "(schemaVersion/command/success/exitCode and payload "
                          "artifactKind/bundleFormat, :85-91) instead of plain text; the "
                          "source makes no `errors` claim on it, so none is written."
                          if json_output else ""),
            "steps": bundle_steps(f"app.{ext}", harness_body,
                                  {"stdout_contains": [needle]},
                                  json_output=json_output,
                                  json_claims=envelope_build(errors=False) if json_output else None,
                                  meta_fields=META)}

    base = "frozen_math_expm1_log1p_identity_literals"
    cases = [
        bundle_case(f"build_emits_{base}_in_js_input", "js", False),
        bundle_case(f"build_emits_{base}_in_ts_input", "ts", False),
        bundle_case(f"json_build_emits_{base}_in_js_input", "js", True),
        bundle_case(f"json_build_emits_{base}_in_ts_input", "ts", True),
        bundle_case(f"build_emits_{base}_in_jsx_input", "jsx", False, loop_note),
        bundle_case(f"json_build_emits_{base}_in_jsx_input", "jsx", True, loop_note),
        bundle_case(f"build_emits_{base}_in_tsx_input", "tsx", False, loop_note),
        bundle_case(f"json_build_emits_{base}_in_tsx_input", "tsx", True, loop_note),
    ]

    def harness_case(case_name, command, entry, json_output):
        step = harness_step(
            command, entry, json_output=json_output, thread_flags=True, env_var=HARNESS_ENV,
            json_claims=harness_json(command, stdout_pin=pin, stderr=True,
                                     errors=False) if json_output else None,
            asserts={} if json_output else {"stdout_contains": [needle]})
        return {"name": case_name,
                "rationale": harness_prose.format(
                    cmd=("--output json " if json_output else "") + command) +
                    (json_note if json_output else text_note),
                "steps": [step]}

    # Source fn order, :232 onward.
    cases += [
        harness_case(f"run_supports_{base}_when_browser_harness_is_configured_in_js_input",
                     "run", "main.js", False),
        harness_case(f"run_supports_{base}_when_browser_harness_is_configured_in_ts_input",
                     "run", "main.ts", False),
        harness_case(f"test_supports_{base}_when_browser_harness_is_configured_in_js_input",
                     "test", "smoke.test.js", False),
        harness_case(f"test_supports_{base}_when_browser_harness_is_configured_in_ts_input",
                     "test", "smoke.test.ts", False),
        harness_case(f"run_supports_{base}_when_browser_harness_is_configured_in_json_js_input",
                     "run", "main.js", True),
        harness_case(f"test_supports_{base}_when_browser_harness_is_configured_in_json_js_input",
                     "test", "smoke.test.js", True),
        harness_case(f"run_supports_{base}_when_browser_harness_is_configured_in_json_ts_input",
                     "run", "main.ts", True),
        harness_case(f"test_supports_{base}_when_browser_harness_is_configured_in_json_ts_input",
                     "test", "smoke.test.ts", True),
        harness_case(f"json_run_supports_{base}_when_browser_harness_is_configured_in_jsx_input",
                     "run", "main.jsx", True),
        harness_case(f"json_run_supports_{base}_when_browser_harness_is_configured_in_tsx_input",
                     "run", "main.tsx", True),
        harness_case(f"json_test_supports_{base}_when_browser_harness_is_configured_in_jsx_input",
                     "test", "smoke.test.jsx", True),
        harness_case(f"json_test_supports_{base}_when_browser_harness_is_configured_in_tsx_input",
                     "test", "smoke.test.tsx", True),
    ]

    assert len(cases) == 20, f"rule 7: expected 20 named siblings, built {len(cases)}"
    source = {}
    for ext in EXTS:
        source[f"app.{ext}"] = bundle_src
    for ext in EXTS:
        source[f"main.{ext}"] = run_src
    for ext in EXTS:
        source[f"smoke.test.{ext}"] = test_src
    assert_shared_identity(source, [bundle_src, run_src, test_src])
    return ("math_expm1_log1p_frozen_aliases.toml", header, None, source, cases)


# ==========================================================================
# 5. browser_math_hypot_frozen_aliases.rs -- 14 fns, 32 invocations, NO matrix.
#    The enumerator cannot bind this file's loops; the arithmetic below is by
#    hand, from the loop bodies.
# ==========================================================================
@target("math_hypot_frozen_aliases")
def hypot_frozen_aliases():
    name = "math_hypot_frozen_aliases"
    text = rs(name)
    bundle_src = check_program(
        "bundle", fixture_in_fn(text, "browser_bundle_global_this_math_hypot_frozen_aliases_source"))
    run_src = check_program(
        "run", fixture_in_fn(text, "browser_harness_global_this_math_hypot_frozen_aliases_run_source"))
    test_src = check_program(
        "test", fixture_in_fn(text, "browser_harness_global_this_math_hypot_frozen_aliases_test_source"))
    harness_body = check_program(
        "harness body",
        fixture_starting(text, "assert_browser_bundle_global_this_math_hypot_frozen_aliases",
                         "const mod = await import("),
        must_contain="await import(")

    needle = "5\n"
    pin = "5\n" * 15   # live-captured, both commands, every entry filename used below

    header = f"""\
EXTRA-CLAIM DECLARATIONS (U14's `extra` direction, fix round 1 / I6).
check_extra_claims.py compares this file's claim strings against the
source's and fails on any that appear nowhere in the .rs. The entries
below are the deliberate exceptions; a genuinely new one will not be
on this list and will fail the gate.
EXTRA-OK: '5\\n5\\n5\\n5\\n5\\n5\\n5\\n5\\n5\\n5\\n5\\n5\\n5\\n5\\n5\\n' -- live-captured exact `json.stdout` pin; source asserts `.contains` on a JSON leaf, which has no substring form, so ruling 3 requires an exact pin captured from the real binary
EXTRA-OK: 'main_test_entry.js' -- U5-renamed [source] entry filename; passed on argv only, referenced by no fixture body (checked), so the rename cannot change the program
EXTRA-OK: 'main_test_entry.ts' -- U5-renamed [source] entry filename; passed on argv only, referenced by no fixture body (checked), so the rename cannot change the program
Migrated from tests/browser_math_hypot_frozen_aliases.rs.

{no_rust_comments(name)}

RULE 7 / U1 -- INVOCATION ARITHMETIC, DERIVED BY HAND. 14 #[test] fns, 32 real
invocations. enumerate_invocations.py cannot close this file: it reports
`('command', 'source_name', 'source', ...)` as unresolved loop variables for
the 8-tuple loop, and it only ever reads the FIRST helper call in a loop body,
so it prints 6 bundle invocations where there are 8 and a TOTAL of 30 where
there are 32. Both loop bodies were therefore read directly:
  * `assert_browser_bundle_global_this_math_hypot_frozen_aliases(filename,
    json_output)` -- 4 unlooped fns (:257 app.js/false, :262 app.ts/false,
    :267 app.js/true, :272 app.ts/true) PLUS
    `build_emits_global_this_math_hypot_frozen_alias_slice_in_jsx_and_tsx_input`
    (:277-282), which loops `for filename in ["app.jsx", "app.tsx"]` and calls
    the helper TWICE per iteration (:279 false, :280 true) = 2 x 2 = 4.
    Bundle total 4 + 4 = 8.
  * `assert_browser_harness_global_this_math_hypot_frozen_aliases(command,
    filename, source, json_output)` -- 8 unlooped fns (:285, :296, :307, :318,
    :329, :340, :351, :362) PLUS
    `run_and_test_supports_global_this_math_hypot_frozen_alias_slice_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input`
    (:373-426), a nested loop: 8 (command, source_name, source) tuples x
    `for json_output in [false, true]` = 16. Harness total 8 + 16 = 24.
  * 8 + 24 = 32 invocations, from 14 #[test] fns, mapping to 32 [[case]]
    entries below.

FOUR INVOCATIONS ARE DUPLICATED BY THE SOURCE ITSELF, and the duplicates are
kept. The loop's first and third tuples are ("run", "main.js", run_source) and
("run", "main.ts", run_source), which at both `json_output` values reproduce
exactly what the four unlooped fns at :285, :296, :329 and :340 already do. So
28 of the 32 invocations are distinct commands and 4 are repeats. Per rule 6 a
[[case]] is the only remaining trace of its #[test] fn, and two distinct fns
are not folded into one case even when their invocations are literally
identical -- so all 32 are written, and the four `loop_run_...` siblings that
duplicate a standalone fn say so in their own rationale. The loop's `test`
tuples are NOT duplicates: they use `smoke.test.<ext>` where the unlooped test
fns use `main.<ext>` (see the U5 note).

{NO_MATRIX}
The loop-split siblings are named for the cell each one runs and carry a
"loop_" name prefix so the producing fn is unambiguous (rule 5: descriptive, never
numeric); the 12 unlooped cases keep their source #[test] fn's name verbatim.

U5 -- [source] KEY DISAMBIGUATION. The source writes the TEST program to
`main.js`/`main.ts` for the four unlooped `test_supports_...` fns (:311, :323,
:355, :367) while the `run` fns write the RUN program to the same
`main.js`/`main.ts` names -- harmless there, because each invocation gets its
own private tempdir, but impossible in one flat file-wide [source] table. The
two test-command entries are therefore renamed to `main_test_entry.js` and
`main_test_entry.ts`. That is the safe direction of U5: the filename is passed
on argv and is never referenced by string from inside any fixture body (these
programs contain no dynamic import or require specifier at all), and the
rename was verified
behaviour-neutral against the real binary -- `kali test` on `main.js` and on
`main_test_entry.js` both report total/passed/failed = 1/1/0 with identical
stdout, so the non-`.test.`-suffixed character of the source's entry name is
preserved. The `run` entries keep `main.<ext>` unchanged.

{NO_HOIST}

{U2_NOTE}

{rule13(name)}

ASSERTION SHAPE, mirrored from the source and nothing more.
Bundle helper: `exit = "success"` on the build (:125) and on the harness
process (:181); in json mode schemaVersion/command/success/exitCode and payload
artifactKind/bundleFormat (:134-140) PLUS `errors = []` (:141-144) -- this file
DOES assert the empty errors array on its build envelope; the emitted
`app/app.meta.json` metadata (:152-153), read outside the `if json_output`;
then the bundle-harness `stdout_contains = ["5\\n"]` (:188).
Harness helper: `exit = "success"` (:220); non-json `stdout_contains = ["5\\n"]`
(:252); json mode schemaVersion/command/success/payload(hostContract,
runtimeBackend) (:228-232), `exitCode` plus `payload.exitCode` for run
(:234-235) or payload total/passed/failed for test (:237-239), the exact
`json.stdout` pin resolving :241-247, `stderr = ""` (:248) AND `errors = []`
(:249).
The source DOES pass `--max-threads 0 --max-spawned-processes 0` on the harness
argv (:212-215); the build argv has neither. No `.matches(...).count()` claim
exists in this file, so no `stdout_count`/`json_count` key appears.

{PIN_NOTE} Captured value for both commands, for `main.<ext>`,
`smoke.test.<ext>` and the renamed `main_test_entry.<ext>` alike: fifteen `5`
lines, one per frozen alias spelling the program calls with (3, 4)."""

    bundle_prose = (
        "Migrated from browser_math_hypot_frozen_aliases.rs. "
        "`assert_browser_bundle_global_this_math_hypot_frozen_aliases` builds a browser "
        "bundle (`kali build --bundle --api browser`), asserts the emitted "
        "`app/app.meta.json` metadata, then runs the bundle glue under the "
        "browser-bundle-harness contract. The bundled program builds fourteen "
        "`Object.freeze`d aliases of `Math.hypot` across every dotted/bracketed/"
        "single-quoted/parenthesised spelling of the root, calls each with (3, 4) alongside "
        "the unfrozen `globalThis.Math.hypot`, and each prints `5`. The source asserts only "
        "`stdout.contains(\"5\\n\")` (:188), carried as `stdout_contains` at that strength -- "
        "not strengthened to the fifteen-line output that actually appears, which the source "
        "never claims (controller ruling 3)."
    )
    bundle_json_note = (
        " This sibling asserts the JSON build envelope (schemaVersion/command/success/"
        "exitCode, payload artifactKind and bundleFormat, and an empty `errors` array, "
        ":134-144) instead of plain text."
    )
    loop_bundle_note = (
        " This case is one of the four rule-5 siblings split out of the single #[test] fn "
        "`build_emits_global_this_math_hypot_frozen_alias_slice_in_jsx_and_tsx_input` "
        "(:277-282), which loops over [\"app.jsx\", \"app.tsx\"] and calls the helper twice per "
        "iteration (json_output false at :279, true at :280) -- four independent invocations, "
        "four siblings."
    )
    harness_prose = (
        "Migrated from browser_math_hypot_frozen_aliases.rs. "
        "`assert_browser_harness_global_this_math_hypot_frozen_aliases` runs "
        "`kali {cmd} --api browser --max-threads 0 --max-spawned-processes 0` with the "
        "browser harness backed by `node`, against a program that builds fourteen "
        "`Object.freeze`d aliases of `Math.hypot` across every spelling of the root and calls "
        "each with (3, 4); every call prints `5`. "
    )
    json_note = (
        "This is the json branch (:226-249). " + PIN_NOTE.replace("\n", " ") +
        " The captured value is fifteen `5` lines. `stderr` is asserted exactly empty (:248) "
        "and `errors` exactly empty (:249); both are real source assertions here."
    )
    text_note = (
        "This is the non-json branch (:250-253): the only output claim is "
        "`stdout.contains(\"5\\n\")`, carried as `stdout_contains` at the same strength."
    )
    rename_note = (
        " U5: the source writes this TEST program to `main.js`/`main.ts` -- the same names "
        "its `run` cases use for the RUN program -- which one flat file-wide [source] table "
        "cannot represent, so the entry is renamed to `main_test_entry.<ext>`. The name is "
        "passed on argv and referenced by no fixture body, and the rename was verified "
        "behaviour-neutral against the real binary."
    )
    dup_note = (
        " This invocation is also performed by an unlooped #[test] fn earlier in the source; "
        "both are kept as separate cases because a [[case]] is the only remaining trace of "
        "its fn and rule 6 forbids folding two distinct fns into one, even when their "
        "invocations are identical."
    )

    def bundle_case(case_name, ext, json_output, extra=""):
        return {"name": case_name,
                "rationale": bundle_prose + extra + (bundle_json_note if json_output else ""),
                "steps": bundle_steps(f"app.{ext}", harness_body,
                                      {"stdout_contains": [needle]},
                                      json_output=json_output,
                                      json_claims=envelope_build(errors=True) if json_output else None,
                                      meta_fields=META)}

    def harness_case(case_name, command, entry, json_output, extra=""):
        step = harness_step(
            command, entry, json_output=json_output, thread_flags=True, env_var=HARNESS_ENV,
            json_claims=harness_json(command, stdout_pin=pin, stderr=True,
                                     errors=True) if json_output else None,
            asserts={} if json_output else {"stdout_contains": [needle]})
        return {"name": case_name,
                "rationale": harness_prose.format(
                    cmd=("--output json " if json_output else "") + command) +
                    (json_note if json_output else text_note) + extra,
                "steps": [step]}

    slice_ = "global_this_math_hypot_frozen_alias_slice"
    cases = [
        bundle_case(f"build_emits_{slice_}_in_js_input", "js", False),
        bundle_case(f"build_emits_{slice_}_in_ts_input", "ts", False),
        bundle_case(f"json_build_emits_{slice_}_in_js_input", "js", True),
        bundle_case(f"json_build_emits_{slice_}_in_ts_input", "ts", True),
        bundle_case(f"loop_build_emits_{slice_}_in_jsx_input", "jsx", False, loop_bundle_note),
        bundle_case(f"loop_json_build_emits_{slice_}_in_jsx_input", "jsx", True, loop_bundle_note),
        bundle_case(f"loop_build_emits_{slice_}_in_tsx_input", "tsx", False, loop_bundle_note),
        bundle_case(f"loop_json_build_emits_{slice_}_in_tsx_input", "tsx", True, loop_bundle_note),
    ]

    cfg = "when_browser_harness_is_configured"
    cases += [
        harness_case(f"run_supports_{slice_}_{cfg}_in_js_input", "run", "main.js", False),
        harness_case(f"run_supports_{slice_}_{cfg}_in_ts_input", "run", "main.ts", False),
        harness_case(f"test_supports_{slice_}_{cfg}_in_js_input",
                     "test", "main_test_entry.js", False, rename_note),
        harness_case(f"test_supports_{slice_}_{cfg}_in_ts_input",
                     "test", "main_test_entry.ts", False, rename_note),
        harness_case(f"run_supports_{slice_}_{cfg}_in_json_js_input", "run", "main.js", True),
        harness_case(f"run_supports_{slice_}_{cfg}_in_json_ts_input", "run", "main.ts", True),
        harness_case(f"test_supports_{slice_}_{cfg}_in_json_js_input",
                     "test", "main_test_entry.js", True, rename_note),
        harness_case(f"test_supports_{slice_}_{cfg}_in_json_ts_input",
                     "test", "main_test_entry.ts", True, rename_note),
    ]

    loop_note = (
        " This case is one of the sixteen rule-5 siblings split out of the single #[test] fn "
        "`run_and_test_supports_global_this_math_hypot_frozen_alias_slice_when_browser_harness"
        "_is_configured_in_js_ts_jsx_and_tsx_input` (:373-426), whose eight (command, "
        "source_name, source) tuples are each run at `json_output` false and true."
    )
    # The loop's tuple order is js, ts, jsx, tsx with run/test alternating; the
    # inner `for json_output in [false, true]` runs both modes per tuple.
    for ext in EXTS:
        for command, stem in (("run", "main"), ("test", "smoke.test")):
            for json_output in (False, True):
                dup = dup_note if (command == "run" and ext in ("js", "ts")) else ""
                prefix = "loop_json_" if json_output else "loop_"
                cases.append(harness_case(
                    f"{prefix}{command}_supports_{slice_}_{cfg}_in_{ext}_input",
                    command, f"{stem}.{ext}", json_output, loop_note + dup))

    assert len(cases) == 32, f"rule 7: expected 32 named siblings, built {len(cases)}"
    source = {}
    for ext in EXTS:
        source[f"app.{ext}"] = bundle_src
    for ext in EXTS:
        source[f"main.{ext}"] = run_src
    for ext in EXTS:
        source[f"smoke.test.{ext}"] = test_src
    for ext in ("js", "ts"):
        source[f"main_test_entry.{ext}"] = test_src
    assert_shared_identity(source, [bundle_src, run_src, test_src])
    return ("math_hypot_frozen_aliases.toml", header, None, source, cases)


def assert_shared_identity(source, bodies):
    """U13's 'assert the identity, don't eyeball it', for a declined hoist.

    Every [source] value must be one of the fixtures pulled from the .rs, and
    every duplicate must be the SAME object -- so the repetition below is
    provably repetition of one extracted program, not two texts that merely
    look alike.
    """
    for filename, body in source.items():
        if not any(body is b for b in bodies):
            raise AssertionError(f"[source] {filename!r} is not one of the extracted fixtures")


def main(argv):
    names = argv or sorted(REGISTRY)
    for name in names:
        if name not in REGISTRY:
            raise SystemExit(f"unknown target {name!r}; known: {sorted(REGISTRY)}")
        out, header, matrix, source, cases = REGISTRY[name]()
        write(os.path.join(CASES, out), emit(header.split("\n"), matrix, source, cases))


if __name__ == "__main__":
    main(sys.argv[1:])
