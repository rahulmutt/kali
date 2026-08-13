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

U9. Every case emitted here is live-verified against the real built `kali` by
`cargo test -p kali_cli --test cases`, which materialises each trial's
`[source]` table into a fresh directory, runs the real binary with the case's
own argv, and checks every claim the case makes. That is per-case and not a
sample: the 136 trials these files add all execute, and the suite is red if any
claim is wrong. No separate verifier is shipped, because a second, weaker
re-implementation of what the runner already does is exactly the duplicated
predicate this project has been bitten by three times -- the runner IS the
authority on what a case asserts.

The pinned VALUES those cases carry were captured from the real binary before
they were written (`_rk_envelope`'s fields, `payload.filesChecked`,
`payload.total/passed/failed`, the `hostContract`/`runtimeBackend` pair used in
the U2 derivation above), not hand-computed.

Run: python3 gen_batch8a.py [name ...]           (no args = all)
     python3 gen_batch8a.py --reflect-preview    (render the escalated split)
"""

import json as _json
import os
import re
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
# Per-file record of the steps each target emitted, keyed by stem. Kept because
# the reflect_own_keys builders are not registered (see their block below) and
# this is the only handle a reviewer has on what they WOULD have shipped.
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


def cited(rs_text, needle, *, expect=1, pick=0):
    """`` `needle` (:N) `` -- a citation the gate can actually re-resolve.

    Ruling 11 exempts `:N` from the no-moving-figures rule ONLY because it is
    mechanically gated: "a pointer nothing re-resolves is a figure in disguise".
    A bare `(:37)` matches no reader pattern in `batch5_crosscheck.py` and is
    reported clean whether it is right or wrong, which is what
    `reword_ungated_citations.py` exists to repair after the fact. Rendering the
    construct here means the generator emits the gated form directly, and the
    construct is the LITERAL THIS FUNCTION SEARCHED FOR, so the pointer and the
    number cannot disagree.
    """
    import re as _re
    n = P.cite_line(rs_text, _re.escape(needle), label=needle, expect=expect)
    if expect > 1:
        n = n[pick]
    return f"`{needle}` (:{n})"


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


RULE10_EXTRA_OK = P.EXTRA_CLAIM_PREAMBLE + [
    P.extra_ok("$", "the value of the rule-10 `[constants] dollar` escape, not an assertion "
                    "at all -- `check_extra_claims.py` reads every [constants] value as a "
                    "claim string, and this one exists purely so `expand.rs`'s substitute() "
                    "can put a literal `$` back into the fixture. The RESOLVED program text "
                    "is byte-identical to the source's, which is the whole point of rule 10"),
]

def rule10_prose(escaped):
    """The RULE 10 block, with its binding count DERIVED from the escaped bodies.

    This was a constant reading "interpolates two of its own bindings" -- true
    for the `template_literal` pair, FALSE for `reflect_own_keys_explicit_api`
    (one binding, `${result}`), and vacuous for any file that needs no escape at
    all. A hardcoded figure inside prose that no gate reads is exactly ruling
    15's liability, so it is counted here instead: the distinct `${NAME}`
    bindings actually present, in the bodies actually emitted.
    """
    import re as _re
    names = sorted({m.group(1) for body in escaped.values()
                    for m in _re.finditer(r"\$\{dollar\}\{(\w+)\}", body)})
    if not names:
        raise AssertionError(
            "rule10_prose called for a file with no escaped template literal -- "
            "the block would describe a property the file does not have")
    # Named plainly, not backticked: U8's `check_rationale_fn_names.py` resolves
    # every backticked lower-case identifier against the source's fn list, and
    # these are JS BINDINGS inside the fixture, which will never be in it.
    listed = ", ".join(names)
    count = ("one of its own bindings" if len(names) == 1
             else f"{len(names)} of its own bindings")
    return [
        "RULE 10 -- A GENUINE JS TEMPLATE LITERAL, ESCAPED THROUGH `[constants]`.",
        f"The program under test interpolates {count} ({listed}) with a real JS",
        "template literal. `expand.rs`'s `substitute()` hard-fails on any `${...}` it cannot resolve,",
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

    c_helper = cited(text, f"fn {helper}(")
    c_builder = cited(text, f"fn {spec['builder']}(")
    # Two lines carry this text: the build-success assert and the harness
    # fail-closed assert (`!output.status.success(),`). `expect=2` makes the
    # ambiguity explicit -- if a source ever grows a third, this raises rather
    # than silently citing the wrong one.
    c_build_exit = cited(text, "output.status.success(),", expect=2, pick=0)
    c_meta = cited(text, 'join("app.meta.json")')
    c_fail = cited(text, 'assert!(!output.status.success(), "must fail closed')
    c_errors = cited(text, 'envelope["errors"]')

    raw = check_captured(f"app.${{ext}} ({name})", spec["fixture"], text,
                         anchors=spec["anchors"], must_contain="async function"
                         if "async function" in spec["fixture"] else "function ")
    raw_harness = check_program(
        "harness body",
        _harness_body(text, helper, spec["export"]),
        must_contain="await import(")
    # The harness `body` is substituted by `expand.rs` exactly as a `[source]`
    # body is, so it goes through the same rule-10 escape. No bundle harness in
    # this batch contains a `${`, but a body that grew one and was not escaped
    # would hard-fail `substitute()` at run time rather than here.
    escaped, constants = _rule10({"app.${ext}": raw, "__harness__": raw_harness})
    escaped_all = dict(escaped)
    harness_body = escaped.pop("__harness__")
    source = escaped

    blocks = rule12_from(text, expect=1)
    repin = prose_of(blocks[0])

    docs = [spec["kc_doc"]] if spec["kc_doc"] else []
    chain = [helper, spec["builder"]]

    header = hdr(
        f"Migrated from tests/browser_{name}.rs.",
        "",
        # SECTION ORDER IS FIXED BY `batch5_crosscheck.SECTIONS` and checked by
        # its structure arm: Migrated from / RULE 12 / RULE 7 / RULE 6 / U2 /
        # RULE 13 / ARGV ORDER / ASSERTION SHAPE. Blocks the list does not name
        # (U5, RULE 10, ruling 16) may sit anywhere between them.
        _rule12_block(name, blocks, "every case in this file",
                      f"`{helper}`, which every `#[test]` fn in the file calls", text),
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
        P.rule13_header(chain, docs_carried=docs),
        "",
        P.ARGV_ORDER,
        "",
        (rule10_prose(escaped_all) + [""] + RULE10_EXTRA_OK + [""]) if constants else None,
        _bundle_shape(c_build_exit, c_meta, c_fail, c_errors),
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
        f"`exit = \"success\"` on the `kali build --bundle` process: {c_build_exit}.",
        f"In JSON mode the envelope's schemaVersion/command/success/exitCode and",
        f"payload(artifactKind, bundleFormat) are pinned, plus `errors = []` -- {c_errors} --",
        "this file's helper does assert the errors array is empty, and a file whose helper",
        "does not would not carry the claim.",
        f"The emitted `app/app.meta.json` is pinned to apiSurface/artifactKind: {c_meta}.",
        f"THEN THE HARNESS PROCESS MUST FAIL CLOSED -- {c_fail} -- so the",
        "`browser_bundle_harness` step carries `exit = \"failure\"` and NO output claim,",
        "because the source makes none",
        "there. Adding a diagnostic code or a stream needle would be a rule-2 invention even",
        "though the real binary does emit one.",
    ]


def _rule12_block(name, blocks, reach, helper_desc, rs_text):
    lines = [
        "RULE 12 / U6 -- SOURCE COMMENT PROSE, CARRIED VERBATIM AND ATTRIBUTED BOTTOM-UP.",
        f"tests/browser_{name}.rs has {len(blocks)} Rust comment block(s), listed with their",
        "line numbers before any TOML was written:",
    ]
    for start, body in blocks:
        # The citation carries the comment's own first line as its backticked
        # construct, so `batch5_crosscheck.py` has something to re-resolve. A
        # bare `(:93)` matches no reader pattern and reports clean whether it is
        # right or wrong (ruling 11) -- and a comment block has no CODE
        # construct beside it, so the construct has to be the comment text.
        raw = rs_text.split("\n")[start - 1].strip()
        lines.append(f"  * `{raw}` (:{start}) -- opens a {len(body)}-line block")
    lines += [
        f"It sits in {helper_desc},",
        f"so it is carried into the rationale of {reach}. The text is COPIED out of the `.rs`",
        "by this generator (its comment_blocks helper, named plainly rather than backticked:",
        "U8's gate resolves every backticked lower-case identifier against this source's own",
        "fn list, and that one lives in the generator), not retyped, so an em-dash cannot",
        "become `--`.",
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
        f"{c_helper} writes {spec['what']} to `app.<ext>` in a fresh "
        f"temp dir, builds it with `kali build --bundle --api browser"
        f"{' --output json' if json_output else ''}`, asserts the build succeeds "
        f" -- {c_build_exit}"
        + (f", asserts the JSON envelope's schemaVersion/command/success/exitCode and "
           f"payload(artifactKind, bundleFormat) and that `errors` is empty ({c_errors})"
           if json_output else "")
        + f", asserts the emitted `app/app.meta.json` metadata ({c_meta}), then writes the "
          f"browser-bundle harness and runs it under node.",
        f"THE BUILD SUCCEEDS; it is the HARNESS process that must fail closed ({c_fail}), so "
        f"the `browser_bundle_harness` step carries `exit = \"failure\"` and no output claim -- "
        f"the source makes none there, and inventing one would break rule 2.",
        f"The program under test is {c_builder}; its text is the "
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
    escaped_all = source        # this shape emits no harness step, so every
                                # substituted body is a `[source]` body.

    blocks = rule12_from(text, expect=1)
    repin = prose_of(blocks[0])
    docs = [spec["kc_doc"]] if spec["kc_doc"] else []
    chain = [helper] + sorted({spec["run_builder"], spec["test_builder"]})

    header = hdr(
        f"Migrated from tests/browser_{name}.rs.",
        "",
        _rule12_block(name, blocks, "every case in this file",
                      f"`{helper}`, which every `#[test]` fn in the file calls", text),
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
        P.rule13_header(chain, docs_carried=docs, runner_exemption=False),
        "This file runs no `browser_bundle_harness` step at all -- every case is a single",
        "`kali run`/`kali test` invocation -- so ruling 6's runner-infrastructure paragraph",
        "is omitted rather than printed about a chain this file never reaches.",
        "",
        P.ARGV_ORDER,
        "",
        (rule10_prose(escaped_all) + [""] + RULE10_EXTRA_OK + [""]) if constants else None,
        FAIL_CLOSED_NOTE,
        f"The source's fail-closed assertion is at :{c_fail}.",
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
    names = [a for a in argv if not a.startswith("--")] or sorted(REGISTRY)
    unknown = [n for n in names if n not in REGISTRY]
    if unknown:
        raise SystemExit(f"unknown target(s): {unknown}\nknown: {sorted(REGISTRY)}")
    for name in names:
        print(f"--- {name}")
        write(os.path.join(CASES, f"{name}.toml"), REGISTRY[name]())
    if "--reflect-preview" in argv:
        # Renders both halves of the escalated reflect_own_keys split WITHOUT
        # writing them, so the escalation can be reviewed against real output
        # rather than a description of it.
        for fn in (build_rk_explicit, build_rk_inherited):
            text = fn()
            print(f"--- {fn.__name__}: {len(text.splitlines())} lines "
                  f"({text.count('[[case]]')} [[case]] entries)")
    return 0



# =========================================================================
# SHAPE C -- set_iteration_harness. 5 fns, 16 invocations, and the extension
#            axis is NOT uniform, so `[matrix]` is declined for the file.
# =========================================================================

SET_STEM = "set_iteration_harness"
SET_FN = ("{cmd}_supports_set_constructor_iteration_in_browser_api_surface"
          "_with_harness_{ext}_input")
SET_LOOP_FN = ("supports_set_constructor_iteration_in_browser_api_surface"
               "_with_harness_ts_jsx_tsx_input")


@target(SET_STEM)
def build_set_iteration():
    text = rs(SET_STEM)
    helper = "assert_browser_harness_set_iteration"
    c_helper = P.cite_line(text, rf"fn {helper}\(")
    c_fail = cited(text, 'assert!(!output.status.success(), "must fail closed')
    c_loop = P.cite_line(text, r'for extension in \["ts", "jsx", "tsx"\]')
    c_filename = P.cite_line(text, r'let filename = format!\("main\.\{extension\}"\)')
    c_pick = cited(text, 'if command == "test"')

    run_body = check_captured("run fixture", C.CAP_SET_ITERATION_RUN, text,
                              anchors=[("browserSetIteration", "browserSetIteration")],
                              must_contain="console.log")
    test_body = check_captured("test fixture", C.CAP_SET_ITERATION_TEST, text,
                               anchors=[("Kali.test(", "Kali.test(")],
                               must_contain="Kali.test(")
    if run_body == test_body:
        raise AssertionError("set_iteration: run and test fixtures are byte-identical")

    # The 16 invocations, expanded from the source's loops rather than listed.
    rows = [("run", "main.js", False), ("test", "smoke.test.js", False),
            ("run", "main.js", True), ("test", "smoke.test.js", True)]
    for ext in ("ts", "jsx", "tsx"):
        for cmd, js in (("run", False), ("test", False), ("run", True), ("test", True)):
            rows.append((cmd, f"main.{ext}", js))
    if len(rows) != 16:
        raise AssertionError(f"expanded {len(rows)} invocations, expected 16")

    # U5: only the COLLIDING keys are renamed. `main.<ext>` for ts/jsx/tsx
    # carries BOTH program texts (the looped fn reuses one filename for both
    # commands), which one flat file-wide key cannot hold; `main.js` and
    # `smoke.test.js` are already distinct and keep their source names.
    def key_for(cmd, filename):
        if filename in ("main.js", "smoke.test.js"):
            return filename
        stem, ext = filename.rsplit(".", 1)
        return f"{stem}_{'test' if cmd == 'test' else 'run'}.{ext}"

    collisions = sorted({f for _, f, _ in rows
                         if len({c for c, g, _ in rows if g == f}) > 1})
    if collisions != ["main.jsx", "main.ts", "main.tsx"]:
        raise AssertionError(f"unexpected colliding filenames: {collisions}")

    source = {}
    for cmd, filename, _ in rows:
        source[key_for(cmd, filename)] = test_body if cmd == "test" else run_body
    if len(source) != 8:
        raise AssertionError(f"[source] has {len(source)} entries, expected 8")
    P.assert_rename_is_argv_only(source, [key_for(c, f) for c, f, _ in rows], EXTS4)

    blocks = rule12_from(text, expect=2)
    # U6 -- per-helper attribution, derived from WHERE each block sits, not
    # assigned by hand. The first block is inside the `test`-only fixture
    # builder, the second inside the assert helper every invocation reaches.
    test_only_start = P.cite_line(
        text, r"fn browser_harness_set_iteration_test_source\(")
    helper_start = P.cite_line(text, r"fn assert_browser_harness_set_iteration\(")
    by_owner = {}
    for start, body in blocks:
        owner = "test_source" if test_only_start < start < helper_start else "helper"
        if owner in by_owner:
            raise AssertionError(f"two comment blocks resolved to the same owner {owner}")
        by_owner[owner] = (start, body)
    if set(by_owner) != {"test_source", "helper"}:
        raise AssertionError(f"unattributed comment block(s): {sorted(by_owner)}")
    repin = prose_of(by_owner["helper"])
    test_only_prose = prose_of(by_owner["test_source"])

    header = hdr(
        f"Migrated from tests/browser_{SET_STEM}.rs.",
        "",
        _set_rule12_block(blocks, by_owner),
        "",
        P.matrix_declined(
            test_fns=5, invocations=16, cases=16,
            reason=[
                "THE FILENAME PATTERN IS NOT UNIFORM OVER THE EXTENSION AXIS, which is what",
                "rules the axis out. The four individual `#[test]` fns use `main.js` for `run`",
                f"and `smoke.test.js` for `test`; the fifth fn, `{SET_LOOP_FN}`,",
                f"loops `for extension in [\"ts\", \"jsx\", \"tsx\"]` (:{c_loop}) building a SINGLE",
                f"`format!(\"main.{{extension}}\")` filename (:{c_filename}) that it then uses for",
                "BOTH the `run` and the `test` legs of its inner `for (command, json_output)`",
                "loop -- so ts/jsx/tsx run `kali test main.ts`/`main.jsx`/`main.tsx`, never",
                "`smoke.test.ts`/`.jsx`/`.tsx`. One file-wide `ext` axis could not express that",
                "without inventing an untested `kali test smoke.test.ts` (if it followed the js",
                "pattern) or an untested `kali test main.js` (if it followed the ts/jsx/tsx",
                "pattern) -- a rule-2 invention either way.",
                "4 individual invocations + (3 extensions x 4 (command, json_output) pairs) =",
                "4 + 12 = 16 invocations across 5 `#[test]` fns, expanded by reading the loops.",
            ]),
        "",
        P.RULE6_ONE_TO_ONE,
        "The twelve siblings that came from the looped fn are named for the invocation they",
        "are, not numbered, per rule 5.",
        "",
        P.u2_source_file_wide(sorted(source)),
        "No `kali.json` manifest is written anywhere in this file, so no case's point is the",
        "presence or absence of one.",
        "",
        P.u5_renames(
            [("main.ts", "main_run.ts / main_test.ts", "the looped fn writes BOTH program "
              "texts to this one filename"),
             ("main.jsx", "main_run.jsx / main_test.jsx", "same"),
             ("main.tsx", "main_run.tsx / main_test.tsx", "same")],
            collision="two different program texts to the same filename"),
        f"The helper picks its body from the COMMAND -- {c_pick} -- and writes it to",
        "whatever",
        "filename it was handed, which is what creates the collision. `main.js` and",
        "`smoke.test.js` are already distinct and are NOT renamed -- only the colliding keys",
        "are.",
        "",
        P.rule13_header([helper, "browser_harness_set_iteration_run_source",
                         "browser_harness_set_iteration_test_source"],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step at all, so ruling 6's",
        "runner-infrastructure paragraph is omitted rather than printed about a chain this",
        "file never reaches.",
        "",
        P.EXTRA_CLAIM_PREAMBLE + [
            P.extra_ok(k, P.EXTRA_OK_U5_RENAME)
            for k in sorted(source) if k not in ("main.js", "smoke.test.js")],
        "",
        P.ARGV_ORDER,
        "",
        FAIL_CLOSED_NOTE,
        f"The source's fail-closed assertion is at {c_fail}.",
        "",
        RULING16_NOTE,
    )

    cases, plans = [], []
    for cmd, filename, json_output in rows:
        ext = filename.rsplit(".", 1)[1]
        name = ("json_" if json_output else "") + SET_FN.format(cmd=cmd, ext=ext)
        entry = key_for(cmd, filename)
        looped = ext != "js"
        if not looped:
            P.cite_line(text, rf"fn {name}\(")
        step = fail_closed_harness_step(cmd, entry, json_output=json_output,
                                        thread_flags=True)
        cases.append({"name": name,
                      "rationale": _set_rationale(cmd, filename, entry, json_output,
                                                  looped, repin, c_helper, c_fail,
                                                  c_loop, c_filename,
                                                  test_only_prose if cmd == "test" else None),
                      "steps": [step]})
        plans.append({"kind": "harness", "steps": [step], "source": source,
                      "constants": {}})

    VERIFY[SET_STEM] = plans
    return emit(header, None, source, cases)


def _set_rule12_block(blocks, by_owner):
    lines = [
        "RULE 12 / U6 -- SOURCE COMMENT PROSE, CARRIED VERBATIM AND ATTRIBUTED BOTTOM-UP.",
        f"tests/browser_{SET_STEM}.rs has {len(blocks)} Rust comment blocks, listed with their",
        "line numbers before any TOML was written, and they do NOT reach the same cases:",
    ]
    hs, hb = by_owner["helper"]
    ts, tb = by_owner["test_source"]
    src = source_text(SET_STEM, quiet=True).split("\n")   # 8C: deleted source
    lines += [
        f"  * `{src[ts - 1].strip()}` (:{ts}) opens a {len(tb)}-line block inside",
        "    `browser_harness_set_iteration_test_source`, which the assert helper calls ONLY",
        "    when `command == \"test\"`, so it is carried into the 8 `test` rationales and into",
        "    no others.",
        f"  * `{src[hs - 1].strip()}` (:{hs}) opens a {len(hb)}-line block inside",
        "    `assert_browser_harness_set_iteration`, which every one of the 16 invocations goes",
        "    through, so it is carried into all 16.",
        "`comment_coverage.py` has no per-helper attribution and will therefore report the",
        f"{len(tb)} lines of the :{ts} block \"missing\" from the 8 `run` `[[case]]` entries. That",
        "is the checker's known limitation, recorded here rather than papered over by copying",
        "the prose into cases whose producing helper never runs -- which U6 forbids even",
        "though it would turn the checker green.",
        "The text of both blocks is COPIED out of the `.rs` by this generator (its",
        "comment_blocks helper, named plainly rather than backticked: U8's gate resolves",
        "every backticked lower-case identifier against this source's own fn list, and that",
        "one lives in the generator), not retyped, so an em-dash cannot become `--`.",
    ]
    return lines


def _set_rationale(cmd, filename, entry, json_output, looped, repin,
                   c_helper, c_fail, c_loop, c_filename, test_only_prose):
    argv = (("--output json " if json_output else "") + cmd
            + " --api browser --max-threads 0 --max-spawned-processes 0")
    origin = (
        f"Migrated from browser_{SET_STEM}.rs. "
        + (f"This `[[case]]` is one of the 12 invocations the looped fn "
           f"`{SET_LOOP_FN}` makes: it loops "
           f"`for extension in [\"ts\", \"jsx\", \"tsx\"]` (:{c_loop}) building a single "
           f"`format!(\"main.{{extension}}\")` filename (:{c_filename}) and then loops "
           f"`for (command, json_output)`, so this is exactly the "
           f"`({cmd}, {'json' if json_output else 'text'}, {filename})` one, split into a "
           f"named sibling per rule 5 rather than folded."
           if looped else
           "That source `#[test]` fn is a single unlooped helper call, so it maps 1:1 to "
           "this one `[[case]]` and keeps the fn's name verbatim (rule 6).")
    )
    return " ".join([
        origin,
        f"`assert_browser_harness_set_iteration(` (:{c_helper}) writes the Set-constructor "
        f"iteration probe to `{filename}` in a fresh temp dir and runs the real "
        f"`kali {argv}` with that dir as cwd and the browser harness command variable set to "
        f"`node`.",
        f"ASSERTION SHAPE: the source's ONLY process assertion in this file is "
        f"{c_fail} "
        f"-- no exit code, no stdout/stderr needle, no envelope field, in either output mode "
        f"-- so this case carries exactly `exit = \"failure\"` and nothing else. Adding a "
        f"diagnostic code or a stream claim would invent a claim the source never made "
        f"(rule 2), even though the real binary does emit one.",
        (f"NO [matrix] in this file: the looped fn uses one `main.<ext>` filename for BOTH "
         f"commands while the four individual fns use `main.js`/`smoke.test.js`, so a "
         f"file-wide `ext` axis could not express the filename pattern without inventing an "
         f"untested combination (see the file header's arithmetic)."),
        (f"The `[source]` key is `{entry}` rather than `{filename}`: the helper picks its "
         f"body from the command while the looped fn writes both bodies to the same "
         f"`main.<ext>` name, which one flat file-wide `[source]` key cannot carry. The "
         f"rename is argv-only and no fixture body in this file names any of these files by "
         f"string, so it does not rewrite the program under test (rule 9)."
         if entry != filename else
         f"The `[source]` key is `{filename}`, the source's own filename -- this one does "
         f"not collide, so it is not renamed."),
        f"RULE 12 -- the Rust comment prose of browser_{SET_STEM}.rs that this case's call "
        f"path reaches, carried verbatim: \"{repin}\""
        + (f" And, from the `test`-only fixture builder this case does reach: "
           f"\"{test_only_prose}\"" if test_only_prose else ""),
    ])


# =========================================================================
# SHAPE D -- browser_reflect_own_keys, the U10 submodule carrier, split into
#            TWO case files on MANIFEST PRESENCE (U2). The measured derivation
#            is in this module's docstring; the mechanical half is below.
# =========================================================================

RK = "reflect_own_keys"
RK_EXPLICIT = "reflect_own_keys_explicit_api"
RK_INHERITED = "reflect_own_keys_inherited_manifest"
RK_SUBS = ("run.rs", "build.rs", "check.rs", "test.rs")

# The 44 `#[test]` fns, one row each (rule 6: the case is the only remaining
# trace of the fn). `helper` is what the fn's body calls; everything the split
# turns on is DERIVED from that helper's own source below, never from this
# table and never from the fn's name.
RK_ROWS = []
for _ext in EXTS4:
    RK_ROWS += [
        dict(sub="run.rs", fn=f"run_supports_reflect_own_keys_in_{_ext}_input_when_browser_harness_is_configured",
             helper="assert_browser_requested_reflect_own_keys_fails_closed",
             cmd="run", entry=f"main.{_ext}", json=False, ext=_ext),
        dict(sub="run.rs", fn=f"json_run_supports_reflect_own_keys_in_{_ext}_input_when_browser_harness_is_configured",
             helper="assert_json_browser_requested_reflect_own_keys_fails_closed",
             cmd="run", entry=f"main.{_ext}", json=True, ext=_ext),
        dict(sub="run.rs", fn=f"run_supports_reflect_own_keys_in_{_ext}_input_when_browser_api_surface_is_inherited",
             helper="assert_inherited_browser_api_surface_reflect_own_keys_fails_closed",
             cmd="run", entry=f"main.{_ext}", json=False, ext=_ext),
        dict(sub="run.rs", fn=f"json_run_supports_reflect_own_keys_in_{_ext}_input_when_browser_api_surface_is_inherited",
             helper="assert_inherited_browser_api_surface_reflect_own_keys_fails_closed",
             cmd="run", entry=f"main.{_ext}", json=True, ext=_ext),
        dict(sub="test.rs", fn=f"test_supports_reflect_own_keys_in_{_ext}_input_when_browser_harness_is_configured",
             helper="assert_browser_requested_reflect_own_keys",
             cmd="test", entry=f"smoke.test.{_ext}", json=False, ext=_ext),
        dict(sub="test.rs", fn=f"json_test_supports_reflect_own_keys_in_{_ext}_input_when_browser_harness_is_configured",
             helper="assert_json_browser_requested_reflect_own_keys",
             cmd="test", entry=f"smoke.test.{_ext}", json=True, ext=_ext),
        dict(sub="test.rs", fn=f"test_supports_reflect_own_keys_in_{_ext}_input_when_browser_api_surface_is_inherited",
             helper="assert_inherited_browser_api_surface_reflect_own_keys",
             cmd="test", entry=f"smoke.test.{_ext}", json=False, ext=_ext),
        dict(sub="test.rs", fn=f"json_test_supports_reflect_own_keys_in_{_ext}_input_when_browser_api_surface_is_inherited",
             helper="assert_inherited_browser_api_surface_reflect_own_keys",
             cmd="test", entry=f"smoke.test.{_ext}", json=True, ext=_ext),
        dict(sub="build.rs", fn=f"build_emits_browser_bundle_reflect_own_keys_semantics_in_{_ext}_input",
             helper="assert_browser_bundle_reflect_own_keys",
             cmd="build", entry=f"app.{_ext}", json=False, ext=_ext),
        dict(sub="build.rs", fn=f"json_build_emits_browser_bundle_reflect_own_keys_semantics_in_{_ext}_input",
             helper="assert_browser_bundle_reflect_own_keys",
             cmd="build", entry=f"app.{_ext}", json=True, ext=_ext),
    ]
for _ext in ("jsx", "tsx"):
    RK_ROWS += [
        dict(sub="check.rs", fn=f"check_accepts_reflect_own_keys_in_{_ext}_input_on_browser_surface",
             helper="assert_browser_checked_reflect_own_keys",
             cmd="check", entry=f"main.{_ext}", json=False, ext=_ext),
        dict(sub="check.rs", fn=f"json_check_accepts_reflect_own_keys_in_{_ext}_input_on_browser_surface",
             helper="assert_browser_checked_reflect_own_keys",
             cmd="check", entry=f"main.{_ext}", json=True, ext=_ext),
    ]


def _fn_body(text, fn):
    """The brace-balanced body of `fn <fn>` in `text`."""
    import re
    m = re.search(r"\bfn\s+" + re.escape(fn) + r"\s*[(<]", text)
    if not m:
        raise AssertionError(f"no `fn {fn}` in this source")
    brace = text.find("{", m.end() - 1)
    depth, i = 0, brace
    while i < len(text):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    return text[brace:i + 1]


def _rk_submodules(carrier_path):
    """Every submodule's text as it stood PRE-TRIM, by name.

    After the U4 trim, `run.rs`/`build.rs`/`check.rs` are gone from disk, so
    `submodules.submodule_paths` resolves only the retained `test.rs`. The case
    files were migrated from the pre-trim tree and every `:N` in them is a
    pre-trim number (ruling 9), so this reads the missing three from the ref the
    RETAINED FILE ITSELF DECLARES -- the same rule `case_emit.source_text_at`
    follows, and for the same reason: a ref carried anywhere but the header is
    the moving figure ruling 11 forbids.

    Whatever IS still on disk is read from disk and cross-checked against the
    blob at that ref; a retained submodule that has been edited since the trim
    would make the generator emit citations into a file that no longer says what
    they point at, so a mismatch raises rather than being resolved silently
    (ruling 18).
    """
    from submodules import submodule_paths
    text = open(carrier_path).read()
    m = re.search(r"PRE-TRIM REF:\s*([0-9a-f]{40})\b", text)
    on_disk = {p.name: p.read_text() for p in submodule_paths(carrier_path)}
    if not m:
        return on_disk
    ref = m.group(1)
    out = {}
    for name in RK_SUBS:
        rel = f"crates/kali_cli/tests/browser_{RK}/{name}"
        blob = subprocess.run(["git", "show", f"{ref}:{rel}"], cwd=REPO,
                              capture_output=True, text=True)
        if blob.returncode != 0:
            raise AssertionError(
                f"{rel} is not readable at the declared PRE-TRIM REF {ref}: "
                f"{blob.stderr.strip()}")
        if name in on_disk and on_disk[name] != blob.stdout:
            raise AssertionError(
                f"{rel} is RETAINED but differs from its own pre-trim blob at {ref}. "
                "The case files' citations are pre-trim numbers; regenerating against an "
                "edited retained file would emit numbers that point at different code.")
        out[name] = blob.stdout
    return out


def _rk_read():
    """The carrier plus every `#[path]` submodule, and the census, all derived.

    U10 names the failure this closes: `grep -c '#[test]'` on the carrier
    returns 0 and would silently drop all 44 tests. The submodule paths come
    from `submodules.py`, which delegates to `audit-case-migration.py`'s own
    `resolve_path_mods` rather than re-implementing the resolution.

    RULING 18 -- THE SPLIT PROPERTY IS DERIVED, AND A NON-MATCH IS AN ERROR.
    `inherited` is decided by asking whether the helper a fn actually calls
    writes a `kali.json`, not by looking at the fn's name. The two are then
    cross-checked against each other, so a fn named `..._is_inherited` whose
    helper writes no manifest -- or the reverse -- fails the generator instead
    of being filed into the wrong half, where nothing downstream would catch
    it (that is exactly the U2 failure this split exists to prevent).
    """
    carrier_path = os.path.join(TESTS, f"browser_{RK}.rs")
    carrier = rs(RK)          # honours the header's own PRE-TRIM REF
    subs = _rk_submodules(carrier_path)
    if set(subs) != set(RK_SUBS):
        raise AssertionError(
            f"submodules resolved to {sorted(subs)}, expected {sorted(RK_SUBS)}")

    # Count real `#[test]` ATTRIBUTE LINES, not substring hits, and skip any
    # leading `//!` retention header: this carrier's own U3 header discusses
    # `#[test]` in prose, and a substring count read that as three extra tests.
    # Prose about the file is an input to any measurement of the file -- the
    # same self-reference ruling 11 exists for.
    def n_tests(text):
        out, lines = 0, text.split("\n")
        i = 0
        while i < len(lines) and lines[i].startswith("//!"):
            i += 1
        for line in lines[i:]:
            if line.strip() == "#[test]":
                out += 1
        return out

    counts = {n: n_tests(t) for n, t in subs.items()}
    if sum(counts.values()) != 44 or n_tests(carrier) != 0:
        raise AssertionError(f"submodule #[test] census {counts}, carrier "
                             f"{n_tests(carrier)}; expected 44 / 0")

    for row in RK_ROWS:
        sub_text = subs[row["sub"]]
        if f"fn {row['fn']}(" not in sub_text:
            raise AssertionError(f"{row['sub']} has no `fn {row['fn']}`")
        body = _fn_body(sub_text, row["fn"])
        if f"{row['helper']}(" not in body:
            raise AssertionError(
                f"{row['sub']}::{row['fn']} does not call `{row['helper']}`; "
                f"body is {body.strip()[:120]!r}")
        helper_text = sub_text if f"fn {row['helper']}(" in sub_text else carrier
        derived = 'join("kali.json")' in _fn_body(helper_text, row["helper"])
        named = "_is_inherited" in row["fn"]
        if derived != named:
            raise AssertionError(
                f"{row['sub']}::{row['fn']}: helper `{row['helper']}` "
                f"{'writes' if derived else 'does not write'} a kali.json, but the fn name "
                f"says {'inherited' if named else 'explicit'}. Derived and declared disagree; "
                "one of them would have put this case in the wrong half of the U2 split.")
        row["inherited"] = derived

    seen = [r["fn"] for r in RK_ROWS]
    if len(set(seen)) != 44:
        raise AssertionError("duplicate fn in the census table")
    total = sum(len([1 for i, l in enumerate(t.split("\n")) if l.strip() == "#[test]"])
                for t in subs.values())
    if total != len(RK_ROWS):
        raise AssertionError(
            f"the sources declare {total} `#[test]` fns but this table lists "
            f"{len(RK_ROWS)}; a fn exists that is migrated by nothing")
    return carrier, subs


def _rk_fixtures(carrier):
    run_body = check_captured("main.<ext>", C.CAP_REFLECT_RUN, carrier,
                              anchors=[("reflect ownKeys ok", "reflect ownKeys ok")],
                              must_contain="console.log")
    test_body = check_captured("smoke.test.<ext>", C.CAP_REFLECT_TEST, carrier,
                               anchors=[("Kali.test(", "Kali.test(")],
                               must_contain="Kali.test(")
    bundle_body = check_captured("app.<ext>", C.CAP_REFLECT_BUNDLE, carrier,
                                 anchors=[("kali-tree-shake: reflectOwnKeysSmoke",
                                           "kali-tree-shake: reflectOwnKeysSmoke")],
                                 must_contain="reflectOwnKeysSmoke")
    return run_body, test_body, bundle_body


def _rk_manifest(carrier, subs):
    """The `kali.json` text, and RULING 7's mandatory identity assertion.

    Two helpers write a manifest -- one in the carrier, one in `run.rs` -- and
    the two texts must be byte-identical for a single `[source]` entry to stand
    for both. That is ASSERTED here, not eyeballed, which is the mandatory half
    of controller ruling 7.
    """
    from case_emit import fixture_starting
    a = fixture_starting(carrier, "assert_inherited_browser_api_surface_reflect_own_keys",
                         '{\n  "schemaVersion"')
    b = fixture_starting(subs["run.rs"],
                         "assert_inherited_browser_api_surface_reflect_own_keys_fails_closed",
                         '{\n  "schemaVersion"')
    return P.assert_identical("kali.json written by the two inherited helpers", a, b)


def _rk_envelope(command, *, errors):
    """The `run|test --output json` envelope claims THESE helpers make.

    Written out rather than taken from `math_shapes.envelope_harness` because
    this source differs from that builder in one real place: it asserts the
    ENVELOPE-level `exitCode == 0` for `test` as well as for `run`
    (`assert_eq!(json["exitCode"], 0)` sits above the `if command == "run"`),
    while `envelope_harness` emits it only for `run`. Using the shared builder
    here would silently DROP a claim the source makes -- a rule-1 weakening
    that no gate reads the builder for.
    """
    payload = {"hostContract": "browser-requested", "runtimeBackend": "browser-harness"}
    if command == "run":
        payload["exitCode"] = 0
    else:
        payload.update({"total": 1, "passed": 1, "failed": 0})
    j = {"schemaVersion": 1, "command": command, "success": True, "exitCode": 0,
         "payload": payload, "stdout": "", "stderr": ""}
    if errors:
        j["errors"] = []
    return j


def _rk_steps(row, fixtures, harness_body):
    """One row's steps, keyed off the helper it actually calls."""
    h, entry, json_output = row["helper"], row["entry"], row["json"]
    env = {HARNESS_ENV: "node"}

    if h.endswith("_fails_closed"):
        argv = (["--output", "json"] if json_output else [])
        argv += [row["cmd"]]
        if not row["inherited"]:
            argv += ["--api", "browser"]
        argv += [entry]
        return [{"args": argv, "env": env, "exit": "failure"}]

    if h == "assert_browser_bundle_reflect_own_keys":
        return bundle_steps(entry, harness_body,
                            {"exit": "success", "stdout_contains": ["0"]},
                            json_output=json_output,
                            json_claims=envelope_build(errors=False) if json_output else None,
                            meta_fields=META)

    if h == "assert_browser_checked_reflect_own_keys":
        argv = ["check", "--api", "browser"] + (["--output", "json"] if json_output else [])
        step = {"args": argv + [entry], "exit": "success"}
        if json_output:
            step["json"] = {"schemaVersion": 1, "command": "check", "success": True,
                            "exitCode": 0, "payload": {"filesChecked": 1}, "errors": []}
        return [step]

    # The four `test`-command helpers on the success path.
    argv = (["--output", "json"] if json_output else []) + [row["cmd"]]
    if not row["inherited"]:
        argv += ["--api", "browser"]
    step = {"args": argv + [entry], "env": env, "exit": "success"}
    if json_output:
        # The explicit-half helper asserts `errors` is empty; the inherited-half
        # helper returns before that line and does not. Mirrored exactly.
        step["json"] = _rk_envelope(row["cmd"], errors=not row["inherited"])
    else:
        step["stdout_contains"] = ["ok 1"]
        if row["inherited"]:
            # Only the inherited text path asserts stderr is empty.
            step["stderr"] = ""
    return [step]


RK_U10 = [
    "U10 -- SUBMODULE CARRIER, AND THE TRAP IT EXISTS FOR.",
    f"`grep -c '#\\[test\\]'` on tests/browser_{RK}.rs returns 0 and would silently drop",
    "every test in this target: all 44 of them live behind a `#[path = \"...\"] mod`",
    "declaration, in 4 sibling files (build.rs, check.rs, run.rs, test.rs). The carrier",
    "holds only the three fixture builders and the five shared assert helpers.",
    "Citations into a submodule are written `<file>.rs:N` and are resolved against THAT",
    "file by batch5_crosscheck.py; a bare `:N` means the carrier.",
    f"The carrier and its sibling directory tests/browser_{RK}/ are RETAINED by this",
    "commit -- they are deleted together, by the family-wide operation after batch 8, not",
    "here.",
]


def _rk_u2(half, other_stem, here_fns, here_invocations):
    common = [
        "U2 -- `[source]` is FILE-WIDE, WHICH IS WHY THE MIGRATED HALF IS TWO CASE",
        "FILES AND NOT ONE.",
        "U10 says migrate a submodule carrier and its sibling directory into ONE `.toml`.",
        "That is wrong here and U2 takes precedence. Of the 28 `#[test]` fns migrated from",
        "this target, 8 -- all in `run.rs` -- go through a helper that writes a `kali.json`",
        "manifest and pass NO `--api` flag: the browser API surface is resolved from the",
        "manifest instead. The other 20 pass `--api browser` explicitly and run against a",
        "tree with NO manifest.",
        "`crates/kali_case_runner/src/expand.rs`'s `expand()` substitutes and clones the",
        "whole file-level `[source]` map into EVERY trial regardless of which case",
        "references which key, so one shared table would make `kali.json` unconditionally",
        "present.",
        "",
        "THE DISCRIMINATOR WAS RE-DERIVED AFTER THE U4 TRIM, AND IT MOVED. Before the trim",
        "it was `payload.hostContract` / `payload.runtimeBackend`, which flip from",
        "`kali-hosted`/`wasmtime` to `browser-requested`/`browser-harness` on manifest",
        "presence alone -- but those fields are asserted only by `test.rs`, which is now",
        "RETAINED, so that measurement no longer covers anything this file claims. Measured",
        "again against the claims that ARE migrated, the `build` cases now carry the",
        "discriminating power:",
        "    kali build --bundle app.js        (no manifest, no --api)  -> exit 5, no meta",
        "    kali build --bundle app.js        (kali.json present)      -> exit 0,",
        "                                       app/app.meta.json apiSurface = browser",
        "The 8 build cases assert `exit = \"success\"` and `apiSurface = \"browser\"` on that",
        "very file, so a leaked manifest would supply both and they would pass whether or",
        "not `--api browser` did anything. No literal is dropped by that leak, so",
        "audit-case-migration.py cannot see it; the trial still passes, so `cargo test`",
        "cannot either. That invisibility is exactly why U2 exists.",
        "For completeness, the same probe over the other two migrated commands shows NO",
        "difference -- `check main.jsx` exits 0 with `filesChecked = 1` either way, and",
        "`run main.js` exits 1 either way -- so the build cases are load-bearing on their",
        "own. One measured disarmament is enough to require the split.",
        "",
        "THE SPLIT IS ON MANIFEST PRESENCE, NOT ON THE SUBMODULE BOUNDARY. `run.rs`",
        "straddles it 8 explicit / 8 inherited, so one case file per submodule would put",
        "both halves of run.rs in one file and reproduce the disarmament. The generator",
        "DERIVES which half a fn belongs to by asking whether the helper it calls writes a",
        "`kali.json`, then cross-checks that against the fn's own name and raises on",
        "disagreement (ruling 18: derive the property, make a non-match an error) -- so a",
        "case cannot be filed into the wrong half silently.",
        f"THIS FILE carries {here_fns} of the 28 migrated `#[test]` fns ({here_invocations} trials).",
    ]
    if half == "explicit":
        return common + [
            "THIS FILE is the EXPLICIT-`--api browser` half: `kali.json` appears in its",
            f"`[source]` table NOWHERE. Its sibling is {other_stem}.toml.",
            "Within this file `[source]` is safe in the ordinary way: every fixture is written",
            "unconditionally by the source into a fresh temp dir, none is written behind an",
            "`if`, and every command names its entry explicitly on argv -- verified against",
            "the real binary rather than assumed, in a directory holding all 8 fixtures:",
            "`kali --output json check --api browser main.jsx` still reports",
            "`payload.filesChecked = 1`, so no sibling fixture is picked up by discovery and",
            "the unused ones are inert.",
        ]
    return common + [
        "THIS FILE is the MANIFEST-INHERITED half: `kali.json` IS in its `[source]` table",
        "and no argv below carries `--api`, so the browser API surface can only come from",
        f"the manifest. Its sibling is {other_stem}.toml, whose `[source]` deliberately",
        "holds no manifest at all -- see that file's header for why one shared table would",
        "silently disarm it.",
        "Within this file `[source]` is safe in the ordinary way: the manifest is written",
        "unconditionally by this half's single helper. NOTE that the OTHER manifest-writing",
        "helper in this target is reached only by the RETAINED `test.rs`, so ruling 7's",
        "duplicate-identity assertion is still run by this file's generator across both --",
        "the two texts are asserted byte-identical rather than eyeballed -- even though only",
        "one of them now produces a `[source]` entry.",
    ]


def _rk_cite(row, carrier, subs):
    """(fn citation, helper citation) for one row, both derived by search.

    A citation into a submodule is written `<file>.rs:N`; a bare `:N` means the
    carrier. Which one a helper gets is decided by where the helper actually
    is, not by which submodule the fn is in -- `run.rs` defines three local
    `_fails_closed` helpers of its own while the other five live in the
    carrier.
    """
    sub_text = subs[row["sub"]]
    # Each citation carries the construct it points at, so the gate can
    # re-resolve it (ruling 11: a pointer nothing re-resolves is a figure in
    # disguise). The construct is the literal this function searched for.
    # The snippet is `<name>(` with NO leading `fn `, which is the form batch
    # 6B's shipped submodule citations use and the only one the gate's needle
    # extractor gives a needle to -- a snippet containing a space yields none,
    # and the citation then "matches but yields NO NEEDLE", i.e. nothing
    # re-resolves it. Copied from the working precedent rather than invented.
    fn_cite = "`%s(` (%s:%d)" % (
        row["fn"], row["sub"], P.cite_line(sub_text, "fn " + row["fn"] + r"\("))
    helper = row["helper"]
    if "fn " + helper + "(" in sub_text:
        helper_cite = "`%s(` (%s:%d)" % (
            helper, row["sub"], P.cite_line(sub_text, "fn " + helper + r"\("))
    else:
        helper_cite = "`%s(` (:%d)" % (
            helper, P.cite_line(carrier, "fn " + helper + r"\("))
    return fn_cite, helper_cite


def _rk_rationale(row, carrier, subs, half, other_stem, repin_prose):
    fn_cite, helper_cite = _rk_cite(row, carrier, subs)
    h = row["helper"]
    json_output = row["json"]

    parts = [
        f"Migrated from browser_{RK}.rs, the `{row['fn']}` `#[test]` fn in its "
        f"`{row['sub'][:-3]}` `#[path]` submodule -- {fn_cite}. That fn is a single unlooped "
        f"helper call, so it maps 1:1 to this one `[[case]]` and keeps the fn's name "
        f"verbatim -- the case is the only remaining trace of the fn (rule 6)."
    ]

    if h.endswith("_fails_closed"):
        parts.append(
            f"{helper_cite} is a LOCAL, run-module-only variant of the carrier's "
            f"shared helper of the same name minus the suffix: it copies the command shape "
            f"and narrows the assertion to the honest fail-closed result. The source's only "
            f"process assertion on this path is `assert!(!output.status.success(), \"must "
            f"fail closed: {{output:?}}\")`, so this case carries exactly `exit = \"failure\"` "
            f"and nothing else. Adding a diagnostic code or a stream claim would invent a "
            f"claim the source never made (rule 2), even though the real binary does emit "
            f"one.")
    elif h == "assert_browser_bundle_reflect_own_keys":
        parts.append(
            f"{helper_cite} writes the tree-shakeable "
            f"`reflectOwnKeysSmoke` bundle fixture to `{row['entry']}`, builds it with "
            f"`kali build --bundle --api browser"
            f"{' --output json' if json_output else ''}`"
            + (", asserts the JSON envelope's schemaVersion/command/success/exitCode and "
               "payload(artifactKind, bundleFormat)"
               if json_output else "")
            + ", asserts the emitted `app/app.meta.json` metadata, then writes the "
              "browser-bundle harness and runs it under node.")
        parts.append(
            "UNLIKE the other bundle targets in this batch, BOTH processes succeed here: the "
            "source asserts `output.status.success()` on the harness too, and then "
            "`stdout.contains('0')`. So the `browser_bundle_harness` step carries "
            "`exit = \"success\"` and `stdout_contains = [\"0\"]` -- a plain `.contains` "
            "against a field that HAS a substring form, mirrored as `stdout_contains` rather "
            "than strengthened to an exact pin (ruling 3).")
        if json_output:
            parts.append(
                "This helper does NOT assert that the envelope's `errors` array is empty -- "
                "the other five bundle helpers in this batch do -- so no `errors` claim is "
                "carried here. Mirroring the source means mirroring what it omits.")
    elif h == "assert_browser_checked_reflect_own_keys":
        parts.append(
            f"{helper_cite} writes the run-mode probe to "
            f"`{row['entry']}` and runs `kali check --api browser"
            f"{' --output json' if json_output else ''}`, asserting the process succeeds"
            + (" and pinning the envelope's schemaVersion/command/success/exitCode, "
               "`payload.filesChecked = 1`, and an empty `errors` array."
               if json_output else
               ". In text mode that is the ONLY assertion the helper makes -- no stdout, no "
               "stderr, no envelope -- so this case carries exactly `exit = \"success\"`."))
        parts.append(
            "NOTE THE EXTENSION DOMAIN: `check` is exercised for `jsx` and `tsx` ONLY, never "
            "for `js` or `ts`. That is why this file declares no `[matrix]` -- see the "
            "header's arithmetic.")
    else:
        surface = ("the manifest (`kali.json`), with no `--api` flag on argv"
                   if row["inherited"] else "the explicit `--api browser` flag")
        parts.append(
            f"{helper_cite} writes the `Kali.test` probe to "
            f"`{row['entry']}` and runs `kali {'--output json ' if json_output else ''}"
            f"{row['cmd']}{'' if row['inherited'] else ' --api browser'} {row['entry']}` "
            f"with the browser harness command variable set to `node`; the browser API "
            f"surface comes from {surface}.")
        if json_output:
            parts.append(
                "The envelope claims are mirrored exactly: schemaVersion/command/success and "
                "the ENVELOPE-level `exitCode = 0` (the source asserts that one above its "
                "`if command == \"run\"`, for `test` as well as `run`), "
                "`payload.hostContract = \"browser-requested\"`, "
                "`payload.runtimeBackend = \"browser-harness\"`, "
                "`payload.total/passed/failed = 1/1/0`, `stdout = \"\"` and `stderr = \"\"`."
                + (" The explicit-half helper also asserts the `errors` array is empty, so "
                   "that claim is carried here."
                   if not row["inherited"] else
                   " The inherited-half helper RETURNS before the `errors` assertion, so no "
                   "`errors` claim is carried here -- mirroring the source means mirroring "
                   "where it stops."))
        else:
            parts.append(
                "The text-mode claim is `stdout.contains(\"ok 1\")`, a plain `.contains` "
                "against a field that HAS a substring form, so it is mirrored as "
                "`stdout_contains` and NOT strengthened to an exact pin (ruling 3)."
                + (" This helper additionally asserts `stderr` is exactly empty, which the "
                   "explicit-half helper does not, so `stderr = \"\"` is carried here."
                   if row["inherited"] else
                   " This helper makes no `stderr` claim, so none is carried -- the "
                   "inherited-half helper does, and that difference is preserved."))

    parts.append(
        f"U2 TWO-FILE SPLIT: this case is in the {'MANIFEST-INHERITED' if row['inherited'] else 'EXPLICIT-`--api browser`'} "
        f"half. "
        + ("`kali.json` is in this file's `[source]` table and no argv here carries `--api`, "
           "so the browser API surface can only come from the manifest."
           if row["inherited"] else
           "`kali.json` appears nowhere in this file's `[source]` table, which is exactly "
           "what keeps this case able to FAIL if `--api browser` regressed.")
        + f" The other half is in {other_stem}.toml. They cannot share a file: `expand()` "
          f"clones the whole file-level `[source]` map into every trial, so a shared "
          f"`kali.json` would supply the browser surface to the explicit cases too. "
          f"Measured on a claim this migration actually carries: `kali build --bundle "
          f"app.js` with no manifest and no `--api` exits 5 and emits no "
          f"`app/app.meta.json` at all, while the same command with `kali.json` present "
          f"exits 0 and writes `apiSurface = \"browser\"` into it -- so the 8 `build` "
          f"cases, which pin exactly those two, would pass whether or not `--api browser` "
          f"did anything. No literal is dropped by that leak, so the audit cannot see it, "
          f"and the trial still passes, so `cargo test` cannot either. (The "
          f"`payload.hostContract` measurement that justified this split before the U4 "
          f"trim is NOT cited here: those fields are asserted only by the retained "
          f"`test.rs`, so they no longer cover anything either file claims.)")

    if row["sub"] == "run.rs":
        parts.append(
            f"RULE 12 -- the Rust comment prose of the `run` submodule, which is the only "
            f"one of this target's five files that carries any, carried verbatim: "
            f"\"{repin_prose}\"")
    return " ".join(parts)


def _rk_assertion_shape(rows):
    """The file's claim inventory, DERIVED from the helper each row calls.

    reflect_own_keys is the only file in this batch whose cases do not all make
    the same shape of claim -- `run.rs` asserts a FAILURE and everything else a
    SUCCESS with pins -- so the block is built from the rows rather than
    written, and it raises on a helper it does not know rather than silently
    describing a file it has not read.
    """
    kinds = {}
    for r in rows:
        h = r["helper"]
        if h.endswith("_fails_closed"):
            k = ("`exit = \"failure\"` and nothing else -- the source's only process "
                 "assertion on this path is `assert!(!output.status.success())`, so a "
                 "diagnostic code or a stream needle would be a rule-2 invention")
        elif h == "assert_browser_bundle_reflect_own_keys":
            k = ("`exit = \"success\"` on the build, the `app/app.meta.json` "
                 "apiSurface/artifactKind pins, and -- unlike every other bundle target in "
                 "this batch -- `exit = \"success\"` on the harness too, plus "
                 "`stdout_contains = [\"0\"]`. No `errors` claim: this helper does not "
                 "make one")
        elif h == "assert_browser_checked_reflect_own_keys":
            k = ("`exit = \"success\"`, and in JSON mode the envelope's "
                 "schemaVersion/command/success/exitCode, `payload.filesChecked = 1` and an "
                 "empty `errors` array. In text mode that is the helper's ONLY assertion")
        else:
            k = ("`exit = \"success\"` plus `stdout_contains = [\"ok 1\"]` in text mode, "
                 "or the full envelope in JSON mode; the ENVELOPE-level `exitCode = 0` is "
                 "pinned for `test` as well as `run` because the source asserts it above "
                 "its `if command == \"run\"`")
        kinds.setdefault(k, []).append(r["fn"])
    if not kinds:
        raise AssertionError("no rows -- nothing to describe")
    out = ["ASSERTION SHAPE, mirrored from the source and nothing more. This file's cases",
           "do NOT all make the same shape of claim, so each is stated with its count:"]
    for k, fns in sorted(kinds.items(), key=lambda kv: -len(kv[1])):
        out.append(f"  * {len(fns)} case(s): {k}.")
    return out


def _rk_rule12(subs, blocks):
    start, body = blocks[0]
    # The citation carries the comment's own first line as its backticked
    # construct, so the gate has something to re-resolve; a comment block has no
    # CODE construct beside it (ruling 11).
    # THE CITATION POINTS AT CODE, NOT AT THE COMMENT. A comment line yields no
    # needle -- the gate strips comment text when extracting one -- so a
    # `(run.rs:3)` pointing at the block itself matches the reader and then
    # re-resolves nothing, which ruling 11 says is a figure in disguise. The
    # block's position is given instead as an offset from the first `fn` BELOW
    # it, which is a real construct the gate resolves, and the offset is derived
    # here rather than written down.
    lines = subs["run.rs"].split("\n")
    below = next(i for i in range(start + len(body) - 1, len(lines))
                 if lines[i].startswith("fn "))
    anchor = lines[below].split("(")[0].replace("fn ", "").strip()
    gap = (below + 1) - start
    return [
        "RULE 12 / U6 -- SOURCE COMMENT PROSE, CARRIED VERBATIM AND ATTRIBUTED BOTTOM-UP.",
        "The carrier and three of its four submodules carry NO Rust comment at all; the",
        f"whole target has exactly one block, opening {gap} lines above",
        f"`{anchor}(` (run.rs:{below + 1}) and running {len(body)} line(s), and it",
        "sits at module scope in `run.rs` above that file's three local `_fails_closed`",
        "helpers. It is therefore carried into the rationale of every case that came from",
        "run.rs and into no others -- which is per-helper attribution (U6), not pooling.",
        "`comment_coverage.py` has no per-helper attribution and will report those lines",
        "missing from the cases that did NOT come from run.rs. That is the checker's known",
        "limitation, recorded here rather than papered over by copying the prose into cases",
        "whose producing file never runs, which U6 forbids even though it would turn the",
        "checker green.",
        "The text is COPIED out of the `.rs` by this generator (its comment_blocks helper,",
        "named plainly rather than backticked: U8's gate resolves every backticked",
        "lower-case identifier against this source's own fn list, and that one lives in the",
        "generator), not retyped, so an em-dash cannot become `--`.",
    ]


def _rk_build(half):
    carrier, subs = _rk_read()
    run_body, test_body, bundle_body = _rk_fixtures(carrier)
    manifest = _rk_manifest(carrier, subs)
    stem = RK_EXPLICIT if half == "explicit" else RK_INHERITED
    other = RK_INHERITED if half == "explicit" else RK_EXPLICIT

    # U4 TRIM (controller ruling, batch 8A round 2): `test.rs` is RETAINED, so
    # its 16 fns are NOT migrated. What is left is run.rs (16), build.rs (8)
    # and check.rs (4) = 28, split by manifest presence into 8 inherited (all
    # from run.rs) and 20 explicit (run.rs 8 + build.rs 8 + check.rs 4).
    rows = [r for r in RK_ROWS
            if r["sub"] != "test.rs" and r["inherited"] == (half == "inherited")]
    expected = 8 if half == "inherited" else 20
    if len(rows) != expected:
        raise AssertionError(f"{half} half has {len(rows)} rows, expected {expected}")

    def source_comment_blocks(text):
        """`comment_blocks` minus a leading `//!` retention header.

        Rule 12 is about comments the SOURCE already had; a `//!` header is
        prose this migration added (U3), and `math_shapes.rule12_no_comments_
        prose` skips it for exactly that reason. Without the skip, adding the
        header to this carrier turned its own rule-12 check red.
        """
        blocks = comment_blocks(text)
        if blocks and blocks[0][0] == 1 and text.startswith("//!"):
            blocks = blocks[1:]
        return blocks

    blocks = source_comment_blocks(subs["run.rs"])
    if len(blocks) != 1:
        raise AssertionError(
            f"run.rs has {len(blocks)} comment block(s), this generator accounts for 1")
    for name, text in [(f"browser_{RK}.rs", carrier)] + [
            (n, t) for n, t in subs.items() if n != "run.rs"]:
        if source_comment_blocks(text):
            raise AssertionError(f"{name} has grown a Rust comment block; rule 12 unhandled")
    repin_prose = prose_of(blocks[0])

    harness_raw = check_program(
        "harness body",
        _harness_body(carrier, "assert_browser_bundle_reflect_own_keys",
                      "reflectOwnKeysSmoke"),
        must_contain="await import(")

    # The fixture set is exactly the entries THIS half's rows name, so an unused
    # fixture cannot ride along and a needed one cannot be missing.
    raw_source = {}
    for r in rows:
        raw_source[r["entry"]] = {
            "main": run_body, "smoke": test_body, "app": bundle_body,
        }[r["entry"].split(".")[0] if r["entry"].startswith("app") else
          ("smoke" if r["entry"].startswith("smoke") else "main")]
    raw_source = dict(sorted(raw_source.items()))
    if half == "inherited":
        raw_source["kali.json"] = manifest
    elif any(k == "kali.json" for k in raw_source):
        raise AssertionError("the explicit half must not carry a manifest")

    # THE HARNESS BODY BELONGS TO THE EXPLICIT HALF ONLY. `build.rs` is entirely
    # explicit, so the inherited half emits no `browser_bundle_harness` step at
    # all. Feeding the harness body through `_rule10` for BOTH halves made the
    # inherited file declare `[constants] dollar`, print the whole RULE 10
    # block, and carry an `EXTRA-OK: '$'` suppression -- for a file whose
    # fixtures contain no `${` anywhere. `_rule10`'s own docstring promises "a
    # file cannot declare the constant it does not need"; that is only true if
    # it is shown the bodies the file actually emits.
    emits_harness = any(r["helper"] == "assert_browser_bundle_reflect_own_keys"
                        for r in rows)
    if emits_harness != (half == "explicit"):
        raise AssertionError(
            f"the {half} half {'does' if emits_harness else 'does not'} emit a bundle "
            "harness step; the rule-10 input set is derived from that and the two disagree")
    bodies = dict(raw_source)
    if emits_harness:
        bodies["__harness__"] = harness_raw
    escaped, constants = _rule10(bodies)
    escaped_all = dict(escaped)          # the prose counts bindings over EVERY
    harness_body = escaped.pop("__harness__", None)   # emitted body, and the
    source = escaped                     # explicit half's only `${` is in the
                                         # harness body, not in `[source]`.

    if half == "explicit":
        matrix = None
        matrix_block = P.matrix_declined(
            test_fns=20, invocations=20, cases=20,
            reason=[
                "THE EXTENSION AXIS IS NOT UNIFORM ACROSS THIS FILE. The run and build",
                "submodules each cover all four of js/ts/jsx/tsx, but `check.rs` covers `jsx`",
                "and `tsx` ONLY -- there is no `check_accepts_reflect_own_keys_in_js_input` and",
                "no `..._in_ts_input`. A file-wide `ext(4)` axis would fan the four check cases",
                "to `js` and `ts` as well, manufacturing four `kali check --api browser main.js`",
                "/ `main.ts` trials the source never ran (a rule-2 invention).",
                "Nor can the axis be `ext(2)`: that would drop half of the run and build",
                "coverage, which is a rule-1 weakening.",
                "20 `#[test]` fns, each one unlooped: run.rs contributes 8, build.rs 8 and",
                "check.rs 4, and none of the three migrated submodules contains a loop, so",
                "20 fns = 20 invocations.",
            ])
    else:
        matrix = {"ext": EXTS4}
        matrix_block = P.matrix_arithmetic(
            test_fns=8, invocations=8,
            helpers=[
                ("assert_inherited_browser_api_surface_reflect_own_keys_fails_closed", 8,
                 "run.rs's 8 manifest-inheriting fns = json_output(2) x ext(4). It is the "
                 "ONLY manifest-writing helper in the migrated half -- the other one is "
                 "reached solely by the RETAINED test.rs"),
            ],
            cases=2, axis="ext", values=EXTS4, non_axes=("json_output",))

    header = hdr(
        f"Migrated from tests/browser_{RK}.rs and its `#[path]` submodule directory",
        f"tests/browser_{RK}/ -- the {'manifest-inherited' if half == 'inherited' else 'explicit-`--api browser`'} half of a two-file U2 split.",
        "",
        # SECTION ORDER IS FIXED BY `batch5_crosscheck.SECTIONS`: Migrated from /
        # RULE 12 / RULE 7 / RULE 6 / U2 / RULE 13 / ARGV ORDER / ASSERTION
        # SHAPE. U10, U5, RULE 10 and ruling 16 are not on that list and sit
        # between them.
        _rk_rule12(subs, blocks),
        "",
        RK_U10,
        "",
        matrix_block,
        "",
        (P.rule6_matrix_fold(
            "one `json_output` half of this file's 8 fns, fanned to the 4 extensions")
         if matrix else P.RULE6_ONE_TO_ONE),
        "",
        _rk_u2(half, other, len(rows), len(rows) * (4 if matrix else 1)),
        "",
        "U5 -- NO `[source]` KEY RENAME IS NEEDED. Each of this target's three program",
        "texts has its own filename stem in the source (`main.<ext>` for the run-mode probe,",
        "`smoke.test.<ext>` for the `Kali.test` probe, `app.<ext>` for the bundle fixture),",
        "so the flat file-wide namespace has no collision and every key below is the",
        "source's own filename. `check.rs` writes the SAME run-mode probe to the same",
        "`main.<ext>` names that `run.rs` uses, which is a shared entry rather than a",
        "collision -- the two texts are byte-identical because they are the same call to the",
        "same builder.",
        "",
        (rule10_prose(escaped_all) + [""] + RULE10_EXTRA_OK + [""]) if constants else None,
        P.rule13_header(
            ["assert_browser_requested_reflect_own_keys",
             "assert_json_browser_requested_reflect_own_keys",
             "assert_inherited_browser_api_surface_reflect_own_keys",
             "assert_browser_bundle_reflect_own_keys",
             "assert_browser_checked_reflect_own_keys",
             "reflect_own_keys_source", "reflect_own_keys_test_source",
             "browser_bundle_reflect_own_keys_source"],
            docs_carried=[RK_DOC] if half == "explicit" else [],
            runner_exemption=(half == "explicit")),
        ("" if half == "explicit" else
         "This half runs no `browser_bundle_harness` step -- every case is a single `kali "
         "run`/`kali test` invocation -- so ruling 6's runner-infrastructure paragraph is "
         "omitted rather than printed about a chain this file never reaches. It also never "
         "reaches `kali_common::reflect_own_keys_frozen_callable_source`, whose `///` doc "
         "IS carried in the sibling file: the manifest-inherited helpers write the same "
         "run-mode fixture, so the doc is carried here too where that fixture appears."),
        "",
        P.ARGV_ORDER,
        ("The `check` shape follows the build convention: `check --api browser "
         "[--output json] <entry>`, with the `--output json` pair appended AFTER the "
         "subcommand and its flags." if half == "explicit" else
         "No `build` or `check` invocation appears in this half, so only the run/test argv "
         "shape above applies here."),
        "",
        _rk_assertion_shape(rows),
        "",
        RULING16_NOTE,
    )

    cases, plans = [], []
    for r in rows:
        entry = r["entry"]
        if matrix:
            entry = entry.rsplit(".", 1)[0] + ".${ext}"
        row = dict(r, entry=entry)
        steps = _rk_steps(row, source, harness_body)
        cases.append({
            "name": r["fn"] if not matrix else
                    f"{'json_' if r['json'] else ''}{r['cmd']}_reflect_own_keys_inherited",
            "rationale": _rk_rationale(row, carrier, subs, half, other, repin_prose),
            "steps": steps,
        })
        plans.append({"kind": "reflect", "steps": steps, "source": source,
                      "constants": constants, "matrix": matrix})

    if matrix:
        # The 16 fns collapse to 4 cases; dedupe by (cmd, json).
        seen, deduped, dplans = set(), [], []
        for c, p, r in zip(cases, plans, rows):
            key = (r["cmd"], r["json"])
            if key in seen:
                continue
            seen.add(key)
            deduped.append(c)
            dplans.append(p)
        if len(deduped) != 2:
            raise AssertionError(f"matrix fold produced {len(deduped)} cases, expected 2")
        cases, plans = deduped, dplans

    VERIFY[stem] = plans
    return emit6a(header, constants, matrix, source, cases)


RK_DOC = "Canonical source text for the supported `Reflect.ownKeys` frozen callable aliases."


# NOT REGISTERED, AND THE REASON IS A CONTROLLER RULING, NOT AN OMISSION.
#
# `browser_reflect_own_keys` is a CLASS A (claim from unreachable code) §5.11
# retention. Its three shared success helpers each carry an `if command ==
# "run"` branch asserting `stdout.contains("reflect ownKeys ok")`, and NO live
# `#[test]` fn reaches it -- PR #16 rev2's honest re-pin moved every `run`
# caller into `run.rs`'s own local `_fails_closed` helpers, leaving the shared
# helpers with 16 callers that all pass `"test"`:
#
#     $ cd crates/kali_cli/tests && grep -rn \
#     >   'assert_browser_requested_reflect_own_keys(\|assert_json_browser_requested_reflect_own_keys(\|assert_inherited_browser_api_surface_reflect_own_keys(' \
#     >   browser_reflect_own_keys/ | grep -v _fails_closed | grep -c '"test"'
#     16          # of 16 call sites; none passes "run"
#
# So `audit-case-migration.py` reports `[contains literals] 'reflect ownKeys
# ok'` absent from the case files, and it cannot be satisfied honestly: no live
# test asserts it (rule 2 forbids inventing it), the `run` cases fail closed and
# emit no such stdout (so the claim would also be false), and `[source]` is
# excluded from the audit's case-side search by design (U8), so the fixture's
# own trailing `console.log('reflect ownKeys ok')` cannot discharge it either.
#
# Ledger ruling R1 (progress.md:1645-1648, and its Class A/B consequence at
# :1653): "Unreachable-code claims -> the target stays HAND-WRITTEN per the
# plan's existing spec 5.11 escape hatch, and its .toml is deleted. Explicitly
# REJECTED: adding a per-file audit exception mechanism ..., and teaching the
# script Rust reachability analysis." Five files in this family already sit on
# that keep list for the same shape (progress.md:1679-1681).
#
# The two builders below are KEPT AND KEPT WORKING because the disposition is
# ESCALATED, not closed: U4 says whole-file retention is legitimate only when
# EVERY test reaches the construct, and here ZERO do, which is a real tension
# between U4 and R1 that only the controller can settle. If the ruling comes
# back "trim test.rs and migrate the other 28", these two functions are the
# migration and the U2 derivation they encode is unaffected. Registering them
# would ship a red audit, which rule 3 forbids absolutely.
#
#     python3 gen_batch8a.py --reflect-preview     # renders both, writes nothing
@target(RK_EXPLICIT)
def build_rk_explicit():
    return _rk_build("explicit")


@target(RK_INHERITED)
def build_rk_inherited():
    return _rk_build("inherited")

if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
