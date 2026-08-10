#!/usr/bin/env python3
r"""Generate the Task 18 batch 8A case files (12 migrated targets, 13 files).

Batch 8A was dispatched with fourteen targets. TWELVE are migrated here. The
other two -- `browser_promise_any_bundle` and `browser_promise_any_harness` --
are design-spec 5.11 WHOLE-FILE RETENTIONS and deliberately have no case file;
they are the exactly-two unadjudicated fixture-self-inspection instances that
controller ruling 10 names by filename, with the reach counts it states, and
ruling 4 fixes their disposition (retain hand-written with a U3 `//!` header;
do NOT extend the audit script for the shape). Re-derived here rather than
taken on trust:

    $ python3 tools/task-18-browser-pilot/find_fixture_self_inspection.py --selftest
    SELFTEST OK -- all 14 known instances re-found, [...]
    $ python3 tools/task-18-browser-pilot/find_fixture_self_inspection.py | tail -3
    UNADJUDICATED: 2 -> ['browser_promise_any_bundle.rs', 'browser_promise_any_harness.rs']

WHY A GENERATOR AND NOT THIRTEEN HAND-WRITTEN FILES. The same reason batches 5,
6A, 6B, 7A and 7B each shipped one: batch 4 shipped cross-file prose divergence
that every per-file gate passed individually, because no gate reads `#` header
prose or `rationale` wording (U8). Every recurring sentence is therefore CALLED
from `batch5_prose` / `math_shapes` / `gen_batch6a`, not retyped. This module
writes only the PER-FILE spec -- the program under test, the invocation
arithmetic, the assertion inventory and the `:N` citations -- which is what
review has to read.

Nothing under `tools/` or `scripts/` is modified by this batch; this file and
`batch8a_captures.py` are added and everything else is used as it stands.

THE BATCH'S ONE STRUCTURAL DECISION, DERIVED RATHER THAN ASSUMED.
`browser_reflect_own_keys` is a `#[path]` submodule carrier (U10) with four
submodules and 44 `#[test]` fns. U10 says migrate a carrier and its sibling
directory into ONE `.toml`. That is WRONG here and U2 takes precedence, for a
reason that is MEASURED, not argued:

    16 of the 44 fns (8 in run.rs, 8 in test.rs) go through helpers that write
    a `kali.json` manifest and pass NO `--api` flag; the browser API surface
    can only come from the manifest. The other 28 pass `--api browser`
    explicitly against a tree with no manifest.

    `expand.rs:195` substitutes and clones the whole file-level `[source]` map
    into EVERY trial, so one shared table would make `kali.json`
    unconditionally present. Probed against the real binary, same argv, the
    manifest's mere presence moves the two fields these cases actually assert:

        $ kali --output json test smoke.test.js     # no manifest, no --api
          hostContract= kali-hosted        runtimeBackend= wasmtime
        $ kali --output json test smoke.test.js     # kali.json present
          hostContract= browser-requested  runtimeBackend= browser-harness

    So a shared `[source]` would let the 28 explicit cases assert
    `hostContract = "browser-requested"` and have it supplied by the leaked
    manifest rather than by the flag under test. No literal is dropped by that
    leak, so `audit-case-migration.py` cannot see it; the trial still passes,
    so `cargo test` cannot either.

THE SPLIT IS ON MANIFEST PRESENCE, NOT ON THE SUBMODULE BOUNDARY, and the
difference is load-bearing: `run.rs` and `test.rs` EACH straddle the manifest
boundary (8 explicit + 8 inherited apiece). One case file per submodule -- the
shape the batch brief floated as acceptable -- would therefore put both halves
of run.rs in one file and both halves of test.rs in one file, which is exactly
the disarmament above, twice. Two files split on the manifest is the correct
answer; four split on the submodule is not. This matches batch 6B's split of
`browser_non_literal_iterator_sources`, the only other four-submodule carrier
migrated so far, which is the same carrier shape (build/check/run/test).

CITATIONS. Every `:N` below is produced by `batch5_prose.cite_line(rs_text,
regex)` at generation time, by SEARCHING the source for the construct. None is
computed by arithmetic and none is carried over from an earlier measurement.
`cite_line` raises unless its anchor matches exactly the expected number of
times, so a vanished or ambiguous anchor is a generator error rather than a
silently wrong number. Citations into a `#[path]` submodule are written
`<file>.rs:N` and are resolved against THAT file by `batch5_crosscheck.py`; a
bare `:N` means the carrier.

RULE 8 / RULE 9. Twelve of the twenty fixture texts in this batch are built by
a `format!`, by a `.lines()/format!/join` re-indentation, or one level removed
inside `kali_common`. None is hand-derived: they are the byte-exact output of
executing the real code and they live in `batch8a_captures.py`, whose docstring
records the exact capture procedure. `check_captured` re-checks each one
against its own `.rs` before it is emitted, so a capture taken before a source
edit fails the generator instead of shipping a program that is no longer the
program under test.

RULE 10. The two `template_literal_string_iteration` targets DO contain a
genuine JS template literal, and `expand.rs:195` substitutes `[source]` bodies,
so both files declare `[constants] dollar = "$"`. That is DERIVED, not marked:
`_rule10` inspects the captured fixture and raises if a file that declares the
constant has no `${` in it, or if a file that does not declare it has one.

U9. Every claim emitted by this generator is live-verified against the real
built `kali` by `--verify`, which runs each emitted case's argv in a fresh temp
dir seeded with that file's own `[source]` table and compares the observed exit
status and output against what the case asserts -- every case, not a sample.

Run: python3 gen_batch8a.py [name ...]     (no args = all)
     python3 gen_batch8a.py --verify       (emit, then live-verify every case)
"""

import json as _json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

from case_emit import emit, write, source_text  # noqa: E402
from math_shapes import (  # noqa: E402
    META, bundle_steps, envelope_build, harness_step,
)
import batch5_prose as P  # noqa: E402
import batch8a_captures as C  # noqa: E402
from gen_batch6a import (  # noqa: E402
    FAIL_CLOSED_NOTE, check_captured, check_program, comment_blocks, emit6a, hdr,
)

EXTS4 = ["js", "ts", "jsx", "tsx"]
HARNESS_ENV = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"

REGISTRY = {}
# Every emitted file's live-verification plan, filled in as each target renders
# so `--verify` runs the SAME argv/source/claims the file was emitted with
# rather than a second reading of the shipped TOML. U9 wants the binary asked
# about the case, not about a re-parse of it.
VERIFY = {}


def target(name):
    def deco(fn):
        REGISTRY[name] = fn
        return fn
    return deco


def rs(name):
    """The source a case file is generated FROM.

    Batch 8A declares no U4 trim-and-keep retention -- its two retentions are
    WHOLE-FILE, which by definition removes nothing, so `source_text`'s
    `PRE-TRIM REF:` branch is not reached for any target here. The precondition
    is checked rather than assumed, so that a later trim of one of these
    sources cannot silently regenerate a smaller case file from a trimmed file.
    """
    text = source_text(name, quiet=True)
    if text.startswith("//!") and "PRE-TRIM REF:" in text:
        raise AssertionError(
            f"browser_{name}.rs has become a U4 TRIM retention (it carries both a `//!` "
            "header and a PRE-TRIM REF). Batch 8A's targets are whole-file retentions or "
            "plain migrations; regenerate against the pre-trim blob deliberately.")
    return text


def rule12_from(rs_text, *, expect):
    """Every Rust comment block in the source, COPIED (rule 12), never retyped.

    `expect` is the number of blocks the caller has accounted for. A source
    that grows a comment this generator does not carry is a rule-12 violation
    no gate would report as such (`comment_coverage.py` reports it as a
    missing line, which is easy to read as the known U6 false positive), so a
    mismatch is raised here instead.
    """
    blocks = comment_blocks(rs_text)
    if len(blocks) != expect:
        raise AssertionError(
            f"rule 12: source has {len(blocks)} Rust comment block(s), this generator "
            f"accounts for {expect}. Blocks at line(s) {[b[0] for b in blocks]}.")
    return blocks


def prose_of(block):
    """A comment block rendered as one verbatim sentence run for a rationale."""
    return " ".join(line for line in block[1] if line).strip()


def _rule10(fixtures):
    """Derive whether this file needs `[constants] dollar`, never mark it.

    Ruling 18: a gate that selects its arm from a marker in prose is only as
    good as the prose. So the caller does not TELL this function whether the
    file has a template literal -- it inspects the fixture texts, returns the
    escaped bodies and the constants table together, and the two cannot
    disagree because they are one computation.
    """
    needs = any("${" in body for body in fixtures.values())
    escaped = {k: v.replace("${", "${dollar}{") for k, v in fixtures.items()}
    constants = {"dollar": "$"} if needs else {}
    for key, body in escaped.items():
        bare = body.replace("${dollar}{", "")
        if "${" in bare:
            raise AssertionError(
                f"[source] {key!r} still carries an unescaped `${{` after the rule-10 "
                "escape; expand.rs's substitute() would hard-fail on it")
    return escaped, constants


RULE10_PROSE = [
    "RULE 10 -- A GENUINE JS TEMPLATE LITERAL, ESCAPED THROUGH `[constants]`.",
    "The program under test interpolates two of its own bindings with a real JS template",
    "literal. `expand.rs`'s `substitute()` hard-fails on any `${...}` it cannot resolve,",
    "and `expand.rs:195` substitutes `[source]` BODIES as well as step fields, so this file",
    "declares `[constants] dollar = \"$\"` and spells every genuine `${` as `${dollar}{`.",
    "The RESOLVED program text is byte-identical to the source's -- this is an encoding of",
    "rule 9, not an exception to it. The generator DERIVES this rather than marking it:",
    "it escapes whatever `${` the captured fixture actually contains and raises if any",
    "survives unescaped, so a file cannot declare the constant it does not need or need",
    "the constant it does not declare.",
]

RULING16_NOTE = [
    "No count of the wider `browser/` corpus appears anywhere in this file (ruling 16): a",
    "family-wide population count has no gateable home inside a case file, and the",
    "remaining batches would falsify it by construction.",
]


# =========================================================================
# SHAPE A -- the five `assert_browser_bundle_<name>(filename, json_output)`
#            targets. 8 fns = ext(4) x json_output(2), uniform over ext.
# =========================================================================

BUNDLE_TARGETS = {
    "promise_all_bundle": dict(
        helper="assert_browser_bundle_promise_all",
        builder="browser_bundle_promise_all_source",
        fixture=C.CAP_PROMISE_ALL_BUNDLE,
        fn_prefix="build_emits_promise_all_in_{ext}_input",
        export="browserPromiseAll",
        anchors=[("// kali-tree-shake: browserPromiseAll",
                  "// kali-tree-shake: browserPromiseAll")],
        kc_fn="promise_all_browser_body_source",
        kc_doc="Canonical browser smoke body for the supported `Promise.all` slice.",
        what="the browser `Promise.all` smoke body",
    ),
    "promise_all_settled_bundle": dict(
        helper="assert_browser_bundle_promise_all_settled",
        builder="browser_bundle_promise_all_settled_source",
        fixture=C.CAP_PROMISE_ALL_SETTLED_BUNDLE,
        fn_prefix="build_emits_promise_all_settled_in_{ext}_input",
        export="browserPromiseAllSettled",
        anchors=[("// kali-tree-shake: browserPromiseAllSettled",
                  "// kali-tree-shake: browserPromiseAllSettled")],
        kc_fn="promise_all_settled_browser_body_source",
        kc_doc="Canonical browser smoke body for the supported `Promise.allSettled` slice.",
        what="the browser `Promise.allSettled` smoke body",
    ),
    "promise_race_bundle": dict(
        helper="assert_browser_bundle_promise_race",
        builder="browser_bundle_promise_race_source",
        fixture=C.CAP_PROMISE_RACE_BUNDLE,
        fn_prefix="build_emits_promise_race_in_{ext}_input",
        export="browserPromiseRace",
        anchors=[("// kali-tree-shake: browserPromiseRace",
                  "// kali-tree-shake: browserPromiseRace")],
        kc_fn="promise_race_browser_body_source",
        kc_doc="Canonical browser smoke body for the supported `Promise.race` slice.",
        what="the browser `Promise.race` smoke body",
    ),
    "string_concatenation_bundle": dict(
        helper="assert_browser_bundle_string_concatenation",
        builder="browser_bundle_string_concatenation_source",
        fixture=C.CAP_STRING_CONCAT_BUNDLE,
        fn_prefix="build_emits_browser_string_concatenation_in_{ext}_input",
        export="browserStringConcatenation",
        anchors=[("// kali-tree-shake: browserStringConcatenation",
                  "// kali-tree-shake: browserStringConcatenation")],
        kc_fn=None,
        kc_doc=None,
        what="the string-concatenation iteration probe",
    ),
    "template_literal_string_iteration_bundle": dict(
        helper="assert_browser_bundle_template_literal_string_iteration",
        builder="browser_bundle_template_literal_string_iteration_source",
        fixture=C.CAP_TLSI_BUNDLE,
        fn_prefix="build_emits_browser_template_literal_string_iteration_in_{ext}_input",
        export="browserTemplateLiteralStringIteration",
        anchors=[("// kali-tree-shake: browserTemplateLiteralStringIteration",
                  "// kali-tree-shake: browserTemplateLiteralStringIteration")],
        kc_fn="browser_template_literal_string_iteration_body_source",
        kc_doc=("Canonical browser body for the supported template-literal string "
                "iteration slice."),
        what="the template-literal string-iteration probe",
    ),
}


def build_bundle(name, spec):
    text = rs(name)
    helper = spec["helper"]

    c_helper = P.cite_line(text, rf"fn {helper}\(")
    c_builder = P.cite_line(text, rf"fn {spec['builder']}\(")
    c_build_exit = P.cite_line(text, r"^\s*output\.status\.success\(\),")
    c_meta = P.cite_line(text, r'join\("app\.meta\.json"\)')
    c_fail = P.cite_line(text, r"must fail closed")
    c_errors = P.cite_line(text, r'envelope\["errors"\]')

    raw = check_captured(f"app.${{ext}} ({name})", spec["fixture"], text,
                         anchors=spec["anchors"], must_contain="async function"
                         if "async function" in spec["fixture"] else "function ")
    source, constants = _rule10({"app.${ext}": raw})

    harness_body = check_program(
        "harness body",
        _harness_body(text, helper, spec["export"]),
        must_contain="await import(")

    blocks = rule12_from(text, expect=1)
    repin = prose_of(blocks[0])

    docs = [spec["kc_doc"]] if spec["kc_doc"] else []
    chain = [helper, spec["builder"]]

    header = hdr(
        f"Migrated from tests/browser_{name}.rs.",
        "",
        P.matrix_arithmetic(
            test_fns=8, invocations=8,
            helpers=[(helper, 8,
                      f"4 individual `#[test]` fns per output mode x 2 output modes; "
                      f"the file contains no loop at all, so 8 fns = 8 invocations")],
            cases=2, axis="ext", values=EXTS4, non_axes=("json_output",)),
        "",
        P.rule6_matrix_fold(
            "one `(json_output)` half of the source's 8 fns, fanned to the 4 extensions"),
        "",
        P.u2_source_file_wide(["app.${ext}"]),
        "",
        "U5 -- NO `[source]` KEY RENAME IS NEEDED. This source writes exactly one program",
        "text, to `app.<ext>` in every test, so the file-wide `[source]` namespace has one",
        "entry per extension and nothing collides.",
        "",
        _bundle_shape(c_build_exit, c_meta, c_fail, c_errors),
        "",
        (RULE10_PROSE + [""]) if constants else None,
        P.ARGV_ORDER,
        "",
        P.rule13_header(chain, docs_carried=docs),
        "",
        _rule12_block(name, blocks, "every case in this file",
                      f"`{helper}`, which every `#[test]` fn in the file calls"),
        "",
        RULING16_NOTE,
    )

    cases, plans = [], []
    for json_output in (False, True):
        fn_names = [spec["fn_prefix"].format(ext=e) for e in EXTS4]
        if json_output:
            fn_names = ["json_" + n for n in fn_names]
        for n in fn_names:
            if not P.cite_line(text, rf"fn {n}\(", expect=1):
                raise AssertionError(f"{name}: no `fn {n}` in source")
        steps = bundle_steps(
            "app.${ext}", harness_body, {"exit": "failure"},
            json_output=json_output,
            json_claims=envelope_build(errors=True) if json_output else None,
            meta_fields=META)
        rationale = _bundle_rationale(
            name, spec, json_output, fn_names, repin, docs,
            c_helper, c_builder, c_build_exit, c_meta, c_fail, c_errors)
        cases.append({
            "name": ("json_" if json_output else "") + f"{name}__" + (
                "json" if json_output else "text"),
            "rationale": rationale,
            "steps": steps,
        })
        plans.append({"kind": "bundle", "json": json_output,
                      "source": source, "constants": constants})

    VERIFY[name] = plans
    return emit6a(header, constants, {"ext": EXTS4}, source, cases)


def _harness_body(text, helper, export):
    """The browser-bundle harness body, pulled out of the `.rs` by CONTENT.

    Content-anchored (`fixture_starting`) rather than by index, so it survives
    a line shift; and the export the harness calls is checked against the
    export the bundle fixture declares, so a body lifted from the wrong helper
    fails here instead of shipping a harness that imports nothing.
    """
    from case_emit import fixture_starting
    body = fixture_starting(text, helper, "const mod = await import(")
    if f"mod.{export}" not in body:
        raise AssertionError(
            f"harness body does not call `mod.{export}`: {body[:80]!r}")
    return body


def _bundle_shape(c_build_exit, c_meta, c_fail, c_errors):
    return [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the `kali build --bundle` process (:{c_build_exit}).",
        f"In JSON mode the envelope's schemaVersion/command/success/exitCode and",
        f"payload(artifactKind, bundleFormat) are pinned, plus `errors = []` (:{c_errors}) --",
        "this file's helper does assert the errors array is empty, and a file whose helper",
        "does not would not carry the claim.",
        f"The emitted `app/app.meta.json` (:{c_meta}) is pinned to apiSurface/artifactKind.",
        f"THEN THE HARNESS PROCESS MUST FAIL CLOSED (:{c_fail}): the `browser_bundle_harness`",
        "step carries `exit = \"failure\"` and NO output claim, because the source makes none",
        "there. Adding a diagnostic code or a stream needle would be a rule-2 invention even",
        "though the real binary does emit one.",
    ]


def _rule12_block(name, blocks, reach, helper_desc):
    lines = [
        "RULE 12 / U6 -- SOURCE COMMENT PROSE, CARRIED VERBATIM AND ATTRIBUTED BOTTOM-UP.",
        f"tests/browser_{name}.rs has {len(blocks)} Rust comment block(s), listed with their",
        "line numbers before any TOML was written:",
    ]
    for start, body in blocks:
        first = next((l for l in body if l), "")
        lines.append(f"  * :{start} ({len(body)} line(s)) -- \"{first}...\"")
    lines += [
        f"It sits in {helper_desc},",
        f"so it is carried into the rationale of {reach}. The text is COPIED out of the `.rs`",
        "by this generator (`comment_blocks`), not retyped, so an em-dash cannot become `--`.",
    ]
    return lines


def _bundle_rationale(name, spec, json_output, fn_names, repin, docs,
                      c_helper, c_builder, c_build_exit, c_meta, c_fail, c_errors):
    listed = ", ".join(f"`{n}`" for n in fn_names)
    mode = "--output json" if json_output else "text output"
    parts = [
        f"Migrated from browser_{name}.rs. This `[[case]]` is the {mode} half of the "
        f"source's 8 `#[test]` fns, matrix-fanned by `ext(4)` to the four extensions, so it "
        f"stands for {listed} -- one trial per fn, assertion mapping 1:1 (rule 6's sanctioned "
        f"rule-7 fold, stated here as the rule requires).",
        f"`{spec['helper']}(` (:{c_helper}) writes {spec['what']} to `app.<ext>` in a fresh "
        f"temp dir, builds it with `kali build --bundle --api browser"
        f"{' --output json' if json_output else ''}`, asserts the build succeeds "
        f"(:{c_build_exit})"
        + (f", asserts the JSON envelope's schemaVersion/command/success/exitCode and "
           f"payload(artifactKind, bundleFormat) and that `errors` is empty (:{c_errors})"
           if json_output else "")
        + f", asserts the emitted `app/app.meta.json` metadata (:{c_meta}), then writes the "
          f"browser-bundle harness and runs it under node.",
        f"THE BUILD SUCCEEDS; it is the HARNESS process that must fail closed (:{c_fail}), so "
        f"the `browser_bundle_harness` step carries `exit = \"failure\"` and no output claim -- "
        f"the source makes none there, and inventing one would break rule 2.",
        f"The program under test is `{spec['builder']}()` (:{c_builder}); its text is the "
        f"byte-exact output of executing that real code (rules 8 and 9), never hand-derived.",
        f"RULE 12 -- the Rust comment prose of browser_{name}.rs, carried verbatim: "
        f"\"{repin}\"",
    ]
    if docs:
        parts.append(P.rule13_carried(docs) + " That doc belongs to "
                     + spec["kc_fn"] + " (named plainly rather than backticked: U8's gate "
                     "resolves backticked identifiers against this source's own fn list and "
                     "this one lives in kali_common).")
    return " ".join(parts)


for _n, _s in BUNDLE_TARGETS.items():
    REGISTRY[_n] = (lambda n=_n, s=_s: build_bundle(n, s))


# =========================================================================
# SHAPE B -- the five uniform-filename harness targets. 16 fns =
#            ext(4) x command(2) x json_output(2), uniform over ext.
# =========================================================================

HARNESS_TARGETS = {
    "promise_all_harness": dict(
        helper="assert_browser_requested_promise_all",
        run_builder="browser_promise_all_run_source",
        test_builder="browser_promise_all_test_source",
        run_fixture=C.CAP_PROMISE_ALL_RUN, test_fixture=C.CAP_PROMISE_ALL_TEST,
        fn_prefix="{cmd}_supports_promise_all_in_{ext}_input_when_browser_harness_is_configured",
        thread_flags=False,
        kc_fn="promise_all_browser_body_source",
        kc_doc="Canonical browser smoke body for the supported `Promise.all` slice.",
        what="the browser `Promise.all` smoke probe",
    ),
    "promise_all_settled_harness": dict(
        helper="assert_browser_requested_promise_all_settled",
        run_builder="browser_promise_all_settled_run_source",
        test_builder="browser_promise_all_settled_test_source",
        run_fixture=C.CAP_PROMISE_ALL_SETTLED_RUN,
        test_fixture=C.CAP_PROMISE_ALL_SETTLED_TEST,
        fn_prefix=("{cmd}_supports_promise_all_settled_in_{ext}_input"
                   "_when_browser_harness_is_configured"),
        thread_flags=False,
        kc_fn="promise_all_settled_browser_body_source",
        kc_doc="Canonical browser smoke body for the supported `Promise.allSettled` slice.",
        what="the browser `Promise.allSettled` smoke probe",
    ),
    "promise_race_harness": dict(
        helper="assert_browser_requested_promise_race",
        run_builder="browser_promise_race_run_source",
        test_builder="browser_promise_race_test_source",
        run_fixture=C.CAP_PROMISE_RACE_RUN, test_fixture=C.CAP_PROMISE_RACE_TEST,
        fn_prefix=("{cmd}_supports_promise_race_in_{ext}_input"
                   "_when_browser_harness_is_configured"),
        thread_flags=False,
        kc_fn="promise_race_browser_body_source",
        kc_doc="Canonical browser smoke body for the supported `Promise.race` slice.",
        what="the browser `Promise.race` smoke probe",
    ),
    "string_concatenation_harness": dict(
        helper="assert_browser_harness_string_concatenation",
        run_builder="browser_string_concatenation_source",
        test_builder="browser_string_concatenation_source",
        run_fixture=C.CAP_STRING_CONCAT_RUN, test_fixture=C.CAP_STRING_CONCAT_TEST,
        fn_prefix=("{cmd}_supports_string_concatenation_iteration_in_browser_api_surface"
                   "_with_harness_{ext}_input"),
        thread_flags=True,
        kc_fn=None, kc_doc=None,
        what="the string-concatenation iteration probe",
    ),
    "template_literal_string_iteration_harness": dict(
        helper="assert_browser_harness_template_literal_string_iteration",
        run_builder="browser_template_literal_string_iteration_source",
        test_builder="browser_template_literal_string_iteration_source",
        run_fixture=C.CAP_TLSI_RUN, test_fixture=C.CAP_TLSI_TEST,
        fn_prefix=("{cmd}_supports_template_literal_iteration_in_browser_api_surface"
                   "_with_harness_{ext}_input"),
        thread_flags=True,
        kc_fn="browser_template_literal_string_iteration_body_source",
        kc_doc=("Canonical browser body for the supported template-literal string "
                "iteration slice."),
        what="the template-literal string-iteration probe",
    ),
}


def build_harness(name, spec):
    text = rs(name)
    helper = spec["helper"]
    c_helper = P.cite_line(text, rf"fn {helper}\(")
    c_fail = P.cite_line(text, r"must fail closed")

    run_raw = check_captured(f"main.${{ext}} ({name})", spec["run_fixture"], text,
                             anchors=[], must_contain="console.log")
    test_raw = check_captured(f"smoke.test.${{ext}} ({name})", spec["test_fixture"], text,
                              anchors=[], must_contain="Kali.test(")
    if run_raw == test_raw:
        raise AssertionError(
            f"{name}: the run and test fixtures are byte-identical; the helper picks its "
            "body from the command, so they must differ")
    source, constants = _rule10({"main.${ext}": run_raw,
                                 "smoke.test.${ext}": test_raw})

    blocks = rule12_from(text, expect=1)
    repin = prose_of(blocks[0])
    docs = [spec["kc_doc"]] if spec["kc_doc"] else []
    chain = [helper] + sorted({spec["run_builder"], spec["test_builder"]})

    header = hdr(
        f"Migrated from tests/browser_{name}.rs.",
        "",
        P.matrix_arithmetic(
            test_fns=16, invocations=16,
            helpers=[(helper, 16,
                      "16 individual `#[test]` fns, each one unlooped call = "
                      "command(run/test) x json_output(2) x ext(4); the file contains no "
                      "loop, so 16 fns = 16 invocations")],
            cases=4, axis="ext", values=EXTS4),
        "",
        P.rule6_matrix_fold(
            "one `(command, json_output)` cell of the source's 16 fns, fanned to the 4 "
            "extensions"),
        "",
        _harness_u2(),
        "",
        "U5 -- NO `[source]` KEY RENAME IS NEEDED. The helper picks its body from the",
        "COMMAND (`if command == \"test\" { ..._test_source() } else { ..._run_source() }`)",
        "and the source hands it a DIFFERENT filename per command for every extension --",
        "`main.<ext>` for `run` and `smoke.test.<ext>` for `test` -- so the two program",
        "texts never land on one key and the flat file-wide namespace has no collision.",
        "(That is the axis on which this file differs from `set_iteration_harness`, whose",
        "looped fn reuses `main.<ext>` for BOTH commands and therefore does need renames.)",
        "",
        FAIL_CLOSED_NOTE,
        f"The source's fail-closed assertion is at :{c_fail}.",
        "",
        (RULE10_PROSE + [""]) if constants else None,
        P.ARGV_ORDER,
        "",
        P.rule13_header(chain, docs_carried=docs, runner_exemption=False),
        "This file runs no `browser_bundle_harness` step at all -- every case is a single",
        "`kali run`/`kali test` invocation -- so ruling 6's runner-infrastructure paragraph",
        "is omitted rather than printed about a chain this file never reaches.",
        "",
        _rule12_block(name, blocks, "every case in this file",
                      f"`{helper}`, which every `#[test]` fn in the file calls"),
        "",
        RULING16_NOTE,
    )

    cases, plans = [], []
    for command in ("run", "test"):
        for json_output in (False, True):
            entry = ("main.${ext}" if command == "run" else "smoke.test.${ext}")
            fn_names = [
                ("json_" if json_output else "")
                + spec["fn_prefix"].format(cmd=command, ext=e) for e in EXTS4]
            for n in fn_names:
                P.cite_line(text, rf"fn {n}\(")
            step = fail_closed_harness_step(
                command, entry, json_output=json_output,
                thread_flags=spec["thread_flags"])
            cases.append({
                "name": ("json_" if json_output else "") + f"{command}_{name}",
                "rationale": _harness_rationale(
                    name, spec, command, json_output, fn_names, repin, docs,
                    c_helper, c_fail, entry),
                "steps": [step],
            })
            plans.append({"kind": "harness", "steps": [step],
                          "source": source, "constants": constants})

    VERIFY[name] = plans
    return emit6a(header, constants, {"ext": EXTS4}, source, cases)


def fail_closed_harness_step(command, entry, *, json_output, thread_flags):
    """`math_shapes.harness_step` for a step that asserts ONLY `exit = failure`.

    `harness_step` raises when `json_output=True` and no `json_claims` are
    given, and that guard is right: a builder that defaulted the envelope on
    would manufacture claims the source never made (rule 2). But these sources
    assert `!output.status.success()` and NOTHING ELSE, in BOTH output modes --
    there is no envelope to pin because the process never produces a valid one.
    So the step is built here instead of teaching the shared builder a
    "claimless JSON" mode that every other caller would then have to opt out of.

    The argv is NOT re-derived: it is taken from `harness_step` itself for the
    text case and the `--output json` pair is inserted at the position
    `harness_step` puts it (BEFORE the subcommand -- see `batch5_prose.
    ARGV_ORDER`), then the two are cross-checked, so this function cannot drift
    from the shared builder's argv order without failing here.
    """
    base = harness_step(command, entry, json_output=False,
                        asserts={"exit": "failure"}, thread_flags=thread_flags)
    argv = (["--output", "json"] if json_output else []) + list(base["args"])
    reference = harness_step(command, entry, json_output=json_output,
                             asserts={"exit": "failure"}, thread_flags=thread_flags,
                             json_claims={} if json_output else None)
    if argv != reference["args"]:
        raise AssertionError(
            f"argv drifted from math_shapes.harness_step: {argv} vs {reference['args']}")
    return {"args": argv, "env": {HARNESS_ENV: "node"}, "exit": "failure"}


def _harness_u2():
    return [
        "U2 -- `[source]` is FILE-WIDE, and that is safe here. Both fixtures",
        "(`main.${ext}`, `smoke.test.${ext}`) are written unconditionally by the source into",
        "a fresh temp dir: no fixture is written behind an `if`, no `kali.json` manifest is",
        "written anywhere in this file, and no case's point is the presence or absence of a",
        "file. Every command below names its entry explicitly on argv, so the unused siblings",
        "in a trial dir are inert -- verified against the real binary rather than assumed:",
        "`kali --output json test smoke.test.js` in a directory holding all four",
        "`smoke.test.<ext>` fixtures still reports `payload.total = 1`, so no sibling is",
        "picked up by discovery.",
    ]


def _harness_rationale(name, spec, command, json_output, fn_names, repin, docs,
                       c_helper, c_fail, entry):
    listed = ", ".join(f"`{n}`" for n in fn_names)
    builder = spec["run_builder"] if command == "run" else spec["test_builder"]
    flags = (" --max-threads 0 --max-spawned-processes 0" if spec["thread_flags"] else "")
    argv = (("--output json " if json_output else "") + command
            + " --api browser" + flags)
    parts = [
        f"Migrated from browser_{name}.rs. This `[[case]]` is the "
        f"`({command}, {'json' if json_output else 'text'})` cell of the source's 16 "
        f"`#[test]` fns, matrix-fanned by `ext(4)` to the four extensions, so it stands for "
        f"{listed} -- one trial per fn, assertion mapping 1:1 (rule 6's sanctioned rule-7 "
        f"fold, stated here as the rule requires).",
        f"`{spec['helper']}(` (:{c_helper}) writes {spec['what']} to `{entry}` in a fresh "
        f"temp dir and runs the real `kali {argv}` with that dir as cwd and the browser "
        f"harness command variable set to `node`.",
        f"ASSERTION SHAPE: the source's ONLY process assertion in this file is "
        f"`assert!(!output.status.success(), \"must fail closed: {{output:?}}\")` (:{c_fail}) "
        f"-- no exit code, no stdout/stderr needle, no envelope field, in either output mode "
        f"-- so this case carries exactly `exit = \"failure\"` and nothing else. Adding a "
        f"diagnostic code or a stream claim would invent a claim the source never made "
        f"(rule 2), even though the real binary does emit one.",
        f"The program under test is `{builder}()`; its text is the byte-exact output of "
        f"executing that real code (rules 8 and 9), never hand-derived.",
        f"RULE 12 -- the Rust comment prose of browser_{name}.rs, carried verbatim: "
        f"\"{repin}\"",
    ]
    if docs:
        parts.append(P.rule13_carried(docs) + " That doc belongs to " + spec["kc_fn"]
                     + " (named plainly rather than backticked: U8's gate resolves "
                     "backticked identifiers against this source's own fn list and this one "
                     "lives in kali_common).")
    return " ".join(parts)


for _n, _s in HARNESS_TARGETS.items():
    REGISTRY[_n] = (lambda n=_n, s=_s: build_harness(n, s))


def main(argv):
    verify = "--verify" in argv
    names = [a for a in argv if not a.startswith("--")] or sorted(REGISTRY)
    unknown = [n for n in names if n not in REGISTRY]
    if unknown:
        raise SystemExit(f"unknown target(s): {unknown}\nknown: {sorted(REGISTRY)}")
    for name in names:
        print(f"--- {name}")
        write(os.path.join(CASES, f"{name}.toml"), REGISTRY[name]())
    if verify:
        import verify_batch8a
        return verify_batch8a.run(VERIFY)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
