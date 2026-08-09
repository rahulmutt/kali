#!/usr/bin/env python3
"""Task 18 batch 5, GROUP C -- five bundle+harness `browser_math_*` targets.

  C1 browser_math_sin_cos_tan_frozen_root.rs        -> math_sin_cos_tan_frozen_root.toml
  C2 browser_math_sinh_cosh_tanh_global_this_root.rs-> math_sinh_cosh_tanh_global_this_root.toml
  C3 browser_math_sqrt_cbrt_bracketed_root.rs       -> math_sqrt_cbrt_bracketed_root.toml
  C4 browser_math_max_min_frozen_aliases.rs         -> math_max_min_frozen_aliases.toml
  C5 browser_math_round_bracketed_root.rs           -> math_round_bracketed_root.toml

Everything recurring is imported, never retyped:

  * fixture text comes out of the `.rs` through `case_emit.fixture_in_fn` /
    `fixture_starting` (rule 9);
  * the two helper shapes come from `math_shapes`, which takes the assertion
    set EXPLICITLY -- these five files differ in exactly the places a default
    would paper over (`errors`, `skipped`, and the thread flags);
  * every recurring sentence comes from `batch5_prose`;
  * every `:N` citation is derived by SEARCHING the source at generation time
    (`batch5_prose.cite_line`, via `cite_in_fn` below, which masks everything
    outside one fn so an anchor that occurs in two helpers is still exact);
  * every exact `json.stdout` pin is live-captured from the real `kali` binary
    for EVERY matrix cell and asserted identical before one pin is emitted
    (U9 + `batch5_prose.assert_identical`).

Run: `python3 gen_batch5_group_c.py`.
"""

import json
import os
import re
import sys
import textwrap

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

from case_emit import fixture_in_fn, fixture_starting, emit, write  # noqa: E402
from math_shapes import (                                    # noqa: E402
    bundle_steps, harness_step, envelope_build, envelope_harness, META,
)
from kali_run import run_kali                                # noqa: E402
import batch5_prose as P                                     # noqa: E402


# --------------------------------------------------------------------------
# generic helpers (mechanics only -- every decision lives in the per-file spec)
# --------------------------------------------------------------------------

WIDTH = 86


def rs(stem):
    return open(os.path.join(TESTS, f"browser_{stem}.rs")).read()


def rs_path(stem):
    return os.path.join(TESTS, f"browser_{stem}.rs")


def fn_line_span(text, fn_name):
    """1-based (first, last) line numbers of `fn <fn_name>`'s body."""
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
    return text.count("\n", 0, marker.start()) + 1, text.count("\n", 0, i) + 1


def cite_in_fn(text, fn_name, pattern, *, expect=1, label=None):
    """`batch5_prose.cite_line`, scoped to one fn's body.

    Several of these sources spell the SAME assertion in two helpers (`assert!(
    output.status.success())` three times in C1). Blanking every line outside
    the fn keeps the line numbering of the real file while making the anchor
    unambiguous, so the citation is still produced by searching -- never by
    arithmetic, never carried over.
    """
    first, last = fn_line_span(text, fn_name)
    lines = text.split("\n")
    masked = "\n".join(
        ln if first <= i + 1 <= last else "" for i, ln in enumerate(lines)
    )
    return P.cite_line(masked, pattern, label=label or f"{fn_name}:{pattern}",
                       expect=expect)


def c(n):
    return f":{n}"


def cr(a, b):
    return f":{a}-{b}"


def wrap(prose):
    """A long sentence rendered as `#` header lines."""
    return textwrap.wrap(prose, width=WIDTH)


def assert_fns_exist(text, names):
    """Rule-13's chain list is asserted against the source, not asserted about."""
    have = set(re.findall(r"\bfn\s+([a-z_][a-z0-9_]*)", text))
    missing = [n for n in names if n not in have]
    if missing:
        raise AssertionError(f"rule-13 chain names not in source: {missing}")
    return names


def assert_absent_in_fn(text, fn_name, needle):
    """Mirror-the-source in the negative direction: prove a flag really is not
    passed before writing "this source passes none"."""
    first, last = fn_line_span(text, fn_name)
    body = "\n".join(text.split("\n")[first - 1:last])
    if needle in body:
        raise AssertionError(f"`{needle}` IS present in `fn {fn_name}` -- prose would lie")


# The four-line preamble is `batch5_prose.EXTRA_CLAIM_PREAMBLE`, not a local copy:
# all four groups had defined their own, and two of them wrapped the identical
# sentences at different columns. Rebound to the shared list mid-batch.
EXTRA_DECL_HEAD = P.EXTRA_CLAIM_PREAMBLE


def expand_source(source, ext):
    return {k.replace("${ext}", ext): v for k, v in source.items()}


def capture_pin(label, cells):
    """U9. Run the real binary once per matrix cell; every cell must agree.

    `cells` is a list of (files, argv, env). Returns the single `json["stdout"]`
    value, after `assert_identical` -- if the cells disagree the matrix is wrong
    and this raises rather than emitting one of them.
    """
    values = []
    for files, argv, env in cells:
        code, out, err, _ = run_kali(files, argv, env=env)
        if code != 0:
            raise AssertionError(f"{label}: kali exited {code}\n{err.decode()}")
        envelope = json.loads(out)
        values.append(envelope["stdout"])
    return P.assert_identical(label, *values)


def with_stdout_pin(envelope, pin):
    """Put the live-captured `stdout` pin next to `stderr` in the envelope."""
    out = {}
    for k, v in envelope.items():
        if k == "stderr":
            out["stdout"] = pin
        out[k] = v
    if "stdout" not in out:
        out["stdout"] = pin
    return out


def count_keys_block(entries):
    """The `THE COUNT KEYS` header block: every count site, searched, mirrored."""
    lines = ["THE COUNT KEYS. Every `.matches(...).count()` site in the source, with the",
             "bound this file mirrors:"]
    for desc, cite, bound in entries:
        lines += textwrap.wrap(f"* {desc} ({cite}) -- {bound}", width=WIDTH,
                               initial_indent="  ", subsequent_indent="    ")
    lines += [
        "Each bound is transcribed, never re-derived: `count() >= N` becomes `at_least = N`,",
        "on the raw surface as `stdout_count` and on the JSON string leaf as `json_count`.",
        "None is strengthened to `exact`, which the source never says, and none is weakened",
        "to a plain contains, which a single occurrence would satisfy.",
    ]
    return lines


ENV_NAME = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"
ENV = {ENV_NAME: "node"}


# ==========================================================================
# C1 -- browser_math_sin_cos_tan_frozen_root
# ==========================================================================

def build_c1():
    stem = "math_sin_cos_tan_frozen_root"
    text = rs(stem)
    BUNDLE_FN = "assert_browser_bundle_frozen_math_sin_cos_tan"
    HARNESS_FN = "assert_browser_harness_frozen_math_sin_cos_tan"
    LOOP_FN = ("run_and_test_supports_frozen_math_sin_cos_tan_zero_identities_"
               "when_browser_harness_is_configured_in_js_and_ts_input")

    BUNDLE_SRC = fixture_in_fn(text, "browser_bundle_frozen_math_sin_cos_tan_source", 0)
    RUN_SRC = fixture_in_fn(text, "browser_harness_frozen_math_sin_cos_tan_run_source", 0)
    TEST_SRC = fixture_in_fn(text, "browser_harness_frozen_math_sin_cos_tan_test_source", 0)
    HARNESS_BODY = fixture_starting(text, BUNDLE_FN, "const mod = await import(")
    if not HARNESS_BODY.startswith("const mod = await import("):
        raise AssertionError(f"wrong harness body extracted: {HARNESS_BODY[:60]!r}")

    # ---- citations, all searched -----------------------------------------
    b_exit = cite_in_fn(text, BUNDLE_FN, r"output\.status\.success\(\)", expect=2)
    b_build_exit, b_harness_exit = b_exit[0], b_exit[1]
    b_env_first = cite_in_fn(text, BUNDLE_FN, r'envelope\["schemaVersion"\]')
    b_env_last = cite_in_fn(text, BUNDLE_FN, r'envelope\["errors"\]')
    b_meta_a = cite_in_fn(text, BUNDLE_FN, r'metadata\["apiSurface"\]')
    b_meta_b = cite_in_fn(text, BUNDLE_FN, r'metadata\["artifactKind"\]')
    b_contains = cite_in_fn(text, BUNDLE_FN, r'stdout\.contains\("1\\n"\)')
    b_count = cite_in_fn(text, BUNDLE_FN, r'stdout\.matches\("0\\n"\)\.count\(\) >= 4')

    h_exit = cite_in_fn(text, HARNESS_FN, r"output\.status\.success\(\)")
    h_env_first = cite_in_fn(text, HARNESS_FN, r'json\["schemaVersion"\]')
    h_env_last = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["runtimeBackend"\]')
    h_run_a = cite_in_fn(text, HARNESS_FN, r'assert_eq!\(json\["exitCode"\], 0\)')
    h_run_b = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["exitCode"\]')
    h_test_a = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["total"\]')
    h_skipped = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["skipped"\]')
    h_leaf = cite_in_fn(text, HARNESS_FN, r'let stdout = json\["stdout"\]\.as_str\(\)')
    h_json_contains = cite_in_fn(text, HARNESS_FN, r'stdout\.contains\("1\\n"\), "json')
    h_json_count = cite_in_fn(
        text, HARNESS_FN, r'stdout\.matches\("0\\n"\)\.count\(\) >= 4, "json')
    h_stderr = cite_in_fn(text, HARNESS_FN, r'json\["stderr"\]')
    h_errors = cite_in_fn(text, HARNESS_FN, r'json\["errors"\]')
    h_txt_contains = cite_in_fn(text, HARNESS_FN, r'stdout\.contains\("1\\n"\), "stdout')
    h_txt_count = cite_in_fn(
        text, HARNESS_FN, r'stdout\.matches\("0\\n"\)\.count\(\) >= 4, "stdout')
    h_threads = cite_in_fn(text, HARNESS_FN, r'\.arg\("--max-threads"\)')
    h_procs = cite_in_fn(text, HARNESS_FN, r'\.arg\("--max-spawned-processes"\)')
    h_envvar = cite_in_fn(text, HARNESS_FN, r'\.env\("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"')
    loop_line = P.cite_line(text, r"^fn " + re.escape(LOOP_FN) + r"\($")

    EXTS = ["js", "ts", "jsx", "tsx"]
    SOURCE = {"app.${ext}": BUNDLE_SRC, "main.${ext}": RUN_SRC,
              "smoke.test.${ext}": TEST_SRC}

    def targv(cmd, entry):
        return ["--output", "json", cmd, "--api", "browser",
                "--max-threads", "0", "--max-spawned-processes", "0", entry]

    pin_run = capture_pin("C1 json.stdout (run) across ext", [
        (expand_source(SOURCE, e), targv("run", f"main.{e}"), ENV) for e in EXTS])
    pin_test = capture_pin("C1 json.stdout (test) across ext", [
        (expand_source(SOURCE, e), targv("test", f"smoke.test.{e}"), ENV) for e in EXTS])

    chain = assert_fns_exist(text, [
        "kali_bin", "browser_bundle_frozen_math_sin_cos_tan_source", BUNDLE_FN,
        "browser_harness_frozen_math_sin_cos_tan_run_source",
        "browser_harness_frozen_math_sin_cos_tan_test_source", HARNESS_FN])

    header = list(EXTRA_DECL_HEAD)
    header.append(P.extra_ok(pin_run, P.EXTRA_OK_JSON_STDOUT))
    if pin_test != pin_run:
        header.append(P.extra_ok(pin_test, P.EXTRA_OK_JSON_STDOUT))
    header += [f"Migrated from tests/browser_{stem}.rs.", ""]
    header += P.rule12_no_comments_prose(rs_path(stem), stem).split("\n") + [""]
    header += P.matrix_arithmetic(
        test_fns=9, invocations=24, cases=6, axis="ext", values=EXTS,
        helpers=[
            (BUNDLE_FN, 8, "ext(js/ts/jsx/tsx) x json_output(false/true), 8 unlooped fns"),
            (HARNESS_FN, 16,
             f"ONE looping fn ({c(loop_line)}): its 8 entries x output_json(false/true)"),
        ]) + [""]
    header += P.rule6_matrix_fold("one `ext` cell of the source") + [
        "THE TWO FOLDS ARE NOT THE SAME KIND OF FOLD, and rule 6 wants that said. For the two",
        "bundle cases an `ext` cell is one of 4 sibling `#[test]` fns. For the four harness",
        "cases it is one `(command, source_name, source)` entry of the SINGLE looping `#[test]`",
        f"fn at {c(loop_line)} -- a loop iteration, not a fn. Both are 4-wide, so the arithmetic",
        "closes either way, but a failing harness trial traces back to a loop entry rather than",
        "to a fn of its own.",
        "",
    ]
    header += P.u2_source_file_wide(["app.${ext}", "main.${ext}", "smoke.test.${ext}"]) + [""]
    header += count_keys_block([
        ("the bundle harness process's raw stdout, `stdout.matches(\"0\\n\").count() >= 4`",
         c(b_count), "`stdout_count` with `at_least = 4`"),
        ("the harness helper's JSON branch, the same count taken against "
         "`json[\"stdout\"].as_str()`", c(h_json_count), "`json_count` with `at_least = 4`"),
        ("the harness helper's text branch, against raw stdout",
         c(h_txt_count), "`stdout_count` with `at_least = 4`"),
    ]) + wrap(
        f"Each count site sits beside a SEPARATE `.contains(\"1\\n\")` claim "
        f"({c(b_contains)}, {c(h_json_contains)}, {c(h_txt_contains)}); the two are different "
        "source claims and both are carried. On the JSON branch the `.contains` half becomes "
        "the exact `json.stdout` pin (ruling 3) while the count half stays a `json_count`, so "
        "the pin and the count coexist on one step -- collapsing them would drop a claim."
    ) + [""]
    header += P.rule13_header(chain) + [""]
    header += wrap(P.migration_note_stale_fn_name(
        LOOP_FN,
        "its `_in_js_and_ts_input` suffix names two extensions, but the "
        "`(command, source_name, source)` table it loops over holds eight entries covering "
        "all four -- `main.js`/`smoke.test.js`, `.ts`, `.jsx` and `.tsx` "
        f"({c(loop_line)}) -- so it really exercises js, ts, jsx and tsx.")) + [""]
    header += P.ARGV_ORDER + [""]
    header += [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"Bundle helper: `exit = \"success\"` on the build ({c(b_build_exit)}) and on the "
        f"harness process ({c(b_harness_exit)});",
        f"in json mode the envelope's schemaVersion/command/success/exitCode/payload("
        f"artifactKind,",
        f"bundleFormat) AND `errors = []` ({cr(b_env_first, b_env_last)}) -- this source DOES "
        "assert the empty",
        "errors array on the BUILD envelope, unlike its C2/C3/C4/C5 siblings in this batch, so",
        f"it is written; the emitted `app/app.meta.json` metadata ({cr(b_meta_a, b_meta_b)}), "
        "claimed in BOTH",
        "modes because the source reads it outside the `if json_output`; then the harness step's",
        "`stdout_contains` + `stdout_count`.",
        f"Harness helper: `exit = \"success\"` ({c(h_exit)}); the argv carries `--max-threads 0`",
        f"({c(h_threads)}) and `--max-spawned-processes 0` ({c(h_procs)}), and the environment "
        f"carries",
        f"KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node ({c(h_envvar)}), spelled as a string literal "
        "in this",
        "source rather than through the `kali_runtime_contract` constant.",
        f"json mode carries schemaVersion/command/success/payload(hostContract, runtimeBackend)",
        f"({cr(h_env_first, h_env_last)}), plus `exitCode` at both levels for `run` "
        f"({cr(h_run_a, h_run_b)}) or payload",
        f"total/passed/failed AND `skipped` for `test` ({cr(h_test_a, h_skipped)}) -- this "
        "source DOES assert",
        "`skipped = 0`, unlike its siblings, so it is written; then the exact `json.stdout` pin",
        f"and the `json_count` for {cr(h_leaf, h_json_count)}, `stderr = \"\"` ({c(h_stderr)}) "
        f"and `errors = []` ({c(h_errors)}).",
        f"Text mode carries the two raw-stdout claims ({c(h_txt_contains)}, "
        f"{c(h_txt_count)}) and nothing else: the",
        "source makes no stderr and no envelope claim on that branch, so none is written.",
    ]

    PROGRAM = ("a program that freezes `Math.sin`, `globalThis.Math.cos` and "
               "`globalThis[\"Math\"][\"tan\"]` (and the three bracketed/dotted spellings of the "
               "same three) and calls each at zero, so its six console.log calls print "
               "0, 1, 0, 0, 1, 0")

    bundle_prose = (
        f"Migrated from browser_{stem}.rs, the four "
        "`build_emits_frozen_math_sin_cos_tan_zero_identities_in_*_input` fns (one per "
        f"extension). `{BUNDLE_FN}` runs `kali build --bundle --api browser` on the "
        "tree-shake-marked bundle fixture, reads the emitted `app/app.meta.json` "
        f"({cr(b_meta_a, b_meta_b)}), then runs the bundle glue under the browser-bundle-harness "
        f"contract against {PROGRAM}. Both exit statuses are asserted ({c(b_build_exit)} for the "
        f"build, {c(b_harness_exit)} for the harness process). The source makes TWO separate "
        f"claims about the harness stdout -- `stdout.contains(\"1\\n\")` ({c(b_contains)}) and "
        f"`stdout.matches(\"0\\n\").count() >= 4` ({c(b_count)}) -- so both are carried, as "
        "`stdout_contains` and `stdout_count`; collapsing them into one would drop a claim. "
        + P.ruling3_substring() + " " + P.ruling3_count('"0\\n"', 4))

    bundle_json_prose = bundle_prose + (
        " This sibling additionally asserts the JSON build envelope -- schemaVersion/command/"
        "success/exitCode, payload artifactKind and bundleFormat, and an empty `errors` array "
        f"({cr(b_env_first, b_env_last)}). Output shape is not a matrix axis because it changes "
        "the assertion shape rather than substituting a string, so it is a separate case.")

    def harness_prose(cmd, json_mode):
        head = (
            f"Migrated from browser_{stem}.rs, the four `ext` cells of the single looping "
            f"`#[test]` fn `{LOOP_FN}` ({c(loop_line)}) that pass command = \"{cmd}\" with "
            f"output_json = {'true' if json_mode else 'false'}. "
            f"`{HARNESS_FN}` runs `kali {cmd} "
            f"{'--output json ' if json_mode else ''}--api browser --max-threads 0 "
            f"--max-spawned-processes 0` ({c(h_threads)}, {c(h_procs)}) with the browser harness "
            f"backed by node ({c(h_envvar)}), against {PROGRAM}. The process exit status is "
            f"asserted at {c(h_exit)}. ")
        if not json_mode:
            return head + (
                f"The text branch makes two claims about raw stdout: "
                f"`stdout.contains(\"1\\n\")` ({c(h_txt_contains)}) and "
                f"`stdout.matches(\"0\\n\").count() >= 4` ({c(h_txt_count)}). "
                + P.ruling3_substring() + " " + P.ruling3_count('"0\\n"', 4))
        env_tail = (
            f"`exitCode` at both the envelope and the payload level ({cr(h_run_a, h_run_b)})"
            if cmd == "run" else
            f"payload total/passed/failed and `skipped = 0` ({cr(h_test_a, h_skipped)})")
        return head + (
            "This sibling asserts the JSON envelope: schemaVersion/command/success, payload "
            f"hostContract/runtimeBackend ({cr(h_env_first, h_env_last)}), {env_tail}, plus "
            f"`stderr` exactly empty ({c(h_stderr)}) and an empty `errors` array "
            f"({c(h_errors)}). Its two claims about the JSON stdout leaf are "
            f"`stdout.contains(\"1\\n\")` ({c(h_json_contains)}) and "
            f"`stdout.matches(\"0\\n\").count() >= 4` ({c(h_json_count)}), taken against "
            f"`json[\"stdout\"].as_str()` ({c(h_leaf)}). " + P.ruling3_json_leaf() +
            " The count claim is NOT folded into that pin: it is a second, independent source "
            "claim, so it is carried alongside as a `json_count`. " +
            P.ruling3_count('"0\\n"', 4, key="json_count"))

    bundle_asserts = {"stdout_contains": ["1\n"],
                      "stdout_count": [{"needle": "0\n", "at_least": 4}]}
    text_asserts = {"stdout_contains": ["1\n"],
                    "stdout_count": [{"needle": "0\n", "at_least": 4}]}

    def json_harness(cmd, entry, pin):
        env = envelope_harness(cmd, stderr=True, errors=True,
                               extra_payload=None if cmd == "run" else {"skipped": 0})
        return harness_step(
            cmd, entry, json_output=True, thread_flags=True,
            json_claims=with_stdout_pin(env, pin),
            asserts={"json_count": [{"path": "stdout", "needle": "0\n", "at_least": 4}]})

    cases = [
        {"name": "build_emits_frozen_math_sin_cos_tan_zero_identities",
         "rationale": bundle_prose,
         "steps": bundle_steps("app.${ext}", HARNESS_BODY, bundle_asserts,
                               json_output=False, meta_fields=META)},
        {"name": "json_build_emits_frozen_math_sin_cos_tan_zero_identities",
         "rationale": bundle_json_prose,
         "steps": bundle_steps("app.${ext}", HARNESS_BODY, bundle_asserts,
                               json_output=True, meta_fields=META,
                               json_claims=envelope_build(errors=True))},
        {"name": "run_supports_frozen_math_sin_cos_tan_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("run", False),
         "steps": [harness_step("run", "main.${ext}", json_output=False,
                                thread_flags=True, asserts=text_asserts)]},
        {"name": "test_supports_frozen_math_sin_cos_tan_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("test", False),
         "steps": [harness_step("test", "smoke.test.${ext}", json_output=False,
                                thread_flags=True, asserts=text_asserts)]},
        {"name": "json_run_supports_frozen_math_sin_cos_tan_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("run", True),
         "steps": [json_harness("run", "main.${ext}", pin_run)]},
        {"name": "json_test_supports_frozen_math_sin_cos_tan_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("test", True),
         "steps": [json_harness("test", "smoke.test.${ext}", pin_test)]},
    ]
    return stem, header, {"ext": EXTS}, SOURCE, cases


# ==========================================================================
# C2 -- browser_math_sinh_cosh_tanh_global_this_root
# ==========================================================================

def build_c2():
    stem = "math_sinh_cosh_tanh_global_this_root"
    text = rs(stem)
    BUNDLE_FN = "assert_browser_bundle_global_this_math_sinh_cosh_tanh"
    HARNESS_FN = "assert_browser_harness_global_this_math_sinh_cosh_tanh"

    BUNDLE_SRC = fixture_in_fn(text, "browser_bundle_global_this_math_sinh_cosh_tanh_source", 0)
    RUN_SRC = fixture_in_fn(
        text, "browser_harness_global_this_math_sinh_cosh_tanh_run_source", 0)
    TEST_SRC = fixture_in_fn(
        text, "browser_harness_global_this_math_sinh_cosh_tanh_test_source", 0)
    HARNESS_BODY = fixture_starting(text, BUNDLE_FN, "const mod = await import(")
    if not HARNESS_BODY.startswith("const mod = await import("):
        raise AssertionError(f"wrong harness body extracted: {HARNESS_BODY[:60]!r}")

    b_exit = cite_in_fn(text, BUNDLE_FN, r"output\.status\.success\(\)", expect=2)
    b_build_exit, b_harness_exit = b_exit[0], b_exit[1]
    b_env_first = cite_in_fn(text, BUNDLE_FN, r'envelope\["schemaVersion"\]')
    b_env_last = cite_in_fn(text, BUNDLE_FN, r'payload\["bundleFormat"\]')
    b_meta_a = cite_in_fn(text, BUNDLE_FN, r'metadata\["apiSurface"\]')
    b_meta_b = cite_in_fn(text, BUNDLE_FN, r'metadata\["artifactKind"\]')
    b_count0 = cite_in_fn(text, BUNDLE_FN, r'stdout\.matches\("0\\n"\)\.count\(\) >= 4')
    b_count1 = cite_in_fn(text, BUNDLE_FN, r'stdout\.matches\("1\\n"\)\.count\(\) >= 2')

    h_exit = cite_in_fn(text, HARNESS_FN, r"output\.status\.success\(\)")
    h_env_first = cite_in_fn(text, HARNESS_FN, r'json\["schemaVersion"\]')
    h_env_last = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["runtimeBackend"\]')
    h_run_a = cite_in_fn(text, HARNESS_FN, r'assert_eq!\(json\["exitCode"\], 0\)')
    h_run_b = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["exitCode"\]')
    h_test_a = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["total"\]')
    h_test_b = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["failed"\]')
    h_json_0 = cite_in_fn(text, HARNESS_FN, r'^\s+\.contains\("0\\n"\),')
    h_json_1 = cite_in_fn(text, HARNESS_FN, r'^\s+\.contains\("1\\n"\),')
    h_stderr = cite_in_fn(text, HARNESS_FN, r'json\["stderr"\]')
    h_txt_0 = cite_in_fn(text, HARNESS_FN, r'stdout\.contains\("0\\n"\), "stdout')
    h_txt_1 = cite_in_fn(text, HARNESS_FN, r'stdout\.contains\("1\\n"\), "stdout')
    h_threads = cite_in_fn(text, HARNESS_FN, r'\.arg\("--max-threads"\)')
    h_procs = cite_in_fn(text, HARNESS_FN, r'\.arg\("--max-spawned-processes"\)')
    h_envvar = cite_in_fn(text, HARNESS_FN, r'BROWSER_HARNESS_COMMAND_ENV')

    EXTS = ["js", "ts", "jsx", "tsx"]
    SOURCE = {"app.${ext}": BUNDLE_SRC, "main.${ext}": RUN_SRC,
              "smoke.test.${ext}": TEST_SRC}

    def targv(cmd, entry):
        return ["--output", "json", cmd, "--api", "browser",
                "--max-threads", "0", "--max-spawned-processes", "0", entry]

    pin_run = capture_pin("C2 json.stdout (run) across ext", [
        (expand_source(SOURCE, e), targv("run", f"main.{e}"), ENV) for e in EXTS])
    pin_test = capture_pin("C2 json.stdout (test) across ext", [
        (expand_source(SOURCE, e), targv("test", f"smoke.test.{e}"), ENV) for e in EXTS])

    chain = assert_fns_exist(text, [
        "kali_bin", "browser_bundle_global_this_math_sinh_cosh_tanh_source", BUNDLE_FN,
        "browser_harness_global_this_math_sinh_cosh_tanh_run_source",
        "browser_harness_global_this_math_sinh_cosh_tanh_test_source", HARNESS_FN])

    header = list(EXTRA_DECL_HEAD)
    header.append(P.extra_ok(pin_run, P.EXTRA_OK_JSON_STDOUT))
    if pin_test != pin_run:
        header.append(P.extra_ok(pin_test, P.EXTRA_OK_JSON_STDOUT))
    header += [f"Migrated from tests/browser_{stem}.rs.", ""]
    header += P.rule12_no_comments_prose(rs_path(stem), stem).split("\n") + [""]
    header += P.matrix_arithmetic(
        test_fns=24, invocations=24, cases=6, axis="ext", values=EXTS,
        helpers=[
            (BUNDLE_FN, 8, "ext(js/ts/jsx/tsx) x json_output(false/true)"),
            (HARNESS_FN, 16, "command(run/test) x ext(4) x json_output(false/true)"),
        ]) + [""]
    header += P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell") + [
        "There is no loop anywhere in this file: every one of the 24 invocations is a single",
        "unlooped `#[test]` fn, so each fanned trial traces back to exactly one source fn.",
        "",
    ]
    header += [
        "CASE NAMING. The source spells the JSON variants with the mode INSIDE the extension",
        "suffix -- `..._configured_in_json_ts_input` -- so stripping `_in_<ext>_input` alone",
        "would collide the JSON case with its text sibling. The mode is therefore carried as a",
        "`json_` PREFIX, matching every other case file in this family and the `json_build_...`",
        "fns this same source already spells that way.",
        "",
    ]
    header += P.u2_source_file_wide(["app.${ext}", "main.${ext}", "smoke.test.${ext}"]) + [""]
    header += count_keys_block([
        ("the bundle harness process's raw stdout, `stdout.matches(\"0\\n\").count() >= 4`",
         c(b_count0), "`stdout_count` with `at_least = 4`"),
        ("the same stdout, `stdout.matches(\"1\\n\").count() >= 2`",
         c(b_count1), "a SECOND `stdout_count` entry with `at_least = 2`"),
    ]) + [
        "Both counts are made on ONE stdout by two separate assertions, so they are two entries",
        "in one `stdout_count` array. The bundle harness step makes NO `.contains` claim at all",
        "in this source -- unlike its C1 and C5 siblings -- so no `stdout_contains` is written",
        "on it (rule 2).",
        "",
    ]
    header += P.rule13_header(chain) + [""]
    header += P.ARGV_ORDER + [""]
    header += [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"Bundle helper: `exit = \"success\"` on the build ({c(b_build_exit)}) and on the "
        f"harness process ({c(b_harness_exit)});",
        "in json mode the envelope's schemaVersion/command/success/exitCode/payload("
        "artifactKind,",
        f"bundleFormat) ({cr(b_env_first, b_env_last)}) and NOTHING else -- the source makes no "
        "`errors` claim",
        "on this build envelope, so none is written; the emitted `app/app.meta.json` metadata",
        f"({cr(b_meta_a, b_meta_b)}), claimed in BOTH modes because the source reads it outside "
        "the",
        "`if json_output`; then the harness step's two `stdout_count` entries.",
        f"Harness helper: `exit = \"success\"` ({c(h_exit)}); the argv carries `--max-threads 0`",
        f"({c(h_threads)}) and `--max-spawned-processes 0` ({c(h_procs)}). The environment "
        "variable is set",
        f"through `kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` ({c(h_envvar)}) rather "
        "than a string",
        "literal; that constant's real value was read from",
        "crates/kali_runtime_contract/src/browser/contract.rs and is",
        f"\"{ENV_NAME}\", which is what the `env` table spells.",
        f"json mode carries schemaVersion/command/success/payload(hostContract, runtimeBackend)",
        f"({cr(h_env_first, h_env_last)}), plus `exitCode` at both levels for `run` "
        f"({cr(h_run_a, h_run_b)}) or payload",
        f"total/passed/failed for `test` ({cr(h_test_a, h_test_b)}) -- the source asserts NO "
        "`skipped` here,",
        "unlike its C1 sibling, so none is written; then the exact `json.stdout` pin standing "
        "for",
        f"the two `.contains` claims ({c(h_json_0)}, {c(h_json_1)}) and `stderr = \"\"` "
        f"({c(h_stderr)}). The source",
        "makes NO `errors` claim on this envelope either, so none is written.",
        f"Text mode carries the two raw-stdout claims ({c(h_txt_0)}, {c(h_txt_1)}) and nothing "
        "else.",
    ]

    PROGRAM = ("a program that calls `globalThis.Math.sinh`, `.cosh` and `.tanh` at zero in "
               "both dotted and bracketed spellings, so its six console.log calls print "
               "0, 1, 0, 0, 1, 0")

    bundle_prose = (
        f"Migrated from browser_{stem}.rs, the four "
        "`build_emits_global_this_math_sinh_cosh_tanh_zero_identities_in_*_input` fns (one per "
        f"extension). `{BUNDLE_FN}` runs `kali build --bundle --api browser` on the "
        "tree-shake-marked bundle fixture, reads the emitted `app/app.meta.json` "
        f"({cr(b_meta_a, b_meta_b)}), then runs the bundle glue under the browser-bundle-harness "
        f"contract against {PROGRAM}. Both exit statuses are asserted ({c(b_build_exit)} for the "
        f"build, {c(b_harness_exit)} for the harness process). The source's only claims about "
        f"the harness stdout are two counts -- `stdout.matches(\"0\\n\").count() >= 4` "
        f"({c(b_count0)}) and `stdout.matches(\"1\\n\").count() >= 2` ({c(b_count1)}) -- carried "
        "as two entries of one `stdout_count` array. There is no `.contains` claim on this step "
        "in the source, so none is written (rule 2: never invent a claim). "
        + P.ruling3_count('"0\\n"', 4))

    bundle_json_prose = bundle_prose + (
        " This sibling additionally asserts the JSON build envelope -- schemaVersion/command/"
        f"success/exitCode and payload artifactKind/bundleFormat ({cr(b_env_first, b_env_last)}). "
        "The source asserts no `errors` array on this envelope, so none is written. Output shape "
        "is not a matrix axis because it changes the assertion shape rather than substituting a "
        "string, so it is a separate case.")

    def harness_prose(cmd, json_mode):
        suffix = "json_" if json_mode else ""
        head = (
            f"Migrated from browser_{stem}.rs, the four `{cmd}_supports_global_this_math_sinh_"
            f"cosh_tanh_zero_identities_when_browser_harness_is_configured_in_"
            f"{'json_' if json_mode else ''}*_input` fns (one per extension). "
            f"`{HARNESS_FN}` runs `kali {cmd} "
            f"{'--output json ' if json_mode else ''}--api browser --max-threads 0 "
            f"--max-spawned-processes 0` ({c(h_threads)}, {c(h_procs)}) with the browser harness "
            f"backed by node, set through `kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` "
            f"({c(h_envvar)}), against {PROGRAM}. The process exit status is asserted at "
            f"{c(h_exit)}. ")
        del suffix
        if not json_mode:
            return head + (
                "The text branch makes two plain claims about raw stdout: "
                f"`stdout.contains(\"0\\n\")` ({c(h_txt_0)}) and `stdout.contains(\"1\\n\")` "
                f"({c(h_txt_1)}). " + P.ruling3_substring())
        env_tail = (
            f"`exitCode` at both the envelope and the payload level ({cr(h_run_a, h_run_b)})"
            if cmd == "run" else
            f"payload total/passed/failed ({cr(h_test_a, h_test_b)}); the source asserts no "
            "`skipped` on this envelope, so none is written")
        return head + (
            "This sibling asserts the JSON envelope: schemaVersion/command/success, payload "
            f"hostContract/runtimeBackend ({cr(h_env_first, h_env_last)}), {env_tail}, plus "
            f"`stderr` exactly empty ({c(h_stderr)}). The source makes no `errors` claim on this "
            "envelope, so none is written. Its two claims about the JSON stdout leaf are "
            f"`.contains(\"0\\n\")` ({c(h_json_0)}) and `.contains(\"1\\n\")` ({c(h_json_1)}). "
            + P.ruling3_json_leaf() +
            " One pin carries both, because an exact equality on that leaf implies every "
            "substring claim taken against it.")

    bundle_asserts = {"stdout_count": [{"needle": "0\n", "at_least": 4},
                                       {"needle": "1\n", "at_least": 2}]}
    text_asserts = {"stdout_contains": ["0\n", "1\n"]}

    def json_harness(cmd, entry, pin):
        env = envelope_harness(cmd, stderr=True, errors=False)
        return harness_step(cmd, entry, json_output=True, thread_flags=True,
                            json_claims=with_stdout_pin(env, pin), asserts={})

    cases = [
        {"name": "build_emits_global_this_math_sinh_cosh_tanh_zero_identities",
         "rationale": bundle_prose,
         "steps": bundle_steps("app.${ext}", HARNESS_BODY, bundle_asserts,
                               json_output=False, meta_fields=META)},
        {"name": "json_build_emits_global_this_math_sinh_cosh_tanh_zero_identities",
         "rationale": bundle_json_prose,
         "steps": bundle_steps("app.${ext}", HARNESS_BODY, bundle_asserts,
                               json_output=True, meta_fields=META,
                               json_claims=envelope_build(errors=False))},
        {"name": "run_supports_global_this_math_sinh_cosh_tanh_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("run", False),
         "steps": [harness_step("run", "main.${ext}", json_output=False,
                                thread_flags=True, asserts=text_asserts)]},
        {"name": "test_supports_global_this_math_sinh_cosh_tanh_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("test", False),
         "steps": [harness_step("test", "smoke.test.${ext}", json_output=False,
                                thread_flags=True, asserts=text_asserts)]},
        {"name": "json_run_supports_global_this_math_sinh_cosh_tanh_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("run", True),
         "steps": [json_harness("run", "main.${ext}", pin_run)]},
        {"name": "json_test_supports_global_this_math_sinh_cosh_tanh_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("test", True),
         "steps": [json_harness("test", "smoke.test.${ext}", pin_test)]},
    ]
    return stem, header, {"ext": EXTS}, SOURCE, cases


# ==========================================================================
# C3 -- browser_math_sqrt_cbrt_bracketed_root
# ==========================================================================

def build_c3():
    stem = "math_sqrt_cbrt_bracketed_root"
    text = rs(stem)
    BUNDLE_FN = "assert_browser_bundle_bracketed_math_sqrt_cbrt"
    HARNESS_FN = "assert_browser_harness_bracketed_math_sqrt_cbrt"

    BUNDLE_SRC = fixture_in_fn(text, "browser_bundle_bracketed_math_sqrt_cbrt_source", 0)
    RUN_SRC = fixture_in_fn(text, "browser_harness_bracketed_math_sqrt_cbrt_run_source", 0)
    TEST_SRC = fixture_in_fn(text, "browser_harness_bracketed_math_sqrt_cbrt_test_source", 0)
    HARNESS_BODY = fixture_starting(text, BUNDLE_FN, "const mod = await import(")
    if not HARNESS_BODY.startswith("const mod = await import("):
        raise AssertionError(f"wrong harness body extracted: {HARNESS_BODY[:60]!r}")

    b_exit = cite_in_fn(text, BUNDLE_FN, r"output\.status\.success\(\)", expect=2)
    b_build_exit, b_harness_exit = b_exit[0], b_exit[1]
    b_env_first = cite_in_fn(text, BUNDLE_FN, r'envelope\["schemaVersion"\]')
    b_env_last = cite_in_fn(text, BUNDLE_FN, r'payload\["bundleFormat"\]')
    b_meta_a = cite_in_fn(text, BUNDLE_FN, r'metadata\["apiSurface"\]')
    b_meta_b = cite_in_fn(text, BUNDLE_FN, r'metadata\["artifactKind"\]')
    b_c2 = cite_in_fn(text, BUNDLE_FN, r'stdout\.contains\("2\\n"\)')
    b_c3 = cite_in_fn(text, BUNDLE_FN, r'stdout\.contains\("-3\\n"\)')

    h_exit = cite_in_fn(text, HARNESS_FN, r"output\.status\.success\(\)")
    h_env_first = cite_in_fn(text, HARNESS_FN, r'json\["schemaVersion"\]')
    h_env_last = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["runtimeBackend"\]')
    h_run_a = cite_in_fn(text, HARNESS_FN, r'assert_eq!\(json\["exitCode"\], 0\)')
    h_run_b = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["exitCode"\]')
    h_test_a = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["total"\]')
    h_test_b = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["failed"\]')
    h_json_2 = cite_in_fn(text, HARNESS_FN, r'^\s+\.contains\("2\\n"\),')
    h_json_3 = cite_in_fn(text, HARNESS_FN, r'^\s+\.contains\("-3\\n"\),')
    h_stderr = cite_in_fn(text, HARNESS_FN, r'json\["stderr"\]')
    h_txt_2 = cite_in_fn(text, HARNESS_FN, r'stdout\.contains\("2\\n"\), "stdout')
    h_txt_3 = cite_in_fn(text, HARNESS_FN, r'stdout\.contains\("-3\\n"\), "stdout')
    h_threads = cite_in_fn(text, HARNESS_FN, r'\.arg\("--max-threads"\)')
    h_procs = cite_in_fn(text, HARNESS_FN, r'\.arg\("--max-spawned-processes"\)')
    h_envvar = cite_in_fn(text, HARNESS_FN, r'\.env\("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"')

    EXTS = ["js", "ts", "jsx", "tsx"]
    SOURCE = {"app.${ext}": BUNDLE_SRC, "main.${ext}": RUN_SRC,
              "smoke.test.${ext}": TEST_SRC}

    def targv(cmd, entry):
        return ["--output", "json", cmd, "--api", "browser",
                "--max-threads", "0", "--max-spawned-processes", "0", entry]

    pin_run = capture_pin("C3 json.stdout (run) across ext", [
        (expand_source(SOURCE, e), targv("run", f"main.{e}"), ENV) for e in EXTS])
    pin_test = capture_pin("C3 json.stdout (test) across ext", [
        (expand_source(SOURCE, e), targv("test", f"smoke.test.{e}"), ENV) for e in EXTS])

    chain = assert_fns_exist(text, [
        "kali_bin", "browser_bundle_bracketed_math_sqrt_cbrt_source", BUNDLE_FN,
        "browser_harness_bracketed_math_sqrt_cbrt_run_source",
        "browser_harness_bracketed_math_sqrt_cbrt_test_source", HARNESS_FN])

    header = list(EXTRA_DECL_HEAD)
    header.append(P.extra_ok(pin_run, P.EXTRA_OK_JSON_STDOUT))
    if pin_test != pin_run:
        header.append(P.extra_ok(pin_test, P.EXTRA_OK_JSON_STDOUT))
    header += [f"Migrated from tests/browser_{stem}.rs.", ""]
    header += P.rule12_no_comments_prose(rs_path(stem), stem).split("\n") + [""]
    header += P.matrix_arithmetic(
        test_fns=24, invocations=24, cases=6, axis="ext", values=EXTS,
        helpers=[
            (BUNDLE_FN, 8, "ext(js/ts/jsx/tsx) x json_output(false/true)"),
            (HARNESS_FN, 16, "command(run/test) x ext(4) x json_output(false/true)"),
        ]) + [""]
    header += P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell") + [
        "There is no loop anywhere in this file: every one of the 24 invocations is a single",
        "unlooped `#[test]` fn, so each fanned trial traces back to exactly one source fn.",
        "",
    ]
    header += P.u2_source_file_wide(["app.${ext}", "main.${ext}", "smoke.test.${ext}"]) + [""]
    header += P.rule13_header(chain) + [""]
    header += P.ARGV_ORDER + [""]
    header += [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"Bundle helper: `exit = \"success\"` on the build ({c(b_build_exit)}) and on the "
        f"harness process ({c(b_harness_exit)});",
        "in json mode the envelope's schemaVersion/command/success/exitCode/payload("
        "artifactKind,",
        f"bundleFormat) ({cr(b_env_first, b_env_last)}) -- the source makes no `errors` claim on "
        "this build",
        f"envelope, so none is written; the emitted `app/app.meta.json` metadata "
        f"({cr(b_meta_a, b_meta_b)}),",
        "claimed in BOTH modes because the source reads it outside the `if json_output`; then",
        f"the harness step's two plain `.contains` claims ({c(b_c2)}, {c(b_c3)}), which stay",
        "`stdout_contains`.",
        f"Harness helper: `exit = \"success\"` ({c(h_exit)}); the argv carries `--max-threads 0`",
        f"({c(h_threads)}) and `--max-spawned-processes 0` ({c(h_procs)}), and the environment "
        "carries",
        f"KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node ({c(h_envvar)}), spelled as a string literal "
        "in this",
        "source rather than through the `kali_runtime_contract` constant.",
        "json mode carries schemaVersion/command/success/payload(hostContract, runtimeBackend)",
        f"({cr(h_env_first, h_env_last)}), plus `exitCode` at both levels for `run` "
        f"({cr(h_run_a, h_run_b)}) or payload",
        f"total/passed/failed for `test` ({cr(h_test_a, h_test_b)}) -- the source asserts NO "
        "`skipped` and NO",
        "`errors` on this envelope, so neither is written; then the exact `json.stdout` pin "
        "standing",
        f"for the two `.contains` claims ({c(h_json_2)}, {c(h_json_3)}) and `stderr = \"\"` "
        f"({c(h_stderr)}).",
        f"Text mode carries the two raw-stdout claims ({c(h_txt_2)}, {c(h_txt_3)}) and nothing "
        "else.",
        "This file uses no count key: the source makes no `.matches(...).count()` claim anywhere.",
    ]

    PROGRAM = ("a program that calls `globalThis[\"Math\"][\"sqrt\"](4)` and "
               "`globalThis[\"Math\"][\"cbrt\"](-27)`, so its two console.log calls print 2 and -3")

    bundle_prose = (
        f"Migrated from browser_{stem}.rs, the four "
        "`build_emits_bracketed_math_sqrt_and_cbrt_zero_identities_in_*_input` fns (one per "
        f"extension). `{BUNDLE_FN}` runs `kali build --bundle --api browser` on the "
        "tree-shake-marked bundle fixture, reads the emitted `app/app.meta.json` "
        f"({cr(b_meta_a, b_meta_b)}), then runs the bundle glue under the browser-bundle-harness "
        f"contract against {PROGRAM}. Both exit statuses are asserted ({c(b_build_exit)} for the "
        f"build, {c(b_harness_exit)} for the harness process). Its two stdout claims are "
        f"`stdout.contains(\"2\\n\")` ({c(b_c2)}) and `stdout.contains(\"-3\\n\")` ({c(b_c3)}). "
        + P.ruling3_substring())

    bundle_json_prose = bundle_prose + (
        " This sibling additionally asserts the JSON build envelope -- schemaVersion/command/"
        f"success/exitCode and payload artifactKind/bundleFormat ({cr(b_env_first, b_env_last)}). "
        "The source asserts no `errors` array on this envelope, so none is written. Output shape "
        "is not a matrix axis because it changes the assertion shape rather than substituting a "
        "string, so it is a separate case.")

    def harness_prose(cmd, json_mode):
        head = (
            f"Migrated from browser_{stem}.rs, the four "
            f"`{'json_' if json_mode else ''}{cmd}_supports_bracketed_math_sqrt_and_cbrt_zero_"
            f"identities_when_browser_harness_is_configured_in_*_input` fns (one per extension). "
            f"`{HARNESS_FN}` runs `kali {cmd} "
            f"{'--output json ' if json_mode else ''}--api browser --max-threads 0 "
            f"--max-spawned-processes 0` ({c(h_threads)}, {c(h_procs)}) with the browser harness "
            f"backed by node ({c(h_envvar)}), against {PROGRAM}. The process exit status is "
            f"asserted at {c(h_exit)}. ")
        if not json_mode:
            return head + (
                "The text branch makes two plain claims about raw stdout: "
                f"`stdout.contains(\"2\\n\")` ({c(h_txt_2)}) and `stdout.contains(\"-3\\n\")` "
                f"({c(h_txt_3)}). " + P.ruling3_substring())
        env_tail = (
            f"`exitCode` at both the envelope and the payload level ({cr(h_run_a, h_run_b)})"
            if cmd == "run" else
            f"payload total/passed/failed ({cr(h_test_a, h_test_b)}); the source asserts no "
            "`skipped` on this envelope, so none is written")
        return head + (
            "This sibling asserts the JSON envelope: schemaVersion/command/success, payload "
            f"hostContract/runtimeBackend ({cr(h_env_first, h_env_last)}), {env_tail}, plus "
            f"`stderr` exactly empty ({c(h_stderr)}). The source makes no `errors` claim on this "
            "envelope, so none is written. Its two claims about the JSON stdout leaf are "
            f"`.contains(\"2\\n\")` ({c(h_json_2)}) and `.contains(\"-3\\n\")` ({c(h_json_3)}). "
            + P.ruling3_json_leaf() +
            " One pin carries both, because an exact equality on that leaf implies every "
            "substring claim taken against it.")

    asserts = {"stdout_contains": ["2\n", "-3\n"]}

    def json_harness(cmd, entry, pin):
        env = envelope_harness(cmd, stderr=True, errors=False)
        return harness_step(cmd, entry, json_output=True, thread_flags=True,
                            json_claims=with_stdout_pin(env, pin), asserts={})

    cases = [
        {"name": "build_emits_bracketed_math_sqrt_and_cbrt_zero_identities",
         "rationale": bundle_prose,
         "steps": bundle_steps("app.${ext}", HARNESS_BODY, asserts,
                               json_output=False, meta_fields=META)},
        {"name": "json_build_emits_bracketed_math_sqrt_and_cbrt_zero_identities",
         "rationale": bundle_json_prose,
         "steps": bundle_steps("app.${ext}", HARNESS_BODY, asserts,
                               json_output=True, meta_fields=META,
                               json_claims=envelope_build(errors=False))},
        {"name": "run_supports_bracketed_math_sqrt_and_cbrt_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("run", False),
         "steps": [harness_step("run", "main.${ext}", json_output=False,
                                thread_flags=True, asserts=asserts)]},
        {"name": "test_supports_bracketed_math_sqrt_and_cbrt_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("test", False),
         "steps": [harness_step("test", "smoke.test.${ext}", json_output=False,
                                thread_flags=True, asserts=asserts)]},
        {"name": "json_run_supports_bracketed_math_sqrt_and_cbrt_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("run", True),
         "steps": [json_harness("run", "main.${ext}", pin_run)]},
        {"name": "json_test_supports_bracketed_math_sqrt_and_cbrt_zero_identities_when_browser_harness_is_configured",
         "rationale": harness_prose("test", True),
         "steps": [json_harness("test", "smoke.test.${ext}", pin_test)]},
    ]
    return stem, header, {"ext": EXTS}, SOURCE, cases


# ==========================================================================
# C4 -- browser_math_max_min_frozen_aliases   (12 of 13 fns; U4 trim-and-keep)
# ==========================================================================

def build_c4():
    stem = "math_max_min_frozen_aliases"
    text = rs(stem)
    BUNDLE_FN = "assert_browser_bundle_global_this_math_max_min_frozen"
    HARNESS_FN = "assert_browser_harness_global_this_math_max_min_frozen"
    RETAINED = ("browser_bundle_global_this_math_max_min_frozen_source_includes_"
                "direct_frozen_math_aliases")

    BUNDLE_SRC = fixture_in_fn(text, "browser_bundle_global_this_math_max_min_frozen_source", 0)
    RUN_SRC = fixture_in_fn(text, "browser_harness_global_this_math_max_min_run_source", 0)
    TEST_SRC = fixture_in_fn(text, "browser_harness_global_this_math_max_min_test_source", 0)
    HARNESS_BODY = fixture_starting(text, BUNDLE_FN, "const mod = await import(")
    if not HARNESS_BODY.startswith("const mod = await import("):
        raise AssertionError(f"wrong harness body extracted: {HARNESS_BODY[:60]!r}")

    b_exit = cite_in_fn(text, BUNDLE_FN, r"output\.status\.success\(\)", expect=2)
    b_build_exit, b_harness_exit = b_exit[0], b_exit[1]
    b_env_first = cite_in_fn(text, BUNDLE_FN, r'envelope\["schemaVersion"\]')
    b_env_last = cite_in_fn(text, BUNDLE_FN, r'payload\["bundleFormat"\]')
    b_meta_a = cite_in_fn(text, BUNDLE_FN, r'metadata\["apiSurface"\]')
    b_meta_b = cite_in_fn(text, BUNDLE_FN, r'metadata\["artifactKind"\]')
    b_c3 = cite_in_fn(text, BUNDLE_FN, r'stdout\.contains\("3\\n"\)')
    b_c1 = cite_in_fn(text, BUNDLE_FN, r'stdout\.contains\("1\\n"\)')

    h_exit = cite_in_fn(text, HARNESS_FN, r"output\.status\.success\(\)")
    h_env_first = cite_in_fn(text, HARNESS_FN, r'json\["schemaVersion"\]')
    h_env_last = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["runtimeBackend"\]')
    h_run_a = cite_in_fn(text, HARNESS_FN, r'assert_eq!\(json\["exitCode"\], 0\)')
    h_run_b = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["exitCode"\]')
    h_test_a = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["total"\]')
    h_test_b = cite_in_fn(text, HARNESS_FN, r'json\["payload"\]\["failed"\]')
    h_json_3 = cite_in_fn(text, HARNESS_FN, r'^\s+\.contains\("3\\n"\),')
    h_json_1 = cite_in_fn(text, HARNESS_FN, r'^\s+\.contains\("1\\n"\),')
    h_stderr = cite_in_fn(text, HARNESS_FN, r'json\["stderr"\]')
    h_errors = cite_in_fn(text, HARNESS_FN, r'json\["errors"\]')
    h_txt_3 = cite_in_fn(text, HARNESS_FN, r'stdout\.contains\("3\\n"\), "stdout')
    h_txt_1 = cite_in_fn(text, HARNESS_FN, r'stdout\.contains\("1\\n"\), "stdout')
    h_envvar = cite_in_fn(text, HARNESS_FN, r'\.env\("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"')
    retained_line = P.cite_line(text, r"^fn " + re.escape(RETAINED) + r"\(\)")
    # The retained test's own needles, derived by searching its body. These are the
    # claims `audit-case-migration.py` reports as missing (ruling 4's blind spot);
    # `verify_c4_audit()` below asserts the audit reports exactly these and no more.
    blocked_needles = cite_in_fn(text, RETAINED, r'source\.contains\("', expect=8)
    # Mirror-the-source in the negative: this helper really does NOT pass them.
    assert_absent_in_fn(text, HARNESS_FN, "--max-threads")
    assert_absent_in_fn(text, HARNESS_FN, "--max-spawned-processes")

    EXTS = ["js", "ts"]
    SOURCE = {"app.${ext}": BUNDLE_SRC, "main.${ext}": RUN_SRC,
              "smoke.test.${ext}": TEST_SRC}

    def targv(cmd, entry):
        return ["--output", "json", cmd, "--api", "browser", entry]

    pin_run = capture_pin("C4 json.stdout (run) across ext", [
        (expand_source(SOURCE, e), targv("run", f"main.{e}"), ENV) for e in EXTS])
    pin_test = capture_pin("C4 json.stdout (test) across ext", [
        (expand_source(SOURCE, e), targv("test", f"smoke.test.{e}"), ENV) for e in EXTS])

    chain = assert_fns_exist(text, [
        "kali_bin", "browser_bundle_global_this_math_max_min_frozen_source", BUNDLE_FN,
        "browser_harness_global_this_math_max_min_run_source",
        "browser_harness_global_this_math_max_min_test_source", HARNESS_FN])

    header = list(EXTRA_DECL_HEAD)
    header.append(P.extra_ok(pin_run, P.EXTRA_OK_JSON_STDOUT))
    if pin_test != pin_run:
        header.append(P.extra_ok(pin_test, P.EXTRA_OK_JSON_STDOUT))
    header += [f"Migrated from tests/browser_{stem}.rs.", ""]
    header += P.partial_retention_note(
        stem=stem, retained_fn=RETAINED, migrated=12, total=13,
        blocking=("its whole body is five `assert!(source.contains(<literal>))` self-checks "
                  f"against that builder's own returned text ({c(retained_line)}).")) + [""]
    header += [
        "AUDIT STATUS FOR THIS PAIR, MEASURED AND ESCALATED, NOT SHIPPED AROUND (rule 3/4).",
    ] + wrap(
        f"`audit-case-migration.py` EXITS 1 on this pair and reports {len(blocked_needles)} "
        f"`[contains literals]` claims absent from this file. Every one of them is a "
        f"`source.contains(\"Object.freeze(...)\")` needle of the RETAINED test "
        f"({cr(blocked_needles[0], blocked_needles[-1])}) -- the fixture-self-inspection claims "
        "described above, which are not migrated and which controller ruling 4 says cannot be. "
        "They are not dropped behaviour claims: each of those literals is present verbatim in "
        "this file's `[source]` bundle body, and the audit excludes everything under `[source]` "
        "from its search by construction. That is exactly the blind spot ruling 4 documents and "
        "forbids tooling around."
    ) + wrap(
        "NOTHING was added to any assertion key to turn the gate green. Doing so would move "
        "fixture text onto an assertion surface and manufacture a claim the source never made "
        "about behaviour (rule 2) -- the false green ruling 4 exists to prevent. This "
        "generator instead RUNS the audit after writing the file and hard-fails unless the "
        "reported difference is exactly those needles, so the escalation cannot quietly grow to "
        "cover a real dropped claim: every `assert_eq!` value, argv token, JSON key and "
        "behaviour-`contains` literal of the migrated 12 is present."
    ) + wrap(
        "The `.rs` is trimmed by the orchestrator, and per ruling 9 the trimmed file carries the "
        "COMPLETE measured red-list for this pair, post-trim and pre-trim alike. Read it there."
    ) + [""]
    header += P.rule12_no_comments_prose(rs_path(stem), stem).split("\n") + [""]
    header += P.matrix_arithmetic(
        test_fns=12, invocations=12, cases=6, axis="ext", values=EXTS,
        helpers=[
            (BUNDLE_FN, 4, "ext(js/ts) x json_output(false/true)"),
            (HARNESS_FN, 8, "command(run/test) x ext(js/ts) x json_output(false/true)"),
        ]) + [
        "(That 12 is the MIGRATED set. The file holds 13 `#[test]` fns; the 13th is the retained",
        "fixture self-inspection named in the PARTIAL MIGRATION block above, and it makes no",
        "helper invocation at all -- it never builds a command.)",
        "THE AXIS HERE IS TWO VALUES, NOT FOUR. Every other bundle+harness file in this batch",
        "runs js/ts/jsx/tsx; this one runs js and ts only, at every one of its 12 invocations.",
        "Declaring `ext(4)` would fan 12 more trials the source never ran -- a rule-2 invention",
        "wearing a matrix, and unfixable per-case because U1 makes the axis file-wide.",
        "",
    ]
    header += P.rule6_matrix_fold("2 source `#[test]` fns, one per `ext` cell") + [
        "(`[ext=jsx]` in the sentence above is the shared vocabulary's generic illustration of a",
        "fanned trial id. THIS file's axis is js/ts, so its trial ids read `[ext=js]` and",
        "`[ext=ts]` and there is no jsx trial -- see THE AXIS HERE IS TWO VALUES above.)",
        "There is no loop anywhere in this file: every one of the 12 migrated invocations is a",
        "single unlooped `#[test]` fn, so each fanned trial traces back to exactly one source fn.",
        "",
    ]
    header += P.u2_source_file_wide(["app.${ext}", "main.${ext}", "smoke.test.${ext}"]) + [""]
    header += P.rule13_header(chain) + [""]
    header += P.ARGV_ORDER + [""]
    header += [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"Bundle helper: `exit = \"success\"` on the build ({c(b_build_exit)}) and on the "
        f"harness process ({c(b_harness_exit)});",
        "in json mode the envelope's schemaVersion/command/success/exitCode/payload("
        "artifactKind,",
        f"bundleFormat) ({cr(b_env_first, b_env_last)}) -- the source makes no `errors` claim on "
        "this build",
        f"envelope, so none is written; the emitted `app/app.meta.json` metadata "
        f"({cr(b_meta_a, b_meta_b)}),",
        "claimed in BOTH modes because the source reads it outside the `if json_output`; then",
        f"the harness step's two plain `.contains` claims ({c(b_c3)}, {c(b_c1)}), which stay",
        "`stdout_contains`.",
        f"Harness helper: `exit = \"success\"` ({c(h_exit)}); the environment carries",
        f"KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node ({c(h_envvar)}). THIS HELPER PASSES NEITHER",
        "`--max-threads` NOR `--max-spawned-processes` -- unlike the other four files in this",
        "group -- so neither appears on argv here. That absence was asserted mechanically in",
        "this file's generator against the helper's body, not eyeballed.",
        "json mode carries schemaVersion/command/success/payload(hostContract, runtimeBackend)",
        f"({cr(h_env_first, h_env_last)}), plus `exitCode` at both levels for `run` "
        f"({cr(h_run_a, h_run_b)}) or payload",
        f"total/passed/failed for `test` ({cr(h_test_a, h_test_b)}) -- no `skipped`, which the "
        "source never",
        f"asserts; then the exact `json.stdout` pin standing for the two `.contains` claims",
        f"({c(h_json_3)}, {c(h_json_1)}), `stderr = \"\"` ({c(h_stderr)}) and `errors = []` "
        f"({c(h_errors)}). This source DOES",
        "assert the empty errors array on the HARNESS envelope (though not on the build one), so",
        "it is written.",
        f"Text mode carries the two raw-stdout claims ({c(h_txt_3)}, {c(h_txt_1)}) and nothing "
        "else.",
        "This file uses no count key: the source makes no `.matches(...).count()` claim anywhere.",
    ]

    PROGRAM = ("a program that computes globalThis.Math.max(1, 2, 3) and "
               "globalThis.Math.min(3, 2, 1) -- the middle argument reaching them through a "
               "local alias bound to 2 -- and then re-invokes each through nine further frozen "
               "Object.freeze(...) spellings, so its twenty console.log calls alternate 3 and 1")

    bundle_prose = (
        f"Migrated from browser_{stem}.rs, the two "
        "`build_emits_global_this_math_max_min_frozen_aliases_in_*_input` fns (one per "
        f"extension -- this source runs js and ts only). `{BUNDLE_FN}` runs `kali build "
        "--bundle --api browser` on the tree-shake-marked bundle fixture, reads the emitted "
        f"`app/app.meta.json` ({cr(b_meta_a, b_meta_b)}), then runs the bundle glue under the "
        f"browser-bundle-harness contract against {PROGRAM}. Both exit statuses are asserted "
        f"({c(b_build_exit)} for the build, {c(b_harness_exit)} for the harness process). Its "
        f"two stdout claims are `stdout.contains(\"3\\n\")` ({c(b_c3)}) and "
        f"`stdout.contains(\"1\\n\")` ({c(b_c1)}). " + P.ruling3_substring())

    bundle_json_prose = bundle_prose + (
        " This sibling additionally asserts the JSON build envelope -- schemaVersion/command/"
        f"success/exitCode and payload artifactKind/bundleFormat ({cr(b_env_first, b_env_last)}). "
        "The source asserts no `errors` array on THIS envelope (it does on the harness one), so "
        "none is written here. Output shape is not a matrix axis because it changes the "
        "assertion shape rather than substituting a string, so it is a separate case.")

    def harness_prose(cmd, json_mode):
        head = (
            f"Migrated from browser_{stem}.rs, the two "
            f"`{'json_' if json_mode else ''}{cmd}_supports_global_this_math_max_min_frozen_"
            f"aliases_when_browser_harness_is_configured_in_*_input` fns (one per extension -- "
            f"this source runs js and ts only). `{HARNESS_FN}` runs `kali {cmd} "
            f"{'--output json ' if json_mode else ''}--api browser` with the browser harness "
            f"backed by node ({c(h_envvar)}), against {PROGRAM}. This helper passes NO "
            "`--max-threads` and NO `--max-spawned-processes`, unlike the other harness helpers "
            "migrated in this batch, so neither appears on argv. The process exit status is "
            f"asserted at {c(h_exit)}. ")
        if not json_mode:
            return head + (
                "The text branch makes two plain claims about raw stdout: "
                f"`stdout.contains(\"3\\n\")` ({c(h_txt_3)}) and `stdout.contains(\"1\\n\")` "
                f"({c(h_txt_1)}). " + P.ruling3_substring())
        env_tail = (
            f"`exitCode` at both the envelope and the payload level ({cr(h_run_a, h_run_b)})"
            if cmd == "run" else
            f"payload total/passed/failed ({cr(h_test_a, h_test_b)}); the source asserts no "
            "`skipped` on this envelope, so none is written")
        return head + (
            "This sibling asserts the JSON envelope: schemaVersion/command/success, payload "
            f"hostContract/runtimeBackend ({cr(h_env_first, h_env_last)}), {env_tail}, plus "
            f"`stderr` exactly empty ({c(h_stderr)}) and an empty `errors` array "
            f"({c(h_errors)}). Its two claims about the JSON stdout leaf are "
            f"`.contains(\"3\\n\")` ({c(h_json_3)}) and `.contains(\"1\\n\")` ({c(h_json_1)}). "
            + P.ruling3_json_leaf() +
            " One pin carries both, because an exact equality on that leaf implies every "
            "substring claim taken against it.")

    asserts = {"stdout_contains": ["3\n", "1\n"]}

    def json_harness(cmd, entry, pin):
        env = envelope_harness(cmd, stderr=True, errors=True)
        return harness_step(cmd, entry, json_output=True, thread_flags=False,
                            json_claims=with_stdout_pin(env, pin), asserts={})

    cases = [
        {"name": "build_emits_global_this_math_max_min_frozen_aliases",
         "rationale": bundle_prose,
         "steps": bundle_steps("app.${ext}", HARNESS_BODY, asserts,
                               json_output=False, meta_fields=META)},
        {"name": "json_build_emits_global_this_math_max_min_frozen_aliases",
         "rationale": bundle_json_prose,
         "steps": bundle_steps("app.${ext}", HARNESS_BODY, asserts,
                               json_output=True, meta_fields=META,
                               json_claims=envelope_build(errors=False))},
        {"name": "run_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured",
         "rationale": harness_prose("run", False),
         "steps": [harness_step("run", "main.${ext}", json_output=False,
                                thread_flags=False, asserts=asserts)]},
        {"name": "test_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured",
         "rationale": harness_prose("test", False),
         "steps": [harness_step("test", "smoke.test.${ext}", json_output=False,
                                thread_flags=False, asserts=asserts)]},
        {"name": "json_run_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured",
         "rationale": harness_prose("run", True),
         "steps": [json_harness("run", "main.${ext}", pin_run)]},
        {"name": "json_test_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured",
         "rationale": harness_prose("test", True),
         "steps": [json_harness("test", "smoke.test.${ext}", pin_test)]},
    ]
    return stem, header, {"ext": EXTS}, SOURCE, cases


# ==========================================================================
# C5 -- browser_math_round_bracketed_root      (NO MATRIX -- 16 named siblings)
# ==========================================================================

def build_c5():
    stem = "math_round_bracketed_root"
    text = rs(stem)
    BUNDLE_FN = "assert_browser_bundle_bracketed_global_this_math_round"
    LOOP_FN = ("run_and_test_supports_bracketed_global_this_math_round_identity_"
               "when_browser_harness_is_configured_in_js_and_ts_input")

    BUNDLE_SRC = fixture_in_fn(text, "browser_bundle_bracketed_global_this_math_round_source", 0)
    HARNESS_BODY = fixture_starting(text, BUNDLE_FN, "const mod = await import(")
    if not HARNESS_BODY.startswith("const mod = await import("):
        raise AssertionError(f"wrong harness body extracted: {HARNESS_BODY[:60]!r}")
    # The loop's four `(command, source_name, source, expected_stdout)` entries hold
    # the run program twice and the test program twice, byte-identical each time.
    RUN_SRC = P.assert_identical("C5 loop run fixture (main.js vs main.ts)",
                                 fixture_in_fn(text, LOOP_FN, 2),
                                 fixture_in_fn(text, LOOP_FN, 10))
    TEST_SRC = P.assert_identical("C5 loop test fixture (smoke.test.js vs smoke.test.ts)",
                                  fixture_in_fn(text, LOOP_FN, 6),
                                  fixture_in_fn(text, LOOP_FN, 14))
    EXPECTED = P.assert_identical(
        "C5 loop expected_stdout, all four entries",
        *[fixture_in_fn(text, LOOP_FN, i) for i in (3, 7, 11, 15)])
    if EXPECTED != "2\n":
        raise AssertionError(f"expected_stdout is {EXPECTED!r}, not '2\\n'")
    if not RUN_SRC.startswith("const value = 1.6;"):
        raise AssertionError(f"wrong run fixture: {RUN_SRC[:40]!r}")
    if not TEST_SRC.startswith("Kali.test("):
        raise AssertionError(f"wrong test fixture: {TEST_SRC[:40]!r}")
    # Ruling 8 check: does the fn's `_in_js_and_ts_input` name match its body?
    loop_names = [fixture_in_fn(text, LOOP_FN, i) for i in (1, 5, 9, 13)]
    if sorted(loop_names) != sorted(["main.js", "smoke.test.js", "main.ts", "smoke.test.ts"]):
        raise AssertionError(f"loop entries are not js/ts only: {loop_names}")

    b_exit = cite_in_fn(text, BUNDLE_FN, r"output\.status\.success\(\)", expect=2)
    b_build_exit, b_harness_exit = b_exit[0], b_exit[1]
    b_env_first = cite_in_fn(text, BUNDLE_FN, r'envelope\["schemaVersion"\]')
    b_env_last = cite_in_fn(text, BUNDLE_FN, r'payload\["bundleFormat"\]')
    b_meta_a = cite_in_fn(text, BUNDLE_FN, r'metadata\["apiSurface"\]')
    b_meta_b = cite_in_fn(text, BUNDLE_FN, r'metadata\["artifactKind"\]')
    b_c2 = cite_in_fn(text, BUNDLE_FN, r'stdout\.contains\("2\\n"\)')
    b_count = cite_in_fn(text, BUNDLE_FN, r'stdout\.matches\("2\\n"\)\.count\(\) >= 2')

    h_exit = cite_in_fn(text, LOOP_FN, r"output\.status\.success\(\)")
    h_env_first = cite_in_fn(text, LOOP_FN, r'json\["schemaVersion"\]')
    h_env_last = cite_in_fn(text, LOOP_FN, r'json\["payload"\]\["runtimeBackend"\]')
    h_run_a = cite_in_fn(text, LOOP_FN, r'assert_eq!\(json\["exitCode"\], 0\)')
    h_run_b = cite_in_fn(text, LOOP_FN, r'json\["payload"\]\["exitCode"\]')
    h_test_a = cite_in_fn(text, LOOP_FN, r'json\["payload"\]\["total"\]')
    h_test_b = cite_in_fn(text, LOOP_FN, r'json\["payload"\]\["failed"\]')
    h_json_c = cite_in_fn(text, LOOP_FN, r'json\["stdout"\]\.as_str\(\)\.expect\("stdout"\)'
                                         r'\.contains\(expected_stdout\)')
    h_stderr = cite_in_fn(text, LOOP_FN, r'json\["stderr"\]')
    h_txt_c = cite_in_fn(text, LOOP_FN, r'stdout\.contains\(expected_stdout\)')
    h_threads = cite_in_fn(text, LOOP_FN, r'\.arg\("--max-threads"\)')
    h_procs = cite_in_fn(text, LOOP_FN, r'\.arg\("--max-spawned-processes"\)')
    h_envvar = cite_in_fn(text, LOOP_FN, r'BROWSER_HARNESS_COMMAND_ENV')
    loop_line = P.cite_line(text, r"^fn " + re.escape(LOOP_FN) + r"\($")

    EXTS = ["js", "ts", "jsx", "tsx"]
    SOURCE = {}
    for e in EXTS:
        SOURCE[f"app.{e}"] = BUNDLE_SRC
    SOURCE["main.js"] = RUN_SRC
    SOURCE["main.ts"] = RUN_SRC
    SOURCE["smoke.test.js"] = TEST_SRC
    SOURCE["smoke.test.ts"] = TEST_SRC
    # Ruling 7: the duplication is asserted mechanically, not eyeballed.
    P.assert_identical("C5 [source] bundle bodies", *[SOURCE[f"app.{e}"] for e in EXTS])
    P.assert_identical("C5 [source] run bodies", SOURCE["main.js"], SOURCE["main.ts"])
    P.assert_identical("C5 [source] test bodies",
                       SOURCE["smoke.test.js"], SOURCE["smoke.test.ts"])

    def targv(cmd, entry):
        return ["--output", "json", cmd, "--api", "browser",
                "--max-threads", "0", "--max-spawned-processes", "0", entry]

    # No `[matrix]` here, so each json case is one invocation; the four are still
    # captured and asserted identical, which is what lets one pin serve them.
    pin = capture_pin("C5 json.stdout across all four harness cells", [
        (SOURCE, targv(cmd, entry), ENV)
        for cmd, entry in [("run", "main.js"), ("test", "smoke.test.js"),
                           ("run", "main.ts"), ("test", "smoke.test.ts")]])

    chain = assert_fns_exist(text, [
        "kali_bin", "browser_bundle_bracketed_global_this_math_round_source", BUNDLE_FN])

    header = list(EXTRA_DECL_HEAD)
    header.append(P.extra_ok(pin, P.EXTRA_OK_JSON_STDOUT))
    header += [f"Migrated from tests/browser_{stem}.rs.", ""]
    header += P.rule12_no_comments_prose(rs_path(stem), stem).split("\n") + [""]
    header += P.matrix_declined(
        test_fns=9, invocations=16, cases=16,
        reason=[
            f"  * `{BUNDLE_FN}` -- 8 invocation(s) =",
            "    ext(js/ts/jsx/tsx) x json_output(false/true), from 8 unlooped `#[test]` fns.",
            f"  * ONE looping `#[test]` fn ({c(loop_line)}) whose body INLINES the harness logic",
            "    instead of calling a helper -- 8 invocation(s) = 4",
            "    `(command, source_name, source, expected_stdout)` entries covering js and ts",
            "    ONLY x `for output_json in [false, true]`.",
            "The bundle half varies over FOUR extensions and the harness half over TWO, so there",
            "is no single file-wide `ext` axis: `ext(4)` would fan the harness cases onto jsx and",
            "tsx, which the source never ran, and `ext(2)` would drop the bundle's jsx/tsx cells.",
        ]) + [""]
    header += P.RULE6_ONE_TO_ONE + [
        "The eight harness cases are the eight loop iterations of that one `#[test]` fn, so their",
        "names are the fn's own stem with `run_and_test` resolved to the `run`/`test` the",
        "iteration actually passed and the extension it actually used.",
        "",
        "RULING 8 CHECKED, AND NO MIGRATION NOTE IS DUE. The looping fn's name ends",
        "`_in_js_and_ts_input`, which on several sources in this batch is stale. Here it is",
        "accurate: this generator reads the four `source_name` literals out of the loop table and",
        "asserts they are exactly main.js, smoke.test.js, main.ts and smoke.test.ts, so the fn",
        "really does cover js and ts only. (Its C1 sibling in this same group carries the same",
        "suffix over an eight-entry table and DOES get a ruling-8 note.)",
        "",
    ]
    header += P.u2_source_file_wide(list(SOURCE)) + [""]
    header += P.RULING7_NO_HOIST + [
        "Concretely: the four `app.<ext>` bodies are one program, `main.js`/`main.ts` are one",
        "program, and `smoke.test.js`/`smoke.test.ts` are one program. Without a `[matrix]` there",
        "is no `${ext}` key to collapse them into, so the duplication is structural, not sloppy.",
        "",
    ]
    header += count_keys_block([
        ("the bundle harness process's raw stdout, `stdout.matches(\"2\\n\").count() >= 2`",
         c(b_count), "`stdout_count` with `at_least = 2`"),
    ]) + wrap(
        f"That count sits beside a SEPARATE `stdout.contains(\"2\\n\")` claim ({c(b_c2)}) on "
        "the same step; the two are different source claims and both are carried. The harness "
        "half of this file makes no count claim at all, so no count key appears on any harness "
        "case."
    ) + [""]
    header += P.rule13_header(chain, extra=[
        "This file has no `assert_browser_harness_*` helper to check: the harness half is written",
        f"inline in the looping `#[test]` fn itself ({c(loop_line)}), which carries no `///` doc",
        "either (it carries no comment of any kind -- see the RULE 12 block above).",
    ]) + [""]
    header += P.ARGV_ORDER + [""]
    header += [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"Bundle helper: `exit = \"success\"` on the build ({c(b_build_exit)}) and on the "
        f"harness process ({c(b_harness_exit)});",
        "in json mode the envelope's schemaVersion/command/success/exitCode/payload("
        "artifactKind,",
        f"bundleFormat) ({cr(b_env_first, b_env_last)}) -- the source makes no `errors` claim on "
        "this build",
        f"envelope, so none is written; the emitted `app/app.meta.json` metadata "
        f"({cr(b_meta_a, b_meta_b)}),",
        "claimed in BOTH modes because the source reads it outside the `if json_output`; then",
        f"the harness step's `stdout_contains` ({c(b_c2)}) and `stdout_count` ({c(b_count)}).",
        f"Inlined harness half: `exit = \"success\"` ({c(h_exit)}); the argv carries "
        "`--max-threads 0`",
        f"({c(h_threads)}) and `--max-spawned-processes 0` ({c(h_procs)}); the environment "
        "variable is set",
        f"through `kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` ({c(h_envvar)}), whose "
        "real value",
        f"was read from crates/kali_runtime_contract/src/browser/contract.rs and is",
        f"\"{ENV_NAME}\".",
        "json mode carries schemaVersion/command/success/payload(hostContract, runtimeBackend)",
        f"({cr(h_env_first, h_env_last)}), plus `exitCode` at both levels for `run` "
        f"({cr(h_run_a, h_run_b)}) or payload",
        f"total/passed/failed for `test` ({cr(h_test_a, h_test_b)}) -- the source asserts NO "
        "`skipped` and NO",
        "`errors` on this envelope, so neither is written; then the exact `json.stdout` pin "
        "standing",
        f"for `.contains(expected_stdout)` ({c(h_json_c)}) and `stderr = \"\"` ({c(h_stderr)}).",
        f"Text mode carries `stdout.contains(expected_stdout)` ({c(h_txt_c)}) and nothing else.",
        "`expected_stdout` is a loop variable, and this generator asserted that all four table",
        "entries bind it to the same literal before writing the needle -- so the needle below is",
        "the source's own literal, not a value chosen for it.",
    ]

    PROGRAM_B = ("a program that calls `globalThis[\"Math\"].round(1.6)` and "
                 "`globalThis[\"Math\"][\"round\"](1.6)`, so both console.log calls print 2")

    def bundle_prose(ext, json_mode):
        base = (
            f"Migrated from browser_{stem}.rs, the single `#[test]` fn "
            f"`{'json_' if json_mode else ''}build_emits_bracketed_global_this_math_round_"
            f"identity_literals_in_{ext}_input`. `{BUNDLE_FN}` runs `kali build --bundle --api "
            "browser` on the tree-shake-marked bundle fixture, reads the emitted "
            f"`app/app.meta.json` ({cr(b_meta_a, b_meta_b)}), then runs the bundle glue under the "
            f"browser-bundle-harness contract against {PROGRAM_B}. Both exit statuses are "
            f"asserted ({c(b_build_exit)} for the build, {c(b_harness_exit)} for the harness "
            f"process). The source makes TWO separate claims about the harness stdout -- "
            f"`stdout.contains(\"2\\n\")` ({c(b_c2)}) and `stdout.matches(\"2\\n\").count() >= 2` "
            f"({c(b_count)}) -- so both are carried, as `stdout_contains` and `stdout_count`; "
            "collapsing them into one would drop a claim. "
            + P.ruling3_substring() + " " + P.ruling3_count('"2\\n"', 2))
        if json_mode:
            base += (
                " This sibling additionally asserts the JSON build envelope -- schemaVersion/"
                "command/success/exitCode and payload artifactKind/bundleFormat "
                f"({cr(b_env_first, b_env_last)}). The source asserts no `errors` array on this "
                "envelope, so none is written.")
        return base

    def harness_case_prose(cmd, ext, json_mode):
        entry = f"main.{ext}" if cmd == "run" else f"smoke.test.{ext}"
        head = (
            f"Migrated from browser_{stem}.rs, ONE iteration of the looping `#[test]` fn "
            f"`{LOOP_FN}` ({c(loop_line)}): the `(\"{cmd}\", \"{entry}\", ...)` table entry with "
            f"output_json = {'true' if json_mode else 'false'}. That fn inlines the harness "
            f"logic rather than calling a helper. It runs `kali {cmd} "
            f"{'--output json ' if json_mode else ''}--api browser --max-threads 0 "
            f"--max-spawned-processes 0` ({c(h_threads)}, {c(h_procs)}) with the browser harness "
            f"backed by node, set through `kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` "
            f"({c(h_envvar)}), against {PROGRAM_B}. The process exit status is asserted at "
            f"{c(h_exit)}. ")
        if not json_mode:
            return head + (
                "The text branch makes one plain claim about raw stdout, "
                f"`stdout.contains(expected_stdout)` ({c(h_txt_c)}), where `expected_stdout` is "
                "the table entry's own fourth field and is the literal \"2\\n\" in all four "
                "entries. " + P.ruling3_substring())
        env_tail = (
            f"`exitCode` at both the envelope and the payload level ({cr(h_run_a, h_run_b)})"
            if cmd == "run" else
            f"payload total/passed/failed ({cr(h_test_a, h_test_b)}); the source asserts no "
            "`skipped` on this envelope, so none is written")
        return head + (
            "This sibling asserts the JSON envelope: schemaVersion/command/success, payload "
            f"hostContract/runtimeBackend ({cr(h_env_first, h_env_last)}), {env_tail}, plus "
            f"`stderr` exactly empty ({c(h_stderr)}). The source makes no `errors` claim on this "
            "envelope, so none is written. Its stdout claim is "
            f"`json[\"stdout\"].as_str().contains(expected_stdout)` ({c(h_json_c)}), with "
            "`expected_stdout` bound to \"2\\n\" in all four table entries. "
            + P.ruling3_json_leaf())

    bundle_asserts = {"stdout_contains": ["2\n"],
                      "stdout_count": [{"needle": "2\n", "at_least": 2}]}
    text_asserts = {"stdout_contains": ["2\n"]}

    cases = []
    for ext in EXTS:
        cases.append({
            "name": f"build_emits_bracketed_global_this_math_round_identity_literals_in_{ext}_input",
            "rationale": bundle_prose(ext, False),
            "steps": bundle_steps(f"app.{ext}", HARNESS_BODY, bundle_asserts,
                                  json_output=False, meta_fields=META)})
    for ext in EXTS:
        cases.append({
            "name": f"json_build_emits_bracketed_global_this_math_round_identity_literals_in_{ext}_input",
            "rationale": bundle_prose(ext, True),
            "steps": bundle_steps(f"app.{ext}", HARNESS_BODY, bundle_asserts,
                                  json_output=True, meta_fields=META,
                                  json_claims=envelope_build(errors=False))})
    for json_mode in (False, True):
        for ext in ("js", "ts"):
            for cmd in ("run", "test"):
                entry = f"main.{ext}" if cmd == "run" else f"smoke.test.{ext}"
                prefix = "json_" if json_mode else ""
                name = (f"{prefix}{cmd}_supports_bracketed_global_this_math_round_identity_"
                        f"when_browser_harness_is_configured_in_{ext}_input")
                if json_mode:
                    step = harness_step(
                        cmd, entry, json_output=True, thread_flags=True,
                        json_claims=with_stdout_pin(
                            envelope_harness(cmd, stderr=True, errors=False), pin),
                        asserts={},
                        env_var=ENV_NAME)
                else:
                    step = harness_step(cmd, entry, json_output=False, thread_flags=True,
                                        asserts=text_asserts, env_var=ENV_NAME)
                cases.append({"name": name,
                              "rationale": harness_case_prose(cmd, ext, json_mode),
                              "steps": [step]})

    if len(cases) != 16:
        raise AssertionError(f"C5 must write 16 cases, wrote {len(cases)}")
    return stem, header, None, SOURCE, cases


# ==========================================================================

BUILDERS = [build_c1, build_c2, build_c3, build_c4, build_c5]

# C4's audit is EXPECTED to exit 1, on exactly the retained fixture-self-inspection
# test's needles and nothing else (controller ruling 4). An expected red that is not
# pinned is indistinguishable from a real dropped claim the next time this runs, so
# the expectation is checked mechanically rather than remembered.
C4_EXPECTED_MISSING = {
    "Object.freeze(Math.max)",
    "Object.freeze(Math.min)",
    'Object.freeze(Math["max"])',
    'Object.freeze(Math["min"])',
    'Object.freeze(globalThis.Math["max"])',
    'Object.freeze(globalThis.Math["min"])',
    'Object.freeze(globalThis["Math"]["max"])',
    'Object.freeze(globalThis["Math"]["min"])',
}


def verify_c4_audit(stem):
    """Run the real audit and hard-fail unless its difference is exactly the
    retained test's fixture-text needles."""
    import subprocess
    proc = subprocess.run(
        [sys.executable, os.path.join(REPO, "scripts/audit-case-migration.py"),
         f"browser_{stem}.rs", f"cases/browser/{stem}.toml"],
        cwd=TESTS, capture_output=True, text=True)
    reported = set(re.findall(r"^\s+\[contains literals\] '(.*)'$", proc.stdout, re.M))
    reported |= set(re.findall(r'^\s+\[contains literals\] "(.*)"$', proc.stdout, re.M))
    other = re.findall(r"^\s+\[(?!contains literals)([a-z ]+)\]", proc.stdout, re.M)
    if reported != C4_EXPECTED_MISSING or other:
        raise AssertionError(
            f"{stem}: audit difference is NOT the ruling-4 blind spot alone.\n"
            f"  reported contains-literals: {sorted(reported)}\n"
            f"  other missing claim kinds:  {other}\n"
            f"  expected exactly:           {sorted(C4_EXPECTED_MISSING)}\n"
            "A real claim may have been dropped -- do NOT ship this pair.")
    print(f"  {stem}: audit exit={proc.returncode}, difference is exactly the "
          f"{len(reported)} retained fixture-self-inspection needles (ruling 4). "
          "Escalated, not shipped around.")


def main():
    for builder in BUILDERS:
        stem, header, matrix, source, cases = builder()
        write(os.path.join(CASES, f"{stem}.toml"),
              emit(header, matrix, source, cases))
        if stem == "math_max_min_frozen_aliases":
            verify_c4_audit(stem)


if __name__ == "__main__":
    main()
