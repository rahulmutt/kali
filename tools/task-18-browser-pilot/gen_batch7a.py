#!/usr/bin/env python3
r"""Generate the Task 18 batch 7A case files (13 migrated targets).

Batch 7A migrates the thirteen `browser_object_*` targets (171 `#[test]` fns)
registered below. There are NO design-spec 5.11 retentions in this batch:
`find_fixture_self_inspection.py` puts the whole unadjudicated set outside it,
and the `.matches(` / `.lines()` / `.iter().all|any(` / `#[path` census returns
zero hits across all thirteen. Both scans were re-derived at dispatch and their
commands and outputs are recorded in the batch report.

WHY A GENERATOR AND NOT THIRTEEN HAND-WRITTEN FILES. Same reason batch 5 and
batch 6A used one: batch 4 shipped cross-file prose divergence that every
per-file gate passed individually, because no gate reads `#` header prose or
`rationale` wording (U8). Every recurring sentence is therefore CALLED from
`batch5_prose`, not retyped; the batch-6A-local blocks this batch also needs are
IMPORTED from `gen_batch6a` rather than copied, which is the same discipline one
level up. This module writes only the PER-FILE spec -- the program under test,
the invocation arithmetic, the assertion inventory and the `:N` citations --
which is what review has to read.

Nothing under `tools/` or `scripts/` is modified by this batch; this file and
`batch7a_captures.py` are added and everything else is used as it stands.

CITATIONS. Every `:N` below is produced by `batch5_prose.cite_line(rs_text,
regex)` at generation time, by SEARCHING the source for the construct. None is
computed by arithmetic and none is carried over from an earlier measurement.
`cite_line` raises unless its anchor matches exactly the expected number of
times, so a vanished or ambiguous anchor is a generator error rather than a
silently wrong number.

RULE 8 / RULE 9. Five of these sources build a fixture with `format!`, with a
`source.replace(...)`, or one level removed inside kali_common::object. None of those
texts is hand-derived: they are the byte-exact output of executing the real code
and they live in `batch7a_captures.py`, whose docstring records the exact capture
procedure. `check_captured` re-checks each one against its own `.rs` before it is
emitted, so a capture taken before a source edit fails the generator instead of
shipping a program that is no longer the program under test.

RULE 10. No fixture and no harness body in this batch contains a genuine JS
template literal, so no file here declares `[constants] dollar`. That is CHECKED
rather than assumed: `assert_no_template_literals` greps every emitted `[source]`
value and every step `body` and raises on `${`.

RULE 11. One source in this batch makes an OR-shaped assertion
(`stderr.contains("E5506") || stdout.contains("E5506")`). `_stream` resolves it
against the real binary per output mode -- refusing to answer if the binary is
absent or if the observation is ambiguous -- and the source's disjunction is
carried into every affected rationale.

U9. Every exact pin is live-captured from the real built `kali` via
`kali_run.py`, for EVERY cell of the file's matrix axis, and
`batch5_prose.assert_identical` asserts the cells agree with each other AND with
the embedded constant before one pin is emitted. See `_pin`.

Run: python3 gen_batch7a.py [name ...]   (no args = all)
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
KALI_COMMON_OBJECT = os.path.join(REPO, "crates/kali_common/src/object.rs")

from case_emit import emit, fixture_in_fn, fixture_starting, write  # noqa: E402
from lexer import find_string_literals  # noqa: E402
from math_shapes import (  # noqa: E402
    META, envelope_build, envelope_harness, rule12_no_comments_prose,
)
import batch5_prose as P  # noqa: E402
import batch7a_captures as C  # noqa: E402

# Reused, not retyped. `batch5_prose`'s own note says a recurring fact belongs in
# one place and that the second call site is the moment to hoist it; these blocks
# already exist in batch 6A's generator and this batch is their second caller, so
# they are imported. Hoisting them INTO `batch5_prose` was not available: batch
# 7A's dispatch forbids modifying any existing file under `tools/`.
from gen_batch6a import (  # noqa: E402
    FAIL_CLOSED_NOTE, check_captured, check_program, comment_blocks, hdr,
    kali_common_doc,
)

EXTS4 = ["js", "ts", "jsx", "tsx"]
HARNESS_ENV = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"
# ^ the value of `kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV`. Every
# source in this batch passes the CONSTANT by name rather than spelling the
# literal, so `assert_env_name` reads it out of the contract crate instead of
# assuming it.

REGISTRY = {}


def target(name):
    def deco(fn):
        REGISTRY[name] = fn
        return fn
    return deco


def rs(name):
    """The source a case file is generated FROM.

    Batch 7A declares no U4 trim-and-keep retention, so no case file's source is
    a pre-trim blob and this is a plain read. `gen_batch6a.rs`'s `PRE-TRIM REF:`
    branch is deliberately not reproduced -- a code path no file in this batch
    exercises is a code path nothing here tests -- but the precondition that
    makes it unnecessary IS checked, so a later trim cannot silently regenerate
    a smaller case file from a trimmed source.
    """
    text = open(os.path.join(TESTS, f"browser_{name}.rs")).read()
    if text.startswith("//!"):
        raise AssertionError(
            f"browser_{name}.rs has a `//!` header -- batch 7A declares no retentions, so "
            "this is an unexpected trim; regenerate from the pre-trim blob instead")
    return text


def assert_env_name():
    """`KALI_BROWSER_BUNDLE_HARNESS_COMMAND` is the value of the constant every
    source in this batch passes by name. Read, not assumed."""
    text = ""
    for root, _dirs, files in os.walk(
            os.path.join(REPO, "crates/kali_runtime_contract/src")):
        for f in sorted(files):
            if f.endswith(".rs"):
                text += open(os.path.join(root, f)).read()
    m = re.search(r"BROWSER_HARNESS_COMMAND_ENV\s*:\s*&(?:'static\s+)?str\s*=\s*\"([^\"]+)\"",
                  text)
    if not m:
        raise AssertionError("cannot read BROWSER_HARNESS_COMMAND_ENV from kali_runtime_contract")
    if m.group(1) != HARNESS_ENV:
        raise AssertionError(
            f"BROWSER_HARNESS_COMMAND_ENV is {m.group(1)!r}, not {HARNESS_ENV!r}")
    return HARNESS_ENV


def test_fns(rs_text):
    """Every `#[test] fn` name in the source, in file order."""
    return re.findall(r"#\[test\]\s*\n\s*fn\s+([a-z0-9_]+)", rs_text)


def assert_fns(rs_text, *names):
    """A case name may only be derived from a `#[test]` fn that really exists.

    U8's gate resolves backticked names in PROSE, and its prefix rule would
    accept a case name derived from a fn that has since been renamed, so the
    derivation itself is checked here at the point it is made.
    """
    have = set(test_fns(rs_text))
    missing = [n for n in names if n not in have]
    if missing:
        raise AssertionError(f"not `#[test]` fns in this source: {missing}")
    return names


EXT_SUFFIX = re.compile(r"_(in_)?(?:js|ts|jsx|tsx)(?:_(?:js|ts|jsx|tsx))*_input$")


def strip_ext_suffix(fn):
    """`..._in_js_input`, `..._with_harness_js_input`,
    `..._in_js_ts_jsx_tsx_input` -> the stem a matrix-folded case is named after.

    Returns `(stem, glob)`, where `glob` is the shape of the family the fold
    covers -- `<stem>_in_*_input` or `<stem>_*_input`, whichever this source
    actually spells. Two of this batch's sources use the second form, and a
    rationale that quoted the first would be citing a family of fns that does
    not exist (U8 is exactly about a rationale's own claims being audited by
    nothing).
    """
    m = EXT_SUFFIX.search(fn)
    if not m:
        raise AssertionError(f"`{fn}` does not end in an `_<ext...>_input` suffix")
    stem = fn[:m.start()]
    return stem, f"{stem}_{'in_' if m.group(1) else ''}*_input"


def cite(rs_text, snippet, *, occurrence=1, expect=1):
    """A citation the gate can actually READ: `` `<snippet>` (:N) ``.

    Ruling 11 exempts `:N` from "no figure an edit can move" ONLY because it is
    mechanically gated, and `batch5_crosscheck.py`'s reader needs a backticked
    construct within 40 characters of the number -- a bare `:N` in prose matches
    nothing and reports clean whether it is right or wrong. Batch 7's own fix
    round 1 found two such citations hiding in shipped files, so this generator
    cannot write a bare one: every citation it emits is produced HERE, from the
    snippet, by SEARCHING the source for it. The number and the construct beside
    it therefore cannot disagree, and the gate re-resolves both.

    `expect` is the number of lines the snippet occurs on; `occurrence` picks
    which. Both are required rather than defaulted-and-forgiving, so a snippet
    that becomes ambiguous after a source edit fails the generator instead of
    silently citing the wrong one of two identical lines.
    """
    if "`" in snippet or "\n" in snippet:
        raise AssertionError(f"a cited snippet cannot contain a backtick or newline: {snippet!r}")
    if not 3 <= len(snippet) <= 200:
        raise AssertionError(
            f"cited snippet is {len(snippet)} chars; batch5_crosscheck's SNIPPET_MAX is 200 "
            f"and its minimum is 3: {snippet!r}")
    hits = [i + 1 for i, line in enumerate(rs_text.split("\n")) if snippet in line]
    if len(hits) != expect:
        raise AssertionError(
            f"citation snippet {snippet!r}: {len(hits)} match(es) {hits}, wanted {expect}")
    return f"`{snippet}` (:{hits[occurrence - 1]})"


def cites(rs_text, snippet, expect):
    """`cite` for every occurrence of a snippet, in source order."""
    return [cite(rs_text, snippet, occurrence=i + 1, expect=expect) for i in range(expect)]


def assert_count(rs_text, needle, expect):
    """Assert a source contains `expect` occurrences of `needle`, and print NO
    number into the artifact.

    Ruling 15's third answer applied to a prose quantifier's supporting figure:
    the count is what the sentence rests on, so it is CHECKED here on every
    generator run, and the line numbers it would otherwise quote are deleted
    rather than written into a header where nothing re-resolves them.
    """
    got = sum(line.count(needle) for line in rs_text.split("\n"))
    if got != expect:
        raise AssertionError(f"{needle!r}: {got} occurrence(s), wanted {expect}")
    return expect


def contains_needles(rs_text, receiver):
    """Every `<receiver>.contains(<string literal>)` needle, in source order,
    de-duplicated.

    Pulled out of the `.rs` rather than retyped. One source in this batch makes
    thirteen distinct `stdout.contains(...)` claims whose needles differ by a
    single word; a retyped list that dropped or merged one would still look
    right and would silently weaken the claim (rule 1).
    """
    out = []
    for lit in find_string_literals(rs_text):
        start = lit["start"]
        prefix = rs_text[max(0, start - 60):start]
        if re.search(re.escape(receiver) + r"\s*\n?\s*\.contains\(\s*$", prefix):
            if lit["value"] not in out:
                out.append(lit["value"])
    if not out:
        raise AssertionError(f"no `{receiver}.contains(<literal>)` sites found")
    return out


# --------------------------------------------------------------------------
# U9 live capture, and rule 11's live resolution.
# --------------------------------------------------------------------------

def _pin(label, embedded, cells):
    """Re-capture an exact `json.stdout` pin from the real binary for every
    matrix cell, and assert every cell agrees with every other AND with the
    embedded constant.

    `assert_identical` over N copies of one constant would prove nothing; the
    capture is what makes the assertion real. Skipped LOUDLY if the built binary
    is absent, rather than reporting a green that was never run.
    """
    from kali_run import KALI, run_kali
    if not os.path.exists(KALI):
        print(f"  !! {KALI} absent -- pin {label} NOT re-captured this run")
        return embedded
    captured = []
    for entry, program, command in cells:
        args = ["--output", "json", command, "--api", "browser",
                "--max-threads", "0", "--max-spawned-processes", "0", entry]
        code, out, err, _dir = run_kali({entry: program}, args,
                                        env={HARNESS_ENV: "node"})
        if code != 0:
            raise AssertionError(f"live capture failed for {label} {entry}: {err!r}")
        captured.append(_json.loads(out)["stdout"])
    return P.assert_identical(f"{label}, live-captured over {len(cells)} cell(s), "
                              "against the embedded constant", embedded, *captured)


def _stream(label, cells):
    """Rule 11: resolve an OR over two streams against the real binary.

    Returns the ONE stream that actually carries the needle. Raises if the cells
    disagree, or if a cell has the needle in both streams or neither: a
    disjunction that does not resolve to exactly one stream is not narrowable,
    and guessing would be precisely the weakening rule 11 exists to prevent.
    """
    from kali_run import KALI, run_kali
    if not os.path.exists(KALI):
        raise AssertionError(
            f"{KALI} absent -- the rule-11 OR for {label} cannot be resolved by guessing; "
            "build the binary and re-run the generator")
    answers = set()
    for entry, program, args, needle in cells:
        _code, out, err, _dir = run_kali({entry: program}, args)
        in_out, in_err = needle in out.decode(), needle in err.decode()
        if in_out == in_err:
            raise AssertionError(
                f"{label} {entry}: {needle!r} in stdout={in_out} stderr={in_err} -- the OR "
                "does not resolve to exactly one stream")
        answers.add("stdout" if in_out else "stderr")
    if len(answers) != 1:
        raise AssertionError(f"{label}: cells disagree on the carrying stream: {answers}")
    return answers.pop()


# --------------------------------------------------------------------------
# Shared header/rationale chunks new in this batch.
# --------------------------------------------------------------------------

def matrix_block(*, test_fns, invocations, cases, axis, values, helpers,
                 non_axes=("command", "json_output"), non_axis_lines=None):
    """`batch5_prose.matrix_arithmetic`, optionally with a file-specific reason
    for the non-axis dimensions.

    `matrix_not_axes`'s stock sentences say `json_output` "switches between a
    text claim and a JSON-envelope claim" and `command` "switches the envelope's
    payload". Both are true of most of this family and FALSE of the five
    fail-closed targets here, whose only claim in either output mode is
    `exit = "failure"` -- prose describing a state the file does not have, which
    is the failure class `batch5_prose` exists to stop. Those files pass their
    own `non_axis_lines`.

    The stock tail is located by comparison and its absence RAISES, so a change
    to the shared block cannot silently leave two non-axis paragraphs in a file.
    """
    lines = P.matrix_arithmetic(test_fns=test_fns, invocations=invocations, cases=cases,
                                axis=axis, values=values, helpers=helpers, non_axes=non_axes)
    if non_axis_lines is None:
        return lines
    tail = P.matrix_not_axes(non_axes)
    if lines[-len(tail):] != tail:
        raise AssertionError(
            "batch5_prose.matrix_arithmetic no longer ends with matrix_not_axes' block; "
            "the replacement below would append rather than replace")
    return lines[:-len(tail)] + non_axis_lines


FAIL_CLOSED_NON_AXES = [
    "`command` and `json_output` are NOT matrix axes, per rule 7 and design spec 5.6's own",
    "note, and the reason here is not the usual one: this target asserts nothing but process",
    "failure, so neither dimension changes what is asserted -- they change the argv and, for",
    "`command`, the entry filename.",
    "`command` selects a different `[source]` entry (`main.<ext>` for run, `smoke.test.<ext>`",
    "for test), and a `[matrix]` axis substitutes ONE string uniformly across every case.",
    "`json_output` appends an argv PAIR rather than substituting a value.",
    "Each is written as sibling `[[case]]` entries instead, which is also what rule 6",
    "requires: the source has its own `#[test]` fns for every combination.",
]

FAIL_CLOSED_NON_AXES_BUILD = [
    "`json_output` is NOT a matrix axis, per rule 7 and design spec 5.6's own note, and the",
    "reason here is not the usual one: this target asserts nothing but process failure, so",
    "the output mode changes no assertion -- it appends an argv PAIR rather than",
    "substituting a value, which a `[matrix]` axis cannot express.",
    "It is written as sibling `[[case]]` entries instead, which is also what rule 6",
    "requires: the source has its own `#[test]` fns for both.",
]


def rule12_carried(stem, rs_text, *, reaching):
    """The header block for a source that DOES carry Rust comments.

    `math_shapes.rule12_no_comments_prose` is the no-comment discharge and RAISES
    when a source has prose; nine of this batch's thirteen sources have prose, so
    this is its counterpart. The comment text itself is EXTRACTED by
    `comment_blocks`, never retyped -- rule 12 is explicit that an em-dash
    retyped as `--` is a violation the mechanical checker catches.
    """
    blocks = comment_blocks(rs_text)
    if not blocks:
        raise AssertionError(f"browser_{stem}.rs carries no Rust comment block")
    distinct = []
    for _start, texts in blocks:
        joined = " ".join(t.strip() for t in texts if t.strip())
        if joined and joined not in distinct:
            distinct.append(joined)
    lines = [
        "RULE 12 (carry every source comment verbatim) -- THIS SOURCE HAS PROSE, and it is",
        "carried into EVERY case's own `rationale`, not just into this header.",
        f"`grep -nE '^\\s*//'` over tests/browser_{stem}.rs returns "
        f"{sum(len(b[1]) for b in blocks)} Rust comment line(s)",
        f"in {len(blocks)} contiguous block(s), of {len(distinct)} distinct text(s):",
    ]
    for text in distinct:
        lines.append(f"  * \"{text}\"")
    lines += [
        f"Every block sits inside `{reaching}`,",
        "which every `[[case]]` below reaches, so U6's bottom-up attribution puts all of it in",
        "every rationale: the attribution is per helper and this helper is universal in this",
        "file, which is NOT the over-attribution U6 forbids (that is copying a block into cases",
        "its producing helper does not reach).",
        "A pointer (\"see the file header\") would not satisfy rule 12: a reader of one failing",
        "trial sees only that trial's `rationale`.",
    ]
    if len(blocks) > len(distinct):
        lines += [
            f"The {len(blocks)} blocks are {len(distinct)} distinct text(s) repeated; the",
            "rationale below carries each distinct text once. `comment_coverage.py` checks each",
            "comment LINE's text for membership in each rationale, so one copy discharges every",
            "repetition, and N identical copies would be noise rather than coverage.",
        ]
    return lines


def rule12_none(stem, rs_text):
    """The rule-12 discharge for a source that carries NO Rust comment prose.

    `math_shapes.rule12_no_comments_prose` is the shared version and it is CALLED
    below, purely for its guard: it raises if the source does carry Rust
    comments, which is what makes the accompanying `--allow-empty` an honest
    discharge rather than a vacuous green. Its RENDERING is not used, for one
    mechanical reason: it emits `The N other `//` occurrence(s) in the file
    (:11) sit ...`, whose `:11` sits beside a two-character backtick.
    `batch5_crosscheck.py`'s `CITE` needs a backticked construct of at least
    three characters, so that citation is UNGATED -- nothing re-resolves it and
    it reports clean whether it is right or wrong (ruling 11) -- and the shipped
    files that carry it are in `UNGATED_REDLIST`. Batch 7A's dispatch forbids
    editing anything under `tools/` that already exists, and `citation_sweep.sh`
    records that the disposition for an ungated citation is REWORD, not
    red-list: "the artifact was made gateable rather than the gate made blind."
    So the block is rendered here in a gateable, figure-free form instead.

    It also drops the `N other `//` occurrence(s)` COUNT. That figure is neither
    gated nor pinned, and ruling 15's third answer is to delete such a figure
    rather than record a command beside it. What replaces it is a check: the
    generator asserts that every `//` outside a Rust comment line is a
    `// kali-tree-shake:` marker, and names the markers rather than counting
    them.
    """
    rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem)
    lines = rs_text.split("\n")
    stray = [(i + 1, ln.strip()) for i, ln in enumerate(lines)
             if "//" in ln and not re.match(r"\s*//", ln)]
    out = [
        "RULE 12 (carry every source comment verbatim): `grep -nE '^\\s*//'` over",
        f"tests/browser_{stem}.rs returns NOTHING -- the file has no Rust comment lines at",
        "all. The generator re-derives that on every run and RAISES rather than emitting this",
        "block if the source ever grows one, so the `--allow-empty` discharge below cannot",
        "become a vacuous green by drift.",
    ]
    if stray:
        for _n, ln in stray:
            if "// kali-tree-shake:" not in ln:
                raise AssertionError(
                    f"browser_{stem}.rs has a non-marker `//` outside a comment line: {ln!r}")
        markers = sorted({ln[ln.index("//"):] for _n, ln in stray})
        out.append("The file's only other `//` sequences are kali-tree-shake markers inside its")
        out.append("JS fixture bodies -- program text carried verbatim into [source], not Rust")
        out.append("prose. The generator checks that every one of them is such a marker:")
        for marker in markers:
            out.append(f"  * {marker}")
    else:
        out.append("The file contains no `//` of any kind -- it declares no bundle fixture, so")
        out.append("there is not even a kali-tree-shake marker in it.")
    out.append("There is therefore no prose to move into any `rationale`, and")
    out.append("comment_coverage.py is run with --allow-empty for this pair.")
    return out


def rule12_rationale(rs_text, stem):
    """The source's comment text, joined, ready to append to a `rationale`.

    `comment_coverage.py` normalises whitespace and then requires each comment
    LINE's text to appear in each case's rationale, so joining a block's lines
    with single spaces discharges every line of that block at once.
    """
    seen, out = [], []
    for _start, texts in comment_blocks(rs_text):
        joined = " ".join(t.strip() for t in texts if t.strip())
        if joined and joined not in seen:
            seen.append(joined)
            out.append(joined)
    return (f" RULE 12 -- the Rust comment prose of browser_{stem}.rs, carried verbatim: "
            + " ".join(f"\"{t}\"" for t in out))


RULE13_KALI_COMMON_FNS = [
    "object_has_own_combined_frozen_callable_condition_source",
    "object_has_own_frozen_callable_condition_source",
    "object_has_own_property_call_frozen_callable_condition_source",
    "object_has_own_frozen_callable_source",
    "object_has_own_property_call_frozen_callable_source",
    "object_has_own_property_call_binding_source",
    "object_has_own_property_call_source",
]


def rule13_kali_common_docs():
    """The `///` doc of every kali_common::object helper in the has-own call
    chain, EXTRACTED from the crate rather than retyped."""
    return [(fn, kali_common_doc(fn, KALI_COMMON_OBJECT)) for fn in RULE13_KALI_COMMON_FNS]


def rule13_kali_common_block(chain_fns, *, runner_exemption):
    docs = rule13_kali_common_docs()
    lines = P.rule13_header(chain_fns, docs_carried=[d for _f, d in docs],
                            runner_exemption=runner_exemption)
    lines.append("The kali_common::object helpers whose output lands in [source], with the")
    lines.append("`///` doc carried into every rationale they reach:")
    for fn, doc in docs:
        # The helper NAMES are deliberately not backticked. U8's gate
        # (`check_rationale_fn_names.py`) resolves every backticked lower-case
        # identifier against the source `.rs`'s own fn list, and these live in
        # `kali_common`, which that file only `use`s -- so backticking them turns
        # the U8 arm red on correct prose, exactly as backticking a JS
        # `import()` call did for U5's shared block. Same fix: name them
        # plainly.
        lines.append(f"  * {fn}")
        lines.append(f"      \"{doc}\"")
    return lines


def rule13_kali_common_rationale():
    docs = rule13_kali_common_docs()
    return (" " + P.rule13_carried([d for _f, d in docs])
            + " Those docs belong to, in call-chain order: "
            + ", ".join(fn for fn, _d in docs)
            + " (named plainly rather than backticked: U8's gate resolves backticked"
              " identifiers against this source's own fn list and these live in kali_common)."
            )


def assert_bodies_identical(label, source, keys):
    """Ruling 7's MANDATORY mechanical duplicate-identity assertion.

    Ruling 7 declines U13's hoist for `browser/` but makes the identity check
    mandatory: "duplication without a check is just duplication". Every group of
    `[source]` keys this batch fills from one fixture builder is compared here,
    byte for byte, before emission.
    """
    return P.assert_identical(f"{label} ({', '.join(keys)})", *[source[k] for k in keys])


ARGV_ORDER_HARNESS_ONLY = [
    "ARGV ORDER is transcribed in the exact order the source's `Command` builder appends it",
    "and is not normalised here:",
    "  * run/test: `[--output json] <run|test> --api browser --max-threads 0",
    "             --max-spawned-processes 0 <entry>` -- the `--output json` pair is appended",
    "             BEFORE the subcommand, and both thread flags are always passed.",
    "The source passes an absolute `dir.path().join(filename)` as the entry; the case runner",
    "passes the bare filename relative to the trial dir, matching every previously shipped",
    "`browser/` case file.",
]

ARGV_ORDER_BUILD_ONLY = [
    "ARGV ORDER is transcribed in the exact order the source's `Command` builder appends it",
    "and is not normalised here:",
    "  * build: `build --bundle --api browser [--output json] <entry>` -- the `--output json`",
    "           pair is appended AFTER the subcommand and its flags.",
    "The source passes an absolute `dir.path().join(filename)` as the entry; the case runner",
    "passes the bare filename relative to the trial dir, matching every previously shipped",
    "`browser/` case file.",
]

ARGV_ORDER_CHECK_BUILD_HARNESS = [
    "ARGV ORDER is transcribed in the exact order each source helper's `Command` builder",
    "appends it, which differs between the three helper shapes and is not normalised here:",
    "  * check:   `check --api browser [--output json] <entry>` -- the `--output json` pair",
    "             is appended AFTER the subcommand and its flags.",
    "  * build:   `build --bundle --api browser [--output json] <entry>` -- likewise AFTER.",
    "  * run/test: `[--output json] <run|test> --api browser --max-threads 0",
    "             --max-spawned-processes 0 <entry>` -- the `--output json` pair is appended",
    "             BEFORE the subcommand.",
    "The source passes an absolute `dir.path().join(filename)` as the entry; the case runner",
    "passes the bare filename relative to the trial dir, matching every previously shipped",
    "`browser/` case file.",
]

NO_TEMPLATE_LITERAL = [
    "RULE 10 -- NOT NEEDED HERE, and that is checked rather than assumed. `expand.rs`'s",
    "`substitute()` hard-fails on any `${...}` it cannot resolve, so a fixture carrying a",
    "genuine JS template literal must declare `[constants] dollar = \"$\"` and spell the",
    "genuine `${` as `${dollar}{`. No `[source]` value and no step `body` in this file",
    "contains `${` at all -- the generator greps every emitted string and raises if one does",
    "-- so this file declares no `[constants]` table.",
]

RUNNER_HARNESS_STEP = [
    "THE `browser_bundle_harness` STEP is the migrated form of the source's own harness run.",
    "The source writes `kali_runtime_contract::browser_bundle_harness_script(\"app\", false,",
    "<body>)` next to the emitted bundle directory and executes it under",
    "`browser_harness_command_parts_for`; the case runner does exactly that from the step's",
    "`entry` and `body` (crates/kali_case_runner/src/steps.rs). The prelude resolves the",
    "bundle relative to the harness script's own URL rather than to the process working",
    "directory, so the runner's cwd (the trial root) and the source's cwd (the bundle dir)",
    "produce the same resolution -- which is why every previously shipped `browser/` bundle",
    "case file spells it this way.",
]


def assert_no_template_literals(source, cases):
    for name, body in source.items():
        if "${" in body.replace("${ext}", ""):
            raise AssertionError(f"[source] {name!r} contains a genuine `${{` -- rule 10 applies")
    for case in cases:
        for step in case["steps"]:
            if "${" in str(step.get("body", "")):
                raise AssertionError(f"step `body` in {case['name']} contains `${{`")
    return True


def build(header, matrix, source, cases):
    assert_no_template_literals(source, cases)
    return emit(header, matrix, source, cases)


def failing_harness_step(command, entry, json_output, env):
    argv = (["--output", "json"] if json_output else [])
    argv += [command, "--api", "browser", "--max-threads", "0",
             "--max-spawned-processes", "0", entry]
    return {"args": argv, "env": {env: "node"}, "exit": "failure"}


def failing_build_step(entry, json_output, extra=None):
    argv = ["build", "--bundle", "--api", "browser"]
    if json_output:
        argv += ["--output", "json"]
    argv += [entry]
    step = {"args": argv, "exit": "failure"}
    step.update(extra or {})
    return step


def ok_harness_cli_step(command, entry, json_output, env, *, asserts, json_claims=None):
    argv = (["--output", "json"] if json_output else [])
    argv += [command, "--api", "browser", "--max-threads", "0",
             "--max-spawned-processes", "0", entry]
    step = {"args": argv, "env": {env: "node"}}
    step.update(asserts)
    if json_output:
        if json_claims is None:
            raise AssertionError("a json_output step needs its json claims stated")
        step["json"] = json_claims
    return step


def bundle_success_steps(entry, bundle_dir, harness_body, harness_asserts, *,
                         json_output, envelope):
    argv = ["build", "--bundle", "--api", "browser"]
    if json_output:
        argv += ["--output", "json"]
    argv += [entry]
    build_step = {"args": argv, "exit": "success"}
    if json_output:
        build_step["json"] = envelope
    steps = [build_step,
             {"kind": "file_json", "path": f"{bundle_dir}/{bundle_dir}.meta.json",
              "fields": META}]
    harness = {"kind": "browser_bundle_harness", "entry": bundle_dir, "body": harness_body}
    harness.update(harness_asserts)
    steps.append(harness)
    return steps


# ==========================================================================
# F1. browser_object_enumeration_spread_runtime.rs
#     16 fns / 16 invocations, [matrix] ext, fail-closed.
# ==========================================================================

@target("object_enumeration_spread_runtime")
def gen_object_enumeration_spread_runtime():
    stem = "object_enumeration_spread_runtime"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_requested_object_enumeration_spread"

    program = check_program("spread", fixture_in_fn(text, "object_enumeration_spread_source"))
    source = {"main.${ext}": program, "smoke.test.${ext}": program}
    assert_bodies_identical("the one fixture both commands write", source,
                            ["main.${ext}", "smoke.test.${ext}"])

    c_write = cite(text, 'object_enumeration_spread_source()).expect("write source")')
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")
    c_fail = cite(text, "assert!(!output.status.success()")

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_carried(stem, text, reaching=helper),
        "",
        matrix_block(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[(helper, 16,
                      "command(run/test) x ext(js/ts/jsx/tsx) x json_output(false/true), a "
                      "complete cross product. Every `#[test]` fn is one unlooped call and "
                      "the file contains no loop at all.")],
            non_axis_lines=FAIL_CLOSED_NON_AXES),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        "",
        P.RULING7_NO_HOIST,
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "object_enumeration_spread_source", helper],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so its call chain never reaches",
        "`kali_runtime_contract`'s two harness helpers and ruling 6's exemption has nothing",
        "to exempt here; it is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        f"`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        FAIL_CLOSED_NOTE,
        f"The source writes the SAME fixture for both commands {c_write} and its only",
        f"assertion is at {c_fail}. There is no stdout, stderr, exit-code or JSON claim",
        "anywhere in the file, in either output mode, so the `--output json` siblings below",
        "assert nothing the text siblings do not. They are still their own `[[case]]` entries",
        "because the source has its own `#[test]` fns for them (rule 6) and because their",
        "argv genuinely differs.",
    )

    prose = rule12_rationale(text, stem)
    cases = []
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        for json_output in (False, True):
            fn = (("json_" if json_output else "") + command
                  + "_supports_object_enumeration_spread_in_browser_api_surface_with_harness"
                    "_js_input")
            assert_fns(text, fn)
            name, glob = strip_ext_suffix(fn)
            cases.append({
                "name": name,
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the four `{glob}` fns (one "
                    f"per extension). `{helper}` writes the shared Object-enumeration-spread "
                    f"program {c_write} and asks `kali` to {command} it against the browser "
                    "API surface with the browser harness backed by node. The program spreads "
                    "Object.keys / Object.values / Object.entries and Reflect.ownKeys over an "
                    "Object.fromEntries result through every bracketed, single-quoted and "
                    "frozen root spelling, then prints its keys and its awaited entries. kali "
                    "does not support that: the source's ONLY assertion is that the process "
                    f"fails {c_fail}, so this step carries `exit = \"failure\"` and nothing "
                    "else. Adding a diagnostic code or a stdout claim the source never made "
                    "would be a rule-2 invention, and `exit = \"failure\"` is exactly as strong "
                    "as the assertion it replaces."
                    + (" This sibling issues the `--output json` argv; the source asserts "
                       "nothing at all about the envelope, so neither does this case."
                       if json_output else "")
                    + prose),
                "steps": [failing_harness_step(command, entry, json_output, env)],
            })
    return build(header, {"ext": EXTS4}, source, cases)


# ==========================================================================
# F2. browser_object_enumeration_wrapped_bundle.rs
#     6 fns / 8 invocations, MATRIX DECLINED (js body differs), rule-11 OR.
# ==========================================================================

@target("object_enumeration_wrapped_bundle")
def gen_object_enumeration_wrapped_bundle():
    stem = "object_enumeration_wrapped_bundle"
    text = rs(stem)
    helper = "assert_browser_bundle_wrapped_object_enumeration"

    ts_body = check_program("wrapped ts", fixture_in_fn(
        text, "browser_bundle_wrapped_object_enumeration_source"),
        must_contain="function browserWrappedObjectEnumeration()")
    js_body = check_program("wrapped js", fixture_in_fn(
        text, "browser_bundle_wrapped_object_enumeration_js_source"),
        must_contain="function browserWrappedObjectEnumeration()")
    if ts_body == js_body:
        raise AssertionError("the js and non-js fixtures are identical -- the split is pointless")
    source = {"app.js": js_body, "app.ts": ts_body, "app.jsx": ts_body, "app.tsx": ts_body}
    assert_bodies_identical("the non-js fixture, written by three cells", source,
                            ["app.ts", "app.jsx", "app.tsx"])

    needle = "E5506"
    c_sel = cite(text, 'let source = if filename.ends_with(".js")')
    c_fail = cite(text, "assert!(!output.status.success()")
    c_or = cite(text, 'stderr.contains("E5506") || stdout.contains("E5506")')
    c_loop = cites(text, 'for filename in ["app.jsx", "app.tsx"]', 2)

    text_stream = _stream("E5506, text mode", [
        (f"app.{e}", source[f"app.{e}"],
         ["build", "--bundle", "--api", "browser", f"app.{e}"], needle) for e in EXTS4])
    json_stream = _stream("E5506, --output json mode", [
        (f"app.{e}", source[f"app.{e}"],
         ["build", "--bundle", "--api", "browser", "--output", "json", f"app.{e}"], needle)
        for e in EXTS4])
    if text_stream == json_stream:
        raise AssertionError(
            "the two output modes resolve to the same stream; the per-mode split below would "
            "be describing a distinction this binary does not make")

    disjunction = ('`assert!(stderr.contains("E5506") || stdout.contains("E5506"), "stdout: '
                   '{stdout}\\nstderr: {stderr}")`')

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_carried(stem, text, reaching=helper),
        "",
        P.matrix_declined(
            test_fns=6, invocations=8, cases=8,
            reason=[
                "`ext` LOOKS like a clean four-value axis and the arithmetic would even close",
                "(2 output modes x 4 extensions = 8 = the invocation count), but the fixture",
                f"BODY is not uniform over it: the helper picks its source with an `if",
                f"filename.ends_with(\".js\")` {c_sel}, so `app.js` gets a DIFFERENT program",
                "from `app.ts`/`app.jsx`/`app.tsx` -- the js variant declares a plain object",
                "literal and adds spread re-checks, the non-js variant declares",
                "`as const` / `satisfies unknown` wrappers. A `[matrix]` axis substitutes one",
                "string uniformly into every case (design spec 5.6: \"only for variation that",
                "substitutes uniformly\"); it cannot select a body.",
            ]),
        "",
        P.RULE6_ONE_TO_ONE,
        "Two of the six source fns are LOOPS over `[\"app.jsx\", \"app.tsx\"]`",
        f"({c_loop[0]} and {c_loop[1]}), each making two independent invocations against two",
        "independent programs. Rule 5 splits those into two named siblings each, suffixed with",
        "the extension they ran -- not numbered -- which is why 6 fns become 8 cases.",
        "",
        P.u2_source_file_wide(sorted(source)),
        "",
        P.RULING7_NO_HOIST,
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_bundle_wrapped_object_enumeration_source",
                         "browser_bundle_wrapped_object_enumeration_js_source", helper],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step -- the build never succeeds, so the",
        "source never gets as far as writing one -- and its call chain therefore never reaches",
        "`kali_runtime_contract`'s harness helpers. Ruling 6's exemption is not stated because",
        "it has nothing to exempt here.",
        "",
        ARGV_ORDER_BUILD_ONLY,
        "",
        FAIL_CLOSED_NOTE,
        f"This source makes ONE further claim beyond the failure {c_fail}, and it is",
        "OR-SHAPED (rule 11):",
        f"  {disjunction}",
        f"  -- whose cited construct is {c_or}.",
        "The case format has no disjunction, so per rule 11 the OR was RESOLVED against the",
        "real built `kali`, not reproduced and not dropped. Both modes were run for all four",
        "extensions, and the answer is per output mode and is unambiguous in each:",
        f"  * text mode          -> the code lands on {text_stream}, so those cases carry",
        f"                          `{text_stream}_contains = [\"{needle}\"]`.",
        f"  * `--output json`    -> it lands on {json_stream} (inside the JSON envelope), so",
        f"                          those cases carry `{json_stream}_contains = [\"{needle}\"]`.",
        "The generator refuses to emit if a cell shows the needle on both streams or neither,",
        "or if the four extensions disagree -- a disjunction that does not resolve to exactly",
        "one stream is not narrowable. This is a PRESENCE claim, so narrowing it is a verified",
        "strengthening (every run satisfying the new assertion satisfies the old); rule 2's",
        "asymmetry forbids the same narrowing for an ABSENCE claim, and none is made here.",
        "The source's full disjunction sentence is carried into every affected rationale, so",
        "the narrowing is recorded rather than silent.",
        "The json cases assert the code as a plain `stdout_contains` substring rather than as",
        "`json.errors.0.code`: the source spells it as a substring search over raw stdout, and",
        "mirroring the source is ruling 3.",
    )

    prose = rule12_rationale(text, stem)
    plan = [
        ("build_emits_wrapped_object_enumeration_semantics_in_ts_input", "app.ts", False, None),
        ("build_emits_wrapped_object_enumeration_semantics_in_js_input", "app.js", False, None),
        ("build_emits_wrapped_object_enumeration_semantics_in_jsx_tsx_input", "app.jsx", False,
         "jsx"),
        ("build_emits_wrapped_object_enumeration_semantics_in_jsx_tsx_input", "app.tsx", False,
         "tsx"),
        ("json_build_emits_wrapped_object_enumeration_semantics_in_ts_input", "app.ts", True,
         None),
        ("json_build_emits_wrapped_object_enumeration_semantics_in_js_input", "app.js", True,
         None),
        ("json_build_emits_wrapped_object_enumeration_semantics_in_jsx_tsx_input", "app.jsx",
         True, "jsx"),
        ("json_build_emits_wrapped_object_enumeration_semantics_in_jsx_tsx_input", "app.tsx",
         True, "tsx"),
    ]
    assert_fns(text, *{fn for fn, _e, _j, _s in plan})

    cases = []
    for fn, entry, json_output, split in plan:
        stream = json_stream if json_output else text_stream
        variant = "js" if entry == "app.js" else "non-js"
        cases.append({
            "name": fn + (f"_{split}" if split else ""),
            "rationale": (
                f"Migrated from browser_{stem}.rs, the source fn `{fn}`"
                + (f", the {split} half of its two-filename loop (rule 5: two independent "
                   "programs become two named siblings, never one folded case)"
                   if split else "")
                + f". `{helper}` writes the {variant} wrapped-object-enumeration fixture to "
                  f"{entry} and runs `kali build --bundle --api browser`"
                + (" with `--output json`" if json_output else "")
                + ". The program enumerates a four-key object whose keys mix integer-like and "
                  "string names through Object.keys / Object.values / Object.entries, pins the "
                  "integer-first ordering, and re-checks it through frozen bracketed-root "
                  "`Object.freeze((globalThis[\"Object\"]))[\"keys\"]` spellings. kali fails "
                  f"closed on it: the source asserts the process fails {c_fail} and that "
                  f"the diagnostic code {needle} appears. The source's full disjunction "
                  f"sentence, carried verbatim per rule 11: {disjunction}; its cited construct "
                  f"is {c_or}. The "
                  "case format has no disjunction, so per rule 11 that OR was resolved against "
                  f"the real binary rather than reproduced: in this output mode the code lands on "
                  f"{stream}, so the claim is carried as `{stream}_contains`. Narrowing a "
                  "PRESENCE claim to the stream that actually carries it is a verified "
                  "strengthening -- every run satisfying it satisfies the original OR."
                + prose),
            "steps": [failing_build_step(entry, json_output,
                                         {f"{stream}_contains": [needle]})],
        })
    return build(header, None, source, cases)


# ==========================================================================
# F3. browser_object_enumeration_wrapped_harness.rs
#     16 fns / 16 invocations, MATRIX DECLINED (js bodies differ), fail-closed.
# ==========================================================================

@target("object_enumeration_wrapped_harness")
def gen_object_enumeration_wrapped_harness():
    stem = "object_enumeration_wrapped_harness"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_harness_wrapped_object_enumeration"

    run_body = check_program("wrapped run", fixture_in_fn(
        text, "browser_harness_wrapped_object_enumeration_run_source"))
    js_run_body = check_program("wrapped js run", fixture_in_fn(
        text, "browser_harness_wrapped_object_enumeration_js_run_source"))
    test_body = check_program("wrapped test", fixture_in_fn(
        text, "browser_harness_wrapped_object_enumeration_test_source"))
    js_test_body = check_program("wrapped js test", fixture_in_fn(
        text, "browser_harness_wrapped_object_enumeration_js_test_source"))
    for a, b, label in ((run_body, js_run_body, "run"), (test_body, js_test_body, "test")):
        if a == b:
            raise AssertionError(f"the js and non-js {label} fixtures are identical")

    source = {"main.js": js_run_body, "main.ts": run_body, "main.jsx": run_body,
              "main.tsx": run_body,
              "smoke.test.js": js_test_body, "smoke.test.ts": test_body,
              "smoke.test.jsx": test_body, "smoke.test.tsx": test_body}
    assert_bodies_identical("the non-js run fixture", source, ["main.ts", "main.jsx", "main.tsx"])
    assert_bodies_identical("the non-js test fixture", source,
                            ["smoke.test.ts", "smoke.test.jsx", "smoke.test.tsx"])

    c_fail = cite(text, "assert!(!output.status.success()")
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_carried(stem, text, reaching=helper),
        "",
        P.matrix_declined(
            test_fns=16, invocations=16, cases=16,
            reason=[
                "`ext` LOOKS like a clean four-value axis and the arithmetic would close",
                "(4 command/output-mode combinations x 4 extensions = 16 = the invocation",
                "count), but the fixture BODY is not uniform over it. This source has FOUR",
                "fixture builders, and each `#[test]` fn passes the source explicitly: the",
                "`.js` cells get `..._js_run_source` / `..._js_test_source` (a plain object",
                "literal plus spread re-checks) and the `.ts`/`.jsx`/`.tsx` cells get",
                "`..._run_source` / `..._test_source` (`as const` and `satisfies unknown`",
                "wrappers). A `[matrix]` axis substitutes one string uniformly into every case",
                "(design spec 5.6: \"only for variation that substitutes uniformly\"); it",
                "cannot select a body.",
            ]),
        "",
        P.RULE6_ONE_TO_ONE,
        "",
        P.u2_source_file_wide(sorted(source)),
        "",
        P.RULING7_NO_HOIST,
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(
            ["kali_bin", "browser_harness_wrapped_object_enumeration_run_source",
             "browser_harness_wrapped_object_enumeration_js_run_source",
             "browser_harness_wrapped_object_enumeration_test_source",
             "browser_harness_wrapped_object_enumeration_js_test_source", helper],
            runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so its call chain never reaches",
        "`kali_runtime_contract`'s two harness helpers and ruling 6's exemption is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        f"`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        FAIL_CLOSED_NOTE,
        f"The source's only assertion is at {c_fail}, for every one of its 16 fns, in both",
        "output modes. There is no stdout, stderr, exit-code or JSON claim in the file.",
    )

    prose = rule12_rationale(text, stem)
    cases = []
    for json_output in (False, True):
        for command, prefix in (("run", "main"), ("test", "smoke.test")):
            for ext in ("ts", "js", "jsx", "tsx"):
                fn = (("json_" if json_output else "")
                      + f"{command}_supports_wrapped_object_enumeration_when_browser_harness"
                        f"_is_configured_in_{ext}_input")
                assert_fns(text, fn)
                entry = f"{prefix}.{ext}"
                variant = "js" if ext == "js" else "non-js"
                cases.append({
                    "name": fn,
                    "rationale": (
                        f"Migrated from browser_{stem}.rs, the source fn `{fn}` (one case per source fn, rule 6). "
                        f"`{helper}` writes the {variant} wrapped-object-enumeration "
                        f"{command} fixture to {entry} and asks `kali` to {command} it against "
                        "the browser API surface with the browser harness backed by node"
                        + (", with `--output json`" if json_output else "")
                        + ". The program enumerates a four-key object whose keys mix "
                          "integer-like and string names, pins the integer-first ordering "
                          "through Object.keys / Object.values / Object.entries, and re-checks "
                          "it through five frozen bracketed-root spellings of "
                          "`Object.freeze((globalThis[\"Object\"]))[\"keys\"]`. kali fails "
                          f"closed on it: the source's ONLY assertion is that the process "
                          f"fails {c_fail}, so this step carries `exit = \"failure\"` and "
                          "nothing else. Adding a diagnostic code or a stdout claim the source "
                          "never made would be a rule-2 invention."
                        + (" This sibling issues the `--output json` argv; the source asserts "
                           "nothing about the envelope, so neither does this case."
                           if json_output else "")
                        + prose),
                    "steps": [failing_harness_step(command, entry, json_output, env)],
                })
    return build(header, None, source, cases)


# ==========================================================================
# F4. browser_object_from_entries.rs
#     8 fns / 8 invocations, [matrix] ext, fail-closed build.
# ==========================================================================

@target("object_from_entries")
def gen_object_from_entries():
    stem = "object_from_entries"
    text = rs(stem)
    helper = "assert_browser_bundle_object_from_entries"

    program = check_program("from_entries", fixture_in_fn(
        text, "browser_bundle_object_from_entries_source"),
        must_contain="function browserObjectFromEntries()")
    source = {"app.${ext}": program}

    c_write = cite(text,
                   'browser_bundle_object_from_entries_source()).expect("write source")')
    c_fail = cite(text, "assert!(!output.status.success()")

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_carried(stem, text, reaching=helper),
        "",
        matrix_block(
            test_fns=8, invocations=8, cases=2, axis="ext", values=EXTS4,
            helpers=[(helper, 8,
                      "ext(js/ts/jsx/tsx) x json_output(false/true), a complete cross product. "
                      "Every `#[test]` fn is one unlooped call and the file contains no loop "
                      "at all.")],
            non_axes=("json_output",), non_axis_lines=FAIL_CLOSED_NON_AXES_BUILD),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["app.${ext}"]),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_bundle_object_from_entries_source", helper],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step -- the build never succeeds, so the",
        "source never writes one -- so ruling 6's exemption has nothing to exempt and is not",
        "stated.",
        "",
        ARGV_ORDER_BUILD_ONLY,
        "",
        FAIL_CLOSED_NOTE,
        f"The source writes one fixture {c_write} and asserts only {c_fail}. There is no",
        "stdout, stderr, exit-code or JSON claim in the file, in either output mode.",
    )

    prose = rule12_rationale(text, stem)
    cases = []
    for json_output in (False, True):
        fn = ("json_" if json_output else "") + \
            "build_emits_object_from_entries_semantics_in_js_input"
        assert_fns(text, fn)
        name, glob = strip_ext_suffix(fn)
        cases.append({
            "name": name,
            "rationale": (
                f"Migrated from browser_{stem}.rs, the four `{glob}` fns (one per "
                f"extension). `{helper}` writes the browser Object.fromEntries fixture "
                f"{c_write} and runs `kali build --bundle --api browser`"
                + (" with `--output json`" if json_output else "")
                + ". The program builds objects from entry pairs through eleven spellings -- "
                  "wrapped, frozen, conditional, frozen-callable and every bracketed "
                  "`globalThis[\"Object\"][\"fromEntries\"]` root -- and asserts the resulting "
                  "key, entry and value ordering. kali fails closed on it: the source's ONLY "
                  f"assertion is that the process fails {c_fail}, so this step carries "
                  "`exit = \"failure\"` and nothing else. Adding a diagnostic code or a stdout "
                  "claim the source never made would be a rule-2 invention."
                + (" This sibling issues the `--output json` argv; the source asserts nothing "
                   "about the envelope, so neither does this case." if json_output else "")
                + prose),
            "steps": [failing_build_step("app.${ext}", json_output)],
        })
    return build(header, {"ext": EXTS4}, source, cases)


# ==========================================================================
# F5. browser_object_from_entries_harness.rs
#     16 fns / 16 invocations, MATRIX DECLINED (ts-only block), fail-closed.
# ==========================================================================

@target("object_from_entries_harness")
def gen_object_from_entries_harness():
    stem = "object_from_entries_harness"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_harness_object_from_entries"

    anchors_run = [("__TS_ONLY__", "const wrappedEntries = ("),
                   ("assertFromEntriesShape(bracketedFromEntries);",
                    "assertFromEntriesShape(bracketedFromEntries);")]
    anchors_test = [("__TS_ONLY__", "Kali.test('object fromEntries ordering'"),
                    ("assertFromEntriesShape(Object.fromEntries(conditionalEntries));",
                     "assertFromEntriesShape(Object.fromEntries(conditionalEntries));")]
    run_plain = check_captured("from_entries run (js/jsx)", C.CAP_FROM_ENTRIES_HARNESS_RUN_PLAIN,
                               text, anchors=anchors_run)
    run_ts = check_captured("from_entries run (ts/tsx)", C.CAP_FROM_ENTRIES_HARNESS_RUN_TS,
                            text, anchors=anchors_run)
    test_plain = check_captured("from_entries test (js/jsx)",
                                C.CAP_FROM_ENTRIES_HARNESS_TEST_PLAIN, text,
                                anchors=anchors_test)
    test_ts = check_captured("from_entries test (ts/tsx)", C.CAP_FROM_ENTRIES_HARNESS_TEST_TS,
                             text, anchors=anchors_test)
    for plain, ts, label in ((run_plain, run_ts, "run"), (test_plain, test_ts, "test")):
        if plain == ts:
            raise AssertionError(f"the {label} plain and as-const captures are identical")
        if "as const" in plain or "as const" not in ts:
            raise AssertionError(f"the {label} captures are the wrong way round")

    # THE SUBSTITUTE FOR check_fixtures.py's RED ARM ON THIS PAIR (see the header
    # block below). The gate compares the source's fixture-shaped LITERALS
    # against the case file's program texts; this source's two literals are
    # UNRESOLVED `.replace` templates carrying a `__TS_ONLY__` needle, so they
    # provably do not appear in the case file and the gate reports both as
    # unmatched. That report is correct and is not worked around. What is done
    # instead is a stronger check with two measured sides: each capture is
    # required to equal the source's own template with the source's own needle
    # replaced by the source's own block -- every one of those three strings
    # pulled out of the `.rs` by the lexer, never retyped. A capture from the
    # wrong builder, or one taken before a source edit, fails here.
    for fn_name, plain, ts in (
            ("browser_harness_object_from_entries_run_source", run_plain, run_ts),
            ("browser_harness_object_from_entries_test_source", test_plain, test_ts)):
        template = fixture_in_fn(text, fn_name, 0)
        needle = fixture_in_fn(text, fn_name, 1)
        block = fixture_in_fn(text, fn_name, 2)
        if needle != "  __TS_ONLY__" or "as const" not in block:
            raise AssertionError(f"{fn_name}: the replace needle/block moved")
        if template.replace(needle, "") != plain:
            raise AssertionError(f"{fn_name}: the plain capture is not template-minus-needle")
        if template.replace(needle, block) != ts:
            raise AssertionError(f"{fn_name}: the as-const capture is not template-plus-block")

    source = {"main.js": run_plain, "main.jsx": run_plain,
              "main.ts": run_ts, "main.tsx": run_ts,
              "smoke.test.js": test_plain, "smoke.test.jsx": test_plain,
              "smoke.test.ts": test_ts, "smoke.test.tsx": test_ts}
    assert_bodies_identical("the js/jsx run capture", source, ["main.js", "main.jsx"])
    assert_bodies_identical("the ts/tsx run capture", source, ["main.ts", "main.tsx"])
    assert_bodies_identical("the js/jsx test capture", source,
                            ["smoke.test.js", "smoke.test.jsx"])
    assert_bodies_identical("the ts/tsx test capture", source,
                            ["smoke.test.ts", "smoke.test.tsx"])

    c_replace = cites(text, "source.replace(", 2)
    c_fail = cite(text, "assert!(!output.status.success()")
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_carried(stem, text, reaching=helper),
        "",
        P.matrix_declined(
            test_fns=16, invocations=16, cases=16,
            reason=[
                "`ext` LOOKS like a clean four-value axis and the arithmetic would close",
                "(4 command/output-mode combinations x 4 extensions = 16 = the invocation",
                "count), but the fixture BODY is not uniform over it. Both fixture builders",
                "take an `include_ts_as_const: bool`, and each `#[test]` fn passes it",
                "explicitly: the `.ts`/`.tsx` cells get an `as const` / `satisfies unknown`",
                "block spliced in and the `.js`/`.jsx` cells get nothing.",
                "A `[matrix]` axis substitutes one string uniformly into every case (design",
                "spec 5.6: \"only for variation that substitutes uniformly\"); it cannot select",
                "a body.",
            ]),
        "",
        P.RULE6_ONE_TO_ONE,
        "",
        P.u2_source_file_wide(sorted(source)),
        "",
        P.RULING7_NO_HOIST,
        "",
        "RULE 8 -- THE FIXTURES ARE CAPTURED, NEVER HAND-DERIVED. Both builders resolve their",
        f"text with `source.replace(\"  __TS_ONLY__\", ...)` ({c_replace[0]}, {c_replace[1]}),",
        "and the needle CARRIES TWO LEADING SPACES while the replacement is either the empty",
        "string or a block that supplies its own indentation. Hand-applying that is",
        "exactly the substitution trap rule 8 exists to prevent -- a mis-indented result is",
        "still a valid program and would still fail closed, so the real-binary run would",
        "verify the corrupted fixture against itself. The four texts below are the byte-exact",
        "output of EXECUTING the real builders (see tools/task-18-browser-pilot/",
        "batch7a_captures.py for the capture procedure), and the generator re-checks each one",
        "against anchors present in both the producing Rust source and the captured text",
        "before emitting it.",
        "",
        "CHECK_FIXTURES.PY GOES RED ON THIS PAIR, AND THAT REPORT IS CORRECT.",
        "`verify_pair.sh object_from_entries_harness` exits non-zero at the rule-9 fixture arm,",
        "reporting both of this source's program-shaped literals as UNMATCHED. That is a true",
        "statement about them: they are the UNRESOLVED `.replace` TEMPLATES, each carrying a",
        "`__TS_ONLY__` needle, and the case file correctly holds the four RESOLVED texts",
        "instead -- rule 8 requires the resolved string, so an exact match is the wrong test",
        "here. The gate's own tolerance for this shape is its `format!`-segment arm, which",
        "splits a template on `{...}` placeholders and requires each literal segment to",
        "survive; a `.replace` template has no `{...}`, so the whole template is one segment",
        "and the arm cannot fire. It is a gap in the gate for `.replace`-built fixtures, NOT a",
        "dropped or rewritten program, and it is reported rather than worked around.",
        "WHAT REPLACES IT is a stronger check with two measured sides, asserted in this file's",
        "generator on every run: each of the four texts below must equal the source's own",
        "template with the source's own `__TS_ONLY__` needle replaced by the source's own",
        "as-const block (or by the empty string) -- template, needle and block all pulled out",
        "of the `.rs` by the lexer, never retyped. The gate compares one measured string",
        "against one literal; this compares one EXECUTED string against an independently",
        "reconstructed one, so a capture from the wrong builder or from before a source edit",
        "fails the generator.",
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_harness_object_from_entries_run_source",
                         "browser_harness_object_from_entries_test_source", helper],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so ruling 6's exemption has nothing",
        "to exempt and is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        f"`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        FAIL_CLOSED_NOTE,
        f"The source's only assertion is at {c_fail}, for every one of its 16 fns, in both",
        "output modes.",
    )

    prose = rule12_rationale(text, stem)
    cases = []
    for json_output in (False, True):
        for command, prefix in (("run", "main"), ("test", "smoke.test")):
            for ext in EXTS4:
                fn = (("json_" if json_output else "")
                      + f"{command}_supports_object_from_entries_when_browser_harness_is"
                        f"_configured_in_{ext}_input")
                assert_fns(text, fn)
                entry = f"{prefix}.{ext}"
                ts = ext in ("ts", "tsx")
                cases.append({
                    "name": fn,
                    "rationale": (
                        f"Migrated from browser_{stem}.rs, the source fn `{fn}` (one case per source fn, rule 6). "
                        f"`{helper}` writes the Object.fromEntries {command} fixture to "
                        f"{entry} -- the variant WITH the `as const` / `satisfies unknown` "
                        "block, because this cell passes `include_ts_as_const: true`"
                        if ts else
                        f"Migrated from browser_{stem}.rs, the source fn `{fn}` (one case per source fn, rule 6). "
                        f"`{helper}` writes the Object.fromEntries {command} fixture to "
                        f"{entry} -- the variant WITHOUT the `as const` / `satisfies unknown` "
                        "block, because this cell passes `include_ts_as_const: false`")
                    + (f" -- and asks `kali` to {command} it against the browser API surface "
                       "with the browser harness backed by node"
                       + (", with `--output json`" if json_output else "")
                       + ". The program builds objects from entry pairs through ten bracketed, "
                         "single-quoted, frozen and conditional `Object.fromEntries` spellings "
                         "and asserts the resulting key, entry and value ordering. kali fails "
                         f"closed on it: the source's ONLY assertion is that the process fails "
                         f"{c_fail}, so this step carries `exit = \"failure\"` and nothing "
                         "else; adding a claim the source never made would be a rule-2 "
                         "invention. The fixture text is the byte-exact output of executing "
                         f"the real `source.replace(...)` (rule 8, {c_replace[0]}), never "
                         "hand-derived."
                       + (" This sibling issues the `--output json` argv; the source asserts "
                          "nothing about the envelope, so neither does this case."
                          if json_output else "")
                       + prose),
                    "steps": [failing_harness_step(command, entry, json_output, env)],
                })
    return build(header, None, source, cases)


# ==========================================================================
# F6. browser_object_has_own_bundle.rs
#     8 fns / 8 invocations, MATRIX DECLINED (ts body differs).
#     Build SUCCEEDS; the bundle harness is what fails closed.
# ==========================================================================

@target("object_has_own_bundle")
def gen_object_has_own_bundle():
    stem = "object_has_own_bundle"
    text = rs(stem)
    helper = "assert_browser_bundle_object_has_own"

    anchors = [("const object = {{ a: 1, \"b\": 2 }};", "const object = { a: 1, \"b\": 2 };"),
               ("console.log('browser object hasOwn ok');",
                "console.log('browser object hasOwn ok');")]
    js_body = check_captured("has_own bundle js", C.CAP_HAS_OWN_BUNDLE_JS, text, anchors=anchors)
    ts_body = check_captured(
        "has_own bundle ts", C.CAP_HAS_OWN_BUNDLE_TS, text,
        anchors=[("const object = ({{ a: 1, \"b\": 2 }} as const);",
                  "const object = ({ a: 1, \"b\": 2 } as const);"),
                 ("console.log('browser object hasOwn ok');",
                  "console.log('browser object hasOwn ok');")])
    if js_body == ts_body:
        raise AssertionError("the js and ts has-own captures are identical")

    source = {"app.js": js_body, "app.jsx": js_body, "app.ts": ts_body, "app.tsx": ts_body}
    assert_bodies_identical("the js/jsx capture", source, ["app.js", "app.jsx"])
    assert_bodies_identical("the ts/tsx capture", source, ["app.ts", "app.tsx"])

    harness_body = check_program(
        "has_own harness body",
        fixture_starting(text, helper, "const mod = await import(bundleJs.href);"),
        must_contain="await mod.browserObjectHasOwn();")

    c_build_ok = cite(text, "output.status.success(),", occurrence=1, expect=2)
    c_env_json = cite(text, 'assert_eq!(envelope["schemaVersion"], 1)')
    c_errors = cite(text, 'assert!(envelope["errors"]')
    c_meta = cite(text, 'assert_eq!(metadata["apiSurface"], "browser")')
    c_script = cite(text, "kali_runtime_contract::browser_bundle_harness_script(")
    c_fail = cite(text, "assert!(!output.status.success()")

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_carried(stem, text, reaching=helper),
        "",
        P.matrix_declined(
            test_fns=8, invocations=8, cases=8,
            reason=[
                "`ext` LOOKS like a clean four-value axis and the arithmetic would close",
                "(2 output modes x 4 extensions = 8 = the invocation count), but the fixture",
                "BODY is not uniform over it: each `#[test]` fn passes its source explicitly,",
                "and the `.js`/`.jsx` cells get `browser_bundle_object_has_own_js_source()`",
                "(a bare object literal) while the `.ts`/`.tsx` cells get",
                "`browser_bundle_object_has_own_ts_source()` (the same object wrapped",
                "`as const`). A `[matrix]` axis substitutes one string uniformly into every",
                "case (design spec 5.6); it cannot select a body.",
            ]),
        "",
        P.RULE6_ONE_TO_ONE,
        "",
        P.u2_source_file_wide(sorted(source)),
        "Every entry has the stem `app`, so `kali build --bundle` writes the same `app/`",
        "output directory whichever one a case names -- and only one build runs per trial, so",
        "there is no collision. That is why no U5 rename is needed here.",
        "",
        P.RULING7_NO_HOIST,
        "",
        "RULE 8 / RULE 9 -- THE FIXTURES ARE CAPTURED, NEVER HAND-DERIVED. Both builders are",
        "one inline `format!` with `{{`/`}}` brace-collapse whose three arguments are computed",
        "by FOUR kali_common::object helpers, some of which join an alias table and some of",
        "which wrap another helper's output. That is rule 8's `format!` trap and rule 9's",
        "\"one level removed inside a",
        "library crate\" case at once, and a hand-derived approximation would still be a valid",
        "program that still built, so the real-binary run would verify the corrupted fixture",
        "against itself. The two texts below are the byte-exact output of EXECUTING the real",
        "builders (see tools/task-18-browser-pilot/batch7a_captures.py for the capture",
        "procedure), and the generator re-checks each against anchors present in both the",
        "producing Rust source and the captured text before emitting it.",
        "",
        NO_TEMPLATE_LITERAL,
        "",
        rule13_kali_common_block(
            ["kali_bin", "browser_bundle_object_has_own_js_source",
             "browser_bundle_object_has_own_ts_source", helper],
            runner_exemption=True),
        "",
        ARGV_ORDER_BUILD_ONLY,
        "",
        RUNNER_HARNESS_STEP,
        f"The source builds that script at {c_script} with `entry = \"app\"` and",
        "`allow_subpaths = false`, which is exactly what the runner's step kind passes.",
        "",
        "ASSERTION SHAPE -- A SUCCESSFUL BUILD WHOSE HARNESS THEN FAILS CLOSED. This target is",
        "not one of the batch's plain fail-closed files: `kali build --bundle` SUCCEEDS",
        f"{c_build_ok}, the emitted metadata is asserted, and it is the BUNDLE HARNESS run",
        f"that must fail {c_fail}.",
        f"  * `exit = \"success\"` on the build step {c_build_ok}.",
        f"  * In json mode, the envelope's schemaVersion/command/success/exitCode and the",
        f"    payload's artifactKind/bundleFormat {c_env_json}, plus an EMPTY `errors`",
        f"    array {c_errors}.",
        f"  * `app/app.meta.json`'s apiSurface/artifactKind {c_meta}, asserted in BOTH",
        "    modes, because the source reads that file outside the `if json_output` block.",
        f"  * `exit = \"failure\"` on the harness step {c_fail} -- and nothing else. The",
        "    source makes no stdout, stderr or exit-code claim about the harness process, so",
        "    neither does this file; adding one would be a rule-2 invention.",
    )

    prose = rule12_rationale(text, stem) + rule13_kali_common_rationale()
    cases = []
    for json_output in (False, True):
        for ext in ("js", "jsx", "ts", "tsx"):
            fn = ("json_" if json_output else "") + f"build_emits_browser_object_has_own_in_{ext}_input"
            assert_fns(text, fn)
            entry = f"app.{ext}"
            variant = ("the `as const` TypeScript variant" if ext in ("ts", "tsx")
                       else "the plain JavaScript variant")
            steps = bundle_success_steps(
                entry, "app", harness_body, {"exit": "failure"},
                json_output=json_output, envelope=envelope_build(errors=True))
            cases.append({
                "name": fn,
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the source fn `{fn}` (one case per source fn, rule 6). "
                    f"`{helper}` writes {variant} of the browser Object.hasOwn fixture to "
                    f"{entry}, builds it with `kali build --bundle --api browser`"
                    + (" with `--output json`" if json_output else "")
                    + ", asserts the emitted app/app.meta.json metadata, then writes the "
                      "browser-bundle harness and runs it under node. The program checks "
                      "Object.hasOwn and Object.prototype.hasOwnProperty.call through every "
                      "bracketed, single-quoted, parenthesized and frozen-callable spelling "
                      "`kali_common` knows about. THE BUILD SUCCEEDS "
                    + f"{c_build_ok}; it is the HARNESS process that must fail closed "
                      f"{c_fail}, so the `browser_bundle_harness` step carries "
                      "`exit = \"failure\"` and no output claim -- the source makes none."
                    + (" This sibling additionally asserts the build JSON envelope -- "
                       "schemaVersion/command/success/exitCode, payload artifactKind/"
                       f"bundleFormat, and the empty `errors` array {c_errors} -- rather "
                       "than plain text; output shape is not a matrix axis because it changes "
                       "the assertion shape, so it is a separate case."
                       if json_output else "")
                    + " The fixture text is the byte-exact output of executing the real "
                      "`format!` and its four kali_common::object helpers (rules 8 and 9), "
                      "never hand-derived."
                    + prose),
                "steps": steps,
            })
    return build(header, None, source, cases)


# ==========================================================================
# F7. browser_object_has_own_from_entries.rs
#     33 fns / 53 invocations, MATRIX DECLINED (frozen cells are not uniform),
#     three helpers, U5 renames.
# ==========================================================================

@target("object_has_own_from_entries")
def gen_object_has_own_from_entries():
    stem = "object_has_own_from_entries"
    text = rs(stem)
    env = assert_env_name()
    h_bundle = "assert_browser_bundle_object_has_own_from_entries_with_source"
    h_check = "assert_browser_check_object_has_own_from_entries_with_source"
    h_harness = "assert_browser_harness_object_has_own_from_entries"

    freeze_anchor = ("const object = Object.fromEntries(", "const object = Object.freeze(")
    bundle_plain = check_captured("hofe bundle plain", C.CAP_HOFE_BUNDLE_PLAIN, text,
                                  anchors=[("browserObjectHasOwnFromEntries",
                                            "browserObjectHasOwnFromEntries")])
    bundle_frozen = check_captured("hofe bundle frozen", C.CAP_HOFE_BUNDLE_FROZEN, text,
                                   anchors=[freeze_anchor])
    run_plain = check_captured("hofe run plain", C.CAP_HOFE_RUN_PLAIN, text,
                               anchors=[("browserObjectHasOwnFromEntries();",
                                         "browserObjectHasOwnFromEntries();")])
    run_frozen = check_captured("hofe run frozen", C.CAP_HOFE_RUN_FROZEN, text,
                                anchors=[freeze_anchor])
    test_plain = check_captured("hofe test plain", C.CAP_HOFE_TEST_PLAIN, text,
                                anchors=[("Kali.test('object hasOwn fromEntries'",
                                          "Kali.test('object hasOwn fromEntries'")])
    test_frozen = check_captured("hofe test frozen", C.CAP_HOFE_TEST_FROZEN, text,
                                 anchors=[freeze_anchor])
    for plain, frozen, label in ((bundle_plain, bundle_frozen, "bundle"),
                                 (run_plain, run_frozen, "run"),
                                 (test_plain, test_frozen, "test")):
        if plain == frozen:
            raise AssertionError(f"the {label} plain and frozen captures are identical")
        if frozen != plain.replace("const object = Object.fromEntries(",
                                   "const object = Object.freeze(Object.fromEntries("):
            raise AssertionError(
                f"the {label} frozen capture is not the plain capture with exactly the "
                "`Object.freeze(` wrap the source's `source.replace(` call applies")

    source = {}
    for ext in EXTS4:
        source[f"app.{ext}"] = bundle_plain
        source[f"app_frozen.{ext}"] = bundle_frozen
        source[f"main.{ext}"] = run_plain
        source[f"main_frozen.{ext}"] = run_frozen
        source[f"smoke.test.{ext}"] = test_plain
        source[f"smoke_frozen.test.{ext}"] = test_frozen
    for stem_key, label in (("app", "plain bundle"), ("app_frozen", "frozen bundle"),
                            ("main", "plain run"), ("main_frozen", "frozen run")):
        assert_bodies_identical(f"the {label} capture", source,
                                [f"{stem_key}.{e}" for e in EXTS4])
    assert_bodies_identical("the plain test capture", source,
                            [f"smoke.test.{e}" for e in EXTS4])
    assert_bodies_identical("the frozen test capture", source,
                            [f"smoke_frozen.test.{e}" for e in EXTS4])

    renamed = ["app_frozen.${ext}", "main_frozen.${ext}", "smoke_frozen.test.${ext}"]
    P.assert_rename_is_argv_only(source, renamed, EXTS4)

    harness_body = check_program(
        "hofe bundle harness body",
        fixture_starting(text, h_bundle, "const mod = await import(bundleJs.href);"),
        must_contain="await mod.browserObjectHasOwnFromEntries();")

    c_freeze = cites(text, 'const object = Object.freeze(Object.fromEntries(', 3)
    c_build_ok = cites(text, "output.status.success(),", 4)
    c_errors = cite(text, 'assert!(envelope["errors"]')
    c_meta = cite(text, 'assert_eq!(metadata["apiSurface"], "browser")')
    c_bundle_stdout = cites(
        text, 'stdout.contains("browser object hasOwn fromEntries ok")', 3)
    c_files_checked = cite(text, 'json["payload"]["filesChecked"]')
    c_json_stdout = cite(text, 'let stdout = json["stdout"].as_str()')
    c_json_stderr = cite(text, 'assert_eq!(json["stderr"], "")')
    c_ok1 = cite(text, 'stdout.contains("ok 1")')
    assert_count(text, "for filename in [", 4)   # asserted, not printed: ruling 15

    pin = _pin("browser object hasOwn fromEntries ok",
               "browser object hasOwn fromEntries ok\n",
               [(f"main.{e}", run_plain, "run") for e in EXTS4]
               + [(f"smoke.test.{e}", test_plain, "test") for e in EXTS4]
               + [(f"main_frozen.{e}", run_frozen, "run") for e in EXTS4]
               + [(f"smoke_frozen.test.{e}", test_frozen, "test") for e in EXTS4])

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        [P.extra_ok(pin, P.EXTRA_OK_JSON_STDOUT)],
        [P.extra_ok(f"{s}.{e}" if not s.endswith(".test") else f"{s}.{e}",
                    P.EXTRA_OK_U5_RENAME)
         for s in ("app_frozen", "main_frozen", "smoke_frozen.test") for e in EXTS4],
        [P.extra_ok("app_frozen/app_frozen.meta.json",
                    "the U5-renamed bundle output path; `kali build --bundle` names its output "
                    "directory after the input STEM, so the renamed entry moves the emitted "
                    "meta file with it (U5's own requirement that the `file_json` path tracks "
                    "the rename)"),
         P.extra_ok("app_frozen",
                    "the U5-renamed bundle directory, passed as the `browser_bundle_harness` "
                    "step's `entry` -- same reason as the meta path above")],
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_none(stem, text),
        "",
        P.matrix_declined(
            test_fns=33, invocations=53, cases=53,
            reason=[
                "`ext` is NOT a uniform axis in this source, and the arithmetic proves it. The",
                "PLAIN cells do form a complete cross (4 extensions x 2 output modes for the",
                "bundle helper, and likewise for run and for test), but the FROZEN cells do",
                "not: `app.js` / `main.js` / `smoke.test.js` are exercised in TEXT MODE ONLY,",
                "while `ts`, `jsx` and `tsx` are exercised in both. Declaring `ext` an axis",
                "would fan a `--output json` frozen trial for `js` that the source never ran,",
                "which manufactures an untested combination and is a rule-2 invention as well",
                "as a rule-7 arithmetic failure.",
                "The check helper's own cells DO form a complete cross, but U1 is explicit",
                "that `[matrix]` is FILE-WIDE with no per-case opt-out, so one non-uniform",
                "helper drops the axis for the whole file.",
            ]),
        "",
        P.RULE6_ONE_TO_ONE,
        "Four of the 33 source fns are LOOPS (`for filename in [...]`), and each of the four",
        "loops over three filenames AND both output modes, making six independent invocations",
        "against six independent programs. Rule 5 splits every one of them into named siblings",
        "-- suffixed with the extension and output mode they ran, not numbered -- which is why",
        "33 fns become 53 cases: 29 unlooped invocations + 4 x 6 looped ones = 53 invocations,",
        "53 cases. The generator asserts the loop count against the source before emitting and",
        "writes no line number for the loops: a figure that is neither gated nor pinned to an",
        "immutable ref is deleted (ruling 15's third answer), and the count is checked here",
        "rather than quoted there.",
        "",
        P.u2_source_file_wide(sorted(source)),
        "The `kali check` and `kali test` steps below were checked against the real binary",
        "with all 24 fixtures present in one directory: `payload.filesChecked` is still 1 and",
        "`payload.total` is still 1, so neither command discovers its siblings and the",
        "file-wide `[source]` table does not change what any case measures.",
        "",
        P.u5_renames(
            [("app.<ext>", "app_frozen.<ext>",
              "the FROZEN bundle/check program, which the plain build already claims `app.<ext>`"),
             ("main.<ext>", "main_frozen.<ext>", "the FROZEN harness run program"),
             ("smoke.test.<ext>", "smoke_frozen.test.<ext>",
              "the FROZEN harness test program; the `.test.` infix is kept so the entry still "
              "reads as a test file")],
            collision="six different program texts across three filename families"),
        "Because `kali build --bundle` names its output directory after the input STEM, the",
        "renamed frozen entry moves the emitted bundle: those cases read",
        "`app_frozen/app_frozen.meta.json` and pass `entry = \"app_frozen\"` to the harness",
        "step, per U5's own requirement that the `file_json` path and the harness `entry`",
        "track the rename rather than staying hardcoded to `app`.",
        "",
        P.RULING7_NO_HOIST,
        "",
        "RULE 8 / RULE 9 -- THE THREE FROZEN FIXTURES ARE CAPTURED, NEVER HAND-DERIVED. Each",
        "is built off its plain counterpart by a `source.replace(` call. The three",
        "REPLACEMENT literals -- not the needles --",
        "are " + ", ".join(c_freeze) + ";",
        "the third carries two leading spaces because the test fixture indents that line, and",
        "each needle beside them carries the same indentation as its replacement. The",
        "generator additionally asserts, mechanically, that each frozen capture is EXACTLY its",
        "plain capture with the `Object.freeze(` wrap applied -- so a capture taken before a",
        "source edit fails the generator rather than shipping a program that is no longer the",
        "program under test. The plain texts were captured through the same run rather than",
        "read out of the `.rs`, so that comparison has two measured sides.",
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(
            ["kali_bin", "browser_bundle_object_has_own_from_entries_source",
             "browser_bundle_object_has_own_frozen_from_entries_source",
             "browser_harness_object_has_own_from_entries_run_source",
             "browser_harness_object_has_own_frozen_from_entries_run_source",
             "browser_harness_object_has_own_from_entries_test_source",
             "browser_harness_object_has_own_frozen_from_entries_test_source",
             h_bundle, h_check, h_harness],
            runner_exemption=True),
        "",
        ARGV_ORDER_CHECK_BUILD_HARNESS,
        f"The harness helper's env value is `{env}`, read from the",
        "`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        "name rather than assumed. The check and build helpers set no env.",
        "",
        RUNNER_HARNESS_STEP,
        "",
        "ASSERTION SHAPE, mirrored from the source and nothing more. THREE HELPERS, THREE",
        "SHAPES:",
        f"  * `{h_check}`: `exit = \"success\"`; in",
        f"    json mode schemaVersion/command/success/exitCode, `payload.filesChecked = 1`",
        f"    {c_files_checked} and an empty `errors` array. It asserts NOTHING about",
        "    stdout in either mode, so neither does this file.",
        f"  * `{h_bundle}`:",
        f"    `exit = \"success\"` on the build {c_build_ok[0]}; in json mode the envelope and",
        f"    an empty `errors` array {c_errors}; `app*/app*.meta.json`'s apiSurface/",
        f"    artifactKind in BOTH modes {c_meta}; then the bundle harness, `exit =",
        f"    \"success\"` with `stdout_contains` {c_bundle_stdout[0]}.",
        f"  * `{h_harness}`: `exit = \"success\"`; in text",
        f"    mode `stdout_contains` {c_bundle_stdout[1]} plus `\"ok 1\"` for the `test`",
        f"    command only {c_ok1}; in json mode the envelope, an exact `json.stdout` pin",
        f"    {c_json_stdout}, `json.stderr = \"\"` {c_json_stderr} and an empty `errors`",
        "    array.",
        P.ruling3_substring(),
        P.ruling3_json_leaf(),
        f"The live-captured value is {pin!r}, identical across all 16 cells "
        "(4 extensions x plain/frozen x run/test) and re-captured by the generator on every",
        "run.",
    )

    cases = []
    prose_pin = P.ruling3_substring()
    prose_leaf = P.ruling3_json_leaf()

    def harness_json(command):
        j = envelope_harness(command, stderr=True, errors=True)
        out = {}
        for key, value in j.items():
            if key in ("stderr", "errors") and "stdout" not in out:
                out["stdout"] = pin
            out[key] = value
        out.setdefault("stdout", pin)
        return out

    # --- check helper (8) ---
    check_plan = [
        ("check_emits_frozen_object_has_own_from_entries_in_js_input", "js", False, None),
        ("json_check_emits_frozen_object_has_own_from_entries_in_js_input", "js", True, None),
    ]
    for ext in ("ts", "jsx", "tsx"):
        for json_output in (False, True):
            check_plan.append(
                ("check_emits_frozen_object_has_own_from_entries_in_ts_jsx_tsx_input", ext,
                 json_output, f"{ext}_{'json' if json_output else 'text'}"))
    assert_fns(text, *{fn for fn, _e, _j, _s in check_plan})
    for fn, ext, json_output, split in check_plan:
        entry = f"app_frozen.{ext}"
        argv = ["check", "--api", "browser"] + (["--output", "json"] if json_output else [])
        argv += [entry]
        step = {"args": argv, "exit": "success"}
        if json_output:
            step["json"] = {"schemaVersion": 1, "command": "check", "success": True,
                            "exitCode": 0, "payload": {"filesChecked": 1}, "errors": []}
        cases.append({
            "name": fn + (f"_{split}" if split else ""),
            "rationale": (
                f"Migrated from browser_{stem}.rs, the source fn `{fn}`"
                + (f", the {split.replace('_', ' / ')} cell of its three-filename x "
                   "two-output-mode loop (rule 5: six independent invocations become six "
                   "named siblings)" if split else "")
                + f". `{h_check}` writes the FROZEN Object.hasOwn-from-entries program to "
                  f"{entry} and runs `kali check --api browser`"
                + (" with `--output json`" if json_output else "")
                + ". The program freezes an Object.fromEntries result and checks membership "
                  "through eighteen Object.hasOwn and Object.prototype.hasOwnProperty.call "
                  "spellings. The check succeeds. This helper asserts the process succeeds and, "
                  "in json mode, schemaVersion/command/success/exitCode, "
                  f"`payload.filesChecked = 1` {c_files_checked} and an empty `errors` "
                  "array. It makes no stdout claim in either mode, so neither does this case -- "
                  "adding one would be a rule-2 invention. The entry is U5-renamed from "
                  "`app.<ext>` because the plain bundle program already claims that key in this "
                  "file-wide `[source]` table; the name is passed to `kali` on argv only and is "
                  "referenced by no fixture body, checked mechanically in this file's "
                  "generator."),
            "steps": [step],
        })

    # --- bundle helper: plain (8) then frozen (7) ---
    bundle_plan = []
    for json_output in (False, True):
        for ext in ("js", "jsx", "ts", "tsx"):
            bundle_plan.append(
                (("json_" if json_output else "")
                 + f"build_emits_object_has_own_from_entries_in_{ext}_input",
                 ext, json_output, None, False))
    bundle_plan.append(("build_emits_frozen_object_has_own_from_entries_in_js_input", "js",
                        False, None, True))
    for ext in ("ts", "jsx", "tsx"):
        for json_output in (False, True):
            bundle_plan.append(
                ("build_emits_frozen_object_has_own_from_entries_in_ts_jsx_tsx_input", ext,
                 json_output, f"{ext}_{'json' if json_output else 'text'}", True))
    assert_fns(text, *{fn for fn, _e, _j, _s, _f in bundle_plan})
    for fn, ext, json_output, split, frozen in bundle_plan:
        entry = (f"app_frozen.{ext}" if frozen else f"app.{ext}")
        bundle_dir = "app_frozen" if frozen else "app"
        cases.append({
            "name": fn + (f"_{split}" if split else ""),
            "rationale": (
                f"Migrated from browser_{stem}.rs, the source fn `{fn}`"
                + (f", the {split.replace('_', ' / ')} cell of its three-filename x "
                   "two-output-mode loop (rule 5)" if split else "")
                + f". `{h_bundle}` writes the "
                + ("FROZEN" if frozen else "plain")
                + f" Object.hasOwn-from-entries program to {entry}, builds it with `kali build "
                  "--bundle --api browser`"
                + (" with `--output json`" if json_output else "")
                + f", asserts the emitted {bundle_dir}/{bundle_dir}.meta.json metadata "
                  f"{c_meta}, then writes the browser-bundle harness and runs it under node. "
                  "The program checks membership in an "
                + ("Object.freeze(Object.fromEntries(...))" if frozen
                   else "Object.fromEntries(...)")
                + " result through eighteen Object.hasOwn and "
                  "Object.prototype.hasOwnProperty.call spellings, including conditional and "
                  "frozen callables, then prints its ok line. Both processes succeed "
                  f"{c_build_ok[0]}, and the harness's stdout claim is "
                  f"{c_bundle_stdout[0]}. " + prose_pin
                + (" This sibling additionally asserts the build JSON envelope -- "
                   "schemaVersion/command/success/exitCode, payload artifactKind/bundleFormat, "
                   f"and the empty `errors` array {c_errors} -- rather than plain text; "
                   "output shape is not a matrix axis because it changes the assertion shape, "
                   "so it is a separate case." if json_output else "")
                + (" The entry is U5-renamed from `app.<ext>` because the plain bundle program "
                   "already claims that key; because `kali build --bundle` names its output "
                   "directory after the input stem, the meta path and the harness `entry` "
                   "track the rename rather than staying hardcoded to `app`." if frozen else "")
                + " The fixture text is the byte-exact output of executing the real fixture "
                  "builder (rules 8 and 9), never hand-derived."),
            "steps": bundle_success_steps(
                entry, bundle_dir, harness_body,
                {"exit": "success",
                 "stdout_contains": ["browser object hasOwn fromEntries ok"]},
                json_output=json_output, envelope=envelope_build(errors=True)),
        })

    # --- harness helper: plain run/test (16) then frozen run/test (14) ---
    harness_plan = []
    for command, prefix in (("run", "main"), ("test", "smoke.test")):
        for json_output in (False, True):
            for ext in ("js", "jsx", "ts", "tsx"):
                harness_plan.append(
                    ((("json_" if json_output else "") + f"{command}_supports_object_has_own"
                      f"_from_entries_when_browser_harness_is_configured_in_{ext}_input"),
                     command, f"{prefix}.{ext}", json_output, None, False))
    for command, prefix in (("run", "main_frozen"), ("test", "smoke_frozen.test")):
        harness_plan.append(
            (f"{command}_supports_frozen_object_has_own_from_entries_when_browser_harness_is"
             f"_configured_in_js_input", command, f"{prefix}.js", False, None, True))
        for ext in ("ts", "jsx", "tsx"):
            for json_output in (False, True):
                harness_plan.append(
                    (f"{command}_supports_frozen_object_has_own_from_entries_when_browser"
                     f"_harness_is_configured_in_ts_jsx_tsx_input", command,
                     f"{prefix}.{ext}", json_output,
                     f"{ext}_{'json' if json_output else 'text'}", True))
    assert_fns(text, *{fn for fn, _c, _e, _j, _s, _f in harness_plan})
    for fn, command, entry, json_output, split, frozen in harness_plan:
        asserts = {"exit": "success"}
        if not json_output:
            needles = ["browser object hasOwn fromEntries ok"]
            if command == "test":
                needles.append("ok 1")
            asserts["stdout_contains"] = needles
        cases.append({
            "name": fn + (f"_{split}" if split else ""),
            "rationale": (
                f"Migrated from browser_{stem}.rs, the source fn `{fn}`"
                + (f", the {split.replace('_', ' / ')} cell of its three-filename x "
                   "two-output-mode loop (rule 5)" if split else "")
                + f". `{h_harness}` writes the "
                + ("FROZEN" if frozen else "plain")
                + f" Object.hasOwn-from-entries {command} program to {entry} and asks `kali` to "
                  f"{command} it against the browser API surface with the browser harness "
                  "backed by node"
                + (", with `--output json`" if json_output else "")
                + ". The program checks membership in an "
                + ("Object.freeze(Object.fromEntries(...))" if frozen
                   else "Object.fromEntries(...)")
                + " result through eighteen Object.hasOwn and "
                  "Object.prototype.hasOwnProperty.call spellings and prints its ok line. The "
                  "process succeeds."
                + ((f" In json mode the source reads json[\"stdout\"] {c_json_stdout} and "
                    f"asserts a substring match on it, plus {c_json_stderr} and an empty "
                    "`errors` array, "
                    "and for `run` the envelope and payload exitCode / for `test` the payload "
                    "total, passed and failed. " + prose_leaf)
                   if json_output else
                   (f" Its text-mode claim is {c_bundle_stdout[1]}"
                    + (f", plus {c_ok1} for the `test` command"
                       if command == "test" else "")
                    + ". " + prose_pin))
                + (" The entry is U5-renamed because the plain program already claims the "
                   "unsuffixed key in this file-wide `[source]` table; the name is passed to "
                   "`kali` on argv only and is referenced by no fixture body, checked "
                   "mechanically in this file's generator." if frozen else "")
                + " The fixture text is the byte-exact output of executing the real fixture "
                  "builder (rules 8 and 9), never hand-derived."),
            "steps": [ok_harness_cli_step(command, entry, json_output, env, asserts=asserts,
                                          json_claims=harness_json(command))],
        })

    if len(cases) != 53:
        raise AssertionError(f"{len(cases)} cases, expected 53 (one per invocation)")
    return build(header, None, source, cases)


# ==========================================================================
# F8. browser_object_has_own_harness.rs
#     16 fns / 16 invocations, [matrix] ext, fail-closed.
# ==========================================================================

@target("object_has_own_harness")
def gen_object_has_own_harness():
    stem = "object_has_own_harness"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_harness_object_has_own"

    run_body = check_captured(
        "has_own harness run", C.CAP_HAS_OWN_HARNESS_RUN, text,
        anchors=[("const object = Object.fromEntries([[\"a\", 1], [\"b\", 2]]);",
                  "const object = Object.fromEntries([[\"a\", 1], [\"b\", 2]]);"),
                 ("console.log('browser object hasOwn ok');",
                  "console.log('browser object hasOwn ok');")])
    test_body = check_captured(
        "has_own harness test", C.CAP_HAS_OWN_HARNESS_TEST, text,
        anchors=[("Kali.test('object hasOwn primitive literals'",
                  "Kali.test('object hasOwn primitive literals'"),
                 ("console.log('browser object hasOwn ok');",
                  "console.log('browser object hasOwn ok');")])
    if run_body == test_body:
        raise AssertionError("the run and test has-own harness captures are identical")

    source = {"main.${ext}": run_body, "smoke.test.${ext}": test_body}

    c_fail = cite(text, "assert!(!output.status.success()")
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")
    c_format = [cite(text, "format!(", occurrence=2, expect=4),
                cite(text, "format!(", occurrence=4, expect=4)]

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_carried(stem, text, reaching=helper),
        "",
        matrix_block(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[(helper, 16,
                      "command(run/test) x ext(js/ts/jsx/tsx) x json_output(false/true), a "
                      "complete cross product. Every `#[test]` fn is one unlooped call and "
                      "the file contains no loop at all. Unlike this batch's other has-own "
                      "targets, BOTH fixture builders here take no parameter, so the same "
                      "text goes to all four extensions and `ext` really is uniform.")],
            non_axis_lines=FAIL_CLOSED_NON_AXES),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        "",
        "RULE 8 / RULE 9 -- THE FIXTURES ARE CAPTURED, NEVER HAND-DERIVED. Both builders are",
        f"one inline `format!` with `{{{{`/`}}}}` brace-collapse ({c_format[0]}, {c_format[1]})",
        "whose three arguments are computed by FOUR kali_common::object helpers, some of which",
        "join an alias table and some of which wrap another helper's output. That is rule 8's",
        "`format!` trap and rule 9's \"one level removed inside a library crate\" case at once,",
        "and a hand-derived approximation would still be a valid program. The two texts below are",
        "the byte-exact output of EXECUTING the real builders (see tools/task-18-browser-",
        "pilot/batch7a_captures.py for the capture procedure), and the generator re-checks",
        "each against anchors present in both the producing Rust source and the captured text.",
        "",
        NO_TEMPLATE_LITERAL,
        "",
        rule13_kali_common_block(
            ["kali_bin", "browser_harness_object_has_own_run_source",
             "browser_harness_object_has_own_test_source", helper],
            runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so its call chain never reaches",
        "`kali_runtime_contract`'s two harness helpers and ruling 6's exemption is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        "`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        FAIL_CLOSED_NOTE,
        f"The source's only assertion is at {c_fail}, for every one of its 16 fns, in both",
        "output modes.",
    )

    prose = rule12_rationale(text, stem) + rule13_kali_common_rationale()
    cases = []
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        for json_output in (False, True):
            fn = (("json_" if json_output else "") + command
                  + "_supports_object_has_own_when_browser_harness_is_configured_in_js_input")
            assert_fns(text, fn)
            name, glob = strip_ext_suffix(fn)
            cases.append({
                "name": name,
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the four `{glob}` fns (one "
                    f"per extension). `{helper}` writes the Object.hasOwn {command} fixture and "
                    f"asks `kali` to {command} it against the browser API surface with the "
                    "browser harness backed by node"
                    + (", with `--output json`" if json_output else "")
                    + ". The program checks membership through every bracketed, single-quoted, "
                      "parenthesized and frozen-callable Object.hasOwn and "
                      "Object.prototype.hasOwnProperty.call spelling `kali_common` knows about. "
                      "kali fails closed on it: the source's ONLY assertion is that the process "
                      f"fails {c_fail}, so this step carries `exit = \"failure\"` and nothing "
                      "else; adding a claim the source never made would be a rule-2 invention. "
                      "The fixture text is the byte-exact output of executing the real "
                      "`format!` and its four kali_common::object helpers (rules 8 and 9), "
                      "never hand-derived."
                    + (" This sibling issues the `--output json` argv; the source asserts "
                       "nothing about the envelope, so neither does this case."
                       if json_output else "")
                    + prose),
                "steps": [failing_harness_step(command, entry, json_output, env)],
            })
    return build(header, {"ext": EXTS4}, source, cases)


# ==========================================================================
# F9. browser_object_is_alias_chain_harness.rs
#     16 fns / 16 invocations, [matrix] ext, successful run with exact pins.
# ==========================================================================

@target("object_is_alias_chain_harness")
def gen_object_is_alias_chain_harness():
    stem = "object_is_alias_chain_harness"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_harness_object_is_alias_chain"

    anchors = [("const object = { a: 1 };", "const object = { a: 1 };"),
               ("unexpected browser Object.is alias chain result",
                "unexpected browser Object.is alias chain result")]
    run_body = check_captured("alias chain run", C.CAP_IS_ALIAS_CHAIN_RUN, text, anchors=anchors)
    test_body = check_captured("alias chain test", C.CAP_IS_ALIAS_CHAIN_TEST, text,
                               anchors=anchors, must_contain="Kali.test(")
    if run_body == test_body:
        raise AssertionError("the run and test alias-chain captures are identical")
    source = {"main.${ext}": run_body, "smoke.test.${ext}": test_body}

    c_code = cite(text, "assert_eq!(output.status.code(), Some(0))")
    c_run_stdout = cite(text, 'assert_eq!(json["stdout"], "browser object is alias chain ok')
    c_test_stdout = cite(text, 'assert_eq!(json["stdout"], "")')
    c_stderr = cite(text, 'assert_eq!(json["stderr"], "")')
    c_text_run = cite(text, 'stdout.contains("browser object is alias chain ok")')
    c_ok1 = cite(text, 'stdout.contains("ok 1")')
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")
    c_format = cite(text, '"test" => format!(')

    run_pin = _pin("run stdout", "browser object is alias chain ok\n",
                   [(f"main.{e}", run_body, "run") for e in EXTS4])
    test_pin = _pin("test stdout", "", [(f"smoke.test.{e}", test_body, "test") for e in EXTS4])

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_none(stem, text),
        "",
        matrix_block(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[(helper, 16,
                      "command(run/test) x ext(js/ts/jsx/tsx) x json_output(false/true), a "
                      "complete cross product. Every `#[test]` fn is one unlooped call and "
                      "the file contains no loop at all.")]),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        "",
        "RULE 8 -- THE FIXTURES ARE CAPTURED, NEVER HAND-DERIVED. One builder produces both",
        f"texts from a shared body with a `match command` and two `format!`s {c_format}: the",
        "`test` arm wraps the body in `Kali.test('browser object is alias chain', () => {...})`",
        "and every other arm appends a `console.log`. The two texts below are the byte-exact",
        "output of EXECUTING the real builder (see tools/task-18-browser-pilot/",
        "batch7a_captures.py), and the generator re-checks each against anchors present in both",
        "the producing Rust source and the captured text.",
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_object_is_alias_chain_source", helper],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so ruling 6's exemption has nothing",
        "to exempt and is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        "`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"  * `exit = 0`, not `exit = \"success\"`. The source asserts BOTH `status.success()`",
        f"    and `assert_eq!(output.status.code(), Some(0))` {c_code}; the exact-code form",
        "    of design spec 5.4's `exit` key carries both at once, and downgrading it to the",
        "    status class would drop the source's exact claim (rule 1).",
        "  * In json mode: schemaVersion/command/success and payload hostContract/",
        "    runtimeBackend; for `run`, `payload.exitCode = 0` and an EXACT `json.stdout`",
        f"    {c_run_stdout}; for `test`, payload total/passed/failed and an EXACT",
        f"    `json.stdout = \"\"` {c_test_stdout}; `json.stderr = \"\"` {c_stderr} and an",
        "    empty `errors` array in both.",
        "    NOTE the envelope-level `exitCode` is NOT asserted -- this source checks only",
        "    `payload.exitCode` for `run` -- so it is omitted rather than added, per rule 2.",
        "    Both `json.stdout` pins are EXACT IN THE SOURCE (`assert_eq!`), so they are exact",
        "    here by rule 1's non-negotiable direction, not by strengthening; U9 re-captures",
        "    both from the real binary for all four `ext` cells on every generator run, and",
        f"    they are {run_pin!r} and {test_pin!r}.",
        f"  * In text mode: `stdout_contains` -- the run cases `\"browser object is alias chain",
        f"    ok\"` {c_text_run}, the test cases `\"ok 1\"` {c_ok1}.",
        P.ruling3_substring(),
    )

    cases = []
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        for json_output in (False, True):
            fn = (("json_" if json_output else "") + command
                  + "_supports_object_is_alias_chain_when_browser_harness_is_configured"
                    "_in_js_input")
            assert_fns(text, fn)
            name, glob = strip_ext_suffix(fn)
            asserts = {"exit": 0}
            if not json_output:
                asserts["stdout_contains"] = (
                    ["browser object is alias chain ok"] if command == "run" else ["ok 1"])
            j = envelope_harness(command, stderr=True, errors=True)
            j.pop("exitCode", None)
            claims = {}
            for key, value in j.items():
                if key in ("stderr", "errors") and "stdout" not in claims:
                    claims["stdout"] = run_pin if command == "run" else test_pin
                claims[key] = value
            claims.setdefault("stdout", run_pin if command == "run" else test_pin)
            cases.append({
                "name": name,
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the four `{glob}` fns (one "
                    f"per extension). `{helper}` writes the Object.is alias-chain "
                    f"{command} fixture and asks `kali` to {command} it against the browser API "
                    "surface with the browser harness backed by node"
                    + (", with `--output json`" if json_output else "")
                    + ". The program aliases an object and an array, freezes each, and pins "
                      "that Object.is reports reference identity through the optional-chained, "
                      "bracketed, mixed and dotted `globalThis.Object.is` spellings while "
                      "reporting false for two structurally equal but distinct literals. It "
                      "succeeds, and the source asserts both `status.success()` and an exact "
                      f"exit code of 0 {c_code}, which is carried as `exit = 0`."
                    + ((" In json mode this case pins schemaVersion/command/success, the "
                        "payload's hostContract and runtimeBackend, "
                        + ("`payload.exitCode = 0`" if command == "run"
                           else "the payload's total, passed and failed")
                        + ", an empty `errors` array, `json.stderr = \"\"` "
                          f"{c_stderr} and an EXACT `json.stdout` "
                        + (f"{c_run_stdout}" if command == "run"
                           else f"{c_test_stdout}")
                        + ". That pin is exact IN THE SOURCE (`assert_eq!`), so carrying it "
                          "exactly is rule 1's non-negotiable direction rather than a "
                          "strengthening; U9 re-captured it from the real binary for all four "
                          "`ext` cells. The envelope-level `exitCode` is NOT asserted by this "
                          "source, so it is not asserted here (rule 2).")
                       if json_output else
                       (" Its text-mode claim is a plain "
                        + (f"{c_text_run}" if command == "run" else f"{c_ok1}")
                        + ". " + P.ruling3_substring()))
                    + " The fixture text is the byte-exact output of executing the real "
                      "`format!` (rule 8), never hand-derived."),
                "steps": [ok_harness_cli_step(command, entry, json_output, env, asserts=asserts,
                                              json_claims=claims)],
            })
    return build(header, {"ext": EXTS4}, source, cases)


# ==========================================================================
# F10. browser_object_is_bundle.rs
#      8 fns / 8 invocations, MATRIX DECLINED (ts body differs).
# ==========================================================================

@target("object_is_bundle")
def gen_object_is_bundle():
    stem = "object_is_bundle"
    text = rs(stem)
    helper = "assert_browser_bundle_object_is"

    js_body = check_program("object is js", fixture_in_fn(
        text, "browser_bundle_object_is_js_source"))
    ts_body = check_program("object is ts", fixture_in_fn(
        text, "browser_bundle_object_is_ts_source"))
    if js_body == ts_body:
        raise AssertionError("the js and ts object-is fixtures are identical")
    source = {"app.js": js_body, "app.jsx": js_body, "app.ts": ts_body, "app.tsx": ts_body}
    assert_bodies_identical("the js/jsx fixture", source, ["app.js", "app.jsx"])
    assert_bodies_identical("the ts/tsx fixture", source, ["app.ts", "app.tsx"])

    harness_body = check_program(
        "object is harness body",
        fixture_starting(text, helper, "const mod = await import(bundleJs.href);"),
        must_contain="await mod.browserObjectIs();")

    c_build_ok = cites(text, "output.status.success(),", 2)
    c_errors = cite(text, 'assert!(envelope["errors"]')
    c_meta = cite(text, 'assert_eq!(metadata["apiSurface"], "browser")')
    c_stdout = cite(text, 'stdout.contains("browser object is ok")')

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_none(stem, text),
        "",
        P.matrix_declined(
            test_fns=8, invocations=8, cases=8,
            reason=[
                "`ext` LOOKS like a clean four-value axis and the arithmetic would close",
                "(2 output modes x 4 extensions = 8 = the invocation count), but the fixture",
                "BODY is not uniform over it: each `#[test]` fn passes its source explicitly,",
                "and the `.js`/`.jsx` cells get `browser_bundle_object_is_js_source()` (a bare",
                "`const zero = 0;`) while the `.ts`/`.tsx` cells get",
                "`browser_bundle_object_is_ts_source()` (`const zero = (0 as const);`). A",
                "`[matrix]` axis substitutes one string uniformly into every case (design spec",
                "5.6); it cannot select a body.",
            ]),
        "",
        P.RULE6_ONE_TO_ONE,
        "",
        P.u2_source_file_wide(sorted(source)),
        "Every entry has the stem `app`, so `kali build --bundle` writes the same `app/` output",
        "directory whichever one a case names, and only one build runs per trial. No U5 rename",
        "is needed.",
        "",
        P.RULING7_NO_HOIST,
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_bundle_object_is_js_source",
                         "browser_bundle_object_is_ts_source", helper]),
        "",
        ARGV_ORDER_BUILD_ONLY,
        "",
        RUNNER_HARNESS_STEP,
        "",
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"  * `exit = \"success\"` on the build {c_build_ok[0]} and on the browser-bundle",
        f"    harness {c_build_ok[1]}.",
        "  * In json mode, the build envelope's schemaVersion/command/success/exitCode and the",
        f"    payload's artifactKind/bundleFormat, plus an EMPTY `errors` array {c_errors}.",
        f"  * `app/app.meta.json`'s apiSurface/artifactKind {c_meta}, asserted in BOTH",
        "    modes, because the source reads that file outside the `if json_output` block.",
        f"  * The harness's one stdout claim, {c_stdout}.",
        P.ruling3_substring(),
    )

    cases = []
    for json_output in (False, True):
        for ext in ("js", "ts", "jsx", "tsx"):
            fn = ("json_" if json_output else "") + f"build_emits_browser_object_is_in_{ext}_input"
            assert_fns(text, fn)
            variant = ("the `as const` TypeScript variant" if ext in ("ts", "tsx")
                       else "the plain JavaScript variant")
            cases.append({
                "name": fn,
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the source fn `{fn}` (one case per source fn, rule 6). "
                    f"`{helper}` writes {variant} of the browser Object.is fixture to app.{ext}, "
                    "builds it with `kali build --bundle --api browser`"
                    + (" with `--output json`" if json_output else "")
                    + ", asserts the emitted app/app.meta.json metadata, then runs the bundle "
                      "glue under the browser-bundle-harness contract backed by node. The "
                      "program pins Object.is over signed zeros, awaited aliases, primitives, "
                      "BigInts, NaN, infinities, frozen objects and every bracketed "
                      "`globalThis[\"Object\"][\"is\"]` spelling, then prints its ok line. Both "
                      f"processes succeed ({c_build_ok[0]}, {c_build_ok[1]}), and the "
                      f"harness's only output claim is {c_stdout}. " + P.ruling3_substring()
                    + (" This sibling additionally asserts the build JSON envelope -- "
                       "schemaVersion/command/success/exitCode, payload artifactKind/"
                       f"bundleFormat, and the empty `errors` array {c_errors} -- rather "
                       "than plain text; output shape is not a matrix axis because it changes "
                       "the assertion shape, so it is a separate case." if json_output else "")),
                "steps": bundle_success_steps(
                    f"app.{ext}", "app", harness_body,
                    {"exit": "success", "stdout_contains": ["browser object is ok"]},
                    json_output=json_output, envelope=envelope_build(errors=True)),
            })
    return build(header, None, source, cases)


# ==========================================================================
# F11. browser_object_is_harness.rs
#      16 fns / 16 invocations, [matrix] ext, thirteen stdout needles.
# ==========================================================================

@target("object_is_harness")
def gen_object_is_harness():
    stem = "object_is_harness"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_harness_object_is"

    run_body = check_program("object is run", fixture_in_fn(
        text, "browser_harness_object_is_run_source"))
    test_body = check_program("object is test", fixture_in_fn(
        text, "browser_harness_object_is_test_source"))
    source = {"main.${ext}": run_body, "smoke.test.${ext}": test_body}

    needles = contains_needles(text, "stdout")
    if "ok 1" not in needles:
        raise AssertionError("the `ok 1` needle vanished from this source")
    base_needles = [n for n in needles if n != "ok 1"]
    if len(base_needles) != 13:
        raise AssertionError(f"{len(base_needles)} base stdout needles, expected 13")

    c_success = cite(text, "output.status.success(),")
    c_exit = cite(text, 'assert_eq!(json["exitCode"], 0)')
    c_json_stdout = cite(text, 'let stdout = json["stdout"].as_str()')
    c_stderr = cite(text, 'assert_eq!(json["stderr"], "")')
    c_ok1 = cite(text, 'stdout.contains("ok 1")')
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")

    run_pin = _pin("run stdout", _RUN_PIN_IS_HARNESS,
                   [(f"main.{e}", run_body, "run") for e in EXTS4])
    test_pin = _pin("test stdout", _TEST_PIN_IS_HARNESS,
                    [(f"smoke.test.{e}", test_body, "test") for e in EXTS4])

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        [P.extra_ok(run_pin, P.EXTRA_OK_JSON_STDOUT),
         P.extra_ok(test_pin, P.EXTRA_OK_JSON_STDOUT)],
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_carried(stem, text, reaching=helper),
        "",
        matrix_block(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[(helper, 16,
                      "command(run/test) x ext(js/ts/jsx/tsx) x json_output(false/true), a "
                      "complete cross product. Every `#[test]` fn is one unlooped call and "
                      "the file contains no loop at all; both fixture builders are "
                      "parameterless, so `ext` really is uniform.")]),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_harness_object_is_run_source",
                         "browser_harness_object_is_test_source", helper],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so ruling 6's exemption has nothing",
        "to exempt and is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        "`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"  * `exit = \"success\"` {c_success}. This source checks `status.success()` only --",
        "    it does NOT check `status.code()` -- so the status class is what is carried, not",
        "    an exact code. (Its sibling object_is_alias_chain_harness.toml DOES carry an exact",
        "    code, because its source asserts one.)",
        f"  * TEXT MODE: THIRTEEN separate `stdout.contains(...)` needles, each its own source",
        "    line, extracted from the `.rs` by the generator rather than retyped -- they differ",
        "    by a single word and a retyped list that merged two would silently weaken the",
        "    claim. The `test` cases carry a fourteenth, `\"ok 1\"` "
        f"{c_ok1}.",
        P.ruling3_substring(),
        f"  * JSON MODE: the same thirteen claims are taken against the string leaf",
        f"    json[\"stdout\"] {c_json_stdout}, which has NO substring form.",
        P.ruling3_json_leaf(),
        "    The two pins differ by exactly the trailing `same(objectAlias, object)` line the",
        "    `test` fixture adds, and each was re-captured for all four `ext` cells.",
        "    The envelope also carries schemaVersion/command/success, payload hostContract/",
        f"    runtimeBackend, `exitCode = 0` at BOTH envelope and payload level for `run`",
        f"    {c_exit} or payload total/passed/failed for `test`, `json.stderr = \"\"`",
        f"    {c_stderr} and an empty `errors` array.",
    )

    prose = rule12_rationale(text, stem)
    cases = []
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        for json_output in (False, True):
            fn = (("json_" if json_output else "") + command
                  + "_supports_object_is_numeric_literals_when_browser_harness_is_configured"
                    "_in_js_input")
            assert_fns(text, fn)
            name, glob = strip_ext_suffix(fn)
            asserts = {"exit": "success"}
            if not json_output:
                asserts["stdout_contains"] = (
                    base_needles + ["ok 1"] if command == "test" else list(base_needles))
            j = envelope_harness(command, stderr=True, errors=True)
            claims = {}
            for key, value in j.items():
                if key in ("stderr", "errors") and "stdout" not in claims:
                    claims["stdout"] = run_pin if command == "run" else test_pin
                claims[key] = value
            claims.setdefault("stdout", run_pin if command == "run" else test_pin)
            cases.append({
                "name": name,
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the four `{glob}` fns (one "
                    f"per extension). `{helper}` writes the Object.is {command} fixture and asks "
                    f"`kali` to {command} it against the browser API surface with the browser "
                    "harness backed by node"
                    + (", with `--output json`" if json_output else "")
                    + ". The program prints Object.is results for signed zeros, primitives, "
                      "null, NaN, the infinities and `void 0`, then prints a labelled line per "
                      "same-reference spelling -- optional-chained, bracketed, mixed, dotted, "
                      "root-bracketed and their frozen counterparts. The process succeeds "
                      f"{c_success}; the source checks the status class only, not an exact "
                      "code, so this case carries `exit = \"success\"`."
                    + ((f" In json mode the source takes its thirteen claims against the string "
                        f"leaf json[\"stdout\"] {c_json_stdout}, and also pins "
                        + ("the envelope and payload `exitCode = 0` " + f"{c_exit}"
                           if command == "run"
                           else "the payload's total, passed and failed")
                        + f", `json.stderr = \"\"` {c_stderr} and an empty `errors` array. "
                        + P.ruling3_json_leaf())
                       if json_output else
                       (" In text mode all thirteen claims are plain `.contains(...)` against "
                        "raw stdout, each on its own source line, and they are carried "
                        "individually rather than merged"
                        + (f"; the `test` command adds a fourteenth, {c_ok1}"
                           if command == "test" else "")
                        + ". " + P.ruling3_substring()))
                    + prose),
                "steps": [ok_harness_cli_step(command, entry, json_output, env, asserts=asserts,
                                              json_claims=claims)],
            })
    return build(header, {"ext": EXTS4}, source, cases)


_RUN_PIN_IS_HARNESS = (
    "0\n1\n1\n1\n1\n1\n1\n1\n1\nsame-reference true\nsame-reference-await true\n"
    "same-reference-optional-chain true\nsame-reference-bracketed true\n"
    "same-reference-mixed true\nsame-reference-root true\nsame-reference-dot true\n"
    "same-reference-root-bracketed true\nsame-reference-freeze true\n"
    "same-reference-freeze-bracketed true\nsame-reference-freeze-mixed true\n"
    "same-reference-freeze-root true\nsame-reference-freeze-root-bracketed true\n"
    "1\n1\n1\n1\n1\n1\n")

_TEST_PIN_IS_HARNESS = _RUN_PIN_IS_HARNESS + "1\n"


# ==========================================================================
# F12. browser_object_is_primitive_literals_harness.rs
#      4 fns / 16 invocations, [matrix] ext.
# ==========================================================================

@target("object_is_primitive_literals_harness")
def gen_object_is_primitive_literals_harness():
    stem = "object_is_primitive_literals_harness"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_harness_object_is"

    run_body = check_program("primitive literals run", fixture_in_fn(
        text, "object_is_browser_harness_source"))
    test_body = check_program("primitive literals test", fixture_in_fn(
        text, "object_is_browser_harness_test_source"))
    source = {"main.${ext}": run_body, "smoke.test.${ext}": test_body}

    c_success = cite(text, "output.status.success(),")
    c_exit = cite(text, 'assert_eq!(json["exitCode"], 0)')
    c_json_stdout = cite(text, 'let stdout = json["stdout"].as_str()')
    c_stderr = cite(text, 'assert_eq!(json["stderr"], "")')
    c_text = cites(text, 'stdout.contains("browser object is primitive literals ok")', 2)
    c_ok1 = cite(text, 'stdout.contains("ok 1")')
    assert_count(text, "for filename in [", 4)   # asserted, not printed: ruling 15
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")

    pin = _pin("primitive literals stdout", _PRIM_PIN,
               [(f"main.{e}", run_body, "run") for e in EXTS4]
               + [(f"smoke.test.{e}", test_body, "test") for e in EXTS4])

    header = hdr(
        P.EXTRA_CLAIM_PREAMBLE,
        [P.extra_ok(pin, P.EXTRA_OK_JSON_STDOUT)],
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_none(stem, text),
        "",
        matrix_block(
            test_fns=4, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[(helper, 16,
                      "command(run/test) x json_output(false/true) x ext(js/ts/jsx/tsx). Unlike "
                      "the rest of this batch, EACH of the four `#[test]` fns is itself a LOOP "
                      "over all four filenames (`for filename in [...]`, a count the "
                      "generator asserts against the source before emitting), so 4 fns make "
                      "16 invocations. Both fixture builders are "
                        "parameterless, so every cell of a loop runs the same program text and "
                        "`ext` is uniform.")]),
        "",
        P.rule6_matrix_fold("exactly one source `#[test]` fn, whose own four-filename loop the "
                            "`ext` axis reproduces cell for cell"),
        "This is the cleanest possible matrix fold: the source ALREADY wrote the axis, as a",
        "`for filename in [...]` loop, and `[matrix] ext` is the same four iterations expressed",
        "as trials. No case corresponds to more than one fn.",
        "",
        P.u2_source_file_wide(["main.${ext}", "smoke.test.${ext}"]),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "object_is_browser_harness_source",
                         "object_is_browser_harness_test_source", helper],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so ruling 6's exemption has nothing",
        "to exempt and is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        "`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"  * `exit = \"success\"` {c_success} -- the status class, because this source",
        "    checks `status.success()` and no exact code.",
        f"  * TEXT MODE: `stdout_contains` on the one needle the source asserts",
        f"    {c_text[1]}, plus `\"ok 1\"` for the `test` command only {c_ok1}.",
        P.ruling3_substring(),
        f"  * JSON MODE: the same claim against the string leaf json[\"stdout\"]",
        f"    {c_json_stdout}, plus schemaVersion/command/success, payload hostContract/",
        f"    runtimeBackend, envelope and payload `exitCode = 0` for `run` {c_exit} or",
        f"    payload total/passed/failed for `test`, `json.stderr = \"\"` {c_stderr} and an",
        "    empty `errors` array.",
        P.ruling3_json_leaf(),
        "    The same pin serves `run` and `test`: the `test` fixture's Kali.test body prints",
        "    the identical lines, and the `ok 1` TAP line is written to the process's own",
        "    stdout rather than into the JSON leaf. That is not assumed -- the generator",
        "    re-captures all eight cells (4 extensions x run/test) and requires them equal.",
    )

    cases = []
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        for json_output in (False, True):
            fn = (("json_" if json_output else "") + command
                  + "_supports_object_is_primitive_literals_when_browser_harness_is_configured"
                    "_in_js_ts_jsx_tsx_input")
            assert_fns(text, fn)
            name, glob = strip_ext_suffix(fn)
            asserts = {"exit": "success"}
            if not json_output:
                needles = ["browser object is primitive literals ok"]
                if command == "test":
                    needles.append("ok 1")
                asserts["stdout_contains"] = needles
            j = envelope_harness(command, stderr=True, errors=True)
            claims = {}
            for key, value in j.items():
                if key in ("stderr", "errors") and "stdout" not in claims:
                    claims["stdout"] = pin
                claims[key] = value
            claims.setdefault("stdout", pin)
            cases.append({
                "name": name,
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the single source fn `{fn}`, whose own "
                    "`for filename in [...]` loop over the four extensions is exactly what the "
                    f"`ext` axis reproduces. `{helper}` writes the Object.is primitive-literals "
                    f"{command} fixture and asks `kali` to {command} it against the browser API "
                    "surface with the browser harness backed by node"
                    + (", with `--output json`" if json_output else "")
                    + ". The program prints Object.is results for a zero alias against -0, the "
                      "signed zeros, unary-plus integers, booleans, strings, frozen numbers, "
                      "BigInts, null, the infinities and NaN, and for every bracketed and "
                      "frozen-callable `globalThis[\"Object\"][\"is\"]` spelling, then prints "
                      f"its ok line. The process succeeds {c_success}."
                    + ((f" In json mode the source takes its claim against the string leaf "
                        f"json[\"stdout\"] {c_json_stdout} and also pins "
                        + ("the envelope and payload `exitCode = 0` " + f"{c_exit}"
                           if command == "run"
                           else "the payload's total, passed and failed")
                        + f", `json.stderr = \"\"` {c_stderr} and an empty `errors` array. "
                        + P.ruling3_json_leaf())
                       if json_output else
                       (f" Its text-mode claim is a plain substring match, {c_text[1]}"
                        + (f", plus {c_ok1} for the `test` command"
                           if command == "test" else "")
                        + ". " + P.ruling3_substring()))),
                "steps": [ok_harness_cli_step(command, entry, json_output, env, asserts=asserts,
                                              json_claims=claims)],
            })
    return build(header, {"ext": EXTS4}, source, cases)


_PRIM_PIN = ("0\n0\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n"
             "browser object is primitive literals ok\n")


# ==========================================================================
# F13. browser_object_is_void_undefined_bundle.rs
#      8 fns / 8 invocations, [matrix] ext.
# ==========================================================================

@target("object_is_void_undefined_bundle")
def gen_object_is_void_undefined_bundle():
    stem = "object_is_void_undefined_bundle"
    text = rs(stem)
    helper = "assert_browser_bundle_object_is_void_undefined"

    program = check_program("void undefined", fixture_in_fn(
        text, "browser_bundle_object_is_void_undefined_source"))
    source = {"app.${ext}": program}

    harness_body = check_program(
        "void undefined harness body",
        fixture_starting(text, helper, "const mod = await import(bundleJs.href);"),
        must_contain="await mod.objectIsVoidUndefined();")

    c_build_ok = cites(text, "output.status.success(),", 2)
    c_meta = cite(text, 'assert_eq!(metadata["apiSurface"], "browser")')
    c_stdout = cite(text, r'stdout.contains("1\n1\n1\n")')
    c_payload = cite(text, 'assert_eq!(payload["bundleFormat"], "esm")')

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_none(stem, text),
        "",
        matrix_block(
            test_fns=8, invocations=8, cases=2, axis="ext", values=EXTS4,
            helpers=[(helper, 8,
                      "ext(js/ts/jsx/tsx) x json_output(false/true), a complete cross product. "
                      "Every `#[test]` fn is one unlooped call and the file contains no loop "
                      "at all; the single fixture builder is parameterless, so `ext` is "
                      "uniform.")],
            non_axes=("json_output",)),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(["app.${ext}"]),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_bundle_object_is_void_undefined_source", helper]),
        "",
        ARGV_ORDER_BUILD_ONLY,
        "",
        RUNNER_HARNESS_STEP,
        "",
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"  * `exit = \"success\"` on the build {c_build_ok[0]} and on the browser-bundle",
        f"    harness {c_build_ok[1]}.",
        "  * In json mode, the envelope's schemaVersion/command/success/exitCode and the",
        f"    payload's artifactKind/bundleFormat {c_payload}.",
        "    THERE IS NO `errors = []` CLAIM HERE. Most bundle targets in this family assert",
        "    the envelope's `errors` array is empty; this source does not, and rule 2 forbids",
        "    adding an assertion the source never made, so `errors` is absent from the json",
        "    claims below. That omission is deliberate and is the reason it is called out.",
        f"  * `app/app.meta.json`'s apiSurface/artifactKind {c_meta}, asserted in BOTH modes,",
        "    because the source reads that file outside the `if json_output` block.",
        f"  * The harness's one stdout claim, {c_stdout}.",
        P.ruling3_substring(),
    )

    cases = []
    for json_output in (False, True):
        fn = ("json_" if json_output else "") + "build_emits_object_is_void_undefined_in_js_input"
        assert_fns(text, fn)
        name, glob = strip_ext_suffix(fn)
        cases.append({
            "name": name,
            "rationale": (
                f"Migrated from browser_{stem}.rs, the four `{glob}` fns (one per "
                f"extension). `{helper}` writes the void-0 Object.is fixture, builds it with "
                "`kali build --bundle --api browser`"
                + (" with `--output json`" if json_output else "")
                + ", asserts the emitted app/app.meta.json metadata, then runs the bundle glue "
                  "under the browser-bundle-harness contract backed by node. The program aliases "
                  "`void 0` and prints Object.is(alias, void 0) through the dotted and bracketed "
                  "`globalThis` roots -- three `true` lines -- then returns them as an array. "
                  f"Both processes succeed ({c_build_ok[0]}, {c_build_ok[1]}), and the "
                  f"harness's only output claim is {c_stdout}. "
                + P.ruling3_substring()
                + (" This sibling additionally asserts the build JSON envelope -- "
                   "schemaVersion/command/success/exitCode and payload artifactKind/"
                   f"bundleFormat {c_payload} -- rather than plain text; output shape is not "
                   "a matrix axis because it changes the assertion shape, so it is a separate "
                   "case. Note this source makes NO `errors = []` claim, unlike most bundle "
                   "targets in this family, so none is asserted here (rule 2)."
                   if json_output else "")),
            "steps": bundle_success_steps(
                "app.${ext}", "app", harness_body,
                {"exit": "success", "stdout_contains": ["1\n1\n1\n"]},
                json_output=json_output, envelope=envelope_build(errors=False)),
        })
    return build(header, {"ext": EXTS4}, source, cases)


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
