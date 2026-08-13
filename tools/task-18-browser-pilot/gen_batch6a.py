#!/usr/bin/env python3
"""Generate the Task 18 batch 6A case files (15 migrated targets).

Batch 6A migrates 16 hand-written targets. Fifteen of them get a case file,
generated here; the sixteenth,
`browser_non_literal_dynamic_import_harness_jsx_tsx.rs`, is a whole-file design
spec 5.11 retention and has no case file by construction.

WHY A GENERATOR AND NOT FIFTEEN HAND-WRITTEN FILES. Batch 4 shipped
cross-file prose divergence that every per-file gate passed individually, and
batch 5's answer was to put every recurring sentence in `batch5_prose` and call
it rather than retype it. That is followed here without exception: this module
writes only the PER-FILE spec -- the program under test, the invocation
arithmetic, the assertion inventory and the `:N` citations -- which is what
review has to read.

CITATIONS. Every `:N` below is produced by `batch5_prose.cite_line(rs_text,
regex)` at generation time, by SEARCHING the source for the construct. None is
computed by arithmetic and none is carried over from an earlier measurement.
`cite_line` raises unless its anchor matches exactly the expected number of
times, so a vanished or ambiguous anchor is a generator error rather than a
silently wrong number.

RULE 8 / RULE 9. Six of these sources build a fixture with `format!`, with
a `replace` call, or one level removed inside `kali_common`. None of those texts
is hand-derived: they are the byte-exact output of executing the real code, and
they live in `batch6a_captures.py`, whose docstring records the exact capture
procedure. `check_captured` re-checks each one against its own `.rs` before it
is emitted, so a capture taken before a source edit fails the generator instead
of shipping a program that is no longer the program under test.

RULE 10. `browser_math_sqrt_cbrt_bundle.rs`'s browser-bundle harness body
contains a GENUINE JS template literal. `expand.rs`'s `substitute()` hard-fails
on any `${...}` it cannot resolve, and a step `body` IS substituted
(`expand.rs:148`), so that file declares `[constants] dollar = "$"` and spells
the genuine `${` as `${dollar}{`. The resolved program text is unchanged.

U9. Every exact pin is live-captured from the real built `kali` via
`kali_run.py`, for EVERY cell of the file's matrix axis (and for both commands
where the file has two), and `batch5_prose.assert_identical` asserts the cells
agree with each other AND with the embedded constant before one pin is emitted.
See `_pin`.

Run: python3 gen_batch6a.py [name ...]   (no args = all)
"""

import json as _json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")
KALI_COMMON_NUMBER = os.path.join(REPO, "crates/kali_common/src/number.rs")

from case_emit import emit, fixture_in_fn, fixture_starting, write, source_text  # noqa: E402
from math_shapes import (  # noqa: E402
    META, bundle_steps, envelope_build, envelope_harness, harness_step,
)
import batch5_prose as P  # noqa: E402
import batch6a_captures as C  # noqa: E402

EXTS4 = ["js", "ts", "jsx", "tsx"]
HARNESS_ENV = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"
# ^ the value of `kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV`, read from
# crates/kali_runtime_contract/src/browser/contract.rs rather than assumed: five
# of these sources pass the constant and five spell the literal, and the
# migrated `env` must resolve to the same variable either way.

REGISTRY = {}


def target(name):
    def deco(fn):
        REGISTRY[name] = fn
        return fn
    return deco


def rs(name):
    """The source a case file is generated FROM.

    For a U4 trim-and-keep target that is NOT the working-tree file. Once the
    trim lands, the working tree holds only the retained half, so regenerating
    from it would silently emit a different (and much smaller) case file -- and
    on batch 6A's one trimmed target it does not even get that far: the anchor
    `errors.iter().all(` then matches twice, once in the retained code and once
    in the retention header's own prose. The generator therefore reads the
    PRE-TRIM blob, and it takes the ref from the retained file's own
    `PRE-TRIM REF:` line rather than from a constant here -- a ref carried
    anywhere but the header is the moving figure ruling 11 forbids.
    BATCH 8C: the body below was a fourth hand-rolled copy of this resolution
    (working tree, then `PRE-TRIM REF:` blob) and it read the working tree
    FIRST, so the family deletion turned it into a `FileNotFoundError`.
    `case_emit.source_text` applies the same rule, plus the family-deletion
    fallback, and is the one the sweep and `reword_ungated_citations` already
    share -- so a trimmed target cannot be read one way here and another way
    there.
    """
    return source_text(name)


def hdr(*chunks):
    """Flatten header chunks (str or list[str]) into one list of `#` lines."""
    out = []
    for chunk in chunks:
        if chunk is None:
            continue
        pieces = [chunk] if isinstance(chunk, str) else list(chunk)
        for piece in pieces:
            out.extend(str(piece).split("\n"))
    return out


def para(*chunks):
    return " ".join(c.strip() for c in chunks if c)


def check_program(label, body, *, must_contain="console.log"):
    """Guard the wrong-literal-extraction bug class at generation time.

    A fixture pulled from the wrong place still produces a parseable case file
    (batch 4 shipped `"app.${ext}" = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"`
    once). Anything written into `[source]` or a harness `body` must look like
    the program it claims to be before it is emitted.
    """
    if must_contain not in body:
        raise AssertionError(f"fixture {label!r} does not look like a program: {body[:80]!r}")
    return body


def check_captured(label, body, rs_text, *, anchors, must_contain="console.log"):
    """A captured fixture must still line up with the code that produced it.

    The capture is the only thing standing between rule 8/9 and a hand-typed
    approximation, and a STALE capture (taken before a source edit) reproduces
    the OLD program while `check_fixtures.py` stays green, because the template
    it compares against is the same template. So the generator requires every
    named anchor to be present both in the producing source and in the captured
    text. An anchor is a PAIR, `(as it is spelled in the producing Rust source,
    as it must appear in the resolved text)`, because those two spellings
    genuinely differ -- a `concat!`/`format!` template writes `\\n` and `{{`
    where the resolved program has a real newline and a single brace. Spelling
    both halves is what makes the check a real comparison rather than a
    restatement of one side.
    """
    check_program(label, body, must_contain=must_contain)
    for in_source, in_capture in anchors:
        if in_source not in rs_text:
            raise AssertionError(
                f"{label}: anchor {in_source[:40]!r} is not in the producing source -- "
                "the anchor is stale, so this check proves nothing")
        if in_capture not in body:
            raise AssertionError(
                f"{label}: captured text is missing {in_capture[:40]!r} -- the capture is "
                "stale or came from the wrong source")
    return body


def comment_blocks(rs_text):
    """Every contiguous run of Rust `//` comment lines, as (first_line, [texts]).

    Rule 12 requires the text COPIED, not retyped -- an em-dash retyped as `--`
    is a violation the mechanical checker catches -- so every comment carried
    into a rationale below is pulled out of the `.rs` here rather than written
    into this module.

    A `//` inside a JS fixture body (`// kali-tree-shake: ...`) is program text,
    not Rust prose, so string-literal spans are excluded by OFFSET, using
    `lexer.find_string_literals`. An earlier version of this function masked the
    text with `strip_block_comments_and_strings` and zipped the two line lists;
    that silently dropped whole comment blocks, because the masker collapses the
    newlines inside a multi-line string and its output therefore has FEWER lines
    than the input. It found 2 of 3 blocks in one of this batch's files and 0 of
    1 in two others -- a wrong answer that looked like a right one, which is why
    this now works in offsets and assumes nothing about line alignment.
    """
    from lexer import find_string_literals
    spans = [(lit["start"], lit["end"]) for lit in find_string_literals(rs_text)]

    def in_literal(offset):
        return any(a <= offset < b for a, b in spans)

    out, cur, start = [], [], None
    offset = 0
    for i, line in enumerate(rs_text.split("\n")):
        stripped = line.lstrip()
        is_comment = (stripped.startswith("//")
                      and not in_literal(offset + (len(line) - len(stripped))))
        if is_comment:
            if not cur:
                start = i + 1
            cur.append(re.sub(r"^\s*///?!?\s?", "", line).rstrip())
        elif cur:
            out.append((start, cur))
            cur = []
        offset += len(line) + 1
    if cur:
        out.append((start, cur))
    return out


def kali_common_doc(fn_name, path=KALI_COMMON_NUMBER):
    """The `///` doc immediately above a `kali_common` fn, EXTRACTED not retyped."""
    text = open(path).read()
    m = re.search(r"///([^\n]*)\n(?:pub )?(?:const )?fn " + re.escape(fn_name) + r"\b", text)
    if not m:
        raise AssertionError(f"no `///` doc immediately above `fn {fn_name}` in {path}")
    return m.group(1).strip()


def rs_doc(rs_text, fn_name):
    """The `///` doc block immediately above a fn in the source `.rs` itself."""
    m = re.search(r"((?:^///[^\n]*\n)+)(?:pub )?fn " + re.escape(fn_name) + r"\b",
                  rs_text, re.M)
    if not m:
        raise AssertionError(f"no `///` doc immediately above `fn {fn_name}`")
    return [re.sub(r"^///\s?", "", ln) for ln in m.group(1).rstrip("\n").split("\n")]


# --------------------------------------------------------------------------
# U9 live capture.
# --------------------------------------------------------------------------

def _pin(label, embedded, cells):
    """Re-capture an exact pin from the real binary for every cell, and assert
    every cell agrees with every other AND with the embedded constant.

    `assert_identical` over N copies of one constant would prove nothing; the
    capture is what makes the assertion real. Skipped LOUDLY if the built
    binary is absent, rather than reporting a green that was never run.
    """
    from kali_run import KALI, run_kali
    if not os.path.exists(KALI):
        print(f"  !! {KALI} absent -- pin {label} NOT re-captured this run")
        return embedded
    captured = []
    for entry, program, command, thread_flags in cells:
        args = ["--output", "json", command, "--api", "browser"]
        if thread_flags:
            args += ["--max-threads", "0", "--max-spawned-processes", "0"]
        args += [entry]
        code, out, err, _dir = run_kali({entry: program}, args,
                                        env={HARNESS_ENV: "node"})
        if code != 0:
            raise AssertionError(f"live capture failed for {label} {entry}: {err!r}")
        captured.append(_json.loads(out)["stdout"])
    return P.assert_identical(f"{label}, live-captured over {len(cells)} cell(s), "
                              "against the embedded constant", embedded, *captured)


# --------------------------------------------------------------------------
# Shared header chunks that are NOT in batch5_prose because they are new here.
# Anything used by two or more files in this batch lives here, once, for the
# same reason batch5_prose exists.
# --------------------------------------------------------------------------

# HOISTED to batch5_prose in batch 6B, on its third call site; re-exported
# here so this generator's call sites are unchanged.
assert_rename_is_argv_only = P.assert_rename_is_argv_only


def extra_ok_renames(pairs, exts):
    """`EXTRA-OK` lines for U5-renamed entry filenames, one per expanded cell.

    `check_extra_claims.py` expands `[matrix]` before comparing, so a renamed
    `main_plain.${ext}` is checked as `main_plain.js`, `main_plain.ts`, ... and
    each expanded name needs its own declaration.
    """
    out = []
    for stem in pairs:
        for ext in exts:
            out.append(P.extra_ok(f"{stem}.{ext}", P.EXTRA_OK_U5_RENAME))
    return out


FORMAT_BUILT_FILENAME = (
    "the entry filename, which this source builds with `format!(\"main.{extension}\")` "
    "inside a loop rather than spelling as a literal, so the expanded name appears nowhere "
    "in the .rs even though the source really does pass exactly this argv token"
)


FAIL_CLOSED_NOTE = [
    "ASSERTION SHAPE -- THIS TARGET ASSERTS A FAILURE, NOT A SUCCESS. The source's only",
    "process assertion is `assert!(!output.status.success(), ...)`, so the migrated step",
    "carries `exit = \"failure\"` and nothing else. That is the whole claim; adding a",
    "diagnostic-code or stdout claim the source never made would be a rule-2 invention,",
    "and `exit = \"failure\"` is exactly as strong as the assertion it replaces.",
]


def stale_name_note(fns, what):
    """Controller ruling 8: a source fn whose NAME misdescribes its own body."""
    return [
        "MIGRATION NOTE (controller ruling 8) -- THE SOURCE FN NAMES ARE STALE.",
        f"Every `#[test]` fn in this source is named for what it was once thought to prove, but {what}",
        "The names are carried into the case names below unchanged and are NOT corrected:",
        "a fn name is not a comment so U7 does not literally apply, the `.rs` files are",
        "deleted wholesale after batch 8, and editing a source invalidates every audit run",
        "against its pre-trim blob. The discrepancy is recorded here so the case file",
        "preserves what the source actually did.",
        f"Affected: all {fns} `#[test]` fns.",
    ]


COMMENT_COVERAGE_MULTI_HELPER = [
    "U6 / comment_coverage.py -- A KNOWN FALSE `missing`, DOCUMENTED RATHER THAN PAPERED OVER.",
    "This source carries several DISTINCT comment blocks, one per assert helper, each",
    "reaching a disjoint subset of the cases below. U6 is explicit that a comment belongs",
    "in the rationale of exactly the cases its producing helper reaches, and that copying",
    "every block into every case to make the checker green is over-attribution and is",
    "FORBIDDEN even though it turns the checker green. `comment_coverage.py` has no",
    "per-helper attribution (its own module docstring records the limit), so it reports",
    "each block as missing from the cases it correctly does not appear in. That report is",
    "the checker's gap, not this file's: the attribution below is per helper, bottom-up.",
]


# ==========================================================================
# F1. browser_math_sqrt_cbrt_bundle.rs -- 8 fns / 8 invocations, [matrix] ext.
# ==========================================================================

@target("math_sqrt_cbrt_bundle")
def gen_math_sqrt_cbrt_bundle():
    stem = "math_sqrt_cbrt_bundle"
    text = rs(stem)
    helper = "assert_browser_bundle_math_sqrt_cbrt"

    c_build_exit, c_harness_exit = P.cite_line(
        text, r"output\.status\.success\(\)", label="status.success", expect=2)
    c_env_first = P.cite_line(text, r'assert_eq!\(envelope\["schemaVersion"\]')
    c_env_bundle_format = P.cite_line(text, r'assert_eq!\(payload\["bundleFormat"\]')
    c_env_errors = P.cite_line(text, r'assert!\(envelope\["errors"\]')
    c_env_is_empty = P.cite_line(text, r"\.is_empty\(\)\)")
    c_meta_api = P.cite_line(text, r'assert_eq!\(metadata\["apiSurface"\]')
    c_meta_kind = P.cite_line(text, r'assert_eq!\(metadata\["artifactKind"\]')
    c_two = P.cite_line(text, r'stdout\.contains\("2\\n"\)')
    c_minus_three = P.cite_line(text, r'stdout\.contains\("-3\\n"\)')
    c_zero_char = P.cite_line(text, r"stdout\.contains\('0'\)")
    c_template = P.cite_line(text, r"unexpected result ")

    program = check_program("app.${ext}", fixture_in_fn(
        text, "browser_bundle_math_sqrt_cbrt_source"))
    raw_body = check_program(
        "harness body",
        fixture_starting(text, helper, "const mod = await import("),
        must_contain="await import(")
    if "${result}" not in raw_body:
        raise AssertionError("the rule-10 template literal has left this fixture")
    body = raw_body.replace("${", "${dollar}{")

    rule10 = [
        "RULE 10 -- A GENUINE JS TEMPLATE LITERAL, ESCAPED THROUGH `[constants]`.",
        f"The browser-bundle harness body (:{c_template}) interpolates the JS binding the",
        "fixture itself declares into an Error message, with a real JS template literal.",
        "`expand.rs`'s `substitute()` hard-fails on any `${...}` it cannot resolve, and a",
        "step `body` IS substituted, so this file declares `[constants] dollar = \"$\"` and",
        "spells the genuine `${` as `${dollar}{`. The RESOLVED program text is byte-identical",
        "to the source's -- this is an encoding of rule 9, not an exception to it.",
        "This is the first pair in the family to need it while its `.rs` still exists, so",
        "note what `check_fixtures.py` does with it: the stored body differs from the `.rs`",
        "literal by exactly that escape, so the gate falls through to its `format!`-segment",
        "arm and matches on the segments either side of the placeholder. It exits 0, and its",
        "\"format!-built (segments matched)\" label is the only inaccuracy -- reported, not",
        "worked around.",
    ]

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the `kali build` process (:{c_build_exit}) and on the",
        f"browser-bundle harness process (:{c_harness_exit}).",
        "In json mode, the build envelope's schemaVersion/command/success/exitCode and the",
        f"payload's artifactKind/bundleFormat (:{c_env_first}-{c_env_bundle_format}), AND an",
        f"empty `errors` array (:{c_env_errors}-{c_env_is_empty}).",
        f"The emitted app/app.meta.json apiSurface/artifactKind (:{c_meta_api}-{c_meta_kind})",
        "is asserted in BOTH modes, because the source reads that file outside the",
        "`if json_output` block.",
        f"The harness step carries this file's THREE stdout claims: `.contains(\"2\\n\")`",
        f"(:{c_two}), `.contains(\"-3\\n\")` (:{c_minus_three}) and `.contains('0')`",
        f"(:{c_zero_char}). The third is a CHAR literal, which",
        "`audit-case-migration.py`'s `.contains` extractor does not see (its literal pattern",
        "requires double quotes), so nothing would have reported it missing -- it is carried",
        "anyway, because rule 1 is about the source's claims and not about what the audit",
        "can see.",
        "There is no count claim in this file, so no `stdout_count`; no stderr claim on",
        "either process; and the build envelope's stdout leaf is never read, so no",
        "`json.stdout` pin.",
        "The source passes no --max-threads / --max-spawned-processes arguments, so neither",
        "appears on argv.",
    ]

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        P.extra_ok("0", "the source's `.contains('0')` CHAR literal, carried per rule 1; "
                        "`check_extra_claims.py` accepts it because the character does occur "
                        "in the .rs, but the audit's own extractor never saw it"),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=8, invocations=8, cases=2, axis="ext", values=EXTS4,
            non_axes=("json_output",),
            helpers=[(helper, 8,
                      "ext(js/ts/jsx/tsx) x json_output(false/true), a complete cross\n"
                      "    product. Every `#[test]` fn is one unlooped call and the file\n"
                      "    contains no loop at all.")],
        ),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["app.${ext}"]),
        "",
        rule10,
        "",
        P.rule13_header(["kali_bin", "browser_bundle_math_sqrt_cbrt_source", helper]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    prog_desc = (
        f"`{helper}` builds a browser bundle with `kali build --bundle --api browser`, "
        "asserts the emitted app/app.meta.json metadata, then runs the bundle glue under "
        "the browser-bundle-harness contract backed by node, against a program that "
        "computes Math.sqrt(4) and Math.cbrt(-27) -- so its two console.log calls print 2 "
        "and -3 -- and whose harness body additionally calls the exported function with "
        "two BigInt arguments and prints String(result), which is 0."
    )
    claims = (
        f"Its three claims about that output are separate source lines: `.contains(\"2\\n\")` "
        f"(:{c_two}), `.contains(\"-3\\n\")` (:{c_minus_three}) and the char-literal "
        f"`.contains('0')` (:{c_zero_char}); all three are carried."
    )
    ruling3 = P.ruling3_substring()
    envelope_sentence = (
        "This sibling additionally asserts the build JSON envelope -- schemaVersion/command/"
        "success/exitCode, payload artifactKind/bundleFormat, and the empty `errors` array "
        f"the source checks at :{c_env_errors}-{c_env_is_empty} -- rather than plain text; "
        "output shape is not a matrix axis because it changes the assertion shape, so it is "
        "a separate case."
    )

    asserts = {"stdout_contains": ["2\n", "-3\n", "0"]}
    cases = [
        {
            "name": "build_emits_math_sqrt_and_cbrt_zero_identities",
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four "
                "`build_emits_math_sqrt_and_cbrt_zero_identities_in_*_input` fns (one per "
                "extension).", prog_desc, claims, ruling3),
            "steps": bundle_steps("app.${ext}", body, asserts,
                                  json_output=False, meta_fields=META),
        },
        {
            "name": "json_build_emits_math_sqrt_and_cbrt_zero_identities",
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four "
                "`json_build_emits_math_sqrt_and_cbrt_zero_identities_in_*_input` fns (one "
                "per extension).", prog_desc, claims, ruling3, envelope_sentence),
            "steps": bundle_steps("app.${ext}", body, asserts,
                                  json_output=True,
                                  json_claims=envelope_build(errors=True),
                                  meta_fields=META),
        },
    ]
    return emit6a(header, {"dollar": "$"}, {"ext": EXTS4},
                  {"app.${ext}": program}, cases)


# ==========================================================================
# F2. browser_math_sqrt_cbrt_frozen_aliases.rs -- 16 fns / 16 invocations.
#     [matrix] DECLINED: the two helpers cover DIFFERENT extension sets.
# ==========================================================================

@target("math_sqrt_cbrt_frozen_aliases")
def gen_math_sqrt_cbrt_frozen_aliases():
    stem = "math_sqrt_cbrt_frozen_aliases"
    text = rs(stem)
    bundle_helper = "assert_browser_bundle_math_sqrt_cbrt_frozen_aliases"
    harness_helper = "assert_browser_harness_math_sqrt_cbrt_frozen_aliases"

    c_build_exit, c_harness_proc_exit, c_cli_exit = P.cite_line(
        text, r"output\.status\.success\(\)", label="status.success", expect=3)
    c_env_first = P.cite_line(text, r'assert_eq!\(envelope\["schemaVersion"\]')
    c_env_bundle_format = P.cite_line(text, r'assert_eq!\(payload\["bundleFormat"\]')
    c_env_errors = P.cite_line(text, r'assert!\(envelope\["errors"\]')
    c_meta_api = P.cite_line(text, r'assert_eq!\(metadata\["apiSurface"\]')
    c_meta_kind = P.cite_line(text, r'assert_eq!\(metadata\["artifactKind"\]')
    c_bundle_contains, c_json_contains, c_text_contains = P.cite_line(
        text, r'contains\("2\\n2', label="the long stdout needle", expect=3)
    c_json_first = P.cite_line(text, r'assert_eq!\(json\["schemaVersion"\]')
    c_json_backend = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["runtimeBackend"\]')
    c_json_exit = P.cite_line(text, r'assert_eq!\(json\["exitCode"\]')
    c_json_skipped = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["skipped"\]')
    c_json_stderr = P.cite_line(text, r'assert_eq!\(json\["stderr"\]')
    c_json_errors = P.cite_line(text, r'assert!\(json\["errors"\]')
    c_ok1 = P.cite_line(text, r'stdout\.contains\("ok 1"\)')

    bundle_program = check_program("app.<ext>", fixture_in_fn(
        text, "browser_bundle_math_sqrt_cbrt_frozen_aliases_source"))
    run_program = check_program("main.<ext>", fixture_in_fn(
        text, "browser_harness_math_sqrt_cbrt_frozen_aliases_run_source"))
    test_program = check_program("smoke.test.<ext>", fixture_in_fn(
        text, "browser_harness_math_sqrt_cbrt_frozen_aliases_test_source"))
    body = check_program("harness body", fixture_starting(
        text, bundle_helper, "const mod = await import("), must_contain="await import(")
    needle = _long_needle(text, '2\n2\n')

    pin = _pin("math_sqrt_cbrt_frozen_aliases json.stdout",
               "2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n2\n-3\n-3\n-3\n-3\n-3\n-3\n-3\n-3\n-3\n-3\n",
               [(f"main.{e}", run_program, "run", False) for e in ("js", "ts")]
               + [(f"smoke.test.{e}", test_program, "test", False) for e in ("js", "ts")])

    decline = [
        "The two helpers in this file cover DIFFERENT extension sets. The bundle helper",
        f"`{bundle_helper}` runs over js/ts/jsx/tsx;",
        f"the harness helper `{harness_helper}` runs over js/ts ONLY",
        "-- there is no jsx or tsx harness test here at all.",
        "So no single file-wide `ext` axis closes: a 4-value axis fans the 8 harness",
        "invocations into 16 and invents the jsx/tsx harness runs the source never made",
        "(rule 2), and a 2-value axis drops the bundle's jsx and tsx runs outright (rule 1).",
    ]

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        "BUNDLE helper: `exit = \"success\"` on the `kali build` process",
        f"(:{c_build_exit}) and on the browser-bundle harness process",
        f"(:{c_harness_proc_exit}); in json mode the envelope's",
        f"schemaVersion/command/success/exitCode and payload artifactKind/bundleFormat",
        f"(:{c_env_first}-{c_env_bundle_format}) plus the empty `errors` array",
        f"(:{c_env_errors}); the emitted app/app.meta.json apiSurface/artifactKind",
        f"(:{c_meta_api}-{c_meta_kind}) in BOTH modes, because the source reads that file",
        f"outside the `if json_output`; then the harness step's single long",
        f"`.contains(...)` claim (:{c_bundle_contains}), which stays `stdout_contains`.",
        f"HARNESS helper: `exit = \"success\"` (:{c_cli_exit}); the environment carries",
        f"{HARNESS_ENV}=node, spelled as a string literal in this source rather than",
        "through the `kali_runtime_contract` constant; it passes no --max-threads /",
        "--max-spawned-processes arguments, so neither appears on argv.",
        f"json mode carries schemaVersion/command/success/payload(hostContract,",
        f"runtimeBackend) (:{c_json_first}-{c_json_backend}), plus `exitCode` at both levels",
        f"for `run` (:{c_json_exit}) or payload total/passed/failed/skipped for `test`",
        f"(:{c_json_skipped} is the last of the four), then the exact `json.stdout` pin",
        f"standing for the `.contains(...)` claim at :{c_json_contains}, `stderr = \"\"`",
        f"(:{c_json_stderr}) and the empty `errors` array (:{c_json_errors}).",
        f"Text mode carries the same long needle against raw stdout (:{c_text_contains}) and,",
        f"for `test` only, `.contains(\"ok 1\")` (:{c_ok1}).",
        "This file uses no count key: the source makes no `.matches(...).count()` claim.",
    ]

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        P.extra_ok(pin, P.EXTRA_OK_JSON_STDOUT),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_declined(test_fns=16, invocations=16, cases=16, reason=decline),
        "",
        P.RULE6_ONE_TO_ONE,
        "",
        P.u2_source_file_wide(
            ["app.js", "app.ts", "app.jsx", "app.tsx", "main.js", "main.ts",
             "smoke.test.js", "smoke.test.ts"]),
        "",
        P.RULING7_NO_HOIST,
        "",
        P.rule13_header([
            "kali_bin", "browser_bundle_math_sqrt_cbrt_frozen_aliases_source",
            bundle_helper,
            "browser_harness_math_sqrt_cbrt_frozen_aliases_run_source",
            "browser_harness_math_sqrt_cbrt_frozen_aliases_test_source",
            harness_helper]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    bundle_desc = (
        f"`{bundle_helper}` builds a browser bundle with `kali build --bundle --api "
        "browser`, asserts the emitted app/app.meta.json metadata, then runs the bundle "
        "glue under the browser-bundle-harness contract backed by node, against a program "
        "that calls Math.sqrt(4) through eleven frozen/bracketed/parenthesized alias forms "
        "and Math.cbrt(-27) through ten, so stdout is eleven 2s followed by ten -3s."
    )
    harness_desc = (
        f"`{harness_helper}` writes the same alias slice as a plain program and runs `kali "
        "run` (or a Kali.test wrapper under `kali test`) with the browser harness backed "
        "by node."
    )
    cases = []
    for ext in EXTS4:
        for jo in (False, True):
            name = f"{'json_' if jo else ''}build_emits_math_sqrt_and_cbrt_frozen_aliases_in_{ext}_input"
            cases.append({
                "name": name,
                "rationale": para(
                    f"Migrated from browser_{stem}.rs, `{name}` -- one source `#[test]` fn, "
                    "one case (no `[matrix]` in this file; see the header).",
                    bundle_desc,
                    f"Its single stdout claim is the long `.contains(...)` needle at "
                    f":{c_bundle_contains}.",
                    P.ruling3_substring(),
                    ("This sibling additionally asserts the build JSON envelope rather than "
                     "plain text; output shape is not a matrix axis because it changes the "
                     "assertion shape, so it is a separate case.") if jo else None),
                "steps": bundle_steps(
                    f"app.{ext}", body, {"stdout_contains": [needle]},
                    json_output=jo, json_claims=envelope_build(errors=True),
                    meta_fields=META),
            })
    for command, entry_stem, program in (("run", "main", run_program),
                                         ("test", "smoke.test", test_program)):
        for ext in ("ts", "js"):
            for jo in (False, True):
                name = (f"{'json_' if jo else ''}{command}_supports_math_sqrt_and_cbrt_"
                        f"frozen_aliases_when_browser_harness_is_configured_in_{ext}_input")
                asserts = {"stdout_contains": [needle]}
                if command == "test":
                    asserts["stdout_contains"] = [needle, "ok 1"]
                if jo:
                    asserts = {}
                cases.append({
                    "name": name,
                    "rationale": para(
                        f"Migrated from browser_{stem}.rs, `{name}` -- one source `#[test]` "
                        "fn, one case.",
                        harness_desc,
                        (P.ruling3_json_leaf() if jo else
                         f"Its stdout claim is the long `.contains(...)` needle at "
                         f":{c_text_contains}. " + P.ruling3_substring()),
                        (f"For `test` the source also asserts `.contains(\"ok 1\")` "
                         f"(:{c_ok1}), which the json branch does not make."
                         if command == "test" and not jo else None)),
                    "steps": [harness_step(
                        command, f"{entry_stem}.{ext}", json_output=jo, asserts=asserts,
                        json_claims=_harness_json_with_stdout(
                            command, pin,
                            extra_payload={"skipped": 0} if command == "test" else None,
                            stderr=True, errors=True),
                        env_var=HARNESS_ENV)],
                })
    source = {}
    for ext in EXTS4:
        source[f"app.{ext}"] = bundle_program
    for ext in ("js", "ts"):
        source[f"main.{ext}"] = run_program
    for ext in ("js", "ts"):
        source[f"smoke.test.{ext}"] = test_program
    P.assert_identical("the four app.<ext> bundle fixtures",
                       *[source[f"app.{e}"] for e in EXTS4])
    P.assert_identical("the two main.<ext> run fixtures",
                       *[source[f"main.{e}"] for e in ("js", "ts")])
    P.assert_identical("the two smoke.test.<ext> fixtures",
                       *[source[f"smoke.test.{e}"] for e in ("js", "ts")])
    return emit(header, None, source, cases)


# ==========================================================================
# F3. browser_math_sqrt_cbrt_global_this_root.rs -- 24 fns / 24 invocations.
# ==========================================================================

@target("math_sqrt_cbrt_global_this_root")
def gen_math_sqrt_cbrt_global_this_root():
    stem = "math_sqrt_cbrt_global_this_root"
    text = rs(stem)
    bundle_helper = "assert_browser_bundle_global_this_math_sqrt_cbrt"
    harness_helper = "assert_browser_harness_global_this_math_sqrt_cbrt"

    c_build_exit, c_harness_proc_exit, c_cli_exit = P.cite_line(
        text, r"output\.status\.success\(\)", label="status.success", expect=3)
    c_env_first = P.cite_line(text, r'assert_eq!\(envelope\["schemaVersion"\]')
    c_env_bundle_format = P.cite_line(text, r'assert_eq!\(payload\["bundleFormat"\]')
    c_meta_api = P.cite_line(text, r'assert_eq!\(metadata\["apiSurface"\]')
    c_meta_kind = P.cite_line(text, r'assert_eq!\(metadata\["artifactKind"\]')
    c_bundle_two = P.cite_line(text, r'stdout\.contains\("2\\n"\)', expect=2)[0]
    c_text_two = P.cite_line(text, r'stdout\.contains\("2\\n"\)', expect=2)[1]
    c_text_minus = P.cite_line(text, r'stdout\.contains\("-3\\n"\)', expect=2)[1]
    c_json_first = P.cite_line(text, r'assert_eq!\(json\["schemaVersion"\]')
    c_json_backend = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["runtimeBackend"\]')
    c_json_exit = P.cite_line(text, r'assert_eq!\(json\["exitCode"\]')
    c_json_failed = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["failed"\]')
    c_json_stdout = P.cite_line(text, r'json\["stdout"\]', expect=2)[0]
    c_json_stderr = P.cite_line(text, r'assert_eq!\(json\["stderr"\]')
    c_threads = P.cite_line(text, r'\.arg\("--max-threads"\)')
    c_procs = P.cite_line(text, r'\.arg\("--max-spawned-processes"\)')
    c_env_const = P.cite_line(text, r"BROWSER_HARNESS_COMMAND_ENV")

    bundle_program = check_program("app.${ext}", fixture_in_fn(
        text, "browser_bundle_global_this_math_sqrt_cbrt_source"))
    run_program = check_program("main.${ext}", fixture_in_fn(
        text, "browser_harness_global_this_math_sqrt_cbrt_run_source"))
    test_program = check_program("smoke.test.${ext}", fixture_in_fn(
        text, "browser_harness_global_this_math_sqrt_cbrt_test_source"))
    body = check_program("harness body", fixture_starting(
        text, bundle_helper, "const mod = await import("), must_contain="await import(")

    pin = _pin("math_sqrt_cbrt_global_this_root json.stdout", "2\n-3\n",
               [(f"main.{e}", run_program, "run", True) for e in EXTS4]
               + [(f"smoke.test.{e}", test_program, "test", True) for e in EXTS4])

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"BUNDLE helper: `exit = \"success\"` on the build (:{c_build_exit}) and on the",
        f"harness process (:{c_harness_proc_exit}); in json mode the envelope's",
        f"schemaVersion/command/success/exitCode and payload artifactKind/bundleFormat",
        f"(:{c_env_first}-{c_env_bundle_format}) -- this source makes NO `errors` claim on",
        "the build envelope, so no `errors = []` is written, unlike",
        "browser_math_sqrt_cbrt_bundle.rs migrated alongside it in this batch; the emitted",
        f"app/app.meta.json apiSurface/artifactKind (:{c_meta_api}-{c_meta_kind}) in BOTH",
        "modes, because the source reads that file outside the `if json_output`; then the",
        f"harness step's two plain `.contains` claims (:{c_bundle_two} and the line after",
        "it), which stay `stdout_contains`.",
        f"HARNESS helper: `exit = \"success\"` (:{c_cli_exit}); the argv carries",
        f"`--max-threads 0` (:{c_threads}) and `--max-spawned-processes 0` (:{c_procs}), and",
        f"the environment carries the variable named by",
        f"`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` (:{c_env_const}), set to node.",
        f"json mode carries schemaVersion/command/success/payload(hostContract,",
        f"runtimeBackend) (:{c_json_first}-{c_json_backend}), plus `exitCode` at both levels",
        f"for `run` (:{c_json_exit}) or payload total/passed/failed for `test`",
        f"(:{c_json_failed} is the last of the three) -- the source asserts NO `skipped` and",
        "NO `errors` on this envelope, so neither is written; then the exact `json.stdout`",
        f"pin standing for the two `.contains` claims against that leaf (:{c_json_stdout} is",
        f"the first) and `stderr = \"\"` (:{c_json_stderr}).",
        f"Text mode carries the two raw-stdout claims (:{c_text_two}, :{c_text_minus}) and",
        "nothing else -- there is no `ok 1` claim anywhere in this file.",
        "This file uses no count key: the source makes no `.matches(...).count()` claim.",
    ]

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        P.extra_ok(pin, P.EXTRA_OK_JSON_STDOUT),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=24, invocations=24, cases=6, axis="ext", values=EXTS4,
            helpers=[
                (bundle_helper, 8,
                 "ext(js/ts/jsx/tsx) x json_output(false/true)"),
                (harness_helper, 16,
                 "command(run/test) x ext(4) x json_output(false/true)"),
            ]),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["app.${ext}", "main.${ext}", "smoke.test.${ext}"]),
        "",
        P.rule13_header([
            "kali_bin", "browser_bundle_global_this_math_sqrt_cbrt_source",
            "browser_harness_global_this_math_sqrt_cbrt_run_source",
            "browser_harness_global_this_math_sqrt_cbrt_test_source",
            bundle_helper, harness_helper]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    bundle_desc = (
        f"`{bundle_helper}` builds a browser bundle with `kali build --bundle --api "
        "browser`, asserts the emitted app/app.meta.json metadata, then runs the bundle "
        "glue under the browser-bundle-harness contract backed by node, against a program "
        "that calls globalThis.Math.sqrt(4) and globalThis.Math.cbrt(-27), so its two "
        "console.log calls print 2 and -3."
    )
    harness_desc = (
        f"`{harness_helper}` writes the same globalThis-rooted slice as a plain program and "
        "runs `kali run` (or a Kali.test wrapper under `kali test`) with the browser "
        "harness backed by node and both concurrency limits pinned to 0."
    )

    cases = []
    for jo in (False, True):
        name = f"{'json_' if jo else ''}build_emits_global_this_math_sqrt_and_cbrt_zero_identities"
        cases.append({
            "name": name,
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four `{name}_in_*_input` fns (one "
                "per extension).",
                bundle_desc,
                f"Its two stdout claims are separate source lines, `.contains(\"2\\n\")` "
                f"(:{c_bundle_two}) and `.contains(\"-3\\n\")` on the line after it.",
                P.ruling3_substring(),
                ("This sibling additionally asserts the build JSON envelope rather than "
                 "plain text; output shape is not a matrix axis because it changes the "
                 "assertion shape, so it is a separate case.") if jo else None),
            "steps": bundle_steps("app.${ext}", body,
                                  {"stdout_contains": ["2\n", "-3\n"]},
                                  json_output=jo, json_claims=envelope_build(errors=False),
                                  meta_fields=META),
        })
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        for jo in (False, True):
            name = (f"{command}_supports_global_this_math_sqrt_and_cbrt_zero_identities_"
                    f"when_browser_harness_is_configured"
                    f"{'_in_json' if jo else ''}")
            cases.append({
                "name": name,
                "rationale": para(
                    f"Migrated from browser_{stem}.rs, the four "
                    f"`{name}_*_input` fns (one per extension).",
                    harness_desc,
                    (P.ruling3_json_leaf() if jo else
                     f"Its two stdout claims are separate source lines (:{c_text_two}, "
                     f":{c_text_minus}). " + P.ruling3_substring())),
                "steps": [harness_step(
                    command, entry, json_output=jo,
                    asserts={} if jo else {"stdout_contains": ["2\n", "-3\n"]},
                    json_claims=_harness_json_with_stdout(command, pin, stderr=True),
                    thread_flags=True, env_var=HARNESS_ENV)],
            })
    return emit(header, {"ext": EXTS4},
                {"app.${ext}": bundle_program,
                 "main.${ext}": run_program,
                 "smoke.test.${ext}": test_program}, cases)


# ==========================================================================
# F4. browser_math_sqrt_cbrt_harness.rs -- 16 fns / 16 invocations.
# ==========================================================================

@target("math_sqrt_cbrt_harness")
def gen_math_sqrt_cbrt_harness():
    stem = "math_sqrt_cbrt_harness"
    text = rs(stem)
    helper = "assert_browser_harness_math_sqrt_cbrt"

    c_exit = P.cite_line(text, r"output\.status\.success\(\)")
    c_json_first = P.cite_line(text, r'assert_eq!\(json\["schemaVersion"\]')
    c_json_backend = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["runtimeBackend"\]')
    c_json_exit = P.cite_line(text, r'assert_eq!\(json\["exitCode"\]')
    c_json_skipped = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["skipped"\]')
    c_json_contains, c_text_contains = P.cite_line(
        text, r'contains\("2\\n-3"\)', label="the 2/-3 needle", expect=2)
    c_json_stderr = P.cite_line(text, r'assert_eq!\(json\["stderr"\]')
    c_json_errors = P.cite_line(text, r'assert!\(json\["errors"\]')
    c_ok1 = P.cite_line(text, r'stdout\.contains\("ok 1"\)')
    c_env = P.cite_line(text, rf'\.env\("{HARNESS_ENV}"')

    run_program = check_program("main.${ext}", fixture_in_fn(
        text, "browser_harness_math_sqrt_cbrt_run_source"))
    test_program = check_program("smoke.test.${ext}", fixture_in_fn(
        text, "browser_harness_math_sqrt_cbrt_test_source"))

    pin = _pin("math_sqrt_cbrt_harness json.stdout", "2\n-3\n",
               [(f"main.{e}", run_program, "run", False) for e in EXTS4]
               + [(f"smoke.test.{e}", test_program, "test", False) for e in EXTS4])

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the one `kali` process (:{c_exit}); the environment carries",
        f"{HARNESS_ENV}=node (:{c_env}), spelled as a string literal in this source rather",
        "than through the `kali_runtime_contract` constant. The source passes no",
        "--max-threads / --max-spawned-processes arguments, so neither appears on argv.",
        f"json mode carries schemaVersion/command/success/payload(hostContract,",
        f"runtimeBackend) (:{c_json_first}-{c_json_backend}), plus `exitCode` at both levels",
        f"for `run` (:{c_json_exit}) or payload total/passed/failed/skipped for `test`",
        f"(:{c_json_skipped} is the last of the four), then the exact `json.stdout` pin",
        f"standing for the `.contains(\"2\\n-3\")` claim at :{c_json_contains}, `stderr = \"\"`",
        f"(:{c_json_stderr}) and the empty `errors` array (:{c_json_errors}).",
        f"Text mode carries the same needle against raw stdout (:{c_text_contains}) and, for",
        f"`test` only, `.contains(\"ok 1\")` (:{c_ok1}).",
        "This file uses no count key: the source makes no `.matches(...).count()` claim.",
    ]

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        P.extra_ok(pin, P.EXTRA_OK_JSON_STDOUT),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[(helper, 16, "command(run/test) x ext(js/ts/jsx/tsx) x\n"
                                  "    json_output(false/true), a complete cross product")]),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        "",
        P.rule13_header([
            "kali_bin", "browser_harness_math_sqrt_cbrt_run_source",
            "browser_harness_math_sqrt_cbrt_test_source", helper]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    desc = (
        f"`{helper}` writes a program calling Math.sqrt(4) and Math.cbrt(-27) (or the same "
        "two calls inside a Kali.test wrapper) and runs `kali run` / `kali test --api "
        "browser` with the browser harness backed by node, so stdout carries 2 then -3."
    )
    cases = []
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        for jo in (False, True):
            name = (f"{'json_' if jo else ''}{command}_supports_math_sqrt_and_cbrt_literal_"
                    "identities_when_browser_harness_is_configured")
            asserts = {}
            if not jo:
                asserts["stdout_contains"] = ["2\n-3"] + (["ok 1"] if command == "test" else [])
            cases.append({
                "name": name,
                "rationale": para(
                    f"Migrated from browser_{stem}.rs, the four `{name}_in_*_input` fns "
                    "(one per extension).",
                    desc,
                    (P.ruling3_json_leaf() if jo else
                     f"Its stdout claim is `.contains(\"2\\n-3\")` (:{c_text_contains}). "
                     + P.ruling3_substring()),
                    (f"For `test` the source also asserts `.contains(\"ok 1\")` (:{c_ok1}), "
                     "which the json branch does not make."
                     if command == "test" and not jo else None)),
                "steps": [harness_step(
                    command, entry, json_output=jo, asserts=asserts,
                    json_claims=_harness_json_with_stdout(
                        command, pin,
                        extra_payload={"skipped": 0} if command == "test" else None,
                        stderr=True, errors=True),
                    env_var=HARNESS_ENV)],
            })
    return emit(header, {"ext": EXTS4},
                {"main.${ext}": run_program, "smoke.test.${ext}": test_program}, cases)


# ==========================================================================
# F5. browser_math_tan_zero_identities.rs -- 8 fns / 8 invocations, ext(ts/js).
# ==========================================================================

@target("math_tan_zero_identities")
def gen_math_tan_zero_identities():
    stem = "math_tan_zero_identities"
    text = rs(stem)
    helper = "assert_browser_harness_math_tan"
    exts = ["ts", "js"]

    c_exit = P.cite_line(text, r"output\.status\.success\(\)")
    c_json_first = P.cite_line(text, r'assert_eq!\(json\["schemaVersion"\]')
    c_json_backend = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["runtimeBackend"\]')
    c_json_exit = P.cite_line(text, r'assert_eq!\(json\["exitCode"\]')
    c_json_failed = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["failed"\]')
    c_json_contains, c_text_contains = P.cite_line(
        text, r'contains\("0\\n"\)', label="the 0 needle", expect=2)
    c_json_stderr = P.cite_line(text, r'assert_eq!\(json\["stderr"\]')
    c_env = P.cite_line(text, rf'\.env\("{HARNESS_ENV}"')

    run_program = check_program("main.${ext}", fixture_in_fn(
        text, "browser_harness_math_tan_run_source"))
    test_program = check_program("smoke.test.${ext}", fixture_in_fn(
        text, "browser_harness_math_tan_test_source"))

    pin = _pin("math_tan_zero_identities json.stdout", "0\n",
               [(f"main.{e}", run_program, "run", False) for e in exts]
               + [(f"smoke.test.{e}", test_program, "test", False) for e in exts])

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the one `kali` process (:{c_exit}); the environment carries",
        f"{HARNESS_ENV}=node (:{c_env}). The source passes no --max-threads /",
        "--max-spawned-processes arguments, so neither appears on argv.",
        f"json mode carries schemaVersion/command/success/payload(hostContract,",
        f"runtimeBackend) (:{c_json_first}-{c_json_backend}), plus `exitCode` at both levels",
        f"for `run` (:{c_json_exit}) or payload total/passed/failed for `test`",
        f"(:{c_json_failed} is the last of the three) -- the source asserts NO `skipped` and",
        "NO `errors` on this envelope, so neither is written; then the exact `json.stdout`",
        f"pin standing for the `.contains(\"0\\n\")` claim at :{c_json_contains} and",
        f"`stderr = \"\"` (:{c_json_stderr}).",
        f"Text mode carries the same needle against raw stdout (:{c_text_contains}) and",
        "nothing else -- there is no `ok 1` claim anywhere in this file, on either branch.",
        "This file uses no count key: the source makes no `.matches(...).count()` claim.",
    ]

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=8, invocations=8, cases=4, axis="ext", values=exts,
            helpers=[(helper, 8, "command(run/test) x ext(ts/js) x\n"
                                 "    json_output(false/true), a complete cross product")]),
        "THE AXIS HAS TWO VALUES, NOT FOUR. This source runs ts and js only; it declares no",
        "jsx or tsx test at all, and the axis is the set the source actually exercises",
        "rather than the js/ts/jsx/tsx set most of this family uses. A four-value axis here",
        "would invent eight runs the source never made (rule 2).",
        "",
        P.rule6_matrix_fold("2 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        "",
        P.rule13_header([
            "kali_bin", "browser_harness_math_tan_run_source",
            "browser_harness_math_tan_test_source", helper]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    desc = (
        f"`{helper}` writes a program computing Math.tan(0) from a const-bound zero (or the "
        "same call inside a Kali.test wrapper) and runs `kali run` / `kali test --api "
        "browser` with the browser harness backed by node, so stdout carries a single 0."
    )
    cases = []
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        for jo in (False, True):
            name = (f"{'json_' if jo else ''}{command}_supports_math_tan_zero_identity_"
                    "when_browser_harness_is_configured")
            cases.append({
                "name": name,
                "rationale": para(
                    f"Migrated from browser_{stem}.rs, the two `{name}_in_*_input` fns (one "
                    "per extension).",
                    desc,
                    (P.ruling3_json_leaf() if jo else
                     f"Its stdout claim is `.contains(\"0\\n\")` (:{c_text_contains}). "
                     + P.ruling3_substring())),
                "steps": [harness_step(
                    command, entry, json_output=jo,
                    asserts={} if jo else {"stdout_contains": ["0\n"]},
                    json_claims=_harness_json_with_stdout(command, pin, stderr=True),
                    env_var=HARNESS_ENV)],
            })
    return emit(header, {"ext": exts},
                {"main.${ext}": run_program, "smoke.test.${ext}": test_program}, cases)


# ==========================================================================
# F8. browser_nullish_coalescing_harness.rs -- 4 fns / 12 invocations,
#     ext(ts/jsx/tsx).
# ==========================================================================

@target("nullish_coalescing_harness")
def gen_nullish_coalescing_harness():
    stem = "nullish_coalescing_harness"
    text = rs(stem)
    helper = "assert_browser_harness_nullish_coalescing"
    exts = ["ts", "jsx", "tsx"]

    c_exit = P.cite_line(text, r"output\.status\.success\(\)")
    c_code = P.cite_line(text, r"assert_eq!\(output\.status\.code\(\)")
    c_json_first = P.cite_line(text, r'assert_eq!\(json\["schemaVersion"\]')
    c_json_success = P.cite_line(text, r'assert_eq!\(json\["success"\]')
    c_json_errors = P.cite_line(text, r'assert!\(json\["errors"\]')
    c_threads = P.cite_line(text, r'\.arg\("--max-threads"\)')
    c_procs = P.cite_line(text, r'\.arg\("--max-spawned-processes"\)')
    c_env_const = P.cite_line(text, r"BROWSER_HARNESS_COMMAND_ENV")

    run_program = check_program("main.${ext}", fixture_in_fn(text, "browser_run_source"))
    test_program = check_program("smoke.test.${ext}", fixture_in_fn(
        text, "browser_test_source"), must_contain="Kali.test")

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more, and this file's shape",
        "is unusual: IT MAKES NO STDOUT CLAIM AT ALL.",
        f"The source asserts the process succeeded (:{c_exit}) and then, separately, that its",
        f"exit code is exactly `Some(0)` (:{c_code}). The exact form is the stronger of the",
        "two and implies the other, so the migrated step carries `exit = 0` rather than",
        "`exit = \"success\"` -- ruling 3's mirror-the-source direction, an exact source",
        "assertion becoming an exact pin.",
        f"In json mode it carries schemaVersion/command/success (:{c_json_first}-",
        f"{c_json_success}) and the empty `errors` array (:{c_json_errors}). It reads NO",
        "payload field, NO stdout leaf and NO stderr leaf, so none is written -- the program",
        "prints 3 and the source never looks at it. Inventing a stdout claim here because",
        "the output is knowable is exactly the rule-2 invention the `extra` gate exists for.",
        f"The argv carries `--max-threads 0` (:{c_threads}) and `--max-spawned-processes 0`",
        f"(:{c_procs}), and the environment carries the variable named by",
        f"`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` (:{c_env_const}), set to node.",
    ]

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=4, invocations=12, cases=4, axis="ext", values=exts,
            helpers=[(helper, 12, "command(run/test) x ext(ts/jsx/tsx) x\n"
                                  "    json_output(false/true). Each of the 4 `#[test]` fns\n"
                                  "    is a `for filename in [...]` loop over three\n"
                                  "    filenames, so 4 fns make 12 invocations")]),
        "THE AXIS HAS THREE VALUES, NOT FOUR. This source loops over ts, jsx and tsx only;",
        "it declares no js test at all. A four-value axis would invent four runs the source",
        "never made (rule 2).",
        "",
        P.rule6_matrix_fold("one source `#[test]` fn, whose own `for filename in [...]` loop\n"
                            "over three filenames is what the three `ext` cells reproduce"),
        "",
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        "",
        P.rule13_header(["kali_bin", "parse_json_stdout", "browser_run_source",
                         "browser_test_source", helper]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    desc = (
        f"`{helper}` writes a program whose two nullish-coalescing expressions bind `null ?? "
        "1` and `void 0 ?? 2` (or the same two inside a Kali.test wrapper) and runs `kali "
        "run` / `kali test --api browser` with the browser harness backed by node and both "
        "concurrency limits pinned to 0."
    )
    cases = []
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        for jo in (False, True):
            name = (f"{'json_' if jo else ''}{command}_supports_nullish_coalescing_in_"
                    "browser_api_surface_with_harness_input_matrix")
            step = harness_step(command, entry, json_output=jo, asserts={},
                                json_claims={"schemaVersion": 1, "command": command,
                                             "success": True, "errors": []},
                                thread_flags=True, env_var=HARNESS_ENV)
            step["exit"] = 0
            cases.append({
                "name": name,
                "rationale": para(
                    f"Migrated from browser_{stem}.rs, `{name}` -- one source `#[test]` fn "
                    "whose `for filename in [...]` loop is reproduced by this file's three "
                    "`ext` cells.",
                    desc,
                    "This case makes no stdout claim because the source makes none: it "
                    "asserts the process succeeded and that its exit code is exactly 0, and "
                    "nothing about what the program printed.",
                    ("On the json branch it additionally reads the envelope's "
                     "schemaVersion/command/success and asserts the `errors` array is "
                     "empty; output shape is not a matrix axis because it changes the "
                     "assertion shape, so it is a separate case." if jo else None)),
                "steps": [step],
            })
    return emit(header, {"ext": exts},
                {"main.${ext}": run_program, "smoke.test.${ext}": test_program}, cases)


# ==========================================================================
# F9. browser_number_predicates_bundle.rs -- 8 fns / 8 invocations.
#     [matrix] DECLINED: the fixture BODY differs by extension.
# ==========================================================================

@target("number_predicates_bundle")
def gen_number_predicates_bundle():
    stem = "number_predicates_bundle"
    text = rs(stem)
    common = open(KALI_COMMON_NUMBER).read()
    helper = "assert_browser_bundle_number_predicates"

    c_build_exit, c_harness_exit = P.cite_line(
        text, r"output\.status\.success\(\)", label="status.success", expect=2)
    c_env_first = P.cite_line(text, r'assert_eq!\(envelope\["schemaVersion"\]')
    c_env_bundle_format = P.cite_line(text, r'assert_eq!\(payload\["bundleFormat"\]')
    c_env_errors = P.cite_line(text, r'assert!\(envelope\["errors"\]')
    c_meta_api = P.cite_line(text, r'assert_eq!\(metadata\["apiSurface"\]')
    c_meta_kind = P.cite_line(text, r'assert_eq!\(metadata\["artifactKind"\]')
    c_ok = P.cite_line(text, r'stdout\.contains\("browser number predicates ok"\)')
    c_js_builder = P.cite_line(text, r'number_predicates_browser_bundle_source\("1"\)')
    c_ts_builder = P.cite_line(text, r'number_predicates_browser_bundle_source\("1 as const"\)')

    bundle_anchors = [
        ('"// kali-tree-shake: browserNumberPredicates\\n"',
         "// kali-tree-shake: browserNumberPredicates\n"),
        ("""console.log('browser number predicates ok');\\n""",
         "  console.log('browser number predicates ok');\n"),
    ]
    js_program = check_captured("app.js/app.jsx", C.CAP_NUMBER_BUNDLE_JS, common,
                                anchors=bundle_anchors)
    ts_program = check_captured("app.ts/app.tsx", C.CAP_NUMBER_BUNDLE_TS, common,
                                anchors=bundle_anchors)
    if "const alias = 1;" not in js_program or "const alias = 1 as const;" not in ts_program:
        raise AssertionError("the two captured programs do not carry their alias literals")
    body = check_program("harness body", fixture_starting(
        text, helper, "const mod = await import("), must_contain="await import(")

    docs = [kali_common_doc("number_predicates_browser_bundle_source")]

    decline = [
        "THE FIXTURE BODY DIFFERS BY EXTENSION, so no `[matrix]` can carry this file.",
        f"`app.js` and `app.jsx` are built from",
        f"kali_common::number_predicates_browser_bundle_source(\"1\") (:{c_js_builder});",
        f"`app.ts` and `app.tsx` from the same builder with \"1 as const\" (:{c_ts_builder}),",
        "which emits a TypeScript `as const` assertion the JS inputs must not carry.",
        "A `[matrix] ext` axis substitutes into the `[source]` KEY `app.${ext}`, and one key",
        "can hold exactly one body, so the axis cannot express two programs.",
    ]

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the `kali build` process (:{c_build_exit}) and on the",
        f"browser-bundle harness process (:{c_harness_exit}).",
        "In json mode, the build envelope's schemaVersion/command/success/exitCode and the",
        f"payload's artifactKind/bundleFormat (:{c_env_first}-{c_env_bundle_format}), plus",
        f"the empty `errors` array (:{c_env_errors}).",
        f"The emitted app/app.meta.json apiSurface/artifactKind (:{c_meta_api}-{c_meta_kind})",
        "is asserted in BOTH modes, because the source reads that file outside the",
        "`if json_output` block.",
        f"The harness step carries this file's ONE stdout claim,",
        f"`.contains(\"browser number predicates ok\")` (:{c_ok}) -- the fixture itself does",
        "all the predicate checking and throws on any mismatch, so that single line is the",
        "whole output contract.",
        "There is no count claim, no stderr claim on either process, and the build",
        "envelope's stdout leaf is never read, so no `json.stdout` pin.",
        "The source passes no --max-threads / --max-spawned-processes arguments, so neither",
        "appears on argv.",
    ]

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_declined(test_fns=8, invocations=8, cases=8, reason=decline),
        "",
        P.RULE6_ONE_TO_ONE,
        "",
        P.u2_source_file_wide(["app.js", "app.ts", "app.jsx", "app.tsx"]),
        "All four share the stem `app`, so `kali build --bundle` emits `app/` for every one",
        "and the harness `entry` and the `app/app.meta.json` path hold unchanged; no U5",
        "rename is needed or made.",
        "",
        P.RULING7_NO_HOIST,
        "",
        [
            "RULE 8 / RULE 9 -- the [source] programs are built one level removed, inside",
            "kali_common, by a `format!` over a `concat!` template, so neither appears as a",
            "string literal in this `.rs` at all. Both bodies below are the byte-exact OUTPUT",
            "of executing the real builder, captured by a temporary test target that",
            "`include!`d this `.rs` and dumped the two fixture builders' return values; the",
            "procedure is recorded in `tools/task-18-browser-pilot/batch6a_captures.py`. The",
            "generator re-checks each capture against crates/kali_common/src/number.rs's own",
            "template segments before emitting it, so a capture taken before a source edit",
            "fails the generator rather than shipping a program that is no longer the program",
            "under test.",
        ],
        "",
        P.rule13_header(
            ["kali_bin", "browser_bundle_number_predicates_js_source",
             "browser_bundle_number_predicates_ts_source", helper],
            docs_carried=docs,
            extra=["The documented helper is kali_common::number_predicates_browser_bundle_source,",
                   "in crates/kali_common/src/number.rs. Its `///` doc is extracted from that file",
                   "by this generator, not retyped, and carried verbatim into the rationale of",
                   "every case whose [source] body it produced (U6)."]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    desc = (
        f"`{helper}` builds a browser bundle with `kali build --bundle --api browser`, "
        "asserts the emitted app/app.meta.json metadata, then runs the bundle glue under "
        "the browser-bundle-harness contract backed by node, against a program that "
        "exercises the whole supported Number-predicate slice through direct, aliased, "
        "frozen, bracketed and parenthesized access forms and throws unless every one "
        "returns the expected boolean."
    )
    cases = []
    for ext in EXTS4:
        program_kind = "js" if ext in ("js", "jsx") else "ts"
        for jo in (False, True):
            name = f"{'json_' if jo else ''}build_emits_browser_number_predicates_in_{ext}_input"
            cases.append({
                "name": name,
                "rationale": para(
                    f"Migrated from browser_{stem}.rs, `{name}` -- one source `#[test]` fn, "
                    "one case (no `[matrix]` in this file; the fixture body differs by "
                    "extension, see the header).",
                    desc,
                    f"This case's `app.{ext}` body is the {program_kind} form of the shared "
                    "builder"
                    + (", whose alias binding carries the TypeScript `as const` assertion."
                       if program_kind == "ts" else ", whose alias binding is a plain 1."),
                    f"Its single stdout claim is `.contains(\"browser number predicates "
                    f"ok\")` (:{c_ok}).",
                    P.ruling3_substring(),
                    P.rule13_carried(docs),
                    ("This sibling additionally asserts the build JSON envelope rather than "
                     "plain text; output shape is not a matrix axis because it changes the "
                     "assertion shape, so it is a separate case.") if jo else None),
                "steps": bundle_steps(
                    f"app.{ext}", body,
                    {"stdout_contains": ["browser number predicates ok"]},
                    json_output=jo, json_claims=envelope_build(errors=True),
                    meta_fields=META),
            })
    source = {"app.js": js_program, "app.jsx": js_program,
              "app.ts": ts_program, "app.tsx": ts_program}
    P.assert_identical("the js/jsx bundle fixtures", source["app.js"], source["app.jsx"])
    P.assert_identical("the ts/tsx bundle fixtures", source["app.ts"], source["app.tsx"])
    if source["app.js"] == source["app.ts"]:
        raise AssertionError("the js and ts programs must differ -- the whole reason the "
                             "matrix is declined")
    return emit(header, None,
                {"app.js": js_program, "app.ts": ts_program,
                 "app.jsx": js_program, "app.tsx": ts_program}, cases)


# ==========================================================================
# F10. browser_number_predicates_harness.rs -- 16 fns / 16 invocations.
# ==========================================================================

@target("number_predicates_harness")
def gen_number_predicates_harness():
    stem = "number_predicates_harness"
    text = rs(stem)
    common = open(KALI_COMMON_NUMBER).read()
    helper = "assert_browser_harness_number_predicates"

    c_exit = P.cite_line(text, r"output\.status\.success\(\)")
    c_json_first = P.cite_line(text, r'assert_eq!\(json\["schemaVersion"\]')
    c_json_backend = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["runtimeBackend"\]')
    c_json_exit = P.cite_line(text, r'assert_eq!\(json\["exitCode"\]')
    c_json_stdout_eq = P.cite_line(text, r'json\["stdout"\],$')
    c_json_skipped = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["skipped"\]')
    c_json_stdout_contains = P.cite_line(text, r'assert!\(json\["stdout"\]')
    c_json_stderr = P.cite_line(text, r'assert_eq!\(json\["stderr"\]')
    c_text_contains = P.cite_line(text, r'stdout\.contains\($')
    c_ok1 = P.cite_line(text, r'stdout\.contains\("ok 1"\)')
    c_env = P.cite_line(text, rf'\.env\("{HARNESS_ENV}"')

    run_program = check_captured(
        "main.${ext}", C.CAP_NUMBER_HARNESS_RUN, common,
        anchors=[("const alias = {alias_literal}; const finite = Number.isFinite;",
                  "const alias = 1; const finite = Number.isFinite;"),
                 ("console.log(Number.isFinite(alias))",
                  "console.log(Number.isFinite(alias))")])
    test_program = check_captured(
        "smoke.test.${ext}", C.CAP_NUMBER_HARNESS_TEST, common,
        anchors=[("Kali.test('number predicates', () => {{ {} {} }});",
                  "Kali.test('number predicates', () => { const alias = 1;"),
                 ("console.log(safeInteger(alias))",
                  "console.log(safeInteger(alias))")])
    needle = _long_needle(text, "1\n1\n1\n0\n")

    pin = _pin("number_predicates_harness json.stdout", needle,
               [(f"main.{e}", run_program, "run", False) for e in EXTS4]
               + [(f"smoke.test.{e}", test_program, "test", False) for e in EXTS4])

    docs_run = [kali_common_doc(n) for n in ("number_predicates_runtime_source",
                                             "number_predicates_preamble_source",
                                             "number_predicates_console_log_body_source")]
    docs_test = [kali_common_doc(n) for n in ("number_predicates_test_source",
                                              "number_predicates_preamble_source",
                                              "number_predicates_console_log_body_source")]

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the one `kali` process (:{c_exit}); the environment carries",
        f"{HARNESS_ENV}=node (:{c_env}). The source passes no --max-threads /",
        "--max-spawned-processes arguments, so neither appears on argv.",
        f"json mode carries schemaVersion/command/success/payload(hostContract,",
        f"runtimeBackend) (:{c_json_first}-{c_json_backend}).",
        "THE TWO JSON BRANCHES ASSERT THE SAME LEAF IN DIFFERENT SHAPES, and the difference",
        "is preserved rather than normalised:",
        f"  * `run` asserts `exitCode` at both levels (:{c_json_exit}) and then an EXACT",
        f"    `assert_eq!(json[\"stdout\"], ...)` (:{c_json_stdout_eq}) -- already exact in",
        "    the source, so ruling 3's exact-source-assertion direction applies directly.",
        f"  * `test` asserts payload total/passed/failed/skipped (:{c_json_skipped} is the",
        f"    last of the four) and a `.contains(...)` on the same leaf",
        f"    (:{c_json_stdout_contains}) -- a plain `.contains` against a JSON leaf, which",
        "    has no substring form, so ruling 3 makes it an exact pin, live-captured.",
        "Both land on the same string, and the live capture asserts that across all eight",
        "run/test x ext cells before either is emitted.",
        f"Both branches then carry `stderr = \"\"` (:{c_json_stderr}). Neither reads `errors`,",
        "so no `errors` claim is written.",
        f"Text mode carries the same needle against raw stdout (:{c_text_contains}) and, for",
        f"`test` only, `.contains(\"ok 1\")` (:{c_ok1}).",
        "This file uses no count key: the source makes no `.matches(...).count()` claim.",
    ]

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[(helper, 16, "command(run/test) x ext(js/ts/jsx/tsx) x\n"
                                  "    json_output(false/true), a complete cross product")]),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        "",
        [
            "RULE 8 / RULE 9 -- both [source] programs are built one level removed, inside",
            "kali_common, by `format!`, so neither appears as a string literal in this `.rs`.",
            "Both bodies below are the byte-exact OUTPUT of executing the real builders,",
            "captured by a temporary test target that `include!`d this `.rs` and dumped their",
            "return values; the procedure is recorded in",
            "`tools/task-18-browser-pilot/batch6a_captures.py`.",
        ],
        "",
        [
            "CHECK_FIXTURES.PY IS VACUOUS ON THIS PAIR, AND THAT REPORT IS CORRECT.",
            "`verify_pair.sh number_predicates_harness` exits non-zero at the rule-9 fixture",
            "arm with `VACUOUS: no fixture-shaped literals found`. That is a true statement",
            "about this `.rs`: BOTH of its fixtures are built inside kali_common, so the file",
            "contains no program-shaped string literal for the gate to compare -- its `.rs`",
            "holds only the two builder call sites and the assertion needles. The gate's",
            "vacuity floor is doing exactly its job (a vacuous green is the dangerous",
            "direction) and it is NOT worked around here.",
            "The real check is the same gate pointed at the crate that actually holds the",
            "fixtures, which is where rule 9's one-level-removed clause puts it:",
            "  python3 tools/task-18-browser-pilot/check_fixtures.py \\",
            "      crates/kali_common/src/number.rs \\",
            "      crates/kali_cli/tests/cases/browser/number_predicates_harness.toml \\",
            "      crates/kali_cli/tests/cases/browser/number_predicates_bundle.toml",
            "It exits 0, over a corpus of 66 fixtures against the two case files together --",
            "strictly more than the pairwise run could ever have checked. Run it whenever",
            "either of those two case files changes.",
        ],
        "",
        P.rule13_header(
            ["kali_bin", "browser_number_predicates_run_source",
             "browser_number_predicates_test_source", helper],
            docs_carried=docs_run + docs_test[:1],
            extra=["The documented chain, in call order, all in crates/kali_common/src/number.rs:",
                   "  number_predicates_runtime_source and number_predicates_test_source, each of",
                   "  which calls number_predicates_preamble_source and",
                   "  number_predicates_console_log_body_source.",
                   "Their `///` docs are extracted from that file by this generator, not retyped,",
                   "and carried verbatim into the rationale of every case they reach -- the",
                   "runtime-source doc into the `run` cases only and the Kali.test-source doc into",
                   "the `test` cases only, per U6, never pooled across both."]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    desc = (
        f"`{helper}` writes the canonical supported Number-predicate slice as a plain "
        "program (or the same slice inside a Kali.test wrapper) and runs `kali run` / `kali "
        "test --api browser` with the browser harness backed by node; the program's 43 "
        "console.log calls print the predicate results as 1s and 0s."
    )
    cases = []
    for command, entry, docs in (("run", "main.${ext}", docs_run),
                                 ("test", "smoke.test.${ext}", docs_test)):
        for jo in (False, True):
            name = (f"{'json_' if jo else ''}{command}_supports_number_predicates_"
                    "when_browser_harness_is_configured")
            asserts = {}
            if not jo:
                asserts["stdout_contains"] = [needle] + (["ok 1"] if command == "test" else [])
            if jo:
                ruling3 = (
                    "The source already asserts this leaf exactly, with `assert_eq!`, so the "
                    "exact `json.stdout` pin follows directly from ruling 3's "
                    "exact-source-assertion direction; U9's live capture confirms it against "
                    "the real binary for every cell."
                    if command == "run" else P.ruling3_json_leaf())
            else:
                ruling3 = (f"Its stdout claim is the long `.contains(...)` needle at "
                           f":{c_text_contains}. " + P.ruling3_substring())
            cases.append({
                "name": name,
                "rationale": para(
                    f"Migrated from browser_{stem}.rs, the four `{name}_in_*_input` fns "
                    "(one per extension).",
                    desc, ruling3,
                    (f"For `test` the source also asserts `.contains(\"ok 1\")` (:{c_ok1}), "
                     "which the json branch does not make."
                     if command == "test" and not jo else None),
                    P.rule13_carried(docs)),
                "steps": [harness_step(
                    command, entry, json_output=jo, asserts=asserts,
                    json_claims=_harness_json_with_stdout(
                        command, pin,
                        extra_payload={"skipped": 0} if command == "test" else None,
                        stderr=True),
                    env_var=HARNESS_ENV)],
            })
    return emit(header, {"ext": EXTS4},
                {"main.${ext}": run_program, "smoke.test.${ext}": test_program}, cases)


# ==========================================================================
# F11. browser_object_computed_numeric_keys_bundle.rs -- 4 fns / 16 invocations.
# ==========================================================================

@target("object_computed_numeric_keys_bundle")
def gen_object_computed_numeric_keys_bundle():
    stem = "object_computed_numeric_keys_bundle"
    text = rs(stem)
    helper = "assert_browser_bundle_computed_numeric_keys"

    c_build_exit, c_harness_exit = P.cite_line(
        text, r"output\.status\.success\(\)", label="status.success", expect=2)
    c_env_first = P.cite_line(text, r'assert_eq!\(envelope\["schemaVersion"\]')
    c_env_bundle_format = P.cite_line(text, r'assert_eq!\(payload\["bundleFormat"\]')
    c_env_errors = P.cite_line(text, r'assert!\(envelope\["errors"\]')
    c_meta_api = P.cite_line(text, r'assert_eq!\(metadata\["apiSurface"\]')
    c_meta_kind = P.cite_line(text, r'assert_eq!\(metadata\["artifactKind"\]')
    c_neg = P.cite_line(text, r'stdout\.contains\("neg"\)')
    c_zero = P.cite_line(text, r'stdout\.contains\("zero"\)')
    c_format = P.cite_line(text, r"^\s*&format!\($")

    plain_program = check_program("app_plain.${ext}", fixture_in_fn(
        text, "browser_bundle_computed_numeric_keys_source"))
    await_program = check_program("app_await.${ext}", fixture_in_fn(
        text, "browser_bundle_computed_numeric_keys_with_await_wrappers_source"))
    plain_body = check_captured(
        "harness body (plain)", C.CAP_COMPUTED_BUNDLE_BODY_PLAIN, text,
        anchors=[("const mod = await import(bundleJs.href);",
                  "const mod = await import(bundleJs.href);"),
                 ('"computedNumericObjectKeys"', "await mod.computedNumericObjectKeys();")],
        must_contain="await import(")
    await_body = check_captured(
        "harness body (await wrappers)", C.CAP_COMPUTED_BUNDLE_BODY_AWAIT, text,
        anchors=[("const mod = await import(bundleJs.href);",
                  "const mod = await import(bundleJs.href);"),
                 ('"computedNumericObjectKeysWithAwaitWrappers"',
                  "await mod.computedNumericObjectKeysWithAwaitWrappers();")],
        must_contain="await import(")

    renames = [
        ("app.${ext} (the plain-computed-key program)", "app_plain.${ext}",
         "the source writes TWO different programs to the same `app.<ext>` filename in "
         "different tests"),
        ("app.${ext} (the await-wrapped program)", "app_await.${ext}",
         "same filename, second program"),
    ]

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the `kali build` process (:{c_build_exit}) and on the",
        f"browser-bundle harness process (:{c_harness_exit}).",
        "In json mode, the build envelope's schemaVersion/command/success/exitCode and the",
        f"payload's artifactKind/bundleFormat (:{c_env_first}-{c_env_bundle_format}), plus",
        f"the empty `errors` array (:{c_env_errors}).",
        f"The emitted <entry>/<entry>.meta.json apiSurface/artifactKind",
        f"(:{c_meta_api}-{c_meta_kind}) is asserted in BOTH modes, because the source reads",
        "that file outside the `if json_output` block.",
        f"The harness step carries the file's three stdout claims, `.contains(\"neg\")`",
        f"(:{c_neg}), `.contains(\"pos\")` on the next line and `.contains(\"zero\")`",
        f"(:{c_zero}), all of which stay `stdout_contains`.",
        "There is no count claim, no stderr claim on either process, and the build",
        "envelope's stdout leaf is never read, so no `json.stdout` pin.",
        "The source passes no --max-threads / --max-spawned-processes arguments, so neither",
        "appears on argv.",
    ]

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        extra_ok_renames(["app_plain", "app_await"], EXTS4),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=4, invocations=16, cases=4, axis="ext", values=EXTS4,
            non_axes=("json_output",),
            helpers=[(helper, 16, "program(plain/await-wrapped) x ext(js/ts/jsx/tsx) x\n"
                                  "    json_output(false/true). Each of the 4 `#[test]` fns\n"
                                  "    is a `for filename in [...]` loop over four\n"
                                  "    filenames, so 4 fns make 16 invocations")]),
        "THE PROGRAM DIMENSION IS NOT AN AXIS EITHER, and for a different reason from",
        "`json_output`: the two programs are two different FILES, not one file's text under",
        "substitution. `[source]` is file-wide (U2), so both are written into every trial",
        "dir and each case names its own entry on argv; an axis cannot express that, because",
        "an axis substitutes into a `[source]` KEY and one key holds one body.",
        "",
        P.rule6_matrix_fold("one source `#[test]` fn, whose own `for filename in [...]` loop\n"
                            "over four filenames is what the four `ext` cells reproduce"),
        "",
        P.u2_source_file_wide(["app_plain.${ext}", "app_await.${ext}"]),
        "",
        P.u5_renames(renames),
        "`kali build --bundle` names its output directory after the input STEM, so each",
        "case's `file_json` path and browser-bundle-harness `entry` track the rename rather",
        "than staying hardcoded to `app` -- `app_plain/app_plain.meta.json` and",
        "`entry = \"app_plain\"`, and likewise for the await-wrapped pair.",
        "",
        [
            "RULE 8 / RULE 9 -- the two browser-bundle harness BODIES are `format!`-built,",
            f"inline inside the assert helper (:{c_format}) with a placeholder for the",
            "exported function name, so neither exists as a string literal anywhere. Both are",
            "the byte-exact OUTPUT of executing the real code, captured by running this",
            f"target's own tests with {HARNESS_ENV} pointed at a wrapper that copies the",
            "harness script it is handed and then execs node, and subtracting",
            "kali_runtime_contract::browser_bundle_harness_prelude(\"app\", false) from each",
            "captured script -- `browser_bundle_harness_script` is defined as prelude + body,",
            "so the remainder IS the resolved body. The procedure is recorded in",
            "`tools/task-18-browser-pilot/batch6a_captures.py`.",
        ],
        "",
        P.rule13_header([
            "kali_bin", "browser_bundle_computed_numeric_keys_source",
            "browser_bundle_computed_numeric_keys_with_await_wrappers_source", helper]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    cases = []
    for kind, entry_stem, program, body, fn_infix in (
        ("plain", "app_plain", plain_program, plain_body, ""),
        ("await", "app_await", await_program, await_body, "await_wrapped_"),
    ):
        for jo in (False, True):
            name = f"{'json_' if jo else ''}build_emits_{fn_infix}computed_numeric_object_keys"
            desc = (
                f"`{helper}` builds a browser bundle with `kali build --bundle --api "
                "browser`, asserts the emitted metadata, then runs the bundle glue under "
                "the browser-bundle-harness contract backed by node, against a program "
                "whose object literal uses computed numeric keys "
                + ("wrapped in await expressions (`[await 1]`, `[+(await 2)]`, "
                   "`[(0, await 0)]`)" if kind == "await" else
                   "with unary signs and a negative zero (`[-1]`, `[+2]`, `[(-0)]`)")
                + ", then reads them back and prints neg, pos and zero.")
            steps = bundle_steps(
                f"{entry_stem}.${{ext}}", body,
                {"stdout_contains": ["neg", "pos", "zero"]},
                json_output=jo, json_claims=envelope_build(errors=True),
                meta_fields=META)
            steps[1]["path"] = f"{entry_stem}/{entry_stem}.meta.json"
            steps[2]["entry"] = entry_stem
            cases.append({
                "name": name,
                "rationale": para(
                    f"Migrated from browser_{stem}.rs, `{name}_in_js_ts_jsx_and_tsx_input` "
                    "-- one source `#[test]` fn, whose `for filename in [...]` loop is "
                    "reproduced by this file's four `ext` cells.",
                    desc,
                    f"Its three stdout claims are separate source lines (:{c_neg} and the "
                    f"two after it, ending at :{c_zero}).",
                    P.ruling3_substring(),
                    ("This sibling additionally asserts the build JSON envelope rather than "
                     "plain text; output shape is not a matrix axis because it changes the "
                     "assertion shape, so it is a separate case.") if jo else None),
                "steps": steps,
            })
    source = {"app_plain.${ext}": plain_program, "app_await.${ext}": await_program}
    assert_rename_is_argv_only(
        source, ["app.${ext}", "app_plain.${ext}", "app_await.${ext}"], EXTS4)
    return emit(header, {"ext": EXTS4}, source, cases)


# ==========================================================================
# F12. browser_object_computed_numeric_keys_harness.rs -- 8 fns / 32 invocations.
# ==========================================================================

@target("object_computed_numeric_keys_harness")
def gen_object_computed_numeric_keys_harness():
    stem = "object_computed_numeric_keys_harness"
    text = rs(stem)
    helper = "assert_browser_harness_computed_numeric_keys"

    c_exit = P.cite_line(text, r"output\.status\.success\(\)")
    c_json_first = P.cite_line(text, r'assert_eq!\(json\["schemaVersion"\]')
    c_json_backend = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["runtimeBackend"\]')
    c_json_exit = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["exitCode"\]')
    c_json_failed = P.cite_line(text, r'assert_eq!\(json\["payload"\]\["failed"\]')
    c_json_neg = P.cite_line(text, r'stdout\.contains\("neg"\), "json')
    c_json_stderr = P.cite_line(text, r'assert_eq!\(json\["stderr"\]')
    c_json_errors = P.cite_line(text, r'assert!\(json\["errors"\]')
    c_text_neg = P.cite_line(text, r'stdout\.contains\("neg"\), "stdout')
    c_text_zero = P.cite_line(text, r'stdout\.contains\("zero"\), "stdout')
    c_threads = P.cite_line(text, r'\.arg\("--max-threads"\)')
    c_procs = P.cite_line(text, r'\.arg\("--max-spawned-processes"\)')
    c_env_const = P.cite_line(text, r"BROWSER_HARNESS_COMMAND_ENV")
    c_format = P.cite_line(text, r'"test" => format!\(', expect=2)[0]

    plain_run = check_captured(
        "main_plain.${ext}", C.CAP_COMPUTED_KEYS_RUN, text,
        anchors=[("const obj = { [-1]: 'neg', [+2]: 'pos', [(-0)]: 'zero' };",
                  "const obj = { [-1]: 'neg', [+2]: 'pos', [(-0)]: 'zero' };")])
    plain_test = check_captured(
        "smoke_plain.test.${ext}", C.CAP_COMPUTED_KEYS_TEST, text,
        anchors=[("Kali.test('computed numeric object keys', () => {{\\n{body}}});\\n",
                  "Kali.test('computed numeric object keys', () => {\n")])
    await_run = check_captured(
        "main_await.${ext}", C.CAP_COMPUTED_AWAIT_RUN, text,
        anchors=[("computedNumericObjectKeysWithAwaitWrappers();",
                  "computedNumericObjectKeysWithAwaitWrappers();")])
    await_test = check_captured(
        "smoke_await.test.${ext}", C.CAP_COMPUTED_AWAIT_TEST, text,
        anchors=[("  return computedNumericObjectKeysWithAwaitWrappers();\\n}});\\n",
                  "  return computedNumericObjectKeysWithAwaitWrappers();\n});\n")])

    pins = {
        ("run", "plain"): _pin("computed keys run/plain json.stdout", "neg\npos\nzero\n",
                               [(f"main_plain.{e}", plain_run, "run", True) for e in EXTS4]),
        ("run", "await"): _pin("computed keys run/await json.stdout", "neg\npos\nzero\n",
                               [(f"main_await.{e}", await_run, "run", True) for e in EXTS4]),
        ("test", "plain"): _pin("computed keys test/plain json.stdout", "neg\npos\nzero\n",
                                [(f"smoke_plain.test.{e}", plain_test, "test", True)
                                 for e in EXTS4]),
        ("test", "await"): _pin(
            "computed keys test/await json.stdout", "neg\npos\nzero\nneg\npos\nzero\n",
            [(f"smoke_await.test.{e}", await_test, "test", True) for e in EXTS4]),
    }

    renames = [
        ("main.${ext} (plain computed keys)", "main_plain.${ext}",
         "the source writes two different run programs to the same `main.<ext>` filename"),
        ("main.${ext} (await wrappers)", "main_await.${ext}", "same filename, second program"),
        ("smoke.test.${ext} (plain computed keys)", "smoke_plain.test.${ext}",
         "the source writes two different Kali.test programs to the same "
         "`smoke.test.<ext>` filename; the `.test.` infix is preserved so the file is still "
         "a test file to `kali test`"),
        ("smoke.test.${ext} (await wrappers)", "smoke_await.test.${ext}",
         "same filename, second program"),
    ]

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the one `kali` process (:{c_exit}); the argv carries",
        f"`--max-threads 0` (:{c_threads}) and `--max-spawned-processes 0` (:{c_procs}), and",
        "the environment carries the variable named by",
        f"`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` (:{c_env_const}), set to node.",
        f"json mode carries schemaVersion/command/success/payload(hostContract,",
        f"runtimeBackend) (:{c_json_first}-{c_json_backend}), then for `run` the payload's",
        f"`exitCode` ONLY (:{c_json_exit}) -- this source does NOT assert the envelope-level",
        "`exitCode` that most of the family does, and that absence is preserved -- or for",
        f"`test` the payload total/passed/failed (:{c_json_failed} is the last of the three),",
        "with no `skipped` claim.",
        f"The three `.contains` claims on the json stdout leaf (:{c_json_neg} and the two",
        "after it) become one exact `json.stdout` pin, because a JSON leaf has no substring",
        f"form; then `stderr = \"\"` (:{c_json_stderr}) and the empty `errors` array",
        f"(:{c_json_errors}).",
        f"Text mode carries the same three claims against raw stdout (:{c_text_neg} to",
        f":{c_text_zero}) and nothing else -- there is no `ok 1` claim anywhere in this file.",
        "THE FOUR PINS ARE NOT ALL THE SAME STRING. Three of the four (run/plain, run/await,",
        "test/plain) print neg, pos, zero once; test/await prints them TWICE, because that",
        "fixture both calls the async function at top level and returns it from the",
        "`Kali.test` body. Each pin is captured for its own case, per cell, and the four are",
        "not collapsed.",
        "This file uses no count key: the source makes no `.matches(...).count()` claim.",
    ]

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        extra_ok_renames(["main_plain", "main_await"], EXTS4),
        extra_ok_renames(["smoke_plain.test", "smoke_await.test"], EXTS4),
        P.extra_ok("neg\npos\nzero\n", P.EXTRA_OK_JSON_STDOUT),
        P.extra_ok("neg\npos\nzero\nneg\npos\nzero\n", P.EXTRA_OK_JSON_STDOUT),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        "",
        P.matrix_arithmetic(
            test_fns=8, invocations=32, cases=8, axis="ext", values=EXTS4,
            helpers=[(helper, 32, "program(plain/await-wrapped) x command(run/test) x\n"
                                  "    ext(js/ts/jsx/tsx) x json_output(false/true). Each of\n"
                                  "    the 8 `#[test]` fns is a `for filename in [...]` loop\n"
                                  "    over four filenames, so 8 fns make 32 invocations")]),
        "THE PROGRAM DIMENSION IS NOT AN AXIS EITHER, and for a different reason from",
        "`command`/`json_output`: the two programs are different FILES, not one file's text",
        "under substitution, and an axis substitutes into a `[source]` KEY, which holds one",
        "body.",
        "",
        P.rule6_matrix_fold("one source `#[test]` fn, whose own `for filename in [...]` loop\n"
                            "over four filenames is what the four `ext` cells reproduce"),
        "",
        P.u2_source_file_wide(["main_plain.${ext}", "main_await.${ext}",
                               "smoke_plain.test.${ext}", "smoke_await.test.${ext}"]),
        "",
        P.u5_renames(renames),
        "",
        [
            "RULE 8 / RULE 9 -- the two `test` fixtures are `format!`-built",
            f"(:{c_format} is the first), so their resolved text appears in no string literal",
            "in the `.rs`: the `.rs` holds the template and the shared `body` separately. All",
            "four bodies below are the byte-exact OUTPUT of executing the real builders,",
            "captured by a temporary test target that `include!`d this `.rs` and dumped the",
            "two builders' return values for both command arms; the procedure is recorded in",
            "`tools/task-18-browser-pilot/batch6a_captures.py`.",
        ],
        "",
        P.rule13_header(["kali_bin", "computed_numeric_keys_source",
                         "computed_numeric_keys_with_await_wrappers_source", helper]),
        "",
        P.ARGV_ORDER,
        "",
        shape,
    )

    cases = []
    for kind, fn_infix in (("plain", ""), ("await", "await_wrapped_")):
        for command in ("run", "test"):
            entry = (f"main_{kind}.${{ext}}" if command == "run"
                     else f"smoke_{kind}.test.${{ext}}")
            for jo in (False, True):
                name = (f"{'json_' if jo else ''}{command}_supports_{fn_infix}"
                        "computed_numeric_object_keys_when_browser_harness_is_configured")
                cases.append({
                    "name": name,
                    "rationale": para(
                        f"Migrated from browser_{stem}.rs, "
                        f"`{name}_in_js_ts_jsx_and_tsx_input` -- one source `#[test]` fn, "
                        "whose `for filename in [...]` loop is reproduced by this file's "
                        "four `ext` cells.",
                        f"`{helper}` runs `kali {command} --api browser` with the browser "
                        "harness backed by node and both concurrency limits pinned to 0, "
                        "against a program whose object literal uses computed numeric keys "
                        + ("wrapped in await expressions" if kind == "await"
                           else "with unary signs and a negative zero")
                        + ", then reads them back and prints neg, pos and zero.",
                        (P.ruling3_json_leaf() if jo else
                         f"Its three stdout claims are separate source lines (:{c_text_neg} "
                         f"to :{c_text_zero}). " + P.ruling3_substring()),
                        ("This fixture prints the three values TWICE -- it calls the async "
                         "function at top level and also returns it from the `Kali.test` "
                         "body -- so its pin is the doubled string, not the same one the "
                         "other three cases carry."
                         if (kind == "await" and command == "test" and jo) else None)),
                    "steps": [harness_step(
                        command, entry, json_output=jo,
                        asserts={} if jo else {"stdout_contains": ["neg", "pos", "zero"]},
                        json_claims=_computed_keys_json(command, pins[(command, kind)]),
                        thread_flags=True, env_var=HARNESS_ENV)],
                })
    source = {"main_plain.${ext}": plain_run, "main_await.${ext}": await_run,
              "smoke_plain.test.${ext}": plain_test,
              "smoke_await.test.${ext}": await_test}
    assert_rename_is_argv_only(
        source, ["main.${ext}", "smoke.test.${ext}", "main_plain.${ext}",
                 "main_await.${ext}", "smoke_plain.test.${ext}",
                 "smoke_await.test.${ext}"], EXTS4)
    return emit(header, {"ext": EXTS4}, source, cases)


# ==========================================================================
# F13. browser_object_entries_harness.rs -- 32 fns / 32 invocations.
# ==========================================================================

@target("object_entries_harness")
def gen_object_entries_harness():
    stem = "object_entries_harness"
    text = rs(stem)
    helper = "assert_browser_harness_object_entries"

    c_fail = P.cite_line(text, r"assert!\(!output\.status\.success\(\)")
    c_threads = P.cite_line(text, r'\.arg\("--max-threads"\)')
    c_procs = P.cite_line(text, r'\.arg\("--max-spawned-processes"\)')
    c_env_const = P.cite_line(text, r"BROWSER_HARNESS_COMMAND_ENV")
    c_frozen_replace = P.cite_line(text, r'"  const values = Object\.freeze', expect=2)[0]
    blocks = comment_blocks(text)
    if len(blocks) != 1:
        raise AssertionError(f"expected one comment block, found {len(blocks)}")
    c_comment, comment_lines = blocks[0]
    comment = " ".join(comment_lines)

    plain_run = check_program("main_plain.${ext}", fixture_in_fn(
        text, "browser_harness_object_entries_run_source"))
    plain_test = check_program("smoke_plain.test.${ext}", fixture_in_fn(
        text, "browser_harness_object_entries_test_source"))
    frozen_run = check_captured(
        "main_frozen.${ext}", C.CAP_ENTRIES_FROZEN_RUN, text,
        anchors=[('"  const values = Object.freeze({ \\"b\\": 1, \\"a\\": 2 });"',
                  '  const values = Object.freeze({ "b": 1, "a": 2 });')])
    frozen_test = check_captured(
        "smoke_frozen.test.${ext}", C.CAP_ENTRIES_FROZEN_TEST, text,
        anchors=[('"  const values = Object.freeze({ \\"b\\": 1, \\"a\\": 2 });"',
                  '  const values = Object.freeze({ "b": 1, "a": 2 });')])
    for label, frozen, plain in (("run", frozen_run, plain_run),
                                 ("test", frozen_test, plain_test)):
        if frozen == plain:
            raise AssertionError(f"the frozen {label} fixture is identical to the plain one -- "
                                 "the `.replace` capture did not take")

    renames = [
        ("main.${ext} (unfrozen receiver)", "main_plain.${ext}",
         "the source writes two different run programs to the same `main.<ext>` filename"),
        ("main.${ext} (Object.freeze'd receiver)", "main_frozen.${ext}",
         "same filename, second program"),
        ("smoke.test.${ext} (unfrozen receiver)", "smoke_plain.test.${ext}",
         "the source writes two different Kali.test programs to the same "
         "`smoke.test.<ext>` filename; the `.test.` infix is preserved so the file is still "
         "a test file to `kali test`"),
        ("smoke.test.${ext} (Object.freeze'd receiver)", "smoke_frozen.test.${ext}",
         "same filename, second program"),
    ]

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        extra_ok_renames(["main_plain", "main_frozen"], EXTS4),
        extra_ok_renames(["smoke_plain.test", "smoke_frozen.test"], EXTS4),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        [
            "RULE 12 (carry every source comment verbatim): this source carries exactly ONE",
            f"Rust comment block (:{c_comment}), attached to the single assert helper. Every",
            "case below is produced by that helper, so the block is carried into every",
            "rationale -- which is per-helper attribution (U6), not pooling: there is one",
            "helper and one block. The text is COPIED out of the `.rs` by this file's",
            "generator, never retyped.",
        ],
        "",
        P.matrix_arithmetic(
            test_fns=32, invocations=32, cases=8, axis="ext", values=EXTS4,
            helpers=[(helper, 32, "program(unfrozen/frozen receiver) x command(run/test) x\n"
                                  "    ext(js/ts/jsx/tsx) x json_output(false/true), a\n"
                                  "    complete cross product. Every `#[test]` fn is one\n"
                                  "    unlooped call and the file contains no loop at all")]),
        "THE PROGRAM DIMENSION IS NOT AN AXIS EITHER: the two programs are different FILES,",
        "not one file's text under substitution, and an axis substitutes into a `[source]`",
        "KEY, which holds one body.",
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["main_plain.${ext}", "main_frozen.${ext}",
                               "smoke_plain.test.${ext}", "smoke_frozen.test.${ext}"]),
        "",
        P.u5_renames(renames),
        "",
        [
            "RULE 9 -- the two frozen fixtures are built by a Rust str::replace call over the",
            f"unfrozen ones (:{c_frozen_replace}), so their resolved text appears in no",
            "string literal in the `.rs`. Both are the byte-exact OUTPUT of executing the",
            "real builders,",
            "captured by a temporary test target that `include!`d this `.rs`; the procedure",
            "is recorded in `tools/task-18-browser-pilot/batch6a_captures.py`. The generator",
            "additionally asserts each frozen body actually DIFFERS from its unfrozen",
            "counterpart, so a capture that silently missed the replacement fails here.",
        ],
        "",
        stale_name_note(32, "the body asserts the command FAILS -- the source's only\n"
                            "process assertion is `assert!(!output.status.success(), ...)`. The names date from\n"
                            "before the honest re-pin recorded in the comment block this file carries."),
        "",
        P.rule13_header([
            "kali_bin", "browser_harness_object_entries_run_source",
            "browser_harness_object_entries_test_source",
            "browser_harness_object_entries_frozen_run_source",
            "browser_harness_object_entries_frozen_test_source", helper]),
        "",
        P.ARGV_ORDER,
        "",
        FAIL_CLOSED_NOTE,
        f"The `!success` assertion is at :{c_fail}. The argv carries `--max-threads 0`",
        f"(:{c_threads}) and `--max-spawned-processes 0` (:{c_procs}), and the environment",
        "carries the variable named by",
        f"`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` (:{c_env_const}), set to node.",
        "There is no json claim on either branch: the source parses no envelope at all, in",
        "either mode -- `--output json` changes only the argv here.",
    )

    cases = []
    for kind, fixture_run, fixture_test in (("plain", plain_run, plain_test),
                                            ("frozen", frozen_run, frozen_test)):
        for command in ("run", "test"):
            entry = (f"main_{kind}.${{ext}}" if command == "run"
                     else f"smoke_{kind}.test.${{ext}}")
            for jo in (False, True):
                name = (f"{'json_' if jo else ''}{command}_supports_object_entries_iteration_"
                        f"when_browser_harness_is_configured_{kind}_receiver")
                cases.append({
                    "name": name,
                    "rationale": para(
                        f"Migrated from browser_{stem}.rs, the four "
                        f"`{'json_' if jo else ''}{command}_supports_object_entries_"
                        "iteration_when_browser_harness_is_configured_in_*_input` fns (one "
                        "per extension)"
                        + (", in their Object.freeze'd-receiver form." if kind == "frozen"
                           else ", in their plain-receiver form."),
                        f"`{helper}` runs `kali {command} --api browser` with the browser "
                        "harness backed by node and both concurrency limits pinned to 0, "
                        "against a program that reads Object.entries through sixteen "
                        "direct, global, bracketed, parenthesized and frozen access forms"
                        + (" over an Object.freeze'd receiver object." if kind == "frozen"
                           else " over a plain receiver object."),
                        comment,
                        "MIGRATION NOTE (controller ruling 8): the source fn name says "
                        "\"supports\", but the body asserts the command FAILS. The name is "
                        "carried unchanged; see this file's header for why.",
                        "The whole claim is that the process fails, so the migrated step "
                        "carries `exit = \"failure\"` and nothing else."),
                    "steps": [{
                        "args": (["--output", "json"] if jo else [])
                        + [command, "--api", "browser", "--max-threads", "0",
                           "--max-spawned-processes", "0", entry],
                        "env": {HARNESS_ENV: "node"},
                        "exit": "failure",
                    }],
                })
    source = {"main_plain.${ext}": plain_run, "main_frozen.${ext}": frozen_run,
              "smoke_plain.test.${ext}": plain_test,
              "smoke_frozen.test.${ext}": frozen_test}
    assert_rename_is_argv_only(
        source, ["main.${ext}", "smoke.test.${ext}", "main_plain.${ext}",
                 "main_frozen.${ext}", "smoke_plain.test.${ext}",
                 "smoke_frozen.test.${ext}"], EXTS4)
    return emit(header, {"ext": EXTS4}, source, cases)


# ==========================================================================
# F14. browser_object_entries_iteration.rs -- 18 fns / 24 invocations.
# ==========================================================================

@target("object_entries_iteration")
def gen_object_entries_iteration():
    stem = "object_entries_iteration"
    text = rs(stem)
    alias_helper = "assert_browser_bundle_object_entries_iteration"
    direct_helper = "assert_browser_bundle_direct_object_entries_iteration"
    global_helper = "assert_browser_bundle_global_object_entries_iteration"

    fails = P.cite_line(text, r"assert!\(!output\.status\.success\(\)",
                        label="fail-closed asserts", expect=3)
    c_build_success = P.cite_line(text, r"output\.status\.success\(\)", expect=5)
    c_direct_build_ok, c_global_build_ok = c_build_success[1], c_build_success[3]
    c_env_first = P.cite_line(text, r'assert_eq!\(envelope\["schemaVersion"\]', expect=2)[0]
    c_env_exit = P.cite_line(text, r'assert_eq!\(envelope\["exitCode"\]', expect=2)[0]
    c_env_errors = P.cite_line(text, r'assert!\(envelope\["errors"\]', expect=2)[0]
    c_meta_api = P.cite_line(text, r'assert_eq!\(metadata\["apiSurface"\]', expect=2)[0]
    c_meta_kind = P.cite_line(text, r'assert_eq!\(metadata\["artifactKind"\]', expect=2)[0]

    blocks = comment_blocks(text)
    if len(blocks) != 3:
        raise AssertionError(f"expected three comment blocks, found {len(blocks)}")
    comments = {}
    for (line, lines), key in zip(blocks, ("alias", "direct", "global")):
        comments[key] = (line, " ".join(lines))

    alias_program = check_program("app_alias.${ext}", fixture_in_fn(
        text, "browser_bundle_object_entries_iteration_source"), must_contain="Object.entries")
    direct_program = check_program("app_direct.${ext}", fixture_in_fn(
        text, "browser_bundle_direct_object_entries_iteration_source"),
        must_contain="Object.entries")
    global_program = check_program("app_global.${ext}", fixture_in_fn(
        text, "browser_bundle_global_object_entries_iteration_source"),
        must_contain="Object.entries")
    direct_body = check_program("direct harness body", fixture_starting(
        text, direct_helper, "const mod = await import("), must_contain="await import(")
    global_body = check_program("global harness body", fixture_starting(
        text, global_helper, "const mod = await import("), must_contain="await import(")

    renames = [
        ("app.${ext} (aliased receiver, build fails)", "app_alias.${ext}",
         "the source writes THREE different programs to the same `app.<ext>` filename in "
         "different tests"),
        ("app.${ext} (direct object literals)", "app_direct.${ext}", "same filename, second "
         "program"),
        ("app.${ext} (globalThis-rooted receiver)", "app_global.${ext}", "same filename, "
         "third program"),
    ]

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        extra_ok_renames(["app_alias", "app_direct", "app_global"], EXTS4),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        [
            "RULE 12 (carry every source comment verbatim): this source carries THREE Rust",
            f"comment blocks (:{comments['alias'][0]}, :{comments['direct'][0]},",
            f":{comments['global'][0]}), one attached to each of its three assert helpers.",
            "Each is carried into the rationale of exactly the two cases its own helper",
            "produces -- bottom-up, per helper (U6) -- and the text is COPIED out of the",
            "`.rs` by this file's generator, never retyped: two of the three contain em",
            "dashes that a retyped version would render as `--`.",
        ],
        "",
        COMMENT_COVERAGE_MULTI_HELPER,
        "",
        P.matrix_arithmetic(
            test_fns=18, invocations=24, cases=6, axis="ext", values=EXTS4,
            non_axes=("json_output",),
            helpers=[
                (alias_helper, 8, "ext(js/ts/jsx/tsx) x json_output(false/true), from 8\n"
                                  "    unlooped `#[test]` fns"),
                (direct_helper, 8, "ext(4) x json_output(false/true), from just 2 `#[test]`\n"
                                   "    fns, each a `for filename in [...]` loop over four\n"
                                   "    filenames"),
                (global_helper, 8, "ext(4) x json_output(false/true), from 8 unlooped\n"
                                   "    `#[test]` fns"),
            ]),
        "18 fns but 24 invocations: the two `*_direct_*` fns loop and the other 16 do not.",
        "THE HELPER DIMENSION IS NOT AN AXIS: the three helpers write three different",
        "programs to three different files AND assert different things (the first asserts",
        "the BUILD fails; the other two assert the build succeeds and the browser-bundle",
        "harness fails), so each is sibling cases, not an axis.",
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell -- except the two\n"
                            "`*_direct_*` cases, where the 4 cells reproduce one fn's own\n"
                            "`for filename in [...]` loop over four filenames"),
        "",
        P.u2_source_file_wide(["app_alias.${ext}", "app_direct.${ext}", "app_global.${ext}"]),
        "",
        P.u5_renames(renames),
        "`kali build --bundle` names its output directory after the input STEM, so each",
        "case's `file_json` path and browser-bundle-harness `entry` track the rename rather",
        "than staying hardcoded to `app`.",
        "",
        stale_name_note(18, "no fn in this file asserts a successful end-to-end run:\n"
                            "every one of the three helpers ends in "
                            "`assert!(!output.status.success(), ...)`,\n"
                            "at the build step or at the browser-bundle harness step."),
        "",
        P.rule13_header([
            "kali_bin", "browser_bundle_object_entries_iteration_source",
            "browser_bundle_direct_object_entries_iteration_source",
            "browser_bundle_global_object_entries_iteration_source",
            alias_helper, direct_helper, global_helper]),
        "",
        P.ARGV_ORDER,
        "",
        [
            "ASSERTION SHAPE, mirrored from the source and nothing more, and the three",
            "helpers differ:",
            f"  * `{alias_helper}` asserts ONLY that the",
            f"    `kali build` process FAILS (:{fails[0]}). It reads no envelope even in json",
            "    mode, no metadata file and runs no harness -- the build never emits one. So",
            "    its two cases are a single `cli` step with `exit = \"failure\"`, and the json",
            "    sibling differs from the text one on argv alone.",
            f"  * `{direct_helper}` asserts the build SUCCEEDS",
            f"    (:{c_direct_build_ok}); in json mode the envelope's",
            f"    schemaVersion/command/success/exitCode (:{c_env_first}-{c_env_exit}) and the",
            f"    empty `errors` array (:{c_env_errors}) -- and NO payload claim, unlike every",
            "    other bundle helper in this batch, which is preserved rather than",
            f"    normalised; the emitted metadata's apiSurface/artifactKind",
            f"    (:{c_meta_api}-{c_meta_kind}) in BOTH modes; and then that the",
            f"    browser-bundle HARNESS process fails (:{fails[1]}).",
            f"  * `{global_helper}` is the same shape as the",
            f"    direct one, on its own fixture, ending in the same harness-level failure",
            f"    (:{fails[2]}); its build-success assert is at :{c_global_build_ok}.",
            "No helper makes any stdout, stderr or count claim, so none is written.",
        ],
    )

    cases = []
    for jo in (False, True):
        name = f"{'json_' if jo else ''}build_emits_object_entries_iteration_semantics"
        cases.append({
            "name": name,
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four `{name}_in_*_input` fns (one per "
                "extension).",
                f"`{alias_helper}` runs `kali build --bundle --api browser` on a program "
                "that reads Object.entries through eighteen access forms over an aliased "
                "receiver, and asserts the build fails closed.",
                comments["alias"][1],
                "MIGRATION NOTE (controller ruling 8): the source fn name says \"emits\", but "
                "the body asserts the build FAILS. The name is carried unchanged; see this "
                "file's header for why.",
                "The whole claim is that the build fails, so this case is one `cli` step "
                "with `exit = \"failure\"` and nothing else -- the source reads no envelope, "
                "no metadata and runs no harness on this path."),
            "steps": [{
                "args": ["build", "--bundle", "--api", "browser"]
                + (["--output", "json"] if jo else []) + ["app_alias.${ext}"],
                "exit": "failure",
            }],
        })
    for key, helper, entry_stem, program, body, fn_infix in (
        ("direct", direct_helper, "app_direct", direct_program, direct_body, "direct_"),
        ("global", global_helper, "app_global", global_program, global_body, "global_"),
    ):
        for jo in (False, True):
            name = (f"{'json_' if jo else ''}build_emits_{fn_infix}object_entries_"
                    "iteration_semantics")
            steps = [
                {"args": ["build", "--bundle", "--api", "browser"]
                 + (["--output", "json"] if jo else []) + [f"{entry_stem}.${{ext}}"],
                 "exit": "success"},
                {"kind": "file_json", "path": f"{entry_stem}/{entry_stem}.meta.json",
                 "fields": META},
                {"kind": "browser_bundle_harness", "entry": entry_stem, "body": body,
                 "exit": "failure"},
            ]
            if jo:
                steps[0]["json"] = {"schemaVersion": 1, "command": "build",
                                    "success": True, "exitCode": 0, "errors": []}
            cases.append({
                "name": name,
                "rationale": para(
                    f"Migrated from browser_{stem}.rs, "
                    + (f"the one fn `{name}_in_js_ts_jsx_tsx_input`, whose `for filename in "
                       "[...]` loop over four filenames is what this file's four `ext` cells "
                       "reproduce." if key == "direct" else
                       f"the four `{name}_in_*_input` fns (one per extension)."),
                    f"`{helper}` runs `kali build --bundle --api browser` on a program that "
                    "reads Object.entries through "
                    + ("twelve direct-object-literal access forms" if key == "direct"
                       else "nine globalThis-rooted access forms")
                    + ", asserts the build SUCCEEDS and that the emitted metadata names the "
                    "browser bundle, then runs the emitted bundle under the "
                    "browser-bundle-harness contract and asserts THAT process fails closed.",
                    comments[key][1],
                    "MIGRATION NOTE (controller ruling 8): the source fn name says \"emits\", "
                    "which is true of the build step, but the test's terminal claim is that "
                    "the harness run fails. The name is carried unchanged; see this file's "
                    "header for why.",
                    ("On the json branch the source additionally reads the build envelope's "
                     "schemaVersion/command/success/exitCode and asserts the `errors` array "
                     "is empty -- and asserts NO payload field, which is preserved rather "
                     "than normalised against the other bundle helpers in this batch."
                     if jo else None)),
                "steps": steps,
            })
    source = {"app_alias.${ext}": alias_program, "app_direct.${ext}": direct_program,
              "app_global.${ext}": global_program}
    assert_rename_is_argv_only(
        source, ["app.${ext}", "app_alias.${ext}", "app_direct.${ext}",
                 "app_global.${ext}"], EXTS4)
    return emit(header, {"ext": EXTS4}, source, cases)


# ==========================================================================
# F15. browser_object_enumeration_finalization_bundle.rs -- 8 fns / 8 invocations.
# ==========================================================================

@target("object_enumeration_finalization_bundle")
def gen_object_enumeration_finalization_bundle():
    stem = "object_enumeration_finalization_bundle"
    text = rs(stem)
    helper = "assert_browser_object_enumeration_finalization"

    c_fail = P.cite_line(text, r"assert!\(!output\.status\.success\(\)")
    c_or = P.cite_line(text, r'stderr\.contains\("E5506"\) \|\| stdout\.contains\("E5506"\)')
    blocks = comment_blocks(text)
    if len(blocks) != 1:
        raise AssertionError(f"expected one comment block, found {len(blocks)}")
    c_comment, comment_lines = blocks[0]
    comment = " ".join(comment_lines)

    program = check_program("app.${ext}", fixture_in_fn(
        text, "browser_object_enumeration_finalization_source"))

    disjunction = (
        "The source's own disjunction, carried verbatim per rule 11: "
        "`stderr.contains(\"E5506\") || stdout.contains(\"E5506\")`."
    )

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        [
            "RULE 12 (carry every source comment verbatim): this source carries exactly ONE",
            f"Rust comment block (:{c_comment}), attached to its single assert helper, which",
            "every case below is produced by. It is carried into every rationale, and the",
            "text is COPIED out of the `.rs` by this file's generator, never retyped.",
        ],
        "",
        P.matrix_arithmetic(
            test_fns=8, invocations=8, cases=2, axis="ext", values=EXTS4,
            non_axes=("json_output",),
            helpers=[(helper, 8, "ext(js/ts/jsx/tsx) x json_output(false/true), a complete\n"
                                 "    cross product. Every `#[test]` fn is one unlooped call\n"
                                 "    and the file contains no loop at all")]),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["app.${ext}"]),
        "",
        stale_name_note(8, "the body asserts the build FAILS closed, with an E5506\n"
                           "diagnostic. No bundle is emitted at all, so the `build_emits_*` naming\n"
                           "describes an outcome this test specifically pins as not happening."),
        "",
        P.rule13_header([
            "kali_bin", "browser_object_enumeration_finalization_source", helper]),
        "",
        P.ARGV_ORDER,
        "",
        [
            "ASSERTION SHAPE, mirrored from the source and nothing more.",
            f"`exit = \"failure\"` on the `kali build` process (:{c_fail}), plus the E5506",
            f"diagnostic (:{c_or}).",
            "RULE 11 -- THE DIAGNOSTIC CLAIM IS AN OR ACROSS TWO STREAMS, AND IT IS RESOLVED",
            "AGAINST THE REAL BINARY RATHER THAN REPRODUCED. The source accepts E5506 on",
            "EITHER stderr or stdout; the case format has no disjunction. Running the real",
            "`kali` over this fixture, for all four extensions and both output modes, shows",
            "the two modes differ and each is deterministic: in TEXT mode the code lands on",
            "stderr and never on stdout; in JSON mode it lands on stdout, inside the error",
            "envelope, and never on stderr. So the text case pins `stderr_contains` and the",
            "json case pins `stdout_contains`. This is a PRESENCE claim, so narrowing it is",
            "a verified strengthening -- every run satisfying the narrowed claim satisfies",
            "the original. (An ABSENCE OR may not be narrowed; rule 2. This is not one.)",
            "The source makes no other claim: no exit code, no envelope field, no metadata",
            "read, and no harness run -- the build never emits a bundle.",
        ],
    )

    cases = []
    for jo in (False, True):
        name = f"{'json_' if jo else ''}build_emits_object_enumeration_finalization"
        step = {
            "args": ["build", "--bundle", "--api", "browser"]
            + (["--output", "json"] if jo else []) + ["app.${ext}"],
            "exit": "failure",
        }
        step["stdout_contains" if jo else "stderr_contains"] = ["E5506"]
        cases.append({
            "name": name,
            "rationale": para(
                f"Migrated from browser_{stem}.rs, the four `{name}_in_*_input` fns (one per "
                "extension).",
                f"`{helper}` runs `kali build --bundle --api browser` on a program that "
                "probes return/throw/break/continue finalization through Object.keys, "
                "Object.values, Object.entries and Reflect.ownKeys, synchronously and under "
                "`for await`, and asserts the build fails closed with E5506.",
                comment,
                "MIGRATION NOTE (controller ruling 8): the source fn name says \"emits\", but "
                "the body asserts the build FAILS. The name is carried unchanged; see this "
                "file's header for why.",
                disjunction,
                ("Resolved against the real binary per rule 11: in json mode the code is "
                 "carried on stdout, inside the error envelope, so this case pins "
                 "`stdout_contains`."
                 if jo else
                 "Resolved against the real binary per rule 11: in text mode the code is "
                 "carried on stderr, so this case pins `stderr_contains`."),
                "Narrowing a presence OR to the stream that actually carries it is a "
                "verified strengthening, not a weakening."),
            "steps": [step],
        })
    return emit(header, {"ext": EXTS4}, {"app.${ext}": program}, cases)


# ==========================================================================
# F16. browser_object_enumeration_finalization_harness.rs -- 10 fns / 10 invs.
#      [matrix] DECLINED: json_output is exercised for `js` only.
# ==========================================================================

@target("object_enumeration_finalization_harness")
def gen_object_enumeration_finalization_harness():
    stem = "object_enumeration_finalization_harness"
    text = rs(stem)
    helper = "assert_browser_object_enumeration_finalization"

    c_fail = P.cite_line(text, r"assert!\(!output\.status\.success\(\)")
    c_threads = P.cite_line(text, r'\.arg\("--max-threads"\)')
    c_procs = P.cite_line(text, r'\.arg\("--max-spawned-processes"\)')
    c_env_const = P.cite_line(text, r"BROWSER_HARNESS_COMMAND_ENV")
    blocks = comment_blocks(text)
    if len(blocks) != 1:
        raise AssertionError(f"expected one comment block, found {len(blocks)}")
    c_comment, comment_lines = blocks[0]
    comment = " ".join(comment_lines)

    run_program = check_program("main.<ext>", fixture_in_fn(
        text, "browser_object_enumeration_finalization_run_source"),
        must_contain="Object.keys")
    test_program = check_program("smoke.test.<ext>", fixture_in_fn(
        text, "browser_object_enumeration_finalization_test_source"),
        must_contain="Kali.test")

    decline = [
        "`json_output` IS NOT EXERCISED UNIFORMLY ACROSS THE EXTENSIONS. Both commands run",
        "over js/ts/jsx/tsx in TEXT mode, but the source declares a JSON test for `js` ONLY",
        "-- there is no json ts, jsx or tsx test here, for either command. A four-value",
        "`ext` axis over a json case would manufacture six runs the source never made (rule",
        "2), and there is no per-case opt-out.",
    ]

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        [
            "RULE 12 (carry every source comment verbatim): this source carries exactly ONE",
            f"Rust comment block (:{c_comment}), attached to its single assert helper, which",
            "every case below is produced by. It is carried into every rationale, and the",
            "text is COPIED out of the `.rs` by this file's generator, never retyped.",
        ],
        "",
        P.matrix_declined(test_fns=10, invocations=10, cases=10, reason=decline),
        "",
        P.RULE6_ONE_TO_ONE,
        "",
        P.u2_source_file_wide(
            [f"main.{e}" for e in EXTS4] + [f"smoke.test.{e}" for e in EXTS4]),
        "",
        P.RULING7_NO_HOIST,
        "",
        stale_name_note(10, "the body asserts the command FAILS -- the source's only\n"
                            "process assertion is `assert!(!output.status.success(), ...)`."),
        "",
        P.rule13_header([
            "kali_bin", "browser_object_enumeration_finalization_run_source",
            "browser_object_enumeration_finalization_test_source", helper]),
        "",
        P.ARGV_ORDER,
        "",
        FAIL_CLOSED_NOTE,
        f"The `!success` assertion is at :{c_fail}. The argv carries `--max-threads 0`",
        f"(:{c_threads}) and `--max-spawned-processes 0` (:{c_procs}), and the environment",
        "carries the variable named by",
        f"`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` (:{c_env_const}), set to node.",
        "There is no json claim on either branch: the source parses no envelope at all, in",
        "either mode -- `--output json` changes only the argv here. That is also why this",
        "file's two json cases exist at all: they differ from their text siblings on argv",
        "alone, and dropping them would drop two real invocations.",
    )

    invocations = ([("run", e, False) for e in EXTS4]
                   + [("test", e, False) for e in EXTS4]
                   + [("run", "js", True), ("test", "js", True)])
    cases = []
    for command, ext, jo in invocations:
        entry = f"main.{ext}" if command == "run" else f"smoke.test.{ext}"
        name = (f"{'json_' if jo else ''}{command}_supports_object_enumeration_finalization_"
                f"when_browser_harness_is_configured_in_{ext}_input")
        cases.append({
            "name": name,
            "rationale": para(
                f"Migrated from browser_{stem}.rs, `{name}` -- one source `#[test]` fn, one "
                "case (no `[matrix]` in this file; see the header).",
                f"`{helper}` runs `kali {command} --api browser` with the browser harness "
                "backed by node and both concurrency limits pinned to 0, against a program "
                "that probes return/throw/break/continue finalization through Object.keys, "
                "Object.values, Object.entries and Reflect.ownKeys, synchronously and under "
                "`for await`, and asserts the command fails closed.",
                comment,
                "MIGRATION NOTE (controller ruling 8): the source fn name says \"supports\", "
                "but the body asserts the command FAILS. The name is carried unchanged; see "
                "this file's header for why.",
                "The whole claim is that the process fails, so the migrated step carries "
                "`exit = \"failure\"` and nothing else."),
            "steps": [{
                "args": (["--output", "json"] if jo else [])
                + [command, "--api", "browser", "--max-threads", "0",
                   "--max-spawned-processes", "0", entry],
                "env": {HARNESS_ENV: "node"},
                "exit": "failure",
            }],
        })
    source = {}
    for e in EXTS4:
        source[f"main.{e}"] = run_program
    for e in EXTS4:
        source[f"smoke.test.{e}"] = test_program
    P.assert_identical("the four main.<ext> fixtures", *[source[f"main.{e}"] for e in EXTS4])
    P.assert_identical("the four smoke.test.<ext> fixtures",
                       *[source[f"smoke.test.{e}"] for e in EXTS4])
    return emit(header, None, source, cases)


# ==========================================================================
# F6. browser_math_unsupported_member_calls_harness_jsx_tsx.rs
#     U4 TRIM-AND-KEEP: 3 of 6 `#[test]` fns migrate here; 3 are retained
#     hand-written per design spec 5.11 (an `errors` quantifier).
# ==========================================================================

@target("math_unsupported_member_calls_harness_jsx_tsx")
def gen_math_unsupported_member_calls():
    stem = "math_unsupported_member_calls_harness_jsx_tsx"
    text = rs(stem)
    helper = "assert_browser_harness_math_sqrt_success"
    retained_helper = "assert_browser_harness_unsupported_math_rejection"

    c_quantifier = P.cite_line(text, r"errors\.iter\(\)\.all\(")
    c_reject_helper = P.cite_line(text, rf"fn {retained_helper}\(")
    c_helper = P.cite_line(text, rf"fn {helper}\(")
    c_exit = P.cite_line(text, r"        output\.status\.success\(\),")
    c_json_first = P.cite_line(text, r'assert_eq!\(json\["schemaVersion"\], 1\);', expect=2)[1]
    c_json_success = P.cite_line(text, r'assert_eq!\(json\["success"\], true\);')
    c_json_errors = P.cite_line(text, r'assert!\(json\["errors"\]')
    c_json_count = P.cite_line(text, r'stdout\.matches\("1\.2649110640673518"\)\.count\(\)',
                               expect=2)[0]
    c_text_count = P.cite_line(text, r'stdout\.matches\("1\.2649110640673518"\)\.count\(\)',
                               expect=2)[1]
    c_build_skip_json, c_build_skip_text = P.cite_line(
        text, r'if command != "build"', label="the build guard", expect=2)
    c_build_skip = c_build_skip_json
    c_ok1 = P.cite_line(text, r'stdout\.contains\("ok 1"\)')
    c_bundle_flag = P.cite_line(text, r'cli\.arg\("--bundle"\);', expect=2)[1]
    c_env_const = P.cite_line(text, r"BROWSER_HARNESS_COMMAND_ENV", expect=2)[1]

    blocks = comment_blocks(text)
    if len(blocks) != 1:
        raise AssertionError(f"expected one comment block, found {len(blocks)}")
    c_doc, doc_lines = blocks[0]
    doc = " ".join(doc_lines)

    run_program = check_program("main.<ext>", fixture_in_fn(
        text, "browser_harness_run_source"))
    test_program = check_program("smoke.test.<ext>", fixture_in_fn(
        text, "browser_harness_test_source"))
    needle = "1.2649110640673518"
    if needle not in run_program or needle in text.split("fn browser_harness_run_source")[0]:
        pass  # the needle is an ASSERTION literal, not fixture text; see the shape block.

    decline = [
        "THE THREE MIGRATED `#[test]` FNS COVER DIFFERENT EXTENSION SETS. The `run` and",
        "`test` fns each loop over js/ts/jsx/tsx; the `build` fn loops over jsx/tsx ONLY.",
        "A four-value file-wide `ext` axis would fan the build cases over js and ts and",
        "invent four runs the source never made (rule 2); a two-value axis would drop half",
        "of the run and test invocations (rule 1).",
    ]

    partial = [
        "PARTIAL MIGRATION (U4 trim-and-keep) -- 3 of the source's 6 `#[test]` fns are",
        "migrated here. The other three, every `*_rejects_broader_math_atan2_*` fn, route",
        f"through `{retained_helper}` (:{c_reject_helper}),",
        "which asserts a QUANTIFIER over the JSON `errors` array,",
        f"`errors.iter().all(...)` (:{c_quantifier}). Design spec 5.4 offers only closed",
        "dotted-path indexing into JSON -- \"no slices, no wildcards, no negative-from-end",
        "indexing, no filters\" -- so a dotted path can pin the FIRST array element and",
        "nothing more; narrowing \"every error has this code\" to \"error 0 has this code\" is",
        "a weakening, and rule 1 forbids weakening. The human partner has ruled this shape a",
        "design spec 5.11 outlier: NO ASSERTION KEY IS BEING ADDED FOR IT.",
        "Each of those three fns calls the rejecting helper TWICE per extension, once with",
        "the JSON-output flag false and once true, and the true call is unconditional, so",
        "all three reach the quantifier and none can be split. The three migrated fns reach",
        f"`{helper}` (:{c_helper}) instead, which is the count",
        "shape and is fully expressible.",
        "",
        "THE `.rs` HAS SINCE BEEN TRIMMED to exactly those three retained tests plus the two",
        "fixture builders they read, carrying a `//!` retention header. TWO CONSEQUENCES A",
        "LATER READER MUST NOT MISREAD:",
        "  * EVERY `:N` LINE CITATION IN THIS FILE IS A PRE-TRIM LINE NUMBER. Audit and diff",
        "    this pair against the pre-trim source from git history, not against the working",
        "    tree.",
        "  * THE POST-TRIM PAIR IS THE WRONG COMPARISON FOR EVERY GATE, not just the audit.",
        "    The retained `.rs` carries the COMPLETE measured red-list (ruling 9) -- which",
        "    gates go red post-trim and which are green. Read it there, so there is one",
        "    source of truth.",
    ]

    # TWO builders, one per helper. Derived rather than asserted: fix round 2's
    # N2 was this block claiming a single shared builder, contradicting the
    # comment below it that already said "each of the two helpers".
    c_builder_reject, c_builder_success = P.cite_line(
        text, r"^    let mut cli = Command::new\(kali_bin\(\)\);$",
        label="the per-helper Command builders", expect=2)
    # Four matches: each of the two helpers has an argv `if json_output {` and an
    # assertion-branch one. Index 2 is the argv guard inside the MIGRATED helper.
    c_json_flag = P.cite_line(text, r"^    if json_output \{$", expect=4)[2]
    c_subcommand = P.cite_line(text, r"^    cli\.arg\(command\);$", expect=2)[1]
    c_bundle_if = P.cite_line(text, r"^    if bundle \{$", expect=2)[1]
    c_api = P.cite_line(text, r'^    cli\.arg\("--api"\)\.arg\("browser"\)\.arg', expect=2)[1]

    argv_order = [
        "ARGV ORDER is transcribed in the exact order the source's `Command` builder appends",
        "it, and THIS SOURCE DOES NOT HAVE THE FAMILY'S USUAL TWO SHAPES. It has TWO",
        f"builders, one per helper (`Command::new(kali_bin())` at :{c_builder_reject} and",
        f":{c_builder_success}) -- but they are not the family's build-vs-run/test pair: they",
        "append argv in the IDENTICAL order, so the two helpers differ in what they assert",
        "and not in how they invoke. That single order appends the `--output json` pair",
        f"BEFORE the subcommand (:{c_json_flag}, immediately above `cli.arg(command)` at",
        f":{c_subcommand}) -- for EVERY command, `build` included. That is not the shape most",
        "of this family's bundle helpers use, where the pair is appended after the subcommand",
        "and its flags, so the shared boilerplate is deliberately not used here:",
        "  * every command: `[--output json] <build|run|test> [--bundle] --api browser <entry>`",
        f"`--bundle` is appended after the subcommand (:{c_bundle_if}) and only when the",
        f"caller asks for it, which only the `build` fns do; `--api browser` and the entry",
        f"follow (:{c_api}).",
        "The source passes NO --max-threads and NO --max-spawned-processes argument, on any",
        "command, so neither appears on any argv below.",
        "The source passes an absolute `dir.path().join(filename)` as the entry; the case",
        "runner passes the bare filename relative to the trial dir, matching every previously",
        "shipped `browser/` case file.",
    ]

    count_keys = [
        "THE COUNT KEYS. The source makes exactly one occurrence-count claim per branch, and",
        "it is spelled with `==`, not `>=`:",
        f"  * `assert_eq!(stdout.matches(\"{needle}\").count(), 6)` against the JSON stdout",
        f"    leaf (:{c_json_count}) -> `json_count` with `path = \"stdout\"` and `exact = 6`.",
        f"  * the same claim against raw stdout (:{c_text_count}) -> `stdout_count` with",
        "    `exact = 6`.",
        "`exact`, never `at_least`: ruling 3's mirror-the-source direction makes an exact",
        "source assertion an exact pin, and `at_least = 6` would be a weakening the source",
        "never wrote. Counting is non-overlapping and left-to-right, as Rust's `str::matches`",
        "is.",
        f"BOTH COUNT CLAIMS ARE SKIPPED FOR `build` (:{c_build_skip}): the source guards them",
        "with `if command != \"build\"`, because a build emits no program output. The four",
        "build cases below therefore carry no count key at all, and adding one would be a",
        "rule-2 invention.",
    ]

    shape = [
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"`exit = \"success\"` on the one `kali` process (:{c_exit}).",
        f"json mode carries schemaVersion/command/success (:{c_json_first}-{c_json_success})",
        f"and the empty `errors` array (:{c_json_errors}), plus the `json_count` claim above",
        "for run and test. It reads NO payload field and NO stderr leaf, so neither is",
        "written.",
        f"Text mode carries the `stdout_count` claim and, for `test` only,",
        f"`.contains(\"ok 1\")` (:{c_ok1}).",
        f"`build` additionally passes `--bundle` (:{c_bundle_flag}), and the environment",
        "carries the variable named by",
        f"`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` (:{c_env_const}), set to node.",
        "The source passes no --max-threads / --max-spawned-processes arguments, so neither",
        "appears on argv.",
    ]

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        [P.extra_ok(f"{st}.{e}", FORMAT_BUILT_FILENAME)
         for st in ("main", "smoke.test") for e in EXTS4],
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        partial,
        "",
        [
            f"RULE 12 (carry every source comment verbatim): the source's one comment block",
            f"(:{c_doc}) is the `///` doc on `{helper}`, the",
            "helper every case below is produced by, so it is carried into every rationale.",
            "It is also this file's rule-13 material -- same text, one obligation discharged",
            "twice over -- and it is COPIED out of the `.rs` by this file's generator, never",
            "retyped.",
        ],
        "",
        P.matrix_declined(test_fns=6, invocations=20, cases=20, reason=decline),
        "20 invocations, not 40: the other 20 belong to the three retained fns and are not",
        "this file's to carry.",
        "",
        P.RULE6_ONE_TO_ONE,
        "Each source `#[test]` fn here is a `for extension in [...]` loop making two",
        "invocations per extension, so one fn maps to 8 cases (or 4, for the jsx/tsx-only",
        "build fn); the mapping is still 1:1 from invocation to case.",
        "",
        P.u2_source_file_wide([f"main.{e}" for e in EXTS4]
                              + [f"smoke.test.{e}" for e in EXTS4]),
        "",
        P.RULING7_NO_HOIST,
        "",
        P.rule13_header(
            ["kali_bin", "browser_harness_run_source", "browser_harness_test_source"],
            extra=[f"`{helper}` DOES carry a `///` doc comment, and it is",
                   "carried verbatim into the rationale of every case below -- every one of them",
                   "is produced by that helper. Its text states what a PASS means for these",
                   "cases (which value node prints for Math.sqrt(1.6), and why it appears exactly",
                   "six times), so it is claim prose in ruling 6's sense and not runner",
                   "infrastructure."]),
        "",
        argv_order,
        "",
        count_keys,
        "",
        shape,
    )

    invocations = ([("run", e, jo) for e in EXTS4 for jo in (False, True)]
                   + [("test", e, jo) for e in EXTS4 for jo in (False, True)]
                   + [("build", e, jo) for e in ("jsx", "tsx") for jo in (False, True)])
    cases = []
    for command, ext, jo in invocations:
        entry = f"smoke.test.{ext}" if command == "test" else f"main.{ext}"
        name = (f"{'json_' if jo else ''}{command}_supports_math_sqrt_member_calls_"
                f"in_{ext}_input")
        args = (["--output", "json"] if jo else []) + [command]
        if command == "build":
            args += ["--bundle"]
        args += ["--api", "browser", entry]
        step = {"args": args, "env": {HARNESS_ENV: "node"}, "exit": "success"}
        if jo:
            step["json"] = {"schemaVersion": 1, "command": command,
                            "success": True, "errors": []}
            if command != "build":
                step["json_count"] = [{"path": "stdout", "needle": needle, "exact": 6}]
        else:
            if command != "build":
                step["stdout_count"] = [{"needle": needle, "exact": 6}]
            if command == "test":
                step["stdout_contains"] = ["ok 1"]
        source_fn = (
            f"{command}_supports_math_sqrt_member_calls_in_browser_api_surface_with_harness_"
            + ("jsx_and_tsx_input" if command == "build" else "js_ts_jsx_and_tsx_input"))
        cases.append({
            "name": name,
            "rationale": para(
                f"Migrated from browser_{stem}.rs, `{source_fn}` -- one of that fn's "
                f"`for extension in [...]` loop iterations, in its "
                f"{'json' if jo else 'text'}-output invocation.",
                f"`{helper}` runs `kali {command} --api browser` "
                + ("with `--bundle` " if command == "build" else "")
                + "with the browser harness backed by node, against a program that calls "
                "Math.sqrt(1.6) through six access forms.",
                doc,
                ("This case makes no count claim: the source guards both count assertions "
                 f"with `if command != \"build\"` (:{c_build_skip}), because a build emits no "
                 "program output." if command == "build" else
                 P.ruling3_count_exact(f'"{needle}"', 6,
                                       key="json_count" if jo else "stdout_count")),
                (f"For `test` the source also asserts `.contains(\"ok 1\")` (:{c_ok1}), which "
                 "the json branch does not make."
                 if command == "test" and not jo else None)),
            "steps": [step],
        })
    source = {}
    for e in EXTS4:
        source[f"main.{e}"] = run_program
    for e in EXTS4:
        source[f"smoke.test.{e}"] = test_program
    P.assert_identical("the four main.<ext> fixtures", *[source[f"main.{e}"] for e in EXTS4])
    P.assert_identical("the four smoke.test.<ext> fixtures",
                       *[source[f"smoke.test.{e}"] for e in EXTS4])
    return emit(header, None, source, cases)


def _computed_keys_json(command, pin):
    """This file's harness envelope: `run` asserts the PAYLOAD `exitCode` only.

    `math_shapes.envelope_harness` emits `exitCode` at both the envelope and the
    payload level for `run`, because that is what most of the family asserts.
    This source asserts only the payload one, and rule 2 forbids adding the
    other, so the envelope-level key is dropped here rather than by changing the
    shared builder.
    """
    j = _harness_json_with_stdout(command, pin, stderr=True, errors=True)
    if command == "run":
        j.pop("exitCode", None)
    return j


def _long_needle(text, prefix):
    """The one `.contains(<literal>)` needle in the file that starts with `prefix`.

    Pulled out of the `.rs` rather than retyped: it is 40+ characters of `2\\n`
    and `-3\\n` and a retyped one differing by a single repetition would still
    look right and would silently weaken the claim.
    """
    from lexer import find_string_literals
    hits = {lit["value"] for lit in find_string_literals(text)
            if lit["value"].startswith(prefix)}
    if len(hits) != 1:
        raise AssertionError(f"{len(hits)} literal(s) start with {prefix!r}, wanted 1")
    return hits.pop()


def _harness_json_with_stdout(command, stdout_pin, *, extra_payload=None,
                              stderr=False, errors=False):
    """`envelope_harness` with an exact `json.stdout` pin spliced in.

    `math_shapes.envelope_harness` has no `stdout` parameter because most of
    this migration asserts a COUNT on that leaf rather than an equality; these
    files instead make a plain `.contains` (or an `assert_eq!`) against
    json["stdout"], which per ruling 3 becomes an exact pin. Spliced here
    rather than by changing the shared builder.
    """
    base = envelope_harness(command, stderr=stderr, errors=errors,
                            extra_payload=extra_payload)
    out = {}
    for key, value in base.items():
        if key in ("stderr", "errors") and "stdout" not in out:
            out["stdout"] = stdout_pin
        out[key] = value
    out.setdefault("stdout", stdout_pin)
    return out


# ==========================================================================
# emit6a -- `emit` plus a `[constants]` table.
# ==========================================================================

def emit6a(header_lines, constants, matrix, source, cases):
    """`case_emit.emit` with a `[constants]` table spliced in.

    `case_emit.emit` predates any `browser/` file needing `[constants]` and has
    no parameter for it. Rather than change a module five shipped batches
    depend on, the table is rendered here and inserted immediately before the
    first structural table `emit` produces. Keeps `emit` the single renderer
    for everything else, including the fixed step-key order.
    """
    from toml_emit import toml_string
    text = emit(header_lines, matrix, source, cases)
    if not constants:
        return text
    block = ["[constants]"]
    for name, value in constants.items():
        block.append(f"{name} = {toml_string(value, multiline=False)}")
    block.append("")
    lines = text.split("\n")
    for i, line in enumerate(lines):
        if line.startswith("[matrix]") or line.startswith("[source]") or line.startswith("[[case]]"):
            return "\n".join(lines[:i] + block + lines[i:])
    raise AssertionError("no structural table found to insert [constants] before")


def main(argv):
    names = [a for a in argv if not a.startswith("--")] or sorted(REGISTRY)
    unknown = [n for n in names if n not in REGISTRY]
    if unknown:
        raise SystemExit(f"unknown target(s): {unknown}\nknown: {sorted(REGISTRY)}")
    for name in names:
        print(f"--- {name}")
        write(os.path.join(CASES, f"{name}.toml"), REGISTRY[name]())
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
